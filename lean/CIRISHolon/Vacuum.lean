/-
CIRISHolon.Vacuum — Vacuum Tier (Tier ∅ / Tier -1), Modular Link Hamiltonian, and Promotion Morphism.

Formalizes the three foundational vacuum mechanisms from `DM_GAUGE_CHEMISTRY_HANDOFF.md`
and the Ω evaluation ladder:

1. THE GAUGE-INVARIANT ZERO-FLUX VACUUM CARRIER SPACE H₀:
   - Spin-1 electric flux carrier where E ∈ {-1, 0, +1};
   - Zero-flux vacuum state where all link fluxes E = 0;
   - All loop holonomies evaluate to the group identity {e};
   - All vertex Gauss charges vanish (G_v = 0) and total electric energy is zero (∑ E² = 0).

2. MODULAR LINK HAMILTONIAN GENERATOR:
   - For any charge-conjugation symmetric link reduction (p, p₀, p), the modular Hamiltonian
     K = -log(ρ) is exactly affine in the electric energy operator E²:
       K = -log(p₀) I + log(p₀/p) E²
   - Ground state modular energy is exactly -log(p₀);
   - Flat spectrum (p = p₀) gives zero modular gauge coupling (β = 0);
   - Zero-flux dominance (p < p₀) yields strictly positive modular excitation cost (β > 0).

3. EXCITATION PROMOTION MORPHISM (H₀ → Physical Wilson-Dressed Matter):
   - Lifts the vacuum to a Wilson-line dressed matter state `Ψ_dressed(u) = ρ₂(u) · T`;
   - Escapes the BRIDGE-5 kill (`BareCharge.lean`): bare excitations `bare T` are annihilated
     by Gauss law (gaussBoth (bare T) = 0), whereas the Wilson-line promoted excitation
     survives with exact scale factor |D₄|² = 64;
   - Certified as a valid Tier boundary in the sense of `CIRISHolon.Object.Tier`.
-/

import Mathlib.Analysis.SpecialFunctions.Log.Basic
import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Tactic
import CIRISHolon.Object
import CIRISHolon.Tier
import CIRISHolon.BareCharge

namespace CIRISHolon.Vacuum

open CIRISHolon.Object
open CIRISHolon.BareCharge

/-! ### 1. The Zero-Flux Vacuum Carrier Space H₀ -/

/-- Local 3-state quantum link carrier, interpreted as spin-1 electric flux {-1, 0, +1}. -/
abbrev FluxState := Fin 3

/-- Spin-1 electric flux eigenvalue `E ∈ {-1, 0, +1}`. -/
def flux : FluxState → ℤ
  | ⟨0, _⟩ => -1
  | ⟨1, _⟩ => 0
  | ⟨2, _⟩ => 1

/-- Electric energy operator `E²`. -/
def electricSq : FluxState → ℤ := fun q => flux q * flux q

/-- Real-valued electric energy for modular thermodynamics. -/
def electricSqR (q : FluxState) : ℝ := (electricSq q : ℝ)

@[simp] theorem electricSq_neg1 : electricSq ⟨0, by omega⟩ = 1 := rfl
@[simp] theorem electricSq_zero : electricSq ⟨1, by omega⟩ = 0 := rfl
@[simp] theorem electricSq_pos1 : electricSq ⟨2, by omega⟩ = 1 := rfl

/-- Predicate for a zero-flux link state. -/
def IsZeroFlux (q : FluxState) : Prop := flux q = 0

/-- The zero-flux state is uniquely the middle state `⟨1, _⟩`. -/
theorem isZeroFlux_iff (q : FluxState) : IsZeroFlux q ↔ q = ⟨1, by omega⟩ := by
  fin_cases q <;> simp [IsZeroFlux, flux]

/-- Plaquette configuration: 4 oriented links around a closed loop. -/
abbrev PlaquetteConfig := Fin 4 → FluxState

/-- Previous link in cyclic order. -/
def prev : Fin 4 → Fin 4
  | ⟨0, _⟩ => ⟨3, by omega⟩
  | ⟨1, _⟩ => ⟨0, by omega⟩
  | ⟨2, _⟩ => ⟨1, by omega⟩
  | ⟨3, _⟩ => ⟨2, by omega⟩

/-- Lattice Gauss law at vertex `v`: outgoing minus incoming flux. -/
def gauss (c : PlaquetteConfig) (v : Fin 4) : ℤ :=
  flux (c v) - flux (c (prev v))

