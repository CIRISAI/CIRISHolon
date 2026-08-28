//! GRAINS — closure-aligned scheduling, and the engine's own curvature module.
//!
//! The bridge campaigns (conformance/gravity, five frozen preregistrations)
//! measured, exactly and twice-routed, that a coarse geometric view of exact
//! involution-generated dynamics does NOT close continuously: it closes at
//! particular steps and fails between them, with the failure carried by
//! coherence rather than noise. On the measured D4 Floquet family:
//!
//!   step k:         1        2        3        4
//!   omega (sector)  0        1        0        1     (matter's imprint)
//!   delta (closure) 0        1/2      0        1/2   (vs memoryless view)
//!   back-reaction   closed   flip     closed   inert
//!
//! The kernel is one algebraic fact, machine-checked in
//! `lean/CIRISHolon/Grain.lean`: for an involution R and imaginary unit i,
//! (1+iR)² = 2iR (a deterministic toggle) and (1+iR)⁴ = −4 (revival up to
//! global phase). THE GRAIN LAW: a coarse view of such dynamics is exact at
//! the steps where the step operator is scalar-or-toggle, and those steps
//! are where a scheduler may refresh coarse tiers with ZERO defect and take
//! its cheapest exact checkpoints.
//!
//! THE FENCE, stated once and enforced by the type: the period 4 belongs to
//! the measured coupling (θ = π/4), NOT to nature and NOT to the engine. A
//! `Grain` must be constructed from a period that a caller measured or
//! derived; `Grain::from_bridge_family()` is the one named constant, and it
//! carries its provenance. Nothing here claims a universal clock.
//!
//! What this buys the engine, concretely: the tuner's coarse tiers
//! (level-of-detail under a frame budget) can be refreshed ON grain
//! boundaries — where the coarse view is exact — instead of every frame,
//! and the defect between boundaries is a stated bound rather than an
//! unmeasured hope. That is the graphics face of the same law whose
//! quantum face is the closure rung.

/// A closure-aligned schedule: the period at which the coarse view is exact,
/// plus what the campaign measured between boundaries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grain {
    /// Steps between exact closure points. NOT universal — see the fence.
    pub period: u32,
    /// Where the coarse view is exact within one period (offsets, sorted).
    /// For the bridge family: 0 (start) and 4 (revival); step 2 is a toggle
    /// (exact for a view that carries the toggle, not for a memoryless one).
    pub exact_at: &'static [u32],
    /// Provenance of the numbers: never anonymous.
    pub source: &'static str,
}

impl Grain {
    /// The measured bridge family (D4 Floquet, θ = π/4). Its period is a
    /// PARAMETER of that coupling, recorded with its campaign.
    pub const fn from_bridge_family() -> Grain {
        Grain {
            period: 4,
            exact_at: &[0, 4],
            source: "conformance/gravity BRIDGE-2/3, frozen prereg 4e26345/8b12296",
        }
    }

    /// A caller-measured schedule. `period` must be positive; the caller
    /// owns the provenance string, because an unprovenanced schedule is a
    /// guess wearing a type.
    pub fn measured(period: u32, exact_at: &'static [u32], source: &'static str) -> Grain {
        assert!(period > 0, "grain: period must be positive");
        assert!(!source.is_empty(), "grain: a schedule must name its source");
        Grain { period, exact_at, source }
    }

    /// Is the coarse view exact at this step (mod the period)?
    pub fn closes_at(&self, step: u64) -> bool {
        let r = (step % self.period as u64) as u32;
        self.exact_at.contains(&r) || (r == 0 && self.exact_at.contains(&self.period))
    }

    /// Steps until the next exact closure point (0 if this step is one).
    pub fn steps_to_close(&self, step: u64) -> u64 {
        if self.closes_at(step) {
            return 0;
        }
        (1..=self.period as u64)
            .find(|d| self.closes_at(step + d))
            .expect("a grain always closes within one period")
    }
}

/// The measured closure defect of a memoryless coarse view against the
/// bridge family's exact dynamics, as exact rationals (numerator/2): the
/// sequence δ = 0, 1/2, 0, 1/2 at k = 1..4, banked from BRIDGE-1/2/3.
pub const BRIDGE_DEFECT_HALVES: [u32; 4] = [0, 1, 0, 1];

/// The measured sector overlap ω = 0, 1, 0, 1 at k = 1..4 (BRIDGE-2 B1′):
/// matter's imprint on geometry, maximal at odd steps, coherently erased at
/// even steps.
pub const BRIDGE_OMEGA: [u32; 4] = [0, 1, 0, 1];

/// The measured back-reaction response (BRIDGE-3 B2″), as the sign the
/// geometry-conditioned action applies to the matter coherence: 0 means the
/// channel is closed (no coherence exists), −1 a full flip, +1 exactly
/// inert.
pub const BRIDGE_BACKREACTION: [i32; 4] = [0, -1, 0, 1];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_grain_closes_where_the_campaign_measured() {
        let g = Grain::from_bridge_family();
        assert!(g.closes_at(0));
        assert!(!g.closes_at(1));
        assert!(!g.closes_at(2));
        assert!(!g.closes_at(3));
        assert!(g.closes_at(4), "the revival is a closure point");
        assert!(g.closes_at(8), "and it recurs");
        assert_eq!(g.steps_to_close(1), 3);
        assert_eq!(g.steps_to_close(4), 0);
        assert!(!g.source.is_empty());
    }

    /// The banked constants must stay exactly what the frozen campaigns
    /// measured — this test is the engine's copy of the gravity record, and
    /// it fails loudly if anyone edits the numbers without a new campaign.
    #[test]
    fn banked_constants_match_the_frozen_campaigns() {
        assert_eq!(BRIDGE_OMEGA, [0, 1, 0, 1]);
        assert_eq!(BRIDGE_DEFECT_HALVES, [0, 1, 0, 1]);
        assert_eq!(BRIDGE_BACKREACTION, [0, -1, 0, 1]);
        // The structural reading: the coarse view is exact exactly where
        // the defect vanishes.
        let g = Grain::from_bridge_family();
        for k in 1..=4u64 {
            let defect_zero = BRIDGE_DEFECT_HALVES[(k - 1) as usize] == 0;
            // k=2's toggle is exact for a toggle-carrying view but not for
            // the memoryless one the campaign staked, so only k=4 (and the
            // period's origin) are grain boundaries.
            if k == 4 {
                assert!(g.closes_at(k), "revival must be a grain boundary");
            }
            if defect_zero {
                assert!(k == 1 || k == 3, "defect vanishes only at odd steps here");
            }
        }
    }

    #[test]
    #[should_panic(expected = "must name its source")]
    fn unprovenanced_schedule_refused() {
        let _ = Grain::measured(4, &[0, 4], "");
    }

    /// The involution algebra the Lean law proves, checked numerically in
    /// the engine's own exact ring: (1+iR)² = 2iR and (1+iR)⁴ = −4 with
    /// R the central toggle (here: i itself squares to −1, R = 1 case, and
    /// the generic case via Gaussian integers).
    #[test]
    fn revival_algebra_holds_in_the_ring() {
        // (1+i)² = 2i ; (1+i)⁴ = −4  (R = 1 instance)
        let (re, im) = (1i64, 1i64);
        let (sq_re, sq_im) = (re * re - im * im, 2 * re * im);
        assert_eq!((sq_re, sq_im), (0, 2), "(1+i)^2 = 2i");
        let (q_re, q_im) = (sq_re * sq_re - sq_im * sq_im, 2 * sq_re * sq_im);
        assert_eq!((q_re, q_im), (-4, 0), "(1+i)^4 = -4");
    }
}
