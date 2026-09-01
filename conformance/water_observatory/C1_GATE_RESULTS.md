# C1_GATE_RESULTS — the ring-polymer carrier, graded

**Prereg:** `conformance/water_observatory/C1_GATE_PREREG.md`, frozen 2026-09-01, ADMITTED by
`Audit/prereg_audit.py` before any stage ran.
**Instrument:** `engine/crates/holon-chem/src/rpmd.rs`, driven by
`engine/crates/holon-chem/examples/c1_campaign.rs`. Fast gates:
`engine/crates/holon-chem/tests/c1_quantum_nuclei.rs` (10 tests, all passing).
**Raw logs:** `engine/output/c1/{dvr,ladder,production,square,price}.log`.
**Machine:** i9-13900HX, load average 66–82 throughout (other lanes). No verdict here is a
function of wall time; the one timing gate (G7) is pinned with `taskset` and reported on
both core classes.

---

## VERDICT

**C1 is real physics and it hits its referee. Of the eight staked gates, seven pass on every
clause and one — G6, the freeze's own discriminating-power condition — FAILED; G4's second
clause fired inside an otherwise passing gate. Both fired clauses are the freeze's errors,
not the instrument's, and both stay in the record marked dead.**

Ring-polymer molecular dynamics on the engine's own STO-3G FCI H–H curve reproduces that
curve's exact anharmonic vibrational zero-point energy to **−0.0805% ± 0.0552%** at 256 beads
— five times inside the staked band —
against a sinc-DVR reference that certifies its own convergence on four axes and agrees with
an independent Numerov solve to 7e-10 hartree. The bead-convergence law is confirmed as a
**parameter-free forward prediction across the factor of 52** the freeze staked it over
(and holds at two further bead counts outside the staked set, widening the span to 280). At
one bead the ring-polymer integrator reproduces the classical trajectory **bit for bit** over
5000 steps, and its two-bead control separates by 0.234 bohr. The bead-forgetting commuting
square is exactly closed at `P = 1` and provably open above it, with its defect obeying the
two scaling laws its mechanism predicts to **2.0000** and **2.0074**.

**The isotope shift is the sharpest reading here.** With the reduced mass as the only thing
that moved, RPMD puts `ZPE(D₂)/ZPE(H₂)` at **0.7096184 ± 0.0006597**: 0.46σ from its own
reference, and **3.4σ from the harmonic ratio 0.7073785**. The instrument does not merely
agree with the reference — it resolves the anharmonic isotope effect, in a direction the
freeze staked backwards.

Three defects were found and fixed BEFORE the freeze, all on plants; one wrong closed-form
coefficient and one reporting bug were found DURING the run by the tests and the gates. The
freeze's D₂ ratio clause has the wrong SIGN in it, derivably, and that is reported as a
fired clause rather than quietly corrected. §5 is the list.

---

## 1. What ran

| stage | what it did | wall |
|---|---|---|
| `dvr` | four spectral references (H₂/D₂ × exact solver/banked table), each asserting its own convergence on four axes | ~4 min |
| `ladder` | PIMD on the banked H–H curve at `P` = 1…512, 8 chains × 400 000 sampled steps | ~30 min |
| `production` | `P` = 256 at `dt` = 4 and `dt` = 2, H₂ and D₂, 8 chains × 800 000 sampled steps | ~28 min |
| `square` | the classical limit at `P` = 1 and the bead-forgetting square with its budget | ~2 min |
| `price` | the two unit costs and the cost-model check, pinned on both core classes | ~3 min |

Roughly 1.1e10 potential evaluations of the banked curve, all of them in range: every
sampling run reports **zero excursions**. Wall times were taken at load average 66–82 on a
shared box and are recorded, not claimed — no gate here is a function of them.

---

## 2. The gates