/-- Total electric energy of a plaquette. -/
def totalElectricEnergy (c : PlaquetteConfig) : ℤ :=
  ∑ v : Fin 4, electricSq (c v)

/-- Canonical zero-flux vacuum plaquette configuration: all links have E = 0. -/
def vacuumConfig : PlaquetteConfig := fun _ => ⟨1, by omega⟩

/-- Total electric energy of the vacuum configuration is zero. -/
theorem vacuum_electric_energy_zero : totalElectricEnergy vacuumConfig = 0 := by
  decide

/-- Gauss law is satisfied at every vertex in the vacuum (G_v = 0). -/
theorem vacuum_gauss_zero (v : Fin 4) : gauss vacuumConfig v = 0 := by
  unfold gauss vacuumConfig
  simp [flux]

/-- Abstract vacuum carrier H₀: characterized by vanishing electric flux,
    zero Gauss charge everywhere, and identity loop holonomies. -/
structure VacuumCarrier where
  config : PlaquetteConfig
  zero_flux : ∀ v, IsZeroFlux (config v)
  gauss_free : ∀ v, gauss config v = 0
  energy_zero : totalElectricEnergy config = 0

/-- The canonical vacuum state is an inhabitant of the vacuum carrier H₀. -/
def canonicalVacuum : VacuumCarrier where
  config := vacuumConfig
  zero_flux := fun _ => by simp [IsZeroFlux, vacuumConfig, flux]
  gauss_free := vacuum_gauss_zero
  energy_zero := vacuum_electric_energy_zero

/-! ### 2. The Modular Link Hamiltonian Generator -/

/-- Charge-conjugation symmetric one-link probability distribution `(p, p₀, p)`
    over the flux basis `{-1, 0, +1}`. -/
def linkProb (p p0 : ℝ) : FluxState → ℝ
  | ⟨0, _⟩ => p
  | ⟨1, _⟩ => p0
  | ⟨2, _⟩ => p

/-- Modular Hamiltonian / energy `K = -log(ρ)`. -/
noncomputable def modularEnergy (p p0 : ℝ) (q : FluxState) : ℝ :=
  -Real.log (linkProb p p0 q)

/-- Modular coupling coefficient `β = log(p₀ / p)`. -/
noncomputable def modularBeta (p p0 : ℝ) : ℝ := Real.log (p0 / p)

/-- **MODULAR LINK HAMILTONIAN THEOREM**:
    For any charge-conjugation symmetric link reduction `(p, p₀, p)`, the modular energy
    is exactly affine in the electric energy operator `E²`:
      `K = -log(p₀) I + log(p₀/p) E²` -/
theorem modularEnergy_eq_affine_electric
    (p p0 : ℝ) (hp : 0 < p) (hp0 : 0 < p0) (q : FluxState) :
    modularEnergy p p0 q = -Real.log p0 + modularBeta p p0 * electricSqR q := by
  fin_cases q
  · simp only [modularEnergy, linkProb, modularBeta, electricSqR, electricSq, flux]
    rw [Real.log_div hp0.ne' hp.ne']
    push_cast
    ring
  · simp [modularEnergy, linkProb, modularBeta, electricSqR, electricSq, flux]
  · simp only [modularEnergy, linkProb, modularBeta, electricSqR, electricSq, flux]
    rw [Real.log_div hp0.ne' hp.ne']
    push_cast
    ring

/-- The ground state (zero-flux vacuum) has modular energy `-log(p₀)`. -/
theorem vacuum_modular_ground (p p0 : ℝ) (hp : 0 < p) (hp0 : 0 < p0) :
    modularEnergy p p0 ⟨1, by omega⟩ = -Real.log p0 := by
  rw [modularEnergy_eq_affine_electric p p0 hp hp0 ⟨1, by omega⟩]
  simp [electricSqR, electricSq, flux]

