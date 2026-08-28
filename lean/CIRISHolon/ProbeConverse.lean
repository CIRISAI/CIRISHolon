/-
CIRISHolon.ProbeConverse — rung 3, the HARD half, both ways.

`Probe.lean` machine-checks: same Ω ⇒ every probe agrees.  This file settles
the converse, which turns out to be FALSE in general and TRUE exactly under
observability.

1. `probeAgreement_not_implies_omegaIso` — a COUNTEREXAMPLE at |S| = 3,
   |V| = 2: two holons with the SAME surjective view, both Closed, both with
   bijective (reversible) dynamics, agreeing on every probe at every depth
   from every state, that admit NO Ω-isomorphism — for ANY state bijection
   and ANY view bijection.  The difference lives strictly inside a view
   fiber: H₁ leaves the fiber {0,1} alone, H₂ swaps it, and no probe stream
   can see a permutation of states that share a stream.

2. `omegaIso_of_probeAgreement` — the converse holds, with the identical
   witnesses σ, τ, as soon as the target holon is OBSERVABLE (distinct
   states have distinct probe streams).  `observable_transfer` shows
   observability is a property of the probe data, so the hypothesis can be
   read off either side.

Together: probe data determines Ω exactly up to the choice of a lift through
the observable (Moore) quotient.  Ω's identity is strictly finer than its
probe behaviour, and the excess is fiber-internal.
-/
import Mathlib.Tactic
import CIRISHolon.Omega
import CIRISHolon.Probe

namespace CIRISHolon.ProbeConverse

open CIRISHolon.Omega CIRISHolon.Probe

/-- Depth `n+1` at `s` is depth `n` at the stepped state. -/
theorem probe_succ {S V : Type} (H : Holon S V) (n : ℕ) (s : S) :
    probe H (n + 1) s = probe H n (H.dyn s) := by
  unfold probe
  rw [Function.iterate_succ_apply]

/-- **Observability**: no two distinct states share a probe stream. -/
def Observable {S V : Type} (H : Holon S V) : Prop :=
  ∀ s s' : S, (∀ n, probe H n s = probe H n s') → s = s'

/-- **Probe agreement, formalization A**: matched initial states give matched
    probe streams.  Note what is NOT required — `σ` need not intertwine the
    dynamics; only the observations are constrained. -/
structure ProbeAgreement {S₁ S₂ V₁ V₂ : Type}
    (H₁ : Holon S₁ V₁) (H₂ : Holon S₂ V₂) where
  σ : S₁ ≃ S₂
  τ : V₁ ≃ V₂
  probe_eq : ∀ (n : ℕ) (s : S₁), probe H₂ n (σ s) = τ (probe H₁ n s)

/-- Every Ω-isomorphism is a probe agreement (`Probe.lean`, repackaged). -/
def ProbeAgreement.ofOmegaIso {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂} (φ : OmegaIso H₁ H₂) :
    ProbeAgreement H₁ H₂ :=
  ⟨φ.σ, φ.τ, same_omega_implies_probe_agreement φ⟩

/-- Observability is a property of the PROBE data, so it may be assumed on
    either side of a probe agreement. -/
