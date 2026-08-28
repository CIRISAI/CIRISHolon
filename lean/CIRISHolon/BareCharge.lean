/-
CIRISHolon.BareCharge — BRIDGE-5's kill, mechanized at minimal scale.

BRIDGE-5 (`conformance/gravity/BRIDGE5_RESULTS.md`) was VOIDed by one fact:
**Gauss acts INDEPENDENTLY at each vertex**, and `Σ_g ρ₂(g) = 0` by irrep
orthogonality, so the per-vertex projector annihilates every tensor component
that is not invariant under the SINGLE-vertex action.  The bare pair singlet
`|00⟩+|11⟩` is invariant under the DIAGONAL action `ρ₂(g) ⊗ ρ₂(g)` — which is
what made it look like a gauge-invariant channel — and is nevertheless killed,
because Gauss never applies the diagonal action.  The successor's correction
(M14, `M-BARE-CHARGE`) is that a charged pair is gauge-invariant only when
DRESSED BY A WILSON LINE: the physical state is `Σ_ij ρ₂(U_γ)_ij |ij⟩`, and the
dressing survives the very same average.

SCOPE, honestly.  This is the MINIMAL model that carries the argument: two
matter sites joined by ONE edge, gauge group D₄, matter in the 2-dimensional
irrep `ρ₂`, amplitudes exact integers.  It is NOT BRIDGE-5's lattice, NOT its
Floquet dynamics, and NOT a statement about the physical instrument's other
gates.  What is machine-checked here is exactly the mechanism that killed
BRIDGE-5 and exactly the mechanism by which M14's dressing escapes it:

* `sum_rho2`               — `Σ_{g ∈ D₄} ρ₂(g) = 0` (irrep orthogonality, by
                             exhaustion over the eight group elements)
* `bare_singlet_diagonally_invariant`
                           — the trap: the bare singlet IS fixed by the
                             diagonal action, at every group element
* `gauss1_bare`            — the kill, and it is universal in the tensor: the
                             one-vertex Gauss average annihilates EVERY bare
                             (configuration-independent) matter tensor, the
                             singlet among them
* `gaussBoth_bare`         — hence the two-site average is zero as well
* `dressed_invariant_v1` / `dressed_invariant_v2`
                           — the Wilson-dressed pair is fixed by the gauge
                             action at EACH vertex separately
* `gauss1_dressed`, `gaussBoth_dressed`, `gaussBoth_dressed_ne_zero`
                           — so the two-site average returns it undamaged,
                             scaled by |D₄|² = 64, and is nonzero

Convention.  A configuration is the single edge's group element `u ∈ D₄`.  A
state assigns to each configuration a 2×2 integer matrix `Ψ u`, whose `(i,j)`
entry is the amplitude of the spinor pair `|ij⟩` (site 1 carries `i`, site 2
carries `j`).  The gauge action at site 1 by `g` sends `u ↦ g·u` and rotates
the first index: `(g·Ψ)(u) = ρ₂(g) · Ψ(g⁻¹u)`.  At site 2 by `g` it sends
`u ↦ u·g⁻¹` and rotates the second: `(g·Ψ)(u) = Ψ(u·g) · ρ₂(g⁻¹)`.  The Gauss
projector at a site is the unnormalized average of that action over the group.
D₄ and `ρ₂` are BRIDGE-1's own tables (`conformance/gravity/bridge1.py`):
`g = k + 4b` denotes `r^k s^b`, and `ρ₂(r^k s^b) = R^k S^b`.
-/
import Mathlib.Tactic

namespace CIRISHolon.BareCharge

/-! ### D₄, exactly as the instrument encodes it -/

/-- `g = k + 4b` is `r^k s^b`; the product is BRIDGE-1's `MUL` table. -/
private def mulN (a b : ℕ) : ℕ :=
  ((a % 4 + (if (a / 4) % 2 = 0 then b % 4 else (4 - b % 4) % 4)) % 4)
    + 4 * ((a / 4 + b / 4) % 2)

/-- The D₄ product. -/
def dmul (a b : Fin 8) : Fin 8 := ⟨mulN a.val b.val % 8, Nat.mod_lt _ (by norm_num)⟩

/-- The D₄ inverse: rotations invert, reflections are involutions. -/
def dinv (a : Fin 8) : Fin 8 := if a.val < 4 then ⟨(4 - a.val) % 4, by omega⟩ else a

theorem dmul_assoc (a b c : Fin 8) : dmul (dmul a b) c = dmul a (dmul b c) := by
  revert a b c; decide

theorem dmul_one (a : Fin 8) : dmul a 0 = a := by revert a; decide

