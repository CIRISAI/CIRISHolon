/-
CIRISHolon.Mixing — why stochastic tiers self-heal and deterministic ones never do.

Transplanted whole from CIRISAI/CIRISOntology (Core/Mixing.lean). Engineering
reading: the closure DEFECT of a view — how far the step keeps two merged
readings apart, in total variation — contracts at the Dobrushin rate of the
kernel (defect_le_alpha_pow), so a noisy tier forgets a certification miss
geometrically; a DETERMINISTIC step's defect is {0,1}-valued and never
contracts (det_defect_zero_or_one), so replay/netcode error on a
deterministic tier is forever. Zero defect for a deterministic step IS the
Closed predicate of CIRISHolon.Object, given a number
(det_defect_eq_zero_iff_closed). Comment references to Core/Habit.lean and
campaign files refer to the upstream repository.
-/
import CIRISHolon.Object
import Mathlib.Tactic

namespace CIRISHolon.Mixing

open Finset

/-! ## 0. Distributions and total variation on a finite set -/

section TV

variable {α : Type*} [Fintype α]

/-- A probability distribution on a finite set: nonnegative, total mass one. -/
structure IsDist (μ : α → ℝ) : Prop where
  nonneg : ∀ a, 0 ≤ μ a
  sum_one : ∑ a, μ a = 1

/-- **TOTAL VARIATION DISTANCE**, in the half-L¹ normalisation: `0` for equal
    distributions, `1` for distributions with disjoint support. -/
noncomputable def tv (μ ν : α → ℝ) : ℝ := (∑ a, |μ a - ν a|) / 2

theorem tv_nonneg (μ ν : α → ℝ) : 0 ≤ tv μ ν := by
  have h : 0 ≤ ∑ a, |μ a - ν a| := Finset.sum_nonneg fun a _ => abs_nonneg _
  unfold tv; linarith

theorem tv_self (μ : α → ℝ) : tv μ μ = 0 := by simp [tv]

theorem tv_comm (μ ν : α → ℝ) : tv μ ν = tv ν μ := by
  simp only [tv]
  congr 1
  exact Finset.sum_congr rfl fun a _ => abs_sub_comm _ _

/-- Two distributions are at TV distance at most one. -/
theorem tv_le_one {μ ν : α → ℝ} (hμ : IsDist μ) (hν : IsDist ν) : tv μ ν ≤ 1 := by
  have hpt : ∀ a ∈ (univ : Finset α), |μ a - ν a| ≤ μ a + ν a := by
    intro a _
    have h1 := hμ.nonneg a
    have h2 := hν.nonneg a
    rcases abs_cases (μ a - ν a) with ⟨he, _⟩ | ⟨he, _⟩ <;> rw [he] <;> linarith
  have hsum := Finset.sum_le_sum hpt
  rw [Finset.sum_add_distrib, hμ.sum_one, hν.sum_one] at hsum
  simp only [tv]
  linarith

/-! ### The positive/negative decomposition of a zero-sum difference -/

theorem max_sub_max_neg (a : ℝ) : max a 0 - max (-a) 0 = a := by
  rcases le_total 0 a with h | h
  · rw [max_eq_left h, max_eq_right (by linarith)]; ring
  · rw [max_eq_right h, max_eq_left (by linarith)]; ring

theorem max_add_max_neg (a : ℝ) : max a 0 + max (-a) 0 = |a| := by
  rcases le_total 0 a with h | h
  · rw [max_eq_left h, max_eq_right (by linarith), abs_of_nonneg h]; ring
  · rw [max_eq_right h, max_eq_left (by linarith), abs_of_nonpos h]; ring

/-- For a zero-sum vector the positive and negative parts carry equal mass, and
    that mass is half the L¹ norm. This is the arithmetic behind
    "TV = the mass that must move". -/
theorem sum_max_eq_half {d : α → ℝ} (h : ∑ a, d a = 0) :
    ∑ a, max (d a) 0 = (∑ a, |d a|) / 2 := by
  have hsplit : ∑ a, max (d a) 0 - ∑ a, max (-(d a)) 0 = 0 := by
    rw [← Finset.sum_sub_distrib]
    rw [Finset.sum_congr rfl fun a _ => max_sub_max_neg (d a)]
    exact h
  have habs : ∑ a, max (d a) 0 + ∑ a, max (-(d a)) 0 = ∑ a, |d a| := by
    rw [← Finset.sum_add_distrib]
    exact Finset.sum_congr rfl fun a _ => max_add_max_neg (d a)
  linarith

/-- TV as the mass of the positive part — the form the Doeblin bound needs. -/
theorem tv_eq_sum_max {μ ν : α → ℝ} (hμ : IsDist μ) (hν : IsDist ν) :
    tv μ ν = ∑ a, max (μ a - ν a) 0 := by
  have h0 : ∑ a, (μ a - ν a) = 0 := by
    rw [Finset.sum_sub_distrib, hμ.sum_one, hν.sum_one]; ring
  rw [sum_max_eq_half h0]
  rfl

/-! ### Point masses -/

variable [DecidableEq α]

/-- The point mass at `a`. -/
noncomputable def dirac (a : α) : α → ℝ := fun b => if b = a then 1 else 0

theorem isDist_dirac (a : α) : IsDist (dirac a) := by
  refine ⟨fun b => ?_, ?_⟩
  · unfold dirac; split <;> norm_num
  · simp [dirac]

