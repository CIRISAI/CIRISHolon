# HOUSEHOLD-0 (Germany) — household size and buildings by number of dwellings per 1 km cell, by the cell's own density; the twin's rule scored, and a test of whether the US table's shape carries across. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. The lead approved
a European country by its data ("whatever european country has the most data available");
the scout named Germany (`EUROPE_SCOUT.md`). Nothing has been fetched or computed. Data
facts are from that report (documentation only) and are re-checked against the files' own
headers in the build. The US campaign of the same name is read (`HOUSEHOLD0_RESULTS.md`);
its numbers appear below as inputs and are not re-fitted.*

## Why

The US campaign scored the twin's household rule on census units that are not tiles, found
it certified on the mean and refused on composition, and found density class a weak key
there. Germany publishes household size on a regular 1 km grid, so the reference's unit IS
the tile, and it publishes counts of buildings by number of dwellings, which no US table
has. Two things are unknown: whether density class is a better key where the unit is a
tile, and whether anything of the US table carries to a second country.

## Carrier

- **Files.** Zensus 2022, grid results, reference date 15 May 2022, the 1 km files only:
  household size (1, 2, 3, 4, 5, 6-or-more persons, and a total), mean household size,
  population, buildings with dwellings by number of dwellings (1, 2, 3–6, 7–12,
  13-or-more), dwellings by building type and size, buildings by building type and size.
  Six ZIPs, about 130 MB; each holds three resolutions and only the 1 km csv is opened.
- **Terms.** Datenlizenz Deutschland – Namensnennung – Version 2.0. The source note carried
  by every table: "Statistische Ämter des Bundes und der Länder, Zensus 2022, Ergebnisse in
  Gitterzellen; dl-de/by-2-0, www.govdata.de/dl-de/by-2-0; data changed: aggregated by
  density class", with the dataset address. Raw files are not committed; digests are.
