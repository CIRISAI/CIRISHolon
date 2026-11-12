# UNEVEN-0 — how unevenly people sit within a 14 km cell, measured for every cell from the population grid alone. PREREGISTRATION (DRAFT)

**REVIEWED 2026-11-11: freeze only after the named fixes in `UNEVEN0_REVIEW.md` (the class of an empty pixel, the grid's edge counts, a gate that gates). Kept as drafted.**

*DRAFT of 2026-11-11. NOT FROZEN, and not to be frozen before a second reader has attacked it.
Nothing has been fetched or computed. It needs one file, the same GHS-POP 2020 grid the CEWP
twin already ships a reduction of, at its native 30 arc-second resolution (482 MB compressed,
CC BY 4.0, from the publisher's host); the file is not on this machine and the fetch waits on
the freeze and on the lead's word for that one download.*

## Why

The review of STREETREF-1 (`STREETREF1_REVIEW.md`) found that a street reference keyed by a
14 km cell's class would mostly measure something else: where people sit WITHIN the cell. The
twin spreads a cell's people evenly over it, so every tile of a cell gets the cell's class
and the cell's streets. A real cell averaging 300 people per km² is mostly fields with a
town in it. That mismatch needs no street data to measure. It is in the population grid
itself, exhaustively, for every populated cell on Earth, and it decides how every later
reference must be keyed.

In the world model's terms this is the first scene certificate with a reference in hand: the
generator is the twin's even spread (a section of the cell's count, exactly), the reference
is the real fine grid, and the observable is the class a tile is given.

## Carrier

- **The fine grid.** GHS-POP R2023A, epoch 2020, WGS84, 30 arc-seconds: people per pixel.
  Each 1/8 degree cell holds exactly 15 × 15 = 225 pixels.
- **The cell.** As the twin defines it (`classes.json`, twin/globe `a29e87d`): a 1/8 degree
  cell; its density is its people over its WHOLE area; under `1` per km² it contributes
  nobody and is excluded. Class thresholds `0 / 250 / 1,500 / 6,000 / 12,000` per km².
- **The tile.** One 30 arc-second pixel, classed by its own people over its own area,
  `R² · Δφ · Δλ · cos φ` at the pixel's central latitude with `R = 6,371.0088` km.
- **Per cell, the readings.**
  - `same_tiles`: the share of its 225 pixels whose own class equals the cell's class.
  - `same_people`: the share of its PEOPLE living in pixels whose own class equals the cell's.
  - `empty_tiles`: the share of its pixels under the floor of `1` per km².
  - `up_people`: the share of its people in pixels of a HIGHER class than the cell's.
  - `gini`: the Gini coefficient of people over its 225 pixels.
- **Per class** (the twin's 9 regions × 5 density classes, where the region table is
  available; the 5 density classes alone otherwise): the distribution of each reading over
  cells, as quantiles at `0.1, 0.25, 0.5, 0.75, 0.9`, weighted two ways: every cell once, and
  every cell by its people.
- **What is kept.** The per-class tables, and one row per cell of the five readings (about
  300,000 populated cells; a few MB). No pixel values. The publisher's credit travels with
  both.

## Stakes

- **G0 (the section law, on real data).** For every cell the 225 pixels sum to the cell's
  people as this campaign computes it, EXACT in the arithmetic used (stated in the build).
  And against the twin's own shipped cell values (`pop1`, a log code with a stated error of
  at most about `7 %` per cell after its rescale): at least `99 %` of populated cells agree
  within `8 %`. If not, the two repositories are not reading the same grid and nothing runs.
- **S1 (the even spread mislabels most tiles).** In the village and the suburban class, the
  median over cells of `same_tiles` is below `0.5`. *Kill:* above `0.8` in both. Then the
  even spread is nearly right about tiles and the review's first objection is wrong.
- **S2 (and mislabels most people).** In the suburban class, the people-weighted median of
  `same_people` is below `0.5`, and of `up_people` above `0.4`: most people in a
  "suburban" cell live at a higher density than the cell's. *Kill:* `same_people` above
  `0.8`.
- **S3 (it is worst where it matters least by area and most by people).** The median `gini`
  falls from the village class to the dense-fabric class: lower in each denser class than
  in the one before, for at least 3 of the 4 steps. No kill: either pattern is a reading.
- **S4 (how many cells the top classes hold).** The number of cells in the apartment-blocks
  and dense-fabric classes, per region. No stake; the review estimated 150 to 300 and 30 to
  60 in the world from city sizes, and this is the count.

## What would make a stake vacuous

- S1 and S2's directions are expected from first principles; their bars (`0.5`, `0.4`) are
  the author's and are not sourced. A declared pilot on one 10 × 10 degree block, excluded
  from the stakes, would source them, and is owed before the freeze.
- A pixel of about 0.9 km is not a street leaf. The twin's leaf can be much smaller, and
  unevenness below a kilometre is invisible here. Every reading is a LOWER bound on how
  uneven a cell is at the leaf's grain.
- The fine grid is itself a model: a disaggregation of census counts by built-up area. It is
  the reference, and the certificate is against it.

## Plants

- **P-FLAT** (carrier: a synthetic cell with every pixel equal; sector: the spread). It must
  read `same_tiles = 1`, `gini = 0`, `empty_tiles = 0`.
- **P-TOWN** (carrier: a synthetic cell with all its people in 9 of 225 pixels at a stated
  density; sector: the spread). Readings stated to 3 digits before the run from the
  construction.
- **P-LAT** (carrier: P-TOWN at 0 and at 60 degrees north; sector: the pixel's area). The
  pixel class must change where the halved area doubles its density, at the stated pixel.
- **P-EDGE** (carrier: the grid's first and last rows and the date line; sector: the cell
  index). 15 × 15 pixels must map to one cell everywhere, with no cell short of 225.
- **P-NODATA** (carrier: a synthetic cell with the publisher's no-data value in 10 pixels;
  sector: the count). They must count as zero people and still as tiles.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction for every plant here: each carrier is built to hold the spread,
the latitude, the edge or the missing values the plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** S1 and S2 met. The even spread is a section of the cell's count and a poor match
  to where people are: every reference below the cell must be keyed by the TILE's own class,
  and the twin's in-cell weighting is the first thing a certified scene must supply.
- **(k)** S1 or S2 killed. The even spread is nearly right at this grain, and the street
  reference may be keyed by the cell's class after all.

## What this does not claim

Nothing about streets, buildings or any network. Nothing below about 1 km. Nothing about the
truth of the fine grid. One epoch.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (G0: the pixels of a cell sum to the cell)
witness: `CIRISHolon.SceneCert.section_does_not_fix_fibre` (the even spread satisfies the section law and need not match)
witness: none (S1–S4 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader and runs before the grid is read.
- M-BAR-FROM-THE-READ: the bars are unsourced today; the declared pilot is owed before the freeze.
- M-BASE-RATE-OMITTED: every reading is printed both per cell and weighted by people.
- M-POPULATION-CHOICE: no sample is drawn; every populated cell is read.
- M-PROVENANCE-OVERREACH: the file's sha256 is checked against the publisher's and the twin's recorded value; a digest names bytes.
