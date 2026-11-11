/-
CIRISHolon.ScenePopulation — every generated object a holon: the population law,
the edge law, and the object with two parents. Owed by issue #3 after the CEWP
twin's tier inventory (2026-11-11): the twin apportions households exactly from
a real population grid down to its leaf, and below that has no street object, no
intersection graph and no stored link from a building to its street.

`SceneCert.Generator` is a section of a coarse view. This file names the two
views a scene of OBJECTS is generated under, and shows the second is the first.

  (i)   POPULATION. Objects live at addresses `F`; an object carries a count
        `n : F → ℕ` (zero where nothing is). Containment is `v : F → C`. The
        parent's reading is `pop v n c`, the sum over its children. It is a
        view, it composes along containment (`pop_comp`), and it conserves the
        total (`pop_total`). A `PopGenerator` is a `Generator` under it:
        generation as a function of the parent's population.

  (ii)  EDGES. A relationship is a count on a PAIR of objects. The coarse edge
        between two parents is the sum of the fine edges between their
        children: `edgeView v = pop (Prod.map v v)`. So the edge law is the
        population law on pairs, and everything in (i) holds for it at once
        (`edgeView_comp`, `edgeView_total`). A relationship between two objects
        with the same parent lands on that parent's diagonal entry and on no
        other; one that crosses lands off the diagonal, where the parent's own
        parent reads it (`edgeView_internal`). That is "a relationship is owned
        by the lowest holon containing both ends", as arithmetic.

  (iii) THE FENCE ON (ii). The edge law fixes how many relationships cross, not
        which objects they join: `edge_law_does_not_fix_wiring`. Reachability
        of a particular object is a fine reading, and owes a certificate.

  (iv)  TWO PARENTS. A household sits in a geographic cell and on an access
        network. Both parents read the same children, so their totals agree
        (`two_parents_agree`) and each is a marginal of one joint table
        (`joint_marginal_left`, `joint_marginal_right`). Two parent readings
        with different totals admit no generator at all
        (`no_generator_of_totals_ne`): the twin's cohort tier and street tier,
        which drop different people at the root, cannot both parent one scene.

Addresses are fixed and finite here. A generator that creates objects is one
that writes nonzero counts at some addresses; the statement does not need the
set of objects to vary.
-/
import Mathlib.Algebra.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Prod
import Mathlib.Tactic
import CIRISHolon.SceneCert

namespace CIRISHolon.ScenePopulation

open Finset CIRISHolon.SceneCert

variable {F C D : Type*} [Fintype F] [Fintype C] [DecidableEq C] [DecidableEq D]

/-! ### (i) The population view -/

/-- The parent's reading: the sum of its children's counts. -/
def pop (v : F → C) (n : F → ℕ) (c : C) : ℕ := ∑ x, if v x = c then n x else 0

/-- **Nothing is created or lost by reading coarsely.** -/
theorem pop_total (v : F → C) (n : F → ℕ) : ∑ c, pop v n c = ∑ x, n x := by
  unfold pop
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl fun x _ => ?_
  rw [Finset.sum_eq_single (v x)]
  · simp
  · intro c _ hc
    have : ¬ v x = c := fun h => hc h.symm
    simp [this]
  · intro h; exact absurd (Finset.mem_univ _) h

/-- **Containment composes.** Reading streets into districts and districts into
    cells is reading streets into cells. -/
theorem pop_comp (v : F → C) (u : C → D) (n : F → ℕ) :
    pop u (pop v n) = pop (u ∘ v) n := by
  funext d
  unfold pop
  calc ∑ c, (if u c = d then ∑ x, (if v x = c then n x else 0) else 0)
      = ∑ c, ∑ x, (if u c = d then (if v x = c then n x else 0) else 0) := by
        refine Finset.sum_congr rfl fun c _ => ?_
        by_cases h : u c = d <;> simp [h]
    _ = ∑ x, ∑ c, (if u c = d then (if v x = c then n x else 0) else 0) :=
        Finset.sum_comm
    _ = ∑ x, (if (u ∘ v) x = d then n x else 0) := by
        refine Finset.sum_congr rfl fun x _ => ?_
        rw [Finset.sum_eq_single (v x)]
        · simp
        · intro c _ hc
          have : ¬ v x = c := fun h => hc h.symm
          simp [this]
        · intro h; exact absurd (Finset.mem_univ _) h

/-- A generator of objects under their parents' populations: for every seed the
    children's counts sum to the parent's, parent by parent. -/
abbrev PopGenerator (S : Type*) (v : F → C) := Generator (C → ℕ) S (F → ℕ) (pop v)

/-- Whatever the seed, a generated scene holds exactly the people it was
    generated under. -/
theorem PopGenerator.total {S : Type*} {v : F → C} (g : PopGenerator S v)
    (c : C → ℕ) (s : S) : ∑ x, g.gen c s x = ∑ k, c k := by
  rw [← pop_total v, g.aggregates]

