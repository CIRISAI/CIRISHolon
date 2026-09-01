//! T3 SCALE-UP: the gates for dynamic storage, periodic boundaries and cutoff-local
//! enumeration.
//!
//! FSD-W1 §10 lists T3 as owed: "dynamic storage, cell lists, PBC, the ledgered-hand
//! column". This file is where each of those stops being a claim. The discipline is the
//! campaign's: every gate here is DEMONSTRATED FIRING as well as passing, because a gate
//! that has never failed has never gated (WB-8.2), and a check that passes in every
//! configuration has checked nothing (M-VACUOUS-SUCCESS).
//!
//! The plants, by name:
//!
//! | plant | what it breaks | what must fire |
//! |---|---|---|
//! | P-2 | the periodic box's translation invariance | `pbc_translation_residual` |
//! | P-2b | the minimum image itself (walls instead) | the same gate, the other way |
//! | P-T3-a | the many-body enumeration's locality | complete-vs-local energy identity |
//! | P-T3-b | a receipt column that stops being posted | `work_columns_ok` |

use holon_render::cells::Route;
use holon_render::sim::{Boundary, Dims, Sim, DEFAULT_SCENE_ATOMS};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

fn loaded_sim() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

/// A deterministic lattice of `nx*ny*nz` atoms inside a periodic box, spaced so no pair
/// opens inside the repulsive wall. No RNG anywhere: a reported run re-runs byte for byte
/// (WB-5.4).
fn lattice(nx: usize, ny: usize, nz: usize, spacing: f64) -> Box<Sim> {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Periodic;
    s.width = nx as f64 * spacing;
    s.height = ny as f64 * spacing;
    s.depth = nz as f64 * spacing;
    let n = nx * ny * nz;
    s.resize_storage(n);
    for i in 0..n {
        let ix = i % nx;
        let iy = (i / nx) % ny;
        let iz = i / (nx * ny);
        s.atoms[i].x = (ix as f64 + 0.5) * spacing;
        s.atoms[i].y = (iy as f64 + 0.5) * spacing;
        s.atoms[i].z = (iz as f64 + 0.5) * spacing;
        s.atoms[i].vx = 0.0;
        s.atoms[i].vy = 0.0;
        s.atoms[i].vz = 0.0;
    }
    s.sync_species();
    s.rebase();
    s
}

// ---------------------------------------------------------------- 1. dynamic storage

/// THE CAP IS GONE. `MAX_ATOMS = 16` used to clamp `reset`, so a scene asking for more
/// silently got sixteen. Asking for more now gets more, and every per-atom buffer agrees
/// about how many there are.
#[test]
fn the_scene_is_no_longer_capped_at_sixteen() {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    for n in [0usize, 1, 2, 16, 17, 64, 500] {
        s.reset(n);
        assert_eq!(s.n, n, "reset({n}) produced {} atoms", s.n);
        assert_eq!(s.atoms.len(), n);
        assert!(
            s.storage_ok(),
            "the per-atom buffers disagree about the scene size at n = {n}"
        );
    }
    // And the old constant is now honestly named: it is the viewer's opening scene, not a
    // capacity, and nothing clamps to it.
    assert_eq!(DEFAULT_SCENE_ATOMS, 16);
    s.reset(DEFAULT_SCENE_ATOMS * 4);
    assert_eq!(s.n, 64);
}

/// A scene far past the old cap still integrates, still closes its ledger, and still
/// obeys its own bound. The point is not the number 500; it is that nothing in the
/// dynamics was reading a fixed array bound.
#[test]
fn a_scene_past_the_old_cap_still_holds_its_gates() {
    let mut s = lattice(8, 8, 8, 5.0);
    assert_eq!(s.n, 512);
    for _ in 0..20 {
        s.step_frame(16);
    }
    assert!(s.storage_ok());
    assert_eq!(s.w_ext, 0.0, "a free periodic run injected external work");
    assert!(
        s.energy_gate(),
        "512 atoms: drift {:.4e} exceeds bound {:.4e}",
        s.drift_peak,
        s.drift_bound()
    );
    assert!(s.work_columns_ok(), "the receipt columns stopped summing");
}

