# Q-SEAM RESULTS — the certificate is correct but uninformative, and the share instrument is dead

**Verdict up front, both legs, both stated as loudly as a survival would be.**

**Q5 — the kill does NOT fire, and the certificate does not earn its keep.** `C4 = C1 ∧ C3` passes
all four clauses of the joint gate (FP = 0, coverage 0.667, every `U = 0` certified, the plant
refused at all five N). Both mutants fail as the mutation test requires. But the prereg's own
§7(b), written before any data existed, is what applies: **M3, the best post-hoc fixed cutoff
`U/t ≤ 0.25`, achieves FP = 0 with coverage 1.000 against C4's 0.667.** The refusal is sound and
the chart-internal *warrant* is not supported — reading the sweep parameter does the same job,
better. **CORRECT BUT UNINFORMATIVE**, in exactly those words.

**Q6 — the KILL FIRES**, on two of its three clauses. `ΔB4` does not predict where the chart
fails: partial Spearman **0.099, p = 0.334** against a staked `≥ 0.50, p < 0.01`. Instrument-as-
certificate is **DEAD**; recorded, kept, marked. It touches nothing in Q5 and nothing in the stance.

The Q6 kill is a **falsification, not a VOID**, and the reason is the plumb line: `I_C^(3)` was
*derived in the prereg* to be exactly zero on this family, and the instrument reads
**1.213e-13** against a 1e-12 gate. The estimator is validated, so the null it returns is the
family's answer and not the instrument's failure.

**70 of 70 configurations passed every exactness gate. Zero VOID.**

**Neither verdict depends on an amendment.** G-E4b fired against its staked 1e-14 and was amended
(A2); re-adjudicating under the *frozen* reading, which VOIDs N = 6, leaves both verdicts and Q5's
severity headline unchanged (§2.5). The amendment is not what decided anything.

---

## 1. What was staked, and what happened — every prediction, including the ones I got wrong

| Prediction | Staked in | Outcome |
|---|---|---|
| **P-C1-FAIL** — C1 certifies the plant | A1/H1 | **CONFIRMED** |
| **P-C2-FAIL** — C2 fails plant-refusal | §4 | **REFUTED** — C2 refuses the plant |
| **P-C3-FAIL** — C3 has FPs, at N = 8, 10 | A1/H1 | **CONFIRMED in fact, WRONG in direction** — FPs at *every* N |
| **P-C4** — the conjunction passes | A1 | **CONFIRMED** |
| **P-Q5-N** — C1's boundary shrinks with N, `U*(2)/U*(10) ≥ 2` | §5.3 | **FAILED, and its premise was wrong.** The chart's per-site error is **intensive** and converges with N, so the chart-honest boundary sits at `U/t ≤ 0.25` at **every** N. The target boundary does not move — therefore no criterion could track it, and **a fixed cutoff can be perfect**, which is why M3 beats the certificate. **This outranks the certificate verdict for anyone building the next one.** (§2.3) |
| **P-Q6-A** — partial ρ ≥ 0.50, p < 0.01 | §6.4 | **NOT CONFIRMED** (0.099, p = 0.334) |
| **P-Q6-B** — isotonic ratio ≤ 0.70 | §6.4 | **NOT CONFIRMED** (25.17) |
| **P-Q6-C** — boundary CV ≤ 0.35 | §6.4 | **NOT CONFIRMED** (0.887) |
| **G-Q6-PLUMB** — `I_C^(3)` exactly zero | §6.2, derived | **PASS** (1.213e-13) |
| **G-E4b (AS FROZEN)** — raw dense vs Lanczos ≤ 1e-14 | §3 | **FIRED at 9.6e-14** — kept, marked, never silently replaced |
| **G-E4b (A2, Rayleigh-quotient form)** — ≤ 1e-13 | A2/T1 | PASS (worst 2.9e-14) |

Four confirmed, one refuted, one confirmed-with-the-direction-wrong, three not confirmed, one gate
fired on its own threshold. Nothing was reinterpreted after the fact.

---

## 2. Q5 — the joint gate

Chart-honest configurations: **15 of 70** (identical under Q5's six-observable and Q6's
five-observable definitions — `D_bool` never binds first, so A1/H2's de-circularization did not
move the honest set, only what Q6 correlates against).

