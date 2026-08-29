//! The three-body term of the many-body expansion for hydrogen, tabulated.
//!
//! # What this is for
//!
//! The atom sandbox's force loop is pairwise-additive, so sixteen hydrogens condense into
//! one droplet instead of eight molecules. Real hydrogen saturates because valence is a
//! MANY-BODY fact. This module computes the missing term, exact-in-model and
//! constant-free:
//!
//! ```text
//! V_tot(3) = E(H3) - 3 E(H)          V2(r) = E2(r) - 2 E(H)
//! dE3      = V_tot(3) - sum_pairs V2(r_ij)
//! ```
//!
//! — the same definition the feasibility probe used (`examples/mbe3_probe.rs`), so the
//! disclosed priors in `SATURATION1_PREREG.md` are readings of THIS function.
//!
//! # The coordinates, and why they are not the three sides
//!
//! `dE3` is a totally symmetric function of the triangle's three sides, and a table over
//! sides directly has two problems: the sorted region `a <= b <= c` is not a box, and the
//! triangle inequality `c <= a + b` cuts a corner off any box that contains it, so a
//! tensor-product interpolant's stencil would reach nodes that are not geometries at all.
//!
//! The table is therefore built on `(x, y, u)`: the two SHORTEST sides and the cosine of
//! the angle between them,
//!
//! ```text
//! x = s1,  y = s2,  u = (s1^2 + s2^2 - s3^2) / (2 s1 s2)
//! ```
//!
//! where `s1 <= s2 <= s3` are the sorted sides. EVERY point of the box
//! `[R_LO, R_HI] x [R_LO, R_HI] x [U_LO, U_HI]` is a realisable triangle — `z^2 = x^2 +
//! y^2 - 2xy u` is positive for every `|u| <= 1` — so there is no hole for the stencil to
//! fall into, and `u` rather than the angle is the coordinate because `z^2` is LINEAR in
//! `u` while `dz/dtheta` collapses at the collinear configurations that matter most.
//!
//! Evaluation SORTS the three sides first, which is exact in floating point, so the value
//! and the gradient are invariant under all six permutations bit-for-bit rather than to
//! within a tolerance. That is the symmetry plant's target and it passes by construction.
//!
//! # Where the tail lives (and why `R_HI` is a bound on the MIDDLE side)
//!
//! `dE3` vanishes when one atom is far from BOTH of the others, which for sorted sides is
//! the statement that `s2` is large — not that `s3` is. Measured on this model: at
//! `s2 = 7` the worst `|dE3|` over the whole shell is 6.4e-5 Ha, at `s2 = 8` it is
//! 6.0e-6 Ha, at `s2 = 9`, 4.7e-7 Ha. A cutoff on the LONGEST side is not a cutoff at all
//! — three atoms strung out over 7 bohr with 3.5 bohr spacings carry 1.9e-2 Ha of
//! three-body energy, and severing them is what the prereg's original any-side cutoff
//! did. `R_HI` therefore bounds `x` and `y`, the two shortest sides; `s3` is bounded only
//! by the triangle inequality and runs out to `2 R_HI`. This is the prereg's AMENDMENT A1
//! domain. See `tests/trimer.rs`, gate T2, which reports both shells.
//!
//! # The interpolant
//!
//! Tensor-product Catmull-Rom (cubic Hermite whose node slopes are centred differences,
//! one-sided at the two ends). It is C1 by construction — each node's slope is one fixed
//! linear functional of the node values, so both intervals meeting at a node use the same
//! slope — and it needs node VALUES only, which is what lets the whole table stream in as
//! a flat array with no mixed-derivative columns to keep true. Forces come from
//! differentiating the interpolant analytically, so the dynamics' energy function IS the
//! tabulated surface and conservation holds against it exactly.

use crate::special::boys0;
use crate::sto3g::{sto3g_hydrogen, Contraction, PI_POW_2_5};

const PI: f64 = core::f64::consts::PI;

// ================================================================ the solver
//
// A value-only, allocation-free electronic-structure path for N hydrogens in the STO-3G
// basis, N <= 3. It exists because the general N-centre route (`pair::solve_geometry`)
// costs ~5 ms per H3 point on this machine — its Hermite `E` tables and `R` tensors are
// sized for p functions and carried in second-order dual numbers — and a table needs
// thousands of points, at load, in a browser. This path is s-only and f64-only: the
// interpolant supplies the derivatives, so no dual numbers are needed, and hydrogen has
// no p shell, so the closed forms below are the whole integral set.
//
// It is a SECOND implementation of one model, which is a cost. It is paid for by
// `tests/trimer.rs::the_fast_path_agrees_with_the_general_n_centre_route`, which holds it
// to 1e-12 hartree against `pair::solve_geometry` over a staked spanning set — so the
// 50-digit referee chain reaches this path through that gate rather than around it.

/// Largest system this path handles: three hydrogens, nine determinants.
const MAX_ORB: usize = 3;
const MAX_DET: usize = 9;

/// One contracted-primitive pair: the Gaussian product's exponent, its weight (the two
/// contraction coefficients, the two primitive normalisations and the `K_ab` overlap
/// factor, multiplied once), the product centre, and `mu = ab/p` for the kinetic term.
#[derive(Clone, Copy, Default)]
struct PrimPair {
    p: f64,
    mu: f64,
    w: f64,
    c: [f64; 3],
}

#[inline]
fn dist2(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    // `(xx + yy) + zz`, matching the renderer's summation order.
    dx * dx + dy * dy + dz * dz
}

/// Weight below which a primitive pair is dropped.
///
/// A pair's whole contribution to any integral is proportional to its weight `w`, whose
/// only geometry dependence is the Gaussian overlap `exp(-mu R^2)`; the remaining factors
/// are `O(1)` for this basis and `F_0 <= 1`. Dropping pairs at `1e-18` therefore costs at
/// most a few times that in hartree — twenty decades below the interpolation error this
/// table is built to, and five below the 1e-12 the fast path is held to against the
/// general N-centre route, which is the gate that would catch it if the reasoning were
/// wrong. The screen matters because most of the grid is at long range, where nearly
/// every primitive pair is dead.
const W_SCREEN: f64 = 1e-18;

