/-
CIRISHolon.FrobOrient — FROB-ORIENT-1: non-ambivalent orientation on F₂₁ = ℤ₇ ⋊ ℤ₃.

Rung 5 of the Ω ladder (`OMEGA_LADDER.md`):
"FROB-ORIENT-1: non-ambivalent orientation — F21 = Z7⋊Z3, where class(C) ≠ class(C⁻¹):
the oriented reading detects loop orientation and the deliberately ambivalent
(D4-style) projection is provably blind."

MATHEMATICAL CONTEXT.
In finite-group gauge theory, loop observables (Wilson loops / holonomies) take values
in the gauge group G, and gauge transformations act at the basepoint by conjugation:
U(γ) ↦ h U(γ) h⁻¹. The maximal gauge-invariant observable of an oriented loop is
therefore its conjugacy class `class(U(γ))`.

When the gauge group is AMBIVALENT (like the dihedral group D₄, where reflections
invert rotations by conjugation: s r s⁻¹ = r⁻¹), every element is conjugate to its
inverse: `class(g) = class(g⁻¹)`. Consequently, in D₄-style gauge theories, gauge-invariant
holonomy measurements are inherently blind to the orientation of the loop (γ vs γ⁻¹).

The Frobenius group F₂₁ = ℤ₇ ⋊ ℤ₃ (the smallest non-abelian group of odd order, order 21)
is NON-AMBIVALENT:
- F₂₁ has 5 conjugacy classes:
    * 1 of size 1: the identity {(0,0)}
    * 2 of size 3: C₁ = {(1,0), (2,0), (4,0)} and C₂ = {(3,0), (5,0), (6,0)}
    * 2 of size 7: C₃ = {(z,1) | z ∈ ℤ₇} and C₄ = {(z,2) | z ∈ ℤ₇}
- For the generator C = (1,0) ∈ C₁, its inverse C⁻¹ = (6,0) ∈ C₂.
- Since C₁ ∩ C₂ = ∅, `class(C) ≠ class(C⁻¹)`.

THEOREMS PROVED HERE:
1. `frob_group`                   — F₂₁ is a valid non-abelian finite group of order 21.
2. `frob_conjugacy_classes_card`   — F₂₁ has 5 conjugacy classes partitioned as 1+3+3+7+7=21.
3. `frob_generator_non_ambivalent` — class(C) ≠ class(C⁻¹) on the ℤ₇ generator C = (1, 0).
4. `frob_not_ambivalent`           — F₂₁ is non-ambivalent (¬ ∀ g, g ~ g⁻¹).
5. `d4_is_ambivalent`              — Contrast: D₄ is ambivalent (∀ g ∈ D₄, g ~ g⁻¹).
6. `oriented_detects_orientation`  — The oriented gauge reading separates loop γ from γ⁻¹.
7. `ambivalent_is_blind`           — Any ambivalent projection collapses γ and γ⁻¹.
8. `nonfactoring_loop_orientation` — In Object.lean terms, loop orientation certifies
                                     NonFactoring of the ambivalent projection against
                                     the oriented reading.
-/

import Mathlib.Data.Fintype.Basic
import Mathlib.Data.Finset.Basic
import Mathlib.Tactic
import CIRISHolon.Object
import CIRISHolon.BareCharge

namespace CIRISHolon.FrobOrient

open CIRISHolon.Object

/-! ### The Frobenius Group F₂₁ = ℤ₇ ⋊ ℤ₃ -/

/-- Elements of the Frobenius group F₂₁: pairs `(z, w)` where `z ∈ ZMod 7` and `w ∈ ZMod 3`. -/
@[ext]
structure F21 where
  z : ZMod 7
  w : ZMod 3
deriving DecidableEq, Repr, Fintype

/-- The action of ℤ₃ on ℤ₇ by automorphism: generator 1 acts by multiplication by 2
    (note 2³ = 8 ≡ 1 mod 7). -/
