//! **The kernel seam.** What a node's coordinates MEAN, kept in one place so the mesh
//! never has to know.
//!
//! # Why this trait exists (WB-8.7)
//!
//! Before it, `generate` knew that a node was a triangle: it held `[Species; 3]`, it built
//! the centres itself from `(x, y, u)`, and the grid had exactly three axes. Every new
//! composition would therefore have needed its own generator — a per-composition carve-out,
//! which is the DRY residual WB-8.7 makes the falsifier rather than the plan.
//!
//! The fold splits the pipeline at the only place the compositions actually differ:
//!
//! | the mesh owns | the surface owns |
//! |---|---|
//! | which node is solved where, in what order, from what start | what the node's coordinates are |
//! | the leases, the receipts, the digest, the partition | how coordinates become Cartesian centres |
//! | the VOID rules that are about the SOLVE | which atoms are there |
//! | | what to subtract to get the stored many-body term |
//!
//! Nothing in the mesh mentions a triangle any more, and nothing here mentions a worker.
//!
//! # The third case, which is the reason this is an enum and not a function
//!
//! A 3-axis trimer box was chosen so that **every** point of it is a realisable triangle —
//! `TableGrid`'s own header says so, and it was chosen that way precisely so a stencil could
//! never fall into a hole. Six interatomic distances have no such box: most of the
//! 6-cube is not the distance matrix of any four points in three dimensions, and the
//! embeddable set is a curved region inside it.
//!
//! So a node can be a coordinate tuple that is not a geometry. It cannot simply be skipped
//! (an interpolating stencil would read a hole), and it must never be scored (it is not the
//! point it claims to be). [`Realised::Continued`] is that third case: the surface hands
//! back the centres of the nearest point it CAN realise, the node is filled from those, and
//! it carries [`crate::VoidReason::NotAGeometry`] so that no accuracy statistic can include
//! it by accident. The exclusion is by construction, in the record, not by a filter someone
//! has to remember to write.

use holon_chem::elements::Species;

/// The closest two nuclei may be, in bohr, before a surface refuses the node.
///
/// Not a tolerance and not a tuning knob: below this the minimal-basis overlap matrix is
/// numerically singular, `holon_chem::pair::geometry_problem` PANICS rather than returning an
/// error, and the panic happens inside a worker thread and destroys the whole table. 0.1 bohr
/// is two orders below the shortest bond any of these boxes contains, so it cannot refuse a
/// geometry anyone meant to ask for.
pub const MIN_SEPARATION: f64 = 0.1;

/// The smallest distance between any two of these centres.
pub fn min_separation(centers: &[[f64; 3]]) -> f64 {
    let mut m = f64::INFINITY;
    for i in 0..centers.len() {
        for j in (i + 1)..centers.len() {
            let (a, b) = (&centers[i], &centers[j]);
            let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
            if d < m {
                m = d;
            }
        }
    }
    m
}

/// What a surface makes of one node's coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum Realised {
    /// The coordinates are a geometry; here are the Cartesian centres.
    Geometry(Vec<[f64; 3]>),
    /// The coordinates are NOT a geometry (e.g. outside the embeddability region). The
    /// surface supplies the centres of the point it is continued from, and the coordinates
    /// of that point, so the node is filled but never scored.
    Continued { centers: Vec<[f64; 3]>, from: Vec<f64> },
    /// Nothing sensible can be done here.
    Refused,
}

/// One tabulated surface: what the axes mean, which atoms are there, and what the stored
/// number is.
///
/// `Sync` because the mesh hands `&self` to every worker. It is a pure function of its
/// coordinates — a surface that cached would make a node's value depend on which worker
/// reached it first, which is the one thing this crate exists to rule out.
pub trait Surface: Sync {
    /// The atoms, in the order [`Surface::realise`] returns their centres.
    fn species(&self) -> &[Species];

    /// How many grid axes this surface's coordinates have. Checked against the grid.
    fn dim(&self) -> usize;

    /// Turn one node's coordinates into Cartesian centres, or say why not.
    fn realise(&self, coords: &[f64]) -> Realised;

    /// Turn a converged TOTAL energy at these coordinates into the stored many-body term.
    ///
    /// The identity for a surface that stores totals; `E_total - E_MBE(k-1)` for one that
    /// stores the k-body term. It takes the coordinates as well as the energy because the
    /// lower-body reference is a function of the geometry.
    ///
    /// **Known seam limit (DRY residual):** this is a value-only hook, so a record's
    /// `d1_bits`/`d2_bits` carry the derivatives of the TOTAL, not of the subtracted term.
    /// On every surface built so far the centres are `D2::c` constants and both derivatives
    /// are exactly zero, so nothing currently depends on it; a surface that seeds a dual
    /// would need this widened rather than worked around.
    fn subtract(&self, coords: &[f64], e_total: f64) -> f64;

