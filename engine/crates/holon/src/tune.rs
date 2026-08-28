//! THE TUNING MODULE — organic degradation under a DX-declared policy.
//!
//! The caller declares what is HELD (never degrades) and what may DEGRADE,
//! in order, to declared degrees. The selector walks the degradation order
//! and returns the first configuration the policy accepts; refusal is the
//! TOTAL fallback, reached only when nothing lawful remains. What degraded
//! is REPORTED — the certificate carries it; nothing degrades silently.
//!
//! The two archetypal policies are the engine's two faces:
//!   * hold EXACTNESS, degrade latency — the referee/quantum face. Accuracy
//!     degradation under this hold is a POLICY ERROR at construction, not a
//!     runtime surprise: the exact tiers have no lawful approximation.
//!   * hold LATENCY (a frame budget), degrade detail/accuracy to declared
//!     degrees — the graphics face. This is level-of-detail generalized:
//!     real-time rendering in a browser is this policy applied at 60 Hz,
//!     and each tier's speedups widen what fits inside the frame budget.
//!
//! The law itself is machine-checked (`lean/CIRISHolon/Tune.lean`):
//! `select_sound` (the hold is held by whatever is returned),
//! `select_complete` (refusal only when nothing lawful exists), and
//! `exact_never_degraded`. WHY a tuned configuration is ideal on this
//! hardware is the Limits ledger's job (`Limits.lean`): held-exact gate
//! configs sit within a constant of the mechanized L1/L4 floors, so past a
//! point there is nothing left to tune — the sweep measures the constant,
//! the theorem explains why it stops.
//!
//! Calibration is RENTED (house rule): a sweep table carries the host
//! fingerprint and epoch it was measured on; a table from another host or
//! epoch is ignored, never trusted. The static routing rules below encode
//! only what the lanes MEASURED; interactions not yet swept (e.g.
//! magic5 × sliced) are named `Unswept` and route to the certified default
//! rather than to a guess.

/// What the policy holds — the axis that never degrades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hold {
    /// Exact results only. The referee face.
    Exactness,
    /// Stay inside a latency budget (e.g. a frame). The graphics face.
    Latency { budget_ms: f64 },
}

/// A declared degradable axis, with its degree. Order matters: degradation
/// walks the list front to back — that is what "organically" means here.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Degrade {
    /// Allow up to this factor more wall time.
    Latency { up_to_factor: f64 },
    /// Drop level of detail down to this floor (graphics tiers).
    Detail { min_level: u32 },
    /// Refuse work above this magic budget instead of attempting it.
    Scope { max_t: u32 },
    /// Allow approximation to this epsilon — LAWFUL ONLY where a tier
    /// declares a tolerance (the ℝ tiers); never under Hold::Exactness.
    Accuracy { eps: f64 },
}

/// A validated policy. Construction is where lawfulness is enforced.
#[derive(Clone, Debug)]
pub struct Policy {
    pub hold: Hold,
    pub degrade: Vec<Degrade>,
    /// Optional closure-aligned schedule (`grain.rs`): when present, coarse
    /// tiers refresh on grain boundaries — where the coarse view is EXACT —
    /// instead of every step, and the between-boundary defect is a stated
    /// bound rather than an unmeasured hope. The period is always a
    /// measured parameter carrying its provenance, never a universal clock.
    pub grain: Option<crate::grain::Grain>,
}

#[derive(Debug, PartialEq)]
pub enum PolicyError {
    /// Accuracy degradation requested under an exactness hold: the exact
    /// tiers have no lawful approximation, so this is refused at the door.
    AccuracyUnderExactness,
}

impl Policy {
    pub fn new(hold: Hold, degrade: Vec<Degrade>) -> Result<Policy, PolicyError> {
        if hold == Hold::Exactness
            && degrade.iter().any(|d| matches!(d, Degrade::Accuracy { .. }))
        {
            return Err(PolicyError::AccuracyUnderExactness);
        }
        Ok(Policy { hold, degrade, grain: None })
    }

    /// The referee default: exact, latency degrades unboundedly, scope caps
    /// only where the caller says so.
    pub fn exact() -> Policy {
        Policy {
            hold: Hold::Exactness,
            degrade: vec![Degrade::Latency { up_to_factor: f64::INFINITY }],
            grain: None,
        }
    }
}

/// The measured tuning surface: which branch evaluator carries the magic
/// tier. Encodes ONLY what the lanes measured; unswept combinations refuse
/// to guess.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Decomp {
    /// Block-deduplicated pruned enumeration (the certified default; wins
    /// where dedup collapses the branch space: t > n).
    Pruned,
    /// 64-branches-per-word sliced evaluation (wins where branches share
    /// structure dedup would destroy: t ≤ n; measured 14–26× there).
    Sliced,
    /// Magic5FromCat recursive decomposition (fewest branches from t ≥ 5;
    /// realized α 0.4111 at t=28, 0.4027 at t=64).
    Magic5,
}