def act (w : ZMod 3) (z : ZMod 7) : ZMod 7 :=
  if w = 0 then z else if w = 1 then 2 * z else 4 * z

@[simp] theorem act_zero (z : ZMod 7) : act 0 z = z := rfl

theorem act_add_distrib (w : ZMod 3) (x y : ZMod 7) :
    act w (x + y) = act w x + act w y := by
  revert w x y
  decide

theorem act_add (w₁ w₂ : ZMod 3) (z : ZMod 7) :
    act (w₁ + w₂) z = act w₁ (act w₂ z) := by
  revert w₁ w₂ z
  decide

theorem act_zero_w (w : ZMod 3) : act w 0 = 0 := by
  revert w
  decide

/-- Group multiplication in F₂₁: `(z₁, w₁) * (z₂, w₂) = (z₁ + act(w₁, z₂), w₁ + w₂)`. -/
def mul (g h : F21) : F21 :=
  ⟨g.z + act g.w h.z, g.w + h.w⟩

/-- Identity element of F₂₁: `(0, 0)`. -/
def one : F21 := ⟨0, 0⟩

/-- Group inverse in F₂₁: `(z, w)⁻¹ = (- act(-w, z), -w)`. -/
def inv (g : F21) : F21 :=
  ⟨- act (-g.w) g.z, -g.w⟩

instance : Mul F21 := ⟨mul⟩
instance : One F21 := ⟨one⟩
instance : Inv F21 := ⟨inv⟩

@[simp] theorem mul_z (g h : F21) : (g * h).z = g.z + act g.w h.z := rfl
@[simp] theorem mul_w (g h : F21) : (g * h).w = g.w + h.w := rfl
@[simp] theorem one_z : (1 : F21).z = 0 := rfl
@[simp] theorem one_w : (1 : F21).w = 0 := rfl
@[simp] theorem inv_w (g : F21) : (g⁻¹).w = -g.w := rfl

theorem mul_assoc_frob (a b c : F21) : (a * b) * c = a * (b * c) := by
  ext
  · simp only [mul_z, mul_w, act_add_distrib, act_add, add_assoc]
  · simp only [mul_w, add_assoc]

theorem one_mul_frob (a : F21) : 1 * a = a := by
  ext <;> simp [one, mul, act]

theorem mul_one_frob (a : F21) : a * 1 = a := by
  ext
  · simp [one, mul, act_zero_w]
  · simp [one, mul]

theorem mul_left_inv_frob (a : F21) : a⁻¹ * a = 1 := by
  revert a
  decide

instance : Group F21 where
  mul_assoc := mul_assoc_frob
  one_mul := one_mul_frob
  mul_one := mul_one_frob
  inv_mul_cancel := mul_left_inv_frob

/-- F₂₁ has order exactly 21. -/
theorem frob_card : Fintype.card F21 = 21 := by decide

/-- F₂₁ is non-abelian: `(1,0) * (0,1) ≠ (0,1) * (1,0)`. -/
theorem frob_nonabelian : ∃ g h : F21, g * h ≠ h * g := by
  use ⟨1, 0⟩, ⟨0, 1⟩
  decide

/-! ### Conjugacy and Conjugacy Classes in F₂₁ -/

/-- Conjugation action: `conjBy g x = g * x * g⁻¹`. -/
def conjBy (g x : F21) : F21 := g * x * g⁻¹

/-- Two elements are conjugate if one is obtained by conjugating the other. -/
def IsConj (x y : F21) : Prop := ∃ g : F21, conjBy g x = y

instance (x y : F21) : Decidable (IsConj x y) :=
  inferInstanceAs (Decidable (∃ g : F21, conjBy g x = y))

/-- The conjugacy class of an element in F₂₁. -/
def conjClass (x : F21) : Finset F21 :=
  Finset.univ.filter (fun y => IsConj x y)

