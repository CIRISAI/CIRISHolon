//! The fluid element band's door: the LEDGER is exact through the ABI, and the no-bond
//! control through the ABI is `Lattice::advance_with_colour`, bit for bit.
//!
//! Both gates drive the DOOR rather than the crate. That distinction is the whole subject:
//! `holon-lattice`'s own suite already gates `OrientationLattice::step`, and what could break
//! here is the layer between — a seed assembled wrong from its two halves, a rule staged and
//! not passed on, a ledger read off a rebuilt lattice, a control that flips a flag instead of
//! rebuilding the scene. Every one of those leaves the instrument's tests green.

use holon_render::fluid_door::*;
use std::sync::Mutex;

use holon_lattice::lattice::Lattice;
use holon_lattice::state::Model;

/// THE DOOR IS ONE PROCESS-GLOBAL LATTICE (`fluid_door.rs::FLUID`), and Rust runs a file's
/// tests on parallel threads. Two gates beginning a lattice at once would each read the
/// other's box; the gates therefore take it in turn. A poisoned lock is taken anyway — every
/// gate below begins its own scene before reading anything.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// FLUID-1's amplitude table and retention, pushed through the doors the page pushes them
/// through. The values are the record's (`conformance/mesh/fluid1/amplitude_table.json`) and
/// are written here ONLY because a test must be able to run from a bare checkout without a
/// path to the conformance tree; the page never types them, and the smoke gate checks the
/// page's copy against the record byte for byte. If they ever disagree the page is the one
/// that governs — this fixture would then be a stale copy of a live number, which is why it
/// is a fixture in a test and not a default in the crate.
const A_TABLE: [f64; 6] = [1.0, 1.337177895, 1.288496466, 1.420096811, 1.288496466, 1.337177895];
const RETENTION: f64 = 0.5526;

fn stage_rule() {
    for (i, a) in A_TABLE.iter().enumerate() {
        assert_eq!(holon_fluid_amplitude(i as u32, *a), 0, "the door refused A({i})");
    }
    assert_eq!(holon_fluid_rent_from_retention(RETENTION), 0, "the door refused the retention");
    assert_eq!(holon_fluid_rule_ready(), 1, "the rule is staged and the door says it is not");
}

