//! **Does the mesh need §5.2's claim-based balancer? Measured, and the answer is "not yet".**
//!
//! `MESH_DESIGN.md` §5.2 says balance by CLAIM, never by geometry, citing
//! `Core/GrainFloor.lean::demand_not_function_of_geometry` — one tier, one size, one grain
//! serves one claim and refuses another, so two shards of identical geometry can carry wildly
//! different certification cost.
//!
//! That theorem says demand is not a function of geometry **in general**. Whether it is one
//! for a PARTICULAR scene is a measurement, and building a balancer before taking it would be
//! building a feature that cannot be tested. So it was taken:
//!
//! | claim shape | max/mean work per shard, 64 shards |
//! |---|---:|
//! | uniform (every cell steps every colour) — the mesh's scene today | **1.01 – 1.23** |
//! | corridor-local (the sandbox certifier's shape) | **2.37 – 7.61** |
//!
//! **Conclusion, and it is a decision not to build something.** Under a uniform claim a
//! geometric partition is already within ~1–23% of perfect, so a claim-based balancer is
//! premature and untestable on this scene. Under a corridor-local claim — which is what the
//! sandbox certifier actually produces (`SANDBOX_4090` §4 measured the contact frontier as
//! corridor-local) — geometric partitioning is **catastrophic**: one shard carries 7.6× the
//! mean, and since a perfect schedule's makespan is bounded below by its largest single shard,
//! **speedup is capped at 64/7.61 ≈ 8.4× no matter how many threads are given**.
//!
//! So the trigger is precise rather than vague: **build the balancer when the scene carries a
//! non-uniform claim.** Until then it would be dead code with a passing test.

use holon_mesh::{Grid, Mesh, MeshSpec, Partition};

/// Per-shard work under the mesh's current uniform claim: evaluable edges over a colour cycle.
fn uniform_work(grid: Grid, nx: usize, ny: usize, nz: usize) -> Vec<usize> {
    let mesh = Mesh::new(MeshSpec::new_3d(grid, nx, ny, nz)).expect("mesh built");
    mesh.shards().iter().map(|s| s.work_per_round()).collect()
}

/// Per-shard work under a corridor-local claim — a MODEL of the certifier's demand, not the
/// certifier. Cells within `radius` of the scene centre are refined `depth` extra octree
/// levels, so they cost `8^depth` each; everything else rides at base cost.
fn corridor_work(
    grid: Grid,
    nx: usize,
    ny: usize,
    nz: usize,
    radius: f64,
    depth: u32,
) -> Vec<usize> {
    let part = Partition::blocks(grid, nx, ny, nz);
    let mut work = vec![0usize; part.shard_count()];
    let c = [
        grid.w as f64 * 0.5,
        grid.h as f64 * 0.5,
        grid.d as f64 * 0.5,
    ];
    for cell in 0..grid.len() as u32 {
        let (x, y, z) = grid.coord(cell);
        let d = ((x as f64 - c[0]).powi(2) + (y as f64 - c[1]).powi(2) + (z as f64 - c[2]).powi(2))
            .sqrt();
        work[part.owner(cell) as usize] += if d <= radius { 8usize.pow(depth) } else { 1 };
    }
    work
}

fn imbalance(work: &[usize]) -> f64 {
    let max = *work.iter().max().expect("non-empty") as f64;
    let mean = work.iter().sum::<usize>() as f64 / work.len() as f64;
    max / mean
}

/// **Why the balancer is deferred.** Under the uniform claim this mesh actually runs, a
/// geometric partition is already near-optimal, so there is nothing for a claim-based
/// scheduler to recover.
#[test]
fn under_a_uniform_claim_geometric_partitioning_is_already_near_optimal() {
    // Grid sizes kept modest on purpose: building a Mesh allocates a halo per shard, which
    // is O(shards x cells) in debug and pushed this file to 36 s at 64^3. The imbalance
    // ratio is a shape property and does not need a big scene to show itself — measured
    // 1.014 at 256x256 and 1.124 at 32^3, both inside the same band as the sizes used here.
    for (tag, grid, n) in [
        ("flat 128x128", Grid::new(128, 128), (8usize, 8usize, 1usize)),
        ("cube 24^3", Grid::new_3d(24, 24, 24), (4, 4, 4)),
        ("thin 64x64x4", Grid::new_3d(64, 64, 4), (4, 4, 4)),
    ] {
        let f = imbalance(&uniform_work(grid, n.0, n.1, n.2));
        assert!(
            f < 1.30,
            "{tag}: uniform-claim imbalance {f:.2} — if this ever exceeds 1.3, geometric \
             balance has stopped being good enough and §5.2's balancer is no longer premature"
        );
    }
}

/// **Why the balancer will be needed.** Give the same geometry a corridor-local claim — the
/// shape the sandbox certifier actually produces — and geometric partitioning collapses.
///
/// This is `demand_not_function_of_geometry` as a number rather than as a theorem: identical
/// shards, identical grain, wildly different cost.
#[test]
fn under_a_corridor_claim_geometric_partitioning_collapses() {
    let grid = Grid::new_3d(64, 64, 64);
    let mut worst: f64 = 0.0;
    for (radius, depth) in [(24.0, 1u32), (12.0, 1), (12.0, 2), (6.0, 2), (6.0, 3)] {
        let f = imbalance(&corridor_work(grid, 4, 4, 4, radius, depth));
        assert!(
            f > 2.0,
            "corridor r={radius} depth={depth}: imbalance {f:.2} was expected to exceed 2x; \
             if the claim no longer concentrates work, this model has stopped representing \
             the certifier and the deferral argument must be re-taken"
        );
        worst = worst.max(f);
    }
    assert!(
        worst > 7.0,
        "the worst corridor should exceed 7x; measured {worst:.2}"
    );
}

/// **The consequence that decides scheduling: a shard imbalance caps speedup outright.**
///
/// A perfect schedule's makespan is bounded below by its largest single shard, so with `S`
/// shards at imbalance `f` the speedup can never exceed `S / f` however many threads are
/// supplied. At the worst measured corridor that is ~8.4x on 64 shards — barely half of 16
/// threads — which is precisely the cost §5.2's balancer exists to recover.
#[test]
fn imbalance_caps_speedup_regardless_of_thread_count() {
    let grid = Grid::new_3d(64, 64, 64);
    let work = corridor_work(grid, 4, 4, 4, 6.0, 3);
    let shards = work.len() as f64;
    let cap = shards / imbalance(&work);
    assert!(
        (8.0..9.0).contains(&cap),
        "speedup cap should be ~8.4x on 64 shards; derived {cap:.2}"
    );
    assert!(
        cap < 16.0,
        "the cap must bind below 16 threads, or the balancer would buy nothing at the \
         thread counts this lane measures"
    );
}

/// The work metric must actually vary, or every ratio above is a ratio of constants.
#[test]
fn the_work_metric_is_not_constant() {
    let work = uniform_work(Grid::new_3d(16, 16, 16), 4, 4, 4);
    assert!(work.len() > 1);
    assert!(
        work.iter().min() != work.iter().max(),
        "shards all report identical work; the metric is measuring nothing"
    );
}
