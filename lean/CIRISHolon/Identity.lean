/-
CIRISHolon.Identity — rung 2 closed by rung 3's fork: the commitment.

`ProbeConverse` proves Ω's raw data exceeds its probe behaviour by exactly
fiber-internal structure; CROSS-FACE-1 measured that the cost face does
not see that excess either (the counterexample holons share a view
transfer, hence a rent). The commitment, recorded as a DEFINITION rather
than an ambiguity: **holon identity is the observable (Moore) quotient** —
fiber-internal dynamics is GAUGE, exactly as gauge theory treats
orbit-internal data. Relative to the CURRENT tuple of faces: a future face
that measures fibers would REFINE this identity, and the definitions are
written so that refinement extends rather than contradicts.
-/
import Mathlib.Tactic
import CIRISHolon.Omega
import CIRISHolon.Probe

namespace CIRISHolon.Identity

open CIRISHolon.Omega CIRISHolon.Probe

/-- Probe-equivalence relative to chosen identifications: matched states
    give matched probe streams at every depth — the equivalence the
    counterexample shows is COARSER than Ω-isomorphism. -/
def ProbeEquiv {S₁ S₂ V₁ V₂ : Type} (H₁ : Holon S₁ V₁) (H₂ : Holon S₂ V₂)
    (σ : S₁ ≃ S₂) (τ : V₁ ≃ V₂) : Prop :=
  ∀ (n : ℕ) (s : S₁),
    H₂.view (H₂.dyn^[n] (σ s)) = τ (H₁.view (H₁.dyn^[n] s))

/-- **The identity commitment**: holons are THE SAME when probe-equivalent
    under some identifications. Fiber-internal structure is gauge. By
    `ProbeConverse` this is strictly coarser than Ω-isomorphism in general
    and coincides with it exactly on observable holons. -/
def SameHolon {S₁ S₂ V₁ V₂ : Type} (H₁ : Holon S₁ V₁) (H₂ : Holon S₂ V₂) : Prop :=
  ∃ (σ : S₁ ≃ S₂) (τ : V₁ ≃ V₂), ProbeEquiv H₁ H₂ σ τ

/-- Ω-isomorphic holons are the same holon — identity is never finer than
    what the tuple's faces can see. -/
theorem sameHolon_of_omegaIso {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂}
    (φ : OmegaIso H₁ H₂) : SameHolon H₁ H₂ :=
  ⟨φ.σ, φ.τ, fun n s => same_omega_implies_probe_agreement φ n s⟩

end CIRISHolon.Identity
