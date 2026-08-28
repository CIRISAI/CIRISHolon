/-
CIRISHolon.CosetRandomizer — OMEGA-CIRCUITS-1's structure theorem at its
smallest nontrivial instances.

OMEGA-CIRCUITS-1 (`conformance/omega/OMEGA_CIRCUITS1_RESULTS.md`, prereg
Theorems C1/C2) derived, before its instrument ran, that a Clifford Born kernel
is a COSET RANDOMIZER — `K(s'|s) = 2^{−dim V}·1[s' ∈ c₀ + As + V]` for one
subspace `V` shared by every `s` — and that on an `F₂`-linear view this forces
`λ ∈ {0, 1}` and quantizes the rent onto the dyadic ladder `W = 1 − 2^{−h}`.
That is `M-RING-MIXING` one ring scale down: at scale √3 a `Z[ω]` circulant is
permutation-or-maximal (`CIRISHolon.RingMixing`), at scale √2 a Clifford kernel
is permutation-or-randomizing.  The consequence the campaign had to record is
that the frozen headline form CANNOT be tested at intermediate `λ` anywhere in
the (Clifford step, linear view) sector — the sector has no intermediate `λ`.

SCOPE, honestly and narrowly.  This file proves the structure theorem at ONE
qubit for `H` and at TWO qubits for `CX`, with the FULL basis view, by
exhaustion over the Born table with exact integer amplitudes.  It is NOT the
general theorem: arbitrary `n`, arbitrary Clifford `U`, and arbitrary `F₂`-linear
`B` are NOT covered, and the Dehaene–De Moor / Van den Nest normal form that
proves C1 in general is credited and NOT mechanized.  What the two instances do
carry is both endpoints of the dichotomy — `H` is the `h = m` end (`λ = 0`,
`W = 1/2`), `CX` is the `h = 0` end (`λ = 1`, `W = 0`) — so the claim "there is
nothing between" has its two ends pinned by machine here and its middle owed.

One further fence.  `λ = 0` is mechanized in full: the transfer matrix IS the
uniform projector, checked entry by entry.  `λ = 1` is mechanized only up to an
operator-norm step: an explicit mean-zero eigenvector of eigenvalue 1 is
exhibited and checked, which gives `λ ≥ 1`; `λ ≤ 1` follows from double
stochasticity (also checked here, as row and column sums) by an argument in
`ℓ²(μ)` that is NOT carried out in Lean.  The header says so rather than the
theorem names implying otherwise.

CONVENTIONS.  A basis state of `n` qubits is `Fin (2^n)` with the bit string
read big-endian (`q₀` most significant).  Amplitudes are exact integers carrying
a global `2^{−e/2}`, as the instrument's `Cyc` ring does; the Born weight is the
squared amplitude over `2^e`, so `K = num / den` with `den = 2^e`.  For both
instances `den = |V|`, which is `2^{dim V}` — Theorem C1's normalization.
-/
import Mathlib.Tactic

namespace CIRISHolon.CosetRandomizer

/-! ### `F₂ⁿ` at n = 1 and n = 2 -/

/-- Bitwise xor on `F₂`. -/
def xor2 : Fin 2 → Fin 2 → Fin 2 := ![![0, 1], ![1, 0]]

/-- Bitwise xor on `F₂²`, states encoded as `2·q₀ + q₁`. -/
def xor4 : Fin 4 → Fin 4 → Fin 4 :=
  ![![0, 1, 2, 3], ![1, 0, 3, 2], ![2, 3, 0, 1], ![3, 2, 1, 0]]

theorem xor2_group :
    (∀ x, xor2 x 0 = x) ∧ (∀ x, xor2 x x = 0) ∧
      (∀ x y z, xor2 (xor2 x y) z = xor2 x (xor2 y z)) := by decide

theorem xor4_group :
    (∀ x, xor4 x 0 = x) ∧ (∀ x, xor4 x x = 0) ∧
      (∀ x y z, xor4 (xor4 x y) z = xor4 x (xor4 y z)) := by decide

/-! ### Instance 1 — the Hadamard on one qubit -/

/-- `⟨s'|H|s⟩` in exact integers, carrying a global `2^{−1/2}`. -/
def hAmp : Fin 2 → Fin 2 → ℤ := ![![1, 1], ![1, -1]]

