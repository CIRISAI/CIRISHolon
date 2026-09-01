/-
CIRISHolon.Carrier — the HORIZONTAL axis of the fold (WB-8.1), in the SAME
shape as the vertical one.

`Object.lean` asks one question of a MOTION: does the reading survive it
(`Closed v T`)? This file asks the identical question of a CHANGE OF CARRIER —
Born–Oppenheimer classical nuclei → ring-polymer quantum nuclei → real-time MPS
electronic dynamics → spinorial/Dirac → QED. The square is the same square,
rotated ninety degrees, and `closed_transports` is that sentence as a theorem:
a transport carries closure both ways, so climbing the tower never manufactures
closure and never destroys it.

Three things are stated here that the engine (`holon-chem/src/tower.rs`) can
only enforce with types, never prove:

1. **THE FIBER.** A carrier names its own state space AND its own operator type.
   A theory is the DEPENDENT PAIR `(C : Carrier) × C.TermSet` — the term set's
   type is a function of the carrier, so `A.add (a : A.Op) (b : B.Op)` is not a
   bad idea, it is not a term. The gate is demonstrated FIRING (`#guard_msgs`
   below), because a type-level gate proven only by code that compiles has never
   been seen to fire — the same standard the Rust side's `compile_fail` doctest
   is held to.

2. **THE CERTIFICATE.** `Transport A B` is lift-state + operator picture-change
   + the commuting square, carried as a FIELD: a transport that does not commute
   cannot be constructed. `Transport.comp` pastes two squares, and the three
   category laws hold definitionally — the tower is a category, so a certificate
   composes to any height (`Tower.climb`, `Tower.climb_square`).

3. **THE PRICE.** Selection is the corridor rule: argmin price over the
   ADMISSIBLE set, never over the list. `select_admissible` is the refusal by
   theorem — cheapness alone cannot select the dead chart — and
   `select_eq_none` is the fence: when nothing is in budget the rule returns
   nothing rather than the cheapest wrong answer.

The composed theorem the file exists for is `Tower.climb_total`: a theory's
total energy survives a climb of ARBITRARY height. Transport the terms one at a
time, add them in the top fiber, read the sum on the lifted state — the number
is the one the bottom carrier read. Terms never add across fibers, and nothing
is double counted at any seam.

SCOPE, stated because the engine is bigger than the proof. This file is about
the ALGEBRA of the fold: fibers, squares, composition, selection. It says
nothing about whether any particular physical transport (C0→C1 replication,
C1→C2 picture change) actually commutes — that is a MEASUREMENT, made by the
engine's certificate, and `CommutingCertificate.closure_defect` is where the
measured number lives. What is proved here is that IF the squares commute THEN
the tower's readings are stable under arbitrary climbs, and that a fence is
never silently discharged by a cheaper chart.
-/
import CIRISHolon.Object
import Mathlib.Tactic

namespace CIRISHolon

/-! ### 1. The index: carriers, fibers, theories -/

/-- A CARRIER is an index. It names its own state space, its own operator type,
    and the reading that turns an operator into a number on a state. The
    additive structure is the fiber's: `zero` and `add` are operations on THIS
    carrier's `Op` and on nothing else. -/
structure Carrier : Type 1 where
  /-- The carrier's state space. -/
  State : Type
  /-- The carrier's operator type — the fiber. -/
  Op : Type
  /-- The empty term: nothing contributed. -/
  zero : Op
  /-- Term addition, INSIDE the fiber. -/
  add : Op → Op → Op
  /-- The reading: what an operator says about a state. -/
  eval : Op → State → ℝ
  /-- The empty term contributes nothing. -/
  eval_zero : ∀ s, eval zero s = 0
  /-- Readings are additive: this is what makes a term set a sum rather than a
      list, and what a picture change has to respect to be a transport. -/
  eval_add : ∀ (o₁ o₂ : Op) (s : State), eval (add o₁ o₂) s = eval o₁ s + eval o₂ s

namespace Carrier

/-- The terms a theory carries on `C`. The TYPE depends on `C`; that dependence
    is the whole gate. -/
abbrev TermSet (C : Carrier) : Type := List C.Op

/-- The fiber's total: fold the term set with the carrier's own addition. -/
def total (C : Carrier) : C.TermSet → C.Op
  | [] => C.zero
  | o :: rest => C.add o (C.total rest)

