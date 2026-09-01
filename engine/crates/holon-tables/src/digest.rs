//! The certificate: an exact, order-free digest that convicts a corrupted shard.
//!
//! # This is the part the merge law actually covers
//!
//! `lean/CIRISHolon/MergeLaw.lean` proves its theorems over an `AddCommMonoid`, and
//! instantiates them at `Lanes = Fin 4 -> Z` with `laneDigest` reducing every lane mod `p`.
//! The theorems that matter here:
//!
//! * `shardedFold_invariant` — every sharding of a run folds to the same value, so shard
//!   count, shard sizes and schedule are all quotiented away;
//! * `digest_commutes` — a monoid homomorphism commutes with the whole sharded fold;
//! * `digest_convicts` — if a claimed merged result's digest disagrees with the fold of the
//!   per-shard digests, the claim is NOT the true merge. In an exact ring there is no
//!   tolerance, so there is no false alarm.
//!
//! The carrier here is `[u64; 4]` under **wrapping** addition, i.e. `(Z/2^64)^4`. That is
//! an additive commutative monoid, and two's-complement addition is associative and
//! commutative unconditionally — on overflow as much as off it, which is exactly the
//! argument `holon-gpu`'s header makes for taking the ring out of a device reduction. So
//! the fold's value does not depend on how it was cut, and that is a property of the
//! arithmetic rather than a measurement that came out lucky.
//!
//! # What it does NOT cover, said plainly
//!
//! The TABLE is `f64`. Float addition is not associative and the table is not a sum
//! anyway. Nothing in the Lean says anything about the table's contents; the contents are
//! shard-invariant because [`crate::grid`] makes each node's computation independent of its
//! shard, which is a construction, not a theorem. Conflating the two would be laundering an
//! integer theorem into a claim about floats.
//!
//! # Why a hash and not a sum of energies
//!
//! A plain sum of the energy bits would be a homomorphism too, and useless: two corruptions
//! could cancel, and a swap of two nodes' values would be invisible because addition does
//! not know which node it came from. Each node is therefore hashed WITH ITS INDEX into four
//! independent 64-bit lanes, and it is the lanes that are summed. A dropped node, a
//! duplicated node, a swapped pair, and a single flipped bit all move the sum.

use crate::node::NodeRecord;

/// Four exact lanes, summed with wrapping addition — the monoid the merge law is
/// instantiated at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Digest(pub [u64; 4]);

impl Digest {
    /// The monoid's identity, and the digest of an empty shard.
    pub const ZERO: Digest = Digest([0; 4]);

    /// The merge: lane-wise wrapping addition. Associative and commutative unconditionally,
    /// which is what makes [`Digest::fold`] independent of how the work was cut.
    #[inline]
    pub fn merge(self, other: Digest) -> Digest {
        let mut out = [0u64; 4];
        for i in 0..4 {
            out[i] = self.0[i].wrapping_add(other.0[i]);
        }
        Digest(out)
    }

    /// `shardedFold`: fold a run, or a shard, or a multiset of shard digests. One function
    /// for all three, because in the monoid they are the same operation — which is the
    /// content of `shardedFold_eq_sum`.
    pub fn fold<I: IntoIterator<Item = Digest>>(items: I) -> Digest {
        items.into_iter().fold(Digest::ZERO, Digest::merge)
    }