// ------------------------------------------------------------- 2. periodic boundaries

/// PLANT P-2. Translating every atom by one box vector leaves the energy BIT-IDENTICAL
/// under the minimum-image convention: a periodic box has no origin, so where the scene
/// sits inside it is not a physical fact and must not reach a number.
///
/// The float precondition is checked by the gate itself (`NAN` when the shift was not
/// exact), and the box edges here are powers of two so that it is.
#[test]
fn p2_a_periodic_scene_does_not_know_where_it_is() {
    let mut s = lattice(4, 4, 4, 8.0);
    assert_eq!(s.width, 32.0);
    let residual = s.pbc_translation_residual();
    assert!(
        residual.is_finite(),
        "the harness's own translation was not exact, so the gate refused to compare"
    );
    assert_eq!(
        residual, 0.0,
        "translating by one box vector moved the energy by {residual:.3e}"
    );

    // Not vacuous: the same gate on a scene that HAS an origin reads large. Walls are the
    // physical case where the box's position is a real fact, and the plant is the
    // demonstration that the gate can tell the two apart.
    s.boundary = Boundary::Walls;
    let walled = s.pbc_translation_residual();
    assert!(
        walled > 1e-6,
        "P-2 passed with walls on ({walled:.3e}), so it is measuring nothing"
    );
    s.boundary = Boundary::Periodic;
    assert_eq!(s.pbc_translation_residual(), 0.0, "the scene did not restore");
}

/// PLANT P-2b: the minimum image is what makes P-2 pass, not the wrap. Two atoms placed
/// either side of a face are NEAR each other, and a naive difference says they are a box
/// apart — so the two must disagree, and the periodic answer must be the short one.
#[test]
fn p2b_the_minimum_image_is_doing_the_work() {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.width = 32.0;
    s.height = 32.0;
    s.depth = 32.0;
    s.resize_storage(2);
    s.atoms[0].x = 0.5;
    s.atoms[0].y = 16.0;
    s.atoms[0].z = 16.0;
    s.atoms[1].x = 31.5;
    s.atoms[1].y = 16.0;
    s.atoms[1].z = 16.0;
    s.sync_species();

    s.boundary = Boundary::Open;
    s.rebase();
    let e_open = s.energy();

    s.boundary = Boundary::Periodic;
    s.rebase();
    let e_pbc = s.energy();

    // Under the minimum image they are 1 bohr apart, well inside the bond; without it
    // they are 31 bohr apart and effectively free, which on this curve is a millionth of
    // the well. The two readings must be nowhere near each other.
    assert!(
        (e_pbc - e_open).abs() > 1e-3,
        "the periodic reading ({e_pbc:.6e}) did not differ from the naive one \
         ({e_open:.6e}); the images are not being taken"
    );
    assert!(
        e_pbc < -1e-2,
        "atoms 1 bohr apart should be deep in the well, not at {e_pbc:.6e}"
    );
    assert!(
        e_open.abs() < 1e-9,
        "atoms 31 bohr apart should be effectively free, not at {e_open:.6e}"
    );
}

/// The wrap does no work and delivers no impulse. An atom crossing a face is a change of
/// representation, not an event — so both ledgers must be silent about it.
#[test]
fn crossing_a_face_is_not_an_event() {
    let mut s = lattice(4, 4, 4, 8.0);
    // Push everything along +x hard enough that some atoms cross a face during the run.
    for i in 0..s.n {
        let v = s.atoms[i].vx;
        s.set_velocity_3d(i, v + 0.01, s.atoms[i].vy, s.atoms[i].vz);
    }
    s.rebase();
    let x_before: Vec<f64> = (0..s.n).map(|i| s.atoms[i].x).collect();
    for _ in 0..40 {
        s.step_frame(16);
    }
    let crossed = (0..s.n).filter(|&i| s.atoms[i].x < x_before[i]).count();
    assert!(
        crossed > 0,
        "no atom crossed a face, so the wrap was never exercised"
    );
    for i in 0..s.n {
        assert!(
            s.atoms[i].x >= 0.0 && s.atoms[i].x < s.width,
            "atom {i} is outside the box at x = {}",
            s.atoms[i].x
        );
    }
    assert_eq!(s.w_ext, 0.0, "the wrap posted work");
    assert!(
        s.energy_gate(),
        "drift {:.4e} exceeds bound {:.4e} across {crossed} face crossings",
        s.drift_peak,
        s.drift_bound()
    );
}

