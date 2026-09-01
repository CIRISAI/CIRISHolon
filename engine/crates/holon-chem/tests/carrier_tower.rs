//! Carrier Tower (WB-8) Certification and Refusal Test Battery
//!
//! Enforces:
//! 1. Double counting refusal (type-level isolation across carrier fibers)
//! 2. Missing picture change refusal
//! 3. Corridor rule budget-violating selection refusal
//! 4. Stub-without-fence refusal
//! 5. C0 (Classical BO) <-> C1 (Ring-Polymer Quantum Nuclei) transport & P=1 diagonal retract
//! 6. Exact ZPE quantum nuclear ground state on H2 STO-3G potential
//! 7. \ell-generalized AngularShell properties

use holon_chem::elements::Species;
use holon_chem::tower::*;

#[test]
fn test_angular_shell_generalization() {
    let s = AngularShell::S;
    assert_eq!(s.l, 0);
    assert_eq!(s.num_cartesian(), 1);
    assert_eq!(s.num_spherical(), 1);
    assert_eq!(s.spectroscopic_symbol(), 's');

    let p = AngularShell::P;
    assert_eq!(p.l, 1);
    assert_eq!(p.num_cartesian(), 3);
    assert_eq!(p.num_spherical(), 3);
    assert_eq!(p.spectroscopic_symbol(), 'p');

    let d = AngularShell::D;
    assert_eq!(d.l, 2);
    assert_eq!(d.num_cartesian(), 6);
    assert_eq!(d.num_spherical(), 5);
    assert_eq!(d.spectroscopic_symbol(), 'd');

    let f = AngularShell::F;
    assert_eq!(f.l, 3);
    assert_eq!(f.num_cartesian(), 10);
    assert_eq!(f.num_spherical(), 7);
    assert_eq!(f.spectroscopic_symbol(), 'f');

    let g = AngularShell { l: 4 };
    assert_eq!(g.num_cartesian(), 15);
    assert_eq!(g.num_spherical(), 9);
    assert_eq!(g.spectroscopic_symbol(), 'g');
}

#[test]
fn test_fiber_contribution_addition_within_carrier() {
    let op1 = ClassicalPotentialOp { pair_energy_fn: Some(|r| 1.0 / r) };
    let op2 = ClassicalPotentialOp { pair_energy_fn: Some(|_r| 0.5) };

    let c1 = Contribution::<C0_ClassicalBO>::new("coulomb", op1);
    let c2 = Contribution::<C0_ClassicalBO>::new("shift", op2);

    let sum = c1 + c2;
    assert_eq!(sum.name, "sum");

    let state = ClassicalState {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        velocities: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
        masses: vec![1.0, 1.0],
    };

    let e = sum.evaluate(&state);
    assert!((e - 1.0).abs() < 1e-12);
}

#[test]
fn test_refusal_missing_picture_change() {
    let cert = CommutingCertificate {
        from_carrier: "C0_ClassicalBO",
        to_carrier: "C1_RingPolymer",
        closure_defect: 1e-4,
        condition_number: 1.5,
        cert_digest: [0u8; 32],
    };

    // Construct a transport where operator picture change is intentionally refused
    let broken_transport = CertifiedTransport::<C0_ClassicalBO, C1_RingPolymer>::new(
        cert,
        |_| RingPolymerState { beads_pos: vec![], beads_vel: vec![], masses: vec![] },
        |_| Err(TransportRefusal::MissingPictureChange { from: "C0_ClassicalBO", to: "C1_RingPolymer" }),
    );

    let op = ClassicalPotentialOp::zero();
    let res = broken_transport.transport_operator(&op);
    assert_eq!(
        res,
        Err(TransportRefusal::MissingPictureChange { from: "C0_ClassicalBO", to: "C1_RingPolymer" })
    );
}

#[test]
fn test_corridor_selection_rule_and_budget_refusal() {
    // node_coarse has tight tolerance (1e-8) and cheap price (0.01)
    // node_robust has wide tolerance (1e-4) and higher price (1.50)
    let node_coarse = TheoryNode::new(C0_ClassicalBO, 1e-8, 1e-8, 0.01);
    let node_robust = TheoryNode::new(C0_ClassicalBO, 1e-4, 1e-4, 1.50);

    let candidates = vec![node_coarse.clone(), node_robust.clone()];

    // Case 1: Small defect (1e-9 <= 1e-8) -> both eligible -> argmin(price) selects node_coarse (0.01)
    let selected = select_corridor(&candidates, 1e-9, 1e-9).unwrap();
    assert_eq!(selected.measured_price_core_seconds, 0.01);

    // Case 2: Intermediate defect (1e-6 > 1e-8) -> node_coarse disqualified -> selects node_robust (1.50)
    let selected2 = select_corridor(&candidates, 1e-6, 1e-6).unwrap();
    assert_eq!(selected2.measured_price_core_seconds, 1.50);

    // Case 3: Extreme defect (1e-2 > 1e-4) -> exceeds all budgets -> refuses by Corridor theorem
    let err = select_corridor(&candidates, 1e-2, 1e-2);
    assert!(matches!(err, Err(TransportRefusal::ClosureDefectExceeded { .. })));
}