    /// The monoid element of a single node record.
    ///
    /// # What is in, and what is deliberately out
    ///
    /// IN: the node index — without it the digest could not tell a swap of two values from
    /// a no-op — the energy and its two derivatives, and the STATUS, so a VOIDed node can
    /// never be quietly replaced by a scored one.
    ///
    /// OUT: the iteration counts and the exit reason. They are in the RECORD, and they are
    /// not in the certificate, because they describe HOW the number was reached rather than
    /// WHAT the table contains. A certificate over "how" fires on any solver refactor that
    /// changes an iteration count while changing no physics — a committed table's
    /// certificate would break for a reason that is not a corruption, and a detector that
    /// cries wolf on benign change is one nobody keeps.
    ///
    /// (The tables lane proposed this split — index + value bit-exact, provenance carried
    /// but uncertified — and it is the more robust of the two. The stricter comparison is
    /// still available and still used: [`crate::GenOutcome::table_bytes`] covers every
    /// field including iterations, and the shard-invariance gate compares THAT, because
    /// under canonical chains the iteration counts are deterministic too and a difference
    /// there would be a real signal.)
    pub fn of_record(r: &NodeRecord) -> Digest {
        // Four independent lanes from four different salts. Splitmix64's finalizer is used
        // as the mixing function: it is a bijection on u64, so no field can be lost, and
        // avalanche means a one-bit change in any input moves about half the output bits.
        const SALT: [u64; 4] = [
            0x9e37_79b9_7f4a_7c15,
            0xbf58_476d_1ce4_e5b9,
            0x94d0_49bb_1331_11eb,
            0x2545_f491_4f6c_dd1d,
        ];
        let fields: [u64; 5] = [
            r.node as u64,
            r.energy_bits,
            r.d1_bits,
            r.d2_bits,
            r.status_code(),
        ];
        let mut lanes = [0u64; 4];
        for (l, lane) in lanes.iter_mut().enumerate() {
            let mut h = SALT[l];
            for (idx, f) in fields.iter().enumerate() {
                h ^= mix(f.wrapping_add(SALT[l]).wrapping_add(idx as u64));
                h = mix(h);
            }
            *lane = h;
        }
        Digest(lanes)
    }

    /// The digest of a whole table, in canonical node order. This is the "claimed merged
    /// result" of `digest_convicts`: computed from the ASSEMBLED artifact, independently of
    /// whatever the workers reported.
    pub fn of_table(records: &[NodeRecord]) -> Digest {
        Digest::fold(records.iter().map(Digest::of_record))
    }

    /// Hex, for a log line or a commit message.
    pub fn hex(&self) -> String {
        self.0
            .iter()
            .map(|l| format!("{l:016x}"))
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Splitmix64's finalizer — a bijection on `u64` with good avalanche.
#[inline]
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The verdict of comparing an assembled table against the workers' own reported digests.
///
/// `digest_convicts` in one type: a disagreement is a PROOF that the assembled table is not
/// the true merge of the shards. Exact arithmetic, no tolerance, so `Convicted` never
/// happens to a clean run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Certificate {
    /// The assembled table's digest equals the fold of the per-shard digests.
    Clean { digest: Digest },
    /// They disagree. The assembled table is not the merge of the shards that were run.
    Convicted {
        assembled: Digest,
        folded_shards: Digest,
    },
}

impl Certificate {
    /// Compare an assembled table against the fold of the shard digests.
    pub fn check(assembled: Digest, shard_digests: &[Digest]) -> Certificate {
        let folded = Digest::fold(shard_digests.iter().copied());
        if assembled == folded {
            Certificate::Clean { digest: assembled }
        } else {
            Certificate::Convicted {
                assembled,
                folded_shards: folded,
            }
        }
    }

