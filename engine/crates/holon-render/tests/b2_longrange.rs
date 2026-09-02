//! B2 — the long-range sector's invariants, at suite cost.
//!
//! The measured battery lives in `examples/b2_longrange.rs` and is priced in curve
//! generation (the O–O solve alone was 961 s of CPU in B1b). What is here is everything
//! that can be checked WITHOUT a solve: the refusals, the construction guarantees the
//! conservation gates rest on, and the two claims this subsystem makes about not disturbing
//! anything that existed before it.
//!
//! The curves are synthetic power laws, and that is honest rather than convenient: these
//! tests are about the SECTOR's arithmetic and refusals, not about any table's physics.
//! Where a test would need a real curve to mean anything, it is not here — it is a gate in
//! the instrument, run against the committed tables.

use holon_render::cells::BoxGeom;
use holon_render::longrange::{
    BoxKey, CurveTail, FarPlant, FarRefusal, FarSector, TailBand, SHELL_CAP,
};
use holon_render::sim::{Boundary, Dims, Sim};

/// A synthetic `−r^(−p)` curve with `n` knots out to `r_max`, carrying a full disclosure
/// record so R5 does not fire on tests that are about something else.
fn power(p: f64, r_max: f64) -> CurveTail {
    let n = 40usize;
    let r: Vec<f64> = (0..n)
        .map(|k| 4.0 + (r_max - 4.0) * k as f64 / (n - 1) as f64)
        .collect();
    let u: Vec<f64> = r.iter().map(|x| -x.powf(-p)).collect();
    CurveTail {
        hi_b: p / r_max,
        r,
        u,
        solver_exit: "Converged",
        solver_budget_iterations: 5000,
        uncertainty_hartree: 1.0e-11,
    }
}

fn open_geom() -> BoxGeom {
    BoxGeom::new(1.0e6, 1.0e6, 1.0e6, false)
}

// ---------------------------------------------------------------- G3, the measurement

#[test]
fn the_exponent_is_measured_from_the_knots_and_not_assumed() {
    // A curve that IS a power law reads its own exponent back, and its exponential
    // extrapolation index agrees — for `u ∼ r^(−p)` the logarithmic derivative is exactly
    // `p/r`, so `hi_b · r_max == p`. That identity is what G3's band is slack around.
    for p in [4.0, 6.0, 6.5] {
        let f = power(p, 20.0).fit();
        assert!(
            (f.p_fit - p).abs() < 1.0e-9,
            "fitted {} for a true {p}",
            f.p_fit
        );
        assert!(f.residual < 1.0e-12, "a pure power law has no fit residual");
        assert!((f.exp_index - p).abs() < 1.0e-9);
    }
    assert_eq!(power(6.0, 20.0).fit().band, TailBand::Adopting);
    // Outside the band on either side, the curve is FENCED rather than adopted.
    assert_eq!(power(4.0, 20.0).fit().band, TailBand::Fenced);
    assert_eq!(power(12.0, 20.0).fit().band, TailBand::Fenced);
}

#[test]
fn the_tail_matches_the_table_exactly_at_the_seam() {
    // `C_p` is DETERMINED by the match at `R_s`, not fitted, so the model and the table
    // agree there to the bit. A seam that did not close would put a step in the energy
    // exactly where the two sectors hand over.
    let c = power(6.0, 20.0);
    let r_s = 20.0;
    let f = FarSector::build(&[Some(c.clone())], r_s, 1.0e-9, Dims::Two).expect("builds");
    let m = f.model(0).expect("a model for the loaded slot");
    let (u, _) = m.eval(r_s);
    assert!(
        (u - c.u_at(r_s)).abs() <= f64::EPSILON * u.abs().max(1.0e-300),
        "seam mismatch: model {u:e} vs table {:e}",
        c.u_at(r_s)
    );
}

// ---------------------------------------------------------------- the refusals (G11)

