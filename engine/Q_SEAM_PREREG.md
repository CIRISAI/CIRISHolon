# Q-SEAM PREREG — the certified meet between an exact quantum reference and a Boolean-occupancy chart

**Frozen 2026-08-23, before any instrument exists.** No crate `q-seam` exists at the time of
this commit; no ground state, no self-consistent field, no error, no share has been computed
for this campaign. Every number below is either (a) an analytic value derived here in closed
form, (b) a threshold or tolerance *chosen* and hereby fixed, or (c) a forward prediction.
Class (b) items are marked **STAKED**; class (c) items are marked **P-**.

The product is not a better quantum solver. The product is a **refusal**: a chart that matches
the exact reference where it is certified to, and declines to speak where correlation makes it
lie. Q5 builds and mutation-tests that refusal. Q6 asks, separably, whether the exact state's
beyond-pair structure predicts *where* the chart fails.

---

## 0. DEVIATIONS FROM THE COMMISSION, DECLARED UP FRONT

Three, each with its reason. They are deviations, not silent choices.

**D1 — the residual class is not `~1e-15`, and cannot be.** The vacuum tier's `5.6e-17`
(`gauge.rs`, `one_plaquette_vacuum`) is a property of a **3×3** matrix solved by Jacobi to
`JACOBI_TOL = 1e-15`. The reference here reaches Hilbert-space dimension 63 504; its ground
state comes from Lanczos, and its honest residual class is set by that. §3 stakes a **residual
ladder** instead of one blanket figure: bitwise-exact where exactness is available (Hermiticity,
sector conservation, integer counts), `≤ 1e-14` relative where a dense independent solver is
affordable (N ≤ 6), `≤ 1e-12` relative where only Lanczos is (N = 8, 10), and `≤ 1e-11` on the
derived symmetry residuals, which inherit the eigenvector's error. Promising 1e-15 at dimension
6e4 would be a claim I could not cash.

**D2 — the workspace manifest is not touched by this commit.** `sim_engine/Cargo.toml` is a
shared file on a tree with several live agents. Adding `crates/q-seam` to `members` is a
one-line coordination item for the integrator, reported rather than performed. Until then the
crate is built with an explicit `--manifest-path`.

**D3 — N = 12 is out of scope, with the reason.** At N = 12 half filling the Sz = 0 sector is
C(12,6)² = 853 776 states and the Q6 pass is C(24,4) = 10 626 quadruples × 853 776 basis states
≈ 9e9 histogram updates. That is a detached-compute job (§8) for a rung that adds a fifth point
to a four-point trend. It is declared out, not left as an open promise.

---

## 1. THE REFERENCE FAMILY

**One-dimensional Hubbard chain, open boundary conditions, half filling, Sz = 0.**

```
H = -t Σ_{i=1}^{N-1} Σ_σ ( c†_{iσ} c_{i+1,σ} + h.c. )  +  U Σ_{i=1}^{N} n_{i↑} n_{i↓}
```

with `t = 1` fixing the energy unit throughout. Sector: `N_↑ = N_↓ = N/2`.

**Why this family and not transverse-field Ising** (two sentences, as asked). TFI is a *free*
theory — Jordan–Wigner plus Bogoliubov diagonalizes it exactly at every field, so in the
Bogoliubov mode basis a Boolean-occupancy chart is exact for the entire sweep and the certificate
would be measuring nothing but my choice of basis. The Hubbard chain has no such basis: its
one-body density matrix has strictly fractional natural occupations for every `U > 0`, which is
`Core/ModeChart.lean`'s fence (`meanOcc_fractional_exists`) made physical, and `U/t` is the
correlation knob that carries the chart from exact (`U = 0`) to catastrophically wrong
(`U/t = 16`).

**Why open boundaries.** A periodic ring at half filling with `N ≡ 0 (mod 4)` has a degenerate
open-shell non-interacting ground state — the "ground state" would not be unique, the mean-field
chart would be ill-defined, and every symmetry residual in §3 would be meaningless. The open
chain is closed-shell and non-degenerate at `U = 0` for every even N. This is a known trap and
it is avoided by construction, not by luck.

**The grid — fixed now, 70 configurations.**

- `N ∈ {2, 4, 6, 8, 10}` (Hilbert dimensions 4, 36, 400, 4 900, 63 504).
- `U/t ∈ {0, 0.125, 0.25, 0.5, 0.75, 1, 1.5, 2, 3, 4, 6, 8, 12, 16}` (14 values, dense at the
  small-U end where the certified boundary is expected to sit).
- **The PLANT** is `U/t = 16` at every N: the sweep's far end, deep in the Mott regime.

### 1.1 Analytic values available in advance (the ruler is gauged before it is used)

House lesson: *gauge a ruler with planted values before staking a band.* Three closed forms are
derived here and become gates in §3.

**(i) The whole `U = 0` column.** Single-particle levels `ε_k = −2t cos(kπ/(N+1))`, k = 1..N.

```
E(U=0) = −4t Σ_{k=1}^{N/2} cos(kπ/(N+1))
Δ(N)  ≡ ε_{N/2+1} − ε_{N/2} = 4t sin( π / (2(N+1)) )        [derived: the sum-to-product identity
                                                              collapses because (2a+1)θ/2 = π/2]
```

| N | dim (Sz=0) | E(U=0)/N/t | Δ(N)/t |
|---|---|---|---|
| 2 | 4 | −1.000000 | 2.000000 |
| 4 | 36 | −1.118034 | 1.236068 |
| 6 | 400 | −1.164653 | 0.890084 |
| 8 | 4 900 | −1.189693 | 0.694593 |
| 10 | 63 504 | −1.205335 | 0.569259 |

The closed form for `Δ(N)` was cross-checked against direct level differences
`ε_{N/2+1} − ε_{N/2}` and agrees to machine precision at all five N; the table is printed to six
decimals and the instrument must reproduce it (G-E7). `Δ(N)` shrinking as `≈ 2πt/(N+1)` is the
mechanism behind every N-dependence staked below: the chart's own denominator gets smaller, so
it breaks at smaller U as N grows.

**(ii) The whole N = 2 column, at every U.** The Hubbard dimer at half filling in Sz = 0:

```
E₀(U) = ( U − √(U² + 16t²) ) / 2
⟨Σ_i n_{i↑}n_{i↓}⟩ = ∂E₀/∂U = ( 1 − U/√(U² + 16t²) ) / 2      [Hellmann–Feynman]
```

So the entire N = 2 row of the sweep — energy and double occupancy at all 14 U values — is
analytically known before the instrument runs, and becomes a full-sweep ruler, not just a
zero-point check.

**(iii) Two exact statements about the exact state that make the plant's failure certain.**
On a bipartite lattice at half filling the particle–hole map `c_{iσ} → (−1)^i c†_{iσ}` is a
symmetry, so `⟨n_{iσ}⟩ = 1/2` exactly at every site, every spin, every U. And by **Lieb's
theorem** (PRL 62, 1201 (1989)) the half-filled bipartite Hubbard ground state has total spin
`S = |N_A − N_B|/2 = 0` and is unique up to spin degeneracy — so `m_i ≡ ⟨n_{i↑} − n_{i↓}⟩ = 0`
exactly, at every site, every U. A mean-field chart that breaks spin symmetry therefore reports
a magnetization the exact state provably does not have. **The plant's chart failure is not
hoped for; it is a theorem.**

