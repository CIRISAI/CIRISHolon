//! SATURATION-1 gate C1: the ledger still balances while the third body acts.
//!
//! One gate per conservation law, never combined, each printing its measured margin —
//! the house style of `tests/ledger.rs`, which these scenes are the three-body twins of.
//!
//! The three-body surface is generated once for the whole binary (7,293 electronic
//! structure solves) and copied into each scene, so the tests measure dynamics rather than
//! table building.

use holon_render::sim::{Boundary, Dims, Sim, K_B, M_H, DEFAULT_SCENE_ATOMS};
use holon_render::{generate_table, generate_trimer_table, TABLE_OK};
use std::sync::OnceLock;

/// A `Sim` with both tables loaded. Boxed: it carries the whole three-body table, which is
/// a fixed array of 14,157 doubles and does not belong on a test's stack.
fn loaded() -> &'static Sim {
    static S: OnceLock<Box<Sim>> = OnceLock::new();
    S.get_or_init(|| {
        let mut s = Box::new(Sim::empty());
        assert_eq!(
            generate_table(&mut s, 0.3, 10.0, 492),
            TABLE_OK,
            "the pair curve did not generate"
        );
        assert_eq!(
            generate_trimer_table(&mut s),
            1,
            "the three-body table did not generate"
        );
        s
    })
}

/// A fresh scene sharing the loaded tables.
fn scene() -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    let src = loaded();
    assert_eq!(generate_table(&mut s, 0.3, 10.0, 492), TABLE_OK);
    s.trimer = src.trimer.clone();
    s
}

fn run(s: &mut Sim, frames: usize, substeps: u32) {
    for _ in 0..frames {
        s.step_frame(substeps);
    }
}

/// Run, and report the largest `|E_three|` the scene ever held.
///
/// The FINAL value is not the right carrier: three hydrogens cannot form a compact
/// trimer — that is the campaign's whole claim — so a three-atom NVE scene ends dispersed
/// with the term at exactly zero however violently it acted on the way there.
fn run_tracking_three(s: &mut Sim, frames: usize, substeps: u32) -> f64 {
    let mut peak = 0.0f64;
    for _ in 0..frames {
        s.step_frame(substeps);
        peak = peak.max(s.e_three.abs());
    }
    peak
}

// ------------------------------------------------------------------ the table itself

/// What the table says about itself, printed once so every other number in this file has
/// its instrument on the record.
#[test]
fn the_three_body_table_reports_its_own_scale() {
    let s = loaded();
    let t = &s.trimer;
    println!(
        "trimer table: {} nodes from {} solves; peak |dE3| = {:.6} Ha; \
         curvature envelope {:.3} Ha/bohr^2, per-gradient {:.3} /bohr; \
         sort kink {:.3e} Ha/bohr; E(H) = {:.9} Ha",
        t.meta.n_nodes, t.meta.solves, t.meta.peak, t.curvature_envelope,
        t.curvature_per_gradient, t.sort_kink, t.meta.e_h_atom
    );
    assert!(t.loaded, "the table did not close");
    assert!(t.meta.peak > 1.0, "the compact corner lost its scale");
    assert!(
        t.curvature_envelope.is_finite() && t.curvature_envelope > 0.0,
        "the curvature envelope is not a number the bound can use"
    );
}

// ------------------------------------------------------------------ the staked scenes

/// SCENE A — three atoms, open box, pure NVE. The smallest scene in which the three-body
/// term exists at all, and the one where nothing else can be blamed: no walls, no spring,
/// no thermostat, so `W_ext` must be exactly zero and every joule that moves is the
/// integrator's.
///
/// The triangle opens compact (2.2 bohr sides) with the third atom moving across, so the
/// run sweeps the surface rather than sitting on one point of it.
fn staked_trimer_nve() -> Box<Sim> {
    let mut s = scene();
    s.boundary = Boundary::Open;
    s.dims = Dims::Two;
    s.reset(3);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    s.set_position(0, cx - 1.1, cy - 0.6);
    s.set_position(1, cx + 1.1, cy - 0.6);
    s.set_position(2, cx, cy + 1.3);
    s.set_velocity(0, 0.0015, 0.0005);
    s.set_velocity(1, -0.0015, 0.0005);
    s.set_velocity(2, 0.0004, -0.0010);
    s.rebase();
    s
}

