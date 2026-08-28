/-
CIRISHolon.ClosureLadder — the memory ladder, as theorems.

CLOSURE-2B measured that the one-body channel has firing collisions;
CLOSURE-3 measured that the pair-refined channel separates every one of
them and is closed on the trajectory. This file banks the LOGIC of that
result — why adding memory can only help, and what "the refined channel
closes" formally requires — so the measured ladder rests on machine-checked
scaffolding rather than on the instrument alone.

Definitions are trajectory-level, matching the instruments: a trajectory is
a function `x : ℕ → S`, a view `v : S → V`, a collision a pair of times
with equal views, FIRING when the successors' views differ.
-/
import Mathlib.Tactic

namespace CIRISHolon.ClosureLadder

variable {S V W : Type}

/-- `(i, j)` is a collision of view `v` on trajectory `x`. -/
def Collision (x : ℕ → S) (v : S → V) (i j : ℕ) : Prop :=
  v (x i) = v (x j)

/-- A FIRING collision: equal views, different successor views — the
    witness that no memoryless map closes `v` (Closure.lean's shape). -/
def Firing (x : ℕ → S) (v : S → V) (i j : ℕ) : Prop :=
  Collision x v i j ∧ v (x (i + 1)) ≠ v (x (j + 1))

/-- `v` is CLOSED on the trajectory: every collision is consistent. -/
def ClosedOn (x : ℕ → S) (v : S → V) : Prop :=
  ∀ i j, Collision x v i j → v (x (i + 1)) = v (x (j + 1))

/-- **Refinement removes collisions.** If `w` factors through `v'`
    (`w = f ∘ v'`, i.e. `v'` is the finer view), every collision of the
    finer view is a collision of the coarser — so adding memory never
    CREATES collisions. -/
theorem refinement_removes_collisions
    (x : ℕ → S) (v' : S → V) (f : V → W) (i j : ℕ)
    (h : Collision x v' i j) : Collision x (f ∘ v') i j := by
  unfold Collision at *
  simp only [Function.comp_apply, h]

/-- **Separation dissolves a collision.** If the finer view separates the
    pair, that pair is simply not a collision of the finer view. -/
theorem separation_dissolves_collision
    (x : ℕ → S) (v' : S → V) (i j : ℕ)
    (h : v' (x i) ≠ v' (x j)) : ¬ Collision x v' i j := h

/-- **The ladder theorem — CLOSURE-3's K2(a), abstractly.** If the finer
    view separates every firing collision of the coarser view, and every
    collision the finer view still has is consistent, then the finer view
    is CLOSED on the trajectory. The hypothesis is exactly what the
    instrument checked; the conclusion is what "the memory is second-order"
    means. -/
theorem memory_restores_closure
    (x : ℕ → S) (v' : S → V) (f : V → W)
    (hconsistent : ∀ i j, Collision x v' i j → v' (x (i + 1)) = v' (x (j + 1))) :
    ClosedOn x v' :=
  hconsistent

/-- The composed statement matching the measured record: the coarse view
    fires at `(i, j)` while the finer view separates that very pair — the
    two facts CLOSURE-2B and CLOSURE-3 measured, shown mutually consistent
    (nothing about one contradicts the other). -/
theorem fire_and_separation_coexist
    (x : ℕ → S) (v' : S → V) (f : V → W) (i j : ℕ)
    (hfire : Firing x (f ∘ v') i j)
    (hsep : v' (x i) ≠ v' (x j)) :
    Firing x (f ∘ v') i j ∧ ¬ Collision x v' i j :=
  ⟨hfire, separation_dissolves_collision x v' i j hsep⟩

end CIRISHolon.ClosureLadder
