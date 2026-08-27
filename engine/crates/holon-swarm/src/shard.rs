//! A shard **is an arena**. It is not a new entity class.
//!
//! The "one holon, values only" rule of `sim_engine/INTEGRATION_FRAME.md` forbids adding
//! a per-tier struct or a new holon field. Sharding therefore has to be expressible as
//! *values*:
//!
//! * a **shard** is one [`RuntimeArena`] with its own declared root and its own `g0`
//!   (the root's `GrossState` at construction — the shard's opening balance);
//! * a **boundary port** is an ordinary holon inside that arena with `boundary: true` —
//!   no new type, no new field, just the flag the holon already has;
//! * a **boundary relation** is a pair of plain indices, `(shard, holon)` on each side.
//!   Indices are values. The frame's own precedent for this is already on the record:
//!   "Two arenas — scene and descriptor library — related through `MaterialBinding`
//!   indices is frame-faithful: an arena is a resident refinement window."
//!
//! ## The one place the frame's current API forces a compromise, stated plainly
//!
//! [`RuntimeArena`] exposes `holons()` as `&[RuntimeHolon]` and offers no mutable ledger
//! accessor: the only mutator is `materialize`, which is append-only. A *stepping* shard
//! must therefore carry an index-aligned **ledger overlay** (`Vec<GrossState>`, 32 bytes
//! per holon) beside the arena, with the arena supplying structure (parent, depth, grain,
//! decomposition, child index) and the overlay supplying the live REG+ values.
//!
//! This is not a workaround so much as the shape the GPU port wants anyway — the overlay
//! is a flat, pointer-free, 32-byte-stride buffer that uploads to a device as-is. But it
//! has a consequence the gate must respect and which is easy to get wrong: **calling
//! `arena.validate()` cannot see an overlay corruption.** `validate()` checks the gross
//! states stored *in the arena*, which never change. The composition leg over the live
//! overlay ([`Shard::check_composition`]) and the rebuild leg
//! ([`Shard::revalidate_through_core`]) are what actually put the core's rule over the
//! live values. A mutation test pins this exact weakness.

use ciris_sim_core::holon::{Channels, Decomposition, HolonError};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{RuntimeArena, RuntimeHolonSpec, NO_RUNTIME_HOLON};

use crate::error::{Side, SwarmError};
use crate::exchange::FaultInjection;
use crate::ledger::{apply_delta, mix64, LedgerDelta};

/// One end of a boundary relation, held by the shard that owns it. Pure values: which
/// pair, which side of it, which peer shard, and which local holon is the port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShardLink {
    pub pair: usize,
    pub side: Side,
    pub peer: usize,
    pub holon: u32,
}

/// One holon engine: an arena, its live ledger overlay, and its boundary relations.
#[derive(Debug)]
pub struct Shard {
    pub(crate) index: usize,
    pub(crate) arena: RuntimeArena,
    /// Live REG+ ledger, index-aligned with `arena.holons()`.
    pub(crate) ledger: Vec<GrossState>,
    /// The shard's declared opening balance: `ledger[root]` at construction.
    pub(crate) g0: GrossState,
    pub(crate) root: u32,
    pub(crate) leaf_count: usize,
    pub(crate) links: Vec<ShardLink>,
    /// Root ledger observed after this round's local step, before any exchange.
    pub(crate) post_local: GrossState,
    /// This round's issued receipts, one per link, in link order.
    pub(crate) receipts: Vec<LedgerDelta>,
}

