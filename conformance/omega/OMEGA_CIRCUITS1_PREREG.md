# Pre-registration — OMEGA-CIRCUITS-1: the frozen rent law on the engine's own qubit circuits

*Rung 7 of OMEGA_LADDER.md, the JOINT-LAW face. DRAFT, written before any
rung-7 circuit instrument exists. The law is NOT re-derived and NOT refitted:
`conformance/omega/CROSSFACE1_PREREG.md` is frozen, and every quantity below is
computed from that freeze's Theorems 1–4 applied to a new domain — the
stabilizer/Clifford+T tier of the Rust engine (`engine/crates/holon/src/{tableau,
adaptive,ledger}.rs`). What IS derived here is the domain's own structure
theorem, which turns the frozen law into exact dyadic and quadratic-irrational
rationals computable from circuit coefficients alone.*

misfits: contacts **M-RING-MIXING** — this freeze's central finding is that
misfit's qubit analogue at ring scale √2, derived rather than stumbled into
(§2, Theorem C2): every Clifford Born kernel is a coset randomizer, so on an
F_2-linear view the mixing modulus is permutation-or-maximal-mixing and NOTHING
between, and the two escapes are named and staked; **M-ONE-MODEL-DELTA** (rent
is the frozen minimax / best-memoryless collision statistic, never a delta
against a chosen Markov model — Theorem 1 is imported unchanged);
**M-FINAL-VIEW-COLLISIONS** (every rent below is the staked view's OWN one-step
collision statistic, enumerated on that view itself and never inherited from a
coarser or finer one); **M-NONBIJECTIVE-STEP** (G0 verifies unitarity by exact
row AND column sums before any reading; the non-unital `Step::Reset` is
excluded BY NAME and is plant (iv)); **M-FIXED-POINT-TRAJECTORY** (no trajectory
carrier anywhere — every reading is UNIVERSAL, an exhaustive sum over all 2^n
computational basis states with uniform μ); **M-PROBE-EIGENSTATE** (the honest
statement of G0's weak half: uniform μ is invariant under EVERY unital step, so
invariance is recorded, not earned; G0's content is the exact double-stochasticity
check, which plant (iv) fires); **M-PLANT-OBS** and **M-PLANT-SECTOR** (four
plants in §6, each re-derived for THIS instrument, each pre-checked to fire on a
DERIVATION-side face, each naming the sector its carrier must be nonzero in);
**M-NULL-MISSTAKE** (the staked null is the quantity the law constrains — an
exact rent in Q or Q(√2) — never an equality between different carriers);
**M-PARITY-PROTECT** (three staked views are parity readouts; that SWAP's parity
view is a protected sector reading a constant is the CONTENT of stake C4, not a
surprise, and it is staked as W = 0 in advance); **M-LOOP-BLIND** (the zero-rent
views here are the blind ones — parity discards which qubit, and that is what
buys W = 0; no distant-influence or separated-observable claim is made anywhere);
**M-ELECTRIC-BASIS** (WILSON-1's convicted Fourier kernel is the Hadamard, and
this freeze uses it as DYNAMICS — H is a member of the engine's own gate
alphabet `affine::Gate` and the tier's own motion, never a smuggled basis change,
and there is no gauge constraint in this domain for it to violate);
**M-COND-PROBE** (the non-Clifford rotation of stakes C12/C13 sits INSIDE the
step — the step IS `H·T·H` — and is endogenous dynamics, not an operator applied
after it; no conditioned-operator correction is claimed or needed);
**M-KINEMATIC-NONLOCAL** (the GHZ and teleportation steps entangle, so every
two-qubit marginal below is correlated by preparation; no locality claim, and no
separation of dynamical propagation from kinematic correlation, is made or
needed); **M-HOMOG** (where §7 says the obstruction is localized to one sector,
"localized" is about the (step-class, view-class) product and carries NO spatial
meaning — the qubit index in this domain has no geometry); **M-VOLUME-SCALE** (no
lattice or N-convergence limit is taken; every circuit is finite, n ≤ 5 on the
staked faces and n ≤ 4 on the sweep, and each is read exhaustively at its own
size); **M-STALE-INSTRUMENT** (stated against this freeze and left open: this
campaign is run under a NO-COMMIT instruction, so the instrument and this freeze
live outside the repository and NOTHING here is banked until the owning agent
commits them together — the results below are reproducible-on-demand, not
banked).

---

## 0. What is new here, and what is imported unchanged

Imported unchanged from the frozen CROSS-FACE-1 freeze, with no refit and no
parameter: the face `(S, K, μ, v)`; the Born kernel `K(s'|s) = |⟨s'|U|s⟩|²` for a
unitary substrate; **Theorem 1**, `W(v) = 1 − Σ_i max_j P_ij`; **Theorem 2**, the
cross-face inequality; **Theorem 3**, `W = (1−1/N)(1−λ)` on uniform normal views
whose non-trivial eigenvalues all have modulus λ and whose row maximum sits on
the diagonal; **Theorem 4**, the product law. Nothing in §2 below weakens or
re-states them.

