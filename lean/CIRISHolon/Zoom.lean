/-
CIRISHolon.Zoom — the zoom law's kill, mechanized: which quantity nests
under acuity.

WHAT THIS IS.  SELECTOR-4 staked gate Z1 as "the selected set is
non-increasing as acuity refines" — the lead's zoom principle as the freeze
transcribed it.  The run killed it (branch (b), all four gauge worlds), and
the run's log derived WHY the death was forced by the construction rather
than contingent on the data.  This file is that derivation as theorems, per
the house rule that every mechanizable kill gets its brick.

The model is the campaign's own shape, abstracted to exactly what the
argument uses.  A VIEW is a separation relation (which pairs of candidates
the observer can tell apart).  REFINEMENT means sharpening: every separation
the coarse view makes, the fine view keeps.  Two derived objects:

* `ident v x` — the identity set: everything `x` cannot be told from under
  `v`.  This is the observer's indistinguishability, the |Ident| of the log.
* `selected need v` — the passing set for any criterion of SELECTOR-4's
  shape: `x` passes iff the view separates it from EVERY rival the
  criterion requires (`need x`).  Fiber-injectivity, bootstrap separation,
  and the gauntlet's per-world tests are all of this form.

Three things are proved, and together they are the kill:

* `ident_antitone` — refining the view SHRINKS every identity set.  This is
  the quantity the lead's intuition names, and it does nest under zoom.
* `selected_monotone` — refining the view GROWS the selected set.  The
  staked Z1 direction asserts the opposite inclusion; this theorem is why
  no run could ever have satisfied it on this criterion shape.
* `selected_strictly_grows` — a two-candidate witness where the growth is
  STRICT (`decide` on a four-point instance), so the staked law is not
  merely unprovable here, it is FALSE: there is a refinement that strictly
  enlarges the passing set.  The witness is the campaign in miniature — at
  the coarse view nothing is separated and nothing passes; at the fine view
  something is, and does.  Exactly the A0-vs-A3 columns of the log.

SCOPE, stated.  Views are arbitrary relations; no symmetry of `sep` is
assumed because none is used.  This says nothing about WHICH worlds select
(that is the measured content of `conformance/omega/selector4.log`); it
says only which DIRECTION each derived quantity moves under refinement, for
any criterion of the stated shape.  The corrected re-stake ("|Ident| is the
nesting quantity") is a successor's to freeze; this brick is what makes its
direction a theorem rather than a hope.
-/

import Mathlib.Tactic

namespace CIRISHolon.Zoom

/-- A view: which pairs the observer separates. -/
structure View (α : Type) where
  sep : α → α → Prop

/-- `w` refines `v`: every separation `v` makes, `w` keeps. -/
def Refines {α : Type} (v w : View α) : Prop :=
  ∀ x y, v.sep x y → w.sep x y

/-- The identity set of `x` under `v`: what `x` cannot be told from. -/
def ident {α : Type} (v : View α) (x : α) : Set α :=
  { y | ¬ v.sep x y }

/-- The selected set for a criterion demanding separation from every rival
in `need x`. -/
def selected {α : Type} (need : α → Set α) (v : View α) : Set α :=
  { x | ∀ y ∈ need x, v.sep x y }

/-- Refining the view shrinks every identity set: indistinguishability is
the quantity that nests under zoom. -/
theorem ident_antitone {α : Type} {v w : View α} (h : Refines v w) (x : α) :
    ident w x ⊆ ident v x := by
  intro y hy hsep
  exact hy (h x y hsep)

/-- Refining the view grows the selected set: the staked Z1 direction is
the reverse inclusion, and this is why it could never hold for a
separation-demanding criterion. -/
theorem selected_monotone {α : Type} (need : α → Set α) {v w : View α}
    (h : Refines v w) : selected need v ⊆ selected need w := by
  intro x hx y hy
  exact h x y (hx y hy)

/-- The two-point witness: candidates `0, 1`, each needing separation from
the other.  The blind view separates nothing; the sharp view separates the
pair.  Membership of `0`: fails at the blind view, holds at the sharp one. -/
def blind : View (Fin 2) := ⟨fun _ _ => False⟩

def sharp : View (Fin 2) := ⟨fun x y => x ≠ y⟩

def rival : Fin 2 → Set (Fin 2) := fun x => { y | y ≠ x }

theorem sharp_refines_blind : Refines blind sharp := by
  intro x y h
  exact h.elim

/-- Strictness: the selected set genuinely grows — `0` is selected at the
sharp view and not at the blind one.  With `selected_monotone` this kills
the staked direction outright: a refinement exists under which the passing
set strictly enlarges. -/
theorem selected_strictly_grows :
    (0 : Fin 2) ∈ selected rival sharp ∧ (0 : Fin 2) ∉ selected rival blind := by
  constructor
  · intro y hy
    exact fun h => hy h.symm
  · intro h
    exact h 1 (show (1 : Fin 2) ≠ 0 by decide)

end CIRISHolon.Zoom