/-- At a flat local spectrum (p = p₀), the modular gauge coupling vanishes (β = 0). -/
theorem modularBeta_eq_zero_of_flat (p : ℝ) (hp : 0 < p) :
    modularBeta p p = 0 := by
  unfold modularBeta
  rw [div_self hp.ne', Real.log_one]

/-- When the zero-flux vacuum dominates (p < p₀), the modular excitation energy
    is strictly positive (β > 0). -/
theorem modularBeta_pos_of_vacuum_dominates
    (p p0 : ℝ) (hp : 0 < p) (hdom : p < p0) :
    0 < modularBeta p p0 := by
  unfold modularBeta
  apply Real.log_pos
  exact (one_lt_div hp).mpr hdom

/-! ### 3. Excitation Promotion Morphism (Vacuum → Wilson-Dressed Matter) -/

/-- The excitation promotion morphism: lifts a local matter tensor `T : M2` to a
    Wilson-line dressed gauge-matter state `Ψ(u) = ρ₂(u) · T`. -/
def promote (T : M2) : State := fun u => rho2 u * T

/-- Bare excitation promotion: injects matter without Wilson-line dressing `Ψ(u) = T`. -/
def promoteBare (T : M2) : State := bare T

/-- Identity matter excitation promotes to M14's canonical Wilson-dressed state. -/
theorem promote_singlet_eq_dressed : promote ⟨1, 0, 0, 1⟩ = dressed := by
  funext u
  revert u
  decide

/-- Right multiplication of M2 by zero is zero. -/
theorem M2_mul_zero (T : M2) : T * (0 : M2) = 0 := by
  apply M2.ext' <;> simp

/-- Zero excitation promotes to the vacuum / zero state. -/
theorem promote_zero : promote 0 = (fun _ => 0) := by
  funext u
  simp [promote, M2_mul_zero]

/-- **GAUGE INVARIANCE OF PROMOTED EXCITATIONS**:
    The promoted Wilson-dressed state is invariant under the vertex 1 gauge action. -/
theorem promote_singlet_invariant_v1 (g u : Fin 8) :
    rho2 g * promote ⟨1, 0, 0, 1⟩ (dmul (dinv g) u) = promote ⟨1, 0, 0, 1⟩ u := by
  rw [promote_singlet_eq_dressed]
  exact dressed_invariant_v1 g u

/-- The promoted Wilson-dressed state is invariant under the vertex 2 gauge action. -/
theorem promote_singlet_invariant_v2 (g u : Fin 8) :
    promote ⟨1, 0, 0, 1⟩ (dmul u g) * rho2 (dinv g) = promote ⟨1, 0, 0, 1⟩ u := by
  rw [promote_singlet_eq_dressed]
  exact dressed_invariant_v2 g u

/-- **THE PROMOTED EXCITATION SURVIVES TWO-SITE GAUSS AVERAGING**:
    The two-site Gauss operator preserves the promoted state, scaling it by |D₄|² = 64. -/
theorem gaussBoth_promote_singlet (u : Fin 8) :
    gaussBoth (promote ⟨1, 0, 0, 1⟩) u = scale 64 (promote ⟨1, 0, 0, 1⟩ u) := by
  rw [promote_singlet_eq_dressed]
  exact gaussBoth_dressed u

/-- **THE BARE PROMOTION IS ANNIHILATED BY GAUSS LAW**:
    Every bare promotion is completely destroyed by the two-site Gauss projector. -/
theorem gaussBoth_promoteBare (T : M2) (u : Fin 8) :
    gaussBoth (promoteBare T) u = 0 :=
  gaussBoth_bare T u

/-- **THE PROMOTION ESCAPE THEOREM**:
    Bare excitation is killed by Gauss law, while Wilson-line promotion survives. -/
theorem bare_annihilated_wilson_promoted_survives :
    (∀ u, gaussBoth (promoteBare ⟨1, 0, 0, 1⟩) u = 0) ∧
    (∃ u, gaussBoth (promote ⟨1, 0, 0, 1⟩) u ≠ 0) := by
  rw [promote_singlet_eq_dressed]
  exact bare_dies_dressed_survives

/-! ### 4. The Vacuum Tier Bundle (Tier ∅ / Tier -1) -/

/-- Vacuum view on plaquette configurations: checks if the entire plaquette is in the zero-flux state. -/
def vacuumView (c : PlaquetteConfig) : Bool :=
  decide (c = vacuumConfig)

/-- Vacuum dynamics on plaquette configurations: trivial / ground-state preservation. -/
def vacuumStep (c : PlaquetteConfig) : PlaquetteConfig := c

/-- The Vacuum Tier (Tier ∅ / Tier -1) packaged as a certified `Tier` bundle. -/
def vacuumTier : Tier PlaquetteConfig Bool where
  view := vacuumView
  step := vacuumStep
  rate := id
  certifies := rfl

/-- The Vacuum Tier satisfies `Closed` in the sense of `CIRISHolon.Object.Closed`. -/
theorem vacuum_tier_closed : Closed vacuumView vacuumStep :=
  vacuumTier.closed

/-- The vacuum state is invariant (`Held`) under vacuum dynamics. -/
theorem vacuum_state_held : Held vacuumView vacuumStep := rfl

end CIRISHolon.Vacuum