/-- Two point masses are at distance `0` if equal and `1` if not. -/
theorem tv_dirac (a b : α) : tv (dirac a) (dirac b) = if a = b then 0 else 1 := by
  by_cases hab : a = b
  · subst hab; simp [tv_self]
  · have hba : ¬ b = a := fun h => hab h.symm
    have hpt : ∀ c ∈ (univ : Finset α),
        |dirac a c - dirac b c| = (if c = a then (1:ℝ) else 0) + (if c = b then 1 else 0) := by
      intro c _
      unfold dirac
      by_cases hca : c = a
      · have hcb : ¬ c = b := by rw [hca]; exact hab
        simp [hca, hcb, hab, hba]
      · by_cases hcb : c = b <;> simp [hca, hcb, hab, hba]
    simp only [tv]
    rw [Finset.sum_congr rfl hpt, Finset.sum_add_distrib]
    simp [hab]

end TV

/-! ## 1. Kernels, their action, and the Dobrushin coefficient -/

section Kernel

variable {X : Type*} [Fintype X]

/-- A row-stochastic kernel on a finite state space. -/
structure IsStoch (T : X → X → ℝ) : Prop where
  nonneg : ∀ x y, 0 ≤ T x y
  row_sum : ∀ x, ∑ y, T x y = 1

/-- One step of the chain applied to a distribution. -/
noncomputable def app (μ : X → ℝ) (T : X → X → ℝ) : X → ℝ := fun y => ∑ x, μ x * T x y

/-- `m` steps of the chain applied to a distribution. -/
noncomputable def iter (T : X → X → ℝ) (μ : X → ℝ) : ℕ → (X → ℝ)
  | 0 => μ
  | m + 1 => app (iter T μ m) T

@[simp] theorem iter_zero (T : X → X → ℝ) (μ : X → ℝ) : iter T μ 0 = μ := rfl

@[simp] theorem iter_succ (T : X → X → ℝ) (μ : X → ℝ) (m : ℕ) :
    iter T μ (m + 1) = app (iter T μ m) T := rfl

theorem isDist_app {T : X → X → ℝ} (hT : IsStoch T) {μ : X → ℝ} (hμ : IsDist μ) :
    IsDist (app μ T) := by
  refine ⟨fun y => Finset.sum_nonneg fun x _ => mul_nonneg (hμ.nonneg x) (hT.nonneg x y), ?_⟩
  unfold app
  rw [Finset.sum_comm]
  rw [Finset.sum_congr rfl fun x _ => by rw [← Finset.mul_sum, hT.row_sum x, mul_one]]
  exact hμ.sum_one

theorem isDist_iter {T : X → X → ℝ} (hT : IsStoch T) {μ : X → ℝ} (hμ : IsDist μ) :
    ∀ m, IsDist (iter T μ m)
  | 0 => hμ
  | m + 1 => isDist_app hT (isDist_iter hT hμ m)

variable [Nonempty X]

/-- **THE DOBRUSHIN ERGODICITY COEFFICIENT** (Dobrushin 1956): the largest total
    variation distance between two rows of the kernel. `α(T) = 0` means the
    chain forgets its start in one step; `α(T) = 1` means some pair of starting
    states is still perfectly distinguishable after one step. -/
noncomputable def alpha (T : X → X → ℝ) : ℝ :=
  (univ : Finset (X × X)).sup' Finset.univ_nonempty (fun p => tv (T p.1) (T p.2))

theorem tv_row_le_alpha (T : X → X → ℝ) (x y : X) : tv (T x) (T y) ≤ alpha T :=
  Finset.le_sup' (fun p : X × X => tv (T p.1) (T p.2)) (Finset.mem_univ (x, y))

theorem alpha_nonneg (T : X → X → ℝ) : 0 ≤ alpha T := by
  obtain ⟨x⟩ := ‹Nonempty X›
  have h := tv_row_le_alpha T x x
  rwa [tv_self] at h

theorem alpha_le_one {T : X → X → ℝ} (hT : IsStoch T) : alpha T ≤ 1 :=
  Finset.sup'_le _ _ fun p _ => tv_le_one ⟨fun y => hT.nonneg p.1 y, hT.row_sum p.1⟩
    ⟨fun y => hT.nonneg p.2 y, hT.row_sum p.2⟩

/-- **DOBRUSHIN CONTRACTION, SIGNED FORM** (Dobrushin 1956). On the zero-sum
    subspace — the differences of distributions — the kernel shrinks the L¹ norm
    by the ergodicity coefficient. Mathlib v4.14 has neither the coefficient nor
    this inequality (its `SignedMeasure.totalVariation` is the Jordan
    decomposition, a different object), so it is proved here from scratch, by
    the textbook coupling identity.

    Note, stated because the hypothesis a reader expects is absent: the
    INEQUALITY needs no row-stochasticity. It is linear algebra about `α`. What
    stochasticity buys is `alpha_le_one`, i.e. that the bound says anything. -/