| criterion | FP | coverage | `U=0` | plant | verdict | certified boundary by N (2,4,6,8,10) |
|---|---|---|---|---|---|---|
| C1 self-audit | 20 | 0.667 | ok | **CERTIFIED** | fail | 16, 16, 16, 16, 16 |
| C2 stability | 10 | 1.000 | ok | refused | fail | 0.75 × 5 |
| C3 theorem-pinned | 19 | 1.000 | ok | refused | fail | 2.0, 1.5, 1.5, 1.0, 1.0 |
| **C4 = C1 ∧ C3** | **0** | **0.667** | **ok** | **refused** | **PASS** | 0.125 × 5 |
| M1 certify-everywhere | 55 | 1.000 | ok | **CERTIFIED** | fail (as required) | 16 × 5 |
| M2 refuse-everywhere | 0 | 0.000 | **FAIL** | refused | fail (as required) | — |
| M3 best fixed cutoff `U ≤ 0.25` | 0 | **1.000** | ok | refused | **PASS** | 0.25 × 5 |
| M4 best `U ≤ a + b/(N+1)` | 0 | **1.000** | ok | refused | **PASS** — and it **degenerates to M3** (`b = 0`) | 0.25 × 5 |

The mutation test did its job: M1 certifies the plant and carries 55 false positives; M2 refuses
`U = 0`. A certificate that never refuses proves nothing, and this one is shown failing rather
than asserted to fail.

### 2.1 Why C1 certifies the plant — A1/H1 confirmed numerically

At N = 8, `U/t = 16`, the chart's own MP2 self-audit reports an energy error of **0.00048** where
the truth is **0.05215**, and a `D_bool` of **0.00005** where the truth is **0.41**. Two and four
orders of magnitude too small, in the direction that matters.

The mechanism is the one A1 sharpened and it is now measured: past the Coulson–Fischer point the
broken UHF determinant is self-consistent with its own gap of order `U`, and the on-site
interaction couples opposite spins whose occupied orbitals have localized onto *different*
sublattices — numerators die, denominators grow, amplitudes vanish. **A self-consistent lie audits
clean.** The deeper statement, which is why C3 exists: C1 estimates the *correction* to the
chart's prediction, while the certificate needs the *deviation* from the truth, and for an
observable pinned by a symmetry the chart has broken those are different objects. This is pinned
as a test (`the_broken_branch_audits_clean_while_lying`), not left as prose.

> **WARRANT WEAKENED BY A LATER CAMPAIGN — read this before carrying the citation forward.**
> Q5 justified `m_i = 0` by **Lieb 1989** (spin-reflection positivity, half filling, bipartite).
> `Q7_SEAM_PREREG.md` §2.2 / D0-d derives the same fact from **spin-independence of `H` plus
> uniqueness of the ground state in the `S_z = 0` sector** — no bipartite lattice, no half filling,
> no particle–hole. **The result here is unchanged; the warrant was over-strong.** It matters
> because the heavy version does not survive a site potential, so a reader carrying the Lieb
> citation forward would conclude the anchor dies under inhomogeneity when in fact it survives.

### 2.2 Why P-C2-FAIL was refuted — and it is not the reason I would have wanted

C2 **does** refuse the plant, so the prediction is wrong and that is reported as loudly as a
confirmation. But not because it detects an instability. A determinant that has broken a
continuous symmetry carries an exact **Goldstone zero mode** — a flat direction, not an
instability — and the criterion as staked reads the raw lowest eigenvalue, which that zero pins to
0. Measured: `null_modes` goes 0 → 1 exactly at the symmetry-breaking transition, and
`λ_min_projected` (the diagnostic, Goldstone removed) recovers to 0.90, 0.65, 0.15 at N = 4 for
`U = 2, 4, 16` — i.e. the *genuine* stability does come back on the broken branch, exactly as
P-C2-FAIL argued. **Taken literally, the stability criterion collapses into the symmetry
criterion.** C2 fails anyway, on FP = 0.

The Hessian validates itself: at `U = 0` its lowest eigenvalue is `2·Δ(N)` at every N, to five
digits — the particle–hole excitation ladder, reproduced by a numerical second difference.

### 2.3 P-Q5-N failed, and the premise was wrong — the campaign's main methodological finding

I argued the chart must break earlier at larger N because the mean-field gap `Δ(N) = 4t·sin(π/2(N+1))`
shrinks. **That was wrong, and the measurement says why.** The chart's per-site error is an
*intensive* quantity that converges in the thermodynamic limit rather than growing: the normalized
energy error at `U/t = 1` runs **1.539, 1.290, 1.175, 1.108, 1.065** across N = 2…10 — decreasing
and converging.

So the chart-honest boundary is `U/t ≤ 0.25` **at every N**. The sweep's second axis does not move
the answer.

Three consequences, all of which bite:

1. **No criterion could have had a usefully N-moving boundary**, because the target boundary does
   not move. P-Q5-N was unachievable by construction, not merely unachieved.
2. **A fixed cutoff can be perfect**, which is exactly what M3 turns out to be. The severity
   baseline I added as my own adversary was handed an advantage by my own design.
3. **This limits what Q5 could ever have shown.** The two-axis test had, in practice, one axis.

That is a design flaw, it is mine, and it was foreseeable: I reasoned from the gap without
checking whether the *error* inherits the gap's N-dependence. It does not.

