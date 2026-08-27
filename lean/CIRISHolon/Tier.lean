/-
CIRISHolon.Tier — the certified boundary as a bundle: a Tier cannot be
constructed without its commuting square, stacking is closed_comp packaged,
and two implementations certifying the same boundary provably agree on every
reachable reading. Reopens the Object namespace so the bundle lives beside
the question it answers.
-/
import CIRISHolon.Object
import Mathlib.Tactic

namespace CIRISHolon.Object

/-- A certified tier boundary: a view, the fine step, the coarse update, and
    the commuting square as a FIELD — a `Tier` cannot be constructed without
    its certificate. The budgeted (approximate) form adds `Budget` fields when
    the API meets engine code; this is the exact kernel. -/
structure Tier (X C : Type*) where
  view : X → C
  step : X → X
  rate : C → C
  certifies : view ∘ step = rate ∘ view

namespace Tier

variable {X C D : Type*}

/-- The bundle witnesses `Closed`, definitionally. -/
theorem closed (t : Tier X C) : Closed t.view t.step :=
  ⟨t.rate, t.certifies⟩

/-- Pointwise form of the certificate. -/
theorem certifies_apply (t : Tier X C) (x : X) :
    t.view (t.step x) = t.rate (t.view x) := congrFun t.certifies x

/-- The trivial tier: every step certifies itself under the identity view. -/
def refl (T : X → X) : Tier X X := ⟨id, T, T, rfl⟩

/-- **Stacking**: a tier over a tier is a tier — the coarse tier's fine step
    must be the fine tier's coarse update, and the composite certificate is
    `closed_comp`. The eight-tier ladder is `stack` applied seven times. -/
def stack (lo : Tier X C) (hi : Tier C D) (h : hi.step = lo.rate) :
    Tier X D where
  view := hi.view ∘ lo.view
  step := lo.step
  rate := hi.rate
  certifies := by
    have hv := lo.certifies
    have hu := hi.certifies
    rw [h] at hu
    calc (hi.view ∘ lo.view) ∘ lo.step
        = hi.view ∘ (lo.view ∘ lo.step) := rfl
      _ = hi.view ∘ (lo.rate ∘ lo.view) := by rw [hv]
      _ = (hi.view ∘ lo.rate) ∘ lo.view := rfl
      _ = (hi.rate ∘ hi.view) ∘ lo.view := by rw [hu]
      _ = hi.rate ∘ (hi.view ∘ lo.view) := rfl

@[simp] theorem stack_view (lo : Tier X C) (hi : Tier C D) (h : hi.step = lo.rate) :
    (lo.stack hi h).view = hi.view ∘ lo.view := rfl

@[simp] theorem stack_rate (lo : Tier X C) (hi : Tier C D) (h : hi.step = lo.rate) :
    (lo.stack hi h).rate = hi.rate := rfl

/-- Two tiers over the same (view, step) agree on every reachable reading:
    the coarse simulator is determined, not designed. -/
theorem rate_agree (t t' : Tier X C) (hv : t.view = t'.view)
    (hs : t.step = t'.step) (x : X) :
    t.rate (t.view x) = t'.rate (t.view x) := by
  have h1 := t.certifies_apply x
  have h2 := t'.certifies_apply x
  rw [← hv, ← hs] at h2
  rw [← h1, ← h2]

end Tier

end CIRISHolon.Object