theorem sum_abs_app_le {T : X → X → ℝ} {d : X → ℝ} (hd : ∑ x, d x = 0) :
    ∑ z, |∑ x, d x * T x z| ≤ alpha T * ∑ x, |d x| := by
  classical
  obtain ⟨dp, dm, s, hdp0, hdm0, hdpm, hSp, hSm, hs2⟩ :
      ∃ (dp dm : X → ℝ) (s : ℝ), (∀ x, 0 ≤ dp x) ∧ (∀ x, 0 ≤ dm x) ∧
        (∀ x, dp x - dm x = d x) ∧ (∑ x, dp x = s) ∧ (∑ x, dm x = s) ∧
        (∑ x, |d x| = 2 * s) := by
    refine ⟨fun x => max (d x) 0, fun x => max (-(d x)) 0, (∑ x, |d x|) / 2,
      fun x => le_max_right _ _, fun x => le_max_right _ _,
      fun x => max_sub_max_neg (d x), ?_, ?_, by ring⟩
    · simpa using sum_max_eq_half hd
    · have hneg : ∑ x, (-(d x)) = 0 := by rw [Finset.sum_neg_distrib, hd]; ring
      have h := sum_max_eq_half hneg
      have habs : ∑ x, |(-(d x))| = ∑ x, |d x| :=
        Finset.sum_congr rfl fun x _ => abs_neg _
      rw [habs] at h
      simpa using h
  have hs0 : 0 ≤ s := by
    rw [← hSp]; exact Finset.sum_nonneg fun x _ => hdp0 x
  rw [hs2]
  rcases eq_or_lt_of_le hs0 with hzero | hpos
  · -- degenerate: the difference vanishes identically
    have hdpz : ∀ x, dp x = 0 := by
      intro x
      have h : ∑ x, dp x = 0 := by rw [hSp, ← hzero]
      exact (Finset.sum_eq_zero_iff_of_nonneg fun x _ => hdp0 x).mp h x (Finset.mem_univ x)
    have hdmz : ∀ x, dm x = 0 := by
      intro x
      have h : ∑ x, dm x = 0 := by rw [hSm, ← hzero]
      exact (Finset.sum_eq_zero_iff_of_nonneg fun x _ => hdm0 x).mp h x (Finset.mem_univ x)
    have hdz : ∀ x, d x = 0 := by
      intro x; rw [← hdpm x, hdpz x, hdmz x]; ring
    have hL : ∑ z, |∑ x, d x * T x z| = 0 := by
      refine Finset.sum_eq_zero fun z _ => ?_
      have hz : ∑ x, d x * T x z = 0 :=
        Finset.sum_eq_zero fun x _ => by rw [hdz x]; ring
      rw [hz, abs_zero]
    rw [hL, ← hzero]
    simp
  · -- the coupling identity
    have key : ∀ z, s * (∑ x, d x * T x z)
        = ∑ x, ∑ y, dp x * dm y * (T x z - T y z) := by
      intro z
      have hinner : ∀ x, ∑ y, dp x * dm y * (T x z - T y z)
          = (dp x * T x z) * (∑ y, dm y) - dp x * (∑ y, dm y * T y z) := by
        intro x
        have hstep : ∀ y, dp x * dm y * (T x z - T y z)
            = (dp x * T x z) * dm y - dp x * (dm y * T y z) := fun y => by ring
        rw [Finset.sum_congr rfl fun y _ => hstep y, Finset.sum_sub_distrib,
          ← Finset.mul_sum, ← Finset.mul_sum]
      rw [Finset.sum_congr rfl fun x _ => hinner x, Finset.sum_sub_distrib, hSm]
      have h1 : ∑ x, (dp x * T x z) * s = (∑ x, dp x * T x z) * s := by
        rw [← Finset.sum_mul]
      have h2 : ∑ x, dp x * (∑ y, dm y * T y z) = s * (∑ y, dm y * T y z) := by
        rw [← Finset.sum_mul, hSp]
      have hsplit : ∑ x, d x * T x z
          = (∑ x, dp x * T x z) - (∑ x, dm x * T x z) := by
        rw [← Finset.sum_sub_distrib]
        exact Finset.sum_congr rfl fun x _ => by rw [← hdpm x]; ring
      rw [h1, h2, hsplit]
      ring
    -- pointwise absolute bound
    have keyabs : ∀ z, s * |∑ x, d x * T x z|
        ≤ ∑ x, ∑ y, dp x * dm y * |T x z - T y z| := by
      intro z
      have h1 : s * |∑ x, d x * T x z| = |s * (∑ x, d x * T x z)| := by
        rw [abs_mul, abs_of_nonneg hs0]
      rw [h1, key z]
      calc |∑ x, ∑ y, dp x * dm y * (T x z - T y z)|
          ≤ ∑ x, |∑ y, dp x * dm y * (T x z - T y z)| :=
            Finset.abs_sum_le_sum_abs _ _
        _ ≤ ∑ x, ∑ y, dp x * dm y * |T x z - T y z| := by
            refine Finset.sum_le_sum fun x _ => ?_
            refine le_trans (Finset.abs_sum_le_sum_abs _ _) ?_
            refine Finset.sum_le_sum fun y _ => ?_
            rw [abs_mul, abs_of_nonneg (mul_nonneg (hdp0 x) (hdm0 y))]
    -- sum over the target state, then bound each row pair by α
    have hsum : s * (∑ z, |∑ x, d x * T x z|) ≤ 2 * alpha T * s * s := by
      have hstep1 : s * (∑ z, |∑ x, d x * T x z|)
          = ∑ z, s * |∑ x, d x * T x z| := by rw [Finset.mul_sum]
      have hstep2 : ∑ z, s * |∑ x, d x * T x z|
          ≤ ∑ z, ∑ x, ∑ y, dp x * dm y * |T x z - T y z| :=
        Finset.sum_le_sum fun z _ => keyabs z
      have hswap : ∑ z, ∑ x, ∑ y, dp x * dm y * |T x z - T y z|
          = ∑ x, ∑ y, dp x * dm y * (2 * tv (T x) (T y)) := by
        rw [Finset.sum_comm]
        refine Finset.sum_congr rfl fun x _ => ?_
        rw [Finset.sum_comm]
        refine Finset.sum_congr rfl fun y _ => ?_
        rw [← Finset.mul_sum]
        congr 1
        simp only [tv]
        ring
      have hbound : ∑ x, ∑ y, dp x * dm y * (2 * tv (T x) (T y))
          ≤ ∑ x, ∑ y, dp x * dm y * (2 * alpha T) := by
        refine Finset.sum_le_sum fun x _ => Finset.sum_le_sum fun y _ => ?_
        refine mul_le_mul_of_nonneg_left ?_ (mul_nonneg (hdp0 x) (hdm0 y))
        have h := tv_row_le_alpha T x y
        linarith
      have hrhs : ∑ x, ∑ y, dp x * dm y * (2 * alpha T) = 2 * alpha T * s * s := by
        have h1 : ∀ x, ∑ y, dp x * dm y * (2 * alpha T)
            = (dp x * (2 * alpha T)) * s := by
          intro x
          have hstep : ∀ y, dp x * dm y * (2 * alpha T)
              = (dp x * (2 * alpha T)) * dm y := fun y => by ring
          rw [Finset.sum_congr rfl fun y _ => hstep y, ← Finset.mul_sum, hSm]
        rw [Finset.sum_congr rfl fun x _ => h1 x, ← Finset.sum_mul, ← Finset.sum_mul, hSp]
        ring
      calc s * (∑ z, |∑ x, d x * T x z|)
          = ∑ z, s * |∑ x, d x * T x z| := hstep1
        _ ≤ ∑ z, ∑ x, ∑ y, dp x * dm y * |T x z - T y z| := hstep2
        _ = ∑ x, ∑ y, dp x * dm y * (2 * tv (T x) (T y)) := hswap
        _ ≤ ∑ x, ∑ y, dp x * dm y * (2 * alpha T) := hbound
        _ = 2 * alpha T * s * s := hrhs
    nlinarith [hsum, hpos]

