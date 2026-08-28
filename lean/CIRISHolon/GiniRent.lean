/-
CIRISHolon.GiniRent — the exact rent of a uniform-relaxation view, and what
the informed-rent stake actually turned out to be.

Rung 7, OMEGA-RATCHET-1, legs R-A and R-B of `OMEGA_RATCHET1_NOTE.md` §8.

WHAT THIS FILE MECHANIZES (R-A).  `CROSSFACE1_PREREG.md` Theorem 1 defines the
partition-face rent of a view as the Bayes error of the best memoryless coarse
model, `W(v) = 1 − Σ_i max_j P_ij`.  Its Theorem 3 evaluates that exactly on
UNIFORM `μ` with a normal transfer whose non-trivial spectrum is a single `λ`:
`W = (1 − 1/N)(1 − λ)`.  This file evaluates it exactly OFF the uniform line,
for the kernel class `M = λ·I + (1−λ)·Π`:

  * `rent_uniformRelax`      — unconditional closed form, every `μ`, every `λ`
  * `rent_uniformRelax_gini` — under a stated fence it collapses to
                               `(1 − Σ_i μ_i²)·(1 − λ)`: rent = Gini index ×
                               (1 − retention)
  * `rent_uniform`           — CROSS-FACE-1 Theorem 3, recovered as the
                               uniform-`μ` corollary, fence discharged
  * `rent_amnesia`           — the `λ = 0` end, `W = 1 − μ_max`, which is the
                               shape Theorem 6 returns
  * `d4_classA_rent`         — the freeze's MEASURED D4 `v_classA = 3/4`,
                               reproduced from class sizes (1,1,2,2,2)/8; and
    `d4_classA_fence_fails` / `d4_gini_would_be_wrong` — the same datum used as
                               a REFUTER: the fence fails there, and anyone who
                               extends Theorem 3 off the uniform line by pattern
                               match gets 25/32 instead of 3/4.

WHICH PART OF G1 THIS DISCHARGES, PRECISELY.  The freeze's gate G1 carries the
witness line "none (the inequality is proved in §2 by hand and checked
numerically over 4000 random kernels/views, worst residual −1.1e−16; it is not
yet mechanized)".  That line covers **Theorem 2**, the expander-mixing
INEQUALITY `W ≥ (1−μ_max) − λ·(Σσ_i)·σ_max`, for arbitrary faces.  This file
does NOT prove Theorem 2 and does not touch its witness line.  What it
discharges is narrower and should be quoted narrowly: on the kernel class
`M = λI + (1−λ)Π` the rent is now known EXACTLY by machine, so on that class
Theorem 2 is a corollary rather than a hypothesis, and Theorem 3 — previously
proved by hand for uniform `μ` — is now machine-checked and generalised.
STILL OWED, and not to be reported as done: Theorem 2 for general kernels,
Theorem 4 (the perimeter law), Theorem 6 (the class algebra), and Theorem 1
itself, whose `(≤)` half is a policy CONSTRUCTION that this file does not
formalise — `rent` here is Theorem 1's right-hand side taken as the definition.

WHAT THIS FILE RECORDS AS A FIRED KILL (R-B).  `OMEGA_RATCHET1_NOTE.md` §8
staked an "informed rent" dichotomy: that for a policy class equivariant under
a group `Γ`, and a view `v` that is not `Γ`-invariant, no policy in the class
holds `v` closed at any budget.  **That stake is false, and `staked_R_B_dichotomy_is_false`
is the counterexample**: the memory-carrying repair `R(s,s') = s` — restore the
pre-motion state, which CROSS-FACE-1's own definition permits, since a policy
is a kernel `R(s''|s,s')` that sees the pair — is `Γ`-equivariant, holds the
FINEST view closed with `F = id`, and costs a finite amount.  Equivariance of
the policy is not the obstruction.

What survives, and is proved here instead, is the statement the fiber floor
actually rests on:

  * `equivariant_kernel_colSum_eq_one` / `equivariant_uniform_invariant` — a
    `Γ`-equivariant Markov kernel with `Γ` transitive is DOUBLY STOCHASTIC, so
    the uniform measure is invariant.  This is the mechanism behind the
    maintained-holonomy campaign's measured `1/d` floor: a repair that names no
    point of the fiber leaves a fiber kernel that commutes with the fiber's
    symmetry, and such a kernel cannot prefer any point.
  * `knowing_stationary` — the separator, so the above is not vacuous: the
    design-KNOWING kernel is not equivariant and its stationary measure is
    `f* = (q + (1−q)p) / (1 − (1−q)(1−2p))`, exactly, which is `> 1/2` for
    `q > 0`.
  * `fiber_surcharge` — the staked forward number of §8 R-B, proved: on the §6
    model `W(v_full) − W(v_view) = p·(1−γ)` exactly.  The surcharge for holding
    the design is the fiber's own rent.

