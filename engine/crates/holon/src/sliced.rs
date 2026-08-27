//! BRANCH SLICING — the tier-0 word-parallel trick promoted to the branch axis.
//!
//! Tier 0 stores one observable across many degrees of freedom in a word and
//! moves 64 of them per instruction ([`crate::plane::BitPlane`]). The magic
//! tier has a second axis nobody was spending words on: a Clifford+T circuit
//! written as `Σ_b c_b |φ_b⟩` runs the SAME gate sequence down every branch,
//! and the branches differ only in the `Z`'s that T-resolution injects. So the
//! branch index is a plane axis too. This module stores 64 branches
//! interleaved — every word is one state bit across 64 branches — and runs
//! them through the circuit together.
//!
//! # The structural theorem this rests on (and it is a theorem, not a hope)
//!
//! Take 64 branches of one circuit. Write their affine states as
//! `(R, h, d, J, γ)` ([`crate::affine::Affine`]). Then:
//!
//! > **`R`, `J` and `k` are identical in every branch, at every point in the
//! > circuit; only `h`, `d` and `γ` can differ, and `d` can differ only by
//! > MULTIPLES OF 2 — so `d_a mod 2` is also branch-independent, for every
//! > column `a`.**
//!
//! Proof by induction over the update rules in `affine.rs`, one write at a
//! time. The base case is `|0…0⟩`: one state, so all three hold.
//!
//! * `T`-resolution injects `Z(q)`, and `Affine::z` writes only `d_a += 2`
//!   (parity preserved) and a `γ` phase. So the branch-generating step itself
//!   never touches `R`, `J`, `k`, and never changes any `d` parity.
//! * every write to `d` in the file is one of: `+2` (`z`, `flip`'s
//!   neighbour bump, `pin_remove`'s neighbour bump), `±1` (`s`, whose bump is
//!   `1` or `3` — both ODD, so the parity flip is the same in every branch
//!   whatever `h_q` is), `d_b += d_a` (`fold`, parity-additive), `d_b += δ+2`
//!   with `δ = d_a` odd (`gauss_sum_out`, parity flip), `d_p ↦ −d_p` (`flip`,
//!   parity-fixed), or a push of `0`/`2` (`h_gate`, parity 0). Every one
//!   preserves branch-independence of the parity.
//! * every write to `R` (`fold`, `swap_cols`, `remove_col`, `cx`, `h_gate`)
//!   is conditioned on `R` alone.
//! * every write to `J` is conditioned on `R`, on `J`, or on `d_a mod 2` —
//!   the last through `fold`'s `J_ab ^= d_a & 1`, which by the parity clause
//!   is the same bit in every branch.
//! * every CONTROL-FLOW branch in the file reads `R`, `J`, or `δ mod 2`
//!   (`gauss_sum_out`'s even/odd split). All three are branch-independent, so
//!   64 branches walk the same code path — which is what makes them sliceable
//!   at all.
//!
//! What is left over is genuinely per-branch: `h` (through `pin_remove` with a
//! branch-dependent `val`, reachable from `gauss_sum_out`'s even-δ case),
//! `d`'s high bit, `γ`, and the annihilation flag. Those are the four things
//! this module carries in lanes.
//!
//! The theorem is not left as prose: [`SlicedAffine::parity_is_lane_uniform`]
//! checks it, `debug_assert!` fires on it inside [`SlicedAffine::fold`] (the
//! one place the parity is READ as a decision), and the conformance test
//! compares every one of the 64 lanes against a scalar `Affine` run of that
//! same branch, exactly.
//!
//! # What that buys, and what it does not
//!
//! The shared part is all of the F₂ linear algebra — reduced column echelon
//! form, the dependence repair, the amplitude solve — which is the `O(n²k)`
//! part and the whole cost centre. It is now paid ONCE per 64 branches.
//! The per-lane part is `Z[ω]` ring multiplies on `γ` and the branch weight,
//! which cannot be word-parallel because exact `i128` coefficients are not
//! bits. That is exactly where the 64× ideal is lost, it is measured in
//! `src/bin/holon-sliced-bench.rs`, and it is reported as a number rather
//! than as a hope.
//!
//! # Exactness
//!
//! Nothing here approximates. The lane arithmetic on `d` is 2-bit-plane
//! addition mod 4 (a carry and three XORs), the lane arithmetic on `h` and on
//! the annihilation flag is a mask, and every ring operation is a `Cyc` call
//! on the same operand the scalar path uses — so a lane's weight is
//! bit-identical to that branch's weight, not merely equal to it. `Cyc`'s
//! overflow refusal is untouched and still fires.
//!
//! ONE thing is reordered rather than transcribed, and it is named here
//! because "bit-identical" is a claim that has to survive it: consecutive
//! multiplications by powers of `i` are POSTED to a 2-bit lane plane and
//! cashed once ([`SlicedConfig::defer_phase`]). That is exact — a unit
//! multiply is a signed permutation of the coefficient vector and
//! `Cyc::normalize` is equivariant under it, so `i^a · i^b` and `i^{a+b}`
//! agree in the representation and not only in the value — and it is not left
//! at "exact by argument": the conformance test compares the DEFERRED lanes
//! against UNDEFERRED scalar branches, so the two schedules are made to agree
//! bit for bit on live circuits, and `defer_phase = false` restores the
//! per-branch schedule exactly if that ever stops being true.
//!
//! # One merge law
//!
//! The lanes of a block fold with `merge::fold`; the blocks fold with
//! `mesh::fold_amplitude` through the [`Blocks`] adapter, which reads a
//! [`BranchBlockSource`] as a [`BranchSource`] whose "branch" is a block of
//! 64. There is no second accumulation mechanism in this file, and no second
//! sharding mechanism: the mesh's chart and its shard-index-ordered reduction
//! are the ones that run.
//!
//! # Branch indexing
//!
//! Branch `b ∈ [0, 2^t)` resolves T-site `j` (counted from the first T-gate)
//! by bit `j` of `b`: bit clear takes the identity term, bit set takes the
//! `Z` term. Block `B` holds branches `[64B, 64B + 64)`, so T-sites 0..6 vary
//! WITHIN a block (a fixed lane mask each) and T-sites 6.. are fixed across
//! it (all lanes or none). `t < 6` leaves the high lanes inactive, and they
//! are inactive by carrying a zero weight — never by a special case.
//!
//! This is a different traversal order from `prune::run_pruned`, which pushes
//! the identity child before the `Z` child and so puts the FIRST T-site in
//! the HIGH bit. Both enumerate the same `2^t` branches once each; the sum is
//! the same object, and the fold is order-independent by the merge law.
//!
//! Zero runtime dependencies (`std` only).

