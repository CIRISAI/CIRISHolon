# SLOW-2 — READ: branch (c). TICA and VAMP re-find q, and SLOW-1's search is the same instrument. No learned dictionary carries +0.02 beyond q. The integration kill fires: F fails PS-3, and so does SPIB

*2026-09-25. Prereg `SLOW2_PREREG.md`, frozen alone as `3f7c390`. Notes on building,
committed alone before the read as `5660c20`, recorded the PS-3 failures of F and E before any
real walk was read. Reader: `slow2_compare.py`. Read: `slow2/slow2_read.txt` (the run),
`slow2/slow2_verdict.txt` (the verdict), `slow2/slow2_read.json` (every number).*

*Carrier: SLOW-1's walks (the rigid operator, 250 waters, 50 ps at 20 fs). TRAIN = T293
seeds 0 and 1; HELD-OUT = T293 seed 2; CONTROL = T400 seed 0. Nets on the GPU (RTX 4090
Laptop, CUDA, deterministic algorithms); CPU work pinned to cores 16–20. The run took
692 s.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **C1** the baselines re-find | TICA's and VAMP's f1 load ≥ 0.8 on q (HELD-OUT) | TICA **0.839**, VAMP **0.836** (A 0.836) | **MET.** SLOW-1's search added nothing over TICA on this problem. |
| **C2** a learned dictionary goes beyond q | the best of D/E/F: ≥ +0.02 beyond velocity and q on HELD-OUT, PS-2 ≤ 0.01, ≤ +0.005 at 400 K | best is **D (VAMPnet): +0.0128** [+0.0121, +0.0139]; PS-2 −0.0005; 400 K +0.0007 | **NOT MET** (and no arm reaches +0.014) |
| **C3** ours ≥ SPIB | F's mean ≥ E's mean on HELD-OUT | F +0.0076 [+0.0072, +0.0083] against E +0.0022 [+0.0005, +0.0033] | met by the letter, separated beyond the spread. **Not readable: both arms fail PS-3.** |
| **Integration kill** | F below E, OR F fails any plant | F fails **PS-3** (0.684 < 0.9) | **FIRES** |
| plants, A B C D | PS-1, PS-2, PS-3, PS-4 as frozen | all pass (PS-3 1.000, 1.000, 1.000, 0.935; PS-4 max ≤ 0.081) | pass |
| plants, E (SPIB) | the same | PS-3 **0.254 FAIL** (collapsed to 2 states on every seed); PS-1, PS-2, PS-4 pass | **E not read** |
| plants, F (λ = 10) | the same | PS-3 **0.684 FAIL**; PS-1, PS-2, PS-4 pass | **F not read** |

**Branch (c).** No learned arm meets C2. **Tetrahedral order is all the structure carries on
this operator at these lags,** as far as these learners, these inputs and this carried
target can see. The integration kill fires beside it. The (e) clause does not govern the
branch, because the arm C2 would read (D) passes every plant. It governs E and F, which are
not read.

## 1. The arms (HELD-OUT T293 seed 2; CONTROL T400 seed 0)

**Common to every arm:** `q`'s own carry beyond velocity is **+0.0317** on HELD-OUT and
**+0.0013** at 400 K. This is the base rate (M-BASE-RATE-OMITTED). R² of velocity alone is
about 0.