/// The minimum image is only the minimum image while the cutoff fits in half the box, and
/// the engine says so rather than letting the reduction lie.
#[test]
fn a_box_too_small_for_its_cutoff_is_refused() {
    let mut s = lattice(4, 4, 4, 10.0);
    // No three-body table and no declared pair cutoff: nothing to fit, vacuously fine.
    assert!(s.pbc_ok());
    // Declare a pair cutoff the box CAN hold. At 1e-6 Ha the derived window ends at
    // 15.81 bohr (examples/t3_cutoff_ladder.rs), which fits inside this box's half-edge
    // of 20 — and a tighter budget would NOT, which is the next assertion's point.
    assert!(
        s.set_pair_cutoff(1e-6),
        "a 40-bohr box could not hold a cutoff derived at 1e-6 Ha"
    );
    let (cut, half) = s.pbc_margin();
    assert!(cut <= half, "cutoff {cut} exceeds the half-edge {half}");
    assert!(s.pbc_ok());

    // Now shrink the box under it. The gate must notice.
    s.width = 2.0 * cut - 1.0;
    assert!(
        !s.pbc_ok(),
        "a box of {} cannot hold a cutoff of {cut}, and the gate said it could",
        s.width
    );

    // And the refusal is a refusal, not a warning: a budget this box cannot honour is
    // rejected outright rather than quietly shrunk to fit, which would replace a declared
    // truncation with an undeclared one.
    let mut small = lattice(4, 4, 4, 8.0);
    assert!(
        !small.set_pair_cutoff(1e-12),
        "a 32-bohr box accepted a cutoff of 29.2 bohr, which is past its own half-edge"
    );
    assert!(
        small.pair_switch().is_none(),
        "the refused cutoff was left installed anyway"
    );
}

// --------------------------------------------------- 3. locality is not approximation

/// PLANT P-T3-a. The cutoff-local many-body enumeration must produce the SAME NUMBER as
/// the complete one — not nearly, exactly. It can, because the three- and four-body
/// tables are exact zeros outside their domains, and the enumeration is sorted back into
/// the complete loop's own order so the floating-point sum is unchanged.
///
/// The complete route and the cell route are compared on one configuration, driven
/// through the public knob that chooses between them.
#[test]
fn the_two_routes_agree_bit_for_bit() {
    // Big enough that the cell decomposition actually engages: 64+ atoms and at least
    // three cells on the shortest axis.
    let mut local = lattice(6, 6, 6, 8.0);
    assert_eq!(local.n, 216);
    assert!(
        local.set_pair_cutoff(1e-6),
        "no pair cutoff could be derived at 1e-6 Ha"
    );
    assert_eq!(
        local.route(),
        Route::Cells,
        "the cell decomposition did not engage, so this test compares one route with itself"
    );
    let e_local = local.energy();
    let f_local: Vec<(f64, f64, f64)> = (0..local.n).map(|i| local.internal_force(i)).collect();

    // The same scene, same truncation, forced onto the complete enumeration by making the
    // box too coarse to decompose. The physics is identical; only the traversal differs.
    let mut complete = lattice(6, 6, 6, 8.0);
    assert!(complete.set_pair_cutoff(1e-6));
    complete.force_complete_route();
    assert_eq!(complete.route(), Route::Complete);
    let e_complete = complete.energy();
    let f_complete: Vec<(f64, f64, f64)> =
        (0..complete.n).map(|i| complete.internal_force(i)).collect();

    assert_eq!(
        e_local.to_bits(),
        e_complete.to_bits(),
        "the cell route read {e_local:.17e} where the complete route read {e_complete:.17e}"
    );
    for i in 0..local.n {
        assert_eq!(
            f_local[i].0.to_bits(),
            f_complete[i].0.to_bits(),
            "atom {i}: fx {:.17e} vs {:.17e}",
            f_local[i].0,
            f_complete[i].0
        );
        assert_eq!(f_local[i].1.to_bits(), f_complete[i].1.to_bits());
        assert_eq!(f_local[i].2.to_bits(), f_complete[i].2.to_bits());
    }
}