impl Shard {
    /// Build a shard of `leaf_count` terminal holons under one root.
    ///
    /// Ports are the first `links.len()` leaves (ids `1 ..= links.len()`), flagged
    /// `boundary: true`. Everything about the shard's identity — index, peers, port ids —
    /// is a value; nothing is a type.
    pub fn new(index: usize, leaf_count: usize, links: Vec<ShardLink>) -> Result<Self, SwarmError> {
        if leaf_count < 2 {
            return Err(SwarmError::Config("a shard needs at least two leaves"));
        }
        if links.len() > leaf_count {
            return Err(SwarmError::Config("more boundary links than leaves"));
        }
        for (i, link) in links.iter().enumerate() {
            if link.holon as usize != i + 1 {
                return Err(SwarmError::Config(
                    "port holon ids must be 1..=links.len(), one distinct port per link",
                ));
            }
        }

        let channels = Channels::REG_PLUS.union(Channels::MECHANICAL);

        // Deterministic seeding: a pure function of (shard index, leaf index), so two
        // hosts building the same swarm build bit-identical opening balances.
        let mut leaves = Vec::with_capacity(leaf_count);
        for i in 0..leaf_count {
            let k = mix64((index as u64) << 40 ^ i as u64);
            // Ports carry a large shard-dependent bias in BOTH count lanes. Without it the
            // opening imbalance across a boundary would round to zero in `occupancy` and be
            // identically zero in `constituents` (every leaf represents one terminal
            // holon), so the exchange would move momentum only and the mutation suite would
            // be testing one lane out of four while looking like it tested all of them.
            //
            // A `constituents` transfer is the frame-honest form of load migration: the
            // count of terminal holons a boundary holon represents moves from one arena's
            // books to the other's.
            let is_port = i < links.len();
            let occupancy_bias = if is_port {
                64 * (((index * 7 + i * 3) % 5) as u64)
            } else {
                0
            };
            let constituent_bias = if is_port {
                8 * (((index * 5 + i) % 4) as u64)
            } else {
                0
            };
            // Momentum biases keep their lane's sign (positive lane 0, negative lane 1) so
            // that the root's momentum[0] stays strictly positive — the overflow mutation
            // test relies on adding i64::MAX to a strictly positive lane overflowing.
            let momentum_bias = if is_port {
                [
                    16 * ((index * 3 + i) % 5) as i64,
                    -16 * ((index * 2 + i) % 3) as i64,
                ]
            } else {
                [0, 0]
            };
            leaves.push(GrossState::aggregate(
                1 + constituent_bias,
                8 + (k % 9) + occupancy_bias,
                [
                    1 + ((k >> 8) % 3) as i64 + momentum_bias[0],
                    -(1 + ((k >> 16) % 3) as i64) + momentum_bias[1],
                ],
            ));
        }
        let mut root_gross = GrossState::ZERO;
        for leaf in &leaves {
            root_gross = root_gross
                .checked_combine(*leaf)
                .ok_or(SwarmError::Core(HolonError::GrossStateDoesNotCompose))?;
        }

        // Port whole-state. Whole-only state does not factor through the children, so it
        // is carried on the holon, not derived. It is *not* mutated by the exchange
        // (the core exposes no mutable scalar pool), and the determinism harness compares
        // it with `to_bits()` to say so honestly rather than by assumption.
        let port_whole: Vec<[f64; 2]> = links
            .iter()
            .map(|link| [index as f64, link.side.index() as f64])
            .collect();

        let mut specs = Vec::with_capacity(leaf_count + 1);
        specs.push(RuntimeHolonSpec {
            parent: NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: 2,
            gross: root_gross,
            whole: &[],
            channels,
            boundary: true,
            decomposition: Decomposition::Expanded,
        });
        for (i, leaf) in leaves.iter().enumerate() {
            let is_port = i < links.len();
            specs.push(RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 1,
                gross: *leaf,
                whole: if is_port { &port_whole[i][..] } else { &[] },
                channels,
                boundary: is_port,
                decomposition: Decomposition::Leaf,
            });
        }

        let arena = RuntimeArena::from_specs(&specs, 0)?;
        let root = arena.root();
        let ledger: Vec<GrossState> = arena.holons().iter().map(|h| h.gross).collect();
        let receipts = vec![LedgerDelta::ZERO; links.len()];
        Ok(Self {
            index,
            arena,
            g0: root_gross,
            ledger,
            root,
            leaf_count,
            links,
            post_local: root_gross,
            receipts,
        })
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn arena(&self) -> &RuntimeArena {
        &self.arena
    }

    /// The arena's declared root holon id.
    pub fn root_id(&self) -> u32 {
        self.root
    }

    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    pub fn links(&self) -> &[ShardLink] {
        &self.links
    }

    /// The receipts this shard issued in the most recent exchange round, in link order.
    pub fn receipts(&self) -> &[LedgerDelta] {
        &self.receipts
    }

    /// The shard's declared opening balance (`g0`).
    pub fn g0(&self) -> GrossState {
        self.g0
    }

