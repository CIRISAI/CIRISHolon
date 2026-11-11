/-
CIRISHolon.SceneCert — the certificate for a GENERATED scene standing in for a
real one. Owed by issue #3 (2026-11-11), drafted the same day.

`Closed v T` certifies a coarse view against a finer MOTION. A generated scene
(streets under a population cell, a halo catalogue under a density field, a
solvent box under a thermodynamic state) has no finer motion to commute with:
what stands below it is a REFERENCE SAMPLE of reality. This file says what kind
of object that certificate is, and it is two objects, not one.

  (i)   THE SECTION LAW — exact. A generator is a section of the coarse view:
        `v (gen c s) = c` for EVERY seed. This is issue #3 part 2 (conditional
        consistency), and it is the only half that is a square. It composes
        with levels of detail (`Generator.comp`) and under an ordinary tier
        with no further precondition (`Generator.under_tier`): the coarse
        motion of a generated scene is the parent's, whatever the seed.
        `section_does_not_fix_fibre` is its fence: the law says nothing about
        which point of the fibre was generated.

  (ii)  THE MATCH — statistical, and NOT a square. A transport cost between the
        generated ensemble and the reference under a coupling, read on the
        declared observables. It obeys the triangle (`cost_triangle`: reality →
        heavy model → distilled rules adds budgets) and contracts under a
        Lipschitz consumer (`cost_map_le`, `mean_reading_le`).

  (iii) THE FENCE ON (ii). Matching every observable's own distribution does
        not bound a consumer that reads two of them together:
        `marginals_match_joint_differs`. A per-observable budget certifies
        per-observable consumers and no others.

  (iv)  COMPOSITION (issue #3 part 7). Reality → generated scene → coarse view.
        `stacked_le`: `K · cost + n · ε`. Its preconditions are in the
        statement and envelope containment is not among the sufficient ones:
        the consumer FACTORS through the certified observables (it is typed
        `Q → ℝ`), it is `K`-Lipschitz there, the cost is on the JOINT reading
        `o`, and the tier's own certificate holds on the REAL scenes.

  (v)   THE VERDICT (parts 3 and 4). Outside the envelope: refused. Inside,
        below the resolution floor: unresolved, whatever the match reads.
        `verdict_certified_iff`, `unresolved_ignores_match`.

What is NOT here and stays a tested property: that any particular generator
meets its budget; the effective-sample-size rule behind `resolved`; the
reference's own sampling error against reality (a budget term of its own —
the certificate is against the SAMPLE); determinism across platforms.
The envelope's non-transport between classes is `Transport.lean`'s second
square and is not restated.
-/
import Mathlib.Analysis.Normed.Group.Basic
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Tactic
import CIRISHolon.Tier

namespace CIRISHolon.SceneCert

open Finset CIRISHolon.Object

/-! ### (i) The section law: conditional consistency, exact, for every seed -/

/-- A generator under a coarse view `v`: from a parent reading `c` and a seed
    `s` it produces a fine scene, and the scene aggregates back to its parent.
    The law is a FIELD: a `Generator` cannot be constructed without it, and it
    is quantified over every seed, not held on average. -/
structure Generator (C S F : Type*) (v : F → C) where
  gen : C → S → F
  aggregates : ∀ c s, v (gen c s) = c

namespace Generator

variable {C S F : Type*} {v : F → C}

