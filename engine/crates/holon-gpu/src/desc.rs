//! The branch descriptor: one `holon::prune::Affine` in the shape the device
//! wants, plus the CPU reference that must agree with the device exactly.
//!
//! **Why a descriptor at all.** `Affine`'s `R`, `h`, `d`, `J` are private, and
//! adding accessors to a file another lane is actively editing is not a change
//! this crate gets to make. `Affine::canon_key()` is public, documented, and
//! serializes exactly those four (plus `n`, `k`, and the zero flag), so
//! [`AffineDesc::from_branch`] DECODES that. That is a dependence on a byte
//! format, and it is treated as one: the decoder length-checks every field, and
//! [`AffineDesc::agrees_with`] then drives the decoded descriptor against the
//! `Affine` it came from through `Affine::amplitude` itself, on a determining
//! set of basis states. A wrong decode cannot survive that check quietly, and
//! the conformance test runs it on every branch of a real circuit.
//!
//! `canon_key` carries a `debug_assert!` that the state is canonical. Every
//! branch out of `run_pruned`/`run_naive` is (`merge_block` canonicalizes before
//! the `disable_merge` fork), so that holds for the supported input.

use holon::ledger::Cyc;
use holon::prune::Affine;

use crate::ring;


/// Rows: one `u64` bitmask per qubit. 64 is the cap, and it is the word.
pub const N_CAP: usize = 64;
/// Columns: bit 63 of each augmented row carries the right-hand side, so the
/// affine dimension gets bits 0..62.
pub const K_CAP: usize = 63;

/// The rotation code a branch that contributes exactly zero reports.
pub const R_ZERO: u8 = 8;
/// The code for "R had dependent columns" — the `Affine` invariant broken.
pub const R_RANK_BAD: u8 = 9;

#[derive(Clone, Debug, PartialEq)]
pub enum DescError {
    TooManyQubits(usize),
    TooManyColumns(usize),
    /// `canon_key`'s byte layout did not match what this decoder expects. The
    /// message names the two lengths so a format change is diagnosable rather
    /// than a mystery.
    KeyLayout { got: usize, want: usize },
    QubitCountMismatch { key: usize, state: usize },
}

impl std::fmt::Display for DescError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DescError::TooManyQubits(n) => {
                write!(f, "n = {n} exceeds the GPU path's cap of {N_CAP} qubits")
            }
            DescError::TooManyColumns(k) => {
                write!(f, "k = {k} exceeds the GPU path's cap of {K_CAP} columns")
            }
            DescError::KeyLayout { got, want } => write!(
                f,
                "Affine::canon_key returned {got} bytes, this decoder expects {want}: \
                 the key's byte layout has changed and the decoder must be updated"
            ),
            DescError::QubitCountMismatch { key, state } => {
                write!(f, "canon_key says n = {key}, the Affine says n = {state}")
            }
        }
    }
}

impl std::error::Error for DescError {}

/// One branch, packed. Public fields: this is a data object, not an abstraction.
#[derive(Clone, Debug, PartialEq)]
pub struct AffineDesc {
    pub n: usize,
    pub k: usize,
    /// The `Affine`'s zero flag: the state is identically zero, whatever `y` is.
    pub zero: bool,
    /// `n` entries; bit `a` of entry `row` is `R[row][a]`.
    pub r_rows: Vec<u64>,
    /// `h` as a bitmask over rows.
    pub h: u64,
    /// `d[a] mod 4`, two bits per column: column `a` at `d[a >> 5] >> ((a & 31) * 2)`.
    pub d: [u64; 2],
    /// `k` entries; bit `b` of entry `a` is `J[a][b]`, upper triangle only
    /// (`b > a`) — the phase sum only ever reads `b > a`.
    pub j_rows: Vec<u64>,
    /// `weight * gamma`, the branch's whole ring content. Everything the
    /// per-`y` evaluation adds on top of this is a rotation by `omega^r`.
    pub base: Cyc,
}

