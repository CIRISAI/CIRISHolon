# UNEVEN-0 — how unevenly people sit within a 14 km cell: the transition table from a cell's class to its tiles' classes, for every populated cell, from the population grid alone. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`UNEVEN0_REVIEW.md`) whose
named fixes are applied here, and on the lead's approval of the same day for the one download
it needs ("I approve that data request"). It replaces `UNEVEN0_PREREG_DRAFT.md`, kept. At this
commit nothing has been fetched, no instrument exists, and nothing has been computed on the
grid by anyone; the review ran only synthetic index arithmetic.*

*What this is and is not. It is a reading of the reference's own structure within a cell,
against the even spread the CEWP twin uses today. It is NOT a scene certificate: it has no
budget, no consumer and no envelope. Its stakes are expected to be met by a wide margin (the
review's back-of-envelope says so), so its verdict is close to a confirmation. Its value is
the table it produces, which is the input every later reference and the twin's own placement
need.*

## Why

The review of STREETREF-1 found that a street reference keyed by a 14 km cell's class would
mostly measure where people sit within the cell. The twin spreads a cell's people evenly, so
every tile of a cell gets the cell's class. This campaign measures, exhaustively and with no
map data, how a real cell's tiles and people are distributed over the classes.

## Carrier

- **The file.** GHS-POP R2023A, epoch 2020, WGS84, 30 arc-seconds, from the publisher's
  host (European Commission, Joint Research Centre; CC BY 4.0). The zip is 482,351,880 bytes
  with sha256 `8de311f4…89c45` and the GeoTIFF inside 382,936,052 bytes with sha256
  `630f8b25…35e43`, as the twin's manifest records them (`tools/geodata/geodata_manifest.tsv`
  at twin/geodata). Both are checked before a pixel is read. People per pixel, float64,
  no-data `−200`; 43,202 × 21,384 pixels; the origin as the file states it.
- **Assignment.** A pixel goes whole to the 1/8 degree cell holding its CENTRE, by the twin's
  rule: column `floor((lon + 180) · 8) mod 2880`, row `floor((90 − lat) · 8)`. Interior cells
  hold 225 pixels. The grid is 2 columns wider than 2,880 × 15, so cell columns 0 and 2,879
  hold 240; cell rows 7 and 1,432 hold 180. The people in pixel columns 0 and 43,201 are
  printed.
- **No-data and negatives** count as zero people and still as tiles.
- **Areas.** A pixel's area is `R² · Δφ · Δλ · cos φ` at its central latitude,
  `R = 6,371.0088` km. A cell's area is the exact band on the same sphere, as the twin's
  `pcCellKm2` computes it, so a cell's class agrees with the twin's. The number of cells
  within `0.5 %` of a class threshold is printed.
- **Cell classes.** The twin's five (twin/globe `a29e87d`): lower bounds
  `0 / 250 / 1,500 / 6,000 / 12,000` people per km² of the cell's WHOLE area; a cell under
  `1` per km² contributes nobody and is not read.
