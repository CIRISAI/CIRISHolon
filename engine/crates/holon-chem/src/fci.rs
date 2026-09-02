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

use crate::sigma_op::{CpuProvider, DeviceClass, SigmaProvider};
use crate::dual::D2;
use crate::md::AoIntegrals;

/// Largest orbital count the determinant machinery accepts.
///
/// Sixty-four spatial orbitals fit a `u64` mask per spin string. The bound is the mask
/// width and nothing else: a string is one bit per spatial orbital, so `MAX_ORB` and the
/// mask type move together and neither may be changed alone.
///
/// # Why this was widened, and what it is NOT
///
/// It was `u32` (thirty-two orbitals) through ELEMENTS-1 and MIXTURES-1, which covered
/// every first- and second-row species. It does not cover ELEMENTS-3: a xenon atom is 29
/// contracted functions in STO-3G with six Cartesian d components per d shell, so Xe2 is
/// 58 spatial orbitals and could not be REPRESENTED at all, let alone solved.
///
/// This is a widening of the address space, not a claim about tractability. Whether a
/// space of a given size can be solved is a separate question answered by `n_det` and the
/// route chosen for it; `MAX_ORB` only says which spaces can be written down. Most spaces
/// this bound now admits are astronomically beyond the determinant route and reach the
/// answer, if at all, through DMRG under MIXTURES-1's D1 rule.
pub const MAX_ORB: usize = 64;

/// A whole determinant as ONE integer: the alpha string in bits `0..n`, the beta string in
/// bits `n..2n`, matching the blocked spin-orbital convention stated below.
///
/// # Why this is wider than [`Mask`] and not the same type
///
/// A determinant packs TWO strings, so it needs `2 * MAX_ORB` bits where a string needs
/// `MAX_ORB`. At the old bound those were 64 and 32, and `u64` happened to serve both --
/// which is exactly why the packing was written as `(ma as u64) | ((mb as u64) << n)` and
/// read as obviously fine. At the new bound the string is 64 bits and the determinant
/// needs 128, so the coincidence is gone and the two roles are separate types.
///
/// This packing is used only by the two CHECKING routes -- [`FciSpace::sigma_reference`]
/// and [`dense_hamiltonian_ladder`] -- which are quadratic in the determinant count and
/// therefore run on small spaces. The production route never forms it.
pub type Det = u128;

/// One spin string's occupation mask: bit `p` set means spatial orbital `p` is occupied.
///
/// Named rather than spelled `u64` at each use, because the width is a load-bearing
/// invariant shared by [`MAX_ORB`], [`excite`], [`Strings`] and every shift below. When
/// the two were independent literals, widening one and missing the other is precisely the
/// defect the W1 plant reproduces.
pub type Mask = u64;

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
    // f64 instantiation of the one generic body in `tier`.
    crate::tier::jacobi_eigh_t::<f64>(a_in, n)
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
    pub masks: Vec<Mask>,
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
pub fn excite(mask: Mask, p: usize, q: usize) -> Option<(f64, Mask)> {
    if (mask >> q) & 1 == 0 {
        return None;
    }
    let m1 = mask ^ (1 << q);
    let s1 = if (mask & (((1 as Mask) << q) - 1)).count_ones() & 1 == 1 {
        -1.0
    } else {
        1.0
    };
    if (m1 >> p) & 1 == 1 {
        return None;
    }
    let s2 = if (m1 & (((1 as Mask) << p) - 1)).count_ones() & 1 == 1 {
        -1.0
    } else {
        1.0
    };
    Some((s1 * s2, m1 | (1 << p)))
}

impl Strings {
    pub fn new(n_orb: usize, n_elec: usize) -> Strings {
        Strings::with_mask_width(n_orb, n_elec, MAX_ORB)
    }

