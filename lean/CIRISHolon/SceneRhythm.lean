/-
CIRISHolon.SceneRhythm — the population law in time: continuity with a declared
source. Written 2026-11-11 for two consumers that turned out to ask for the same
thing: the CEWP twin (who is where, by hour of the week) and the lepton ladder
(the axial anomaly, where the handed charge is NOT conserved and the amount by
which it is not is the thing to measure).

A RHYTHM is a count per cell per time bin, a flow per pair of cells per bin, and
a source per cell per bin. The law:

    what a cell holds next = what it holds − what leaves + what arrives + its source.

  * `total_step`, `conserved`     — the flows cancel in the total: with a zero
                                    source the total never changes, and with a
                                    source it changes by exactly the source.
  * `gpop_out`, `gpop_inn`        — reading coarsely commutes with counting what
                                    leaves and what arrives (the outflow law of
                                    `Ladder`, for signed counts, both directions).
  * `continuity_coarse`           — **a rhythm that obeys the law finely obeys it
                                    coarsely**, with the coarse flows and the
                                    coarse source. Flows inside a coarse cell land
                                    on its diagonal and cancel. A certificate
                                    issued at one tier is a certificate at every
                                    tier above it, with nothing more to check.
  * `source_is_the_defect`        — the source is not a free parameter: given the
                                    counts and the flows it is determined. So
                                    "the source is zero" (people, electric charge)
                                    and "the source is the field times 1/π" (the
                                    handed charge) are both statements one
                                    measures, by the same subtraction.

What this file does NOT say: what any source is. That people are conserved by
moving is a declaration a consumer makes and a checker tests; that the handed
charge's source is the electric field is physics, staked in ANOMALY-0.
-/
import Mathlib.Algebra.BigOperators.Group.Finset
import Mathlib.Data.Fintype.Prod
import Mathlib.Tactic
import CIRISHolon.SceneCharge

namespace CIRISHolon.SceneRhythm

open Finset CIRISHolon.SceneCharge

variable {C D : Type*} [Fintype C] [Fintype D] [DecidableEq D]

/-- What leaves a cell in one bin, to any cell. -/
def out (f : C × C → ℤ) (c : C) : ℤ := ∑ d, f (c, d)

/-- What arrives at a cell in one bin, from any cell. -/
def inn (f : C × C → ℤ) (c : C) : ℤ := ∑ d, f (d, c)

/-- **THE LAW.** Counts `N t`, flows `f t` and sources `s t`, bin by bin. -/
def Continuity (N : ℕ → C → ℤ) (f : ℕ → C × C → ℤ) (s : ℕ → C → ℤ) : Prop :=
  ∀ t c, N (t + 1) c = N t c - out (f t) c + inn (f t) c + s t c

omit [Fintype D] [DecidableEq D] in
/-- Everything that leaves somewhere arrives somewhere. -/
theorem out_total_eq_inn_total (f : C × C → ℤ) : ∑ c, out f c = ∑ c, inn f c := by
  unfold out inn
  exact Finset.sum_comm

omit [Fintype D] [DecidableEq D] in
/-- In one bin the total changes by exactly the total source. -/
theorem total_step {N : ℕ → C → ℤ} {f : ℕ → C × C → ℤ} {s : ℕ → C → ℤ}
    (h : Continuity N f s) (t : ℕ) :
    ∑ c, N (t + 1) c = ∑ c, N t c + ∑ c, s t c := by
  have h1 : ∑ c, N (t + 1) c = ∑ c, (N t c - out (f t) c + inn (f t) c + s t c) :=
    Finset.sum_congr rfl fun c _ => h t c
  rw [h1, Finset.sum_add_distrib, Finset.sum_add_distrib, Finset.sum_sub_distrib,
    out_total_eq_inn_total]
  ring

