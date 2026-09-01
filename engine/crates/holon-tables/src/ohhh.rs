//! The four-body `(O,H,H,H)` surface, as the folded mesh sees it.
//!
//! This is the `Surface` the DE4-TABLE campaign generates through the leased pipeline. It
//! exists here rather than in `holon-chem` only because the trait does: the coordinates,
//! the canonical form, the embedding and the continuation all live in
//! `holon_chem::quaternary_table`, and this module is the adapter, not a second copy of
//! them. Nothing below re-derives a physical fact.
//!
//! # What one node costs, and why the pair curves are cached
//!
//! A node's stored value is `dE4 = E_FCI(OH3) - E_MBE3(OH3)`. The mesh supplies the
//! `E_FCI` half by solving the four-centre problem; [`OhhhSurface::subtract`] supplies the
//! `E_MBE3` half, which is six pair terms, three (O,H,H) table reads and one (H,H,H) table
//! read. Measured (`holon-chem/examples/de4_price.rs`), the naive route spends 241 ms of a
//! 522 ms median on that second half, almost all of it in six FRESH two-centre solves —
//! and there are only two distinct pairs in the whole composition, `O-H` and `H-H`. The
//! curves are therefore sampled ONCE at construction and interpolated, which is what the
//! ozone generator already does and what makes the difference between a 36 core-hour run
//! and a 21 core-hour one.
//!
//! The cache is built in the constructor and never written afterwards, so `Surface`'s
//! purity contract holds: a node's value cannot depend on which worker reached it first.

use holon_chem::elements::{Species, HYDROGEN, OXYGEN};
use holon_chem::pair::{atom_energy, pair_point};
use holon_chem::quaternary_table::{canonical_index, elliptope_scale, embed_ohhh, gram_det};
use holon_chem::trimer::TrimerTable;
use holon_chem::water::WaterTable;

use crate::surface::{min_separation, Realised, Surface, MIN_SEPARATION};

/// A pair curve sampled once and read by cubic Hermite, standing in for a fresh
/// two-centre solve at every node.
///
/// The knots are uniform in `R`. That is deliberately NOT the `R^{-1/4}` grid the shipped
/// pair tables use: this cache is a cost optimisation inside one generation run and its
/// error has to be far below the node's own, so it buys accuracy with knot count (1024
/// over 12 bohr, ~0.012 bohr spacing) rather than with a cleverer grid whose derivation
/// would then have to be defended.
#[derive(Clone, Debug)]
pub struct PairCurve {
    lo: f64,
    hi: f64,
    e: Vec<f64>,
}

impl PairCurve {
    pub fn sample(a: Species, b: Species, lo: f64, hi: f64, n: usize) -> PairCurve {
        let mut e = Vec::with_capacity(n);
        for i in 0..n {
            let r = lo + (hi - lo) * i as f64 / (n - 1) as f64;
            e.push(pair_point(a, b, r).e);
        }
        PairCurve { lo, hi, e }
    }

    /// The pair energy at `r`. Below `lo` it holds the first knot and above `hi` the last;
    /// both are outside any geometry this table admits, and a node that reached them would
    /// already have been refused for atom overlap.
    pub fn at(&self, r: f64) -> f64 {
        let n = self.e.len();
        let t = (r - self.lo) / (self.hi - self.lo) * (n - 1) as f64;
        if !(t > 0.0) {
            return self.e[0];
        }
        if t >= (n - 1) as f64 {
            return self.e[n - 1];
        }
        let i = t.floor() as usize;
        let s = t - i as f64;
        // Catmull-Rom on a uniform grid, one-sided at the two ends.
        let p = |k: isize| self.e[(k.clamp(0, n as isize - 1)) as usize];
        let (p0, p1, p2, p3) = (
            p(i as isize - 1),
            p(i as isize),
            p(i as isize + 1),
            p(i as isize + 2),
        );
        let m1 = 0.5 * (p2 - p0);
        let m2 = 0.5 * (p3 - p1);
        let s2 = s * s;
        let s3 = s2 * s;
        (2.0 * s3 - 3.0 * s2 + 1.0) * p1
            + (s3 - 2.0 * s2 + s) * m1
            + (-2.0 * s3 + 3.0 * s2) * p2
            + (s3 - s2) * m2
    }
}