| gate | what it staked | measured | verdict |
|---|---|---|---|
| **G0** | referee returns `Ok` with all four residuals ≤ 1e-9 Ha on 4 references, and reproduces both closed-form plants to ≤ 1e-12 Ha on all 6 levels | worst residual over the four references **7.14e-10**; harmonic plant **4.0e-15**, Morse plant **6.6e-15** | **PASS** |
| **G1** | `\|ZPE_RPMD(H₂) − ZPE_DVR(H₂)\|` ≤ 0.40% of ZPE, statistical error ≤ 0.12% | **−0.0805%** with **0.0552%** of noise; the `dt/2` repeat moves it by 0.0517% against a combined 0.094% | **PASS** |
| **G2** | `\|ZPE_DVR(banked) − ZPE_DVR(exact)\|` ≤ 0.05% of ZPE | the two references agree to **all 12 printed digits**; the interpolant's own Hermite error is **3.41e-14 Ha** | **PASS**, by ≳8 orders |
| **G3(a)** | measured `E_cv(P) − E_cv(256)` matches the closed form to `max(3σ, 12%)` at P = 2…64 | worst departure **11.37%** at P = 64 against a criterion of 19.02% there; ≤ 6.43% at every P where the criterion is 12% | **PASS** |
| **G3(b)** | fitted exponent over P ∈ {16, 32, 64} within ±0.25 of **1.7730** | **1.7416** (0.031 from the stake); the closed form's own exponent on the quantity actually fitted is 1.8167, and the measurement sits between the two | **PASS** |
| **G4** | `\|ZPE_RPMD(D₂) − ZPE_DVR(D₂)\|` ≤ 0.40%; **and** the ratio `ZPE(D₂)/ZPE(H₂)` sits BELOW `sqrt(mu_H2/mu_D2)` | clause 1: **−0.0379%** with 0.0747% of noise. clause 2: the ratio is **0.7096184 ± 0.0006597**, which sits **3.4σ ABOVE** the harmonic 0.7073785 | **clause 1 PASS, clause 2 FIRED** |
| **G5** | (i) square exact at P = 1; (ii) open above it; (iii) `force_gap ~ R_g^2` in [1.6, 2.4] and `defect ~ dt^2` in [1.7, 2.3] | (i) **0.000e0** on positions, velocities and force gap; `H_P` drift −1.95e-16 relative. (ii) 2.6e-4 bohr at P = 2, nonzero at every P ≥ 2. (iii) **2.0074** and **2.0000** | **PASS ×3** |
| **G6** | `\|ZPE_DVR − ω_harm/2\|` must exceed **3× G1's band** = 1.20% of ZPE_DVR | **0.9473%** (1.0693e-4 Ha) | **FAIL** (see §5) |
| **G7** | run wall time predicted from its own unit costs to within a factor 3, on a declared core | P-core **2.138 / 1.471**; E-core **0.979 / 1.100**; potential-call counts reproduce to **1.000** | **PASS**, both core classes |
| **gate (c)** | P = 1 recovers the classical trajectory; trajectory must MOVE; P = 2 control must separate | worst `\|dR\|` **0.000e0** and `\|dV\|` **0.000e0** over 5000 steps, path length **24.49 bohr**, control separates by **0.234 bohr** | **PASS** |

---

## 3. The references (G0, G2)

Four sinc-DVR references, each refusing rather than returning unless the Lanczos Ritz
residual, a grid halving, a box widening and an independent Numerov solve all agree.

| | H₂ / exact solver | H₂ / banked table | D₂ / exact solver | D₂ / banked table |
|---|---|---|---|---|
| ritz | 3.609e-12 | 3.566e-12 | 1.410e-14 | 5.882e-15 |
| grid halving | 2.852e-10 | 2.852e-10 | 4.330e-14 | 3.308e-14 |
| box widening | 5.304e-10 | 5.304e-10 | 5.085e-14 | 4.508e-14 |
| Numerov | 7.142e-10 | 7.142e-10 | 1.235e-13 | 1.219e-13 |
| solves / potential calls | 4 / 22 532 | 4 / 22 532 | 4 / 22 532 | 4 / 22 532 |

**H₂'s grid and box residuals are four orders of magnitude worse than D₂'s (2.85e-10 against
4.33e-14) and only 2× inside the 1e-9 tolerance.** That is the lighter mass: H₂'s wavefunction is broader and leaks slightly more
at the box edges. It is reported because it is the tightest margin in the freeze, and it is
irrelevant to any verdict — the quantity the gates read is a zero-point energy at the 1e-5
level, five orders above it.

**Spectra (hartree, relative to `V(R_e) = −1.137306051222`):**

