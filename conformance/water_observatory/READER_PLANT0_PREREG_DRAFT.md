# READER-PLANT-0 — is the relaxation reader's label a reading of the mode, or of the signal-to-noise? The record's own reader, fed planted modes at the banked signal-to-noise. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-23. NOT FROZEN, and not to be frozen before a second reader has attacked
it. Nothing here has been built or run on the instrument under test: the Rust harness is
written and UNBUILT (a no-build day on the shared box). What HAS been run is a python port
on synthetic series, once, and its printed output is the prediction this draft stakes
(§4). No campaign walk was opened. Every number about the record cites its file.*

## 0. Why

`SEAM_NS0_REVIEW.md` (finding 2, 2026-11-23) reported, from a port of the reader's window
rule, that a planted UNDERDAMPED mode is labelled OVERDAMPED most of the time at the
signal-to-noise the 50 m/s arms were read at, and always UNDERDAMPED at the 200 m/s arms'.
If so, RESPONSE-1's two labels are readings of the kick's size and not of the liquid.
`RESPONSE1_RESULTS.md` records that as reported and NOT adopted, and names what is owed:
the underdamped plant fired on the record's own reader at that signal-to-noise. This is it.

It is a prediction about OUR INSTRUMENT, written before the test. It is not physics about
water, and it does not say whether the mode at 50 m/s is under- or overdamped (§9).

## 1. The instrument under test, and how it ran

- **The reader.** `holon_lens::field::fit_relaxation(y, dt, noise)`
  (`engine/crates/holon-lens/src/field.rs`, lines 1086–1160 at `8b9464d`). Three rules
  decide the label. (i) REFUSED if `|y₀| < 3·noise`. (ii) The window ends at the first
  index from 3 at which five consecutive points are under `2·noise`. (iii) UNDERDAMPED iff,
  within that window, some point has the sign opposite to `y₀` and magnitude above
  `max(4·noise, 0.1·|y₀|)`; otherwise OVERDAMPED, fitted as one exponential.
- **How RESPONSE-1 called it.** `engine/crates/holon-lens/examples/rung2.rs`, lines 638–649:
  the driven current mode `j_k^s` is aligned over the cycles by `align_cycles` (`field.rs`,
  line 748: sign `(−1)^c`, baseline the whole-run mean), and the ALIGNED mean is fitted
  with `noise` = the SD of the aligned mean's last quarter (`Aligned::noise`). Each single
  cycle is also fitted, with the mean per-cycle tail SD.
