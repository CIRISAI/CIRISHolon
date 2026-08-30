//! SATURATION-2 gate C1: the ledger still balances with a HETERONUCLEAR triple paying.
//!
//! One gate per conservation law, never combined, each printing its measured margin — the
//! house style of `tests/ledger.rs` and of `tests/saturation.rs`, which these scenes are
//! the two-element twins of.
//!
//! # What is new here and therefore what can newly go wrong
//!
//! SATURATION-1's third body was hydrogen and so was everything else, which meant one
//! table, one mass, one curvature envelope and one order of the three sides. This campaign
//! breaks all four at once:
//!
//! * the triple loop DISPATCHES on composition, so a triple can now be served the wrong
//!   surface;
//! * the sides are no longer interchangeable — `WaterTable::eval` wants the two O-H sides
//!   first and the H-H side third — so they can now be handed over in the wrong order;
//! * the drift bound's stiffness has to come from whichever table served the triple;
//! * oxygen is sixteen times hydrogen's mass, so a momentum ledger that had never seen two
//!   masses in one box is now seeing them.
//!
//! Each has a test below, and the fence — (O, O, H) and (O, O, O), which SATURATION-2 does
//! not tabulate — is checked to be COUNTED rather than silently absorbed.

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::pair::{generate_pair_table, PairTable};
use holon_render::bank::Host;
use holon_render::sim::{Boundary, Dims, Sim, K_B};
use holon_render::{generate_trimer_table, load_pair_table, load_water_table, TABLE_OK};
use std::sync::OnceLock;

/// Knots per fixture curve. Small on purpose, and for the reason `tests/mixtures.rs`
/// records: the interpolant accuracy of the PAIR curves is MIXTURES-1's gate, not this
/// one's, and every knot is a full CI solve at both ends of the debug profile.
const FIXTURE_KNOTS: usize = 24;

fn water_table_text() -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../holon-chem/tests/data/s2/s2_water_table.txt");
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("the committed (O,H,H) table is present: {}: {e}", p.display()))
}

struct Bank {
    hh: PairTable,
    oh: PairTable,
    trimer: holon_chem::trimer::TrimerTable,
    water: holon_chem::water::WaterTable,
}

/// The three surfaces this file needs, built ONCE for the whole binary. The (O, H, H)
/// table is LOADED rather than generated: 441 determinants a node is not a thing a test
/// suite pays for.
fn banked() -> &'static Bank {
    static B: OnceLock<Bank> = OnceLock::new();
    B.get_or_init(|| {
        let mut probe = Box::new(Sim::empty());
        assert_eq!(
            generate_trimer_table(&mut probe),
            1,
            "the H3 table did not generate"
        );
        let mut w = Box::new(Sim::empty());
        assert_eq!(
            load_water_table(&mut w, &water_table_text()),
            1,
            "the committed (O,H,H) table did not load; if the grid constants moved it has \
             to be regenerated, not re-read"
        );
        Bank {
            hh: generate_pair_table(HYDROGEN, HYDROGEN, FIXTURE_KNOTS),
            oh: generate_pair_table(OXYGEN, HYDROGEN, FIXTURE_KNOTS),
            trimer: probe.trimer.clone(),
            water: w.water.clone(),
        }
    })
}

/// A fresh scene carrying the H-H and O-H curves, the H3 table and the (O, H, H) table.
fn scene() -> Box<Sim> {
    let b = banked();
    let mut s = Box::new(Sim::empty());
    assert_eq!(load_pair_table(&mut s, &b.hh, Host::Native), TABLE_OK);
    assert_eq!(load_pair_table(&mut s, &b.oh, Host::Native), TABLE_OK);
    s.trimer = b.trimer.clone();
    s.water = b.water.clone();
    s
}

fn run(s: &mut Sim, frames: usize, substeps: u32) -> f64 {
    let mut peak = 0.0f64;
    for _ in 0..frames {
        s.step_frame(substeps);
        peak = peak.max(s.e_three.abs());
    }
    peak
}

/// SCENE A — one oxygen and two hydrogens, open box, pure NVE. The smallest scene in which
/// the heteronuclear triple exists at all, and the one where nothing else can be blamed:
/// no walls, no spring, no thermostat, so `W_ext` is exactly zero and every joule that
/// moves is the integrator's.
///
/// It opens NEAR the model's own water optimum (1.94 bohr, 96.8 degrees) but not on it,
/// and with the hydrogens moving, so the run sweeps the surface rather than sitting on one
/// point of it. Opening exactly at the minimum would be a scene in which the three-body
/// force is nearly stationary — a conservation gate that never loaded the term it claims
/// to be gating.
fn staked_water_nve() -> Box<Sim> {
    let mut s = scene();
    s.boundary = Boundary::Open;
    s.dims = Dims::Two;
    s.reset(3);
    assert!(s.set_species(0, OXYGEN), "oxygen did not register");
    assert!(s.set_species(1, HYDROGEN));
    assert!(s.set_species(2, HYDROGEN));
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    // 2.10 bohr at +-55 degrees: off the optimum in both coordinates.
    let (r, half) = (2.10f64, 55.0f64.to_radians());
    s.set_position(0, cx, cy);
    s.set_position(1, cx + r * half.cos(), cy + r * half.sin());
    s.set_position(2, cx + r * half.cos(), cy - r * half.sin());
    s.set_velocity(1, -0.0016, 0.0011);
    s.set_velocity(2, -0.0016, -0.0011);
    s.rebase();
    s
}