// `rotation_code` and `point` walk `aug`, `r_rows` and `j_rows` by INDEX where an
// iterator would read more idiomatically. That is deliberate, and follows
// `prune.rs`'s own note about its port of `magic.rs`: these bodies are a
// line-for-line twin of `kernels/fold.cu`'s `branch_code`, and a diff between the
// two should show only the language. An iterator rewrite would make the device
// and host versions stop looking alike, which is the one property that makes a
// disagreement between them easy to localize.
#[allow(clippy::needless_range_loop)]
impl AffineDesc {
    /// Decode a branch. `weight` is `Branch::weight`; `state` supplies `gamma`
    /// and, through `canon_key`, the rest.
    pub fn from_branch(weight: Cyc, state: &Affine) -> Result<Self, DescError> {
        let key = state.canon_key();
        if key.len() < 9 {
            return Err(DescError::KeyLayout { got: key.len(), want: 9 });
        }
        let zero = key[0] != 0;
        let n = u32::from_le_bytes(key[1..5].try_into().unwrap()) as usize;
        let k = u32::from_le_bytes(key[5..9].try_into().unwrap()) as usize;
        if n != state.n_qubits() {
            return Err(DescError::QubitCountMismatch { key: n, state: state.n_qubits() });
        }
        if n > N_CAP {
            return Err(DescError::TooManyQubits(n));
        }
        if k > K_CAP {
            return Err(DescError::TooManyColumns(k));
        }

        // canon_key writes, in order: n*k bits of R (row-major), n bits of h,
        // k*(k-1)/2 bits of J's strict upper triangle, padded to a byte, then k
        // raw bytes of d. Anything else and the length check below fires.
        let n_bits = n * k + n + k * (k - 1) / 2;
        let want = 9 + n_bits.div_ceil(8) + k;
        if key.len() != want {
            return Err(DescError::KeyLayout { got: key.len(), want });
        }
        let bits = &key[9..];
        let bit = |i: usize| -> bool { bits[i >> 3] >> (i & 7) & 1 == 1 };

        let mut cursor = 0usize;
        let mut r_rows = vec![0u64; n];
        for r_row in r_rows.iter_mut() {
            for a in 0..k {
                if bit(cursor) {
                    *r_row |= 1u64 << a;
                }
                cursor += 1;
            }
        }
        let mut h = 0u64;
        for row in 0..n {
            if bit(cursor) {
                h |= 1u64 << row;
            }
            cursor += 1;
        }
        let mut j_rows = vec![0u64; k];
        for a in 0..k {
            for b in a + 1..k {
                if bit(cursor) {
                    j_rows[a] |= 1u64 << b;
                }
                cursor += 1;
            }
        }
        debug_assert_eq!(cursor, n_bits);

        let d_bytes = &key[9 + n_bits.div_ceil(8)..];
        let mut d = [0u64; 2];
        for (a, &byte) in d_bytes.iter().enumerate() {
            d[a >> 5] |= ((byte & 3) as u64) << ((a & 31) * 2);
        }

        Ok(AffineDesc {
            n,
            k,
            zero,
            r_rows,
            h,
            d,
            j_rows,
            base: ring::normalize(weight.mul(state.gamma())),
        })
    }

    /// The rotation code for basis state `y` (packed as a bitmask over qubits):
    /// `0..8` for `omega^r`, [`R_ZERO`] when the branch contributes exactly
    /// zero, [`R_RANK_BAD`] when `R` turned out to have dependent columns.
    ///
    /// This is the CPU twin of the device's `branch_code` — the same
    /// Gauss-Jordan on the same packed augmented rows, so a disagreement is a
    /// real disagreement and not a difference of algorithm. The CPU `Affine`
    /// eliminates on `Vec<Vec<bool>>` instead; both compute the unique solution
    /// of a full-column-rank system, so they agree by uniqueness, and
    /// [`Self::agrees_with`] checks that they do.
    pub fn rotation_code(&self, y: u64) -> u8 {
        if self.zero {
            return R_ZERO;
        }
        let n = self.n;
        let k = self.k;
        let rhs = y ^ self.h;
        let mut aug = [0u64; N_CAP];
        for row in 0..n {
            aug[row] = self.r_rows[row] | ((rhs >> row & 1) << 63);
        }
        let mut rr = 0usize;
        for c in 0..k {
            let Some(p) = (rr..n).find(|&p| aug[p] >> c & 1 == 1) else {
                return R_RANK_BAD;
            };
            aug.swap(rr, p);
            let piv = aug[rr];
            for p2 in 0..n {
                if p2 != rr && aug[p2] >> c & 1 == 1 {
                    aug[p2] ^= piv;
                }
            }
            rr += 1;
        }
        for row in rr..n {
            if aug[row] >> 63 != 0 {
                return R_ZERO;
            }
        }
        let mut u = 0u64;
        for c in 0..k {
            u |= (aug[c] >> 63 & 1) << c;
        }
        let mut ip = 0u32;
        let mut sgn = 0u32;
        let mut uu = u;
        while uu != 0 {
            let a = uu.trailing_zeros() as usize;
            uu &= uu - 1;
            ip += (self.d[a >> 5] >> ((a & 31) * 2) & 3) as u32;
            let above = if a == 63 { 0 } else { !0u64 << (a + 1) };
            sgn ^= (self.j_rows[a] & u & above).count_ones() & 1;
        }
        ((2 * (ip & 3) + 4 * sgn) & 7) as u8
    }

