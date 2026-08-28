/-
OMEGA-BREAK-1 — the intervention face, machine-checked.

Rung 3 proved Ω's data exceeds its probe behaviour by fiber-internal structure,
and `Identity.lean` declared that excess GAUGE.  This file settles what happens
when the admissible observations include BLIND INTERVENTIONS: a fixed
perturbation `a : S → S`, applied without conditioning on anything the meter
reports, interleaved with steps.

Two halves, both proved here:

* `gauge_safety` — the SAFETY half.  If every knob used preserves the gauge
  relation (probe-stream equality), then no experiment over {step, knobs}
  separates two probe-equivalent dynamics.  Corollaries: preparations
  (`a = c ∘ v`) are always safe; and on a CLOSED holon every view-covariant
  knob is safe, because closure forces the gauge relation to BE the view's
  fibers.

* `omega_break` — the BREAK.  The rung-3 counterexample itself, `H₁` and `H₂`
  of `ProbeConverse`, is `SameHolon` (probe-equivalent under the identity
  identifications) and yet a SINGLE blind transposition, applied after one
  step, makes the two read differently.  The knob is exhibited, and
  `knob_does_not_descend` shows exactly why the safety theorem does not apply
  to it: it maps a gauge-equivalent pair to a gauge-inequivalent one.

Together: holon identity is a function of the ACT vocabulary, not of the read
vocabulary alone.  `Identity.lean`'s `SameHolon` is the empty-knob end of that
family.
-/
import Mathlib.Tactic
import CIRISHolon.Omega
import CIRISHolon.Probe
import CIRISHolon.ProbeConverse
import CIRISHolon.Identity

namespace CIRISHolon.Break

open CIRISHolon.Omega CIRISHolon.Probe CIRISHolon.Identity CIRISHolon.ProbeConverse

variable {S V : Type}

/-! ### The gauge relation and the experiments -/

/-- The GAUGE relation: two states are identified when no probe separates them.
    This is the fiber of `Identity.lean`'s Moore quotient. -/
def Gauge (v : S → V) (f : S → S) (x y : S) : Prop :=
  ∀ n, v (f^[n] x) = v (f^[n] y)

theorem Gauge.rfl' (v : S → V) (f : S → S) (x : S) : Gauge v f x x := fun _ => rfl

theorem Gauge.view {v : S → V} {f : S → S} {x y : S} (h : Gauge v f x y) :
    v x = v y := h 0

theorem Gauge.trans' {v : S → V} {f : S → S} {x y z : S}
    (h₁ : Gauge v f x y) (h₂ : Gauge v f y z) : Gauge v f x z :=
  fun n => (h₁ n).trans (h₂ n)

/-- The gauge relation is carried by the dynamics. -/
theorem Gauge.step {v : S → V} {f : S → S} {x y : S} (h : Gauge v f x y) :
    Gauge v f (f x) (f y) := by
  intro n
  have h1 : f^[n] (f x) = f^[n + 1] x := (Function.iterate_succ_apply f n x).symm
  have h2 : f^[n] (f y) = f^[n + 1] y := (Function.iterate_succ_apply f n y).symm
  rw [h1, h2]
  exact h (n + 1)

/-- Probe equivalence in normal form: identical probe streams from every state,
    at every depth.  This is `Identity.ProbeEquiv` with `σ = τ = id`. -/
def PE (v : S → V) (f g : S → S) : Prop := ∀ n s, v (g^[n] s) = v (f^[n] s)

/-- The exact content of "the difference is fiber-internal": a probe-equivalent
    pair moves every state to gauge-equivalent successors. -/
theorem gauge_of_pe {v : S → V} {f g : S → S} (h : PE v f g) (z : S) :
    Gauge v f (f z) (g z) := by
  intro n
  have e1 : f^[n] (f z) = f^[n + 1] z := (Function.iterate_succ_apply f n z).symm
  have e2 : g^[n] (g z) = g^[n + 1] z := (Function.iterate_succ_apply g n z).symm
  calc v (f^[n] (f z)) = v (f^[n + 1] z) := by rw [e1]
    _ = v (g^[n + 1] z) := (h (n + 1) z).symm
    _ = v (g^[n] (g z)) := by rw [e2]
    _ = v (f^[n] (g z)) := h n (g z)

/-- One letter of an experiment: advance the dynamics, or apply a blind knob. -/
inductive Letter (S : Type) where
  | step : Letter S
  | knob : (S → S) → Letter S

/-- Run an experiment word against a dynamics. -/
def runL (f : S → S) : List (Letter S) → S → S
  | [], s => s
  | Letter.step :: w, s => runL f w (f s)
  | Letter.knob a :: w, s => runL f w (a s)

/-- A knob PRESERVES THE GAUGE when it maps gauge-equivalent states to
    gauge-equivalent states — equivalently, when it descends to the Moore
    quotient that `Identity.lean` takes for the holon's identity. -/
def PreservesGauge (v : S → V) (f : S → S) (a : S → S) : Prop :=
  ∀ x y, Gauge v f x y → Gauge v f (a x) (a y)

/-! ### The safety half -/

/-- **T1, the safety theorem.**  If every knob occurring in the experiment
    preserves the gauge, then the experiment cannot separate a probe-equivalent
    pair — from any pair of gauge-equivalent starting states, and in particular
    from a common one. -/