@[simp] theorem total_nil (C : Carrier) : C.total [] = C.zero := rfl

@[simp] theorem total_cons (C : Carrier) (o : C.Op) (rest : C.TermSet) :
    C.total (o :: rest) = C.add o (C.total rest) := rfl

/-- The reading of a total is the total of the readings. -/
theorem eval_total (C : Carrier) (s : C.State) :
    ∀ ts : C.TermSet, C.eval (C.total ts) s = (ts.map (fun o => C.eval o s)).sum := by
  intro ts
  induction ts with
  | nil => simp [C.eval_zero]
  | cons o rest ih => simp [C.eval_add, ih]

end Carrier

/-- A THEORY is the dependent pair: a carrier, and the terms it carries.
    `WB-8.2`'s `TheoryNode<C: Carrier>` with the carrier promoted from a type
    parameter to a component. -/
def Theory : Type 1 := Σ C : Carrier, C.TermSet

namespace Theory

/-- The carrier a theory lives on. -/
def carrier (t : Theory) : Carrier := t.1

/-- The terms, typed in that carrier's fiber. -/
def terms (t : Theory) : t.carrier.TermSet := t.2

/-- The theory's energy on a state of ITS OWN carrier. There is no other state
    it can be asked about. -/
def energy (t : Theory) (s : t.carrier.State) : ℝ :=
  t.carrier.eval (t.carrier.total t.terms) s

end Theory

/-! ### 2. The fiber gate, demonstrated firing

`A.add` accepts `A.Op` and nothing else. Below is that refusal happening —
the Lean analogue of `tower.rs`'s `compile_fail` doctest, and held to the same
standard: a gate that has never failed has never gated. -/

/-- error: application type mismatch
  A.add a b
argument
  b
has type
  B.Op : Type
but is expected to have type
  A.Op : Type
-/
#guard_msgs in
example (A B : Carrier) (a : A.Op) (b : B.Op) : A.Op := A.add a b

/-! ### 3. The certificate: transport as a commuting square -/

/-- A CERTIFIED TRANSPORT from carrier `A` to carrier `B`.

    Three components, exactly the engine's three: a state lift, an operator
    picture change, and the commuting square — carried as a FIELD, so a
    transport whose square does not commute is not a transport. `picture_zero`
    and `picture_add` are the fiber half: the picture change is a map of term
    algebras, which is what lets a theory be transported term by term with
    nothing dropped and nothing counted twice. -/
structure Transport (A B : Carrier) where
  /-- Lift a state into the finer carrier. -/
  lift : A.State → B.State
  /-- Change the operator's picture. -/
  picture : A.Op → B.Op
  /-- **THE SQUARE.** The transported operator read on the lifted state is the
      original operator read on the original state. -/
  square : ∀ (o : A.Op) (s : A.State), B.eval (picture o) (lift s) = A.eval o s
  /-- Nothing becomes something across the seam. -/
  picture_zero : picture A.zero = B.zero
  /-- Addition downstairs is addition upstairs — never a cross-fiber sum. -/
  picture_add : ∀ o₁ o₂ : A.Op,
    picture (A.add o₁ o₂) = B.add (picture o₁) (picture o₂)

namespace Transport

variable {A B C D : Carrier}

/-- Staying put is a transport. -/
def refl (A : Carrier) : Transport A A where
  lift := _root_.id
  picture := _root_.id
  square := fun _ _ => rfl
  picture_zero := rfl
  picture_add := fun _ _ => rfl

/-- **Squares paste.** The composite carries the composite certificate. -/
def comp (f : Transport A B) (g : Transport B C) : Transport A C where
  lift := fun s => g.lift (f.lift s)
  picture := fun o => g.picture (f.picture o)
  square := fun o s => by
    show C.eval (g.picture (f.picture o)) (g.lift (f.lift s)) = A.eval o s
    rw [g.square, f.square]
  picture_zero := by
    show g.picture (f.picture A.zero) = C.zero
    rw [f.picture_zero, g.picture_zero]
  picture_add := fun o₁ o₂ => by
    show g.picture (f.picture (A.add o₁ o₂))
        = C.add (g.picture (f.picture o₁)) (g.picture (f.picture o₂))
    rw [f.picture_add, g.picture_add]

