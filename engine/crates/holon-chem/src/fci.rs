//! Determinant full CI: the exact-in-basis ground state of any first-row species this
//! crate can build a basis for, with its first and second derivative with respect to the
//! internuclear separation.
//!
//! # Why this exists when `h2.rs` already solves H2
//!
//! `h2.rs` solves H2 in closed form because symmetry hands it the molecular orbitals and
//! two electrons in two orbitals is a 2x2 matrix. Neither survives the first row. Ten
//! orbitals and fourteen electrons is 14 400 determinants, no symmetry shortcut writes
//! them down, and the answer is an eigenvalue rather than a root of a quadratic. So this
//! module does the general thing: build the determinant space, apply the Hamiltonian to
//! a vector without ever forming the matrix, and find the lowest eigenpair iteratively.
//!
//! # The orbital basis, and why it can be chosen freely
//!
//! FULL CI is invariant under any unitary transformation of the orbitals it is full in:
//! rotating the orbitals permutes the determinant space onto itself and leaves the span
//! — and therefore the lowest eigenvalue — untouched. That single fact does two jobs
//! here, and `tests/fci.rs` checks it directly rather than taking it on trust.
//!
//! First, it removes the SCF from the differentiation path. The orbitals are made
//! orthonormal by a CHOLESKY factorisation of the overlap (`X = L^{-T}`, so
//! `X^T S X = I`), which is a smooth, closed-form, derivative-friendly function of the
//! overlap matrix — unlike the symmetric orthogonalisation `S^{-1/2}`, whose eigenvectors
//! are not differentiable where eigenvalues meet, and unlike an SCF, which is a fixed
//! point rather than an expression.
//!
//! Second, it lets a good orbital basis be chosen WITHOUT paying for its derivatives. A
//! constant orthogonal matrix `U` — here from a cheap self-consistent field run at f64 —
//! is applied after the Cholesky step and treated as independent of `R`. Because
//! `E[X(R) U] = E[X(R)]` holds identically in `R` for every fixed `U`, both sides have
//! the same derivatives, so `U` improves the conditioning of the CI problem and changes
//! neither the energy nor its slope nor its curvature. What it buys is a compact ground
//! state, which is the difference between Davidson converging in tens of iterations and
//! hundreds.
//!
//! # The derivatives
//!
//! `E'` is Hellmann–Feynman, `E' = v^T H' v`, exact because `v` is a variational
//! eigenvector. `E''` needs the response: `E'' = v^T H'' v + 2 v^{(1)T} H' v` with
//! `(H - E) v^{(1)} = -(H' - E') v` and `v^{(1)}` orthogonal to `v`. That linear system
//! is positive semi-definite on the complement of `v` (E is the LOWEST eigenvalue), so
//! it is solved by projected conjugate gradients on the same matrix-free `H`. No finite
//! difference appears anywhere: the second derivative is as exact as the energy.
//!
//! # Two routes, because one route is an opinion
//!
//! [`FciSpace::sigma`] is the production route: the string factorisation of Knowles and
//! Handy (*Chem. Phys. Lett.* **111**, 315 (1984)), which never forms a matrix element
//! it does not need. [`FciSpace::sigma_reference`] enumerates the connected determinants
//! explicitly and applies the Slater–Condon rules to each pair, sharing no loop structure
//! with the first. [`dense_hamiltonian_ladder`] goes further down and builds the matrix
//! from raw creation and annihilation operators with their fermionic signs applied one at
//! a time — no Slater–Condon rule anywhere — which is feasible only for small spaces and
//! is what validates the rules the other two use.

use crate::dual::D2;
use crate::md::AoIntegrals;

/// Largest orbital count the determinant machinery accepts.
///
/// Twenty spin orbitals fit a `u32` mask with room to spare, and ten spatial orbitals is
/// a first-row diatomic in a minimal basis — the whole of what ELEMENTS-1 stakes.
pub const MAX_ORB: usize = 16;

// ------------------------------------------------- the spin-orbital ordering convention
//
// A determinant is `(prod_{p in alpha} a+_{p,alpha}) (prod_{q in beta} a+_{q,beta}) |vac>`
// with each block in ascending orbital order: alpha spin orbitals occupy bits `0..n`, beta
// spin orbitals bits `n..2n`. BLOCKED, not interleaved, and that is a load-bearing choice
// rather than a layout preference.
//
// The string factorisation computes each spin's fermionic signs within its own string —
// it never sees the other spin's occupation. That is only correct in the blocked ordering:
// a beta operator there passes the whole alpha block, costing `(-1)^{n_alpha}`, and every
// term of the Hamiltonian carries an EVEN number of beta operators, so those factors
// cancel exactly. In an interleaved ordering they do not, and the resulting Hamiltonian is
// the correct one conjugated by a diagonal matrix of signs — same spectrum, different
// matrix. That is a genuinely easy mistake to keep: every eigenvalue test passes and only
// a matrix-VECTOR comparison against a second route can see it, which is what
// `tests/fci.rs` does and how this comment came to be written.

// ------------------------------------------------------------------ linear algebra
//
// Small, dependency-free, and only the three operations the module needs. A general
// matrix library would be a dependency this crate does not have and does not want.