use crate::affine::{i_pow, Gate};
use crate::ledger::Cyc;
use crate::merge::{self, MergeLedger};
use crate::prune::t_coeffs;
use crate::BranchSource;

/// Branches per word. The whole module is this constant.
pub const LANES: usize = 64;

/// Lane mask of T-site `site` in block `block`: the lanes whose branch index
/// has bit `site` set, i.e. the lanes that take the `Z` term.
///
/// Sites below 6 vary inside the block and get a fixed stripe; sites at 6 and
/// above are decided by the block index, so the mask is all-ones or zero.
#[inline]
pub fn t_site_mask(site: usize, block: u64) -> u64 {
    const STRIPE: [u64; 6] = [
        0xAAAA_AAAA_AAAA_AAAA,
        0xCCCC_CCCC_CCCC_CCCC,
        0xF0F0_F0F0_F0F0_F0F0,
        0xFF00_FF00_FF00_FF00,
        0xFFFF_0000_FFFF_0000,
        0xFFFF_FFFF_0000_0000,
    ];
    if site < 6 {
        STRIPE[site]
    } else if (block >> (site - 6)) & 1 == 1 {
        u64::MAX
    } else {
        0
    }
}

/// Lanes that carry a real branch when the circuit has `t` T-gates: all 64
/// once `t ≥ 6`, the low `2^t` otherwise.
#[inline]
pub fn active_lanes(t: usize) -> u64 {
    if t >= 6 {
        u64::MAX
    } else {
        // 2^t ≤ 32 here, so the shift is always in range.
        ((1u128 << (1usize << t)) - 1) as u64
    }
}

/// Blocks needed for `t` T-gates: `⌈2^t / 64⌉`, and at least one.
#[inline]
pub fn block_count(t: usize) -> u64 {
    1u64 << (t.saturating_sub(6).min(63))
}

// ------------------------------------------------------- the sliced state

/// 64 affine branches, interleaved. `r`, `j` and `k` are SHARED (the theorem
/// in the module header); `h`, `d` and `gamma` are per-lane.
#[derive(Clone, Debug)]
pub struct SlicedAffine {
    n: usize,
    /// `R`: n rows × k columns. Branch-independent.
    r: Vec<Vec<bool>>,
    /// `J`: k × k symmetric, diagonal unused. Branch-independent.
    j: Vec<Vec<bool>>,
    /// `h`, one lane-word per qubit: bit L = branch L's `h_row`.
    h: Vec<u64>,
    /// `d` as two bit-planes per column: value in lane L is
    /// `bit_L(d0[a]) + 2·bit_L(d1[a])`, mod 4.
    d0: Vec<u64>,
    d1: Vec<u64>,
    /// The per-lane global scalar. Exact, so it cannot be a plane.
    gamma: Vec<Cyc>,
    /// Lanes whose branch has been annihilated. Their `gamma` is `ZERO`, so
    /// the flag is a diagnostic — the arithmetic already carries the fact.
    zero: u64,
    /// PENDING per-lane power of `i`, as a 2-bit plane pair — the deferral
    /// described on [`SlicedConfig::defer_phase`]. Accumulating into it is one
    /// word op for 64 branches; cashing it is 64 exact multiplies, so it is
    /// cashed as late as the ring allows.
    pacc0: u64,
    pacc1: u64,
    /// False makes every phase update cash immediately — the per-branch
    /// path's own schedule, kept so the bench can measure the two arms on
    /// exactly the same algorithm.
    defer: bool,
}

impl SlicedAffine {
    /// 64 copies of `|0…0⟩` on n qubits, with phase deferral on.
    pub fn new(n: usize) -> Self {
        SlicedAffine::with_deferral(n, true)
    }

    pub fn with_deferral(n: usize, defer: bool) -> Self {
        SlicedAffine {
            n,
            r: vec![Vec::new(); n],
            j: Vec::new(),
            h: vec![0u64; n],
            d0: Vec::new(),
            d1: Vec::new(),
            gamma: vec![Cyc::ONE; LANES],
            zero: 0,
            pacc0: 0,
            pacc1: 0,
            defer,
        }
    }

    pub fn n_qubits(&self) -> usize {
        self.n
    }

    /// The shared affine dimension.
    pub fn k(&self) -> usize {
        self.d0.len()
    }