| | H₂ | D₂ |
|---|---|---|
| `E₀ − V_min` (**ZPE**) | **0.011288114850** | **0.008006844017** |
| `E₁ − V_min` | 0.032993286034 | 0.023580135202 |
| `E₂ − V_min` | 0.053667369982 | 0.038628333597 |
| `ω_e = E₁ − E₀` | 0.021705171 a.u. = 4763.73 cm⁻¹ | 0.015573291 a.u. = 3417.94 cm⁻¹ |
| `ω_e x_e` | 0.000515544 a.u. = 113.15 cm⁻¹ | 0.000262546 a.u. = 57.62 cm⁻¹ |
| ZPE ÷ harmonic ZPE | 0.990616 (−0.938%) | 0.993317 (−0.668%) |
| thermal(300 K) − ZPE | +2.6e-12 Ha | +1.2e-9 Ha |

**G2's number, stated plainly:** the exact-solver reference and the banked-table reference
agree to every digit printed, because the 4096-knot cubic Hermite interpolant on the
`R^{-1/4}` grid departs from the model it was built from by **3.41e-14 hartree** and moves
the curve's minimum by **8.9e-16 hartree**. The interpolant is not an approximation at the
scale this gate reads; it is the same curve. That is what licenses sampling on it, and the
licence is measured rather than assumed.

---

## 4. The instrument

### The headline (G1) and the `dt` check

`P = 256`, 8 chains × 800 000 sampled steps, banked curve, zero excursions.

| run | `ZPE_RPMD` | stat err | `− ZPE_DVR` | as % of ZPE | stat % | `tau` | wall |
|---|---|---|---|---|---|---|---|
| **H₂, `dt` = 4** | **0.0112790306** | 6.24e-6 | −9.084e-6 Ha | **−0.0805%** | 0.0552% | 18.6 | 394 s |
| H₂, `dt` = 2 | 0.0112731879 | 8.65e-6 | −1.493e-5 Ha | −0.1322% | 0.0766% | 31.5 | 440 s |
| **D₂, `dt` = 4** | **0.0080038071** | 5.98e-6 | −3.038e-6 Ha | **−0.0379%** | 0.0747% | 27.9 | 432 s |
| D₂, `dt` = 2 | 0.0080027206 | 8.05e-6 | −4.125e-6 Ha | −0.0515% | 0.1006% | 41.7 | 403 s |

against `ZPE_DVR(H₂) = 0.0112881149` and `ZPE_DVR(D₂) = 0.0080068440`. **G1's band is 0.40%
and its statistical requirement is 0.12%; the H₂ headline is 0.0805% off with 0.0552% of
noise — five times inside the band and twice inside the noise requirement. G4's first clause
is the D₂ row at 0.0379% off with 0.0747% of noise, ten times inside the band.**

The `dt` check is clean on both isotopes: halving the time step moves H₂ by 0.0517% against
a combined error of 0.094%, and D₂ by 0.0136% against 0.126%, so the `dt = 4` staked in the
freeze is not buying its agreement with a step-size error. The correlation time roughly
doubles when `dt` halves (18.6 → 31.5 for H₂, 27.9 → 41.7 for D₂), which is what it must do
at fixed step count, and the error bars grow accordingly — the `dt` = 2 rows are convergence
checks, not better measurements. Every run reports **zero excursions**: no bead ever left the
banked table's domain, so nothing here was extrapolated.

### The isotope shift (G4)

The Born–Oppenheimer surface does not know about isotopes. Between the two production runs
the ONLY thing that changed is the reduced mass, computed from the engine's own declared
atomic masses.

| | `ZPE(D₂) / ZPE(H₂)` |
|---|---|
| harmonic, `sqrt(mu_H2/mu_D2)` | 0.7073785 |
| **RPMD, measured** | **0.7096184 ± 0.0006597** |
| DVR reference | 0.7093163 |

* **RPMD against its reference: +3.02e-4, which is 0.46σ.** The instrument reproduces the
  reference's isotope ratio.
* **RPMD against the harmonic value: +2.24e-3, which is 3.4σ.** The measurement RESOLVES the
  anharmonic isotope effect — it is not merely consistent with the reference, it is
  distinguishable from the harmonic arithmetic that a curve-blind instrument would produce.
  This is the sharpest single reading in the campaign, and it is the one thing here that
  nothing was built to reproduce.
* The sign is ABOVE the harmonic ratio, not below, which is where the freeze's second clause
  fired. §5 has the derivation; both the reference and the instrument agree with it.