#[test]
fn r3_refuses_a_near_radius_inside_a_curve_s_support() {
    // The configuration B1b audited, exactly: c* = 15.0 against the O–O curve's r_max of
    // 20.0. What B2 changes is that this is now unrepresentable rather than merely known.
    match FarSector::build(&[Some(power(6.0, 20.0))], 15.0, 1.0e-9, Dims::Two) {
        Err(FarRefusal::SubSupport { r_s, r_max, .. }) => {
            assert_eq!(r_s, 15.0);
            assert_eq!(r_max, 20.0);
        }
        _ => panic!("R3 did not fire on a sub-support near radius"),
    }
}

#[test]
fn r1_refuses_a_kernel_its_image_lattice_cannot_sum() {
    // `p ≤ d` is the ionic `r⁻¹` case, and it fails in 2D and in 3D alike. The refusal
    // names the exit, because a fence without one is suppression.
    for (dims, d) in [(Dims::Two, 2usize), (Dims::Three, 3usize)] {
        match FarSector::build(&[Some(power(1.0, 20.0))], 20.0, 1.0e-9, dims) {
            Err(FarRefusal::ExponentTooShallow { p, d: got, exit }) => {
                assert!((p - 1.0).abs() < 1.0e-9);
                assert_eq!(got, d);
                assert!(exit.contains("Ewald") || exit.contains("PME"));
            }
            _ => panic!("R1 did not fire at p = 1, d = {d}"),
        }
    }
    // And the boundary is where the argument puts it: p just above d is admitted, p just
    // below is not. A refusal that fired everywhere would prove nothing about the licence.
    assert!(FarSector::build(&[Some(power(2.5, 20.0))], 20.0, 1.0e-9, Dims::Two).is_ok());
    assert!(FarSector::build(&[Some(power(2.5, 20.0))], 20.0, 1.0e-9, Dims::Three).is_err());
}

#[test]
fn r1_refuses_a_charged_scene_because_this_force_law_has_no_charge() {
    assert!(FarSector::admit_charge(0.0).is_ok());
    match FarSector::admit_charge(-1.0) {
        Err(FarRefusal::ChargedScene { charge, exit }) => {
            assert_eq!(charge, -1.0);
            assert!(exit.contains("node C"));
        }
        _ => panic!("R1's charge prong did not fire"),
    }
}

#[test]
fn r5_refuses_a_tail_parameter_without_its_solve_s_exit_and_budget() {
    // A capped residual is not monotone in effort. B1b banked that the O–O curve exits
    // IterationCap at 5000 iterations, so every constant fitted from it inherits that.
    let mut no_exit = power(6.0, 20.0);
    no_exit.solver_exit = "";
    assert!(matches!(
        FarSector::build(&[Some(no_exit)], 20.0, 1.0e-9, Dims::Two),
        Err(FarRefusal::UndisclosedSolve { missing: "solver_exit", .. })
    ));
    let mut no_budget = power(6.0, 20.0);
    no_budget.solver_budget_iterations = 0;
    assert!(matches!(
        FarSector::build(&[Some(no_budget)], 20.0, 1.0e-9, Dims::Two),
        Err(FarRefusal::UndisclosedSolve { missing: "solver_budget_iterations", .. })
    ));
}

#[test]
fn r4_hands_out_a_bracket_and_never_a_scalar_on_a_fenced_tail() {
    let fenced = FarSector::build(&[Some(power(12.0, 20.0))], 20.0, 1.0e-9, Dims::Two)
        .expect("builds");
    assert!(fenced.is_fenced());
    assert!(matches!(
        fenced.scalar_ok(-1.0e-6, -1.0e-9),
        Err(FarRefusal::FencedTailScalar { .. })
    ));
    let clean =
        FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
    assert!(!clean.is_fenced());
    assert!(clean.scalar_ok(-1.0e-6, -1.0e-9).is_ok());
}

#[test]
fn r2_refuses_when_the_image_shells_cannot_meet_the_declared_budget() {
    let mut f = FarSector::build(&[Some(power(3.0, 20.0))], 20.0, 1.0e-30, Dims::Two)
        .expect("builds");
    let geom = BoxGeom::new(34.6, 20.8, 20.8, true);
    match f.resolve_shells(&[(0.0, 0.0, 0.0), (5.0, 0.0, 0.0)], &[0, 0], geom) {
        Err(FarRefusal::ImageBudget { cap, .. }) => assert_eq!(cap, SHELL_CAP),
        other => panic!("R2 did not fire: {other:?}"),
    }
}

