//! **Symmetry orbit reduction: one solve per orbit, the rest mirrored.**
//!
//! The folded generator solves every node of its grid. The hand-rolled generators it
//! replaced did not — each of them walks only the `i <= j` half of its own grid and mirrors
//! the same float into the other half — so the fold bought DRY at the price of a factor of
//! two on the 3-body tables and a factor of six on the 4-body one. This file is the
//! acceptance argument for buying it back once, in the mesh, for any symmetry:
//!
//! * **(a)** the number of nodes actually SOLVED is the Burnside orbit count of the grid —
//!   asserted against the closed form AND against a direct enumeration, because a reduction
//!   that skipped a few extra nodes would also "save" solves;
//! * **(b)** every mirrored node's energy bits are its representative's, EXACTLY;
//! * **(c)** the table is bit-identical at 1, 4 and 8 workers — the warm chains in pass 1
//!   are still a pure function of the grid and the symmetry, and the fill in pass 2 is pure
//!   memory in canonical order;
//! * **(d)** a node and every one of its six relabellings carry the identical value.
//!
//! # M-VACUOUS-SUCCESS: (b) and (d) are free if the symmetry is fictional
//!
//! A generator that mirrored ARBITRARY nodes would pass (b) and (d) trivially — a copy is a
//! copy whatever it copies. So [`the_declared_symmetry_is_the_surfaces_real_symmetry`] runs
//! the SAME grid with NO symmetry declared, solves all 4096 nodes independently, and asserts
//! that the six relabellings of each node agree on the physics anyway. That is the test that
//! makes the other three mean something, and it is also the control for the acceptance bar:
//! with no symmetry declared, nothing is mirrored and the generator does what it always did.

use holon_chem::elements::{by_symbol, Species};
use holon_chem::quaternary_table::{
    canonical_index, elliptope_scale, embed_ohhh, gram_det, pair_slot, NR, NU, PERMS, R_HI, R_LO,
    STRETCH_A, U_HI, U_LO,
};
use holon_chem::trimer::TrimerTable;
use holon_chem::water::WaterTable;

use holon_tables::generate::generate_surface;
use holon_tables::grid::{Axis, NdGrid, NodeId, Serpentine};
use holon_tables::ohhh::OhhhSurface;
use holon_tables::surface::{min_separation, MIN_SEPARATION};
use holon_tables::{NodeRecord, Realised, Surface, SurfaceSpec};

// ---------------------------------------------------------------------------------------
// The cheap stand-in
// ---------------------------------------------------------------------------------------

/// The `(O,H,H,H)` surface's COORDINATES and SYMMETRY over four HYDROGENS.
///
/// A real `(O,H,H,H)` node is a 1,568-determinant FCI solve; four hydrogens is 36. The
/// mesh's behaviour under a declared symmetry is what is under test here and it does not
/// depend on which nuclei are at the centres, so the stand-in buys two orders of magnitude
/// of test time and nothing is weakened — the real [`OhhhSurface`] is exercised on its own
/// grid in [`the_real_ohhh_surface_delegates_its_symmetry`], which is where the delegation
/// is actually proved.
///
/// `realise` is [`OhhhSurface::realise`]'s shape (elliptope continuation included) so the
/// Geometry, Continued and Refused branches are all genuinely taken; `symmetric` selects
/// whether the surface declares the relabelling orbit or leaves the trait's default, which
/// is how the control arm is built without a second copy of anything.
struct CheapOhhh {
    species: [Species; 4],
    symmetric: bool,
}

impl CheapOhhh {
    fn new(symmetric: bool) -> CheapOhhh {
        let h = by_symbol("H").unwrap();
        CheapOhhh { species: [h; 4], symmetric }
    }
}

impl Surface for CheapOhhh {
    fn species(&self) -> &[Species] {
        &self.species
    }

    fn dim(&self) -> usize {
        6
    }