---

## 2. THE CLASSICAL CHART

**A single Slater determinant — Boolean occupancy over spin-orbital modes, exactly
`Core/ModeChart.lean`'s `OccState M`.** Spin-unrestricted Hartree–Fock (UHF):

```
F_{iσ} = h + U · diag( n_{i,−σ} ),      n_{iσ} = Σ_{p ∈ occ_σ} |φ^σ_p(i)|²
```

self-consistent, filling the lowest N/2 orbitals per spin. The chart's one-body density matrix
is idempotent by construction — its mode occupations are exactly `{0,1}`, which is precisely why
the chart *cannot see its own fractionality*, and why §4's certificate must be built from
something other than those occupations.

**Determinism, fixed now.** Three initial guesses — uniform (`n_{iσ} = 1/2`), Néel
(`n_{iσ} = 1/2 ± 0.25(−1)^i`), and a fixed-seed pseudorandom guess (seed `0x5EAM`, stated) —
each run to convergence with linear mixing `α = 0.3`, stopping at `max_{iσ} |Δn_{iσ}| ≤ 1e-12`,
cap 5 000 iterations. **The chart is the lowest-energy converged solution of the three.** No
adaptive restarts, no tuning; a configuration where none of the three converges is VOID and is
reported as VOID, never as a refusal.

### 2.1 The observable set — FIXED NOW, six entries, no additions

| # | Observable | Chart prediction | Exact | Tolerance τ (**STAKED**) |
|---|---|---|---|---|
| O1 | energy per site, `e = E/(N t)` | `E_UHF/(N t)` | `E₀/(N t)` | `0.02` |
| O2 | double occupancy per site, `d = (1/N)Σ_i ⟨n_{i↑}n_{i↓}⟩` | `(1/N)Σ_i n^MF_{i↑} n^MF_{i↓}` | exact expectation | `0.02` |
| O3 | site occupation, `max_i \|Δn_i\|`, `n_i = ⟨n_{i↑}+n_{i↓}⟩` | `n^MF_{i↑}+n^MF_{i↓}` | `= 1` exactly (PH) | `0.02` |
| O4 | magnetization, `max_i \|Δm_i\|`, `m_i = ⟨n_{i↑}−n_{i↓}⟩` | `n^MF_{i↑}−n^MF_{i↓}` | `= 0` exactly (Lieb) | `0.05` |
| O5 | bond order, `max_i \|Δb_i\|`, `b_i = Σ_σ ⟨c†_{iσ}c_{i+1,σ}+h.c.⟩` | from `ρ^MF` | exact expectation | `0.02` |
| O6 | Boolean defect, `D_bool = max_p min(n_p, 1−n_p)` over natural occupations of the spin-resolved 1-RDM | `0` exactly (idempotent) | exact 1-RDM spectrum | `0.05` |

O6 **is** the ModeChart fence as a number: the chart says every mode is determinate; the exact
state says how wrong that is.

**A configuration is CHART-HONEST iff all six are within tolerance.** The tolerance vector is
fixed here and is never revisited. Note the self-limiting structure that protects it from being
gamed: *loosening* a tolerance enlarges the honest set and makes certify-everywhere **harder**
to beat, so a permissive tolerance costs me, it does not help me.

**Normalized error.** `E_tot ≡ max_o ( |error_o| / τ_o )`, in units of tolerances. `E_tot ≤ 1`
is exactly chart-honest. `E_tot` is the single error number used by Q6.

---

## 3. EXACTNESS GATES — numeric, staked, and run BEFORE anything downstream

Nothing in §4–§6 is computed for a configuration that has not passed §3. A gate failure makes
the configuration VOID (excluded and reported as excluded), never a refusal and never a datum.

| Gate | Statement | **STAKED** threshold |
|---|---|---|
| G-E1 | Hamiltonian Hermiticity, `max\|H−Hᵀ\|` | **exactly 0.0** (bitwise) |
| G-E2 | every matrix element connects identical `(N_↑,N_↓)` | **0 violations** (integer) |
| G-E3 | fermionic signs: bit-trick construction vs. an independent explicit Jordan–Wigner build at N = 2, 4 | `≤ 1e-15` max element |
| G-E4a | eigen-residual `‖Hv−E v‖₂ / (‖v‖₂ · max(1,\|E\|))`, N = 8, 10 | `≤ 1e-12` |
| G-E4b | N ≤ 6: independent dense Jacobi vs. Lanczos, `\|ΔE\|/\|E\|` | `≤ 1e-14` |
| G-E4c | the crate's dense solver validated against `ciris_sim_core::linalg::jacobi_eigen` on 3×3/4×4 fixtures | `≤ 1e-14` |
| G-E5a | particle–hole residual `max_{iσ} \|⟨n_{iσ}⟩ − 1/2\|` | `≤ 1e-11` |
| G-E5b | spin-flip residual `max_i \|m_i\|` | `≤ 1e-11` |
| G-E5c | reflection residual `max_i \|⟨n_i⟩ − ⟨n_{N+1−i}⟩\|` | `≤ 1e-11` |
| G-E5d | total spin `\|⟨S²⟩\|` | `≤ 1e-10` |
| G-E6 | non-degeneracy `E₁ − E₀` within the sector, reported per configuration | `≥ 1e-6 t` |
| G-E7 | `U = 0` column against §1.1(i) closed form, and every chart error at `U = 0` | `≤ 1e-12` rel. / `≤ 1e-12` |
| G-E8 | N = 2 column, all 14 U, against §1.1(ii) closed forms (E and d) | `≤ 1e-12 t` |
| G-C1 | SCF convergence `max_{iσ}\|Δn_{iσ}\|` | `≤ 1e-12` |
| G-C2 | chart 1-RDM idempotency `‖ρ²−ρ‖_max` — Booleanity **checked, not assumed** | `≤ 1e-12` |
| G-C3 | variational sanity `E_UHF ≥ E₀` | `≥ −1e-10` |

G-E5b deserves a note: it is *both* an exactness gate on the reference *and* the exact side of
observable O4. The same theorem that certifies the solver is the one that convicts the chart.

---

## 4. THE CERTIFICATE — two candidates, thresholds fixed before any error exists

Both are **computable from the chart's own data alone**. Neither reads the exact state. This is
the whole point: a certificate that needs the answer certifies nothing.

### C1 — the self-audit (the substantive candidate)

The chart estimates its own error in each observable to leading order in `U`, using
second-order (MP2-style) amplitudes over its **own** converged orbitals `{φ^σ_p, ε^σ_p}`:

```
t_{ia,jb}^{σσ'} = ⟨ij|ab⟩ / ( ε_i^σ + ε_j^{σ'} − ε_a^σ − ε_b^{σ'} )
```

