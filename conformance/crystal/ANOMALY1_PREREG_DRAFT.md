# ANOMALY-1 — rungs 0 and I of the matter sweep: the current's source is the field MINUS ITS SCREENED VALUE, times a number the statics fix. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-23. NOT FROZEN, and not to be frozen before a second reader has attacked
it under the brief of §13. Nothing in it has been run: no Hamiltonian of the campaign has
been built, no instrument executed. What WAS computed for it is in §6 and is of two kinds
only: synthetic sums of cosines, and operator algebra at a gate point (`N = 6, 8`,
`x = 0.7`) with residuals printed and readings not. It is rungs 0 and I of
`MATTER_SWEEP.md` (`382243c`), and it is built so that its instrument and gates are the
ones rung IV will need.*

*On the name. The 2026-11-11 review ruled that a field oscillating at the gap is a plasma
reading and "not to be called an anomaly reading" (`KINK0_ANOMALY0_REVIEW.md` §1). This
draft keeps the sweep's label for the rung and says here what is and is not read: the
anomaly's coefficient enters as an exact sum rule that cannot fail and is NOT staked; what is
staked is whether the two-variable view (field, current) is CLOSED. The lead may rename it.*

## Why

The review's sentence for an honest reading: "That the source is the field times a number
AT ALL TIMES. On the lattice the source is `{E, K}` plus hopping terms, and the stake is
that the remainder shrinks with `x`." Worked through on paper (§3), that sentence is wrong
by one term on this lattice, and the corrected sentence makes a prediction from numbers the
record already holds:

> after a small uniform step `δ` of the background, the bulk current's source is
> `(Ω²/x) · (E − E_eq)`: the field MINUS the part of the step the vacuum never screens,
> times a number `Ω² = −2x⟨K⟩ / (1 − w₀)` fixed by two ground-state readings.

`w₀` is the fraction of a small background left standing in the massless lattice vacuum. In
the continuum it is zero. On this lattice it is not, and the record's own numbers force it to
be large: at least about `0.28` at `x = 4` (§3). Nobody on this record has read it.

## What is already known (credited; none of it is claimed)

- The model, its photon mass and its anomaly: Schwinger (1962); Johnson (1963); Adler; Bell
  and Jackiw (1969). The spin form: Banks, Susskind and Kogut (1976).
- That the massless continuum model does not depend on the background, and the massive one
  does: Coleman, Jackiw and Susskind (1975); Coleman (1976).
- The `τ = 0⁺` rate of the current is the lattice f-sum rule (the review, §1.1). The bound a
  sum rule and a static response put on the lowest excitation is the single-mode bound
  (Feynman; Bijl; memory, not checked at source).
- That a staggered chain with no mass term behaves as a continuum model with a small
  positive mass, of order `g²a`, i.e. `m/g` of order `1/√x` (Dempsey, Klebanov, Pufu and Zan,
  2022; memory, not checked at source). With that, a nonzero `w₀` falling as `1/√x` is
  KNOWN IN KIND, and mass perturbation theory (Adam 1997; memory) gives its first-order
  value. **The author did that arithmetic from memory before writing §3** and found the
  first-order value consistent with the closed view to first order and larger than the
  banked-derived number of §3 at `x = 4`. The coefficient is not typed here, as in
  `MATTER_SWEEP.md` §4, because rung II reads it.
- Real-time response of this model to a field quench: Buyens et al. (2014, 2017); Zache et
  al. (2019) (memory). A screen at source is owed before a freeze (`MATTER_SWEEP.md` §7).

Every stake is a re-finding in kind. §11 says what could come out otherwise.

## 1. Carrier

- **Hamiltonian** (`kink0/kink0.py`, unchanged):
  `W = x Σ (σ⁺_n σ⁻_{n+1} + h.c.) + (μ/2) Σ (−1)ⁿ σᶻ_n + Σ_{n<N−1} (L_n + α_n)²`,
  `L_n = Σ_{k≤n} q_k`, `q_k = (σᶻ_k + (−1)ᵏ)/2`, `μ = 2 (m/g) √x`. Open chain, `N` even,
  the neutral sector. Time `τ` is in units of `W`: a frequency `ω` here is `2√x · (M/g)`.
- **The campaign is MASSLESS** (`μ = 0`), background uniform. The mass term appears only in
  rung 0's gates, at a gate point (§2).