/-- The Born numerator `|⟨s'|H|s⟩|²·2`. -/
def hBornNum (s' s : Fin 2) : ℤ := hAmp s' s * hAmp s' s

/-- The Born denominator: `2^e` for the amplitude scale `2^{−e/2}`, `e = 1`. -/
def hBornDen : ℕ := 2

/-- G0, the double-stochasticity check the campaign runs before any reading:
    exact row AND column sums. -/
theorem hBorn_doubly_stochastic :
    (∀ s, hBornNum 0 s + hBornNum 1 s = (hBornDen : ℤ)) ∧
      (∀ s', hBornNum s' 0 + hBornNum s' 1 = (hBornDen : ℤ)) := by decide

/-- `H`'s randomizing subspace: all of `F₂`, so `dim V = 1`. -/
def V_H : Fin 2 → Bool := fun _ => true

/-- `H`'s `X`-block: the identity (`H X H† = Z`, so the `X` support is not
    translated). -/
def A_H : Fin 2 → Fin 2 := id

theorem V_H_subgroup :
    V_H 0 = true ∧ ∀ x y, V_H x = true → V_H y = true → V_H (xor2 x y) = true := by decide

theorem A_H_linear (x y : Fin 2) : A_H (xor2 x y) = xor2 (A_H x) (A_H y) := by
  revert x y; decide

/-- **Theorem C1 at one qubit.**  The Born kernel of `H` is supported on, and
    constant on, the coset `A_H s + V_H`, with value `1/|V_H|`. -/
theorem hBorn_coset_randomizer (s' s : Fin 2) :
    hBornNum s' s = (if V_H (xor2 s' (A_H s)) then 1 else 0) := by
  revert s' s; decide

/-- The normalization is Theorem C1's: `den = |V| = 2^{dim V}`. -/
theorem hBornDen_eq_card :
    hBornDen = (Finset.univ.filter (fun x => V_H x = true)).card := by decide

/-- Constancy on cosets, stated directly: two outputs differing by an element of
    `V` carry the same weight. -/
theorem hBorn_constant_on_cosets (s x y : Fin 2) (h : V_H (xor2 x y) = true) :
    hBornNum x s = hBornNum y s := by revert s x y; decide

/-! ### Instance 2 — CNOT on two qubits -/

/-- `CX` with control `q₀`: `(x₀, x₁) ↦ (x₀, x₀ ⊕ x₁)`, an `F₂`-linear
    permutation of the basis. -/
def cxMap : Fin 4 → Fin 4 := ![0, 1, 3, 2]

theorem cxMap_linear (x y : Fin 4) : cxMap (xor4 x y) = xor4 (cxMap x) (cxMap y) := by
  revert x y; decide

theorem cxMap_bijective : Function.Bijective cxMap := by
  refine Finite.injective_iff_bijective.mp ?_
  decide

