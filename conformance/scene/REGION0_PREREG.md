# REGION-0 — a US state for each cell of the twin's 1/8° grid, reduced from Natural Earth's states and provinces. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`REGION0_REVIEW.md`)
whose named fixes are applied here. It replaces `REGION0_PREREG_DRAFT.md`, kept. The
lead's standing word, 2026-11-12: "You have approval for data sources, lets make holon
rich and gorgeous". This source was not put to them by name before; it is public domain
as read below, and it is reported to them with the result. At this commit the file has
not been downloaded.*

## What it is for

BUILDGEN-1 was killed because a dwelling's joint is regional below climate: the same
climate zone is one fuel mix in one state and another in the next. A generator keyed by
region needs the caller to know its region. The twin holds countries alone. This campaign
makes the key: a US state code per cell, in the same shape as the climate grid. It
certifies nothing about any state.

## Carrier

- **File:** `ne_50m_admin_1_states_provinces.zip`. The server at
  `https://naciscdn.org/naturalearth/50m/cultural/` reports 911,408 bytes, last modified
  13 May 2022; Natural Earth's download page lists it as version 5.1.1. The version is
  taken from the archive's own version file, and the digest is recorded. The default
  file is used on purpose and not the "without large lakes" one, so that lake water
  carries a state's code and the consumer's mask decides.
- **Terms, read on Natural Earth's page:** "All versions of Natural Earth raster + vector
  map data found on this website are in the public domain." and "No permission is needed
  to use Natural Earth." The page is read again at the fetch; if it cannot be read or no
  longer says so, nothing is written.
- Not committed; deleted after the run.

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
coordinate. It fixes the columns for country, state name and postal code. A state's code
is its 2-digit federal code as a number (Alabama 1 to Wyoming 56, the District of
Columbia 11), from the postal code by a table in the script; if the attribute table
carries a federal-code column, the script's table must agree with it for each US shape,
or branch (e).

## What is built

`tables/region0/`, each grid 4,147,200 bytes unpacked, one byte per cell in the twin's
order, key `cell`, gzipped with no time stamp:

- `us_state_cell.bin.gz`: the state's code; 0 where no sample point of the cell lies in a
  US state. Each cell is sampled at 25 points: sub-row j and sub-column i (0 to 4) of
  cell (r, c) sit at latitude 90 − (5r + j + 0.5) × 0.025 and longitude −180 + (5c + i +
  0.5) × 0.025, odd multiples of 0.0125°. The state with the most points wins; a tie
  goes to the lower code.
- `us_state_cell_share.bin.gz`: the winner's share of the cell's points that lie in any
  state, round-half-up of 255 × k / n.
- `us_state_cell_classed.bin.gz`: how many of the 25 points lie in any state.
- **Inside** is even-odd over all rings of one record, ring orientation ignored; an edge
  from (x1, y1) to (x2, y2) is crossed at a sample latitude y when (y1 ≤ y) differs from
  (y2 ≤ y), so a point level with a vertex is counted once. A point inside more than one
  state's shape goes to the lower code, and the count of such points is printed. Only
  shapes whose country column is the United States are rasterised.