- **Operators on link `n`.** `K_n = σ⁺_n σ⁻_{n+1} + h.c.`; `J_n = i(σ⁺_n σ⁻_{n+1} − h.c.)`;
  `E_n = L_n + α_n`; `K⁽²⁾_n = σ⁺_n σᶻ_{n+1} σ⁻_{n+2} + h.c.`
- **The bulk window.** The four links `n = N/2 − 2, …, N/2 + 1`: two even and two odd, since
  `⟨K_n⟩` alternates link to link on an open chain (the review). A bar over a symbol is the
  mean over these four. The window's midpoint is half a link right of the chain's, as
  KINK-0's two-link mean is.
- **Sizes.** `N ∈ {14, 16, 18, 20, 22}`; sector dimensions `3,432` to `705,432`.
- **Couplings.** Statics (S1): `x ∈ {4, 9, 16}`, the three at which the vector gap is banked
  (`SCHWINGER3_RESULTS.md`: `M_∞/g = 0.664187, 0.627240, 0.608620`). Dynamics (S2):
  `x ∈ {2, 4}`, where the clean window is long enough (§6, last table).
- **Step.** `δ = 0.05`, and `0.025` for the linearity gate. The review found one whole unit
  is not linear response here (a fitted frequency `13 %` low at `ε = 1`).
- **What is fresh.** At `x = 4` the massless ground energy at `N = 12` is banked (G0), the gap
  is banked, and `⟨K⟩` follows from banked energies. The review saw `⟨K⟩` and an oscillation
  at `x = 5, 10, 20`, `N = 12`. Neither `w₀` nor a real-time response has been computed at
  `x ∈ {2, 4, 9, 16}` by anyone. M-VOLUME-SCALE is committed, not merely contacted: the
  registry's floor for a continuum reading (`N ≳ 20√x`, so 40 to 80) is not met and NO
  continuum sentence is made; every statement is about this lattice at fixed `x`, in the bulk.

## 2. Rung 0 — the instrument, as a GATE

New code, `anomaly1/anomaly1.py`, importing `kink0.py`'s builder: a vectorised sector
builder; complex readers for `J_n`, `{E_n, K_n}`, `σᶻ_n − σᶻ_{n+1}` and `K⁽²⁾_n`; a Krylov
time step for a background that is constant within a step and may change between steps (what
rung III's ramp needs). All gates below run at **the gate point**: `x = 0.7`, a random
background per link in `[−0.6, 0.6]`, and, where a mass is needed, `m/g = 0.37`. That point
is outside the reserved region of §12 and outside the campaign.

| gate | what | bar |
|---|---|---|
| **G0, G0b** | `kink0.py`'s two gates, unchanged | `10⁻⁶`; `10⁻¹⁰` and `> 1` (as banked) |
| **G-ID** (EXACT) | the three identities the review names, and a fourth this draft adds for the mass, as matrices on the neutral sector at `N = 8, 10`: `dL_n/dτ = −x J_n`; `dq_n/dτ = x(J_{n−1} − J_n)`; `dJ_n/dτ = −{E_n, K_n} − 2μ(−1)ⁿ K_n + x(σᶻ_n − σᶻ_{n+1}) − x(K⁽²⁾_n − K⁽²⁾_{n−1})`, with `dO/dτ = i[W, O]` | largest entry of each difference `≤ 10⁻¹⁰` |
| **G-STEP** | the Krylov step against the dense propagator at `N = 12` (dimension 924), mass on, three steps with three different backgrounds; the same three run backwards return the start | state difference `≤ 10⁻⁹` in norm, both |
| **G-FD** | the reader's `S̄` (the identity's right side) against a centred finite difference of `J̄` at step `h = 10⁻³`, at 20 times | `≤ 10⁻⁵` of the largest `|S̄|` |
| **G-SUM** (EXACT) | on the dense neutral sector at `N = 8, 10, 12`, massless, uniform background: `Σ_n w_n ω_n² = −2x⟨K⟩_all` and `w_n ≥ 0`, with `w_n = (4/(N−1)) |⟨n|Σ L|0⟩|² / ω_n`; and `1 − Σ w_n` equal to the second difference of `E₀(α)` over `2(N−1)δ²` | `10⁻¹⁰` relative; `10⁻⁵` for the second difference at `δ = 10⁻³` |