/-- `⟨s'|CX|s⟩`: a permutation unitary, amplitudes `0` or `1`, global scale 1. -/
def cxAmp (s' s : Fin 4) : ℤ := if s' = cxMap s then 1 else 0

def cxBornNum (s' s : Fin 4) : ℤ := cxAmp s' s * cxAmp s' s

def cxBornDen : ℕ := 1

theorem cxBorn_doubly_stochastic :
    (∀ s, cxBornNum 0 s + cxBornNum 1 s + cxBornNum 2 s + cxBornNum 3 s
        = (cxBornDen : ℤ)) ∧
      (∀ s', cxBornNum s' 0 + cxBornNum s' 1 + cxBornNum s' 2 + cxBornNum s' 3
        = (cxBornDen : ℤ)) := by decide

/-- `CX`'s randomizing subspace: trivial, so `dim V = 0` — the kernel is
    deterministic. -/
def V_CX : Fin 4 → Bool := fun x => x = 0

theorem V_CX_subgroup :
    V_CX 0 = true ∧ ∀ x y, V_CX x = true → V_CX y = true → V_CX (xor4 x y) = true := by
  decide

/-- **Theorem C1 at two qubits.**  Same shape, opposite end of the ladder: the
    coset is a single point. -/
theorem cxBorn_coset_randomizer (s' s : Fin 4) :
    cxBornNum s' s = (if V_CX (xor4 s' (cxMap s)) then 1 else 0) := by
  revert s' s; decide

theorem cxBornDen_eq_card :
    cxBornDen = (Finset.univ.filter (fun x => V_CX x = true)).card := by decide

theorem cxBorn_constant_on_cosets (s x y : Fin 4) (h : V_CX (xor4 x y) = true) :
    cxBornNum x s = cxBornNum y s := by revert s x y; decide

/-! ### The two ends of the λ dichotomy, on the full basis view -/

/-- **`λ = 0` at one qubit, in full.**  With the full view, `B = id`, so
    `ker B = 0` and `h = dim V = 1 = m`.  Theorem C2 then says `M = Π`, and here
    that is checked entry by entry: every transfer entry is `1/2`. -/
theorem H_transfer_is_uniform (x y : Fin 2) : hBornNum y x = 1 := by revert x y; decide

/-- The eigenvector exhibiting `λ ≥ 1` for `CX`: mean zero under uniform `μ`,
    nonzero, and fixed by the transfer, because `(Mf)(x) = f(cxMap x)`.  With
    `h = 0 < m = 2`, this is Theorem C2's `λ = 1` branch.  The remaining step —
    `λ ≤ 1` from double stochasticity, in `ℓ²(μ)` — is NOT mechanized. -/
def cxWitness : Fin 4 → ℤ := ![1, -1, 0, 0]

theorem cxWitness_mean_zero :
    cxWitness 0 + cxWitness 1 + cxWitness 2 + cxWitness 3 = 0 := by decide

theorem cxWitness_ne_zero : cxWitness 0 ≠ 0 := by decide

theorem cxWitness_fixed (x : Fin 4) : cxWitness (cxMap x) = cxWitness x := by
  revert x; decide

/-- The transfer really does act by precomposition with `cxMap`, which is what
    makes the witness above an eigenvector. -/
theorem cx_transfer_apply (x : Fin 4) :
    cxBornNum 0 x * cxWitness 0 + cxBornNum 1 x * cxWitness 1
      + cxBornNum 2 x * cxWitness 2 + cxBornNum 3 x * cxWitness 3
      = cxWitness (cxMap x) := by revert x; decide

/-! ### The dyadic rent ladder at both ends -/

def hRowMax (x : Fin 2) : ℤ := max (hBornNum 0 x) (hBornNum 1 x)

theorem hRowMax_sum : hRowMax 0 + hRowMax 1 = 2 := by decide

/-- Rent `W = 1 − Σ_x max_y P_{x,y}` with `P_{x,y} = μ_x·K(y|x)`, `μ` uniform on
    `2` states and `K = num / den`. -/
def hRent : ℚ := 1 - ((hRowMax 0 + hRowMax 1 : ℤ) : ℚ) / (2 * (hBornDen : ℚ))

/-- **`W = 1 − 2^{−h}` at `h = 1`.** -/
theorem hRent_eq : hRent = 1 - 1 / (2 : ℚ) ^ (1 : ℕ) := by
  unfold hRent; rw [hRowMax_sum]; norm_num [hBornDen]

def cxRowMax (x : Fin 4) : ℤ :=
  max (max (cxBornNum 0 x) (cxBornNum 1 x)) (max (cxBornNum 2 x) (cxBornNum 3 x))

theorem cxRowMax_sum : cxRowMax 0 + cxRowMax 1 + cxRowMax 2 + cxRowMax 3 = 4 := by decide

def cxRent : ℚ :=
  1 - ((cxRowMax 0 + cxRowMax 1 + cxRowMax 2 + cxRowMax 3 : ℤ) : ℚ)
        / (4 * (cxBornDen : ℚ))

/-- **`W = 1 − 2^{−h}` at `h = 0`.** -/
theorem cxRent_eq : cxRent = 1 - 1 / (2 : ℚ) ^ (0 : ℕ) := by
  unfold cxRent; rw [cxRowMax_sum]; norm_num [cxBornDen]

/-- The two instances, side by side: same coset shape, the two ends of the
    dyadic ladder, and `λ` pinned at `0` and (up to the un-mechanized
    operator-norm half) at `1`.  Nothing here rules out an intermediate `λ`
    outside the (Clifford, linear) sector — that is where the campaign's S3
    confirmations live. -/
theorem two_ends :
    (∀ s' s, hBornNum s' s = (if V_H (xor2 s' (A_H s)) then 1 else 0)) ∧
      hRent = 1 / 2 ∧
      (∀ s' s, cxBornNum s' s = (if V_CX (xor4 s' (cxMap s)) then 1 else 0)) ∧
      cxRent = 0 := by
  refine ⟨hBorn_coset_randomizer, ?_, cxBorn_coset_randomizer, ?_⟩
  · rw [hRent_eq]; norm_num
  · rw [cxRent_eq]; norm_num

end CIRISHolon.CosetRandomizer
