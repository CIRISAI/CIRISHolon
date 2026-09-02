//! THE OBSERVER'S FRAME AS AN ALLOCATION LAW (ACUITY-B, frozen 2026-09-02).
//!
//! Reading B of `OBJECT.md` asks what the observer's frame selects. Half of the answer is
//! a theorem and never runs: under the two-box law the zoom never touches the physics, so
//! any verdict computed from a thing's own atoms is frame-invariant by construction — the
//! tree falls whether or not anyone is there. The other half is ALLOCATION, and it is
//! measured here: the frame selects which holons run FINE; the unobserved are carried
//! COARSE; and the only question that pays is whether carrying the unobserved region coarse
//! changes the observed thing, against how much work it saves.
//!
//! The coarse law: a coarse composite moves as ONE object on its conserved totals — its
//! members share its centre-of-mass velocity, it is accelerated by the summed forces its
//! members receive from FINE atoms (Newton pairs applied on both sides, so momentum is
//! exact), its internal relative velocities are banked at coarsening and restored at
//! re-entry. Pairs, triples and quadruples whose members are ALL coarse are not evaluated:
//! that is the saving, and inter-composite coarse interaction is the approximation.
//!
//! Every membership transition is a ledgered scene event: the energy it moves is measured
//! and posted to the `acuity` receipt column, so the drift gate stays closed by
//! construction and the column IS the energy cost of the observer's frame — reported
//! beside the speedup, never hidden.
//!
//! SCOPE, stated: pair-sector transitions are accounted exactly. Three- and four-body
//! all-coarse skipping is counted but its transition energy is NOT yet posted, so G2 is
//! exact only on pair-only scenes (the freeze's first reading). That accounting is the
//! named follow-up before any water reading.

/// The scene box in world coordinates: an axis-aligned cube.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AcuityFrame {
    pub center: [f64; 3],
    pub half: f64,
}

impl AcuityFrame {
    pub fn contains(&self, p: [f64; 3]) -> bool {
        (p[0] - self.center[0]).abs() <= self.half
            && (p[1] - self.center[1]).abs() <= self.half
            && (p[2] - self.center[2]).abs() <= self.half
    }

    /// A frame that covers everything: the G0 identity configuration.
    pub fn everything() -> Self {
        Self {
            center: [0.0; 3],
            half: f64::INFINITY,
        }
    }
}

/// The exact integer work partition the frame produced. `pairs_fine + pairs_skipped` must
/// equal the pairs the force pass examined — a partition, not an estimate (G4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AcuityWork {
    pub pairs_fine: u64,
    pub pairs_skipped: u64,
    pub triples_skipped: u64,
    pub quads_skipped: u64,
    /// Membership transitions (atoms changing fine/coarse), for the ledger's narrative.
    pub transitions: u64,
}

impl AcuityWork {
    pub const fn zero() -> Self {
        Self {
            pairs_fine: 0,
            pairs_skipped: 0,
            triples_skipped: 0,
            quads_skipped: 0,
            transitions: 0,
        }
    }
    pub fn pairs_examined(&self) -> u64 {
        self.pairs_fine + self.pairs_skipped
    }
    /// Fraction of examined pairs not evaluated. Zero when nothing was coarse.
    pub fn pair_saving(&self) -> f64 {
        let n = self.pairs_examined();
        if n == 0 {
            0.0
        } else {
            self.pairs_skipped as f64 / n as f64
        }
    }
}

/// How a neighbour pair is treated under the current and previous membership.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairKind {
    /// Evaluated and accumulated as always.
    Live,
    /// Both atoms coarse now and before: not evaluated.
    Skip,
    /// Both coarse now, not before: evaluated ONCE for the ledger, not accumulated.
    TransitionOut,
    /// Not both coarse now, but both before: evaluated and accumulated, and its value
    /// posted to the ledger as re-admitted potential.
    TransitionIn,
}

/// Planted defects, each in the sector it acts on. Production leaves this at `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AcuityPlant {
    #[default]
    None,
    /// Drop the reaction on the coarse side of a fine-coarse pair (sector: momentum).
    DropReaction,
    /// Apply transitions without posting them (sector: the ledger).
    SkipLedger,
    /// Count skipped pairs as fine (sector: the work counter).
    Miscount,
}