/// SCENE B — two oxygens and four hydrogens with the thermostat on: walls, external work,
/// two masses, and every composition of triple the dispatch has a case for, including the
/// fenced ones.
fn staked_mixed_thermostat() -> Box<Sim> {
    let mut s = scene();
    s.boundary = Boundary::Walls;
    s.dims = Dims::Two;
    s.reset(6);
    for (i, sp) in [OXYGEN, OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN]
        .into_iter()
        .enumerate()
    {
        assert!(s.set_species(i, sp), "species {i} did not register");
    }
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    for i in 0..6 {
        let th = i as f64 * core::f64::consts::TAU / 6.0;
        s.set_position(i, cx + 4.0 * th.cos(), cy + 4.0 * th.sin());
        let m = s.atoms[i].mass();
        let v = (K_B * 2000.0 / m).sqrt();
        s.set_velocity(i, -v * th.sin(), v * th.cos());
    }
    s.rebase();
    s.thermostat_on = true;
    s.target_temperature = 600.0;
    s.thermostat_tau = 2000.0;
    s
}

// ------------------------------------------------------------------ the table itself

#[test]
fn the_water_table_reports_its_own_scale() {
    let t = &banked().water;
    println!(
        "(O,H,H) table: {} nodes from {} solves; peak |dE3| = {:.6} Ha; \
         curvature envelope {:.3} Ha/bohr^2, per-gradient {:.3} /bohr; \
         sort kink {:.3e} Ha/bohr; E(O) = {:.9}, E(H) = {:.9} Ha",
        t.meta.n_nodes,
        t.meta.solves,
        t.meta.peak,
        t.curvature_envelope,
        t.curvature_per_gradient,
        t.sort_kink,
        t.meta.e_o_atom,
        t.meta.e_h_atom
    );
    assert!(t.loaded, "the table did not close");
    assert!(t.meta.peak > 0.1, "the compact corner lost its scale");
    assert!(
        t.curvature_envelope.is_finite() && t.curvature_envelope > 0.0,
        "the curvature envelope is not a number the bound can use"
    );
}

// ------------------------------------------------------------------ the force IS a gradient

#[test]
fn the_triple_force_is_minus_the_gradient_of_the_triple_energy() {
    // C1 rests on this and nothing else: if the analytic side-derivatives are not the
    // derivatives of the value the ledger books, the energy cannot be conserved and no
    // amount of drift-bound tuning would fix it. Checked by central differences against
    // the tabulated surface itself, on geometries that span the domain.
    let t = &banked().water;
    const H: f64 = 1e-5;
    let mut worst = 0.0f64;
    let mut worst_at = (0.0, 0.0, 0.0);
    for &(x, y, z) in &[
        (1.94f64, 1.94f64, 2.90f64),
        (1.60, 2.90, 3.10),
        (2.50, 2.50, 1.60),
        (1.20, 4.00, 4.50),
        (3.00, 6.00, 4.20),
        // Inside the closed-angle fence: the linear-in-u extension has to be a genuine
        // first-order Taylor there, or the gradient stops being the value's gradient
        // exactly where an exactly collinear approach lands.
        (1.94, 3.30, 1.37),
    ] {
        let (_, g) = t.eval(x, y, z);
        let sides = [x, y, z];
        for a in 0..3 {
            let mut lo = sides;
            let mut hi = sides;
            lo[a] -= H;
            hi[a] += H;
            let (vlo, _) = t.eval(lo[0], lo[1], lo[2]);
            let (vhi, _) = t.eval(hi[0], hi[1], hi[2]);
            let fd = (vhi - vlo) / (2.0 * H);
            let d = (fd - g[a]).abs();
            // Relative where the gradient is large, absolute where it is small.
            let tol = 1e-6 + 1e-3 * g[a].abs();
            if d / tol > worst {
                worst = d / tol;
                worst_at = (x, y, z);
            }
            assert!(
                d <= tol,
                "at sides ({x}, {y}, {z}) the analytic dF/ds{a} is {:.9e} and the finite \
                 difference of the surface is {fd:.9e}, off by {d:.3e} against a tolerance \
                 of {tol:.3e}",
                g[a]
            );
        }
    }
    println!(
        "gradient-vs-surface: worst residual is {worst:.3} of its own tolerance, at \
         sides {worst_at:?}"
    );
}