### The bead ladder (G3), H₂ on the banked curve, 8 chains × 400 000 sampled steps

| P | `E_cv − V_min` | err | predicted `E_P − E_256` | measured `E_cv(P) − E_cv(256)` | departure | criterion |
|---|---|---|---|---|---|---|
| 1 | 0.000953421 | 1.98e-6 | −1.043251e-2 | −1.033819e-2 | −0.90% | outside the staked set |
| 2 | 0.001882803 | 5.34e-6 | −9.508172e-3 | −9.408806e-3 | −1.05% | 12% |
| 4 | 0.003626162 | 8.73e-6 | −7.777567e-3 | −7.665447e-3 | −1.44% | 12% |
| 8 | 0.006326055 | 8.15e-6 | −5.059622e-3 | −4.965554e-3 | −1.86% | 12% |
| 16 | 0.009018816 | 9.38e-6 | −2.264946e-3 | −2.272793e-3 | +0.35% | 12% |
| 32 | 0.010533410 | 8.90e-6 | −7.124136e-4 | −7.581990e-4 | +6.43% | 12% |
| 64 | 0.011088357 | 8.08e-6 | −1.825031e-4 | −2.032520e-4 | +11.37% | 19.02% (3σ) |
| 128 | 0.011238683 | 7.90e-6 | −3.721432e-5 | −5.292600e-5 | +42.2% | outside the staked set |
| 256 | 0.011291609 | 8.28e-6 | 0 | 0 | — | — |
| 512 | 0.011284564 | 8.67e-6 | +9.360941e-6 | +7.045e-6 | — | outside the staked set |

Read against the reference rather than against P = 256, the two largest bead counts
straddle it: `P = 256` sits **+0.031%** above `ZPE_DVR` and `P = 512` sits **−0.031%**
below, both within one standard error of it.

**These are NOT independent of the production run below and are not quoted as if they
were.** The ladder and the production stage use the same seed base and the same dynamics,
so their `P = 256` chains are the same trajectories sampled over overlapping windows —
the ladder takes steps 40 000…440 000, production takes 80 000…880 000, a 45% overlap.
The staked configuration is production's, and production's number is the headline; the
ladder's `P = 256` row is a shorter, correlated view of the same chains.

`E_1 = 0.000953421 ± 2e-6` against `kT = 1/β = 0.000950043`: the one-bead ring polymer is
the classical oscillator, measured, and consistent at 1.7σ. The `+0.36%` residual is not
noise pretending to be zero — it is the classical anharmonic correction to `<V>`, which is
`kT/2` only in a harmonic well, and at 300 K the classical particle explores `±0.063 bohr`
of a curve that is not one.

**The primitive estimator is carried as a check and it degrades with `P`, exactly as its
variance scaling says it must.** It agrees with the centroid-virial estimator to 1.8e-6 at
`P = 32` and 1.4e-5 at `P = 64` (1.3σ), then departs: −3.4e-5 at `P = 128` and **−9.3e-5 at
`P = 256`**, where the quoted errors are ~1e-5. This is not a discrepancy about the physics —
the primitive estimator is a difference of two numbers near 0.12 hartree producing 0.011,
so its relative variance grows as `sqrt(P)` and its blocking plateau is not reached at this
sample size. It is reported because the freeze asked for two estimators and the honest
reading is that the second one **confirms up to `P = 64` and stops being usable above it**.
The gate uses the centroid-virial estimator, which is why the freeze names it primary.

### The commuting square (G5), and its budget

Free-ring-polymer draws at 300 K, 32 draws per row, mean ± spread:

