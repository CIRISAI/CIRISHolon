//! ADAPTIVE AT SCALE — mid-circuit measurement on the transposed engine.
//!
//! `adaptive.rs` pays the completion debt; this file makes it reach. The
//! measurement that motivated it (`examples/bigscale.rs`, banked in
//! BENCHMARKS): `adaptive.rs` runs on `PackedTableau`, the row-major
//! reference, whose gate path is 145–484× slower per gate than the
//! column-major engine that won the stim bake-off — 343× at n = 131072. A
//! distance-221 surface code round is ~244k gate applications, so the
//! reference alone puts the flagship out of reach by four orders of
//! magnitude. Nothing here is a new algorithm; it is Aaronson–Gottesman on
//! the layout that suits each half of the work.
//!
//! THE DIVISION OF LABOUR, and why it is exactly this:
//!
//! * GATES are per-qubit, so they want COLUMN-major: one column is
//!   `2n/64` contiguous words. They run on `ColTableau`.
//! * The DESTABILIZER PRODUCT is per-row, so it wants ROW-major: a row is
//!   `n/64` contiguous words. It runs on `PackedTableau`, the certified
//!   reference, unchanged.
//! * The DETERMINISM SCAN — "does any stabilizer anticommute with Z_q?" —
//!   is a COLUMN question that the row-major reference answers with `n`
//!   scalar bit-gets across `2n` separate heap allocations. Measured, that
//!   loop is ~99.5% of a deterministic measurement at n≈98k (2.2 ms of
//!   2.2 ms, with the product itself a measured mean of 1.7 rowsums). It
//!   moves to `ColTableau::first_x_row_in`, a contiguous scan with an early
//!   exit.
//!
//! So a round is: gates on the column engine, then the measurement batch
//! with its scans column-side and its rowsums row-side. The transpose is
//! LAZY — see the next paragraph — so the common round does none at all.
//!
//! THE SINGLE-TERM SHORTCUT, which is where the speed actually comes from.
//! A deterministic outcome is the sign of `∏_{i∈H} stab_{i+n}`, H the
//! destabilizer hit set. Profiled on a real surface code (`examples/
//! measure_profile.rs`), |H| is 1 in the steady state — mean 1.8 over a whole
//! run, max 5 — and a product of ONE row is just that row's SIGN BIT, which
//! the column engine already holds. So the common measurement reads one bit,
//! touches no row, and needs no row-major tableau: at d=101 that is 0.9 us
//! of scan against the 0.6 s per-round transpose it skips. The reference is
//! materialized only when a measurement genuinely needs rows (|H| >= 2, or a
//! random outcome's cascade), and a run that never needs one never allocates
//! it — halving the working set as well.
//!
//! WHY DEFERRING THE RESETS IS EXACT, not a convenience: a reset is
//! `X` on the measured qubit, and the reference interleaves it
//! (measure a₁, reset a₁, measure a₂, …). Here every measurement in the
//! batch happens first and the resets follow. A Pauli on qubit a₁ and a
//! projective measurement on a disjoint qubit a₂ are operations on disjoint
//! tensor factors, so they commute — outcomes, post-state and the order in
//! which the seeded stream is consumed are all unchanged. The conformance
//! gate checks exactly that, against the reference, tableau and all.
//!
//! STALENESS, stated honestly: a RANDOM outcome collapses the state through
//! the row-major tableau, which leaves the column engine's mirror stale for
//! the rest of the batch. Rather than patch it per collapse, the engine
//! FALLS BACK to the reference's own row-major scan for the remainder of
//! that batch and rebuilds once at the end. Deterministic measurements are
//! read-only, so a batch with no coins never falls back — which is every
//! round after the first in a QEC memory experiment, and the ones that
//! dominate. `scan_fast` / `scan_fallback` count it so the split is
//! reported, never assumed.

use crate::coltableau::ColTableau;
use crate::tableau::{PackedTableau, PauliRow};

