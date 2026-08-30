//! Tests for enhanced OpenQASM 2/3 front-end rewriter and symplectic tableau canonicalization.

use holon_qasm::qasm::*;
use holon_qasm::*;

fn max_dist_err(
    a: &std::collections::BTreeMap<String, f64>,
    b: &std::collections::BTreeMap<String, f64>,
) -> f64 {
    let keys: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).collect();
    keys.into_iter()
        .map(|k| (a.get(k).unwrap_or(&0.0) - b.get(k).unwrap_or(&0.0)).abs())
        .fold(0.0, f64::max)
}

#[test]
fn test_openqasm2_and_openqasm3_parsing() {
    // OpenQASM 2.0 format
    let qasm2 = r#"
        OPENQASM 2.0;
        include "qelib1.inc";
        qreg q[2];
        creg c[2];
        h q[0];
        cx q[0], q[1];
        measure q[0] -> c[0];
        measure q[1] -> c[1];
    "#;
    let c2 = parse_qasm(qasm2).expect("parse QASM 2.0");
    assert_eq!(c2.n_qubits, 2);
    assert_eq!(c2.n_clbits, 2);
    assert_eq!(c2.gates, vec![Gate::H(0), Gate::Cx(0, 1)]);

    // OpenQASM 3.0 format
    let qasm3 = r#"
        OPENQASM 3.0;
        include "stdgates.inc";
        qubit[2] q;
        bit[2] c;
        h q[0];
        cx q[0], q[1];
        c[0] = measure q[0];
        c[1] = measure q[1];
    "#;
    let c3 = parse_qasm(qasm3).expect("parse QASM 3.0");
    assert_eq!(c3.n_qubits, 2);
    assert_eq!(c3.n_clbits, 2);
    assert_eq!(c3.gates, vec![Gate::H(0), Gate::Cx(0, 1)]);
}

#[test]
fn test_general_single_qubit_decompositions() {
    // 1. U3(pi/2, 0, pi) is Hadamard
    let src_u3_h = r#"
        OPENQASM 3.0;
        qubit[1] q;
        bit[1] c;
        u3(pi/2, 0, pi) q[0];
        c[0] = measure q[0];
    "#;
    let c_u3 = parse_qasm(src_u3_h).expect("parse u3");
    let dist_u3 = run_statevector(&c_u3);

    let src_h = r#"
        OPENQASM 2.0;
        qreg q[1];
        creg c[1];
        h q[0];
        measure q[0] -> c[0];
    "#;
    let c_h = parse_qasm(src_h).expect("parse h");
    let dist_h = run_statevector(&c_h);
    assert!(
        max_dist_err(&dist_u3, &dist_h) < 1e-12,
        "U3(pi/2, 0, pi) matches H"
    );

    // 2. Rx, Ry, Rz rotations
    let src_rot = r#"
        OPENQASM 3.0;
        qubit[2] q;
        bit[2] c;
        rx(pi) q[0];
        ry(pi/2) q[1];
        rz(pi/4) q[1];
        c[0] = measure q[0];
        c[1] = measure q[1];
    "#;
    let c_rot = parse_qasm(src_rot).expect("parse rotations");
    let dist_rot = run_statevector(&c_rot);
    assert_eq!(c_rot.n_qubits, 2);
    assert!(!dist_rot.is_empty());

    // 3. U2 gate: U2(0, pi) is H
    let src_u2 = r#"
        OPENQASM 3.0;
        qubit[1] q;
        u2(0, pi) q[0];
    "#;
    let c_u2 = parse_qasm(src_u2).expect("parse u2");
    let dist_u2 = run_statevector(&c_u2);
    assert!(
        max_dist_err(&dist_u2, &dist_h) < 1e-12,
        "U2(0, pi) matches H"
    );

    // 4. Arithmetic expressions in angle parameters
    let src_expr = r#"
        OPENQASM 3.0;
        qubit[1] q;
        rz(2 * pi / 4) q[0];
        rx(pi - pi / 2) q[0];
    "#;
    let c_expr = parse_qasm(src_expr).expect("parse arithmetic expr");
    assert!(!c_expr.gates.is_empty());
}