/-- **The tower is a category** (left unit). -/
theorem refl_comp (f : Transport A B) : (Transport.refl A).comp f = f := rfl

/-- **The tower is a category** (right unit). -/
theorem comp_refl (f : Transport A B) : f.comp (Transport.refl B) = f := rfl

/-- **The tower is a category** (associativity). Definitional: pasting three
    squares does not depend on which two are pasted first. -/
theorem comp_assoc (f : Transport A B) (g : Transport B C) (h : Transport C D) :
    (f.comp g).comp h = f.comp (g.comp h) := rfl

/-- The square as a function equation — the shape `Object.Closed` is written in,
    rotated: the reading composed with the lift IS the reading downstairs. -/
theorem reading_held (f : Transport A B) (o : A.Op) :
    (fun s => B.eval (f.picture o) (f.lift s)) = A.eval o :=
  funext (f.square o)

/-- A theory transports TERM BY TERM: the picture of the total is the total of
    the pictures. Nothing is dropped at the seam and nothing is counted twice.
    Proved by induction on the term set, which is where the fiber's `zero` and
    `add` clauses of the certificate are spent. -/
theorem picture_total (f : Transport A B) :
    ∀ ts : A.TermSet, f.picture (A.total ts) = B.total (ts.map f.picture) := by
  intro ts
  induction ts with
  | nil => simpa using f.picture_zero
  | cons o rest ih => simp [f.picture_add, ih]

/-- **The two axes are one square.** If the lift intertwines a motion
    (`lift ∘ T = T' ∘ lift`), every reading that is `Closed` upstairs is `Closed`
    downstairs WITH THE SAME UPDATE. Climbing the tower neither manufactures
    closure nor destroys it — which is exactly why a fence may be discharged by
    climbing, and never by renaming. -/