    /// **What the stored number is a residual OF** — the subtraction basis, one of the three
    /// axes of a table's identity beside the device class and the solver budget.
    ///
    /// REQUIRED rather than defaulted, and that is the point: a default of "total" would let a
    /// surface that DOES subtract inherit a manifest line saying it does not, which is the
    /// silent-difference failure `subtract`'s own comment is written against. A new surface
    /// must say what its numbers mean before it can produce any.
    fn basis(&self) -> &'static str;

    /// The canonical representative of this node's symmetry orbit, as grid indices.
    /// Default: every node is its own representative (no symmetry declared).
    ///
    /// # What the generator does with it
    ///
    /// A node whose representative is not itself is **never solved**. It is FILLED, in a
    /// second pass, with a bit-for-bit copy of its representative's record — see
    /// [`crate::generate::generate_surface_with_progress`]. That is the whole of the DRY
    /// win: the hand-rolled generators in `holon-chem` each solve only the `i <= j` half of
    /// their grid and mirror the same float, and this is that trick said once, for any
    /// symmetry, on the one pipeline.
    ///
    /// # The contract, which the mesh checks
    ///
    /// * the result has one index per axis, each in range;
    /// * it is **idempotent** — `canonical(canonical(x)) == canonical(x)`. A representative
    ///   that was itself mirrored would leave a node filled from a slot nobody solved, and
    ///   the generator would report "never solved" from a place that cannot explain it.
    ///
    /// # The contract the mesh CANNOT check, and so does not pretend to
    ///
    /// The declared orbit must be a real symmetry **of the surface and of the grid
    /// together**: the representative's node must be the same physical point, so its solved
    /// energy is the mirror's energy. A symmetry that permutes indices BETWEEN axes (which
    /// is what a relabelling symmetry does) therefore additionally requires those axes to be
    /// identical in `n`, `lo`, `hi` and `map` — otherwise the relabelled index tuple names a
    /// different geometry, and the fill is wrong while looking perfectly right. Nothing here
    /// can see the physics, so this is stated rather than asserted, and it is the reason a
    /// surface declares its own symmetry instead of the mesh guessing one.
    fn canonical(&self, idx: &[usize]) -> Vec<usize> {
        idx.to_vec()
    }
}

/// The 3-body trimer surface: the two short sides and the cosine between them.
///
/// This is the surface the SATURATION-2/3 tables were built on, lifted out of `generate`
/// unchanged. `realise` is `generate::triangle`'s arithmetic character for character,
/// because those tables are gated on bit-identity and `y * u` computed differently is a
/// different table.
#[derive(Clone, Copy, Debug)]
pub struct TrimerSurface {
    pub species: [Species; 3],
}

impl TrimerSurface {
    pub fn new(species: [Species; 3]) -> TrimerSurface {
        TrimerSurface { species }
    }
}

impl Surface for TrimerSurface {
    fn species(&self) -> &[Species] {
        &self.species
    }

    fn dim(&self) -> usize {
        3
    }

    fn realise(&self, coords: &[f64]) -> Realised {
        assert_eq!(coords.len(), 3, "the trimer surface has three coordinates");
        let (x, y, u) = (coords[0], coords[1], coords[2]);
        let s = (1.0 - u * u).max(0.0).sqrt();
        // Every point of the trimer box is a realisable triangle — that is why the box was
        // chosen in these coordinates (see `TableGrid`'s header), so there is no Continued
        // case here and none is invented.
        Realised::Geometry(vec![[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * u, y * s, 0.0]])
    }

    fn subtract(&self, _coords: &[f64], e_total: f64) -> f64 {
        // The 3-body tables store the TOTAL energy; the many-body subtraction happens in
        // the evaluator, not in the table. Stated as the identity rather than left implicit,
        // so that a surface which DOES subtract is a visible difference and not a silent one.
        e_total
    }

    fn basis(&self) -> &'static str {
        "none: stores E_total (electronic + nuclear); the many-body subtraction is the \
         evaluator's, not the table's"
    }
}