    fn realise(&self, coords: &[f64]) -> Realised {
        assert_eq!(coords.len(), 6);
        let r = [coords[0], coords[1], coords[2]];
        let u = [coords[3], coords[4], coords[5]];
        if r.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return Realised::Refused;
        }
        let inside = gram_det(u) >= 0.0;
        let (uu, _t) = if inside { (u, 1.0) } else { elliptope_scale(u) };
        let centers = match embed_ohhh(r, uu) {
            Some(c) => c.to_vec(),
            None => return Realised::Refused,
        };
        if min_separation(&centers) < MIN_SEPARATION {
            return Realised::Refused;
        }
        if inside {
            Realised::Geometry(centers)
        } else {
            Realised::Continued {
                centers,
                from: vec![r[0], r[1], r[2], uu[0], uu[1], uu[2]],
            }
        }
    }

    fn subtract(&self, _coords: &[f64], e_total: f64) -> f64 {
        e_total
    }

    fn basis(&self) -> &'static str {
        // A test fixture, and it says so rather than borrowing a production basis string —
        // a manifest line copied from a stand-in is how a fixture's identity leaks into a
        // real artifact's.
        "test fixture: stores whatever the cheap surrogate returned"
    }

    fn canonical(&self, idx: &[usize]) -> Vec<usize> {
        if !self.symmetric {
            return idx.to_vec();
        }
        let (i, k) = canonical_index([idx[0], idx[1], idx[2]], [idx[3], idx[4], idx[5]]);
        vec![i[0], i[1], i[2], k[0], k[1], k[2]]
    }
}

// ---------------------------------------------------------------------------------------
// The grid, the orbit count, and the relabelling
// ---------------------------------------------------------------------------------------

/// `nr` nodes on each of the three radial axes and `nu` on each of the three cosine axes,
/// on `quaternary_table`'s own frozen box and under its own axis maps.
///
/// The three radial axes are identical to each other and so are the three cosine axes.
/// That is not decoration: relabelling moves indices BETWEEN axes, so it is the condition
/// under which a relabelled index tuple names the same geometry at all. It is asserted in
/// [`the_orbit_grids_axes_are_uniform`] rather than left to the reader.
fn small_grid(nr: usize, nu: usize) -> NdGrid {
    let rr = if nr >= 2 { 2 } else { 1 };
    let ru = if nu >= 2 { 2 } else { 1 };
    NdGrid::new(vec![
        Axis::stretched(nr, R_LO, R_HI, STRETCH_A, rr),
        Axis::stretched(nr, R_LO, R_HI, STRETCH_A, rr),
        Axis::stretched(nr, R_LO, R_HI, STRETCH_A, rr),
        Axis::linear(nu, U_LO, U_HI, ru),
        Axis::linear(nu, U_LO, U_HI, ru),
        Axis::linear(nu, U_LO, U_HI, ru),
    ])
    // A new surface takes the rule that is actually adjacent; see `Serpentine`.
    .with_serpentine(Serpentine::Reflected)
}

/// Burnside's lemma for `S3` relabelling the three hydrogens of an `nr^3 x nu^3` grid.
///
/// The identity fixes everything; each of the three transpositions fixes the tuples with two
/// radial indices equal and the two cosines it swaps equal (`nr^2 nu^2`); each of the two
/// 3-cycles fixes the tuples with all three radial indices equal and all three cosines equal
/// (`nr nu`). Written out because it is the EXPECTED value the test is judged against, and a
/// count derived from the implementation would be no test at all.
fn burnside(nr: usize, nu: usize) -> usize {
    (nr * nr * nr * nu * nu * nu + 3 * nr * nr * nu * nu + 2 * nr * nu) / 6
}