    /// The basis state at parameter `u`: `y = R u + h`.
    ///
    /// Worth stating plainly because it is a trap this crate walked into: `u =
    /// 0` gives `y = h`, and at `u = 0` NO `d[a]` and NO `J[a][b]` is ever read.
    /// A probe state built as "some branch's `h`" therefore exercises the F2
    /// solve and skips the entire phase polynomial — silently, with everything
    /// still agreeing. Both the test suite's plants and the benchmark's probe
    /// states go through here with a nonzero `u` for that reason.
    pub fn point(&self, u: u64) -> u64 {
        let mut y = self.h;
        for a in 0..self.k {
            if u >> a & 1 == 1 {
                for row in 0..self.n {
                    y ^= (self.r_rows[row] >> a & 1) << row;
                }
            }
        }
        y
    }

    /// `coeff_b * <y|phi_b>`, exact — the descriptor's whole contribution.
    pub fn amplitude(&self, y: u64) -> Cyc {
        match self.rotation_code(y) {
            R_ZERO => Cyc::ZERO,
            R_RANK_BAD => panic!("AffineDesc: R has dependent columns (rank < k)"),
            r => ring::rot(self.base, r),
        }
    }

    /// The falsifying check on the decode AND on this module's amplitude: does
    /// `weight * Affine::amplitude(y)`, computed by `holon` itself, equal
    /// `self.amplitude(y)` as a STRUCT, on a determining set of `y`?
    ///
    /// The set is the coset point `h`, every `h ^ R e_a`, every `h ^ R (e_a ^
    /// e_b)` up to `budget`, and every single-qubit flip of `h` (which is off
    /// the coset for the pivot rows, so it exercises the zero path too). That is
    /// the same shape `Affine::amplitudes_agree` uses, and for the same reason:
    /// the phase polynomial is determined by its values on singles and pairs.
    pub fn agrees_with(&self, weight: Cyc, state: &Affine, budget: usize) -> bool {
        let probe = |y: u64| -> bool {
            let ybits: Vec<bool> = (0..self.n).map(|q| y >> q & 1 == 1).collect();
            let theirs = ring::normalize(weight.mul(state.amplitude(&ybits)));
            let mine = self.amplitude(y);
            theirs == mine
        };
        let point = |u: u64| -> u64 { self.point(u) };
        if !probe(point(0)) {
            return false;
        }
        let mut checked = 1usize;
        for a in 0..self.k {
            if checked >= budget {
                break;
            }
            if !probe(point(1 << a)) {
                return false;
            }
            checked += 1;
        }
        'pairs: for a in 0..self.k {
            for b in a + 1..self.k {
                if checked >= budget {
                    break 'pairs;
                }
                if !probe(point((1 << a) | (1 << b))) {
                    return false;
                }
                checked += 1;
            }
        }
        for q in 0..self.n {
            if !probe(self.h ^ (1 << q)) {
                return false;
            }
        }
        true
    }
}

/// Pack a `&[bool]` basis state into the bitmask the descriptor path uses.
#[inline]
pub fn pack_y(y: &[bool]) -> u64 {
    let mut out = 0u64;
    for (q, &b) in y.iter().enumerate() {
        out |= (b as u64) << q;
    }
    out
}

/// A batch of descriptors, as a `holon::BranchSource`.
///
/// This is the CPU baseline of the benchmark and the reference of the
/// determinism test: `holon::mesh::fold_amplitude` folds THIS, with `holon`'s
/// own `Cyc` and `holon`'s own merge law, and the GPU has to match it. It is a
/// faster CPU path than folding `PrunedSum` directly (packed `u64` rows against
/// `Vec<Vec<bool>>`), which makes every speedup quoted against it the
/// CONSERVATIVE number — the honest direction for a claim.
pub struct DescSource {
    pub descs: Vec<AffineDesc>,
    pub n: usize,
}

impl DescSource {
    pub fn from_pruned(sum: &holon::prune::PrunedSum) -> Result<Self, DescError> {
        let descs = sum
            .branches
            .iter()
            .map(|b| AffineDesc::from_branch(b.weight, &b.state))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DescSource { descs, n: sum.n })
    }
}

impl holon::BranchSource for DescSource {
    fn n_branches(&self) -> u64 {
        self.descs.len() as u64
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        self.descs[branch as usize].amplitude(pack_y(y))
    }
    fn n_qubits(&self) -> usize {
        self.n
    }
}