/// Live primitive pairs for one centre pair, screened. Returns how many survived.
fn prim_pairs(ca: &[f64; 3], cb: &[f64; 3], b: &Contraction, out: &mut [PrimPair; 9]) -> usize {
    let r2 = dist2(ca, cb);
    let mut m = 0;
    for i in 0..3 {
        for j in 0..3 {
            let (pa, pb) = (b.prim[i], b.prim[j]);
            let p = pa.alpha + pb.alpha;
            let mu = pa.alpha * pb.alpha / p;
            let k = (-(mu * r2)).exp();
            let w = b.coeff[i] * b.coeff[j] * pa.norm * pb.norm * k;
            if w.abs() < W_SCREEN {
                continue;
            }
            out[m] = PrimPair {
                p,
                mu,
                w,
                c: [
                    (ca[0] * pa.alpha + cb[0] * pb.alpha) / p,
                    (ca[1] * pa.alpha + cb[1] * pb.alpha) / p,
                    (ca[2] * pa.alpha + cb[2] * pb.alpha) / p,
                ],
            };
            m += 1;
        }
    }
    m
}

/// Ground-state total energy of `n` hydrogen atoms at `centers`, STO-3G full CI, hartree.
///
/// The Sz sector is the minimal one for the electron count, which is the prereg's
/// declared choice: H is a doublet, H2 a singlet, H3 a doublet.
pub fn hydrogen_energy(centers: &[[f64; 3]]) -> f64 {
    let n = centers.len();
    debug_assert!(n >= 1 && n <= MAX_ORB);
    let basis = sto3g_hydrogen();

    // --- primitive-pair data for every centre pair -------------------------------
    let mut pp = [[PrimPair::default(); 9]; MAX_ORB * MAX_ORB];
    let mut np = [0usize; MAX_ORB * MAX_ORB];
    for i in 0..n {
        for j in 0..n {
            let mut buf = [PrimPair::default(); 9];
            np[i * MAX_ORB + j] = prim_pairs(&centers[i], &centers[j], &basis, &mut buf);
            pp[i * MAX_ORB + j] = buf;
        }
    }

    // --- one-electron integrals ---------------------------------------------------
    let mut s = [0.0f64; MAX_ORB * MAX_ORB];
    let mut h = [0.0f64; MAX_ORB * MAX_ORB];
    for i in 0..n {
        for j in 0..=i {
            let r2 = dist2(&centers[i], &centers[j]);
            let mut sij = 0.0;
            let mut hij = 0.0;
            for m in &pp[i * MAX_ORB + j][..np[i * MAX_ORB + j]] {
                let pref = (PI / m.p).powf(1.5);
                sij += m.w * pref;
                // kinetic
                hij += m.w * m.mu * (3.0 - 2.0 * m.mu * r2) * pref;
                // nuclear attraction, summed over every centre
                let two_pi_over_p = 2.0 * PI / m.p;
                for c in centers.iter().take(n) {
                    let t = m.p * dist2(&m.c, c);
                    hij -= m.w * two_pi_over_p * boys0(t);
                }
            }
            s[i * MAX_ORB + j] = sij;
            s[j * MAX_ORB + i] = sij;
            h[i * MAX_ORB + j] = hij;
            h[j * MAX_ORB + i] = hij;
        }
    }

    // --- two-electron integrals, eight-fold symmetric -----------------------------
    let mut g = [0.0f64; MAX_ORB * MAX_ORB * MAX_ORB * MAX_ORB];
    let idx4 = |p: usize, q: usize, r: usize, t: usize| {
        ((p * MAX_ORB + q) * MAX_ORB + r) * MAX_ORB + t
    };
    for i in 0..n {
        for j in 0..=i {
            for k in 0..=i {
                let l_max = if k == i { j } else { k };
                for l in 0..=l_max {
                    let mut acc = 0.0;
                    for m in &pp[i * MAX_ORB + j][..np[i * MAX_ORB + j]] {
                        for o in &pp[k * MAX_ORB + l][..np[k * MAX_ORB + l]] {
                            let sum = m.p + o.p;
                            let alpha = m.p * o.p / sum;
                            let t = alpha * dist2(&m.c, &o.c);
                            acc += m.w * o.w * 2.0 * PI_POW_2_5 / (m.p * o.p * sum.sqrt())
                                * boys0(t);
                        }
                    }
                    // One computed value written to all eight permutations, so the CI
                    // code sees (ij|kl) == (kl|ij) EXACTLY rather than to roundoff.
                    for &(a, b, c, d) in &[
                        (i, j, k, l),
                        (j, i, k, l),
                        (i, j, l, k),
                        (j, i, l, k),
                        (k, l, i, j),
                        (l, k, i, j),
                        (k, l, j, i),
                        (l, k, j, i),
                    ] {
                        g[idx4(a, b, c, d)] = acc;
                    }
                }
            }
        }
    }

    // --- orthonormalise: S = L L^T, X = (L^-1)^T, so X^T S X = I --------------------
    let x = cholesky_inverse_transpose(&s, n);

    // --- transform to the orthonormal basis ----------------------------------------
    let mut hx = [0.0f64; MAX_ORB * MAX_ORB];
    for p in 0..n {
        for q in 0..n {
            let mut acc = 0.0;
            for i in 0..n {
                for j in 0..n {
                    acc += x[i * MAX_ORB + p] * x[j * MAX_ORB + q] * h[i * MAX_ORB + j];
                }
            }
            hx[p * MAX_ORB + q] = acc;
        }
    }
    // Four quarter-transforms rather than one eightfold contraction.
    let mut t1 = [0.0f64; MAX_ORB * MAX_ORB * MAX_ORB * MAX_ORB];
    let mut t2 = [0.0f64; MAX_ORB * MAX_ORB * MAX_ORB * MAX_ORB];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for d in 0..n {
                    let mut acc = 0.0;
                    for l in 0..n {
                        acc += g[idx4(i, j, k, l)] * x[l * MAX_ORB + d];
                    }
                    t1[idx4(i, j, k, d)] = acc;
                }
            }
        }
    }
    for i in 0..n {
        for j in 0..n {
            for c in 0..n {
                for d in 0..n {
                    let mut acc = 0.0;
                    for k in 0..n {
                        acc += t1[idx4(i, j, k, d)] * x[k * MAX_ORB + c];
                    }
                    t2[idx4(i, j, c, d)] = acc;
                }
            }
        }
    }
    for i in 0..n {
        for b in 0..n {
            for c in 0..n {
                for d in 0..n {
                    let mut acc = 0.0;
                    for j in 0..n {
                        acc += t2[idx4(i, j, c, d)] * x[j * MAX_ORB + b];
                    }
                    t1[idx4(i, b, c, d)] = acc;
                }
            }
        }
    }
    let mut gx = [0.0f64; MAX_ORB * MAX_ORB * MAX_ORB * MAX_ORB];
    for a in 0..n {
        for b in 0..n {
            for c in 0..n {
                for d in 0..n {
                    let mut acc = 0.0;
                    for i in 0..n {
                        acc += t1[idx4(i, b, c, d)] * x[i * MAX_ORB + a];
                    }
                    gx[idx4(a, b, c, d)] = acc;
                }
            }
        }
    }

    // --- full CI in the minimal Sz sector, plus nuclear repulsion -------------------
    let n_elec = n;
    let sz2 = n_elec % 2;
    let na = (n_elec + sz2) / 2;
    let nb = (n_elec - sz2) / 2;
    let e_elec = fci_ground(n, na, nb, &hx, &gx);

    let mut e_nuc = 0.0;
    for i in 0..n {
        for j in 0..i {
            e_nuc += 1.0 / dist2(&centers[i], &centers[j]).sqrt();
        }
    }
    e_elec + e_nuc
}