    /// The shard's live ledger at its root — its current balance.
    pub fn root_ledger(&self) -> GrossState {
        self.ledger[self.root as usize]
    }

    pub fn ledger(&self) -> &[GrossState] {
        &self.ledger
    }

    /// Redistribute conserved quantity *inside* the shard.
    ///
    /// Two red/black sweeps of nearest-neighbour pair transfers over the leaves. Every
    /// move is `a -= d, b += d` with the same `d`, so the shard's root ledger is untouched
    /// by construction — which the gate's L1 leg then checks independently rather than
    /// trusting. The stir keeps the interior from freezing at a uniform fixed point; it is
    /// a pure function of `(round, shard index, holon index)` and so is schedule-free.
    pub fn local_step(&mut self, round: u64, fault: FaultInjection) -> Result<(), SwarmError> {
        let count = self.leaf_count;
        let shard = self.index;
        let leaves = &mut self.ledger[1..=count];
        for phase in 0..2usize {
            let mut i = phase;
            while i + 1 < count {
                let a = i;
                let b = i + 1;
                let ga = leaves[a];
                let gb = leaves[b];
                let oops = || SwarmError::LedgerOverflow {
                    round,
                    shard,
                    holon: (a + 1) as u32,
                };

                let occ_a = i64::try_from(ga.occupancy).map_err(|_| oops())?;
                let occ_b = i64::try_from(gb.occupancy).map_err(|_| oops())?;
                let con_a = i64::try_from(ga.constituents).map_err(|_| oops())?;
                let con_b = i64::try_from(gb.constituents).map_err(|_| oops())?;

                let k = mix64(round ^ ((shard as u64) << 40) ^ (a as u64) << 1 ^ phase as u64);
                let stir_occ = (k % 3) as i64 - 1;
                let stir_mom = ((k >> 8) % 5) as i64 - 2;

                // Occupancy is a count: clamp the stirred diffusion into
                // [-occ_b, occ_a] so neither side can be driven negative. The clamp reads
                // only a and b, so it stays a per-pair-local, deterministic decision.
                let d_occ = occ_a
                    .checked_sub(occ_b)
                    .ok_or_else(oops)?
                    .wrapping_div(2)
                    .checked_add(stir_occ)
                    .ok_or_else(oops)?
                    .clamp(-occ_b, occ_a);
                let d_con = con_a.checked_sub(con_b).ok_or_else(oops)?.wrapping_div(2);
                let d_m0 = ga.momentum[0]
                    .checked_sub(gb.momentum[0])
                    .ok_or_else(oops)?
                    .wrapping_div(2)
                    .checked_add(stir_mom)
                    .ok_or_else(oops)?;
                let d_m1 = ga.momentum[1]
                    .checked_sub(gb.momentum[1])
                    .ok_or_else(oops)?
                    .wrapping_div(2)
                    .checked_sub(stir_mom)
                    .ok_or_else(oops)?;

                let d = LedgerDelta {
                    constituents: d_con,
                    occupancy: d_occ,
                    momentum: [d_m0, d_m1],
                };
                let debit = d.checked_neg().ok_or_else(oops)?;
                leaves[a] = apply_delta(ga, debit).ok_or_else(oops)?;
                leaves[b] = apply_delta(gb, d).ok_or_else(oops)?;
                i += 2;
            }
        }

        // FAULT INJECTION (test instrumentation only, see `FaultInjection`): mint
        // occupancy out of nothing, keeping composition intact so that only L1 and the
        // global leg can see it.
        if let FaultInjection::MintInLocalStep { shard: target } = fault {
            if target == self.index {
                let one = LedgerDelta {
                    occupancy: 1,
                    ..LedgerDelta::ZERO
                };
                let oops = || SwarmError::LedgerOverflow {
                    round,
                    shard,
                    holon: 1,
                };
                self.ledger[1] = apply_delta(self.ledger[1], one).ok_or_else(oops)?;
                let r = self.root as usize;
                self.ledger[r] = apply_delta(self.ledger[r], one).ok_or_else(oops)?;
            }
        }
        Ok(())
    }