#[test]
fn test_symplectic_tableau_canonicalization() {
    // Two different Clifford gate sequences implementing the same Bell state preparation
    // Sequence 1: H(0), CX(0, 1)
    let c1 = Circuit {
        n_qubits: 2,
        n_clbits: 2,
        gates: vec![Gate::H(0), Gate::Cx(0, 1)],
        measures: vec![(0, 0), (1, 1)],
    };

    // Sequence 2: H(1), CX(1, 0), SWAP(0, 1)
    // SWAP(0, 1) = CX(0, 1) CX(1, 0) CX(0, 1)
    let c2 = Circuit {
        n_qubits: 2,
        n_clbits: 2,
        gates: vec![
            Gate::H(1),
            Gate::Cx(1, 0),
            Gate::Cx(0, 1),
            Gate::Cx(1, 0),
            Gate::Cx(0, 1),
        ],
        measures: vec![(0, 0), (1, 1)],
    };

    let mut t1 = Tableau::new(2, Mutation::None);
    for &g in &c1.gates {
        t1.apply(g);
    }

    let mut t2 = Tableau::new(2, Mutation::None);
    for &g in &c2.gates {
        t2.apply(g);
    }

    assert!(
        are_tableaux_equivalent(&t1, &t2),
        "Bell state circuits must have identical canonical tableaux"
    );

    let canon1 = canonicalize_circuit(&c1).expect("canonicalize c1");
    let canon2 = canonicalize_circuit(&c2).expect("canonicalize c2");
    assert_eq!(
        canon1.x[2..4],
        canon2.x[2..4],
        "Stabilizer X matrices match in canonical form"
    );
    assert_eq!(
        canon1.z[2..4],
        canon2.z[2..4],
        "Stabilizer Z matrices match in canonical form"
    );
    assert_eq!(
        canon1.r[2..4],
        canon2.r[2..4],
        "Stabilizer phases match in canonical form"
    );
}

#[test]
fn test_tableau_canonicalization_identities() {
    // Identity 1: H X H = Z
    let c_h1 = Circuit {
        n_qubits: 1,
        n_clbits: 1,
        gates: vec![Gate::H(0), Gate::X(0), Gate::H(0)],
        measures: vec![(0, 0)],
    };
    let c_h2 = Circuit {
        n_qubits: 1,
        n_clbits: 1,
        gates: vec![Gate::Z(0)],
        measures: vec![(0, 0)],
    };
    let mut t1 = Tableau::new(1, Mutation::None);
    for &g in &c_h1.gates {
        t1.apply(g);
    }
    let mut t2 = Tableau::new(1, Mutation::None);
    for &g in &c_h2.gates {
        t2.apply(g);
    }
    assert!(
        are_tableaux_equivalent(&t1, &t2),
        "H X H and Z must produce identical canonical tableaux"
    );

    // Identity 2: Sdg S = I
    let c_id = Circuit {
        n_qubits: 1,
        n_clbits: 1,
        gates: vec![Gate::S(0), Gate::Sdg(0)],
        measures: vec![(0, 0)],
    };
    let c_zero = Circuit {
        n_qubits: 1,
        n_clbits: 1,
        gates: vec![],
        measures: vec![(0, 0)],
    };
    let mut t_id = Tableau::new(1, Mutation::None);
    for &g in &c_id.gates {
        t_id.apply(g);
    }
    let mut t_zero = Tableau::new(1, Mutation::None);
    for &g in &c_zero.gates {
        t_zero.apply(g);
    }
    assert!(
        are_tableaux_equivalent(&t_id, &t_zero),
        "Sdg S and I must produce identical canonical tableaux"
    );

    // Identity 3: Two SWAP decompositions: CX(0,1)CX(1,0)CX(0,1) == CX(1,0)CX(0,1)CX(1,0)
    let c_sw1 = Circuit {
        n_qubits: 2,
        n_clbits: 2,
        gates: vec![Gate::Cx(0, 1), Gate::Cx(1, 0), Gate::Cx(0, 1)],
        measures: vec![(0, 0), (1, 1)],
    };
    let c_sw2 = Circuit {
        n_qubits: 2,
        n_clbits: 2,
        gates: vec![Gate::Cx(1, 0), Gate::Cx(0, 1), Gate::Cx(1, 0)],
        measures: vec![(0, 0), (1, 1)],
    };
    let mut t_sw1 = Tableau::new(2, Mutation::None);
    for &g in &c_sw1.gates {
        t_sw1.apply(g);
    }
    let mut t_sw2 = Tableau::new(2, Mutation::None);
    for &g in &c_sw2.gates {
        t_sw2.apply(g);
    }
    assert!(
        are_tableaux_equivalent(&t_sw1, &t_sw2),
        "Both SWAP constructions produce identical canonical tableaux"
    );
}
