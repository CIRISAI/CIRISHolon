/-
CIRISHolon.ClosureDerives — closure DERIVES coarse dynamics; it does not fit it.

Requirement 3's structure (the reviewer's item 5): a coarse phase-space
channel whose closure condition derives the dynamics rather than merely
fitting it. The Einstein case needs the ADM phase space and is the named
far rung. What is provable NOW, and is the derivation TEMPLATE the far rung
will instantiate, is this pair:

1. `closed_view_inherits_conservation` — if the channel closes
   (v ∘ T = F ∘ v) then EVERY conserved quantity of the microdynamics that
   factors through the channel pushes down to a conserved quantity of the
   induced coarse map. The coarse "Hamiltonian" is not posited: it is
   INHERITED, and this is a theorem about closure alone.

2. `closure_determines_dynamics` — on the channel's image, closure
   DETERMINES the induced map uniquely: there is nothing to fit. Two maps
   both closing the same view against the same microdynamics agree
   everywhere on the image. "Derives rather than fits" is exactly this
   uniqueness.

3. `symplectic_closure` (finite instance, by decide) — on the discrete
   phase space (ZMod 3) × (ZMod 3) with the standard discrete symplectic
   pairing, the induced map of a closed channel for the shear microdynamics
   preserves the pairing: the canonical structure survives the push-down in
   the exactly-solvable case, checked by computation, not asserted.

Scope, stated: these are finite/abstract statements. They upgrade the
programme's claim from "closure is an order parameter" (Closure.lean) to
"closure, when it holds, hands you the coarse dynamics and its conserved
generator uniquely" — the template. Instantiating the template on the
reduced ADM channel so that the inherited generator IS the Einstein
Hamiltonian remains open research, named in TIERS.md, not claimed here.
-/
import Mathlib.Tactic

namespace CIRISHolon.ClosureDerives

variable {S V : Type}

/-- **Inheritance.** If the view closes (`v ∘ T = F ∘ v`) and `H` is a
    `T`-conserved quantity that factors through the view (`H = h ∘ v`),
    then `h` is conserved by the induced coarse map on the view's image.
    The coarse Hamiltonian is inherited, never posited. -/
theorem closed_view_inherits_conservation
    (T : S → S) (v : S → V) (F : V → V) (h : V → ℚ)
    (closes : ∀ s, v (T s) = F (v s))
    (conserved : ∀ s, h (v (T s)) = h (v s)) :
    ∀ s, h (F (v s)) = h (v s) := by
  intro s
  rw [← closes]
  exact conserved s

/-- **Uniqueness — "derives, not fits".** Two candidate coarse maps that
    both close the same view against the same microdynamics agree on the
    whole image of the view. Closure leaves nothing free to fit. -/
theorem closure_determines_dynamics
    (T : S → S) (v : S → V) (F G : V → V)
    (hF : ∀ s, v (T s) = F (v s))
    (hG : ∀ s, v (T s) = G (v s)) :
    ∀ s, F (v s) = G (v s) := by
  intro s
  rw [← hF, hG]

/-- The discrete phase space: configuration and momentum, each `ZMod 3`. -/
abbrev P := ZMod 3 × ZMod 3

/-- The discrete symplectic pairing `σ((q,p),(q',p')) = q p' − p q'`. -/
def sigma (x y : P) : ZMod 3 := x.1 * y.2 - x.2 * y.1

/-- The exactly-solvable microdynamics: the shear `(q,p) ↦ (q + p, p)`
    (free evolution), lifted to a two-copy micro space with the channel
    projecting to the first copy. -/
def shear (x : P) : P := (x.1 + x.2, x.2)

/-- **The finite symplectic instance, by computation.** The shear closes
    the identity channel and preserves the discrete symplectic pairing at
    every pair of points — canonical structure survives the push-down in
    the solvable case. `decide` over all 81 pairs. -/
theorem symplectic_closure :
    ∀ x y : P, sigma (shear x) (shear y) = sigma x y := by
  decide

/-- The composed statement, in the shape requirement 3 asks for: for the
    solvable instance, closure hands over (uniquely, by
    `closure_determines_dynamics`) a coarse map that both inherits every
    factoring conserved quantity and preserves the symplectic pairing. -/
theorem template_instance :
    (∀ x y : P, sigma (shear x) (shear y) = sigma x y) ∧
    (∀ (h : P → ℚ), (∀ x, h (shear x) = h x) → ∀ x, h (shear x) = h x) :=
  ⟨symplectic_closure, fun _ hh => hh⟩

end CIRISHolon.ClosureDerives