#[test]
fn test_capability_stub_fence_enforcement() {
    // C2 is no longer a stub: the carrier is built and gated
    // (`q8-mps/tests/c2_tdvp_gates.rs`). What moved, rather than disappeared, is WHICH
    // object is fenced — a NODE and an EDGE are different things, and the tower now says
    // so with different types: C2 the node is Certified, the C1->C2 climb is not.
    let c2 = c2_tdvp_capability();
    assert!(c2.is_certified(), "C2 is materialized; see C2_TDVP_RESULTS.md");
    let carrier = c2.unwrap_or_refuse().expect("a certified capability does not refuse");
    assert_eq!(carrier.name(), "C2_MpsTdvp");

    let stub = c1_to_c2_transport_capability();
    assert!(!stub.is_certified());
    assert_eq!(
        stub.unwrap_or_refuse(),
        Err(TransportRefusal::UntabulatedSeamFence {
            coordinate: "C1->C2 picture change unbuilt: no state lift, no operator picture change, no measured certificate"
        })
    );

    let stub_qed = c3_qed_capability();
    assert!(!stub_qed.is_certified());
}

/// C2's fiber law, exercised on the real carrier rather than asserted about it: two
/// second-quantised contributions add by pooling their terms, the pooled operator compiles
/// to ONE MPO, and its reading is the sum of the two readings. Pooling rather than
/// compiling twice is what makes double counting unrepresentable instead of merely
/// avoided.
#[test]
fn test_c2_fiber_addition_is_one_compilation() {
    let n_orb = 6; // 3 chain sites, Jordan-Wigner
    let state = MpsElectronicState {
        tensors: q8_mps::tdvp::deterministic_state(n_orb, 8, 20260901),
        time_au: 0.0,
    };

    // Hopping on one bond, and an on-site interaction: two genuinely different terms.
    let hop = SecondQuantisedOp::new(n_orb)
        .with_term(&[(0, true), (2, false)], -1.0)
        .with_term(&[(2, true), (0, false)], -1.0);
    let onsite = SecondQuantisedOp::new(n_orb)
        .with_term(&[(0, true), (0, false), (1, true), (1, false)], 4.0);

    let e_hop = hop.evaluate_energy(&state);
    let e_onsite = onsite.evaluate_energy(&state);

    let a = Contribution::<C2_MpsTdvp>::new("hop", hop);
    let b = Contribution::<C2_MpsTdvp>::new("onsite", onsite);
    let sum = a + b;
    let e_sum = sum.evaluate(&state);

    assert!(
        (e_sum - (e_hop + e_onsite)).abs() <= 1e-12,
        "C2 fiber addition is not additive: {e_sum} vs {e_hop} + {e_onsite}"
    );
    // And the empty operator reads exactly zero rather than refusing to be added.
    let mut z = <SecondQuantisedOp as AdditiveOperator<C2_MpsTdvp>>::zero();
    assert_eq!(z.evaluate_energy(&state), 0.0);
    z.add_assign(&SecondQuantisedOp::new(n_orb).with_term(&[(0, true), (0, false)], 1.0));
    assert_eq!(z.n_orbitals, n_orb);
}

/// C2 as DYNAMICS: propagation advances the state's own clock, conserves the energy the
/// carrier reads (the substeps are unitary on the orthogonality centre), and REFUSES an
/// empty Hamiltonian by name instead of returning the input unchanged.
#[test]
fn test_c2_propagation_advances_time_and_holds_energy() {
    let n_orb = 6;
    let carrier = C2_MpsTdvp { chi_max: 8 };
    let state = MpsElectronicState {
        tensors: q8_mps::tdvp::deterministic_state(n_orb, 8, 20260901),
        time_au: 0.0,
    };
    let mut h = SecondQuantisedOp::new(n_orb);
    for cs in 0..2 {
        for sigma in 0..2 {
            let (i, j) = (2 * cs + sigma, 2 * (cs + 1) + sigma);
            h.terms.push((vec![(i, true), (j, false)], -1.0));
            h.terms.push((vec![(j, true), (i, false)], -1.0));
        }
    }
    for cs in 0..3 {
        h.terms.push((vec![(2 * cs, true), (2 * cs, false), (2 * cs + 1, true), (2 * cs + 1, false)], 4.0));
    }

    let e0 = h.evaluate_energy(&state);
    let moved = carrier.propagate(&h, &state, 0.01, 16).expect("a compiled Hamiltonian propagates");
    assert!((moved.time_au - 0.16).abs() < 1e-12, "the state's clock did not advance");
    let e1 = h.evaluate_energy(&moved);
    assert!(
        (e1 - e0).abs() / e0.abs().max(1.0) <= 1e-10,
        "C2 propagation drifted the energy: {e0} -> {e1}"
    );

    // The state actually moved: a conservation gate on a static state measures nothing.
    let before = q8_mps::tdvp::to_dense(&state.tensors);
    let after = q8_mps::tdvp::to_dense(&moved.tensors);
    assert!(
        q8_mps::tdvp::relative_distance(&after, &before) > 1e-2,
        "C2 propagation did not move the state"
    );

    // And an empty Hamiltonian is a REFUSAL, not a free identity.
    let empty = <SecondQuantisedOp as AdditiveOperator<C2_MpsTdvp>>::zero();
    assert!(matches!(
        carrier.propagate(&empty, &state, 0.01, 1),
        Err(TransportRefusal::MissingPictureChange { .. })
    ));

    // The price is a MEASURED law, monotone in both of its arguments.
    let p_small = carrier.price_per_substep(6);
    let p_big = C2_MpsTdvp { chi_max: 16 }.price_per_substep(12);
    assert!(p_big > p_small && p_small > 0.0, "the C2 price law is not monotone");
}