    /// Lanes annihilated so far.
    pub fn zero_mask(&self) -> u64 {
        self.zero
    }

    /// The module header's parity clause, as a check rather than a claim:
    /// every column's `d` parity is the same in all 64 lanes.
    pub fn parity_is_lane_uniform(&self) -> bool {
        self.d0.iter().all(|&w| w == 0 || w == u64::MAX)
    }

    // ------------------------------------------------------ lane arithmetic

    /// `d_a += c (mod 4)` on the masked lanes. Two-bit-plane addition: the
    /// low plane is an XOR and the high plane takes the carry (or borrow).
    #[inline]
    fn d_add_const(&mut self, a: usize, c: u8, mask: u64) {
        match c & 3 {
            0 => {}
            2 => self.d1[a] ^= mask,
            1 => {
                let carry = self.d0[a] & mask;
                self.d0[a] ^= mask;
                self.d1[a] ^= carry;
            }
            _ => {
                let borrow = !self.d0[a] & mask;
                self.d0[a] ^= mask;
                self.d1[a] ^= borrow;
            }
        }
    }

    /// `d_b += d_a (mod 4)` on the masked lanes.
    #[inline]
    fn d_add_col(&mut self, b: usize, a: usize, mask: u64) {
        debug_assert_ne!(a, b);
        let (v0, v1) = (self.d0[a], self.d1[a]);
        let (x0, x1) = (self.d0[b], self.d1[b]);
        let carry = x0 & v0;
        let s0 = x0 ^ v0;
        let s1 = x1 ^ v1 ^ carry;
        self.d0[b] = (x0 & !mask) | (s0 & mask);
        self.d1[b] = (x1 & !mask) | (s1 & mask);
    }

    /// `d_a ↦ −d_a (mod 4)` on the masked lanes. Negation mod 4 keeps the low
    /// bit and XORs it into the high bit — which is why it cannot disturb the
    /// parity the theorem depends on.
    #[inline]
    fn d_neg(&mut self, a: usize, mask: u64) {
        self.d1[a] ^= self.d0[a] & mask;
    }

    /// `γ ·= c` on the masked lanes — the same `Cyc::mul` the scalar engine
    /// calls, on the same operand, so the lane is bit-identical to the branch.
    #[inline]
    fn gamma_mul(&mut self, c: Cyc, mask: u64) {
        let mut bits = mask;
        while bits != 0 {
            let l = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            self.gamma[l] = self.gamma[l].mul(c);
        }
    }

    /// `γ ·= i^c` on the masked lanes — POSTED to the pending power, not paid.
    #[inline]
    fn phase_bump_const(&mut self, c: u8, mask: u64) {
        match c & 3 {
            0 => {}
            2 => self.pacc1 ^= mask,
            1 => {
                let carry = self.pacc0 & mask;
                self.pacc0 ^= mask;
                self.pacc1 ^= carry;
            }
            _ => {
                let borrow = !self.pacc0 & mask;
                self.pacc0 ^= mask;
                self.pacc1 ^= borrow;
            }
        }
        if !self.defer {
            self.flush_phase();
        }
    }

    /// `γ ·= i^{d_a}` on the masked lanes — likewise posted. This is the shape
    /// the pivot loop runs `k` times a pass, and posting it is ONE word op for
    /// all 64 branches where paying it is 64 exact ring multiplies.
    #[inline]
    fn phase_bump_d(&mut self, a: usize, mask: u64) {
        let (v0, v1) = (self.d0[a] & mask, self.d1[a] & mask);
        let carry = self.pacc0 & v0;
        self.pacc0 ^= v0;
        self.pacc1 ^= v1 ^ carry;
        if !self.defer {
            self.flush_phase();
        }
    }

    /// Cash the pending power: one `Cyc::mul_i_pow` per lane, grouped by the
    /// power so the ring call is made once per group member and not once per
    /// posting. Exact — `mul_i_pow` is bit-identical to the general multiply,
    /// and `i^a · i^b = i^{a+b}` holds in the REPRESENTATION as well as the
    /// value because a unit multiply is a signed permutation of the
    /// coefficient vector and `normalize` is equivariant under it.
    fn flush_phase(&mut self) {
        let (p0, p1) = (self.pacc0, self.pacc1);
        if p0 == 0 && p1 == 0 {
            return;
        }
        for (m, k) in [(p0 & !p1, 1u8), (!p0 & p1, 2), (p0 & p1, 3)] {
            let mut bits = m;
            while bits != 0 {
                let l = bits.trailing_zeros() as usize;
                bits &= bits - 1;
                self.gamma[l] = self.gamma[l].mul_i_pow(k);
            }
        }
        self.pacc0 = 0;
        self.pacc1 = 0;
    }

    /// `γ.m += delta` on the masked lanes (the ring's `2^{−m/2}` bookkeeping;
    /// the scalar engine writes `m` directly here too).
    #[inline]
    fn gamma_m_add(&mut self, delta: i32, mask: u64) {
        let mut bits = mask;
        while bits != 0 {
            let l = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            self.gamma[l].m += delta;
        }
    }

    /// Annihilate the masked lanes. The weight of a dead branch is exactly
    /// zero, and `Cyc::ZERO` absorbs every later multiply, so nothing
    /// downstream needs a special case for it.
    fn kill(&mut self, mask: u64) {
        self.zero |= mask;
        self.pacc0 &= !mask;
        self.pacc1 &= !mask;
        let mut bits = mask;
        while bits != 0 {
            let l = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            self.gamma[l] = Cyc::ZERO;
        }
    }

