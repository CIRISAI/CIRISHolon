//! Gates for the three amendments: the derived timescale, the composite-holon layer, and
//! the three fences.

use holon::tune::{Degrade, Hold, Policy, PolicyError};
use holon_render::clock::{n_max, Rung, AU_TO_FS, STEPS_PER_PERIOD};
use holon_render::holon::{CLOSURE_DEFECT_MAX, DWELL_K};
use holon_render::sim::{Boundary, Sim, M_H};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).expect("placeholder curve present")
}

fn loaded_sim() -> Sim {
    let mut s = Sim::empty();
    holon_render::json::load_into(&mut s.table, &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

// ================================================================ the three clocks

#[test]
fn dt_is_derived_from_the_curve_not_chosen() {
    let s = loaded_sim();
    let mu = 0.5 * M_H;
    let k_e = s.table.curvature(s.table.r_e).abs();
    let omega_e = (k_e / mu).sqrt();

    println!(
        "omega_e = {:.6e} a.u. ({:.1} cm^-1), period = {:.3} a.u. = {:.4} fs",
        s.timescale.omega_e,
        s.timescale.omega_e / 4.556335e-6,
        s.timescale.period,
        s.timescale.period * AU_TO_FS
    );
    println!(
        "dt_reference = period/{STEPS_PER_PERIOD} = {:.4} a.u.;  dt in force = {:.4} a.u.",
        s.timescale.dt_reference,
        s.dt()
    );
    assert!(
        (s.timescale.omega_e - omega_e).abs() < 1e-12,
        "omega was not read off the table's own curvature"
    );
    let expect_dt_ref = (core::f64::consts::TAU / omega_e) / STEPS_PER_PERIOD;
    assert!(
        (s.timescale.dt_reference - expect_dt_ref).abs() < 1e-9,
        "dt_reference is not period/{STEPS_PER_PERIOD}"
    );
    // At the well bottom the reference step meets the accuracy target by construction.
    assert!((omega_e * s.timescale.dt_reference - s.timescale.accuracy_target()).abs() < 1e-12);
}

#[test]
fn default_sim_speed_makes_one_vibration_watchable() {
    let s = loaded_sim();
    let period_fs = s.timescale.period * AU_TO_FS;
    let wall_seconds = period_fs / s.timescale.sim_speed_fs_per_wallsec;
    println!(
        "period = {period_fs:.4} fs; default sim-speed = {:.4} fs/wall-s => one vibration in \
         {wall_seconds:.2} wall-seconds",
        s.timescale.sim_speed_fs_per_wallsec
    );
    assert!(
        (wall_seconds - 2.0).abs() < 1e-9,
        "one vibration takes {wall_seconds} s"
    );
}

#[test]
fn the_bound_uses_the_curvature_envelope_not_the_equilibrium_curvature() {
    // FENCE 3. A pair with enough energy to climb the repulsive wall can reach curvature
    // far above the well's, and a bound derived from E''(R_e) alone reads green straight
    // through the collision that violates it.
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    s.set_position(0, cx - 4.0, cy);
    s.set_position(1, cx + 4.0, cy);
    s.set_velocity(0, 0.006, 0.0);
    s.set_velocity(1, -0.006, 0.0);
    s.rebase();

    let omega_e = s.timescale.omega_e;
    let omega_env = s.timescale.omega_env;
    let k_e = s.table.curvature(s.table.r_e).abs();
    println!(
        "e_rel_max = {:.4e} Eh -> inner turning point {:.4} bohr",
        s.timescale.e_rel_max, s.timescale.r_inner
    );
    println!(
        "k(R_e) = {k_e:.4}  k_env = {:.4}  (ratio {:.2}x);  omega_e = {omega_e:.4e}  \
         omega_env = {omega_env:.4e}  (ratio {:.2}x)",
        s.timescale.k_env,
        s.timescale.k_env / k_e,
        omega_env / omega_e
    );
    println!(
        "dt_reference = {:.4} -> dt refined to {:.4} (x{:.3});  omega_env*dt = {:.4} vs target {:.4}",
        s.timescale.dt_reference,
        s.dt(),
        s.dt() / s.timescale.dt_reference,
        s.timescale.omega_dt(),
        s.timescale.accuracy_target()
    );
    assert!(
        omega_env > omega_e,
        "the envelope did not exceed the equilibrium frequency for a pair that can reach the wall"
    );
    // HOLD = EXACTNESS: dt was refined so the accuracy target is still met.
    assert!(
        s.timescale.omega_dt() <= s.timescale.accuracy_target() * (1.0 + 1e-9),
        "omega_env*dt = {} exceeds the target {}",
        s.timescale.omega_dt(),
        s.timescale.accuracy_target()
    );
    assert!(s.dt() < s.timescale.dt_reference, "dt was not refined");

    // And the run holds its (envelope-derived) bound through the actual collision.
    for _ in 0..4_000 {
        s.step_frame(8);
    }
    println!(
        "through the collision: drift_peak = {:.3e}, bound = {:.3e}, ratio = {:.4}",
        s.drift_peak,
        s.drift_bound(),
        s.drift_peak / s.drift_bound()
    );
    assert!(
        s.energy_gate(),
        "the envelope bound did not survive the collision"
    );
}

#[test]
fn the_bound_is_never_stale_when_dt_moves() {
    // "A changed dt with a stale bound is a defect." There is no stored bound to go
    // stale: it is recomputed from the live dt and the live envelope on every read.
    let mut s = loaded_sim();
    s.reset(2);
    let before = s.timescale.relative_drift_bound();
    let dt_before = s.dt();

    // The multiplier is a multiple of the DERIVED REFERENCE, not of whatever the envelope
    // has refined dt down to. That is the point of the toggle: it says "run at the
    // reference step (or coarser) even though the envelope wants finer", which is exactly
    // the accuracy the user is choosing to give up.
    s.timescale.allow_dt_growth = true;
    s.timescale.set_dt_multiplier(2.0);
    let after = s.timescale.relative_drift_bound();
    let dt_ratio = s.dt() / dt_before;
    println!(
        "dt {dt_before:.4} -> {:.4} (reference {:.4}, x{dt_ratio:.3}): relative bound \
         {before:.4e} -> {after:.4e} (x{:.3})",
        s.dt(),
        s.timescale.dt_reference,
        after / before
    );
    assert!(
        (s.dt() - 2.0 * s.timescale.dt_reference).abs() < 1e-9,
        "the multiplier is not relative to the derived reference"
    );
    // The bound goes as (omega dt)^2, so it must track the dt change quadratically.
    assert!(
        (after / before - dt_ratio * dt_ratio).abs() / (dt_ratio * dt_ratio) < 1e-9,
        "the bound did not track dt: {after} / {before} against a dt ratio of {dt_ratio}"
    );
}

#[test]
fn accuracy_cannot_be_degraded_under_an_exactness_hold() {
    // The amendment's "declaredly, never silently" is enforced by the engine's own
    // constructor, not by our care: this configuration cannot be built.
    let refused = Policy::new(Hold::Exactness, vec![Degrade::Accuracy { eps: 1e-3 }]);
    assert_eq!(refused.err(), Some(PolicyError::AccuracyUnderExactness));

    let mut s = loaded_sim();
    s.reset(2);
    // Default: exactness held, latency degrades without limit.
    assert!(s.timescale.policy(16.7).is_ok());
    assert_eq!(s.timescale.policy(16.7).unwrap().hold, Hold::Exactness);
    // Toggle on: a DIFFERENT policy, holding latency and degrading accuracy declaredly.
    s.timescale.allow_dt_growth = true;
    let p = s.timescale.policy(16.7).unwrap();
    assert_eq!(p.hold, Hold::Latency { budget_ms: 16.7 });
    assert!(matches!(p.degrade[0], Degrade::Accuracy { .. }));
}

#[test]
fn shortfall_dilates_time_and_leaves_dt_alone() {
    // Rung (i): the device cannot deliver the requested sim-speed. Time dilates; the
    // timestep, and therefore the accuracy contract, does not move.
    let mut s = loaded_sim();
    s.reset(2);
    let dt_before = s.dt();
    let bound_before = s.timescale.relative_drift_bound();

    // At the DEFAULT sim-speed one 60 Hz frame asks for only a couple of substeps, so the
    // shortfall has to be manufactured: ask for fast dynamics, then starve the budget.
    s.timescale.sim_speed_fs_per_wallsec *= 200.0;
    let plan = s.timescale.plan_frame(1.0 / 60.0, 3);
    println!(
        "budget 3 substeps: took {}, dilation {:.4}, rung {:?}",
        plan.substeps, plan.dilation, plan.rung
    );
    assert_eq!(plan.substeps, 3, "the budget was not respected");
    assert_eq!(plan.rung, Rung::TimeDilated);
    assert!(plan.dilation < 1.0, "a shortfall reported no dilation");
    assert_eq!(s.dt(), dt_before, "time dilation changed the timestep");
    assert_eq!(
        s.timescale.relative_drift_bound(),
        bound_before,
        "time dilation changed the accuracy contract"
    );

    // With a budget that covers the demand, nothing gives.
    let plan = s.timescale.plan_frame(1.0 / 60.0, u32::MAX);
    assert_eq!(plan.rung, Rung::Exact);
    assert_eq!(plan.dilation, 1.0);
    assert!(
        plan.substeps > 3,
        "the unstarved frame took no more than the starved one"
    );
}

#[test]
fn the_accumulator_carries_its_remainder_and_never_stretches_dt() {
    // Substeps are whole; the leftover simulated time is carried, not rounded away, and
    // dt is never rescaled to make a frame come out even.
    let mut s = loaded_sim();
    s.reset(2);
    let dt = s.dt();
    // A wall interval deliberately not commensurate with dt.
    let wall = 0.0071;
    let mut total = 0u64;
    for _ in 0..500 {
        let plan = s.timescale.plan_frame(wall, u32::MAX);
        total += plan.substeps as u64;
        assert_eq!(s.dt(), dt, "dt was stretched to fit a frame");
    }
    let expected = (s.timescale.sim_speed_fs_per_wallsec * wall * 500.0 / AU_TO_FS) / dt;
    println!(
        "500 frames of {wall}s: {total} substeps taken, {expected:.2} owed (deficit {:.3} steps)",
        expected - total as f64
    );
    // Carrying the remainder means the running total tracks the owed time to under one
    // step, forever. Rounding it away instead would drift without bound.
    assert!(
        (expected - total as f64).abs() < 1.0,
        "the accumulator lost time: {} vs {expected}",
        total
    );
}

#[test]
fn a_stalled_frame_reports_its_lost_time_as_dilation() {
    // A backgrounded tab hands back a multi-second interval. Honouring it would produce a
    // catch-up burst that stalls the page again, so the interval is capped -- and the cap
    // is reported, because sim-time dropped on the floor is the same quiet clock-rewriting
    // that silent substep-dropping would be.
    let mut s = loaded_sim();
    s.reset(2);
    let plan = s.timescale.plan_frame(4.0, u32::MAX);
    println!(
        "a 4.000 s stall (cap {:.3} s): substeps {}, dilation {:.4}, rung {:?}",
        holon_render::clock::MAX_FRAME_SECONDS,
        plan.substeps,
        plan.dilation,
        plan.rung
    );
    assert_eq!(
        plan.rung,
        Rung::TimeDilated,
        "a capped stall reported as exact"
    );
    assert!(
        (plan.dilation - holon_render::clock::MAX_FRAME_SECONDS / 4.0).abs() < 1e-9,
        "the reported dilation does not match the time actually dropped"
    );

    // An ordinary frame is not capped and reports no dilation.
    let plan = s.timescale.plan_frame(1.0 / 60.0, u32::MAX);
    assert_eq!(plan.dilation, 1.0);
    assert_eq!(plan.rung, Rung::Exact);
}

// ================================================================ capacity

#[test]
fn n_max_solves_the_pair_budget_exactly() {
    // N(N-1)/2 pairs per substep must fit the measured pair throughput.
    for (pairs_per_sec, required) in [(1.0e8, 1.0e4), (1.0e7, 1.0e4), (1.0e9, 5.0e3)] {
        let n = n_max(pairs_per_sec, required);
        let p = pairs_per_sec / required;
        let used = n * (n - 1.0) / 2.0;
        let next = (n + 1.0) * n / 2.0;
        println!(
            "  P = {p:.1} affordable pairs/substep -> N_max = {n}  (uses {used}, N+1 needs {next})"
        );
        assert!(used <= p, "N_max does not fit the budget");
        assert!(next > p, "N_max is not maximal");
    }
    // ATOMWORLD.md banks N_max ~ sqrt(P); the exact answer is asymptotically sqrt(2P).
    let p: f64 = 1.0e6;
    let exact = n_max(p * 1.0e4, 1.0e4);
    println!(
        "banked sqrt(P) = {:.0} vs exact {:.0} (ratio {:.4}, i.e. sqrt(2))",
        p.sqrt(),
        exact,
        exact / p.sqrt()
    );
    assert!((exact / p.sqrt() - 2.0f64.sqrt()).abs() < 0.01);
}

#[test]
fn perf_substeps_per_second_native() {
    // The native numbers the device classes get projected from.
    use std::time::Instant;
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(holon_render::sim::MAX_ATOMS);
    let pairs = (s.n * (s.n - 1) / 2) as f64;

    // Warm up, then measure.
    for _ in 0..2_000 {
        s.step();
    }
    let n = 40_000;
    let t0 = Instant::now();
    for _ in 0..n {
        s.step();
    }
    let elapsed = t0.elapsed().as_secs_f64();
    let sps = n as f64 / elapsed;
    let pps = sps * pairs;
    s.timescale.substeps_per_second = sps;
    s.timescale.calibrated = true;

    let required = s.timescale.required_substeps_per_second();
    let nmax = n_max(pps, required);
    println!(
        "PERF (native, N = {}, {pairs} pairs, O(N^2) exact table):",
        s.n
    );
    println!("  substeps/sec = {sps:.3e}");
    println!("  pairs/sec    = {pps:.3e}");
    println!(
        "  dt = {:.4} a.u.; sim-speed {:.4} fs/wall-s => required {required:.3e} substeps/sec",
        s.dt(),
        s.timescale.sim_speed_fs_per_wallsec
    );
    println!("  N_max at that sim-speed on THIS machine = {nmax}");
    assert!(sps > 1.0e4, "suspiciously slow: {sps:.3e} substeps/sec");
}

#[test]
fn the_census_layer_cost_is_measured_not_asserted() {
    // "Being matter is expensive, being a holon is cheap" has to be a number.
    //
    // METHOD, and why it is not the obvious one: the first version of this benchmark took
    // one timing of each arm and divided. On this (shared, contended) machine that read
    // +16.42%, -16.36% and -1.86% on three consecutive runs — and a NEGATIVE overhead is
    // not a small error, it is proof the instrument was measuring the machine rather than
    // the code. So: many short PAIRED samples, and the MINIMUM of each arm rather than the
    // mean. Contention can only ever make a sample slower, so the minimum is the sample
    // least contaminated by it, while a mean is dragged around by whatever else the box is
    // doing. The spread is printed alongside so the reader can see the noise the minimum
    // is protecting against.
    use std::time::Instant;

    let sample = |enabled: bool, scene_hot: bool| -> f64 {
        let mut s = loaded_sim();
        s.boundary = Boundary::Open;
        s.reset(holon_render::sim::MAX_ATOMS);
        if scene_hot {
            // A realistic scene: most pairs unbound, so the layer has little to resolve.
            for i in 0..s.n {
                let a = 0.9 * i as f64;
                s.set_velocity(i, 0.004 * a.cos(), 0.004 * a.sin());
            }
            s.rebase();
        }
        s.holons.enabled = enabled;
        for _ in 0..20 {
            s.step_frame(64);
        }
        let t0 = Instant::now();
        for _ in 0..60 {
            s.step_frame(64);
        }
        t0.elapsed().as_secs_f64() / 60.0
    };

    for (label, hot) in [
        ("at rest (every pair bound: worst case)", false),
        ("hot (most pairs unbound)", true),
    ] {
        let mut on = Vec::new();
        let mut off = Vec::new();
        for _ in 0..15 {
            on.push(sample(true, hot));
            off.push(sample(false, hot));
        }
        let min = |v: &Vec<f64>| v.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = |v: &Vec<f64>| v.iter().cloned().fold(0.0f64, f64::max);
        let (mon, moff) = (min(&on), min(&off));
        let overhead = (mon - moff) / moff;
        println!("frame cost, N = 16 (120 pairs), 64 substeps/frame, {label}:");
        println!(
            "  composite layer ON  = {:.4} ms/frame  (spread {:.4}-{:.4})",
            mon * 1e3,
            mon * 1e3,
            max(&on) * 1e3
        );
        println!(
            "  composite layer OFF = {:.4} ms/frame  (spread {:.4}-{:.4})",
            moff * 1e3,
            moff * 1e3,
            max(&off) * 1e3
        );
        println!(
            "  census overhead     = {:+.2}% of the frame",
            100.0 * overhead
        );
        assert!(
            overhead < 0.05,
            "the composite layer costs {:.1}% of a frame, which is not 'cheap'",
            100.0 * overhead
        );
    }
}

// ================================================================ the capture plant

#[test]
fn capture_plant_an_isolated_pair_never_forms_a_molecule() {
    // FENCE 2. An isolated two-body system with W_ext = 0 conserves its pair energy, so
    // capture is impossible: there is no channel to carry the surplus away. A bond formed
    // here convicts the integrator or the predicate.
    //
    // "Many encounters" for a two-body system means an ENSEMBLE: one isolated pair gets
    // exactly one encounter before separating forever, so the sweep runs many independent
    // ones across approach speed and impact parameter (hence angular momentum), which is
    // a broader plant than one long trajectory could be.
    let mut worst_e_rel = f64::INFINITY;
    let mut closest = f64::INFINITY;
    let mut runs = 0;
    for speed_step in 0..6 {
        for impact_step in 0..5 {
            let v = 0.0015 + 0.0015 * speed_step as f64;
            let b = 0.4 * impact_step as f64;
            let mut s = loaded_sim();
            s.boundary = Boundary::Open;
            s.reset(2);
            let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
            s.set_position(0, cx - 5.0, cy - 0.5 * b);
            s.set_position(1, cx + 5.0, cy + 0.5 * b);
            s.set_velocity(0, v, 0.0);
            s.set_velocity(1, -v, 0.0);
            s.rebase();
            runs += 1;

            for _ in 0..2_500 {
                s.step_frame(8);
                closest = closest.min(s.pairs[0].r);
                worst_e_rel = worst_e_rel.min(s.pairs[0].e_rel);
                assert_eq!(s.w_ext, 0.0, "the plant is not isolated: W_ext moved");
                assert_eq!(
                    s.holons.molecule_count(),
                    0,
                    "CAPTURE PLANT VIOLATED: a molecule formed at v = {v}, b = {b}, \
                     R = {:.4}, E_rel = {:.4e}",
                    s.pairs[0].r,
                    s.pairs[0].e_rel
                );
            }
            assert_eq!(
                s.holons.census.formations, 0,
                "a row was created and destroyed"
            );
        }
    }
    println!(
        "capture plant: {runs} isolated encounters, closest approach {closest:.4} bohr, \
         lowest E_rel {worst_e_rel:.4e} Eh, molecules formed: 0"
    );
    assert!(
        closest < 1.2,
        "the ensemble never actually collided (min R = {closest})"
    );
    assert!(
        worst_e_rel >= 0.0,
        "an isolated pair went below the asymptote"
    );
}

// ================================================================ composite holons

/// Drive the scripted push to the moment a molecule appears, returning the sim.
///
/// Binding is achieved by NEGATIVE spring work: the user grabs one atom at closest
/// approach, holds while the pair separates (the spring loads at the atoms' expense), and
/// releases once the pair is bound, carrying the stored energy out of the scene. That
/// extraction is the third-body channel — the capture plant above shows there is no other.
fn form_a_molecule() -> Sim {
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    s.set_position(0, cx - 4.0, cy);
    s.set_position(1, cx + 4.0, cy);
    s.set_velocity(0, 0.002, 0.0);
    s.set_velocity(1, -0.002, 0.0);
    s.rebase();

    let mut last = f64::INFINITY;
    loop {
        s.step_frame(8);
        if s.pairs[0].r > last {
            break;
        }
        last = s.pairs[0].r;
    }
    s.grab(0);
    let mut held = 0;
    while s.pairs[0].e_rel >= 0.0 {
        s.step_frame(8);
        held += 1;
        assert!(held < 20_000, "never became bound");
    }
    s.release();
    assert!(
        s.w_ext < 0.0,
        "binding was not achieved by NEGATIVE external work"
    );
    s
}

#[test]
fn formation_is_accounting_only_and_the_global_ledger_is_untouched() {
    // FENCE 2, first half. Creating a row redistributes ledger LABELS; it must not touch
    // the dynamical state. Asserted BIT-IDENTICALLY, because "close enough" is exactly
    // the gap a leak would hide in.
    let mut s = form_a_molecule();
    let mut caught = false;
    for _ in 0..400 {
        for _ in 0..8 {
            s.step();
        }
        let before_molecules = s.holons.molecule_count();
        let e_before = s.energy();
        let l_before = s.ledger();
        let w_before = s.w_ext;
        let p_before = s.momentum();

        s.close_grain();

        let e_after = s.energy();
        assert_eq!(
            e_before.to_bits(),
            e_after.to_bits(),
            "closing the grain moved the energy: {e_before} -> {e_after}"
        );
        assert_eq!(
            l_before.to_bits(),
            s.ledger().to_bits(),
            "the global ledger moved"
        );
        assert_eq!(w_before.to_bits(), s.w_ext.to_bits(), "external work moved");
        assert_eq!(p_before, s.momentum(), "momentum moved");

        if s.holons.molecule_count() > before_molecules {
            caught = true;
            let row = s.holons.live_rows().next().unwrap().1;
            println!(
                "FORMATION at frame {}, t = {:.1} a.u.: members {:?}, e_bond = {:.6e} Eh, \
                 closure defect = {:.3e} (max {CLOSURE_DEFECT_MAX:.0e})",
                row.formed_at_frame,
                row.formed_at_time,
                &row.members[..row.member_count as usize],
                row.e_bond,
                row.closure_defect_at_formation
            );
            println!("  E before = {e_before:.12}  E after = {e_after:.12}  (bit-identical)");
            assert!(
                row.closure_defect_at_formation <= CLOSURE_DEFECT_MAX,
                "a row formed with an unbounded closure defect"
            );
            break;
        }
    }
    assert!(caught, "no formation event was observed");
    assert_eq!(s.holons.molecule_count(), 1);
    assert_eq!(s.holons.census.molecules, 1);
    // The bond-sector row is a VIEW of energy the global ledger already holds.
    let row_energy = s.holons.bond_sector_energy();
    println!(
        "bond-sector row = {row_energy:.6e} Eh; pair E_rel = {:.6e} Eh",
        s.pairs[0].e_rel
    );
    assert!((row_energy - s.pairs[0].e_rel).abs() < 1e-12);
    assert!(s.energy_gate(), "the energy gate broke across formation");
}

#[test]
fn dissociation_returns_every_joule_and_the_defect_rises() {
    // Pull the molecule apart through the spring and check the books close.
    let mut s = form_a_molecule();
    for _ in 0..200 {
        s.step_frame(8);
    }
    assert_eq!(s.holons.molecule_count(), 1, "no molecule to dissociate");
    let row = *s.holons.live_rows().next().unwrap().1;
    let defect_at_formation = row.closure_defect_at_formation;
    let l_before = s.ledger();

    // Grab one member and haul it away.
    s.grab(0);
    let mut pulled = 0;
    while s.holons.molecule_count() > 0 && pulled < 20_000 {
        let a = &s.atoms[1];
        let (bx, by) = (a.x, a.y);
        // Drag the anchor steadily away from the partner: one anchor move per frame, held
        // constant across the frame's substeps.
        let dx = s.atoms[0].x - bx;
        let dy = s.atoms[0].y - by;
        let norm = (dx * dx + dy * dy).sqrt().max(1e-9);
        s.move_anchor(bx + dx / norm * 14.0, by + dy / norm * 14.0);
        s.step_frame(8);
        pulled += 1;
    }
    s.release();
    s.step_frame(8);

    println!(
        "DISSOLUTION after {pulled} frames: formations = {}, dissolutions = {}",
        s.holons.census.formations, s.holons.census.dissolutions
    );
    // Read from the layer, not from the snapshot taken before the pull: `row` is a COPY
    // made while the molecule was still quietly bound, so its defect field describes that
    // moment and not this one. Reporting it as "at dissolution" would have been a lie the
    // assertion below would not have caught.
    println!(
        "  closure defect at formation = {defect_at_formation:.3e} (snapshot before the pull: \
         {:.3e}), at dissolution = {:.3e}, rose by {:.1}x",
        row.closure_defect,
        s.holons.last_dissolution_defect,
        s.holons.last_dissolution_defect
            / s.holons.last_dissolution_defect_at_formation.max(1e-300)
    );
    assert!(
        s.holons.last_dissolution_defect > s.holons.last_dissolution_defect_at_formation,
        "the closure defect did not rise across dissolution: {:.3e} -> {:.3e}",
        s.holons.last_dissolution_defect_at_formation,
        s.holons.last_dissolution_defect
    );
    println!(
        "  ledger before = {l_before:.12}, after = {:.12}, W_ext = {:.6}",
        s.ledger(),
        s.w_ext
    );
    assert_eq!(
        s.holons.molecule_count(),
        0,
        "the molecule did not dissolve"
    );
    assert_eq!(s.holons.census.dissolutions, 1);
    // Every joule returned: E - W_ext is where it started, within the derived bound.
    assert!(
        s.energy_gate(),
        "dissociation leaked: drift {:.3e} exceeds bound {:.3e}",
        s.drift_peak,
        s.drift_bound()
    );
    assert!(s.momentum_gate(), "dissociation broke the momentum ledger");
    // FENCE 1, second half: losing closure is what dissolution IS.
    assert_eq!(
        s.holons.dissolutions_without_defect_rise, 0,
        "a molecule dissolved without its closure defect rising"
    );
}

#[test]
fn dwell_hysteresis_is_deterministic_and_the_census_matches_ground_truth() {
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(3);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    // Three atoms at rest. Every pair is bound (a pair at rest has E_rel = U(R) < 0), so
    // all three are eligible and the canonical rule has to decide between them.
    s.set_position(0, cx - 0.7, cy);
    s.set_position(1, cx + 0.7, cy);
    s.set_position(2, cx + 5.3, cy);
    for i in 0..3 {
        s.set_velocity(i, 0.0, 0.0);
    }
    s.rebase();

    // No row may exist before the dwell is satisfied. The first boundary cannot form one
    // in any case: the closure defect needs a previous boundary to be measured against.
    // K consecutive satisfied boundaries means the row appears AT the Kth, so boundaries
    // 1..K-1 must be empty. Boundary 1 could not form one in any case: the closure defect
    // needs a previous boundary to be measured against, so it reads infinite there.
    for boundary in 1..=(DWELL_K as usize + 2) {
        s.step_frame(1);
        let live = s.holons.molecule_count();
        println!(
            "  boundary {boundary}: molecules = {live}, pair(0,1) bonded = {}, e_rel = {:.4e}",
            s.pairs[0].bonded, s.pairs[0].e_rel
        );
        if boundary < DWELL_K as usize {
            assert_eq!(
                live, 0,
                "a row formed after only {boundary} boundaries (K = {DWELL_K})"
            );
        }
    }
    assert!(
        s.holons.molecule_count() >= 1,
        "the dwell never let anything form"
    );

    // GROUND TRUTH, checked structurally rather than by re-deriving the layer:
    //  - the census count equals the live row count;
    //  - no atom belongs to two composites;
    //  - every live row's members are a pair the predicate currently calls bonded.
    let mut seen = [false; holon_render::sim::MAX_ATOMS];
    let mut counted = 0;
    for (_, row) in s.holons.live_rows() {
        counted += 1;
        for m in &row.members[..row.member_count as usize] {
            assert!(!seen[*m as usize], "atom {m} is in two composites at once");
            seen[*m as usize] = true;
        }
        let (a, b) = (row.members[0] as usize, row.members[1] as usize);
        let p = s.pairs[..s.pair_count]
            .iter()
            .find(|p| (p.i, p.j) == (a.min(b), a.max(b)))
            .expect("row names a pair that does not exist");
        assert!(
            p.bonded,
            "a live row names a pair the predicate calls unbound"
        );
    }
    assert_eq!(counted, s.holons.molecule_count());
    assert_eq!(s.holons.census.molecules, counted);
    assert_eq!(s.holons.census.atoms, 3);
    assert_eq!(
        s.holons.census.candidate_evaluations, 3,
        "3 atoms is 3 candidate pairs"
    );
    assert_eq!(s.holons.census.global_views, 3);

    // CANONICAL RESOLUTION: the most-bound pair claims its atoms. (0,1) at 1.4 bohr is
    // far more bound than (1,2) at 4.6, so it wins and (1,2) and (0,2) are blocked.
    let row = s.holons.live_rows().next().unwrap().1;
    println!(
        "  canonical winner: members {:?}, e_bond = {:.4e} Eh",
        &row.members[..2],
        row.e_bond
    );
    assert_eq!(
        &row.members[..2],
        &[0u8, 1u8],
        "the most-bound pair did not win"
    );
    assert_eq!(counted, 1, "an atom was claimed twice");
}

#[test]
fn a_pair_held_by_the_spring_is_bound_but_not_closed() {
    // FENCE 1, first half. The energy threshold proves a bound pair; it does not prove an
    // autonomous molecular view. A pair being driven by the user's spring can satisfy the
    // energy criterion while its own one-step closure defect is large, and the layer must
    // refuse it a row on that basis and COUNT the refusal.
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(2);
    let (cx, cy) = (0.5 * s.width, 0.5 * s.height);
    s.set_position(0, cx - 0.7, cy);
    s.set_position(1, cx + 0.7, cy);
    s.set_velocity(0, 0.0, 0.0);
    s.set_velocity(1, 0.0, 0.0);
    s.rebase();
    s.grab(0);

    let mut max_defect: f64 = 0.0;
    for k in 0..200 {
        // Shake the grabbed atom hard enough that the pair's bond energy cannot sit still.
        let theta = k as f64 * 0.6;
        s.move_anchor(cx - 0.7 + 2.5 * theta.cos(), cy + 2.5 * theta.sin());
        s.step_frame(8);
        max_defect = max_defect.max(s.holons.candidates[0].closure_defect.min(1e9));
    }
    println!(
        "driven pair: bonded = {}, E_rel = {:.4e}, peak closure defect = {max_defect:.3e} \
         (threshold {CLOSURE_DEFECT_MAX:.0e}), molecules = {}, closure rejections = {}",
        s.pairs[0].bonded,
        s.pairs[0].e_rel,
        s.holons.molecule_count(),
        s.holons.census.closure_rejections
    );
    assert!(
        max_defect > CLOSURE_DEFECT_MAX,
        "the driven pair never showed an open view: {max_defect:.3e}"
    );
    assert!(
        s.holons.census.closure_rejections > 0,
        "boundness and closure never disagreed, so the gate was never exercised"
    );
}