from which come `Ê₂` (correlation energy), `d̂₂`, `n̂₂`, `m̂₂`, `b̂₂`, and `D̂₂` (the leading
occupation-number correction, i.e. the chart's own prediction of the ModeChart fence quantity).

> **C1: CERTIFIED iff `|estimate_o| ≤ κ · τ_o` for all six observables, with κ = 0.5.**
> **STAKED: κ = 0.5** — certify only when the chart's own estimate of its error is at most half
> the tolerance. This is the T2 discipline (a gate that consumes its own reference bias)
> transplanted: the safety factor is the stake, and if MP2 is wrong in sign or magnitude the
> FP gate in §5 will convict it.

### C2 — the stability margin (the cheap structural rival)

`λ_min(N,U)` = the lowest eigenvalue of the chart's Thouless stability matrix (Thouless, Nucl.
Phys. 21, 225 (1960)) over both the spin-conserving and spin-flip blocks — how far the converged
determinant is from a saddle.

> **C2: CERTIFIED iff `λ_min(N,U) ≥ 0.5 · λ_min(N,0)`.**
> **STAKED: factor 0.5**, self-normalizing against the same N's non-interacting stability, so no
> free energy scale is introduced. At `U = 0`, `λ_min(N,0) = Δ(N)` from §1.1(i).

**P-C2-FAIL (a forward prediction about the instrument, staked so the outcome is informative
either way).** I predict **C2 fails the plant-refusal gate at large U**. Reason, stated now:
past the Coulson–Fischer point (Coulson & Fischer, Phil. Mag. 40, 386 (1949)) the UHF solution
breaks spin symmetry and becomes *stable again* on the broken branch, so `λ_min` can recover
above threshold in exactly the Mott regime where O4 is a pure lie. If C2 nonetheless refuses the
plant, that prediction is wrong and I will say so in the title line of the results.

**No third candidate will be introduced after data exist.** Two is the cap and this is it.

---

## 5. THE GATES, AND THE MUTATION TEST

### 5.1 Classification

Per configuration, given a criterion and the §2.1 tolerance vector:

| | chart honest (`E_tot ≤ 1`) | chart wrong (`E_tot > 1`) |
|---|---|---|
| **certified** | TP | **FP — the certificate lying. Fatal.** |
| **refused** | FN (conservative; costs coverage) | TN |

### 5.2 The joint gate — a criterion PASSES iff all four hold

1. **`FP = 0`.** Not "few". Zero. A certificate that ever certifies a wrong chart is dead.
2. **`coverage ≡ TP/(TP+FN) ≥ 0.50`** over chart-honest configurations. This is what
   refuse-everywhere cannot buy.
3. **Every `U = 0` configuration certified** (5 of them). A criterion that refuses the case
   where the chart is provably exact is broken, not conservative.
4. **The PLANT refused**: `U/t = 16` refused at every N (5 of them).

### 5.3 The mutants — RUN and REPORTED, never asserted

| Mutant | Definition | Expected fate | Status |
|---|---|---|---|
| **M1** certify-everywhere | always certify | fails (1) and (4) by construction | kill-relevant |
| **M2** refuse-everywhere | always refuse | fails (2) and (3) | kill-relevant |
| **M3** best fixed cutoff | certify iff `U/t ≤ u*`, `u*` chosen **post hoc to be optimal**, every advantage given | reported | severity baseline |
| **M4** best scaling cutoff | certify iff `U/t ≤ a + b/(N+1)`, `(a,b)` fitted **post hoc** | reported | severity baseline |

M1 and M2 are the commission's mutation test and both must be *shown* to fail, in the results,
with their numbers. M3 and M4 are my own adversaries: if a post-hoc-tuned cutoff on the sweep
parameter matches a chart-internal criterion, the criterion is *correct but uninformative*, and
§7 fixes in advance that this is what I will report.

**P-Q5-N (staked forward prediction about the certified region).** If C1 is doing real work
rather than reading `U` in disguise, its certified boundary must **shrink with N**, because
`Δ(N)` shrinks (§1.1(i)). Staked: `U*_certified(N)` is non-increasing in N across
`N = 2,4,6,8,10`, with `U*(2) / U*(10) ≥ 2.0`. A flat boundary would mean C1 is a fixed-U cutoff
wearing a lab coat.

### 5.4 KILL — Q5, separable, stated absolutely

> **The Q5 kill FIRES iff neither C1 nor C2 passes all four clauses of §5.2.**
> Then the certificate is decoration: record DEAD, keep marked, and say so in the title line.
> The kill is stated absolutely rather than relative to M1 because M1 fails clauses (1) and (4)
> by construction — "beats certify-everywhere" would be too weak a bar and I am not taking it.

This kill takes down the certificate and **nothing else**. It does not touch Q6, does not touch
`Core/ModeChart.lean`, and makes no claim about mean-field theory in general — only about
whether *these two staked criteria* on *this family* separate honest from lying.

---

## 6. Q6 — THE SHARE AS THE FAILURE INSTRUMENT (separable, prereg'd here, own kill)

Q6 reads **only the exact state**. It never touches the chart's data, so its verdict is
independent of Q5's.

### 6.1 The statistic, and a derivation done before the instrument exists

Slots: the `2N` Boolean spin-orbital occupancies `x_{iσ} = n_{iσ} ∈ {0,1}`. The exact ground
state gives an exact probability distribution over Fock-basis strings, `P(x) = |ψ(x)|²`.

> **Named honestly, once:** this is a **classical statistic of a quantum state** — the Fock-basis
> measurement distribution. It is not a quantum-sector reading and will not be described as one.

**Primary statistic — `B4`, the beyond-pair information of a 4-slot marginal:**

```
B4(Q) = D( P_Q ‖ Q_maxent )  =  H(Q_maxent) − H(P_Q)      [nats]
```
where `Q_maxent` is the maximum-entropy distribution on the 16 cells matching all 1- and
2-marginals of the quadruple `Q`. Aggregate: **`B4_mean`**, the mean over **all** `C(2N,4)`
quadruples (4 845 at N = 10; the full grid, because *the equally-spaced diagonal is provably
blind* — house lesson, and there is no sampling cost here to justify a subset).

**Primary predictor: `ΔB4 = B4_mean(N,U) − B4_mean(N,0)`.** Justified, not patched: at `U = 0`
the state is a Slater determinant, whose Fock-basis distribution is a determinantal process with
genuinely non-pairwise structure, and the fixed `Σ_i x_{iσ} = N/2` constraint adds more (*a
global linear constraint on the slots manufactures higher-order structure* — house lesson, named
here before it can be discovered as a surprise). All of that is structure the chart handles
**exactly**. Subtracting the `U = 0` column removes exactly the part that is not correlation.
`B4_mean` raw and `B4_mean/B4_mean(0)` are **declared secondaries that cannot change the verdict.**

### 6.2 The plumb line — a theorem-predicted machine zero, derived here

The ground state is unique in the `(N/2,N/2)` sector and particle–hole symmetric, so
`P(x) = P(x̄)` under global complement. Marginalizing preserves that. On **three** slots:
complement symmetry leaves `p_{000}=p_{111}, p_{001}=p_{110}, p_{010}=p_{101}, p_{100}=p_{011}`
— 3 free parameters after normalization; and it forces all three fields of the pair-maxent
family to vanish, leaving exactly 3 couplings to match exactly 3 pair marginals (the 1-marginals
are all 1/2 and carry no information). Parameter count matches constraint count, so
`Q_maxent = P` exactly:

> **`I_C^(3) ≡ 0` on every configuration of this family, exactly.**