- **Tile classes.** SIX: empty (under `1` per km² of the tile's own area), then the five.
  An empty tile is not village.
- **Grain.** Every reading at three tile sizes: 1 pixel; blocks of 2 × 2 pixels; blocks of
  4 × 4 pixels, the count along longitude scaled by `1/cos φ` and rounded, so the three are
  about 0.9, 1.9 and 3.7 km on a side everywhere.
- **Weights.** Every summary twice: each cell once (the weight for things laid out per area,
  such as streets, and the generator's own error rate), and each cell by its people (the
  weight for households, devices and a mesh). The number of cells behind each number is
  printed.
- The **regions.** The five density classes alone. The twin's region table is not yet published
  as data; when it is, the same per-cell rows are re-summarised by region with no new read.

## What is produced

1. **The transition table.** For each of the five cell classes: the share of its tiles, and
   of its people, in each of the six tile classes. At three grains, two weights.
2. **Per cell, one row:** its class, people, the six tile shares, the six people shares, the
   Gini coefficient of people over its tiles (population form), at 1 pixel grain. About
   300,000 rows. No pixel value is kept.
3. **Counts:** cells per class, and per class the number holding at least one tile of each
   class.

The publisher's credit travels with every table: *"GHS-POP R2023A, European Commission, Joint
Research Centre; Schiavina, Freire, Carioli and MacManus (2023), doi:10.2905/2FF68A52-5B5B-4A22-8F40-C41DA8332CFE; CC BY 4.0."*

## Stakes

- **G0 (the same grid as the twin's; if any part fails, nothing runs).**
  1. The sum of all pixels is `7,840,952,947.01` to `0.01`.
  2. Every half-degree total equals the twin's shipped `pop0` (float32) to a relative
     `10⁻⁶` or `0.5` people, whichever is larger.
  3. The campaign's 1/8 degree sums, encoded by the twin's own formula
     (`floor((log10(p) + 1) · 255/8) + 1`, zero under `0.1` person, after its rescale of each
     4 × 4 block to the half-degree total), equal the twin's shipped `pop1` in every cell but
     at most `100`.
- **S1 (the even spread mislabels most tiles).** In the suburban class, at 1 pixel grain,
  the median over cells of the share of tiles in the cell's own class is below `0.5`, under
  both weights. The village class is reported. *Kill:* above `0.8`.
- **S2 (and most people).** In the suburban class, at 1 pixel grain, weighted by people: the
  median share of a cell's people in tiles of the cell's own class is below `0.5`, and the
  median share in tiles of a HIGHER class is above `0.4`. *Kill:* the own-class share above
  `0.8`.
- The **reported, not staked:** everything else, including the Gini by class (for classes
  holding at least 30 cells, with the note that a cell's mean bounds its Gini from above, so
  a fall with density is partly arithmetic), and the counts of cells in the top two classes
  (the review estimated 150 to 300 and 30 to 60 from city sizes).

**The bars are the author's priors, not sourced.** The reviewer's back-of-envelope for a
suburban cell (a town on 15 to 20 % of the area at about 3,000 per km², the rest near 100)
gives an own-class tile share of `0.1` to `0.25`, an own-class people share of `0.15` to
`0.3`, and a higher-class people share of `0.6` to `0.75`. No pilot is run: any block of the
same grid would make the rest a replication.

## What limits every reading

- A tile of about 0.9 km is not a street leaf; unevenness below a kilometre is invisible
  here, and the file is a mass-preserving resampling of the publisher's equal-area grid,
  which blurs by an amount that varies over the globe. Every reading is a LOWER bound on how
  uneven a cell is.
- The fine grid is itself a model: census counts disaggregated onto built-up area.
- Coastal cells read as uneven because sea pixels hold nobody and the cell's class uses its
  whole area. The file carries no land mask; this cannot be separated here.

## Plants (each runs before the real grid is read, on synthetic rasters)

- **P-FLAT.** A cell with every pixel equal at 400 per km²: own-class tile share `1`, Gini
  `0`, empty share `0`.
- **P-TOWN.** 9 pixels (rows 6 to 8, columns 6 to 8) at 3,000 per km², the rest empty, cell
  top at 0.125 N: cell density `120.000`, village; own-class tile share `0.000`; empty share
  `0.960`; own-class people share `0.000`; higher-class people share `1.000`; Gini `0.960`.
- **P-LAT.** The same 9 pixels holding 1,000 people each. Cell top at 0.125 N: each pixel
  `1,164.6` per km², suburban. Cell top at 60.125 N: `2,333.7` per km², terraced. The cell is
  village both times (`46.59` and `93.35`).
- **P-ASSIGN.** A raster of the file's size and origin with one person in each of: a pixel
  whose centre is 0.45 pixel from a cell edge; pixel columns 0 and 43,201; pixel rows 0 and
  21,383. Each must land in its stated cell: cell columns 2,879 and 0, cell rows 7 and 1,432.
- **P-SHIFT.** G0 run with every pixel assigned one column east: part 2 must fail.
- **P-FLIP.** G0 run with the rows reversed: part 1 holds and part 2 fails.
- **P-NODATA.** A cell with `−200` in 10 pixels: they count as zero people and as 10 tiles.

P-SHIFT and P-FLIP need the real grid and the twin's shipped values; they run after G0
passes and before any stake line is printed.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier: each synthetic raster is built to hold the spread, latitude, edge or missing
values its plant names; P-SHIFT and P-FLIP act on a grid whose columns and rows differ, which
G0 part 2's own passing shows.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** S1 and S2 met. A reference keyed only by a cell's class describes a mixture of
  tile classes, with the weights in the transition table; later references carry the tile's
  own class as a key, and whether that key is enough is theirs to show.
- **(k)** S1 or S2 killed. The even spread is nearly right at this grain.
- **(b)** S1 or S2 neither met nor killed: stated with the reading.

## Order

1. This file, alone. 2. The fetch, with size and digests checked. 3. The instrument and its
synthetic plants. 4. G0, P-SHIFT, P-FLIP. 5. The tables and the two stakes. The raw file is
deleted after the tables are written and checked.

## What this does not claim

Nothing about streets, buildings or any network; nothing below about 1 km; nothing about the
truth of the fine grid; one epoch; no certificate.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (the pixels of a cell sum to the cell; an instance by construction)
witness: `CIRISHolon.SceneCert.section_does_not_fix_fibre` (the even spread satisfies the section law and need not match)
witness: none (S1 and S2 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: every plant is built for this reader; the synthetic ones run before the grid is read.
- M-BAR-FROM-THE-READ: the two bars are the author's priors with their reasoning stated, set before any read; no pilot touches the grid.
- M-BASE-RATE-OMITTED: beside every own-class share, the share of ALL tiles in that class is printed.
- M-POPULATION-CHOICE: no sample is drawn; completeness is gated against the publisher's world total in G0.
- M-PROVENANCE-OVERREACH: both digests are checked; a digest names bytes, and that those bytes are the twin's grid rests on G0 parts 2 and 3.
- M-FLOOR-UNSTAKED: per-class summaries are printed only for classes holding at least 30 cells, with the count.
- M-VACUOUS-SUCCESS: the run prints the number of pixels and cells read.
