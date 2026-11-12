# HOUSEHOLD-0 (Germany) — household size per 1 km cell by the cell's own density, with the 10 km file as referee for disclosure bias; the twin's rule scored, and whether its family or only its numbers are wrong. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`HOUSEHOLD0_DE_REVIEW.md`)
whose named fixes are applied here, on the lead's approval of a European country by its
data and the scout's choice of Germany (`EUROPE_SCOUT.md`). It replaces
`HOUSEHOLD0_DE_PREREG_DRAFT.md`, kept. At this commit nothing has been fetched, no German
instrument exists, and nothing has been computed on the census by anyone. The US campaign
(`HOUSEHOLD0_RESULTS.md`) is read; its numbers appear only in the reading S5.*

*Forecasts on the record. The twin's rule is refused on both consumers nationally (author
and reviewer, from remembered figures). For S1b, S2 and S3 the author has no forecast.*

## Carrier

- **Files.** Zensus 2022 grid results, reference date 15 May 2022. Household size (1, 2, 3,
  4, 5, 6-or-more persons, and a total) at 1 km and at 10 km; mean household size at 1 km;
  population at 1 km; dwellings by building type and size and buildings by building type
  and size at 1 km and 10 km. Five ZIPs, about 110 MB; the 100 m files are never opened.
- **Terms.** Datenlizenz Deutschland – Namensnennung – Version 2.0. The source note on each
  table: "Statistische Ämter des Bundes und der Länder, Zensus 2022, Ergebnisse in
  Gitterzellen; dl-de/by-2-0, www.govdata.de/dl-de/by-2-0; data changed: aggregated by
  density class", with the dataset address. Raw files are not committed; digests are.
- **The unit and its class.** A 1 km cell, classed by the population file's count for it at
  the twin's bounds `0 / 250 / 1,500 / 6,000 / 12,000` per km². That count includes
  communal residents and is itself perturbed; cells on the border hold only their German
  part. Cells with households and no population row are counted and left out.
- **Disclosure control.** Counts are perturbed by a cell-key method with unpublished
  parameters: zeros stay, original 1s and 2s never appear, a total is perturbed apart from
  its parts. Whether a small count keeps its expectation is not published.
  - No cell is excluded by size. Shares are computed from the six size columns.
  - The **column check.** Each 10 km cell is given the class that holds most households
    among its 1 km cells. Per such group and per column, the sum over its 1 km cells is
    divided by the sum of the 10 km file's column over its 10 km cells. A column whose
    ratio is off by more than 3 % is not published for that class; its households are
    pooled with the band below ("5 or more", then "4 or more"). A class with fewer than 20
    such 10 km cells is marked `unchecked`.
  - The ratio of parts to total is printed and decides nothing. A "–" is read as zero.
- **Persons per household.** Primary: the mean-size file times the published total column,
  summed, over the summed totals (the count the publisher divided by). Bracketed means stay
  in and are counted. Cells with no mean are left out of this method; a class where they
  hold over 1 % of households is marked limited. Also printed: population over households,
  and the residual mean of the 6+ band, which should lie between 6 and 8.
- The **blocks.** A 50 km square of the grid. Hold-out: the bounding box of populated cells
  is cut into six bands (three by easting, two by northing, equal widths); the six, in
  order (south-west first, easting within northing), are permuted by
  `numpy.random.default_rng(20261113).permutation`; the first three are FIT, the rest
  SCORE. The same analysis with interleaved 100 km squares is printed beside it.
- A class being **resolved**: effective blocks `(Σw)² / Σw²` at least `30` in each set a
  comparison uses.

## Generators

- **TWIN-SIZE-0**: the twin's own file as scored in the US (digest `438568ab…`): mean
  `2.500`, one-person share `0.1824`.
- **TWIN-FAMILY-1** (a port, labelled so): the twin's rule with two numbers freed. Leaves
  of 60 households holding `round(60 p)` people; sizes drawn `clamp(round(p + s z), 1, 6)`;
  the leaf's people apportioned by largest remainder. `p` is set to FIT's national persons
  per household. `s` is the value on the grid `0.30, 0.35, … 4.00` whose one-person share
  is nearest FIT's national share, at 200,000 households, seed 7. The port matched the
  twin's file to `0.0014` in each share at the twin's own numbers.
- **FIT-CLASS-1** and **FIT-NATION-1**: FIT's pooled values per class and overall.

## The match

- **Budgets, and the regime they are priced in.** Households per leaf: 5 % of the
  REFERENCE's persons per household (the US campaign's `0.12` was 5 % of the twin's 2.5; a
  relative budget is what the consumer's reasoning gives, and it is about `0.10` here).
  Composition: `0.03` in the one-person share, absolute. Both the author's.
- The **gap.** The 95th percentile of the difference between random halves of the blocks
  (1,000 splits, seed 20261114). In a full count it measures how different places are.
- **Verdict.** Envelope, then resolved, then: `certified` if miss + gap ≤ budget; `refused`
  if miss − gap > budget; otherwise `unresolved`, reason "places differ by more than the
  budget resolves".

## Stakes

- **G0.** Column names as the scout lists them. Nationally the six size columns sum to
  within 1 % of the total column, at 1 km and at 10 km; the 1 km and 10 km household totals
  agree within 0.5 %; persons per household lies between `1.8` and `2.3`; the residual mean
  of the 6+ band lies between `5.5` and `9`. If not, nothing runs.