On **four** slots the same count gives 7 free parameters against 6 couplings — one residual
degree of freedom, which is precisely `B4`. This is why the primary statistic is at k = 4 and
not k = 3, and it is consistent with the programme's sign-symmetry result (odd orders killed,
even orders survive from four slots up).

> **G-Q6-PLUMB (STAKED): the instrument must read `|I_C^(3)| ≤ 1e-12` at every configuration.**
> A failure here means the estimator is broken and **Q6 is VOID, not falsified** — no verdict
> until repaired. This is the Planck-plumb-line move: a column whose exact value is known in
> advance validates the pipeline.

### 6.3 Estimator and its gates

Exact marginals from the wavefunction — **no sampling**, so there is no finite-N floor, no
permutation floor for the estimator, no tie fraction, and no `b ≥ 3` binning anywhere (the slots
are natively binary). The floor here is numerical, not statistical, and saying so is a real
advantage of using an exact reference.

- Solver: Newton on the 10 dual parameters (4 fields, 6 couplings) of the pair-maxent family.
- **G-Q6-1 (STAKED): max marginal residual `≤ 1e-13`.** A quadruple that fails is VOID and
  counted; if `> 20%` of quadruples are VOID at a configuration, that configuration's `ΔB4` is
  VOID and reported as such.
- **No data-dependent exclusion.** Quadruples are never dropped for having small cell
  probabilities — only for solver non-convergence. (The Mott end drives cells like
  `n_{i↑}=n_{i↓}=1` toward zero; the min-cell-probability distribution is reported as a
  diagnostic, not used as a filter.)
- Entropies accumulated in log space with Kahan summation (*near-ceiling numerics* lesson).

### 6.4 The staked predictions

The naive test — "`ΔB4` correlates with chart error across the sweep" — is nearly a tautology,
since both rise with `U`. *A residual is never support.* So the sweep's **second axis** carries
the test: chart error at fixed `U` grows with `N` (via `Δ(N)`), and a genuine instrument must
track that too. Q6 statistics use `N ∈ {4,6,8,10}` (N = 2 has exactly one quadruple and is
excluded from aggregates; stated now).