    // ------------------------------------------------------------ gauge moves

    /// `u_a := u_a ⊕ u_b` — the elementary F₂ column operation, phase carried.
    /// The one place the parity theorem is READ as a decision (`J_ab ^= d_a&1`),
    /// so it is also the place the theorem is asserted.
    fn fold(&mut self, a: usize, b: usize) {
        assert_ne!(a, b);
        for row in 0..self.n {
            let ra = self.r[row][a];
            self.r[row][b] ^= ra;
        }
        let da_par = self.d0[a];
        debug_assert!(
            da_par == 0 || da_par == u64::MAX,
            "branch slicing: d parity is not lane-uniform — the structural theorem is broken"
        );
        let jab_old = self.j[a][b];
        let ja_row: Vec<bool> = self.j[a].clone();
        self.d_add_col(b, a, u64::MAX);
        self.j[a][b] ^= da_par != 0;
        self.j[b][a] = self.j[a][b];
        for c in 0..self.k() {
            if c != a && c != b && ja_row[c] {
                self.j[b][c] = !self.j[b][c];
                self.j[c][b] = self.j[b][c];
            }
        }
        if jab_old {
            self.d_add_const(b, 2, u64::MAX);
        }
    }

    /// `u_p := 1 ⊕ u_p` on the masked lanes — moving the coset origin along
    /// column p, per branch.
    fn flip(&mut self, p: usize, mask: u64) {
        self.phase_bump_d(p, mask);
        for a in 0..self.k() {
            if a != p && self.j[p][a] {
                self.d_add_const(a, 2, mask);
            }
        }
        self.d_neg(p, mask);
        for row in 0..self.n {
            if self.r[row][p] {
                self.h[row] ^= mask;
            }
        }
    }

    fn swap_cols(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        for row in 0..self.n {
            self.r[row].swap(a, b);
        }
        self.d0.swap(a, b);
        self.d1.swap(a, b);
        self.j.swap(a, b);
        for jr in &mut self.j {
            jr.swap(a, b);
        }
    }

    fn remove_col(&mut self, a: usize) {
        for row in 0..self.n {
            self.r[row].remove(a);
        }
        self.d0.remove(a);
        self.d1.remove(a);
        self.j.remove(a);
        for jr in &mut self.j {
            jr.remove(a);
        }
    }

    /// Remove column a with `u_a` pinned to 1 on the masked lanes and 0 on the
    /// rest. This is the ONE move that makes `h` genuinely per-branch.
    fn pin_remove(&mut self, a: usize, val: u64) {
        for row in 0..self.n {
            if self.r[row][a] {
                self.h[row] ^= val;
            }
        }
        self.phase_bump_d(a, val);
        for c in 0..self.k() {
            if c != a && self.j[a][c] {
                self.d_add_const(c, 2, val);
            }
        }
        self.remove_col(a);
    }

    /// Sum out phase-only column a. The even/odd split reads `δ mod 2`, which
    /// the theorem makes lane-uniform — so all 64 lanes take the same arm, and
    /// only the VALUES inside the arm are per-lane.
    fn gauss_sum_out(&mut self, a: usize) {
        debug_assert!(
            (0..self.n).all(|row| !self.r[row][a]),
            "gauss_sum_out on a column that still carries an x-dependence"
        );
        debug_assert!(
            self.d0[a] == 0 || self.d0[a] == u64::MAX,
            "branch slicing: δ parity is not lane-uniform"
        );
        let odd = self.d0[a] != 0;
        let l: Vec<usize> = (0..self.k()).filter(|&b| b != a && self.j[a][b]).collect();
        if !odd {
            // δ ∈ {0,2}: a CONSTRAINT Λ ≡ eps, and eps is per-lane.
            let eps = self.d1[a];
            if l.is_empty() {
                self.kill(eps);
                self.gamma_m_add(-2, !eps);
                self.remove_col(a);
            } else {
                let c = l[0];
                for &b in &l[1..] {
                    self.fold(c, b);
                }
                self.gamma_m_add(-2, u64::MAX);
                self.remove_col(a);
                let c_adj = if c > a { c - 1 } else { c };
                self.pin_remove(c_adj, eps);
            }
        } else {
            // δ odd: ×(1+i^δ) with δ ∈ {1,3} per lane, then d_b += δ+2 for
            // b ∈ L and the pairwise J flips across L (the XOR expansion —
            // `affine.rs` carries its receipt).
            let m1 = self.d0[a] & !self.d1[a];
            let m3 = self.d0[a] & self.d1[a];
            // The one GENERAL ring multiply in the engine. The pending power
            // is cashed first, so the multiplication order a lane sees is the
            // per-branch path's order exactly.
            self.flush_phase();
            if m1 != 0 {
                self.gamma_mul(Cyc::ONE.merge(i_pow(1)), m1);
            }
            if m3 != 0 {
                self.gamma_mul(Cyc::ONE.merge(i_pow(3)), m3);
            }
            for &b in &l {
                self.d_add_col(b, a, u64::MAX);
                self.d_add_const(b, 2, u64::MAX);
            }
            for i1 in 0..l.len() {
                for i2 in i1 + 1..l.len() {
                    let (b1, b2) = (l[i1], l[i2]);
                    self.j[b1][b2] = !self.j[b1][b2];
                    self.j[b2][b1] = self.j[b1][b2];
                }
            }
            self.remove_col(a);
        }
    }

