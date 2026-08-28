/-
CIRISHolon.Closure — the COLLISION theorem: no memoryless map, not merely
"worse than the one we tried".

An external review (2026-08-28) made a sharp point about BRIDGE-1/2's δ:
it measures the coarse view against ONE preregistered Markov model, so it
earns "worse than that model", not "best memoryless". But the measured
sequence already proves the stronger statement, and this file banks it.

The campaigns measured the coarse view's value at four steps:

    v₁ = v₃ = (½, ½),    v₂ = (0, 1),    v₄ = (1, 0).

If any time-homogeneous memoryless map F closed the view it would need
F(v₁) = v₂ and F(v₃) = v₄ simultaneously. But v₁ = v₃ while v₂ ≠ v₄ — a
FUNCTION cannot send one input to two outputs. So no such F exists, for
ANY memoryless model, not just the one staked.

`collision_refutes_memoryless` is that argument, and it needs nothing about
the physics: it is the pigeonhole for functions. `fiber_defect_half` then
gives the quantitative version — the two required successors are at maximal
total-variation distance, so the best single prediction is wrong by exactly
½ in the minimax sense, which is where the measured δ = ½ comes from.

Scope, stated because the review was right to insist: this refutes
MEMORYLESS closure of the DECLARED view. A classical model carrying extra
memory (a phase label) can reproduce the sequence — coarse-graining-induced
memory is standard (Mori–Zwanzig; process-tensor witnesses). The claim is
that the declared coarse view is non-autonomous and its missing memory has
a measured cost, not that no classical model exists.
-/
import Mathlib.Tactic

namespace CIRISHolon.Closure

/-- A coarse view's value: a distribution on two outcomes, as a pair. -/
abbrev View := ℚ × ℚ

/-- **THE COLLISION THEOREM.** If a view takes the same value at two steps
    but its successors differ, no time-homogeneous map on views can
    reproduce the trajectory. Nothing about geometry enters: this is the
    pigeonhole for functions, which is exactly why it is stronger than a
    comparison against any particular model. -/
theorem collision_refutes_memoryless
    {v₁ v₃ v₂ v₄ : View} (hsame : v₁ = v₃) (hdiff : v₂ ≠ v₄) :
    ¬∃ F : View → View, F v₁ = v₂ ∧ F v₃ = v₄ := by
  rintro ⟨F, h1, h3⟩
  exact hdiff (h1 ▸ hsame ▸ h3 ▸ rfl)

/-- The measured sequence, as data. -/
def v₁ : View := (1/2, 1/2)
def v₂ : View := (0, 1)
def v₃ : View := (1/2, 1/2)
def v₄ : View := (1, 0)

/-- The campaigns' own numbers satisfy the theorem's hypotheses. -/
theorem bridge_sequence_collides :
    ¬∃ F : View → View, F v₁ = v₂ ∧ F v₃ = v₄ :=
  collision_refutes_memoryless rfl (by decide)

/-- Total-variation distance between two-outcome distributions. -/
def tv (a b : View) : ℚ := |a.1 - b.1|

/-- **The quantitative version.** The two successors the view would need
    are at maximal distance, so any single prediction is wrong by at least
    half that distance — the minimax fiber defect is ½, which is the
    measured δ. -/
theorem fiber_defect_half : tv v₂ v₄ = 1 := by
  simp [tv, v₂, v₄]

/-- Best-single-prediction error: for any candidate `y`, the worse of its
    two errors is at least ½. This is the exact sense in which "no
    memoryless model does better", replacing a comparison against one
    chosen model. -/
theorem minimax_error_at_least_half (y : View) :
    (1/2 : ℚ) ≤ max (tv y v₂) (tv y v₄) := by
  rcases le_total y.1 (1/2 : ℚ) with h | h
  · have : (1/2 : ℚ) ≤ tv y v₄ := by
      simp only [tv, v₄]
      rw [abs_sub_comm]
      rw [abs_of_nonneg (by linarith)]
      linarith
    exact le_trans this (le_max_right _ _)
  · have : (1/2 : ℚ) ≤ tv y v₂ := by
      simp only [tv, v₂]
      rw [abs_of_nonneg (by linarith)]
      linarith
    exact le_trans this (le_max_left _ _)

end CIRISHolon.Closure
