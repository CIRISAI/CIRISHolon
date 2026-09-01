//! **The acceptance argument for the WB-8.7 fold.**
//!
//! The 3-body tables are gated on bit-identity, and after the fold the 3-axis path runs
//! through [`NdGrid`] rather than through [`TableGrid`]'s own methods. That is only safe if
//! the two agree EXACTLY — not approximately, not on a sample, and not on the four functions
//! somebody remembered to check.
//!
//! So this file checks all six canonical functions on every node of every grid in a sweep of
//! extents and region shapes, with `f64` compared as raw bits. If a single ulp of a single
//! coordinate moved, the whole campaign's committed tables would move with it, and the run
//! that discovered it would be an expensive one. This is the cheap one.
//!
//! The second half is the new capability the first half licenses: a 6-axis grid, where the
//! partition must still be a partition and the traversal must still walk neighbour to
//! neighbour.

use holon_tables::grid::{Axis, AxisMap, NdGrid, NodeId, RegionId, Serpentine, TableGrid};

fn table_grid(nx: usize, ny: usize, nu: usize, region: [usize; 3]) -> TableGrid {
    // A deliberately un-round box: nothing here should be exact in binary, so that a
    // difference in the ORDER of the interpolant's multiply and divide shows up as a
    // difference in the bits rather than cancelling.
    TableGrid::new(
        nx,
        ny,
        nu,
        region,
        (1.3, 3.7),
        (0.9, 4.1),
        (-0.85, 0.55),
    )
}

/// **THE GATE.** `NdGrid::from_table_grid` reproduces `TableGrid` bit for bit, node by node,
/// on every canonical function, over a sweep of extents and region shapes.
#[test]
fn nd_grid_reproduces_table_grid_exactly() {
    let dims = [
        (1, 1, 1),
        (4, 4, 2), // the leased gate's shape
        (2, 2, 2),
        (5, 3, 7),
        (9, 2, 2),
        (3, 3, 3),
        (6, 5, 4),
        (7, 4, 5),
        (2, 9, 3),
        (33, 3, 13), // the trimer table's own axis sizes, cut down in one coordinate
    ];
    let regions = [
        [1, 1, 1],
        [2, 2, 2], // the production region shape
        [3, 1, 2],
        [3, 5, 4],
        [4, 4, 4],
        [9, 9, 9],
        [2, 3, 5],
    ];

    let mut checked_nodes = 0usize;
    let mut checked_grids = 0usize;

    for (nx, ny, nu) in dims {
        for region in regions {
            let t = table_grid(nx, ny, nu, region);
            let g = NdGrid::from_table_grid(&t);
            checked_grids += 1;

            assert_eq!(g.dim(), 3);
            assert_eq!(
                g.n_nodes(),
                t.n_nodes(),
                "n_nodes disagrees at {nx}x{ny}x{nu} region {region:?}"
            );
            assert_eq!(
                g.region_extents(),
                vec![t.region_extents().0, t.region_extents().1, t.region_extents().2],
                "region_extents disagrees at {nx}x{ny}x{nu} region {region:?}"
            );
            assert_eq!(
                g.n_regions(),
                t.n_regions(),
                "n_regions disagrees at {nx}x{ny}x{nu} region {region:?}"
            );

            // Every node: the index round trip, the region label, and the geometry BITS.
            for i in 0..nx {
                for j in 0..ny {
                    for k in 0..nu {
                        let want = t.node_id(i, j, k);
                        let got = g.node_id(&[i, j, k]);
                        assert_eq!(
                            got, want,
                            "node_id({i},{j},{k}) disagrees at {nx}x{ny}x{nu} region {region:?}"
                        );

                        let tc = t.coords(want);
                        assert_eq!(
                            g.coords(want),
                            vec![tc.0, tc.1, tc.2],
                            "coords({want}) disagrees at {nx}x{ny}x{nu} region {region:?}"
                        );

                        let (x, y, u) = t.geometry(want);
                        let ndg = g.geometry(want);
                        // BITS, not approximate equality. The whole point is that a
                        // reassociated interpolant would still be "equal" to 1e-15 and would
                        // still invalidate every committed table.
                        assert_eq!(
                            [ndg[0].to_bits(), ndg[1].to_bits(), ndg[2].to_bits()],
                            [x.to_bits(), y.to_bits(), u.to_bits()],
                            "geometry({want}) differs IN BITS at {nx}x{ny}x{nu} region \
                             {region:?}: NdGrid gave {ndg:?}, TableGrid gave {:?}",
                            (x, y, u)
                        );

                        assert_eq!(
                            g.region_of(want),
                            t.region_of(want),
                            "region_of({want}) disagrees at {nx}x{ny}x{nu} region {region:?}"
                        );
                        checked_nodes += 1;
                    }
                }
            }

            // Every region's traversal ORDER, which is what decides every warm start.
            for r in 0..t.n_regions() as RegionId {
                assert_eq!(
                    g.region_nodes(r),
                    t.region_nodes(r),
                    "region_nodes({r}) disagrees at {nx}x{ny}x{nu} region {region:?}"
                );
            }
            assert_eq!(
                g.partition(),
                t.partition(),
                "the partition disagrees at {nx}x{ny}x{nu} region {region:?}"
            );
        }
    }

    // M-VACUOUS-SUCCESS: a sweep that checked nothing would pass.
    assert!(checked_grids >= 70, "only {checked_grids} grids were compared");
    assert!(checked_nodes >= 5_000, "only {checked_nodes} nodes were compared");
    println!(
        "NdGrid == TableGrid bit for bit over {checked_grids} grids and {checked_nodes} nodes \
         (node_id, coords, geometry-bits, region_of, region_nodes, partition)"
    );
}

