/-
CIRISHolon.Object — the object, stated once, as the engineering contract.

A holon exposes VIEWS. A view `v : X → C` is a lossy reading of a state
space; a MOTION `T : X → X` is a step of dynamics, a re-root of context, or a
change of chart. The whole contract is one question:

    does the reading survive the motion?

* `Closed v T` — yes: some update `h` on readings commutes with the motion.
  A TIER of this engine is exactly a Closed view of the tier below, and `h`
  is the coarse dynamics the engine may run in its place.
* `Held v T`  — yes, unchanged: the reading is invariant. Conserved
  quantities, paid-up maintenance.
* `NonFactoring` — no, with a certificate: two states that agree on every
  reading and are split by the motion. `nonfactoring_iff_not_closed` proves
  this is EXACTLY the failure of `Closed`: every conformance failure this
  engine can have is such a witness pair, and the test harness's job is to
  hunt them.

Four consequences ship with the question, each load-bearing for engineering:

1. CONDITIONING — a chart that nearly cancels amplifies per-term noise by the
   inverse of its coherence, exactly (`sum_perturb_le`, `sum_perturb_attained`).
   Design rule: never expose an ill-conditioned aggregate as engine state;
   an all-nonnegative chart is perfectly conditioned (`coherence_of_nonneg`).
2. RENT — maintained state under decay `lam` and dose `q` retains exactly
   `Ginf = q/((1−lam)+q·lam)` of its target, with closed-form transient and
   price `Wstar` for a chosen retention (`rent_closed_form`, `Ginf_at_Wstar`).
   Design rule: level-of-detail refresh budgets are computed from this law,
   never tuned.
3. THE DIAGONAL — the classical engine is a RETRACT of a quantum carrier:
   `bornView ∘ diagEmbed = id`, the lifted channel factors through the
   classical state space by definition, and Born readout is a Closed view
   with the classical step itself as its update (`diag_view_closed_of_classical`).
4. THE WALL — a coherence-generating motion splits the diagonal view
   (`hadamard_splits_diagonal`, `diag_not_closed_under_coherence`): the
   classical tier ends exactly where coherence begins. Design rule: past the
   wall the engine spends known-exponential resources, delegates to hardware,
   or REFUSES BY NAME — it never pretends.

Provenance: every theorem here is transplanted verbatim (or with only local
renaming) from CIRISAI/CIRISOntology, where it was machine-checked and where
the measured campaign record behind the design rules lives. This file is the
engineering excerpt, not the research programme.
-/
import Mathlib.Analysis.SpecificLimits.Basic
import Mathlib.Data.Matrix.Basic
import Mathlib.Tactic

namespace CIRISHolon.Object

/-! ### The question -/

variable {X C : Type*}

/-- A view survives a motion: some update on readings commutes. A TIER is a
    Closed view of the tier below. -/
def Closed (v : X → C) (T : X → X) : Prop := ∃ h : C → C, v ∘ T = h ∘ v

/-- The reading survives UNCHANGED. -/
def Held (v : X → C) (T : X → X) : Prop := v ∘ T = v

/-- The no, with a certificate: two states agreeing under every view,
    split in a quantity. -/
def NonFactoring {ι : Type*} {View : ι → Type*} {Datum : Type*}
    (view : (i : ι) → X → View i) (q : X → Datum) : Prop :=
  ∃ x y : X, (∀ i, view i x = view i y) ∧ q x ≠ q y

/-- Closure is fiber-invariance: the motion never splits a reading class. -/
theorem closed_iff_fiber_invariant [Nonempty C] {v : X → C} {T : X → X} :
    Closed v T ↔ ∀ x y, v x = v y → v (T x) = v (T y) := by
  constructor
  · rintro ⟨h, hh⟩ x y hxy
    have hx := congrFun hh x
    have hy := congrFun hh y
    simp only [Function.comp_apply] at hx hy
    rw [hx, hy, hxy]
  · intro hf
    classical
    refine ⟨fun c => if hc : ∃ x, v x = c then v (T hc.choose)
                     else Classical.arbitrary C, ?_⟩
    funext x
    have hex : ∃ y, v y = v x := ⟨x, rfl⟩
    simp only [Function.comp_apply, dif_pos hex]
    exact (hf _ x hex.choose_spec).symm