    /// The string list a mask of `mask_width` bits can hold.
    ///
    /// # Why the width is a parameter and not a constant
    ///
    /// This is the W1 plant's entry point, and it is the honest way to write the plant:
    /// the quantity the widening changed is the number of bits a string mask has, so the
    /// plant sets that quantity rather than mutating a result. `Strings::new` passes
    /// [`MAX_ORB`] and is the only production caller.
    ///
    /// With `mask_width >= n_orb` this is exactly `new` -- every string of `n_elec`
    /// electrons in `n_orb` orbitals fits, and the parameter is inert. With
    /// `mask_width < n_orb` it produces what a narrower mask could actually have held:
    /// the strings whose occupied orbitals all lie below bit `mask_width`, and no others.
    /// That is not an approximation of the old defect, it IS the old defect -- masks that
    /// cannot address the high orbitals silently enumerate a smaller space and the CI
    /// solves a different problem without saying so.
    ///
    /// Passing a width below `n_orb` outside a plant is a bug; nothing in the production
    /// path does it.
    pub fn with_mask_width(n_orb: usize, n_elec: usize, mask_width: usize) -> Strings {
        assert!(n_orb <= MAX_ORB);
        let mut masks = Vec::new();
        if n_elec <= n_orb {
            if n_elec == 0 {
                masks.push(0);
            } else {
                // Gosper's hack, in the mask width. Both the seed and the stopping
                // limit are computed in u128 rather than in `Mask`, because at the top of
                // the range `1 << n_elec` and `1 << n_orb` are exactly the shifts that
                // overflow the very type being widened -- the u32 code had to special-case
                // `n_orb == 32` and `n_elec == 32` for the same reason, and a widening
                // that moved the type but kept the special case at 32 would silently
                // produce an EMPTY string list at 64.
                let mut v: Mask = ((1u128 << n_elec) - 1) as Mask;
                // The reachable range is capped by the mask width as well as by the
                // orbital count -- for the production width the cap is inert, and for the
                // plant it is the whole of the defect.
                let limit: u128 = 1u128 << n_orb.min(mask_width);
                while (v as u128) < limit {
                    masks.push(v);
                    let c = v & (!v).wrapping_add(1);
                    let r = v.wrapping_add(c);
                    if r == 0 || (r as u128) >= limit {
                        break;
                    }
                    v = (((r ^ v) >> 2) / c) | r;
                }
            }
        }
        let lookup = if n_orb <= 16 {
            let mut lk = vec![-1i32; 1usize << n_orb];
            for (i, &m) in masks.iter().enumerate() {
                lk[m as usize] = i as i32;
            }
            lk
        } else {
            Vec::new()
        };
        let get_idx = |m: Mask, lk: &[i32], ms: &[Mask]| -> Option<usize> {
            if !lk.is_empty() {
                let idx = lk[m as usize];
                if idx >= 0 {
                    Some(idx as usize)
                } else {
                    None
                }
            } else {
                ms.binary_search(&m).ok()
            }
        };
        // Excitations are generated only WITHIN the addressable orbitals. For the
        // production width this is every orbital and the bound is inert; for the plant it
        // is what makes the truncated space closed under the Hamiltonian instead of
        // leaking to strings the mask cannot name. The integral index below still uses the
        // full `n_orb`, because `ci.k` and `ci.g` are indexed by the real orbital number.
        let n_addressable = n_orb.min(mask_width);
        let mut singles = Vec::with_capacity(masks.len());
        for &m in masks.iter() {
            let mut list = Vec::new();
            for q in 0..n_addressable {
                for p in 0..n_addressable {
                    if let Some((sgn, m2)) = excite(m, p, q) {
                        let idx = get_idx(m2, &lookup, &masks)
                            .expect("excitation inside the addressable orbitals stays in the list");
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

    pub fn index_of(&self, mask: Mask) -> Option<usize> {
        if !self.lookup.is_empty() {
            if (mask as usize) < self.lookup.len() {
                let i = self.lookup[mask as usize];
                if i >= 0 {
                    Some(i as usize)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            self.masks.binary_search(&mask).ok()
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
        FciSpace::with_mask_width(n_orb, n_alpha, n_beta, MAX_ORB)
    }

    /// The determinant space a string mask of `mask_width` bits can address. See
    /// [`Strings::with_mask_width`]: `new` passes [`MAX_ORB`] and this exists for the W1
    /// plant, which needs to build the space a 32-bit mask would have built.
    pub fn with_mask_width(
        n_orb: usize,
        n_alpha: usize,
        n_beta: usize,
        mask_width: usize,
    ) -> FciSpace {
        let alpha = Strings::with_mask_width(n_orb, n_alpha, mask_width);
        let beta = Strings::with_mask_width(n_orb, n_beta, mask_width);
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
        let occ = |m: Mask| -> Vec<usize> { (0..n).filter(|&p| (m >> p) & 1 == 1).collect() };
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

    /// `sigma = H c`, by the Knowles-Handy string factorisation direct-CI method.
    #[inline]
    pub fn sigma_direct(&self, ci: &CiInts, c: &[f64], out: &mut [f64]) {
        sigma_direct(self, ci, c, out);
    }

    /// `sigma = H c`, by the string factorisation.
    ///
    /// Three blocks, because the Hamiltonian splits into terms that move only beta
    /// electrons, only alpha electrons, and one of each — and each block is a different
    /// contraction over the same single-excitation lists.
    #[inline]
    pub fn sigma(&self, ci: &CiInts, c: &[f64], out: &mut [f64]) {
        sigma_direct(self, ci, c, out);
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
        let combine = |ma: Mask, mb: Mask| -> Det { (ma as Det) | ((mb as Det) << n) };
        let dets: Vec<Det> = (0..self.n_det)
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

/// `sigma = H c`, evaluated directly via the Knowles-Handy string factorisation
/// without allocating the full CI Hamiltonian matrix.
///
/// The Hamiltonian splits into three blocks: beta-only excitations, alpha-only excitations,
/// and mixed alpha-beta excitations. Each block is contracted directly using single-excitation
/// lists over alpha and beta occupation strings.
pub fn sigma_direct(space: &FciSpace, ci: &CiInts, c: &[f64], sigma: &mut [f64]) {
    // ONE body, in `tier::sigma_direct_t`, generic over the scalar; this is its f64
    // instantiation and monomorphises to the pre-generic code (same loops, same ops).
    crate::tier::sigma_direct_t::<f64>(space, &ci.k, &ci.g, c, sigma);
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
pub fn slater_condon(di: Det, dj: Det, ci: &CiInts, n: usize) -> f64 {
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
            if (cur & (((1 as Det) << so) - 1)).count_ones() & 1 == 1 {
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

    let combine = |ma: Mask, mb: Mask| -> Det { (ma as Det) | ((mb as Det) << n) };
    let dets: Vec<Det> = (0..nd)
        .map(|d| combine(space.alpha.masks[d / nb], space.beta.masks[d % nb]))
        .collect();
    let mut index = std::collections::HashMap::new();
    for (i, &d) in dets.iter().enumerate() {
        index.insert(d, i);
    }

    // Ladder primitives. `true` creates.
    fn ladder(det: Det, is_create: bool, so: u32) -> Option<(f64, Det)> {
        let occupied = (det >> so) & 1 == 1;
        if is_create == occupied {
            return None;
        }
        let below = (det & (((1 as Det) << so) - 1)).count_ones();
        let sgn = if below & 1 == 1 { -1.0 } else { 1.0 };
        Some((sgn, det ^ ((1 as Det) << so)))
    }
    fn apply(det: Det, ops: &[(bool, u32)]) -> Option<(f64, Det)> {
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
    /// WHICH SOLVER produced this. Carried on the solution rather than inferred by a
    /// caller from `n_det`, because [`solve`] switches routes on a size threshold and a
    /// caller that re-derives the switch is a second copy of the rule.
    pub route: SolverRoute,
    /// HOW FAR BELOW `min_i H_ii` the returned energy sits, when the route computed the
    /// diagonal. `None` from a route that does not (the MPS path), so a caller must decide
    /// what to do about a missing check rather than reading a default as a pass.
    ///
    /// # The one thing a residual cannot tell you
    ///
    /// A residual is small for ANY eigenvector, so no residual threshold can detect a solve
    /// that converged cleanly onto the WRONG one. Measured on this crate by the
    /// `saturation3-mesh` lane: a deliberately wrong warm start landed 7.47 hartree above
    /// the ground state and reported a residual of 5.98e-11 against the correct solve's
    /// 5.24e-11 — indistinguishable — with an identical exit reason.
    ///
    /// This is the cheap bound that does detect it. For any normalised trial vector,
    /// `E_0 <= <psi|H|psi>`; a single determinant is such a vector and gives `H_ii`; so
    /// `E_0 <= min_i H_ii` rigorously, with no assumption about the solver. `diag` is
    /// already computed for the preconditioner, so it costs one pass over an array the
    /// solve is holding anyway.
    ///
    /// A NEGATIVE margin means the answer cannot be the ground state and the node must
    /// VOID. Measured across every triple type SATURATION-3 builds, correct solves sit
    /// 0.018 to 0.073 hartree BELOW the bound — nine orders above the residual scale, so
    /// this is not a threshold that can drift into firing (`examples/s3_variational_guard.rs`,
    /// zero false positives in five of five).
    ///
    /// WHAT IT DOES NOT CATCH, because a guard oversold is worse than none: an excited
    /// state lying BELOW `min_i H_ii` passes. The bound is NECESSARY, not sufficient — the
    /// `<S^2>` audit and dual-route agreement remain what catch the rest.
    pub variational_margin: Option<f64>,
    /// WHY the solve stopped. See [`SolveExit`]: `residual` says where it ended, this says
    /// whether it was finished, and reading the first as the second is what this field
    /// exists to prevent.
    pub exit: SolveExit,
    /// WHICH DEVICE CLASS produced this. RESOURCE_DESIGN **D0** — the class belongs to the
    /// artifact, not to the schedule.
    ///
    /// This is not provenance decoration. SATURATION-3 G2 measured the two arms on the real
    /// `(O,O,O)` problem: they agree to 3.033e-15 relative and **91.0% of the 207,025 entries
    /// differ BITWISE**. Both are correct answers. A Davidson driven by one takes a different
    /// path from a Davidson driven by the other, so two `Solution`s of different classes may
    /// never be mixed inside one bit-gated table however closely their energies agree —
    /// and a table that mixed them would fail every bit-identity gate in the engine for a
    /// reason that is not a defect.
    ///
    /// Stamped by the solve from the provider that supplied its sigma operators, and REFUSED
    /// if any operator disagreed with the provider about what it was. It is never inferred by
    /// a caller from a build flag: a flag says what was compiled in, not what ran.
    pub device: DeviceClass,
}

/// Which solver produced a [`Solution`], and therefore what the number is worth.
///
/// # Why this is on the solution and not in a comment
///
/// [`solve`] takes the MPS route as the leased overflow when the door refuses the determinant working set — to DMRG. That is a
/// defensible engineering choice and it was, until this type existed, INVISIBLE: the curve
/// came back through the same struct, into a `PairTable` stamped with a provenance string
/// reading "engine-computed STO-3G FCI (determinant, Knowles-Handy)", and nothing anywhere
/// could tell a caller that the numbers in front of it were variational within a bond
/// budget rather than exact in the model. SiO, one of MIXTURES-1's own D1 overlap species,
/// is 132,496 determinants and was travelling that path wearing the determinant label.
///
/// A DMRG energy is an upper bound converged to a declared tolerance, not an exact-in-model
/// value, and the difference is the entire content of gate D1. So the route travels with
/// the number.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SolverRoute {
    /// Determinant CI: Davidson over the full space in the minimal `S_z` sector. Exact in
    /// the declared model, up to the Davidson residual the solution reports.
    Determinant,
    /// MPS/DMRG: variational inside a bond-dimension budget. NOT exact in model; its error
    /// is bounded by the discarded weight the solution reports, and its admission to the
    /// sandbox is gate D1's business.
    Dmrg,
}

impl SolverRoute {
    /// Whether a curve from this route may be presented as exact in the declared model.
    pub fn is_exact_in_model(self) -> bool {
        matches!(self, SolverRoute::Determinant)
    }

    /// The route's own name, for a provenance line.
    pub fn label(self) -> &'static str {
        match self {
            SolverRoute::Determinant => "determinant, Knowles-Handy",
            SolverRoute::Dmrg => "DMRG",
        }
    }
}

/// WHY an iterative solve stopped.
///
/// # The distinction this exists to keep
///
/// `davidson_eigh` has three exits and reported one number: the residual. So a solve that
/// CONVERGED at 9e-12 and one that hit its ITERATION CAP at 9e-12 were indistinguishable in
/// every record downstream, and a campaign lost the ability to say why thirteen heavy atoms
/// stopped short of their 1e-11 target — the publication bar at 1e-10 was sorting
/// gave-up solves rather than separating converged from not.
///
/// A residual is a measurement of where a solve ENDED. It is not a statement about whether
/// the solve was finished, and the two were being read as one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SolveExit {
    /// The residual fell below the requested tolerance. The only exit that means "done".
    Converged,
    /// The iteration cap was reached with the residual still above tolerance. The answer is
    /// the best available, and it is NOT converged however small the residual looks.
    IterationCap,
    /// The Krylov subspace stopped growing: no new direction survived orthogonalisation
    /// against the existing basis. More iterations cannot help — this one is a property of
    /// the space and the start vector, not of the budget.
    Stagnated,
    /// A one-determinant space. Exact by construction with no iteration performed, so
    /// "converged" would overstate what happened.
    Trivial,
}

// There is deliberately no `Dmrg` variant. The MPS route's own convergence flag is mapped
// onto `Converged`/`IterationCap` in `solve_mps`, and `Solution::route` already says which
// solver produced the answer — a variant meaning "ask the other field" would be a second
// place for the same fact to be stored and a first place for it to disagree.

impl SolveExit {
    /// Whether the solve reached its tolerance. `false` for every exit that stopped early,
    /// however small the residual it stopped at.
    pub fn is_converged(self) -> bool {
        matches!(self, SolveExit::Converged | SolveExit::Trivial)
    }

    pub fn label(self) -> &'static str {
        match self {
            SolveExit::Converged => "converged",
            SolveExit::IterationCap => "iteration cap",
            SolveExit::Stagnated => "subspace stagnated",
            SolveExit::Trivial => "trivial (one determinant)",
        }
    }
}

/// The residual [`solve_determinant`] asks Davidson for.
///
/// # Why 1e-10 and not 1e-11, which is what it used to ask
///
/// It asked for 1e-11 and could never get it. `davidson_eigh_from` accepts a new subspace
/// direction only if its norm after orthogonalisation exceeds **1e-10** (see the
/// `if nw > 1e-10` guard in its expansion block), so **the delivered tolerance is bounded
/// below by that expansion floor**: any ask deeper than 1e-10 is unreachable by
/// construction on a system whose subspace has to keep growing. Every multi-determinant
/// curve this crate ships was stopping between 9.5e-11 and 10.0e-11 with
/// [`SolveExit::Stagnated`] — six systems in a band far too tight to be six independent
/// limits.
///
/// Raising the ask to the floor is truth-in-labeling. It changes no solver behaviour that
/// was reachable before, and it restores meaning to [`crate::pair::CONVERGED_RESIDUAL`]:
/// at a REACHABLE target a solve below the bar genuinely exits `Converged`, so the bar
/// separates converged from not instead of sorting gave-up solves in with finished ones.
/// Solves that stagnate ABOVE 1e-10 still stagnate and are still refused.
///
/// # This is the label, not the fix
///
/// Lowering the expansion floor is the real solver improvement and is a named successor
/// with its own re-banking campaign — B1's bit-identity reference, the banked records, the
/// referee pins all move when it lands. It is deliberately not done here and must never be
/// done as a side effect. Until it lands, an ask deeper than this constant is a claim the
/// solver cannot honour.
/// The f64 tier's declared arithmetic boundary for this solver: the norm below which a
/// Gram-Schmidt'd candidate direction is roundoff rather than information, and therefore
/// the floor under every deliverable residual. This is a TIER EDGE, not a tolerance
/// choice — three formerly independent 1e-10 literals (both orthogonalisation guards and
/// the requested tolerance) were this one boundary written three times, and the
/// publication bar sat exactly ON it, which made six of nine staked curves' verdicts a
/// coin flip against ambient perturbation (measured 2026-08-30: every non-trivial
/// all-electron curve stagnates at 96-100% of it).
///
/// A solve that exits `SolveExit::Stagnated` with its residual near this floor has not
/// failed — it has EXHAUSTED THIS TIER. Anything that genuinely needs a deeper residual
/// does not get it by editing this constant: it overflows to the next tier of arithmetic
/// (the high-precision referee route, the same boundary ELEMENTS-3 recorded as "route C,
/// availability owed") through a resource lease, per the holon lifecycle. Editing this
/// number moves no energy and every verdict; it is load-bearing for every lane's gates.
pub const DAVIDSON_EXPANSION_FLOOR: f64 = crate::scalar::F64_EXPANSION_FLOOR;

pub const DAVIDSON_REQUESTED_TOLERANCE: f64 = DAVIDSON_EXPANSION_FLOOR;

// The hard determinant cap (2,000,000) and the MPS routing threshold (50,000) used to
// live here. Both were prices measured in one regime and consumed as physics; both are
// gone. A space is admitted by its PRICE at the door — `budget::price_determinant` against
// the resource layer's real reservation — and routed to the MPS tier only as the
// overflow of a determinant refusal (RESOURCE_DESIGN D3b). See `budget.rs`.

/// Iteration cap for [`davidson`], as a settable global.
///
/// # Why this exists, and why it is not a tuning knob
///
/// `PairCache::get` refuses a curve whose solve gave up. That refusal needs a test that goes
/// in the FRONT DOOR — driving the production entry point and requiring it to come back
/// refused — because the failure it guards against is not "the verdict is computed wrongly"
/// but "nothing acts on the verdict", and only a plant through the real path can tell those
/// apart. A test that plants a bad `PairMeta` directly would be testing the unit under the
/// entry point, which is exactly the level at which this defect hid.
///
/// So a solve has to be made to genuinely fail. Capping the iterations does that honestly:
/// the eigenvector really is unconverged, the residual really is large, and everything
/// downstream sees the failure it would see in the wild. Faking the residual instead would
/// test the plumbing while leaving the physics path unexercised.
///
/// # This is NOT test-only, and saying so cost the campaign a silent regime change
///
/// The line that stood here read "Production never touches it. `tests/front_door.rs` is
/// its only caller." That was false: three production solve paths LOAD it on every solve,
/// and because the comment said otherwise nobody was watching it. Commit `451db31` — whose
/// subject is "integrate OOH trimer surface and dispatch into Sim dynamics" — moved it
/// from 1200 to 4000, and every artifact generated before and after that render commit was
/// produced under a different solver regime with nothing in either recording which. That is
/// the whole argument for [`crate::pair::PairMeta::solver_budget`] in one commit.
///
/// What it actually is: the PROCESS-WIDE DEFAULT budget, overridable for a test that needs
/// a solve to genuinely fail. Callers that care state their own budget instead — see
/// [`solve_determinant_from_with_budget`].
pub static DAVIDSON_MAX_ITER: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(DAVIDSON_DEFAULT_BUDGET);

/// The default iteration BUDGET a solve is given, and it is derived rather than chosen.
///
/// A budget is not a tolerance and not a tier edge. `SolveExit::IterationCap` means the
/// solve ran out of TIME on an arithmetic tier that can still deliver;
/// `SolveExit::Stagnated` near [`DAVIDSON_EXPANSION_FLOOR`] means the tier itself is
/// exhausted and the answer overflows to the next one. They have opposite remedies and
/// only the exit reason tells them apart.
///
/// THE RECEIPT: the O-O pair curve is the hardest solve this crate ships, and at its worst
/// knot (r = 4.2244 bohr, 2025 determinants) a ladder of budgets measured the residual
/// falling 1.08e-4 -> 9.53e-11 with the energy settling, converging at **3738 iterations**
/// (`examples/s3_oo_trace`). 5000 covers that with a third of it again in margin. It is not
/// a bound on every solve — nine knots of that curve do not converge at 20000 either — and
/// a budget that runs out is reported as `IterationCap`, never as an answer.
///
/// Raising this re-banks every artifact whose manifest declares a lower one. That is the
/// point of the manifest: the change is visible instead of silent.
pub const DAVIDSON_DEFAULT_BUDGET: usize = 5000;

/// The budget this process gives a solve that does not state its own.
pub fn davidson_budget() -> usize {
    DAVIDSON_MAX_ITER.load(std::sync::atomic::Ordering::Relaxed)
}

/// Lowest eigenpair by Davidson iterative diagonalization, matrix-free.
///
/// Uses `sigma_direct` as the primary matrix-vector multiplier for large CI spaces
/// (> 1,000 determinants), avoiding allocation of the full Hamiltonian matrix.
pub fn davidson_eigh(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    tol: f64,
    max_iter: usize,
) -> (f64, Vec<f64>, usize, f64) {
    let (e, v, iters, resid, _) = davidson_eigh_with_exit(space, ci, diag, tol, max_iter);
    (e, v, iters, resid)
}

/// [`davidson_eigh`], reporting WHY it stopped.
///
/// Additive: `davidson_eigh` calls this and drops the reason, so every existing caller is
/// unchanged. New callers should prefer this one — see [`SolveExit`] for why a residual
/// alone cannot answer the question.
pub fn davidson_eigh_with_exit(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    tol: f64,
    max_iter: usize,
) -> (f64, Vec<f64>, usize, f64, SolveExit) {
    davidson_eigh_from(space, ci, diag, tol, max_iter, None)
}

/// [`davidson_eigh_with_exit`], with the start vector supplied by the caller — the WARM
/// START.
///
/// # What a warm start may and may not change
///
/// A neighbouring geometry's converged CI vector is an excellent guess at this one's, and
/// starting there is worth a large fraction of the iterations on a smooth surface. That is
/// a statement about the PATH. It is not a statement about the ANSWER, and the distinction
/// is the whole of SATURATION-3's plant (iii): a warm start may change how the solve gets
/// there, never where it arrives. A table whose entries depended on which neighbour
/// happened to be solved first would not be a function of geometry at all, and nothing
/// downstream could tell.
///
/// This entry point therefore relaxes nothing. Same tolerance, same iteration cap, same
/// convergence test, same exit reasons, same returned residual. `start = None` reproduces
/// [`davidson_eigh_with_exit`]'s own start vector bit-for-bit, so every existing caller is
/// unaffected — that delegation is the only change to the cold path.
///
/// # The perturbation is kept, and it is not belt-and-braces
///
/// `H` commutes with `S^2`, so a Krylov space never leaves the spin sector of the vector
/// it started from. The cold start breaks that symmetry deliberately, because a solve that
/// stays in the wrong sector converges CLEANLY to the wrong answer — carbon came back 0.07
/// hartree above its ground state with a small residual and nothing in the solve to say so.
///
/// A warm start inherits its NEIGHBOUR's spin sector, which is usually the sector wanted
/// and is exactly why warm starting works at all. But "usually" is not a guarantee, and a
/// warm start that had silently walked a table into the wrong multiplet would be the carbon
/// failure again with a longer fuse and a whole surface behind it. So the same
/// deterministic perturbation is added ON TOP of the supplied vector rather than in place
/// of it: the guess is kept, the symmetry is still broken, and the `<S^2>` audit downstream
/// still has something to find.
///
/// # Panics
///
/// If `start` is `Some(v)` and `v.len() != space.n_det`. A start vector of the wrong length
/// is a caller that has confused two geometries' spaces; padding or truncating it silently
/// would turn that mistake into a wrong number instead of a stopped process.
pub fn davidson_eigh_from(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    tol: f64,
    max_iter: usize,
    start_vector: Option<&[f64]>,
) -> (f64, Vec<f64>, usize, f64, SolveExit) {
    // ONE driver body, in `tier::davidson_eigh_from_t`, generic over the scalar — the
    // start-vector policy, thick restart, two-candidate expansion and degeneracy-safe
    // preconditioner all live there now, once. This is the f64 instantiation. (The old
    // body's `n_det > 1000` branch chose between `sigma_direct` and `space.sigma`, which
    // are the same function — the generic driver makes the one call both arms made.)
    crate::tier::davidson_eigh_from_t::<f64>(space, &ci.k, &ci.g, diag, tol, max_iter, start_vector)
}

/// Lowest eigenpair by Davidson, matrix-free.
///
/// Aliases [`davidson_eigh`].
#[inline]
pub fn davidson(
    space: &FciSpace,
    ci: &CiInts,
    diag: &[f64],
    tol: f64,
    max_iter: usize,
) -> (f64, Vec<f64>, usize, f64) {
    davidson_eigh(space, ci, diag, tol, max_iter)
}

// There is deliberately no `davidson_with_exit` alias mirroring `davidson`. Nothing calls
// it, and the dead-API audit was right to say so: `davidson_eigh_with_exit` is the API, and
// a parallel name kept for symmetry is one more public item nobody consumes.

/// Solve `(H - E) w = b` on the orthogonal complement of `v`, by projected conjugate
/// gradients with a Jacobi preconditioner.
///
/// The operator is positive semi-definite there because `E` is the LOWEST eigenvalue, so
/// CG is the right method rather than a convenient one; the projection removes the one
/// direction where it is singular.
#[allow(clippy::too_many_arguments)]
fn cg_response(
    op: &mut dyn crate::sigma_op::SigmaOp<f64>,
    diag: &[f64],
    e: f64,
    v: &[f64],
    b: &[f64],
    tol: f64,
    max_iter: usize,
) -> (Vec<f64>, usize, f64) {
    let nd = op.n_det();
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
        op.apply(&p, &mut hp);
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

/// MPS/DMRG solver route for ground state and energy derivatives.
/// Maps molecular orbital integrals (h_pq, g_pqrs) to a 1D Matrix Product Operator
/// and computes ground state energy and Hellmann-Feynman derivatives with DMRG.
pub fn solve_mps(space: &FciSpace, mo: &MoIntegrals, max_bond_dim: usize) -> Solution {
    solve_mps_with(space, mo, max_bond_dim, MPS_DEFAULT_SWEEPS, MPS_DEFAULT_TOL)
}

/// Sweep budget [`solve_mps`] uses when the caller does not say.
pub const MPS_DEFAULT_SWEEPS: usize = 25;
/// Per-sweep energy tolerance [`solve_mps`] uses when the caller does not say.
pub const MPS_DEFAULT_TOL: f64 = 1e-9;

/// [`solve_mps`] with the sweep budget and tolerance under the caller's control.
///
/// Gate D1 needs these settable and it needs them REPORTED: a harness that prints
/// "sweeps 40, tol 1e-11" in its header while the solver runs 25 and 1e-9 is describing a
/// run that did not happen. The constants above are what the ordinary path uses, named so
/// that the default is a stated value rather than two literals inside a call.
pub fn solve_mps_with(
    space: &FciSpace,
    mo: &MoIntegrals,
    max_bond_dim: usize,
    max_sweeps: usize,
    tol: f64,
) -> Solution {
    let n_orb = mo.n;
    let n_alpha = space.alpha.n_elec;
    let n_beta = space.beta.n_elec;

    let h0: Vec<f64> = mo.h.iter().map(|d| d.v).collect();
    let g0: Vec<f64> = mo.g.iter().map(|d| d.v).collect();
    let h1: Vec<f64> = mo.h.iter().map(|d| d.d).collect();
    let g1: Vec<f64> = mo.g.iter().map(|d| d.d).collect();
    let h2: Vec<f64> = mo.h.iter().map(|d| d.e).collect();
    let g2: Vec<f64> = mo.g.iter().map(|d| d.e).collect();

    let res = q8_mps::dmrg::solve_electronic_ground_state(
        n_orb,
        n_alpha,
        n_beta,
        &h0,
        &g0,
        max_bond_dim,
        max_sweeps,
        tol,
    )
    .expect("MPS DMRG ground state solver failed");

    let last_de = if res.energy_history.len() >= 2 {
        (res.energy_history[res.energy_history.len() - 1] - res.energy_history[res.energy_history.len() - 2]).abs()
    } else {
        0.0
    };

    assert!(
        res.converged || last_de < 1e-6,
        "MPS DMRG ground state solver did not reach energy convergence: dE = {:.3e}",
        last_de
    );

    let mpo1 = q8_mps::mpo::Mpo::from_electronic_integrals(n_orb, &h1, &g1);
    let e1 = mpo1.expectation(&res.tensors);

    let mpo2 = q8_mps::mpo::Mpo::from_electronic_integrals(n_orb, &h2, &g2);
    let e2 = mpo2.expectation(&res.tensors);

    let max_dw = res.discarded_weight.iter().copied().fold(0.0f64, f64::max);

    Solution {
        e: D2::new(res.energy, e1, e2),
        vector: vec![],
        // NONE, not a default that reads as a pass: this route never forms the determinant
        // diagonal, so the variational bound is genuinely unavailable here rather than
        // satisfied. A caller that needs the check on an MPS result has to say so.
        variational_margin: None,
        davidson_iters: res.sweeps_used,
        cg_iters: 0,
        residual: max_dw.max(res.worst_lanczos_residual),
        cg_residual: 0.0,
        route: SolverRoute::Dmrg,
        // The DMRG route has no Krylov subspace to stagnate and no Davidson cap; its own
        // convergence flag is the only thing it can report, and it is mapped rather than
        // invented. `SolveExit::Dmrg` would lose the flag, so the flag wins and the route
        // field already says which solver produced this.
        exit: if res.converged {
            SolveExit::Converged
        } else {
            SolveExit::IterationCap
        },
        // The MPS route is host arithmetic throughout — `q8-mps` has no device arm — so this is
        // a fact about what ran, not a default. It is written here rather than left to a
        // `Default` impl on purpose: a default class is how an artifact acquires a stamp nobody
        // checked, and D0's whole content is that the stamp must be a measurement of the run.
        device: DeviceClass::Cpu,
    }
}

/// Solve for the ground state and both derivatives of its energy, choosing the route by
/// the size of the determinant space.
///
/// Spaces whose determinant working set the door refuses go to the MPS/DMRG solver. The returned
/// [`Solution::route`] says which one ran, and every consumer that presents the number to
/// anybody is required to read it — see [`SolverRoute`] for what went wrong while nothing
/// did.
///
/// A caller that needs the exact-in-model answer whatever it costs — gate D1's overlap
/// comparison is exactly that caller — must use [`solve_determinant`] instead, which has no
/// threshold and no fallback.
pub fn solve(space: &FciSpace, mo: &MoIntegrals) -> Solution {
    match try_solve(space, mo) {
        Ok(s) => s,
        Err(r) => panic!("{r}"),
    }
}

/// [`solve`] with the admission door's refusal returned rather than raised: the
/// determinant route if its working set is admitted; else the MPS route as the leased
/// overflow, if ITS price is admitted; else the refusal naming both prices.
pub fn try_solve(space: &FciSpace, mo: &MoIntegrals) -> Result<Solution, crate::budget::Refused> {
    let det = crate::budget::price_determinant(space.n_det);
    match crate::budget::admit(&det) {
        Ok(_) => Ok(solve_determinant(space, mo)),
        Err(det_refusal) => {
            let mpo = crate::budget::price_mpo(space.n_orb);
            match crate::budget::admit(&mpo) {
                Ok(_) => Ok(solve_mps(space, mo, 64)),
                Err(_) => Err(det_refusal),
            }
        }
    }
}

/// The determinant route, unconditionally: Davidson over the whole space, whatever its
/// size, with no size threshold and no DMRG fallback.
///
/// This is what "exact in model" means in this crate, and it exists as its own entry point
/// because gate D1 compares DMRG against exact FCI on species that are large enough for
/// [`solve`] to have sent BOTH sides of that comparison to DMRG. A validation whose
/// reference silently becomes the thing under test measures nothing, and the shape is not
/// hypothetical: SiO is 132,496 determinants, past the threshold, and is one of D1's two
/// staked overlap species.
pub fn solve_determinant(space: &FciSpace, mo: &MoIntegrals) -> Solution {
    solve_determinant_from(space, mo, None)
}

/// [`solve_determinant`], starting the Davidson from a caller-supplied vector.
///
/// This is the table generator's entry point: SATURATION-3 walks a grid whose neighbouring
/// nodes differ by one small step in one coordinate, and the converged CI vector at the
/// previous node is a very good guess at this one's. `start = None` is exactly
/// [`solve_determinant`], so the cold path is unchanged.
///
/// # What this does NOT change
///
/// Only the Davidson's first basis vector. The tolerance, the iteration cap, the exit
/// reasons, the response solve for the second derivative, and the hard determinant cap all
/// behave identically — a warm start is an optimisation of the path and must never be an
/// argument for accepting a looser answer. Whether the answer it reaches is the same answer
/// is a MEASUREMENT, not an assumption, and it is SATURATION-3's plant (iii).
pub fn solve_determinant_from(
    space: &FciSpace,
    mo: &MoIntegrals,
    start_vector: Option<&[f64]>,
) -> Solution {
    solve_determinant_from_with_budget(space, mo, start_vector, davidson_budget())
}

/// [`solve_determinant_from`], with the iteration BUDGET stated by the caller.
///
/// The host-provider convenience over [`solve_determinant_with_budget`], which is where the
/// reasoning lives. A caller that states a budget here owes its artifact's manifest the
/// same number.
pub fn solve_determinant_from_with_budget(
    space: &FciSpace,
    mo: &MoIntegrals,
    start_vector: Option<&[f64]>,
    budget: usize,
) -> Solution {
    // The host provider, which is what this entry point has always used. `solve_determinant_with`
    // is the same body with the operator source made explicit.
    solve_determinant_with_budget(space, mo, start_vector, &CpuProvider, budget)
        .expect("the host sigma provider cannot fail to build an operator")
}

/// [`solve_determinant_from`], with the SIGMA PROVIDER — and therefore the device class of the
/// artifact — supplied by the caller (RESOURCE_DESIGN **D0**).
///
/// # Every application of the Hamiltonian comes from the ONE provider
///
/// A determinant solve applies `H` four times over: the Davidson eigensolve, the first
/// derivative `H'`, the second `H''`, and the conjugate-gradient response (`H` again). Running
/// the eigensolve on one class and the derivatives on another would return a `Solution` whose
/// energy and whose curvature came from different arithmetic — a MIXED artifact, which is
/// precisely what D0 forbids. So all four operators come from one provider, and the class every
/// operator reports is CHECKED against the provider's rather than assumed: a provider that
/// handed back an operator of another class is a bug that would otherwise mis-stamp a table.
///
/// # Fallible, and the failure is LOUD
///
/// A device provider can fail to build an operator — no device, no VRAM, a lease refused. D4
/// says that produces a refusal or a *stated* Degrade, never a quiet run on the host: a silent
/// fallback here would return a `Solution` stamped `Gpu` that a CPU computed, which is worse
/// than either device. So this returns `Err` and the caller decides, in the open.
pub fn solve_determinant_with(
    space: &FciSpace,
    mo: &MoIntegrals,
    start_vector: Option<&[f64]>,
    provider: &dyn SigmaProvider,
) -> Result<Solution, String> {
    solve_determinant_with_budget(space, mo, start_vector, provider, davidson_budget())
}

/// [`solve_determinant_with`], with the ITERATION BUDGET stated by the caller.
///
/// # Why the budget is a parameter and not a constant
///
/// A budget silently gating physics is the same disease as an unreachable tolerance: both
/// decide an answer while looking like housekeeping, and both did. The tolerance became a
/// caller-visible quantity when the ask was made reachable; this is the same move for the
/// other axis, and it is the entry point anything that cares about its own budget should
/// use. What a caller passes here is what its artifact's manifest must declare — see
/// [`crate::pair::PairMeta::solver_budget`]. The budget is part of the artifact's identity,
/// under the same law as device class and region shape: two tables built under different
/// budgets are two artifacts, and a manifest that does not say which is a manifest that
/// cannot be re-derived from.
///
/// Running out of it is reported as [`SolveExit::IterationCap`] and never as an answer.
pub fn solve_determinant_with_budget(
    space: &FciSpace,
    mo: &MoIntegrals,
    start_vector: Option<&[f64]>,
    provider: &dyn SigmaProvider,
    budget: usize,
) -> Result<Solution, String> {
    // THE ADMISSION, not a cap. The Davidson vectors are what the space costs — `n_det`
    // doubles per vector, up to 48 basis and 48 sigma vectors — so the working set is
    // priced from those allocations and the resource layer's probe RESERVES it before a
    // single sigma is built. A machine that admits the reservation runs the space; one that
    // does not gets a refusal naming the bytes. No number here says what "too big" is.
    if let Err(r) = crate::budget::admit(&crate::budget::price_determinant(space.n_det)) {
        panic!("solve_determinant: {r}");
    }
    let ci0 = ci_ints(mo, Order::Value);
    let ci1 = ci_ints(mo, Order::First);
    let ci2 = ci_ints(mo, Order::Second);
    let diag = space.diagonal(&ci0);
    let device = provider.device();

    // D0's enforcement point. An operator whose class disagrees with the provider that built it
    // would put one class's arithmetic under another class's stamp, and no downstream gate could
    // tell — so it is refused here rather than discovered in a table.
    let check = |op: &dyn crate::sigma_op::SigmaOp<f64>, which: &str| -> Result<(), String> {
        if op.device() != device {
            return Err(format!(
                "the {which} operator reports device class {:?} while its provider reports \
                 {device:?}. A solve whose stages ran on different classes is a MIXED artifact, \
                 which D0 forbids: two devices can agree to 3e-15 and differ on 91% of entries \
                 bitwise, so the trailing bits of this energy would depend on which stage ran \
                 where.",
                op.device()
            ));
        }
        if op.n_det() != space.n_det {
            return Err(format!(
                "the {which} operator is built for {} determinants and the space has {}",
                op.n_det(),
                space.n_det
            ));
        }
        Ok(())
    };

    let mut op0 = provider.op_for(space, &ci0)?;
    check(op0.as_ref(), "energy")?;
    let (e, v, iters, residual, exit) = crate::tier::davidson_eigh_from_op(
        op0.as_mut(),
        &diag,
        DAVIDSON_REQUESTED_TOLERANCE,
        budget,
        start_vector,
    );

    // E' = <v|H'|v>: exact for a variational eigenvector, with no response needed,
    // because the eigenvector's own derivative enters at second order.
    let nd = space.n_det;
    let mut h1v = vec![0.0f64; nd];
    {
        let mut op1 = provider.op_for(space, &ci1)?;
        check(op1.as_ref(), "first-derivative")?;
        op1.apply(&v, &mut h1v);
    }
    let e1 = dot(&v, &h1v);

    // E'' = <v|H''|v> + 2 <v^(1)|H'|v>, with (H - E) v^(1) = -(H' - E') v.
    let mut h2v = vec![0.0f64; nd];
    {
        let mut op2 = provider.op_for(space, &ci2)?;
        check(op2.as_ref(), "second-derivative")?;
        op2.apply(&v, &mut h2v);
    }
    let e2_direct = dot(&v, &h2v);
    let mut rhs = h1v.clone();
    axpy(-e1, &v, &mut rhs);
    for x in rhs.iter_mut() {
        *x = -*x;
    }
    // The response solve re-uses the ENERGY operator: same Hamiltonian, same class, and on a
    // device arm the tables it uploaded are still resident.
    let (w, cg_iters, cg_residual) =
        cg_response(op0.as_mut(), &diag, e, &v, &rhs, 1e-10, 2000);
    let e2 = e2_direct + 2.0 * dot(&w, &h1v);

    // The variational bound, from the diagonal the preconditioner already built. See
    // `Solution::variational_margin`: this is the only check in the record that can catch a
    // solve which converged cleanly onto the wrong eigenvector, because a residual is small
    // for any eigenvector at all.
    let min_diag = diag.iter().copied().fold(f64::INFINITY, f64::min);
    Ok(Solution {
        e: D2::new(e, e1, e2),
        vector: v,
        variational_margin: Some(min_diag - e),
        davidson_iters: iters,
        cg_iters,
        residual,
        cg_residual,
        route: SolverRoute::Determinant,
        exit,
        device,
    })
}


// --- small vector helpers, written here because the crate has no linear algebra dep ---

fn dot(a: &[f64], b: &[f64]) -> f64 {
    crate::tier::dot_t::<f64>(a, b)
}

fn axpy(a: f64, x: &[f64], y: &mut [f64]) {
    crate::tier::axpy_t::<f64>(a, x, y)
}

fn norm(a: &[f64]) -> f64 {
    crate::tier::norm_t::<f64>(a)
}

// ------------------------------------------------------------------ the spin sector
//
// A CHECK on the multiplicity of the converged state, not an assumption about it.

/// `<S^2>` of a normalised CI vector.
///
/// # Why this exists, and why the mitigation it backs up was not enough
///
/// `H` commutes with `S^2`, so a Krylov or Davidson space never leaves the spin sector of
/// the vector it started from. `davidson` perturbs its start vector precisely to break
/// that — but a perturbation is a MITIGATION, and nothing downstream could tell whether it
/// had worked. A small residual, a converged Ritz value and two independent sigma routes
/// agreeing are all statements about the subspace the solve happened to be in. That is how
/// carbon came back cleanly converged 0.07 hartree above its ground state: the solve was
/// correct, in the wrong sector.
///
/// So the sector is now measured. From `S^2 = S_- S_+ + S_z(S_z + 1)` and `S_- = S_+^†`,
///
/// ```text
/// <S^2> = || S_+ psi ||^2 + S_z (S_z + 1),      S_+ = sum_p a+_{p,alpha} a_{p,beta}
/// ```
///
/// which is one pass over the determinants — negligible even at 14 400. The result is
/// `S(S+1)` for the state's actual total spin, so it also hands back the multiplicity as a
/// DERIVED quantity rather than an assumed one.
///
/// Credit where it is due: the sibling `elements-referee` lane built this check first,
/// from this crate's carbon failure, and its `<S^2>` agrees with the periodic table's
/// multiplicities for all ten atoms.
///
/// # The one sign that does not matter
///
/// `S_+` raises `n_alpha` by one for every term, so the factor `(-1)^{n_alpha}` that a
/// beta annihilation picks up crossing the alpha block is common to all of them. It is
/// dropped: the result is a NORM, and a global sign cannot survive squaring.
pub fn s_squared(space: &FciSpace, c: &[f64]) -> f64 {
    let n = space.n_orb;
    let (na, nb) = (space.alpha.n_elec, space.beta.n_elec);
    let sz = 0.5 * (na as f64 - nb as f64);
    let floor = sz * (sz + 1.0);
    // `S_+` annihilates the state when there is no beta electron to raise, or no alpha
    // hole to raise it into. Both are real cases here (neon, and any fully alpha-filled
    // sector), and both would build a degenerate target space if not caught.
    if nb == 0 || na + 1 > n {
        return floor;
    }

    let target = FciSpace::new(n, na + 1, nb - 1);
    let mut raised = vec![0.0f64; target.n_det];
    let nb_src = space.beta.len();
    let nb_dst = target.beta.len();
    for (ia, &ma) in space.alpha.masks.iter().enumerate() {
        for (ib, &mb) in space.beta.masks.iter().enumerate() {
            let amp = c[ia * nb_src + ib];
            if amp == 0.0 {
                continue;
            }
            for p in 0..n {
                // The orbital must hold a beta electron and no alpha one.
                if (mb >> p) & 1 == 0 || (ma >> p) & 1 == 1 {
                    continue;
                }
                let sgn_b = if (mb & (((1 as Mask) << p) - 1)).count_ones() & 1 == 1 {
                    -1.0
                } else {
                    1.0
                };
                let sgn_a = if (ma & (((1 as Mask) << p) - 1)).count_ones() & 1 == 1 {
                    -1.0
                } else {
                    1.0
                };
                let (Some(ja), Some(jb)) = (
                    target.alpha.index_of(ma | (1 << p)),
                    target.beta.index_of(mb ^ (1 << p)),
                ) else {
                    continue;
                };
                raised[ja * nb_dst + jb] += sgn_a * sgn_b * amp;
            }
        }
    }
    raised.iter().map(|x| x * x).sum::<f64>() + floor
}

/// Total spin `S` read back from `<S^2> = S(S+1)`, and the multiplicity `2S+1` — refused
/// unless the value is a spin eigenvalue AND a spin this many electrons can actually have.
///
/// # Three conditions, and the third is the one that is easy to leave out
///
/// 1. `2S` must be (near) an integer, and 2. `S(S+1)` must reproduce `<S^2>`. Together
///    those say the reading is a spin eigenvalue rather than a mixture that happens to land
///    somewhere plausible.
/// 3. `2S` must have the SAME PARITY as the electron count. An even number of electrons can
///    only make integer `S`, an odd number only half-integer. Nothing about conditions 1 and
///    2 knows this, so without it a reading of exactly `0.75` reads back as a clean doublet
///    — for an eighteen-electron molecule, which is impossible.
///
/// The sibling `elements-referee` lane hit exactly that: past 8.9 bohr every vector of F2's
/// degenerate ground level agreed on a doublet, its "all vectors agree" test passed them,
/// and all of them were arbitrary mixtures. AGREEMENT IS A CONSISTENCY PROPERTY AND
/// VALIDITY IS A DIFFERENT QUESTION — a set of readings can be unanimous and jointly
/// meaningless. This crate's reader already refused their particular values by condition 2
/// (F2's far tail here reads `<S^2> = 1.000000`, which is not `S(S+1)` for any half-integer
/// `S`), so the defect did not bite; condition 3 is what makes it unreachable rather than
/// unlucky.
pub fn multiplicity(s_sq: f64, n_electrons: usize, tol: f64) -> Option<(f64, usize)> {
    // S(S+1) = s_sq  =>  S = (-1 + sqrt(1 + 4 s_sq)) / 2.
    let s = 0.5 * ((1.0 + 4.0 * s_sq).max(0.0).sqrt() - 1.0);
    let twice = (2.0 * s).round();
    if twice < 0.0 || (2.0 * s - twice).abs() > tol {
        return None;
    }
    let s = 0.5 * twice;
    if (s * (s + 1.0) - s_sq).abs() > tol {
        return None;
    }
    // The parity condition. `2S` and the electron count must agree mod 2.
    if (twice as usize) % 2 != n_electrons % 2 {
        return None;
    }
    Some((s, (twice as usize) + 1))
}
