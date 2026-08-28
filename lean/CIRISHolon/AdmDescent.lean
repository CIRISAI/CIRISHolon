/-
CIRISHolon.AdmDescent — EINSTEIN-ADM-1's CORRECTED gate (E1'), as a machine
artifact instead of an instrument printout.

EINSTEIN-ADM-1 (`conformance/gravity/EINSTEIN_ADM1_RESULTS.md`) first scored
closure along a TRAJECTORY and the external re-review convicted it: the flat
carrier was a uniform superposition over a sector, hence EXACTLY STATIONARY
under a sector-preserving permutation, so all 28 "consistent collisions" were
one unchanged state repeating.  The lesson is registered as
`M-FIXED-POINT-TRAJECTORY`, the headline was WITHDRAWN, and the re-review named
the stronger fact that was actually available: the microscopic mapping-class
step DESCENDS to the conjugation quotient, universally and carrier-free.
Amendment ADM-1C restaked the gate on that, and this file is that gate proved
rather than printed.

THE MODEL.  The one-plaquette D₄ torus of the instrument: a configuration is
the pair of holonomies `(a, b) ∈ D₄ × D₄`, 64 in all.  The gauge action is
SIMULTANEOUS conjugation.  The mapping-class generators are the Dehn twists
`T(a,b) = (a, ab)` and `S(a,b) = (b, bab⁻¹)`, and the step is `S ∘ T`, exactly
the instrument's `T_MAP` then `S_MAP`.  Flatness — in 2+1 dimensions the
Einstein equation itself — is triviality of the commutator `[a,b]`.

WHAT IS PROVED, all by exhaustion over the 64 configurations (and, where the
statement quantifies over gauge elements or pairs, over 8 × 64 or 64 × 64):

* `orbitRep_conj`      — the chosen canonical representative really is a
                         conjugation invariant, so it is an orbit LABEL
* `orbit_card`         — there are exactly 28 labels
* `descend_welldef`    — the composite step DESCENDS: the label of the
                         successor is a function of the label alone (this is
                         E1's content, checked universally rather than along a
                         carrier, which is what `M-FIXED-POINT-TRAJECTORY`
                         demands)
* `descend_injective`, `descend_surjective` — the descended map is a BIJECTION
                         of the 28 labels
* `descend_nontrivial` — and it is NOT the identity (the vacuity check the
                         trajectory form lacked, and the exact defect the
                         re-review found)
* `flat_preserved`     — flatness is carried by the step, and
  `flat_gauge_invariant` — flatness is a property of the label, so the
                         constraint descends to the quotient too

SCOPE, honestly.  This is a discrete 2+1 toy at one plaquette with gauge group
D₄.  It is NOT 3+1, NOT continuum, NOT local degrees of freedom, and NOT matter
back-reaction.  And it does not derive the dynamics: the mapping-class action is
INPUT, and its closure and constraint-preservation are what is checked — the
record's own correctly sized wording.  Picking the step OUT of a closure
principle remains open.

D₄ is BareCharge's table, reused rather than restated.
-/
import Mathlib.Tactic
import CIRISHolon.BareCharge

set_option maxRecDepth 100000
set_option maxHeartbeats 4000000

namespace CIRISHolon.AdmDescent

open CIRISHolon.BareCharge (dmul dinv)

/-- A configuration of the one-plaquette torus: the two holonomies. -/
abbrev Cfg := Fin 8 × Fin 8

/-! ### The gauge action and its orbit labels -/

/-- Simultaneous conjugation — the gauge action on the torus. -/
def conjBy (g : Fin 8) (x : Cfg) : Cfg :=
  (dmul (dmul g x.1) (dinv g), dmul (dmul g x.2) (dinv g))

theorem conjBy_one (x : Cfg) : conjBy 0 x = x := by revert x; decide

theorem conjBy_comp (g h : Fin 8) (x : Cfg) :
    conjBy g (conjBy h x) = conjBy (dmul g h) x := by revert g h x; decide

/-- A total order on configurations, used only to pick a canonical
    representative of each orbit. -/
def code (x : Cfg) : ℕ := x.1.val + 8 * x.2.val

def minBy (x y : Cfg) : Cfg := if code y < code x then y else x

/-- The canonical representative: the least element of the conjugation orbit. -/
def orbitRep (x : Cfg) : Cfg :=
  minBy (minBy (minBy (minBy (minBy (minBy (minBy
    (conjBy 0 x) (conjBy 1 x)) (conjBy 2 x)) (conjBy 3 x))
    (conjBy 4 x)) (conjBy 5 x)) (conjBy 6 x)) (conjBy 7 x)

/-- `orbitRep` lands inside the orbit it labels. -/
theorem orbitRep_conjugate (x : Cfg) : ∃ g : Fin 8, orbitRep x = conjBy g x := by
  revert x; decide