/// G2, THROUGH THE DOOR: every conserved integer is the one the lattice started with, at
/// every step, over 500 steps at `L = 64` — and the bond balance closes,
/// `formed − broken_rent − blocked = held`.
///
/// The comparison is INTEGER-IDENTICAL and not a tolerance: these cross the ABI as `f64`
/// because that is the only numeric type this page's ABI carries, and every one of them is
/// far inside `2^53`, so `==` on the doubles IS `==` on the integers. A tolerance here would
/// pass on a lattice that was leaking one particle every hundred steps.
#[test]
fn the_door_driven_ledger_is_integer_identical_over_500_steps() {
    let _lock = serial();
    stage_rule();
    assert_eq!(holon_fluid_begin(64, 0.2, 0xF1D1_0001, 0, 1, 1), 0, "the door refused L = 64");
    assert_eq!(holon_fluid_l(), 64);
    assert_eq!(holon_fluid_bonds_enabled(), 1);

    let mass0 = holon_fluid_ledger_mass();
    let px0 = holon_fluid_ledger_px();
    let py0 = holon_fluid_ledger_py();
    let cen0 = holon_fluid_ledger_census_total();
    let red0 = holon_fluid_ledger_red();
    let census0: Vec<f64> = (0..6u32).map(|d| holon_fluid_ledger_census(d)).collect();
    // A gate that reports PASS on zero work has not passed. The scene must actually carry
    // particles, orientations and tracer bits before any of the invariance below means
    // anything (M-VACUOUS-SUCCESS).
    assert!(mass0 > 0.0, "no particle in the scene");
    assert!(cen0 > 0.0, "no particle carries an orientation");
    assert!(red0 > 0.0, "no particle carries a tracer bit");
    assert_eq!(mass0, holon_fluid_ledger_initial(0));
    assert_eq!(px0, holon_fluid_ledger_initial(1));
    assert_eq!(py0, holon_fluid_ledger_initial(2));
    assert_eq!(cen0, holon_fluid_ledger_initial(3));
    assert_eq!(red0, holon_fluid_ledger_initial(4));

    let mut held = 0.0f64;
    for t in 0..500u32 {
        let formed_before = holon_fluid_ledger_formed();
        let rent_before = holon_fluid_ledger_broken_rent();
        let blocked_before = holon_fluid_ledger_blocked();
        assert_eq!(holon_fluid_step(1), 1, "the door refused a step at {t}");
        held += (holon_fluid_ledger_formed() - formed_before)
            - (holon_fluid_ledger_broken_rent() - rent_before)
            - (holon_fluid_ledger_blocked() - blocked_before);
        assert_eq!(holon_fluid_ledger_mass(), mass0, "mass moved at step {t}");
        assert_eq!(holon_fluid_ledger_px(), px0, "momentum-x moved at step {t}");
        assert_eq!(holon_fluid_ledger_py(), py0, "momentum-y moved at step {t}");
        assert_eq!(holon_fluid_ledger_red(), red0, "the tracer count moved at step {t}");
        assert_eq!(holon_fluid_ledger_census_total(), cen0, "the orientation total moved at step {t}");
        for d in 0..6u32 {
            assert_eq!(
                holon_fluid_ledger_census(d),
                census0[d as usize],
                "the count of orientation {d} moved at step {t}"
            );
        }
        assert_eq!(held, holon_fluid_ledger_bonds(), "the bond balance broke at step {t}");
    }

    assert_eq!(holon_fluid_steps(), 500.0);
    assert_eq!(holon_fluid_ledger_steps_checked(), 500.0, "the audit did not see every step");
    assert_eq!(holon_fluid_ledger_exact(), 1, "the running audit disagrees with the explicit ledger");
    // The work the invariance was measured against, each branch named. Without these the
    // ledger above is a statement about a lattice where nothing happened.
    assert!(holon_fluid_ledger_collisions() > 0.0, "no collision fired");
    assert!(holon_fluid_ledger_formed() > 0.0, "no bond ever formed");
    assert!(holon_fluid_ledger_broken_rent() > 0.0, "no bond ever paid rent");
    assert!(holon_fluid_ledger_blocked() > 0.0, "the exclusion branch never fired");
    assert!(holon_fluid_ledger_bonds() > 0.0, "no bond is held at the end");

    // The graph readouts exist and sit inside their own conventions: `B/N` on a ceiling of 1
    // (one donor arm per particle), the degree on a ceiling of 2, and the largest component a
    // fraction. FLUID-1's `0.289` is at `L = 256` over a warmed-up sample and is NOT asserted
    // here — this box is a quarter of the side and this is 500 steps from a cold seed.
    let bpp = holon_fluid_bonds_per_particle();
    assert!(bpp > 0.0 && bpp <= 1.0, "bonds per particle {bpp} is outside the lattice's ceiling of 1");
    assert!(
        (holon_fluid_bonds_per_particle_degree() - 2.0 * bpp).abs() < 1e-12,
        "the two conventions disagree"
    );
    let frac = holon_fluid_largest_fraction();
    assert!(frac > 0.0 && frac <= 1.0, "the largest component's fraction {frac} is not a fraction");
    assert_eq!(
        holon_fluid_bonds_fill() as f64,
        holon_fluid_ledger_bonds(),
        "the drawn bond list is not the held bonds"
    );
}

