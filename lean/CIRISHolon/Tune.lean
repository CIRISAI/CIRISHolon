/-
CIRISHolon.Tune — the organic-degradation law, machine-checked.

The engine's tuning module (`engine/crates/holon/src/tune.rs`) selects a
configuration under a DX-declared policy: one axis is HELD (exactness, or a
latency budget), the rest may degrade in declared order, and refusal is the
total fallback. This file mechanizes the law that makes that "organic"
rather than hopeful:

  * `select_sound`    — whatever the selector returns satisfies the hold.
  * `select_complete` — the selector refuses ONLY when no offered
                        configuration satisfies the hold: refusal is last.
  * `exact_never_degraded` — under an exactness hold, anything returned is
                        exact. The graphics face (hold latency, degrade
                        detail) and the referee face (hold exactness,
                        degrade latency) are the same theorem with the two
                        holds swapped — level-of-detail rendering at a frame
                        budget and exact refereeing are one selector.

WHY a held configuration is IDEAL on given hardware is the Limits ledger's
half (`Limits.lean`): the mechanized floors (L1 gate-touch, L4 coefficient
bits) mean a held-exact configuration within a small constant of the floor
has nothing left to tune — the sweep measures the constant, the floor
theorem explains why the sweep stops. The BEGGING entries (L2 word width,
L3 stabilizer rank) are exactly the axes where the sweep should keep
finding wins; a sweep that stalls on a begging axis is a finding, not a
fact of nature.
-/
import Mathlib.Data.List.Basic

namespace CIRISHolon.Tune

/-- An abstract configuration: whether it is exact, and its cost against
    the latency budget's unit. -/
structure Config where
  exact : Bool
  cost : ℕ
  deriving Repr, DecidableEq

/-- The held axis. -/
inductive Hold
  | exactness
  | latency (budget : ℕ)
  deriving Repr, DecidableEq

/-- What it means for a configuration to satisfy the hold. -/
def holds : Hold → Config → Prop
  | .exactness, c => c.exact = true
  | .latency b, c => c.cost ≤ b

instance : ∀ h c, Decidable (holds h c)
  | .exactness, _ => inferInstanceAs (Decidable (_ = true))
  | .latency _, _ => inferInstanceAs (Decidable (_ ≤ _))

/-- The selector: the offered list IS the degradation order (front = least
    degraded); the first lawful configuration wins, refusal only at the
    end. -/
def select (h : Hold) : List Config → Option Config
  | [] => none
  | c :: rest => if holds h c then some c else select h rest

/-- **Soundness**: whatever is returned satisfies the hold. -/
theorem select_sound (h : Hold) :
    ∀ (cs : List Config) (c : Config), select h cs = some c → holds h c := by
  intro cs
  induction cs with
  | nil => intro c hc; simp [select] at hc
  | cons a rest ih =>
    intro c hc
    by_cases ha : holds h a
    · simp [select, ha] at hc; exact hc ▸ ha
    · simp [select, ha] at hc; exact ih c hc

/-- **Completeness**: refusal happens only when NOTHING offered satisfies
    the hold — degradation is walked to the end before refusing. -/
theorem select_complete (h : Hold) :
    ∀ cs : List Config, select h cs = none → ∀ c ∈ cs, ¬holds h c := by
  intro cs
  induction cs with
  | nil => intro _ c hc; cases hc
  | cons a rest ih =>
    intro hnone c hc
    by_cases ha : holds h a
    · simp [select, ha] at hnone
    · simp [select, ha] at hnone
      rcases List.mem_cons.mp hc with rfl | hmem
      · exact ha
      · exact ih hnone c hmem

/-- **The exactness face**: under an exactness hold, anything the selector
    returns is exact — approximation cannot leak past the policy. -/
theorem exact_never_degraded (cs : List Config) (c : Config)
    (hc : select .exactness cs = some c) : c.exact = true :=
  select_sound .exactness cs c hc

/-- **The graphics face**: under a latency hold, anything returned fits the
    frame budget — real-time rendering's law, same selector. -/
theorem frame_budget_held (b : ℕ) (cs : List Config) (c : Config)
    (hc : select (.latency b) cs = some c) : c.cost ≤ b :=
  select_sound (.latency b) cs c hc

end CIRISHolon.Tune