The sign of the `K⁽²⁾` term is this draft's convention for the string and differs from the
review's wording; the mass term `−2μ(−1)ⁿK_n` is derived here and is what rung IV's remainder
will read. Rung 0 is a gate and earns nothing (the review's ruling).

## 3. The derivation on paper, and the number it forces

For a uniform step `δ` on the massless chain in its ground state, linear response gives, for
the mean field over all links,

`δE(τ)/δ = w₀ + Σ_n w_n cos(ω_n τ)`,  `w_n ≥ 0`,  `w₀ = 1 − Σ_n w_n`,

and two exact statements about the weights (both checked as algebra, §6):

1. `Σ_n w_n ω_n² = −2x⟨K⟩ ≡ F`. The f-sum rule: the `τ = 0⁺` source of the current. This is
   where the anomaly's coefficient lives (`⟨K⟩ → −2/π` is the continuum's `1/π`), and it
   cannot fail.
2. `w₀ = (1/(2(N−1))) ∂²E₀/∂α²`: the static fraction of the step left unscreened.

Every `ω_n` is at least the lowest excitation the field couples to, `ω₁`. So
`ω₁² (1 − w₀) ≤ F`, with equality exactly when one mode carries everything. That is the
closed view: `d(δE)/dτ = −x J` (exact) and `dJ/dτ = (Ω²/x)(E − E_eq)`,
`Ω² = F/(1 − w₀)`, `E_eq = w₀ δ`.

**The number.** With the banked gaps and `−π⟨K⟩/2` between `0.985` and `1.003` (the
review's pilot read `0.9920, 0.9983, 1.0013` at `x = 5, 10, 20`):

| `x` | banked `M_∞/g` | `w₀` if the view is closed | with the gap's `3.3 %` spread | `w₀ √x` | "fully screened" (`w₀ = 0`) would need |
|---|---|---|---|---|---|
| 4 | `0.664187` | `0.276 – 0.289` | `0.23 – 0.33` | `0.55 – 0.58` | the bound violated by `0.38 – 0.41` |
| 9 | `0.627240` | `0.189 – 0.203` | `0.13 – 0.25` | `0.57 – 0.61` | by `0.23 – 0.26` |
| 16 | `0.608620` | `0.138 – 0.154` | `0.08 – 0.21` | `0.55 – 0.61` | by `0.16 – 0.18` |

Read plainly: `MATTER_SWEEP.md` §4's sentence "with massless fermions the background is
screened and nothing depends on it" cannot be true of this lattice at these couplings,
because an inequality with no free parameter forbids it. If the view is open, `w₀` is LARGER
than the table's value, never smaller. The `3.3 %` is the review's figure for the two
estimates of the gap at `x = 4`; it is carried to `x = 9` and `16` as an assumption and
marked as one (§13).

## 4. Readings

All on the neutral sector, massless.

- **Statics**, at each `x` and `N`: the ground states at `α = 0, +δ, −δ`. `K̄` at `α = 0`.
  `w₀ = [Ē(+δ) − Ē(−δ)] / 2δ` with `Ē = mean ⟨L_n⟩ + α`: the odd part, so the `α = 0`
  offset of an open staggered chain drops out. `F = −2x K̄`, `Ω² = F/(1 − w₀)`.
- **The saturation defect.** `D_sat = 1 − (1 − w₀) ω_ref² / F`, `ω_ref = 2√x · M_∞/g`
  banked. Zero if closed; equal to the share of `F` above the lowest mode, weighted by
  `1 − ω₁²/ω_n²`; never negative for bulk quantities.
- **Dynamics**, at `x ∈ {2, 4}`, `N ∈ {20, 22}`: the `α = 0` ground state evolved under
  `α = δ`. At each of 200 times in the window: `Ē(τ)`, and `S̄(τ)`, the mean over the four
  links of the right side of G-ID's third identity (operator expectations, no time
  derivative taken). The pieces of `S̄` are printed: the field part `−⟨{E, K}⟩` and the two
  hopping parts, which telescope to the window's ends.
- **The window is fixed before any dynamic number is looked at:** `τ_w = 0.40 × 2π/Ω`, with
  `Ω` from the `N = 22` statics. The code prints `τ_w` and only then evolves.