/// The node index tuple relabelled by `sigma`: the radial index of hydrogen `a` and the
/// cosine index of the pair `{a, b}` move TOGETHER.
fn relabel(idx: &[usize], sigma: [usize; 3]) -> Vec<usize> {
    let (i, k) = ([idx[0], idx[1], idx[2]], [idx[3], idx[4], idx[5]]);
    vec![
        i[sigma[0]],
        i[sigma[1]],
        i[sigma[2]],
        k[pair_slot(sigma[0], sigma[1])],
        k[pair_slot(sigma[1], sigma[2])],
        k[pair_slot(sigma[2], sigma[0])],
    ]
}

/// The value fields of a record, as the bits they are compared by: this is WHAT the table
/// says, with nothing about how it was reached.
fn value_of(r: &NodeRecord) -> (u64, u64, u64, u64) {
    (r.energy_bits, r.d1_bits, r.d2_bits, r.status_code())
}

const NR_T: usize = 4;
const NU_T: usize = 4;

// ---------------------------------------------------------------------------------------
// The tests
// ---------------------------------------------------------------------------------------

/// The precondition the whole file rests on: the axes an orbit permutes between are the same
/// axis. Asserted, because a ragged grid would break the reduction while every count below
/// still came out right.
#[test]
fn the_orbit_grids_axes_are_uniform() {
    let g = small_grid(NR_T, NU_T);
    for d in 1..3 {
        assert_eq!(g.axes[0], g.axes[d], "radial axis {d} differs from radial axis 0");
    }
    for d in 4..6 {
        assert_eq!(g.axes[3], g.axes[d], "cosine axis {d} differs from cosine axis 3");
    }
}

/// **(a) and (b).** The generator solves exactly one node per orbit, and every other node
/// carries its representative's bits.
#[test]
fn the_orbit_reduction_solves_one_node_per_orbit_and_mirrors_the_rest() {
    let surface = CheapOhhh::new(true);
    let grid = small_grid(NR_T, NU_T);
    let spec = SurfaceSpec::new(&surface, grid);
    let n_nodes = spec.grid.n_nodes();
    assert_eq!(n_nodes, NR_T.pow(3) * NU_T.pow(3));
    assert!(
        spec.grid.n_regions() >= 8,
        "the grid must cut into more regions than the worker count, or the reduction is \
         tested on a serial mesh"
    );

    let run = generate_surface(&spec, 4);
    let solved = run.records.len() - run.mirrored;

    // (a) THE ORBIT COUNT — against the closed form, and against a direct enumeration of the
    // representatives, so a reduction that dropped extra nodes could not agree with both.
    let enumerated = (0..n_nodes)
        .filter(|&n| {
            let idx = spec.grid.coords(n as NodeId);
            surface.canonical(&idx) == idx
        })
        .count();
    assert_eq!(
        burnside(NR_T, NU_T),
        enumerated,
        "Burnside's closed form and a direct enumeration of the representatives disagree"
    );
    assert_eq!(
        solved,
        burnside(NR_T, NU_T),
        "{solved} nodes were solved where the grid has {} orbits",
        burnside(NR_T, NU_T)
    );
    assert_eq!(run.mirrored, n_nodes - burnside(NR_T, NU_T));
    assert_eq!(
        run.cold_solves + run.warm_solves,
        solved,
        "the solve accounting does not add up to the solved nodes"
    );
    // M-VACUOUS-SUCCESS in both directions: a run that mirrored everything, or nothing, would
    // satisfy some of the checks below by having nothing to check.
    assert!(run.mirrored > 0, "nothing was mirrored, so this test says nothing");
    assert!(solved > 0 && solved < n_nodes, "the reduction is degenerate");
    assert!(
        run.records.iter().any(|r| r.is_ok()),
        "not one node of the grid was a geometry that scored"
    );
    assert!(run.certificate.is_clean(), "{:?}", run.certificate);

    // (b) EVERY MIRROR CARRIES ITS REPRESENTATIVE'S BITS — and the record says it is a mirror
    // rather than a solve, so "how many nodes did this grid actually solve" is answerable
    // from the table itself.
    let mut checked = 0usize;
    for (n, r) in run.records.iter().enumerate() {
        assert_eq!(r.node as usize, n, "the table is not in canonical node order");
        let idx = spec.grid.coords(n as NodeId);
        let rep = spec.grid.node_id(&surface.canonical(&idx));
        if rep == n as NodeId {
            assert!(!r.mirrored, "node {n} is its own representative and is marked mirrored");
            continue;
        }
        assert!(r.mirrored, "node {n} is not its own representative and was not marked mirrored");
        let src = &run.records[rep as usize];
        assert_eq!(
            r.energy_bits, src.energy_bits,
            "node {n} was mirrored from node {rep} and its energy bits differ; the fill is a \
             recomputation, not a copy"
        );
        // The whole record, not only the energy: a mirror is the representative's record
        // readdressed, and `mirrored` is the only field that may differ from it.
        assert_eq!(
            NodeRecord { node: src.node, mirrored: src.mirrored, ..*r },
            *src,
            "node {n}'s mirror differs from node {rep}'s record in more than its address"
        );
        checked += 1;
    }
    assert_eq!(checked, run.mirrored);

    println!(
        "orbit reduction on a {NR_T}^3 x {NU_T}^3 grid: {n_nodes} box -> {solved} solved \
         ({} mirrored), {:.3}x fewer solves; {} scored, digest {}",
        run.mirrored,
        n_nodes as f64 / solved as f64,
        run.records.iter().filter(|r| r.is_ok()).count(),
        run.digest().hex()
    );
}