theorem one_dmul (a : Fin 8) : dmul 0 a = a := by revert a; decide

theorem dmul_dinv (a : Fin 8) : dmul a (dinv a) = 0 := by revert a; decide

theorem dinv_dmul (a : Fin 8) : dmul (dinv a) a = 0 := by revert a; decide

/-! ### 2×2 integer matrices — the carrier of `ρ₂` and of the matter tensor -/

/-- A 2×2 integer matrix `⟨a, b; c, d⟩`.  Kept as a bare structure (not
    `Matrix`) so that every statement below is settled by `decide`. -/
structure M2 where
  a : ℤ
  b : ℤ
  c : ℤ
  d : ℤ
deriving DecidableEq, Repr

instance : Add M2 := ⟨fun x y => ⟨x.a + y.a, x.b + y.b, x.c + y.c, x.d + y.d⟩⟩
instance : Mul M2 :=
  ⟨fun x y => ⟨x.a * y.a + x.b * y.c, x.a * y.b + x.b * y.d,
               x.c * y.a + x.d * y.c, x.c * y.b + x.d * y.d⟩⟩
instance : Zero M2 := ⟨⟨0, 0, 0, 0⟩⟩

@[simp] theorem add_a (x y : M2) : (x + y).a = x.a + y.a := rfl
@[simp] theorem add_b (x y : M2) : (x + y).b = x.b + y.b := rfl
@[simp] theorem add_c (x y : M2) : (x + y).c = x.c + y.c := rfl
@[simp] theorem add_d (x y : M2) : (x + y).d = x.d + y.d := rfl
@[simp] theorem mul_a (x y : M2) : (x * y).a = x.a * y.a + x.b * y.c := rfl
@[simp] theorem mul_b (x y : M2) : (x * y).b = x.a * y.b + x.b * y.d := rfl
@[simp] theorem mul_c (x y : M2) : (x * y).c = x.c * y.a + x.d * y.c := rfl
@[simp] theorem mul_d (x y : M2) : (x * y).d = x.c * y.b + x.d * y.d := rfl
@[simp] theorem zero_a : (0 : M2).a = 0 := rfl
@[simp] theorem zero_b : (0 : M2).b = 0 := rfl
@[simp] theorem zero_c : (0 : M2).c = 0 := rfl
@[simp] theorem zero_d : (0 : M2).d = 0 := rfl

theorem M2.ext' {x y : M2} (ha : x.a = y.a) (hb : x.b = y.b)
    (hc : x.c = y.c) (hd : x.d = y.d) : x = y := by
  cases x; cases y; simp_all

/-- Integer scaling, used only to state the size of an unnormalized average. -/
def scale (n : ℤ) (x : M2) : M2 := ⟨n * x.a, n * x.b, n * x.c, n * x.d⟩

/-- The unnormalized group average of a group-indexed family. -/
def sum8 (f : Fin 8 → M2) : M2 := f 0 + f 1 + f 2 + f 3 + f 4 + f 5 + f 6 + f 7

theorem sum8_mul_right (f : Fin 8 → M2) (T : M2) :
    sum8 (fun g => f g * T) = sum8 f * T := by
  apply M2.ext' <;> simp [sum8] <;> ring

theorem zero_mul' (T : M2) : (0 : M2) * T = 0 := by apply M2.ext' <;> simp

theorem sum8_zero : sum8 (fun _ => (0 : M2)) = 0 := by apply M2.ext' <;> simp [sum8]

/-! ### The 2-dimensional irrep -/

/-- `ρ₂(r^k s^b) = R^k S^b` with `R = ⟨0,-1;1,0⟩`, `S = ⟨1,0;0,-1⟩` —
    BRIDGE-1's `rho2`, tabulated. -/
def rho2 : Fin 8 → M2 :=
  ![⟨1, 0, 0, 1⟩, ⟨0, -1, 1, 0⟩, ⟨-1, 0, 0, -1⟩, ⟨0, 1, -1, 0⟩,
    ⟨1, 0, 0, -1⟩, ⟨0, 1, 1, 0⟩, ⟨-1, 0, 0, 1⟩, ⟨0, -1, -1, 0⟩]

/-- `ρ₂` really is a representation of the tabulated group. -/
theorem rho2_hom (g h : Fin 8) : rho2 (dmul g h) = rho2 g * rho2 h := by
  revert g h; decide

/-- **Irrep orthogonality at minimal scale**: the group sum of a nontrivial
    irrep vanishes.  This single line is what killed BRIDGE-5. -/
theorem sum_rho2 : sum8 rho2 = 0 := by decide

/-! ### States, the per-vertex Gauss projectors, and the two carriers -/