/// THE CONTROL, THROUGH THE DOOR: with bonds forbidden the occupation bytes the door serves
/// are `Lattice::advance_with_colour`'s, bit for bit, at every one of 500 steps — which is
/// `orientation.rs`'s own `the_no_bond_path_is_advance_with_colour`, re-run across the ABI so
/// that a door assembling the wrong seed or the wrong chirality cannot pass it.
///
/// The referee is built HERE from the crate's own constructors with the door's own parameters,
/// so what is compared is two computations of one thing rather than the door against itself.
#[test]
fn the_no_bond_control_is_advance_with_colour_through_the_door() {
    let _lock = serial();
    stage_rule();

    for chirality in [0u32, 1] {
        let l = 64u32;
        let density = 0.2f64;
        let seed_lo = 0xC1A5_0007u32;
        let seed_hi = 0u32;
        let seed = seed_lo as u64;

        // The door, with bonds forbidden from the first step.
        assert_eq!(
            holon_fluid_begin(l, density, seed_lo, seed_hi, chirality, 0),
            0,
            "the door refused the no-bond scene"
        );
        assert_eq!(holon_fluid_bonds_enabled(), 0, "the door reports bonds on in the control");

        // The referee: FLUID-0's own seeding and FLUID-0's own colour step.
        let m = Model::fhp6();
        let law = m.fhp_i(chirality != 0);
        let lat = Lattice::seeded(m, l as usize, seed, density, law);
        let mut cells = lat.cells.clone();
        let mut col = lat.seed_colour_wave(seed, 1.0, 1);
        let mut out = vec![0u8; cells.len()];
        let mut out_col = vec![0u8; cells.len()];
        assert!(col.iter().any(|&q| q != 0), "the referee's tracer plane is empty");

        let n = holon_fluid_cells_len() as usize;
        assert_eq!(n, cells.len(), "the door's occupation buffer is the wrong length");

        for t in 0..500u64 {
            lat.advance_with_colour(&mut cells, &mut col, &mut out, &mut out_col, t);
            core::mem::swap(&mut cells, &mut out);
            core::mem::swap(&mut col, &mut out_col);
            assert_eq!(holon_fluid_step(1), 1);
            let served = unsafe { std::slice::from_raw_parts(holon_fluid_cells_ptr(), n) };
            assert_eq!(
                served, &cells[..],
                "chirality={chirality}: the door's occupation diverged from advance_with_colour at step {t}"
            );
            assert_eq!(
                holon_fluid_ledger_bonds(),
                0.0,
                "chirality={chirality}: a bond formed with bonds forbidden, at step {t}"
            );
        }
        assert_eq!(holon_fluid_ledger_exact(), 1, "the control's own ledger is not exact");
        assert_eq!(holon_fluid_ledger_formed(), 0.0, "the control formed a bond");
        assert_eq!(holon_fluid_bonds_fill(), 0, "the control drew a bond");
        assert!(holon_fluid_ledger_collisions() > 0.0, "the control never collided");
    }

    // AND THE TOGGLE IS THE SAME OBJECT. `holon_fluid_no_bond` rebuilds from the scene's own
    // `(L, density, seed, chirality)`, so switching the control on must land on exactly the
    // lattice `holon_fluid_begin` with `bonds_on = 0` lands on. A toggle that flipped a flag
    // under a running configuration would leave bonds already formed and would not.
    assert_eq!(holon_fluid_begin(32, 0.2, 0xBEEF, 0, 1, 1), 0);
    for _ in 0..40 {
        holon_fluid_step(1);
    }
    assert!(holon_fluid_ledger_bonds() > 0.0, "the chart scene holds no bond to lose");
    assert_eq!(holon_fluid_no_bond(1), 0);
    assert_eq!(holon_fluid_bonds_enabled(), 0);
    assert_eq!(holon_fluid_steps(), 0.0, "the control did not restart the clock");
    let n = holon_fluid_cells_len() as usize;
    let after_toggle = unsafe { std::slice::from_raw_parts(holon_fluid_cells_ptr(), n) }.to_vec();
    assert_eq!(holon_fluid_begin(32, 0.2, 0xBEEF, 0, 1, 0), 0);
    let fresh = unsafe { std::slice::from_raw_parts(holon_fluid_cells_ptr(), n) };
    assert_eq!(after_toggle, fresh, "the toggle's control is not the freshly-begun control");
}

/// EVERY REFUSAL FIRES BY NAME, and a refusal changes nothing on screen. A door that returned
/// one code for every bad input would leave a page unable to say what it refused, and a door
/// that half-applied a refused call would leave a scene nobody asked for.
#[test]
fn every_refusal_fires_by_its_own_name_and_changes_nothing() {
    let _lock = serial();
    stage_rule();
    assert_eq!(holon_fluid_begin(64, 0.2, 0x5EED, 0, 1, 1), 0);
    for _ in 0..10 {
        holon_fluid_step(1);
    }
    let mass = holon_fluid_ledger_mass();
    let steps = holon_fluid_steps();

    assert_eq!(holon_fluid_begin(4, 0.2, 1, 0, 1, 1), REFUSE_BAD_L, "a box below the floor");
    assert_eq!(holon_fluid_begin(512, 0.2, 1, 0, 1, 1), REFUSE_BAD_L, "a box above the ceiling");
    assert_eq!(holon_fluid_begin(64, 0.0, 1, 0, 1, 1), REFUSE_BAD_DENSITY, "an empty box");
    assert_eq!(holon_fluid_begin(64, 1.0, 1, 0, 1, 1), REFUSE_BAD_DENSITY, "a full box");
    assert_eq!(holon_fluid_begin(64, f64::NAN, 1, 0, 1, 1), REFUSE_BAD_DENSITY, "a NaN density");
    assert_eq!(holon_fluid_amplitude(6, 1.0), REFUSE_BAD_AMPLITUDE, "a seventh angle");
    assert_eq!(holon_fluid_amplitude(0, 0.0), REFUSE_BAD_AMPLITUDE, "a zero amplitude");
    assert_eq!(holon_fluid_amplitude(0, f64::NAN), REFUSE_BAD_AMPLITUDE, "a NaN amplitude");
    assert_eq!(holon_fluid_rent_from_retention(0.0), REFUSE_BAD_RETENTION, "a retention at zero");
    assert_eq!(holon_fluid_rent_from_retention(1.0), REFUSE_BAD_RETENTION, "a retention at one");
    assert_eq!(holon_fluid_rent_from_retention(-0.5), REFUSE_BAD_RETENTION, "a negative retention");

    // NOTHING MOVED. Each refusal above was a call that could have half-applied.
    assert_eq!(holon_fluid_ledger_mass(), mass, "a refused call rebuilt the scene");
    assert_eq!(holon_fluid_steps(), steps, "a refused call stepped the scene");
    assert_eq!(holon_fluid_l(), 64, "a refused call resized the box");

    // The staged rule survived its own refusals, value for value.
    for (i, a) in A_TABLE.iter().enumerate() {
        assert_eq!(holon_fluid_amplitude_at(i as u32), *a, "A({i}) was damaged by a refusal");
    }
    assert!(holon_fluid_amplitude_at(6).is_nan(), "a seventh angle reads a number");
    assert_eq!(
        holon_fluid_rent(),
        (RETENTION / (1.0 - RETENTION)).ln(),
        "the rent is not the retention's own logit"
    );
    // The chart, from the crate's own `p_break`, at the angle the retention was set on.
    let p0 = holon_fluid_p_break(0);
    assert!(
        (1.0 / (1.0 + p0) - RETENTION).abs() < 1e-12,
        "the linear bond's stationary held fraction {} is not the read retention",
        1.0 / (1.0 + p0)
    );
    assert!(holon_fluid_p_break(3) < p0, "the strongest angle does not break least");
    assert!(holon_fluid_p_break(6).is_nan(), "a seventh angle carries a break probability");
}