SCOPE.  Everything is over `ℚ` on finite types, exact.  `rent` is a count of
DISPLACED MASS per step; it is not an energy and not a bit count, and no
Landauer normalisation is asserted anywhere — the predecessor programme's K4
fired at 3–5 dex on exactly that step and is not repeated.  Nothing here is a
claim about any wild process.

Companion: `OMEGA_RATCHET1_NOTE.md` and `verify.py` (exact-rational checks of
every number below, plus the numbers this file does not reach).
-/
import Mathlib.Tactic

namespace CIRISHolon.GiniRent

open Finset

/-! ### 0. A helper the rest of the file leans on -/

/-- Nonnegative scaling commutes with a binary max. -/
lemma mul_max_of_nonneg (c a b : ℚ) (hc : 0 ≤ c) :
    c * max a b = max (c * a) (c * b) := by
  rcases le_total a b with h | h
  · rw [max_eq_right h, max_eq_right (by nlinarith)]
  · rw [max_eq_left h, max_eq_left (by nlinarith)]

/-! ### 1. The partition face, transcribed

`CROSSFACE1_PREREG.md` §1–§2: a face is `(S, K, μ, v)` with `μ` a `K`-invariant
measure; `P_ij = μ_i · K(j|i)`; and the rent of the view whose blocks are the
index set is `W = 1 − Σ_i max_j P_ij`. -/

variable {ι : Type*} [Fintype ι] [DecidableEq ι] [Nonempty ι]

/-- `max_j P_ij`, the best single successor block for block `i`. -/
noncomputable def rowSup (μ : ι → ℚ) (K : ι → ι → ℚ) (i : ι) : ℚ :=
  univ.sup' univ_nonempty (fun j => μ i * K i j)

/-- **THE RENT** of Theorem 1, with `P_ij = μ_i · K(j|i)`. -/
noncomputable def rent (μ : ι → ℚ) (K : ι → ι → ℚ) : ℚ :=
  1 - ∑ i, rowSup μ K i

/-- The largest block measure. -/
noncomputable def muMax (μ : ι → ℚ) : ℚ := univ.sup' univ_nonempty μ

/-- The smallest block measure. -/
noncomputable def muMin (μ : ι → ℚ) : ℚ := univ.inf' univ_nonempty μ

omit [DecidableEq ι] in
lemma le_muMax (μ : ι → ℚ) (i : ι) : μ i ≤ muMax μ :=
  le_sup' μ (mem_univ i)

omit [DecidableEq ι] in
lemma muMin_le (μ : ι → ℚ) (i : ι) : muMin μ ≤ μ i :=
  inf'_le μ (mem_univ i)

/-! ### 2. The uniform-relaxation kernel and its exact rent -/

/-- `M = λ·I + (1−λ)·Π`: with probability `λ` stay put, otherwise resample from
    the stationary measure.  Every two-state chain carrying a stationary
    measure is of this form; at `N ≥ 3` it is a hypothesis. -/
noncomputable def uniformRelax (μ : ι → ℚ) (lam : ℚ) : ι → ι → ℚ :=
  fun i j => lam * (if i = j then 1 else 0) + (1 - lam) * μ j

omit [Nonempty ι] in
/-- The face is LEGAL, half one: `uniformRelax` is row-stochastic. -/
lemma uniformRelax_row {μ : ι → ℚ} {lam : ℚ} (hsum : ∑ i, μ i = 1) (i : ι) :
    ∑ j, uniformRelax μ lam i j = 1 := by
  simp only [uniformRelax, sum_add_distrib, ← mul_sum, hsum, sum_ite_eq, mem_univ,
    if_true, mul_one]
  ring

omit [Nonempty ι] in
/-- The face is LEGAL, half two: `μ` is `uniformRelax`-INVARIANT.  Without this
    the rent below would be computed on a face CROSS-FACE-1's §1 does not
    admit — its `P_ij` presumes a stationary view marginal. -/
lemma uniformRelax_invariant {μ : ι → ℚ} {lam : ℚ} (hsum : ∑ i, μ i = 1) (j : ι) :
    ∑ i, μ i * uniformRelax μ lam i j = μ j := by
  have hA : ∀ i : ι, μ i * uniformRelax μ lam i j
      = (if i = j then lam * μ j else 0) + μ i * ((1 - lam) * μ j) := by
    intro i
    simp only [uniformRelax]
    by_cases h : i = j
    · rw [if_pos h, if_pos h, h]; ring
    · rw [if_neg h, if_neg h]; ring
  rw [sum_congr rfl (fun i (_ : i ∈ univ) => hA i), sum_add_distrib,
      sum_ite_eq' univ j (fun _ => lam * μ j), if_pos (mem_univ j), ← sum_mul, hsum,
      one_mul]
  ring

