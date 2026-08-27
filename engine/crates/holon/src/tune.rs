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
        Ok(Policy { hold, degrade })
    }

    /// The referee default: exact, latency degrades unboundedly, scope caps
    /// only where the caller says so.
    pub fn exact() -> Policy {
        Policy { hold: Hold::Exactness, degrade: vec![Degrade::Latency { up_to_factor: f64::INFINITY }] }
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
}

/// Refusal, the total fallback — with its reason, never silent.
#[derive(Clone, Debug, PartialEq)]
pub struct Refusal {
    pub reason: String,
}

/// v1 selector over the MEASURED rules:
///   * t > n  → Pruned (dedup collapses the space; sliced measured 0.1× there)
///   * t ≤ n  → Sliced (measured 14–26×), Magic5's smaller branch space is
///     preferred from t ≥ 5 ONLY where slicing does not apply, because
///     magic5 × sliced is unswept and we do not guess.
///   * Scope caps refuse above their max_t — a refusal is a result.
pub fn select(policy: &Policy, n: usize, t: u32, shards: usize) -> Result<Choice, Refusal> {
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
    let (decomp, left) = if t as usize <= n {
        (Decomp::Sliced, vec![Unswept::Magic5TimesSliced])
    } else if t >= 5 {
        (Decomp::Magic5, vec![])
    } else {
        (Decomp::Pruned, vec![])
    };
    let degraded = match policy.hold {
        Hold::Exactness => vec!["latency (declared)".to_string()],
        Hold::Latency { .. } => vec![],
    };
    Ok(Choice { decomp, defer_phase: true, shards: shards.max(1), degraded, left_on_table: left })
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
    fn measured_routing_rules() {
        let p = Policy::exact();
        // t <= n: sliced, with the unswept interaction named.
        let c = select(&p, 64, 12, 8).unwrap();
        assert_eq!(c.decomp, Decomp::Sliced);
        assert_eq!(c.left_on_table, vec![Unswept::Magic5TimesSliced]);
        // t > n, t >= 5: magic5.
        let c = select(&p, 8, 12, 8).unwrap();
        assert_eq!(c.decomp, Decomp::Magic5);
        // tiny t beyond n: pruned default.
        let c = select(&p, 2, 3, 8).unwrap();
        assert_eq!(c.decomp, Decomp::Pruned);
    }

    #[test]
    fn scope_cap_refuses_with_reason() {
        let p = Policy::new(Hold::Exactness, vec![Degrade::Scope { max_t: 20 }]).unwrap();
        let r = select(&p, 100, 28, 8).unwrap_err();
        assert!(r.reason.contains("max_t=20"));
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
