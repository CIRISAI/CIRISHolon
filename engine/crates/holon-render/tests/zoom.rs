//! THE ZOOM LAW'S ENGINE HALF — which reading nests when the observer sharpens.
//!
//! SELECTOR-4's Z1 staked the lead's zoom principle as "the selected set is
//! non-increasing as acuity refines" and the run killed it: for any criterion
//! that DEMANDS separations, refining the view can only grow the passing set,
//! and the quantity that actually nests is the IDENTITY set — what the
//! observer cannot yet tell apart. `lean/CIRISHolon/Zoom.lean` mechanizes the
//! direction (`ident_antitone`, `selected_monotone`, and a strict witness);
//! this file is the same law wearing atoms.
//!
//! The engine has two observers of one scene, one strictly sharper than the
//! other: BOUNDNESS (the cluster reading — components of the bonded-pair
//! graph) and CLOSURE (the census — a molecule row demands the pair's own
//! energy be autonomous, not merely negative). Closure separates everything
//! boundness separates and more, so under the corrected law the sharper
//! observer must read AT LEAST as many components: refinement splits, never
//! merges. The witness is D1's own tetramer finding, reconstructed: two H2
//! molecules parked ~6 bohr apart form ONE bound component (the tail is
//! still ~1e-4 Ha there, so a slow mutual approach keeps the cross pair
//! strictly bonded) and TWO closed molecules. 1 cluster, 2 rows — the count
//! grew with acuity, exactly the direction the staked Z1 forbade.

use holon_render::sim::{Boundary, Sim};

fn potential_source() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/viewer/h2_potential.json");
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        panic!("cannot read {path}: {e}. Run: cargo run -p holon-render --example make_placeholder")
    })
}

fn loaded_sim() -> Sim {
    let mut s = Sim::empty();
    holon_render::json::load_into(&mut s.table, &potential_source()).expect("table loads");
    s.adopt_table_timescale();
    s
}

/// Two parked molecules: the coarse observer reads one thing, the sharp
/// observer reads two, and the sharper count is the larger one.
#[test]
fn refining_the_observer_never_merges_what_it_could_already_split() {
    let mut s = loaded_sim();
    s.boundary = Boundary::Open;
    s.reset(4);
    let (cx, cy, cz) = (0.5 * s.width, 0.5 * s.height, 0.5 * s.depth);
    let r_e = s.table.r_e;
    // Pair A centred 3 bohr left of centre, pair B 3 bohr right; each at its
    // own equilibrium separation, drifting together at 5e-6 a.u. so the
    // cross pairs sit STRICTLY inside their turning points (an exactly-at-rest
    // pair sits ON its turning point and the strict criterion falls by solver
    // rounding — the lesson the cluster regression already carries).
    for (i, (dx, side)) in [
        (-3.0 - 0.5 * r_e, 1.0),
        (-3.0 + 0.5 * r_e, 1.0),
        (3.0 - 0.5 * r_e, -1.0),
        (3.0 + 0.5 * r_e, -1.0),
    ]
    .into_iter()
    .enumerate()
    {
        s.set_position_3d(i, cx + dx, cy, cz);
        s.set_velocity_3d(i, side * 5e-6, 0.0, 0.0);
    }
    s.rebase();
    // Enough grain boundaries for the census's dwell and closure history; the
    // approach covers < 1e-3 bohr over the run, so the geometry is static.
    for _ in 0..20 {
        s.step_frame(8);
    }

    let (clusters, in_clusters) = s.cluster_count();
    let molecules = s.holons.molecule_count();
    println!(
        "boundness reads {clusters} component ({in_clusters} atoms, {} bonded pairs); \
         closure reads {molecules} molecules; rejections {}",
        s.bonded_count(),
        s.holons.census.closure_rejections,
    );

    // The coarse observer: one bound component holding all four atoms (the
    // intramolecular pairs and at least one cross pair are all bonded).
    assert_eq!((clusters, in_clusters), (1, 4), "the parked pair must read as ONE bound component");
    // The sharp observer: two closed molecules — the cross pair is bound but
    // never autonomous, so it gets an edge and not a row.
    assert_eq!(molecules, 2, "closure must split what boundness merges");
    // The law itself, stated as the inequality the staked Z1 forbade: the
    // sharper observer's count is never the smaller one.
    assert!(
        molecules >= clusters,
        "refinement merged components: molecules {molecules} < clusters {clusters}"
    );
}
