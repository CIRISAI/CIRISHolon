# HOUSEGEN-2 — placing a tile and spreading tiles: households per tile from its exact density, its wider cell and its building mix, with a tile effect in one of two families, compared on held-out tiles. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`HOUSEGEN2_REVIEW.md`)
whose named fixes are applied here. It replaces `HOUSEGEN2_PREREG_DRAFT.md`, kept. It
follows `HOUSEGEN1_RESULTS.md`. At this commit nothing below is built or computed. It needs
four files of the approved German dataset fetched again (household size, mean household
size, population, dwellings by building type).*

*Forecasts on the record. The reviewer expects the mixture family to beat the tilt (S5)
and the building mix to place tiles far better than density (S2b). The author expects the
same of S5 from the published class rows, and has no forecast for S2, S2b or S3.*

## Two consumers

- The **placement** consumer: what is this tile like. Served by a row chosen from what the twin knows
  about the tile, with no random effect.
- The **spread** consumer: how different are tiles from each other. Served by a tile effect: a seeded
  draw, shared by every leaf of the tile, that moves its row. It is a draw from tiles like
  this one and not a prediction of it.

## Placement: four ways to choose a tile's row

- **R0** — the class row (HOUSEGEN-1).
- **R1** — the tile's own population in 16 bands (bounds at the powers of two from 4 to
  65,536 per km², the last open).
- **R2c** — the band by the class of the 1/8 degree cell the tile sits in (16 × 5), the
  cell's density being the census population of the tiles centred in it over the cell's
  whole area. A cell of the table with `n` fitted tiles uses its own row with weight
  `n / (n + 200)` and R1's with the rest.
- **R3** — the band by the share of the tile's dwellings in one- and two-family houses, in
  five equal bands (16 × 5), shrunk the same way. A tile with no dwelling row uses R1.

## Spread: two one-parameter families

From a base row, a ladder of 33 levels (0 is the row itself):
- **TILT:** `s_k · exp(−θ k)`, renormalised, `θ = 0.05 ×` level.
- **MIX:** the one-person share moved by `0.02 ×` level and held within `0.02 … 0.98`; the
  other shares rescaled in proportion.

The level is seeded on the tile's key, so all leaves of a tile share it. Its spread is
fitted per population band on FIT tiles of at least 80 households, household-weighted, on
residuals from R1's row: the variance of the census one-person share across tiles minus
the variance of the shipped draw simulated at level 0 on those tiles' populations, floored
at zero, converted to levels by the ladder's slope at level 0.

## Carrier

Germany's 1 km grid as in `HOUSEHOLD0_DE_PREREG.md`; FIT and SCORE are its three bands of
six, same seed. Each table and spread is fitted on FIT tiles only. Each SCORE tile is
generated once per generator from its census population count; seed and tile key are its
index among household cells sorted by grid coordinate. A tile's 1/8 degree cell is found
from its centre by the inverse of the grid's projection; a cell's class uses tiles of both
halves (it is an input the twin holds, not a label).

## Stakes

On SCORE tiles of at least 80 households, household-weighted. A tile is IN when its
generated household count is within 5 % of the census count and its generated one-person
share within `0.03` plus two binomial standard errors. Each difference between generators
carries a bootstrap over 50 km blocks (500 resamples, seed 20261116). Beside each share:
the fraction of the between-tile variance of the one-person share that the row explains,
and the share IN that a row set to each tile's own census one-person share would reach
under the better family.

- **G0.** The section law for each row and level; one digest on native and wasm32; two
  calls with one tile key and different seeds share a level; R0 reproduces HOUSEGEN-1's
  per-tile shares to `0.005`. If not, nothing runs.