/-- A state: an amplitude matrix per edge configuration.  `(Ψ u).a` is the
    amplitude of `|00⟩` at configuration `u`, `.b` of `|01⟩`, and so on. -/
abbrev State := Fin 8 → M2

/-- Gauss average at site 1: `u ↦ g·u`, first spinor index rotated. -/
def gauss1 (Ψ : State) : State := fun u => sum8 (fun g => rho2 g * Ψ (dmul (dinv g) u))

/-- Gauss average at site 2: `u ↦ u·g⁻¹`, second spinor index rotated. -/
def gauss2 (Ψ : State) : State := fun u => sum8 (fun g => Ψ (dmul u g) * rho2 (dinv g))

/-- The joint (two-site) Gauss average — the projector BRIDGE-5 actually
    applied, and the one whose independence at the two vertices is the point. -/
def gaussBoth (Ψ : State) : State := gauss2 (gauss1 Ψ)

/-- A BARE tensor: the same matter matrix at every configuration, carrying no
    gauge field.  BRIDGE-5's singlet is `bare ⟨1,0,0,1⟩`. -/
def bare (T : M2) : State := fun _ => T

/-- The WILSON-DRESSED pair of M14: the amplitude at configuration `u` is
    `ρ₂(u)` itself, so the gauge field carries the invariance. -/
def dressed : State := rho2

/-! ### The trap, the kill, and the escape -/

/-- **The trap.**  The bare singlet is invariant under the DIAGONAL action
    `T ↦ ρ₂(g) T ρ₂(g)⁻¹` at every group element — which is exactly why it
    read as a gauge-invariant channel and got staked as one. -/
theorem bare_singlet_diagonally_invariant (g : Fin 8) :
    rho2 g * ⟨1, 0, 0, 1⟩ * rho2 (dinv g) = (⟨1, 0, 0, 1⟩ : M2) := by
  revert g; decide

/-- **The kill, in its general form.**  Gauss acts at ONE vertex at a time, and
    the one-vertex average annihilates every bare tensor whatsoever — no choice
    of matter matrix escapes, the diagonal-invariant singlet included. -/
theorem gauss1_bare (T : M2) : ∀ u, gauss1 (bare T) u = 0 := by
  intro u
  show sum8 (fun g => rho2 g * T) = 0
  rw [sum8_mul_right, sum_rho2, zero_mul']

/-- Hence the two-site average of a bare tensor is zero: the sector BRIDGE-5's
    R1 and R3 were reading was empty, which is why they read identically zero. -/
theorem gaussBoth_bare (T : M2) : ∀ u, gaussBoth (bare T) u = 0 := by
  intro u
  show sum8 (fun g => gauss1 (bare T) (dmul u g) * rho2 (dinv g)) = 0
  have h : (fun g => gauss1 (bare T) (dmul u g) * rho2 (dinv g))
      = fun g : Fin 8 => (0 : M2) := by
    funext g; rw [gauss1_bare T (dmul u g), zero_mul']
  rw [h, sum8_zero]

/-- **The escape, vertex 1.**  The dressed pair is fixed by the gauge action at
    site 1 configuration by configuration — invariance BY CONSTRUCTION, which is
    what M14 demanded and what the bare singlet never had. -/
theorem dressed_invariant_v1 (g u : Fin 8) :
    rho2 g * dressed (dmul (dinv g) u) = dressed u := by
  revert g u; decide

/-- **The escape, vertex 2.** -/
theorem dressed_invariant_v2 (g u : Fin 8) :
    dressed (dmul u g) * rho2 (dinv g) = dressed u := by
  revert g u; decide

/-- So the one-vertex average returns the dressed pair scaled by `|D₄| = 8`. -/
theorem gauss1_dressed (u : Fin 8) : gauss1 dressed u = scale 8 (dressed u) := by
  revert u; decide

/-- And the two-site average returns it scaled by `|D₄|² = 64`. -/
theorem gaussBoth_dressed (u : Fin 8) : gaussBoth dressed u = scale 64 (dressed u) := by
  revert u; decide

/-- **The two carriers, side by side.**  Under the same two-site Gauss average
    the bare singlet is annihilated and the Wilson-dressed pair survives
    nonzero.  This is BRIDGE-5's VOID and M14's correction in one statement. -/
theorem bare_dies_dressed_survives :
    (∀ u, gaussBoth (bare ⟨1, 0, 0, 1⟩) u = 0) ∧ (∃ u, gaussBoth dressed u ≠ 0) :=
  ⟨gaussBoth_bare _, ⟨0, by decide⟩⟩

end CIRISHolon.BareCharge