/// The exponential stretch reproduces `holon_chem::water::r_of_tau`'s own expression, which
/// is what a folded water/ooh/ozone axis has to be.
#[test]
fn exp_stretch_is_the_chem_crates_r_of_tau() {
    const A: f64 = 3.0;
    let (lo, hi, n) = (1.6f64, 14.0f64, 65usize);
    let axis = Axis::stretched(n, lo, hi, A, 8);
    for i in 0..n {
        let tau = i as f64 / (n - 1) as f64;
        let want = lo + (hi - lo) * ((A * tau).exp() - 1.0) / (A.exp() - 1.0);
        assert_eq!(
            axis.coord(i).to_bits(),
            want.to_bits(),
            "stretched node {i} differs in bits: {} vs {want}",
            axis.coord(i)
        );
    }
    // The endpoints are the box, exactly.
    assert_eq!(axis.coord(0), lo);
    assert!((axis.coord(n - 1) - hi).abs() < 1e-12);
    // And it really does stretch: the first step is far shorter than the last.
    let first = axis.coord(1) - axis.coord(0);
    let last = axis.coord(n - 1) - axis.coord(n - 2);
    assert!(last > 5.0 * first, "the stretch is not stretching: {first} vs {last}");
}

// ---------------------------------------------------------------------------
// Six axes: the shape the fold exists for
// ---------------------------------------------------------------------------

/// The six-axis grid used below: a 4-body distance box, small enough to enumerate whole.
///
/// Region extents on the INTERIOR axes are odd on purpose — see
/// [`the_sum_parity_serpentine_is_not_unconditionally_adjacent`] for the reason, which is a
/// property of the legacy rule rather than of this grid.
fn six_axis_grid(serpentine: Serpentine) -> NdGrid {
    NdGrid::new(vec![
        Axis::linear(4, 1.4, 3.0, 2),
        Axis { n: 3, lo: 1.4, hi: 3.0, map: AxisMap::ExpStretch { a: 2.0 }, region: 3 },
        Axis::linear(3, 1.4, 3.0, 1),
        Axis::linear(6, 1.4, 3.0, 3),
        Axis::linear(3, 1.4, 3.0, 1),
        Axis::linear(4, 1.4, 3.0, 2),
    ])
    .with_serpentine(serpentine)
}

/// **Adjacency on six axes.** Consecutive nodes in a region traversal differ in exactly one
/// coordinate, by exactly one — the property that makes a warm start a good guess, and the
/// whole reason for the serpentine.
#[test]
fn six_axis_traversal_steps_are_l1_adjacent() {
    for serpentine in [Serpentine::SumParity, Serpentine::Reflected] {
        let g = six_axis_grid(serpentine);
        assert!(
            g.adjacency_is_guaranteed(),
            "{serpentine:?}: this grid does not satisfy the rule's own adjacency condition, so \
             the assertion below would be testing the grid rather than the traversal"
        );
        let mut steps = 0usize;
        for r in 0..g.n_regions() as RegionId {
            let nodes = g.region_nodes(r);
            assert!(!nodes.is_empty(), "region {r} is empty");
            for w in nodes.windows(2) {
                let a = g.coords(w[0]);
                let b = g.coords(w[1]);
                let d: usize = a.iter().zip(b.iter()).map(|(x, y)| x.abs_diff(*y)).sum();
                assert_eq!(
                    d, 1,
                    "{serpentine:?}: traversal step {a:?} -> {b:?} is not adjacent (L1 {d}); \
                     every node but the seed would not be warm-started from its neighbour"
                );
                steps += 1;
            }
        }
        // M-VACUOUS-SUCCESS: a grid of singleton regions has no steps to check.
        assert!(steps > 100, "{serpentine:?}: only {steps} traversal steps were checked");
        println!("{serpentine:?}: {steps} six-axis traversal steps, all L1-adjacent");
    }
}

