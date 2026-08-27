/-
CIRISHolon.Tiers — the algebra of stacking.

What makes an eight-tier ladder sound is not eight certifications of one
predicate; it is that certified squares COMPOSE (`closed_comp`, `Closed.comp`),
that the coarse dynamics at each boundary is DETERMINED rather than designed
(`rate_unique_on_range` — two implementations that both certify must agree on
every reachable reading), and that chart switching commutes with simulation
wherever the loop is a symmetry (`holonomy_commutes_with_rate`,
`curvature_iff_held`). Transplanted from CIRISAI/CIRISOntology; the stacking
theorem is stated here explicitly because the engine uses it seven times.
-/
import CIRISHolon.Object
import Mathlib.Tactic

namespace CIRISHolon.Tiers

open CIRISHolon.Object


/-- Every step closes SOME view — itself. "Is a tier" is empty as a unary
    predicate; the engineering content is always the RELATION (v, T, budget). -/
theorem exists_closed_view (T : X → X) : Closed T T := ⟨T, rfl⟩

/-- An invariant reading is in particular a Closed one. -/
theorem held_imp_closed {v : X → C} {T : X → X} (h : Held v T) :
    Closed v T := ⟨id, by rw [h]; rfl⟩

/-- **The coarse dynamics is DETERMINED, never designed**: any two updates
    witnessing closure agree on the view's range. A tier's simulator is a
    consequence of (T, v), and two implementations that both certify must
    agree on every reachable reading. -/
theorem rate_unique_on_range {v : X → C} {T : X → X} {φ ψ : C → C}
    (h₁ : v ∘ T = φ ∘ v) (h₂ : v ∘ T = ψ ∘ v) (x : X) : φ (v x) = ψ (v x) :=
  congrFun (h₁.symm.trans h₂) x

/-- **Tier stacking is sound.** If `v` is Closed under `T` with update `h`,
    and `u` is Closed under `h` with update `g`, then the composite view
    `u ∘ v` is Closed under `T` with update `g`. The eight-tier ladder is
    this theorem applied seven times — each boundary certified separately,
    the stack sound by composition. -/
theorem closed_comp {D : Type*} {v : X → C} {u : C → D} {T : X → X}
    {h : C → C} {g : D → D}
    (hv : v ∘ T = h ∘ v) (hu : u ∘ h = g ∘ u) :
    (u ∘ v) ∘ T = g ∘ (u ∘ v) := by
  calc (u ∘ v) ∘ T = u ∘ (v ∘ T) := rfl
    _ = u ∘ (h ∘ v) := by rw [hv]
    _ = (u ∘ h) ∘ v := rfl
    _ = (g ∘ u) ∘ v := by rw [hu]
    _ = g ∘ (u ∘ v) := rfl

/-- The existential form: a stack of certified tiers is a certified tier. -/
theorem Closed.comp {D : Type*} {v : X → C} {T : X → X}
    (hv : Closed v T) {u : C → D} (hu : ∀ h : C → C, v ∘ T = h ∘ v → Closed u h) :
    Closed (u ∘ v) T := by
  obtain ⟨h, hh⟩ := hv
  obtain ⟨g, hg⟩ := hu h hh
  exact ⟨g, closed_comp hh hg⟩

/-- **Chart-loop consistency commutes with simulation.** If a re-root loop is
    a symmetry of the step and the view is Closed, the loop's transport
    commutes with the tier's update on every reachable reading — switching
    charts and stepping may be done in either order. -/
theorem holonomy_commutes_with_rate
    {v : X → C} {T rloop : X → X} {γ h : C → C}
    (hcarry  : v ∘ rloop = γ ∘ v)
    (hclosed : v ∘ T = h ∘ v)
    (heqv    : T ∘ rloop = rloop ∘ T)
    (x : X) : γ (h (v x)) = h (γ (v x)) := by
  have hc : ∀ y, γ (v y) = v (rloop y) := fun y => (congrFun hcarry y).symm
  have hr : ∀ y, h (v y) = v (T y)     := fun y => (congrFun hclosed y).symm
  have he : ∀ y, T (rloop y) = rloop (T y) := fun y => congrFun heqv y
  rw [hr, hc, ← he x, hc, hr]

/-- Zero chart-loop holonomy is exactly `Held` on the context axis: the
    round-trip reading agrees with the direct one for every state. -/
theorem curvature_iff_held
    {v : X → C} {rloop : X → X} {γ : C → C}
    (hcarry : v ∘ rloop = γ ∘ v) :
    (∀ x : X, γ (v x) = v x) ↔ v ∘ rloop = v := by
  constructor
  · intro hfix; funext x; exact (congrFun hcarry x).trans (hfix x)
  · intro hheld x; exact ((congrFun hcarry x).symm.trans (congrFun hheld x))

end CIRISHolon.Tiers
