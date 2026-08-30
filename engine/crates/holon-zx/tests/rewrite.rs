//! Tests for native ZX graph rewriting passes:
//! - Spider fusion (Z-Z and X-X)
//! - Local complementation
//! - Pivoting
//! - Phase gadget extraction & identity removal
//! - Maximum T-count reduction pre-extraction

use holon::qasm::Surface::*;
use holon_zx::graph::{EdgeType, ZxGraph};
use holon_zx::{canonicalize, from_surface};

#[test]
fn test_spider_fusion_zz_and_xx() {
    // 1. Test Z-Z fusion
    let mut g = ZxGraph::with_capacity(10);
    let z1 = g.add_z_spider(1); // phase π/4
    let z2 = g.add_z_spider(2); // phase 2π/4
    g.set_edge(z1, z2, Some(EdgeType::Normal));

    assert!(g.check_spider_fusion(z1, z2));
    g.spider_fusion(z1, z2);
    assert_eq!(g.phase[z1], 3, "Z-Z fusion adds phases: 1 + 2 = 3 (mod 8)");
    assert!(!g.alive[z2], "absorbed spider is marked dead");

    // 2. Test X-X fusion
    let mut gx = ZxGraph::with_capacity(10);
    let x1 = gx.add_x_spider(3); // 3π/4
    let x2 = gx.add_x_spider(7); // 7π/4
    gx.set_edge(x1, x2, Some(EdgeType::Normal));

    assert!(gx.check_spider_fusion(x1, x2));
    gx.spider_fusion(x1, x2);
    assert_eq!(gx.phase[x1], 2, "X-X fusion: (3 + 7) mod 8 = 2");
    assert!(!gx.alive[x2]);
}

#[test]
fn test_identity_removal() {
    let mut g = ZxGraph::with_capacity(10);
    let a = g.add_z_spider(1);
    let v = g.add_z_spider(0); // degree-2 phase-0 spider
    let b = g.add_z_spider(2);
    g.set_edge(a, v, Some(EdgeType::Normal));
    g.set_edge(v, b, Some(EdgeType::Hadamard));

    assert!(g.check_remove_id(v));
    g.remove_id(v);
    assert!(!g.alive[v]);
    assert!(g.has_edge(a, b));
    assert_eq!(
        g.edge_type(a, b),
        Some(EdgeType::Hadamard),
        "Normal ⊕ Hadamard = Hadamard"
    );
}

#[test]
fn test_local_complementation() {
    let mut g = ZxGraph::with_capacity(10);
    // Clifford spider v with phase π/2 (phase 2)
    let v = g.add_z_spider(2);
    let n1 = g.add_z_spider(0);
    let n2 = g.add_z_spider(0);
    let n3 = g.add_z_spider(0);

    g.set_edge(v, n1, Some(EdgeType::Hadamard));
    g.set_edge(v, n2, Some(EdgeType::Hadamard));
    g.set_edge(v, n3, Some(EdgeType::Hadamard));

    assert!(g.check_local_comp(v));
    g.local_comp(v);

    assert!(!g.alive[v], "spider v removed");
    // All neighbour pairs now have Hadamard edges
    assert!(g.has_edge(n1, n2) && g.is_h(n1, n2));
    assert!(g.has_edge(n2, n3) && g.is_h(n2, n3));
    assert!(g.has_edge(n1, n3) && g.is_h(n1, n3));
    // Neighbour phases shifted by -2 = 6 (mod 8)
    assert_eq!(g.phase[n1], 6);
    assert_eq!(g.phase[n2], 6);
    assert_eq!(g.phase[n3], 6);
}

#[test]
fn test_pivoting_interior_pauli() {
    let mut g = ZxGraph::with_capacity(10);
    // Two Pauli spiders (phases 0 and π = 4) connected by Hadamard edge
    let u = g.add_z_spider(0);
    let v = g.add_z_spider(4);
    g.set_edge(u, v, Some(EdgeType::Hadamard));

    let a = g.add_z_spider(1); // neighbour of u only
    let b = g.add_z_spider(2); // neighbour of v only
    let c = g.add_z_spider(3); // shared neighbour

    g.set_edge(u, a, Some(EdgeType::Hadamard));
    g.set_edge(u, c, Some(EdgeType::Hadamard));
    g.set_edge(v, b, Some(EdgeType::Hadamard));
    g.set_edge(v, c, Some(EdgeType::Hadamard));

    assert!(g.check_pivot(u, v));
    g.pivot(u, v);

    assert!(!g.alive[u]);
    assert!(!g.alive[v]);

    // Phases updated:
    // a: 1 + pv = 1 + 4 = 5
    // b: 2 + pu = 2 + 0 = 2
    // c: 3 + pu + pv + 4 = 3 + 0 + 4 + 4 = 11 mod 8 = 3
    assert_eq!(g.phase[a], 5);
    assert_eq!(g.phase[b], 2);
    assert_eq!(g.phase[c], 3);

    // Toggled edges: a-b, a-c, b-c
    assert!(g.has_edge(a, b));
    assert!(g.has_edge(a, c));
    assert!(g.has_edge(b, c));
}

#[test]
fn test_phase_gadget_extraction_and_fusion() {
    let prog = vec![
        T(0),
        Cx(1, 0),
        T(0),
        Cx(1, 0),
        T(0),
        T(1),
        H(0),
        T(0),
        H(0),
        T(0),
    ];
    let mut g = from_surface(2, &prog).unwrap();
    let initial_t = g.t_count();
    g.full_reduce();
    let final_t = g.t_count();

    assert!(
        final_t < initial_t,
        "full_reduce achieves significant T-count reduction: {initial_t} -> {final_t}"
    );
}

#[test]
fn test_maximum_t_count_reduction_pre_extraction() {
    // CCX (Toffoli) standard decomposition has 7 T gates
    let toffoli = vec![
        H(2),
        Cx(1, 2),
        Tdg(2),
        Cx(0, 2),
        T(2),
        Cx(1, 2),
        Tdg(2),
        Cx(0, 2),
        T(1),
        T(2),
        H(2),
        Cx(0, 1),
        T(0),
        Tdg(1),
        Cx(0, 1),
    ];
    let mut g = from_surface(3, &toffoli).unwrap();
    assert_eq!(g.t_count(), 7, "raw Toffoli has 7 T gates");
    g.full_reduce();
    // Pre-extraction T-count of a single Toffoli or inverse-cancelling pairs
    let (_simplified, red) = canonicalize(3, &toffoli).unwrap();
    assert!(
        red.t_after <= 7,
        "T-count after simplification must be <= 7"
    );

    // Two adjacent Toffolis cancel out to identity (0 T gates)
    let mut two_toffolis = toffoli.clone();
    two_toffolis.extend(toffoli);
    let mut g2 = from_surface(3, &two_toffolis).unwrap();
    assert_eq!(g2.t_count(), 14);
    g2.full_reduce();
    assert_eq!(
        g2.t_count(),
        0,
        "Two adjacent Toffolis completely cancel to T-count 0 pre-extraction"
    );

    let (_s2, red2) = canonicalize(3, &two_toffolis).unwrap();
    assert_eq!(red2.t_after, 0, "canonicalized circuit has 0 T gates");
}