/// Cholesky factor `L` of a symmetric positive-definite matrix, `S = L L^T`, carrying
/// derivatives. Returns `None` if a pivot is not positive — which for an overlap matrix
/// means the basis has gone linearly dependent, and a silent answer would be worse than
/// a refusal.
pub fn cholesky(s: &[D2], n: usize) -> Option<Vec<D2>> {
    let mut l = vec![D2::c(0.0); n * n];
    for i in 0..n {
        for j in 0..=i {
            let mut acc = s[i * n + j];
            for k in 0..j {
                acc = acc - l[i * n + k] * l[j * n + k];
            }
            if i == j {
                // `!(x > 0)` rather than `x <= 0`, and the negation is the point: it
                // rejects NaN as well as a non-positive pivot, where `<=` would accept
                // NaN and hand it to `sqrt`. `table.rs` fences its grid the same way and
                // for the same reason.
                #[allow(clippy::neg_cmp_op_on_partial_ord)]
                if !(acc.v > 0.0) {
                    return None;
                }
                l[i * n + i] = acc.sqrt();
            } else {
                l[i * n + j] = acc / l[j * n + j];
            }
        }
    }
    Some(l)
}

/// The orthonormalising transform `X = (L^{-1})^T`, so that `X^T S X = I`.
///
/// Equivalently: Gram–Schmidt on the basis functions in their declared order, written as
/// a matrix so that it differentiates as an expression rather than as a procedure.
pub fn cholesky_orthonormaliser(s: &[D2], n: usize) -> Option<Vec<D2>> {
    let l = cholesky(s, n)?;
    // Forward-substitute for L^{-1}, one column of the identity at a time.
    let mut linv = vec![D2::c(0.0); n * n];
    for col in 0..n {
        for i in col..n {
            let mut acc = if i == col { D2::c(1.0) } else { D2::c(0.0) };
            for k in col..i {
                acc = acc - l[i * n + k] * linv[k * n + col];
            }
            linv[i * n + col] = acc / l[i * n + i];
        }
    }
    let mut x = vec![D2::c(0.0); n * n];
    for i in 0..n {
        for p in 0..n {
            x[i * n + p] = linv[p * n + i];
        }
    }
    Some(x)
}

/// Eigenvalues and eigenvectors of a real symmetric matrix by cyclic Jacobi rotations,
/// ascending. Columns of the returned matrix are the eigenvectors.
///
/// Jacobi rather than anything faster because these matrices are at most 16x16, and this
/// one has no pivoting, no shifts and no convergence heuristics to get wrong.
pub fn jacobi_eigh(a_in: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = a_in.to_vec();
    let mut v = vec![0.0f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }
    for _ in 0..100 {
        let mut off = 0.0f64;
        for i in 0..n {
            for j in (i + 1)..n {
                off += a[i * n + j] * a[i * n + j];
            }
        }
        if off <= 1e-30 {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p * n + q].abs() < 1e-300 {
                    continue;
                }
                let theta = (a[q * n + q] - a[p * n + p]) / (2.0 * a[p * n + q]);
                let t = if theta >= 0.0 {
                    1.0 / (theta + (1.0 + theta * theta).sqrt())
                } else {
                    -1.0 / (-theta + (1.0 + theta * theta).sqrt())
                };
                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;
                for k in 0..n {
                    let akp = a[k * n + p];
                    let akq = a[k * n + q];
                    a[k * n + p] = c * akp - s * akq;
                    a[k * n + q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p * n + k];
                    let aqk = a[q * n + k];
                    a[p * n + k] = c * apk - s * aqk;
                    a[q * n + k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let vkp = v[k * n + p];
                    let vkq = v[k * n + q];
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&i, &j| a[i * n + i].partial_cmp(&a[j * n + j]).unwrap());
    let evals: Vec<f64> = idx.iter().map(|&i| a[i * n + i]).collect();
    let mut evecs = vec![0.0f64; n * n];
    for (newc, &oldc) in idx.iter().enumerate() {
        for r in 0..n {
            evecs[r * n + newc] = v[r * n + oldc];
        }
    }
    (evals, evecs)
}

// ------------------------------------------------------------------ MO integrals

/// One- and two-electron integrals in an orthonormal molecular-orbital basis, carrying
/// derivatives.
pub struct MoIntegrals {
    pub n: usize,
    /// `h_pq`, row-major.
    pub h: Vec<D2>,
    /// `(pq|rs)` in chemist notation, indexed `[(p*n+q)*n2 + (r*n+s)]`.
    pub g: Vec<D2>,
}

impl MoIntegrals {
    #[inline]
    pub fn g_at(&self, p: usize, q: usize, r: usize, s: usize) -> D2 {
        let n = self.n;
        self.g[(p * n + q) * n * n + (r * n + s)]
    }
}

/// Transform AO integrals into the orthonormal basis `c` (AO index major, MO index
/// minor), carrying derivatives through every contraction.
pub fn transform(ao: &AoIntegrals, c: &[D2], n: usize) -> MoIntegrals {
    let mut h = vec![D2::c(0.0); n * n];
    for p in 0..n {
        for q in 0..n {
            let mut acc = D2::c(0.0);
            for i in 0..n {
                let cip = c[i * n + p];
                if cip.v == 0.0 && cip.d == 0.0 && cip.e == 0.0 {
                    continue;
                }
                for j in 0..n {
                    acc = acc + cip * c[j * n + q] * ao.h(i, j);
                }
            }
            h[p * n + q] = acc;
        }
    }

    // Four quarter transformations. Doing it in one O(n^8) contraction would be a
    // different answer only in cost, but at n = 10 that cost is 10^8 dual-number
    // multiplications per geometry, which is the difference between a curve and a wait.
    let n2 = n * n;
    let mut t1 = vec![D2::c(0.0); n2 * n2];
    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                for s in 0..n {
                    let mut acc = D2::c(0.0);
                    for l in 0..n {
                        acc = acc + ao.g(i, j, k, l) * c[l * n + s];
                    }
                    t1[((i * n + j) * n + k) * n + s] = acc;
                }
            }
        }
    }
    let mut t2 = vec![D2::c(0.0); n2 * n2];
    for i in 0..n {
        for j in 0..n {
            for r in 0..n {
                for s in 0..n {
                    let mut acc = D2::c(0.0);
                    for k in 0..n {
                        acc = acc + t1[((i * n + j) * n + k) * n + s] * c[k * n + r];
                    }
                    t2[((i * n + j) * n + r) * n + s] = acc;
                }
            }
        }
    }
    for x in t1.iter_mut() {
        *x = D2::c(0.0);
    }
    for i in 0..n {
        for q in 0..n {
            for r in 0..n {
                for s in 0..n {
                    let mut acc = D2::c(0.0);
                    for j in 0..n {
                        acc = acc + t2[((i * n + j) * n + r) * n + s] * c[j * n + q];
                    }
                    t1[((i * n + q) * n + r) * n + s] = acc;
                }
            }
        }
    }
    let mut g = vec![D2::c(0.0); n2 * n2];
    for p in 0..n {
        for q in 0..n {
            for r in 0..n {
                for s in 0..n {
                    let mut acc = D2::c(0.0);
                    for i in 0..n {
                        acc = acc + t1[((i * n + q) * n + r) * n + s] * c[i * n + p];
                    }
                    g[((p * n + q) * n + r) * n + s] = acc;
                }
            }
        }
    }
    MoIntegrals { n, h, g }
}

