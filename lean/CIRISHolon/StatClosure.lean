/-
CIRISHolon.StatClosure — the composition of STATISTICAL closure certificates.

Owed by FAST_AND_SLOW.md §3 (2026-09-19): `Budget.lean` composes exact and
worst-case-approximate locality; the fluid tier's certificate is an RMS ratio
`D = √(Σ residual² / Σ observed²)` over windows and cells (`holon-lens`,
`continuity`, `continuity_integral`), and RESPONSE-1 reads it on ONE cell. What
carries one cell's `D ≤ β` to a lattice of cells and to `n` steps of running the
coarse law alone? Two theorems, and one hypothesis that is not a theorem.

  (i)  SPATIAL POOLING is the mediant inequality. The lattice's pooled ratio is a
       weighted mean of the cells' ratios (weights: each cell's observed power),
       so it is at most the worst cell's. No independence, no locality: it is the
       arithmetic of the ratio. `pooled_le_of_cells`, `pooled_eq_weighted_mean`.
  (ii) THE STATISTICAL HORIZON is Minkowski plus non-expansiveness. In the space
       whose seminorm is the RMS over the ensemble, if the coarse law `h` is
       `K`-Lipschitz and the true coarse step `T` differs from it by at most `ε`
       in RMS, then running `h` alone for `n` steps drifts from the truth by at
       most the geometric budget, linear `n·ε` for `K ≤ 1`. Independence of the
       residuals is NOT needed for this bound; it would buy `√n` and is not
       claimed. `iterate_dist_le_horizonBudget`, `iterate_dist_le_linear`.

  The HYPOTHESIS that no theorem here discharges: a cell certified under the TRUE
  neighbours of its own box (the arm) behaves the same under COARSE neighbours
  (the lattice). That is `Budget.LipDependsWithin` at radius one on the cell
  graph — locality of the coarse law — and it is what Leg B's held-out boundary
  histories TEST. Once it holds, `Budget.lipDependsWithin_comp` chains cells in
  space and the theorems here carry the certificate in count and in time.
-/
import Mathlib.Analysis.Normed.Group.Basic
import Mathlib.Algebra.Order.BigOperators.Group.Finset
import Mathlib.Tactic
import CIRISHolon.Budget

namespace CIRISHolon.StatClosure

open Finset

/-! ### (i) Spatial pooling: the mediant -/