/// Where a measurement's determinism question was answered.
#[derive(Default, Clone, Copy, Debug)]
pub struct ScanStats {
    /// Answered by the column scan (contiguous words).
    pub scan_fast: u64,
    /// Answered by the row-major fallback, after a collapse went stale.
    pub scan_fallback: u64,
    /// Outcomes forced by the state.
    pub deterministic: u64,
    /// Outcomes that were fair coins.
    pub random: u64,
    /// Total terms summed in destabilizer products (the O(1)-in-n part).
    pub product_terms: u64,
    /// Deterministic measurements answered from a single sign bit, with no
    /// row read and so no transpose.
    pub single_term: u64,
    /// How many times the row-major reference had to be materialized.
    pub transposes: u64,
    /// Rows actually updated by collapse cascades.
    pub cascade_terms: u64,
    /// Total X-weight of the pivot rows those cascades multiplied by — the
    /// quantity that decides whether the column mirror can be PATCHED
    /// through a collapse instead of abandoned.
    pub pivot_weight: u64,
    /// Largest single pivot X-weight seen.
    pub pivot_weight_max: u64,
    /// Collapses after which the X mirror was PATCHED and stayed usable.
    pub mirror_patched: u64,
    /// Collapses after which the patch was too expensive and the mirror was
    /// dropped, sending the rest of the batch to the row-major scan.
    pub mirror_dropped: u64,
}

pub struct ColAdaptive {
    pub n: usize,
    col: ColTableau,
    /// The row-major reference — allocated LAZILY, on the first measurement
    /// that needs a row, and then refilled in place. A run whose batches are
    /// all single-term never allocates it at all, which halves the working
    /// set as well as skipping the transpose.
    packed: Option<PackedTableau>,
    /// Do `packed`'s contents match the current state?
    packed_valid: bool,
    /// Is a measurement batch open?
    in_batch: bool,
    /// Is `col`'s X PLANE a faithful mirror? Enough for every scan.
    mirror_x_valid: bool,
    /// Is ALL of `col` (X, Z and signs) faithful? Required by the
    /// single-term sign shortcut, which reads a sign bit directly.
    mirror_full_valid: bool,
    /// Did anything in this batch modify `packed`?
    dirty: bool,
    rng: u64,
    pub seed: u64,
    pub stats: ScanStats,
}

fn splitmix(state: &mut u64) -> bool {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    (z ^ (z >> 31)) & 1 == 1
}

impl ColAdaptive {
    pub fn new(n: usize, seed: u64) -> Self {
        ColAdaptive {
            n,
            col: ColTableau::new(n),
            packed: None,
            packed_valid: false,
            in_batch: false,
            mirror_x_valid: false,
            mirror_full_valid: false,
            dirty: false,
            rng: seed,
            seed,
            stats: ScanStats::default(),
        }
    }

    #[inline]
    fn assert_unitary_phase(&self) {
        debug_assert!(!self.in_batch, "gates belong outside a measurement batch");
    }

    pub fn h(&mut self, q: usize) {
        self.assert_unitary_phase();
        self.col.h(q);
        self.packed_valid = false;
    }
    pub fn s(&mut self, q: usize) {
        self.assert_unitary_phase();
        self.col.s(q);
        self.packed_valid = false;
    }
    pub fn sdg(&mut self, q: usize) {
        self.assert_unitary_phase();
        self.col.sdg(q);
        self.packed_valid = false;
    }
    pub fn x_gate(&mut self, q: usize) {
        self.assert_unitary_phase();
        self.col.x_gate(q);
        self.packed_valid = false;
    }
    pub fn z_gate(&mut self, q: usize) {
        self.assert_unitary_phase();
        self.col.z_gate(q);
        self.packed_valid = false;
    }
    pub fn cx(&mut self, c: usize, t: usize) {
        self.assert_unitary_phase();
        self.col.cx(c, t);
        self.packed_valid = false;
    }

    /// Enter a measurement batch.
    ///
    /// This does NOT transpose. The row-major reference is materialized
    /// LAZILY, on the first measurement that actually needs a row — see
    /// `measure`. A steady-state QEC round needs none, and at d=101 the
    /// transpose it skips is 0.6 s against 0.03 s of measuring.
    pub fn begin_batch(&mut self) {
        assert!(!self.in_batch, "batch already open");
        self.in_batch = true;
        self.mirror_x_valid = true;
        self.mirror_full_valid = true;
        self.dirty = false;
    }

    /// Leave a measurement batch: one transpose back, only if a collapse
    /// actually moved the reference out from under the column engine.
    pub fn end_batch(&mut self) {
        assert!(self.in_batch, "no batch open");
        if self.dirty {
            let packed = self.packed.as_ref().expect("dirty without a reference");
            self.col.load_from_packed(packed);
            self.mirror_x_valid = true;
            self.mirror_full_valid = true;
            self.dirty = false;
        }
        self.in_batch = false;
    }