    // ------------------------------------------------------- Clifford gates

    pub fn x(&mut self, q: usize) {
        self.h[q] ^= u64::MAX;
    }

    /// `Z(q)` on the masked lanes — the branch-dependent gate, and the only
    /// one. Everything else in the alphabet runs on all 64 at once.
    pub fn z(&mut self, q: usize, mask: u64) {
        let hm = self.h[q] & mask;
        if hm != 0 {
            self.phase_bump_const(2, hm);
        }
        for a in 0..self.k() {
            if self.r[q][a] {
                self.d_add_const(a, 2, mask);
            }
        }
    }

    pub fn s(&mut self, q: usize) {
        let a_set: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        let hq = self.h[q];
        if hq != 0 {
            self.phase_bump_const(1, hq);
        }
        // bump = 3 where h_q, 1 elsewhere: both ODD, which is why `s` cannot
        // break the parity clause even though `h` is per-branch.
        for &a in &a_set {
            self.d_add_const(a, 1, !hq);
            self.d_add_const(a, 3, hq);
        }
        for i in 0..a_set.len() {
            for jj in i + 1..a_set.len() {
                let (a, b) = (a_set[i], a_set[jj]);
                self.j[a][b] = !self.j[a][b];
                self.j[b][a] = self.j[a][b];
            }
        }
    }

    pub fn sdg(&mut self, q: usize) {
        self.s(q);
        self.s(q);
        self.s(q);
    }

    pub fn cx(&mut self, c: usize, t: usize) {
        for a in 0..self.k() {
            let rc = self.r[c][a];
            self.r[t][a] ^= rc;
        }
        let hc = self.h[c];
        self.h[t] ^= hc;
    }

    pub fn h_gate(&mut self, q: usize) {
        let support: Vec<usize> = (0..self.k()).filter(|&a| self.r[q][a]).collect();
        let a_star = if support.is_empty() {
            None
        } else {
            let a = support[0];
            for &b in &support[1..] {
                self.fold(a, b);
            }
            Some(a)
        };
        let v = self.k();
        for row in 0..self.n {
            self.r[row].push(false);
        }
        // `d_v = 2` where h_q, else 0 — parity 0 in every lane.
        self.d0.push(0);
        self.d1.push(self.h[q]);
        for jr in &mut self.j {
            jr.push(false);
        }
        self.j.push(vec![false; v + 1]);
        if let Some(a) = a_star {
            self.j[a][v] = true;
            self.j[v][a] = true;
        }
        for a in 0..self.k() {
            self.r[q][a] = false;
        }
        self.r[q][v] = true;
        self.h[q] = 0;
        for g in self.gamma.iter_mut() {
            g.m += 1;
        }
        if let Some(a) = a_star {
            if !(0..self.n).all(|row| !self.r[row][a]) {
                if let Some(subset) = self.dependent_subset(a) {
                    for b in subset {
                        self.fold(b, a);
                    }
                }
            }
            if (0..self.n).all(|row| !self.r[row][a]) {
                self.gauss_sum_out(a);
            }
        }
    }

    /// If column a is an XOR of other columns, return that subset. Reads `R`
    /// only, so its answer is the same for all 64 branches by the theorem.
    fn dependent_subset(&self, a: usize) -> Option<Vec<usize>> {
        let k = self.k();
        let others: Vec<usize> = (0..k).filter(|&b| b != a).collect();
        let mut rows: Vec<(Vec<bool>, bool)> = (0..self.n)
            .map(|r| (others.iter().map(|&b| self.r[r][b]).collect(), self.r[r][a]))
            .collect();
        let m = others.len();
        let mut piv = vec![usize::MAX; m];
        let mut rr = 0;
        for col in 0..m {
            if let Some(p) = (rr..self.n).find(|&p| rows[p].0[col]) {
                rows.swap(rr, p);
                for p2 in 0..self.n {
                    if p2 != rr && rows[p2].0[col] {
                        let src = rows[rr].clone();
                        rows[p2].0.iter_mut().zip(&src.0).for_each(|(x, y)| *x ^= *y);
                        rows[p2].1 ^= src.1;
                    }
                }
                piv[col] = rr;
                rr += 1;
            }
        }
        if rows[rr..].iter().any(|r| r.1) {
            return None;
        }
        let mut subset = Vec::new();
        for col in 0..m {
            if piv[col] != usize::MAX && rows[piv[col]].1 {
                subset.push(others[col]);
            }
        }
        Some(subset)
    }

    /// Apply a Clifford gate to all 64 branches. `T`/`T†` branch, and a branch
    /// is not a state update — the driver resolves them into a lane mask.
    pub fn apply(&mut self, g: Gate) {
        match g {
            Gate::X(q) => self.x(q),
            Gate::Z(q) => self.z(q, u64::MAX),
            Gate::S(q) => self.s(q),
            Gate::Sdg(q) => self.sdg(q),
            Gate::H(q) => self.h_gate(q),
            Gate::Cx(c, t) => self.cx(c, t),
            Gate::T(_) | Gate::Tdg(_) => panic!("magic tier branches must be Clifford"),
        }
    }

    // ------------------------------------------------------- canonical form

    /// Reduced column echelon form of the shared `R`, phase polynomial carried.
    fn rcef(&mut self) -> Vec<(usize, usize)> {
        let mut pivots = Vec::new();
        let mut p = 0usize;
        for row in 0..self.n {
            if p >= self.k() {
                break;
            }
            let Some(c) = (p..self.k()).find(|&c| self.r[row][c]) else {
                continue;
            };
            self.swap_cols(c, p);
            for c2 in 0..self.k() {
                if c2 != p && self.r[row][c2] {
                    self.fold(p, c2);
                }
            }
            pivots.push((row, p));
            p += 1;
        }
        pivots
    }