/// The truncation is DECLARED, not default. A scene that has not asked for one gets the
/// complete pair sum, and the reported floor says so.
#[test]
fn the_pair_truncation_is_opt_in_and_reports_what_it_drops() {
    let mut s = lattice(4, 4, 4, 10.0);
    assert_eq!(s.truncation_floor(), 0.0);
    assert!(s.pair_switch().is_none());
    let e_complete = s.energy();

    assert!(s.set_pair_cutoff(1e-6));
    assert_eq!(s.truncation_floor(), 1e-6);
    let (r_in, r_cut) = s.pair_switch().expect("a cutoff was declared");
    assert!(r_cut > r_in, "the switch window is inverted");
    let e_trunc = s.energy();

    // The energy moved, by less than the declared budget times the pairs it could touch.
    let moved = (e_trunc - e_complete).abs();
    let pairs = holon_render::sim::complete_pairs(s.n) as f64;
    assert!(
        moved <= 1e-6 * pairs,
        "the truncation dropped {moved:.3e}, past its own budget of {:.3e}",
        1e-6 * pairs
    );
    assert!(
        moved > 0.0,
        "the truncation dropped nothing at all, so it is not a truncation"
    );

    // And it is reversible.
    s.clear_pair_cutoff();
    assert_eq!(s.truncation_floor(), 0.0);
    assert_eq!(
        s.energy().to_bits(),
        e_complete.to_bits(),
        "clearing the cutoff did not restore the complete sum"
    );
}

/// A truncated pair potential is STILL A POTENTIAL — that is what the C² switch buys, and
/// it is why the energy gate stays an exact statement under truncation rather than an
/// approximate one. A hard cutoff would leave a step at the crossing and the drift would
/// find it.
#[test]
fn a_truncated_run_still_closes_its_ledger() {
    let mut s = lattice(6, 6, 6, 8.0);
    assert!(s.set_pair_cutoff(1e-6));
    // Give it enough kinetic energy that pairs actually cross the switch window.
    for i in 0..s.n {
        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        s.set_velocity_3d(i, sign * 0.004, sign * 0.002, sign * 0.001);
    }
    s.rebase();
    for _ in 0..40 {
        s.step_frame(16);
    }
    assert_eq!(s.w_ext, 0.0);
    assert!(
        s.energy_gate(),
        "a switched truncation leaked: drift {:.4e} against bound {:.4e}",
        s.drift_peak,
        s.drift_bound()
    );
}

// ------------------------------------------------------------- 4. the receipt columns