/// One derivative component of the MO integrals, in the form the CI code consumes.
///
/// `k` is the ONE-electron integral with the exchange trace folded in,
/// `k_pq = h_pq - 1/2 sum_r (pr|rq)`, which is what turns the two-electron operator into
/// the plain product `E_pq E_rs` and is why the sigma routine below has no separate
/// exchange pass.
pub struct CiInts {
    pub n: usize,
    pub k: Vec<f64>,
    /// `(pq|rs)` as an `n^2 x n^2` matrix.
    pub g: Vec<f64>,
}

/// Which derivative order of the MO integrals to extract.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Value,
    First,
    Second,
}

pub fn ci_ints(mo: &MoIntegrals, order: Order) -> CiInts {
    let n = mo.n;
    let pick = |x: D2| match order {
        Order::Value => x.v,
        Order::First => x.d,
        Order::Second => x.e,
    };
    let g: Vec<f64> = mo.g.iter().map(|&x| pick(x)).collect();
    let mut k = vec![0.0f64; n * n];
    for p in 0..n {
        for q in 0..n {
            let mut acc = pick(mo.h[p * n + q]);
            for r in 0..n {
                acc -= 0.5 * g[(p * n + r) * n * n + (r * n + q)];
            }
            k[p * n + q] = acc;
        }
    }
    CiInts { n, k, g }
}

// ------------------------------------------------------------------ strings

/// The occupation strings of one spin, with the single-excitation couplings that
/// generate the whole Hamiltonian.
pub struct Strings {
    pub n_orb: usize,
    pub n_elec: usize,
    pub masks: Vec<u32>,
    /// Mask to string index, or `-1`.
    lookup: Vec<i32>,
    /// `singles[j]` lists `(pq, sign, i)` with `a+_p a_q |j> = sign |i>`, `pq = p*n+q`.
    pub singles: Vec<Vec<(u16, f64, u32)>>,
}

/// Apply `a+_p a_q` to an occupation mask, returning the fermionic sign and the result.
///
/// The sign is counted, not tabulated: annihilating `q` costs the parity of the occupied
/// orbitals below it, and creating `p` costs the parity below `p` in what is left.
#[inline]
pub fn excite(mask: u32, p: usize, q: usize) -> Option<(f64, u32)> {
    if (mask >> q) & 1 == 0 {
        return None;
    }
    let m1 = mask ^ (1 << q);
    let s1 = if (mask & ((1u32 << q) - 1)).count_ones() & 1 == 1 {
        -1.0
    } else {
        1.0
    };
    if (m1 >> p) & 1 == 1 {
        return None;
    }
    let s2 = if (m1 & ((1u32 << p) - 1)).count_ones() & 1 == 1 {
        -1.0
    } else {
        1.0
    };
    Some((s1 * s2, m1 | (1 << p)))
}

impl Strings {
    pub fn new(n_orb: usize, n_elec: usize) -> Strings {
        assert!(n_orb <= MAX_ORB);
        let mut masks = Vec::new();
        for m in 0u32..(1u32 << n_orb) {
            if m.count_ones() as usize == n_elec {
                masks.push(m);
            }
        }
        let mut lookup = vec![-1i32; 1usize << n_orb];
        for (i, &m) in masks.iter().enumerate() {
            lookup[m as usize] = i as i32;
        }
        let mut singles = Vec::with_capacity(masks.len());
        for &m in masks.iter() {
            let mut list = Vec::new();
            for q in 0..n_orb {
                for p in 0..n_orb {
                    if let Some((sgn, m2)) = excite(m, p, q) {
                        let idx = lookup[m2 as usize];
                        debug_assert!(idx >= 0);
                        list.push(((p * n_orb + q) as u16, sgn, idx as u32));
                    }
                }
            }
            singles.push(list);
        }
        Strings {
            n_orb,
            n_elec,
            masks,
            lookup,
            singles,
        }
    }

