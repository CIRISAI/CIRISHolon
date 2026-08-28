//! H2 in the STO-3G minimal basis, solved exactly in that basis (full CI).
//!
//! # The model, in one paragraph
//!
//! Two hydrogens, one contracted 1s function on each. Symmetry fixes the molecular
//! orbitals of a homonuclear diatomic in a minimal basis without any SCF iteration:
//! `sigma_g = (chi_A + chi_B)/sqrt(2(1+S))` and `sigma_u = (chi_A - chi_B)/sqrt(2(1-S))`.
//! Two electrons in two orbitals gives a six-determinant Fock space whose singlet
//! `M_s = 0` sector is a 2x2 problem in `{(sigma_g)^2, (sigma_u)^2}`; the open-shell
//! singlet is ungerade and does not couple to either. So the exact-in-basis ground state
//! is the lower root of a 2x2 matrix, in closed form — which is why the whole curve is
//! cheap enough to compute at page load.
//!
//! # Two routes, because one route is an opinion
//!
//! [`fci_route_a`] is that closed form, reached through the Slater-Condon rules.
//! [`fci_route_b`] builds the six-determinant Hamiltonian from raw ladder operators with
//! their fermionic signs applied explicitly — no Slater-Condon rule anywhere — and
//! diagonalises it. They share the integrals and nothing else, so agreement between them
//! tests the CI algebra rather than the arithmetic. Route (a) is the primary evaluator
//! (it carries derivatives; route (b) is values only) and the gate is that they agree.
//!
//! # What is NOT claimed
//!
//! Every number here is EXACT-IN-MODEL for STO-3G FCI, and STO-3G is a small basis. The
//! equilibrium separation and well depth this returns are properties of the model, not
//! predictions of experiment, and nothing in this crate compares them to experiment.

use crate::dual::D2;
use crate::sto3g::{prim_eri, prim_kinetic, prim_nuclear, prim_overlap, sto3g_hydrogen, Contraction};

/// One point of the curve: energy and its two exact derivatives.
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub r: f64,
    /// Total energy (electronic + nuclear repulsion), hartree.
    pub e: f64,
    /// The FORCE, `-dE/dR`, hartree/bohr. Positive means repulsive.
    pub f: f64,
    /// `d2E/dR2`, hartree/bohr^2.
    pub e2: f64,
}

/// The intermediate quantities behind one point, exposed so a caller can check the
/// build rather than only its answer.
#[derive(Clone, Copy, Debug)]
pub struct Pieces {
    pub r: f64,
    /// `<chi_A|chi_B>`, the only geometry-dependent scalar the MO coefficients need.
    pub s_ab: D2,
    pub overlap: Mat2,
    pub kinetic: Mat2,
    pub nuclear: Mat2,
    pub hcore: Mat2,
    pub eri: Eri2,
    /// One-electron integrals in the MO basis; index 0 is `sigma_g`, 1 is `sigma_u`.
    pub hmo: Mat2,
    pub gmo: Eri2,
    /// The lower CI root, WITHOUT nuclear repulsion.
    pub e_electronic: D2,
    /// `1/R`.
    pub e_nuclear: D2,
    pub e_total: D2,
}

impl Pieces {
    pub fn point(&self) -> Point {
        Point {
            r: self.r,
            e: self.e_total.v,
            f: -self.e_total.d,
            e2: self.e_total.e,
        }
    }
}

/// A 2x2 matrix over the two contracted AOs, carrying derivatives.
type Mat2 = [[D2; 2]; 2];

/// The two-electron integral array, `(ij|kl)` in chemist notation.
type Eri2 = [[[[D2; 2]; 2]; 2]; 2];