- **The view defect** (THE dynamic quantity; one mode, one window, one norm):
  `D_view = rms_τ [ S̄(τ) − (Ω²/x)(Ē(τ) − Ē(0⁻) − w₀ δ) ] / rms_τ [ S̄(τ) ]` over `[0, τ_w]`,
  with `Ē(0⁻)` the pre-step value. Nothing in it is fitted.
- **Reported on the same `0.40` window, not staked:** the naive form's defect (the review's sentence: offset dropped,
  number `F`), which §6 predicts from `w₀` alone and which is what "shrinks with `x`";
  `w₀ √x`; the trajectory defect of `Ē` against `w₀ + (1 − w₀) cos Ωτ`; the box gap of each
  `N` from the two lowest energies.

## 5. Stakes

- **G-SIZE** (a gate per `x`). `|w₀(22) − w₀(20)| ≤ 0.015` and `|K̄(22)/K̄(20) − 1| ≤ 0.005`.
  Failing it makes that `x` VOID for S1 and S2. Reason for `0.015`: §6 shows a static
  misreading of that size moves `D_view` by `0.03`, under half of S2's bar. At `x = 16` the
  correlation length is about `6.6` sites against `11` to the end, so this gate may well
  fire there; that is its job.
- **G-LIN** (S2 only). The normalised `S̄/δ` and `Ē/δ` at `δ = 0.05` and `0.025` differ by at
  most `0.02` of `rms S̄/δ` over the window.
- **G-CLEAN** (S2 only). The same at `N = 22` against `N = 20`, each on its own window
  `[0, τ_w]`, differ by at most `0.02` of `rms S̄`. If not, the ends have reached the centre
  and S2 is VOID at that `x`. The light-cone bound in §6 says the window is clean at `x = 2`
  and marginal at `x = 4`.
- **S1 (statics; the forced number).** At each non-void `x ∈ {4, 9, 16}`:
  `−0.10 ≤ D_sat ≤ 0.10`. *Kill:* `D_sat ≥ 0.22` at any `x`. Below `−0.10` is not a kill of
  the view: it contradicts an inequality, so it convicts the referee or the size, branch (r).
- **S2 (dynamics; the source at all times).** At each non-void `x ∈ {2, 4}`, `N = 22`,
  `δ = 0.05`: `D_view ≤ 0.07`. *Kill:* `D_view ≥ 0.16`.
- **S1 and S2 are two readings of one claim**, the first through a banked referee and blunt,
  the second with no referee and sharp. Neither is graded by a fit, a classifier or a
  signal-to-noise: every input is an expectation value in an exact state.

## 6. On paper, before any of it is proposed: does each quantity separate the readings?

Synthetic responses only: `w₀` plus one or two cosines, the second carrying a share `φ` of
`F` at `r` times the lowest frequency. No Hamiltonian was built for these tables.

**`D_view` on the fixed window `0.40 × 2π/Ω`** (independent of `w₀`):

| response | reads | against S2 (`≤ 0.07` MET, `≥ 0.16` KILL) |
|---|---|---|
| one mode (the closed view) | `0.000` | MET |
| one mode, statics misread by `±0.015` / `±0.03` | `0.031` / `0.061 – 0.066` | MET / MET, close: why G-SIZE is `0.015` |
| `φ = 0.05` at `2ω₁` | `0.043` | MET: S2 cannot see below about `φ = 0.08` |
| `φ = 0.10` at `2ω₁` | `0.086` | between |
| `φ = 0.15` at `2ω₁` | `0.129` | between |
| `φ = 0.20` at `2ω₁` | `0.171` | KILL |
| `φ = 0.30` at `2ω₁` | `0.253` | KILL |
| `φ = 0.10` at `3ω₁` | `0.171` | KILL |
| `φ = 0.05` at `6ω₁` | `0.080` | between: the quantity weighs high frequencies (§13.2) |

**`D_sat`** (exactly `φ(1 − 1/r²)`), read through a referee `3.3 %` uncertain:

| response | `D_sat` | as read | against S1 (`±0.10` MET, `≥ 0.22` KILL) |
|---|---|---|---|
| one mode | `0.000` | `−0.067 … +0.065` | MET |
| `φ = 0.10` at `2ω₁` | `0.075` | `+0.013 … +0.135` | MET or between |
| `φ = 0.20` at `2ω₁` | `0.150` | `+0.093 … +0.205` | between |
| `φ = 0.30` at `2ω₁` | `0.225` | `+0.173 … +0.275` | between or KILL |
| `φ = 0.40` at `2ω₁` | `0.300` | `+0.253 … +0.345` | KILL |
| `w₀ = 0` at `x = 4` (the continuum's sentence) | — | `−0.38 … −0.41` | branch (r) |

So S1 is certain only below `φ ≈ 0.04` and above `φ ≈ 0.4`. It is kept because it is the
only reading that uses the banked gap, and because its number (§3's table) is written here
before it is read. S2 carries the resolution.

**Plants, on paper** (the closed response; a plant must land on the far side of its bar):

| plant | reads | must read | margin |
|---|---|---|---|
| P-NAIVE, `w₀ = 0.23 / 0.28 / 0.33` (the range §3 gives at `x = 4`) | `0.350 / 0.426 / 0.502` | `≥ 0.25` | `0.10` at the low edge |
| P-NAIVE at `w₀ = 0.14` (`x = 16`) | `0.213` | — | would NOT clear `0.25`: one reason S2 is not run at `x = 16` |
| P-NUMBER, `Ω²` scaled by `1.25` or `0.75` | `0.250` (exactly `|scale − 1|`) | `≥ 0.20` | `0.05` |
| P-SYNTH, the grader fed `φ = 0.30` at `2ω₁`; fed one mode | `0.253`; `4 × 10⁻¹⁷` | `≥ 0.16`; `≤ 10⁻⁶` | `0.09` |

**Algebra, at the gate point** (`N = 6, 8`, `x = 0.7`, dense; residuals only were printed):
the three identities and the mass term to `6 × 10⁻¹⁵`, with coefficients
`[1, −2μ, x, −x, x]` on `[−{E,K}, (−1)ⁿK, σᶻ_n − σᶻ_{n+1}, K⁽²⁾_n, K⁽²⁾_{n−1}]`; the f-sum
rule to `2 × 10⁻¹⁵`; `w₀` against the second difference to `4 × 10⁻⁷`; all `w_n ≥ 0`.

**The clean window**, a LOWER bound from the fastest signal the hop allows (`2x` sites per
unit `τ`), in periods of the lowest mode, `(N/2 − 2)(M/g)/(2π√x)`:

| `x` | `N = 18` | `N = 20` | `N = 22` |
|---|---|---|---|
| 2 (`M/g = 0.70` ASSUMED; not banked) | `0.55` | `0.63` | `0.71` |
| 4 | `0.37` | `0.42` | `0.48` |
| 9 | `0.23` | `0.27` | `0.30` |
| 16 | `0.17` | `0.19` | `0.22` |

At `0.15` of a period every response reads as its own `τ = 0⁺` sum rule and P-NAIVE reads
`0.06`: a window that short cannot grade anything. That is why S2 is at `x ∈ {2, 4}` and why
the window is `0.40` and not longer. The review reported a clean window near half a period
at `N = 12`, longer than this bound; G-CLEAN measures it and the bound is not relied on.

## 7. What would make a stake vacuous

- **F is not staked.** `S̄(0⁺) = F δ/x` is an identity, and `⟨K⟩ → −2/π` is guaranteed. This
  is the review's first finding and ANOMALY-0's S3. It is printed and labelled EXACT.
- **`D_sat ≥ 0` is a theorem.** Its lower side is not a stake; it is branch (r).
- **S2 at a short window** passes by construction (above). The window is fixed at `0.40`
  and G-CLEAN voids the stake where the chain cannot give it.
- **A step applied to an eigenstate of the new Hamiltonian** would not move
  (M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE). The run asserts `rms S̄ > 10⁻³ δ` before
  grading.
- **MET does not mean closed.** It means the share above the lowest mode is under about
  `0.08` at `2ω₁`, on a `0.40`-period window, at `N = 22`. The sentence in branch (a) says
  that and no more.

## 8. Plants

plants: each runs FIRST at the campaign's own points and decides branch (e); none can be
pre-checked there without seeing the campaign.

- **P-NAIVE** (carrier: the S2 runs at `x = 2` and `x = 4`, `N = 22`; sector: the offset
  `w₀ δ` in the grader, nonzero if `w₀ ≥ 0.2`, which §3 forces at `x = 4`). The grader with
  the offset dropped and the number set to `F` must read `≥ 0.25`. If `w₀ < 0.2` is read at
  that `x`, the plant's sector is empty and the plant is VOID, not passed.
- **P-NUMBER** (carrier: the same runs; sector: `Ω²`, nonzero by `F > 0`). With `Ω²` scaled
  by `1.25` the grader must read at least `0.20` and more than the unscaled reading.
- **P-SYNTH** (carrier: the grader's own code path, fed arrays; sector: the second cosine,
  share `0.30`). Must read `≥ 0.16`; fed one cosine, `≤ 10⁻⁶`.
- **P-COULOMB-OFF** (carrier: the `x = 2`, `N = 20` run evolved with the field term's
  coefficient set to zero after the step; sector: `−{E, K}` in the reader, nonzero while
  `⟨K⟩ ≠ 0`). G-FD must then FAIL by more than `0.5` of `rms S̄`: the reader's source holds a
  field part the evolution no longer has.
- **P-MASS-SIGN** (carrier: the gate point, `N = 8`, `m/g = 0.37`; sector: the mass term of
  G-ID, nonzero at `μ = 0.62`). With the mass sign reversed in the reader, G-ID's third
  identity must fail by more than `0.1`.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector each plant acts on is nonzero in
its carrier: the offset `w₀ δ` is at least `0.2 × 0.05` where §3's inequality holds, and the
plant is void where it is not; `Ω²` is positive while `⟨K⟩ < 0`; P-SYNTH's second cosine
carries `0.30` of the sum; the field part of the source is `2δ|⟨K⟩|` at `τ = 0⁺`; the mass
term is `μ = 0.62` at the gate point.

## 9. Branches (in this order)

- **(e)** A gate of §2, G-FD, or a plant fails or is void. Nothing is read.
- **(v)** A solve raises, or every `x` fails G-SIZE. The stakes that read those states are
  VOID, named.
- **(r)** `D_sat < −0.10` at a non-void `x`. An exact inequality is contradicted beyond the
  referee's spread: the banked gap or the bulk reading is wrong at that `x`. No statement
  about the view is made there; the `x` is named.
- **(k)** S1's or S2's kill fires at any non-void `x`. On this lattice at that coupling the
  (field, current) view is OPEN: a share of the current's source is not the field. Stated
  with the `x`, the defect, and the printed pieces of `S̄`.
- **(a)** `x = 4` is non-void for S1 and at least one `x` is non-void for S2, and every
  non-void stake is MET. Then: on this lattice the bulk current's source after a small step
  is the field minus its unscreened part, times `−2x⟨K⟩/(1 − w₀)`, to `D_view` over `0.40`
  of a period; the unscreened fraction is `w₀(x)` as read; and the banked gap is recovered
  from two ground-state readings to `D_sat`. A re-finding (§11).
- **(b)** Otherwise: each stake stated as MET, between or VOID, with its number.

## 10. Lattice artefacts, each with how it is bounded

- **Staggering.** Link expectations alternate; every bulk reading is a mean over 4 links, 2 even and 2 odd.
- **Open ends.** Charge piles at the ends under a uniform background; the bulk sees it only
  after a signal crosses. Statics: G-SIZE. Dynamics: G-CLEAN, with the light-cone bound as a
  cross-check.
- **Finite size.** No extrapolation is staked. The five sizes are printed; the stakes read
  `N = 22` and the gates compare it with `N = 20`.
- **Gauss-law elimination.** `L_n` is the sum of charges to the left: a field of `0` enters at
  the left end, and a uniform `α` is a field applied at both ends. A different boundary
  field is a different model (the review, on P-BACKGROUND).
- **The window's hopping terms** telescope to its ends and are divided by four, not by
  zero. They are printed, and they are inside `S̄`, so `D_view` charges them to the view.
- **Nonlinearity in `δ`.** G-LIN.
- **The time step.** G-STEP and G-FD; the Krylov tolerance is `10⁻¹⁰` per step.
- **The banked gap's regime.** It is an `N → ∞` extrapolation of sizes 40 to 144 at bond
  dimension 64; it is spent here at `N = 22` only through `D_sat`, with its spread carried.

## 11. What this earns, said before it is read

A re-finding of Schwinger (1962) and of the lattice literature above; branch (a) would add
nothing to physics. Against the bar ("a forced derivation of a nontrivial piece of known
physics"), the lead should weigh three sentences:

- **Forced:** yes, in one place. Closure of the view plus two exact identities gives `w₀`
  from the banked gap with no free parameter, and an inequality that holds whether or not
  the view is closed.
- **Known physics:** the lattice chain's residual dependence on the background. Known in
  kind; whether the value at `x = 4` is in print was not checked.
- **Could come out otherwise:** yes. Nothing forces one mode to carry the sum rule at
  `x = 2` or `4`, where the chain's effective mass is not small. The author expects MET at
  `x = 4`, from the first-order check named above, and has no expectation at `x = 2`.
- **Not blind:** the number is written from banked data, and the author carries the
  first-order estimate. It is a prediction with its reason stated, not a blind read.

## 12. What must stay unread for rung IV

Rung IV's stake is the late field minus the equilibrium field, with a mass, after a ramp,
at a point nobody has computed. Its point is not chosen. So a REGION is reserved, and no
pilot, plant, gate, reviewer or author may compute in it before rung IV is frozen:

> for `m/g > 0` and `1.5 ≤ x ≤ 25`: (i) the neutral ground state's bulk field at any uniform
> `α` other than `0` and `½`; (ii) any real-time evolution after any change of a uniform
> background; (iii) the gap of the neutral sector at any `α`.

Already seen in it and to be named in rung IV's freeze: KINK-0 (`x = 3`, `m/g = 0.8, 0.15`,
`α ∈ {0, ½}`: centre fields `0.473` and `0.260`); the reviews' grids (`x = 4`,
`m/g ∈ {1, 0.1}`; `x ∈ {2.5, 3.5, 6}`, seven masses), all at `α ∈ {0, ½}`.

This draft keeps out of it: the campaign is massless; every use of a mass is at `x = 0.7`
with a random background per link, which is not a uniform `α` and brackets nothing.

**One leak that cannot be closed.** `w₀(x)` is the `m → 0` edge of rung II's surface at
small `α`. Reading it tells rung IV the equilibrium field's slope at one edge. Rung IV
should therefore be staked at an `α_f` that is not small and a mass well away from zero,
and its freeze should say that this edge was known.

## 13. For the reviewer, and what is weakest

**The brief.** Attack the algebra, the tables and the branches. You may compute: operator
identities and sum rules at `x ≤ 1`, `N ≤ 10`, any mass, random link backgrounds; and
synthetic series. You may NOT compute: anything in §12's region; `w₀`, `D_sat`, `D_view` or
a real-time response on the massless chain at any `x` between `1.5` and `25`, since two
points either side of the campaign's would bracket it (KINK-0's lesson). No pilot is
declared for this campaign: every bar comes from §6's synthetic tables, and the gates
(G-SIZE, G-LIN, G-CLEAN) take the place of a pilot's calibration.