/// `X = (L^-1)^T` for `S = L L^T`. Upper triangular, and `X^T S X = I` exactly in exact
/// arithmetic — the same orthonormaliser the general route uses, written for a fixed
/// 3x3 so no allocation is needed.
fn cholesky_inverse_transpose(s: &[f64; MAX_ORB * MAX_ORB], n: usize) -> [f64; MAX_ORB * MAX_ORB] {
    let mut l = [0.0f64; MAX_ORB * MAX_ORB];
    for i in 0..n {
        for j in 0..=i {
            let mut acc = s[i * MAX_ORB + j];
            for k in 0..j {
                acc -= l[i * MAX_ORB + k] * l[j * MAX_ORB + k];
            }
            if i == j {
                l[i * MAX_ORB + i] = acc.max(0.0).sqrt();
            } else {
                l[i * MAX_ORB + j] = acc / l[j * MAX_ORB + j];
            }
        }
    }
    // Forward-substitute for L^-1 (lower triangular), then transpose.
    let mut inv = [0.0f64; MAX_ORB * MAX_ORB];
    for col in 0..n {
        inv[col * MAX_ORB + col] = 1.0 / l[col * MAX_ORB + col];
        for row in (col + 1)..n {
            let mut acc = 0.0;
            for k in col..row {
                acc -= l[row * MAX_ORB + k] * inv[k * MAX_ORB + col];
            }
            inv[row * MAX_ORB + col] = acc / l[row * MAX_ORB + row];
        }
    }
    let mut x = [0.0f64; MAX_ORB * MAX_ORB];
    for i in 0..n {
        for j in 0..n {
            x[i * MAX_ORB + j] = inv[j * MAX_ORB + i];
        }
    }
    x
}

/// Every `n`-bit mask with `k` bits set, ascending.
fn strings(n: usize, k: usize, out: &mut [u32; MAX_DET]) -> usize {
    let mut m = 0;
    for mask in 0u32..(1u32 << n) {
        if mask.count_ones() as usize == k {
            out[m] = mask;
            m += 1;
        }
    }
    m
}

/// Annihilate spin-orbital `(spin, p)`. Spin-orbitals are ordered alpha-then-beta, so a
/// beta operator carries the parity of the alpha occupation as well as its own.
#[inline]
fn ann(a: &mut u32, b: &mut u32, spin: usize, p: usize) -> Option<f64> {
    let below = if spin == 0 {
        if (*a >> p) & 1 == 0 {
            return None;
        }
        let below = (*a & ((1u32 << p) - 1)).count_ones();
        *a ^= 1u32 << p;
        below
    } else {
        if (*b >> p) & 1 == 0 {
            return None;
        }
        let below = a.count_ones() + (*b & ((1u32 << p) - 1)).count_ones();
        *b ^= 1u32 << p;
        below
    };
    Some(if below & 1 == 1 { -1.0 } else { 1.0 })
}

#[inline]
fn cre(a: &mut u32, b: &mut u32, spin: usize, p: usize) -> Option<f64> {
    let below = if spin == 0 {
        if (*a >> p) & 1 == 1 {
            return None;
        }
        let below = (*a & ((1u32 << p) - 1)).count_ones();
        *a |= 1u32 << p;
        below
    } else {
        if (*b >> p) & 1 == 1 {
            return None;
        }
        let below = a.count_ones() + (*b & ((1u32 << p) - 1)).count_ones();
        *b |= 1u32 << p;
        below
    };
    Some(if below & 1 == 1 { -1.0 } else { 1.0 })
}