    /// Put all 64 branches in canonical form and STRIP the per-lane global
    /// scalar into `weight` — the same move `prune::merge_block` makes when it
    /// posts `b.weight = b.weight.mul(g)`, one lane at a time.
    ///
    /// An annihilated lane carries `γ = ZERO`, so the strip sets its weight to
    /// exactly zero, which is what the scalar `canonicalize` returns for it.
    pub fn canonicalize_into(&mut self, weight: &mut [Cyc]) {
        loop {
            let pivots = self.rcef();
            if let Some(c) = (0..self.k()).find(|&c| (0..self.n).all(|row| !self.r[row][c])) {
                self.gauss_sum_out(c);
                continue;
            }
            for &(row, col) in &pivots {
                let m = self.h[row];
                if m != 0 {
                    self.flip(col, m);
                }
            }
            break;
        }
        self.flush_phase();
        for (l, w) in weight.iter_mut().enumerate() {
            *w = w.mul(self.gamma[l]);
            self.gamma[l] = Cyc::ONE;
        }
    }
}

// ------------------------------------------------------------- one block

/// One block: 64 branches of a circuit, evolved together, with the per-lane
/// T-expansion weight held alongside.
#[derive(Clone, Debug)]
pub struct SlicedBlock {
    pub state: SlicedAffine,
    /// `c_b · γ_b`, per lane — the T-expansion coefficient with every stripped
    /// global scalar already folded in.
    pub weight: Vec<Cyc>,
}

impl SlicedBlock {
    /// Exact amplitude of basis state `y` in each of the 64 lanes.
    ///
    /// The `R u = y ⊕ h` solve is done ONCE for all 64: `R` is shared, so the
    /// elimination is shared, and the per-branch right-hand side rides in a
    /// 64-bit lane word through the same row operations. What used to be 64
    /// separate `O(n·k²)` F₂ eliminations is one, with 64 lanes of RHS
    /// carried for free.
    pub fn amplitudes(&self, y: &[bool], out: &mut [Cyc]) {
        assert_eq!(out.len(), LANES, "amplitudes: out must hold 64 lanes");
        let st = &self.state;
        assert_eq!(y.len(), st.n, "amplitudes: |y| must be the qubit count");
        let k = st.k();
        let words = k.div_ceil(64).max(1);
        // Pack R row-wise (k bits per row) and put y ⊕ h in the lane word.
        let mut rows: Vec<Vec<u64>> = vec![vec![0u64; words]; st.n];
        let mut rhs: Vec<u64> = vec![0u64; st.n];
        for row in 0..st.n {
            let dst = &mut rows[row];
            for a in 0..k {
                if st.r[row][a] {
                    dst[a >> 6] |= 1u64 << (a & 63);
                }
            }
            rhs[row] = if y[row] { u64::MAX } else { 0 } ^ st.h[row];
        }
        // Full reduction, exactly the scalar solver's shape.
        let mut pivot_row = vec![usize::MAX; k];
        let mut rr = 0usize;
        let mut piv = vec![0u64; words];
        for col in 0..k {
            let (wi, bi) = (col >> 6, col & 63);
            let Some(p) = (rr..st.n).find(|&p| rows[p][wi] >> bi & 1 == 1) else {
                continue;
            };
            rows.swap(rr, p);
            rhs.swap(rr, p);
            piv.copy_from_slice(&rows[rr]);
            let prhs = rhs[rr];
            for p2 in 0..st.n {
                if p2 != rr && rows[p2][wi] >> bi & 1 == 1 {
                    rhs[p2] = crate::simd::fused_row_xor(&mut rows[p2], rhs[p2], &piv, prhs);
                }
            }
            pivot_row[col] = rr;
            rr += 1;
        }
        // Lanes whose y is off that branch's affine subspace read exactly zero.
        let mut off = 0u64;
        for row in rr..st.n {
            off |= rhs[row];
        }
        assert!(
            (0..k).all(|col| pivot_row[col] != usize::MAX),
            "affine invariant broken: R has dependent columns (rank < k)"
        );
        // u, per lane, one word per column — free: it is the reduced RHS.
        // Σ_a u_a d_a mod 4 as a 2-bit-plane accumulation.
        let (mut a0, mut a1) = (0u64, 0u64);
        for c in 0..k {
            let uw = rhs[pivot_row[c]];
            let v0 = st.d0[c] & uw;
            let v1 = st.d1[c] & uw;
            let carry = a0 & v0;
            a0 ^= v0;
            a1 ^= v1 ^ carry;
        }
        // Σ_{a<b} J_ab u_a u_b mod 2. AND distributes over XOR, so the inner
        // sum is one XOR-reduction of u-words over J's row.
        let mut sign = 0u64;
        for a in 0..k {
            let ua = rhs[pivot_row[a]];
            if ua == 0 {
                continue;
            }
            let mut t = 0u64;
            for b in a + 1..k {
                if st.j[a][b] {
                    t ^= rhs[pivot_row[b]];
                }
            }
            sign ^= ua & t;
        }
        debug_assert!(
            st.pacc0 == 0 && st.pacc1 == 0,
            "amplitudes: a phase posting was never cashed"
        );
        for (l, slot) in out.iter_mut().enumerate() {
            let bit = 1u64 << l;
            if off & bit != 0 {
                *slot = Cyc::ZERO;
                continue;
            }
            let ip = ((a0 >> l) & 1) as u8 + 2 * (((a1 >> l) & 1) as u8);
            let mut amp = st.gamma[l].mul_i_pow(ip);
            if sign & bit != 0 {
                amp = amp.mul_i_pow(2);
            }
            *slot = self.weight[l].mul(amp);
        }
    }