- **The plant as it was actually fired.** PR-2, staked in `RESPONSE1_PREREG.md` line 95 ("a
  synthetic underdamped wave `A e^{−Γt} cos(ωt)` | period to 2 %, `Γ` to 5 %, classified
  underdamped"), exists once: `field.rs` lines 2643–2655,
  `pr2_an_underdamped_mode_reads_period_and_envelope`: amplitude `20`, noise `0.05`, so a
  signal-to-noise of **400**; `Γ/ω = 0.04`; one seed; noise passed as known; no alignment.
  No amendment of RESPONSE-1 fires it again. **PR-2 was never fired at a signal-to-noise of
  6 to 9, nor at the banked `Γ/ω` of about 0.2, nor through `align_cycles`.** PR-7
  (`field.rs` line 2722) is the only plant at low signal-to-noise, and it plants an
  exponential.
- **What the record read with it** (`replace0/response1_L*_seed*/response1.read.txt`, in git):

| arm | aligned SNR (line 9) | aligned label (line 22) | points | rate or `Γ`, `ω` |
|---|---|---|---|---|
| 50 m/s seed 0 | 6.4 | OVERDAMPED | 11 | `λ = 8.247e12` |
| 50 m/s seed 1 | 9.4 | OVERDAMPED | 11 | `λ = 1.038e13` |
| 50 m/s seed 2 | 6.8 | OVERDAMPED | 8 | `λ = 1.227e13` |
| 200 m/s seed 0 | 38.4 | UNDERDAMPED | — | `Γ = 2.245e12`, `ω = 8.548e12` |
| 200 m/s seed 1 | 76.1 | UNDERDAMPED | — | `Γ = 1.698e12`, `ω = 8.993e12` |
| 200 m/s seed 2 | 29.4 | UNDERDAMPED | — | `Γ = 2.127e12`, `ω = 8.784e12` |

  And the record already holds the effect on real data: on the three 200 m/s arms, whose
  aligned mean reads UNDERDAMPED, the SINGLE cycles (per-cycle signal-to-noise 7.6 to 9.2:
  kick `1.87–2.10e-2` over per-cycle noise `2.28–2.61e-3`, line 9) read OVERDAMPED on 11,
  12 and 11 of 12 (lines 10–21). The same reader, the same mode, two labels, by SNR.

## 2. Carrier (synthetic; nothing banked is read at run time)

Twelve cycles of 314 readouts at `10.0082` fs after one pre-kick row, cycle `c` with sign
`(−1)^c`, exactly the shape of an arm. A unit kick. Planted forms for the current mode:

| form | series after the kick | where its constants come from |
|---|---|---|
| **U** | `e^{−Γt}(cos ωt − (Γ/ω) sin ωt)`, `Γ = 2.0e12`, `ω = 8.8e12` | the review's rounding of the three banked 200 m/s fits above; the current of a kicked damped oscillator |
| **Uc** | `e^{−Γt} cos ωt`, same constants | PR-2's own form |
| **E** | as U with `Γ = 5.0e12`, `ω = 5.9e12` | the equilibrium law (§8): a fit to the banked equilibrium density autocorrelation |
| **O2** | `(λ₂e^{−λ₂t} − λ₁e^{−λ₁t})/(λ₂ − λ₁)`, `λ₁ = 5.5e12`, `λ₂ = 1.04e13` | the record's two 50 m/s rates read as an overdamped second-order mode |
| **O1** | `e^{−λt}`, `λ = 1.0e13` | PR-1's notion of overdamped, near the banked pooled current rate `1.03e13` (`RESPONSE1_RESULTS.md` line 56) |
| **N** | nothing | noise only |

**Noise.** The reader's residual tails are not in git (the read files hold fits, not
series), so the noise cannot be resampled from the record. Three declared arms, ALL graded,
so that no one of them is a choice: **W** white Gaussian; **C** first-order with a 50 fs
correlation time (the review's two); **F** the thermal noise the planted law itself implies
— the velocity of the same damped oscillator driven by white force (for O1, first-order at
its own rate). F is the only one with a reason: by fluctuation and dissipation a mode's
noise has the mode's own correlation.

**Two paths.** **A**: one cycle, the noise handed to the reader as known (the review's
port). **B, the graded one**: the twelve cycles through `align_cycles`, the reader's own
tail noise, per-cycle noise `√12 / SNR` so that the expected aligned signal-to-noise is the
banked one. Path B is the reader as it ran.

## 3. Stakes

Per cell (form, noise arm, SNR), `p_over` is the fraction of 2,000 trials labelled
OVERDAMPED on the aligned mean, path B. One binomial standard error is at most `0.011`.

- **S1 (the stake).** `J = p_over(6.4) · p_over(6.8) · p_over(9.4)` for form U: the chance
  that the banked triple of labels (three OVERDAMPED) arises from the 200 m/s mode itself.
  **MET if `J ≥ 0.25` in all three noise arms. REFUTED if `J ≤ 0.05` in all three.**
- **S2 (the other direction, secondary).** `K = p_under(29.4) · p_under(38.4) ·
  p_under(76.1)` for form O2: the chance that an OVERDAMPED second-order mode gives the
  banked triple of UNDERDAMPED labels. **MET if `K ≥ 0.15` in all three noise arms; REFUTED
  if `K ≤ 0.03` in all three.**
- **S3 (port against reader).** Every graded `p_over` and `p_under` of §4's table is within
  `0.06` of the port's value (five standard errors of the difference of two 2,000-trial
  fractions at `p = 0.5`, `5 × 0.0158 = 0.079`, tightened to `0.06` because the port is a
  line-for-line copy and the only licensed difference is the random stream). A cell outside
  is a difference between the port and the reader, to be found and named before any branch
  is read.