/-! ### (ii) The edge view: the population law on pairs -/

/-- The coarse relationship between two parents: the sum of the fine
    relationships between their children. -/
def edgeView (v : F → C) (w : F × F → ℕ) : C × C → ℕ := pop (Prod.map v v) w

/-- **Relationships aggregate through every level.** -/
theorem edgeView_comp (v : F → C) (u : C → D) [Fintype D] (w : F × F → ℕ) :
    edgeView u (edgeView v w) = edgeView (u ∘ v) w := by
  unfold edgeView
  rw [pop_comp]
  rfl

/-- No relationship is created or lost by reading coarsely. -/
theorem edgeView_total (v : F → C) (w : F × F → ℕ) :
    ∑ p, edgeView v w p = ∑ e, w e := pop_total _ _

omit [Fintype C] in
/-- **Ownership.** A relationship between two objects with the same parent is
    read on that parent's own diagonal entry: it contributes to `(c, c)` with
    `c` their parent and, being a single fine edge, to nothing else. -/
theorem edgeView_internal [DecidableEq F] (v : F → C) (x y : F) (h : v x = v y) (p : C × C) :
    edgeView v (fun e => if e = (x, y) then 1 else 0) p
      = if p = (v x, v x) then 1 else 0 := by
  unfold edgeView pop
  rw [Finset.sum_eq_single (x, y)]
  · simp [Prod.map, h, eq_comm]
  · intro e _ he; simp [he]
  · intro hx; exact absurd (Finset.mem_univ _) hx

/-- A generator of relationships under the parents' relationships. -/
abbrev EdgeGenerator (S : Type*) (v : F → C) :=
  Generator (C × C → ℕ) S (F × F → ℕ) (edgeView v)

/-! ### (iii) The fence: the edge law does not fix the wiring -/

/-- Two cells of two objects each. An object is (its cell, its index). -/
def wiringA : (Bool × Bool) × (Bool × Bool) → ℕ :=
  fun e => if e.1.1 = false ∧ e.2.1 = true ∧ e.1.2 = e.2.2 then 1 else 0

/-- The same two crossings, both leaving from one object. -/
def wiringB : (Bool × Bool) × (Bool × Bool) → ℕ :=
  fun e => if e = ((false, false), (true, false)) then 2 else 0

/-- How many relationships leave an object. -/
def outDegree (w : (Bool × Bool) × (Bool × Bool) → ℕ) (x : Bool × Bool) : ℕ := ∑ y, w (x, y)

/-- **THE COARSE EDGE DOES NOT SAY WHO IS CONNECTED.** Both wirings put two
    relationships between the two cells and none inside either, so every
    coarse reading of them agrees. In one, each object of the first cell has a
    link out; in the other, one object has both and its neighbour has none.
    A consumer that asks whether a given home is reachable reads the fine
    graph, and the edge section law does not answer it. -/
theorem edge_law_does_not_fix_wiring :
    edgeView Prod.fst wiringA = edgeView Prod.fst wiringB ∧
    outDegree wiringA (false, true) = 1 ∧ outDegree wiringB (false, true) = 0 := by
  refine ⟨?_, by decide, by decide⟩
  funext ⟨c, d⟩
  cases c <;> cases d <;> decide

/-! ### (iv) An object with two parents -/

variable {C₂ : Type*} [Fintype C₂] [DecidableEq C₂]

/-- **Two parents of the same children hold the same total.** -/
theorem two_parents_agree (v₁ : F → C) (v₂ : F → C₂) (n : F → ℕ) :
    ∑ c, pop v₁ n c = ∑ d, pop v₂ n d := by
  rw [pop_total, pop_total]

/-- Each parent's reading is a marginal of the one joint table. -/
theorem joint_marginal_left (v₁ : F → C) (v₂ : F → C₂) (n : F → ℕ) :
    pop Prod.fst (pop (fun x => (v₁ x, v₂ x)) n) = pop v₁ n := by
  rw [pop_comp]; rfl

theorem joint_marginal_right (v₁ : F → C) (v₂ : F → C₂) (n : F → ℕ) :
    pop Prod.snd (pop (fun x => (v₁ x, v₂ x)) n) = pop v₂ n := by
  rw [pop_comp]; rfl

/-- **Parents that disagree on the total admit no scene.** If the two coarse
    readings hold different totals, no assignment of counts to objects
    aggregates to both, for any seed, by any generator. -/
theorem no_generator_of_totals_ne (v₁ : F → C) (v₂ : F → C₂)
    (c₁ : C → ℕ) (c₂ : C₂ → ℕ) (h : ∑ c, c₁ c ≠ ∑ d, c₂ d) :
    ¬ ∃ n : F → ℕ, pop v₁ n = c₁ ∧ pop v₂ n = c₂ := by
  rintro ⟨n, h₁, h₂⟩
  apply h
  rw [← h₁, ← h₂]
  exact two_parents_agree v₁ v₂ n

end CIRISHolon.ScenePopulation