/// The `(O,H,H,H)` four-body surface on the frozen six-coordinate domain.
///
/// Coordinates are `(R1, R2, R3, u12, u23, u31)` — three O-H distances and the three
/// cosines of the H-O-H angles — for the reason measured in
/// `DE4_TABLE_PREREG.md` (M3): a box in the six interatomic DISTANCES is 6.53% geometries,
/// while this one is 61.69% and the shortfall is a single smooth condition rather than
/// three coupled ones.
pub struct OhhhSurface {
    species: [Species; 4],
    water: WaterTable,
    trimer: TrimerTable,
    oh: PairCurve,
    hh: PairCurve,
    e_o: f64,
    e_h: f64,
}

impl OhhhSurface {
    /// Build the surface, sampling both pair curves once.
    ///
    /// `r_hi` should be the grid's own `R_HI`; the H-H curve is sampled to `2 * r_hi`
    /// because two hydrogens at opposite ends of the domain are that far apart.
    pub fn new(water: WaterTable, trimer: TrimerTable, r_lo: f64, r_hi: f64) -> OhhhSurface {
        OhhhSurface {
            species: [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
            water,
            trimer,
            oh: PairCurve::sample(OXYGEN, HYDROGEN, r_lo * 0.5, r_hi * 1.05, 1024),
            hh: PairCurve::sample(HYDROGEN, HYDROGEN, r_lo * 0.5, 2.0 * r_hi * 1.05, 1024),
            e_o: atom_energy(OXYGEN),
            e_h: atom_energy(HYDROGEN),
        }
    }

    /// The H-H distance implied by two O-H distances and the cosine between them.
    #[inline]
    fn hh_side(ri: f64, rj: f64, u: f64) -> f64 {
        (ri * ri + rj * rj - 2.0 * ri * rj * u).max(0.0).sqrt()
    }

    /// `E_MBE3` at these coordinates: six pairs, three (O,H,H) triples, one (H,H,H) triple,
    /// and the isolated atoms. The same decomposition as
    /// `holon_chem::quaternary::ohhh_mbe3_energy`, reading cached pair curves instead of
    /// re-solving each pair.
    pub fn mbe3(&self, coords: &[f64]) -> f64 {
        let (r1, r2, r3) = (coords[0], coords[1], coords[2]);
        let (u12, u23, u31) = (coords[3], coords[4], coords[5]);
        let d12 = Self::hh_side(r1, r2, u12);
        let d23 = Self::hh_side(r2, r3, u23);
        let d31 = Self::hh_side(r3, r1, u31);

        let v_oh = |r: f64| self.oh.at(r) - self.e_o - self.e_h;
        let v_hh = |r: f64| self.hh.at(r) - 2.0 * self.e_h;
        let pairs =
            v_oh(r1) + v_oh(r2) + v_oh(r3) + v_hh(d12) + v_hh(d23) + v_hh(d31);

        let triples = self.water.eval(r1, r2, d12).0
            + self.water.eval(r2, r3, d23).0
            + self.water.eval(r3, r1, d31).0
            + self.trimer.eval([d12, d23, d31]).0;

        self.e_o + 3.0 * self.e_h + pairs + triples
    }
}

impl Surface for OhhhSurface {
    fn species(&self) -> &[Species] {
        &self.species
    }

    fn dim(&self) -> usize {
        6
    }