/-- **The no is exactly the failure of the yes**: a NonFactoring witness for
    the single view `v` and quantity `v ∘ T` is the obstruction to `Closed`.
    Every conformance failure is such a pair; the harness hunts them. -/
theorem nonfactoring_iff_not_closed [Nonempty C] {v : X → C} {T : X → X} :
    NonFactoring (fun _ : Unit => v) (v ∘ T) ↔ ¬ Closed v T := by
  rw [closed_iff_fiber_invariant]
  constructor
  · rintro ⟨x, y, hv, hq⟩ hall
    exact hq (hall x y (hv ()))
  · intro h
    push_neg at h
    obtain ⟨x, y, hxy, hne⟩ := h
    exact ⟨x, y, fun _ => hxy, hne⟩

/-! ### Conditioning: what a chart may amplify -/

open Finset in
/-- Coherence of a finite family: aggregate magnitude over aligned magnitude. -/
noncomputable def coherence {n : ℕ} (a : Fin n → ℝ) : ℝ :=
  |∑ i, a i| / ∑ i, |a i|

open Finset in
/-- A per-term relative perturbation of size `ε` moves an aggregate by at most
    `ε` times the aligned sum — `ε / coherence` in the chart's own units. -/
theorem sum_perturb_le {n : ℕ} (a δ : Fin n → ℝ) (ε : ℝ)
    (h : ∀ i, |δ i| ≤ ε * |a i|) :
    |∑ i, (a i + δ i) - ∑ i, a i| ≤ ε * ∑ i, |a i| := by
  have : (∑ i, (a i + δ i)) - ∑ i, a i = ∑ i, δ i := by
    rw [Finset.sum_add_distrib]; ring
  rw [this, Finset.mul_sum]
  exact (Finset.abs_sum_le_sum_abs _ _).trans (Finset.sum_le_sum fun i _ => h i)

open Finset in
/-- The bound is exact: the aligned perturbation attains it. -/
theorem sum_perturb_attained {n : ℕ} (a : Fin n → ℝ) (ε : ℝ) (hε : 0 ≤ ε) :
    ∃ δ : Fin n → ℝ, (∀ i, |δ i| ≤ ε * |a i|) ∧
      |∑ i, (a i + δ i) - ∑ i, a i| = ε * ∑ i, |a i| := by
  refine ⟨fun i => ε * |a i|, fun i => ?_, ?_⟩
  · rw [abs_mul, abs_of_nonneg hε, abs_abs]
  · have h1 : (∑ i, (a i + ε * |a i|)) - ∑ i, a i = ε * ∑ i, |a i| := by
      rw [Finset.sum_add_distrib, Finset.mul_sum]; ring
    rw [h1, abs_of_nonneg]
    exact mul_nonneg hε (Finset.sum_nonneg fun i _ => abs_nonneg _)

open Finset in
/-- An all-nonnegative chart with a nonzero reading is perfectly conditioned. -/
theorem coherence_of_nonneg {n : ℕ} (a : Fin n → ℝ) (h : ∀ i, 0 ≤ a i)
    (hz : ∑ i, a i ≠ 0) : coherence a = 1 := by
  unfold coherence
  have he : ∀ i ∈ Finset.univ, |a i| = a i := fun i _ => abs_of_nonneg (h i)
  rw [Finset.sum_congr rfl he, abs_of_nonneg (Finset.sum_nonneg fun i _ => h i)]
  exact div_self hz

/-! ### Rent: what maintenance costs, exactly -/