/// Place four atoms from their six interatomic distances, in the axis order
/// `[d01, d02, d03, d12, d13, d23]`.
///
/// # The continuation, stated exactly
///
/// The embedding is the standard one: atom 0 at the origin, atom 1 on `+x`, atom 2 in the
/// `xy` plane with `y >= 0`, atom 3 with `z >= 0`. Two square roots can go negative, and
/// each is a different way of leaving the embeddable set:
///
/// * `y2^2 < 0` — atoms 0, 1, 2 violate a triangle inequality;
/// * `z3^2 < 0` — the four distances have a negative Cayley–Menger determinant: they are a
///   valid triangle plus a fourth point that cannot reach.
///
/// In both cases the offending square is clamped to zero, which is the NEAREST point of the
/// feasible set along that one coordinate — a degenerate (collinear, or planar) arrangement
/// that really is a geometry. The distances actually realised are returned as
/// [`Realised::Continued::from`], recomputed from the centres rather than assumed, so the
/// record says where the number came from rather than where it was asked for.
///
/// # Why the continuation can still refuse, which cost this lane a run
///
/// Clamping is not free: it can bring two centres onto the SAME POINT. Measured on the
/// 6-axis `[1.4, 3.0]` box, 6 of its 64 corners do — e.g. `d = (3.0, 1.4, 1.4, 1.4, 1.4,
/// 1.4)`, where atoms 2 and 3 both clamp to `(1.5, 0, 0)` — even though no pair was ASKED to
/// be closer than 1.4 bohr. The continuation invented the coincidence.
///
/// That is not a slightly-wrong geometry, it is a fatal one: two coincident centres make the
/// STO-3G overlap matrix singular, and `holon_chem::pair::geometry_problem` responds with
/// `expect("overlap not positive definite")` — **a panic inside a worker thread, which takes
/// the whole scoped generation down with it.** One bad node would destroy a multi-hour
/// table. So the realised centres are checked against [`MIN_SEPARATION`] and the node is
/// REFUSED when the continuation collapses, which is exactly what refusal is for: the
/// continuation, not the coordinate, is what has nothing sensible to offer here.
///
/// [`Realised::Refused`] therefore covers two cases: coordinates that are not distances at
/// all (non-finite or non-positive), and coordinates whose continuation is degenerate.
pub fn embed_tetramer(d: &[f64]) -> Realised {
    assert_eq!(d.len(), 6, "a four-atom distance surface has six coordinates");
    if d.iter().any(|v| !v.is_finite() || *v <= 0.0) {
        return Realised::Refused;
    }
    let (d01, d02, d03, d12, d13, d23) = (d[0], d[1], d[2], d[3], d[4], d[5]);

    let mut continued = false;

    let x2 = (d01 * d01 + d02 * d02 - d12 * d12) / (2.0 * d01);
    let mut y2sq = d02 * d02 - x2 * x2;
    if y2sq < 0.0 {
        y2sq = 0.0;
        continued = true;
    }
    let y2 = y2sq.sqrt();

    let x3 = (d01 * d01 + d03 * d03 - d13 * d13) / (2.0 * d01);
    // With atom 2 collinear with 0 and 1 there is no `y` direction to solve in, so atom 3's
    // `y` is not determined by the distances; the continuation puts it in the same plane.
    let y3 = if y2 > 0.0 {
        (d02 * d02 + d03 * d03 - d23 * d23 - 2.0 * x2 * x3) / (2.0 * y2)
    } else {
        continued = true;
        0.0
    };
    let mut z3sq = d03 * d03 - x3 * x3 - y3 * y3;
    if z3sq < 0.0 {
        z3sq = 0.0;
        continued = true;
    }
    let z3 = z3sq.sqrt();

    let centers = vec![[0.0, 0.0, 0.0], [d01, 0.0, 0.0], [x2, y2, 0.0], [x3, y3, z3]];
    // The clamp can collapse two centres onto one; see the header. A geometry that would
    // panic the solver is refused here, where it costs one node, rather than there, where it
    // costs the table.
    if min_separation(&centers) < MIN_SEPARATION {
        return Realised::Refused;
    }
    if !continued {
        return Realised::Geometry(centers);
    }
    // What was ACTUALLY realised, measured off the centres.
    let dist = |a: &[f64; 3], b: &[f64; 3]| {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    };
    let from = vec![
        dist(&centers[0], &centers[1]),
        dist(&centers[0], &centers[2]),
        dist(&centers[0], &centers[3]),
        dist(&centers[1], &centers[2]),
        dist(&centers[1], &centers[3]),
        dist(&centers[2], &centers[3]),
    ];
    Realised::Continued { centers, from }
}

/// A four-atom surface parameterised by its six interatomic distances — the 6-axis shape
/// the fold exists to serve.
///
/// It stores the TOTAL energy. A lane that wants `dE4` implements [`Surface`] itself with
/// its own [`Surface::subtract`] and reuses [`embed_tetramer`]; the many-body reference
/// needs the water and trimer tables, and those belong to the physics crate rather than to
/// the mesh.
#[derive(Clone, Copy, Debug)]
pub struct DistanceTetramer {
    pub species: [Species; 4],
}

impl DistanceTetramer {
    pub fn new(species: [Species; 4]) -> DistanceTetramer {
        DistanceTetramer { species }
    }
}

impl Surface for DistanceTetramer {
    fn species(&self) -> &[Species] {
        &self.species
    }

    fn dim(&self) -> usize {
        6
    }

    fn realise(&self, coords: &[f64]) -> Realised {
        embed_tetramer(coords)
    }