- A sidecar: the code table (`code`, `postal`, `name`); cells by state; attribution; "a
  cell's state is the majority of 25 sample points; the share of the whole cell is the
  winner's points over 25, recoverable from the two other grids; a code with a low count
  is mostly sea or another country, so the consumer's own country and land grids decide,
  or it may require a count of 13 or more"; that the District of Columbia may hold no
  cell; that small islands dropped at this map scale leave US land with code 0;
  `holds_no_real_feature: true`, on the twin's lead's words of 2026-11-12 ("A state is a
  coarse statistical unit, not a feature, so I accept it").

## Plants (synthetic shapes through the same rasteriser)

- **P-SQUARE.** A square with corners on stated cell edges fills exactly the cells it covers.
- **P-HOLE.** A square with a square hole leaves the hole empty.
- **P-BORDER.** Two squares meeting within a cell: the larger part wins with its share; a
  10-to-10 split goes to the lower code.
- **P-OVERLAP.** Two overlapping squares: the overlap goes to the lower code and is counted.
- **P-ORIENT.** A shape in the north-west lands in low rows and low columns.
- **P-MULTI.** A state of two separate parts gets both.
- **P-ANTI.** A state of two parts, longitude 179.5 to 180 and −180 to −179.5, fills
  columns 2876 to 2879 and 0 to 3 in its rows and no cell between; a single ring
  spanning over 180° of longitude stops the run.
- **P-AREA.** All sample points of the globe sum to 4πR² (510,065,600 km² to 5
  significant figures), and a 1° by 1° square on the equator reads 12,364 km² to 4.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic shape holds what its plant names.

## Stakes (reading checks; this is a key)

Each is read from the packed bytes as they would ship.

- **R1 (labels), 50 points,** one in each state, each the centre of a cell. The cell's
  byte equals the code written here. Met: at least 48 of 50. *Misread:* 44 or fewer;
  nothing is written. Between: written, each miss named.

| point | cell centre (lat, lon) | code | cell |
|---|---|---|---|
| Syracuse | 43.0625, -76.1875 | 36 | 1080830 |
| Los Angeles | 34.0625, -118.1875 | 6 | 1287854 |
| Chicago | 41.9375, -87.6875 | 17 | 1106658 |
| Houston | 29.8125, -95.3125 | 48 | 1385957 |
| Phoenix | 33.4375, -112.0625 | 4 | 1302303 |
| Denver | 39.6875, -104.9375 | 8 | 1158360 |
| Atlanta | 33.8125, -84.4375 | 13 | 1293884 |
| Seattle | 47.5625, -122.3125 | 53 | 976781 |
| Orlando | 28.5625, -81.4375 | 12 | 1414868 |
| Minneapolis | 44.9375, -93.3125 | 27 | 1037493 |
| Fairbanks | 64.8125, -147.6875 | 2 | 579138 |
| Hawaii Island | 19.5625, -155.4375 | 15 | 1621636 |
| Boise | 43.5625, -116.1875 | 16 | 1068990 |
| Nashville | 36.1875, -86.8125 | 47 | 1239145 |
| Montgomery | 32.3125, -86.3125 | 1 | 1328429 |
| Little Rock | 34.6875, -92.3125 | 5 | 1273661 |
| Hartford | 41.8125, -72.6875 | 9 | 1109658 |
| Dover | 39.1875, -75.5625 | 10 | 1170115 |
| Indianapolis | 39.8125, -86.1875 | 18 | 1155630 |
| Des Moines | 41.5625, -93.5625 | 19 | 1115251 |
| Wichita | 37.6875, -97.3125 | 20 | 1204501 |
| Lexington | 38.0625, -84.4375 | 21 | 1195964 |
| Alexandria LA | 31.3125, -92.4375 | 22 | 1351420 |
| Augusta ME | 44.3125, -69.8125 | 23 | 1052081 |
| Baltimore | 39.3125, -76.5625 | 24 | 1167227 |
| Worcester | 42.3125, -71.8125 | 25 | 1098145 |
| Lansing | 42.6875, -84.5625 | 26 | 1089403 |
| Jackson MS | 32.3125, -90.1875 | 28 | 1328398 |
| Columbia MO | 38.9375, -92.3125 | 29 | 1175741 |
| Billings | 45.8125, -108.5625 | 30 | 1017211 |
| Lincoln | 40.8125, -96.6875 | 31 | 1132506 |
| Elko | 40.8125, -115.8125 | 32 | 1132353 |
| Concord NH | 43.1875, -71.5625 | 33 | 1077987 |
| New Brunswick NJ | 40.4375, -74.4375 | 34 | 1141324 |
| Albuquerque | 35.0625, -106.6875 | 35 | 1264906 |
| Raleigh | 35.8125, -78.6875 | 37 | 1247850 |
| Bismarck | 46.8125, -100.8125 | 38 | 994233 |
| Columbus OH | 39.9375, -82.9375 | 39 | 1152776 |
| Oklahoma City | 35.4375, -97.5625 | 40 | 1256339 |
| Bend | 44.0625, -121.3125 | 41 | 1057429 |
| State College | 40.8125, -77.8125 | 42 | 1132657 |
| West Warwick RI | 41.6875, -71.5625 | 44 | 1112547 |
| Columbia SC | 34.0625, -81.0625 | 45 | 1288151 |
| Pierre | 44.4375, -100.3125 | 46 | 1048957 |
| Salt Lake City | 40.8125, -111.9375 | 49 | 1132384 |
| Montpelier | 44.3125, -72.5625 | 50 | 1052059 |
| Richmond VA | 37.5625, -77.4375 | 51 | 1207540 |
| Charleston WV | 38.3125, -81.6875 | 54 | 1190226 |
| Madison | 43.0625, -89.4375 | 55 | 1080724 |
| Casper | 42.8125, -106.3125 | 56 | 1086349 |

- **R2 (areas), 10 states.** A sample point in fine row q (0 to 7199) carries area
  R² × Δλ × (sin φn − sin φs), with φn = 90° − q × 0.025°, φs = φn − 0.025°, Δλ = 0.025°
  in radians, R = 6371.0088 km. A state's sampled area against its land area in km² from
  memory: Alaska 1,477,953; Texas 676,587; California 403,466; Montana 376,962; New
  Mexico 314,161; Arizona 294,207; Nevada 284,332; Colorado 268,431; Oregon 248,608;
  Wyoming 251,470. Within −3 % to +6 % (the shapes include inland water); Alaska −8 % to
  +10 % (its coast is generalised). Met: at least 9 of 10. *Misread:* 5 or fewer;
  nothing is written. Between: written, each miss named.
- **R3 (all present).** Each of the 50 states has at least 1 cell. Otherwise nothing is
  written. The District of Columbia is printed, no verdict.
- **R4 (containment).** Each coded cell lies in latitude 18.5 to 71.75 and in longitude
  −180 to −66.75 or 172 to 180. Otherwise misread.
- **Printed, no verdict.** Cells by state; the share of coded cells whose winner holds
  under 0.8; coded cells with a count under 13; Alaska's cells east of 172°E; points
  that lie in two shapes.

## What would make this vacuous

- R1 catches a flipped, half-globe-shifted or mislabelled grid. R2 catches a wrong cell
  size, a swap of latitude and longitude, a wrong area formula and a dropped or doubled
  part. Neither sees a shift of one cell; P-SQUARE and P-ORIENT carry that.
- The outline is generalised for a 1 to 50 million map: a border can sit a few km off,
  and small islands and keys are dropped, leaving US land with code 0.
- Latitudes are treated as on a sphere, which under-reads Alaska by about 0.6 %.
- A state is a key. Nothing about its dwellings follows from this grid.

## Branches

Exactly one fires, in this order.

- **(e)** A plant fails; or the terms page cannot be read or no longer reads public
  domain; or the columns fixed before the parse are absent; or a US shape's postal code
  is not a state's, or fewer than 50 states are among the shapes, or a federal-code
  column disagrees with the script's table; or a vertex is out of range or a ring spans
  over 180° of longitude; or the rasteriser stops. Nothing is written.
- **(r)** Otherwise, R1 or R2 reads misread, or R3 or R4 fails. Nothing is written.
- **(a)** Otherwise. The three grids and the sidecar are written, with R1 and R2 as read.

---
witness: none (a reduction of a published map; R1 to R3 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the rasteriser that writes the grid.
- M-BAR-FROM-THE-READ: the cities, areas and bars are stated here before the file is fetched; the areas are marked as from memory.
- M-PROVENANCE-OVERREACH: digest, version and terms recorded; the grid is called a key.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
