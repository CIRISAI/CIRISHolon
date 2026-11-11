/-
CIRISHolon.SceneCharge — the population law for charges: counts in any additive
group, and what changes at the floor. Written 2026-11-11 for the question
`LEPTON_LADDER.md` leaves open (is a fermion an input, or can it be a closed
view of a vacuum that holds none?).

`ScenePopulation.pop` counts in ℕ: people, households, devices. A charge does
not. Fermion parity counts in ℤ/2; electric charge counts in ℤ. This file
restates the population view for a count in any additive commutative monoid and
proves the two things that are DIFFERENT once the count can cancel.

  * `gpop_total`, `gpop_comp`, `gpop_nat` — the view, its conservation and its
    composition are unchanged; on ℕ it is `pop`.
  * `vacuum_can_pair` — in ℤ/2 and in ℤ an empty parent can hold non-empty
    children. `Ladder.vacuum_generates_vacuum` is a theorem about ℕ and is
    FALSE for charges: the vacuum can generate a pair. The floor of the ladder
    is a floor for counts, not for charges.
  * `no_lone_charge` — what does survive: under an empty parent no scene holds
    exactly one charged child. A charge arrives with a partner, and the
    relationship between the two is not optional. This is the population-law
    face of `BareCharge`'s annihilation and `Vacuum`'s dressed promotion.
  * `odd_parent_has_odd_child` — parity is carried: a parent of odd parity has
    a child of odd parity, whatever the seed.

What this file does NOT say: that any such pair is a fermion, that it is closed
or carried, or that it is a lepton. Those are measured stakes (FERMION-0), not
consequences of counting.
-/
import Mathlib.Algebra.BigOperators.Group.Finset
import Mathlib.Data.ZMod.Basic
import Mathlib.Tactic
import CIRISHolon.ScenePopulation

namespace CIRISHolon.SceneCharge

open Finset CIRISHolon.ScenePopulation

variable {F C D : Type*} [Fintype F] [Fintype C] [DecidableEq C] [DecidableEq D]
variable {M : Type*} [AddCommMonoid M]

/-- The parent's reading of a charge: the sum of its children's, in `M`. -/
def gpop (v : F → C) (n : F → M) (c : C) : M := ∑ x, if v x = c then n x else 0

omit [Fintype C] in
/-- On ℕ this is `ScenePopulation.pop`. -/
theorem gpop_nat (v : F → C) (n : F → ℕ) : gpop v n = pop v n := rfl

/-- Conservation: the total charge is the same read coarsely or finely. -/
theorem gpop_total (v : F → C) (n : F → M) : ∑ c, gpop v n c = ∑ x, n x := by
  unfold gpop
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl fun x _ => ?_
  rw [Finset.sum_eq_single (v x)]
  · simp
  · intro c _ hc
    have : ¬ v x = c := fun h => hc h.symm
    simp [this]
  · intro h; exact absurd (Finset.mem_univ _) h

/-- Containment composes for charges as it does for counts. -/
theorem gpop_comp (v : F → C) (u : C → D) (n : F → M) :
    gpop u (gpop v n) = gpop (u ∘ v) n := by
  funext d
  unfold gpop
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

/-- **THE VACUUM CAN PAIR.** With a count that can cancel, an empty parent
    holds non-empty children: two odd children in ℤ/2, a `+1` and a `−1` in ℤ.
    The floor theorem for counts does not hold for charges. -/
theorem vacuum_can_pair :
    (∃ n : Bool → ZMod 2, gpop (fun _ => ()) n = (fun _ => 0) ∧ n ≠ fun _ => 0) ∧
    (∃ n : Bool → ℤ, gpop (fun _ => ()) n = (fun _ => 0) ∧ n ≠ fun _ => 0) := by
  refine ⟨⟨fun _ => 1, ?_, ?_⟩, ⟨fun b => if b then 1 else -1, ?_, ?_⟩⟩
  · funext u; simp [gpop]; decide
  · intro h; have := congrFun h true; revert this; decide
  · funext u; simp [gpop]
  · intro h; have := congrFun h true; simp at this

omit [Fintype C] in
/-- **NO LONE CHARGE.** If a parent reads zero and every child but one is
    empty, that one is empty too. A charge under an empty parent has a partner
    under the same parent. -/
theorem no_lone_charge (v : F → C) (n : F → M) (x : F)
    (hrest : ∀ y, y ≠ x → n y = 0) (hzero : gpop v n (v x) = 0) : n x = 0 := by
  unfold gpop at hzero
  rw [Finset.sum_eq_single x] at hzero
  · simpa using hzero
  · intro y _ hy; simp [hrest y hy]
  · intro h; exact absurd (Finset.mem_univ _) h

omit [Fintype C] in
/-- **PARITY IS CARRIED.** A parent of odd parity has a child of odd parity. -/
theorem odd_parent_has_odd_child (v : F → C) (n : F → ZMod 2) (c : C)
    (h : gpop v n c = 1) : ∃ x, v x = c ∧ n x = 1 := by
  by_contra hno
  push_neg at hno
  have hall : ∀ x, (if v x = c then n x else 0) = 0 := by
    intro x
    by_cases hx : v x = c
    · have h1 := hno x hx
      have : n x = 0 := by
        have h01 : ∀ a : ZMod 2, a = 0 ∨ a = 1 := by decide
        rcases h01 (n x) with h0 | h0
        · exact h0
        · exact absurd h0 h1
      simp [hx, this]
    · simp [hx]
  have : gpop v n c = 0 := by
    unfold gpop
    exact Finset.sum_eq_zero fun x _ => hall x
  rw [this] at h
  exact absurd h (by decide)

end CIRISHolon.SceneCharge