    /// The block's contribution to `⟨y|ψ⟩`: the 64 lanes folded in ascending
    /// lane order through the one merge law.
    pub fn amplitude(&self, y: &[bool]) -> Cyc {
        let mut buf = vec![Cyc::ZERO; LANES];
        self.amplitudes(y, &mut buf);
        merge::fold(buf)
    }
}

// --------------------------------------------------------- the batched contract

/// The batched branch contract: a source that hands out BLOCKS of 64
/// consecutive branch indices rather than one branch at a time.
///
/// Alongside [`BranchSource`], never instead of it — [`Blocks`] is the
/// adapter that puts a block source back on the mesh's one fold.
pub trait BranchBlockSource: Sync {
    fn n_blocks(&self) -> u64;
    fn n_qubits(&self) -> usize;
    /// The 64 branch amplitudes of `block`, in ascending lane order.
    fn block_amplitudes(&self, block: u64, y: &[bool], out: &mut [Cyc]);
    /// The block's folded contribution. Default: the one merge law over the
    /// 64 lanes, in ascending order.
    fn block_amplitude(&self, block: u64, y: &[bool]) -> Cyc {
        let mut buf = vec![Cyc::ZERO; LANES];
        self.block_amplitudes(block, y, &mut buf);
        merge::fold(buf)
    }
}

/// A [`BranchBlockSource`] read as a [`BranchSource`] whose "branch" is a
/// block of 64. This is the whole of the integration: `mesh::fold_amplitude`,
/// `mesh::shard_ranges` and `merge::fold` are then the code that runs, with no
/// second sharding mechanism and no second accumulation anywhere.
pub struct Blocks<'a, S: BranchBlockSource>(pub &'a S);

impl<S: BranchBlockSource> BranchSource for Blocks<'_, S> {
    fn n_branches(&self) -> u64 {
        self.0.n_blocks()
    }
    fn amplitude_of(&self, branch: u64, y: &[bool]) -> Cyc {
        self.0.block_amplitude(branch, y)
    }
    fn n_qubits(&self) -> usize {
        self.0.n_qubits()
    }
}

// ------------------------------------------------------------------ driver

#[derive(Clone, Copy, Debug)]
pub struct SlicedConfig {
    /// Defer the per-lane phase multiplies: post `i^k` into a 2-bit lane plane
    /// (one word op for all 64 branches) and cash it once, rather than paying
    /// 64 exact ring multiplies at every posting. Exact either way — the
    /// conformance test compares the DEFERRED path lane-by-lane against an
    /// UNDEFERRED scalar branch, so the equality of the two schedules is
    /// tested, not assumed.
    ///
    /// `false` runs the per-branch path's own schedule. It exists so the bench
    /// can put both arms on exactly one algorithm and report what slicing
    /// alone buys, separately from what the deferral buys — the same deferral
    /// is available to the per-branch engine and has not been applied there.
    pub defer_phase: bool,
    /// Refuse to materialise more blocks than this. A magic tier that
    /// silently blows up is worse than one that stops — the same fence
    /// `PruneConfig::max_working_set` holds on the per-branch path, moved to
    /// the axis this path grows along.
    pub max_blocks: usize,
    /// Canonicalize after every `merge_every` T-sites. `1` mirrors the
    /// per-branch default and is what the conformance test pins.
    pub merge_every: usize,
}

