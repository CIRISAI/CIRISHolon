/-
CIRISHolon.RingMixing — LOCAL-1C's kill (`M-RING-MIXING`), mechanized: a
finite exhaustion plus a proof that the finite box is all there is.

LOCAL-1 (`conformance/gravity/LOCAL1_RESULTS.md`) read `R ≡ 0` twice.  The
second time the cause was not the carrier and not the probe: the electric term
itself was a MAXIMALLY-MIXING unitary, so one application thermalized the flux
distribution and the propagation observable saturated before it could move.
The registered lesson, `M-RING-MIXING`, is that a ring-scale constraint can
FORCE that: at scale √3 a `Z[ω]` circulant unitary is permutation-phase or
maximal-mixing and nothing in between, so a propagation probe needs a weak
coupler, which may only exist at a higher ring scale.

WHAT IS PROVED HERE, EXACTLY.  `Z[ω]` is the Eisenstein ring, `ω` a primitive
cube root of unity, `ω² = −1 − ω`; an element `a + bω` is the pair `(a, b)` and
its squared modulus is `a² − ab + b²`.  A circulant `(c₀ + c₁L + c₂L²)/√3` on
three flux states is unitary exactly when its three eigenvalues
`c₀ + c₁ω^k + c₂ω^{2k}` all have squared modulus 3.

`ring_dichotomy` is the statement, and it is UNCONDITIONAL over `Z[ω]`:
**every scale-√3 unitary circulant triple is either permutation-phase (one
coefficient, of squared modulus 3, the other two zero) or maximal-mixing (all
three of squared modulus 1) — there is nothing in between.**  It reaches all of
the infinite ring in two steps, both machine-checked:

* the search box is FORCED, not chosen.  `parseval` is the polynomial identity
  `Σ_k |λ_k|² = 3·Σ_i |c_i|²` (proved by `ring`, no bound anywhere), so the
  hypothesis pins `Σ_i |c_i|² = 3`; with `znorm_nonneg` each coefficient has
  squared modulus at most 3, and `znorm_le_three_box` turns that into
  `|a| ≤ 2, |b| ≤ 2`.  No unitary triple lives outside the box.
* inside the box, `ring_dichotomy_box` decides all 25³ = 15625 triples by
  exhaustion.

So the finite core is 15625 cases and the reduction to it is a proof, not a
stipulation.  What is NOT claimed: anything about other scales (see the two
examples below), and anything about non-circulant or higher-dimensional
unitaries.

Two CHECKED EXAMPLES at higher scales, both showing the dichotomy is a property
of scale √3 and not of the ring:

* `weakCoupler27` — LOCAL-1D's actual escape, `c = (5+4ω, 2+ω, −1−2ω)` at
  scale 3√3: eigenvalue norms all 27, coefficient norms 21/3/3.  Neither
  branch: more than one nonzero, and the norms are unequal.
* `scale3_counterexample` — **a CORRECTION to the record.**  LOCAL-1's prose
  says "At scale 3 only trivial circulants exist."  It does not:
  `c = (−2−2ω, −2−2ω, 1+ω)` has eigenvalue norms all 9, so it is unitary at
  scale 3, and its coefficient norms are 4/4/1 — a genuine weak coupler one
  ring scale BELOW the one the campaign went looking for.  Under the reading
  used here (scale `s` means all three eigenvalue norms equal `s²`) the
  parenthetical is false; if the campaign's scale-3 search carried a further
  constraint, that constraint is what the record owes.
-/
import Mathlib.Tactic

set_option maxRecDepth 100000
set_option maxHeartbeats 4000000

namespace CIRISHolon.RingMixing

/-! ### `Z[ω]`, exactly -/

/-- An Eisenstein integer `a + bω`, `ω` a primitive cube root of unity. -/
abbrev ZW := ℤ × ℤ

def zadd (x y : ZW) : ZW := (x.1 + y.1, x.2 + y.2)

/-- Multiplication, reduced by `ω² = −1 − ω`. -/
def zmul (x y : ZW) : ZW := (x.1 * y.1 - x.2 * y.2, x.1 * y.2 + x.2 * y.1 - x.2 * y.2)

/-- The squared modulus (the field norm) `|a + bω|² = a² − ab + b²`. -/
def znorm (x : ZW) : ℤ := x.1 * x.1 - x.1 * x.2 + x.2 * x.2

def w : ZW := (0, 1)
def w2 : ZW := (-1, -1)

theorem w_cubed : zmul w w = w2 ∧ zmul w w2 = (1, 0) := by decide

/-! ### The three eigenvalues of a circulant triple -/

def eig0 (c0 c1 c2 : ZW) : ZW := zadd (zadd c0 c1) c2
def eig1 (c0 c1 c2 : ZW) : ZW := zadd (zadd c0 (zmul c1 w)) (zmul c2 w2)
def eig2 (c0 c1 c2 : ZW) : ZW := zadd (zadd c0 (zmul c1 w2)) (zmul c2 w)

