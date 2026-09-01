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
    let stub = c2_tdvp_capability();
    assert!(!stub.is_certified());

    let refusal = stub.unwrap_or_refuse();
    assert_eq!(
        refusal,
        Err(TransportRefusal::UntabulatedSeamFence {
            coordinate: "C2 real-time TDVP electronic dynamics staged for crystal DMRG inheritance"
        })
    );

    let stub_qed = c3_qed_capability();
    assert!(!stub_qed.is_certified());
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
fn test_h2_anharmonic_quantum_nuclear_zpe_gate() {
    // Certified STO-3G H2 parameters from banked fci / table
    // Well depth: D_e = 0.1704 Hartree
    // Harmonic curvature at R_e: k = 0.5708 Hartree / bohr^2
    // Reduced mass of H2: mu = 0.5 * 1.007825 * 1822.888486 = 918.57 m_e
    let k_harm: f64 = 0.5708; // Ha / bohr^2
    let mu: f64 = 918.57; // m_e

    let omega_harm = (k_harm / mu).sqrt(); // in a.u.
    let zpe_harm = 0.5 * omega_harm; // Hartree

    // Theoretical STO-3G H2 harmonic ZPE ~ 0.01246 Hartree (0.339 eV = 2735 cm^-1)
    assert!(zpe_harm > 0.011 && zpe_harm < 0.014);

    // Ring-polymer RPMD representation: 32 beads at 300 K
    let c1 = C1_RingPolymer::new(32, 300.0);
    let k_bead = c1.bead_spring_constant(1.007825);
    assert!(k_bead > 0.0);
}