    pub fn len(&self) -> usize {
        self.masks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.masks.is_empty()
    }

    pub fn index_of(&self, mask: u32) -> Option<usize> {
        let i = self.lookup[mask as usize];
        if i < 0 {
            None
        } else {
            Some(i as usize)
        }
    }
}

// ------------------------------------------------------------------ the CI space

/// The determinant space of one species at one `S_z`, and the machinery that applies the
/// Hamiltonian to a vector in it.
pub struct FciSpace {
    pub n_orb: usize,
    pub alpha: Strings,
    pub beta: Strings,
    pub n_det: usize,
}

impl FciSpace {
    pub fn new(n_orb: usize, n_alpha: usize, n_beta: usize) -> FciSpace {
        let alpha = Strings::new(n_orb, n_alpha);
        let beta = Strings::new(n_orb, n_beta);
        let n_det = alpha.len() * beta.len();
        FciSpace {
            n_orb,
            alpha,
            beta,
            n_det,
        }
    }

    /// Diagonal of the Hamiltonian, for the Davidson preconditioner and its start vector.
    ///
    /// ```text
    /// <D|H|D> = sum_{p in occ} h_pp + 1/2 sum_{p,q in occ} [ (pp|qq) - delta_spin (pq|qp) ]
    /// ```
    ///
    /// written over SPATIAL orbitals, where the spin delta becomes: the exchange term
    /// survives within the alpha set and within the beta set and not between them.
    pub fn diagonal(&self, ci: &CiInts) -> Vec<f64> {
        let n = self.n_orb;
        let n2 = n * n;
        let nb = self.beta.len();
        let mut d = vec![0.0f64; self.n_det];
        let g = |p: usize, q: usize, r: usize, s: usize| ci.g[(p * n + q) * n2 + (r * n + s)];
        // The physical one-electron integral: `k` carries the exchange fold that turns
        // the two-electron operator into a plain product, and the diagonal is stated in
        // terms of the unfolded `h`.
        let hph = |p: usize, q: usize| -> f64 {
            let mut x = ci.k[p * n + q];
            for r in 0..n {
                x += 0.5 * g(p, r, r, q);
            }
            x
        };
        let occ = |m: u32| -> Vec<usize> { (0..n).filter(|&p| (m >> p) & 1 == 1).collect() };
        for (ia, &ma) in self.alpha.masks.iter().enumerate() {
            let oa = occ(ma);
            for (ib, &mb) in self.beta.masks.iter().enumerate() {
                let ob = occ(mb);
                let mut e = 0.0f64;
                for &p in oa.iter().chain(ob.iter()) {
                    e += hph(p, p);
                }
                for set in [&oa, &ob] {
                    for &p in set.iter() {
                        for &q in set.iter() {
                            e += 0.5 * (g(p, p, q, q) - g(p, q, q, p));
                        }
                    }
                }
                for &p in oa.iter() {
                    for &q in ob.iter() {
                        e += g(p, p, q, q);
                    }
                }
                d[ia * nb + ib] = e;
            }
        }
        d
    }

    /// `sigma = H c`, by the string factorisation.
    ///
    /// Three blocks, because the Hamiltonian splits into terms that move only beta
    /// electrons, only alpha electrons, and one of each — and each block is a different
    /// contraction over the same single-excitation lists.
    pub fn sigma(&self, ci: &CiInts, c: &[f64], out: &mut [f64]) {
        let n = self.n_orb;
        let n2 = n * n;
        let na = self.alpha.len();
        let nb = self.beta.len();
        for x in out.iter_mut() {
            *x = 0.0;
        }

        // --- same-spin blocks. For each source string, walk one single excitation and
        // then a second from where it landed; the composite sign is the product, and the
        // resulting vector over destination strings is that source's column.
        let mut f = vec![0.0f64; na.max(nb)];

        for jb in 0..nb {
            for x in f[..nb].iter_mut() {
                *x = 0.0;
            }
            for &(kl, s1, kb) in self.beta.singles[jb].iter() {
                f[kb as usize] += s1 * ci.k[kl as usize];
                for &(ij, s2, ib) in self.beta.singles[kb as usize].iter() {
                    f[ib as usize] += 0.5 * s1 * s2 * ci.g[ij as usize * n2 + kl as usize];
                }
            }
            for (ib, &fv) in f[..nb].iter().enumerate() {
                if fv == 0.0 {
                    continue;
                }
                for ia in 0..na {
                    out[ia * nb + ib] += fv * c[ia * nb + jb];
                }
            }
        }

        for ja in 0..na {
            for x in f[..na].iter_mut() {
                *x = 0.0;
            }
            for &(kl, s1, ka) in self.alpha.singles[ja].iter() {
                f[ka as usize] += s1 * ci.k[kl as usize];
                for &(ij, s2, ia) in self.alpha.singles[ka as usize].iter() {
                    f[ia as usize] += 0.5 * s1 * s2 * ci.g[ij as usize * n2 + kl as usize];
                }
            }
            for (ia, &fv) in f[..na].iter().enumerate() {
                if fv == 0.0 {
                    continue;
                }
                let (src, dst) = (ja * nb, ia * nb);
                for ib in 0..nb {
                    out[dst + ib] += fv * c[src + ib];
                }
            }
        }

        // --- the mixed block: one alpha excitation and one beta excitation, coupled
        // through the two-electron integral. `t` is the beta half, built once per alpha
        // string and then contracted for each of that string's own excitations.
        let mut t = vec![0.0f64; n2 * nb];
        let mut vrow = vec![0.0f64; nb];
        for ja in 0..na {
            for x in t.iter_mut() {
                *x = 0.0;
            }
            let crow = &c[ja * nb..(ja + 1) * nb];
            // Indexed rather than iterated: the body scatters into `t` at indices the
            // excitation list supplies, so the loop variable is the SOURCE string label
            // and not merely a cursor into `crow`.
            #[allow(clippy::needless_range_loop)]
            for jb in 0..nb {
                let cv = crow[jb];
                if cv == 0.0 {
                    continue;
                }
                for &(kl, s, ib) in self.beta.singles[jb].iter() {
                    t[kl as usize * nb + ib as usize] += s * cv;
                }
            }
            for &(ij, sa, ia) in self.alpha.singles[ja].iter() {
                for x in vrow.iter_mut() {
                    *x = 0.0;
                }
                let grow = &ci.g[ij as usize * n2..(ij as usize + 1) * n2];
                for (kl, &gv) in grow.iter().enumerate() {
                    if gv == 0.0 {
                        continue;
                    }
                    let trow = &t[kl * nb..(kl + 1) * nb];
                    for ib in 0..nb {
                        vrow[ib] += gv * trow[ib];
                    }
                }
                let dst = ia as usize * nb;
                for ib in 0..nb {
                    out[dst + ib] += sa * vrow[ib];
                }
            }
        }
    }