/// WB-4.3: the hand's work lands in its own column, and the columns sum to the total.
#[test]
fn the_hand_gets_its_own_receipt_column() {
    let mut s = lattice(3, 3, 3, 8.0);
    assert_eq!(s.work.hand, 0.0);
    assert_eq!(s.work.thermostat, 0.0);

    s.grab(0);
    assert_eq!(s.work.hand, 0.0, "the grab itself injected work");
    let (ax, ay, az) = (s.atoms[0].x + 1.0, s.atoms[0].y, s.atoms[0].z);
    s.move_anchor_3d(ax, ay, az);
    for _ in 0..10 {
        s.step_frame(16);
    }
    s.release();

    assert!(s.work.hand != 0.0, "a drag that did no work is not a drag");
    assert_eq!(s.work.thermostat, 0.0, "the thermostat was never on");
    assert!(
        s.work_columns_ok(),
        "columns {:?} sum to {:.17e} but w_ext is {:.17e}",
        s.work,
        s.work.total(),
        s.w_ext
    );

    // The thermostat gets its own, and the two do not contaminate each other.
    let hand_after_drag = s.work.hand;
    s.thermostat_on = true;
    s.target_temperature = 600.0;
    for _ in 0..20 {
        s.step_frame(16);
    }
    assert!(s.work.thermostat != 0.0, "the thermostat moved no energy");
    assert_eq!(
        s.work.hand, hand_after_drag,
        "the thermostat's work was posted to the hand's column"
    );
    assert!(s.work_columns_ok());
}

/// PLANT P-T3-b: a column that stops being posted. The total still closes — that is the
/// whole point — so `energy_gate` stays green and only the attribution gate can see it.
#[test]
fn p_t3_b_a_column_that_stops_being_posted_is_caught() {
    let mut s = lattice(3, 3, 3, 8.0);
    s.grab(0);
    let (ax, ay, az) = (s.atoms[0].x + 1.0, s.atoms[0].y, s.atoms[0].z);
    s.move_anchor_3d(ax, ay, az);
    for _ in 0..10 {
        s.step_frame(16);
    }
    assert!(s.work_columns_ok(), "the control must pass");
    let drift_gate_before = s.energy_gate();

    // The plant: the hand's receipt is lost while the total keeps it. This is exactly what
    // a forgotten `self.work.hand += ...` beside a live `self.w_ext += ...` looks like.
    s.work.hand = 0.0;

    assert!(
        !s.work_columns_ok(),
        "a lost receipt column did not trip the attribution gate; residual {:.3e} against \
         bound {:.3e}",
        (s.w_ext - s.work.total()).abs(),
        s.work_columns_bound()
    );
    assert_eq!(
        s.energy_gate(),
        drift_gate_before,
        "the energy gate noticed a defect it cannot see — which would mean these two gates \
         are not independent after all"
    );
}

// ---------------------------------------------------------------------- 5. the fence

/// The fence count is a statement about COMPOSITION, and moving the enumeration off
/// `O(N³)` did not move it. Held against the enumerated count on a scene carrying both
/// served and fenced triples.
#[test]
fn the_fence_count_survives_going_local() {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.resize_storage(6);
    // Six hydrogens, no three-body table loaded at all: the pre-T3 loop returned before
    // counting, and so does the census. The identity is that both say the same thing.
    for i in 0..6 {
        s.atoms[i].x = 4.0 * i as f64;
        s.atoms[i].y = 12.0;
        s.atoms[i].z = 12.0;
    }
    s.sync_species();
    s.rebase();
    assert_eq!(
        s.fenced_triples(),
        s.fence_untabulated,
        "the census and the force loop disagree about the fence"
    );
    // A scene of three atoms has one triple; of six, twenty. The count is combinatorial
    // and must track the census rather than the geometry.
    let before = s.fenced_triples();
    for i in 0..6 {
        s.atoms[i].x = 4.0 * i as f64 + 1000.0;
    }
    s.recompute();
    assert_eq!(
        s.fenced_triples(),
        before,
        "moving the atoms a thousand bohr apart changed a composition count"
    );
}

// ------------------------------------------- 6. the truncation is still a conservative force