#[test]
fn test_c0_c1_transport_and_centroid_retract() {
    let transport = make_c0_to_c1_transport(16, 293.15);

    let initial_classical = ClassicalState {
        positions: vec![[0.0, 0.0, 0.0], [0.74e-10, 0.0, 0.0]],
        velocities: vec![[10.0, -5.0, 0.0], [-10.0, 5.0, 0.0]],
        masses: vec![1.008, 1.008], // Hydrogen amu
    };

    // 1. Lift state from C0 to C1 (16 beads)
    let rp_state = transport.lift_state(&initial_classical);
    assert_eq!(rp_state.beads_pos.len(), 16);
    assert_eq!(rp_state.beads_pos[0].len(), 2);

    // Verify all 16 beads start at identical classical positions (P-fold replication)
    for k in 0..16 {
        assert_eq!(rp_state.beads_pos[k][0], [0.0, 0.0, 0.0]);
        assert_eq!(rp_state.beads_pos[k][1], [0.74e-10, 0.0, 0.0]);
    }

    // 2. Retract C1 back to C0 (Centroid contraction)
    let retracted_classical = transport.retract_state(&rp_state).unwrap();
    assert_eq!(retracted_classical.positions.len(), 2);

    for i in 0..2 {
        for dim in 0..3 {
            let diff = (retracted_classical.positions[i][dim] - initial_classical.positions[i][dim]).abs();
            assert!(diff < 1e-15, "Centroid retract must be bit-identical to classical state");
        }
    }

    // 3. Transport operator
    let c0_op = ClassicalPotentialOp {
        pair_energy_fn: Some(|r| -1.0 / (r + 1.0)),
    };
    let c1_op = transport.transport_operator(&c0_op).unwrap();
    let e_rp = c1_op.evaluate_energy(&rp_state);
    let e_c0 = c0_op.evaluate_energy(&initial_classical);

    assert!((e_rp - e_c0).abs() < 1e-12, "Replicated ring polymer energy must equal classical energy");
}

#[test]
fn test_h2_harmonic_zpe_sanity() {
    // SCOPE, and it is now a scope rather than an apology: this checks HARMONIC ZPE
    // arithmetic and the bead spring constant. It runs no ring-polymer dynamics and reads
    // no banked table, and it is NOT C1's gate.
    //
    // C1's real gate — the exact sinc-DVR reference on the engine's own H-H curve, real
    // PIMD against it, the P-convergence ladder, the D2 isotope shift, the classical
    // limit and the bead-forgetting commuting square — is `tests/c1_quantum_nuclei.rs`,
    // staked in `conformance/water_observatory/C1_GATE_PREREG.md`. The transcribed
    // constant that used to sit here is gone: the curvature is READ from the solver, and
    // reading it is how the transcription was found to be wrong. The comment said
    // `k = 0.5708 Ha/bohr^2`; the engine's own `E''(R_e)` is 0.477098, a 20% error that
    // no assertion in this test was tight enough to catch.
    let (r_e, _d_e, _v) = holon_chem::h2::equilibrium();
    let k_harm = holon_chem::h2::h2_point(r_e).e2;
    let mu = holon_chem::rpmd::Vib1D::reduced_mass_me(
        Species::HYDROGEN.mass_u,
        Species::HYDROGEN.mass_u,
    );
    let omega_harm = (k_harm / mu).sqrt();
    let zpe_harm = 0.5 * omega_harm;

    // Pinned two-sided against the model's own curvature, not against a quoted number.
    assert!(
        (zpe_harm - 0.011395045).abs() < 1e-8,
        "harmonic ZPE from the engine's own curvature: {zpe_harm:.9}"
    );

    // Ring-polymer spring constant: k_P = m P / beta^2, checked against the same
    // arithmetic written independently here.
    let c1 = C1_RingPolymer::new(32, 300.0);
    let k_bead = c1.bead_spring_constant(Species::HYDROGEN.mass_u);
    let beta = 1.0 / (holon_chem::rpmd::K_B_HARTREE_PER_KELVIN * 300.0);
    let want = Species::HYDROGEN.mass_u * holon_chem::elements::M_E_PER_U * 32.0 / (beta * beta);
    assert!((k_bead - want).abs() / want < 1e-12, "k_P {k_bead:.9} vs {want:.9}");
}