| P | `R_g` (bohr) | `force_gap` (Ha/bohr) | `defect_pos` (bohr) | `defect_vel` | `H_P` drift / \|H_P\| |
|---|---|---|---|---|---|
| 1 | 0.0000 | **0.000e0** | **0.000e0** | **0.000e0** | −1.95e-16 |
| 2 | 0.322 ± 0.085 | 5.87e-2 ± 5.8e-2 | 2.56e-4 ± 2.5e-4 | 1.27e-4 | −3.01e-5 |
| 4 | 0.362 ± 0.065 | 5.35e-2 ± 5.3e-2 | 2.33e-4 ± 2.3e-4 | 1.15e-4 | −3.76e-5 |
| 8 | 0.377 ± 0.052 | 4.97e-2 ± 4.6e-2 | 2.16e-4 ± 2.0e-4 | 1.07e-4 | −3.79e-5 |
| 16 | 0.363 ± 0.058 | 4.23e-2 ± 3.8e-2 | 1.84e-4 ± 1.7e-4 | 9.07e-5 | −4.19e-5 |
| 32 | 0.367 ± 0.045 | 5.01e-2 ± 4.9e-2 | 2.18e-4 ± 2.1e-4 | 1.07e-4 | −6.48e-5 |
| 64 | 0.379 ± 0.042 | 5.32e-2 ± 5.4e-2 | 2.32e-4 ± 2.3e-4 | 1.13e-4 | −1.17e-4 |

`R_g` is flat in `P`, and that is correct rather than suspicious — it is a closed form, and
the table matches it. Summing the free ring polymer's mode variances gives
`<R_g^2> = (beta / 4m) (1 - 1/P^2)` in three dimensions, i.e.
`R_g = 0.3785 sqrt(1 - 1/P^2)` bohr for hydrogen at 300 K:

| P | 2 | 4 | 8 | 16 | 32 | 64 |
|---|---|---|---|---|---|---|
| predicted | 0.3278 | 0.3664 | 0.3755 | 0.3777 | 0.3783 | 0.3784 |
| measured | 0.322 ± 0.085 | 0.362 ± 0.065 | 0.377 ± 0.052 | 0.363 ± 0.058 | 0.367 ± 0.045 | 0.379 ± 0.042 |

Every row agrees, and the quoted `±` is the SPREAD over the 32 draws, not the standard error
of their mean — read against the standard error (spread ÷ √32) the worst row is 1.4σ. The
`P = 2` value is genuinely 13% below the asymptote and the closed form says it should be. That the noisiest column in the freeze reproduces a closed form is worth more
than the defect numbers beside it, because it says the ring being handed to the square is
the ring the theory describes.

The **`H_P` drift is the C1 carrier's conservation budget** and it grows with `P` — 3.0e-5 at
two beads to 1.2e-4 at 64 — because more beads at fixed `dt` sample steeper parts of the
curve.

**The mechanism, on a deterministic ring so the law is read and not the noise:**

| `dt` | `defect_pos` | | ring spread | `R_g` | `force_gap` |
|---|---|---|---|---|---|
| 8.0 | 4.512e-5 | | 0.020 | 7.07e-3 | 1.613e-4 |
| 4.0 | 1.128e-5 | | 0.040 | 1.41e-2 | 6.457e-4 |
| 2.0 | 2.820e-6 | | 0.080 | 2.83e-2 | 2.591e-3 |
| 1.0 | 7.051e-7 | | 0.160 | 5.66e-2 | 1.049e-2 |
| 0.5 | 1.763e-7 | | 0.320 | 1.13e-1 | 4.414e-2 |
| **fit** | **2.0000** | | 0.640 | 2.26e-1 | 2.234e-1 |
| | | | **fit (≤ 0.16)** | | **2.0074** |

The two widest rings are **outside the quadratic region** — beads at `R ≈ 0.75` and `R ≈ 2.0`
bohr on a curve whose minimum is at 1.389 — and are reported as the law's measured domain
edge rather than folded into the fit.

### The price (G7)

| | P-core (CPU 0) | E-core (CPU 24) |
|---|---|---|
| banked table, per call | 80.38 ns | 139.11 ns |
| exact STO-3G FCI, per call | 47 696 ns (**593×**) | 119 645 ns (**860×**) |
| normal-mode transform, per element | 1.9218 ns | 3.0227 ns |
| P = 64, predicted / observed | 2.09 s / 4.47 s → **2.138** | 3.37 s / 3.30 s → **0.979** |
| P = 256, predicted / observed | 27.25 s / 40.08 s → **1.471** | 43.18 s / 47.51 s → **1.100** |
| potential calls, observed / expected | **1.000** | **1.000** |