/// **(c) and (d).** The mirrored table is bit-identical at 1, 4 and 8 workers, and every one
/// of a node's six relabellings carries the identical value.
///
/// (c) is the property the two-pass structure exists for. A node's representative generally
/// lives in a DIFFERENT region owned by a DIFFERENT worker, so an in-line fill would have to
/// read a slot that might not be written yet — and any repair of that by waiting would make
/// the table a function of the schedule, which is the one thing this crate forbids.
#[test]
fn the_mirrored_table_is_bit_identical_across_worker_counts() {
    let surface = CheapOhhh::new(true);
    let spec = SurfaceSpec::new(&surface, small_grid(NR_T, NU_T));

    let one = generate_surface(&spec, 1);
    let four = generate_surface(&spec, 4);
    let many = generate_surface(&spec, 8);

    for (label, run) in [("1", &one), ("4", &four), ("8", &many)] {
        assert!(
            run.certificate.is_clean(),
            "the {label}-worker run's certificate was not clean: {:?}",
            run.certificate
        );
        assert_eq!(run.records.len(), spec.grid.n_nodes());
        assert_eq!(run.mirrored, one.mirrored, "the {label}-worker run mirrored a different set");
        assert!(run.total_davidson_iters > 0, "the {label}-worker run did no Davidson work");
    }
    // `table_bytes` covers the iteration counts too, so this also says the warm chains in
    // pass 1 did not move with the worker count.
    assert_eq!(one.table_bytes(), four.table_bytes(), "1 and 4 workers disagree");
    assert_eq!(one.table_bytes(), many.table_bytes(), "1 and 8 workers disagree");
    assert_eq!(one.digest(), four.digest());
    assert_eq!(one.digest(), many.digest());
    assert_eq!(one.cold_solves, four.cold_solves, "the cold-seed count moved with the workers");
    assert_eq!(one.warm_solves, four.warm_solves, "the warm chain moved with the workers");

    // (d) EVERY RELABELLING. Not a sample: all six images of all 4096 nodes.
    let mut compared = 0usize;
    let mut nontrivial = 0usize;
    for n in 0..spec.grid.n_nodes() {
        let idx = spec.grid.coords(n as NodeId);
        let here = value_of(&one.records[n]);
        for &s in PERMS.iter() {
            let image = spec.grid.node_id(&relabel(&idx, s));
            assert_eq!(
                value_of(&one.records[image as usize]),
                here,
                "node {n} and its relabelling {s:?} (node {image}) carry different values"
            );
            compared += 1;
            if image != n as NodeId {
                nontrivial += 1;
            }
        }
    }
    assert!(
        nontrivial > 0,
        "every relabelling was the identity, so the orbit check was vacuous"
    );
    println!(
        "bit-identical at 1/4/8 workers over {} nodes ({} solved, {} mirrored); {compared} \
         relabellings compared, {nontrivial} of them non-trivial; digest {}",
        one.records.len(),
        one.records.len() - one.mirrored,
        one.mirrored,
        one.digest().hex()
    );
}

