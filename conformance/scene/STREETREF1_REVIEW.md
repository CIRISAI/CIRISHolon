# STREETREF-1 — adversarial review of the draft, 2026-11-11: REDESIGN before any freeze

*A second reader attacked `STREETREF1_PREREG_DRAFT.md` on the day the twin's class rules
arrived. Nothing was fetched and no map service was called; its computations were synthetic
(a grid, rotated grids, spokes, finite grid graphs). Its cell counts are first-principles
estimates from city sizes, not data, and its licence remarks are from the draft's own quotes.
This file is the author's condensation. The draft is kept as written and is not a candidate
for a freeze.*

**Verdict.** The licence reading and the idea are sound. The sampling unit, the stakes and
the observables' conventions are not yet a design.

## 1. A reference keyed only by the cell's class cannot referee street form

A 1/8 degree cell averaging 300 people per km² is mostly fields with a town in it. A tile
drawn uniformly is a mixture: usually near-empty, sometimes town streets. The twin's
generator puts the same suburban streets on every tile. Every per-tile comparison then fails
on every observable, and the failure is about WHERE people sit in the cell, which is already
known and needs no street data. The draft's S4 does not carry it: "no street at all" in a
tile is rare even in farmland, so that share can be small while most tiles are still fields.

**The fix, and a cheaper campaign that comes first.** Class every drawn tile by its OWN
density from a fine population grid (the "class judged on a leaf" the twin's rule already
names) and key the reference by (region, tile class), with the parent's class as a second
key. And before any map data: the unevenness itself is measurable exhaustively from the
population grid alone. That is its own campaign (UNEVEN-0).

## 2. Two stakes cannot fail

- **S1 is circular.** Its budget is the level split halves pass 95 % of the time. Blocks
  drawn by one rule from one class are exchangeable whether or not the class is one
  population, so a held-out set passes about 95 % of the time by construction, and "the
  classes are not populations" follows from nothing it reads. It also borrowed its budget's
  definition from SCENE-1, a draft on hold.
- **S3 passes on the literature** and its kill needs no pair of 36 to exceed a noise
  quantile, which some pair does by chance.
- **Rewritten S1:** the spread between blocks WITHIN a class is smaller than the spread of
  an equal-sized sample pooled across the region's classes, in at least 2 of every 3
  resolved classes. That can fail.

## 3. The top two density classes are nearly empty, and the observables have no conventions

- **Occupancy** (an estimate): dense fabric needs about 2 million people in one cell
  averaged over its whole area; of order 30 to 60 such cells exist in the world and no region
  reaches 40. Apartment blocks: perhaps 150 to 300 cells, with 40 or more only in South and
  East Asia. So about 27 to 31 of the 45 classes would resolve, and the classes where the
  twin's generator is most distinctive are almost unreferenced. The cells that do exist are
  neighbours in a few cities and are not independent blocks.
- **Sampling.** Drawing in proportion to people describes the tile a random PERSON lives
  near; drawing uniformly over cells describes the tile under a random CELL. They certify
  different consumers. In the village class, which spans 1 to 250 per km², a
  people-proportional draw is almost all near-suburbs. Stratify by log-density, record every
  inclusion probability, publish both weightings, cap the cells per 1.5 degree cell and make
  that larger cell the block.
- **Three plants fail as written.** Computed: a 100 m grid reads `20.00` km per km² only if
  its lines sit off the tile's edge (`21.00` otherwise, outside the 2 % bar); its four-way
  share is `0.82` unless cut ends are dropped; its orientation entropy is `ln 2` or `ln 4`
  depending on whether both directions are counted, and jumps a whole unit if a bin edge
  sits on the grid's bearing; the rotation plant at 17 degrees is blind to that; 36 spokes
  read the maximum only with both directions counted.
- **Betweenness is dominated by the window.** A perfect grid's value is `0` on an infinite
  plane and `0.20`, `0.27`, `0.31` on patches of 6, 11 and 21 lines. The artefact depends on
  how much of the tile the network fills, which is what varies with class. Report it; do not
  stake it until a correction is frozen. The citation it rests on was not verified.
- **A tile of 0.02 degrees is not a square:** 2.2 km by 1.1 km at 60 degrees north. Use a
  metric tile, and compute bearings in a local projection, with a plant at high latitude.
- **Conventions owed in the text:** what an intersection is (merge within 12 m), how a road
  cut by the tile edge is treated (read degree on a buffered graph), which tags, divided
  carriageways, loops in sinuosity, a minimum street length before entropy is reported.

## 4. Licence, fetch and buildings

- "No coordinate is kept" is cosmetic: the committed draw gives every tile's box. Say that
  locations are public and geometry is withheld.
- "Cannot recreate any part" is overstated: a 201-point quantile function for a tile with
  fewer than 201 features IS the sorted list of every feature. Use a coarser grid and
  suppress below a feature count.
- A live query is not reproducible, since the map changes daily. Dated regional extracts are
  reproducible and put no load on the query service; they need about 80 GB streamed one
  continent at a time.
- Take buildings out of this campaign. They can only return `unresolved` or `refused`
  without a second source, and they dominate the download.

## 5. The smallest honest version

1. **UNEVEN-0 first:** from the population grid alone, for every cell, the share of its
   tiles and of its people whose own class equals the cell's.
2. Then streets only, from dated extracts, in three regions chosen before any read, three
   tile classes by the tile's own density, 40 blocks each: about 1,100 tiles, all resolved.
   Stakes: the rewritten S1, and density class separating two named adjacent pairs.

## 6. Also flagged

The reference is not blind to the generator (the draft's own table says what the twin's
streets look like); class labels are from 2020 and a map from 2026; two misfit ids were used
outside the registry's sense; the data plan's request counts are stale (36 classes).