// ------------------------------------------------- what the conservation gates rest on

#[test]
fn the_far_force_is_equal_and_opposite_to_the_bit() {
    // G5's strict half. `+f` and `−f` are one computed value with opposite signs, so their
    // sum is bit-zero — not small, zero. A construction that merely happened to be close
    // would make the momentum gate a measurement of how close.
    let f0 = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
    for d in [20.5f64, 25.0, 40.0, 100.0] {
        let mut far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two)
            .expect("builds");
        let pos = [(0.0, 0.0, 0.0), (d, 0.3, 0.0)];
        let mut f = [(0.0, 0.0, 0.0); 2];
        let read = far.accumulate(&pos, &[0, 0], open_geom(), &mut f, &[20.0]);
        if read.contributions == 0 {
            continue; // outside R_f; the pair is not in this sector's domain
        }
        assert_eq!(f[0].0 + f[1].0, 0.0);
        assert_eq!(f[0].1 + f[1].1, 0.0);
        assert_eq!(f[0].2 + f[1].2, 0.0);
    }
    assert!(f0.r_f() > f0.r_s());
}

#[test]
fn the_far_force_is_central_which_is_what_conserves_angular_momentum() {
    // Equal and opposite is not enough: a force with a component perpendicular to the
    // separation conserves P exactly and destroys L. G6 is the gate for it and P3 is the
    // plant; this is the construction the two are checking.
    let mut far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two)
        .expect("builds");
    let pos = [(0.0, 0.0, 0.0), (14.0, 18.0, 0.0)];
    let mut f = [(0.0, 0.0, 0.0); 2];
    let read = far.accumulate(&pos, &[0, 0], open_geom(), &mut f, &[20.0]);
    assert!(read.contributions > 0, "the probe pair must be in the far sector");
    // cross(r_ij, F_i) == 0 for a central force.
    let (dx, dy) = (pos[1].0 - pos[0].0, pos[1].1 - pos[0].1);
    let cross = dx * f[0].1 - dy * f[0].0;
    let scale = (dx.abs() * f[0].1.abs()).max(dy.abs() * f[0].0.abs()).max(1.0e-300);
    assert!(cross.abs() / scale < 1.0e-14, "non-central by {cross:e}");
}

#[test]
fn the_non_central_plant_leaves_linear_momentum_exactly_alone() {
    // P3's premise, checked rather than asserted: rotating the pair force keeps it equal
    // and opposite, so the LINEAR sum is still bit-zero while the couple is not. If this
    // failed, P3 would be firing both gates and demonstrating nothing about either.
    let mut far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two)
        .expect("builds");
    far.plant = Some(FarPlant::NonCentralForce);
    let pos = [(0.0, 0.0, 0.0), (14.0, 18.0, 0.0)];
    let mut f = [(0.0, 0.0, 0.0); 2];
    let read = far.accumulate(&pos, &[0, 0], open_geom(), &mut f, &[20.0]);
    assert!(read.contributions > 0);
    assert_eq!(f[0].0 + f[1].0, 0.0);
    assert_eq!(f[0].1 + f[1].1, 0.0);
    let (dx, dy) = (pos[1].0 - pos[0].0, pos[1].1 - pos[0].1);
    let cross = dx * f[0].1 - dy * f[0].0;
    assert!(cross != 0.0, "the plant produced no couple; its carrier is 0.0");
}

