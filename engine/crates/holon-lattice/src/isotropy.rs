//! The fourth-rank momentum-flux tensor of a direction set — the NECESSARY lattice
//! condition, measured, with the sufficient ones fenced.
//!
//! `engine/MESH_DESIGN.md` §2.1 carries the warrant in prose (the LOCAL copy: the file
//! exists in the sibling CIRISOntology tree too, and a citation into that tree is not a
//! tracked file here): no 3D Bravais lattice with a single
//! speed has an isotropic fourth-rank tensor, which is why FCHC-24 was chosen, and the
//! hexagonal lattice's fourth-order isotropy is the whole reason the founding 64-state
//! object means anything. This module puts that warrant on the tier's own instrument.
//!
//! # This is NOT a claim of the Navier–Stokes limit
//!
//! Fourth-rank isotropy is necessary and nowhere near sufficient. Unmeasured here, and named
//! as the exit in LG_PREREG §3: the kinematic viscosity against the model's own prediction,
//! semi-detailed balance of the collision table, and the `g(ρ) ≠ 1` Galilean defect. No
//! document in this programme may say this tier has a Navier–Stokes limit until those three
//! are run.
//!
//! # Axial ledger, Euclidean metric
//!
//! `P` is carried in `Core/Lattice.lean`'s axial integer coordinates, where conservation is
//! integer identity. Isotropy is a METRIC statement, so it is read in the Euclidean
//! embedding `M = [[1, 1/2], [0, √3/2]]`. `M` is linear and invertible (`det M = √3/2`), so
//! axial conservation and Euclidean conservation are the same statement — the one float
//! computation in this crate is not a weakening of the ledger.
//!
//! # The failing row is the point
//!
//! HPP-4 is measured alongside FHP-6 every time. A routine that has only ever printed
//! "isotropic" has not been shown able to print anything else, and the verdict is computed
//! from the direction set, never from the model's name (M-TAG-AS-PROPERTY).

use crate::state::Model;

/// The Euclidean embedding of a model's directions. Axial hex coordinates go through
/// `M = [[1, 1/2], [0, √3/2]]`; a square lattice's are already Euclidean.
pub fn embed(model: &Model) -> Vec<[f64; 2]> {
    let hex = model.n_dirs() == 6;
    model
        .dirs
        .iter()
        .map(|d| {
            let (a, b) = (d[0] as f64, d[1] as f64);
            if hex {
                [a + 0.5 * b, (3.0_f64).sqrt() / 2.0 * b]
            } else {
                [a, b]
            }
        })
        .collect()
}

/// How far a direction set is from carrying an isotropic fourth-rank tensor.
#[derive(Clone, Copy, Debug)]
pub struct Isotropy {
    pub second_rank_trace_ratio: f64,
    pub t4_xxxx: f64,
    pub t4_xxyy: f64,
    /// The isotropic coefficient the set would need: `T⁴_xxxx / 3`.
    pub a: f64,
    /// `max |T⁴ − A(δδ + δδ + δδ)|`. Machine zero for FHP-6; 2/3 for HPP-4.
    pub residual: f64,
}

/// Measure `T⁴_αβγδ = Σᵢ cᵢα cᵢβ cᵢγ cᵢδ` against `A(δδ + δδ + δδ)`.
pub fn measure(model: &Model) -> Isotropy {
    let c = embed(model);
    let mut t2 = [[0.0f64; 2]; 2];
    let mut t4 = [[[[0.0f64; 2]; 2]; 2]; 2];
    for v in &c {
        for a in 0..2 {
            for b in 0..2 {
                t2[a][b] += v[a] * v[b];
                for g in 0..2 {
                    for d in 0..2 {
                        t4[a][b][g][d] += v[a] * v[b] * v[g] * v[d];
                    }
                }
            }
        }
    }
    let coeff = t4[0][0][0][0] / 3.0;
    let dl = |i: usize, j: usize| if i == j { 1.0 } else { 0.0 };
    let mut residual: f64 = 0.0;
    for a in 0..2 {
        for b in 0..2 {
            for g in 0..2 {
                for d in 0..2 {
                    let iso = dl(a, b) * dl(g, d) + dl(a, g) * dl(b, d) + dl(a, d) * dl(b, g);
                    residual = residual.max((t4[a][b][g][d] - coeff * iso).abs());
                }
            }
        }
    }
    Isotropy {
        second_rank_trace_ratio: t2[0][0] / t2[1][1],
        t4_xxxx: t4[0][0][0][0],
        t4_xxyy: t4[0][0][1][1],
        a: coeff,
        residual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// G12, both rows, against the frozen reference table of LG_PREREG §5.2.
    #[test]
    fn fhp_is_isotropic_to_fourth_order_and_hpp_is_not() {
        let f = measure(&Model::fhp6());
        assert!((f.t4_xxxx - 2.25).abs() < 1e-12);
        assert!((f.t4_xxyy - 0.75).abs() < 1e-12);
        assert!((f.a - 0.75).abs() < 1e-12);
        assert!(f.residual < 1e-12, "FHP-6 residual {}", f.residual);

        let h = measure(&Model::hpp4());
        assert!((h.t4_xxxx - 2.0).abs() < 1e-12);
        assert!(h.t4_xxyy.abs() < 1e-12);
        assert!(h.residual > 0.66, "HPP-4 residual {} — the failing row did not fail", h.residual);
    }

    /// M-PRESENTATION-VERDICT: the criterion is about a direction SET, so re-presenting the
    /// same set must give a bit-identical verdict. A permuted ordering and a rotated basis
    /// are the same lattice.
    #[test]
    fn the_verdict_is_invariant_under_re_presentation() {
        let m = Model::fhp6();
        let base = measure(&m);
        let mut permuted = m.clone();
        permuted.dirs.rotate_left(3);
        permuted.name = "FHP-6 (reordered)";
        let p = measure(&permuted);
        assert_eq!(p.t4_xxxx.to_bits(), base.t4_xxxx.to_bits());
        assert_eq!(p.t4_xxyy.to_bits(), base.t4_xxyy.to_bits());

        // The same six directions presented already-Euclidean, as a four-direction model
        // would be: a model with n_dirs != 6 skips the axial map, so feeding the embedded
        // vectors back rounded to integers is NOT the same set and must NOT be tried. What
        // is tried is the sign-reversed presentation, which is the same set.
        let mut flipped = m.clone();
        flipped.dirs = m.dirs.iter().map(|d| [-d[0], -d[1]]).collect();
        assert!((measure(&flipped).residual - base.residual).abs() < 1e-15);
    }

    /// The embedding is what the derivation says it is: six unit vectors 60° apart.
    #[test]
    fn the_axial_embedding_is_the_hexagon() {
        for v in embed(&Model::fhp6()) {
            assert!(((v[0] * v[0] + v[1] * v[1]) - 1.0).abs() < 1e-12, "{v:?} is not a unit vector");
        }
    }
}