/-- **`orbitRep` is a gauge invariant** — the v_ADM label of the instrument. -/
theorem orbitRep_conj (g : Fin 8) (x : Cfg) : orbitRep (conjBy g x) = orbitRep x := by
  revert g x; decide

theorem orbitRep_idem (x : Cfg) : orbitRep (orbitRep x) = orbitRep x := by
  revert x; decide

/-- The label set: the canonical representatives. -/
def labels : Finset Cfg := Finset.univ.image orbitRep

/-- **28 orbits**, matching the instrument's `N_ORBITS`. -/
theorem orbit_card : labels.card = 28 := by decide

/-! ### The mapping-class step -/

/-- The Dehn twist `T : (a, b) ↦ (a, ab)`. -/
def twistT (x : Cfg) : Cfg := (x.1, dmul x.1 x.2)

/-- The Dehn twist `S : (a, b) ↦ (b, bab⁻¹)`. -/
def twistS (x : Cfg) : Cfg := (x.2, dmul (dmul x.2 x.1) (dinv x.2))

/-- The composite step of the instrument: `T` then `S`. -/
def step (x : Cfg) : Cfg := twistS (twistT x)

theorem step_bijective : Function.Bijective step := by
  refine Finite.injective_iff_bijective.mp ?_
  decide

/-- The descended map, read on labels. -/
def descend (x : Cfg) : Cfg := orbitRep (step x)

/-! ### E1' — the quotient-dynamics theorem -/

/-- **The step DESCENDS.**  Gauge-equivalent configurations have
    gauge-equivalent successors, so the label of the successor is a function of
    the label alone.  Checked on all 8 × 64 gauge-element/configuration pairs —
    carrier-free, which is exactly what `M-FIXED-POINT-TRAJECTORY` requires and
    what the withdrawn trajectory form could not supply. -/
theorem descend_welldef (g : Fin 8) (x : Cfg) : descend (conjBy g x) = descend x := by
  revert g x; decide

/-- The equivalent form: equal labels in, equal labels out. -/
theorem descend_of_orbitRep_eq (x y : Cfg) (h : orbitRep x = orbitRep y) :
    descend x = descend y := by
  revert x y; decide

/-- **The descended map is injective on labels.** -/
theorem descend_injective (x y : Cfg) (h : descend x = descend y) :
    orbitRep x = orbitRep y := by
  revert x y; decide

/-- **And surjective**: it permutes the 28 labels rather than collapsing them. -/
theorem descend_surjective : Finset.univ.image descend = labels := by decide

/-- **And it is NOT the identity** — the vacuity check the trajectory gate
    lacked.  `(e, r)` and its successor sit in different orbits. -/
theorem descend_nontrivial : descend (0, 1) ≠ orbitRep (0, 1) := by decide

/-- E1', packaged: well defined on labels, a bijection of them, and nontrivial. -/
theorem quotient_dynamics :
    (∀ g x, descend (conjBy g x) = descend x) ∧
    (∀ x y, descend x = descend y → orbitRep x = orbitRep y) ∧
    (Finset.univ.image descend = labels) ∧
    (∃ x, descend x ≠ orbitRep x) :=
  ⟨descend_welldef, descend_injective, descend_surjective, ⟨(0, 1), descend_nontrivial⟩⟩

/-! ### The Einstein constraint at the quotient level -/

/-- The commutator `[a, b] = a b a⁻¹ b⁻¹`. -/
def comm (x : Cfg) : Fin 8 := dmul (dmul x.1 x.2) (dmul (dinv x.1) (dinv x.2))

/-- FLAT: the commutator is trivial.  In 2+1 dimensions this is the Einstein
    equation itself. -/
def flat (x : Cfg) : Prop := comm x = 0

instance (x : Cfg) : Decidable (flat x) := by unfold flat; infer_instance

theorem flat_iff_commute (x : Cfg) : flat x ↔ dmul x.1 x.2 = dmul x.2 x.1 := by
  revert x; decide

/-- 40 of the 64 configurations are flat. -/
theorem flat_card : (Finset.univ.filter flat).card = 40 := by decide

/-- Flatness is a gauge invariant, so it is a property of the LABEL: the
    constraint descends to the quotient alongside the dynamics. -/
theorem flat_gauge_invariant (g : Fin 8) (x : Cfg) : flat (conjBy g x) ↔ flat x := by
  revert g x; decide

/-- **The constraint is preserved by the descended dynamics** — E2's content,
    at the quotient level where no carrier can fake it. -/
theorem flat_preserved (x : Cfg) (h : flat x) : flat (step x) := by
  revert x; decide

/-- Hence the flat sector's labels are carried into flat labels. -/
theorem flat_labels_preserved (x : Cfg) (h : flat x) : flat (descend x) := by
  revert x; decide

/-- 22 of the 28 labels are flat. -/
theorem flat_labels_card :
    ((Finset.univ.filter flat).image orbitRep).card = 22 := by decide

end CIRISHolon.AdmDescent