/-- **THE LATTICE'S DEFECT IS AT MOST THE WORST CELL'S.** If every cell `c` has
    residual power `R c` and observed power `O c > 0` with `R c ≤ β² · O c`
    (its own `D_c ≤ β`), the pooled ratio `Σ R / Σ O` is at most `β²`. This is
    what lets a certificate read on one cell (RESPONSE-1's 432-water box) stand
    for a lattice of them, cell by cell, with no assumption on how the cells'
    residuals are correlated. -/
theorem pooled_le_of_cells {ι : Type*} (s : Finset ι) (hs : s.Nonempty)
    (R O : ι → ℝ) (β : ℝ)
    (hO : ∀ c ∈ s, 0 < O c) (hcell : ∀ c ∈ s, R c ≤ β ^ 2 * O c) :
    (∑ c ∈ s, R c) / (∑ c ∈ s, O c) ≤ β ^ 2 := by
  have hsumO : 0 < ∑ c ∈ s, O c := Finset.sum_pos hO hs
  rw [div_le_iff₀ hsumO, Finset.mul_sum]
  exact Finset.sum_le_sum hcell

/-- The pooled ratio IS a weighted mean of the cells' ratios, weighted by each
    cell's observed power — so it also lies at or above the best cell's, and a
    lattice cannot launder a bad cell below its best one. -/
theorem pooled_eq_weighted_mean {ι : Type*} (s : Finset ι)
    (R O : ι → ℝ) (hO : ∀ c ∈ s, 0 < O c) (hsum : 0 < ∑ c ∈ s, O c) :
    (∑ c ∈ s, R c) / (∑ c ∈ s, O c)
      = ∑ c ∈ s, (O c / ∑ c' ∈ s, O c') * (R c / O c) := by
  rw [Finset.sum_div]
  apply Finset.sum_congr rfl
  intro c hc
  have h := hO c hc
  field_simp
  ring

/-- The defect itself, as the instrument prints it: `D = √(ratio)`. Monotone,
    so the bound on the ratio is the bound on `D`. -/
theorem pooled_defect_le_of_cells {ι : Type*} (s : Finset ι) (hs : s.Nonempty)
    (R O : ι → ℝ) (β : ℝ) (hβ : 0 ≤ β)
    (hO : ∀ c ∈ s, 0 < O c) (hcell : ∀ c ∈ s, R c ≤ β ^ 2 * O c) :
    Real.sqrt ((∑ c ∈ s, R c) / (∑ c ∈ s, O c)) ≤ β := by
  have h := pooled_le_of_cells s hs R O β hO hcell
  calc Real.sqrt ((∑ c ∈ s, R c) / (∑ c ∈ s, O c))
      ≤ Real.sqrt (β ^ 2) := Real.sqrt_le_sqrt h
    _ = β := Real.sqrt_sq hβ

/-! ### (ii) The statistical horizon: RMS errors under a non-expansive coarse law -/

variable {E : Type*} [SeminormedAddCommGroup E]

/-- **RUNNING THE COARSE LAW ALONE.** `T` is the true coarse step (the fine
    dynamics seen through the view), `h` the law the engine runs in its place;
    the seminorm is the RMS over the ensemble (windows × seeds), so
    `‖T x − h x‖ ≤ ε` is the one-step certificate `D ≤ β` with `ε = β · ‖observed‖`,
    and `‖h x − h y‖ ≤ K ‖x − y‖` is the law's non-expansiveness in RMS. Then
    after `n` steps the run has drifted from the truth by at most the geometric
    budget of `Budget.lean`. Minkowski is the whole proof: the new error is the
    new residual plus the amplified old error. -/
theorem horizonBudget_succ (ε K : ℝ) (n : ℕ) :
    Budget.horizonBudget ε K (n + 1) = ε + K * Budget.horizonBudget ε K n := by
  unfold Budget.horizonBudget
  have hs : ∑ i ∈ range (n + 1), K ^ i = 1 + K * ∑ i ∈ range n, K ^ i := by
    rw [Finset.sum_range_succ', pow_zero, Finset.mul_sum]
    simp only [pow_succ']
    ring
  rw [hs]
  ring

theorem iterate_dist_le_horizonBudget (T h : E → E) (ε K : ℝ) (hK0 : 0 ≤ K)
    (hres : ∀ x, ‖T x - h x‖ ≤ ε)
    (hlip : ∀ x y, ‖h x - h y‖ ≤ K * ‖x - y‖) (x : E) :
    ∀ n : ℕ, ‖T^[n] x - h^[n] x‖ ≤ Budget.horizonBudget ε K n := by
  intro n
  induction n with
  | zero => simp [Budget.horizonBudget]
  | succ n ih =>
      rw [Function.iterate_succ_apply', Function.iterate_succ_apply']
      have step : ‖T (T^[n] x) - h (h^[n] x)‖
          ≤ ‖T (T^[n] x) - h (T^[n] x)‖ + ‖h (T^[n] x) - h (h^[n] x)‖ := by
        have := norm_sub_le_norm_sub_add_norm_sub (T (T^[n] x)) (h (T^[n] x)) (h (h^[n] x))
        exact this
      have hK : ‖h (T^[n] x) - h (h^[n] x)‖ ≤ K * Budget.horizonBudget ε K n := by
        calc ‖h (T^[n] x) - h (h^[n] x)‖ ≤ K * ‖T^[n] x - h^[n] x‖ := hlip _ _
          _ ≤ K * Budget.horizonBudget ε K n := mul_le_mul_of_nonneg_left ih hK0
      calc ‖T (T^[n] x) - h (h^[n] x)‖
          ≤ ‖T (T^[n] x) - h (T^[n] x)‖ + ‖h (T^[n] x) - h (h^[n] x)‖ := step
        _ ≤ ε + K * Budget.horizonBudget ε K n := add_le_add (hres _) hK
        _ = Budget.horizonBudget ε K (n + 1) := (horizonBudget_succ ε K n).symm

/-- **THE NON-EXPANSIVE CASE IS LINEAR IN RMS TOO**: for `0 ≤ K ≤ 1` the run
    drifts by at most `n·ε`. This is the statistical twin of
    `Budget.horizonBudget_le_of_nonexpansive`, and it needs no independence of
    the per-step residuals: correlated residuals still only ADD. -/
theorem iterate_dist_le_linear (T h : E → E) (ε K : ℝ) (hε : 0 ≤ ε)
    (hK0 : 0 ≤ K) (hK1 : K ≤ 1)
    (hres : ∀ x, ‖T x - h x‖ ≤ ε)
    (hlip : ∀ x y, ‖h x - h y‖ ≤ K * ‖x - y‖) (x : E) (n : ℕ) :
    ‖T^[n] x - h^[n] x‖ ≤ n * ε :=
  le_trans (iterate_dist_le_horizonBudget T h ε K hK0 hres hlip x n)
    (Budget.horizonBudget_le_of_nonexpansive hε hK0 hK1 n)

end CIRISHolon.StatClosure
