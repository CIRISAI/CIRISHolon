/-
CIRISHolon.Stabilizer — the 1-rebit kernel of the Clifford stratum: the
tableau view (signed-Pauli label, none off the set) is CLOSED under Hadamard
conjugation and NOT closed under the 3-4-5 rotation — a rational
non-Clifford whose witness pair prices magic out of the stabilizer tier, in
the same currency as every other wall in this contract. The n-qubit
efficiency statement is Gottesman–Knill (credited; Aaronson–Gottesman for
tableaux), not claimed here; this is its mechanism as a closure theorem.
-/
import CIRISHolon.Object
import Mathlib.Tactic

namespace CIRISHolon.Object

section Stabilizer

open Matrix

/-- The eight signed Paulis of one rebit: ±I, ±X, ±Z, ±XZ. -/
noncomputable def pauli : Fin 8 → Matrix (Fin 2) (Fin 2) ℝ
  | 0 => !![1, 0; 0, 1]
  | 1 => !![0, 1; 1, 0]
  | 2 => !![1, 0; 0, -1]
  | 3 => !![0, -1; 1, 0]
  | 4 => !![-1, 0; 0, -1]
  | 5 => !![0, -1; -1, 0]
  | 6 => !![-1, 0; 0, 1]
  | 7 => !![0, 1; -1, 0]

/-- The TABLEAU VIEW: the label of a matrix if it is a signed Pauli, `none`
    otherwise. This is the lossy reading a stabilizer tier runs on. -/
noncomputable def label (M : Matrix (Fin 2) (Fin 2) ℝ) : Option (Fin 8) :=
  if h : ∃ i, pauli i = M then some h.choose else none

theorem label_pauli_isSome (i : Fin 8) : ∃ j, label (pauli i) = some j := by
  unfold label
  rw [dif_pos ⟨i, rfl⟩]
  exact ⟨_, rfl⟩

theorem label_eq_none {M : Matrix (Fin 2) (Fin 2) ℝ}
    (h : ∀ i, pauli i ≠ M) : label M = none := by
  unfold label
  rw [dif_neg]
  rintro ⟨i, hi⟩
  exact h i hi

/-- Hadamard conjugation, kept rational (see `hadamardMap`). -/
noncomputable def hconj (M : Matrix (Fin 2) (Fin 2) ℝ) :
    Matrix (Fin 2) (Fin 2) ℝ := hadamardMap M

/-- The Hadamard maps the signed-Pauli set INTO itself: the tableau tier's
    invariance, computed case by case (I→I, X→Z, Z→X, XZ→−XZ, signs carried). -/
theorem hconj_pauli : ∀ i : Fin 8, ∃ j : Fin 8, hconj (pauli i) = pauli j := by
  intro i
  fin_cases i
  · exact ⟨0, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨2, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨1, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨7, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨4, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨6, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨5, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩
  · exact ⟨3, by ext a b; fin_cases a <;> fin_cases b <;>
      simp [hconj, hadamardMap, pauli, Matrix.mul_apply, Fin.sum_univ_two] <;> norm_num⟩

/-- Hadamard conjugation is an involution: H'² = 2·1 and the ½'s cancel. -/
theorem hconj_involutive : Function.Involutive hconj := by
  intro M
  unfold hconj hadamardMap
  have hH2 : (!![1,1;1,-1] : Matrix (Fin 2) (Fin 2) ℝ) * !![1,1;1,-1]
      = (2:ℝ) • (1 : Matrix (Fin 2) (Fin 2) ℝ) := by
    ext a b
    fin_cases a <;> fin_cases b <;>
      simp [Matrix.mul_apply, Fin.sum_univ_two, Matrix.one_apply] <;> norm_num
  rw [Matrix.mul_smul, Matrix.smul_mul, smul_smul]
  have hassoc : (!![1,1;1,-1] : Matrix (Fin 2) (Fin 2) ℝ) *
        (!![1,1;1,-1] * M * !![1,1;1,-1]) * !![1,1;1,-1]
      = (!![1,1;1,-1] * !![1,1;1,-1]) * M * (!![1,1;1,-1] * !![1,1;1,-1]) := by
    simp only [Matrix.mul_assoc]
  rw [hassoc, hH2]
  simp only [Matrix.smul_mul, Matrix.mul_smul, Matrix.one_mul, Matrix.mul_one,
             smul_smul]
  norm_num

/-- **THE TABLEAU VIEW IS CLOSED UNDER THE CLIFFORD MOTION.** Two matrices
    with the same label stay same-labelled after Hadamard conjugation: signed
    Paulis ride the orbit, non-Paulis stay non-Paulis (by involutivity). This
    is the 1-rebit kernel of WHY stabilizer circuits are a cheap tier
    (Gottesman–Knill's mechanism, credited — the n-qubit efficiency statement
    is theirs, not claimed here). -/