theorem gauge_safety {v : S → V} {f g : S → S} (hpe : PE v f g) :
    ∀ (w : List (Letter S)),
      (∀ a, Letter.knob a ∈ w → PreservesGauge v f a) →
      ∀ x y, Gauge v f x y → v (runL f w x) = v (runL g w y) := by
  intro w
  induction w with
  | nil => intro _ x y h; exact h.view
  | cons l w ih =>
      intro hA x y h
      cases l with
      | step =>
          exact ih (fun a ha => hA a (List.mem_cons_of_mem _ ha)) _ _
            (h.step.trans' (gauge_of_pe hpe y))
      | knob a =>
          exact ih (fun b hb => hA b (List.mem_cons_of_mem _ hb)) _ _
            (hA a (List.mem_cons_self _ _) x y h)

/-- The form the criterion uses: a common starting state. -/
theorem gauge_safety_common {v : S → V} {f g : S → S} (hpe : PE v f g)
    (w : List (Letter S)) (hA : ∀ a, Letter.knob a ∈ w → PreservesGauge v f a)
    (s : S) : v (runL f w s) = v (runL g w s) :=
  gauge_safety hpe w hA s s (Gauge.rfl' v f s)

/-- **T2.**  A preparation — "read the meter, then set the state to `c` of what
    you read" — always preserves the gauge, so preparations never break. -/
theorem prep_preservesGauge {v : S → V} {f : S → S} (c : V → S) :
    PreservesGauge v f (fun s => c (v s)) := by
  intro x y h
  simp only [h.view]
  exact Gauge.rfl' v f _

/-! ### Closure collapses the gauge onto the view -/

theorem view_iterate {v : S → V} {f : S → S} {F : V → V}
    (hF : ∀ s, v (f s) = F (v s)) : ∀ n s, v (f^[n] s) = F^[n] (v s) := by
  intro n
  induction n with
  | zero => intro s; rfl
  | succ k ih =>
      intro s
      rw [Function.iterate_succ_apply, ih, hF, Function.iterate_succ_apply]

/-- On a CLOSED holon the gauge relation is exactly the view's fibers: closure
    is precisely the statement that probes see nothing the meter does not. -/
theorem gauge_of_view_of_closed {v : S → V} {f : S → S} {F : V → V}
    (hF : ∀ s, v (f s) = F (v s)) {x y : S} (h : v x = v y) : Gauge v f x y := by
  intro n
  rw [view_iterate hF, view_iterate hF, h]

/-- **T3.**  Hence on a closed holon every VIEW-COVARIANT knob (one that
    descends to the view quotient, `v ∘ a = α ∘ v`) preserves the gauge — the
    programme's closed views are safe against every knob compatible with their
    own meter.  The danger zone is exactly the views that are not closed, i.e.
    the ones carrying nonzero rent. -/
theorem vcov_preservesGauge_of_closed {v : S → V} {f : S → S} {F : V → V}
    (hF : ∀ s, v (f s) = F (v s)) {a : S → S} {α : V → V}
    (ha : ∀ s, v (a s) = α (v s)) : PreservesGauge v f a := by
  intro x y h
  exact gauge_of_view_of_closed hF (by rw [ha, ha, h.view])

/-! ### The break: the rung-3 pair is intervention-distinguishable -/

/-- The knob: the transposition of states `1` and `2`.  A permutation, applied
    BLIND — no conditioning on anything the meter reports. -/
def knobT : Fin 3 → Fin 3 := fun s => if s = 1 then 2 else if s = 2 then 1 else s

theorem knobT_bijective : Function.Bijective knobT := by
  refine Finite.injective_iff_bijective.mp ?_
  decide

/-- `H₁` and `H₂` are the SAME HOLON by `Identity.lean`'s commitment, with the
    identity as both identifications. -/
theorem sameHolon_H₁_H₂ : SameHolon H₁ H₂ :=
  ⟨Equiv.refl _, Equiv.refl _, fun n s => agreement.probe_eq n s⟩

/-- The gauge relation identifies states `0` and `1` (they are the fiber rung 3
    calls conventional). -/
theorem d₁_iterate (n : ℕ) (s : Fin 3) : d₁^[n] s = s := by
  simp [d₁, Function.iterate_id]

theorem gauge_zero_one : Gauge vw d₁ 0 1 := by
  intro n
  rw [d₁_iterate, d₁_iterate]
  decide

/-- **Why the safety theorem does not apply**: the knob fails to descend to the
    Moore quotient.  It takes the gauge-equivalent pair `(0, 1)` to `(0, 2)`,
    which the meter separates at depth 0. -/
theorem knob_does_not_descend : ¬ PreservesGauge vw d₁ knobT := by
  intro h
  have := (h 0 1 gauge_zero_one).view
  revert this
  decide

/-- **THE BREAK.**  One step, then one blind transposition, then read: `H₁` and
    `H₂` — probe-equivalent at every depth from every state, identical under
    every current face, and `SameHolon` — report different values. -/
theorem break_experiment :
    vw (runL d₁ [Letter.step, Letter.knob knobT] 0)
      ≠ vw (runL d₂ [Letter.step, Letter.knob knobT] 0) := by decide

/-- Rung 7's falsifier, in one statement: two holons the current tuple calls
    the same, separated by an admissible blind experiment.  The Moore-quotient
    identity is therefore too coarse as soon as interventions are admissible,
    and the tuple must carry the act vocabulary as data. -/
theorem omega_break :
    SameHolon H₁ H₂ ∧
      vw (runL d₁ [Letter.step, Letter.knob knobT] 0)
        ≠ vw (runL d₂ [Letter.step, Letter.knob knobT] 0) :=
  ⟨sameHolon_H₁_H₂, break_experiment⟩

end CIRISHolon.Break