/-- The triple is unitary at scale `√t`: every eigenvalue has squared modulus
    `t`.  `t = 3` is LOCAL-1's electric term. -/
def UnitaryAt (t : ℤ) (c0 c1 c2 : ZW) : Prop :=
  znorm (eig0 c0 c1 c2) = t ∧ znorm (eig1 c0 c1 c2) = t ∧ znorm (eig2 c0 c1 c2) = t

instance (t : ℤ) (c0 c1 c2 : ZW) : Decidable (UnitaryAt t c0 c1 c2) := by
  unfold UnitaryAt; infer_instance

/-- Permutation-phase: exactly one nonzero coefficient, necessarily of squared
    modulus 3.  The flux distribution is relabelled, never spread. -/
def PermPhase (c0 c1 c2 : ZW) : Prop :=
  (znorm c0 = 3 ∧ c1 = ((0 : ℤ), (0 : ℤ)) ∧ c2 = ((0 : ℤ), (0 : ℤ))) ∨
  (c0 = ((0 : ℤ), (0 : ℤ)) ∧ znorm c1 = 3 ∧ c2 = ((0 : ℤ), (0 : ℤ))) ∨
  (c0 = ((0 : ℤ), (0 : ℤ)) ∧ c1 = ((0 : ℤ), (0 : ℤ)) ∧ znorm c2 = 3)

/-- Maximal mixing: all three coefficients of equal (unit) squared modulus, so
    one application spreads the flux uniformly. -/
def MaxMixing (c0 c1 c2 : ZW) : Prop :=
  znorm c0 = 1 ∧ znorm c1 = 1 ∧ znorm c2 = 1

instance (c0 c1 c2 : ZW) : Decidable (PermPhase c0 c1 c2) := by
  unfold PermPhase; infer_instance

instance (c0 c1 c2 : ZW) : Decidable (MaxMixing c0 c1 c2) := by
  unfold MaxMixing; infer_instance

/-- The two branches are exclusive — the dichotomy is a genuine fork, not two
    names for one set.  (General, not box-bounded.) -/
theorem permPhase_not_maxMixing (c0 c1 c2 : ZW) (hp : PermPhase c0 c1 c2) :
    ¬ MaxMixing c0 c1 c2 := by
  rintro ⟨k0, k1, _⟩
  rcases hp with ⟨_, h, _⟩ | ⟨h, _, _⟩ | ⟨h, _, _⟩
  · rw [h] at k1; simp [znorm] at k1
  · rw [h] at k0; simp [znorm] at k0
  · rw [h] at k0; simp [znorm] at k0

/-! ### The search box -/

/-- The coefficient box: `a, b ∈ {−2, −1, 0, 1, 2}`. -/
def boxVal : Fin 5 → ℤ := ![-2, -1, 0, 1, 2]

def coef (p : Fin 5 × Fin 5) : ZW := (boxVal p.1, boxVal p.2)

/-- The finite core: every scale-√3 unitary `Z[ω]` circulant with coefficients
    in `|a|, |b| ≤ 2` is permutation-phase or maximal-mixing.  15625 triples,
    decided by exhaustion. -/
theorem ring_dichotomy_box (p q r : Fin 5 × Fin 5)
    (h : UnitaryAt 3 (coef p) (coef q) (coef r)) :
    PermPhase (coef p) (coef q) (coef r) ∨ MaxMixing (coef p) (coef q) (coef r) := by
  revert p q r; decide

/-! ### The box is forced: Parseval plus positivity of the norm form -/

/-- **Parseval for the 3-point DFT, over `Z[ω]` and exactly**: the eigenvalue
    norms sum to three times the coefficient norms.  A polynomial identity in
    six integer variables — no bound is used, which is what lets the box
    restriction be discharged rather than assumed. -/
theorem parseval (c0 c1 c2 : ZW) :
    znorm (eig0 c0 c1 c2) + znorm (eig1 c0 c1 c2) + znorm (eig2 c0 c1 c2)
      = 3 * (znorm c0 + znorm c1 + znorm c2) := by
  simp only [znorm, eig0, eig1, eig2, zadd, zmul, w, w2]; ring

/-- The norm form is positive semidefinite: `4(a² − ab + b²) = (2a − b)² + 3b²`. -/
theorem znorm_nonneg (x : ZW) : 0 ≤ znorm x := by
  have h := sq_nonneg (2 * x.1 - x.2)
  have h2 := sq_nonneg x.2
  simp only [znorm]; nlinarith [h, h2]

/-- Squared modulus at most 3 confines an Eisenstein integer to the box. -/
theorem znorm_le_three_box (x : ZW) (h : znorm x ≤ 3) :
    -2 ≤ x.1 ∧ x.1 ≤ 2 ∧ -2 ≤ x.2 ∧ x.2 ≤ 2 := by
  simp only [znorm] at h
  refine ⟨?_, ?_, ?_, ?_⟩ <;> nlinarith [sq_nonneg (2 * x.1 - x.2), sq_nonneg x.2,
    sq_nonneg (2 * x.2 - x.1), sq_nonneg x.1, sq_nonneg (x.1 + x.2)]