    /// Materialize the row-major reference if it is not already current.
    /// Only ever called from inside a batch, and only when `col` is still
    /// authoritative (a collapse implies the reference already exists).
    fn ensure_packed(&mut self) {
        if self.packed_valid {
            return;
        }
        debug_assert!(!self.dirty, "reference stale while it is authoritative");
        let n = self.n;
        let buf = self
            .packed
            .get_or_insert_with(|| PackedTableau::new(n));
        self.col.store_to_packed(buf);
        self.packed_valid = true;
        self.stats.transposes += 1;
    }

    /// Measure qubit `q` in the computational basis, inside an open batch.
    /// Returns `(outcome, was_deterministic)`.
    pub fn measure(&mut self, q: usize) -> (bool, bool) {
        assert!(self.in_batch, "measure outside a batch");
        let n = self.n;

        // ---- the determinism question, column-side where it is cheap ----
        let pivot = if self.mirror_x_valid {
            self.stats.scan_fast += 1;
            self.col.first_x_row_in(q, n, 2 * n)
        } else {
            self.stats.scan_fallback += 1;
            let packed = self.packed.as_ref().expect("fallback without a reference");
            (n..2 * n).find(|&p| packed.rows[p].x.get(q))
        };

        // ---- THE O(1) CASE, and the common one ----
        //
        // A deterministic outcome is the sign of `∏_{i∈H} stab_{i+n}` where H
        // is the destabilizer hit set. Measured on a surface code, |H| is 1
        // in the steady state (and mean 1.8 overall, max 5) — and a product
        // of ONE row is just that row's sign, which the column engine already
        // holds as a single bit. No row is read, so no transpose is needed
        // and the reference is never materialized. |H| = 0 is the empty
        // product, +1.
        if pivot.is_none() && self.mirror_full_valid {
            let mut hits = Vec::new();
            self.col.x_rows_in(q, 0, n, &mut hits);
            if hits.len() <= 1 {
                self.stats.deterministic += 1;
                self.stats.product_terms += hits.len() as u64;
                self.stats.single_term += 1;
                let out = match hits.first() {
                    None => false,
                    Some(&i) => self.col.sign_bit(i + n),
                };
                return (out, true);
            }
        }

        // Everything below needs actual rows.
        self.ensure_packed();
        let packed = self.packed.as_mut().expect("reference materialized");

        match pivot {
            // ---- RANDOM: a fair coin, then the rowsum cascade ----
            //
            // The cascade is driven by the COLUMN when the mirror can supply
            // it: the set of rows to update is literally column `q` of the X
            // plane, so the reference's `for i in 0..2n { if x[i][q] }` scan —
            // 2n cache-missing bit-gets — becomes a read of `2n/64`
            // contiguous words. The mirror is then PATCHED rather than
            // abandoned, which is what keeps the rest of the batch fast.
            Some(p) => {
                let outcome = splitmix(&mut self.rng);
                let pivot_row = packed.rows[p].clone();
                let pw = pivot_row.x.popcount() as u64;
                self.stats.pivot_weight += pw;
                self.stats.pivot_weight_max = self.stats.pivot_weight_max.max(pw);

                // The update mask: column q over all 2n rows, minus the pivot.
                let mask: Option<Vec<u64>> = if self.mirror_x_valid {
                    let mut m = self.col.x_column(q).to_vec();
                    m[p >> 6] &= !(1u64 << (p & 63));
                    Some(m)
                } else {
                    None
                };

                let mut cterms = 0u64;
                match &mask {
                    Some(m) => {
                        for (w, &word) in m.iter().enumerate() {
                            let mut bits = word;
                            while bits != 0 {
                                let i = w * 64 + bits.trailing_zeros() as usize;
                                bits &= bits - 1;
                                packed.rows[i].mul_assign(&pivot_row);
                                cterms += 1;
                            }
                        }
                    }
                    None => {
                        for i in 0..2 * n {
                            if i != p && packed.rows[i].x.get(q) {
                                packed.rows[i].mul_assign(&pivot_row);
                                cterms += 1;
                            }
                        }
                    }
                }
                self.stats.cascade_terms += cterms;

                let old_destab_x = packed.rows[p - n].x.clone();
                packed.rows[p - n] = pivot_row.clone();
                let mut fresh = PauliRow::identity(n);
                fresh.z.set(q, true);
                fresh.r = if outcome { 2 } else { 0 };
                packed.rows[p] = fresh;

                // ---- patch the X mirror, or give up on it honestly ----
                //
                // Step 1 (the cascade): column c changes by `mask` exactly
                // where the pivot's X part is set.
                // Step 2 (row p−n := pivot): its X bits must become the
                // pivot's. After step 1 the bit holds `old ^ (pivot if p−n was
                // in the mask)`, so the flip set is `old` when it was in the
                // mask and `old ^ pivot` when it was not.
                // Step 3 (row p := Z_q): its X bits must become zero, and
                // after step 1 they still hold the pivot's, so flip exactly
                // the pivot's support.
                //
                // The whole patch is bounded by the pivot's X-weight plus the
                // old destabilizer's — measured at mean 1.5 and max 2 for the
                // pivot on a surface code. Past `PATCH_BUDGET` columns the
                // patch would cost more than the scan it saves, so the mirror
                // is dropped instead and the batch says so in `scan_fallback`.
                const PATCH_BUDGET: u32 = 256;
                // `old_destab_x` is read AFTER the cascade and BEFORE the
                // overwrite, so it already equals what the mirror's bit
                // (p−n) holds once step 1 has run — the cascade's effect on
                // that row is baked in on both sides. The flip set is
                // therefore just "what it is" XOR "what it must become",
                // with no separate correction for whether p−n was in the
                // mask. Applying one anyway double-counts, which is the bug
                // the bit-identity gate caught.
                let mut flip2 = old_destab_x;
                flip2.xor_assign(&pivot_row.x);
                let patch_cost = pivot_row.x.popcount() * 2 + flip2.popcount();

                match mask {
                    Some(m) if patch_cost <= PATCH_BUDGET => {
                        // Set bits only — walking all n columns here would
                        // reintroduce the O(n) per measurement this exists to
                        // remove.
                        for (w, &word) in pivot_row.x.words.iter().enumerate() {
                            let mut bits = word;
                            while bits != 0 {
                                let c = w * 64 + bits.trailing_zeros() as usize;
                                bits &= bits - 1;
                                self.col.xor_into_x_column(c, &m);
                                self.col.flip_x_bit(c, p);
                            }
                        }
                        for (w, &word) in flip2.words.iter().enumerate() {
                            let mut bits = word;
                            while bits != 0 {
                                let c = w * 64 + bits.trailing_zeros() as usize;
                                bits &= bits - 1;
                                self.col.flip_x_bit(c, p - n);
                            }
                        }
                        self.stats.mirror_patched += 1;
                        self.mirror_x_valid = true;
                    }
                    _ => {
                        self.mirror_x_valid = false;
                        self.stats.mirror_dropped += 1;
                    }
                }
                // Z and signs are NOT patched, so the sign shortcut stands
                // down until the next full rebuild.
                self.mirror_full_valid = false;
                self.dirty = true;
                self.stats.random += 1;
                (outcome, false)
            }
            // ---- DETERMINISTIC: read-only, so the mirror survives ----
            None => {
                let mut scratch = PauliRow::identity(n);
                let mut terms = 0u64;
                if self.mirror_x_valid {
                    let mut hits = Vec::new();
                    self.col.x_rows_in(q, 0, n, &mut hits);
                    for i in hits {
                        // `hits` indexes destabilizer rows; the product runs
                        // over the MATCHING stabilizers.
                        scratch.mul_assign(&packed.rows[i + n]);
                        terms += 1;
                    }
                } else {
                    for i in 0..n {
                        if packed.rows[i].x.get(q) {
                            scratch.mul_assign(&packed.rows[i + n]);
                            terms += 1;
                        }
                    }
                }
                self.stats.deterministic += 1;
                self.stats.product_terms += terms;
                (scratch.r % 4 == 2, true)
            }
        }
    }

