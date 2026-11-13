# REGION-0 — a US state for each cell of the twin's 1/8° grid, reduced from Natural Earth's states and provinces. PREREGISTRATION DRAFT

*Draft 2026-11-12. Not frozen. The lead's standing word, 2026-11-12: "You have approval
for data sources, lets make holon rich and gorgeous". This source was not put to them by
name before; it is public domain as read below, and it is reported to them with the
result. At this draft the file has not been downloaded.*

## What it is for

BUILDGEN-1 was killed because a dwelling's joint is regional below climate: the same
climate zone is one fuel mix in one state and another in the next. A generator keyed by
region needs the caller to know its region. The twin holds countries alone. This campaign
makes the key: a US state code per cell, in the same shape as the climate grid. It
certifies nothing about any state.

## Carrier

- **File:** `ne_50m_admin_1_states_provinces.zip`, 911,408 bytes, Natural Earth version 5.1
  as its page lists it, from `https://naciscdn.org/naturalearth/50m/cultural/`.
- **Licence, read on its terms page:** "All versions of Natural Earth raster + vector map
  data found on this website are in the public domain." and "No permission is needed to
  use Natural Earth." Credit offered, not required: "Made with Natural Earth." The page
  is read again at the fetch; if it no longer says so, nothing is written.
- Not committed; deleted after the run; digest and bytes recorded.

## What is known before the read

- The twin's grid, from its lead: 2880 by 1440, row 0 northernmost, column 0 at
  longitude −180, flat index r × 2880 + c.
- From memory, as priors: the state each of 14 cities is in, and the land area of 10
  large states as the Census Bureau gives it (below). These are not values of the file.
- Not known: the file's attribute names, whether the District of Columbia is a shape at
  this scale, how shapes that cross longitude 180 are cut.

## Before the parse

The record before the parse may read the archive's file list, the attribute table's
column names, the distinct values of the country column, and the count of shapes, and no
coordinate. It fixes the columns used for country, state name and postal code, and the
code table: a state's code is its 2-digit federal code as a number (Alabama 1 to Wyoming
56, the District of Columbia 11), from the postal code by a table written in the script.

## What is built

`tables/region0/`, each grid 4,147,200 bytes unpacked, one byte per cell in the twin's
order, key `cell`, gzipped with no time stamp:

- `us_state_cell.bin.gz`: the state's code; 0 where no sample point of the cell lies in a
  US state. Each cell is sampled at 25 points (a 5 by 5 lattice of sub-cell centres); the
  state with the most points wins; a tie goes to the lower code.
- `us_state_cell_share.bin.gz`: the winner's share of the cell's points that lie in any
  state, round-half-up of 255 × k / n.
- `us_state_cell_classed.bin.gz`: how many of the 25 points lie in any state.
- A sidecar: the code table (code, postal code, name, and the survey's census division,
  10 of them, from its codebook); cells by state; attribution; "a cell's state is the
  majority of 25 sample points; a border cell holds two states and the share says so; a
  code is not land (the twin's own mask decides)"; `holds_no_real_feature: true`, on the
  twin's lead's word that a state code is a coarse statistical key.

## Plants (synthetic shapes through the same rasteriser)

- **P-SQUARE.** A square of known corners fills exactly the cells inside it.
- **P-HOLE.** A square with a square hole leaves the hole empty.
- **P-BORDER.** Two squares meeting inside a cell: the larger part wins; an exact tie
  goes to the lower code; the share is the winner's points over both.
- **P-ORIENT.** A shape in the north-west lands in low rows and low columns.
- **P-MULTI.** A state of two separate parts gets both.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic shape holds what its plant names.

## Stakes (reading checks; this is a key)

- **R1 (cities), 14.** The cell at each coordinate has the state named: Syracuse 43.05,
  −76.15 New York; Los Angeles 34.05, −118.24 California; Chicago 41.88, −87.63 Illinois;
  Houston 29.76, −95.37 Texas; Phoenix 33.45, −112.07 Arizona; Denver 39.74, −104.99
  Colorado; Atlanta 33.75, −84.39 Georgia; Seattle 47.61, −122.33 Washington; Miami
  25.76, −80.19 Florida; Minneapolis 44.98, −93.27 Minnesota; Anchorage 61.22, −149.90
  Alaska; Honolulu 21.31, −157.86 Hawaii; Boise 43.62, −116.20 Idaho; Nashville 36.16,
  −86.78 Tennessee. Met: at least 13 of 14. *Misread:* 8 or fewer; nothing is written.
  Between: written, each miss named.
- **R2 (areas), 10 states.** The area of a state's sample points on a sphere of radius
  6371.0088 km, against its land area in km² from memory: Alaska 1,477,953; Texas
  676,587; California 403,466; Montana 376,962; New Mexico 314,161; Arizona 294,207;
  Nevada 284,332; Colorado 268,431; Oregon 248,608; Wyoming 251,470. Within `5 %`
  (the shapes include inland water, and the outline is generalised). Met: at least 9 of
  10. *Misread:* 5 or fewer; nothing is written. Between: written, each miss named.
- **R3 (all present).** Each of the 50 states has at least 1 cell. Otherwise nothing is
  written. The District of Columbia (smaller than a cell) is printed, no verdict.
- **Printed, no verdict.** Cells by state; the share of coded cells whose winner holds
  under 0.8; cells with under 5 points in any state.

## What would make this vacuous

- R1 and R2 catch a flipped, shifted or mislabelled grid. They say nothing finer.
- The outline is generalised for a 1 to 50 million map: a border can sit a few km off.
  At 14 km cells that moves few cells, and the share grid marks the ones in doubt.
- A state is a key. Nothing about its dwellings follows from this grid.

## Branches

Exactly one fires, in this order.

- **(e)** A plant fails; or the terms page no longer reads public domain; or the columns
  fixed before the parse are absent; or the rasteriser stops. Nothing is written.
- **(r)** Otherwise, R1 or R2 reads misread, or R3 fails. Nothing is written.
- **(a)** Otherwise. The three grids and the sidecar are written, with R1 and R2 as read.

---
witness: none (a reduction of a published map; R1 to R3 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the rasteriser that writes the grid.
- M-BAR-FROM-THE-READ: the cities, areas and bars are stated here before the file is fetched; the areas are marked as from memory.
- M-PROVENANCE-OVERREACH: digest, version and terms recorded; the grid is called a key.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
