/-
CIRISHolon.Grain — the revival law: why grains exist.

The bridge campaigns measured ω = 0,1,0,1 and δ = 0,½,0,½ on exact Floquet
gauge dynamics. The kernel of both sequences is one algebraic fact: for an
involution R (R² = 1) and a commuting imaginary unit i (i² = −1), the
electric step's numerator satisfies

    (1 + i·R)² = 2·i·R      and      (1 + i·R)⁴ = −4,

so the normalized step U = (1+iR)/√2 obeys U² = iR (a deterministic toggle
— the classical view momentarily exact) and U⁴ = −1 (revival up to global
phase — the coarse view exact again, the state recurred). GRAIN LAW: a
coarse view of involution-generated dynamics closes exactly at the k where
U^k is scalar-or-toggle, and those k are the engine's grain boundaries —
where coarse tiers may refresh with zero defect and exact checkpoints are
cheapest. The period is the coupling's parameter, never a universal clock.
-/
import Mathlib.Tactic.Ring

namespace CIRISHolon.Grain

variable {A : Type*} [CommRing A]

/-- The square of the electric numerator: a pure toggle. -/
theorem step_sq (i R : A) (hi : i ^ 2 = -1) (hR : R ^ 2 = 1) :
    (1 + i * R) ^ 2 = 2 * (i * R) := by
  have : (1 + i * R) ^ 2 = 1 + 2 * (i * R) + i ^ 2 * R ^ 2 := by ring
  rw [this, hi, hR]; ring

/-- The fourth power: a scalar — revival up to global phase. -/
theorem revival_four (i R : A) (hi : i ^ 2 = -1) (hR : R ^ 2 = 1) :
    (1 + i * R) ^ 4 = -4 := by
  have h2 := step_sq i R hi hR
  have : (1 + i * R) ^ 4 = ((1 + i * R) ^ 2) ^ 2 := by ring
  rw [this, h2]
  have : (2 * (i * R)) ^ 2 = 4 * (i ^ 2 * R ^ 2) := by ring
  rw [this, hi, hR]; ring

/-- Eighth power: full identity-scale recurrence — the outer grain. -/
theorem recurrence_eight (i R : A) (hi : i ^ 2 = -1) (hR : R ^ 2 = 1) :
    (1 + i * R) ^ 8 = 16 := by
  have h4 := revival_four i R hi hR
  have : (1 + i * R) ^ 8 = ((1 + i * R) ^ 4) ^ 2 := by ring
  rw [this, h4]; ring

end CIRISHolon.Grain
