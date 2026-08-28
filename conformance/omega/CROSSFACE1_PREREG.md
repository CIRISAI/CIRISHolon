# Pre-registration — CROSS-FACE-1: the rent of a view is its Bayes error, and the mixing modulus fixes it

*Rung 4 of OMEGA_LADDER.md. DRAFT, to be committed ALONE and before any rung-4
instrument exists. The law below is DERIVED first, in full, with its
instantiations on two substrates we already own; the numbers staked in the
gates are then computed from group data and circuit coefficients ALONE and
handed to two substrates whose step maps this freeze never touches.*

misfits: contacts M-ONE-MODEL-DELTA (the cost is DEFINED as the minimax /
best-memoryless error, never a delta against one chosen Markov model — this is
the whole shape of Theorem 1), M-GAUGE-LAUNDER and M-LOOP-BLIND (every view
staked here is a conjugacy-class or loop-holonomy reading; the freeze states
plainly what their blindness does and does NOT buy — Theorem 5′ corrects the
guess that laundering is what makes them free),
M-PLANT-OBS and M-PLANT-SECTOR (three plants below, each pre-checked to fire on
a DERIVATION substrate and each naming the sector its carrier must be nonzero
in), M-NONBIJECTIVE-STEP (G0 verifies bijectivity/unitarity before any reading,
because the exact-rent theorem's attainment half uses invertibility),
M-FIXED-POINT-TRAJECTORY (no trajectory carrier anywhere — every reading is
UNIVERSAL, an exhaustive or measure-weighted sum over the whole state space),
M-STALE-INSTRUMENT (the instrument is committed with its results, and the fast
legs fresh-run in CI), M-FINAL-VIEW-COLLISIONS (the rent is the view's OWN
one-step collision statistic, enumerated for the staked view itself and never
inherited from a coarser one), M-COND-PROBE (the conditioned kick k_charge sits
INSIDE the step, and gate S4 exists precisely because a conditioned operator
breaks the product law — the correction is derived, not fitted),
M-ELECTRIC-BASIS (the convicted Fourier kernel is plant (iii), used as a
planted defect and never as dynamics), M-RING-MIXING (the per-edge mixing
modulus 2/3 is exactly the weak coupler that scale-3√3 search bought; at scale
√3 the modulus is 0 and the perimeter law degenerates, which is why WILSON-2's
fan disk can only supply the degenerate end of the derivation),
M-PROBE-EIGENSTATE (no response probe here; the measure is uniform and no
vacuum is named), M-BARE-CHARGE and M-GAUGE-UNIFORM-MOMENTUM (no charged or
dressed sector is read; every staked view is a pure holonomy or a pure group
label, and no single-edge momentum marginal is used as a channel),
M-NULL-MISSTAKE (the null staked is the quantity the law constrains — an exact
rational rent — not an equality between different carriers), M-PARITY-PROTECT
(the class views ARE protected sectors; that is the content of S1, not a
surprise), M-HOMOG and M-KINEMATIC-NONLOCAL (no locality or spatially-separated
claim is made; the tailed graph is used only as a local1-hosted circuit whose
edge kernels are structurally identical, and distant-influence readings are out
of scope), M-VOLUME-SCALE (no lattice N-convergence limit is taken; both
substrates are finite and read exhaustively at their own fixed size).

---

## 0. Why rung 4 needs a new variable, and the obstruction that forces it

OMEGA_LADDER's rung-4 line reads `(T_c, μ_c) → mixing rate → g_c and minimum
maintenance cost`. Taken literally that is FALSE for half our substrates, and
the library already says so by machine.

`Mixing.lean` carries the Dobrushin coefficient `alpha` and the mixing theorem
`defect_le_alpha_pow`: the closure defect at lag m is at most `alpha^m`. It also
carries `alpha_eq_one_of_injective` — **every injective deterministic step has
`alpha = 1`**. The D4 torus and the 2T torus are permutations. Their micro
mixing rate is exactly 1, the mixing theorem degenerates to `defect ≤ 1`, and no
micro-spectral quantity can bound anything about their maintenance cost. This is
not a gap in the proof; it is a true statement about permutations, and it is the
**named obstruction** this freeze works around rather than papers over:

> **OBSTRUCTION (micro-gap vacuity).** For invertible dynamics there is no micro
> mixing rate. The cross-face variable that carries the law cannot be a function
> of `(T, μ)` alone; it must be a function of `(T, μ, v)`.

The variable this freeze introduces is the **view mixing modulus** λ(v) — the
second singular value of the view's own one-step transfer operator. λ is
computable from `(T, μ)` alone in exactly two structurally identified cases
(Theorem 4 and Theorem 6 below), and those two cases are what make the law
predictive rather than descriptive.

A second fence, stated so the freeze is not read as claiming work already
banked: `Object.lean` already has `Wstar` and `Ginf_at_Wstar`, the price of
holding a SCALAR at retention 1−δ against geometric decay at gap γ. That is the
magnitude face of rent. The quantity below is the PARTITION face — the price of
holding a coarse view CLOSED. They are different numbers about different
objects, and nothing here is inherited from there.

---

## 1. The objects, frozen

A **face** is `(S, K, μ, v)`:

- `S` finite; `K` a row-stochastic kernel on `S`; `μ` a `K`-invariant measure.
  For a permutation substrate `K` is the deterministic kernel of the step and
  `μ` is the counting measure on `S`. For a unitary substrate `K` is the Born
  kernel `K(s'|s) = |⟨s'|U|s⟩|²` — doubly stochastic because `U` is unitary — and
  `μ` is uniform over the basis states. In both cases uniform `μ` is invariant.
- `v : S → V` a view with fibers `b_1 … b_N`, `μ_i := μ(b_i)`.
- `X := v(s)`, `Y := v(s')` with `s ~ μ`, `s' ~ K(·|s)`. `P_ij := Pr[X=i, Y=j]`.
  Invariance of `μ` gives `Σ_i P_ij = μ_j`: the view's marginal is stationary.
- **transfer operator** `M_{j|i} := P_ij / μ_i`, acting on view-functions by
  `(Mf)_i = Σ_j M_{j|i} f_j`; `Π f := Σ_j μ_j f_j`.
- **view mixing modulus** `λ(v) := ‖M − Π‖` on `1^⊥` in the `μ`-inner product
  (the second singular value). `λ = 0` is one-step amnesia; `λ = 1` means some
  non-constant view-function is carried isometrically.
- **maintenance policy**: a kernel `R(s''|s, s')` — a repair that KNOWS where the
  system came from. It **holds the view closed** when there is `F : V → V` with
  `v(s'') = F(v(s))` almost surely. Its **work** is `W(R) := Pr[s'' ≠ s']`, the
  measure of state that has to be moved per step.
- **rent** `W(v) := min over holding policies of W(R)`.

Note what `W` is and is not. It counts DISPLACED MASS per step. It is not an
energy and not a bit count; no Landauer normalisation is asserted anywhere, and
the predecessor programme's own K4 fired at 3–5 dex on exactly that step.

---

## 2. The derivation

**Theorem 1 (exact rent = Bayes error of the best memoryless coarse model).**
> `W(v) = 1 − Σ_i max_j P_ij`.

*Proof.* (≥) Let `(R, F)` hold the view closed. Whenever the repair leaves the
state alone, `s'' = s'`, closure forces `v(s') = F(v(s))`. So
`{s'' = s'} ⊆ {Y = F(X)}` and `Pr[s''=s'] ≤ Pr[Y = F(X)] = Σ_i P_{i,F(i)} ≤ Σ_i max_j P_ij`.
Hence `W(R) ≥ 1 − Σ_i max_j P_ij`.
(≤) Take `F(i) ∈ argmax_j P_ij`; let `R` leave `s'` alone when `v(s') = F(v(s))`
and otherwise move it to a fixed point of block `F(v(s))`. This is a legitimate
kernel because `R` sees the pair `(s, s')`, it holds the view closed by
construction, and its work is `1 − Σ_i P_{i,F(i)}`, the bound. ∎

Two readings this pins. First, `W = 0` exactly when the view is CLOSED
(`closed_iff_fiber_invariant`, and via total variation
`det_defect_eq_zero_iff_closed`), so rent and closure are one object read at two
strengths. Second, `W` is a MINIMAX, not a residual against a chosen model —
M-ONE-MODEL-DELTA is discharged in the definition, and the shape is
`Closure.lean`'s `minimax_error_at_least_half` generalised from two blocks to N.

**Theorem 2 (the cross-face inequality).**
> `W(v) ≥ (1 − μ_max) − λ(v) · (Σ_i σ_i) · σ_max`, where `σ_i := √(μ_i(1−μ_i))`,
> `μ_max := max_j μ_j`, `σ_max := max_i σ_i`.

*Proof.* Write `1_i = μ_i·1 + f_i` with `f_i ⊥ 1` in `ℓ²(μ)`. Then
`P_ij = ⟨1_i, M 1_j⟩_μ` and `⟨μ_i 1, (M−Π)1_j⟩_μ = μ_i(Σ_k μ_k M_{j|k} − μ_j) = 0`
by invariance, so `P_ij − μ_iμ_j = ⟨f_i, (M−Π) f_j⟩_μ`, whence
`|P_ij − μ_iμ_j| ≤ λ σ_i σ_j` — the expander mixing lemma, at the view level.
Therefore `Σ_i max_j P_ij ≤ Σ_i (μ_i μ_max + λ σ_i σ_max) = μ_max + λ σ_max Σ_i σ_i`,
and Theorem 1 gives the claim. ∎

**Theorem 3 (equality on uniform normal views — the headline form).**
> If `μ_i = 1/N` and `M` is normal with every non-trivial eigenvalue of modulus
> `λ` and with the row maximum on the diagonal, then
> **`W(v) = (1 − 1/N)·(1 − λ)`**: **rent = ceiling × (1 − retention)**.

*Proof.* Substituting `μ_i = 1/N` into Theorem 2 gives `(1−1/N)(1−λ)`. For a
circulant `M` with non-trivial eigenvalues `λ`, `M_ii = (1 + (N−1)λ)/N` is the
row maximum, so Theorem 1 returns `1 − (1+(N−1)λ)/N = (N−1)(1−λ)/N`. ∎

**Theorem 4 (the product law — the perimeter law of maintenance).**
> Let the step act on edge variables `d_e ∈ Z_n` by independent circulant Born
> kernels `p_e`, and let the view be a loop holonomy `h = Σ_e s_e d_e`,
> `s_e = ±1`. Then `λ(h) = Π_{e ∈ loop} λ_e` with `λ_e = |Σ_d p_e(d) ω^d|`, and
> **`W(h) = (1 − 1/n)(1 − Π_e λ_e)`**. For a uniform edge kernel this is a
> function of PERIMETER alone: `W(L) = (1 − 1/n)(1 − λ_e^L)`.

*Proof.* From uniform `μ` the digits are i.i.d. uniform, so the holonomy
increment is the convolution of the independent per-edge increments and the
transfer is circulant with character value `Π_e p̂_e(s_e)`. Symmetric kernels
(`p(+1) = p(−1)`) make each factor real and equal to `λ_e`. Theorem 3 applies. ∎

`λ_e` is read off the CIRCUIT, not fitted: for a circulant unitary with
amplitudes `c_0, c_1, c_2` the Born weights are `p_k = |c_k|²/Σ|c_j|²`.

**Theorem 5 (normaliser corollary — zero rent).**
> If the view's fibers are the orbits of a group Γ acting on `S`, and the step
> normalises that action, then `λ = 1` and `W = 0`.

*Proof.* The step permutes Γ-orbits, so `M` is a permutation matrix: closed, and
Theorem 1 returns 0. ∎

**Theorem 5′ (commutator-class corollary — and a correction to the obvious
guess).** On a group torus with the mapping-class step, the two word identities
`comm(a, ab) = a·comm(a,b)·a⁻¹` and `comm(b, bab⁻¹) = b·comm(a,b)⁻¹·b⁻¹` (both
verified exhaustively on D4) compose to
`comm(step(a,b)) = (ab)·a·comm(a,b)⁻¹·a⁻¹·(ab)⁻¹`, a conjugate of the INVERSE.
Hence the class view's transfer `M` is the permutation of classes induced by
inversion — **for every group, ambivalent or not** — so `W(v_comm) = 0` always.
Ambivalence is NOT what buys the zero rent. What ambivalence buys is that the
permutation is the IDENTITY, i.e. that the maintained view is stationary rather
than cycled. The freeze records this because the natural guess — "the class view
is closed iff the classes are ambivalent" — is false, and was believed here
until the identities were composed.

**Theorem 6 (class-algebra corollary).**
> On a group torus `S = G × G` with the mapping-class step
> `(a,b) ↦ (ab, (ab)a(ab)⁻¹)`, the conjugacy-class view of `g_a` has
> `M_{j|i} = |C_j|/|G|` for every `i` — hence `λ = 0` and
> **`W = 1 − max_j |C_j| / |G|`** — and the pair-class view
> `(class a, class b)` has transfer
> `M_{(j,i')|(i,k)} = δ_{i',i} · q_{ik}(j)` with
> `q_{ik}(j) = Pr[class(ab) = j | a ∈ C_i, b ∈ C_k]` the class-multiplication
> law, so `W = 1 − Σ_{i,k} (|C_i||C_k|/|G|²) · max_j q_{ik}(j)`.

*Proof.* For fixed `a`, `b` uniform makes `ab` uniform on `G`, independent of
`a`; the pair case is the same computation with the second coordinate carried
deterministically. ∎

**Theorem 7 (rent is a rate).**
> If the maintenance policy preserves `μ`, the work over `τ` steps is `≥ τ·W(v)`,
> and a policy budgeted below `W` cannot hold the view closed at any step.

*Proof.* Immediate from Theorem 1 applied at each step, the state law being `μ`
throughout. ∎ This is the partition-face analogue of the predecessor rent
clause `rent_holds` / `underpaid_shrinks`; the Lean object `rent_closed_form`
is the magnitude-face statement and is NOT what this theorem says.

**Fence on Theorem 2.** The bound is vacuous whenever `λ = 1`, and `λ = 1`
whenever the view carries a non-constant function isometrically — which every
closed view and every view containing a deterministically-copied coordinate
does. On permutation substrates the operative engine is therefore Theorems 5
and 6, not Theorem 2. That is the obstruction of §0, localised.

---

## 3. Derivation instantiations (exact, on substrates already in the tree)

### (a) D4 one-plaquette torus — `einstein_adm1.py`, |S| = 64, step a permutation

Step verified bijective; cycle type `1·1 + 9·3 + 6·6`; micro `alpha = 1` exactly.
Exact rents by Theorem 1, versus the predictions from group data alone:

| view | blocks | `λ` | exact `W` | predicted from | prediction |
|---|---|---|---|---|---|
| `v_ADM` simultaneous-conjugation orbits | 28 | 1 | **0** | Theorem 5 | 0 ✓ |
| `v_flat` flat / not flat | 2 | 1 | **0** | Theorem 5 | 0 ✓ |
| `v_comm` class of the commutator | 2 | 1 | **0** | Theorem 5′ | 0 ✓, and `M` is exactly `I₂` |
| `v_classA` class of `g_a` | 5 | 0 | **3/4** | Theorem 6, sizes (1,1,2,2,2) | 3/4 ✓ |
| `v_classB` class of `g_b` | 5 | 0 | **3/4** | Theorem 6 | 3/4 ✓ |
| `v_classpair` (class a, class b) | 25 | 1 | **3/32** | Theorem 6, class algebra | 3/32 ✓ |

Theorem 2 is tight at `λ = 0` (bound 3/4 = W) and vacuous at `λ = 1` (bound
−0.29 for `v_ADM`), exactly as §2's fence says.

### (b) Z3 fan disk — `wilson2.py`, 10 edges, 59049 configs × 9 matter, step unitary

Read with the FROZEN instrument's own `step` and `weight_triple`, exact integer
Eisenstein arithmetic, six basis carriers spanning both matter sectors:

- every spoke edge: increment law `(1/3, 1/3, 1/3)`, carrier-independent, so
  `λ_spoke = |1/3 + ω/3 + ω²/3| = 0` exactly — `u_e`'s amplitudes `(1, w, w)`
  have equal modulus. (Carrier-independence survives the conditioned kick on
  `e0` because summing the three diagonal matter components restores it:
  `3|c(Δ)|² + 6|c(Δ−1)|²` is flat when `|c|` is.)
- every rim edge: increment law `(1, 0, 0)` — no factor of the step touches the
  rim — so `λ_rim = 1` exactly.
- every plaquette holonomy: `λ = 0`, exact rent **2/3** = `(1−1/3)(1−0)`.
- the rim loop holonomy: `λ = 1`, exact rent **0**.

The fan disk supplies only the DEGENERATE end of Theorem 4 (`λ_e ∈ {0,1}`),
which is M-RING-MIXING in a new dress: at scale √3 every edge kernel is
permutation-or-maximal-mixing. The perimeter law's content lives at
`0 < λ_e < 1`, and the only substrate with such an edge is held out.

---

## 4. Held out — the substrates this freeze does not touch

**`pt2t.py` (2T torus, 576 configs, permutation)** and **`local1.py` (tailed
graph, 12 edges, 531441 configs × 9 matter, unitary)**. Their step maps are
never evaluated in this freeze. The predictions below use, for 2T, the group
table's class data ONLY (sizes and class multiplication), and for the tailed
graph, `u_e`'s Eisenstein coefficients ONLY:

`c_0 = 5+4w`, `c_1 = 2+w`, `c_2 = −1−2w` → Born weights `(21, 3, 3)/27` →
**`λ_e = 2/3` exactly, the same kernel on all twelve edges.**

2T: `|G| = 24`, seven classes of sizes `(1, 1, 6, 4, 4, 4, 4)`. The group is NOT
ambivalent — its order-3 and order-6 classes are not — but the commutator values
lie in classes `{1}`, `{−1}`, `{six order-4}`, and those three ARE ambivalent.
By Theorem 5′ the mass view is closed either way; what the ambivalence of those
three classes predicts is the sharper fact that its transfer is the IDENTITY.

---

## 5. Gates — all EXACT, all two-branch, all separable

Readings are exact rationals. "Miss" means any inequality of exact rationals.

- **G0 — posability (EXACT).** The 2T step index map is a bijection on all 576
  configs; the tailed-graph step preserves total Born weight exactly on every
  carrier; both `μ` are invariant. Any failure VOIDs that substrate's legs and
  kills nothing. witness: `alpha_eq_one_of_injective` (the permutation leg's
  micro modulus is 1, recorded not assumed)
- **G1 — the inequality holds (EXACT).** Theorem 2 is checked on every view read
  on both substrates: `W ≥ (1−μ_max) − λ·(Σσ_i)·σ_max` with `λ` computed from
  the measured transfer. A violation is a defect in the DERIVATION, not in the
  substrate, and kills the whole freeze. witness: none (the inequality is proved
  in §2 by hand and checked numerically over 4000 random kernels/views, worst
  residual −1.1e-16; it is not yet mechanized)
- **S1 — 2T closed views (EXACT, two-branch, 3 staked facts).** `W(v_orbit) = 0`
  and `W(v_comm) = 0`, both exactly, with `λ = 1` for both; and — the sharper
  third stake — `v_comm`'s transfer on the 3 realised commutator classes is
  exactly the IDENTITY matrix `I₃`, not merely some permutation matrix. Branch (a)
  all three ⇒ Theorem 5 and Theorem 5′ transfer from D4 to 2T with no refit, and
  the ambivalence of the commutator classes is confirmed to control stationarity
  rather than closure. Branch (b) either rent nonzero ⇒ Theorem 5/5′ is false and
  the reading must report which word identity failed; branch (c) rents zero but
  `M ≠ I₃` ⇒ closure survives and the AMBIVALENCE reading is what dies, the
  reading naming the cycled classes. Branch (c) is the one this freeze got wrong
  on its first pass and is staked separately for that reason.
  witness: `closed_view_inherits_conservation`
- **S2 — 2T class algebra (EXACT, two-branch).** `W(v_classA) = 3/4` (from
  `1 − 6/24`) and `W(v_classpair) = 43/144`. Branch (a) both exact ⇒ Theorem 6
  confirmed forward on a group it was never derived on (D4 gave 3/4 and 3/32 by
  the same recipe; 43/144 is a new number). Branch (b) either misses ⇒ Theorem 6
  is false; report the offending `(i,k)` cell of `q_{ik}`.
  witness: none (Theorem 6 is elementary group theory, proved in §2, not mechanized)
- **S3 — the perimeter law (EXACT, two-branch).** On the tailed graph, with
  `λ_e = 2/3` and no free parameter:
  `W(single edge, L=1) = 2/9`; `W(pendant plaquette 10,11,8, L=3) = 38/81`;
  `W(rim loop 5..9, L=5) = 422/729`; `W(plaquette 1,6,2, L=3) = 38/81`.
  Equivalently `W(L) = (2/3)(1 − (2/3)^L)` at `L = 1, 3, 3, 5`. Branch (a) all
  four exact ⇒ the product law holds and rent obeys a PERIMETER law — the first
  cross-face number the programme has predicted before measuring. Branch (b) any
  miss ⇒ Theorem 4 is false on this substrate; report the measured increment law
  and which edge's independence failed. witness: none (Theorem 4 proved in §2 by
  convolution; the derivation example (b) is its degenerate-end check)
- **S4 — the conditioned edge (EXACT, two-branch, the discriminator).** The
  plaquette `4,9,0` contains `e0 = E_STAR`, on which `k_charge` acts INSIDE the
  step. Derived branch laws: off-diagonal matter (6 of 9 basis states) gives
  `(57,57,129)/243`; diagonal matter (3 of 9) gives `(81,57,105)/243`; the
  `μ`-mixture is `(65,57,121)/243` and **`W = 122/243`**. The naive product law,
  which ignores the conditioning, says `38/81 = 114/243`. Branch (a) `122/243`
  ⇒ the conditioned-operator correction is right and M-COND-PROBE's lesson is
  now quantitative. Branch (b) `114/243` ⇒ the conditioning is invisible to this
  view and the correction is spurious; branch (c) neither ⇒ both derivations are
  wrong and S3's product law is put in doubt on any loop touching `e0` only.
  witness: none (derived in §2 and §6; no Lean object covers conditioned kernels)
- **S5 — structural carrier-independence (EXACT, two-branch, VOID-bearing).**
  For every `e0`-free view of S3, the exact increment law must be IDENTICAL
  across a stated carrier set of at least 54 basis states (all 9 matter values ×
  at least 6 configs including the all-zero config). Branch (a) identical ⇒ the
  factorisation Theorem 4 assumes is confirmed on the sample and S3's readings
  are exact conditional on it. Branch (b) not identical ⇒ Theorem 4's
  independence hypothesis fails; S3 is VOID, not killed, and the carrier
  dependence is reported. A sample can refute this structure; it cannot prove
  it, and the freeze does not claim otherwise. witness: none (structural)
- **B3 — standing constraint (EXACT).** Gauss invariance holds on every state
  the rung-4 instrument registers, on both substrates, exactly as the host
  instruments check it. A failure VOIDs the reading. witness: none (each host
  instrument's own `gauss_holds`, reused unmodified)
- **R1 — the rival, refuted or not (EXACT, two-branch).** The rival hypothesis
  is rung 4's literal wording: that the minimum maintenance cost is a function
  of the DYNAMICS' mixing rate. On 2T that rate is 1 for every view (permutation),
  so the rival predicts ONE cost for all four staked 2T views. Our law predicts
  `0, 0, 3/4, 43/144` — four values, two of them equal only by accident of the
  class sizes. Branch (a) the four differ as staked ⇒ the rival is refuted and
  the cross-face variable is view-relative, as §0's obstruction says. Branch (b)
  they coincide ⇒ the obstruction was mis-stated and the micro route is live
  again. witness: `defect_le_alpha_pow`

**Kills, separable.** S1 falsifies Theorem 5 alone. S2 falsifies Theorem 6
alone. S3 falsifies Theorem 4 alone. S4 falsifies the conditioned-edge
correction alone and takes nothing else with it. G1 falsifies Theorem 2 and
therefore the whole freeze. No gate's failure is repairable by refitting a
constant: there is no constant to fit.

---

## 6. Plants (carrier and sector per M-PLANT-SECTOR; each pre-checked to fire on a DERIVATION substrate)

- **(i) best-model → average-model substitution.** Replace `max_j` by the row
  mean in Theorem 1's formula. Carrier: `v_classA`'s transfer rows on the D4
  torus. Sector: the row spread (row max minus row mean), asserted **nonzero in**
  the sector the plant acts on — measured `1/20` on row 0. Pre-checked: the
  reading moves `3/4 → 4/5`. FIRES.
- **(ii) normaliser-breaking twist.** Post-compose the step with the
  non-covariant twist `(a,b) ↦ (a, r·b)` that `einstein_adm1`'s own plant (i)
  uses, then read a view Theorem 5 calls closed. Carrier: `v_ADM`, the
  28-block simultaneous-conjugation orbit view on the D4 torus. Sector: the
  twisted transfer's off-diagonal, asserted **nonzero in** the sector the plant
  acts on — 64 of 64 configs are re-routed. Pre-checked: the rent moves
  `0 → 7/32`. FIRES.
- **(iii) convicted Fourier kernel on one edge.** Substitute WILSON-1's
  basis-change kernel (Born weights `(1,1,1)/3`) for the weak coupler on one
  edge of a staked loop. Carrier: the three-edge pendant loop's increment law.
  Sector: that loop's mixing modulus, asserted **nonzero in** the sector the
  plant acts on — the substituted edge's `λ_e` moves `2/3 → 0`. Pre-checked: the
  predicted rent moves `38/81 → 2/3`. FIRES.

A missed plant VOIDs the leg it guards.

---

## 7. What each outcome buys, and what it does not

S1+S2 passing means the ZERO-rent side of the law transfers between two
different groups with no refit — and it means something the freeze states
plainly rather than hiding: **the views that cost nothing to maintain are the
blind ones**, `v_ADM`'s rent being zero precisely because the orbit label
discards everything the twist could move (M-LOOP-BLIND). But the freeze also
records where that slogan overshoots, because it overshot here first: the
commutator-class view costs zero whether or not it is blind to inversion
(Theorem 5′). Orientation-blindness (M-GAUGE-LAUNDER) does not buy the zero
rent; it buys STATIONARITY — the difference between a maintained view that sits
still and one that cycles at no charge. Cheap maintenance is bought with
discarded structure, but the exchange rate is finer than "blind is free", and
S1's branch (c) is what separates the two.

S3 passing gives the programme its second forward-predicted functional form
(after the pump law): rent obeys a **perimeter law**, `W(L) = (1−1/n)(1−λ_e^L)`,
saturating at the no-information ceiling as the loop grows. It says a large
holon pays nearly the ceiling to hold any long loop closed, and that the whole
saving lives at short perimeter.

S4 is the only gate that can separate a derived correction from a lucky fit,
because the two candidate values `122/243` and `114/243` differ in a way no
parameter can absorb.

What none of this buys: any statement about a WILD process, any thermodynamic
cost, any claim that `W` is an energy, and any claim that Theorem 2 is
mechanized — it is not, and G1's witness line says so. The law is a statement
about finite exact models, and the whole of §3 and §5 stays inside them.