/// AO integrals for the two-centre basis at separation `r`.
///
/// The two-electron array is filled from the six symmetry-unique integrals and their
/// eight permutations, exactly as the referee does: `(ij|kl) = (ji|kl) = (ij|lk) =
/// (kl|ij) = ...`. Computing all sixteen independently would be sixteen slightly
/// different roundings of one number, so the shared value is also the reproducible one.
fn ao_integrals(basis: &Contraction, centre: [D2; 2]) -> (Mat2, Mat2, Mat2, Eri2) {
    let z = D2::c(0.0);
    let mut s = [[z; 2]; 2];
    let mut t = [[z; 2]; 2];
    let mut v = [[z; 2]; 2];

    for i in 0..2 {
        for j in 0..=i {
            let mut si = z;
            let mut ti = z;
            let mut vi = z;
            for a in 0..3 {
                for b in 0..3 {
                    let w = basis.coeff[a] * basis.coeff[b];
                    si = si + prim_overlap(basis.prim[a], centre[i], basis.prim[b], centre[j]) * w;
                    ti = ti + prim_kinetic(basis.prim[a], centre[i], basis.prim[b], centre[j]) * w;
                    for &c in centre.iter() {
                        vi = vi
                            + prim_nuclear(
                                basis.prim[a],
                                centre[i],
                                basis.prim[b],
                                centre[j],
                                c,
                                1.0,
                            ) * w;
                    }
                }
            }
            s[i][j] = si;
            s[j][i] = si;
            t[i][j] = ti;
            t[j][i] = ti;
            v[i][j] = vi;
            v[j][i] = vi;
        }
    }

    let mut eri = [[[[z; 2]; 2]; 2]; 2];
    for i in 0..2 {
        for j in 0..=i {
            for k in 0..2 {
                for l in 0..=k {
                    if (i * (i + 1) / 2 + j) < (k * (k + 1) / 2 + l) {
                        continue;
                    }
                    let mut acc = z;
                    for a in 0..3 {
                        for b in 0..3 {
                            for c in 0..3 {
                                for d in 0..3 {
                                    let w = basis.coeff[a] * basis.coeff[b] * basis.coeff[c]
                                        * basis.coeff[d];
                                    acc = acc
                                        + prim_eri(
                                            basis.prim[a],
                                            centre[i],
                                            basis.prim[b],
                                            centre[j],
                                            basis.prim[c],
                                            centre[k],
                                            basis.prim[d],
                                            centre[l],
                                        ) * w;
                                }
                            }
                        }
                    }
                    for &(p, q, r_, s_) in &[
                        (i, j, k, l),
                        (j, i, k, l),
                        (i, j, l, k),
                        (j, i, l, k),
                        (k, l, i, j),
                        (l, k, i, j),
                        (k, l, j, i),
                        (l, k, j, i),
                    ] {
                        eri[p][q][r_][s_] = acc;
                    }
                }
            }
        }
    }
    (s, t, v, eri)
}