    /// Read-only peek at the current state as the certified reference type.
    /// Only valid outside a batch.
    pub fn to_packed(&self) -> PackedTableau {
        debug_assert!(!self.in_batch, "to_packed inside an open batch");
        self.col.to_packed()
    }

    pub fn col(&self) -> &ColTableau {
        &self.col
    }

    /// Has the row-major reference been allocated at all? A run that never
    /// needs a row never pays for one.
    pub fn reference_allocated(&self) -> bool {
        self.packed.is_some()
    }

    /// Set the state from a reference tableau — for tests and profiling that
    /// need two engines to start from the same place.
    pub fn load_state(&mut self, p: &PackedTableau) {
        assert!(!self.in_batch, "load_state inside an open batch");
        self.col.load_from_packed(p);
        self.packed_valid = false;
    }

    /// The value of the Pauli-Z STRING `∏_q Z_q`, if the state determines it.
    ///
    /// This is the logical observable of a QEC memory experiment: the code's
    /// individual data qubits are each a fair coin, while the logical operator
    /// across them is forced — so checking it is the difference between "the
    /// syndromes look right" and "the encoded bit survived". Same
    /// Aaronson–Gottesman rule as a single-qubit measurement, with
    /// anticommutation read off the symplectic product: an all-Z observable
    /// anticommutes with a row exactly when that row's X-support meets the
    /// string an ODD number of times.
    ///
    /// `None` if the observable is not determined (the state is in a
    /// superposition of its eigenvalues).
    pub fn z_string_value(&self, qubits: &[usize]) -> Option<bool> {
        debug_assert!(!self.in_batch, "z_string_value inside an open batch");
        let packed = self.col.to_packed();
        z_string_value_of(&packed, qubits)
    }
}