#[test]
fn the_far_force_is_minus_the_gradient_of_the_far_energy() {
    // G8, at suite cost and on a synthetic curve. Its job here is to catch a sign or a
    // chain-rule slip; the measured version runs against the committed tables.
    let mut far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two)
        .expect("builds");
    let pos = vec![
        (0.0, 0.0, 0.0),
        (22.0, 1.0, 0.0),
        (3.0, 24.0, 0.0),
        (26.0, 27.0, 0.0),
    ];
    let slots = [0usize, 0, 0, 0];
    let geom = open_geom();
    let mut f = vec![(0.0, 0.0, 0.0); 4];
    let read = far.accumulate(&pos, &slots, geom, &mut f, &[20.0]);
    assert!(read.contributions > 0);
    let h = 1.0e-6;
    for i in 0..4 {
        for ax in 0..2 {
            let (mut p, mut m) = (pos.clone(), pos.clone());
            if ax == 0 {
                p[i].0 += h;
                m[i].0 -= h;
            } else {
                p[i].1 += h;
                m[i].1 -= h;
            }
            let numeric = -(far.energy_at_shells(&p, &slots, geom, 0)
                - far.energy_at_shells(&m, &slots, geom, 0))
                / (2.0 * h);
            let analytic = if ax == 0 { f[i].0 } else { f[i].1 };
            let scale = analytic.abs().max(numeric.abs()).max(1.0e-30);
            assert!(
                (analytic - numeric).abs() / scale < 1.0e-6,
                "atom {i} axis {ax}: analytic {analytic:e} vs numeric {numeric:e}"
            );
        }
    }
}

// ---------------------------------------------------------------- the cache seam (G9)

#[test]
fn the_image_lattice_is_a_pure_function_of_the_box_key() {
    // G9 rests on this: a rescaled sector and a fresh one must build the same list, so a
    // difference between them is a stale cache and nothing else.
    let k = BoxKey {
        lx: 10.0,
        ly: 12.0,
        lz: 12.0,
        periodic: true,
        three_d: false,
        shells: 2,
    };
    let a = FarSector::offsets_for(k);
    let b = FarSector::offsets_for(k);
    assert_eq!(a.len(), 24, "5x5 minus the origin, in two dimensions");
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.0.to_bits(), y.0.to_bits());
        assert_eq!(x.1.to_bits(), y.1.to_bits());
    }
    // A non-wrapping box has no images at all, and that zero is a fact about the box.
    assert!(FarSector::offsets_for(BoxKey { periodic: false, ..k }).is_empty());
    assert!(FarSector::offsets_for(BoxKey { shells: 0, ..k }).is_empty());
    // Three dimensions add the third axis and nothing else.
    assert_eq!(
        FarSector::offsets_for(BoxKey { three_d: true, ..k }).len(),
        5 * 5 * 5 - 1
    );
}

// ------------------------------------------- what B2 promises NOT to disturb

#[test]
fn a_scene_with_no_far_sector_carries_an_exact_zero() {
    // The whole basis for "every pre-B2 replay fingerprint stays valid": `e_far` is not
    // small when there is no far sector, it is exactly 0.0, and adding an exact zero to a
    // finite float changes no bit.
    let mut sim = Box::new(Sim::empty());
    sim.boundary = Boundary::Open;
    sim.dims = Dims::Two;
    sim.width = 34.6;
    sim.height = 20.8;
    sim.depth = 20.8;
    sim.reset(4);
    sim.recompute();
    assert!(sim.far.is_none());
    assert_eq!(sim.e_far.to_bits(), 0.0f64.to_bits());
    for _ in 0..50 {
        sim.step();
        assert_eq!(sim.e_far.to_bits(), 0.0f64.to_bits());
    }
    assert_eq!(sim.far_reading.contributions, 0);
}

#[test]
fn the_physics_digest_is_untouched_without_a_far_sector_and_moves_with_one() {
    // The checkpoint refuses a restore whose physics disagrees. A far sector IS physics the
    // run was performed against, so it must move the digest — and it must move NOTHING when
    // there is none, or every checkpoint banked before B2 would stop validating.
    let mut sim = Box::new(Sim::empty());
    sim.boundary = Boundary::Open;
    sim.dims = Dims::Two;
    sim.width = 34.6;
    sim.height = 20.8;
    sim.depth = 20.8;
    sim.reset(4);
    sim.recompute();
    let without = holon_render::checkpoint::physics_digest(&sim);

    let far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
    sim.far = Some(Box::new(far));
    let with = holon_render::checkpoint::physics_digest(&sim);
    assert_ne!(
        without, with,
        "a declared far sector must move the digest, or a checkpoint restored without it \
         would be a different experiment wearing the same file name"
    );

    // And removing it returns the digest to exactly what it was — the `if let` writes
    // nothing rather than writing a flag.
    sim.far = None;
    assert_eq!(without, holon_render::checkpoint::physics_digest(&sim));
}

