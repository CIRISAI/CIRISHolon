//! The expansion at an order the hand-written path never had: a FIVE-cluster.
//!
//! `cluster::mbe_terms` is written for any arity and any order. Below four the terms are
//! tables; at four and above they are each subcluster's own connected term, computed
//! recursively and exactly. Nothing here is certified physics — the five-cluster's
//! body term is a number no table has been built for — so what is gated is the
//! ARITHMETIC: the identities the expansion must satisfy by construction, on a geometry
//! staked before the run.

use holon_chem::cluster::{
    body_term, cluster_fci_energy, mbe_energy, mbe_terms, subsets_lex, AbInitioPairs,
    SurfaceRegistry,
};
use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::quaternary_table::{de4_ohhh_fci, OHHH};
use holon_chem::{trimer, water};
use std::sync::OnceLock;

fn tables() -> &'static (water::WaterTable, trimer::TrimerTable) {
    static T: OnceLock<(water::WaterTable, trimer::TrimerTable)> = OnceLock::new();
    T.get_or_init(|| {
        let src = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/s2/s2_water_table.txt"),
        )
        .expect("the committed (O, H, H) table");
        (water::from_text(&src).expect("water table parses"), trimer::generate().expect("the H3 table"))
    })
}

/// An asymmetric OHHHH cluster: the four-body quartet the momentum audit uses, plus a
/// fourth hydrogen two bohr from the oxygen on the side none of the others occupy.
const OHHHH: [Species; 5] = [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN];
const G5: [[f64; 3]; 5] = [
    [0.0, 0.0, 0.0],
    [1.83, 0.0, 0.0],
    [-0.61, 1.94, 0.0],
    [-0.55, -0.72, 1.71],
    [1.2, -1.3, -1.0],
];

#[test]
fn a_five_cluster_expands_at_order_four_by_its_own_definition() {
    let (w, t) = tables();
    let reg = SurfaceRegistry::new().with(w).with(t);
    let pairs = AbInitioPairs;

    // E_MBE4 - E_MBE3 is EXACTLY the sum of the five four-subclusters' own terms (the
    // chemistry fold seeds each group on its first term and adds the groups, so the
    // difference of the two folds is the higher group's seeded sum, bit for bit).
    let e3 = mbe_energy(3, &OHHHH, &G5, &pairs, &reg).expect("triples served");
    let e4 = mbe_energy(4, &OHHHH, &G5, &pairs, &reg).expect("triples served");
    let terms = mbe_terms(4, &OHHHH, &G5, &pairs, &reg, false).expect("triples served");
    assert_eq!(terms.higher.len(), 5, "C(5,4) four-subclusters");
    let mut higher = terms.higher[0].v;
    for h in &terms.higher[1..] {
        higher += h.v;
    }
    // Stated as the fold actually associates — `E_MBE4 = E_MBE3 + (four-body group)` —
    // and not as a subtraction, which floating point does not undo exactly.
    assert_eq!(e4.to_bits(), (e3 + higher).to_bits(), "E_MBE4 is E_MBE3 plus the four-body group");

    // Each four-subcluster's term is its own referee's `eps_4` on that subcluster, bit for
    // bit: for the four that carry the oxygen, the OHHH referee the certified tables were
    // built from; for the all-hydrogen one, the same body term on `(H, H, H, H)`.
    for (k, slots) in subsets_lex(5, 4).iter().enumerate() {
        let sub: [[f64; 3]; 4] = core::array::from_fn(|i| G5[slots[i]]);
        assert_eq!(terms.higher[k].slots, *slots);
        let referee = if slots[0] == 0 {
            de4_ohhh_fci(&sub, w, t)
        } else {
            body_term(&[HYDROGEN; 4], &sub, &pairs, &reg).expect("(H,H,H) served")
        };
        assert_eq!(
            terms.higher[k].v.to_bits(),
            referee.to_bits(),
            "subcluster {slots:?}: the recursive term is the subcluster's own referee"
        );
    }

    // The five-cluster's own connected term is E_FCI minus its order-four expansion, and
    // it is finite and small against the total: a five-body correction, not a re-solve.
    let eps5 = body_term(&OHHHH, &G5, &pairs, &reg).expect("triples served");
    let e_fci = cluster_fci_energy(&OHHHH, &G5);
    assert_eq!((e_fci - e4).to_bits(), eps5.to_bits());
    assert!(eps5.is_finite());
    assert!(
        eps5.abs() < 0.05 * e_fci.abs(),
        "eps_5 = {eps5:.6e} Ha against E_FCI = {e_fci:.6e} Ha is not a correction"
    );
    println!("OHHHH: E_FCI {e_fci:.9} E_MBE3 {e3:.9} E_MBE4 {e4:.9} eps5 {eps5:.3e}");

    // The order is clamped to N - 1: asking for order five on a five-cluster is the
    // order-four expansion, not a term of the cluster against itself.
    let e5 = mbe_energy(5, &OHHHH, &G5, &pairs, &reg).expect("triples served");
    assert_eq!(e5.to_bits(), e4.to_bits());
    let _ = OHHH;
}

#[test]
fn a_missing_three_body_family_refuses_the_whole_expansion() {
    let (w, _) = tables();
    let only_water = SurfaceRegistry::new().with(w);
    // The (H, H, H) triple of an OHHH cluster has no family here: refused, not zeroed.
    assert!(mbe_energy(3, &OHHH, &[G5[0], G5[1], G5[2], G5[3]], &AbInitioPairs, &only_water).is_none());
    assert!(body_term(&OHHH, &[G5[0], G5[1], G5[2], G5[3]], &AbInitioPairs, &only_water).is_none());
    // and at order two nothing three-body is asked for, so the same registry serves it
    assert!(mbe_energy(2, &OHHH, &[G5[0], G5[1], G5[2], G5[3]], &AbInitioPairs, &only_water).is_some());
}