// ------------------------------------------------------------------ the dispatch

#[test]
fn the_dispatch_serves_each_composition_its_own_surface() {
    // Three claims, one per case of the dispatch, each read off `e_three` rather than
    // asserted about the code: a scene of one composition must book the energy its own
    // table gives, and the fenced composition must book nothing and be counted.
    let b = banked();

    // (O, H, H): the water table's value, at a geometry well inside its domain.
    let mut s = staked_water_nve();
    s.step_frame(1);
    let (x, y, z) = (
        dist(&s, 0, 1),
        dist(&s, 0, 2),
        dist(&s, 1, 2),
    );
    let (want, _) = b.water.eval(x, y, z);
    assert!(
        (s.e_three - want).abs() < 1e-12,
        "the (O,H,H) triple booked {:.9e} Ha where its own table says {want:.9e}",
        s.e_three
    );
    assert!(
        want.abs() > 1e-4,
        "the carrier is empty: this scene's three-body term is only {want:.3e} Ha, so the \
         dispatch could serve anything and the reading would not move"
    );
    assert_eq!(
        s.fence_untabulated, 0,
        "an (O,H,H) triple was fenced; it has a table"
    );

    // (O, O, H): NOT tabulated. Booked as nothing, counted as one.
    let mut f = scene();
    f.boundary = Boundary::Open;
    f.dims = Dims::Two;
    f.reset(3);
    assert!(f.set_species(0, OXYGEN));
    assert!(f.set_species(1, OXYGEN));
    assert!(f.set_species(2, HYDROGEN));
    let (cx, cy) = (0.5 * f.width, 0.5 * f.height);
    f.set_position(0, cx - 1.2, cy);
    f.set_position(1, cx + 1.2, cy);
    f.set_position(2, cx, cy + 1.8);
    f.rebase();
    f.step_frame(1);
    assert_eq!(
        f.e_three, 0.0,
        "an (O,O,H) triple booked {:.3e} Ha of three-body energy from a table that does \
         not exist",
        f.e_three
    );
    assert_eq!(
        f.fence_untabulated, 1,
        "the (O,O,H) fence was not counted; the prereg requires its incidence reported"
    );
}

fn dist(s: &Sim, i: usize, j: usize) -> f64 {
    let (a, b) = (&s.atoms[i], &s.atoms[j]);
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

#[test]
fn the_dispatch_changes_no_bit_of_a_pure_hydrogen_scene() {
    // The regression fence SATURATION-1 wrote for its own table, aimed at what THIS
    // campaign added. The triple loop now dispatches on composition; a pure-hydrogen
    // scene must come out the far side BIT-FOR-BIT identical to one run with the (O,H,H)
    // surface absent entirely, because the dispatch's H3 branch is the code that was
    // there before and the water branch is never taken.
    //
    // Bit-identity and not a tolerance: the water table contributes an exact zero to a
    // pure-H scene, and an exact zero added to a finite float changes no bit. Anything
    // looser would be measuring the tolerance.
    let mut with = pure_hydrogen_scene();
    let mut without = pure_hydrogen_scene();
    without.water = holon_chem::water::WaterTable::empty();
    assert!(with.water.loaded && !without.water.loaded);
    for _ in 0..40 {
        with.step_frame(16);
        without.step_frame(16);
    }
    assert!(
        with.e_three.abs() > 1e-6,
        "the carrier is empty: this scene's three-body term is {:.3e} Ha, so it is not \
         exercising the branch the dispatch had to keep",
        with.e_three
    );
    for i in 0..with.n {
        for (name, a, b) in [
            ("x", with.atoms[i].x, without.atoms[i].x),
            ("y", with.atoms[i].y, without.atoms[i].y),
            ("vx", with.atoms[i].vx, without.atoms[i].vx),
            ("vy", with.atoms[i].vy, without.atoms[i].vy),
        ] {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "atom {i}'s {name} moved when the (O,H,H) table was present: {a:.17e} \
                 against {b:.17e}. The composition dispatch is not free."
            );
        }
    }
    assert_eq!(
        with.e_three.to_bits(),
        without.e_three.to_bits(),
        "the three-body energy of a pure-hydrogen scene moved"
    );
    assert_eq!(
        with.drift_bound().to_bits(),
        without.drift_bound().to_bits(),
        "the drift bound of a pure-hydrogen scene moved: a loaded (O,H,H) table is \
         contributing stiffness to a scene with no oxygen in it"
    );
}