#[test]
fn list_cutoff_reaches_the_near_radius_when_a_far_sector_is_declared() {
    // B1b's defect, made unrepresentable at the other end: the decomposition must reach as
    // far as the near sector is asked to cover, or the split has a hole in it.
    let mut sim = Box::new(Sim::empty());
    sim.boundary = Boundary::Open;
    sim.dims = Dims::Two;
    sim.width = 60.0;
    sim.height = 60.0;
    sim.depth = 60.0;
    sim.reset(4);
    let before = sim.list_cutoff();
    let far = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
    sim.far = Some(Box::new(far));
    assert!(sim.list_cutoff() >= 20.0);
    assert!(sim.list_cutoff() >= before);
}

#[test]
fn angular_momentum_is_gated_only_where_the_box_conserves_it() {
    // `angular_gate` returns `None` rather than `true` outside its domain, so a caller
    // cannot mistake "not applicable" for a pass. Walls torque, a field picks a direction,
    // and a periodic box's image lattice is not isotropic.
    let mut sim = Box::new(Sim::empty());
    sim.dims = Dims::Two;
    sim.width = 34.6;
    sim.height = 20.8;
    sim.depth = 20.8;
    sim.reset(4);

    sim.boundary = Boundary::Open;
    sim.recompute();
    assert!(sim.angular_gate().is_some(), "an open box conserves L");

    sim.boundary = Boundary::Walls;
    sim.recompute();
    assert!(sim.angular_gate().is_none(), "walls torque");

    sim.boundary = Boundary::Periodic;
    sim.recompute();
    assert!(
        sim.angular_gate().is_none(),
        "a periodic box does no work and delivers no impulse, so it looks like the open box \
         to the energy and momentum ledgers — but it is not isotropic"
    );

    sim.boundary = Boundary::Open;
    sim.g_vec = (0.0, -1.0e-6, 0.0);
    sim.recompute();
    assert!(sim.angular_gate().is_none(), "a uniform field picks out a direction");
}

#[test]
fn the_residual_bound_assumes_nothing_about_how_the_atoms_are_arranged() {
    // M-HOMOG: the standard isotropic tail integral needs a bulk density and `g(r) → 1`,
    // and this campaign's scenes are 12 atoms in a walled box with no bulk. The bound used
    // instead is every pair at worst carrying `|u_far(R_f)|`, which is crude and true.
    let f = FarSector::build(&[Some(power(6.0, 20.0))], 20.0, 1.0e-9, Dims::Two).expect("builds");
    let one = f.residual_bound(2, f.r_f());
    let many = f.residual_bound(12, f.r_f());
    assert!(one > 0.0);
    // 66 pairs against 1: the bound is exactly the pair count times the per-pair worst.
    assert!((many / one - 66.0).abs() < 1.0e-9);
    // And it tightens as the far sum is carried further, which is the only direction a
    // truncation residual may move.
    assert!(f.residual_bound(12, f.r_f() * 2.0) < many);
}

// ------------------------------------------------- the invariants see their own mutations