| arm | own σ₁ at 1 ps | beyond velocity | **beyond velocity and q** (mean [min, max]) | beyond velocity and STRUCT (six) | 400 K beyond velocity and q | f1's R² on q | f1's R² on STRUCT (six) | plants |
|---|---|---|---|---|---|---|---|---|
| A SLOW-1 search | 0.331 | +0.0424 | **+0.0109** | −0.0000 | +0.0001 | **0.836** | 1.000 | pass |
| B TICA | 0.331 | +0.0423 | **+0.0108** | −0.0000 | +0.0001 | **0.839** | 1.000 | pass |
| C VAMP | 0.331 | +0.0424 | **+0.0109** | −0.0000 | +0.0001 | **0.836** | 1.000 | pass |
| D VAMPnet | **0.498** | +0.0428 | **+0.0128** [+0.0121, +0.0139] | +0.0061 | +0.0007 | 0.192 | 0.396 | pass |
| E SPIB | 0.293 | +0.0069 | +0.0022 [+0.0005, +0.0033] | +0.0017 | +0.0001 | 0.046 | 0.220 | **PS-3 fail**: NOT READ |
| **F integrated, λ = 10 (stake)** | 0.496 | +0.0297 | +0.0076 [+0.0072, +0.0083] | +0.0036 | +0.0005 | 0.065 | 0.357 | **PS-3 fail**: NOT READ |
| F, λ = 0 (no stake) | 0.498 | +0.0428 | +0.0128 [+0.0121, +0.0139] | +0.0061 | +0.0007 | 0.192 | 0.396 | not run on λ = 0 |
| F, λ = 1 (no stake) | 0.498 | +0.0438 | +0.0137 [+0.0135, +0.0138] | +0.0061 | +0.0007 | 0.174 | 0.389 | not run on λ = 1 |
| F, λ = 100 (no stake) | 0.441 | +0.0288 | +0.0079 [+0.0073, +0.0090] | +0.0030 | +0.0004 | 0.131 | 0.371 | not run on λ = 100 |
| **R0**: the 84 SHELL columns in the ridge (no dynamics, no stake) | — | — | **+0.0194** | — | +0.0006 | — | — | — |

- **F's λ = 0 row** reproduces D digit for digit. F's training code and deeptime's
  `partial_fit` agree.
- **The plant rows for λ = 0, 1 and 100:** PS-3 and PS-4 were run only on the stake λ, as
  declared. The script's printed "FAIL" on those rows is inherited from λ = 10, and is not a
  measurement.
- **Beyond velocity, STRUCT and the four cage-escape histories:**

  | arm | increment |
  |---|---|
  | D | +0.0044 |
  | F | +0.0026 |
  | E | +0.0013 |
  | A, B, C | 0 |

**What the richer-input nets found.**

- **D, the VAMPnet, is the one real finding here, and it is small.** It closes more than the
  linear sector does: own σ₁ is 0.498 against 0.331 at 1 ps, 0.244 against 0.140 at 5 ps,
  and 0.107 against 0.067 at 10 ps. **It does not re-find q.** Its leading output loads:

  | on | R² |
  |---|---|
  | q | 0.19 |
  | all six STRUCT columns jointly | 0.40 |
  | bond count `nb` | 0.12 |
  | `s2` | 0.03 |
  | `s2w` | 0.00 |
  | density (`n33`, `n50`) | 0.03 |
  | each cage-escape history | ≤ 0.02 |
  | momentum | 0.0000 |

  Sixty per cent of D's leading slow function is therefore nonlinear shell geometry that
  none of SLOW-1's named candidates spans. Beyond velocity and q it carries **+0.0128**,
  which is **+0.0019** over the linear sector's +0.0109. On mobility it adds **+0.006**
  beyond all six STRUCT columns, and +0.004 beyond STRUCT and cage-escape history. At 400 K
  it is +0.0007.
- **D's leading function alone carries less than its 2-D output:** +0.0057 beyond q, against
  +0.0128 for the 2-D output. The carried signal is spread over both output units.
- **A linear readout of the same 84 inputs (R0) carries more than any net's 2-D output
  does:** +0.0194 beyond velocity and q. That is still below the +0.02 bar. The richer
  input holds a little mobility information beyond q, and the slow-variable learners
  compress about two thirds of it into two closed coordinates.
- **SPIB (E)** collapsed to two states on every training seed, after 4–5 refinements and
  25–30 epochs. The package then raised, and the error was caught, as the notes on building
  declared. Its latent loads 0.05 on q and carries almost nothing.