    fn realise(&self, coords: &[f64]) -> Realised {
        assert_eq!(coords.len(), 6, "the (O,H,H,H) surface has six coordinates");
        let r = [coords[0], coords[1], coords[2]];
        let u = [coords[3], coords[4], coords[5]];
        if r.iter().any(|v| !v.is_finite() || *v <= 0.0) {
            return Realised::Refused;
        }

        // Inside the elliptope this is a geometry; outside, the continuation scales the
        // cosine triple radially toward the origin. Radial scaling COMMUTES with permuting
        // the three cosines, so the continuation is S3-equivariant and the table's orbit
        // fill stays exact through it — that property is the reason for choosing a radial
        // projection over a nearest-point one, and plant (ii) tests both directions.
        let inside = gram_det(u) >= 0.0;
        let (uu, _t) = if inside { (u, 1.0) } else { elliptope_scale(u) };

        let centers = match embed_ohhh(r, uu) {
            Some(c) => c.to_vec(),
            None => return Realised::Refused,
        };
        // A geometry that would panic the solver is refused here, where it costs one node,
        // rather than there, where `geometry_problem`'s `expect` costs the whole table.
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

    fn subtract(&self, coords: &[f64], e_total: f64) -> f64 {
        e_total - self.mbe3(coords)
    }

    /// The relabelling orbit's representative, **delegated whole** to
    /// [`holon_chem::quaternary_table::canonical_index`].
    ///
    /// The three hydrogens are indistinguishable, so relabelling them is a group of order
    /// six acting on `[i0, i1, i2, k0, k1, k2]` — the radial index of hydrogen `a` and the
    /// cosine index of the pair `{a, b}` move TOGETHER, which is the fact
    /// `quaternary_table`'s own header records the predecessor sort losing. Five of every
    /// six nodes are therefore the same geometry under a different name, and the generator
    /// solves one of them.
    ///
    /// Delegated rather than reimplemented because the symmetry is a physical fact about the
    /// composition and it is stated once, in the physics crate, next to the `set_orbit` fill
    /// and the `eval` un-permutation that have to agree with it. A second copy here would be
    /// a second place for the same rule to live, and it would disagree SILENTLY: a wrong
    /// orbit map produces a complete table of entirely plausible numbers.
    ///
    /// # What this declaration assumes about the grid
    ///
    /// The three radial axes must be identical to one another (same `n`, `lo`, `hi`, `map`)
    /// and so must the three cosine axes — relabelling moves indices BETWEEN axes, so on a
    /// grid whose radial axes differed the relabelled tuple would name a different geometry.
    /// The frozen domain (`quaternary_table::{NR, NU, R_LO, R_HI, U_LO, U_HI, STRETCH_A}`) is
    /// uniform in exactly that way; a caller who builds a ragged grid has left the symmetry
    /// this method declares.
    fn canonical(&self, idx: &[usize]) -> Vec<usize> {
        assert_eq!(idx.len(), 6, "the (O,H,H,H) surface has six coordinates");
        let (i, k) = canonical_index([idx[0], idx[1], idx[2]], [idx[3], idx[4], idx[5]]);
        vec![i[0], i[1], i[2], k[0], k[1], k[2]]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cached_pair_curve_tracks_a_fresh_solve() {
        // The cache exists to save six two-centre solves a node. It is only allowed to do
        // that if its error is far below the node's own scale; the (O,H,H) table's
        // interpolation scale is 2.47e-4 Ha, so 1e-6 is two orders inside it.
        let c = PairCurve::sample(OXYGEN, HYDROGEN, 0.45, 6.3, 1024);
        let mut worst = 0.0f64;
        for k in 0..40 {
            let r = 0.9 + (5.9 - 0.9) * k as f64 / 39.0;
            let d = (c.at(r) - pair_point(OXYGEN, HYDROGEN, r).e).abs();
            if d > worst {
                worst = d;
            }
        }
        assert!(
            worst < 1e-6,
            "the cached O-H curve is off by {worst:.3e} Ha, which is not far enough below \
             the node scale to be free"
        );
    }

    #[test]
    fn a_node_outside_the_elliptope_is_continued_not_solved() {
        let w = WaterTable::empty();
        let t = TrimerTable::empty();
        let s = OhhhSurface::new(w, t, 0.9, 6.0);
        // Well outside: three mutually obtuse directions that do not exist in R^3.
        let out = [2.0, 2.2, 2.4, -0.9, -0.9, -0.9];
        assert!(gram_det([-0.9, -0.9, -0.9]) < 0.0);
        match s.realise(&out) {
            Realised::Continued { from, .. } => {
                assert!(gram_det([from[3], from[4], from[5]]) >= -1e-12);
            }
            other => panic!("expected a continuation, got {other:?}"),
        }
        // Inside: a real geometry, and the coordinates come back untouched.
        let inside = [1.94, 1.94, 3.0, -0.117, -0.2, -0.2];
        assert!(matches!(s.realise(&inside), Realised::Geometry(_)));
    }
}