#[test]
fn the_plants_are_visible_to_the_invariants_above() {
    // A test that cannot see its own mutation is not a test. Each invariant this file
    // asserts is re-run with the plant that should break it, and the plant must MOVE the
    // quantity the invariant reads — three of seven mutations in a sibling campaign stayed
    // silent for numerical reasons, so a plant is trusted only after it has been watched.
    let curve = || Some(power(6.0, 20.0));
    let pos = vec![(0.0, 0.0, 0.0), (22.0, 1.0, 0.0), (3.0, 24.0, 0.0)];
    let slots = [0usize, 0, 0];
    let geom = open_geom();

    let read = |plant: Option<FarPlant>| {
        let mut far = FarSector::build(&[curve()], 20.0, 1.0e-9, Dims::Two).expect("builds");
        far.plant = plant;
        let mut f = vec![(0.0, 0.0, 0.0); 3];
        let r = far.accumulate(&pos, &slots, geom, &mut f, &[20.0]);
        let sum = (f[0].0 + f[1].0 + f[2].0, f[0].1 + f[1].1 + f[2].1);
        let scale: f64 = f.iter().map(|x| x.0.abs() + x.1.abs()).sum();
        (r, sum, scale)
    };

    let (clean, clean_sum, scale) = read(None);
    assert!(clean.contributions > 0, "the probe scene must reach the far sector");
    assert!(clean.virial != 0.0, "a live far sector owes the pressure a virial");
    // ROUNDOFF, not bit-zero, and the distinction is the point. Each PAIR cancels exactly
    // — `+f` and `−f` are one computed value with opposite signs, which is what
    // `the_far_force_is_equal_and_opposite_to_the_bit` checks on an isolated pair. The
    // TOTAL over several pairs is a floating-point sum whose terms arrive in different
    // orders per atom, so it cancels to roundoff and no further. Asserting bit-zero here
    // would be asserting a property of the summation order.
    let clean_rel = (clean_sum.0.abs() + clean_sum.1.abs()) / scale.max(1.0e-300);
    assert!(
        clean_rel < 1.0e-14,
        "the clean force sum is {clean_rel:e} of the force scale, which is not roundoff"
    );

    // P2 — the one-sided force leaves a residual the size of the forces themselves, which
    // is what makes it visible to a momentum gate whose bound is roundoff-sized.
    let (_, one_sided, one_scale) = read(Some(FarPlant::OneSidedForce));
    let plant_rel = (one_sided.0.abs() + one_sided.1.abs()) / one_scale.max(1.0e-300);
    assert!(
        plant_rel > 1.0e-3,
        "OneSidedForce moved the force sum by only {plant_rel:e} of scale; at that size a \
         momentum gate could not separate it from roundoff"
    );

    // P6 — the omitted band carries real energy, which is what B1b sized.
    let (cut, _, _) = read(Some(FarPlant::TruncatedFarSum));
    assert!(
        (cut.energy - clean.energy).abs() > 0.0,
        "TruncatedFarSum changed no energy; the omitted band is empty"
    );

    // P7 — the virial stops being posted and NOTHING else moves. That second half is what
    // lets P7 show a channel can be perfectly conservative and still be missing from the
    // pressure; a plant that also moved the energy would fire the energy gate too.
    let (no_virial, _, _) = read(Some(FarPlant::OmittedVirial));
    assert_eq!(no_virial.virial, 0.0);
    assert_eq!(no_virial.energy.to_bits(), clean.energy.to_bits());

    // P4 — the step moves the energy by exactly the planted amount per contribution and
    // leaves every force bit-identical. That second half is precisely why the plant is
    // invisible to a drift gate until a pair actually crosses `R_s`: with no force change
    // there is no dynamics change, only a discontinuity in the ledger at the crossing.
    let mut stepped = FarSector::build(&[curve()], 20.0, 1.0e-9, Dims::Two).expect("builds");
    let mut clean_f = vec![(0.0, 0.0, 0.0); 3];
    stepped.accumulate(&pos, &slots, geom, &mut clean_f, &[20.0]);
    let mut stepped = FarSector::build(&[curve()], 20.0, 1.0e-9, Dims::Two).expect("builds");
    stepped.plant = Some(FarPlant::ZeroPointStep);
    let mut plant_f = vec![(0.0, 0.0, 0.0); 3];
    let r = stepped.accumulate(&pos, &slots, geom, &mut plant_f, &[20.0]);
    let expected =
        clean.energy + holon_render::longrange::PLANT_STEP_HARTREE * clean.contributions as f64;
    assert!(
        (r.energy - expected).abs() < 1.0e-18,
        "the step is not exactly the planted constant per contribution: {:e} vs {expected:e}",
        r.energy
    );
    for i in 0..3 {
        assert_eq!(plant_f[i].0.to_bits(), clean_f[i].0.to_bits());
        assert_eq!(plant_f[i].1.to_bits(), clean_f[i].1.to_bits());
    }
}