/// **The reflected rule is adjacent on ANY region shape**, including the even interior
/// extents the legacy rule cannot handle. This is the property a new surface should rely on.
#[test]
fn the_reflected_serpentine_is_adjacent_on_even_region_shapes() {
    let g = NdGrid::new(vec![
        Axis::linear(4, 1.0, 2.0, 2),
        Axis::linear(4, 1.0, 2.0, 2),
        Axis::linear(4, 1.0, 2.0, 2),
        Axis::linear(4, 1.0, 2.0, 2),
        Axis::linear(2, 1.0, 2.0, 2),
        Axis::linear(2, 1.0, 2.0, 2),
    ])
    .with_serpentine(Serpentine::Reflected);
    assert!(g.adjacency_is_guaranteed());
    let mut steps = 0usize;
    for r in 0..g.n_regions() as RegionId {
        for w in g.region_nodes(r).windows(2) {
            let (a, b) = (g.coords(w[0]), g.coords(w[1]));
            let d: usize = a.iter().zip(b.iter()).map(|(x, y)| x.abs_diff(*y)).sum();
            assert_eq!(d, 1, "reflected step {a:?} -> {b:?} is not adjacent (L1 {d})");
            steps += 1;
        }
    }
    assert!(steps > 100, "only {steps} steps checked");
}