The E-core is the adversarial reading and is the headline. Both classes are inside the
staked factor 3, and the model UNDER-predicts (ratio > 1) because it counts only the two
dominant terms and not the estimator loop, the thermostat or the RNG — all `O(P)` per step.
The gate's job is to refuse an impossibility, and the 593–860× table-versus-solver ratio is
the same measurement read forward. The headline H₂ run made **1 802 242 048** potential
calls; on the exact solver those would have cost **23.9 core-hours on a P-core and 59.9 on
an E-core** — one to two and a half core-days for ONE of the four production configurations,
against 394 s of wall on the table. That is why the freeze samples on the table and puts a
referee on both.

---

## 5. What went wrong, and where

Seven corrections. Three cost edits before the freeze; four were found by the gates and the
tests during the run.

### Before the freeze, on plants only

1. **The referee's Krylov dimension was a guess.** It reported a Lanczos residual of
   `1.5e-2` on a grid whose eigenvalues were right to `2e-13`. The residual was honest and
   the dimension was not enough. Fixed by making the dimension adaptive — doubling until
   the residual is inside tolerance or the whole space is spanned — **not** by relaxing the
   tolerance. A fixed dimension cannot tell an under-resourced solver from a hard problem.
2. **The sampler's integrator ordering.** OBABO (the original PILE ordering) measured a
   `+2e-5` to `+4e-5 Ha` bias against the closed-form `E_P` at `dt = 4` — 0.2–0.3% of the
   zero-point energy, which would have eaten most of a 0.40% band before any physics was
   tested. Changed to **BAOAB**, which costs exactly the same and whose configurational
   error is `O(dt^4)`. Re-gauged: at `dt = 4` the residual bias is below the 7e-6 noise
   floor and no `dt` trend is resolvable over `dt` = 8 → 1.
3. **G3's fit window and its target exponent were both wrong.** The first G3 fitted
   `A P^-x` over P ∈ {32…512} against a band around **2**. Plotting the closed form on its
   own axis before staking showed three of those five points would have been below `3σ`, and
   that the closed form's own exponent is 1.576 / 1.773 / 1.933 on the windows that are
   measurable — the asymptotic 2 is not reached anywhere the instrument can see. A band
   around 2 would have graded a correct instrument against the wrong truth.

### During the run, by the gates and the tests

4. **The `P^-2` coefficient in the module's own documentation was wrong by exactly 3.** The
   docstring said `E_P = omega/2 − beta^2 omega^3/(48 P^2)`; the closed form's deficit is
   `/(16 P^2)`, confirmed at ratio 0.99959 (P = 512) to 0.99999 (P = 4096). The `1/48` came
   from differentiating a form of the partition function that treats the ring frequency as
   independent of `beta`, which it is not. **The tell is the classical limit:** any
   expression for `E_P` that does not give exactly `kT` at `P = 1` is not the energy of this
   ensemble, and `harmonic_ring_energy` does. Found by a test that asserted the expansion.
5. **The `P = 1` classical limit was not exact, and should have been.** The ring step and
   velocity Verlet are the same algorithm at one bead, and they disagreed by 1.05e-11 bohr
   over 5000 steps. Cause: the free-ring-polymer propagator was written on momenta, so it
   multiplied by the mass and divided back, which is not the identity in f64. Fixed by
   writing the propagator on velocities and using ONE copy in both the 1D sampler and the 3D
   dynamics. The gate now reads exactly **0.0**.
6. **A reference-zero bug in the campaign's own reporting.** The ladder's `E_cv − E_ref`
   column subtracted a `V_min`-relative reference from an absolute energy and printed
   `−1.15 Ha` residuals on a `1e-5 Ha` quantity. The column is wrong in `ladder.log` and is
   superseded by the `G3(a)` table this document reproduces, which differences like with
   like. No verdict was ever taken from it.
7. **The commuting-square table was reading the RNG, not the ring.** One free-ring-polymer
   draw per `P` gave `R_g` of 0.52, 0.49, 0.48, 0.28, 0.36, 0.31 down the `P` column, and the
   shape that appears to show is noise. Replaced by 32 draws per row with the spread
   printed, and the two scaling laws moved onto a deterministic ring.

### The two fired clauses

**G6 FAILED, and the freeze mis-sized it.** The discriminator required
`|ZPE_DVR − ω_harm/2|` to exceed 3× G1's 0.40% band, i.e. 1.20% of `ZPE_DVR`. It is
**0.9473%** — 1.0693e-4 hartree, which is also −0.938% read against the harmonic zero-point
energy instead; the gate's denominator is `ZPE_DVR` because G1's band is. The
freeze set that 3× rule from the **Morse plant's** anharmonicity of −1.395%, using a plant
calibrated to the target curve's own `ω` and `D_e` as a proxy for the target's anharmonicity
— and the proxy was **49% optimistic**. The STO-3G FCI H–H curve is less anharmonic than a
Morse fitted to two of its moments.