/-- One maintained step: decay by `(1−q)·lam`, deposit `q·s0`. -/
def rentStep (lam q s0 : ℝ) (s : ℝ) : ℝ := (1 - q) * lam * s + q * s0

/-- The maintained orbit from `s0`. -/
def rentOrbit (lam q s0 : ℝ) : ℕ → ℝ
  | 0 => s0
  | n + 1 => rentStep lam q s0 (rentOrbit lam q s0 n)

/-- The fixed point of the maintained step. -/
noncomputable def rentFix (lam q s0 : ℝ) : ℝ := q * s0 / (1 - (1 - q) * lam)

/-- Stationary retention as a fraction of the deposit target. -/
noncomputable def Ginf (lam q : ℝ) : ℝ := q / ((1 - lam) + q * lam)

/-- The price of retention `1 − δ` at gap `γ`. -/
noncomputable def Wstar (γ δ : ℝ) : ℝ := (1 - δ) * γ / (γ + δ * (1 - γ))

/-- The fixed point is the `Ginf` fraction of the target. -/
theorem rentFix_eq_Ginf_mul (lam q s0 : ℝ) :
    rentFix lam q s0 = Ginf lam q * s0 := by
  unfold rentFix Ginf
  have : 1 - (1 - q) * lam = (1 - lam) + q * lam := by ring
  rw [this]; ring

/-- The closed form: fixed point plus a geometrically decaying transient. -/
theorem rent_closed_form (lam q s0 : ℝ) (h : (1 - q) * lam ≠ 1) (n : ℕ) :
    rentOrbit lam q s0 n = rentFix lam q s0 +
      ((1 - q) * lam) ^ n * (s0 - rentFix lam q s0) := by
  have hD : (1 : ℝ) - (1 - q) * lam ≠ 0 := sub_ne_zero.mpr (Ne.symm h)
  have hfix : (1 - q) * lam * rentFix lam q s0 + q * s0 = rentFix lam q s0 := by
    unfold rentFix; field_simp; ring
  induction n with
  | zero => simp [rentOrbit]
  | succ n ih =>
      show rentStep lam q s0 (rentOrbit lam q s0 n) = _
      rw [ih]
      unfold rentStep
      calc (1 - q) * lam * (rentFix lam q s0 +
              ((1 - q) * lam) ^ n * (s0 - rentFix lam q s0)) + q * s0
          = ((1 - q) * lam * rentFix lam q s0 + q * s0) +
              ((1 - q) * lam) ^ (n + 1) * (s0 - rentFix lam q s0) := by ring
        _ = _ := by rw [hfix]

