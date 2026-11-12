# APPEARANCE-0 — a climate class for each cell of the twin's 1/8° grid, reduced from the Köppen-Geiger maps of Beck et al. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`APPEARANCE0_REVIEW.md`)
whose named fixes are applied here. It replaces `APPEARANCE0_PREREG_DRAFT.md`, kept. The lead's word, 2026-11-12: "You have approval for data
sources, lets make holon rich and gorgeous", after this source was put to them by name
(`SCOUT_APPEARANCE.md` section 5, item 1). At this commit the file has not been downloaded.*

## What it is for

The twin's renderer is to read tables keyed by density class and climate zone (built
form, trees, roofs, season). It has no climate key. This campaign makes the key: one
class code per cell of the twin's grid. It certifies nothing about how a place looks.

## Carrier

- **File:** `koppen_geiger_tif.zip` (130,618,411 bytes, md5 `7fc2f5a15d4f5fe0ce59c9a9b502aa09` as the
  record lists it), file 61012822 of the figshare record
  `https://doi.org/10.6084/m9.figshare.21789074` (version 2), linked from
  `https://www.gloh2o.org/koppen/`.
- **Licence, read at both places.** The record's licence field: "CC BY 4.0",
  `https://creativecommons.org/licenses/by/4.0/`. The page: "The data and code are
  provided under the Creative Commons Attribution 4.0 International (CC BY 4.0) license,
  allowing you to freely use, adapt, and share the maps for both commercial and
  non-commercial purposes." One link on that page points at the NonCommercial deed; the
  page's words and the record's field agree with each other and not with that link, and
  they are followed. The record is read again at the fetch; if its licence field no
  longer reads CC BY 4.0, nothing is written.
- The record says it was updated in January 2026 to correct a calculation error; version
  2 is that corrected edition. If the fetched bytes or md5 differ from the record's,
  nothing is written.
- The raster read is the 1991 to 2020 one at 1 km. Other periods, the scenarios and any
  confidence raster in the archive are not read.
- **Attribution asked:** cite Beck et al. (2023), Scientific Data 10, 724.
- The raw file is not committed and is deleted after the run; digest and bytes recorded.

## What is known before the read

- The scout's description: maps for 1991 to 2020 at 1 km, with coarser grids and a
  `legend.txt`, 30 classes. Not read: the archive's file names, the pixel grid's origin
  and size, the compression, the legend's codes.
- The twin's grid, from its lead: 2880 columns by 1440 rows; row 0 northernmost,
  r = floor((90 − lat) / 0.125); column 0 at longitude −180; flat index r × 2880 + c.
- General knowledge used as a reading check below (R1): the Köppen class commonly given
  for 12 well-known places. These are textbook labels, not values of this file.

## Before the parse

The record before the parse may read the archive's file list, `legend.txt`, and each
candidate raster's size, type, compression, no-data value and georeference tags, and no
pixel. It fixes which file is the 1991 to 2020 raster at 1 km (exactly one candidate, or
branch (e)) and the reader used. Required, or branch (e):

- the raster is 43,200 by 21,600 with its upper-left corner at (−180, 90) and a pixel of
  1/120°, so that 15 by 15 pixels nest in each cell;
- the legend holds 30 entries with codes 1 to 30, and code 1 is Af, 4 is BWh, 15 is Cfb,
  29 is ET and 30 is EF (symbols stated here from memory of the published legend, as a
  prior);
- this machine can decode it. There is no coarser fallback: a 0.1° grid does not nest in
  these cells.

## What is built

`tables/appearance0/`, each grid 4,147,200 bytes when unpacked, one byte per cell in the
twin's order (flat index r × 2880 + c, key name `cell`), gzipped with no time stamp or
name so that the bytes are reproducible:

- `koppen_1991_2020_cell.bin.gz`: the class code. 0 is no class. Otherwise the class
  holding the most of the cell's classed source pixels; a tie goes to the lower code.
- `koppen_1991_2020_cell_share.bin.gz`: the majority's share of the cell's classed
  pixels, as round-half-up of 255 × k / n (0 where the class is 0).
- `koppen_1991_2020_cell_classed.bin.gz`: the count of classed source pixels in the cell,
  0 to 225.