- **P-Q6-A (the across-N test).** Partial Spearman `ρ_s( ΔB4 , E_tot | U/t ) ≥ +0.50` with
  `p < 0.01` under a **within-U-column permutation null** — permute the `ΔB4` labels among the
  four N values *inside each U column*, 10 000 draws, exact p reported. This null preserves the
  U-monotone structure exactly and tests only the across-N information, which is the claim.
  **p is quoted, not z** (house rule; and the null's shape is reported before the p).
- **P-Q6-B (the collapse).** Isotonic regression of `E_tot` on `ΔB4` has median absolute residual
  `≤ 0.70 ×` that of isotonic regression of `E_tot` on `U/t`, over all 56 configurations. Same
  estimator class both sides, so the comparison is like-for-like.
- **P-Q6-C (the boundary — the sharpest).** Evaluate `ΔB4` at each N's **last chart-honest**
  configuration. If `ΔB4` is a genuine instrument these are one critical value; if it is a
  thermometer they are four different ones. Staked: **coefficient of variation across the four
  N values `≤ 0.35`.**

### 6.5 KILL — Q6, separable, its own

> **The Q6 kill FIRES iff any of:**
> (a) `ρ_s < 0.20` or `p > 0.05` in P-Q6-A; **or**
> (b) `ΔB4 ≤ 1e-10` (numerical floor) at any configuration with `E_tot ≥ 3` — share at floor
>     where the chart is badly wrong; **or**
> (c) `ΔB4 ≥` the median of the chart-honest configurations' `ΔB4`, at any configuration with
>     `E_tot ≤ 0.5` — large share where the chart is fine.
>
> Firing kills **instrument-as-certificate** and touches nothing in Q5, nothing in the stance.

---

## 7. THE MEANING OF EVERY OUTCOME — fixed before any result

Nothing below may be renegotiated after data exist.

### Q5

- **(a) C1 passes §5.2, M1 and M2 fail, and C1's coverage exceeds M3's.** The certified seam is
  real and earns its structure: a chart auditing itself from its own data separates honest from
  lying better than reading the sweep parameter. This is the deliverable.
- **(b) C1 passes §5.2 but M3 (or M4) matches or beats its coverage.** The certificate is
  **correct but uninformative** — the refusal is sound, and the chart-internal *warrant* is not
  supported, because a post-hoc cutoff on `U` does the same work. Reported in exactly those
  words, in the title line. Not a kill, not a success, and no rescue by adding a third criterion.
- **(c) C1 fails `FP = 0`, C2 passes §5.2.** C1 dead (with the FP-producing observable named);
  the surviving certificate is the structural stability margin, and **P-C2-FAIL was wrong** —
  said as loudly as a confirmation.
- **(d) Neither passes.** **Q5 KILL FIRES.** The certificate is decoration on this family with
  these criteria. Record dead, keep marked. What survives is the exact reference and its gate
  ladder, which is reusable.
- **(e) A criterion refuses `U = 0`.** That criterion is broken, not conservative; reported as a
  construction defect, and it fails clause (3) regardless of its other numbers.
- **(f) The honest set is empty — even `U = 0` is out of tolerance.** Impossible by G-E7
  (mean field is exact at `U = 0`). Observing it means the instrument is broken: **VOID, not a
  kill.**
- **(g) The honest set is everything — even `U/t = 16` is within tolerance.** Impossible by
  Lieb's theorem via O4 (§1.1(iii)). Observing it means the instrument is broken: **VOID, not a
  kill.**
- **(h) P-Q5-N fails — C1's certified boundary does not shrink with N.** C1 is a fixed-U cutoff
  in disguise; report it as such even if C1 passes §5.2, and read outcome (b).

### Q6

- **(a) P-Q6-A, B and C all confirmed.** The beyond-pair share of the exact state predicts where
  the Boolean chart fails, on this family, in a way that survives controlling for the sweep
  parameter. Scope: **one model family, N ≤ 10, 1D, half filling.** No wild claim, no stance edit
  from this file — findings go to the integrator (research first, then stance).
- **(b) A confirmed, B and/or C fail.** The share is a **thermometer, not an instrument**: it
  rises with `U` alongside the error and carries no independent information about where the chart
  breaks. Reported as NOT support.
- **(c) Any kill clause in §6.5 fires.** Instrument-as-certificate is dead, marked, separable —
  Q5's verdict stands untouched whatever it is.
- **(d) G-Q6-PLUMB fails (`|I_C^(3)| > 1e-12`).** The estimator is broken: **Q6 VOID**, no
  verdict at all, and the first hypothesis to check is my §6.2 parameter count, not the physics.

### Both

- **Any §3 gate failing at a configuration** makes that configuration VOID. If more than 10 of
  the 70 configurations are VOID, the campaign is reported as **UNDERPOWERED** and neither kill
  is adjudicated — a kill may not fire on an instrument that could not run.

---

## 8. HOUSE GATES CARRIED

1. **Pre-registration.** This file, committed before the crate exists. Every threshold marked
   **STAKED** and every prediction marked **P-** is frozen at this commit.
2. **Separable kills.** Q5's kill (§5.4) touches the certificate only. Q6's kill (§6.5) touches
   the instrument only. Neither touches the reference, `Core/ModeChart.lean`, or the stance.
3. **Null matched to the generative structure.** §6.4's null permutes within U columns, because
   the U-monotone co-trend is the confound. No iid null anywhere.
4. **No `b ≥ 3` binning.** Slots are natively Boolean.
5. **p, not z** (§6.4), with the null's shape reported before the p.
6. **A residual is never support.** P-Q6-A/B/C are advance predictions with thresholds fixed
   here; a post-hoc-fitted trend would be reported as fitting, never as confirmation.
7. **Fired kills reported as loudly as survivals**, in the title line of `Q_SEAM_RESULTS.md`,
   with the dead criterion kept and marked.
8. **Named denominators.** "coverage" is over chart-honest configurations only; "70
   configurations" is 5 N × 14 U; Q6 aggregates are over 56 (N ≥ 4).
9. **Received numbers.** Every number in this file is derived here in closed form (§1.1) or is a
   stated choice. Nothing is taken from a sibling agent unverified.
10. **Detached compute.** No run here is expected to exceed minutes (worst case: 63 504-dim
    Lanczos, then 4 845 quadruples × 63 504 basis states ≈ 3e8 accumulations). **If any run
    exceeds 5 minutes it is relaunched under `setsid` with a done-marker and a `RESUME.md`**, per
    the standing rule. N = 12 is out of scope (§0/D3).
11. **Shared-tree hygiene.** Commits use pathspec only (`git commit -- <files>`), never bare
    `add`+`commit`. `sim_engine/Cargo.toml` is not touched by me (§0/D2).
12. **No stance or Lean edits.** Findings return to the integrator to cash.

---

## 9. PRIOR ART — credited generously, because the borrowed parts are borrowed

*Convergence findings are hits, not strikes.* Nothing in §1, §2 or §6.1's estimator is new:

- **Hubbard chain, exact solution**: Lieb & Wu, PRL 20, 1445 (1968). Half-filled bipartite
  ground-state spin: Lieb, PRL 62, 1201 (1989).
- **Mean-field symmetry breaking**: Coulson & Fischer, Phil. Mag. 40, 386 (1949). **Stability
  analysis**: Thouless, Nucl. Phys. 21, 225 (1960); Seeger & Pople, JCP 66, 3045 (1977).
- **Natural occupation numbers as the correlation measure**: Löwdin, Phys. Rev. 97, 1474 (1955).
- **Connected information / beyond-pair maxent**: Schneidman, Still, Berry & Bialek, PRL 91,
  238701 (2003); Amari, IEEE TIT 47, 1701 (2001). The `k=3` share as a contingency-table deviance
  is Bartlett (1935).

**What is not borrowed, and is the actual deliverable:** the *certificate* — a refusal criterion
computable from the chart's own data, staked before the instrument existed, mutation-tested
against certify-everywhere and refuse-everywhere, and adversarially compared against a
post-hoc-optimal cutoff. Everyone measures mean-field error; the product here is the machine
declining to speak.

---

## 10. KNOWN HOLES — not gated, and said now rather than discovered later

1. **Tolerances are chosen, not derived.** §2.1's τ vector has no first-principles warrant. The
   protection is structural (§2.1's self-limiting note, and `FP = 0` being unforgiving), not
   principled. A reader who rejects the τ vector rejects the whole Q5 reading, and that is fair.
2. **One family.** Every Q5 and Q6 statement is scoped to the 1D open Hubbard chain at half
   filling, N ≤ 10. Doping, two dimensions, and long-range interaction are untested and will not
   be spoken about.
3. **`κ = 0.5` and the `0.5·λ_min(N,0)` factor are single points**, not scans. Scanning them
   after seeing errors would be tuning; they are fixed here and stay fixed.
4. **MP2 over UHF orbitals is not size-consistent with the broken-symmetry branch** in the usual
   textbook sense, and its estimates past the Coulson–Fischer point may be poor. That is a
   *reason C1 might fail*, and it is named in advance rather than offered afterwards as an excuse.
   — **SUPERSEDED BY A1/H1 (2026-08-23): the mechanism stated here is backwards.** The failure is
   not that MP2 becomes *poor*; it is that MP2 becomes *small and confidently wrong*. Kept, marked,
   per rule 7. See A1/H1.
5. **Q6 says nothing about wild systems.** It is a classical statistic of a model quantum state.

---

## 11. FILES

Crate `sim_engine/crates/q-seam` (workspace member pending §0/D2):
`src/hubbard.rs` (basis, sparse H, symmetry operators) · `src/lanczos.rs` + `src/dense.rs`
(ground state, residual gates) · `src/chart.rs` (UHF SCF, stability matrix, MP2 self-estimate) ·
`src/certificate.rs` (C1, C2, M1–M4, scoring) · `src/share.rs` (pair-maxent Newton, `B4`,
`I_C^(3)`) · `src/bin/q_seam_run.rs` (the sweep) · `tests/` (§3 gates as `#[test]`).

Outputs → `sim_engine/output/q_seam/*.json`. Verdict → `sim_engine/Q_SEAM_RESULTS.md`.

---

# AMENDMENT A1 — adversarial review, adopted 2026-08-23, BEFORE any instrument exists

**Still no crate, no ground state, no error, no share.** A1 was written in response to the team
lead's attack on the frozen text above and is committed before the first line of `q-seam` is
written. It **supersedes** §10 hole 4 (marked in place, not deleted) and **adds** to §4, §5, §6
and §7. Nothing already staked is loosened: every threshold frozen above stays frozen, and A1
introduces exactly **one** new numerical convention, which is the reuse of the already-staked
`κ = 0.5`.

## A1/H1 — C1 is blind to symmetry-breaking failure, and the mechanism is the reverse of §10.4

**Adopted, and sharpened.** The lead's correction is right and my §10.4 had it backwards. Past
the Coulson–Fischer point the broken UHF determinant is self-consistent with its own gap of
order `U`, so its own MP2 numerators are suppressed (the on-site interaction couples opposite
spins, whose occupied orbitals have localized onto *different* sublattices, killing the overlap)
while its denominators grow like `2U`. **The amplitudes go to zero. A self-consistent lie audits
clean.**

The sharper statement, which is the real content and the reason C3 below is principled rather
than ad hoc:

> **C1's self-audit estimates the CORRECTION to the chart's prediction. The certificate needs the
> DEVIATION from the truth. For an observable whose exact value is pinned by a symmetry the chart
> has broken, those two are not the same object.** At `U/t = 16` the chart's error in O4 is not a
> perturbative correction to `m^MF`; it is the whole of `m^MF`, an order-parameter-sized quantity
> that no order-by-order expansion *around the broken determinant* can see.

So C1's blindness is specific and pre-nameable: **C1 can see perturbative failure and cannot see
symmetry-breaking failure.** Restated as a prediction, replacing §10.4:

> **P-C1-FAIL (STAKED).** C1 certifies the plant and fails clause (4), at every N where UHF has
> broken symmetry. Combined with the already-staked **P-C2-FAIL**, this means: *as frozen above,
> I expect the Q5 kill to fire on both original candidates.*

That is exactly why C3 is added now and not later. If both blind candidates failed and the kill
fired, the recorded verdict would have been "a chart cannot certify itself" when the true verdict
is "these two chart-internal quantities are both blind to the same failure mode." **Adding C3
before any data exists is what freeze-design-early is for**; adding it afterwards would have been
tuning, and would not have been allowed.

### C3 — the theorem-pinned observables (new candidate, chart-data-only)

The finite reference provably keeps three symmetries (§1.1(iii) and §3): particle–hole
(`⟨n_{iσ}⟩ = 1/2`), spin (`m_i = 0`, Lieb 1989), and left–right reflection. Each pins an exact
value. And here is the fact that makes this a certificate rather than a heuristic:

> **For an observable whose exact value is fixed by a theorem, the chart can compute its own
> error EXACTLY, from its own data, without ever touching the exact state.** The chart's error in
> O4 *is* `max_i |m^MF_i|`. Its error in O3 *is* `max_i |n^MF_i − 1|`. No estimate, no expansion,
> no reference state — a theorem supplies the truth and the chart supplies the deviation.

> **C3: CERTIFIED iff all three hold —**
> `max_i |m^MF_i| ≤ κ·τ_m = 0.025`, `max_{iσ} |n^MF_{iσ} − 1/2| ≤ κ·τ_n = 0.010`,
> `max_i |n^MF_i − n^MF_{N+1−i}| ≤ κ·τ_n = 0.010`.
> **No new constant**: `κ = 0.5` and the `τ` vector are the ones already frozen in §2.1 and §4.

Honesty notes, written next to it as instructed:

- **C3 refuses the plant near-trivially** (`|m^MF| → 1 ≫ 0.025`). Clause (4) is therefore *not*
  informative for C3. Its informative clauses are **FP = 0**, **coverage ≥ 0.5**, and **beating
  M3/M4**; and **P-Q5-N is its real test** — the Coulson–Fischer boundary genuinely moves with N
  (it tracks `Δ(N)`, §1.1(i)), so a flat certified boundary convicts C3 as a U-cutoff in a lab
  coat exactly as §7(h) says.
- **C3 is blind to everything C1 can see.** It checks O3 and O4 only. It says nothing about the
  energy, the double occupancy, the bond order or `D_bool`, so it will certify any configuration
  where the chart is symmetric but perturbatively wrong.
- **P-C3-FAIL (STAKED, and I name the direction).** C3 is at risk of FPs in the pre-breaking
  window, where the correlation-energy error exceeds `τ_E = 0.02` while the chart is still
  symmetric. RHF's error is already ≈ `0.018 t` at `U/t = 0.5` and ≈ `0.07 t` at `U/t = 1`, and
  the UHF instability sits near `U_c ~ Δ(N)`. **I predict C3 passes FP = 0 at N = 2, 4 and is at
  risk at N = 8, 10**, where the energy boundary may fall below the symmetry boundary. **The
  escape I am explicitly not taking: I will not add an energy clause to C3 after seeing data.**

### C4 — the conjunction (declared now, flagged as beyond the instruction)

C1 and C3 are blind in complementary directions, which is a structural fact, not an observation.
The conjunction is therefore the criterion the problem actually points at:

> **C4 = C1 ∧ C3.** CERTIFIED iff both certify. **Introduces NO new threshold and no new
> constant** — it is the conjunction of two criteria already frozen. **P-C4 (STAKED): C4 is the
> one I expect to pass all four clauses.**

**Flagged for the lead as an addition beyond the instruction**, which said to add C3. It is
staked pre-data with zero free parameters; if it had been introduced after seeing errors it would
be tuning and inadmissible. Veto it and the campaign runs C1/C2/C3 unchanged.

**Anti-shopping clause.** Four candidates is four shots, and the protection is stated now:
`FP = 0` is absolute, every threshold is frozen pre-data, each candidate's expected fate is
staked above (**P-C1-FAIL, P-C2-FAIL, P-C3-FAIL, P-C4**), and **the results title line names the
fate of all four, never only the survivor.** In particular: **a pass by C4 alone is reported as
"the conjunction passed; neither component did" — never as "the certificate works."**

### §5.4 KILL, re-worded for four candidates

> **The Q5 kill FIRES iff NONE of C1, C2, C3, C4 passes all four clauses of §5.2.**

## A1/H2 — `D_bool` is removed from Q6's error target (partial circularity)

**Adopted.** `D_bool`'s chart prediction is `0` *structurally* — by idempotency of the chart's
1-RDM, not by any physical claim — so "the chart's error in O6" is definitionally a correlation
functional of the exact state. Correlating `ΔB4` against an `E_tot` containing it is correlating
one correlation functional of the exact state against another, and it taints P-Q6-A/B/C.

> **Q6's primary error target is `E5 ≡ max over {O1 e, O2 d, O3 n_i, O4 m_i, O5 bond} of
> |error_o| / τ_o`.** The `D_bool`-inclusive `E_tot` is reported for Q6 as **diagnostic only** and
> cannot change any Q6 verdict.
> **Q5 keeps `E_tot` with `D_bool`** — as a certification observable it is a legitimate honesty
> reading. The split is only about what Q6 correlates against.

The precise distinction, since it is the content of the hole: O6 is excluded because its chart
prediction is a *structural zero*. O2 (double occupancy) is a two-point correlator of the exact
state but is **kept**, because the chart makes a real, varying, falsifiable prediction for it. And
after the removal, `E5` contains two clauses (O3, O4) that are **pure chart quantities** with no
dependence on the exact state at all — the opposite of circular.

**Consequential edits, stated so nothing is left ambiguous:** every Q6 statistic uses `E5` —
§6.4's P-Q6-A and P-Q6-B, §6.4's P-Q6-C ("last chart-honest" now means **last `E5`-honest**), and
§6.5's kill clauses (b) and (c) (`E5 ≥ 3` and `E5 ≤ 0.5`). Q5's honest set (six observables) and
Q6's honest set (five) may differ; **both cardinalities are reported.**

## A1/P1 — Lanczos determinism (pinned)

Replay discipline applies to instruments. **Staked now:**

- Start vector `v0` from a **named, seeded PRNG (SplitMix64, seed `0x515F_5EAM_0000_0001`)**,
  entries uniform on `[−1,1]`, normalized, over the sector basis in canonical sorted order. A
  fixed all-ones start is **rejected**: it is a symmetry eigenvector and can be exactly orthogonal
  to a ground state of the opposite parity under global spin flip, which is a convention-dependent
  hazard I will not take.
- **Full reorthogonalization** against all previous Krylov vectors. Iteration cap 500.
- **Orthogonality guard:** report `|⟨v0|ψ0⟩|`; if `< 1e-8` the start was near-orthogonal.
- **Deterministic restart policy:** if the G-E4a residual gate or the orthogonality guard fails,
  restart **once** with `seed + 1` and **report the restart**. A second failure VOIDs the
  configuration. No adaptive retries, no unbounded restarts.
- Every seed, iteration count and residual is written to the run JSON, so the ladder is replayable.

## A1/P2 — Newton failure VOIDs the configuration (pinned, stricter than the frozen text)

**Adopted, replacing §6.3's 20% rule**, which was a threshold on a data-dependent quantity and
therefore a researcher degree of freedom I should not have left myself.

> **Any quadruple that fails to converge VOIDs its whole configuration for Q6**, counted toward
> the §7 underpowered clause. Never a zero, never a datum.

Engineering mitigation (this is the right response to the strictness, not a loosened rule):
near-deterministic marginals push the maxent MLE to the boundary of the exponential family, where
the dual parameters diverge — that is mathematics, not solver weakness, and IPF converges to the
closure where Newton cannot. **Staked:** Newton first; on failure, IPF (capped, stated); a
quadruple counts as converged iff **either** meets the `≤ 1e-13` marginal-residual gate; VOID only
if both fail. **And the house lesson is carried explicitly** — the programme has caught IPF
*one-sidedly overstating* the share on near-deterministic states, so where both converge they must
agree on `B4` to `≤ 1e-10`, disagreement VOIDs, and **the fraction of quadruples reached only by
IPF is reported per configuration** as the boundary diagnostic. If this triggers the underpowered
clause, that is reported as underpowered — not worked around.

## A1/P3 — M3/M4 objective (pinned)

> **M3 and M4 maximize coverage subject to `FP = 0` AND the plant refused at all five N.**
> Tie-break: the **most conservative** admissible cutoff (smallest `u*`; for M4, smallest `a`).
> If no cutoff satisfies the constraints, **M3 (or M4) is INFEASIBLE and is reported as such** —
> which is itself informative, since it would mean no cutoff on the sweep parameter can do the
> job, and strengthens whichever chart-internal criterion passes.

## A1/P4 — N = 2 in Q6 (pinned: OUT, with the reason)

`2N = 4` slots gives exactly `C(4,4) = 1` quadruple, so `B4_mean` performs no aggregation; and
the sector constraints `Σ_i x_{i↑} = Σ_i x_{i↓} = 1` over two sites pin nearly the whole 16-cell
table, so what `B4` reads there is the constraint, not correlation. **N = 2 is OUT of every Q6
correlation, weight zero, and this was the intent of §6.4's "N ∈ {4,6,8,10}".** Its value is still
computed and reported as a **diagnostic** — it is a useful check that the estimator behaves at the
boundary of the exponential family, which is precisely where A1/P2 says the risk lives.

## A1/F — the free claim, taken

**G-E7 is the large-dimension solver validation, and the exactness ladder now says so.** The
3×3/4×4 `jacobi_eigen` fixtures (G-E4c) validate the dense solver at toy size and cannot reach
dimension 63 504. The `U = 0` closed-form column of §1.1(i) does: it pins the Lanczos ground-state
energy against an analytic value at **all five N, including dim 63 504**, and the N = 2 analytic
column (G-E8) pins it across the whole `U` sweep. Together they are the validation of the
large-dimension path. No code change; the claim was already owned and is now stated.

## A1 — deviations resolved

D1 (residual ladder) and D3 (N = 12 out) **accepted by the lead as written**. D2 resolved: the
`sim_engine/Cargo.toml` member line is the **lead's** to add, by pathspec, once the crate skeleton
compiles; until then `q-seam` builds with an explicit `--manifest-path`.

## A1 — outcome meanings, extended (§7 additions)

- **(i) C4 passes, C1 and C3 individually fail.** Reported as **"the conjunction passed; neither
  component did"**. The reading: chart-internal certification works, but only when a perturbative
  auditor and a theorem-pinned auditor are *composed* — because each is blind where the other
  sees. Still subject to (b): if M3/M4 match C4's coverage, it is correct-but-uninformative.
- **(j) P-C1-FAIL is wrong — C1 refuses the plant.** Then the broken branch's MP2 amplitudes are
  not suppressed the way A1/H1 argues, and **A1/H1's mechanism is refuted**; report it in the
  title line as loudly as a confirmation, and say that §10.4's original wording, though wrong in
  its own reasoning, reached the right expectation for the wrong reason.
- **(k) P-C3-FAIL fires as predicted (C3 has FPs at N = 8, 10).** C3 dead as a standalone
  criterion, kept and marked; C4 is unaffected only if C1's energy clause catches those same
  configurations, which is exactly what the conjunction is for and is therefore a real test of it.

---

# AMENDMENT A2 — a staked gate fired on my own threshold, 2026-08-23

**Still no chart, no certificate, no share, no error.** A2 covers only the exactness ladder (§3),
which by design runs before anything downstream. It reports one **missed gate**, one **solver
defect the gate caught**, one **threshold I am changing after seeing data** — declared as such —
and one **typographical correction**. No tolerance, no certificate threshold, no prediction and
no kill condition is touched.

## A2/M — G-E4b MISSED as staked. Kept, marked.

> **G-E4b as frozen: `|ΔE|/|E| ≤ 1e-14` between Lanczos and an independent dense
> diagonalization at N ≤ 6. MEASURED: 9.6e-14 at N = 6, U = 0. THE GATE FIRED.**

Reported as loudly as a pass, per rule 7. Nine of the ten implemented gates passed on the first
run; this one did not, and what it caught was **my threshold**, not the physics.

### What the arbiter said

The `U = 0` closed form (§1.1(i)) is exact to machine precision and independent of both solvers,
so it adjudicates. At N = 6 (dim 400), relative error against the analytic value:

| solver | vs. analytic at N = 6, U = 0 |
|---|---|
| Lanczos | **1.11e-14** |
| dense cyclic Jacobi (eigenvalue) | **1.07e-13** |

The disagreement was dominated by the **dense** side. That is the expected `O(n)·ε·‖A‖` accumulation
of cyclic Jacobi — at n = 400, `400 × 2.2e-16 × 7 ≈ 6e-13`, and 1.07e-13 sits inside it.

### The solver defect the gate caught, and its fix

The first printing of `dense.rs` stopped sweeping at `off ≤ 1e-15 · √(Σ diag²)`, which scales like
`√n` and therefore *loosens as the matrix grows*. That was a real defect and it is fixed
(convergence now measured against the full Frobenius norm, plus a stagnation break). **Measured
after the fix: unchanged to every printed digit** — the solver was already stagnating at its
arithmetic floor, so the criterion was not the binding constraint. **The defect was real and was
not the cause**, and both halves of that sentence are reported.

The cause was structural, and so is the repair: the cross-check now takes the dense **eigenvector**
and forms its **Rayleigh quotient through the exact operator**, which is second-order in the
eigenvector error and so does not accumulate the eigenvalue rounding. Measured:

| N (dim) | dense eigenvalue vs. Lanczos | Rayleigh quotient vs. Lanczos | RQ vs. analytic |
|---|---|---|---|
| 2 (4) | 3.3e-16 | 1.1e-16 | 2.2e-16 |
| 4 (36) | 3.6e-15 | 4.8e-15 | 2.0e-16 |
| 6 (400) | 9.6e-14 | **1.3e-14** | **2.2e-15** |

## A2/T1 — the replacement threshold, and the fact that I set it after seeing data

> **G-E4b (amended): the dense eigenvector's Rayleigh quotient through the exact operator agrees
> with Lanczos to `|ΔE|/|E| ≤ 1e-13` at N ≤ 6. The raw dense eigenvalue comparison is retained
> and REPORTED as a diagnostic, no longer gated.**

**This threshold was chosen after seeing data, and saying so is the point.** Three things bound
what that licence is worth:

1. **It is not fitted to the observed disagreement.** 1e-13 comes from the *arbiter*, not from
   the 1.3e-14 I measured: the two solvers' own errors against the analytic value are 1.11e-14 and
   2.16e-15, so any honest cross-check threshold must exceed their sum, and 1e-13 is one order
   above the worst single-side error with the `O(n)·ε` scaling argument behind it.
2. **It is an instrument gate, not a scientific stake.** It certifies that Lanczos converged to
   the right state. It is not a tolerance, not a certificate threshold, not a prediction.
3. **The gates that actually warrant the reference are untouched and all pass**: G-E4a (residual
   `≤ 1e-12`), G-E7 (analytic `U = 0` column, `≤ 1e-12`, at all five N **including dim 63 504**),
   G-E8 (analytic N = 2 column at all 14 U, `≤ 1e-12`). None moved.

**What I got wrong, stated once:** I staked G-E4b from the Lanczos accuracy class and applied it
to a *comparison*, whose error budget is the sum of two solvers' errors and grows with dimension.
Two independent eigensolvers cannot agree to 1e-14 at dim 400. That was foreseeable before the
run and I did not foresee it.

## A2/T2 — typographical correction: the pinned seeds were not valid hexadecimal

A1/P1 printed the Lanczos start seed as `0x515F_5EAM_0000_0001` and §2 printed the SCF guess seed
as `0x5EAM`. **`M` is not a hexadecimal digit**, so neither literal exists. Corrected, with the
intent preserved:

- Lanczos start seed: **`0x515F_5EA0_0000_0001`** (`lanczos::START_SEED`).
- SCF pseudorandom guess seed: **`0x5EA0`**.

A typo in a pinned constant is still a pin that could not be honoured, so it is corrected in the
open rather than silently in the code.

## A2 — status of the ladder after this amendment

Implemented and **passing**: G-E1 (bitwise 0.0), G-E2 (0 violations), G-E3 (nearest-neighbour JW
signs are `+1` on every state of every chain — *and* the general string is shown to fire negative
on a non-adjacent hop, so the gate is not vacuous), G-E4a, G-E4b (amended), G-E4c, G-E6, G-E7,
G-E8, plus a test that the §1.1(i) table printed in this document is itself reproduced.
Not yet implemented (they need the chart and the share): G-E5a–d, G-C1, G-C2, G-C3, G-Q6-PLUMB,
G-Q6-1.

**A2 reproducibility.** Every number in A2's two tables is printed by
`crates/q-seam/examples/a2_solver_arbitration.rs` (raw dense vs. Lanczos vs. analytic) and
`crates/q-seam/examples/a2_rayleigh_check.rs` (the Rayleigh-quotient form). `examples/scaling_n10.rs`
prints the dim-63 504 timings and residuals. Nothing in A2 is asserted without a runnable source.

---

# AMENDMENT A3 — adversarial review round 2, adopted 2026-08-23

Two rulings from the team lead, both adopted. **One is a rule I had already applied before the
instruction arrived and am now recording; the other required a computation I had not run.** Both
are marked as such, because "we were already doing that" is only worth saying if it is checkable.

## A3/R1 — the severity comparison binds C4 exactly as it binds C1, C2 and C3

Written into the C4 stake, as instructed:

> **A C4 pass that does not beat M3/M4 is reported as CORRECT BUT UNINFORMATIVE.** Two weak
> components can conjoin into a U-cutoff in a lab coat, and the conjunction earns no exemption
> from the severity baselines simply because it is a conjunction.

**Status: already applied.** `Q_SEAM_RESULTS.md` recorded exactly this verdict before A3 existed —
C4 passes the joint gate, M3 beats its coverage 1.000 to 0.667, and the headline reads CORRECT BUT
UNINFORMATIVE. Recording the rule changes no number and no verdict; it removes the possibility
that a future reader thinks the exemption was ever available. The reporting rule stands verbatim:
a C4-alone pass is **"the conjunction passed; neither component did"**, never "the certificate
works".

## A3/R2 — the robustness clause: an amendment may never be what decides a kill

> **STANDING RULE.** Where an amendment changes which configurations are admissible, both
> adjudications are computed and reported. If the kill verdict differs between the frozen reading
> and the amended reading, **the kill is UNADJUDICATED.**

### A3/R2.1 — a factual correction to the ruling's premise

The ruling states the fired G-E4b would VOID **N = 4 and 6**. Measured, over all 14 U values, the
worst raw dense-vs-Lanczos disagreement is:

| N | worst raw disagreement | frozen G-E4b (≤ 1e-14) |
|---|---|---|
| 2 | 1.221e-15 | passes |
| 4 | 4.885e-15 | **passes** |
| 6 | 1.155e-13 | **WOULD VOID** |
| 8, 10 | — | not covered: no dense cross-check exists at those dimensions |

So the frozen reading VOIDs **N = 6 and only N = 6**, 14 configurations of 70. N = 4 clears the
frozen threshold by a factor of two thousand. Reproduced by
`crates/q-seam/examples/frozen_ge4b.rs`.

### A3/R2.2 — both adjudications, computed

**Q5.** Frozen reading (56 configurations, 12 chart-honest):

| criterion | full reading | frozen reading |
|---|---|---|
| C1 | FP=20, plant CERTIFIED, fail | FP=16, plant CERTIFIED, fail |
| C2 | FP=10, fail | FP=8, fail |
| C3 | FP=19, fail | FP=15, fail |
| **C4** | **FP=0, cov 0.667, PASS** | **FP=0, cov 0.667, PASS** |
| M1 / M2 | both fail (as required) | both fail (as required) |
| M3 best fixed cutoff | `U ≤ 0.25`, cov **1.000** | `U ≤ 0.25`, cov **1.000** |
| M4 | degenerates to M3 (`b = 0`) | degenerates to M3 (`b = 0`) |

Criteria passing: `[C4]` under both. Kill fires: **false under both**. Severity under the frozen
reading: C4 0.667 vs M3 1.000 — **still CORRECT BUT UNINFORMATIVE**. The headline is identical.

**Q6.** Frozen reading:

| test | full | frozen |
|---|---|---|
| partial ρ | 0.099, p = 0.334 | 0.193, p = 0.228 |
| isotonic ratio | 25.17 | 15.24 |
| boundary CV | 0.887 | 0.907 |
| clauses firing | (a), (c) | (a), (c) |

Kill fires: **true under both**.

> **VERDICT ON THE ROBUSTNESS CLAUSE: the adjudications AGREE on both legs. Neither kill — nor
> Q5's severity headline — depends on amendment A2.** Nothing is UNADJUDICATED. Had they
> disagreed, both would have been reported and both kills suspended.

## A3/R3 — the √n defect is recorded as its own line

Carried into `Q_SEAM_RESULTS.md` as an item in its own right, per the ruling: the gate caught a
**real** convergence-test defect (a stopping criterion that scaled like `√n` and so loosened as
the matrix grew), **and** that defect was **not** the cause of the miss (fixing it changed nothing
to the last printed digit — the solver was already at its arithmetic floor). Both halves are
stated together, because a record that reports only the half that flatters the process is not a
record.