/-- Dosing at `Wstar γ δ` holds retention at exactly `1 − δ` (`lam = 1 − γ`). -/
theorem Ginf_at_Wstar (γ δ : ℝ) (hγ : 0 < γ) (hδ : 0 < δ) (hδ1 : δ < 1) :
    Ginf (1 - γ) (Wstar γ δ) = 1 - δ := by
  have hE : (0:ℝ) < γ + δ * (1 - γ) := by nlinarith
  have hE' : γ + δ * (1 - γ) ≠ 0 := ne_of_gt hE
  unfold Ginf Wstar
  have hden : (1 - (1 - γ)) + (1 - δ) * γ / (γ + δ * (1 - γ)) * (1 - γ)
      = γ / (γ + δ * (1 - γ)) := by
    field_simp
    ring
  rw [hden, div_div_eq_mul_div, div_mul_cancel₀ _ hE',
      mul_div_cancel_right₀ _ hγ.ne']

/-! ### The diagonal, and the wall -/

/-- A classical state on `n` outcomes. -/
abbrev CState (n : ℕ) := Fin n → ℝ

/-- A row-stochastic classical map. -/
abbrev SMap (n : ℕ) := Fin n → Fin n → ℝ

open Finset in
/-- The classical push-forward. -/
def push {n : ℕ} (T : SMap n) (p : CState n) : CState n := fun j => ∑ i, p i * T i j

/-- The lift of a classical state: the diagonal density matrix. -/
def diagEmbed {n : ℕ} (p : CState n) : Matrix (Fin n) (Fin n) ℝ :=
  Matrix.diagonal p

/-- The diagonal (Born) view of a density matrix. -/
def bornView {n : ℕ} (ρ : Matrix (Fin n) (Fin n) ℝ) : CState n := fun i => ρ i i

/-- The lifted channel: measure, evolve classically, re-prepare. -/
def liftChannel {n : ℕ} (T : SMap n) (ρ : Matrix (Fin n) (Fin n) ℝ) :
    Matrix (Fin n) (Fin n) ℝ :=
  Matrix.diagonal (push T (bornView ρ))

/-- The classical engine is a RETRACT of the quantum carrier. -/
theorem bornView_diagEmbed {n : ℕ} : (bornView (n := n)) ∘ diagEmbed = id := by
  funext p i
  simp [bornView, diagEmbed, Matrix.diagonal]

/-- The lifted channel factors through the classical state space, by
    definition: the classical step conjugated by the retract pair. -/
theorem liftChannel_factors {n : ℕ} (T : SMap n) :
    liftChannel T = diagEmbed ∘ push T ∘ (bornView (n := n)) := rfl

/-- The diagonal-lift square commutes. -/
theorem lift_commutes {n : ℕ} (T : SMap n) (p : CState n) :
    liftChannel T (diagEmbed p) = diagEmbed (push T p) := by
  unfold liftChannel diagEmbed bornView
  simp [Matrix.diagonal]

/-- Born readout is a Closed view of the lifted dynamics, with the classical
    step itself as the update. -/
theorem diag_view_closed_of_classical {n : ℕ} (T : SMap n) :
    bornView ∘ liftChannel T = push T ∘ (bornView (n := n)) := by
  funext ρ i
  unfold liftChannel bornView
  simp [Matrix.diagonal]

/-- Two states with the same diagonal; one carries coherence. -/
noncomputable def ρplus : Matrix (Fin 2) (Fin 2) ℝ := !![1/2, 1/2; 1/2, 1/2]
noncomputable def ρmix : Matrix (Fin 2) (Fin 2) ℝ := !![1/2, 0; 0, 1/2]

/-- Hadamard conjugation, kept rational: `(1/2)·H′ρH′` with unnormalized `H′`. -/
noncomputable def hadamardMap (ρ : Matrix (Fin 2) (Fin 2) ℝ) :
    Matrix (Fin 2) (Fin 2) ℝ :=
  let H' : Matrix (Fin 2) (Fin 2) ℝ := !![1, 1; 1, -1]
  (1 / 2 : ℝ) • (H' * ρ * H')

/-- **THE WALL**: same Born view, split by a coherent motion. -/
theorem hadamard_splits_diagonal :
    bornView ρplus = bornView ρmix ∧
    bornView (hadamardMap ρplus) ≠ bornView (hadamardMap ρmix) := by
  constructor
  · funext i
    fin_cases i <;> simp [bornView, ρplus, ρmix]
  · intro h
    have h0 := congrFun h 0
    simp [bornView, hadamardMap, ρplus, ρmix, Matrix.mul_apply,
          Fin.sum_univ_two, Matrix.smul_apply] at h0

/-- The wall as a non-closure certificate: the diagonal view of a
    coherence-generating motion is not Closed. The classical tier ends
    exactly where coherence begins. -/
theorem diag_not_closed_under_coherence :
    ¬ Closed (bornView (n := 2)) hadamardMap :=
  (nonfactoring_iff_not_closed).mp
    ⟨ρplus, ρmix, fun _ => hadamard_splits_diagonal.1,
      hadamard_splits_diagonal.2⟩

end CIRISHolon.Object
