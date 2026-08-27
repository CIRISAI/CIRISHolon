//! The scenes both engines are asked to simulate, and the EXACT solution they are both
//! measured against.
//!
//! A scene is a node count, an edge set with a stiffness per edge, and a set of initial
//! positions. Nothing else — no colliders, no gravity, no rest lengths. That is the
//! whole of the overlap between the two engines (see the report's fairness section),
//! and confining the comparison to it is what makes the comparison mean anything.

/// A spring network: `n` unit point masses, `edges` of `(i, j, stiffness)` with `i < j`,
/// at `pos0`, all at rest.
#[derive(Clone, Debug)]
pub struct Scene {
    pub name: String,
    pub n: usize,
    pub edges: Vec<(usize, usize, f64)>,
    pub pos0: Vec<[f64; 3]>,
}

impl Scene {
    /// Density of the edge set, `2E / (N(N-1))`. 1.0 for a complete graph.
    pub fn density(&self) -> f64 {
        if self.n < 2 {
            return 0.0;
        }
        2.0 * self.edges.len() as f64 / (self.n as f64 * (self.n as f64 - 1.0))
    }

    /// The dense coupling matrix, which is what `ciris-sim-core` requires. Building it
    /// is `O(N^2)` in memory whatever the edge count — the core has no sparse
    /// representation, and that is a finding rather than an oversight.
    pub fn coupling(&self) -> Vec<f64> {
        let mut c = vec![0.0f64; self.n * self.n];
        for &(i, j, k) in &self.edges {
            c[i * self.n + j] = k;
            c[j * self.n + i] = k;
        }
        c
    }

    /// Deterministic spread over the unit sphere by the golden-angle spiral. No RNG
    /// anywhere in this comparison, so both engines get byte-identical starts.
    pub fn spiral_positions(n: usize) -> Vec<[f64; 3]> {
        let ga = 2.399_963_229_728_653_f64;
        (0..n)
            .map(|i| {
                let z = 1.0 - 2.0 * (i as f64 + 0.5) / (n as f64);
                let r = (1.0f64 - z * z).max(0.0).sqrt();
                let th = ga * (i as f64);
                [r * th.cos(), r * th.sin(), z]
            })
            .collect()
    }

    /// A complete graph with unit stiffness — the shape the CIRIS object actually has,
    /// and the case where both engines do `O(N^2)` work so generality is matched.
    pub fn complete(n: usize) -> Scene {
        let mut edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                edges.push((i, j, 1.0));
            }
        }
        Scene { name: format!("complete K{n}"), n, edges, pos0: Self::spiral_positions(n) }
    }

    /// The built-in eleven-kind object: complete, with the MEASURED couplings.
    pub fn k11() -> Scene {
        let n = ciris_sim_core::N;
        let mut edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                let k = ciris_sim_core::COUPLING[i][j];
                if k != 0.0 {
                    edges.push((i, j, k));
                }
            }
        }
        Scene { name: "K11 (measured couplings)".into(), n, edges, pos0: Self::spiral_positions(n) }
    }

    /// A 3D cubic spring lattice, `s` nodes per side: sparse, `O(N)` edges. This is the
    /// shape a general-purpose engine is built for and the case where an all-pairs
    /// integrator is expected to lose.
    pub fn lattice(s: usize) -> Scene {
        let n = s * s * s;
        let idx = |x: usize, y: usize, z: usize| (z * s + y) * s + x;
        let mut edges = Vec::new();
        for z in 0..s {
            for y in 0..s {
                for x in 0..s {
                    if x + 1 < s { edges.push((idx(x, y, z), idx(x + 1, y, z), 1.0)); }
                    if y + 1 < s { edges.push((idx(x, y, z), idx(x, y + 1, z), 1.0)); }
                    if z + 1 < s { edges.push((idx(x, y, z), idx(x, y, z + 1), 1.0)); }
                }
            }
        }
        // Start displaced from the lattice sites so there is motion to get wrong.
        let spiral = Self::spiral_positions(n);
        let pos0 = (0..n)
            .map(|i| {
                let (x, y, z) = (i % s, (i / s) % s, i / (s * s));
                [
                    x as f64 + 0.10 * spiral[i][0],
                    y as f64 + 0.10 * spiral[i][1],
                    z as f64 + 0.10 * spiral[i][2],
                ]
            })
            .collect();
        Scene { name: format!("lattice {s}^3"), n, edges, pos0 }
    }
}
