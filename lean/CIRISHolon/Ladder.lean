/-
CIRISHolon.Ladder — the theory of the generated world, stated once: universe to
vacuum. Written 2026-11-11 at the lead's word ("lock down the theory"), on top
of `SceneCert` (issue #3) and `ScenePopulation`.

THE THEORY, IN FIVE SENTENCES.

  1. Everything is a holon: an object with a POPULATION (counts of what it
     holds), a view upward (its parent reads that population), and a generator
     downward (its children, from its population and a seed).
  2. THE SECTION LAW: the children's populations sum to the parent's, for every
     seed. RELATIONSHIPS obey the same law on pairs, and are read on the lowest
     holon that holds both ends.
  3. THE LADDER: generators compose, so a chain of them from the top reading to
     the finest scene is one generator, and reading the finest scene back at any
     level returns exactly the scene generated at that level on the way down.
  4. THE MEETING: the scene comes from above (generated under a parent), the
     motion comes from below (a tier certified against the finer motion). Where
     a generator sits under a certified tier, the generated scene moves as its
     parent does, for every seed.
  5. THE FLOOR: under an empty parent every generator returns the empty scene.
     Nothing is generated from nothing; the ladder ends in the vacuum.

WHAT IS PROVED HERE.

  * `Ladder`, `collapse`            — sentence 3: any depth of generators is one.
  * `zoom_consistent`               — sentence 3: zooming out lands on the scene
                                      that was generated there; no level's scene
                                      depends on a deeper seed.
  * `meets_tier`, `meets_stack`     — sentence 4, for one tier and for two
                                      stacked tiers under a two-rung ladder.
  * `outflow_commutes`              — sentence 2 for relationships and
                                      populations together: a parent's outgoing
                                      relationships are the sum of its
                                      children's. For a signed, antisymmetric
                                      flow the internal terms cancel and this is
                                      Gauss's law: what a cell holds is what
                                      crosses its faces. It is why the fluid
                                      element carries its fluxes on the faces,
                                      and why a charge below the atom cannot
                                      exist without its flux line.
  * `vacuum_generates_vacuum`       — sentence 5.

THE LADDER AS THE RECORD DRAWS IT (`TIERS.md`, 2026-09-27), top to bottom, with
what stands under each rung today. The theorems above do not depend on this
list; the list says where they have something to act on.

  universe ............ a WAGER (`TIERS.md` §4); no remainder measured
  planet, region ...... real population and land cover to 1/8 degree (the twin)
  cell, district,
  street, building,
  household, device ... GENERATED in the twin; the section law holds to the
                        household; no reference below the cell yet (STREETREF-1)
  continuum ........... measured once; its dynamics fenced
  fluid element ....... IN THE ENGINE: densities in cells, fluxes on faces
  molecule ............ MEASURED: the rigid unit
  atom, chemistry ..... MEASURED: derived force law
  nucleus ............. DECLARED input
  the fold, vacuum .... MEASURED in 1+1D only; `Vacuum.lean` is the zero-flux
                        carrier and the promotion out of it

WHAT THE THEORY DOES NOT SAY. Each is a statement already in this lake or this
record, and none is repaired here.

  * A section is not a match. Which point of the fibre was generated is
    statistical (`SceneCert.section_does_not_fix_fibre`).
  * A count is not a wiring (`ScenePopulation.edge_law_does_not_fix_wiring`).
  * Closed is not carried. A sector can obey every law here and be something
    the tier above does not read (the hydrogen-bond network, `TIERS.md` §1).
  * Population is the diagonal. Where coherence begins the population view is
    not closed (`Object.diag_not_closed_under_coherence`), so below that line a
    scene is not a function of its populations and this file's ladder carries
    the conserved charges only.
  * No law is shared between rungs. What repeats is the procedure.
-/
import Mathlib.Algebra.BigOperators.Group.Finset
import Mathlib.Tactic
import CIRISHolon.ScenePopulation

namespace CIRISHolon.World

open Finset CIRISHolon.Object CIRISHolon.SceneCert CIRISHolon.ScenePopulation

/-! ### The ladder: any depth of generators is one generator -/

/-- A chain of generators from a top reading `C` down to a finest scene `F`.
    Each rung has its own seed type; the view of the whole is the composite of
    the rungs' views. -/
inductive Ladder : (C F : Type) → (F → C) → Type 1 where
  | rung {C F : Type} {v : F → C} (S : Type) (g : Generator C S F v) : Ladder C F v
  | step {C M F : Type} {v : M → C} {w : F → M} (S : Type) (g : Generator C S M v)
      (rest : Ladder M F w) : Ladder C F (v ∘ w)

namespace Ladder

/-- The seed of a ladder: one seed per rung. -/
def seed : {C F : Type} → {v : F → C} → Ladder C F v → Type
  | _, _, _, .rung S _ => S
  | _, _, _, .step S _ rest => S × seed rest

/-- How many rungs. -/
def depth : {C F : Type} → {v : F → C} → Ladder C F v → ℕ
  | _, _, _, .rung _ _ => 1
  | _, _, _, .step _ _ rest => depth rest + 1

/-- **A ladder is a generator.** Whatever its depth, the finest scene reads
    back to the top reading it was generated under, for every seed: the
    section law is a field of the result. -/
def collapse : {C F : Type} → {v : F → C} → (l : Ladder C F v) → Generator C (seed l) F v
  | _, _, _, .rung _ g => g
  | _, _, _, .step _ g rest => g.comp (collapse rest)

/-- The top reading survives every rung. -/
theorem top_reading {C F : Type} {v : F → C} (l : Ladder C F v) (c : C) (s : seed l) :
    v ((collapse l).gen c s) = c :=
  (collapse l).aggregates c s

/-- **ZOOM IS CONSISTENT.** Generate all the way down, then read the finest
    scene back at the first rung's level: the result is exactly the scene the
    first rung generated, and it does not depend on any deeper seed. A level
    can be drawn before the levels under it exist, and nothing changes when
    they arrive. -/
theorem zoom_consistent {C M F : Type} {v : M → C} {w : F → M} (S : Type)
    (g : Generator C S M v) (rest : Ladder M F w) (c : C) (s : S) (r : seed rest) :
    w ((collapse (.step S g rest)).gen c (s, r)) = g.gen c s :=
  (collapse rest).aggregates (g.gen c s) r

/-! ### The meeting: the scene from above, the motion from below -/

/-- **A generated world moves as its parent does.** If the ladder's view is a
    certified tier's view, then after one fine step the tier reads the coarse
    update of the top reading, for every seed at every rung. -/
theorem meets_tier {X C : Type} (t : Tier X C) (l : Ladder C X t.view) (c : C)
    (s : seed l) : t.view (t.step ((collapse l).gen c s)) = t.rate c :=
  Generator.under_tier t (collapse l) c s

/-- Two certified tiers stacked from below, two generators composed from
    above: the fine step of a scene generated two rungs down is read at the
    top as the top's own update. Each side composes on its own, and they meet
    with no further hypothesis than the two that make each a stack. -/
theorem meets_stack {X M C : Type} (lo : Tier X M) (hi : Tier M C)
    (h : hi.step = lo.rate) {S S' : Type}
    (g : Generator C S M hi.view) (k : Generator M S' X lo.view)
    (c : C) (s : S) (r : S') :
    hi.view (lo.view (lo.step (k.gen (g.gen c s) r))) = hi.rate c := by
  have hlo := lo.certifies_apply (k.gen (g.gen c s) r)
  rw [hlo, k.aggregates, ← h, hi.certifies_apply, g.aggregates]

end Ladder

/-! ### Relationships and populations together -/

section Outflow

variable {F C : Type*} [Fintype F] [Fintype C] [DecidableEq C]

/-- **THE OUTFLOW LAW.** Everything leaving a parent, to any parent including
    itself, is the sum of everything leaving its children. Reading
    relationships coarsely and then counting what leaves is the same as
    counting what leaves each object and then reading coarsely. -/
theorem outflow_commutes (v : F → C) (w : F × F → ℕ) (c : C) :
    ∑ d, edgeView v w (c, d) = pop v (fun x => ∑ y, w (x, y)) c := by
  unfold edgeView pop
  rw [Finset.sum_comm, Fintype.sum_prod_type]
  refine Finset.sum_congr rfl fun x _ => ?_
  by_cases h : v x = c
  · rw [if_pos h]
    refine Finset.sum_congr rfl fun y _ => ?_
    rw [Finset.sum_eq_single (v y)]
    · simp [Prod.map, h]
    · intro d _ hd
      have : ¬ v y = d := fun e => hd e.symm
      simp [Prod.map, this]
    · intro hh; exact absurd (Finset.mem_univ _) hh
  · rw [if_neg h]
    refine Finset.sum_eq_zero fun y _ => Finset.sum_eq_zero fun d _ => ?_
    simp [Prod.map, h]

end Outflow

/-! ### The floor -/

section Floor

variable {F C : Type*} [Fintype F] [Fintype C] [DecidableEq C]

/-- **NOTHING FROM NOTHING.** Under a parent that holds nothing, every
    generator returns the empty scene at every address, for every seed. The
    recursion has a floor, and the floor is the vacuum. -/
theorem vacuum_generates_vacuum {S : Type*} (v : F → C) (g : PopGenerator S v)
    (s : S) (x : F) : g.gen (fun _ => 0) s x = 0 := by
  have htot := g.total (fun _ => 0) s
  simp only [Finset.sum_const_zero] at htot
  exact (Finset.sum_eq_zero_iff.mp htot) x (Finset.mem_univ x)

end Floor

end CIRISHolon.World