    /// `sigma = H c` by an INDEPENDENT route: enumerate the determinants connected to
    /// each determinant and apply the Slater–Condon rules pair by pair.
    ///
    /// Shares no loop structure, no intermediate and no factorisation with [`Self::sigma`]
    /// — only the integrals and the rules. It is `O(N_det * N_connected)` rather than the
    /// string method's factorised cost, which is why it is a checker and not the
    /// production path.
    pub fn sigma_reference(&self, ci: &CiInts, c: &[f64], out: &mut [f64]) {
        let n = self.n_orb;
        let nb = self.beta.len();
        for x in out.iter_mut() {
            *x = 0.0;
        }
        let combine = |ma: u32, mb: u32| -> u64 { (ma as u64) | ((mb as u64) << n) };
        let dets: Vec<u64> = (0..self.n_det)
            .map(|d| combine(self.alpha.masks[d / nb], self.beta.masks[d % nb]))
            .collect();

        for i in 0..self.n_det {
            for j in 0..self.n_det {
                let (di, dj) = (dets[i], dets[j]);
                let diff = di ^ dj;
                if diff.count_ones() > 4 {
                    continue;
                }
                let h = slater_condon(di, dj, ci, n);
                if h != 0.0 {
                    out[i] += h * c[j];
                }
            }
        }
    }
}

/// `<D_i| H |D_j>` for two determinants given as interleaved spin-orbital masks, by the
/// Slater–Condon rules.
///
/// Written against the rules as stated, in terms of the PHYSICAL one- and two-electron
/// integrals rather than the folded `k` — a second expression of the same operator, so
/// that agreement with the string route is evidence about the algebra rather than about
/// one shared rewriting.
///
/// # The phase is applied, not derived
///
/// The rules give a matrix element up to a sign, and the sign is the one place where a
/// plausible-looking closed form ("count the occupied orbitals between them") is wrong
/// whenever the two intervals overlap. So it is not stated as a formula here: the
/// corresponding string of creation and annihilation operators is APPLIED to `dj` and
/// the sign it produces is used. That is the definition rather than a rule about it.
pub fn slater_condon(di: u64, dj: u64, ci: &CiInts, n: usize) -> f64 {
    let n2 = n * n;
    let hph = |p: usize, q: usize| -> f64 {
        let mut x = ci.k[p * n + q];
        for r in 0..n {
            x += 0.5 * ci.g[(p * n + r) * n2 + (r * n + q)];
        }
        x
    };
    let g = |p: usize, q: usize, r: usize, s: usize| ci.g[(p * n + q) * n2 + (r * n + s)];
    let spatial = |so: usize| so % n;
    let spin = |so: usize| so / n;

    let only_i: Vec<usize> = (0..2 * n)
        .filter(|&b| (di >> b) & 1 == 1 && (dj >> b) & 1 == 0)
        .collect();
    let only_j: Vec<usize> = (0..2 * n)
        .filter(|&b| (dj >> b) & 1 == 1 && (di >> b) & 1 == 0)
        .collect();
    if only_i.len() != only_j.len() || only_i.len() > 2 {
        return 0.0;
    }
    let common: Vec<usize> = (0..2 * n)
        .filter(|&b| (di >> b) & 1 == 1 && (dj >> b) & 1 == 1)
        .collect();

    // `ops` are applied right to left, so the annihilations happen first.
    let apply = |ops: &[(bool, usize)]| -> f64 {
        let mut sgn = 1.0f64;
        let mut cur = dj;
        for &(is_create, so) in ops.iter().rev() {
            let occupied = (cur >> so) & 1 == 1;
            if is_create == occupied {
                return 0.0;
            }
            if (cur & ((1u64 << so) - 1)).count_ones() & 1 == 1 {
                sgn = -sgn;
            }
            cur ^= 1 << so;
        }
        if cur == di {
            sgn
        } else {
            0.0
        }
    };

    match only_i.len() {
        0 => {
            let mut e = 0.0;
            for &p in common.iter() {
                e += hph(spatial(p), spatial(p));
            }
            for &p in common.iter() {
                for &q in common.iter() {
                    e += 0.5 * g(spatial(p), spatial(p), spatial(q), spatial(q));
                    if spin(p) == spin(q) {
                        e -= 0.5 * g(spatial(p), spatial(q), spatial(q), spatial(p));
                    }
                }
            }
            e
        }
        1 => {
            let (m, p) = (only_j[0], only_i[0]);
            if spin(m) != spin(p) {
                return 0.0;
            }
            let (sm, sp) = (spatial(m), spatial(p));
            let mut e = hph(sp, sm);
            for &r in common.iter() {
                let sr = spatial(r);
                e += g(sp, sm, sr, sr);
                if spin(r) == spin(p) {
                    e -= g(sp, sr, sr, sm);
                }
            }
            e * apply(&[(true, p), (false, m)])
        }
        _ => {
            let (m, nn) = (only_j[0], only_j[1]);
            let (p, q) = (only_i[0], only_i[1]);
            let mut e = 0.0;
            if spin(p) == spin(m) && spin(q) == spin(nn) {
                e += g(spatial(p), spatial(m), spatial(q), spatial(nn));
            }
            if spin(p) == spin(nn) && spin(q) == spin(m) {
                e -= g(spatial(p), spatial(nn), spatial(q), spatial(m));
            }
            e * apply(&[(true, p), (true, q), (false, nn), (false, m)])
        }
    }
}

