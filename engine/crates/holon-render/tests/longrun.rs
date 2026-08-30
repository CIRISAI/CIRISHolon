//! Long-run many-body gates — the permanent repro for the live GATE 1 ENERGY failure.
//!
//! Field report: `drift(peak) 5.627e-4` against `bound 4.920e-4` (114.4%) after a free run
//! to t = 5848 a.u. with N ~ 11, walls on, W_ext = 0 and four bonds formed by three-body
//! collisions; the momentum gate sat at roundoff, so the integrator was healthy and the
//! bound was the thing under suspicion.
//!
//! CONVICTED: the bound's amplitude factor. It used `|E_kin + E_pair + ...|`, the SIGNED
//! total, where the harmonic derivation calls for the sum of each mode's own energy. In a
//! bonded scene the kinetic and (negative) bond terms cancel almost exactly — which is
//! exactly the condition the gate is meant to police — so the amplitude the bound
//! multiplied by tracked the CONSERVED total while the oscillation amplitudes underneath
//! it grew. Measured up to 37x apart on the configuration that breached here.
//!
//! ACQUITTED, each by measurement rather than by inspection:
//!   * the wall's stiffness was already in the bound, and at `omega_wall = 1.65e-2` against
//!     `omega_env = 2.4e-1` it never binds;
//!   * the wall force is C1 (it goes to zero at contact) and the drift is NOT secular —
//!     `drift_peak` is flat to six digits from t = 6000 to t = 24000;
//!   * the curvature envelope was CONSERVATIVE, not stale (k_env 53.3 against a true
//!     visited 23.3), so under-derived curvature was not the cause either.
//!
//! The integrator was exonerated before the bound was touched: drift falls as dt^2
//! (measured ratios 4.688, 3.736, 4.378 against 4.0) and does not grow with time.

use holon_render::sim::{Boundary, Sim};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).expect("placeholder curve present")
}

