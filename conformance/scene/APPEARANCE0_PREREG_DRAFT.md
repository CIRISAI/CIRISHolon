# APPEARANCE-0 — a climate class for each cell of the twin's 1/8° grid, reduced from the Köppen-Geiger maps of Beck et al. PREREGISTRATION DRAFT

*Draft 2026-11-12. Not frozen. The lead's word, 2026-11-12: "You have approval for data
sources, lets make holon rich and gorgeous", after this source was put to them by name
(`SCOUT_APPEARANCE.md` section 5, item 1). At this draft the file has not been downloaded.*

## What it is for

The twin's renderer is to read tables keyed by density class and climate zone (built
form, trees, roofs, season). It has no climate key. This campaign makes the key: one
class code per cell of the twin's grid. It certifies nothing about how a place looks.

## Carrier

- **File:** `koppen_geiger_tif.zip` (130,618,411 bytes, md5 `7fc2f5a1…aa09` as the record
  lists it), file 61012822 of the figshare record
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
candidate raster's size, type, compression and georeference tags, and no pixel. It fixes:
which raster is "1991 to 2020 at the finest grid this machine's tools can decode" (the
1 km grid if its compression can be read here, else the next coarser, said plainly); the
code-to-class legend; how a source pixel maps to a cell.

## What is built

`tables/appearance0/`:

- `koppen_cell.bin.gz`: 4,147,200 bytes when unpacked, one class code per cell in the
  twin's order. 0 is no class (sea, or no land pixel in the cell). Otherwise the class
  holding the most of the cell's classed source pixels; a tie goes to the lower code.
- `koppen_cell_share.bin.gz`: the same shape; the majority class's share of the cell's
  classed pixels, in 255ths (0 where the class is 0).
- A sidecar: the legend (code, symbol, name) as the file gives it; the count of cells by
  class; period 1991 to 2020; the source grid used; the tie rule; the attribution and
  licence; `holds_no_real_feature: true` (a class per grid cell is a climate statistic,
  not a feature); the key name `cell`.

## Plants (a synthetic raster through the same reducer)

- **P-ORIENT.** A raster with one class in its north-west corner block lands in cell 0;
  one in the south-east lands in the last cell.
- **P-MAJOR.** A cell with 60 % of one class and 40 % of another gets the first, share
  153 of 255.
- **P-TIE.** An exact tie gets the lower code.
- **P-SEA.** A cell with no classed pixel is 0 with share 0; a cell that is 90 % sea gets
  the class of its land pixels.
- **P-WRAP.** Longitude −180 is column 0 and +179.9 is column 2879.
- **P-LEGEND.** A code not in the legend stops the run.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic raster holds what its plant names.

## Stakes

- **R1 (the grid reads as the textbook reads), 12 places.** The cell containing each
  place has the class commonly given for it, or one of the listed neighbours: Singapore
  Af; Manaus Af or Am; Cairo BWh; Riyadh BWh; Phoenix BWh; Madrid BSk or Csa; Rome Csa;
  London Cfb; Sydney Cfa; Moscow Dfb; Yakutsk Dfd or Dfc; Nuuk ET. Met: at least 10 of 12.
  *Misread:* 6 or fewer; nothing is written (the orientation or the legend is wrong).
  Between: written, with each miss named.
- **R2 (land).** The share of cells with a class lies between `0.25` and `0.45`. Outside:
  nothing is written.
- **S1 (does one class per cell mislead?).** Among cells with a class, the share whose
  majority holds under half of the cell's classed pixels. Met: at most `0.10`. *Kill:*
  over `0.25`; then a single class per cell is not a fair reduction and the twin must
  read the share file and blend. Between: neither; the share file is recommended.
- **Printed, no verdict.** Cells by class; the 5 largest classes; the share of classed
  cells at each main group (A, B, C, D, E).

## What would make this vacuous

- R1 is 12 places chosen for being unambiguous; it catches a flipped or shifted grid and
  a scrambled legend, and nothing subtler.
- The classes are a model's output from gridded climate data for 1991 to 2020, not
  observations at a place, and say nothing of a year.
- A climate class is a key. No appearance follows from it until a table keyed by it is
  certified against something.

## Branches

Exactly one fires, in this order.

- **(e)** A plant fails; or the record's licence field no longer reads CC BY 4.0; or the
  archive holds no 1991 to 2020 raster this machine can decode; or the reducer stops.
  Nothing is written.
- **(r)** Otherwise, R1 reads misread or R2 is outside its range. Nothing is written.
- **(a)** Otherwise. The two grids and the sidecar are written, with R1 and S1 as read.

---
witness: none (a reduction of a published map; R1, R2 and S1 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the reducer that writes the grid.
- M-BAR-FROM-THE-READ: the bars and the 12 places are stated here before the file is fetched.
- M-PROVENANCE-OVERREACH: digest, version, period and licence recorded; the grid is called a key, not an appearance.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