/// **M-VACUOUS-SUCCESS, and the acceptance bar's control.** With NO symmetry declared the
/// generator solves every node — and the six relabellings then agree on the PHYSICS anyway,
/// which is what makes the orbit map a symmetry rather than an arbitrary pairing.
///
/// Without this, the mirror checks above are satisfied by any implementation that copies any
/// node onto any other: a copy is a copy whatever it copies.
#[test]
fn the_declared_symmetry_is_the_surfaces_real_symmetry() {
    let blind = CheapOhhh::new(false);
    let spec = SurfaceSpec::new(&blind, small_grid(NR_T, NU_T));
    let n_nodes = spec.grid.n_nodes();

    let run = generate_surface(&spec, 4);
    // THE ACCEPTANCE BAR, on this path: no symmetry declared, nothing mirrored, every node
    // solved exactly as before.
    assert_eq!(run.mirrored, 0, "a surface that declared no symmetry mirrored {} nodes", run.mirrored);
    assert_eq!(run.cold_solves + run.warm_solves, n_nodes);
    assert!(run.records.iter().all(|r| !r.mirrored));
    assert!(run.certificate.is_clean());

    // Now the physics. Each node here was solved on its own, from its own warm start, at its
    // own embedding; congruent geometries of identical nuclei must agree to solver noise.
    // The crate's own measurement puts warm-vs-cold scatter at 3.4e-13 to 4.3e-12 hartree, so
    // this floor is five orders above the noise and eight below the 7.47 hartree a trapped
    // solve misses by.
    const AGREEMENT: f64 = 1e-7;
    let mut worst = 0.0f64;
    let mut compared = 0usize;
    let mut scored = 0usize;
    for n in 0..n_nodes {
        let idx = spec.grid.coords(n as NodeId);
        let here = &run.records[n];
        for &s in PERMS.iter() {
            let image = spec.grid.node_id(&relabel(&idx, s));
            if image == n as NodeId {
                continue;
            }
            let there = &run.records[image as usize];
            assert_eq!(
                here.status_code(),
                there.status_code(),
                "node {n} and its relabelling {s:?} (node {image}) reached different statuses \
                 ({:?} vs {:?}); the orbit map names points the surface does not agree about",
                here.status,
                there.status
            );
            compared += 1;
            if here.is_ok() {
                scored += 1;
                let d = (here.energy() - there.energy()).abs();
                worst = worst.max(d);
            }
        }
    }
    assert!(compared > 0, "no non-trivial relabelling existed, so this proves nothing");
    assert!(scored > 0, "no relabelled pair scored, so no energy was ever compared");
    assert!(
        worst < AGREEMENT,
        "two independently-solved relabellings of one geometry differ by {worst:.3e} Ha. The \
         declared orbit map is NOT a symmetry of this surface, and every mirrored table built \
         on it is wrong while looking right."
    );
    println!(
        "the symmetry is real: {compared} relabelled pairs solved independently, {scored} of \
         them scored, worst energy disagreement {worst:.3e} Ha (floor {AGREEMENT:.0e})"
    );
}