/// The determinant Hamiltonian built from RAW LADDER OPERATORS, with every fermionic sign
/// applied one operator at a time and no Slater–Condon rule anywhere.
///
/// ```text
/// H = sum_pq h_pq a+_p a_q + 1/2 sum_pqrs (pq|rs) a+_p a+_r a_s a_q
/// ```
///
/// This is the deepest of the three routes and the slowest: it costs `N_det * n^4` and
/// forms a dense matrix, so it is only usable on small spaces. What it is FOR is
/// checking the Slater–Condon rules the other two routes rely on — a rule mistyped in
/// both of those would agree with itself, and only a route that does not use the rules
/// can see it.
pub fn dense_hamiltonian_ladder(space: &FciSpace, ci: &CiInts, n: usize) -> Vec<f64> {
    let nd = space.n_det;
    let nb = space.beta.len();
    let n2 = n * n;
    let hph = |p: usize, q: usize| -> f64 {
        let mut x = ci.k[p * n + q];
        for r in 0..n {
            x += 0.5 * ci.g[(p * n + r) * n2 + (r * n + q)];
        }
        x
    };

    let combine = |ma: u32, mb: u32| -> u32 { ma | (mb << n) };
    let dets: Vec<u32> = (0..nd)
        .map(|d| combine(space.alpha.masks[d / nb], space.beta.masks[d % nb]))
        .collect();
    let mut index = std::collections::HashMap::new();
    for (i, &d) in dets.iter().enumerate() {
        index.insert(d, i);
    }

    // Ladder primitives. `true` creates.
    fn ladder(det: u32, is_create: bool, so: u32) -> Option<(f64, u32)> {
        let occupied = (det >> so) & 1 == 1;
        if is_create == occupied {
            return None;
        }
        let below = (det & ((1u32 << so) - 1)).count_ones();
        let sgn = if below & 1 == 1 { -1.0 } else { 1.0 };
        Some((sgn, det ^ (1 << so)))
    }
    fn apply(det: u32, ops: &[(bool, u32)]) -> Option<(f64, u32)> {
        let mut sgn = 1.0f64;
        let mut cur = det;
        for &(is_create, so) in ops.iter().rev() {
            let (s, next) = ladder(cur, is_create, so)?;
            sgn *= s;
            cur = next;
        }
        Some((sgn, cur))
    }

    let mut h = vec![0.0f64; nd * nd];
    let nso = 2 * n;
    for (col, &d) in dets.iter().enumerate() {
        for p in 0..nso {
            for q in 0..nso {
                if p / n != q / n {
                    continue;
                }
                let hv = hph(p % n, q % n);
                if hv == 0.0 {
                    continue;
                }
                if let Some((sg, nd2)) = apply(d, &[(true, p as u32), (false, q as u32)]) {
                    if let Some(&row) = index.get(&nd2) {
                        h[row * nd + col] += sg * hv;
                    }
                }
            }
        }
        for p in 0..nso {
            for q in 0..nso {
                if p / n != q / n {
                    continue;
                }
                for r in 0..nso {
                    for s in 0..nso {
                        if r / n != s / n {
                            continue;
                        }
                        let gv = ci.g[((p % n) * n + q % n) * n2 + ((r % n) * n + s % n)];
                        if gv == 0.0 {
                            continue;
                        }
                        if let Some((sg, nd2)) = apply(
                            d,
                            &[
                                (true, p as u32),
                                (true, r as u32),
                                (false, s as u32),
                                (false, q as u32),
                            ],
                        ) {
                            if let Some(&row) = index.get(&nd2) {
                                h[row * nd + col] += sg * gv / 2.0;
                            }
                        }
                    }
                }
            }
        }
    }
    h
}

// ------------------------------------------------------------------ eigensolver