/-- **DOBRUSHIN CONTRACTION** (Dobrushin 1956). One step of the chain shrinks
    total variation distance by the ergodicity coefficient. -/
theorem tv_app_le (T : X → X → ℝ) {μ ν : X → ℝ}
    (hμ : IsDist μ) (hν : IsDist ν) :
    tv (app μ T) (app ν T) ≤ alpha T * tv μ ν := by
  have hd : ∑ x, (μ x - ν x) = 0 := by
    rw [Finset.sum_sub_distrib, hμ.sum_one, hν.sum_one]; ring
  have hmain := sum_abs_app_le (T := T) hd
  have hpt : ∀ z, app μ T z - app ν T z = ∑ x, (μ x - ν x) * T x z := by
    intro z
    unfold app
    rw [← Finset.sum_sub_distrib]
    exact Finset.sum_congr rfl fun x _ => by ring
  have hL : ∑ z, |app μ T z - app ν T z| = ∑ z, |∑ x, (μ x - ν x) * T x z| :=
    Finset.sum_congr rfl fun z _ => by rw [hpt z]
  simp only [tv]
  rw [hL]
  linarith

/-- The contraction, iterated: after `m` steps the TV distance has been shrunk by
    `α(T)^m`. -/
theorem tv_iter_le {T : X → X → ℝ} (hT : IsStoch T) {μ ν : X → ℝ}
    (hμ : IsDist μ) (hν : IsDist ν) :
    ∀ m, tv (iter T μ m) (iter T ν m) ≤ alpha T ^ m * tv μ ν
  | 0 => by simp
  | m + 1 => by
      have hstep := tv_app_le T (isDist_iter hT hμ m) (isDist_iter hT hν m)
      have hind := tv_iter_le hT hμ hν m
      have hmul : alpha T * tv (iter T μ m) (iter T ν m)
          ≤ alpha T * (alpha T ^ m * tv μ ν) :=
        mul_le_mul_of_nonneg_left hind (alpha_nonneg T)
      calc tv (iter T μ (m + 1)) (iter T ν (m + 1))
          = tv (app (iter T μ m) T) (app (iter T ν m) T) := rfl
        _ ≤ alpha T * tv (iter T μ m) (iter T ν m) := hstep
        _ ≤ alpha T * (alpha T ^ m * tv μ ν) := hmul
        _ = alpha T ^ (m + 1) * tv μ ν := by ring

/-! ### The Doeblin minorization: how much noise buys how much contraction -/

/-- **DOEBLIN (1938), uniform form.** If every transition probability is at least
    `e`, the ergodicity coefficient is at most `1 − e·|X|`. -/
