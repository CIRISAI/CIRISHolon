# HOUSEGEN-2 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `HOUSEGEN2_PREREG_DRAFT.md`. No data fetched or opened. One
single-threaded synthetic script (`housegen2/review_synthetic.py`) on a made-up row and
made-up tile sizes; real numbers used are the five class rows already published. The
author's condensation; the draft is kept as written.*

**Verdict.** The main fault: the draft's one-parameter family is very probably wrong, and
this repository's own published rows already said so.

1. **The exponential tilt ties the mean to the one-person share the wrong way.** A tilt
   that raises the one-person share also shrinks the multi-person households. The
   published German class rows do the opposite: the mean size of multi-person households
   is `2.87, 2.83, 2.83, 2.86, 2.89` from village to dense fabric while the one-person
   share runs from `0.32` to `0.62`. Across classes real tiles follow a MIXTURE: a share of
   one-person households and a fixed row for the rest. Against it the tilt's mean is off
   by 8 to 14 % for a shift of `0.10`, more than the 5 % tolerance. **Fix:** compare both
   families and ship the better; the mixture is as simple in integers.
2. **The tile effect must be seeded on the TILE.** The twin divides a tile over leaves; an
   effect drawn per call averages away. The call needs a tile key.
3. **The 5 × 5 surroundings table** is mostly empty, biases border and coast tiles (the
   file cannot tell empty from abroad, and the twin's grid can), and cannot tell a market
   town from a city centre. **Fix:** the class of the 1/8 degree cell the tile sits in,
   which the twin holds already; shrink thin cells toward the density-only row.

**Also.**
- **A covariate the draft missed:** the share of a tile's dwellings in one- and two-family
  houses. It is on the German grid, the twin generates buildings, and it makes households
  consistent with the buildings they are put in. A relation between tiles is not a
  relation between households, and it must be labelled so.
- **The spread estimator:** household-weighted, on residuals from the row, with the
  shipped draw's own scatter removed by simulating it on the fitted tiles.
- **The 0.03 bars** need a bootstrap over 50 km blocks; the share in tolerance has a low
  ceiling, so the variance explained and an oracle ceiling go beside it. The kill on
  "finer bands of one variable" cannot fire.
- **S3 was fitted and scored on one statistic.** Score what was not fitted: the quantiles
  of the tile one-person share, and the spread of persons per household.
- **A random effect lowers per-tile accuracy by construction.** It is honest only as a
  second consumer: placement (the row) and spread (the effect) get separate verdicts.
- **Neighbour correlation** on residuals, not raw (raw is the density gradient), at
  several distances, with a declared threshold for a block-level effect.
- **Region** is unlearnable under a band hold-out; print the size of a regional offset.
- **Age structure** is the strongest predictor and out of reach: the twin has no ages.

**What the author took and did not take.** The "smallest honest version" is taken: R0,
R1, R2c (cell class) and R3 (building mix) for placement; MIX against TILT; the effect and
the rewritten S3; neighbour correlation on residuals. The 5 × 5 table is dropped.