New, and derived here: the domain's structure theorem. A Clifford step's Born
kernel is not a generic doubly stochastic kernel — it is a COSET RANDOMIZER, and
that single fact computes `λ` and `W` in closed form from circuit data alone,
predicts which faces the frozen headline form can and cannot be tested on, and
produces the obstruction this freeze's §7 hands back to the ladder.

The domain: `engine/crates/holon/src/tableau.rs` (the packed Aaronson–Gottesman
tableau — the tier-1 stabilizer engine), `adaptive.rs` (mid-circuit measurement
with feed-forward: teleportation and the repetition-code syndrome cycle are its
own tests), and `ledger.rs`'s exact ring `Cyc = (c₀ + c₁ω + c₂ω² + c₃ω³)·2^{−m/2}`,
`ω = e^{iπ/4}` — the engine's own exact arithmetic, which this freeze's
instrument mirrors coefficient-for-coefficient rather than approximating.

Two fences before anything else.

> **FENCE (adaptive programs are read in unitary dilation).** `adaptive.rs`'s
> programs contain measurement and classical feed-forward. The frozen face needs
> a step with an invariant μ. Measurement-plus-Pauli-correction is read here in
> its DEFERRED-MEASUREMENT form — the standard replacement of a classical
> control by a quantum control, which is an exact identity for the final state of
> the controlled qubit — so the step is a Clifford unitary and the face is
> posable. `Step::Reset` is NOT covered: reset is non-unital, uniform μ is not
> invariant under it, and G0 refuses that face rather than reading it. This is
> stated as a domain limit, not repaired.

> **FENCE (`W` counts displaced mass).** As in the frozen freeze: `W` is the
> minimum per-step measure of state a repair must move to hold the view closed.
> It is not an energy, not a bit count, and no Landauer normalisation is asserted
> — the predecessor programme's own K4 fired at 3–5 dex on exactly that step.
> Nothing below is a thermodynamic claim about a quantum computer.

---

## 1. The objects, instantiated on qubits

`S = F₂ⁿ`, the computational basis; `μ` uniform (`μ(s) = 2^{−n}`);
`K(s'|s) = |⟨s'|U|s⟩|²` for a fixed step unitary `U`. `K` is doubly stochastic
because `U` is unitary, so uniform `μ` is invariant — recorded, not earned
(M-PROBE-EIGENSTATE).

Two view classes, and the distinction is the whole content of §2:

- **linear views**: `v(s) = Bs` for a surjective F_2-linear `B : F₂ⁿ → F₂^m`.
  This class contains every computational-basis marginal on a subset `A` of
  qubits (`B` = the coordinate projection) and every Z-type Pauli parity readout
  (`B` = one functional, `m = 1`). `N = 2^m`, all fibers of size `2^{n−m}`, so
  `μ` on the view is uniform.
- **non-linear views**: any other partition. The staked ones are the
  weight-threshold view (`w(s) ≤ 1` vs `w(s) ≥ 2` on three qubits, four states
  each — not an affine subspace, since `000 + 001 + 010 = 011` is outside) and
  the Hamming-weight view (blocks 1, 3, 3, 1 — non-uniform μ).

---

## 2. The derivation

**Theorem C1 (the Clifford Born kernel is a coset randomizer).**
> For a Clifford `U` on `n` qubits there are a subspace `V ⊆ F₂ⁿ`, a linear map
> `A : F₂ⁿ → F₂ⁿ` and a constant `c₀ ∈ F₂ⁿ` with
> **`K(s'|s) = 2^{−dim V}·1[s' ∈ c₀ + As + V]`** — the same `V` for every `s`.