theorem tableau_closed_under_hadamard : Closed label hconj := by
  rw [closed_iff_fiber_invariant]
  intro M N hMN
  by_cases hM : ∃ i, pauli i = M
  · have hN : ∃ k, pauli k = N := by
      by_contra hc
      push_neg at hc
      rw [label_eq_none hc] at hMN
      unfold label at hMN
      rw [dif_pos hM] at hMN
      simp at hMN
    have hlab : hM.choose = hN.choose := by
      have h1 : label M = some hM.choose := by unfold label; rw [dif_pos hM]
      have h2 : label N = some hN.choose := by unfold label; rw [dif_pos hN]
      rw [h1, h2] at hMN
      exact Option.some_inj.mp hMN
    have hMN' : M = N := by
      rw [← hM.choose_spec, ← hN.choose_spec, hlab]
    rw [hMN']
  · -- M is not a Pauli; then neither is N (same label = none), and
    -- neither image is (involutivity pulls a Pauli image back to a Pauli).
    push_neg at hM
    have hMnone := label_eq_none hM
    have hNnone : label N = none := by rw [← hMN, hMnone]
    have hNnot : ∀ i, pauli i ≠ N := by
      intro i hi
      obtain ⟨j, hj⟩ := label_pauli_isSome i
      rw [hi] at hj
      rw [hj] at hNnone
      simp at hNnone
    have himg : ∀ (P : Matrix (Fin 2) (Fin 2) ℝ), (∀ i, pauli i ≠ P) →
        ∀ i, pauli i ≠ hconj P := by
      intro P hP i hi
      obtain ⟨j, hj⟩ := hconj_pauli i
      have : hconj (hconj P) = hconj (pauli i) := by rw [hi]
      rw [hconj_involutive P, hj] at this
      exact hP j this.symm
    rw [label_eq_none (himg M hM), label_eq_none (himg N hNnot)]

/-- The 3-4-5 rotation: a RATIONAL orthogonal matrix that is not a Clifford. -/
noncomputable def R345 : Matrix (Fin 2) (Fin 2) ℝ := !![3/5, -4/5; 4/5, 3/5]
noncomputable def R345t : Matrix (Fin 2) (Fin 2) ℝ := !![3/5, 4/5; -4/5, 3/5]

noncomputable def rconj (M : Matrix (Fin 2) (Fin 2) ℝ) :
    Matrix (Fin 2) (Fin 2) ℝ := R345 * M * R345t

theorem RRt : R345 * R345t = 1 := by
  ext a b
  fin_cases a <;> fin_cases b <;>
    simp [R345, R345t, Matrix.mul_apply, Fin.sum_univ_two, Matrix.one_apply] <;>
    norm_num

theorem pauli0_eq_one : pauli 0 = 1 := by
  ext a b
  fin_cases a <;> fin_cases b <;> simp [pauli, Matrix.one_apply]

/-- The rotation undoes its transpose-conjugation. -/
theorem rconj_rtconj (M : Matrix (Fin 2) (Fin 2) ℝ) :
    rconj (R345t * M * R345) = M := by
  unfold rconj
  have hassoc : R345 * (R345t * M * R345) * R345t
      = (R345 * R345t) * M * (R345 * R345t) := by
    simp only [Matrix.mul_assoc]
  rw [hassoc, RRt, Matrix.one_mul, Matrix.mul_one]

/-- The pulled-back X is not a signed Pauli (its (0,0) entry is 24/25). -/
theorem pullback_not_pauli : ∀ i, pauli i ≠ R345t * pauli 1 * R345 := by
  intro i h
  have h00 := congrFun (congrFun h 0) 0
  fin_cases i <;>
    simp [pauli, R345, R345t, Matrix.mul_apply, Fin.sum_univ_two] at h00 <;>
    norm_num at h00

/-- Twice the identity is not a signed Pauli either. -/
theorem twoI_not_pauli : ∀ i, pauli i ≠ (2 : ℝ) • pauli 0 := by
  intro i h
  have h00 := congrFun (congrFun h 0) 0
  fin_cases i <;>
    simp [pauli, Matrix.smul_apply] at h00 <;> norm_num at h00

/-- The rotation fixes scalar matrices. -/
theorem rconj_twoI : rconj ((2 : ℝ) • pauli 0) = (2 : ℝ) • pauli 0 := by
  unfold rconj
  rw [pauli0_eq_one, Matrix.mul_smul, Matrix.mul_one, Matrix.smul_mul, RRt]

/-- **THE MAGIC WALL.** The tableau view is NOT Closed under the 3-4-5
    rotation: the pulled-back X and the scalar 2·I agree on the view (both
    unlabelled), and the rotation sends one back onto a Pauli and not the
    other. A non-Clifford motion prices out of the stabilizer tier — this is
    the 1-rebit kernel of why magic costs, in the same witness-pair currency
    as every other wall in this contract. -/
theorem tableau_not_closed_under_rotation : ¬ Closed label rconj := by
  rw [closed_iff_fiber_invariant]
  intro hall
  have hpair := hall (R345t * pauli 1 * R345) ((2:ℝ) • pauli 0)
    (by rw [label_eq_none pullback_not_pauli, label_eq_none twoI_not_pauli])
  rw [rconj_rtconj, rconj_twoI, label_eq_none twoI_not_pauli] at hpair
  obtain ⟨j, hj⟩ := label_pauli_isSome 1
  rw [hj] at hpair
  simp at hpair

end Stabilizer

end CIRISHolon.Object