/// Eight hydrogens on a ring — SATURATION-1's own scene shape, so the branch the dispatch
/// had to keep is the one that is exercised.
fn pure_hydrogen_scene() -> Box<Sim> {
    let mut s = scene();
    s.boundary = Boundary::Walls;
    s.dims = Dims::Two;
    s.reset(8);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    for i in 0..8 {
        let th = i as f64 * core::f64::consts::TAU / 8.0;
        s.set_position(i, cx + 4.0 * th.cos(), cy + 4.0 * th.sin());
        let v = (K_B * 2000.0 / s.atoms[i].mass()).sqrt();
        s.set_velocity(i, -v * th.sin(), v * th.cos());
    }
    s.rebase();
    s
}

// ------------------------------------------------------------------ C1: energy

#[test]
fn c1_energy_gate_heteronuclear_nve() {
    let mut s = staked_water_nve();
    assert!(s.water.loaded, "the (O,H,H) table is not loaded");
    let peak_three = run(&mut s, 400, 64);
    assert!(
        peak_three > 1e-3,
        "the three-body term never exceeded {peak_three:.3e} Ha in this scene, so the gate \
         did not exercise the sector it claims to gate"
    );
    let bound = s.drift_bound();
    println!(
        "C1 energy (NVE, O + 2H): peak |E_three| = {peak_three:.6} Ha, drift_peak = \
         {:.3e}, bound = {bound:.3e}, ratio = {:.4}",
        s.drift_peak,
        s.drift_peak / bound
    );
    assert!(
        s.drift_peak <= bound,
        "C1 FIRED (energy): drift {:.3e} against a derived bound of {bound:.3e}",
        s.drift_peak
    );
}

#[test]
fn c1_energy_gate_mixed_thermostatted() {
    let mut s = staked_mixed_thermostat();
    let peak_three = run(&mut s, 300, 64);
    let bound = s.drift_bound();
    println!(
        "C1 energy (thermostat, 2 O + 4 H): peak |E_three| = {peak_three:.6} Ha, \
         drift_peak = {:.3e}, bound = {bound:.3e}, ratio = {:.4}, fenced triples = {}",
        s.drift_peak,
        s.drift_peak / bound,
        s.fence_untabulated
    );
    assert!(
        peak_three > 1e-4,
        "the three-body term never exceeded {peak_three:.3e} Ha; this scene did not \
         exercise the sector"
    );
    assert!(
        s.fence_untabulated > 0,
        "a box with two oxygens produced no (O,O,H) or (O,O,O) triples, so the fence's \
         incidence in this scene is not being counted"
    );
    assert!(
        s.drift_peak <= bound,
        "C1 FIRED (energy, thermostatted): drift {:.3e} against {bound:.3e}",
        s.drift_peak
    );
}

// ------------------------------------------------------------------ C1: momentum

#[test]
fn c1_momentum_gate_heteronuclear_nve() {
    // ONE GATE PER LAW. An open NVE box has no external impulse at all, so the momentum
    // residual is purely the internal forces failing to cancel — and the (O,H,H) triple's
    // three sides are applied equal-and-opposite by construction, so the correct reading
    // is roundoff, not a bound with room in it.
    let mut s = staked_water_nve();
    let peak_three = run(&mut s, 400, 64);
    let bound = s.momentum_bound();
    println!(
        "C1 momentum (NVE, O + 2H): peak |E_three| = {peak_three:.6} Ha, residual = \
         {:.3e}, bound = {bound:.3e}, ratio = {:.4}",
        s.momentum_residual_peak,
        s.momentum_residual_peak / bound
    );
    assert!(
        peak_three > 1e-3,
        "the sector was not exercised: peak |E_three| = {peak_three:.3e} Ha"
    );
    assert!(
        s.momentum_residual_peak <= bound,
        "C1 FIRED (momentum): residual {:.3e} against {bound:.3e}",
        s.momentum_residual_peak
    );
}

#[test]
fn c1_momentum_gate_mixed_thermostatted() {
    // The scene that has never existed before this campaign: two masses in one box, with
    // a thermostat doing external work on both. Oxygen is sixteen times hydrogen, so a
    // ledger that had quietly assumed one mass would show up here.
    let mut s = staked_mixed_thermostat();
    let peak_three = run(&mut s, 300, 64);
    let bound = s.momentum_bound();
    println!(
        "C1 momentum (thermostat, 2 O + 4 H): peak |E_three| = {peak_three:.6} Ha, \
         residual = {:.3e}, bound = {bound:.3e}, ratio = {:.4}",
        s.momentum_residual_peak,
        s.momentum_residual_peak / bound
    );
    assert!(
        s.momentum_residual_peak <= bound,
        "C1 FIRED (momentum, thermostatted): residual {:.3e} against {bound:.3e}",
        s.momentum_residual_peak
    );
}
