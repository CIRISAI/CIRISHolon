# UNEVEN-0 — READ: branch (a). In a typical "suburban" 14 km cell about a quarter of the tiles and of the people are at suburban density, and two thirds of the people live denser. The value is the table.

*2026-11-11. Prereg `UNEVEN0_PREREG.md` (frozen alone, `b796a92`, after review); instrument
`uneven0/uneven0.py` (committed before the grid was read, `6613ef5`); record `uneven0/uneven0.log`; outputs `uneven0/transition_tables.json`,
`uneven0/cells.npz`, and for consumers `tables/uneven0/`. Input: GHS-POP R2023A, epoch 2020,
30 arc-seconds, both digests matching the twin's record; 923,831,568 pixels read, none
no-data, none negative; the raw file deleted after the tables were written.*

*As the freeze said: this is a reading of the reference's own structure within a cell, not
a certificate, and its two stakes were expected to be met. They were, by a wide margin.*

## 0. Verdict

| gate | staked | read | verdict |
|---|---|---|---|
| **G0 (1)** world total | `7,840,952,947.01` to `0.01` | off by `0.004` | **MET** |
| **G0 (2)** half-degree totals against the twin's | all within `10⁻⁶` | `0` of 259,200 differ | **MET** |
| **G0 (3)** 1/8 degree codes against the twin's | at most `100` differ | `0` of 4,147,200 differ | **MET** |
| **S1** suburban cells: median share of tiles in the cell's own class | below `0.5`, both weights; kill above `0.8` | `0.240` each cell once, `0.271` by people | **MET** |
| **S2** suburban cells, by people: share of people in own-class tiles; in higher | below `0.5`; above `0.4` | `0.282`; `0.664` | **MET** |

**Plants.** The four synthetic plants fired with the frozen values. P-ASSIGN and P-EDGE fired
on the file's header: pixel edges sit `0.55` pixel past cell edges, the first and last cell
columns hold 16 pixel columns, cell rows 7 and 1,432 hold 12 rows. P-SHIFT fired (a one-column
shift breaks 38,008 half-degree totals). P-FLIP fired (reversing the rows leaves the world
total unmoved and breaks 90,112).

**The two repositories read the grid identically,** to the last code. The file's two extra
columns do not double-count: pixel column 0 holds `18.81` people and the last column `0.00`.

## 1. The transition table, at one pixel (about 0.9 km), each cell counted once

Share of a cell's TILES in each tile class:

| cell class | cells | empty | village | suburban | terraced | apartment | dense |
|---|---|---|---|---|---|---|---|
| village | 315,283 | 0.663 | 0.304 | 0.029 | 0.003 | 0.000 | 0.000 |
| suburban | 35,868 | 0.226 | 0.410 | 0.276 | 0.075 | 0.010 | 0.003 |
| terraced | 3,911 | 0.120 | 0.237 | 0.271 | 0.250 | 0.079 | 0.043 |
| apartment blocks | 362 | 0.068 | 0.118 | 0.140 | 0.215 | 0.209 | 0.249 |
| dense fabric | 121 | 0.039 | 0.060 | 0.084 | 0.149 | 0.146 | 0.522 |

Share of a cell's PEOPLE in each tile class:

| cell class | empty | village | suburban | terraced | apartment | dense |
|---|---|---|---|---|---|---|
| village | 0.005 | 0.565 | 0.314 | 0.100 | 0.013 | 0.003 |
| suburban | 0.000 | 0.074 | 0.351 | 0.354 | 0.129 | 0.091 |
| terraced | 0.000 | 0.009 | 0.094 | 0.340 | 0.251 | 0.306 |
| apartment blocks | 0.000 | 0.001 | 0.014 | 0.099 | 0.236 | 0.651 |
| dense fabric | 0.000 | 0.000 | 0.004 | 0.033 | 0.086 | 0.877 |

The same tables at 1.9 and 3.7 km, and weighted by people, are in the log and the JSON.

## 2. What it says

- **The even spread puts most people at the wrong density.** In no class below dense fabric
  do more than about a third of a cell's people live in tiles of the cell's own class. In a
  suburban cell 57 % of people live in terraced tiles or denser.
- **Most of a low-density cell is empty.** Two thirds of the tiles of a village cell hold
  under 1 person per km², and a fifth of a suburban cell's.
- **The world's people are not where the world's cells are.** 89 % of populated cells are
  village cells and they hold a quarter of the people; suburban cells are a tenth of the
  cells and hold 42 % of the people.
- **The top classes are small and real:** 362 apartment-block cells and 121 dense-fabric
  cells in the world. The review's estimate from city sizes was 150 to 300 and 30 to 60;
  the grid holds about twice the upper end of the second.
- **Coarsening the tile hides it only slowly.** At 3.7 km a suburban cell's own-class tile
  share rises from 0.28 to 0.46 and its people in denser tiles fall from 0.57 to 0.34.
- **Unevenness falls with density** (median Gini 0.92, 0.77, 0.70, 0.58, 0.50). As the
  freeze said, part of that is arithmetic: a cell's mean bounds its Gini.

## 3. What limits it, and deviations

1. Every figure is a LOWER bound on unevenness: nothing below about 1 km is seen, and the
   file is a resampling of the publisher's equal-area grid.
2. The fine grid is a model: census counts spread over built-up area.
3. Coastal cells read as uneven because sea holds nobody and the class uses whole area.
4. **A precision fault in the summaries.** The "each cell once" means were accumulated in
   single precision, so five of the sixty published share vectors miss a sum of one by 8 to
   259 parts per million. No stake reads them. The instrument is corrected; the raw file had
   already been deleted, so the published table carries the fault, states a tolerance of 300
   parts, and tells a consumer to renormalise.
5. Per-cell rows are stored at half precision (three significant digits) for size.
6. Not split by region: the twin's region table is not yet published as data. The per-cell
   rows allow that split later with no new read of the grid.

## 4. What it does not say

Nothing about streets, buildings or networks; nothing about whether a tile's own class is a
sufficient key for anything; no certificate.