/-- Anything read through the coarse view is blind to the seed. -/
theorem coarse_reading_seed_free {D : Type*} (g : Generator C S F v)
    (u : C → D) (c : C) (s s' : S) :
    u (v (g.gen c s)) = u (v (g.gen c s')) := by
  rw [g.aggregates, g.aggregates]

/-- **Levels of detail stack.** A generator of tiles under cells, over a
    generator of cells under regions, is a generator of tiles under regions,
    seeded by the pair. -/
def comp {S' G : Type*} {w : G → F} (g : Generator C S F v)
    (k : Generator F S' G w) : Generator C (S × S') G (v ∘ w) where
  gen := fun c p => k.gen (g.gen c p.1) p.2
  aggregates := by
    intro c p
    show v (w (k.gen (g.gen c p.1) p.2)) = c
    rw [k.aggregates, g.aggregates]

/-- **The exact half composes under a tier with nothing more to state.** If
    the generator sits under a certified tier's view, the tier's reading of a
    generated scene after one fine step is the coarse update of the PARENT,
    for every seed. No generated detail can move a Closed reading. -/
theorem under_tier (t : Tier F C) (g : Generator C S F t.view) (c : C) (s : S) :
    t.view (t.step (g.gen c s)) = t.rate c := by
  rw [t.certifies_apply, g.aggregates]

end Generator

/-- **THE FENCE ON THE SECTION LAW.** Two generators can both aggregate
    exactly and disagree on a fine reading. Conditional consistency does not
    buy the distributional match; part 2 of the certificate never discharges
    part 1. -/
theorem section_does_not_fix_fibre :
    ∃ (g₁ g₂ : Generator Unit Unit Bool (fun _ => ())) (q : Bool → Bool),
      q (g₁.gen () ()) ≠ q (g₂.gen () ()) :=
  ⟨⟨fun _ _ => true, fun _ _ => rfl⟩, ⟨fun _ _ => false, fun _ _ => rfl⟩, id,
    by decide⟩

/-! ### (ii) The match: a transport cost under a coupling -/

section Cost

variable {ι Q : Type*} [PseudoMetricSpace Q]

/-- The cost of a coupling: `x i` and `y i` are the two ensembles' readings at
    the `i`-th pair. Any coupling's cost is an upper bound on the Wasserstein-1
    distance between the two empirical measures; for real readings paired in
    sorted order it IS that distance. Stating the certificate on a coupling
    keeps it finitary. -/
def cost (s : Finset ι) (x y : ι → Q) : ℝ := ∑ i ∈ s, dist (x i) (y i)

theorem cost_nonneg (s : Finset ι) (x y : ι → Q) : 0 ≤ cost s x y :=
  Finset.sum_nonneg fun _ _ => dist_nonneg

/-- **Distillation chains add.** Reality against the heavy model, the heavy
    model against the distilled rules: the budgets sum, on the metric alone. -/
theorem cost_triangle (s : Finset ι) (x y z : ι → Q) :
    cost s x z ≤ cost s x y + cost s y z := by
  unfold cost
  rw [← Finset.sum_add_distrib]
  exact Finset.sum_le_sum fun i _ => dist_triangle _ _ _

/-- **A Lipschitz consumer contracts the certificate.** -/
theorem cost_map_le {R : Type*} [PseudoMetricSpace R] (φ : Q → R) (K : ℝ)
    (hφ : ∀ a b, dist (φ a) (φ b) ≤ K * dist a b) (s : Finset ι) (x y : ι → Q) :
    cost s (φ ∘ x) (φ ∘ y) ≤ K * cost s x y := by
  unfold cost
  rw [Finset.mul_sum]
  exact Finset.sum_le_sum fun i _ => hφ _ _

/-- The ensemble total of a `K`-Lipschitz reading moves by at most `K` times
    the cost. This is the budget arithmetic: a consumer with tolerance `τ` on
    its total owes the scene a budget of `τ / K`, so a budget belongs to an
    (observable, consumer) pair and never to an observable alone. -/
theorem mean_reading_le (φ : Q → ℝ) (K : ℝ)
    (hφ : ∀ a b, dist (φ a) (φ b) ≤ K * dist a b) (s : Finset ι) (x y : ι → Q) :
    |∑ i ∈ s, φ (x i) - ∑ i ∈ s, φ (y i)| ≤ K * cost s x y := by
  rw [← Finset.sum_sub_distrib]
  calc |∑ i ∈ s, (φ (x i) - φ (y i))|
      ≤ ∑ i ∈ s, |φ (x i) - φ (y i)| := Finset.abs_sum_le_sum_abs _ _
    _ ≤ ∑ i ∈ s, K * dist (x i) (y i) :=
        Finset.sum_le_sum fun i _ => by
          rw [← Real.dist_eq]; exact hφ _ _
    _ = K * cost s x y := by unfold cost; rw [Finset.mul_sum]

/-! ### (iv) Composition: reality → generated scene → coarse view -/

/-- **THE STACKED BOUND.** `gen i` and `real i` are coupled scenes; `o` is the
    JOINT reading of the certified observables; `coarse` is the downstream
    tier's reading, which sees a scene only through `o`; `truth` is what the
    real scene would have read. If `coarse` is `K`-Lipschitz and the tier's
    own certificate holds within `ε` on the real scenes, the downstream error
    of running on the generated scenes is at most `K · cost + n · ε`.

    Every hypothesis is a precondition of issue #3 part 7 and none is implied
    by envelope containment: factoring (in the type of `coarse`), the Lipschitz
    constant, the cost taken on the joint reading, and `htier` on REAL scenes
    (a tier certified only on generated scenes gives no bound here). -/
theorem stacked_le {Scene : Type*} (o : Scene → Q) (coarse : Q → ℝ)
    (truth : Scene → ℝ) (K ε : ℝ)
    (hK : ∀ a b, dist (coarse a) (coarse b) ≤ K * dist a b)
    (s : Finset ι) (gen real : ι → Scene)
    (htier : ∀ i ∈ s, |coarse (o (real i)) - truth (real i)| ≤ ε) :
    ∑ i ∈ s, |coarse (o (gen i)) - truth (real i)|
      ≤ K * cost s (o ∘ gen) (o ∘ real) + s.card * ε := by
  have hpt : ∀ i ∈ s, |coarse (o (gen i)) - truth (real i)|
      ≤ K * dist (o (gen i)) (o (real i)) + ε := by
    intro i hi
    have h1 := hK (o (gen i)) (o (real i))
    rw [Real.dist_eq] at h1
    have h2 := htier i hi
    have hsplit : coarse (o (gen i)) - truth (real i)
        = (coarse (o (gen i)) - coarse (o (real i)))
          + (coarse (o (real i)) - truth (real i)) := by ring
    rw [hsplit]
    exact (abs_add _ _).trans (add_le_add h1 h2)
  calc ∑ i ∈ s, |coarse (o (gen i)) - truth (real i)|
      ≤ ∑ i ∈ s, (K * dist (o (gen i)) (o (real i)) + ε) := Finset.sum_le_sum hpt
    _ = K * cost s (o ∘ gen) (o ∘ real) + s.card * ε := by
        unfold cost
        rw [Finset.sum_add_distrib, Finset.mul_sum, Finset.sum_const, nsmul_eq_mul]
        rfl

end Cost

/-! ### (iii) The fence: matched marginals, different joint -/

/-- Two scenes per ensemble, two Boolean observables per scene. In `ensA` the
    observables move together; in `ensB` they move against each other. -/
def ensA : Fin 2 → Bool × Bool := fun i => if i = 0 then (false, false) else (true, true)
def ensB : Fin 2 → Bool × Bool := fun i => if i = 0 then (false, true) else (true, false)

/-- How many scenes of the ensemble an observable fires on. For a Boolean
    observable this count is its whole distribution. -/
def fires (e : Fin 2 → Bool × Bool) (o : Bool × Bool → Bool) : ℕ :=
  (Finset.univ.filter fun i => o (e i) = true).card

/-- **MARGINALS DO NOT CERTIFY A JOINT CONSUMER.** Each observable has the
    same distribution in both ensembles, so every per-observable distance is
    zero and every per-observable budget is met; a consumer reading the two
    together ("both agree") fires on every scene of one ensemble and none of
    the other. This is the correlated outage the twin already met (mean test
    passed, distributional test refused), one level up: there the mean hid the
    distribution, here the marginals hide the dependence. -/
theorem marginals_match_joint_differs :
    fires ensA Prod.fst = fires ensB Prod.fst ∧
    fires ensA Prod.snd = fires ensB Prod.snd ∧
    fires ensA (fun p => p.1 == p.2) = 2 ∧
    fires ensB (fun p => p.1 == p.2) = 0 := by
  decide

/-! ### (v) The verdict: refused, unresolved, certified -/

/-- The three answers of issue #3. `unresolved` is neither of the others. -/
inductive Verdict
  | certified
  | refused
  | unresolved
  deriving DecidableEq, Repr

/-- The order of the questions is part of the certificate: envelope first
    (a class the reference never covered is refused by name), the resolution
    floor second (a class it covered too thinly is unresolved), the match
    last. The match is not consulted where the reference cannot judge. -/
def verdict (inEnvelope resolved pass : Bool) : Verdict :=
  if !inEnvelope then .refused
  else if !resolved then .unresolved
  else if pass then .certified
  else .refused

theorem verdict_certified_iff (e r p : Bool) :
    verdict e r p = .certified ↔ e = true ∧ r = true ∧ p = true := by
  cases e <;> cases r <;> cases p <;> decide

/-- **A confident picture where the reference cannot judge is not available.**
    Below the floor the verdict does not depend on what the match reads. -/
theorem unresolved_ignores_match (p : Bool) :
    verdict true false p = .unresolved := by
  cases p <;> rfl

/-- Outside the envelope the answer is a refusal whatever else is true. -/
theorem outside_envelope_refused (r p : Bool) :
    verdict false r p = .refused := rfl

end CIRISHolon.SceneCert