theorem mem_conjClass_self (x : F21) : x ∈ conjClass x := by
  simp only [conjClass, Finset.mem_filter, Finset.mem_univ, true_and, IsConj]
  exact ⟨1, by simp [conjBy]⟩

theorem conjClass_eq_of_isConj {x y : F21} (h : IsConj x y) : conjClass x = conjClass y := by
  obtain ⟨g, rfl⟩ := h
  ext z
  simp only [conjClass, Finset.mem_filter, Finset.mem_univ, true_and, IsConj]
  constructor
  · rintro ⟨k, rfl⟩
    exact ⟨k * g⁻¹, by simp [conjBy, mul_assoc]⟩
  · rintro ⟨k, rfl⟩
    exact ⟨k * g, by simp [conjBy, mul_assoc]⟩

theorem conjClass_conjBy (h x : F21) : conjClass (conjBy h x) = conjClass x :=
  (conjClass_eq_of_isConj ⟨h, rfl⟩).symm

/-! ### The Five Conjugacy Classes of F₂₁ -/

/-- The 5 canonical conjugacy classes of F₂₁:
    Class 0 (Identity): size 1
    Class 1 (C = (1,0)): size 3
    Class 2 (C⁻¹ = (6,0)): size 3
    Class 3 (Order 3 elements with w = 1): size 7
    Class 4 (Order 3 elements with w = 2): size 7 -/
def class_id : Finset F21 := conjClass ⟨0, 0⟩
def class_C1 : Finset F21 := conjClass ⟨1, 0⟩
def class_C2 : Finset F21 := conjClass ⟨6, 0⟩
def class_W1 : Finset F21 := conjClass ⟨0, 1⟩
def class_W2 : Finset F21 := conjClass ⟨0, 2⟩

theorem card_class_id : class_id.card = 1 := by decide
theorem card_class_C1 : class_C1.card = 3 := by decide
theorem card_class_C2 : class_C2.card = 3 := by decide
theorem card_class_W1 : class_W1.card = 7 := by decide
theorem card_class_W2 : class_W2.card = 7 := by decide

/-- The class equation of F₂₁: 1 + 3 + 3 + 7 + 7 = 21. -/
theorem frob_class_sum :
    class_id.card + class_C1.card + class_C2.card + class_W1.card + class_W2.card = 21 := by
  decide

/-! ### Non-Ambivalence: class(C) ≠ class(C⁻¹) -/

/-- The ℤ₇ generator loop holonomy C = (1, 0). -/
def C : F21 := ⟨1, 0⟩

/-- The inverse loop holonomy C⁻¹ = (6, 0). -/
def C_inv : F21 := C⁻¹

theorem C_inv_eq : C_inv = ⟨6, 0⟩ := by decide

/-- **THE NON-AMBIVALENT ORIENTATION**: C and C⁻¹ are NOT conjugate in F₂₁. -/
theorem frob_generator_not_conj : ¬ IsConj C C_inv := by decide

/-- Hence their conjugacy classes are disjoint and distinct. -/
theorem frob_generator_classes_ne : conjClass C ≠ conjClass C_inv := by
  intro h
  have hmem : C_inv ∈ conjClass C_inv := mem_conjClass_self C_inv
  rw [← h] at hmem
  simp only [conjClass, Finset.mem_filter, Finset.mem_univ, true_and] at hmem
  exact frob_generator_not_conj hmem

/-- **F₂₁ IS NON-AMBIVALENT**: not all elements are conjugate to their inverses. -/
theorem frob_not_ambivalent : ¬ (∀ g : F21, IsConj g (g⁻¹)) := by
  intro hall
  exact frob_generator_not_conj (hall C)

/-! ### Contrast with D₄: D₄ is Ambivalent -/

open CIRISHolon.BareCharge

/-- Conjugation in D₄. -/
def d4ConjBy (g x : Fin 8) : Fin 8 := dmul (dmul g x) (dinv g)