### 2.4 The gate that fired, and the bug it caught

Recorded as two separate facts, because reporting only the flattering half would not be a record.

**The defect was real.** `dense.rs`'s first printing stopped sweeping at `off ≤ 1e-15·√(Σ diag²)`,
a criterion that scales like `√n` and therefore *loosens as the matrix grows* — at dim 400 it
permitted an off-diagonal norm of ~6e-14. That is a genuine bug in a numerical routine, and the
staked gate is what surfaced it.

**The defect was not the cause.** Fixing it (convergence now measured against the full Frobenius
norm, plus a stagnation break) changed the results **to every printed digit**: the solver was
already stagnating at its arithmetic floor. The real cause was `O(n)·ε` eigenvalue accumulation in
cyclic Jacobi, which no stopping criterion can remove — hence A2's repair, comparing through the
dense eigenvector's Rayleigh quotient instead.

### 2.5 Robustness: the amendment does not decide either kill

Amendment A2 relaxed G-E4b after the gate fired, so the standing question is whether that
amendment is what produced the verdicts. It is not, and this was computed rather than argued
(A3/R2).

Under the **frozen** G-E4b, the gate fires at **N = 6 and only N = 6** — worst raw disagreement
1.221e-15 at N = 2 and 4.885e-15 at N = 4, against 1.155e-13 at N = 6; N = 8 and N = 10 carry no
dense cross-check at all. So the frozen reading VOIDs 14 of 70 configurations.

| | full reading | frozen reading (N=6 VOID) |
|---|---|---|
| Q5 criteria passing | `[C4]` | `[C4]` |
| Q5 kill fires | no | no |
| Q5 severity (C4 vs M3 coverage) | 0.667 vs 1.000 | 0.667 vs 1.000 |
| Q5 headline | correct but uninformative | correct but uninformative |
| Q6 partial ρ (p) | 0.099 (0.334) | 0.193 (0.228) |
| Q6 clauses firing | (a), (c) | (a), (c) |
| Q6 kill fires | **yes** | **yes** |

**The adjudications agree on both legs.** Neither kill, and not Q5's headline, depends on
amendment A2. Had they disagreed, both readings would have been reported and both kills treated as
UNADJUDICATED.

### 2.6 What the certificate is, honestly

C4 passes the gate. It refuses the plant at every N, it never certifies a configuration where the
chart is out of tolerance, and it certifies two thirds of the honest ones. That is a working
refusal and it is the deliverable the commission asked for. It is also, on this family, **beaten
by reading `U`** — and its certified boundary is flat in N, so per §7(h) it is a fixed-`U` cutoff
in a lab coat even though it passes. Both halves are the result.

Reported as the prereg requires (§7(i)): **the conjunction passed; neither component did.** C1
alone certifies the plant; C3 alone carries 19 false positives. C4 is not "the certificate works."

---

## 3. Q6 — the kill, and the plumb line that makes it a falsification

| test | staked | measured | verdict |
|---|---|---|---|
| P-Q6-A partial ρ(ΔB4, E5 \| U) | ≥ 0.50, p < 0.01 | **0.099, p = 0.334** | NOT CONFIRMED |
| P-Q6-B isotonic collapse ratio | ≤ 0.70 | **25.17** | NOT CONFIRMED |
| P-Q6-C boundary CV | ≤ 0.35 | **0.887** | NOT CONFIRMED |

**Null shape reported before the p, as the house rule requires:** the within-U-column permutation
null over 10 000 draws has median 0.0251, 95th percentile 0.2781, range [−0.452, +0.467]. The
observed 0.099 sits at the 67th percentile of its own null. `p` is quoted, not `z`.

**Kill clauses (§6.5, on `E5` per A1/H2):**
- **(a) FIRES** — ρ = 0.099 < 0.20 and p = 0.334 > 0.05.
- (b) does not fire — `ΔB4` is never at its numerical floor where `E5 ≥ 3`.
- **(c) FIRES** — `ΔB4` exceeds the honest median (1.769e-7) at (N=6, U=0.125), (N=8, U=0.125) and
  (N=10, U=0.125), all of which have `E5 ≤ 0.5`. **Large share where the chart is fine.**

P-Q6-B's number deserves emphasis because it is the opposite of the staked direction: `E5`
collapses onto `U/t` **25 times better** than onto `ΔB4`. The share is not merely failing to be an
instrument; on this family it is a worse thermometer than the knob itself.