    pub fn is_clean(&self) -> bool {
        matches!(self, Certificate::Clean { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{NodeRecord, NodeStatus};

    fn rec(node: u32, e: f64) -> NodeRecord {
        NodeRecord {
            node,
            energy_bits: e.to_bits(),
            d1_bits: (e * 0.5).to_bits(),
            d2_bits: (e * 0.25).to_bits(),
            davidson_iters: 12,
            cg_iters: 7,
            exit_code: 0,
            status: NodeStatus::Ok,
            warm: true,
            mirrored: false,
        }
    }

    /// The merge law at this carrier: any way of cutting the run folds to the same value.
    /// This is `shardedFold_invariant` as a test of the Rust instance.
    #[test]
    fn fold_is_independent_of_how_the_run_is_cut() {
        let records: Vec<NodeRecord> = (0..97).map(|i| rec(i, -1.0 - i as f64 * 0.01)).collect();
        let whole = Digest::of_table(&records);

        for cut in [1usize, 2, 3, 5, 8, 16, 97] {
            let shards: Vec<Digest> = records
                .chunks(records.len().div_ceil(cut))
                .map(|c| Digest::fold(c.iter().map(Digest::of_record)))
                .collect();
            assert_eq!(
                whole,
                Digest::fold(shards.iter().copied()),
                "folding in {cut} shards disagreed with the whole run"
            );
        }

        // And in a deliberately scrambled order, since the monoid is COMMUTATIVE and the
        // schedule is supposed to be quotiented away entirely.
        let mut shuffled = records.clone();
        shuffled.reverse();
        assert_eq!(
            whole,
            Digest::of_table(&shuffled),
            "the fold saw the order, so it is not the commutative monoid it claims to be"
        );
    }

    /// Zero false positives: a clean run is never convicted, however it was sharded.
    #[test]
    fn clean_runs_are_never_convicted() {
        let records: Vec<NodeRecord> = (0..64).map(|i| rec(i, -2.0 - i as f64 * 0.001)).collect();
        let assembled = Digest::of_table(&records);
        for cut in [1usize, 4, 7, 64] {
            let shards: Vec<Digest> = records
                .chunks(records.len().div_ceil(cut))
                .map(|c| Digest::fold(c.iter().map(Digest::of_record)))
                .collect();
            assert!(
                Certificate::check(assembled, &shards).is_clean(),
                "a clean {cut}-shard run was convicted"
            );
        }
    }

    /// The other half of the scoping decision: provenance-only changes must NOT move the
    /// digest.
    ///
    /// Asserted because it is a DELIBERATE property and deliberate properties rot. Without
    /// this test, putting the iteration counts back into `of_record` would silently
    /// re-couple every committed table's certificate to solver internals, and every other
    /// test here would still pass.
    #[test]
    fn provenance_only_changes_do_not_move_the_digest() {
        let base = rec(11, -4.25);
        let mut spun = base;
        spun.davidson_iters = base.davidson_iters + 97;
        spun.cg_iters = base.cg_iters + 43;
        spun.exit_code = base.exit_code + 2;
        spun.warm = !base.warm;
        // `mirrored` is in the same class as `warm` and is pinned here for the same reason:
        // it says how the number was reached, and a certificate over "how" would fire on a
        // symmetry declaration that changed no physics.
        spun.mirrored = !base.mirrored;
        assert_ne!(base, spun, "the two records must actually differ, or this proves nothing");
        assert_eq!(
            Digest::of_record(&base),
            Digest::of_record(&spun),
            "the digest moved on a provenance-only change; it has been re-coupled to how \
             the number was reached rather than what it is"
        );

        // And the physical content still moves it, so the insensitivity above is a scoping
        // decision rather than a dead hash.
        let mut moved = base;
        moved.energy_bits ^= 1;
        assert_ne!(
            Digest::of_record(&base),
            Digest::of_record(&moved),
            "the digest is insensitive to the ENERGY, which makes it no certificate at all"
        );
    }

    /// Every corruption the certificate is supposed to catch, caught. A single flipped bit
    /// in one energy, a dropped node, a duplicated node, and a swapped pair of values.
    #[test]
    fn corruption_is_convicted() {
        let records: Vec<NodeRecord> = (0..32).map(|i| rec(i, -3.0 - i as f64 * 0.01)).collect();
        let shards: Vec<Digest> = records
            .chunks(8)
            .map(|c| Digest::fold(c.iter().map(Digest::of_record)))
            .collect();

        // one flipped bit, in the lowest mantissa bit of one energy
        let mut bitflip = records.clone();
        bitflip[13].energy_bits ^= 1;
        assert!(
            !Certificate::check(Digest::of_table(&bitflip), &shards).is_clean(),
            "a one-bit energy corruption was not convicted"
        );

        // a dropped node
        let mut dropped = records.clone();
        dropped.remove(5);
        assert!(
            !Certificate::check(Digest::of_table(&dropped), &shards).is_clean(),
            "a dropped node was not convicted"
        );

        // a duplicated node
        let mut duped = records.clone();
        duped.push(records[9]);
        assert!(
            !Certificate::check(Digest::of_table(&duped), &shards).is_clean(),
            "a duplicated node was not convicted"
        );

        // two nodes' VALUES swapped — the case a plain sum of energies could never see
        let mut swapped = records.clone();
        let (a, b) = (swapped[3].energy_bits, swapped[20].energy_bits);
        swapped[3].energy_bits = b;
        swapped[20].energy_bits = a;
        assert!(
            !Certificate::check(Digest::of_table(&swapped), &shards).is_clean(),
            "a swap of two nodes' energies was not convicted — the digest is not \
             index-aware"
        );

        // a VOIDed node quietly replaced by a scored one
        let mut relabelled = records.clone();
        relabelled[7].status = NodeStatus::Void(crate::node::VoidReason::BudgetExhausted);
        assert!(
            !Certificate::check(Digest::of_table(&relabelled), &shards).is_clean(),
            "a status change was not convicted"
        );
    }
}