/// What one FCI solve produced, with the diagnostics that say how hard it was.
pub struct Solution {
    /// Electronic energy and its two exact derivatives with respect to the separation.
    pub e: D2,
    pub vector: Vec<f64>,
    pub davidson_iters: usize,
    pub cg_iters: usize,
    /// Final Davidson residual norm, and the CG residual as a fraction of its right-hand
    /// side. Reported, because a solve that did not converge must say so rather than
    /// hand back a plausible number.
    pub residual: f64,
    pub cg_residual: f64,
}

/// Lowest eigenpair by Davidson, matrix-free.
pub fn davidson(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    tol: f64,
    max_iter: usize,
) -> (f64, Vec<f64>, usize, f64) {
    let nd = space.n_det;
    if nd == 1 {
        let mut s = vec![0.0f64; 1];
        space.sigma(ci, &[1.0], &mut s);
        return (s[0], vec![1.0], 0, 0.0);
    }
    let max_sub = 48.min(nd);
    let mut basis: Vec<Vec<f64>> = Vec::new();
    let mut hbasis: Vec<Vec<f64>> = Vec::new();

    let start = diag
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap();

    // The start vector is the lowest-diagonal determinant PLUS a small generic
    // perturbation, and the perturbation is not a hedge.
    //
    // `H` commutes with `S^2`, so a Krylov space built from a vector lying in one spin
    // sector never leaves it. A single determinant can be exactly such a vector, and when
    // it is, Davidson converges cleanly — to the lowest state of the WRONG multiplet,
    // reporting a small residual the whole way. That is what carbon did here under a
    // rotated orbital basis: a converged eigenvector 0.07 hartree above the true ground
    // state, with nothing in the solve to say so. Breaking the symmetry of the start
    // guarantees a nonzero overlap with every eigenvector, which is what makes "lowest
    // eigenvalue found" mean the lowest one. Deterministic, so a run is reproducible.
    let mut v0 = vec![0.0f64; nd];
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    for x in v0.iter_mut() {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *x = 1e-3 * (((seed >> 33) as f64 / (1u64 << 30) as f64) - 1.0);
    }
    v0[start] += 1.0;
    basis.push(normalised(&v0));

    let mut theta = diag[start];
    let mut x = basis[0].clone();
    let mut resid = f64::INFINITY;

    for iter in 0..max_iter {
        while hbasis.len() < basis.len() {
            let mut w = vec![0.0f64; nd];
            space.sigma(ci, &basis[hbasis.len()], &mut w);
            hbasis.push(w);
        }
        let m = basis.len();
        let mut sub = vec![0.0f64; m * m];
        for i in 0..m {
            for j in 0..m {
                sub[i * m + j] = dot(&basis[i], &hbasis[j]);
            }
        }
        // Symmetrise: exact in a correct build, and it keeps the Jacobi sweep well posed.
        for i in 0..m {
            for j in 0..i {
                let a = 0.5 * (sub[i * m + j] + sub[j * m + i]);
                sub[i * m + j] = a;
                sub[j * m + i] = a;
            }
        }
        let (evals, evecs) = jacobi_eigh(&sub, m);
        theta = evals[0];
        x = vec![0.0f64; nd];
        for i in 0..m {
            let ci_ = evecs[i * m];
            if ci_ != 0.0 {
                axpy(ci_, &basis[i], &mut x);
            }
        }
        let mut hx = vec![0.0f64; nd];
        for i in 0..m {
            let ci_ = evecs[i * m];
            if ci_ != 0.0 {
                axpy(ci_, &hbasis[i], &mut hx);
            }
        }
        let mut r = hx;
        axpy(-theta, &x, &mut r);
        resid = norm(&r);
        if resid < tol || iter + 1 == max_iter {
            return (theta, x, iter + 1, resid);
        }
        if basis.len() >= max_sub {
            // Thick restart: the Ritz vector AND the direction the solve was moving in.
            // Restarting on the Ritz vector alone throws away the descent direction and
            // the next few iterations spend themselves rediscovering it.
            let mut d = r.clone();
            let p = dot(&x, &d);
            axpy(-p, &x, &mut d);
            let nd_ = norm(&d);
            basis.clear();
            hbasis.clear();
            basis.push(normalised(&x));
            if nd_ > 1e-10 {
                scale(1.0 / nd_, &mut d);
                basis.push(d);
            }
            continue;
        }

        // Davidson preconditioner, with the degeneracy case handled rather than
        // suppressed. A determinant whose diagonal sits ON the Ritz value would be a
        // division by zero, and this is not a corner: an open-shell first-row ATOM has
        // exactly degenerate p occupations, so the determinants the reference couples to
        // are often ALL at the reference's own diagonal. Zeroing them there empties the
        // correction vector and the solve stops after one iteration reporting a residual
        // it has not reduced — which is what this code did until nitrogen and oxygen
        // came back above their own SCF energies. Leaving such a component unscaled
        // keeps its direction, which is all the preconditioner was ever for.
        let mut corr = r.clone();
        for i in 0..nd {
            let d = theta - diag[i];
            if d.abs() > 1e-8 {
                corr[i] = r[i] / d;
            }
        }
        // Two candidate expansion vectors, tried in order: the preconditioned residual,
        // then the raw one. The raw residual is orthogonal to the current Ritz vector by
        // construction and cannot be empty unless the solve has actually converged, so
        // the fallback is what makes "no new direction" mean convergence.
        let mut added = false;
        for cand in [corr, r.clone()] {
            let mut w = cand;
            for b in basis.iter() {
                let p = dot(b, &w);
                axpy(-p, b, &mut w);
            }
            // A second pass: one Gram-Schmidt sweep loses orthogonality when the
            // candidate is nearly in the span already, which is exactly the case here.
            for b in basis.iter() {
                let p = dot(b, &w);
                axpy(-p, b, &mut w);
            }
            let nw = norm(&w);
            if nw > 1e-10 {
                scale(1.0 / nw, &mut w);
                basis.push(w);
                added = true;
                break;
            }
        }
        if !added {
            return (theta, x, iter + 1, resid);
        }
    }
    (theta, x, max_iter, resid)
}

