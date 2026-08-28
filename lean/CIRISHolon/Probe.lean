/-
CIRISHolon.Probe — rung 3, the provable half: same Ω ⇒ probe-agreement.

A PROBE of a holon is anything computed from what the holon exposes: a
function of the view's value and its dynamical images. The easy direction
of representational completeness is a theorem: two systems with the SAME
Ω-data (an isomorphism of views intertwining the dynamics) agree on every
probe. The HARD direction — probe-agreement forces same Ω — is rung 3's
open question, staked for a counterexample hunt on exact models, and is
deliberately NOT asserted here. If a counterexample exists (two systems
every admissible probe treats identically that are NOT Ω-isomorphic), the
hunt finds it; if the converse is provable, this file is where its proof
will live. Neither outcome is presumed.
-/
import Mathlib.Tactic
import CIRISHolon.Omega

namespace CIRISHolon.Probe

open CIRISHolon.Omega

/-- A probe: any observable computed from the view along the trajectory —
    depth-`n` probes read the view's value after `n` steps. -/
def probe {S V : Type} (H : Holon S V) (n : ℕ) (s : S) : V :=
  H.view (H.dyn^[n] s)

/-- An Ω-isomorphism: a state bijection and a view bijection that
    intertwine both the views and the dynamics. -/
structure OmegaIso {S₁ S₂ V₁ V₂ : Type}
    (H₁ : Holon S₁ V₁) (H₂ : Holon S₂ V₂) where
  σ : S₁ ≃ S₂
  τ : V₁ ≃ V₂
  view_eq : ∀ s, H₂.view (σ s) = τ (H₁.view s)
  dyn_eq : ∀ s, H₂.dyn (σ s) = σ (H₁.dyn s)

/-- Iterates transport along the isomorphism. -/
theorem iter_transport {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂} (φ : OmegaIso H₁ H₂) :
    ∀ (n : ℕ) (s : S₁), H₂.dyn^[n] (φ.σ s) = φ.σ (H₁.dyn^[n] s) := by
  intro n
  induction n with
  | zero => intro s; rfl
  | succ k ih =>
      intro s
      rw [Function.iterate_succ_apply, Function.iterate_succ_apply,
          φ.dyn_eq, ih]

/-- **The provable half of representational completeness.** Ω-isomorphic
    systems agree on EVERY depth-`n` probe, up to the view identification:
    no admissible probe can tell them apart. -/
theorem same_omega_implies_probe_agreement {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂} (φ : OmegaIso H₁ H₂) :
    ∀ (n : ℕ) (s : S₁), probe H₂ n (φ.σ s) = φ.τ (probe H₁ n s) := by
  intro n s
  unfold probe
  rw [iter_transport φ n s, φ.view_eq]

end CIRISHolon.Probe