- **The unit and its class.** A 1 km cell, classed by the population file's count for that
  cell (people per km², the cell being 1 km²) at the twin's bounds `0 / 250 / 1,500 / 6,000
  / 12,000`. This is the tile class of UNEVEN-0, read on a national grid and not on the
  population model the twin uses.
- **Disclosure control, stated before the read.** Counts are perturbed by a cell-key method:
  zeros stay zero, original 1s and 2s never appear, noise is mostly within ±2 with mean 0,
  the parameters are unpublished, and a total is perturbed separately from its parts.
  - Shares are computed from the six size columns; a cell's weight is their sum. The
    published total column is used only to measure the noise.
  - Per class, printed: the pooled sum of parts over the pooled totals, and the mean
    absolute difference per cell. A class whose pooled ratio is off by more than 1 % is
    marked `disclosure-limited` and carries no verdict.
  - Cells whose six columns sum to under `10` households are excluded and counted, with
    their share of each class's households by the total column.
  - A "–" is read as zero. The 100 m file is never summed.
- **Persons per household**, three ways, all printed: (1) the mean-household-size file,
  weighted by each cell's households (the value used in verdicts; cells where it is
  bracketed as uncertain are included and counted); (2) population over households (too
  high by the people in communal accommodation); (3) the six bands with the top at `7`.
- The **blocks.** A 50 km square of the grid (from the cell's coordinates). Hold-out is by
  100 km square: the 100 km squares, sorted by (northing, easting), are permuted by
  `numpy.random.default_rng(20261113).permutation`; the first half (rounded up) is FIT,
  the rest SCORE.
- A class being **resolved**: effective blocks `(Σw)² / Σw²` at least `30` in each set a
  comparison uses.
- What is **read per class**: shares of households at `1 … 6+`; persons per household;
  shares of buildings at `1, 2, 3–6, 7–12, 13+` dwellings; dwellings per building overall
  and within the top band (dwellings in buildings of 13 or more, from the type-and-size
  file's largest category, over buildings of 13 or more); households per building.

## Generators

- **TWIN-SIZE-0**: the twin's file as scored in the US (`household0/twin_household_hist.json`,
  digest `438568ab…`): mean `2.500`, one-person share `0.1824`.
- **US-SHAPE-1**: the US tract-key table moved to Germany by shape only. For class `c`:
  German FIT national value plus (US class `c` value − US national value), for each of the
  two quantities. US values as published: persons per household `2.537, 2.507, 2.586,
  2.520, 2.321` against `2.532`; one-person share `0.255, 0.285, 0.300, 0.344, 0.382`
  against `0.286`.
- **FIT-CLASS-1** and **FIT-NATION-1**: as in the US campaign, fitted on FIT.

## The match

- **Budgets**, unchanged from the US: `0.12` on persons per household; `0.03` on the
  one-person share. Both the author's.
- The **reference's own error**: the 95th percentile of the gap between random halves of the
  blocks (1,000 splits, seed 20261114).
- **Verdict order, with the floor split** (the US results named the fault this repairs).
  Envelope; then resolved; then: `certified` if the miss is within budget AND the gap is at
  most half the budget; `refused` if the miss exceeds the budget by more than the gap;
  otherwise `unresolved`.

## Stakes

- **G0.** Column names as the scout lists them; nationally the six size columns sum to
  within 1 % of the total column; the 1 km household total is within 0.5 % of the 10 km
  file's (the one line of that file that is read); national persons per household by
  method (1) lies between `1.8` and `2.3`. If not, nothing runs.
- **S1 (the twin's rule).** Expected: refused on BOTH consumers nationally (German
  households are smaller than `2.5` and far more often one person). *Kill of the
  expectation:* households per leaf certified nationally.
- **S2 (is density a useful key, on tiles).** Units: each SCORE block's households in one
  class, at least `5,000` (a full count, so no sampling floor is needed; the result at
  `20,000` is printed for comparison with the US). The household-weighted share of units
  within both budgets under FIT-CLASS-1 exceeds that under FIT-NATION-1 by at least
  `0.05`. *Kill:* not higher.
- **S3 (classes differ).** As in the US: largest difference between resolved classes at
  least `0.12` or `0.03`. *Kill:* both under half.
- **S5 (the US shape carries).** US-SHAPE-1 is within both budgets of the German reading in
  each resolved class (all blocks). *Kill:* outside a budget in 2 or more resolved classes.
  Then not even the class-to-class differences travel between countries, and each country
  needs its own table.
- **S6 (buildings, a reading).** Shares of buildings by number of dwellings, dwellings per
  building and households per building, per class. No bar: it is the first reference for
  the registry row "households per building", and no generator is offered to score.

## What would make a stake vacuous

- S1's expectation rests on remembered national figures (about 2.0 persons, about two
  fifths one-person). It is close to a confirmation.
- S5 can pass because German classes barely differ and the US increments are small against
  the budgets; the class differences of both countries are printed beside the verdict.
- S2's units are 50 km squares, far larger than a US county in the east and smaller than
  one in the west. The two countries' shares are not on one scale.
- Households here are partly constructed: register persons not matched to a dwelling are
  distributed "by statistical criteria", and there is one household per occupied dwelling
  by definition. A verdict is against that construction.
- The village class is where small counts are, so it is where disclosure control bites.

## Plants

Synthetic carriers through the same code, before any file is opened.

- **P-EQUAL, P-SHIFT, P-MEAN, P-THIN, P-NEFF**: as in the US campaign.
- **P-FLOOR-SPLIT** (a miss of `0.117` with a gap of `0.023`; and a miss of `0.02` with the
  same gap; sector: the split floor). The first is `refused`, the second `unresolved`.
- **P-CKM** (5,000 cells of 300 households with known shares; each count perturbed by a
  noise in ±2 with mean 0, 1s and 2s moved to 0 or 3, the total perturbed separately;
  sector: the disclosure rule). Pooled one-person share within `0.003` of the truth; the
  pooled ratio of parts to totals within 1 %.
- **P-CKM-SMALL** (the same with cells of 12 households; sector: the same). The class is
  marked `disclosure-limited` or its pooled share is off by more than `0.003`; either way
  the run prints which.
- **P-DASH** (a cell with "–" in three columns). Read as zeros, no error.
- **P-COMMA** (a mean household size written `2,13`). Read as `2.13`.
- **P-MISLABEL** (a file whose total column is population, not households). G0 stops it.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** S1's expectation stands. Verdicts are issued per consumer and class; S2, S3, S5
  each reported as met, killed or neither.
- **(b)** S1's expectation is killed. Stated.

## What is published

Under `conformance/scene/tables/household0_de/`: per class, household-size shares in parts
per million, persons per household three ways, building-size shares, dwellings and
households per building, the noise measures and exclusions, the verdicts; and a sidecar.

## What this does not claim

Not devices, storeys or time use; not household size beyond six; not the joint of household
size with building size; not any other country.

---
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (household size and building size are two margins; no joint is published)
witness: none (S1–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader and runs before a file is read.
- M-BAR-FROM-THE-READ: budgets and bars are the US campaign's, fixed before it was read; the split floor is stated here before any German file exists on this machine.
- M-BASE-RATE-OMITTED: national values are printed beside each class's.
- M-POPULATION-CHOICE: each populated 1 km cell is read; exclusions are rules stated above and counted.
- M-PROVENANCE-OVERREACH: each file's digest is recorded; that the files are the release named rests on their own headers.
- M-FLOOR-UNSTAKED: P-THIN, P-NEFF and P-FLOOR-SPLIT.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: S5 is the test of whether a US reading says anything about Germany; nothing here is credited to the US table unless S5 is met.