/// A knob interaction the sweep has not measured yet. Named, not guessed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Unswept {
    /// Sliced evaluation over the Magic5 branch space: structural sharing
    /// plausibly survives (same circuit skeleton) but is UNMEASURED.
    Magic5TimesSliced,
    /// Sliced vs pruned ON THE REWRITTEN AFFINE ENGINE: the pre-merge rule
    /// ("sliced wins at t ≤ n") was FALSIFIED by the first quiet-runner
    /// sweep after the 330–907× per-branch rewrite landed — pruned now
    /// dominates the measured cases (CI sliced surface, entry eight). The
    /// sliced path's winning region on the current engine, if any, is
    /// unmapped.
    SlicedOnRewrittenEngine,
    /// Magic5 vs pruned-dedup head-to-head on the rewritten engine: the
    /// branch-count advantage is proven, the wall-clock crossover is not
    /// yet measured.
    Magic5VersusPruned,
}

/// The chosen configuration, with the report the certificate carries.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub decomp: Decomp,
    pub defer_phase: bool,
    pub shards: usize,
    /// What the policy allowed to degrade on this run (empty = nothing).
    pub degraded: Vec<String>,
    /// Interactions deliberately not exploited because they are unswept.
    pub left_on_table: Vec<Unswept>,
    /// If the policy carries a grain: steps until the next exact closure
    /// point, so a coarse consumer knows when it may refresh with zero
    /// defect (`grain.rs`, and the bridge campaigns behind it).
    pub steps_to_close: Option<u64>,
}

/// Refusal, the total fallback — with its reason, never silent.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    pub reason: String,
}

/// v2 selector — corrected by the first quiet-runner sweep, which FALSIFIED
/// v1's "t ≤ n → sliced" rule: the affine rewrite (330–907×) accelerated the
/// per-branch engine both alternatives ride on, and on the measured surface
/// the pruned-dedup path now dominates every swept case. So v2 routes the
/// certified fast default (Pruned) everywhere and NAMES the unswept
/// alternatives instead of guessing; sliced and magic5 remain callable
/// explicitly, and the next sweep can promote them where they win. Scope
/// caps refuse above their max_t — a refusal is a result. (This correction
/// is the tuner working as designed: routing follows measurement, and a
/// merge that moves the surface moves the routes.)
pub fn select(policy: &Policy, n: usize, t: u32, shards: usize) -> Result<Choice, Refusal> {
    let _ = n;
    for d in &policy.degrade {
        if let Degrade::Scope { max_t } = d {
            if t > *max_t {
                return Err(Refusal {
                    reason: format!(
                        "policy scope: t={t} exceeds declared max_t={max_t}; refusing rather than degrading an undeclared axis"
                    ),
                });
            }
        }
    }
    let (decomp, left) = (
        Decomp::Pruned,
        vec![
            Unswept::SlicedOnRewrittenEngine,
            Unswept::Magic5VersusPruned,
            Unswept::Magic5TimesSliced,
        ],
    );
    let degraded = match policy.hold {
        Hold::Exactness => vec!["latency (declared)".to_string()],
        Hold::Latency { .. } => vec![],
    };
    Ok(Choice {
        decomp,
        defer_phase: true,
        shards: shards.max(1),
        degraded,
        left_on_table: left,
        steps_to_close: policy.grain.map(|g| g.steps_to_close(0)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accuracy_under_exactness_is_refused_at_construction() {
        let e = Policy::new(
            Hold::Exactness,
            vec![Degrade::Accuracy { eps: 1e-9 }],
        );
        assert_eq!(e.unwrap_err(), PolicyError::AccuracyUnderExactness);
    }

    #[test]
    fn v2_routes_the_certified_default_and_names_the_unswept() {
        let p = Policy::exact();
        for (n, t) in [(64usize, 12u32), (8, 12), (2, 3)] {
            let c = select(&p, n, t, 8).unwrap();
            assert_eq!(c.decomp, Decomp::Pruned);
            assert_eq!(
                c.left_on_table,
                vec![
                    Unswept::SlicedOnRewrittenEngine,
                    Unswept::Magic5VersusPruned,
                    Unswept::Magic5TimesSliced,
                ]
            );
        }
    }

    #[test]
    fn scope_cap_refuses_with_reason() {
        let p = Policy::new(Hold::Exactness, vec![Degrade::Scope { max_t: 20 }]).unwrap();
        let r = select(&p, 100, 28, 8).unwrap_err();
        assert!(r.reason.contains("max_t=20"));
    }

    #[test]
    fn grain_reaches_the_choice() {
        let mut p = Policy::exact();
        p.grain = Some(crate::grain::Grain::from_bridge_family());
        let c = select(&p, 8, 3, 4).unwrap();
        assert_eq!(c.steps_to_close, Some(0), "step 0 is a closure point");
    }

    #[test]
    fn graphics_policy_is_lawful() {
        // Hold latency, degrade detail then accuracy: the LOD face.
        let p = Policy::new(
            Hold::Latency { budget_ms: 16.6 },
            vec![Degrade::Detail { min_level: 2 }, Degrade::Accuracy { eps: 1e-3 }],
        );
        assert!(p.is_ok());
    }
}