/// THE QUESTION THE dE4 LANE ASKED, TURNED ON MY OWN SECTOR: is the gradient the derivative
/// of the value this call returns?
///
/// `the_two_routes_agree_bit_for_bit` above proves the cell route and the complete route
/// compute the SAME number. It does not prove either is the right one — two routes through
/// one wrong expression agree perfectly, which is the M-VACUOUS-SUCCESS shape with an extra
/// step. Nothing gated the switched pair term itself until this test: the C² switch
/// multiplies the VALUE by `S`, and the slope it hands the force loop is
/// `S·U' + S'·U` — a product rule that is easy to write with a term missing and impossible
/// to see afterwards, because the force would still be smooth, still be equal and opposite,
/// and still conserve momentum. Only ENERGY would be wrong, exactly as it was in
/// `QuaternaryTable::eval`'s clamped-coordinate-unclamped-slope defect: a constant value
/// carrying a non-zero force.
///
/// So: central-difference the truncated energy and require it to be minus the force the
/// integrator is actually pushed with.
#[test]
fn the_truncated_pair_force_is_minus_the_gradient_of_the_truncated_energy() {
    let mut s = loaded_sim();
    s.dims = Dims::Three;
    s.boundary = Boundary::Open;
    s.resize_storage(6);
    // Separations chosen to STRADDLE the switch window, with a deterministic zigzag so no
    // component of any gradient is accidentally zero.
    let xs = [0.0f64, 2.2, 9.0, 23.5, 38.0, 53.0];
    for i in 0..6 {
        s.atoms[i].x = xs[i];
        s.atoms[i].y = 20.0 + if i % 2 == 0 { 0.35 } else { -0.35 };
        s.atoms[i].z = 20.0 + if i % 3 == 0 { 0.25 } else { -0.15 };
    }
    s.sync_species();
    s.rebase();
    assert!(
        !s.trimer.loaded && !s.water.loaded,
        "a three-body table is loaded, so `internal_force` is not the pair term alone"
    );
    assert!(s.set_pair_cutoff(1e-6));
    let (r_in, r_cut) = s.pair_switch().expect("a cutoff was declared");

    // NON-VACUITY: the switch has to be doing something. A scene whose every pair sits
    // inside `r_in` tests the untruncated curve and says nothing about the product rule.
    let in_window = s
        .neighbours()
        .pairs
        .iter()
        .filter(|p| p.r > r_in && p.r < r_cut)
        .count();
    assert!(
        in_window >= 2,
        "only {in_window} pairs are inside the switch window ({r_in:.3}, {r_cut:.3}); this \
         scene does not exercise the truncation and the test would pass on the bare curve"
    );

    // Central difference of the PAIR energy against the force the loop hands the
    // integrator. `h` is small against the switch window (2 bohr) and large against the
    // f64 noise floor on an energy of order 1e-3 Ha.
    const H: f64 = 1e-5;
    let mut worst_rel: f64 = 0.0;
    let mut scale: f64 = 0.0;
    for i in 0..s.n {
        let f = s.internal_force(i);
        let analytic = [f.0, f.1, f.2];
        let p0 = (s.atoms[i].x, s.atoms[i].y, s.atoms[i].z);
        for axis in 0..3 {
            let mut shift = |d: f64| -> f64 {
                let mut p = [p0.0, p0.1, p0.2];
                p[axis] += d;
                s.set_position_3d(i, p[0], p[1], p[2]);
                s.recompute();
                s.e_pair
            };
            let e_plus = shift(H);
            let e_minus = shift(-H);
            shift(0.0);
            let fd = -(e_plus - e_minus) / (2.0 * H);
            scale = scale.max(analytic[axis].abs());
            worst_rel = worst_rel.max((analytic[axis] - fd).abs());
        }
    }
    let rel = worst_rel / scale.max(1e-30);
    println!(
        "truncated pair force vs -dE/dx: worst absolute {worst_rel:.3e} Ha/bohr against a \
         force scale of {scale:.3e} ({:.2e} relative); {in_window} pairs in the switch window",
        rel
    );
    assert!(
        scale > 1e-8,
        "the scene carries no force worth differentiating ({scale:.3e}); the comparison \
         would be zero against zero"
    );
    assert!(
        rel < 1e-6,
        "the truncated force is not minus the gradient of the truncated energy: worst \
         {worst_rel:.3e} Ha/bohr, {rel:.2e} relative. The C² switch's product rule is the \
         first place to look."
    );
}