impl Default for SlicedConfig {
    fn default() -> Self {
        SlicedConfig { defer_phase: true, max_blocks: 1 << 16, merge_every: 1 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SlicedStats {
    pub t_count: usize,
    pub n_blocks: u64,
    /// `2^t` — every branch is kept; slicing prunes nothing and claims nothing.
    pub branches: u128,
    /// Lanes annihilated across all blocks (the `gauss_sum_out` zero arm).
    pub annihilated_lanes: u64,
}

/// The branch-sliced sum: `⌈2^t/64⌉` blocks, each carrying 64 branches.
pub struct SlicedSum {
    pub n: usize,
    pub blocks: Vec<SlicedBlock>,
    pub stats: SlicedStats,
}

impl BranchBlockSource for SlicedSum {
    fn n_blocks(&self) -> u64 {
        self.blocks.len() as u64
    }
    fn n_qubits(&self) -> usize {
        self.n
    }
    fn block_amplitudes(&self, block: u64, y: &[bool], out: &mut [Cyc]) {
        self.blocks[block as usize].amplitudes(y, out);
    }
}

/// Evolve ONE block of 64 branches through the circuit.
///
/// `block` selects which 64 consecutive branch indices; `t` is the circuit's
/// T-count, needed only to decide which lanes carry a real branch.
pub fn run_block(n: usize, gates: &[Gate], block: u64, t: usize, cfg: &SlicedConfig) -> SlicedBlock {
    let mut st = SlicedAffine::with_deferral(n, cfg.defer_phase);
    let mut weight = vec![Cyc::ONE; LANES];
    let active = active_lanes(t);
    for (l, w) in weight.iter_mut().enumerate() {
        if (active >> l) & 1 == 0 {
            // An inactive lane is a branch that does not exist. It is carried
            // as a zero weight, not as a special case: `Cyc::ZERO` absorbs
            // every multiply and folds to nothing.
            *w = Cyc::ZERO;
        }
    }
    let block_len = cfg.merge_every.max(1);
    let mut site = 0usize;
    let mut since = 0usize;
    for &g in gates {
        match g {
            Gate::T(q) | Gate::Tdg(q) => {
                let (ci, cz) = t_coeffs(matches!(g, Gate::Tdg(_)));
                let zm = t_site_mask(site, block);
                for (l, w) in weight.iter_mut().enumerate() {
                    *w = w.mul(if (zm >> l) & 1 == 1 { cz } else { ci });
                }
                st.z(q, zm);
                site += 1;
                since += 1;
                if since >= block_len {
                    st.canonicalize_into(&mut weight);
                    since = 0;
                }
            }
            other => st.apply(other),
        }
    }
    st.canonicalize_into(&mut weight);
    SlicedBlock { state: st, weight }
}

/// Build every block, serially. The honest baseline for a per-branch
/// throughput number — the thread costs are charged to [`build_sharded`],
/// where they are actually paid.
pub fn build(n: usize, gates: &[Gate], cfg: &SlicedConfig) -> SlicedSum {
    build_sharded(n, gates, cfg, 1)
}

/// Build every block, across `shards` OS threads.
///
/// Deterministic by construction: blocks are independent by the branch
/// indexing, the cut is `mesh::shard_ranges` (a pure function of block count
/// and shard count), and each worker writes disjoint slots. The schedule is
/// not an input.
pub fn build_sharded(n: usize, gates: &[Gate], cfg: &SlicedConfig, shards: usize) -> SlicedSum {
    let t = gates.iter().filter(|g| g.is_t()).count();
    assert!(t < 63, "branch index is a u64: t = {t} has no block numbering");
    let nb = block_count(t);
    assert!(
        nb as usize <= cfg.max_blocks,
        "branch-sliced block count {nb} would exceed max_blocks ({}) at t = {t}",
        cfg.max_blocks
    );
    let mut blocks: Vec<Option<SlicedBlock>> = (0..nb).map(|_| None).collect();
    let ranges = crate::mesh::shard_ranges(nb, shards);
    if ranges.len() <= 1 {
        for (b, slot) in blocks.iter_mut().enumerate() {
            *slot = Some(run_block(n, gates, b as u64, t, cfg));
        }
    } else {
        let mut rest = blocks.as_mut_slice();
        let mut chunks: Vec<(u64, &mut [Option<SlicedBlock>])> = Vec::new();
        for r in &ranges {
            let take = (r.end - r.start) as usize;
            let (head, tail) = rest.split_at_mut(take);
            chunks.push((r.start, head));
            rest = tail;
        }
        std::thread::scope(|scope| {
            for (start, chunk) in chunks {
                scope.spawn(move || {
                    for (i, slot) in chunk.iter_mut().enumerate() {
                        *slot = Some(run_block(n, gates, start + i as u64, t, cfg));
                    }
                });
            }
        });
    }
    let blocks: Vec<SlicedBlock> = blocks.into_iter().map(|b| b.expect("every block built")).collect();
    let annihilated_lanes = blocks
        .iter()
        .map(|b| b.state.zero_mask().count_ones() as u64)
        .sum();
    SlicedSum {
        n,
        stats: SlicedStats {
            t_count: t,
            n_blocks: nb,
            branches: if t >= 127 { u128::MAX } else { 1u128 << t },
            annihilated_lanes,
        },
        blocks,
    }
}

/// The branch-sliced production path: build the blocks, then fold them across
/// the mesh through the one merge law.
///
/// `shards` is passed to BOTH tiers of the recursion — the block build and the
/// block fold — because a shard is a child holon in the same sense at each.
pub fn amplitude(n: usize, gates: &[Gate], y: &[bool], shards: usize) -> Cyc {
    let sum = build_sharded(n, gates, &SlicedConfig::default(), shards);
    crate::mesh::fold_amplitude(&Blocks(&sum), y, shards)
}

/// Exact amplitude of ONE branch through a scalar `Affine`, under this
/// module's branch indexing and its canonicalization schedule.
///
/// This is the referee the conformance test compares each lane against: it is
/// the per-branch path doing exactly what the sliced path claims to be doing
/// 64 at a time, so an equality between them is a statement about the SLICING
/// and about nothing else.
pub fn scalar_branch_amplitude(
    n: usize,
    gates: &[Gate],
    branch: u64,
    y: &[bool],
    cfg: &SlicedConfig,
) -> Cyc {
    let mut st = crate::affine::Affine::new(n);
    let mut w = Cyc::ONE;
    let block_len = cfg.merge_every.max(1);
    let mut site = 0usize;
    let mut since = 0usize;
    for &g in gates {
        match g {
            Gate::T(q) | Gate::Tdg(q) => {
                let (ci, cz) = t_coeffs(matches!(g, Gate::Tdg(_)));
                if (branch >> site) & 1 == 1 {
                    w = w.mul(cz);
                    st.z(q);
                } else {
                    w = w.mul(ci);
                }
                site += 1;
                since += 1;
                if since >= block_len {
                    w = w.mul(st.canonicalize());
                    since = 0;
                }
            }
            other => st.apply(other),
        }
    }
    w = w.mul(st.canonicalize());
    w.mul(st.amplitude(y))
}