Weakest first:

1. **Rung I may be an identity with a referee attached.** If one mode carries the sum rule
   to within the bars for a reason of kinematics (a free boson with a small mass), S1 and
   S2 pass at every `x` and say nothing new. The single-mode bound is old.
2. **`D_view` weighs high frequencies by `ω²`.** A share `0.05` at `6ω₁` reads `0.08`. The
   chain has states at the band edge. A reading in the "between" band may be that tail and
   not an open view at the mode's own cadence; the trajectory defect is printed to tell
   them apart but is not staked, because P-NAIVE reads only `0.13` on it.
3. **S1 is blunt and leans on one borrowed spread.** `3.3 %` is the review's figure at
   `x = 4`; at `x = 9` and `16` it is assumed. If the banked gaps are better than that, the
   bars are slack; if worse, branch (r) will fire for the referee's reasons.
4. **`x = 16` and perhaps `x = 9` may be void.** `N = 22` is `3.3` and `4.6` correlation
   lengths. Then S1 rests on `x = 4` alone.
5. **G-CLEAN at `x = 4` is marginal** (`0.42 – 0.48` against a window of `0.40`, from a
   bound). If it voids, S2 rests on `x = 2`, where no gap is banked and the lattice is
   coarsest.
6. **The bars are proposed from synthetic two-mode series.** A real spectrum is not two
   modes. `0.07` and `0.16` are the lead's to set (M-BAR-FROM-THE-READ does not apply: no
   read exists; M-UNTESTED-GAP does: the band between them is wide).