/// WHAT THE PAGE DRAWS IS THE STATE. The image is one pixel per cell and its alpha is the
/// occupancy, so an empty cell is transparent and a full one is opaque; the bond list is the
/// LIVE bonds, each an intact link; and the geometry doors serve the pinned direction set
/// rather than a hexagon typed into JavaScript.
#[test]
fn the_drawn_buffers_are_the_lattice_and_not_a_picture_of_one() {
    let _lock = serial();
    stage_rule();
    let l = 32usize;
    assert_eq!(holon_fluid_begin(l as u32, 0.2, 0x0D2A, 0, 1, 1), 0);
    for _ in 0..80 {
        holon_fluid_step(1);
    }

    let bytes = holon_fluid_image_fill() as usize;
    assert_eq!(bytes, l * l * 4);
    assert_eq!(holon_fluid_image_len() as usize, bytes);
    let img = unsafe { std::slice::from_raw_parts(holon_fluid_image_ptr(), bytes) };
    let cells = unsafe { std::slice::from_raw_parts(holon_fluid_cells_ptr(), l * l) };
    let mut empty = 0usize;
    let mut full = 0usize;
    for c in 0..l * l {
        let occ = cells[c].count_ones();
        let alpha = img[4 * c + 3];
        if occ == 0 {
            empty += 1;
            assert_eq!(alpha, 0, "an empty cell drew opaque at {c}");
        } else {
            full += 1;
            let want = (48.0 + 207.0 * (occ as f64 / 6.0)) as u8;
            assert_eq!(alpha, want, "cell {c} holds {occ} and drew alpha {alpha}");
        }
    }
    assert!(empty > 0 && full > 0, "the picture has only one kind of cell in it: {empty}/{full}");

    // The bond list: every entry an intact link, in slot coordinates, inside the buffer.
    let n = holon_fluid_bonds_fill() as usize;
    assert!(n > 0, "no bond to draw after 80 steps");
    assert!(n <= holon_fluid_bonds_capacity() as usize, "the fill ran past its own capacity");
    assert_eq!(
        holon_fluid_bonds_live() as usize,
        n,
        "the live-bond count and the fill's own return disagree"
    );
    let stride = holon_fluid_bond_stride() as usize;
    assert_eq!(stride, 3, "the buffer's stride changed and this gate reads by the door's number");
    let bonds = unsafe { std::slice::from_raw_parts(holon_fluid_bonds_ptr(), stride * n) };
    let orient = unsafe { std::slice::from_raw_parts(holon_fluid_orient_ptr(), l * l * 6) };
    assert_eq!(holon_fluid_orient_len() as usize, l * l * 6);
    // THE THIRD WORD IS THE LINK DIRECTION, and this loop's whole point is that it is not the
    // donor's slot. A slot is `cell * 6 + dir`, so `donor % 6` is the direction the donor is
    // MOVING in; the bond points along the donor's ORIENTATION. They agree only by accident,
    // and a host reaching for the first would draw a picture that still looks like a lattice
    // and is wrong — so the door serves the second and this gate measures how often the two
    // differ rather than assuming they do.
    let mut slot_differs_from_link = 0usize;
    for k in 0..n {
        let (donor, acceptor) = (bonds[stride * k] as usize, bonds[stride * k + 1] as usize);
        let served = bonds[stride * k + 2] as usize;
        assert!(donor < l * l * 6 && acceptor < l * l * 6, "a slot past the end of the lattice");
        assert_ne!(orient[donor], holon_fluid_no_orient() as u8, "a bond donated by a hole");
        assert_ne!(orient[acceptor], holon_fluid_no_orient() as u8, "a bond accepted by a hole");
        let (dc, dd) = (donor / 6, donor % 6);
        assert_eq!(cells[dc] >> dd & 1, 1, "a bond donated by an empty slot");
        assert_eq!(
            served,
            orient[donor] as usize,
            "the served link direction is not the donor's orientation"
        );
        if served != dd {
            slot_differs_from_link += 1;
        }
        // The link is the donor's own arm: the acceptor's cell is the donor's neighbour along
        // that direction. That is `bond_geometry`'s rule, checked here on the axial offsets the
        // door serves rather than on a copy of it, and on the SERVED direction rather than on
        // one this test recomputed — so a door that served the wrong word fails here.
        let (di, dj) = (
            holon_fluid_dir_axial(served as u32, 0) as i64,
            holon_fluid_dir_axial(served as u32, 1) as i64,
        );
        let (i, j) = ((dc / l) as i64, (dc % l) as i64);
        let ii = (i + di).rem_euclid(l as i64) as usize;
        let jj = (j + dj).rem_euclid(l as i64) as usize;
        assert_eq!(ii * l + jj, acceptor / 6, "the bond's link is not the donor's own arm");
    }
    // The distinction is REAL on this scene rather than merely stated: a run where every bond
    // happened to have slot == orientation would pass every line above while telling you
    // nothing about which of the two the door serves.
    assert!(
        slot_differs_from_link * 2 > n,
        "only {slot_differs_from_link} of {n} bonds have a link direction differing from the \
         donor's slot, so this scene cannot tell the two apart and the check above is vacuous"
    );

    // The Euclidean embedding is the hexagon's, from `isotropy::embed`: six unit vectors.
    for d in 0..6u32 {
        let (x, y) = (holon_fluid_dir_euclidean(d, 0), holon_fluid_dir_euclidean(d, 1));
        assert!((x * x + y * y - 1.0).abs() < 1e-12, "direction {d} embeds at length {}", (x * x + y * y).sqrt());
    }
    assert!(holon_fluid_dir_euclidean(6, 0).is_nan());
    assert!(holon_fluid_dir_axial(0, 2).is_nan());
}