*Proof.* `U|0ⁿ⟩` is a stabilizer state, and every stabilizer state's amplitudes
are supported on an affine subspace `c₀ + V` of `F₂ⁿ` with uniform modulus
`2^{−dim V/2}` (Dehaene–De Moor / Van den Nest normal form, credited; it is also
exactly what `tableau.rs`'s row-reduced tableau exhibits). For general `s`,
`|s⟩ = X^s|0ⁿ⟩`, so `U|s⟩ = (U X^s U†)·U|0ⁿ⟩`. Clifford conjugation sends
`X^s` to `±i^a X^{As} Z^{Bs}` with `A, B` linear in `s` (the symplectic action on
the Pauli group; `A` is exactly the X-block of the engine's tableau). `X^{As}`
translates the support by `As`; `Z^{Bs}` changes phases only. Moduli are
therefore uniform on `c₀ + As + V` and zero off it. ∎

**Theorem C2 (λ-quantization and the dyadic rent ladder — LINEAR views).**
> With `v(s) = Bs`, `B` surjective onto `F₂^m`, put
> **`H := BA(ker B) + BV ⊆ F₂^m`**, `h := dim H`. Then the transfer is
> `M_{y|x} = 2^{−h}·1[y ∈ φ(x) + H]` for an affine `φ`, and
> **`W(v) = 1 − 2^{−h}`**, **`λ(v) = 0` if `h = m`, `λ(v) = 1` otherwise.**

*Proof.* Given `X = Bs`, `s` is uniform on a coset of `ker B`, so `BAs` is
uniform on a coset of `BA(ker B)`; independently `Bw` is uniform on `BV` for
`w` uniform on `V`. The convolution of the uniform laws of two subgroups of an
abelian group is the uniform law of their sum, so `Y = Bc₀ + BAs + Bw` is
uniform on a coset of `H`, and the coset depends on `x` through an affine map
(well defined mod `H` because changing the preimage moves `BAs` inside
`BA(ker B) ⊆ H`). Rent: `max_y P_{x,y} = (1/N)·2^{−h}` for each of the `N` rows,
so `Σ_x max_y P_{x,y} = 2^{−h}` and Theorem 1 gives `W = 1 − 2^{−h}`. Modulus:
in the character basis `χ_u(y) = (−1)^{u·y}`,
`(Mχ_u)(x) = χ_u(φ(x))·2^{−h}Σ_{g∈H}(−1)^{u·g}`, which is `0` unless `u ∈ H^⊥`
and has modulus 1 pointwise when `u ∈ H^⊥`. The characters `{χ_u}_{u≠0}` are an
orthonormal basis of `1^⊥` in `ℓ²(μ)`, and `Πχ_u = 0` for `u ≠ 0`, so
`‖(M−Π)χ_u‖ ∈ {0, 1}`; `λ = 1` exactly when some `u ≠ 0` lies in `H^⊥`, i.e.
when `H ≠ F₂^m`, and `λ = 0` (indeed `M = Π`) when `H = F₂^m`. Double
stochasticity gives `λ ≤ 1`, so these are the only two values. ∎

> **THE √2-QUANTIZATION OBSTRUCTION (the qubit analogue of M-RING-MIXING).**
> On the stabilizer tier with an F_2-linear view there is NO intermediate mixing
> modulus: `λ ∈ {0, 1}` and the rent lands on the dyadic ladder
> `W ∈ {0, 1/2, 3/4, 7/8, …} = {1 − 2^{−h}}`. At ring scale √3 every `Z[ω]`
> circulant unitary is permutation-or-maximal-mixing (M-RING-MIXING, born at
> LOCAL-1C); at ring scale √2 every Clifford Born kernel is a coset randomizer,
> which is the same statement one ring down. The frozen headline form
> `W = (1−1/N)(1−λ)` therefore CANNOT be tested at intermediate λ anywhere in
> the (Clifford step, linear view) sector — not because the law is weak there,
> but because that sector has no intermediate λ to test it at.

**Corollary C2a (where the headline form fails, and why that is not a defect).**
When `0 < h < m` the reading is `λ = 1` and `W = 1 − 2^{−h} > 0`, while the
headline form evaluated on the AGGREGATE modulus returns `(1−1/N)(1−1) = 0`.
There is no contradiction with Theorem 3: its hypothesis is that EVERY
non-trivial eigenvalue has modulus λ, and a coset randomizer with `0 < h < m`
has eigenvalues of modulus 1 (on `H^⊥`) and 0 (off it). What the qubit domain
supplies is the sharp statement that the hypothesis is load-bearing and by how
much. The correct composition rule is Theorem 4's, in the form of Theorem C3.

**Theorem C3 (retention is multiplicative — Theorem 4's mechanism, generalised).**
> For a product face `(S₁×S₂, K₁⊗K₂, μ₁⊗μ₂, v₁×v₂)`,
> **`1 − W = (1 − W₁)(1 − W₂)`.**

*Proof.* `P_{(i₁i₂),(j₁j₂)} = P^{(1)}_{i₁j₁}P^{(2)}_{i₂j₂}`; the max over
`(j₁,j₂)` is the product of the maxima and the sum over `(i₁,i₂)` factorises, so
`Σ max` is multiplicative. Theorem 1 gives the claim. ∎ Note this needs neither
normality nor circulant edge kernels: it is Theorem 4's convolution step with the
group structure removed, and it is what makes `1 − W` (retention), not `W` and
not `λ`, the composable quantity.

**Corollary C4 (full-spread steps).** If `V = F₂ⁿ` then `K(·|s)` is uniform on
all of `S` for every `s`, so `M = Π` for EVERY view — linear or not — giving
`λ = 0` and `W = 1 − max_j μ_j`, with Theorem 2 tight. This is the only place in
this freeze where a claim is made about non-linear views without enumerating
them.

**Theorem C5 (intermediate λ exists, by two escapes).**
> (a) On a PERMUTATION Clifford step `π` with an equal-block view,
> `M_{j|i} = |b_i ∩ π^{−1}(b_j)|/|b_i|`, which for a non-linear view takes any
> value on the grid `1/|b_i|`; symmetric two-block instances are circulant, hence
> normal with row max on the diagonal, and Theorem 3 applies at
> `λ = |M₀₀ − M₀₁| ∈ (0,1)`. (b) On a non-Clifford step the Born weights leave
> the dyadics: for `U = H·T·H` with the engine's `T = diag(1, ω)`,
> `U = ½[[1+ω, 1−ω],[1−ω, 1+ω]]`, `|1±ω|² = 2±√2`, so the Z-readout transfer is
> the circulant `[[(2+√2)/4, (2−√2)/4],[(2−√2)/4, (2+√2)/4]]`, `λ = √2/2`.

So the obstruction is exactly localized: it is a statement about the PAIR
(Clifford step, linear view), and it has one escape on each coordinate.

---

## 3. Derivation-side instantiations (worked here in full, kernel enumerated by hand)

`A` is read off Clifford conjugation; `V` off the support of `U|0ⁿ⟩`.

| id | n | step `U` | view | `V` | `A` | `h` | `λ` | **`W`** | headline `(1−1/N)(1−λ)` |
|---|---|---|---|---|---|---|---|---|---|
| **C1** | 1 | `S` | Z readout | `{0}` | `id` (`SXS† = Y`) | 0 | 1 | **0** | 0 ✓ |
| **C2** | 1 | `H` | Z readout | `F₂` | `0` (`HXH = Z`) | 1 | 0 | **1/2** | 1/2 ✓ |
| **C3** | 2 | `CX(0,1)` | parity `s₀⊕s₁` | `{0}` | `(s₀, s₀⊕s₁)` | 1 | 0 | **1/2** | 1/2 ✓ |
| **C4** | 2 | `SWAP` | parity `s₀⊕s₁` | `{0}` | swap | 0 | 1 | **0** | 0 ✓ |
| **C5** | 2 | `H(0)` | full basis, `m=2` | `⟨100⟩`→`⟨10⟩` | `diag(0,1)` | 1 | 1 | **1/2** | **0 ✗** |
| **C9** | 3 | `CX(0,2)·CX(0,1)` | weight-threshold | — | permutation | — | **1/2** | **1/4** | 1/4 ✓ |
| **C10** | 3 | `H(0)·H(1)` | weight-threshold | — | — | — | **1/4** | **3/8** | 3/8 ✓ |
| **C12** | 1 | `H·T·H` | Z readout | — | non-Clifford | — | **√2/2** | **(2−√2)/4** | (2−√2)/4 ✓ |

Worked, so the freeze can be checked by a reader without running anything:

- **C3.** `CX` conjugation: `X₀ ↦ X₀X₁`, `X₁ ↦ X₁`, so `A(s) = (s₀, s₀⊕s₁)`;
  `B = (1,1)`, `ker B = {00, 11}`, `BA(s) = s₁`, which is onto `F₂` on `ker B`.
  `h = 1 = m` ⇒ `λ = 0`, `W = 1/2`. Directly: `X = s₀⊕s₁`, `Y = s₁`, and the four
  basis states give `P = ¼·J₂`, `Σ max = 1/2`.
- **C4.** `SWAP` conserves parity, so `BA = B` and `BA(ker B) = 0`, `h = 0`,
  `W = 0`. The parity view is a protected sector reading a constant
  (M-PARITY-PROTECT) and it is staked as such in advance.
- **C5.** `H(0)` on two qubits: `U|s₀s₁⟩ = (|0s₁⟩ + (−1)^{s₀}|1s₁⟩)/√2`, so
  `V = ⟨10⟩`, `A = diag(0,1)`, `B = id`, `h = 1`, `W = 1/2`, and `λ = 1` because
  `f(s) = (−1)^{s₁}` is a non-constant eigenfunction of `M` with eigenvalue 1.
  **This is the first face where the aggregate-λ headline form is wrong (it says
  0), and it is a product face — the correct reading is C3's: retention
  `(1/2)·1 = 1/2`.**
- **C9.** `CX(0,2)·CX(0,1)` is `s ↦ (s₀, s₀⊕s₁, s₀⊕s₂)`. On
  `A = {000,001,010,100}` it fixes the first three and sends `100 ↦ 111`; on the
  complement it fixes three and sends `111 ↦ 100`. So `M = [[3/4,1/4],[1/4,3/4]]`,
  symmetric circulant, row max on the diagonal, `μ` uniform: every hypothesis of
  Theorem 3 holds, `λ = 1/2`, `W = 1/4`.
- **C10.** `H(0)H(1)` randomizes `s₀, s₁` and fixes `s₂`; the weight-threshold
  block `{w ≤ 1}` holds three states with `s₂ = 0` and one with `s₂ = 1`, giving
  `M = [[5/8,3/8],[3/8,5/8]]`, `λ = 1/4`, `W = 3/8`.
- **C12.** As Theorem C5(b). `W = 1 − (2+√2)/4 = (2−√2)/4`, and Theorem 3 returns
  `(1/2)(1 − √2/2) = (2−√2)/4` — **the frozen headline form, at an intermediate
  modulus, exactly, in Q(√2)**.

---

## 4. Held out — the faces this freeze does not enumerate

For everything in this section the Born kernel and the permutation table are
NEVER evaluated in this freeze. The predictions use only `(V, A, c₀)` read off
the circuit, the view's `B` (or its block sizes), Theorem C2/C3/C4, and the
frozen Theorems 1–4. The engine's two flagship adaptive programs are read in
their deferred form exactly as `adaptive.rs` writes them.

**The GHZ step** `U_GHZ = CX(1,2)·CX(0,1)·H(0)` (the engine's Bell/GHZ preparation,
`adaptive.rs`'s teleportation prologue). `U_GHZ|000⟩ = (|000⟩+|111⟩)/√2` so
`V = ⟨111⟩`, `dim V = 1`; conjugation gives `X₀ ↦ Z₀` (X-part `000`),
`X₁ ↦ X₁X₂` (`011`), `X₂ ↦ X₂` (`001`).

**Teleportation, deferred** `U_tel = CZ(0,2)·CX(1,2)·H(0)·CX(0,1)·CX(1,2)·H(1)·H(0)`
— `adaptive.rs::teleportation_works_for_every_seed`'s program with `M(0)`/`M(1)`
and the two `IfBit` corrections replaced by the quantum-controlled `CX(1,2)` and
`CZ(0,2)`. Hand-computed once, in the freeze, and only this: `U_tel|000⟩ = |+++⟩`,
so `V = F₂³` and Corollary C4 applies to EVERY view.

**Repetition-code syndrome cycle, deferred** `U_rep = CX(3,1)·CX(2,4)·CX(1,4)·CX(1,3)·CX(0,3)·X(1)`
— `adaptive.rs::repetition_code_syndrome_cycle_corrects`'s program (data 0,1,2;
ancillas 3,4) with `M(3)` and its `IfBit` correction replaced by `CX(3,1)`, and
the unused `M(4)` dropped. It is a CNOT+X circuit, hence an affine permutation:
`V = {0}`, `c₀ = (0,0,0,1,1)`, and linear part
`A e₀ = (1,1,0,1,0)`, `A e₁ = (0,0,0,1,1)`, `A e₂ = (0,0,1,0,1)`,
`A e₃ = (0,1,0,1,0)`, `A e₄ = (0,0,0,0,1)`.

| id | step | view | `N` | `h` | `λ` | **staked `W`** | aggregate-λ headline |
|---|---|---|---|---|---|---|---|
| **C6** | `U_GHZ` | full basis, `m=3` | 8 | 1 | 1 | **1/2** | 0 ✗ |
| **C7** | `U_GHZ` | total parity `s₀⊕s₁⊕s₂` | 2 | 1 | 0 | **1/2** | 1/2 ✓ |
| **C8** | `U_GHZ` | marginal on qubits {1,2} | 4 | 1 | 1 | **1/2** | 0 ✗ |
| **C11** | `H(0)H(1)H(2)` | weight-threshold | 2 | — | 0 | **1/2** | 1/2 ✓ |
| **C13** | `(HTH)₀ ⊗ (HTH)₁` | full basis, `m=2` | 4 | — | √2/2 | **(5−2√2)/8** | (6−3√2)/8 ✗ |
| **C14** | `[CX(0,2)CX(0,1)] ⊗ H(3)` | weight-threshold × `s₃` | 4 | — | 1/2 | **5/8** | 3/8 ✗ |
| **T1** | `U_tel` | full basis, `m=3` | 8 | 3 | 0 | **7/8** | 7/8 ✓ |
| **T2** | `U_tel` | total parity | 2 | 1 | 0 | **1/2** | 1/2 ✓ |
| **T3** | `U_tel` | marginal on {0,1} | 4 | 2 | 0 | **3/4** | 3/4 ✓ |
| **T4** | `U_tel` | Hamming weight (1,3,3,1) | 4 | — | 0 | **5/8** | n/a (μ non-uniform) |
| **R1v** | `U_rep` | data qubits {0,1,2} | 8 | 1 | 1 | **1/2** | 0 ✗ |
| **R2v** | `U_rep` | syndrome qubits {3,4} | 4 | 2 | 0 | **3/4** | 3/4 ✓ |
| **R3v** | `U_rep` | full basis, `m=5` | 32 | 0 | 1 | **0** | 0 ✓ |

How each was computed, from the structural data only:

- **C6**: `ker B = 0`, `H = BV = ⟨111⟩`, `h = 1`.
- **C7**: `B(111) = 1`, so `BV = F₂ = H` already, `h = 1 = m`, `λ = 0`.
- **C8**: `BV = {00, 11}`; `ker B = ⟨e₀⟩` and `Ae₀ = 0`, so `BA(ker B) = 0`,
  `h = 1 < 2 = m`.
- **C11**: `V = F₂³` (three Hadamards spread `|s⟩` over the whole space), so
  Corollary C4: `M = Π`, `W = 1 − 1/2`.
- **C13**: Theorem C3 on two `H·T·H` factors, retention `((2+√2)/4)² = (3+2√2)/8`.
  The aggregate modulus of the joint transfer is `√2/2` (eigenvalue moduli
  `1, λ, λ, λ²`), so the aggregate-λ headline form returns `(3/4)(1−√2/2)`. The
  two numbers differ in the third digit and no parameter can absorb the gap.
- **C14**: retention `(3/4)·(1/2) = 3/8`. The joint transfer here is **normal**
  and its row maximum IS on the diagonal — two of Theorem 3's three hypotheses
  hold — and only the equal-modulus hypothesis fails (moduli `1, 1/2, 0, 0`).
  The headline form returns `3/8`; the product law returns `5/8`. This face is
  the sharp measurement of which hypothesis carries Theorem 3.
- **T1–T4**: Corollary C4 with `V = F₂³`: `W = 1 − max_j μ_j` for every view.
  T4's blocks have `μ = (1/8, 3/8, 3/8, 1/8)`, so `W = 5/8` with Theorem 2 TIGHT
  at non-uniform μ (`(1−μ_max) − λ(Σσ_i)σ_max = 5/8 − 0`).
- **R1v**: `ker B = ⟨e₃, e₄⟩`; `B A e₃ = (0,1,0)`, `B A e₄ = (0,0,0)`, so
  `H = ⟨(0,1,0)⟩`, `h = 1`. Plainly: the syndrome cycle leaks exactly one bit
  into the data view per round, through data qubit 1's dependence on ancilla 3.
- **R2v**: `ker B = ⟨e₀,e₁,e₂⟩`; `BAe₀ = (1,0)`, `BAe₁ = (1,1)`, `BAe₂ = (0,1)`
  span `F₂²`, `h = 2 = m`, `λ = 0`: syndromes are fresh every round.
- **R3v**: a bijection, so the full view is closed and rent is zero.

---

## 5. Gates — all EXACT, all two-branch, all separable

Readings are exact elements of `Q` or `Q(√2)`. "Miss" means any inequality of
exact values. No gate is repairable by fitting a constant: there is no constant
anywhere in this freeze.

- **G0 — posability (EXACT).** For every staked step: the exact Born kernel's
  row sums AND column sums are 1, so the step is unital and uniform `μ` is
  invariant; each permutation step is verified bijective. Any failure VOIDs that
  step's legs and kills nothing. The honest half: invariance of uniform `μ` is
  automatic for a unital step, so it is recorded, not earned; G0's content is the
  exact double-stochasticity check that plant (iv) fires.
  witness: `alpha_eq_one_of_injective`
- **G1 — Theorem 2 holds (EXACT).** `W ≥ (1−μ_max) − λ·(Σσ_i)·σ_max` on every
  view read, with `λ` from the measured transfer and `σ` bounded by exact
  rational enclosures. A certified violation is a defect in the FROZEN
  derivation and kills this freeze and CROSS-FACE-1 together.
  witness: none (Theorem 2 is proved by hand in the frozen freeze's §2 and is not
  mechanized; its `witness:` line there says the same)
- **G2 — engine agreement (EXACT, two-branch, VOID-bearing).** For every staked
  Clifford circuit and every one of its `2ⁿ` computational basis inputs, the
  engine's own `PackedTableau` (`tableau.rs`, via `measure_peek` per qubit)
  reports the determinate/indeterminate pattern and every determinate value
  exactly as `(c₀, A, V)` predict: qubit `q` is determinate iff the coset
  `c₀ + As + V` has constant `q`-th coordinate. Branch (a) all agree ⇒ the
  substrate read is the ENGINE's, not a Python model of it, and `(c₀, A, V)` are
  the engine's own tableau data. Branch (b) any disagreement ⇒ the instrument or
  the hand derivation misreads the engine; the affected legs are VOID, not
  killed, and the circuit and input are reported.
  witness: none (the engine is Rust; no Lean object mirrors `PackedTableau`)
- **S1 — the held-out engine faces (EXACT, two-branch, 13 staked rationals).**
  Every `W` in §4's table, exactly, together with its `λ ∈ {0,1}` where Theorem
  C2 applies. Branch (a) all 13 ⇒ the frozen rent law lands on a new domain
  class with zero refit, and Theorem C1/C2's closed form computes it from
  circuit coefficients alone. Branch (b) any miss ⇒ report which of `(V, A, B)`
  the reading contradicts and whether the failure is in the structure theorem
  (a wrong `h`) or in the frozen Theorem 1 (a wrong rent at a correct `h`).
  witness: `det_defect_eq_zero_iff_closed`
- **S2 — the √2-quantization obstruction (EXACT, two-branch, sweep).** Over at
  least 20,000 (Clifford circuit, linear view) pairs — `n ∈ {2,3,4}`, depth-12
  circuits drawn uniformly from the engine's own alphabet `{H, S, X, Z, CX}`,
  views spanning `m = 1 … n` — every reading has `λ ∈ {0,1}` EXACTLY and
  `W = 1 − 2^{−h}` EXACTLY with `h = dim(BA(ker B) + BV)` computed from the
  circuit, never from the kernel. Branch (a) all ⇒ the obstruction is real and
  the dyadic ladder is the domain's rent spectrum. Branch (b) any pair with
  `0 < λ < 1`, or any `W ≠ 1 − 2^{−h}` ⇒ Theorem C2 is FALSE; report the circuit,
  the view, the measured `λ²` and the measured rent.
  witness: none (Theorem C2 is proved in §2 by hand; no Lean object in
  lean/CIRISHolon covers Clifford Born kernels — `Stabilizer.lean` carries the
  1-rebit closure kernel, not the n-qubit transfer)
- **S3 — intermediate λ, and the headline form AT it (EXACT, two-branch, 3 stakes).**
  C9 `(λ, W) = (1/2, 1/4)`; C10 `(1/4, 3/8)`; C12 `(√2/2, (2−√2)/4)`; and in all
  three the frozen Theorem 3 equality `W = (1−1/N)(1−λ)` holds EXACTLY. Branch
  (a) ⇒ the obstruction is confined to the (Clifford, linear) pair, both named
  escapes work, and the frozen headline law is confirmed in the qubit domain at
  a genuinely intermediate modulus — including one reading in `Q(√2)`, outside
  the dyadics the tier is made of. Branch (b) any miss ⇒ the headline equality
  fails at intermediate λ on qubits; report which escape failed and whether the
  transfer was in fact normal with its row max on the diagonal.
  witness: `diag_not_closed_under_coherence`
- **S4 — product law over aggregate-λ (EXACT, two-branch, the discriminator).**
  Five faces where the two candidate readings differ and no parameter can absorb
  the gap: C13 `(5−2√2)/8` vs `(6−3√2)/8`; C14 `5/8` vs `3/8`; C5, C6, C8 and
  R1v `1/2` vs `0`. Branch (a) the product-law values ⇒ retention `1 − W`, not
  `W` and not `λ`, is the composable quantity (Theorem C3), and Theorem 3's
  equal-modulus hypothesis is load-bearing quantitatively — C14 fails it while
  satisfying normality and row-max-on-diagonal, so the freeze can say WHICH
  hypothesis carries the theorem. Branch (b) the aggregate-λ values ⇒ Theorem C3
  is false and Theorem 4's product law does not survive the domain change.
  Branch (c) neither ⇒ both are wrong and S1 and S3 are in doubt together.
  witness: none (Theorem C3 is proved in §2 by hand; Theorem 4 is the frozen
  freeze's, whose own `witness:` line is `none`)
- **R1 — the rival, refuted or not (EXACT, two-branch).** The rival is rung 4's
  literal wording carried into rung 6: minimum maintenance cost is a function of
  the DYNAMICS' mixing rate. `U_rep` is a bijection, so its micro modulus is
  exactly 1 for every view and the rival predicts ONE cost for R1v, R2v and R3v.
  This freeze predicts `1/2`, `3/4`, `0` — three values on one step. Branch (a)
  three distinct as staked ⇒ the rival is refuted a second time, now on a qubit
  substrate the frozen freeze never touched, and the cross-face variable is
  view-relative here too. Branch (b) they coincide ⇒ the micro route is live in
  this domain and §0's obstruction was mis-stated.
  witness: `defect_le_alpha_pow`
- **B3 — standing constraints (EXACT).** Every reading is UNIVERSAL: an
  exhaustive sum over all `2ⁿ` basis states under uniform `μ`, with no trajectory
  carrier and no sampled seed anywhere (M-FIXED-POINT-TRAJECTORY — the engine's
  `splitmix` stream is never used, because a seeded run is a trajectory).
  Every view's marginal is verified stationary (`Σ_i P_ij = μ_j`, exactly) before
  its rent is reported. A failure VOIDs that reading.
  witness: `isDist_push`

**Kills, separable.** S1 falsifies the transfer of Theorem C1/C2 to the engine's
own programs and nothing else. S2 falsifies Theorem C2 alone. S3 falsifies the
headline form at intermediate λ alone and leaves the quantized sector standing.
S4 falsifies Theorem C3 alone. G1 falsifies the frozen Theorem 2 and therefore
this freeze and CROSS-FACE-1 together. G2 can only VOID.

---

## 6. Plants (carrier and sector per M-PLANT-SECTOR; each pre-checked to fire on a §3 face)

- **(i) best-model → average-model substitution.** Replace `max_j` by the row
  mean in Theorem 1. Carrier: C9's transfer rows on the `CX(0,2)CX(0,1)` step.
  Sector: the row spread (row max minus row mean), asserted **nonzero in** the
  sector the plant acts on — it is `3/4 − 1/2 = 1/4`. Pre-checked: the reading
  moves `1/4 → 1/2`. FIRES on S3.
- **(ii) drop the spread subgroup.** Compute `h = dim BA(ker B)` and ignore `BV`
  — the natural error, since a permutation step has `V = {0}` and the term is
  invisible on every CNOT-only circuit. Carrier: C2's support subspace on the
  `H` step. Sector: `V` itself, asserted **nonzero in** the sector the plant acts
  on — `dim V = 1`. Pre-checked: the prediction moves `1/2 → 0`. FIRES on S1/S2.
- **(iii) aggregate-λ headline for a product face.** Predict
  `W = (1−1/N)(1−λ)` from the joint transfer's aggregate modulus instead of
  Theorem C3's retention product. Carrier: C5's joint transfer (`H(0) ⊗ I(1)`,
  full-basis view). Sector: the eigenvalue-modulus spread of that transfer,
  asserted **nonzero in** the sector the plant acts on — the moduli are
  `{1, 1, 0, 0}`, spread 1. Pre-checked: the reading moves `1/2 → 0`. FIRES on S4.
- **(iv) non-unital step.** Insert `adaptive.rs`'s `Step::Reset` semantics
  (measure, then correct to `|0⟩`) into a one-qubit step and read the face
  anyway. Carrier: the reset qubit's Born-kernel column sums. Sector: the column
  sum at `s' = 1`, asserted **nonzero in** the sector the plant acts on — the
  defect is `1 − 0 = 1`, the whole column. Pre-checked: G0 refuses the face.
  FIRES on G0, as a VOID.

A missed plant VOIDs the leg it guards.

---

## 7. What each outcome buys, and what it does not

S1 passing is the JOINT-LAW face of rung 7 in its intended form: one frozen
functional, a domain class it was never built for (quantum computation, on the
engine's own tier-0/1 substrates rather than a group torus or a fan disk), zero
refit, and thirteen exact rationals staked before the instrument existed —
including two on the engine's own flagship adaptive programs, read in the
deferred form its own tests certify.

S2 is the finding this freeze expects to be the durable one, and it is an
OBSTRUCTION, not a confirmation: the stabilizer tier cannot exhibit an
intermediate view mixing modulus on a linear view, so the domain's rent spectrum
is the dyadic ladder `1 − 2^{−h}` and nothing else. Read forward, it says the
frozen law's interesting regime is exactly where a quantum computer stops being
classically simulable in the tableau sense — the escape that buys intermediate λ
is the same T gate that buys computational advantage — and read backward, it is
M-RING-MIXING one ring down, which is evidence that the obstruction is a property
of small cyclotomic rings rather than of any one substrate.

S3 is what keeps S2 from being an excuse: the law IS tested at intermediate λ in
this domain, twice by rational arithmetic on a permutation step with a non-linear
view and once in `Q(√2)` on a non-Clifford step, and the frozen Theorem 3
equality is staked to hold exactly at each.

S4 is the only gate that separates a derived composition rule from a lucky fit,
because on C13 and C14 the two candidate readings are both "the law", differing
only in whether it is applied factor-wise or to the aggregate modulus.

R1 is the rival's second refutation, now in a domain where the step is a
bijection for a completely different reason than the group tori were.

What none of this buys: any statement about a WILD quantum process; any
thermodynamic or energetic reading of `W` (see §0's second fence); any claim
about `Step::Reset` or any other non-unital channel, which G0 refuses rather than
approximates; any claim about non-linear views beyond the ones enumerated and
Corollary C4's full-spread case; any mechanization — Theorems C1, C2, C3, C5 are
proved by hand in §2 and their `witness:` lines say `none`; and any banking
under M-STALE-INSTRUMENT, since this campaign commits nothing.