/-- Each row's best successor, exactly: the diagonal, or the largest block. -/
lemma rowSup_uniformRelax {μ : ι → ℚ} {lam : ℚ}
    (hμ : ∀ i, 0 ≤ μ i) (h0 : 0 ≤ lam) (h1 : lam ≤ 1) (i : ι) :
    rowSup μ (uniformRelax μ lam) i
      = μ i * max (lam + (1 - lam) * μ i) ((1 - lam) * muMax μ) := by
  have hc : (0:ℚ) ≤ 1 - lam := by linarith
  rw [mul_max_of_nonneg _ _ _ (hμ i)]
  simp only [rowSup]
  refine le_antisymm (sup'_le _ _ ?_) (max_le ?_ ?_)
  · -- upper bound: every entry of row `i` is under one of the two candidates
    intro j _
    rcases eq_or_ne i j with hij | hij
    · refine le_max_of_le_left (le_of_eq ?_)
      subst hij; simp [uniformRelax]
    · refine le_max_of_le_right ?_
      have hEq : uniformRelax μ lam i j = (1 - lam) * μ j := by
        simp [uniformRelax, hij]
      rw [hEq]
      exact mul_le_mul_of_nonneg_left
        (mul_le_mul_of_nonneg_left (le_muMax μ j) hc) (hμ i)
  · -- attained at the diagonal
    have h := le_sup' (fun j => μ i * uniformRelax μ lam i j) (mem_univ i)
    simpa [uniformRelax] using h
  · -- attained at (or dominated by) the largest block
    obtain ⟨k, -, hk⟩ := exists_mem_eq_sup' (univ_nonempty (α := ι)) μ
    have hpos : (0:ℚ) ≤ lam * (if i = k then 1 else 0) := by
      rcases eq_or_ne i k with h | h
      · simp [h]; linarith
      · simp [h]
    have hstep : (1 - lam) * μ k ≤ uniformRelax μ lam i k := by
      simp only [uniformRelax]; linarith
    calc μ i * ((1 - lam) * muMax μ)
        = μ i * ((1 - lam) * μ k) := by rw [show muMax μ = μ k from hk]
      _ ≤ μ i * uniformRelax μ lam i k := mul_le_mul_of_nonneg_left hstep (hμ i)
      _ ≤ _ := le_sup' (fun j => μ i * uniformRelax μ lam i j) (mem_univ k)

/-- **GINI RENT, unconditional form.**  The exact rent of a uniform-relaxation
    view, for every block measure and every `λ ∈ [0,1]`. -/
theorem rent_uniformRelax {μ : ι → ℚ} {lam : ℚ}
    (hμ : ∀ i, 0 ≤ μ i) (h0 : 0 ≤ lam) (h1 : lam ≤ 1) :
    rent μ (uniformRelax μ lam)
      = 1 - ∑ i, μ i * max (lam + (1 - lam) * μ i) ((1 - lam) * muMax μ) := by
  unfold rent
  congr 1
  exact sum_congr rfl fun i _ => rowSup_uniformRelax hμ h0 h1 i

/-- **GINI RENT.**  Under the fence `(1−λ)(μ_max − μ_min) ≤ λ` — exactly the
    condition that the diagonal is every row's maximum — the rent is the
    Gini–Simpson index of the view times one minus the retention:

      `W(v) = (1 − Σ_i μ_i²) · (1 − λ)`.

    The fence is load-bearing, not decoration; `d4_classA_fence_fails` exhibits
    a case from the freeze's own measured table where it fails and the formula
    is wrong. -/
theorem rent_uniformRelax_gini {μ : ι → ℚ} {lam : ℚ}
    (hμ : ∀ i, 0 ≤ μ i) (hsum : ∑ i, μ i = 1) (h0 : 0 ≤ lam) (h1 : lam ≤ 1)
    (fence : (1 - lam) * (muMax μ - muMin μ) ≤ lam) :
    rent μ (uniformRelax μ lam) = (1 - ∑ i, μ i ^ 2) * (1 - lam) := by
  rw [rent_uniformRelax hμ h0 h1]
  have hmax : ∀ i : ι,
      max (lam + (1 - lam) * μ i) ((1 - lam) * muMax μ) = lam + (1 - lam) * μ i := by
    intro i
    refine max_eq_left ?_
    have hmono : (1 - lam) * (muMax μ - μ i) ≤ (1 - lam) * (muMax μ - muMin μ) :=
      mul_le_mul_of_nonneg_left (by have := muMin_le μ i; linarith) (by linarith)
    nlinarith
  simp only [hmax]
  have hexp : ∑ i, μ i * (lam + (1 - lam) * μ i)
      = lam * (∑ i, μ i) + (1 - lam) * ∑ i, μ i ^ 2 := by
    rw [mul_sum, mul_sum, ← sum_add_distrib]
    exact sum_congr rfl fun i _ => by ring
  rw [hexp, hsum]; ring

/-- **CROSS-FACE-1 Theorem 3, recovered.**  On uniform `μ` the fence is
    automatic and the Gini index is `1 − 1/N`, giving the freeze's headline
    form `rent = ceiling × (1 − retention)`. -/
theorem rent_uniform {lam : ℚ} (h0 : 0 ≤ lam) (h1 : lam ≤ 1) :
    rent (fun _ : ι => ((Fintype.card ι : ℚ))⁻¹)
         (uniformRelax (fun _ : ι => ((Fintype.card ι : ℚ))⁻¹) lam)
      = (1 - ((Fintype.card ι : ℚ))⁻¹) * (1 - lam) := by
  have hcard : (0:ℚ) < (Fintype.card ι : ℚ) := by
    exact_mod_cast Fintype.card_pos
  set c : ℚ := ((Fintype.card ι : ℚ))⁻¹ with hc
  have hcpos : 0 < c := by rw [hc]; positivity
  have hMax : muMax (fun _ : ι => c) = c := sup'_const _ c
  have hMin : muMin (fun _ : ι => c) = c := inf'_const _ c
  have hsum : ∑ _i : ι, c = 1 := by
    rw [sum_const, card_univ, nsmul_eq_mul, hc]
    field_simp
  have hsq : ∑ _i : ι, c ^ 2 = c := by
    rw [sum_const, card_univ, nsmul_eq_mul, hc]
    field_simp; ring
  rw [rent_uniformRelax_gini (fun _ => le_of_lt hcpos) hsum h0 h1
        (by rw [hMax, hMin]; simpa using h0), hsq]

/-- **The amnesia end (`λ = 0`).**  When the transfer forgets in one step the
    rent is `1 − μ_max`, which is the shape CROSS-FACE-1 Theorem 6 returns for
    the class view of a group torus. -/
theorem rent_amnesia {μ : ι → ℚ} (hμ : ∀ i, 0 ≤ μ i) (hsum : ∑ i, μ i = 1) :
    rent μ (uniformRelax μ 0) = 1 - muMax μ := by
  rw [rent_uniformRelax hμ le_rfl zero_le_one]
  have hmax : ∀ i : ι, max (0 + (1 - 0) * μ i) ((1 - 0) * muMax μ) = muMax μ := by
    intro i
    have := le_muMax μ i
    rw [max_eq_right (by linarith)]; ring
  simp only [hmax, ← sum_mul, hsum, one_mul]

/-! ### 3. The freeze's own measured datum, used twice

D4's `v_classA` view has five blocks with class sizes `(1,1,2,2,2)` out of
`|D4| = 8`, and `λ = 0` (Theorem 6: for fixed `a`, `b` uniform makes `ab`
uniform, so every row of the transfer is `μ`).  CROSS-FACE-1 measured its rent
as `3/4`.  Here it is a theorem — and the same datum refutes the naive
extension of Theorem 3 off the uniform line. -/

/-- D4 `v_classA`'s block measure: class sizes (1,1,2,2,2) over |D4| = 8. -/
def d4mu : Fin 5 → ℚ := ![1/8, 1/8, 1/4, 1/4, 1/4]

lemma d4mu_nonneg : ∀ i, 0 ≤ d4mu i := by
  intro i; fin_cases i <;> norm_num [d4mu]

lemma d4mu_sum : ∑ i, d4mu i = 1 := by
  simp [d4mu, Fin.sum_univ_five]; norm_num

lemma d4_muMax : muMax d4mu = 1/4 := by
  refine le_antisymm (sup'_le _ _ ?_) ?_
  · intro j _; fin_cases j <;> norm_num [d4mu]
  · have h := le_sup' d4mu (mem_univ (2 : Fin 5))
    simpa [muMax, d4mu] using h

lemma d4_muMin : muMin d4mu = 1/8 := by
  refine le_antisymm ?_ (le_inf' _ _ ?_)
  · have h := inf'_le d4mu (mem_univ (0 : Fin 5))
    simpa [muMin, d4mu] using h
  · intro j _; fin_cases j <;> norm_num [d4mu]

/-- **The measured value, reproduced.**  `W(v_classA) = 3/4`, from class sizes
    alone — CROSS-FACE-1's S2/§3(a) entry, now machine-checked. -/
theorem d4_classA_rent : rent d4mu (uniformRelax d4mu 0) = 3/4 := by
  rw [rent_amnesia d4mu_nonneg d4mu_sum, d4_muMax]; norm_num

/-- **And the same datum as a refuter, half one:** the Gini fence FAILS at D4's
    `v_classA`, so `rent_uniformRelax_gini` does not apply there. -/
theorem d4_classA_fence_fails : ¬ ((1 - 0) * (muMax d4mu - muMin d4mu) ≤ (0:ℚ)) := by
  rw [d4_muMax, d4_muMin]; norm_num

/-- **Half two:** and it had better not, because the Gini value is the WRONG
    number there — `25/32`, not the measured `3/4`.  Extending Theorem 3 off
    the uniform line by pattern match, without the fence, gives this. -/
theorem d4_gini_would_be_wrong :
    (1 - ∑ i, d4mu i ^ 2) * (1 - 0) ≠ rent d4mu (uniformRelax d4mu 0) := by
  rw [d4_classA_rent]
  simp [d4mu, Fin.sum_univ_five]
  norm_num

/-! ### 3b. The bridge's own headline number, machine-checked

`OMEGA_RATCHET1_NOTE.md` §3 derives, on the two-state rented registry, that the
partition rent peaks at `δ* = q* = √γ/(1+√γ)` with `W_max = 2γ/(1+√γ)²`.  At
`γ = 1/4` that is `δ* = q* = 1/3` and `W_max = 2/9`.  At that operating point
the registry's stationary occupancy is `G = 2/3` and its view mixing modulus is
`λ = (1−q)(1−γ) = 1/2` — and every two-state chain with a stationary measure IS
`λI + (1−λ)Π`, so the theorem above applies to it directly.  This ties the
mechanization to `verify.py`'s R2/R3 rows. -/

/-- The registry's stationary measure at `γ = 1/4`, `q = 1/3`: `(1/3, 2/3)`. -/
def regMu : Bool → ℚ := fun b => if b then 2/3 else 1/3

lemma regMu_nonneg : ∀ b, 0 ≤ regMu b := by intro b; cases b <;> norm_num [regMu]

lemma regMu_sum : ∑ b, regMu b = 1 := by simp [regMu, Fintype.sum_bool]; norm_num

lemma regMu_muMax : muMax regMu = 2/3 := by
  refine le_antisymm (sup'_le _ _ ?_) ?_
  · intro b _; cases b <;> norm_num [regMu]
  · simp [muMax, regMu]

lemma regMu_muMin : muMin regMu = 1/3 := by
  refine le_antisymm ?_ (le_inf' _ _ ?_)
  · simp [muMin, regMu]
  · intro b _; cases b <;> norm_num [regMu]

/-- **THE NOTE'S §3 MAXIMUM, PROVED.**  `W = 2/9` at the peak — which is also
    `2·δ*·W*(γ,δ*)` and `2·G(1−G)(1−λ)`, the bridge's two other forms. -/
theorem registry_rent_peak : rent regMu (uniformRelax regMu (1/2)) = 2/9 := by
  rw [rent_uniformRelax_gini regMu_nonneg regMu_sum (by norm_num) (by norm_num)
        (by rw [regMu_muMax, regMu_muMin]; norm_num)]
  simp [regMu, Fintype.sum_bool]
  norm_num

/-! ### 4. R-B, leg one: the staked dichotomy is FALSE

`OMEGA_RATCHET1_NOTE.md` §8 R-B staked: *for a policy class equivariant under a
group `Γ`, and a view `v` not `Γ`-invariant, no policy in the class holds `v`
closed at any budget.*  Attempting the proof produced a counterexample instead,
and it is recorded here rather than quietly dropped.

The state space is `Bool × Bool` — a view coordinate and a fiber coordinate —
with `Γ = Bool` acting on the fiber by `xor`.  The motion is two independent
symmetric binary channels, which IS `Γ`-equivariant.  The policy is
`restore s t = s`: undo the motion.  CROSS-FACE-1's own definition permits it —
a policy is a kernel `R(s''|s,s')` that *sees the pair*, "a repair that KNOWS
where the system came from" (§1 of the freeze).  It is `Γ`-equivariant, it
holds the FINEST view closed with `F = id`, and it costs `1 − (1−γ)(1−p)`,
which is finite.  Equivariance of the policy is not the obstruction. -/

/-- `Γ = Bool` acting on the fiber coordinate by `xor`. -/
def act (g : Bool) (s : Bool × Bool) : Bool × Bool := (s.1, xor g s.2)

/-- The motion of the §6 fiber model: independent symmetric binary channels on
    the view (rate `γ`) and the fiber (rate `p`).  Uniform `μ` is invariant. -/
def fiberMotion (gam p : ℚ) (s t : Bool × Bool) : ℚ :=
  (if t.1 = s.1 then 1 - gam else gam) * (if t.2 = s.2 then 1 - p else p)

/-- The policy that undoes the motion: legal, memory-carrying, fiber-blind. -/
def restore : Bool × Bool → Bool × Bool → Bool × Bool := fun s _ => s

/-- Displaced mass per step, `Pr[s'' ≠ s']` — CROSS-FACE-1's `W(R)`. -/
noncomputable def work (μ : (Bool × Bool) → ℚ) (K : (Bool × Bool) → (Bool × Bool) → ℚ)
    (R : (Bool × Bool) → (Bool × Bool) → Bool × Bool) : ℚ :=
  ∑ s, ∑ t, μ s * K s t * (if R s t = t then 0 else 1)

lemma fiberMotion_equivariant (gam p : ℚ) (g : Bool) (s t : Bool × Bool) :
    fiberMotion gam p (act g s) (act g t) = fiberMotion gam p s t := by
  obtain ⟨a, b⟩ := s; obtain ⟨c, d⟩ := t
  cases g <;> cases b <;> cases d <;> simp [act, fiberMotion]

/-- **THE COUNTEREXAMPLE, and therefore the fired kill.**  All four clauses of
    the staked dichotomy's hypothesis hold, and its conclusion fails. -/
theorem staked_R_B_dichotomy_is_false (gam p : ℚ) :
    -- the motion is Γ-equivariant
    (∀ g s t, fiberMotion gam p (act g s) (act g t) = fiberMotion gam p s t)
    -- the policy is Γ-equivariant
  ∧ (∀ g s t, restore (act g s) (act g t) = act g (restore s t))
    -- the view under test is the FINEST one, and it is NOT Γ-invariant
  ∧ (∃ g s, act g s ≠ s)
    -- yet the policy holds it closed, with `F = id`
  ∧ (∀ s t, restore s t = id s) := by
  refine ⟨fiberMotion_equivariant gam p, fun _ _ _ => rfl,
          ⟨true, (false, false), by decide⟩, fun _ _ => rfl⟩

/-- And the budget it does it on is finite, not infinite: exactly the
    probability that the motion moved anything. -/
theorem work_restore (gam p : ℚ) :
    work (fun _ => 1/4) (fiberMotion gam p) restore = 1 - (1 - gam) * (1 - p) := by
  simp [work, fiberMotion, restore, Fintype.sum_prod_type, Fintype.sum_bool]
  ring

/-! ### 5. R-B, leg two: what actually carries the fiber floor

The obstruction is not that the policy is equivariant; it is that the
MAINTAINED FIBER KERNEL it leaves behind is.  A `Γ`-equivariant kernel with
`Γ` transitive is doubly stochastic, so the uniform measure is invariant — it
cannot prefer any point of the fiber.  This is the mechanism behind the
maintained-holonomy campaign's measured `1/d` floor. -/

section Equivariant

variable {Φ : Type*} [Fintype Φ] [DecidableEq Φ] [Nonempty Φ]
variable {G : Type*} [Group G] [MulAction G Φ]

/-- Column sum of a kernel: the mass arriving at `y` from a uniform source. -/
def colSum (N : Φ → Φ → ℚ) (y : Φ) : ℚ := ∑ x, N x y

omit [DecidableEq Φ] [Nonempty Φ] in
/-- Equivariance makes the column sum a `Γ`-invariant function. -/
lemma colSum_equivariant (N : Φ → Φ → ℚ)
    (hN : ∀ (g : G) (x y : Φ), N (g • x) (g • y) = N x y) (g : G) (y : Φ) :
    colSum N (g • y) = colSum N y := by
  calc colSum N (g • y)
      = ∑ x, N ((MulAction.toPerm g) x) (g • y) :=
        (Equiv.sum_comp (MulAction.toPerm g) (fun z => N z (g • y))).symm
    _ = ∑ x, N x y := sum_congr rfl fun x _ => hN g x y

omit [DecidableEq Φ] in
/-- **A `Γ`-equivariant kernel with `Γ` transitive is DOUBLY STOCHASTIC.**  The
    columns cannot differ, and they must average to one. -/
theorem equivariant_kernel_colSum_eq_one (N : Φ → Φ → ℚ)
    (hrow : ∀ x, ∑ y, N x y = 1)
    (hN : ∀ (g : G) (x y : Φ), N (g • x) (g • y) = N x y)
    (htrans : ∀ x y : Φ, ∃ g : G, g • x = y) (y : Φ) :
    colSum N y = 1 := by
  obtain ⟨y0⟩ := ‹Nonempty Φ›
  have hconst : ∀ z : Φ, colSum N z = colSum N y0 := by
    intro z
    obtain ⟨g, hg⟩ := htrans y0 z
    rw [← hg]; exact colSum_equivariant N hN g y0
  have hcard : (0:ℚ) < (Fintype.card Φ : ℚ) := by exact_mod_cast Fintype.card_pos
  have htot : ∑ _z : Φ, colSum N y0 = (Fintype.card Φ : ℚ) := by
    rw [← sum_congr rfl (fun z (_ : z ∈ univ) => hconst z)]
    simp only [colSum]
    rw [sum_comm]
    simp [hrow]
  rw [sum_const, card_univ, nsmul_eq_mul] at htot
  have : (Fintype.card Φ : ℚ) * colSum N y0 = (Fintype.card Φ : ℚ) * 1 := by
    rw [mul_one]; exact htot
  rw [hconst y, mul_left_cancel₀ (ne_of_gt hcard) this]

omit [DecidableEq Φ] in
/-- **THE FIBER FLOOR'S MECHANISM.**  Under the same hypotheses the uniform
    measure is invariant: a repair that names no point of the fiber leaves a
    kernel that cannot prefer one. -/
theorem equivariant_uniform_invariant (N : Φ → Φ → ℚ)
    (hrow : ∀ x, ∑ y, N x y = 1)
    (hN : ∀ (g : G) (x y : Φ), N (g • x) (g • y) = N x y)
    (htrans : ∀ x y : Φ, ∃ g : G, g • x = y) (y : Φ) :
    ∑ x, ((Fintype.card Φ : ℚ))⁻¹ * N x y = ((Fintype.card Φ : ℚ))⁻¹ := by
  rw [← mul_sum]
  have : ∑ x, N x y = 1 := equivariant_kernel_colSum_eq_one N hrow hN htrans y
  rw [this, mul_one]

end Equivariant

/-! ### 6. R-B, leg three: the separator

`equivariant_uniform_invariant` would be vacuous if no repair could do better.
The design-KNOWING repair — deposit a NAMED point of the fiber with probability
`q` — is not equivariant, and its stationary measure sits strictly above the
floor for every `q > 0`.  This is `f*` of `OMEGA_RATCHET1_NOTE.md` §6, exactly. -/

/-- Deposit the named design point `false` with probability `q`, after a
    symmetric fiber flip at rate `p`. -/
def knowing (p q : ℚ) : Bool → Bool → ℚ := fun f g =>
  (if g = false then q else 0) + (1 - q) * (if g = f then 1 - p else p)

lemma knowing_row (p q : ℚ) (f : Bool) : ∑ g, knowing p q f g = 1 := by
  cases f <;> simp [knowing, Fintype.sum_bool] <;> ring

/-- It is NOT equivariant under the fiber's `Z₂`, for any nonzero dose. -/
theorem knowing_not_equivariant (p q : ℚ) (hq : q ≠ 0) :
    knowing p q (xor true false) (xor true false) ≠ knowing p q false false := by
  simp [knowing]
  intro h; exact hq (by linarith)

/-- The stationary fiber weight of the design-knowing repair. -/
noncomputable def fstar (p q : ℚ) : ℚ := (q + (1 - q) * p) / (q + 2 * p * (1 - q))

/-- **THE SEPARATOR.**  `(f*, 1 − f*)` is `knowing`-invariant, exactly. -/
theorem knowing_stationary (p q : ℚ) (hD : q + 2 * p * (1 - q) ≠ 0) (g : Bool) :
    fstar p q * knowing p q false g + (1 - fstar p q) * knowing p q true g
      = (if g = false then fstar p q else 1 - fstar p q) := by
  cases g <;> · simp only [knowing, fstar]; field_simp; ring

/-- And it sits strictly above the `1/|Φ|` floor exactly when the dose is
    positive: the design-knowing repair buys a level the blind one cannot. -/
theorem fstar_gt_half (p q : ℚ) (hp : 0 < p) (hq : 0 < q) (hq1 : q ≤ 1) :
    1/2 < fstar p q := by
  have hD : 0 < q + 2 * p * (1 - q) := by nlinarith
  rw [fstar, lt_div_iff₀ hD]
  nlinarith

/-! ### 7. R-B, leg four: the staked forward number

`OMEGA_RATCHET1_NOTE.md` §8 R-B staked `W(v_full) − W(v_view) = p(1−γ)` on the
§6 model, "for any `(γ, p)` a reader picks".  Here it is, over `ℚ`. -/

/-- The view coordinate's own kernel: a symmetric binary channel at rate `γ`. -/
def viewK (gam : ℚ) : Bool → Bool → ℚ := fun a b => if b = a then 1 - gam else gam

/-- Both faces of §7 are LEGAL: row-stochastic, with the uniform measure
    invariant (both channels are symmetric, hence doubly stochastic). -/
lemma viewK_row (gam : ℚ) (a : Bool) : ∑ b, viewK gam a b = 1 := by
  cases a <;> · simp [viewK, Fintype.sum_bool]

lemma viewK_invariant (gam : ℚ) (b : Bool) :
    ∑ a, (1/2 : ℚ) * viewK gam a b = 1/2 := by
  cases b <;> · simp [viewK, Fintype.sum_bool]; ring

lemma fiberMotion_row (gam p : ℚ) (s : Bool × Bool) :
    ∑ t, fiberMotion gam p s t = 1 := by
  obtain ⟨a, b⟩ := s
  cases a <;> cases b <;>
    simp [fiberMotion, Fintype.sum_prod_type, Fintype.sum_bool] <;> ring

lemma fiberMotion_invariant (gam p : ℚ) (t : Bool × Bool) :
    ∑ s, (1/4 : ℚ) * fiberMotion gam p s t = 1/4 := by
  obtain ⟨c, d⟩ := t
  cases c <;> cases d <;>
    simp [fiberMotion, Fintype.sum_prod_type, Fintype.sum_bool] <;> ring

lemma rowSup_viewK {gam : ℚ} (_h0 : 0 ≤ gam) (h1 : gam ≤ 1/2) (a : Bool) :
    rowSup (fun _ => (1/2 : ℚ)) (viewK gam) a = 1/2 * (1 - gam) := by
  simp only [rowSup]
  refine le_antisymm (sup'_le _ _ ?_) ?_
  · intro b _; cases a <;> cases b <;> simp [viewK] <;> linarith
  · have h := le_sup' (fun b => (1/2 : ℚ) * viewK gam a b) (mem_univ a)
    simpa [viewK] using h

lemma rowSup_fiberMotion {gam p : ℚ} (_hg0 : 0 ≤ gam) (hg1 : gam ≤ 1/2)
    (_hp0 : 0 ≤ p) (hp1 : p ≤ 1/2) (s : Bool × Bool) :
    rowSup (fun _ => (1/4 : ℚ)) (fiberMotion gam p) s = 1/4 * ((1 - gam) * (1 - p)) := by
  simp only [rowSup]
  refine le_antisymm (sup'_le _ _ ?_) ?_
  · rintro ⟨c, d⟩ -
    obtain ⟨a, b⟩ := s
    cases a <;> cases b <;> cases c <;> cases d <;>
      simp [fiberMotion] <;> nlinarith
  · have h := le_sup' (fun t => (1/4 : ℚ) * fiberMotion gam p s t) (mem_univ s)
    simpa [fiberMotion] using h

/-- The full (view × fiber) view's rent. -/
theorem rent_fiberMotion {gam p : ℚ} (hg0 : 0 ≤ gam) (hg1 : gam ≤ 1/2)
    (hp0 : 0 ≤ p) (hp1 : p ≤ 1/2) :
    rent (fun _ => (1/4 : ℚ)) (fiberMotion gam p) = gam + p * (1 - gam) := by
  simp only [rent]
  rw [sum_congr rfl (fun s (_ : s ∈ univ) => rowSup_fiberMotion hg0 hg1 hp0 hp1 s)]
  simp [Fintype.sum_prod_type, Fintype.sum_bool]
  ring

/-- The view coordinate alone: CROSS-FACE-1 Theorem 3 at `N = 2`, `λ = 1−2γ`. -/
theorem rent_viewK {gam : ℚ} (h0 : 0 ≤ gam) (h1 : gam ≤ 1/2) :
    rent (fun _ => (1/2 : ℚ)) (viewK gam) = gam := by
  simp only [rent]
  rw [sum_congr rfl (fun a (_ : a ∈ univ) => rowSup_viewK h0 h1 a)]
  simp [Fintype.sum_bool]

/-- **THE STAKED FORWARD NUMBER, PROVED.**  The surcharge for holding the
    design — the extra partition rent of the finer view — is exactly the
    fiber's own rent.  Design-knowing and design-blind repair were matched in
    the magnitude currency (the dose `q`) and were never matched in this one. -/
theorem fiber_surcharge {gam p : ℚ} (hg0 : 0 ≤ gam) (hg1 : gam ≤ 1/2)
    (hp0 : 0 ≤ p) (hp1 : p ≤ 1/2) :
    rent (fun _ => (1/4 : ℚ)) (fiberMotion gam p)
      - rent (fun _ => (1/2 : ℚ)) (viewK gam) = p * (1 - gam) := by
  rw [rent_fiberMotion hg0 hg1 hp0 hp1, rent_viewK hg0 hg1]; ring

end CIRISHolon.GiniRent