What that costs, stated exactly: G1 still discriminates — 0.9473% is **2.37 bands** from the
harmonic arithmetic, not within one — so a G1 pass is not something a purely harmonic
instrument could have produced. But it discriminates at 2.35× where the freeze demanded 3×,
and the freeze's stated VOID consequence ("if the exact answer sits within the band around
the harmonic arithmetic") has a false premise and does **not** trigger. Had G1's band been
staked at 0.30% — which the measured statistics (0.073%) and the measured integrator
systematic (< 0.09%) comfortably support — G6 would have passed at 3.16×. The band was not
retro-fitted and will not be: the gate is reported fired.

**G4's second clause is FIRED, and the fired clause is wrong in the freeze rather than in
the world.** It staked that `ZPE(D₂)/ZPE(H₂)` sits BELOW the harmonic ratio. It sits above,
and one line of algebra says it must: `ω_e ∝ mu^{-1/2}` and `ω_e x_e ∝ mu^{-1}`, so the
FRACTIONAL anharmonic deficit `ω_e x_e / (2 ω_e) ∝ mu^{-1/2}` is SMALLER for the heavier
isotope. D₂ therefore sits closer to its own harmonic value than H₂ does to its (−0.668%
against −0.938%, a ratio of 0.712 against `sqrt(mu_H2/mu_D2) = 0.7074` — the scaling itself,
measured), and the ZPE ratio is pushed UP. The freeze also transcribed
`sqrt(mu_H2/mu_D2)` as 0.70724; computed from the engine's own masses it is **0.7073780**.
Both the reference and the instrument agree with the derivation and disagree with the
freeze. The clause stays in the record, marked dead.

---

## 6. Scope, restated because it bounds every number above

Everything here is **EXACT-IN-MODEL for STO-3G full CI**, and STO-3G is a minimal basis. The
zero-point energies are properties of that model's H–H curve, not predictions of the
vibrational constants of hydrogen, and nothing in this campaign compares them to experiment.
`ω_e = 4763.73 cm⁻¹` for H₂ on this curve against a measured 4401 cm⁻¹ is the basis talking,
not the method.

The vibrational problem is **one-dimensional and `J = 0`**: the relative coordinate of a
diatomic with a reduced mass built from the engine's declared atomic masses
(`Species::HYDROGEN.mass_u`, `rpmd::MASS_U_DEUTERIUM`), computed and never tabulated.
Rotation is excluded by construction — a thermal 3D ring polymer at 300 K carries a quantum
rotational energy (`beta B ≈ 0.32` for H₂ on this curve) that no `J = 0` reference can grade,
so instrument and referee are held to one Hamiltonian rather than compared across two. The
3D machinery that does run — the classical limit and the commuting square — needs no
spectral reference and is graded on its own terms.

A G1 pass earns exactly **"C1 computes the object its own Hamiltonian defines"**. It does not
earn "C1 agrees with hydrogen", and the difference is the basis.

---

## 7. What this leaves owed

- **R-8** in `DRY_RESIDUALS.md`: the C1 carrier's declared operators hold a bare
  `Option<fn(f64) -> f64>`, which cannot close over state, so no potential that owns anything
  — the banked table included — can be transported through them. C1's physics computes pair
  forces directly and goes around the operator it is the physics of. This is the ONE row
  adding C1 cost the register, against a staked zero.
- The `O(P^2)` normal-mode transform is the sampler's cost at large `P`: at `P = 512` it is
  1.0 ms a step and dominates the potential by 12×. A radix-2 real FFT would make it
  `O(P log P)`; nothing in this freeze needed it, and the `P = 512` ladder point is the one
  place it hurt.
- The primitive estimator stops being usable above `P ≈ 64` at this sample size. Either more
  sampling or a lower-variance second estimator is owed before it can check the virial one
  at the headline bead count.
- Rotation, and therefore a 3D thermal C1 reading, needs a `J`-resolved reference before it
  can be graded at all.