/// Everything about one separation.
pub fn h2_pieces(r: f64) -> Pieces {
    let basis = sto3g_hydrogen();
    // Centre A is the origin; centre B IS the independent variable. Seeding the
    // derivative here is the whole of the differentiation: every integral below is an
    // expression in these two, so the chain rule carries `d/dR` through by itself.
    let centre = [D2::c(0.0), D2::var(r)];
    let (s, t, v, eri) = ao_integrals(&basis, centre);

    let mut hcore = [[D2::c(0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            hcore[i][j] = t[i][j] + v[i][j];
        }
    }

    // Symmetry-determined MOs. `c[ao][mo]`, mo 0 = sigma_g, mo 1 = sigma_u.
    let s_ab = s[0][1];
    let cg = 1.0 / (2.0 * (s_ab + 1.0)).sqrt();
    let cu = 1.0 / (2.0 * (1.0 - s_ab)).sqrt();
    let c = [[cg, cu], [cg, -cu]];

    let mut hmo = [[D2::c(0.0); 2]; 2];
    for p in 0..2 {
        for q in 0..2 {
            let mut acc = D2::c(0.0);
            for i in 0..2 {
                for j in 0..2 {
                    acc = acc + c[i][p] * c[j][q] * hcore[i][j];
                }
            }
            hmo[p][q] = acc;
        }
    }

    let mut gmo = [[[[D2::c(0.0); 2]; 2]; 2]; 2];
    for p in 0..2 {
        for q in 0..2 {
            for r_ in 0..2 {
                for t_ in 0..2 {
                    let mut acc = D2::c(0.0);
                    for i in 0..2 {
                        for j in 0..2 {
                            for k in 0..2 {
                                for l in 0..2 {
                                    acc = acc
                                        + c[i][p] * c[j][q] * c[k][r_] * c[l][t_] * eri[i][j][k][l];
                                }
                            }
                        }
                    }
                    gmo[p][q][r_][t_] = acc;
                }
            }
        }
    }

    let e_electronic = fci_route_a(&hmo, &gmo);
    let e_nuclear = 1.0 / D2::var(r);
    Pieces {
        r,
        s_ab,
        overlap: s,
        kinetic: t,
        nuclear: v,
        hcore,
        eri,
        hmo,
        gmo,
        e_electronic,
        e_nuclear,
        e_total: e_electronic + e_nuclear,
    }
}

/// ROUTE (a): the singlet-sector 2x2 CI in `{(sigma_g)^2, (sigma_u)^2}`.
///
/// ```text
/// H11 = 2 h_gg + (gg|gg)      H22 = 2 h_uu + (uu|uu)      H12 = (gu|gu)
/// ```
///
/// from the Slater-Condon rules for a doubly-substituted pair. The lower root of a
/// symmetric 2x2 is closed-form, so this carries its derivatives through `sqrt`.
pub fn fci_route_a(hmo: &Mat2, gmo: &Eri2) -> D2 {
    let h11 = hmo[0][0] * 2.0 + gmo[0][0][0][0];
    let h22 = hmo[1][1] * 2.0 + gmo[1][1][1][1];
    let h12 = gmo[0][1][0][1];
    let tr = h11 + h22;
    let dif = h11 - h22;
    let disc = (dif * dif + h12 * h12 * 4.0).sqrt();
    (tr - disc) * 0.5
}

/// Total energy at `r`, and its exact first and second derivatives.
pub fn h2_point(r: f64) -> Point {
    h2_pieces(r).point()
}

/// Total energy at `r`, hartree.
pub fn h2_energy(r: f64) -> f64 {
    h2_pieces(r).e_total.v
}

// --------------------------------------------------------------- route (b)

/// Sign of `a_p` acting on `det`, or `None` if the orbital is empty.
fn annihilate(det: u32, p: u32) -> Option<(f64, u32)> {
    if (det >> p) & 1 == 0 {
        return None;
    }
    let below = (det & ((1u32 << p) - 1)).count_ones();
    Some((if below & 1 == 1 { -1.0 } else { 1.0 }, det ^ (1 << p)))
}

/// Sign of `a+_p` acting on `det`, or `None` if the orbital is already occupied.
fn create(det: u32, p: u32) -> Option<(f64, u32)> {
    if (det >> p) & 1 == 1 {
        return None;
    }
    let below = (det & ((1u32 << p) - 1)).count_ones();
    Some((if below & 1 == 1 { -1.0 } else { 1.0 }, det | (1 << p)))
}

/// Apply a string of ladder operators right to left. `true` in the tuple means create.
fn apply_string(det: u32, ops: &[(bool, u32)]) -> Option<(f64, u32)> {
    let mut sign = 1.0f64;
    let mut cur = det;
    for &(is_create, p) in ops.iter().rev() {
        let (s, next) = if is_create {
            create(cur, p)?
        } else {
            annihilate(cur, p)?
        };
        sign *= s;
        cur = next;
    }
    Some((sign, cur))
}

/// ROUTE (b): brute-force diagonalisation of the two-electron block of the
/// four-spin-orbital Fock space.
///
/// Spin orbitals are `0 = (g, alpha)`, `1 = (g, beta)`, `2 = (u, alpha)`,
/// `3 = (u, beta)`, so `p >> 1` is the spatial index and `p & 1` the spin.
///
/// ```text
/// H = sum_pq h_pq a+_p a_q + 1/2 sum_pqrs (pq|rs) a+_p a+_r a_s a_q
/// ```
///
/// with spin deltas on `(p,q)` and `(r,s)`. Every fermionic sign comes from the ladder
/// operators explicitly; no Slater-Condon rule is used anywhere in this route, which is
/// what makes agreement with [`fci_route_a`] evidence about the CI algebra.
///
/// Values only — this route exists to check route (a), and a check that shared route
/// (a)'s differentiation machinery would not be checking much.
pub fn fci_route_b_from(hmo: &[[f64; 2]; 2], gmo: &[[[[f64; 2]; 2]; 2]; 2]) -> f64 {
    let dets: [u32; 6] = [0b0011, 0b0101, 0b0110, 0b1001, 0b1010, 0b1100];
    let index = |d: u32| dets.iter().position(|&x| x == d).expect("2-electron det");
    let mut h = [[0.0f64; 6]; 6];

    for &d in dets.iter() {
        let col = index(d);
        for p in 0..4u32 {
            for q in 0..4u32 {
                if p & 1 != q & 1 {
                    continue;
                }
                let hv = hmo[(p >> 1) as usize][(q >> 1) as usize];
                if hv == 0.0 {
                    continue;
                }
                if let Some((sg, nd)) = apply_string(d, &[(true, p), (false, q)]) {
                    h[index(nd)][col] += sg * hv;
                }
            }
        }
        for p in 0..4u32 {
            for q in 0..4u32 {
                if p & 1 != q & 1 {
                    continue;
                }
                for r in 0..4u32 {
                    for s in 0..4u32 {
                        if r & 1 != s & 1 {
                            continue;
                        }
                        let gv = gmo[(p >> 1) as usize][(q >> 1) as usize][(r >> 1) as usize]
                            [(s >> 1) as usize];
                        if gv == 0.0 {
                            continue;
                        }
                        if let Some((sg, nd)) =
                            apply_string(d, &[(true, p), (true, r), (false, s), (false, q)])
                        {
                            h[index(nd)][col] += sg * gv / 2.0;
                        }
                    }
                }
            }
        }
    }

    // Symmetrise defensively: the matrix is symmetric by construction, so this changes
    // nothing a correct build produces, and it keeps the Jacobi sweep well-posed if a
    // future edit breaks that.
    let mut sym = [[0.0f64; 6]; 6];
    for i in 0..6 {
        for j in 0..6 {
            sym[i][j] = 0.5 * (h[i][j] + h[j][i]);
        }
    }
    jacobi_lowest(sym)
}

/// Route (b) at a separation, sharing route (a)'s integrals.
pub fn fci_route_b(r: f64) -> f64 {
    let p = h2_pieces(r);
    let mut hmo = [[0.0f64; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            hmo[i][j] = p.hmo[i][j].v;
        }
    }
    let mut gmo = [[[[0.0f64; 2]; 2]; 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                for l in 0..2 {
                    gmo[i][j][k][l] = p.gmo[i][j][k][l].v;
                }
            }
        }
    }
    fci_route_b_from(&hmo, &gmo) + 1.0 / r
}