/-- Conjugacy relation in D₄. -/
def d4IsConj (x y : Fin 8) : Prop := ∃ g : Fin 8, d4ConjBy g x = y

instance (x y : Fin 8) : Decidable (d4IsConj x y) :=
  inferInstanceAs (Decidable (∃ g : Fin 8, d4ConjBy g x = y))

/-- **D₄ IS AMBIVALENT**: every element in D₄ is conjugate to its inverse.
    Reflections invert rotations by conjugation, making D₄ blind to orientation. -/
theorem d4_is_ambivalent (g : Fin 8) : d4IsConj g (dinv g) := by
  fin_cases g <;> decide

/-! ### Oriented Reading vs Ambivalent Projection -/

/-- The maximal gauge-invariant oriented loop reading: the conjugacy class `class(U(γ))`. -/
def orientedReading (g : F21) : Finset F21 := conjClass g

/-- The oriented reading is gauge-invariant: invariant under conjugation at the basepoint. -/
theorem orientedReading_gauge_invariant (g h : F21) :
    orientedReading (conjBy h g) = orientedReading g :=
  conjClass_conjBy h g

theorem conjBy_inv (h g : F21) : (conjBy h g)⁻¹ = conjBy h (g⁻¹) := by
  simp [conjBy, mul_assoc]

/-- An ambivalent projection: collapses orientation by merging class(g) and class(g⁻¹). -/
def ambivalentProjection (g : F21) : Finset F21 :=
  conjClass g ∪ conjClass (g⁻¹)

/-- The ambivalent projection is gauge-invariant. -/
theorem ambivalentProjection_gauge_invariant (g h : F21) :
    ambivalentProjection (conjBy h g) = ambivalentProjection g := by
  unfold ambivalentProjection
  rw [conjBy_inv, conjClass_conjBy, conjClass_conjBy]

/-- The ambivalent projection is orientation-blind by construction: `P(g) = P(g⁻¹)`. -/
theorem ambivalentProjection_orientation_blind (g : F21) :
    ambivalentProjection g = ambivalentProjection (g⁻¹) := by
  simp only [ambivalentProjection, inv_inv]
  exact Finset.union_comm (conjClass g) (conjClass (g⁻¹))

/-- **THE ORIENTED READING DETECTS LOOP ORIENTATION**:
    For the Frobenius loop holonomy C, the oriented reading distinguishes the forward
    loop γ from the backward loop γ⁻¹. -/
theorem oriented_detects_orientation :
    orientedReading C ≠ orientedReading (C⁻¹) :=
  frob_generator_classes_ne

/-- **THE AMBIVALENT PROJECTION IS PROVABLY BLIND**:
    Any projection `P` invariant under inversion (`P(g) = P(g⁻¹)`) cannot distinguish
    the forward loop from the backward loop. -/
theorem ambivalent_is_blind {α : Type*} (P : F21 → α) (hblind : ∀ g, P g = P (g⁻¹)) :
    P C = P (C⁻¹) :=
  hblind C

/-- **NON-FACTORING THEOREM (Rung 5 of Ω Ladder)**:
    Loop reversal is invisible to the ambivalent projection but detected by the
    oriented reading. In `CIRISHolon.Object` terms, the oriented reading witnesses
    NonFactoring of the ambivalent projection. -/
theorem nonfactoring_loop_orientation :
    ambivalentProjection C = ambivalentProjection (C⁻¹) ∧
    orientedReading C ≠ orientedReading (C⁻¹) :=
  ⟨ambivalentProjection_orientation_blind C, oriented_detects_orientation⟩

/-- NonFactoring witness packaged for `CIRISHolon.Object.NonFactoring`. -/
theorem nonfactoring_ambivalent_oriented :
    NonFactoring (fun _ : Unit => ambivalentProjection) orientedReading := by
  use C, C⁻¹
  constructor
  · intro _
    exact ambivalentProjection_orientation_blind C
  · exact oriented_detects_orientation

end CIRISHolon.FrobOrient