/// **The delegation, on the real surface.** [`OhhhSurface::canonical`] IS
/// `quaternary_table::canonical_index` — asserted over the whole production index box, not a
/// sample — and a real `(O,H,H,H)` run through the leased generator reduces by it.
#[test]
fn the_real_ohhh_surface_delegates_its_symmetry() {
    let s = OhhhSurface::new(WaterTable::empty(), TrimerTable::empty(), R_LO, R_HI);

    // The map itself, over every index tuple of a grid big enough to have all three orbit
    // sizes (1, 3 and 6) in it.
    let mut agreed = 0usize;
    for i0 in 0..NR_T {
        for i1 in 0..NR_T {
            for i2 in 0..NR_T {
                for k0 in 0..NU_T {
                    for k1 in 0..NU_T {
                        for k2 in 0..NU_T {
                            let (ci, ck) = canonical_index([i0, i1, i2], [k0, k1, k2]);
                            assert_eq!(
                                s.canonical(&[i0, i1, i2, k0, k1, k2]),
                                vec![ci[0], ci[1], ci[2], ck[0], ck[1], ck[2]],
                                "OhhhSurface::canonical is not canonical_index at \
                                 ({i0},{i1},{i2},{k0},{k1},{k2}); it is a second copy of the \
                                 symmetry and it has already drifted"
                            );
                            agreed += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(agreed, NR_T.pow(3) * NU_T.pow(3));

    // And a real run: (O,H,H,H) at 1,568 determinants on the smallest grid that still has a
    // non-trivial orbit structure.
    let spec = SurfaceSpec::new(&s, small_grid(2, 2));
    let run = generate_surface(&spec, 4);
    let solved = run.records.len() - run.mirrored;
    assert_eq!(run.records.len(), 64);
    assert_eq!(
        solved,
        burnside(2, 2),
        "the real (O,H,H,H) surface solved {solved} of 64 nodes where the grid has {} orbits",
        burnside(2, 2)
    );
    assert!(run.certificate.is_clean(), "{:?}", run.certificate);
    assert!(run.mirrored > 0 && solved > 0);
    for (n, r) in run.records.iter().enumerate() {
        if !r.mirrored {
            continue;
        }
        let rep = spec.grid.node_id(&s.canonical(&spec.grid.coords(n as NodeId)));
        assert_eq!(
            r.energy_bits, run.records[rep as usize].energy_bits,
            "node {n} was mirrored from node {rep} and the bits differ"
        );
    }
    println!(
        "real OhhhSurface: 64 box -> {solved} solved, {} mirrored ({} scored); digest {}",
        run.mirrored,
        run.records.iter().filter(|r| r.is_ok()).count(),
        run.digest().hex()
    );
}

/// The measurement the reduction is worth on the shape that is actually going to be
/// generated: `quaternary_table`'s frozen `NR = 13`, `NU = 11` domain.
///
/// Counted by enumeration rather than by the closed form, and then checked against the
/// closed form — two routes to one number, because a saving reported from the same code that
/// produces it is not a measurement.
#[test]
fn the_production_shape_reduction_is_measured() {
    let box_nodes = NR.pow(3) * NU.pow(3);
    let mut reps = 0usize;
    for i0 in 0..NR {
        for i1 in 0..NR {
            for i2 in 0..NR {
                for k0 in 0..NU {
                    for k1 in 0..NU {
                        for k2 in 0..NU {
                            if canonical_index([i0, i1, i2], [k0, k1, k2])
                                == ([i0, i1, i2], [k0, k1, k2])
                            {
                                reps += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(box_nodes, 2_924_207, "the frozen domain is not the shape this measures");
    assert_eq!(
        reps,
        burnside(NR, NU),
        "the enumerated representative count and Burnside's closed form disagree"
    );
    assert_eq!(reps, 497_640);
    println!(
        "production shape NR={NR} NU={NU}: {box_nodes} box nodes -> {reps} representatives, \
         {:.4}x fewer FCI solves ({} nodes filled from an orbit)",
        box_nodes as f64 / reps as f64,
        box_nodes - reps
    );
}