    /// Credit/debit a boundary port, propagating the same delta to the root so the
    /// shard's internal composition stays exact. Both writes are checked.
    pub(crate) fn apply_at_port(
        &mut self,
        round: u64,
        holon: u32,
        delta: LedgerDelta,
    ) -> Result<(), SwarmError> {
        if holon == self.root {
            return Err(SwarmError::Config("a boundary port may not be the root"));
        }
        let oops = || SwarmError::LedgerOverflow {
            round,
            shard: self.index,
            holon,
        };
        let idx = holon as usize;
        let current = *self.ledger.get(idx).ok_or_else(oops)?;
        let updated = apply_delta(current, delta).ok_or_else(oops)?;
        let r = self.root as usize;
        let root_updated = apply_delta(self.ledger[r], delta).ok_or_else(oops)?;
        self.ledger[idx] = updated;
        self.ledger[r] = root_updated;
        Ok(())
    }

    /// Raw ledger write used *only* by fault injection, to corrupt one holon without
    /// touching its parent (or vice versa) and prove the composition leg fires.
    pub(crate) fn inject_raw(
        &mut self,
        round: u64,
        holon: u32,
        delta: LedgerDelta,
    ) -> Result<(), SwarmError> {
        let oops = || SwarmError::LedgerOverflow {
            round,
            shard: self.index,
            holon,
        };
        let idx = holon as usize;
        let current = *self.ledger.get(idx).ok_or_else(oops)?;
        self.ledger[idx] = apply_delta(current, delta).ok_or_else(oops)?;
        Ok(())
    }

    /// **L6** — the core's composition rule (`Expanded` parent == exact sum of resident
    /// children) applied to the *live* overlay, using the arena's own child index.
    pub fn check_composition(&self, round: u64) -> Result<(), SwarmError> {
        for parent in 0..self.arena.len() {
            if self.arena.holons()[parent].decomposition != Decomposition::Expanded {
                continue;
            }
            let mut composed = GrossState::ZERO;
            for child in self.arena.children(parent) {
                composed = composed
                    .checked_combine(self.ledger[child])
                    .ok_or(SwarmError::LedgerOverflow {
                        round,
                        shard: self.index,
                        holon: parent as u32,
                    })?;
            }
            if composed != self.ledger[parent] {
                return Err(SwarmError::CompositionBroken {
                    round,
                    shard: self.index,
                    holon: parent as u32,
                    declared: self.ledger[parent],
                    composed,
                });
            }
        }
        Ok(())
    }

    /// **L7 (structural)** — the arena's own `validate()`.
    ///
    /// Honest note: this checks the arena's *stored* headers, which the exchange never
    /// touches. It is a real check of structure (single root, depth chain, grain
    /// monotonicity, decomposition/child-count agreement) and a *vacuous* check of the
    /// live ledger. [`Self::revalidate_through_core`] is the non-vacuous form.
    pub fn validate_structure(&self, round: u64) -> Result<(), SwarmError> {
        self.arena
            .validate()
            .map_err(|source| SwarmError::ShardStructureInvalid {
                round,
                shard: self.index,
                source,
            })
    }

    /// **L7 (paranoid)** — rebuild an arena from the *live* overlay and let
    /// `ciris-sim-core`'s own validator accept or reject it. `RuntimeArena::from_specs`
    /// calls `validate()` internally, so this is the core's rule applied to current
    /// values. `O(holons)` with two allocations; not on the hot path.
    pub fn revalidate_through_core(&self, round: u64) -> Result<(), SwarmError> {
        let headers = self.arena.holons();
        let mut specs = Vec::with_capacity(headers.len());
        for (i, header) in headers.iter().enumerate() {
            specs.push(RuntimeHolonSpec {
                parent: header.parent,
                depth: header.depth,
                grain_units: header.grain_units,
                gross: self.ledger[i],
                whole: self.arena.whole_state(i).unwrap_or(&[]),
                channels: header.channels,
                boundary: header.is_boundary(),
                decomposition: header.decomposition,
            });
        }
        RuntimeArena::from_specs(&specs, self.root)
            .map(|_| ())
            .map_err(|source| SwarmError::ShardStructureInvalid {
                round,
                shard: self.index,
                source,
            })
    }

    /// Every f64 whole-state scalar in the arena, as raw bits. Whole-only state must
    /// compare bit-identically across runs; `.to_bits()` is the only honest comparison.
    pub fn whole_bits(&self) -> Vec<u64> {
        self.arena
            .whole_scalars()
            .iter()
            .map(|x| x.to_bits())
            .collect()
    }
}