theorem exists_box (a : ℤ) (h : -2 ≤ a) (h' : a ≤ 2) : ∃ i : Fin 5, boxVal i = a := by
  interval_cases a
  exacts [⟨0, rfl⟩, ⟨1, rfl⟩, ⟨2, rfl⟩, ⟨3, rfl⟩, ⟨4, rfl⟩]

theorem exists_coef (x : ZW) (h : znorm x ≤ 3) : ∃ p : Fin 5 × Fin 5, coef p = x := by
  obtain ⟨h1, h2, h3, h4⟩ := znorm_le_three_box x h
  obtain ⟨i, hi⟩ := exists_box x.1 h1 h2
  obtain ⟨j, hj⟩ := exists_box x.2 h3 h4
  exact ⟨(i, j), by simp [coef, hi, hj]⟩

/-- Unitarity at scale √3 pins the total coefficient weight to exactly 3. -/
theorem coef_norm_sum (c0 c1 c2 : ZW) (h : UnitaryAt 3 c0 c1 c2) :
    znorm c0 + znorm c1 + znorm c2 = 3 := by
  obtain ⟨h0, h1, h2⟩ := h
  have hp := parseval c0 c1 c2
  rw [h0, h1, h2] at hp
  omega

theorem coef_norm_le (c0 c1 c2 : ZW) (h : UnitaryAt 3 c0 c1 c2) :
    znorm c0 ≤ 3 ∧ znorm c1 ≤ 3 ∧ znorm c2 ≤ 3 := by
  have hs := coef_norm_sum c0 c1 c2 h
  have n0 := znorm_nonneg c0
  have n1 := znorm_nonneg c1
  have n2 := znorm_nonneg c2
  omega

/-- **`M-RING-MIXING`, machine-checked, unconditionally.**  EVERY scale-√3
    unitary `Z[ω]` circulant — no bound on the coefficients — is
    permutation-phase or maximal-mixing.  There is no weak coupler at this
    scale, which is why LOCAL-1C's propagation probe read zero: the only
    alternative to relabelling was thermalizing in one step. -/
theorem ring_dichotomy (c0 c1 c2 : ZW) (h : UnitaryAt 3 c0 c1 c2) :
    PermPhase c0 c1 c2 ∨ MaxMixing c0 c1 c2 := by
  obtain ⟨l0, l1, l2⟩ := coef_norm_le c0 c1 c2 h
  obtain ⟨p, hp⟩ := exists_coef c0 l0
  obtain ⟨q, hq⟩ := exists_coef c1 l1
  obtain ⟨r, hr⟩ := exists_coef c2 l2
  subst hp; subst hq; subst hr
  exact ring_dichotomy_box p q r h

/-- Both branches are occupied — the theorem is not vacuously true and not
    secretly one-sided. -/
theorem permPhase_realized :
    UnitaryAt 3 (1, -1) (0, 0) (0, 0) ∧ PermPhase (1, -1) (0, 0) (0, 0) := by decide

theorem maxMixing_realized :
    UnitaryAt 3 (-1, -1) (-1, -1) (0, 1) ∧ MaxMixing (-1, -1) (-1, -1) (0, 1) := by decide

/-! ### The escapes, one and two scales up -/

/-- LOCAL-1D's weak coupler at scale 3√3: unitary, and in NEITHER branch —
    diagonal weight 21/27, hopping 3/27 each.  The escape the campaign needed
    exists, one scale up. -/
theorem weakCoupler27 :
    UnitaryAt 27 (5, 4) (2, 1) (-1, -2) ∧
      znorm (5, 4) = 21 ∧ znorm (2, 1) = 3 ∧ znorm (-1, -2) = 3 := by decide

theorem weakCoupler27_neither :
    ¬ PermPhase (5, 4) (2, 1) (-1, -2) ∧ ¬ MaxMixing (5, 4) (2, 1) (-1, -2) := by decide

/-- **The correction.**  LOCAL-1's prose says only trivial circulants exist at
    scale 3.  Here is a nontrivial one: unitary (all eigenvalue norms 9), with
    coefficient norms 4/4/1 — moduli 2/3, 2/3, 1/3 — so it is neither
    permutation-phase nor maximal-mixing.  A weak coupler exists one ring scale
    below the one LOCAL-1D adopted. -/
theorem scale3_counterexample :
    UnitaryAt 9 (-2, -2) (-2, -2) (1, 1) ∧
      znorm (-2, -2) = 4 ∧ znorm (1, 1) = 1 ∧
      ¬ PermPhase (-2, -2) (-2, -2) (1, 1) ∧
      ¬ MaxMixing (-2, -2) (-2, -2) (1, 1) := by decide

end CIRISHolon.RingMixing