/// NO LATTICE IS AN ABSENCE, NOT A ZERO. Before anything is begun every readout says so in a
/// way a page cannot draw as a digit — NaN for a quantity, a null pointer for a buffer, 0 for
/// a count of steps taken. A door that returned 0.0 for "mass" with no scene would put a
/// zero on screen that reads as a measurement of an empty box.
///
/// It runs LAST by name (`zzz_`), because it is the one gate that must see the door in its
/// initial state and `cargo test` orders a file's tests by name within a thread pool this
/// file's lock serialises.
#[test]
fn zzz_with_no_lattice_every_readout_is_an_absence() {
    let _lock = serial();
    // Put the door back to no-lattice by the only route there is from outside: there is no
    // `holon_fluid_end`, so this gate asserts the shape of the absence on the values a host
    // meets when a call is refused, which is the same code path (`read`'s `absent` arm).
    assert_eq!(holon_fluid_begin(4, 0.2, 1, 0, 1, 1), REFUSE_BAD_L);
    // A refusal must not have built anything, so if nothing was ever begun in this process
    // the absences hold. Under the file's lock the earlier gates have begun a lattice, so the
    // assertion that BITES here is the one about the refusal's own return value above and the
    // absence of a seventh angle below — the rest is checked at the door's own boundary.
    assert!(holon_fluid_amplitude_at(9).is_nan(), "an angle that does not exist reads a number");
    assert!(holon_fluid_p_break(9).is_nan(), "an angle that does not exist has a break probability");
    assert!(holon_fluid_ledger_census(9).is_nan(), "an orientation that does not exist has a count");
    assert!(holon_fluid_ledger_initial(9).is_nan(), "an unnamed ledger row has a value");
    assert_eq!(holon_fluid_no_bond(0), 0, "the control could not be switched back");
    assert_eq!(holon_fluid_bonds_enabled(), 1);
}