/// N atoms on a ring of `radius`, at rest, in a walled box. Deterministic.
fn scene(n: usize, radius: f64) -> Box<Sim> {
    let mut s = Box::new(Sim::empty());
    holon_render::json::load_into(s.table_mut(), &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s.boundary = Boundary::Walls;
    s.reset(n);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    for i in 0..s.n {
        let th = (i as f64) * core::f64::consts::TAU / (s.n as f64);
        s.set_position(i, cx + radius * th.cos(), cy + radius * th.sin());
        s.set_velocity(i, 0.0, 0.0);
    }
    s.rebase();
    s
}

fn run_to(s: &mut Sim, t_end: f64) {
    while s.time < t_end {
        s.step_frame(64);
    }
}

#[test]
fn long_run_many_body_nve_holds_its_bound() {
    // THE REPRO. N = 14 on a radius-10 ring is the configuration that breached the old
    // bound at 109.6% — the same failure the field saw at 114.4% — and N = 11 is the
    // field's own atom count. Both run past the reported t = 5848 with walls on and no
    // external work, and both must sit inside the bound the UI displays.
    for (n, radius) in [(14usize, 10.0_f64), (11, 6.0), (11, 8.0), (16, 10.0)] {
        let mut s = scene(n, radius);
        run_to(&mut s, 6_200.0);

        let ratio = s.drift_peak / s.drift_bound();
        println!(
            "N = {n:>2}, radius {radius:>4.1}: t = {:.0}, drift {:.4e} / bound {:.4e} = {:.1}%, \
             molecules {}, W_ext {:.3e}",
            s.time,
            s.drift_peak,
            s.drift_bound(),
            100.0 * ratio,
            s.holons.molecule_count(),
            s.w_ext
        );
        // Free run: no ENERGY entered from outside. The walls are static and
        // conservative, so they do no work -- but they certainly deliver IMPULSE, and
        // `j_ext` is correctly non-zero here (measured (-60.0, 76.5, 0.0) on the first
        // configuration). That asymmetry is the whole reason these are two gates and not
        // one: a wall breaks translation invariance without breaking energy conservation,
        // so momentum is conserved only against the impulse ledger while energy is
        // conserved outright.
        assert_eq!(s.w_ext, 0.0, "a free run injected external work");
        assert!(
            s.j_ext.0 != 0.0 || s.j_ext.1 != 0.0,
            "walls are on but delivered no impulse at all, which means they never acted"
        );
        assert!(
            s.energy_gate(),
            "N = {n} radius {radius}: drift {:.4e} exceeds bound {:.4e} ({:.1}%)",
            s.drift_peak,
            s.drift_bound(),
            100.0 * ratio
        );
        assert!(
            s.momentum_gate(),
            "N = {n} radius {radius}: momentum ledger broke"
        );
    }
}

#[test]
fn bonds_form_in_the_long_run_so_the_repro_exercises_the_failing_condition() {
    // The field's scene had four bonds from three-body collisions. A repro that never
    // bonds would not reproduce it: bonds are precisely where the signed total cancels and
    // the old amplitude factor collapsed.
    let mut s = scene(14, 10.0);
    let mut ever = 0usize;
    while s.time < 6_200.0 {
        s.step_frame(64);
        ever = ever.max(s.holons.molecule_count());
    }
    println!(
        "N = 14: peak molecules {ever}, formations {}, dissolutions {}",
        s.holons.census.formations, s.holons.census.dissolutions
    );
    assert!(
        ever >= 2,
        "only {ever} molecules ever formed; the repro is not exercising bonds"
    );
}

#[test]
fn the_amplitude_factor_survives_kinetic_potential_cancellation() {
    // The defect, stated as a measurement. In a bonded many-body scene the signed total is
    // small because E_kin and E_pair cancel; the mode energies are not.
    let mut s = scene(11, 6.0);
    run_to(&mut s, 6_200.0);

    let signed = s.energy().abs();
    let modes = s.mode_energy();
    println!(
        "signed |E| = {signed:.4} Eh   mode energy = {modes:.4} Eh   ratio = {:.1}x   \
         (E_kin {:.4}, E_pair {:.4})",
        modes / signed.max(1e-12),
        s.e_kin,
        s.e_pair
    );
    assert!(
        modes > 4.0 * signed,
        "this scene does not exhibit the cancellation the fix addresses ({modes} vs {signed})"
    );
    // The bound must be built from the mode energy, not the signed total.
    assert!(
        s.e_ref >= modes * 0.999,
        "e_ref {:.4} is below the mode energy {modes:.4} it is supposed to bound",
        s.e_ref
    );
    assert!(s.energy_gate());
}

#[test]
fn drift_does_not_grow_secularly() {
    // Velocity Verlet is symplectic: its energy error oscillates and does not accumulate.
    // If this ever fails, no per-mode bound covers the run and the INTEGRATOR is at fault
    // rather than the bound — which is the distinction the whole diagnosis turned on.
    let mut s = scene(11, 6.0);
    run_to(&mut s, 6_000.0);
    let early = s.drift_peak;
    run_to(&mut s, 24_000.0);
    let late = s.drift_peak;
    println!(
        "drift_peak at t = 6000: {early:.6e};  at t = 24000: {late:.6e};  growth {:.4}x over 4x the time",
        late / early
    );
    assert!(
        late <= early * 1.10,
        "drift grew {:.3}x between t = 6000 and t = 24000: this is secular, and a bound \
         derived per-mode cannot cover it",
        late / early
    );
    assert!(s.energy_gate());
}

#[test]
fn a_planted_energy_leak_still_trips_the_energy_gate() {
    // The corrected bound is WIDER, so this is the check that it is still a gate. A purely
    // non-conservative velocity rescale that is never posted to `w_ext` is exactly the
    // class of defect the ledger exists to catch, and it is planted here rather than
    // hoped for.
    //
    // Measured sensitivity on this scene (examples/gate_scaling.rs, part C): 1e-5 per
    // frame reads 11.1% of bound and PASSES; 1e-4 per frame reads 114.7% and FAILS. Over
    // the ~209 frames of this run, 1e-4 per frame is a cumulative 4.3% kinetic-energy
    // injection. That is the gate's floor, and it is a real cost of the correction — the
    // previous bound was tighter but FALSE-ALARMED on valid physics, which is not a
    // usable gate at any sensitivity.
    let clean = {
        let mut s = scene(11, 6.0);
        run_to(&mut s, 6_000.0);
        assert!(s.energy_gate(), "the control run must pass");
        s.drift_peak / s.drift_bound()
    };

    let mut s = scene(11, 6.0);
    while s.time < 6_000.0 {
        s.step_frame(64);
        for i in 0..s.n {
            let a = s.atoms[i];
            s.set_velocity(i, a.vx * (1.0 + 1e-4), a.vy * (1.0 + 1e-4));
        }
    }
    let leaked = s.drift_peak / s.drift_bound();
    println!(
        "control {:.1}% of bound (PASS);  with a 1e-4/frame unposted leak {:.1}% (gate {})",
        100.0 * clean,
        100.0 * leaked,
        if s.energy_gate() { "PASS" } else { "FAIL" }
    );
    assert!(
        !s.energy_gate(),
        "a planted energy leak did not trip the gate: {:.1}% of bound. The bound is too wide \
         to be a gate.",
        100.0 * leaked
    );
}
