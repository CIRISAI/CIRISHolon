/-
CIRISHolon.Budget — the quantitative scaling warrant: errors compose, and for
a non-expansive tier they only ADD.

Transplanted whole from CIRISAI/CIRISOntology (Core/Aggregation.lean).
Engineering reading: a tier update that depends on a radius-r neighbourhood
up to error ε with amplification K composes with another such map at the sum
of the radii and error εG + KG·εF (lipDependsWithin_comp); over n steps the
budget is the geometric sum, and for K ≤ 1 it is LINEAR — at most n·ε
(horizonBudget_le_of_nonexpansive). That is the theorem behind the battery's
K ≤ 1.05 gate and the reason the engine's residual arithmetic is allowed to
add. Comment references to Core/*.lean files refer to the upstream repository.
-/
import Mathlib.Analysis.SpecialFunctions.Exponential
import Mathlib.Analysis.SpecificLimits.Basic
import Mathlib.Tactic

namespace CIRISHolon.Budget

/-- Exact locality (the ε = 0 case), inlined from the upstream
    Core/Locality.lean so this file is self-contained: the next value at any
    site is unchanged by edits outside the site's r-ball. -/
def DependsWithinExact {V S : Type*} (d : V → V → ℕ) (r : ℕ)
    (F : (V → S) → (V → S)) : Prop :=
  ∀ (v : V) (x y : V → S), (∀ w, d v w ≤ r → x w = y w) → F x v = F y v


open Finset

variable {V : Type*} {S : Type*} [PseudoMetricSpace S]

/-- **THE APPROXIMATE WARRANT.** `F` depends within radius `r` UP TO `ε`: two
    states agreeing inside the `r`-ball produce outputs differing by at most
    `ε`. `DependsWithinExact` is the case `ε = 0`. -/
def DependsWithinUpTo (d : V → V → ℕ) (r : ℕ) (ε : ℝ)
    (F : (V → S) → (V → S)) : Prop :=
  ∀ (v : V) (x y : V → S), (∀ w, d v w ≤ r → x w = y w) → dist (F x v) (F y v) ≤ ε

/-- Half of faithfulness, and the half that needs nothing: an exactly local map
    is local up to zero error. True in any pseudometric. -/
theorem dependsWithinUpTo_zero_of_exact (d : V → V → ℕ) (r : ℕ)
    (F : (V → S) → (V → S)) (h : DependsWithinExact d r F) :
    DependsWithinUpTo d r 0 F := by
  intro v x y hxy
  rw [h v x y hxy]
  simp

/-- **FAITHFULNESS — the generalization extends the rung it claims to extend.**
    At `ε = 0` the approximate predicate IS the exact one. Without this the file
    would be adjacent to `Core/Locality.lean` rather than continuous with it.

    NOTE THE HYPOTHESIS, and it is not bookkeeping: this direction needs a
    genuine METRIC, because in a pseudometric two distinct states can sit at
    distance zero and `ε = 0` then means INDISTINGUISHABLE rather than EQUAL.
    So the approximate warrant reduces to the exact one exactly when the state
    space separates points — which is the same fence `Core/FrameEntropy` draws
    between a reading and a state, met here in the metric coordinate. The
    engine's states (positions, velocities, occupancies) separate; a chart's
    readings need not, and for those the reduction genuinely fails. -/
theorem dependsWithinUpTo_zero_iff {S : Type*} [MetricSpace S] (d : V → V → ℕ)
    (r : ℕ) (F : (V → S) → (V → S)) :
    DependsWithinUpTo d r 0 F ↔ DependsWithinExact d r F := by
  constructor
  · intro h v x y hxy
    exact dist_le_zero.mp (h v x y hxy)
  · exact dependsWithinUpTo_zero_of_exact d r F

/-- Slack is monotone: a radius-`r`, error-`ε` map is also error-`ε'` for any
    `ε' ≥ ε`. Error bounds are upper bounds, as they must be. -/
theorem dependsWithinUpTo_mono_err {d : V → V → ℕ} {r : ℕ} {ε ε' : ℝ}
    (h : ε ≤ ε') {F : (V → S) → (V → S)} (hF : DependsWithinUpTo d r ε F) :
    DependsWithinUpTo d r ε' F :=
  fun v x y hxy => le_trans (hF v x y hxy) h

/-- And in the radius, exactly as in the exact case. -/
theorem dependsWithinUpTo_mono_radius {d : V → V → ℕ} {r r' : ℕ} (h : r ≤ r')
    {ε : ℝ} {F : (V → S) → (V → S)} (hF : DependsWithinUpTo d r ε F) :
    DependsWithinUpTo d r' ε F :=
  fun v x y hxy => hF v x y (fun w hw => hxy w (le_trans hw h))

/-! ### Rung (i): composition, and why it needs an amplification hypothesis

In the exact case, `depends_within_comp` is three lines: equality passes through
any function whatever. Here the outer map can AMPLIFY the inner map's error, so
the outer map's sensitivity must enter the statement. `LipDependsWithin` is the
same locality predicate carrying that sensitivity: agreement to within `δ` on
the ball yields output error at most `ε + K·δ`. At `δ = 0` it is
`DependsWithinUpTo`, so it is a strengthening in the amplification coordinate
only. -/

/-- Locality with an error AND an amplification constant: inputs agreeing to
    within `δ` on the `r`-ball give outputs within `ε + K·δ`. -/
def LipDependsWithin (d : V → V → ℕ) (r : ℕ) (ε K : ℝ)
    (F : (V → S) → (V → S)) : Prop :=
  ∀ (v : V) (δ : ℝ) (x y : V → S),
    (∀ w, d v w ≤ r → dist (x w) (y w) ≤ δ) → dist (F x v) (F y v) ≤ ε + K * δ

/-- The amplification form implies the plain form: take `δ = 0`. -/
theorem dependsWithinUpTo_of_lip {d : V → V → ℕ} {r : ℕ} {ε K : ℝ}
    {F : (V → S) → (V → S)} (hF : LipDependsWithin d r ε K F) :
    DependsWithinUpTo d r ε F := by
  intro v x y hxy
  have h := hF v 0 x y (fun w hw => by rw [hxy w hw]; simp)
  simpa using h

/-- **COMPOSITION WITH ADDITIVE ERROR — rung (i).** The outer map's own error
    plus its sensitivity times the inner map's error, at the sum of the radii.
    The `K·εF` term is exactly what the exact case does not pay, and the
    Lipschitz hypothesis on the OUTER map is what makes the payment finite. -/
theorem lipDependsWithin_comp {d : V → V → ℕ}
    (htri : ∀ a b c, d a c ≤ d a b + d b c)
    {r s : ℕ} {εF KF εG KG : ℝ}
    {F G : (V → S) → (V → S)}
    (hF : LipDependsWithin d r εF KF F)
    (hG : LipDependsWithin d s εG KG G) :
    DependsWithinUpTo d (s + r) (εG + KG * εF) (fun x => G (F x)) := by
  intro v x y hxy
  -- On the s-ball around v, the two evolved states differ by at most εF,
  -- because each such site's own r-ball sits inside the (s+r)-ball of v.
  have hinner : ∀ w, d v w ≤ s → dist (F x w) (F y w) ≤ εF := by
    intro w hw
    have := dependsWithinUpTo_of_lip hF w x y (fun u hu => hxy u (by
      calc d v u ≤ d v w + d w u := htri v w u
        _ ≤ s + r := Nat.add_le_add hw hu))
    exact this
  exact hG v εF (F x) (F y) hinner

/-! ### Rung (ii): the horizon, and the non-expansive case the engine lives in -/

/-- The geometric error budget of `n` steps at per-step error `ε` and
    amplification `K`: `ε · ∑_{i<n} K^i`. -/
noncomputable def horizonBudget (ε K : ℝ) (n : ℕ) : ℝ := ε * ∑ i ∈ range n, K ^ i

/-- **THE NON-EXPANSIVE BUDGET IS LINEAR.** For `K ≤ 1` (and `K ≥ 0`) the
    budget is at most `n·ε` — errors ADD rather than compounding. This is the
    case a stable integrator satisfies (`Core/Habit.lean`: the same constant
    that makes the step injective), and it is why the engine's residual
    arithmetic is allowed to add. -/
theorem horizonBudget_le_of_nonexpansive {ε K : ℝ} (hε : 0 ≤ ε)
    (hK0 : 0 ≤ K) (hK1 : K ≤ 1) (n : ℕ) :
    horizonBudget ε K n ≤ n * ε := by
  unfold horizonBudget
  have hsum : ∑ i ∈ range n, K ^ i ≤ n := by
    calc ∑ i ∈ range n, K ^ i ≤ ∑ _i ∈ range n, (1 : ℝ) :=
          Finset.sum_le_sum (fun i _ => pow_le_one₀ hK0 hK1)
      _ = n := by simp
  calc ε * ∑ i ∈ range n, K ^ i ≤ ε * n := by
        exact mul_le_mul_of_nonneg_left hsum hε
    _ = n * ε := by ring

/-- The expansive case, for completeness: the budget is the geometric sum, and
    it is exactly what compounds when `K > 1`. Stated so the linear case above
    is visibly a SPECIAL case rather than the general claim. -/
theorem horizonBudget_eq_geom {ε K : ℝ} (hK : K ≠ 1) (n : ℕ) :
    horizonBudget ε K n = ε * ((K ^ n - 1) / (K - 1)) := by
  unfold horizonBudget
  rw [geom_sum_eq hK]

/-! ### Rung (ii)b: the horizon — n steps of one habit -/

/-- **THE HORIZON WITH ERROR.** `n` steps of a radius-`r`, error-`ε`,
    amplification-`K` map depend within `n·r` up to the geometric budget
    `ε·∑_{i<n} K^i`. The exact case (`ε = 0`) collapses to
    `Locality.iterate_depends_within`; the non-expansive case is linear by
    `horizonBudget_le_of_nonexpansive`, and that is the engine's regime. -/
theorem iterate_dependsWithinUpTo {d : V → V → ℕ}
    (hrefl : ∀ a, d a a = 0) (htri : ∀ a b c, d a c ≤ d a b + d b c)
    {r : ℕ} {ε K : ℝ}
    {F : (V → S) → (V → S)} (hF : LipDependsWithin d r ε K F) :
    ∀ n : ℕ, DependsWithinUpTo d (n * r) (horizonBudget ε K n) F^[n] := by
  intro n
  induction n with
  | zero =>
      intro v x y hxy
      simp only [Function.iterate_zero, id_eq, horizonBudget, range_zero,
        Finset.sum_empty, mul_zero]
      rw [hxy v (by simp [hrefl v])]
      simp
  | succ k ih =>
      -- F^[k+1] = F ∘ F^[k]: the OUTER map is one step, the inner is k steps.
      intro v x y hxy
      have hinner : ∀ w, d v w ≤ r → dist (F^[k] x w) (F^[k] y w) ≤ horizonBudget ε K k := by
        intro w hw
        exact ih w x y (fun u hu => hxy u (by
          calc d v u ≤ d v w + d w u := htri v w u
            _ ≤ r + k * r := Nat.add_le_add hw hu
            _ = (k + 1) * r := by ring))
      have hstep := hF v (horizonBudget ε K k) (F^[k] x) (F^[k] y) hinner
      have hbudget : ε + K * horizonBudget ε K k = horizonBudget ε K (k + 1) := by
        unfold horizonBudget
        have hs : ∑ i ∈ range (k + 1), K ^ i = K * (∑ i ∈ range k, K ^ i) + 1 := by
          rw [Finset.sum_range_succ']
          simp only [pow_succ, pow_zero]
          rw [← Finset.sum_mul]
          ring
        rw [hs]; ring
      rw [Function.iterate_succ']
      simpa [hbudget] using hstep

/-! ### Rung (iii): N1 — the aggregation warrant, with its residual

The exact collar law (`Locality.restrict_factors_through_collar`) says the state
on a region IS a function of the collar's initial data. With an error term the
honest statement weakens exactly one notch: the state on the region is within
the horizon budget of a function of the collar's data. That residual is the
boundary-supported leak PROGRAM.md's N1 asks for, and the function `g` is the
boundary-supported evolution it names. -/

/-- **N1, THE AGGREGATION WARRANT WITH A RESIDUAL.** After `n` steps of a
    radius-`r` update carrying per-step error `ε` and amplification `K`, the
    value at any site is within `horizonBudget ε K n` of a function of the
    initial data on that site's `n·r`-ball. Fine evolution bounded by a
    boundary-supported residual: the coarse tier's warrant, with its price. -/
theorem aggregation_warrant [Inhabited S] {d : V → V → ℕ}
    (hrefl : ∀ a, d a a = 0) (htri : ∀ a b c, d a c ≤ d a b + d b c)
    {r : ℕ} {ε K : ℝ}
    {F : (V → S) → (V → S)} (hF : LipDependsWithin d r ε K F)
    (n : ℕ) (v : V) [∀ w, Decidable (d v w ≤ n * r)] :
    ∃ g : ({w : V // d v w ≤ n * r} → S) → S,
      ∀ x : V → S, dist (F^[n] x v) (g (fun w => x w.val)) ≤ horizonBudget ε K n := by
  classical
  refine ⟨fun s => F^[n] (fun w => if h : d v w ≤ n * r then s ⟨w, h⟩ else default) v, ?_⟩
  intro x
  exact iterate_dependsWithinUpTo hrefl htri hF n v x _ (fun w hw => by simp [hw])

/-! ### Rung (iv): the quantitative instance — the shape the engine computes

`locality.rs` bounds a horizon leak by `exp(z) · z^d/d!`. That is the tail of
the exponential series past its `d`-th term, and the bound is a consequence of
one factorial fact. Nothing here is about a propagator; it is the inequality
the engine's number rests on, proved. -/

/-- The factorial bound underneath the tail estimate: `d! · k! ≤ (d+k)!`, from
    Mathlib's divisibility statement. -/
theorem factorial_mul_factorial_le (d k : ℕ) :
    (d.factorial * k.factorial : ℝ) ≤ ((d + k).factorial : ℝ) := by
  have h : d.factorial * k.factorial ≤ (d + k).factorial :=
    Nat.le_of_dvd (Nat.factorial_pos _) (Nat.factorial_mul_factorial_dvd_factorial_add d k)
  exact_mod_cast h

/-- **THE TAIL BOUND, term by term.** For `0 ≤ z`, the `(d+k)`-th term of the
    exponential series is at most `(z^d/d!) · (z^k/k!)` — which is what makes
    the tail sum factor as `(z^d/d!)·exp z`, the shape `locality.rs` computes. -/
theorem exp_term_tail_le {z : ℝ} (hz : 0 ≤ z) (d k : ℕ) :
    z ^ (d + k) / ((d + k).factorial : ℝ)
      ≤ (z ^ d / d.factorial) * (z ^ k / k.factorial) := by
  have hd : (0 : ℝ) < d.factorial := by exact_mod_cast Nat.factorial_pos d
  have hk : (0 : ℝ) < k.factorial := by exact_mod_cast Nat.factorial_pos k
  have hprod : (0 : ℝ) < d.factorial * k.factorial := mul_pos hd hk
  have hle := factorial_mul_factorial_le d k
  have hnum : (0 : ℝ) ≤ z ^ d * z ^ k := mul_nonneg (pow_nonneg hz _) (pow_nonneg hz _)
  rw [pow_add, div_mul_div_comm]
  exact div_le_div_of_nonneg_left hnum hprod hle

/-- **THE HORIZON LEAK, summed — the engine's `z^d/d!` in Lean.** The partial
    sums of the exponential tail past the `d`-th term are bounded by
    `(z^d/d!)·exp z`, uniformly in how far the sum runs. This is the inequality
    `locality.rs` rests its bound on, proved; the identification of this SHAPE
    with a Lieb–Robinson propagator bound stays BY PAPER, as the header says. -/
theorem tail_le_exp_mul {z : ℝ} (hz : 0 ≤ z) (d n : ℕ) :
    ∑ k ∈ range n, z ^ (d + k) / ((d + k).factorial : ℝ)
      ≤ (z ^ d / d.factorial) * ∑ k ∈ range n, z ^ k / k.factorial := by
  rw [Finset.mul_sum]
  exact Finset.sum_le_sum (fun k _ => exp_term_tail_le hz d k)

end CIRISHolon.Budget