/// `z_string_value` against an explicit reference tableau.
pub fn z_string_value_of(packed: &PackedTableau, qubits: &[usize]) -> Option<bool> {
    let n = packed.n;
    let anti = |row: &PauliRow| -> bool {
        let mut parity = false;
        for &q in qubits {
            parity ^= row.x.get(q);
        }
        parity
    };
    // Any stabilizer anticommuting ⇒ the observable is not determined.
    for p in n..2 * n {
        if anti(&packed.rows[p]) {
            return None;
        }
    }
    let mut scratch = PauliRow::identity(n);
    for i in 0..n {
        if anti(&packed.rows[i]) {
            scratch.mul_assign(&packed.rows[i + n]);
        }
    }
    Some(scratch.r % 4 == 2)
}

#[cfg(test)]
mod conformance {
    use super::*;
    use crate::adaptive::{self, Step};
    use crate::affine::Gate;

    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 11
        }
        fn below(&mut self, n: usize) -> usize {
            (self.next() % n as u64) as usize
        }
    }

    /// THE PROOF OBLIGATION. A random adaptive program — gates, then a
    /// measurement batch, repeated — must produce from this engine the SAME
    /// outcomes in the same order AND a bit-identical final tableau as the
    /// certified row-major reference running the interleaved form. This is
    /// what licenses the deferred resets, the column scan, and the fallback.
    #[test]
    fn matches_the_row_major_reference_exactly() {
        for n in [4usize, 9, 17, 40, 65, 130] {
            for seed in 0..6u64 {
                let mut rng = Rng(0xADAF_0000 ^ (n as u64) << 8 ^ seed);
                let mut ours = ColAdaptive::new(n, seed);
                let mut refr = PackedTableau::new(n);
                let mut ref_rng = seed;
                let mut ours_out = Vec::new();
                let mut ref_out = Vec::new();

                for _round in 0..4 {
                    // ---- gate phase, identical on both ----
                    for _ in 0..(6 * n) {
                        let q = rng.below(n);
                        let mut q2 = rng.below(n);
                        while q2 == q {
                            q2 = rng.below(n);
                        }
                        match rng.below(6) {
                            0 => {
                                ours.h(q);
                                refr.h(q);
                            }
                            1 => {
                                ours.s(q);
                                refr.s(q);
                            }
                            2 => {
                                ours.sdg(q);
                                refr.sdg(q);
                            }
                            3 => {
                                ours.x_gate(q);
                                refr.x_gate(q);
                            }
                            4 => {
                                ours.z_gate(q);
                                refr.z_gate(q);
                            }
                            _ => {
                                ours.cx(q, q2);
                                refr.cx(q, q2);
                            }
                        }
                    }

                    // ---- measurement batch on a subset of qubits ----
                    let targets: Vec<usize> = (0..n).step_by(2).collect();

                    ours.begin_batch();
                    let mut ours_round = Vec::new();
                    for &q in &targets {
                        ours_round.push(ours.measure(q).0);
                    }
                    ours.end_batch();
                    // deferred resets
                    for (k, &q) in targets.iter().enumerate() {
                        if ours_round[k] {
                            ours.x_gate(q);
                        }
                    }
                    ours_out.extend_from_slice(&ours_round);

                    // reference: interleaved measure-then-reset
                    for &q in &targets {
                        let outcome = match refr.measure_peek(q) {
                            Some(b) => b,
                            None => {
                                let b = super::splitmix(&mut ref_rng);
                                refr.collapse(q, b);
                                b
                            }
                        };
                        if outcome {
                            refr.x_gate(q);
                        }
                        ref_out.push(outcome);
                    }
                }

                assert_eq!(
                    ours_out, ref_out,
                    "n={n} seed={seed}: outcome streams differ"
                );
                let got = ours.to_packed();
                for (i, (a, b)) in got.rows.iter().zip(&refr.rows).enumerate() {
                    assert_eq!(a.x, b.x, "n={n} seed={seed} row {i}: X plane differs");
                    assert_eq!(a.z, b.z, "n={n} seed={seed} row {i}: Z plane differs");
                    assert_eq!(a.r, b.r, "n={n} seed={seed} row {i}: sign differs");
                }
            }
        }
    }

    /// The column scan must be EXERCISED, not merely available: if every
    /// measurement fell back to row-major the test above would still pass
    /// while the optimization did nothing. Assert the fast path fires.
    #[test]
    fn the_fast_scan_actually_carries_the_deterministic_traffic() {
        // A GHZ-style state: measure one qubit (a coin), then the rest are
        // forced — the shape a QEC round has.
        let n = 64;
        let mut a = ColAdaptive::new(n, 7);
        a.h(0);
        for q in 1..n {
            a.cx(0, q);
        }
        a.begin_batch();
        let mut outs = Vec::new();
        for q in 0..n {
            outs.push(a.measure(q));
        }
        a.end_batch();

        assert_eq!(outs[0].1, false, "first GHZ measurement must be a coin");
        for (q, &(o, det)) in outs.iter().enumerate().skip(1) {
            assert!(det, "qubit {q} must be forced after the first collapse");
            assert_eq!(o, outs[0].0, "GHZ outcomes must all agree");
        }
        assert!(a.stats.scan_fast >= 1, "the column scan never ran");
        assert_eq!(a.stats.random, 1, "exactly one coin in a GHZ state");
        assert_eq!(a.stats.deterministic, (n - 1) as u64);
    }

    /// A batch with NO coins must never fall back — that is the claim the
    /// whole design rests on for rounds two and beyond.
    #[test]
    fn a_deterministic_batch_never_falls_back() {
        let n = 48;
        let mut a = ColAdaptive::new(n, 3);
        // |0…0⟩ with some sign flips: every Z measurement is forced.
        for q in (0..n).step_by(3) {
            a.x_gate(q);
        }
        a.begin_batch();
        for q in 0..n {
            let (o, det) = a.measure(q);
            assert!(det, "qubit {q} must be deterministic");
            assert_eq!(o, q % 3 == 0, "qubit {q} outcome");
        }
        a.end_batch();
        assert_eq!(a.stats.scan_fallback, 0, "a coin-free batch fell back");
        assert_eq!(a.stats.scan_fast, n as u64);
        // And the whole point: no row was read, so no transpose happened and
        // the row-major reference was never even allocated.
        assert_eq!(a.stats.single_term, n as u64, "the O(1) path must carry all of it");
        assert_eq!(a.stats.transposes, 0, "a single-term batch must not transpose");
        assert!(!a.reference_allocated(), "a single-term batch must not allocate the reference");
    }

    /// The single-term shortcut must agree with the general product on states
    /// where |H| VARIES — including |H| = 0, 1 and ≥ 2 — or it is a fast wrong
    /// answer. Cross-checked against the row-major reference directly.
    #[test]
    fn single_term_shortcut_agrees_with_the_general_product() {
        let mut rng = Rng(0x51E1_7E12);
        let mut saw_multi = false;
        let mut saw_single = false;
        for n in [6usize, 12, 33, 64, 70] {
            for seed in 0..8u64 {
                let mut ours = ColAdaptive::new(n, seed);
                let mut refr = PackedTableau::new(n);
                // A CNOT-and-phase circuit with NO H on some qubits keeps many
                // measurements deterministic, which is the regime under test.
                for _ in 0..(4 * n) {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    match rng.below(5) {
                        0 => {
                            ours.x_gate(q);
                            refr.x_gate(q);
                        }
                        1 => {
                            ours.z_gate(q);
                            refr.z_gate(q);
                        }
                        2 => {
                            ours.s(q);
                            refr.s(q);
                        }
                        3 if q % 3 == 0 => {
                            ours.h(q);
                            refr.h(q);
                        }
                        _ => {
                            ours.cx(q, q2);
                            refr.cx(q, q2);
                        }
                    }
                }
                ours.begin_batch();
                for q in 0..n {
                    let want = refr.measure_peek(q);
                    let (got, det) = ours.measure(q);
                    match want {
                        Some(b) => {
                            assert!(det, "n={n} seed={seed} q={q}: forced outcome read as a coin");
                            assert_eq!(got, b, "n={n} seed={seed} q={q}: wrong forced outcome");
                        }
                        None => {
                            assert!(!det, "n={n} seed={seed} q={q}: coin read as forced");
                            refr.collapse(q, got);
                        }
                    }
                }
                ours.end_batch();
                if ours.stats.single_term > 0 {
                    saw_single = true;
                }
                if ours.stats.deterministic > ours.stats.single_term {
                    saw_multi = true;
                }
            }
        }
        assert!(saw_single, "the single-term path never fired — test is vacuous");
        assert!(saw_multi, "the multi-term path never fired — test is vacuous");
    }

    /// The logical observable must read as an OBSERVABLE: forced where the
    /// state fixes it, undetermined where it does not, and it must track a
    /// Pauli flip on the string.
    #[test]
    fn z_string_reads_the_logical_observable() {
        // |0…0⟩: every Z string is +1 (false).
        let a = ColAdaptive::new(8, 1);
        assert_eq!(a.z_string_value(&[0, 1, 2, 3]), Some(false));

        // One X flips the string's parity; two X's inside it restore it.
        let mut b = ColAdaptive::new(8, 1);
        b.x_gate(2);
        assert_eq!(b.z_string_value(&[0, 1, 2, 3]), Some(true));
        b.x_gate(1);
        assert_eq!(b.z_string_value(&[0, 1, 2, 3]), Some(false));
        // An X OUTSIDE the string does not move it.
        b.x_gate(6);
        assert_eq!(b.z_string_value(&[0, 1, 2, 3]), Some(false));

        // GHZ on 0..4: each qubit alone is a coin, the full parity is forced.
        let mut c = ColAdaptive::new(8, 1);
        c.h(0);
        for q in 1..4 {
            c.cx(0, q);
        }
        assert_eq!(c.z_string_value(&[0]), None, "a GHZ qubit alone is a coin");
        assert_eq!(
            c.z_string_value(&[0, 1, 2, 3]),
            Some(false),
            "the GHZ parity is forced"
        );
    }

    /// The seeded stream is still a stream: same seed replays, different
    /// seeds diverge — the contract `adaptive.rs` established.
    #[test]
    fn replayable_and_seed_sensitive() {
        let run = |seed: u64| {
            let n = 24;
            let mut a = ColAdaptive::new(n, seed);
            for q in 0..n {
                a.h(q);
            }
            a.begin_batch();
            let v: Vec<bool> = (0..n).map(|q| a.measure(q).0).collect();
            a.end_batch();
            v
        };
        assert_eq!(run(11), run(11), "same seed must replay");
        let set: std::collections::HashSet<Vec<bool>> = (0..16u64).map(run).collect();
        assert!(set.len() > 1, "different seeds must diverge");
    }

    /// Teleportation, ported: the strongest single check that measurement,
    /// feed-forward and the classical register all behave together.
    #[test]
    fn teleportation_works_on_the_column_engine() {
        for seed in 0..32u64 {
            let mut a = ColAdaptive::new(3, seed);
            a.h(0);
            a.h(1);
            a.cx(1, 2);
            a.cx(0, 1);
            a.h(0);
            a.begin_batch();
            let m0 = a.measure(0).0;
            let m1 = a.measure(1).0;
            a.end_batch();
            if m1 {
                a.x_gate(2);
            }
            if m0 {
                a.z_gate(2);
            }
            // q2 must be |+⟩: X-basis measurement is forced to 0.
            a.h(2);
            let mut t = a.to_packed();
            assert_eq!(
                t.measure_peek(2),
                Some(false),
                "seed {seed}: teleported state is not |+⟩"
            );
            let _ = &mut t;
        }
    }

    /// THE MIRROR PATCH'S OWN INVARIANT, gated directly.
    ///
    /// After every collapse that reports `mirror_x_valid`, the column
    /// engine's X plane must equal the reference's X plane BIT FOR BIT —
    /// mid-batch, not merely after the end-of-batch rebuild papers over it.
    /// The downstream tests would catch a violation only when it happened to
    /// change an outcome; this catches it where it happens. (A first version
    /// of the patch applied a correction for whether row p−n was in the
    /// update mask when the value it corrected already had the cascade baked
    /// in — a double-count that survived until a gate looked here.)
    #[test]
    fn the_patched_mirror_stays_bit_identical_to_the_reference() {
        let mut rng = Rng(0x8817_6072);
        let mut collapses = 0u64;
        let mut checked = 0u64;
        for n in [5usize, 13, 32, 64, 96] {
            for seed in 0..5u64 {
                let mut a = ColAdaptive::new(n, seed);
                for _ in 0..(5 * n) {
                    let q = rng.below(n);
                    let mut q2 = rng.below(n);
                    while q2 == q {
                        q2 = rng.below(n);
                    }
                    match rng.below(4) {
                        0 => a.h(q),
                        1 => a.s(q),
                        2 => a.x_gate(q),
                        _ => a.cx(q, q2),
                    }
                }
                a.begin_batch();
                for q in 0..n {
                    let (_o, det) = a.measure(q);
                    if !det {
                        collapses += 1;
                    }
                    // Only comparable when a reference actually exists — a
                    // batch of single-term measurements never builds one, and
                    // there is nothing to diverge from.
                    if a.mirror_x_valid && a.packed_valid {
                        let packed = a.packed.as_ref().expect("packed_valid implies present");
                        checked += 1;
                        for c in 0..n {
                            let col = a.col.x_column(c);
                            for row in 0..2 * n {
                                let mirror = col[row >> 6] >> (row & 63) & 1 == 1;
                                assert_eq!(
                                    mirror,
                                    packed.rows[row].x.get(c),
                                    "n={n} seed={seed} after measuring {q}: \
                                     mirror X[{row}][{c}] diverged"
                                );
                            }
                        }
                    }
                }
                a.end_batch();
            }
        }
        assert!(collapses > 0, "no collapse happened — the test is vacuous");
        assert!(checked > 0, "the mirror was never compared — the test is vacuous");
    }

    /// RESET SEMANTICS, ported: a reset is a real reset. After measure-and-
    /// correct the qubit reads 0 with certainty, whatever state it was in.
    /// (`adaptive.rs::reset_returns_the_qubit_to_zero`, on the col path.)
    #[test]
    fn reset_returns_the_qubit_to_zero_on_the_column_engine() {
        for seed in 0..8u64 {
            let mut a = ColAdaptive::new(2, seed);
            a.h(0);
            a.cx(0, 1);
            a.begin_batch();
            let (o, det) = a.measure(0);
            a.end_batch();
            assert!(!det, "seed {seed}: a Bell-pair qubit must be a coin");
            if o {
                a.x_gate(0);
            }
            assert_eq!(
                a.z_string_value(&[0]),
                Some(false),
                "seed {seed}: reset failed"
            );
        }
    }

    /// THE QEC-SHAPED WORKLOAD, ported: repeated syndrome extraction with
    /// feed-forward correction on a 3-qubit repetition code must return the
    /// data qubits to the codespace, every seed.
    /// (`adaptive.rs::repetition_code_syndrome_cycle_corrects`, on the col
    /// path — the test that most resembles the flagship.)
    #[test]
    fn repetition_code_syndrome_cycle_corrects_on_the_column_engine() {
        for seed in 0..16u64 {
            let mut a = ColAdaptive::new(5, seed);
            a.x_gate(1); // the error
            // syndrome 1: parity of data 0,1 into ancilla 3
            a.cx(0, 3);
            a.cx(1, 3);
            // syndrome 2: parity of data 1,2 into ancilla 4
            a.cx(1, 4);
            a.cx(2, 4);
            a.begin_batch();
            let s0 = a.measure(3).0;
            let s1 = a.measure(4).0;
            a.end_batch();
            assert!(s0 && s1, "seed {seed}: both syndromes must fire");
            // both syndromes flag ⇒ the error is on qubit 1
            if s0 && s1 {
                a.x_gate(1);
            }
            for q in 0..3 {
                assert_eq!(
                    a.z_string_value(&[q]),
                    Some(false),
                    "seed {seed}: data qubit {q} not corrected"
                );
            }
        }
    }

    /// And the reference's own adaptive runner must agree with us on the
    /// program it was written for — the two engines meeting on `adaptive.rs`'s
    /// home ground.
    #[test]
    fn agrees_with_adaptive_run_on_teleportation() {
        for seed in 0..16u64 {
            let prog = vec![
                Step::Gate(Gate::H(0)),
                Step::Gate(Gate::H(1)),
                Step::Gate(Gate::Cx(1, 2)),
                Step::Gate(Gate::Cx(0, 1)),
                Step::Gate(Gate::H(0)),
                Step::Measure { q: 0, c: 0 },
                Step::Measure { q: 1, c: 1 },
            ];
            let r = adaptive::run(3, 2, &prog, seed);

            let mut a = ColAdaptive::new(3, seed);
            a.h(0);
            a.h(1);
            a.cx(1, 2);
            a.cx(0, 1);
            a.h(0);
            a.begin_batch();
            let m0 = a.measure(0).0;
            let m1 = a.measure(1).0;
            a.end_batch();
            assert_eq!(vec![m0, m1], r.bits, "seed {seed}: outcomes differ");
        }
    }
}