- A sidecar: the legend (code, symbol, name; the source's map colours only if marked
  "the source's map colours, not an appearance"); cells by class; the tie count; the
  period; the attribution and licence; sha256 of each grid packed and unpacked; "class
  not 0 does not mean land: a cell with 1 classed pixel in 225 has a class; the twin's
  own land mask decides"; `holds_no_real_feature: true`. On that flag: the grid is keyed
  as the twin's land-cover grid is, and the twin's lead asked for it in this form on
  2026-11-12 ("one class code per cell as a byte array in that order, with the legend in
  the sidecar. Name the key `cell`... A cell with no class (sea) is 0").

## Plants (a synthetic raster through the same reducer)

- **P-ORIENT.** A raster with one class in its north-west corner block lands in cell 0;
  one in the south-east lands in the last cell.
- **P-MAJOR.** A cell with 60 % of one class and 40 % of another gets the first, share
  153 of 255.
- **P-TIE.** An exact tie gets the lower code.
- **P-SEA.** A cell with no classed pixel is 0 with share 0; a cell that is 90 % sea gets
  the class of its land pixels and a classed count of 22 or 23 as the plant builds it.
- **P-WRAP.** Longitude −180 is column 0 and +179.9 is column 2879.
- **P-LEGEND.** A code not in the legend stops the run (the source's no-data value is exempt).

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic raster holds what its plant names.

## Stakes

- **R1 (the grid reads as the textbook reads), 12 places.** The cell is
  r = floor((90 − lat) / 0.125), c = floor((lon + 180) / 0.125) at the coordinates below;
  a cell of class 0 is a miss. Met: at least 10 of the 12. *Misread:* 6 or fewer of the
  12; nothing is written. Between: written, with each miss named.

| place | lat, lon | allowed |
|---|---|---|
| Singapore | 1.35, 103.82 | Af |
| Manaus | −3.10, −60.02 | Af, Am |
| Cairo | 30.04, 31.24 | BWh |
| Riyadh | 24.71, 46.68 | BWh |
| Phoenix | 33.45, −112.07 | BWh, BSh |
| Madrid | 40.42, −3.70 | BSk, Csa |
| Rome | 41.90, 12.50 | Csa |
| London | 51.51, −0.13 | Cfb |
| Sydney | −33.87, 151.21 | Cfa, Cfb |
| Moscow | 55.75, 37.62 | Dfb |
| Yakutsk | 62.03, 129.73 | Dfc, Dfd, Dwc, Dwd |
| Nuuk | 64.18, −51.72 | ET |

- **R2 (land), north of 60°S.** Among rows 0 to 1199 (3,456,000 cells), the count with a
  class lies from 898,560 to 1,244,160 inclusive (a share of 0.26 to 0.36). Outside:
  nothing is written. The share in rows 1200 to 1439 is printed with no verdict.
- **R3 (north and south are not swapped).** The classed share in rows 240 to 479 (30°N to
  60°N) is over 2 times that in rows 960 to 1199 (30°S to 60°S). Otherwise misread.
- **S1 (does one class per cell mislead?).** Among classed cells, the share whose
  majority holds under 0.8 of the cell's classed pixels, from pixel counts. Met: at most
  `0.15`. *Kill:* over `0.35`; then a single class per cell is not a fair reduction, the
  sidecar carries `single_class_fair: false`, and a second-class grid is a further
  amendment. Between: neither. The two bars are priors, not measurements.
- **Printed, no verdict.** Cells by class; the 5 largest; the share at each main group;
  S1's share by main group; the share of classed cells in which pixels outside the
  majority's main group are at least 0.2; the tie count.

## What would make this vacuous

- R1 is 12 places chosen for being unambiguous; it catches a flipped grid, a shift of
  half the globe and a scrambled legend. It cannot see a shift of one cell; the
  georeference requirement and P-ORIENT carry that.
- The count of classed cells is inflated at coasts by the any-land-pixel rule.
- The raw file is deleted, so a later check can compare digests, shapes and class counts
  with the sidecar and cannot regenerate the grids.
- The classes are a model's output from gridded climate data for 1991 to 2020, not
  observations at a place, and say nothing of a year.
- A climate class is a key. No appearance follows from it until a table keyed by it is
  certified against something.

## Branches

Exactly one fires, in this order.

- **(e)** A plant fails; or the record's licence field no longer reads CC BY 4.0; or the
  bytes or md5 differ from the record's; or a requirement of "Before the parse" is not
  met; or the reducer stops. Nothing is written.
- **(r)** Otherwise, R1 reads misread, or R2 is outside its range, or R3 reads misread.
  Nothing is written.
- **(a)** Otherwise. The three grids and the sidecar are written; the sidecar carries
  R1's misses and S1 as met, neither or kill.

---
witness: none (a reduction of a published map; R1, R2 and S1 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the reducer that writes the grid.
- M-BAR-FROM-THE-READ: the bars and the 12 places are stated here before the file is fetched.
- M-PROVENANCE-OVERREACH: digest, version, period and licence recorded; the grid is called a key, not an appearance.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