/// Lowest eigenvalue of a real symmetric 6x6, by cyclic Jacobi rotations.
///
/// Jacobi rather than anything faster because 6x6 runs in microseconds either way and
/// this one has no pivoting, no shifts and no convergence heuristics to get wrong: it
/// drives the off-diagonal norm to zero monotonically and stops when it is at roundoff.
fn jacobi_lowest(mut a: [[f64; 6]; 6]) -> f64 {
    for _ in 0..100 {
        let mut off = 0.0f64;
        for i in 0..6 {
            for j in (i + 1)..6 {
                off += a[i][j] * a[i][j];
            }
        }
        if off <= 1e-32 {
            break;
        }
        for p in 0..6 {
            for q in (p + 1)..6 {
                if a[p][q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = if theta >= 0.0 {
                    1.0 / (theta + (1.0 + theta * theta).sqrt())
                } else {
                    -1.0 / (-theta + (1.0 + theta * theta).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                for k in 0..6 {
                    let akp = a[k][p];
                    let akq = a[k][q];
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..6 {
                    let apk = a[p][k];
                    let aqk = a[q][k];
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
            }
        }
    }
    let mut lo = a[0][0];
    for i in 1..6 {
        if a[i][i] < lo {
            lo = a[i][i];
        }
    }
    lo
}

// ------------------------------------------------- the in-model reference points

/// The hydrogen ATOM in this same basis: one electron, one contracted 1s, one proton.
///
/// `E = <chi|h|chi> / <chi|chi>`. The division is not decoration — it is what makes the
/// answer independent of the renormalisation applied to the tabulated coefficients.
pub fn h_atom_energy() -> f64 {
    let basis = sto3g_hydrogen();
    let o = D2::c(0.0);
    let mut s = 0.0f64;
    let mut t = 0.0f64;
    let mut v = 0.0f64;
    for a in 0..3 {
        for b in 0..3 {
            let w = basis.coeff[a] * basis.coeff[b];
            s += w * prim_overlap(basis.prim[a], o, basis.prim[b], o).v;
            t += w * prim_kinetic(basis.prim[a], o, basis.prim[b], o).v;
            v += w * prim_nuclear(basis.prim[a], o, basis.prim[b], o, o, 1.0).v;
        }
    }
    (t + v) / s
}

/// The in-model dissociation limit: two non-interacting STO-3G hydrogen atoms.
///
/// Computed, never quoted. It is what the renderer takes as the zero of its pair
/// potential, so an error here would shift every energy on the ledger by a constant.
pub fn asymptote() -> f64 {
    2.0 * h_atom_energy()
}

/// The equilibrium separation and well depth, both computed in-model.
///
/// `R_e` is the root of `dE/dR`, which this crate has exactly rather than as a
/// difference quotient. Bisection first, because it cannot leave the bracket; Newton
/// last, because with the exact second derivative in hand it converges to the last bit
/// in three steps and bisection alone stalls at the width where `dE/dR` is roundoff.
///
/// Returns `(R_e, D_e, E(R_e))` with `D_e = E_asymptote - E(R_e)`.
pub fn equilibrium() -> (f64, f64, f64) {
    // Bracket by scan. The range is the physically meaningful one for a bound diatomic
    // in this model and the scan verifies the sign change rather than assuming it.
    let (mut lo, mut hi) = (0.8f64, 3.0f64);
    assert!(
        h2_point(lo).slope() < 0.0 && h2_point(hi).slope() > 0.0,
        "the R_e bracket does not contain a minimum; the curve is not the one this \
         routine was written for"
    );
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if h2_point(mid).slope() < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 1e-8 {
            break;
        }
    }
    let mut r = 0.5 * (lo + hi);
    for _ in 0..8 {
        let p = h2_point(r);
        if p.e2 == 0.0 {
            break;
        }
        let step = p.slope() / p.e2;
        r -= step;
        if step.abs() <= f64::EPSILON * r.abs() {
            break;
        }
    }
    let e_at = h2_energy(r);
    (r, asymptote() - e_at, e_at)
}

impl Point {
    /// `dE/dR`. The stored quantity is the FORCE, which is its negative; every root
    /// finder here wants the slope, so the sign lives in one place instead of at each
    /// call site.
    pub fn slope(&self) -> f64 {
        -self.f
    }
}