7. **The window-mean sum rule is a bulk statement.** The exact identities are for all links
   at each `N`; the stakes use four links and trust G-SIZE.
8. **The memory credits** (Dempsey et al.; Adam; the single-mode bound; Buyens; Zache) are
   unchecked. No stake leans on one; the "Why" does.
9. **The leak of §12.**

## 14. Compute, as tasks (nothing runs until the lead says)

- Build: the vectorised builder, four readers, the Krylov step, the grader; the gates of §2.
  No cargo build.
- Statics: `x ∈ {2, 4, 9, 16}` × five sizes × three backgrounds = 60 ground states (the
  `x = 2` statics feed S2 only), plus 8 at `δ = 0.025`.
- Dynamics: `x ∈ {2, 4}` × `N ∈ {20, 22}` × two steps = 8 evolutions of 200 readouts, plus
  one for P-COULOMB-OFF.
- Memory at `N = 22`: the matrix holds about `7.8 × 10⁶` entries (about `0.1 GB`); a complex
  vector is `11 MB`; thirty Krylov vectors `0.34 GB`; the readers' diagonals and window
  operators under `0.2 GB`. Peak under `1 GB`, against the box rule of `3 GB`. `N = 24`
  (`2.7 × 10⁶` states, about `2 GB`) is NOT used.