/// **The legacy rule is NOT unconditionally adjacent, and the production region shape is
/// exactly a case where it is not.**
///
/// `TableGrid::region_nodes` documents the serpentine as making "consecutive nodes in the
/// traversal always grid-adjacent". That is true only when every axis strictly between the
/// first and the last has odd region extents: on a carry into axis `d`, a later axis `e`
/// keeps its index still only if its reversal flag flips, and under the sum rule that flag's
/// parity moves by `1 + sum_{d<f<e}(len[f]-1)`.
///
/// The 3-axis production shape `[2, 2, 2]` has `len[1] = 2`, so it fails, and the existing
/// `grid::tests::traversal_steps_are_adjacent` passes only because it happens to use
/// `[3, 5, 4]`, whose middle extent is odd. The defect is real but small — one
/// distance-2 step per `i`-plane fold — and it is NOT repaired on the legacy path, because
/// the committed tables were built with it and they are gated on bit-identity.
///
/// This test PINS that, so the claim in the doc comment cannot be read as unconditional and
/// nobody rediscovers it from a table that will not reproduce.
#[test]
fn the_sum_parity_serpentine_is_not_unconditionally_adjacent() {
    let t = TableGrid::new(4, 4, 2, [2, 2, 2], (1.6, 2.2), (1.8, 2.4), (0.1, 0.5));
    let g = NdGrid::from_table_grid(&t);
    assert!(
        !g.adjacency_is_guaranteed(),
        "the production region shape now claims guaranteed adjacency; if the legacy rule was \
         repaired, every committed table's warm-start chain moved with it"
    );

    let far: Vec<(Vec<usize>, Vec<usize>)> = (0..t.n_regions() as RegionId)
        .flat_map(|r| {
            let nodes = t.region_nodes(r);
            nodes
                .windows(2)
                .filter_map(|w| {
                    let a = t.coords(w[0]);
                    let b = t.coords(w[1]);
                    let d = a.0.abs_diff(b.0) + a.1.abs_diff(b.1) + a.2.abs_diff(b.2);
                    (d != 1).then(|| (vec![a.0, a.1, a.2], vec![b.0, b.1, b.2]))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(
        !far.is_empty(),
        "the legacy traversal is adjacent everywhere on [2,2,2] after all; then \
         adjacency_is_guaranteed is too pessimistic and the reflected rule is unnecessary"
    );
    // And the NdGrid fold reproduces the defect exactly, which is what bit-identity means.
    for r in 0..t.n_regions() as RegionId {
        assert_eq!(g.region_nodes(r), t.region_nodes(r));
    }
    // The reflected rule fixes it, on the same grid.
    let fixed = g.clone().with_serpentine(Serpentine::Reflected);
    for r in 0..fixed.n_regions() as RegionId {
        for w in fixed.region_nodes(r).windows(2) {
            let (a, b) = (fixed.coords(w[0]), fixed.coords(w[1]));
            let d: usize = a.iter().zip(b.iter()).map(|(x, y)| x.abs_diff(*y)).sum();
            assert_eq!(d, 1, "the reflected rule left a gap at {a:?} -> {b:?}");
        }
    }
    println!(
        "legacy [2,2,2] traversal has {} non-adjacent step(s), e.g. {:?} -> {:?}; the \
         reflected rule has none on the same grid",
        far.len(),
        far[0].0,
        far[0].1
    );
}

/// **The partition is a partition on six axes**: every node in exactly one region, exactly
/// once, and the region it is listed in is the region `region_of` names.
#[test]
fn six_axis_partition_covers_every_node_exactly_once() {
    for serpentine in [Serpentine::SumParity, Serpentine::Reflected] {
        for g in [
            six_axis_grid(serpentine),
            // Ragged: no region edge divides its axis, so every axis has a short last region.
            NdGrid::new(vec![
                Axis::linear(5, 1.0, 2.0, 2),
                Axis::linear(4, 1.0, 2.0, 3),
                Axis::linear(3, 1.0, 2.0, 2),
                Axis::linear(7, 1.0, 2.0, 4),
                Axis::linear(2, 1.0, 2.0, 3),
                Axis::linear(3, 1.0, 2.0, 2),
            ])
            .with_serpentine(serpentine),
            // Degenerate: a single region over the whole grid, and singleton regions.
            NdGrid::new(vec![
                Axis::linear(2, 1.0, 2.0, 9),
                Axis::linear(2, 1.0, 2.0, 9),
                Axis::linear(2, 1.0, 2.0, 9),
                Axis::linear(2, 1.0, 2.0, 1),
                Axis::linear(2, 1.0, 2.0, 1),
                Axis::linear(2, 1.0, 2.0, 1),
            ])
            .with_serpentine(serpentine),
        ] {
            let mut seen = vec![0u32; g.n_nodes()];
            let mut listed = 0usize;
            for r in 0..g.n_regions() as RegionId {
                let nodes = g.region_nodes(r);
                assert!(!nodes.is_empty(), "region {r} is empty; the partition is malformed");
                for n in nodes {
                    seen[n as usize] += 1;
                    listed += 1;
                    assert_eq!(
                        g.region_of(n),
                        r,
                        "node {n} was listed in region {r} but region_of says otherwise"
                    );
                }
            }
            assert_eq!(listed, g.n_nodes(), "the partition listed {listed} of {} nodes", g.n_nodes());
            assert!(
                seen.iter().all(|&c| c == 1),
                "{serpentine:?}: partition is not a partition ({} nodes not covered exactly \
                 once, of {})",
                seen.iter().filter(|&&c| c != 1).count(),
                g.n_nodes()
            );

            // The index round trip closes over the whole 6-axis grid.
            for id in 0..g.n_nodes() as NodeId {
                assert_eq!(g.node_id(&g.coords(id)), id, "node_id/coords do not round trip");
            }
        }
    }
}

/// The partition is a pure function of the grid — restated on six axes, because it is the
/// load-bearing property and it would be easy to break by adding a parameter.
#[test]
fn six_axis_partition_is_a_pure_function_of_the_grid() {
    let g = six_axis_grid(Serpentine::SumParity);
    assert_eq!(g.partition(), g.partition());
    let again = six_axis_grid(Serpentine::SumParity);
    assert_eq!(g.partition(), again.partition());
    // And the two serpentines really are different traversals of the same partition, so the
    // choice is not decoration.
    let refl = six_axis_grid(Serpentine::Reflected);
    let mut sorted_a: Vec<Vec<NodeId>> = g.partition();
    let mut sorted_b: Vec<Vec<NodeId>> = refl.partition();
    for v in sorted_a.iter_mut().chain(sorted_b.iter_mut()) {
        v.sort_unstable();
    }
    assert_eq!(sorted_a, sorted_b, "the two rules disagree about WHICH nodes are in a region");
}

/// The overflow guard fires rather than wrapping two nodes onto one slot.
#[test]
#[should_panic(expected = "does not fit in a u32 NodeId")]
fn a_grid_too_big_for_a_u32_node_id_is_refused() {
    // 6 axes of 200 nodes = 6.4e13, well past u32.
    NdGrid::new((0..6).map(|_| Axis::linear(200, 1.0, 2.0, 8)).collect());
}