theorem closed_transports {E : Type} (f : Transport A B)
    (w : B.State → E) (T : A.State → A.State) (T' : B.State → B.State)
    (hint : ∀ s, f.lift (T s) = T' (f.lift s))
    (h : Object.Closed w T') :
    Object.Closed (w ∘ f.lift) T := by
  obtain ⟨u, hu⟩ := h
  refine ⟨u, ?_⟩
  funext s
  simp only [Function.comp_apply]
  rw [hint s]
  exact congrFun hu (f.lift s)

end Transport

/-! ### 4. The tower: certificates composed to arbitrary height -/

/-- A TOWER: carriers indexed by refinement level, with a certified transport at
    every rung. The index is `ℕ` because the horizontal axis is a LADDER
    (C0 → C1 → C2 → C3 → …), not a general diagram. -/
structure Tower : Type 1 where
  /-- The carrier at each refinement level. -/
  carrier : ℕ → Carrier
  /-- The certified transport one rung up. -/
  rung : ∀ n : ℕ, Transport (carrier n) (carrier (n + 1))

namespace Tower

/-- Climb `k` rungs starting from level `n`. -/
def climb (T : Tower) (n : ℕ) : (k : ℕ) → Transport (T.carrier n) (T.carrier (n + k))
  | 0 => Transport.refl _
  | k + 1 => (T.climb n k).comp (T.rung (n + k))

@[simp] theorem climb_zero (T : Tower) (n : ℕ) :
    T.climb n 0 = Transport.refl (T.carrier n) := rfl

theorem climb_succ (T : Tower) (n k : ℕ) :
    T.climb n (k + 1) = (T.climb n k).comp (T.rung (n + k)) := rfl

/-- **The certificate survives an arbitrary climb.** Not "each rung commutes" —
    the composite square commutes, at any height, and it is the type of `climb`
    that says so. -/
theorem climb_square (T : Tower) (n k : ℕ)
    (o : (T.carrier n).Op) (s : (T.carrier n).State) :
    (T.carrier (n + k)).eval ((T.climb n k).picture o) ((T.climb n k).lift s)
      = (T.carrier n).eval o s :=
  (T.climb n k).square o s

/-- **THE COMPOSED THEOREM.** A theory's total energy is invariant under a climb
    of arbitrary height: transport the terms one at a time, add them IN THE TOP
    FIBER, and read the sum on the lifted state — the number is the one the
    bottom carrier read.

    Both halves of the fold are spent here: `picture_total` (terms add only
    inside a fiber, and the picture change respects that addition) and
    `climb_square` (the composite certificate commutes). Either one alone is not
    enough, which is the sense in which the tower's laws are not decorative. -/
theorem climb_total (T : Tower) (n k : ℕ)
    (ts : (T.carrier n).TermSet) (s : (T.carrier n).State) :
    (T.carrier (n + k)).eval
        ((T.carrier (n + k)).total (ts.map (T.climb n k).picture))
        ((T.climb n k).lift s)
      = (T.carrier n).eval ((T.carrier n).total ts) s := by
  rw [← Transport.picture_total, climb_square]

end Tower

/-! ### 5. The corridor rule: price is minimised over the ADMISSIBLE set

`tower.rs`'s `select_corridor`, stated so the refusal is a theorem rather than a
code path. Cheapness alone never selects: an out-of-budget node is not a
candidate at any price, and when nothing is in budget the rule returns nothing. -/

/-- A candidate node of the theory diagram: its measured price, and the two
    budgets it must sit inside. -/
structure Node where
  /-- The node's identifying name. -/
  name : String
  /-- Measured price. Never an estimate — see `M-CHEAPER-THAN-ITS-PRICE`. -/
  price : ℝ
  /-- The closure defect this node tolerates. -/
  closureBudget : ℝ
  /-- The conservation drift this node tolerates. -/
  conservationBudget : ℝ

/-- A node is ADMISSIBLE when the measured defect and drift both sit inside its
    declared budgets. -/
def Node.Admissible (defect drift : ℝ) (n : Node) : Prop :=
  defect ≤ n.closureBudget ∧ drift ≤ n.conservationBudget

/-- Admissibility is decided by two real comparisons. The instance is named so
    every `if` below picks the SAME one, and the equation lemmas are therefore
    about the function `select` actually is. -/
noncomputable instance Node.instDecidableAdmissible (defect drift : ℝ) (n : Node) :
    Decidable (n.Admissible defect drift) := by
  unfold Node.Admissible; infer_instance

/-- The cheaper of two nodes. -/
noncomputable def Node.better (a b : Node) : Node :=
  if a.price ≤ b.price then a else b

theorem Node.better_price_le_left (a b : Node) : (a.better b).price ≤ a.price := by
  unfold Node.better
  by_cases h : a.price ≤ b.price
  · rw [if_pos h]
  · rw [if_neg h]; exact le_of_not_le h

theorem Node.better_price_le_right (a b : Node) : (a.better b).price ≤ b.price := by
  unfold Node.better
  by_cases h : a.price ≤ b.price
  · rw [if_pos h]; exact h
  · rw [if_neg h]

theorem Node.better_eq (a b : Node) : a.better b = a ∨ a.better b = b := by
  unfold Node.better
  by_cases h : a.price ≤ b.price
  · exact Or.inl (if_pos h)
  · exact Or.inr (if_neg h)

/-- One step of the corridor fold: an inadmissible node is dropped WITHOUT
    consulting its price; an admissible one competes. -/
noncomputable def selectStep (defect drift : ℝ) (n : Node) (best : Option Node) :
    Option Node :=
  if n.Admissible defect drift then some (best.elim n (Node.better n)) else best

/-- **THE CORRIDOR RULE.** The cheapest ADMISSIBLE node, or `none`. The
    admissibility test gates entry to the comparison; price only ever breaks ties
    among nodes that already passed it. -/
noncomputable def select (defect drift : ℝ) (l : List Node) : Option Node :=
  l.foldr (selectStep defect drift) none

variable {defect drift : ℝ}

@[simp] theorem select_nil : select defect drift [] = none := rfl

@[simp] theorem select_cons (n : Node) (rest : List Node) :
    select defect drift (n :: rest) = selectStep defect drift n (select defect drift rest) :=
  rfl

/-- **THE FENCE.** When nothing is in budget the corridor returns nothing — it
    does not fall back to the cheapest wrong answer. This is the theorem behind
    `TransportRefusal::ClosureDefectExceeded`. -/
theorem select_eq_none_iff : ∀ l : List Node,
    select defect drift l = none ↔ ∀ n ∈ l, ¬ n.Admissible defect drift := by
  intro l
  induction l with
  | nil => simp
  | cons a rest ih =>
      rw [select_cons]
      unfold selectStep
      by_cases ha : a.Admissible defect drift
      · rw [if_pos ha]
        constructor
        · intro h; simp at h
        · intro h; exact absurd ha (h a (by simp))
      · rw [if_neg ha, ih]
        constructor
        · intro h n hn
          rcases List.mem_cons.mp hn with rfl | hn'
          · exact ha
          · exact h n hn'
        · intro h n hn; exact h n (by simp [hn])

/-- Whatever the corridor returns is a member of the list it was offered: the
    rule never invents a chart. -/
theorem select_mem : ∀ (l : List Node) {n : Node},
    select defect drift l = some n → n ∈ l := by
  intro l
  induction l with
  | nil => intro n h; simp at h
  | cons a rest ih =>
      intro n h
      rw [select_cons] at h
      unfold selectStep at h
      by_cases ha : a.Admissible defect drift
      · rw [if_pos ha] at h
        cases hopt : select defect drift rest with
        | none =>
            rw [hopt] at h
            simp only [Option.elim, Option.some.injEq] at h
            subst h; simp
        | some b =>
            rw [hopt] at h
            simp only [Option.elim, Option.some.injEq] at h
            rcases Node.better_eq a b with hb | hb
            · rw [hb] at h; subst h; simp
            · rw [hb] at h; subst h
              exact List.mem_cons_of_mem _ (ih hopt)
      · rw [if_neg ha] at h
        exact List.mem_cons_of_mem _ (ih h)

/-- **THE REFUSAL, BY THEOREM.** Whatever the corridor returns is inside its own
    budgets. A cheaper node that misses a budget is not selected at any price —
    the sentence "cheapness alone selects the dead chart" made unavailable rather
    than merely discouraged. -/
theorem select_admissible : ∀ (l : List Node) {n : Node},
    select defect drift l = some n → n.Admissible defect drift := by
  intro l
  induction l with
  | nil => intro n h; simp at h
  | cons a rest ih =>
      intro n h
      rw [select_cons] at h
      unfold selectStep at h
      by_cases ha : a.Admissible defect drift
      · rw [if_pos ha] at h
        cases hopt : select defect drift rest with
        | none =>
            rw [hopt] at h
            simp only [Option.elim, Option.some.injEq] at h
            exact h ▸ ha
        | some b =>
            rw [hopt] at h
            simp only [Option.elim, Option.some.injEq] at h
            rcases Node.better_eq a b with hb | hb
            · rw [hb] at h; exact h ▸ ha
            · rw [hb] at h; exact h ▸ ih hopt
      · rw [if_neg ha] at h
        exact ih h

/-- **THE MINIMALITY.** The selected node is at most as expensive as every
    admissible candidate. With `select_admissible` this is `argmin price subject
    to the budgets` — the subjection carried by that theorem, the argmin by this
    one. -/
theorem select_min : ∀ (l : List Node) {n : Node},
    select defect drift l = some n →
      ∀ m ∈ l, m.Admissible defect drift → n.price ≤ m.price := by
  intro l
  induction l with
  | nil => intro n h; simp at h
  | cons a rest ih =>
      intro n h m hm hadm
      rw [select_cons] at h
      unfold selectStep at h
      by_cases ha : a.Admissible defect drift
      · rw [if_pos ha] at h
        cases hopt : select defect drift rest with
        | none =>
            rw [hopt] at h
            simp only [Option.elim, Option.some.injEq] at h
            subst h
            rcases List.mem_cons.mp hm with rfl | hm'
            · exact le_rfl
            · exact absurd hadm ((select_eq_none_iff rest).mp hopt m hm')
        | some b =>
            rw [hopt] at h
            simp only [Option.elim, Option.some.injEq] at h
            subst h
            rcases List.mem_cons.mp hm with rfl | hm'
            · exact Node.better_price_le_left _ _
            · exact le_trans (Node.better_price_le_right _ _) (ih hopt m hm' hadm)
      · rw [if_neg ha] at h
        rcases List.mem_cons.mp hm with rfl | hm'
        · exact absurd hadm ha
        · exact ih h m hm' hadm

/-- A node outside its budgets is never the selection, however cheap it is. -/
theorem cheap_but_over_budget_not_selected (l : List Node) {n c : Node}
    (hsel : select defect drift l = some n)
    (hc : ¬ c.Admissible defect drift) : n ≠ c := fun heq =>
  hc (heq ▸ select_admissible l hsel)

end CIRISHolon