    fn subtract(&self, _coords: &[f64], e_total: f64) -> f64 {
        e_total
    }

    fn basis(&self) -> &'static str {
        "none: stores E_total. This surface is the GRID, not the subtraction — a caller          wanting a four-body residual uses `OhhhSurface`, whose basis says so."
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use holon_chem::elements::by_symbol;

    /// A regular tetrahedron embeds, and the centres reproduce the distances asked for.
    #[test]
    fn a_realisable_tetrahedron_is_a_geometry() {
        let d = [2.0f64; 6];
        match embed_tetramer(&d) {
            Realised::Geometry(c) => {
                assert_eq!(c.len(), 4);
                let dist = |a: &[f64; 3], b: &[f64; 3]| {
                    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
                };
                for (i, j) in [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)] {
                    assert!(
                        (dist(&c[i], &c[j]) - 2.0).abs() < 1e-12,
                        "pair ({i},{j}) came back at {}",
                        dist(&c[i], &c[j])
                    );
                }
                assert!(c[3][2] > 0.0, "the tetrahedron came out flat");
            }
            other => panic!("a regular tetrahedron was not realised: {other:?}"),
        }
    }

    /// A point outside the embeddable set is CONTINUED, not refused and not silently
    /// accepted — and the coordinates it was continued from are the ones actually realised.
    #[test]
    fn an_unembeddable_tuple_is_continued_from_a_real_point() {
        // Three mutually-close atoms and a fourth that claims to be 2.0 from all of them
        // while the triangle it must reach over is far too wide: Cayley-Menger negative.
        let d = [1.0, 1.0, 1.0, 1.0, 1.0, 4.0];
        match embed_tetramer(&d) {
            Realised::Continued { centers, from } => {
                assert_eq!(centers.len(), 4);
                assert_eq!(from.len(), 6);
                assert!(
                    from.iter().all(|v| v.is_finite()),
                    "the continued point is not a geometry either: {from:?}"
                );
                assert_ne!(
                    from[5], d[5],
                    "the node claims to have been continued and yet realised exactly the \
                     coordinates it was given"
                );
            }
            other => panic!("an unembeddable tuple gave {other:?}"),
        }
    }

    /// Coordinates that are not distances are REFUSED rather than continued from something
    /// invented.
    #[test]
    fn a_non_distance_is_refused() {
        assert_eq!(embed_tetramer(&[1.0, 1.0, 1.0, 1.0, 1.0, 0.0]), Realised::Refused);
        assert_eq!(embed_tetramer(&[1.0, 1.0, 1.0, 1.0, 1.0, -1.0]), Realised::Refused);
        assert_eq!(
            embed_tetramer(&[1.0, 1.0, 1.0, 1.0, 1.0, f64::NAN]),
            Realised::Refused
        );
    }

    /// A continuation that would put two nuclei on the same point is REFUSED, not handed to
    /// a solver that answers such a geometry with a panic.
    ///
    /// The witness is measured, not invented: on the 6-axis `[1.4, 3.0]` box this tuple
    /// clamps atoms 2 and 3 both onto `(1.5, 0, 0)`, and before the guard it took down a
    /// whole 64-node run from inside a worker thread.
    #[test]
    fn a_continuation_that_collapses_two_nuclei_is_refused() {
        assert_eq!(embed_tetramer(&[3.0, 1.4, 1.4, 1.4, 1.4, 1.4]), Realised::Refused);
        // M-VACUOUS-SUCCESS: the guard must not refuse everything. Nothing the box actually
        // contains as a geometry comes anywhere near the floor.
        match embed_tetramer(&[1.4; 6]) {
            Realised::Geometry(c) => assert!(min_separation(&c) > 1.0),
            other => panic!("a regular tetrahedron was refused: {other:?}"),
        }
        assert!(MIN_SEPARATION < 0.5, "the separation floor is inside the chemistry");
    }

    /// The trimer surface reproduces the generator's own triangle, which is the arithmetic
    /// every committed 3-body table was built on.
    #[test]
    fn the_trimer_surface_is_the_generators_triangle() {
        let h = by_symbol("H").unwrap();
        let s = TrimerSurface::new([h, h, h]);
        assert_eq!(s.dim(), 3);
        for (x, y, u) in [(1.5, 2.5, 0.3), (2.0, 2.0, -0.9), (3.1, 1.2, 0.0)] {
            let sq = (1.0f64 - u * u).max(0.0).sqrt();
            let want = vec![[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * u, y * sq, 0.0]];
            match s.realise(&[x, y, u]) {
                Realised::Geometry(got) => assert_eq!(got, want),
                other => panic!("the trimer surface refused a triangle: {other:?}"),
            }
        }
        assert_eq!(s.subtract(&[1.0, 1.0, 0.0], -1.25), -1.25);
    }
}