theorem observable_transfer {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂} (φ : ProbeAgreement H₁ H₂) :
    Observable H₁ ↔ Observable H₂ := by
  constructor
  · intro h a b hab
    have : φ.σ.symm a = φ.σ.symm b := by
      refine h _ _ fun n => ?_
      have ha := φ.probe_eq n (φ.σ.symm a)
      have hb := φ.probe_eq n (φ.σ.symm b)
      rw [Equiv.apply_symm_apply] at ha hb
      have : φ.τ (probe H₁ n (φ.σ.symm a)) = φ.τ (probe H₁ n (φ.σ.symm b)) := by
        rw [← ha, ← hb, hab n]
      exact φ.τ.injective this
    simpa using congrArg φ.σ this
  · intro h s s' hss
    have : φ.σ s = φ.σ s' := by
      refine h _ _ fun n => ?_
      rw [φ.probe_eq n s, φ.probe_eq n s', hss n]
    exact φ.σ.injective this

/-- **The converse, under observability.**  If two holons agree on every
    probe and the target is observable, the SAME `σ` and `τ` already form an
    Ω-isomorphism: probe data then determines Ω exactly. -/
def omegaIso_of_probeAgreement {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂}
    (φ : ProbeAgreement H₁ H₂) (hobs : Observable H₂) : OmegaIso H₁ H₂ where
  σ := φ.σ
  τ := φ.τ
  view_eq s := φ.probe_eq 0 s
  dyn_eq s := by
    refine hobs _ _ fun n => ?_
    have h1 : probe H₂ n (H₂.dyn (φ.σ s)) = probe H₂ (n + 1) (φ.σ s) :=
      (probe_succ H₂ n (φ.σ s)).symm
    have h2 : probe H₁ (n + 1) s = probe H₁ n (H₁.dyn s) := probe_succ H₁ n s
    rw [h1, φ.probe_eq (n + 1) s, h2, ← φ.probe_eq n (H₁.dyn s)]

/-- The same statement with the hypothesis on the source holon. -/
def omegaIso_of_probeAgreement' {S₁ S₂ V₁ V₂ : Type}
    {H₁ : Holon S₁ V₁} {H₂ : Holon S₂ V₂}
    (φ : ProbeAgreement H₁ H₂) (hobs : Observable H₁) : OmegaIso H₁ H₂ :=
  omegaIso_of_probeAgreement φ ((observable_transfer φ).mp hobs)

/-! ### The counterexample: |S| = 3, |V| = 2 -/

/-- The shared view: states `0` and `1` read `0`; state `2` reads `1`.
    Surjective, and its only non-trivial fiber is `{0,1}`. -/
def vw : Fin 3 → Fin 2 := fun s => if s.val ≤ 1 then 0 else 1

/-- `H₁`'s dynamics: the identity. -/
def d₁ : Fin 3 → Fin 3 := id

/-- `H₂`'s dynamics: swap the two states inside the fiber, fix the third.
    A bijection, and it holds the view exactly as `d₁` does. -/
def d₂ : Fin 3 → Fin 3 := fun s => if s.val = 0 then 1 else if s.val = 1 then 0 else 2

def H₁ : Holon (Fin 3) (Fin 2) := ⟨vw, d₁⟩
def H₂ : Holon (Fin 3) (Fin 2) := ⟨vw, d₂⟩

/-- A view held by the dynamics is held along every iterate — so both
    holons' probe streams are constant in depth. -/
theorem probe_const_of_held {S V : Type} (H : Holon S V)
    (h : ∀ s, H.view (H.dyn s) = H.view s) (n : ℕ) (s : S) :
    probe H n s = H.view s := by
  induction n generalizing s with
  | zero => rfl
  | succ k ih => rw [probe_succ, ih, h]

theorem held₁ : ∀ s, H₁.view (H₁.dyn s) = H₁.view s := by decide
theorem held₂ : ∀ s, H₂.view (H₂.dyn s) = H₂.view s := by decide

/-- Both dynamics are bijections: the counterexample is REVERSIBLE. -/
theorem d₁_bijective : Function.Bijective d₁ := Function.bijective_id
theorem d₂_bijective : Function.Bijective d₂ := by
  refine Finite.injective_iff_bijective.mp ?_
  decide

/-- Both holons are Closed in the sense of `Omega.Closed`: the view's square
    commutes, with `F = id`.  Closure does not rescue the converse. -/
theorem H₁_closed : Closed H₁ := ⟨id, fun s => held₁ s⟩
theorem H₂_closed : Closed H₂ := ⟨id, fun s => held₂ s⟩

/-- The view is surjective: the counterexample is not the degenerate
    "the view sees nothing" case. -/
theorem vw_surjective : Function.Surjective vw := by decide

/-- **Probe agreement**, with the identity as both witnesses: every probe, at
    every depth, from every state, reads the same in `H₁` and `H₂`. -/
def agreement : ProbeAgreement H₁ H₂ where
  σ := Equiv.refl (Fin 3)
  τ := Equiv.refl (Fin 2)
  probe_eq n s := by
    show probe H₂ n s = probe H₁ n s
    rw [probe_const_of_held H₂ held₂, probe_const_of_held H₁ held₁]
    rfl

/-- Neither holon is observable: states `0` and `1` have identical streams. -/
theorem not_observable₁ : ¬ Observable H₁ := by
  intro h
  have : (0 : Fin 3) = 1 := by
    refine h 0 1 fun n => ?_
    rw [probe_const_of_held H₁ held₁, probe_const_of_held H₁ held₁]
    decide
  exact absurd this (by decide)

/-- **No Ω-isomorphism exists** — and the refutation does not even use the
    view: `dyn_eq` alone is unsatisfiable, for EVERY state bijection `σ` and
    EVERY view bijection `τ`.  Since `H₁.dyn = id`, `dyn_eq` forces every
    state in the image of `σ` — i.e. every state — to be a fixed point of
    `H₂.dyn`, and `H₂.dyn 0 = 1`. -/
theorem no_omegaIso : ¬ Nonempty (OmegaIso H₁ H₂) := by
  rintro ⟨φ⟩
  have key := φ.dyn_eq (φ.σ.symm 0)
  rw [Equiv.apply_symm_apply] at key
  have h₁ : H₁.dyn (φ.σ.symm 0) = φ.σ.symm 0 := rfl
  rw [h₁, Equiv.apply_symm_apply] at key
  exact absurd key (by decide)

/-- **RUNG 3, the hard half: representational completeness is FALSE.**
    There are two finite holons that agree on every admissible probe and are
    not Ω-isomorphic. -/
theorem probeAgreement_not_implies_omegaIso :
    ∃ (S₁ S₂ V₁ V₂ : Type) (H₁ : Holon S₁ V₁) (H₂ : Holon S₂ V₂),
      Nonempty (ProbeAgreement H₁ H₂) ∧ ¬ Nonempty (OmegaIso H₁ H₂) :=
  ⟨Fin 3, Fin 3, Fin 2, Fin 2, H₁, H₂, ⟨agreement⟩, no_omegaIso⟩

/-- …and the same pair witnesses that observability is exactly what was
    missing: it fails here, and by `omegaIso_of_probeAgreement` it is
    sufficient. -/
theorem counterexample_is_non_observable :
    ¬ Observable H₁ ∧ Nonempty (ProbeAgreement H₁ H₂) ∧
      ¬ Nonempty (OmegaIso H₁ H₂) :=
  ⟨not_observable₁, ⟨agreement⟩, no_omegaIso⟩

end CIRISHolon.ProbeConverse