/// SCENE B — eight atoms in the box with the thermostat on: walls, external work, and
/// enough atoms that every one of them sits in `C(7,2) = 21` triples.
fn staked_trimer_thermostat() -> Box<Sim> {
    let mut s = scene();
    s.boundary = Boundary::Walls;
    s.dims = Dims::Two;
    s.reset(8);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    // A deterministic ring at 4 bohr: close enough that most triples are inside the
    // table's domain, far enough that nothing opens on the repulsive wall.
    for i in 0..8 {
        let th = i as f64 * core::f64::consts::TAU / 8.0;
        s.set_position(i, cx + 4.0 * th.cos(), cy + 4.0 * th.sin());
        let v = (K_B * 2000.0 / M_H).sqrt();
        s.set_velocity(i, -v * th.sin(), v * th.cos());
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = 600.0;
    s.thermostat_tau = 2000.0;
    s
}

// ------------------------------------------------------------------ C1: energy

#[test]
fn c1_energy_gate_three_body_nve() {
    let mut s = staked_trimer_nve();
    assert!(s.trimer.loaded, "the three-body table is not loaded");
    let peak_three = run_tracking_three(&mut s, 400, 64);

    let bound = s.drift_bound();
    println!(
        "C1 NVE (3 atoms, 400 x 64): |dE|_peak = {:.6e} Eh   bound = {:.6e} Eh   \
         ratio = {:.4}",
        s.drift_peak,
        bound,
        s.drift_peak / bound
    );
    println!(
        "  E_kin = {:.9}  E_pair = {:.9}  E_three = {:.9}  W_ext = {:.9}",
        s.e_kin, s.e_pair, s.e_three, s.w_ext
    );
    println!(
        "  bound parts: k_pair_max = {:.4}  k_three = {:.4} Ha/bohr^2  E_ref = {:.4} Eh",
        s.k_pair_max(),
        s.k_three(),
        s.e_ref
    );
    println!("  peak |E_three| over the run = {peak_three:.6} Eh");
    // The three-body sector must actually have carried something, or the gate is measuring
    // the pair scene it already measured.
    assert!(
        peak_three > 1e-3,
        "the three-body term contributed nothing ({peak_three:.3e}): this is the pair gate \
         in disguise"
    );
    assert!(
        s.k_three() > 0.0,
        "the three-body stiffness never entered the bound"
    );
    assert_eq!(s.w_ext, 0.0, "NVE run injected external work");
    assert_eq!(s.e_wall, 0.0, "walls are off but carry energy");
    assert!(
        s.energy_gate(),
        "drift {:.3e} exceeds bound {:.3e} with the third body paying",
        s.drift_peak,
        bound
    );
}

#[test]
fn c1_energy_gate_three_body_thermostatted() {
    let mut s = staked_trimer_thermostat();
    run(&mut s, 400, 64);
    println!(
        "C1 thermostat (8 atoms, 400 x 64): T = {:.1} K, W_ext = {:.6e} Eh, \
         E_three = {:.6} Eh, |dE|_peak = {:.3e}, bound = {:.3e}, ratio = {:.4}",
        s.temperature(),
        s.w_ext,
        s.e_three,
        s.drift_peak,
        s.drift_bound(),
        s.drift_peak / s.drift_bound()
    );
    println!(
        "  k_pair_max = {:.4}  k_three = {:.4} Ha/bohr^2  clusters = {:?}",
        s.k_pair_max(),
        s.k_three(),
        s.cluster_count()
    );
    assert!(s.w_ext != 0.0, "the thermostat moved no energy");
    assert!(s.e_three != 0.0, "the three-body term contributed nothing");
    assert!(
        s.energy_gate(),
        "the third body broke the energy ledger: drift {:.3e} vs bound {:.3e}",
        s.drift_peak,
        s.drift_bound()
    );
}

// ------------------------------------------------------------------ C1: momentum

/// The momentum gate is asserted SEPARATELY and against its own bound, which the
/// three-body term does not touch: every triple applies its three side forces as one
/// computed value with opposite signs, so the triple's contribution to the total is zero
/// in exact arithmetic and what is left is floating-point cancellation, exactly as for the
/// pair loop.
#[test]
fn c1_momentum_gate_three_body_nve() {
    let mut s = staked_trimer_nve();
    let p0 = s.momentum();
    let peak_three = run_tracking_three(&mut s, 400, 64);
    let p = s.momentum();
    let bound = s.momentum_bound();
    println!(
        "C1 momentum (3 atoms, 400 x 64): |dP|_peak = {:.6e}   bound = {:.6e}   \
         ratio = {:.4}",
        s.momentum_residual_peak,
        bound,
        s.momentum_residual_peak / bound
    );
    println!(
        "  P0 = ({:.6e}, {:.6e}, {:.6e})  P = ({:.6e}, {:.6e}, {:.6e})",
        p0.0, p0.1, p0.2, p.0, p.1, p.2
    );
    println!("  peak |E_three| over the run = {peak_three:.6} Eh");
    assert!(
        peak_three > 1e-3,
        "the three-body term contributed nothing ({peak_three:.3e})"
    );
    assert_eq!(
        s.j_ext,
        (0.0, 0.0, 0.0),
        "no external force acted but impulse accrued"
    );
    assert!(
        s.momentum_gate(),
        "the triple force is not translation-invariant: residual {:.3e} exceeds the \
         roundoff bound {:.3e}",
        s.momentum_residual_peak,
        bound
    );
}

#[test]
fn c1_momentum_gate_three_body_thermostatted() {
    let mut s = staked_trimer_thermostat();
    run(&mut s, 400, 64);
    println!(
        "C1 momentum (8 atoms, thermostat): |dP|_peak = {:.6e}  bound = {:.6e}  \
         ratio = {:.4}",
        s.momentum_residual_peak,
        s.momentum_bound(),
        s.momentum_residual_peak / s.momentum_bound()
    );
    assert!(
        s.momentum_gate(),
        "the third body broke the momentum ledger: {:.3e} vs {:.3e}",
        s.momentum_residual_peak,
        s.momentum_bound()
    );
}

// ------------------------------------------------------------------ the term itself

/// The three-body force is the exact gradient of the three-body energy the ledger sums.
///
/// THE precondition for C1 to be measuring integration error rather than an
/// inconsistency: displace one atom, and the change in `E_three` must be minus the work
/// the three-body force did. Checked on the scene, not on the interpolant, so it exercises
/// the chain rule from sides to positions as well as the table.
#[test]
fn the_triple_force_is_minus_the_gradient_of_the_triple_energy() {
    let mut s = staked_trimer_nve();
    s.set_velocity(0, 0.0, 0.0);
    s.set_velocity(1, 0.0, 0.0);
    s.set_velocity(2, 0.0, 0.0);
    s.rebase();
    let h = 1e-6;
    let mut worst = 0.0f64;
    for atom in 0..3 {
        for axis in 0..2 {
            let (x0, y0) = (s.atoms[atom].x, s.atoms[atom].y);
            let mut e = [0.0f64; 2];
            for (k, sign) in [(0usize, -1.0f64), (1, 1.0)] {
                if axis == 0 {
                    s.set_position(atom, x0 + sign * h, y0);
                } else {
                    s.set_position(atom, x0, y0 + sign * h);
                }
                s.rebase();
                e[k] = s.e_three;
            }
            s.set_position(atom, x0, y0);
            s.rebase();
            let numeric = (e[1] - e[0]) / (2.0 * h);
            // The THREE-BODY force alone, isolated by differencing the internal force with
            // the table on against the same force with it off. No new accessor and no
            // second copy of the chain rule: the number tested is the one the integrator
            // actually pushed with.
            let on = s.internal_force(atom);
            s.trimer.loaded = false;
            s.rebase();
            let off = s.internal_force(atom);
            s.trimer.loaded = true;
            s.rebase();
            let f3 = if axis == 0 { on.0 - off.0 } else { on.1 - off.1 };
            let analytic = -f3;
            let rel = (numeric - analytic).abs() / (analytic.abs() + 1e-4);
            worst = worst.max(rel);
        }
    }
    println!("triple force vs -grad E_three: worst relative gap = {worst:.3e}");
    assert!(
        worst < 1e-5,
        "the triple force is not the gradient of the triple energy: {worst:.3e}"
    );
}

/// The three-body term is EXACTLY absent when no table is loaded — not small, absent — so
/// every gate written before this campaign reads the float it always did.
#[test]
fn without_a_table_the_third_body_changes_no_bit() {
    let mut with_table = scene();
    let mut without = scene();
    without.trimer.loaded = false;
    for s in [&mut with_table, &mut without] {
        s.boundary = Boundary::Open;
        s.reset(2);
        let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
        s.set_position(0, cx - 1.1, cy);
        s.set_position(1, cx + 1.1, cy);
        s.set_velocity(0, 0.002, 0.001);
        s.set_velocity(1, -0.002, 0.001);
        s.rebase();
        run(s, 156, 64);
    }
    // Two atoms: no triples exist, so the two runs must agree bit for bit either way.
    assert_eq!(with_table.energy().to_bits(), without.energy().to_bits());
    assert_eq!(with_table.drift_peak.to_bits(), without.drift_peak.to_bits());
    assert_eq!(
        with_table.drift_bound().to_bits(),
        without.drift_bound().to_bits(),
        "an unreachable three-body term still moved the drift bound"
    );
    assert_eq!(with_table.e_three, 0.0);
    println!(
        "two-atom scene: E = {:.15e}, drift_peak = {:.6e}, bound = {:.6e} — identical with \
         and without a three-body table",
        with_table.energy(),
        with_table.drift_peak,
        with_table.drift_bound()
    );
}

/// The N^3 loop's cost, measured rather than projected — the number the calibration burst
/// sees, and the one the report has to carry.
#[test]
fn the_triple_loop_cost_is_measured() {
    let mut s = staked_trimer_thermostat();
    s.reset(DEFAULT_SCENE_ATOMS);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    for i in 0..DEFAULT_SCENE_ATOMS {
        let th = i as f64 * core::f64::consts::TAU / DEFAULT_SCENE_ATOMS as f64;
        s.set_position(i, cx + 5.0 * th.cos(), cy + 5.0 * th.sin());
    }
    s.rebase();
    s.thermostat_on = false;
    let substeps = 64u32;
    let frames = 40usize;
    let t0 = std::time::Instant::now();
    run(&mut s, frames, substeps);
    let with = t0.elapsed().as_secs_f64() / (frames as f64 * substeps as f64);

    s.trimer.loaded = false;
    s.rebase();
    let t1 = std::time::Instant::now();
    run(&mut s, frames, substeps);
    let without = t1.elapsed().as_secs_f64() / (frames as f64 * substeps as f64);
    println!(
        "N = {DEFAULT_SCENE_ATOMS}, {} triples: {:.2} us per substep with the three-body loop, \
         {:.2} us without — {:.1}x",
        DEFAULT_SCENE_ATOMS * (DEFAULT_SCENE_ATOMS - 1) * (DEFAULT_SCENE_ATOMS - 2) / 6,
        with * 1e6,
        without * 1e6,
        with / without
    );
    assert!(with > 0.0 && without > 0.0);
}

// ------------------------------------------------------------------ plant (iii)

/// PLANT (iii), the structural half: the dynamics provably READS the table inside the
/// perimeter the plant zeroes.
///
/// The prereg's plant is a D1 spot check — zero `dE3` below a 4-bohr perimeter and the
/// MBE3 arm must move back toward the droplet — and that lives in the quench runner, not
/// in a unit test, because it is two full 33-ps runs. What belongs here is the half that
/// can be measured in milliseconds and that the spot check would otherwise be assuming:
/// that the mutation lands on a nonempty sector of the SURFACE and that the force loop
/// reads it. If this fails, the spot check is scored on nothing.
///
/// The perimeter matters and is worth stating plainly: 4 bohr is an equilateral triangle
/// of side 1.33, TIGHTER than the H2 equilibrium separation of 1.389, so the region the
/// plant removes is the compact core and not the approach geometries where a third atom
/// meets a bond. Whether the dynamics visits it is the spot check's question; whether the
/// engine can see it at all is this one's.
#[test]
fn plant_iii_the_force_loop_reads_the_zeroed_region() {
    let mut plain = scene();
    let mut planted = scene();
    planted.trimer.zero_inside_perimeter(4.0);

    // The carrier: the surface inside the perimeter must be nonzero before the plant and
    // exactly zero after it.
    let compact = [1.2f64, 1.2, 1.2]; // perimeter 3.6
    let (v_plain, g_plain) = plain.trimer.eval(compact);
    let (v_planted, _) = planted.trimer.eval(compact);
    println!(
        "plant (iii) carrier: dE3 at the compact triple (perimeter {:.2}) = {v_plain:+.6} Ha \
         before, {v_planted:+.6} Ha after",
        compact.iter().sum::<f64>()
    );
    assert!(
        v_plain > 1e-3,
        "the sector the plant acts on is empty: nothing to zero"
    );
    assert!(
        g_plain.iter().any(|g| g.abs() > 1e-3),
        "the surface is flat where the plant acts: no force to remove"
    );

    // And the force loop reads it: an identical scene differs in E_three and in the force
    // on every atom.
    for s in [&mut plain, &mut planted] {
        s.boundary = Boundary::Open;
        s.reset(3);
        let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
        s.set_position(0, cx - 0.6, cy - 0.35);
        s.set_position(1, cx + 0.6, cy - 0.35);
        s.set_position(2, cx, cy + 0.69);
        s.rebase();
    }
    let df = {
        let a = plain.internal_force(2);
        let b = planted.internal_force(2);
        ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
    };
    println!(
        "plant (iii): E_three {:+.6} -> {:+.6} Ha, and the force on atom 2 moves by \
         {df:.4} Ha/bohr",
        plain.e_three, planted.e_three
    );
    assert!(
        (plain.e_three - planted.e_three).abs() > 1e-3,
        "the plant did not change the energy the ledger sums"
    );
    assert!(
        df > 1e-3,
        "the plant did not change the force the integrator pushes with: the loop is not \
         reading the table where the plant acts"
    );
}

/// PLANT (iii), the DRIVEN half: a trajectory that DOES pass through the zeroed sector
/// diverges, and the divergence is the plant's purpose discharged.
///
/// The staked plant is scored on the D1 outcome shifting, and it does not shift, because
/// the MBE3 trajectories never enter the sector it zeroes: measured over 40,000 grain
/// boundaries on two seeds, the closest any domain triple comes is a perimeter of 8.584
/// bohr, and the count of boundaries with a triple inside the plant's 4 is ZERO. That is
/// M-PLANT-SECTOR's empty sector, and the reason it is empty is the term being tested —
/// the three-body repulsion is what keeps the trajectory out.
///
/// So the sector is entered on purpose here instead. Three atoms are STARTED inside it, at
/// rest, and integrated; with the table intact they are flung apart by a surface worth
/// ~1 Ha, and with the sector zeroed they are not. Same integrator, same seeds, same
/// everything else.
#[test]
fn plant_iii_a_driven_entry_diverges() {
    let mut plain = scene();
    let mut planted = scene();
    planted.trimer.zero_inside_perimeter(4.0);
    let mut readings = Vec::new();
    for s in [&mut plain, &mut planted] {
        s.boundary = Boundary::Open;
        s.reset(3);
        let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
        // A compact equilateral trimer of side 1.2 bohr: perimeter 3.6, well inside the
        // plant's 4, and at rest so the only thing that can move it is the surface.
        s.set_position(0, cx - 0.6, cy - 0.3464);
        s.set_position(1, cx + 0.6, cy - 0.3464);
        s.set_position(2, cx, cy + 0.6928);
        s.set_velocity(0, 0.0, 0.0);
        s.set_velocity(1, 0.0, 0.0);
        s.set_velocity(2, 0.0, 0.0);
        s.rebase();
        let e0 = s.e_three;
        run(s, 60, 64);
        s.refresh_pairs();
        let spread = s.pairs[..s.pair_count]
            .iter()
            .map(|p| p.r)
            .fold(0.0f64, f64::max);
        readings.push((e0, spread, s.e_kin));
    }
    let (e0_plain, spread_plain, kin_plain) = readings[0];
    let (e0_planted, spread_planted, kin_planted) = readings[1];
    println!(
        "plant (iii) driven entry (compact trimer, side 1.2 bohr, at rest, 60 x 64):\n  \
         intact:  E_three(0) = {e0_plain:+.6} Eh -> widest separation {spread_plain:.3} bohr, \
         E_kin = {kin_plain:.6} Eh\n  \
         planted: E_three(0) = {e0_planted:+.6} Eh -> widest separation {spread_planted:.3} \
         bohr, E_kin = {kin_planted:.6} Eh"
    );
    assert!(
        e0_plain > 0.5,
        "the driven scene did not start inside a live sector: E_three = {e0_plain:.3e}"
    );
    assert!(
        e0_planted.abs() < 0.05,
        "the plant did not empty the sector the scene starts in: {e0_planted:.3e}"
    );
    assert!(
        spread_plain > 2.0 * spread_planted,
        "the two trajectories did not diverge ({spread_plain:.3} vs {spread_planted:.3} \
         bohr): the dynamics is not reading the table where the plant acts"
    );
}