/// Lowest eigenvalue of the CI Hamiltonian in the determinant basis.
///
/// The matrix is built by applying the second-quantised Hamiltonian
/// `sum_pq h_pq E_pq + 1/2 sum_pqrs (pq|rs) sum_{sigma,tau} a+_{p sigma} a+_{r tau}
/// a_{s tau} a_{q sigma}` to each determinant, which is mechanical: no Slater-Condon
/// case analysis, so no case to get wrong. At most nine determinants, so it is
/// diagonalised densely rather than iteratively.
fn fci_ground(
    n: usize,
    na: usize,
    nb: usize,
    h: &[f64; MAX_ORB * MAX_ORB],
    g: &[f64; MAX_ORB * MAX_ORB * MAX_ORB * MAX_ORB],
) -> f64 {
    let mut astr = [0u32; MAX_DET];
    let mut bstr = [0u32; MAX_DET];
    let n_a = strings(n, na, &mut astr);
    let n_b = strings(n, nb, &mut bstr);
    let nd = n_a * n_b;

    let index = |a: u32, b: u32| -> usize {
        let mut ia = usize::MAX;
        let mut ib = usize::MAX;
        for (t, &m) in astr.iter().take(n_a).enumerate() {
            if m == a {
                ia = t;
            }
        }
        for (t, &m) in bstr.iter().take(n_b).enumerate() {
            if m == b {
                ib = t;
            }
        }
        ia * n_b + ib
    };
    let idx4 = |p: usize, q: usize, r: usize, t: usize| {
        ((p * MAX_ORB + q) * MAX_ORB + r) * MAX_ORB + t
    };

    let mut hm = [0.0f64; MAX_DET * MAX_DET];
    for ia in 0..n_a {
        for ib in 0..n_b {
            let col = ia * n_b + ib;
            let (ma, mb) = (astr[ia], bstr[ib]);
            for spin in 0..2 {
                for q in 0..n {
                    let (mut a1, mut b1) = (ma, mb);
                    let Some(x1) = ann(&mut a1, &mut b1, spin, q) else {
                        continue;
                    };
                    for p in 0..n {
                        let (mut a2, mut b2) = (a1, b1);
                        let Some(x2) = cre(&mut a2, &mut b2, spin, p) else {
                            continue;
                        };
                        hm[index(a2, b2) * MAX_DET + col] += x1 * x2 * h[p * MAX_ORB + q];
                    }
                }
            }
            for sg in 0..2 {
                for tau in 0..2 {
                    for q in 0..n {
                        let (mut a1, mut b1) = (ma, mb);
                        let Some(x1) = ann(&mut a1, &mut b1, sg, q) else {
                            continue;
                        };
                        for s in 0..n {
                            let (mut a2, mut b2) = (a1, b1);
                            let Some(x2) = ann(&mut a2, &mut b2, tau, s) else {
                                continue;
                            };
                            for r in 0..n {
                                let (mut a3, mut b3) = (a2, b2);
                                let Some(x3) = cre(&mut a3, &mut b3, tau, r) else {
                                    continue;
                                };
                                for p in 0..n {
                                    let (mut a4, mut b4) = (a3, b3);
                                    let Some(x4) = cre(&mut a4, &mut b4, sg, p) else {
                                        continue;
                                    };
                                    hm[index(a4, b4) * MAX_DET + col] +=
                                        0.5 * x1 * x2 * x3 * x4 * g[idx4(p, q, r, s)];
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    jacobi_lowest(&mut hm, nd)
}

/// Lowest eigenvalue of a small dense symmetric matrix by cyclic Jacobi rotations.
fn jacobi_lowest(a: &mut [f64; MAX_DET * MAX_DET], n: usize) -> f64 {
    if n == 1 {
        return a[0];
    }
    for _ in 0..64 {
        let mut off = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                off += a[p * MAX_DET + q] * a[p * MAX_DET + q];
            }
        }
        if off < 1e-30 {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p * MAX_DET + q];
                if apq.abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q * MAX_DET + q] - a[p * MAX_DET + p]) / (2.0 * apq);
                let t = if theta >= 0.0 {
                    1.0 / (theta + (1.0 + theta * theta).sqrt())
                } else {
                    -1.0 / (-theta + (1.0 + theta * theta).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                for k in 0..n {
                    let akp = a[k * MAX_DET + p];
                    let akq = a[k * MAX_DET + q];
                    a[k * MAX_DET + p] = c * akp - s * akq;
                    a[k * MAX_DET + q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p * MAX_DET + k];
                    let aqk = a[q * MAX_DET + k];
                    a[p * MAX_DET + k] = c * apk - s * aqk;
                    a[q * MAX_DET + k] = s * apk + c * aqk;
                }
            }
        }
    }
    let mut lo = a[0];
    for k in 1..n {
        lo = lo.min(a[k * MAX_DET + k]);
    }
    lo
}

// ================================================================ dE3 itself

/// The isolated hydrogen atom in this model. Computed, never quoted; and computed ONCE
/// per call site rather than inside the pair loop, because it is a constant.
pub fn atom_energy() -> f64 {
    hydrogen_energy(&[[0.0, 0.0, 0.0]])
}

/// Total energy of two hydrogens at separation `r`.
pub fn pair_energy(r: f64) -> f64 {
    hydrogen_energy(&[[0.0, 0.0, 0.0], [r, 0.0, 0.0]])
}

/// The three-body interaction at the triangle with sides `(x, y, z)`, hartree.
///
/// `z` is recomputed from `(x, y, u)` by the caller in the table build; this entry point
/// takes the sides directly so a test can ask for a named geometry.
pub fn de3_sides(x: f64, y: f64, z: f64, e_h: f64) -> f64 {
    // Place the triangle: atom 0 at the origin, atom 1 along +x at distance `x`, atom 2
    // at distance `y` from atom 0 with the angle fixed by the law of cosines.
    let cos = ((x * x + y * y - z * z) / (2.0 * x * y)).clamp(-1.0, 1.0);
    let sin = (1.0 - cos * cos).max(0.0).sqrt();
    let e3 = hydrogen_energy(&[[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * cos, y * sin, 0.0]]);
    // dE3 = [E3 - 3 E_H] - sum (E2 - 2 E_H) = E3 + 3 E_H - sum E2.
    e3 + 3.0 * e_h - (pair_energy(x) + pair_energy(y) + pair_energy(z))
}

/// The same, from the table's own coordinates.
pub fn de3_xyu(x: f64, y: f64, u: f64, e_h: f64) -> f64 {
    let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
    de3_sides(x, y, z, e_h)
}

// ================================================================ the grid
//
// Three coordinates, each chosen by MEASUREMENT rather than taste; the numbers are in
// `examples/mbe3_grid_sweep.rs`, which builds the finest candidate once and reads the
// held-out error of every coarser grid inside it.
//
//  * the two short sides are uniform in `tau`, with `r = R_LO + (R_HI - R_LO)
//    (e^{a tau} - 1)/(e^a - 1)`. The surface is steep at contact and flat in the tail, so
//    an exponential stretch puts the knots where the curvature is. It is used INSTEAD of
//    a power law because `dtau/dr` stays finite at the lower edge; a power law's does not,
//    and a coordinate singularity there would be a force singularity. Measured at 41
//    knots: `a = 2` gives 3.4e-5 Ha held-out, `a = 3` gives 4.7e-5, uniform gives 5.3e-4.
//  * the third coordinate is `c = sqrt(1 - u)`, a fixed monotone reparametrisation of the
//    cosine. This one is worth a sentence: at `x = y` the third side is `z = x sqrt(2) c`
//    EXACTLY, so a uniform `c` grid is a uniform `z` grid there — while a uniform `u` grid
//    is uniform in `z^2`, which is coarsest precisely where `z` is smallest and the
//    surface is steepest. Measured at 41x41x13: `c` gives 3.8e-5 Ha, raw `u` gives 1.9e-4,
//    and a coordinate normalised by `min(x, y)` (linear in `z`, but kinked on the `x = y`
//    diagonal) floors out at 4.4e-4 however fine the grid.

/// Grid lower edge in the two short sides, bohr. Below the staked domain's 0.9 on
/// purpose: a query AT 0.9 is then interior to the grid rather than on its boundary,
/// where the node slopes are one-sided.
pub const R_LO: f64 = 0.7;

/// Grid upper edge in the two SHORT sides, bohr — the truncation radius `R_cut`, and the
/// prereg's AMENDMENT A1 value. The domain is `a >= 0.9`, `b <= R_cut`, `c <= a + b`: no
/// independent cut on the longest side, because a near-collinear chain's longest side is
/// the sum of two short ones and is not a distance anything decays over. Measured on the
/// b-shell by the 50-digit referee and independently here: 6.4e-5 Ha at `b = 7`, 6.1e-6 at
/// `b = 8`, 4.7e-7 at `b = 9` — 21x inside the 1e-5 stake.
pub const R_HI: f64 = 9.0;

/// Stretch of the side axis. See the block above.
pub const STRETCH_A: f64 = 2.0;

/// `c = sqrt(1 - u)`. A sorted triple has `u <= x/(2y) <= 1/2`, so `c >= sqrt(1/2) =
/// 0.7071` — strictly inside `C_LO`. `C_HI = sqrt(2)` is the collinear configuration
/// `u = -1`, which IS reachable and IS the domain's hard edge: `u < -1` violates the
/// triangle inequality, so there is no node to put beyond it and the last interval uses
/// the one-sided node slope. The held-out draw includes that edge, so the T1 number
/// already carries whatever it costs.
pub const C_LO: f64 = 0.632_455_532_033_675_9;
pub const C_HI: f64 = 1.414_213_562_373_095_1;

/// Nodes per side axis and per `c`. These sizes ARE the T1 measurement's product: at
/// 33 x 33 x 13 the held-out maximum is 6.3e-5 Ha, sixteen times inside the prereg's
/// 1e-3 kill, for 7,293 electronic-structure solves.
pub const NR: usize = 33;
pub const NU: usize = 13;

/// Total node count.
pub const N_NODES: usize = NR * NR * NU;

/// `e^a - 1`, the stretch's normalisation.
const EXPA1: f64 = 6.389_056_098_930_65; // e^2 - 1

/// Node index for `(i, j, k)` — `x` slowest, `c` fastest.
#[inline]
pub const fn node_index(i: usize, j: usize, k: usize) -> usize {
    (i * NR + j) * NU + k
}

/// The side at side-axis node `i`.
#[inline]
pub fn node_r(i: usize) -> f64 {
    r_of_tau(i as f64 / (NR - 1) as f64)
}

/// The third coordinate at node `k`, and the cosine it stands for.
#[inline]
pub fn node_c(k: usize) -> f64 {
    C_LO + (C_HI - C_LO) * k as f64 / (NU - 1) as f64
}

#[inline]
pub fn r_of_tau(tau: f64) -> f64 {
    R_LO + (R_HI - R_LO) * ((STRETCH_A * tau).exp() - 1.0) / EXPA1
}

#[inline]
pub fn tau_of_r(r: f64) -> f64 {
    (1.0 + (r - R_LO) * EXPA1 / (R_HI - R_LO)).ln() / STRETCH_A
}

/// `dtau/dr`. Finite everywhere on `[R_LO, R_HI]` — that is the whole reason the stretch
/// is exponential rather than a power law.
#[inline]
pub fn dtau_dr(r: f64) -> f64 {
    let k = EXPA1 / (R_HI - R_LO);
    k / (STRETCH_A * (1.0 + (r - R_LO) * k))
}

/// The geometry a node stands for: the two short sides and the third one.
pub fn node_geometry(i: usize, j: usize, k: usize) -> (f64, f64, f64) {
    let (x, y) = (node_r(i), node_r(j));
    let c = node_c(k);
    let u = 1.0 - c * c;
    let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
    (x, y, z)
}

/// What the whole table says about itself.
#[derive(Clone, Copy, Debug)]
pub struct TrimerMeta {
    pub n_nodes: usize,
    pub nr: usize,
    pub nu: usize,
    pub r_lo: f64,
    pub r_hi: f64,
    /// The isolated-atom energy the whole expansion is referenced to.
    pub e_h_atom: f64,
    /// Largest `|dE3|` on any node — the compact corner's value.
    pub peak: f64,
    /// Electronic-structure solves the build actually paid for.
    pub solves: usize,
}

/// The label this crate puts on the surface it computed. Says the model, the arithmetic
/// and the route, and deliberately does NOT say "exact", which would be a claim about the
/// world rather than about the basis.
pub const TRIMER_PROVENANCE: &str = "engine-computed STO-3G FCI three-body term, f64";

/// Compute the table node by node, handing each to `push` as `(index, x, y, c, dE3)`.
///
/// Allocation-free, which is what lets the browser build the table at load and push
/// straight into a fixed-size interpolator. The `x <-> y` symmetry of `dE3` is used: only
/// `i <= j` is solved, and the mirror node is handed over with the SAME float rather than
/// a second rounding of one number.
pub fn stream_trimer_table<F>(mut push: F) -> Option<TrimerMeta>
where
    F: FnMut(usize, f64, f64, f64, f64) -> bool,
{
    let e_h = atom_energy();
    // The two short sides only ever take `NR` distinct values, so their pair energies are
    // solved once each. The third side is a continuum and is not cacheable.
    let mut v_cache = [0.0f64; NR];
    for (i, v) in v_cache.iter_mut().enumerate() {
        *v = pair_energy(node_r(i));
    }
    let mut peak = 0.0f64;
    let mut solves = 0usize;
    for i in 0..NR {
        for j in i..NR {
            let (x, y) = (node_r(i), node_r(j));
            for k in 0..NU {
                let c = node_c(k);
                let u = 1.0 - c * c;
                let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
                let sin = (1.0 - u * u).max(0.0).sqrt();
                let e3 =
                    hydrogen_energy(&[[0.0, 0.0, 0.0], [x, 0.0, 0.0], [y * u, y * sin, 0.0]]);
                let d = e3 + 3.0 * e_h - (v_cache[i] + v_cache[j] + pair_energy(z));
                solves += 1;
                if d.abs() > peak {
                    peak = d.abs();
                }
                if !push(node_index(i, j, k), x, y, c, d) {
                    return None;
                }
                if i != j && !push(node_index(j, i, k), y, x, c, d) {
                    return None;
                }
            }
        }
    }
    Some(TrimerMeta {
        n_nodes: N_NODES,
        nr: NR,
        nu: NU,
        r_lo: R_LO,
        r_hi: R_HI,
        e_h_atom: e_h,
        peak,
        solves,
    })
}

// ================================================================ the interpolant

/// Catmull-Rom weights on a uniform grid of `n` nodes at fractional index `t`.
///
/// Public so the grid-sizing sweep (`examples/mbe3_grid_sweep.rs`) measures the SAME
/// scheme the table ships, rather than a re-implementation of it.
///
/// Returns the base node of a four-wide window and, over that window, the weights for the
/// value and for the derivative WITH RESPECT TO THE INDEX. The node slopes are centred
/// differences in the interior and the three-point one-sided formula at the two ends;
/// each node's slope is one fixed linear functional of the values, used by both intervals
/// that meet there, which is what makes the interpolant C1 across every node.
pub fn cr_weights(n: usize, t: f64) -> (usize, [f64; 4], [f64; 4]) {
    let i = (t.floor() as isize).clamp(0, n as isize - 2) as usize;
    let s = t - i as f64;
    let base = (i as isize - 1).clamp(0, n as isize - 4) as usize;

    // Node slopes as weights over the window.
    let mut d0 = [0.0f64; 4];
    let mut d1 = [0.0f64; 4];
    slope_weights(n, i, base, &mut d0);
    slope_weights(n, i + 1, base, &mut d1);

    let s2 = s * s;
    let s3 = s2 * s;
    let (h00, h10, h01, h11) = (
        2.0 * s3 - 3.0 * s2 + 1.0,
        s3 - 2.0 * s2 + s,
        -2.0 * s3 + 3.0 * s2,
        s3 - s2,
    );
    let (g00, g10, g01, g11) = (
        6.0 * s2 - 6.0 * s,
        3.0 * s2 - 4.0 * s + 1.0,
        -6.0 * s2 + 6.0 * s,
        3.0 * s2 - 2.0 * s,
    );

    let mut w = [0.0f64; 4];
    let mut dw = [0.0f64; 4];
    for a in 0..4 {
        w[a] = h10 * d0[a] + h11 * d1[a];
        dw[a] = g10 * d0[a] + g11 * d1[a];
    }
    w[i - base] += h00;
    dw[i - base] += g00;
    w[i + 1 - base] += h01;
    dw[i + 1 - base] += g01;
    (base, w, dw)
}

/// The node-slope functional for node `k`, expressed over the four-wide window at `base`.
fn slope_weights(n: usize, k: usize, base: usize, out: &mut [f64; 4]) {
    // Every node these formulas touch lies inside the four-wide window: `k == 0` can only
    // arise on the first interval, where `base == 0`, and `k == n - 1` only on the last,
    // where `base == n - 4`. `put` asserts that rather than assuming it.
    let mut put = |node: usize, w: f64| {
        let slot = node
            .checked_sub(base)
            .expect("slope stencil below the window");
        assert!(slot < 4, "slope stencil above the window");
        out[slot] += w;
    };
    if k == 0 {
        put(0, -1.5);
        put(1, 2.0);
        put(2, -0.5);
    } else if k == n - 1 {
        put(n - 3, 0.5);
        put(n - 2, -2.0);
        put(n - 1, 1.5);
    } else {
        put(k - 1, -0.5);
        put(k + 1, 0.5);
    }
}

/// The tabulated three-body surface, and the forces read off it.
///
/// A fixed array: no allocator anywhere, which is what lets the browser hold it. `loaded`
/// is false until every node has arrived, so a half-filled table contributes nothing
/// rather than contributing nonsense — and a `Sim` that never generates one behaves
/// EXACTLY as it did before this module existed, because [`TrimerTable::eval`] returns an
/// identical zero and adding zero to a finite float changes no bit.
///
/// `Clone` so a plant can mutate a copy without paying 7,293 electronic-structure solves
/// to rebuild the original.
#[derive(Clone)]
pub struct TrimerTable {
    v: [f64; N_NODES],
    filled: usize,
    pub loaded: bool,
    pub meta: TrimerMeta,
    /// ABSOLUTE cap on the interpolant's second derivative in SIDE coordinates,
    /// hartree/bohr^2 — the row-sum norm of the 3x3 side-space Hessian, maximised over the
    /// grid and widened.
    pub curvature_envelope: f64,
    /// LOCAL cap, per bohr: `||H|| <= curvature_per_gradient * max_a |dF/ds_a|` everywhere
    /// the sample reached. The drift bound takes the smaller of the two, which is what
    /// makes it a live reading of the configuration rather than a constant pinned to a
    /// corner of the table the trajectory never visits.
    pub curvature_per_gradient: f64,
    /// Largest jump in `dF/ds` across a SORT boundary (two sides equal), hartree/bohr.
    ///
    /// Reported rather than bounded away. The tabulated surface is exactly symmetric in
    /// its first two arguments but only symmetric-to-interpolation-error in the third, so
    /// the composed function of three unsorted sides is continuous — the sorted triple is
    /// a continuous function of the unsorted one — with a KINK where two sides cross. The
    /// potential is therefore still conservative; what the kink costs is a small
    /// discontinuity in the force, and this is its size.
    pub sort_kink: f64,
}

/// Widening factor on the measured curvature envelope. See
/// [`TrimerTable::measure_curvature_envelope`].
pub const ENVELOPE_WIDENING: f64 = 4.0;

impl TrimerTable {
    pub const fn empty() -> Self {
        Self {
            v: [0.0; N_NODES],
            filled: 0,
            loaded: false,
            meta: TrimerMeta {
                n_nodes: N_NODES,
                nr: NR,
                nu: NU,
                r_lo: R_LO,
                r_hi: R_HI,
                e_h_atom: 0.0,
                peak: 0.0,
                solves: 0,
            },
            curvature_envelope: 0.0,
            curvature_per_gradient: 0.0,
            sort_kink: 0.0,
        }
    }

    pub fn begin(&mut self) {
        self.filled = 0;
        self.loaded = false;
        self.curvature_envelope = 0.0;
        self.curvature_per_gradient = 0.0;
        self.sort_kink = 0.0;
    }

    pub fn knot(&mut self, index: usize, value: f64) -> bool {
        if index >= N_NODES || !value.is_finite() {
            return false;
        }
        self.v[index] = value;
        self.filled += 1;
        true
    }

    /// Close the table: adopt the metadata and MEASURE the curvature envelope the drift
    /// bound needs. Returns false if any node is missing.
    pub fn finish(&mut self, meta: TrimerMeta) -> bool {
        if self.filled < N_NODES {
            return false;
        }
        self.meta = meta;
        self.loaded = true;
        self.measure_envelopes();
        true
    }

    /// Raw node value, for the tests and the plants.
    pub fn node(&self, i: usize, j: usize, k: usize) -> f64 {
        self.v[node_index(i, j, k)]
    }

    /// Overwrite one node. The plants use it to mutate a table on purpose; nothing in the
    /// dynamics does.
    pub fn set_node(&mut self, i: usize, j: usize, k: usize, value: f64) {
        self.v[node_index(i, j, k)] = value;
        self.measure_envelopes();
    }

    /// Negate the whole surface — the sign-flip plant, and only the plant.
    pub fn negate(&mut self) {
        for x in self.v.iter_mut() {
            *x = -*x;
        }
        self.measure_envelopes();
    }

    /// Zero the surface wherever the triangle's perimeter is below `p`. The far-field
    /// plant: the dynamics must notice when the table stops being read where the physics
    /// lives.
    pub fn zero_inside_perimeter(&mut self, p: f64) {
        for i in 0..NR {
            for j in 0..NR {
                for k in 0..NU {
                    let (x, y, z) = node_geometry(i, j, k);
                    if x + y + z < p {
                        self.v[node_index(i, j, k)] = 0.0;
                    }
                }
            }
        }
        self.measure_envelopes();
    }

    /// The surface and its three side-derivatives at a triangle, hartree and
    /// hartree/bohr. `r` is the three sides in ANY order; the returned gradient is in the
    /// same order.
    ///
    /// Returns an exact zero outside the domain — the truncation the prereg's T2 gauges —
    /// and below `R_LO` extends the surface linearly in the short sides, C1 at the edge,
    /// so a violent collision meets a continuous force rather than a cliff.
    pub fn eval(&self, r: [f64; 3]) -> (f64, [f64; 3]) {
        if !self.loaded {
            return (0.0, [0.0; 3]);
        }
        // Sort, carrying the permutation. Three elements, exact comparisons, so the six
        // permutations agree bit-for-bit rather than to within a tolerance.
        let mut o = [0usize, 1, 2];
        if r[o[0]] > r[o[1]] {
            o.swap(0, 1);
        }
        if r[o[1]] > r[o[2]] {
            o.swap(1, 2);
        }
        if r[o[0]] > r[o[1]] {
            o.swap(0, 1);
        }
        let (v, g) = self.eval_branch(r[o[0]], r[o[1]], r[o[2]]);
        let mut out = [0.0f64; 3];
        for a in 0..3 {
            out[o[a]] = g[a];
        }
        (v, out)
    }

    /// The surface on ONE branch of the sort: `x` and `y` are taken as the two the table
    /// is indexed by and `z` as the third, whatever their order. Public to the module's
    /// own envelope measurement, which must stay on one branch to see a curvature rather
    /// than the kink where two branches meet.
    fn eval_branch(&self, x: f64, y: f64, z: f64) -> (f64, [f64; 3]) {
        // The MIDDLE side is what makes dE3 vanish; see the module header. The negated
        // comparisons reject NaN as well as an out-of-domain triangle.
        if !(y <= R_HI) || !(x > 0.0) {
            return (0.0, [0.0; 3]);
        }
        let u = ((x * x + y * y - z * z) / (2.0 * x * y)).clamp(-1.0, 1.0);
        let c = (1.0 - u).sqrt();

        // Below the grid's lower edge the surface is extended linearly, so the lookup is
        // clamped and the extrapolation is added back.
        let xq = x.max(R_LO);
        let yq = y.max(R_LO);
        let (f, ftx, fty, fc) = self.eval_grid(xq, yq, c);
        // Index-space derivatives to physical ones.
        let scale = (NR - 1) as f64;
        let fx = ftx * scale * dtau_dr(xq);
        let fy = fty * scale * dtau_dr(yq);
        let fc = fc / ((C_HI - C_LO) / (NU as f64 - 1.0));
        let f = f + (x - xq) * fx + (y - yq) * fy;

        // Chain rule from (x, y, c) to the three sides, with c = sqrt(1 - u):
        //   du/dx = (x^2 - y^2 + z^2) / (2 x^2 y)
        //   du/dy = (y^2 - x^2 + z^2) / (2 x y^2)
        //   du/dz = -z / (x y)
        //   dc/du = -1 / (2c)
        // `c >= sqrt(1/2)` for any sorted triple, so the division is never near zero.
        let (x2, y2, z2) = (x * x, y * y, z * z);
        let dc_du = -0.5 / c.max(1e-6);
        let du_dx = (x2 - y2 + z2) / (2.0 * x2 * y);
        let du_dy = (y2 - x2 + z2) / (2.0 * x * y2);
        let du_dz = -z / (x * y);
        let g = [
            fx + fc * dc_du * du_dx,
            fy + fc * dc_du * du_dy,
            fc * dc_du * du_dz,
        ];
        (f, g)
    }

    /// The interpolant in its own INDEX coordinates: value, and the three partials with
    /// respect to the fractional node index along each axis.
    fn eval_grid(&self, x: f64, y: f64, c: f64) -> (f64, f64, f64, f64) {
        let tx = (tau_of_r(x) * (NR - 1) as f64).clamp(0.0, (NR - 1) as f64);
        let ty = (tau_of_r(y) * (NR - 1) as f64).clamp(0.0, (NR - 1) as f64);
        let tc = ((c - C_LO) / (C_HI - C_LO) * (NU - 1) as f64).clamp(0.0, (NU - 1) as f64);
        let (bx, wx, dwx) = cr_weights(NR, tx);
        let (by, wy, dwy) = cr_weights(NR, ty);
        let (bc, wc, dwc) = cr_weights(NU, tc);

        // Contract x first into a 4x4 slab, once for the value weights and once for the
        // derivative weights; the y and c contractions are then four-term sums.
        let mut p = [[0.0f64; 4]; 4];
        let mut q = [[0.0f64; 4]; 4];
        for b in 0..4 {
            for e in 0..4 {
                let mut acc = 0.0;
                let mut dacc = 0.0;
                for a in 0..4 {
                    let v = self.v[node_index(bx + a, by + b, bc + e)];
                    acc += wx[a] * v;
                    dacc += dwx[a] * v;
                }
                p[b][e] = acc;
                q[b][e] = dacc;
            }
        }
        let (mut f, mut fx, mut fy, mut fc) = (0.0, 0.0, 0.0, 0.0);
        for e in 0..4 {
            let mut sv = 0.0;
            let mut sd = 0.0;
            let mut sq = 0.0;
            for b in 0..4 {
                sv += wy[b] * p[b][e];
                sd += dwy[b] * p[b][e];
                sq += wy[b] * q[b][e];
            }
            f += wc[e] * sv;
            fc += dwc[e] * sv;
            fy += wc[e] * sd;
            fx += wc[e] * sq;
        }
        (f, fx, fy, fc)
    }

    /// Measure the two curvature envelopes and the sort kink, once, when the table closes.
    ///
    /// # What is being bounded, and why there are two numbers
    ///
    /// [`crate::trimer::TrimerTable::eval`] composes the interpolant with a SORT, and a
    /// sort is not differentiable where two of its arguments cross. The composed potential
    /// is still continuous — the sorted triple is a continuous function of the unsorted
    /// one — but its gradient has a kink there, because the table is exactly symmetric in
    /// its first two arguments and only symmetric-to-interpolation-error in the third. A
    /// finite difference straddling that kink reports a second derivative that diverges as
    /// `1/h` and means nothing: measured here at 1283 Ha/bohr^2 with `h = 1e-4`, 152 with
    /// `h = 1e-3`, 37 with `h = 1e-2`, which is the signature of a jump rather than a
    /// curvature. The sampling therefore stays on ONE branch, via `eval_branch`, and the
    /// kink is measured separately and reported as what it is.
    ///
    /// The two envelopes are an ABSOLUTE cap and a LOCAL one, `||H|| <= B max_a |dF/ds_a|`.
    /// The drift bound takes whichever is smaller at the configuration in hand, which is
    /// what keeps it a live reading: a dispersed scene has tiny three-body gradients and
    /// therefore a tiny three-body stiffness, where the absolute cap alone would quote the
    /// compact corner's number forever.
    ///
    /// # The widening, and what it covers
    ///
    /// The interpolant is a tensor-product cubic, so along any one axis its second
    /// derivative is LINEAR — extremal at the cell edges, which the node lattice pins —
    /// while in the other two axes it is cubic and can overshoot the sampled values by the
    /// scheme's Lebesgue constant, `sum_a |w_a| <= 1.25` per axis, hence at most
    /// `1.25^2 = 1.6` for the two. [`ENVELOPE_WIDENING`] is 4, which clears that by 2.5x
    /// and covers the cell-centre-versus-edge difference along the third axis with it.
    /// Erring wide is the safe direction for a term that multiplies a bound.
    fn measure_envelopes(&mut self) {
        // The step is 1e-3 bohr: small against the grid's own spacing (0.06 bohr at the
        // compact end) and large enough that the C1-not-C2 jump at a cell boundary is
        // averaged rather than resolved.
        const HH: f64 = 1e-3;
        // How far from a sort boundary a sample has to sit for the difference to stay on
        // one branch. Forty steps, so the stencil clears it by a wide margin.
        const CLEAR: f64 = 40.0 * HH;
        let mut k_abs = 0.0f64;
        let mut per_grad = 0.0f64;
        for i in 0..(NR - 1) {
            for j in i..(NR - 1) {
                for k in 0..(NU - 1) {
                    let x = 0.5 * (node_r(i) + node_r(i + 1));
                    let y = 0.5 * (node_r(j) + node_r(j + 1));
                    let c = 0.5 * (node_c(k) + node_c(k + 1));
                    let u = 1.0 - c * c;
                    let z = (x * x + y * y - 2.0 * x * y * u).max(0.0).sqrt();
                    let mut sorted = [x, y, z];
                    sorted.sort_by(|a, b| a.partial_cmp(b).expect("finite sides"));
                    if sorted[1] - sorted[0] < CLEAR || sorted[2] - sorted[1] < CLEAR {
                        continue;
                    }
                    if sorted[1] > R_HI {
                        continue;
                    }
                    // Row sums of the side-space Hessian, by central differences of the
                    // analytic gradient ON ONE BRANCH. Symmetric by construction, so the
                    // row-sum norm bounds the spectral norm.
                    let mut rows = 0.0f64;
                    for a in 0..3 {
                        let mut lo = [x, y, z];
                        let mut hi = [x, y, z];
                        lo[a] -= HH;
                        hi[a] += HH;
                        let (_, glo) = self.eval_branch(lo[0], lo[1], lo[2]);
                        let (_, ghi) = self.eval_branch(hi[0], hi[1], hi[2]);
                        let row: f64 = (0..3)
                            .map(|b| ((ghi[b] - glo[b]) / (2.0 * HH)).abs())
                            .sum();
                        rows = rows.max(row);
                    }
                    if rows > k_abs {
                        k_abs = rows;
                    }
                    let (_, g) = self.eval_branch(x, y, z);
                    let gmax = g.iter().fold(0.0f64, |m, v| m.max(v.abs()));
                    if gmax > 1e-9 {
                        per_grad = per_grad.max(rows / gmax);
                    }
                }
            }
        }
        self.curvature_envelope = ENVELOPE_WIDENING * k_abs;
        self.curvature_per_gradient = ENVELOPE_WIDENING * per_grad;
        self.sort_kink = self.measure_sort_kink();
    }

    /// The force discontinuity at a sort boundary: the largest `|dF/ds|` jump across the
    /// surface where the second and third sorted sides cross. Reported, not bounded away.
    fn measure_sort_kink(&self) -> f64 {
        const EPS: f64 = 1e-6;
        let mut worst = 0.0f64;
        for i in 0..NR {
            for j in i..NR {
                let x = node_r(i);
                let y = node_r(j);
                if x < R_LO || y > R_HI || y - x < 0.05 {
                    continue;
                }
                // Two triangles either side of `z = y`, i.e. of the s2 <-> s3 crossing.
                let (_, ga) = self.eval([x, y, y - EPS]);
                let (_, gb) = self.eval([x, y, y + EPS]);
                for a in 0..3 {
                    worst = worst.max((ga[a] - gb[a]).abs());
                }
            }
        }
        worst
    }
}

/// Build the whole table natively. The browser streams instead; this is the convenience
/// the tests and the quench runner use.
pub fn generate() -> Option<TrimerTable> {
    let mut t = TrimerTable::empty();
    t.begin();
    let meta = stream_trimer_table(|i, _x, _y, _c, v| t.knot(i, v))?;
    if !t.finish(meta) {
        return None;
    }
    Some(t)
}