- Single thread, as `kink0.py` sets.

## 15. What this does not claim

No anomaly as a property of a handed count: on a staggered chain `Σ J` is the vector
current (the review, §3). No continuum value. No statement at a mass. Nothing about an
asymmetry; rung I supplies rung IV's instrument and one edge of its surface.

---
witness: `CIRISHolon.SceneRhythm.source_is_the_defect` (the source read as a defect of continuity; it defines the source and says nothing of its form)
witness: none (G-ID and G-SUM are operator identities checked numerically; S1 and S2 are measured)
**misfits:** M-VACUOUS-SUCCESS, M-VOLUME-SCALE, M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE, M-PLANT-OBS, M-PLANT-SECTOR, M-EMPTY-SECTOR, M-SORTS-NOT-SEPARATES, M-BAR-FROM-THE-READ, M-UNTESTED-GAP, M-EXTRAPOLATED-HOLE, M-INEQUALITY-READ-AS-EQUALITY, M-TAG-AS-PROPERTY, M-TRUNCATION-AS-ERRORBAR, M-EXIT-DISCRIMINATOR, M-VALIDATED-NOT-WIRED, M-IDLE-CALIBRATED-TIMEOUT. These are the registered ids this text contacts.

- M-VACUOUS-SUCCESS: the f-sum rule and the sign of `D_sat` are labelled EXACT and not staked; §7.
- M-VOLUME-SCALE: committed; no continuum sentence.
- M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE: the run asserts the source moves before grading.
- M-PLANT-OBS, M-PLANT-SECTOR, M-EMPTY-SECTOR: each plant names its carrier and sector; P-NAIVE is void, not passed, where its sector is empty.
- M-SORTS-NOT-SEPARATES: §6 shows where each hypothesis lands against each bar, and where it does not separate.
- M-BAR-FROM-THE-READ, M-UNTESTED-GAP: bars from synthetic tables, proposed; the band between is wide and said so.
- M-EXTRAPOLATED-HOLE: the banked gap is spent only through `D_sat`, with its spread; the `3.3 %` is carried from `x = 4` and marked.
- M-INEQUALITY-READ-AS-EQUALITY: `ω₁²(1 − w₀) ≤ F` is an inequality; equality is the stake.
- M-TAG-AS-PROPERTY: the current is not called handed.
- M-TRUNCATION-AS-ERRORBAR: exact diagonalisation; no truncation stands in for an error.
- M-EXIT-DISCRIMINATOR: a solve raises or returns; residuals printed.
- M-VALIDATED-NOT-WIRED: P-SYNTH and P-NUMBER run through the grader's own code path, the one S2 is graded by.
- M-IDLE-CALIBRATED-TIMEOUT: no estimator here has a signal-to-noise; the seam review's finding is the reason the window is fixed and nothing is fitted.