theorem alpha_le_one_sub_card_mul {T : X → X → ℝ} (hT : IsStoch T) {e : ℝ}
    (he : ∀ x y, e ≤ T x y) :
    alpha T ≤ 1 - e * (Fintype.card X) := by
  refine Finset.sup'_le _ _ fun p _ => ?_
  have hd1 : IsDist (T p.1) := ⟨fun y => hT.nonneg p.1 y, hT.row_sum p.1⟩
  have hd2 : IsDist (T p.2) := ⟨fun y => hT.nonneg p.2 y, hT.row_sum p.2⟩
  rw [tv_eq_sum_max hd1 hd2]
  have hpt : ∀ z ∈ (univ : Finset X), max (T p.1 z - T p.2 z) 0 ≤ T p.1 z - e := by
    intro z _
    have h1 := he p.1 z
    have h2 := he p.2 z
    rcases max_cases (T p.1 z - T p.2 z) 0 with ⟨he', _⟩ | ⟨he', _⟩ <;> rw [he'] <;> linarith
  refine le_trans (Finset.sum_le_sum hpt) ?_
  rw [Finset.sum_sub_distrib, hT.row_sum p.1]
  simp [Finset.sum_const, mul_comm]

end Kernel

/-! ## 2. Views, pushforward, and the closure defect -/

section Push

variable {X : Type*} [Fintype X]
variable {C : Type*} [Fintype C] [DecidableEq C]

/-- The view's reading of a distribution: the pushforward along `v`. -/
noncomputable def push (v : X → C) (μ : X → ℝ) : C → ℝ :=
  fun c => ∑ x, if v x = c then μ x else 0

theorem isDist_push (v : X → C) {μ : X → ℝ} (hμ : IsDist μ) : IsDist (push v μ) := by
  constructor
  · intro c
    refine Finset.sum_nonneg fun x _ => ?_
    split
    · exact hμ.nonneg x
    · exact le_refl 0
  · show ∑ c, push v μ c = 1
    unfold push
    rw [Finset.sum_comm]
    have hfib : ∀ x : X, ∑ c : C, (if v x = c then μ x else 0) = μ x := by
      intro x; simp
    rw [Finset.sum_congr rfl fun x _ => hfib x]
    exact hμ.sum_one

/-- **COARSE-GRAINING IS NON-EXPANSIVE.** Reading through a view can only bring
    two distributions closer in total variation — the finite, TV-flavoured form
    of the data-processing inequality. -/
theorem tv_push_le (v : X → C) (μ ν : X → ℝ) :
    tv (push v μ) (push v ν) ≤ tv μ ν := by
  have hpt : ∀ c ∈ (univ : Finset C),
      |push v μ c - push v ν c| ≤ ∑ x, if v x = c then |μ x - ν x| else 0 := by
    intro c _
    have hsub : push v μ c - push v ν c
        = ∑ x, (if v x = c then μ x - ν x else 0) := by
      unfold push
      rw [← Finset.sum_sub_distrib]
      refine Finset.sum_congr rfl fun x _ => ?_
      by_cases h : v x = c <;> simp [h]
    rw [hsub]
    refine le_trans (Finset.abs_sum_le_sum_abs _ _) ?_
    refine Finset.sum_le_sum fun x _ => ?_
    by_cases h : v x = c <;> simp [h]
  have hstep : ∑ c, |push v μ c - push v ν c|
      ≤ ∑ c, ∑ x, if v x = c then |μ x - ν x| else 0 := Finset.sum_le_sum hpt
  have hfib : ∑ c, ∑ x, (if v x = c then |μ x - ν x| else 0) = ∑ x, |μ x - ν x| := by
    rw [Finset.sum_comm]
    refine Finset.sum_congr rfl fun x _ => ?_
    simp
  rw [hfib] at hstep
  simp only [tv]
  linarith

omit [Fintype C] in
/-- A point mass reads as a point mass. -/
theorem push_dirac [DecidableEq X] (v : X → C) (x : X) : push v (dirac x) = dirac (v x) := by
  funext c
  unfold push dirac
  by_cases h : v x = c
  · subst h
    rw [Finset.sum_eq_single x]
    · simp
    · intro b _ hbx; simp [hbx]
    · intro hx; exact absurd (Finset.mem_univ x) hx
  · rw [Finset.sum_eq_zero]
    · simp [Ne.symm h, h]
    · intro b _
      by_cases hb : b = x
      · subst hb; simp [h]
      · simp [hb]

end Push

section Defect

variable {X : Type*} [Fintype X] [DecidableEq X]
variable {C : Type*} [Fintype C] [DecidableEq C]

/-- The pairs of states the view cannot tell apart: the union of `v`'s fibers,
    read as a relation. -/
def fiberPairs (v : X → C) : Finset (X × X) :=
  univ.filter (fun p : X × X => v p.1 = v p.2)

omit [DecidableEq X] [Fintype C] in
theorem mem_fiberPairs {v : X → C} {x y : X} (h : v x = v y) :
    (x, y) ∈ fiberPairs v := by
  simp [fiberPairs, h]

omit [DecidableEq X] [Fintype C] in
theorem fiberPairs_nonempty [Nonempty X] (v : X → C) : (fiberPairs v).Nonempty := by
  obtain ⟨x⟩ := ‹Nonempty X›
  exact ⟨(x, x), by simp [fiberPairs]⟩

variable [Nonempty X]

/-- **THE CLOSURE DEFECT AT LAG `m`**, in total variation: the largest distance
    between the view's `m`-step futures, over pairs of states the view cannot
    tell apart. It answers "does within-fiber position still matter for what the
    view will see in `m` steps?" — `0` exactly when it does not. -/
noncomputable def defect (T : X → X → ℝ) (v : X → C) (m : ℕ) : ℝ :=
  (fiberPairs v).sup' (fiberPairs_nonempty v)
    (fun p => tv (push v (iter T (dirac p.1) m)) (push v (iter T (dirac p.2) m)))

/-- The defect dominates the reading of any single fiber pair. -/
theorem le_defect {T : X → X → ℝ} {v : X → C} {m : ℕ} {x y : X} (hxy : v x = v y) :
    tv (push v (iter T (dirac x) m)) (push v (iter T (dirac y) m)) ≤ defect T v m :=
  Finset.le_sup'
    (fun p : X × X => tv (push v (iter T (dirac p.1) m)) (push v (iter T (dirac p.2) m)))
    (mem_fiberPairs hxy)

theorem defect_nonneg (T : X → X → ℝ) (v : X → C) (m : ℕ) : 0 ≤ defect T v m := by
  obtain ⟨x⟩ := ‹Nonempty X›
  have h := le_defect (T := T) (m := m) (rfl : v x = v x)
  rwa [tv_self] at h

theorem defect_zero_lag (T : X → X → ℝ) (v : X → C) : defect T v 0 = 0 := by
  refine le_antisymm (Finset.sup'_le _ _ fun p hp => ?_) (defect_nonneg T v 0)
  simp only [fiberPairs, Finset.mem_filter] at hp
  rw [iter_zero, iter_zero, push_dirac, push_dirac, hp.2, tv_self]

/-- **THE KEYSTONE — THE MIXING THEOREM.** For any finite dynamics given by a
    kernel `T`, the closure defect at lag `m` is bounded by the `m`-th power of
    the chain's Dobrushin ergodicity coefficient. Whenever `α(T) < 1` the defect
    is driven to zero geometrically, at the CHAIN's rate — the view contributes
    nothing to the rate. Row-stochasticity is what makes `α(T) ≤ 1`
    (`alpha_le_one`) and so what makes the bound informative; it is assumed here
    because the iterated contraction is stated for distributions. -/
theorem defect_le_alpha_pow {T : X → X → ℝ} (hT : IsStoch T) (v : X → C) (m : ℕ) :
    defect T v m ≤ alpha T ^ m := by
  refine Finset.sup'_le _ _ fun p _ => ?_
  have h1 : tv (push v (iter T (dirac p.1) m)) (push v (iter T (dirac p.2) m))
      ≤ tv (iter T (dirac p.1) m) (iter T (dirac p.2) m) := tv_push_le _ _ _
  have h2 : tv (iter T (dirac p.1) m) (iter T (dirac p.2) m)
      ≤ alpha T ^ m * tv (dirac p.1) (dirac p.2) :=
    tv_iter_le hT (isDist_dirac p.1) (isDist_dirac p.2) m
  have h3 : tv (dirac p.1) (dirac p.2) ≤ 1 :=
    tv_le_one (isDist_dirac p.1) (isDist_dirac p.2)
  have h4 : alpha T ^ m * tv (dirac p.1) (dirac p.2) ≤ alpha T ^ m * 1 :=
    mul_le_mul_of_nonneg_left h3 (pow_nonneg (alpha_nonneg T) m)
  linarith

/-- The reading of one fiber pair pins the defect from below; with `α(T) = 1` the
    bound above pins it from above. Used by both non-contraction witnesses. -/
theorem defect_eq_one_of_pair {T : X → X → ℝ} (hT : IsStoch T) {v : X → C} {m : ℕ}
    {x y : X} (hxy : v x = v y)
    (hval : tv (push v (iter T (dirac x) m)) (push v (iter T (dirac y) m)) = 1)
    (halpha : alpha T = 1) :
    defect T v m = 1 := by
  refine le_antisymm ?_ ?_
  · have h := defect_le_alpha_pow hT v m
    rwa [halpha, one_pow] at h
  · have h := le_defect (T := T) (m := m) hxy
    rwa [hval] at h

end Defect

/-! ## 3. The deterministic case: no mixing, hence no contraction -/

section Deterministic

variable {X : Type*} [Fintype X] [DecidableEq X]
variable {C : Type*} [Fintype C] [DecidableEq C]

/-- The kernel of a deterministic step map. -/
noncomputable def detKernel (f : X → X) : X → X → ℝ := fun x y => if y = f x then 1 else 0

omit [Fintype X] in
theorem detKernel_row (f : X → X) (x : X) : detKernel f x = dirac (f x) := rfl

theorem detKernel_isStoch (f : X → X) : IsStoch (detKernel f) := by
  refine ⟨fun x y => ?_, fun x => ?_⟩
  · unfold detKernel; split <;> norm_num
  · simp [detKernel]

theorem app_dirac_det (f : X → X) (x : X) :
    app (dirac x) (detKernel f) = dirac (f x) := by
  funext y
  unfold app
  rw [Finset.sum_eq_single x]
  · simp [dirac, detKernel]
  · intro b _ hbx; simp [dirac, hbx]
  · intro hx; exact absurd (Finset.mem_univ x) hx

theorem iter_dirac_det (f : X → X) (x : X) :
    ∀ m, iter (detKernel f) (dirac x) m = dirac (f^[m] x)
  | 0 => by simp
  | m + 1 => by
      rw [iter_succ, iter_dirac_det f x m, app_dirac_det, Function.iterate_succ_apply']

/-- The deterministic defect, computed pairwise: `1` when the view still
    separates the pair after `m` steps, `0` when it does not. -/
theorem det_defect_val (f : X → X) (v : X → C) (m : ℕ) (x y : X) :
    tv (push v (iter (detKernel f) (dirac x) m))
       (push v (iter (detKernel f) (dirac y) m))
      = if v (f^[m] x) = v (f^[m] y) then 0 else 1 := by
  rw [iter_dirac_det, iter_dirac_det, push_dirac, push_dirac, tv_dirac]

variable [Nonempty X]

/-- **THE DETERMINISTIC COROLLARY.** For a deterministic step the closure defect
    takes only the values `0` and `1` — never anything between. A `{0,1}`-valued
    sequence cannot decay at any rate `r < 1` except by reaching `0` exactly, so
    "the defect contracts slowly" is not a possible reading of a deterministic
    substrate: contraction is a stochastic phenomenon. -/
theorem det_defect_zero_or_one (f : X → X) (v : X → C) (m : ℕ) :
    defect (detKernel f) v m = 0 ∨ defect (detKernel f) v m = 1 := by
  obtain ⟨p, _, hp⟩ := Finset.exists_mem_eq_sup' (fiberPairs_nonempty v)
    (fun p : X × X => tv (push v (iter (detKernel f) (dirac p.1) m))
      (push v (iter (detKernel f) (dirac p.2) m)))
  rw [defect, hp, det_defect_val f v m p.1 p.2]
  split
  · exact Or.inl rfl
  · exact Or.inr rfl

/-- The defect is zero exactly when the step never splits a fiber over `m`
    steps. -/
theorem det_defect_eq_zero_iff (f : X → X) (v : X → C) (m : ℕ) :
    defect (detKernel f) v m = 0 ↔ ∀ x y, v x = v y → v (f^[m] x) = v (f^[m] y) := by
  constructor
  · intro h x y hxy
    have hle := le_defect (T := detKernel f) (m := m) hxy
    rw [h, det_defect_val f v m x y] at hle
    by_contra hne
    rw [if_neg hne] at hle
    linarith
  · intro h
    refine le_antisymm (Finset.sup'_le _ _ fun p hp => ?_) (defect_nonneg _ v m)
    simp only [fiberPairs, Finset.mem_filter] at hp
    rw [det_defect_val f v m p.1 p.2, if_pos (h p.1 p.2 hp.2)]

/-- **THE DEFECT *IS* `Core/Habit.lean`'s CLOSURE PREDICATE, GIVEN A NUMBER.**
    Zero defect at lag `m` for a deterministic step is exactly `Closed v (f^[m])`
    — and at `m = 1` this is `closed_iff_fiber_invariant` read through total
    variation. So the mixing theorem is a statement about the fiber ladder's
    "contraction" row and not merely about Markov chains. -/
theorem det_defect_eq_zero_iff_closed [Nonempty C] (f : X → X) (v : X → C) (m : ℕ) :
    defect (detKernel f) v m = 0 ↔ CIRISHolon.Object.Closed v (f^[m]) := by
  rw [det_defect_eq_zero_iff]
  exact CIRISHolon.Object.closed_iff_fiber_invariant.symm

/-- **α = 1 FOR EVERY NON-CONSTANT DETERMINISTIC STEP** — permutations included.
    So `defect_le_alpha_pow` degenerates to `defect ≤ 1`, which is the trivial
    bound: the theorem does not fail on determinism, it says nothing there. -/
theorem alpha_detKernel_eq_one {f : X → X} (h : ∃ x y, f x ≠ f y) :
    alpha (detKernel f) = 1 := by
  obtain ⟨x, y, hxy⟩ := h
  refine le_antisymm (alpha_le_one (detKernel_isStoch f)) ?_
  have hrow := tv_row_le_alpha (detKernel f) x y
  rw [detKernel_row, detKernel_row, tv_dirac, if_neg hxy] at hrow
  exact hrow

/-- A permutation of a state space with at least two states has `α = 1`: a
    bijective dynamics never forgets anything, so it never mixes. -/
theorem alpha_eq_one_of_injective {f : X → X} (hf : Function.Injective f)
    {a b : X} (hab : a ≠ b) : alpha (detKernel f) = 1 :=
  alpha_detKernel_eq_one ⟨a, b, fun h => hab (hf h)⟩

/-- The honest fence on "constant in `m`": a permutation of finite order `N`
    returns the defect to zero at every multiple of `N`, so a PERMUTATION's
    defect can never be constant and positive. What it can be — and the first
    witness below is — is `1` at infinitely many lags, which already refutes
    decay. -/
theorem det_defect_zero_at_period {f : X → X} {N : ℕ} (hN : f^[N] = id) (v : X → C) :
    defect (detKernel f) v N = 0 := by
  rw [det_defect_eq_zero_iff]
  intro x y hxy
  rw [hN]
  exact hxy

end Deterministic

/-! ## 4. Two exhibited witnesses of non-contraction

Both live on the two-slot world `Bool × Bool` under the first-slot view — the
same pair `Core/Habit.lean` uses for `not_closed_witness`. -/

section Witnesses

/-- The permutation witness, inlined from the upstream Core/Habit.lean:
    swap the two slots of a pair. Its first-slot view is not Closed, and its
    defect holds at 1 for every odd lag — determinism never pays down a miss. -/
def swapPair : Bool × Bool → Bool × Bool := fun p => (p.2, p.1)

theorem swapPair_sq : swapPair^[2] = id := by
  funext p; simp [Function.iterate_succ_apply, swapPair]

theorem swapPair_iterate_odd : ∀ k, swapPair^[2 * k + 1] = swapPair
  | 0 => by simp
  | k + 1 => by
      have hk : 2 * (k + 1) + 1 = (2 * k + 1) + 2 := by ring
      rw [hk, Function.iterate_add, swapPair_iterate_odd k, swapPair_sq]
      funext p; simp

/-- **WITNESS 1 — A PERMUTATION.** The swap on two slots, viewed through the
    first slot: the defect is exactly `1` at every odd lag, forever. It does not
    decay, and no geometric bound with rate `< 1` can hold. (`Habit.swapPair`'s
    non-closure is `Habit.not_closed_witness`; this is that fact given a
    dynamical number.) -/
theorem swap_defect_odd (k : ℕ) :
    defect (detKernel swapPair) (Prod.fst : Bool × Bool → Bool) (2 * k + 1) = 1 := by
  refine defect_eq_one_of_pair (detKernel_isStoch swapPair)
    (x := (false, false)) (y := (false, true)) rfl ?_ ?_
  · rw [det_defect_val swapPair (Prod.fst : Bool × Bool → Bool) (2 * k + 1),
      swapPair_iterate_odd k]
    simp [swapPair]
  · exact alpha_detKernel_eq_one ⟨(false, false), (false, true), by decide⟩

/-- Copy the second slot into both: a deterministic, NON-injective step. -/
def copySecond : Bool × Bool → Bool × Bool := fun p => (p.2, p.2)

theorem copySecond_idem : copySecond ∘ copySecond = copySecond := by
  funext p; simp [copySecond]

theorem copySecond_iterate : ∀ m, copySecond^[m + 1] = copySecond
  | 0 => by simp
  | m + 1 => by
      rw [Function.iterate_succ, copySecond_iterate m, copySecond_idem]

/-- **WITNESS 2 — CONSTANT IN `m`.** A non-injective deterministic step whose
    defect is `1` at EVERY lag `m ≥ 1`. Not periodic, not decaying: literally
    constant. This is the sharpest form of "deterministic ⇒ no contraction", and
    it is why the permutation class needs the fence
    `det_defect_zero_at_period` — only outside bijectivity can the defect be
    constant and positive on a finite space. -/
theorem copySecond_defect_succ (m : ℕ) :
    defect (detKernel copySecond) (Prod.fst : Bool × Bool → Bool) (m + 1) = 1 := by
  refine defect_eq_one_of_pair (detKernel_isStoch copySecond)
    (x := (false, false)) (y := (false, true)) rfl ?_ ?_
  · rw [det_defect_val copySecond (Prod.fst : Bool × Bool → Bool) (m + 1),
      copySecond_iterate m]
    simp [copySecond]
  · exact alpha_detKernel_eq_one ⟨(false, false), (false, true), by decide⟩

end Witnesses

/-! ## 5. The positive side: noise buys contraction, at a rate one can name -/

section Noisy

variable {X : Type*} [Fintype X] [DecidableEq X] [Nonempty X]
variable {C : Type*} [Fintype C] [DecidableEq C]

/-- A deterministic engine `f` run with per-step noise: with probability `1-ε`
    the engine steps, with probability `ε` the state is resampled uniformly. -/
noncomputable def noisyKernel (ε : ℝ) (f : X → X) : X → X → ℝ :=
  fun x y => (1 - ε) * (if y = f x then 1 else 0) + ε / (Fintype.card X)

theorem noisyKernel_isStoch {ε : ℝ} (h0 : 0 ≤ ε) (h1 : ε ≤ 1) (f : X → X) :
    IsStoch (noisyKernel ε f) := by
  have hcard : (0 : ℝ) < Fintype.card X := by
    exact_mod_cast Fintype.card_pos_iff.mpr ‹Nonempty X›
  refine ⟨fun x y => ?_, fun x => ?_⟩
  · unfold noisyKernel
    have hq : 0 ≤ ε / (Fintype.card X) := by positivity
    have hind : 0 ≤ (1 - ε) * (if y = f x then (1:ℝ) else 0) := by
      apply mul_nonneg (by linarith)
      split <;> norm_num
    linarith
  · unfold noisyKernel
    rw [Finset.sum_add_distrib, ← Finset.mul_sum]
    have h2 : ∑ y : X, (if y = f x then (1:ℝ) else 0) = 1 := by simp
    have h3 : ∑ _y : X, ε / (Fintype.card X) = ε := by
      rw [Finset.sum_const, Finset.card_univ, nsmul_eq_mul]
      field_simp
    rw [h2, h3]
    ring

omit [Nonempty X] in
/-- Every entry of the noisy kernel is at least `ε/|X|`: the Doeblin
    minorization, exhibited. -/
theorem noisyKernel_ge {ε : ℝ} (h1 : ε ≤ 1) (f : X → X) (x y : X) :
    ε / (Fintype.card X) ≤ noisyKernel ε f x y := by
  unfold noisyKernel
  have hind : 0 ≤ (1 - ε) * (if y = f x then (1:ℝ) else 0) := by
    apply mul_nonneg (by linarith)
    split <;> norm_num
  linarith

/-- The noisy engine's ergodicity coefficient is at most `1 - ε`. -/
theorem alpha_noisyKernel_le {ε : ℝ} (h0 : 0 ≤ ε) (h1 : ε ≤ 1) (f : X → X) :
    alpha (noisyKernel ε f) ≤ 1 - ε := by
  have hcard : (0 : ℝ) < Fintype.card X := by
    exact_mod_cast Fintype.card_pos_iff.mpr ‹Nonempty X›
  have hle := alpha_le_one_sub_card_mul (noisyKernel_isStoch h0 h1 f)
    (noisyKernel_ge h1 f)
  have heq : ε / (Fintype.card X) * (Fintype.card X) = ε := by field_simp
  rwa [heq] at hle

/-- **THE QUANTITATIVE MIXING COROLLARY.** The SAME deterministic engine whose
    defect is constant at `1` (witness 2) has defect at most `(1-ε)^m` once each
    step carries noise `ε`. Contraction is bought by the noise, and the price is
    named: this is the theorem that arm B3 of
    `scratchpad/composition/COMPOSITION2_RESULTS.md` was calibrated against and
    that its deterministic substrate could not supply. -/
theorem defect_noisy_le {ε : ℝ} (h0 : 0 ≤ ε) (h1 : ε ≤ 1) (f : X → X)
    (v : X → C) (m : ℕ) :
    defect (noisyKernel ε f) v m ≤ (1 - ε) ^ m := by
  refine le_trans (defect_le_alpha_pow (noisyKernel_isStoch h0 h1 f) v m) ?_
  exact pow_le_pow_left₀ (alpha_nonneg _) (alpha_noisyKernel_le h0 h1 f) m

end Noisy

end CIRISHolon.Mixing