## 4. The prediction, frozen

`conformance/water_observatory/reader_plant0/port_prediction.py`, run once on 2026-11-23,
single thread, numpy 2.4.0; output `port_prediction.out`,
sha256 `aef21021458bd6a9e8da9ef50cc46abf8f7c299654b092cddd842542f705f72a` (script `cdd232076a925564c96aea66318c95b2f3c4ddcfe955bcdc0865602b10b928a0`).
Its block [1] reprints the review's own run with the review's seed.

**Path B, `p_over` on the aligned mean** (W / C / F):

| form | SNR 6.4 | SNR 6.8 | SNR 9.4 | J | SNR 29.4 | SNR 38.4 | SNR 76.1 |
|---|---|---|---|---|---|---|---|
| U | 0.833 / 0.901 / 0.886 | 0.799 / 0.877 / 0.872 | 0.644 / 0.717 / 0.704 | **0.429 / 0.567 / 0.544** | 0.000 / 0.000 / 0.003 | 0.000 / 0.000 / 0.000 | 0.000 / 0.000 / 0.000 |
| Uc | 0.864 / 0.926 / 0.907 | 0.845 / 0.896 / 0.903 | 0.682 / 0.755 / 0.774 | **0.498 / 0.626 / 0.634** | 0.001 / 0.001 / 0.006 | 0.000 / 0.000 / 0.000 | 0.000 / 0.000 / 0.000 |
| E | 0.991 / 0.973 / 0.957 | 0.989 / 0.978 / 0.965 | 0.944 / 0.961 / 0.960 | **0.925 / 0.914 / 0.887** | 0.161 / 0.184 / 0.212 | 0.029 / 0.025 / 0.040 | 0.000 / 0.000 / 0.000 |
| O2 | 0.992 / 0.947 / 0.929 | 0.999 / 0.968 / 0.955 | 0.994 / 0.986 / 0.991 | **0.985 / 0.904 / 0.879** | 0.450 / 0.557 / 0.596 | 0.235 / 0.267 / 0.311 | 0.001 / 0.001 / 0.002 |
| O1 | 1.000 / 0.992 / 0.986 | 1.000 / 0.998 / 0.990 | 1.000 / 1.000 / 1.000 | **1.000 / 0.990 / 0.976** | 1.000 / 1.000 / 1.000 | 1.000 / 1.000 / 1.000 | 1.000 / 1.000 / 1.000 |

**`K` (three UNDERDAMPED at the 200 m/s signal-to-noise):** U 1.000 / 1.000 / 0.998; Uc 0.999 / 1.000 / 0.994; E 0.815 / 0.796 / 0.756; **O2 0.420 / 0.325 / 0.279**; O1 0.000 / 0.000 / 0.000 (W / C / F).

The review's committed numbers (path A, known noise, SNR 9.4 and 6.4): 65 % and 86 %
white, 84 % and 97 % correlated. This port's path A: `0.640` and `0.849` white, `0.816` and `0.970` correlated.
Path B, the reader's own noise, moves them by at most `0.02` in the white arm and `0.10` in the correlated one (`0.717` and `0.901`), downward.