- **S1 (the tile's own density, a reading).** R1's share IN against R0's. Expected higher
  by at least `0.03`. No kill: finer bands of one variable cannot do worse.
- **S2 (the wider cell).** R2c's share IN exceeds R1's by at least `0.03`, with the 5th
  percentile of the bootstrap difference above zero. *Kill:* the 95th percentile of the
  difference at or below zero.
- **S2b (the building mix).** The same for R3 against R1, on tiles that have a dwelling
  row. *Kill:* the same. R3 uses the census's building mix; the twin would use a generated
  one, so this is an upper bound on what it gains.
- **S5 (which family).** For each tile, each family's level is set so the row's one-person
  share equals the tile's census share; the row's mean household size is then compared
  with the tile's census mean. The share of households whose tile is within 5 %: MIX
  exceeds TILT by at least `0.10`. *Kill:* TILT higher. The better family ships.
- **S3 (the spread, on what was not fitted).** With the better family's effect on R1's
  rows, per density class: the 5, 25, 75 and 95 % quantiles across tiles of the generated
  one-person share within `0.02` of the census's, and the ratio of the census spread of
  persons per household to the generator's between `0.8` and `1.25`. *Kill:* that ratio
  outside `0.67` to `1.5` in 2 or more classes.
- **S4 (what the effect costs placement).** The share IN with the effect, beside the share
  without. Expected lower. Reported.
- **S6 (neighbours, a reading).** The correlation between a tile's residual one-person
  share (census minus R1's row) and that of the tile 1, 2, 4 and 8 km east. If it exceeds
  `0.3` at 1 km, a block-level effect is the next design.
- **S7 (region, a reading).** The household-weighted spread across 50 km blocks of the
  block's mean residual from R1, in the one-person share and in persons per household.

## What would make a stake vacuous

- R2c's cell class and R3's building mix are read from the census. The twin reads
  populations from another model and generates its buildings; gains here are upper bounds.
- R3 relates tiles, not households: a tile with more houses has larger households on
  average; nothing says which household is in which building.
- S5's two families are both set to the tile's true one-person share, so much of the mean
  is arithmetic; only the difference between families is evidence.
- The share IN has a low ceiling (tiles differ by about `0.10`, the tolerance is about
  `0.03`). The variance explained and the oracle share are what make a gain readable.
- The spread is fitted on a variance. S3 scores quantiles and a second quantity; it can
  still pass with a wrong shape between the quantiles read.
- Three bands of six is one draw of the hold-out, and the held-out half holds smaller
  households at equal density. Each generator inherits that.
- Disclosure noise is in the census spread (about a tenth at 80 households).

## Plants

Synthetic tiles through the same scoring.

- **P-FLAT** (rows that depend on nothing). R1, R2c and R3 each within `0.01` of R0.
- **P-DENSITY** (rows that depend on the tile's population alone). R1 gains at least
  `0.05` on R0; R2c and R3 under `0.01` on R1.
- **P-CELL** (rows that depend on the cell class at fixed band). R2c gains at least `0.05`.
- **P-MIX** (tiles made under MIX). S5 favours MIX by at least `0.10`. **P-TILT** (tiles
  made under TILT). S5 favours TILT.
- **P-SPREAD** (tiles made with a level spread of 4). The fitted spread is within 10 %.
- **P-LADDER** (in the crate). Level 0 is the row; the one-person share is monotone in the
  level in both families; each level sums to 1,000,000.
- **P-KEY** (in the crate). One tile key, different seeds: one level.
- **P-SHRINK** (a cell with 50 fitted tiles). Its weight is `0.2`.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing ships.
- **(a)** Otherwise. `household.v2` ships with: for placement, the best of R1, R2c and R3
  among those whose stake was met (R1 if neither); for spread, the better family's effect,
  marked certified for spread only where S3 is met. Each call returns a placement verdict
  and a spread verdict.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (sizes sum to the unit's people for every seed, row and level)
witness: `CIRISHolon.SceneCert.section_does_not_fix_fibre` (an effect gives the spread without placing any one tile)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (R3 relates tiles, not households to buildings)
witness: none (S1–S7 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the scoring that judges the stakes, or the shipped function.
- M-BAR-FROM-THE-READ: tolerances are HOUSEGEN-1's. Known at this commit: R0's per-tile shares and the five class rows, from which the reviewer derived the multi-person means that motivate MIX; S5 is scored tile by tile within classes, which nobody has computed.
- M-BASE-RATE-OMITTED: each generator is scored against R0 and R1.
- M-POPULATION-CHOICE: each SCORE tile of at least 80 households, once.
- M-PROVENANCE-OVERREACH: tables carry the census notice and digests; fitted on FIT, shipped as an all-cell fit and labelled so.
- M-FLOOR-UNSTAKED: P-SHRINK is the plant for thin cells.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
