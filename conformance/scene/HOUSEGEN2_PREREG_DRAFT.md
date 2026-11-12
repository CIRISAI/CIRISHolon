# HOUSEGEN-2 — households per tile from the tile's exact density and its surroundings, with a tile-level effect: four generators compared on held-out tiles. PREREGISTRATION (DRAFT)

*REVIEWED 2026-11-11 (`HOUSEGEN2_REVIEW.md`): fixed in `HOUSEGEN2_PREREG.md`, which replaces this text. Kept as written.*

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. It follows
`HOUSEGEN1_RESULTS.md`, which found the class row is the right centre and far too narrow
(real tiles vary 2 to 12 times more than independent draws) and that for most households
the tile they live in is outside tolerance. Nothing below is built or computed. It needs
the German household-size, mean-size and population grid files fetched again.*

## The question

A tile's density class is five steps. The twin knows more about a tile than that: its
exact population, and the population of the tiles around it. Does using them put more
tiles inside tolerance, and does a tile-level effect then give the right spread?

## The generators compared

Each draws households by HOUSEGEN-1's draw from a row; they differ in the row.

- **R0** — the class row (what shipped).
- **R1** — a row chosen by the tile's own population in 16 bands (bounds at the powers of
  two from 4 to 65,536 people per km², the last open).
- **R2** — a row chosen by the tile's band and the band of its SURROUNDINGS: the mean
  population of the 5 × 5 tiles centred on it, missing tiles counted as empty, in the same
  16 bands. A cell of that 16 × 16 table with under 200 fitted tiles uses R1's row.
- **R2+T** — R2 with a tile effect: the row is tilted by a seeded draw, so that tiles of
  one cell differ from each other as fitted tiles do.

**The tilt.** One number `θ` turns a row `s_k` into `s_k · exp(−θ k)`, renormalised: it
moves the one-person share and the mean together. In the crate it is a compiled-in ladder
of 33 levels per row, integers only. For R2+T the level is drawn from a discrete normal
whose spread, per cell, is fitted so the spread of the one-person share across FITTED
tiles of at least 80 households is matched after the draws' own scatter is removed.

## Carrier

Germany's 1 km grid as in `HOUSEHOLD0_DE_PREREG.md`; FIT and SCORE are its three bands of
six, same seed. Rows, the 16 × 16 table and the tilt spreads are fitted on FIT tiles only.
Each SCORE tile is generated once per generator from its census population count, seed its
index; its surroundings are read from the census population grid, including tiles across
the band boundary (a tile's neighbours are an input the twin also has, not a label).

## Stakes

All on SCORE tiles of at least 80 households, household-weighted, tolerances as in
HOUSEGEN-1's S5 (household count within 5 %; one-person share within `0.03` plus two
binomial standard errors).

- **G0.** Section law for each generator; one digest native and wasm32; R0 reproduces
  HOUSEGEN-1's S5 numbers to `0.005`. If not, nothing runs.
- **S1 (does the tile's own density help).** R1's share of households with BOTH quantities
  in tolerance exceeds R0's by at least `0.03`. *Kill:* not higher.
- **S2 (do the surroundings help).** R2's exceeds R1's by at least `0.03`. *Kill:* not
  higher. Then context adds nothing density does not carry.
- **S3 (the spread).** Under R2+T the ratio of the census spread of the one-person share
  across tiles to the generator's lies between `0.8` and `1.25` in each of the five
  classes. *Kill:* outside `0.67` to `1.5` in 2 or more.
- **S4 (what the tile effect costs).** A random tilt cannot know which way a real tile
  leans, so R2+T's per-tile share in tolerance is expected BELOW R2's. Reported, with the
  class-level verdicts of HOUSEGEN-1's S1 for each generator. No kill. If R2+T is not
  below R2, the tolerance is too loose to see a tile.
- **S5 (one number is enough).** Among SCORE tiles, given a tile's census one-person
  share, the tilted row that matches it predicts the tile's census mean household size
  within 5 % for at least 80 % of households. *Kill:* under 60 %. Then mean and
  composition need separate handles.
- **S6 (neighbours, a reading).** The correlation of the one-person share between a tile
  and its eastern neighbour, in the census and under each generator. No stake: it says
  whether a tile effect must be spatially smooth, which is the next design.

## What would make a stake vacuous

- S1 and S2 can be met by density bands that are simply finer than five classes, with no
  new information. R1 against R0 is exactly that test; R2 against R1 is the one about
  surroundings.
- The surroundings are read from the same census as the households. The twin reads
  populations from another model; a gain here is an upper bound on its gain.
- S3 is fitted and scored on the same statistic in two halves of one country. It shows the
  ladder can carry the spread, not that the spread is right elsewhere.
- Disclosure noise is in the census spread (about a tenth of it at 80 households) and is
  not removed.
- Three bands of six is one draw of the hold-out, as HOUSEGEN-1 found: the held-out half
  holds smaller households at equal density. Each generator inherits that offset.

## Plants

- **P-FLAT** (synthetic tiles whose rows do not depend on density). R1 and R2 gain nothing
  over R0: differences under `0.01`.
- **P-DENSITY** (rows that depend on the tile's population alone). R1 gains at least
  `0.05` over R0; R2 gains under `0.01` over R1.
- **P-CONTEXT** (rows that depend on the surroundings at fixed own population). R2 gains
  at least `0.05` over R1.
- **P-TILT** (synthetic tiles with a known tilt spread). The fitted spread is within 10 %
  and S3's ratio reads between `0.9` and `1.1`.
- **P-LADDER** (level 0 of the ladder). It is the row itself; levels are monotone in the
  one-person share.
- **P-THIN-CELL** (a 16 × 16 cell with 50 fitted tiles). It uses R1's row.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing ships.
- **(a)** S1 or S2 met. The best generator by S1 and S2 ships as `household.v2` with its
  tables, with the tile effect included if S3 is met; the per-tile shares are its envelope.
- **(b)** S1 and S2 both killed. `household.v1` stays; what is published is that a tile's
  exact density and surroundings do not place it, and the tile effect alone ships if S3
  is met.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (sizes sum to the unit's people for every seed and every row)
witness: `CIRISHolon.SceneCert.section_does_not_fix_fibre` (a random tile effect gives the right spread without placing any one tile)
witness: none (S1–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the scoring that judges the stakes.
- M-BAR-FROM-THE-READ: the tolerances are HOUSEGEN-1's; R0's per-tile shares are known from it; no quantity of R1, R2 or the tilt has been computed.
- M-BASE-RATE-OMITTED: each generator is scored against R0, and the legacy rule's shares stand beside them from HOUSEGEN-1.
- M-POPULATION-CHOICE: each SCORE tile of at least 80 households, once.
- M-PROVENANCE-OVERREACH: tables carry the census notice and digests; fitted on FIT, shipped as an all-cell fit and labelled so.
- M-FLOOR-UNSTAKED: P-THIN-CELL is the plant for the cell floor.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