- **F at λ = 10** moved *away* from q (loading 0.065) and toward bond count and density
  (0.14 and 0.11). It kept D's closure (σ₁ 0.496), and it carries *less* on HELD-OUT than
  D does: +0.0076 against +0.0128 beyond q, and +0.030 against +0.043 beyond velocity. The
  carried term, fitted in-batch on TRAIN, did not transfer to the held-out seed. It
  overfits a predictor of the next-5-ps displacement that neither generalises nor stays the
  slow variable (PS-3). λ = 1 is the best F setting (+0.0137), statistically D's number.
  λ = 1 was declared as a report, not as the stake.

**C1 in one line.** The leading directions of A, B and C in STRUCT's standardised coordinates
agree to |cos| **0.9999–1.0000**. Refitted on the 400 K walk itself (no stake), all three
load 0.71 on q and carry +0.0001–0.0002 beyond it. SLOW-1's search, TICA and VAMP are one
instrument on this problem. The programme's contribution is the carried test and the
plants, not the sector.

## 2. What the prereg got wrong (appended to the PREREG under "Notes on building"; no stake moved)

1. **SPIB's convergence settings could not terminate**, and SPIB collapses on this input
   (notes 1 and 2, committed before the read).
2. **F at λ = 10 fails PS-3.** This was found on the synthetic before the read (note 4). The
   prereg chose λ to make the carried term "felt", without a plant on the choice. The plant
   shows that a carried objective at that weight trades the slow variable for an overfit
   predictor.
3. **The branch rule in the reader as first run was wrong.** The run's own printout
   (`slow2_read.txt`, last line) says "(e) a plant fails on F", because the first draft of
   `verdict()` put any F plant failure ahead of the branch. The prereg's (e) is a plant
   failure *on the arm a branch would read*. That arm is D, which passes, and F's failure is
   the integration kill. The rule was corrected to the prereg's text and re-read from the
   saved JSON (`slow2_verdict.txt`). No number changed.
4. **Seen before the full run:** a code-path run on 40 molecules with 3 epochs (NOT A
   READING) printed numbers, including a spurious "F beyond q +0.0167". No setting was
   changed after it. It is recorded here because it was seen.
5. **"Own σ₁" of a 2-D output** is not SLOW-1's six-column σ₁ (note 5). The linear arms'
   0.331 matches SLOW-1's own 0.332 on this seed.

## 3. What this decides

- **Search against the field (C1):** on this problem the search-then-select sector *is*
  TICA/VAMP. That is now said plainly. A claim for the programme's instrument rests on the
  carried test with its re-paired null and its plants, which the field's tools do not ship
  with. It does not rest on the sector the search finds.
- **Learned dictionaries (C2):** a VAMPnet finds a slower coordinate that is not q, and 60 %
  of it is outside the linear span of SLOW-1's dictionary. It carries mobility beyond q by
  +0.013, which is 40 % of q's own carry. That is a direction, and it is below the bar.
  Tetrahedral order remains the variable that carries, on this operator, at 1 ps lag and a
  5 ps horizon.
- **The integration (F):** killed at λ = 10 by its own plant. The honest form of a next
  attempt is λ ≤ 1 with PS-3 run on the λ choice before freezing. It is not proposed here.

## Not tested

Water (this is the rigid operator), experiment, softness or GNN propensity as a contest,
and hyperparameter search.

---
witness: none (a measured read; the gates are numeric)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-MAX-OVER-SUCCESSES, M-ONE-MODEL-DELTA, M-BASE-RATE-OMITTED, M-DEVICE-CLASS. M-MAX-OVER-SUCCESSES: C2's "best" (D) is read as the existence question only. M-ONE-MODEL-DELTA: C3 would earn "against SPIB at these settings" and is not read. M-DEVICE-CLASS: the net numbers are this GPU's, and the training-seed spread is their uncertainty.