**The mechanism of the failure**, since a dead claim should be understood and not just recorded:
`ΔB4` is non-monotone in N at fixed `U`, while `E5` is mildly monotone. At the honest boundary
(`U = 0.25`), `ΔB4` reads 8.08e-8, 2.53e-6, 1.41e-6, 7.12e-7 for N = 4, 6, 8, 10 — peaking at
N = 6 — while `E5` reads 0.649, 0.589, 0.554, 0.531, decreasing smoothly. The N = 4 point is
anomalously low for a structural reason: at N = 4 the sector constraints leave the 4-slot marginals
inside the pairwise family (a 4-of-4-slot marginal at half filling has only 3 free parameters after
complement symmetry, against 6 couplings), so `B4(N=4, U=0)` is machine-zero rather than merely
small. **N = 4 is degenerate for Q6 in the same way A1/P4 found N = 2 to be** — that was not
foreseen, and had it been, N = 4 would have been declared out alongside N = 2.

### 3.1 The plumb line — a derived control, and what it buys

`Q_SEAM_PREREG.md` §6.2 derives, before any instrument existed, that `I_C^(3)` is **exactly zero**
on this family: complement symmetry (from particle–hole plus ground-state uniqueness) forces the
three fields of the pairwise maxent family to vanish, leaving 3 couplings to match exactly 3 pair
marginals — parameter count equals constraint count, so `Q = P`.

Measured, over every triple of every configuration: **worst `|I_C^(3)| = 1.213e-13`**, gate 1e-12.

This is what separates a falsification from a VOID. The same derivation predicts that at k = 4 the
count is 7 free parameters against 6 couplings — one residual degree of freedom, which *is* `B4` —
and that is why the predictor sits at k = 4. The instrument measures the thing it was built to
measure; the thing simply does not predict chart failure here.

**Solver policy (A1/P2) held:** zero configurations VOID, zero quadruples where neither solver met
the 1e-13 marginal gate, and the worst Newton-vs-IPF disagreement was **2.68e-14** against a 1e-10
cross-check gate. The house lesson that IPF one-sidedly overstates the share near determinism was
carried as an active check and never fired.

---

## 4. Scope — what these results do and do not cover

- **One model family.** The 1D open Hubbard chain at half filling, `N ≤ 10`, `Sz = 0`. No doping,
  no two dimensions, no long-range interaction. Nothing here is a claim about mean-field theory in
  general, about quantum chemistry, or about any wild system.
- **Q6's statistic is a CLASSICAL statistic of a quantum state** — the Fock-basis measurement
  distribution. It was labelled that way in the prereg and it is not being upgraded now.
- **The tolerance vector is chosen, not derived** (§10 hole 1). A reader who rejects it rejects the
  Q5 reading, and that is fair. The protection was structural — `FP = 0` is unforgiving, and
  loosening a tolerance makes certify-everywhere *harder* to beat — not principled.
- **`κ = 0.5` and the `0.5·λ_min(N,0)` factor were single points, not scans**, and stayed fixed.
- **No stance or Lean edit follows from this file.** Findings go to the integrator.

## 5. What survives, and is reusable

The exact reference and its gate ladder: 16 executable gates, `N ≤ 10` at Hilbert dimension
63 504, bitwise-exact where exactness exists and analytically gauged where it does not (the whole
`U = 0` column in closed form at all five N, the whole `N = 2` column at all 14 `U` from the dimer
solution and Hellmann–Feynman). Two self-validations worth naming: the in-sector first excited state at `U = 0`, N = 10 reads a gap of **0.569259** — exactly the analytic `Δ(10)`, so the excited-state readout validates itself at dimension 63 504; and the stability Hessian's lowest eigenvalue at `U = 0` is `2·Δ(N)` at every N to five digits, reproducing the particle–hole excitation ladder from a numerical second difference. `⟨S²⟩ ≤ 1e-25` confirms Lieb's theorem to machine precision;
particle–hole, spin-flip and reflection residuals all sit three or more orders inside their staked
1e-11. The spin-factorized Hamiltonian means the largest object ever built is a 252 × 252 matrix.

Also reusable, and the more interesting half: **`D_bool` is `Core/ModeChart.lean`'s fence as a
measured number.** It reads exactly 0.000000 at `U = 0` at every N — where the Boolean chart is
exact — and rises to 0.4405 at the plant, where the natural occupations are near-maximally
fractional. `meanOcc_fractional_exists` is not merely a model witness; on this family it is the
measured face of the chart's failure.

## 6. Files

Prereg (frozen, with amendments A1 and A2): `sim_engine/Q_SEAM_PREREG.md`.
Crate: `sim_engine/crates/q-seam` — `hubbard.rs`, `lanczos.rs`, `dense.rs`, `observables.rs`,
`chart.rs`, `audit.rs`, `certificate.rs`, `share.rs`; runners `src/bin/q_seam_run.rs` (Q5) and
`src/bin/q_seam_q6.rs` (Q6); gates in `tests/exactness.rs` and `tests/chart_gates.rs`.
Outputs: `sim_engine/output/q_seam/{q5,q6}.{json,log}`, run ledger `output/q_seam/RESUME.md`.