omit [Fintype D] [DecidableEq D] in
/-- **CONSERVATION.** With no source the total is the same in every bin. -/
theorem conserved {N : ℕ → C → ℤ} {f : ℕ → C × C → ℤ} {s : ℕ → C → ℤ}
    (h : Continuity N f s) (hs : ∀ t c, s t c = 0) (t : ℕ) :
    ∑ c, N t c = ∑ c, N 0 c := by
  induction t with
  | zero => rfl
  | succ n ih =>
    rw [total_step h n, ih]
    simp [hs]

omit [Fintype D] [DecidableEq D] in
/-- **THE SOURCE IS THE DEFECT.** Given counts and flows the source is whatever
    the count failed to account for; it is measured, not chosen. -/
theorem source_is_the_defect {N : ℕ → C → ℤ} {f : ℕ → C × C → ℤ} {s : ℕ → C → ℤ}
    (h : Continuity N f s) (t : ℕ) (c : C) :
    s t c = N (t + 1) c - N t c + out (f t) c - inn (f t) c := by
  rw [h t c]; ring

/-- Reading coarsely commutes with counting what leaves. -/
theorem gpop_out (v : C → D) (f : C × C → ℤ) :
    gpop v (out f) = out (gpop (Prod.map v v) f) := by
  funext e
  unfold gpop out
  symm
  rw [Finset.sum_comm, Fintype.sum_prod_type]
  refine Finset.sum_congr rfl fun x _ => ?_
  by_cases h : v x = e
  · rw [if_pos h]
    refine Finset.sum_congr rfl fun y _ => ?_
    rw [Finset.sum_eq_single (v y)]
    · simp [Prod.map, h]
    · intro d _ hd
      have : ¬ v y = d := fun e' => hd e'.symm
      simp [Prod.map, this]
    · intro hh; exact absurd (Finset.mem_univ _) hh
  · rw [if_neg h]
    refine Finset.sum_eq_zero fun y _ => Finset.sum_eq_zero fun d _ => ?_
    simp [Prod.map, h]

/-- Reading coarsely commutes with counting what arrives. -/
theorem gpop_inn (v : C → D) (f : C × C → ℤ) :
    gpop v (inn f) = inn (gpop (Prod.map v v) f) := by
  funext e
  unfold gpop inn
  symm
  rw [Finset.sum_comm, Fintype.sum_prod_type_right]
  refine Finset.sum_congr rfl fun y _ => ?_
  by_cases h : v y = e
  · rw [if_pos h]
    refine Finset.sum_congr rfl fun x _ => ?_
    rw [Finset.sum_eq_single (v x)]
    · simp [Prod.map, h]
    · intro d _ hd
      have : ¬ v x = d := fun e' => hd e'.symm
      simp [Prod.map, this]
    · intro hh; exact absurd (Finset.mem_univ _) hh
  · rw [if_neg h]
    refine Finset.sum_eq_zero fun x _ => Finset.sum_eq_zero fun d _ => ?_
    simp [Prod.map, h]

/-- **THE LAW SURVIVES READING COARSELY.** A rhythm that obeys continuity on
    the fine cells obeys it on the coarse ones, with the coarse counts, the
    coarse flows and the coarse source. -/
theorem continuity_coarse (v : C → D) {N : ℕ → C → ℤ} {f : ℕ → C × C → ℤ}
    {s : ℕ → C → ℤ} (h : Continuity N f s) :
    Continuity (fun t => gpop v (N t)) (fun t => gpop (Prod.map v v) (f t))
      (fun t => gpop v (s t)) := by
  intro t e
  show gpop v (N (t + 1)) e
      = gpop v (N t) e - out (gpop (Prod.map v v) (f t)) e
        + inn (gpop (Prod.map v v) (f t)) e + gpop v (s t) e
  rw [← gpop_out, ← gpop_inn]
  have hN : N (t + 1) = fun c => N t c - out (f t) c + inn (f t) c + s t c :=
    funext (h t)
  rw [hN]
  unfold gpop
  rw [← Finset.sum_sub_distrib, ← Finset.sum_add_distrib, ← Finset.sum_add_distrib]
  refine Finset.sum_congr rfl fun c _ => ?_
  by_cases hc : v c = e <;> simp [hc]

end CIRISHolon.SceneRhythm