**Noise-free landings** (the label a clean series gets when the stated noise is `1/SNR`):
`U: SNR 6.4: OVER/11  SNR 6.8: OVER/12  SNR 9.4: OVER/13  SNR 29.4: UNDER/49  SNR 38.4: UNDER/84  SNR 76.1: UNDER/120`  
`Uc: SNR 6.4: OVER/14  SNR 6.8: OVER/14  SNR 9.4: OVER/15  SNR 29.4: UNDER/85  SNR 38.4: UNDER/86  SNR 76.1: UNDER/122`  
`E: SNR 6.4: OVER/9  SNR 6.8: OVER/9  SNR 9.4: OVER/11  SNR 29.4: UNDER/54  SNR 38.4: UNDER/56  SNR 76.1: UNDER/61`  
`O2: SNR 6.4: OVER/7  SNR 6.8: OVER/7  SNR 9.4: OVER/8  SNR 29.4: OVER/11  SNR 38.4: UNDER/54  SNR 76.1: UNDER/67`  
`O1: SNR 6.4: OVER/12  SNR 6.8: OVER/13  SNR 9.4: OVER/16  SNR 29.4: OVER/27  SNR 38.4: OVER/30  SNR 76.1: OVER/37`  
So the mechanism does not need noise: at SNR 6.4 to 9.4 the window closes at the first zero
crossing of U (five points under `2·noise` there) before the rebound, and the rebound
itself, `−0.54` of the kick at 310 fs, is under or beside `4·noise` (`0.63`, `0.59`,
`0.43`). O2's rebound is `−0.13` of the kick at 260 fs: over the reader's `0.1` floor,
so at high signal-to-noise an overdamped second-order mode is labelled UNDERDAMPED.

## 5. Separation, on paper (what each hypothesis reads against each bar)

| hypothesis | `J` for U | against MET `≥ 0.25` / REFUTED `≤ 0.05` |
|---|---|---|
| the reader labels a planted underdamped mode correctly at these SNR (PR-2's contract: `p_over ≤ 0.1` a cell) | `≤ 0.001` | REFUTED, by a factor of fifty under its bar |
| the port is right | `0.429 / 0.567 / 0.544` | MET; the smallest is `0.18` above the bar, about nineteen (one standard error of `J` is about `0.010`) standard errors |
| the port is right about the window and wrong by `0.15` in every cell | about `0.2 – 0.3` | astride the MET bar: BETWEEN, which is what such a miss should read |

| hypothesis | `K` for O2 | against MET `≥ 0.15` / REFUTED `≤ 0.03` |
|---|---|---|
| the label reads the damping (`p_under ≤ 0.1` a cell) | `≤ 0.001` | REFUTED |
| the port is right | `0.420 / 0.324 / 0.278` | MET |

No bar sits within `0.1` of a predicted cell. The bars are not taken from the port's table:
`0.25` is "three labels carry a likelihood ratio under 4 against a form that always reads
OVERDAMPED", `0.05` is "over 20"; `0.15` and `0.03` are the same idea at about 7 and 33.

## 6. Plants (each with its paper landing, from the same port)

| plant | must read | port's landing | if it does not |
|---|---|---|---|
| **P-OVER**: O1 at SNR 6.4, 6.8, 9.4 | OVERDAMPED, `p_over ≥ 0.95` each | 1.000 / 0.992 / 0.986; 1.000 / 0.998 / 0.990; 1.000 / 1.000 / 1.000 | the reader is noise-driven at this SNR and S1 says nothing: VOID |
| **P-HIGH**: U at SNR 29.4, 38.4, 76.1 | UNDERDAMPED, `p_under ≥ 0.95` each (the banked 200 m/s reading reproduced) | 1.000 / 1.000 / 0.998; 1.000 / 1.000 / 1.000; 1.000 / 1.000 / 1.000 | the harness does not reproduce the record: VOID |
| **P-NOISE**: N | REFUSED, `≥ 0.95` | `1.000` REFUSED at all three (W) | the refusal rule is not the reader's: VOID |
| **P-PR2**: Uc with `Γ/ω = 0.04` at SNR 400, path A (PR-2 as fired) | UNDERDAMPED on every trial | the crate's own test, green in CI | the harness is not calling the reader PR-2 tests |

**Carrier and sector of each plant (M-PLANT-SECTOR).** Every plant is on the campaign's own
carrier (the twelve-cycle synthetic arm of §2, path B, except P-PR2 which is path A as PR-2
was fired) and acts on the sector the stake grades: the LABEL of the aligned current mode.
P-OVER and P-HIGH each assert a nonzero count in the label they must read before it is
graded (an arm where every trial is REFUSED is void, not passed); P-NOISE acts on the
refusal rule, the one sector S1 does not grade, and is there to show the kick is what is read.

P-OVER and P-HIGH together are what make S1 a statement about the label and not about
noise: the same reader, the same noise, separates O1 from nothing and U at high SNR from U
at low.

## 7. Branches (exactly one)

In this order: **(V) VOID** — a plant of §6 fails, or S3 fails; nothing is read, the
difference is found first. **(a) CONFIRMED** — S1 MET. **(b) REFUTED** — S1 REFUTED.
**(c) BETWEEN** — anything else, including the three noise arms disagreeing. S2 is read
beside whichever of (a), (b), (c) holds and has its own two outcomes and a between.

**(a) CONFIRMED.** The aligned label at the 50 m/s signal-to-noise does not distinguish the
200 m/s mode from an overdamped one. Dated CORRECTIONs, written here in advance:

1. `RESPONSE1_RESULTS.md`, "What two arms already say", item 1 — for the CURRENT mode only:
   *"'At 50 m/s the driven ... current relax[es] without oscillating' is withdrawn as a
   reading. The reader labels the 200 m/s mode itself OVERDAMPED with probability `J`
   [value] across the three seeds at their signal-to-noise (READER-PLANT-0). The current
   mode's damping at 50 m/s is NOT DETERMINED by its label."*
2. The same file, line 56: *"the pooled longitudinal current rate `1.03 × 10¹³` /s and the
   `ν_l = 1.4 × 10⁻⁶` m²/s taken from it are the rate of ONE exponential fitted to the
   first 8 to 11 points; the reader returns [port: `8.5–9.3 × 10¹²` on 12 to 13 points; from form E, `1.20–1.34 × 10¹³` on 10 to 12] /s from the 200 m/s mode
   under the same window. It is not shown to be a relaxation rate."*
3. The same file, item 2: *"`c_s ≈ 2,800` m/s 'from the product of the two overdamped rates
   at 50 m/s' falls with correction 2."*

   NOT corrected by this branch: "the 200 m/s control is outside linear response". That
   sentence also rests on the density mode's R2 (a different rule: residuals of two forms,
   `rung2.rs` near line 775, not tested here), on R4′ firing on the transverse control,
   and now on §8's equilibrium law, which SUPPORTS it. Nor is the transverse arm's R3
   touched: its mode has no restoring force to hide.