/// Solve `(H - E) w = b` on the orthogonal complement of `v`, by projected conjugate
/// gradients with a Jacobi preconditioner.
///
/// The operator is positive semi-definite there because `E` is the LOWEST eigenvalue, so
/// CG is the right method rather than a convenient one; the projection removes the one
/// direction where it is singular.
#[allow(clippy::too_many_arguments)]
fn cg_response(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    e: f64,
    v: &[f64],
    b: &[f64],
    tol: f64,
    max_iter: usize,
) -> (Vec<f64>, usize, f64) {
    let nd = space.n_det;
    let project = |x: &mut Vec<f64>| {
        let p = dot(v, x);
        axpy(-p, v, x);
    };
    let mut w = vec![0.0f64; nd];
    let mut r = b.to_vec();
    project(&mut r);
    let b_norm = norm(&r);
    if b_norm < 1e-300 {
        return (w, 0, 0.0);
    }
    let precond = |x: &[f64], out: &mut Vec<f64>| {
        for i in 0..nd {
            let d = diag[i] - e;
            out[i] = if d > 1e-6 { x[i] / d } else { x[i] };
        }
    };
    let mut z = vec![0.0f64; nd];
    precond(&r, &mut z);
    project(&mut z);
    let mut p = z.clone();
    let mut rz = dot(&r, &z);
    let mut hp = vec![0.0f64; nd];
    for it in 0..max_iter {
        space.sigma(ci, &p, &mut hp);
        axpy(-e, &p, &mut hp);
        project(&mut hp);
        let php = dot(&p, &hp);
        if php <= 0.0 {
            // The complement of `v` is where `H - E` is positive; landing here means the
            // eigenvector is not converged well enough for the response to be meaningful,
            // and reporting the residual is more honest than continuing.
            return (w, it, norm(&r) / b_norm);
        }
        let alpha = rz / php;
        axpy(alpha, &p, &mut w);
        axpy(-alpha, &hp, &mut r);
        project(&mut r);
        let rn = norm(&r);
        if rn / b_norm < tol {
            return (w, it + 1, rn / b_norm);
        }
        precond(&r, &mut z);
        project(&mut z);
        let rz_new = dot(&r, &z);
        let beta = rz_new / rz;
        rz = rz_new;
        for i in 0..nd {
            p[i] = z[i] + beta * p[i];
        }
    }
    (w, max_iter, norm(&r) / b_norm)
}

/// Solve for the ground state and both derivatives of its energy.
pub fn solve(space: &FciSpace, mo: &MoIntegrals) -> Solution {
    let ci0 = ci_ints(mo, Order::Value);
    let ci1 = ci_ints(mo, Order::First);
    let ci2 = ci_ints(mo, Order::Second);
    let diag = space.diagonal(&ci0);
    let (e, v, iters, residual) = davidson(space, &ci0, &diag, 1e-11, 1200);

    // E' = <v|H'|v>: exact for a variational eigenvector, with no response needed,
    // because the eigenvector's own derivative enters at second order.
    let nd = space.n_det;
    let mut h1v = vec![0.0f64; nd];
    space.sigma(&ci1, &v, &mut h1v);
    let e1 = dot(&v, &h1v);

    // E'' = <v|H''|v> + 2 <v^(1)|H'|v>, with (H - E) v^(1) = -(H' - E') v.
    let mut h2v = vec![0.0f64; nd];
    space.sigma(&ci2, &v, &mut h2v);
    let e2_direct = dot(&v, &h2v);
    let mut rhs = h1v.clone();
    axpy(-e1, &v, &mut rhs);
    for x in rhs.iter_mut() {
        *x = -*x;
    }
    let (w, cg_iters, cg_residual) =
        cg_response(space, &ci0, &diag, e, &v, &rhs, 1e-10, 2000);
    let e2 = e2_direct + 2.0 * dot(&w, &h1v);

    Solution {
        e: D2::new(e, e1, e2),
        vector: v,
        davidson_iters: iters,
        cg_iters,
        residual,
        cg_residual,
    }
}

// --- small vector helpers, written here because the crate has no linear algebra dep ---

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn axpy(a: f64, x: &[f64], y: &mut [f64]) {
    for (yi, xi) in y.iter_mut().zip(x.iter()) {
        *yi += a * xi;
    }
}

fn scale(a: f64, x: &mut [f64]) {
    for xi in x.iter_mut() {
        *xi *= a;
    }
}

fn norm(a: &[f64]) -> f64 {
    dot(a, a).sqrt()
}

fn normalised(a: &[f64]) -> Vec<f64> {
    let mut v = a.to_vec();
    let n = norm(&v);
    if n > 0.0 {
        scale(1.0 / n, &mut v);
    }
    v
}