- **S1 (the twin's rule, a reading).** Verdicts per consumer, nationally and per resolved
  class. Expected: refused on both nationally. *Kill of the expectation:* either certified
  nationally.
- **S1b (the family or its numbers).** TWIN-FAMILY-1 against SCORE, per resolved class: the
  largest absolute error in the shares at 2, 3, 4 and 5-or-more. Met if at most `0.03` in
  at least half the resolved classes. *Kill:* over `0.06` in each resolved class. Then the
  family, not its two numbers, is what the twin must change. Printed with it: the fit's
  own miss in the one-person share, and the share of zero-person households it produces.
- **S2 (is density a useful key, on tiles).** Units: each SCORE block's households in one
  class, at least `5,000` (also printed at `1,000` and `20,000`). The household-weighted
  share of units within both budgets under FIT-CLASS-1 exceeds that under FIT-NATION-1 by
  at least `0.05`. *Kill:* not higher.
- **S3 (classes differ).** The largest difference between resolved classes is at least the
  households-per-leaf budget in persons per household, or at least `0.03` in the
  one-person share. *Kill:* both under half.
- **S5 (the US beside it, a reading).** For classes resolved in both countries: the German
  class values, the US ones moved three ways from suburban as the common reference
  (difference, ratio, log-odds), and the sign of each class-to-class difference in each
  country. No stake.
- **S6 (buildings, a reading).** Per class: shares of buildings and of dwellings by the
  type-and-size categories; dwellings per building overall and within the 13-or-more
  category (same two files, same category); households per building. A top-band value
  under `13` is printed as an alarm. The column check applies to the building counts too.

## What would make a stake vacuous

- S1's expectation rests on remembered national figures and is close to a confirmation.
- S1b scores a port, with the mean fitted exactly and the spread on a grid. If the fit
  cannot reach the one-person share, its miss is printed and S1b is read with it.
- S1b's bars are the author's; `0.03` is the composition budget reused for other shares.
- S2's units are 50 km squares and its hold-out three bands of six. One seeded choice of
  bands is one draw; the interleaved result is the only check on it.
- Households here are partly constructed (unmatched register persons are placed "by
  statistical criteria"; one household per occupied dwelling by definition).
- The column check compares two perturbed files. It can show a bias; it cannot show there
  is none below its 3 %.

## Plants

Synthetic carriers through the same code, before any file is opened.

- **P-EQUAL, P-SHIFT, P-MEAN, P-THIN, P-NEFF**: as in the US campaign, under the verdict
  rule above.
- **P-FLOOR-SPLIT** (at the budget of `0.03`: a miss of `0.117` with a gap of `0.023`; a
  miss of `0.020` with the same gap; a miss of `0.004` with a gap of `0.010`). Refused,
  unresolved, certified.
- **P-CKM** (4,000 cells of 30 households in 40 ten-kilometre cells; a 1 goes to 3 with
  probability one third and a 2 with two thirds, otherwise to 0; other counts get a noise
  in ±2 of mean 0; the 10 km counts are perturbed once). Pooled 6+ share within 5 % of the
  truth, and no column flagged.
- **P-CKM-BIAS** (the same with each 1 set to 0). The column check flags the 5 and the 6+
  columns.
- **P-DASH** (a cell with "–" in three columns). Zeros, no error.
- **P-COMMA** (a mean written `2,13`). Read as `2.13`.
- **P-MISLABEL** (a total column holding population). The 1 % gate stops it.
- **P-FAMILY** (a target made by the port at `p = 2.2`, `s = 1.40`). The fit returns `s`
  within `0.05` and each share within `0.01`.
- **P-TOP13** (a class whose 13-or-more category holds 12 dwellings per building). The
  alarm fires.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. Verdicts are issued per consumer and class; S1, S1b, S2 and S3 are
  each reported as met, killed or neither.

## What is published

Under `conformance/scene/tables/household0_de/`: per class, household-size shares in parts
per million (bands as the column check allows), persons per household three ways, the
column-check ratios, building and dwelling shares, dwellings and households per building,
the verdicts; and a sidecar.

## What this does not claim

Not devices, storeys or time use; not household size beyond six; not the joint of household
size with building size; not any other country; not that the twin's population grid gives
a tile the same class this census does.

---
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (household size and building size are two margins; no joint is published)
witness: none (S1–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader and runs before a file is read.
- M-BAR-FROM-THE-READ: budgets, bars, the grid for the spread and the band cut are stated here before any German file exists on this machine. The households-per-leaf budget is a stated fraction of the reference value, not a number chosen after seeing it.
- M-BASE-RATE-OMITTED: national values are printed beside each class's; S2 scores the class rule against the national one.
- M-POPULATION-CHOICE: each populated 1 km cell with a population row is read; no cell is excluded by size.
- M-PROVENANCE-OVERREACH: each file's digest is recorded; that the files are the release named rests on their own headers.
- M-FLOOR-UNSTAKED: P-THIN, P-NEFF and P-FLOOR-SPLIT.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: nothing here is credited to the US table; S5 prints the two countries side by side and stakes nothing. TWIN-FAMILY-1 is a port and is labelled one.