**(b) REFUTED.** *"`SEAM_NS0_REVIEW.md` finding 2 is withdrawn: on the record's reader the
200 m/s mode is labelled UNDERDAMPED at the 50 m/s signal-to-noise with probability
[value]; the port differs from the reader in [named]. RESPONSE-1's labels stand."* The
paragraph in `RESPONSE1_RESULTS.md` that reports the finding gets the same dated note.

**(c) BETWEEN.** Neither note is written. The per-cell table is banked; the labels are
recorded as "weak evidence, likelihood ratio [1/J]"; the whole-form read of §8 is the next
instrument.

**S2 MET** (beside any of the above): a note that the 200 m/s UNDERDAMPED label alone does
not exclude an overdamped second-order mode; that reading stands on its PERIOD (banked
699–735 fs; O2's single crossing at 130 fs would read `4 × 130 = 520` fs by the reader's
one-crossing rule, U's crossings 357 fs apart read 714 fs) and on R2. No correction.

## 8. The second arm (the equilibrium law), and what was read at zero compute while drafting

By fluctuation and dissipation the linear response of the current to a velocity kick is the
equilibrium current autocorrelation, and the density autocorrelation of the same mode is
`e^{−Γt}(cos ωt + (Γ/ω) sin ωt)` when the current's is form U. The record holds that
autocorrelation in git: `replace0/view_search_scout432_rigid.txt`, lines 7–9, three seeds
of the same 432-water rigid box at equilibrium, ten lags (20 to 500 fs;
`view_search.py` line 169).

**Declared: this was computed today, on banked numbers, before this draft was reviewed. It
is a reading, not a stake, and it is not blind.** Mean of the three seeds against each form
(root-mean-square misfit over the ten lags; the seeds differ by `0.00–0.02` at lags to
200 fs and `0.08–0.15` beyond):

| form | misfit | at 100 / 140 / 200 fs (banked mean `0.81 / 0.66 / 0.43`) |
|---|---|---|
| U (the 200 m/s mode) | `0.275` | `0.67 / 0.41 / 0.02` |
| O2 (the record's two 50 m/s rates) | `0.130` | `0.83 / 0.72 / 0.57` |
| best overdamped pair (`9.7e12`, `1.0e13`: critical) | `0.067` | `0.74 / 0.60 / 0.41` |
| **E: best damped cosine, `Γ = 5.0e12`, `ω = 5.9e12`** (per seed `Γ 4.2–6.5e12`, `ω 5.0–6.2e12`) | `0.023` | `0.79 / 0.65 / 0.43` |

What it says, at the strength of ten lags and three 5 ps walks:

- **The 200 m/s mode is NOT the equilibrium mode.** U misses the banked autocorrelation by
  `0.15` to `0.41` at lags where the seeds agree to `0.02`. The damping under a 200 m/s
  kick is about `2.5 ×` lower than at rest. That SUPPORTS the record's "outside linear
  response", against the review's suspicion and against this draft's own first expectation.
- **The equilibrium mode is not overdamped either.** It is weakly underdamped, `Γ/ω ≈ 0.85`,
  with a rebound of `−0.23` of the kick at 290 fs. At the 50 m/s signal-to-noise that
  rebound is under `4·noise` (`0.43–0.63`), so the reader labels it OVERDAMPED whatever
  the truth (form E in §4: `J` = 0.925 / 0.914 / 0.887).
- **One law gives both banked triples.** Form E through the port reads OVERDAMPED at the
  50 m/s signal-to-noise (`J` above) AND UNDERDAMPED at the 200 m/s one (`K` = 0.815 / 0.796 / 0.756): the
  equilibrium mode alone would produce the record's two labels about seven times in ten.
  The labels therefore carry no evidence of nonlinearity in either direction. What does is
  the fitted period at 200 m/s, 699–735 fs, against E's `2π/ω = 1,065` fs.
- So the likely end state is that BOTH sentences of the record survive in a weaker form
  ("50 m/s: no rebound visible at this SNR; 200 m/s: less damped than equilibrium"), and
  the review's "threefold change is an artefact" is half right: the label is an artefact,
  the change is not.

**The owed arm, not designed further here because it needs the walks:** the 50 m/s aligned
current (pooled over 36 cycles) against E, U and O2 by whole-form residual with nothing
fitted, and the equilibrium autocorrelation re-read with error bars on a finer lag grid.
The banked reads in git do not suffice: they hold fits and summaries, not series.

## 9. What this earns, and what it does not

- A prediction about our own instrument, staked before the instrument is run on it. By
  PREDICT.md's tests: forced (the reader is code), reachable (one small build), blind in
  the sense that the real reader has not been run on any of these series — and LOW IN
  SURPRISE: the port is a line-for-line copy and §4's noise-free landings show the
  mechanism on paper. What could come out otherwise is S3 (the alignment path, the tail
  noise estimate, the whole-run baseline) and the F noise arm.
- It decides three sentences of `RESPONSE1_RESULTS.md` (§7 (a) 1–3) and the standing of one
  finding of a review.
- It does NOT decide whether the mode at 50 m/s is overdamped. §8 is the lead on that.
- It is not a result about water, and not a step toward the 2026-11-13 bar.

## 10. Compute, as tasks (nothing runs until the box is free and the lead says so)

1. Build one existing crate and one new example:
   `cargo run -p holon-lens --release --example reader_plant0` under the cargo guard
   (`holon-lens` has one dependency, `holon-closure`, with none of its own). The example is
   UNBUILT; its first build is its first compile, and compile fixes that do not touch an
   arm are not amendments.
2. Run it once: 2,000 trials × 99 cells of 3,769 samples, the F arm's oscillator at 16
   substeps. Memory is a few megabytes. Its output is committed beside the port's.
3. Grade S3 cell by cell, then the plants, then S1 and S2; write the branch's notes.

## 11. Open to the reviewer (weakest first)

1. **The outcome is nearly known.** The record's own 200 m/s single cycles (§1) already
   show the effect on real data, and the port copies the code. A reviewer may fairly call
   this a unit test with a preregistration attached. Its value is that the test was owed
   as a plant and never fired, and that corrections 1–3 are written before it is.
2. **Form U is the wrong null.** §8 says the equilibrium mode is E, not U. Then S1 answers
   "could the 200 m/s mode hide at 50 m/s", which §8 already answers "it is not there".
   The stake that matters may be S1 on E, where `J` is near 1 by construction.
3. **Path B's per-cycle column is not comparable to the record.** It sets per-cycle noise
   to `√12/SNR`, but the banked ratio of per-cycle to aligned noise is 2.7 to 10, not 3.46
   (the per-cycle tail SD carries more than noise). The comparable figure for the banked
   34 of 36 is path A at SNR 7.6–9.2: about `0.64–0.82` white, `0.82–0.96` correlated; the
   white arm is in tension with 34 of 36 (expected about 27), the correlated arm is not.
4. **The noise is still modelled.** Three arms bracket it; none is the liquid's.
5. **`J` multiplies three cells as if the seeds' labels were independent** given the form.
6. **S3's `0.06`** is the author's tightening.
7. **The fitted period on UNDERDAMPED labels is not printed** by the harness or the port;
   S2's note leans on paper values (520 against 714 fs).
8. **§8 was read before review** and the E form's constants come from it; the density
   mode's own verdict rule (R2) is not tested by anything here.

---
witness: none (a measured property of an estimator on synthetic series; no theorem is cited)
**misfits:** M-PLANT-OBS, M-IDLE-CALIBRATED-TIMEOUT, M-SORTS-NOT-SEPARATES, M-VALIDATED-NOT-WIRED, M-FOREIGN-DOMAIN-CORROBORATION, M-POPULATION-CHOICE, M-BAR-FROM-THE-READ, M-RETRODICTION-AS-PREDICTION, M-ONE-MODEL-DELTA, M-STALE-INSTRUMENT, M-NAMED-POINT-SHIFTED, M-PROVENANCE-OVERREACH, M-PLANT-SECTOR

- M-PLANT-OBS: the founding defect — PR-2 fired once at SNR 400 on another form; §6 re-derives every plant for this instrument and gives its landing.
- M-IDLE-CALIBRATED-TIMEOUT: an estimator calibrated at one signal-to-noise and run at another; the stake is exactly that.
- M-SORTS-NOT-SEPARATES: §5 shows where each hypothesis lands against each bar.
- M-VALIDATED-NOT-WIRED: path B is the reader as RESPONSE-1 called it (`align_cycles`, tail noise), not the bare function.
- M-FOREIGN-DOMAIN-CORROBORATION: the prediction is a python port; S3 grades port against reader cell by cell before any branch.
- M-POPULATION-CHOICE: six forms and three noise arms, all named, all graded or reported; the excluded ones (the liquid's own noise, the density mode's rule) are named in §8 and §11.
- M-BAR-FROM-THE-READ: the bars are likelihood ratios chosen before the port's table was read against them; S3's is the author's and said so.
- M-RETRODICTION-AS-PREDICTION: the prediction follows a reading of the code and of the record's 200 m/s single cycles; §9 and §11.1 say how little it can surprise. §8 is declared a reading, not a stake.
- M-ONE-MODEL-DELTA: branch (a) concludes "not determined", not "underdamped"; §8 carries the other forms.
- M-STALE-INSTRUMENT: the harness is committed before the freeze, and marked UNBUILT.
- M-PROVENANCE-OVERREACH: the digest in §4 fixes the port's OUTPUT as the prediction and nothing more; it does not vouch for the port's fidelity to the reader (S3 does) nor for the constants typed into it (§1, §2 cite each).
- M-PLANT-SECTOR: §6 states each plant's carrier and sector, and voids an all-REFUSED arm.
- M-NAMED-POINT-SHIFTED: "overdamped" names three different things here (one exponential; a second-order mode with two real rates; a label) and each is given its own form.
