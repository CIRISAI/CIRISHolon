# HOUSEHOLD-0 (United States) — persons per household by neighbourhood density and building type, and the first scene certificate issued: the twin's household-size rule against it. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. The lead approved
the dataset the same day ("yes, run the campaign for the US and for whatever european country
has the most data available"). Nothing has been fetched or computed. A second campaign of the
same shape for one European country follows once a scout has named the country. The data
facts below are from `ACS_DOCS.md` (documentation only) and are re-checked against the files'
own headers in the build.*

## Why

The CEWP twin's models read three things about a household: how many persons it holds, its
device mix, and its router's behaviour. For the first, the twin draws a size from a normal
with mean `2.5` and spread `1.1`, rounded and clamped to 1 to 6, the same everywhere, and
then apportions the leaf's people over the households with those sizes as weights. No
reference stands behind that rule. This campaign reads the reference that exists for one
country and issues the first verdict of the scene certificate on a rule the twin runs.

## Carrier

- **Source.** American Community Survey, 5-year estimates 2020–2024, census-tract level
  (84,414 tracts in the 50 states and DC), from the table-based summary file, which needs no
  key and no agreement. Tables: B25009 (occupied housing units by tenure by household size,
  1 to 6 and "7-or-more"), B25008 (population in occupied housing units), B25032 and B25033
  (occupied units, and population in them, by units in structure), B26001 (group-quarters
  population), B28002 (internet subscription types), and the tract gazetteer (land area).
  About 0.4 GB in all. Every estimate has its own 90 % margin of error.
- **Terms.** No licence text is attached to the files; the Bureau asks to be cited as the
  source of the original data only, and forbids re-identification. The notice that travels
  with every table produced here is fixed in the build from the Bureau's own citation page
  and quoted there.
- **The unit and its class.** A tract, classed by ITS OWN density: population in households
  (B25008) over land area, at the twin's five lower bounds (`0 / 250 / 1,500 / 6,000 /
  12,000` per km²) with the twin's floor of `1`. No tract is joined to any grid tile. A tract
  is sized by population (1,200 to 8,000 people) and not by area, so "tract density class"
  is NOT the tile class of UNEVEN-0 and is never called that.
- The **group quarters** (dormitories, barracks, prisons, care homes) are outside every household
  table. A tract whose group-quarters population exceeds 20 % of its total is flagged and
  reported separately.
- The **blocks.** A county is one block. Hold-out is by STATE: the states are split into two
  halves by a seeded draw (seed 20261111) before any file is read; FIT is one half, SCORE
  the other. A class is RESOLVED when each half holds at least 30 counties with at least one
  tract of that class.
- **What is read, per density class** (and per class by five structure types where B25033
  allows): the distribution of household size over `1, 2, 3, 4, 5, 6, 7+` as shares of
  households, pooled over tracts; persons per household (household population over
  households); and, reported only, the share of households whose only internet subscription
  is a cellular data plan.
- **The open top band.** "7-or-more" has no stated mean. Its mean is derived per class as
  (household population − Σ k × households of size k, k = 1 to 6) over households of 7 or
  more, and is printed; where that derivation gives a value below 7 the class is flagged.

## The generator under test, and the fitted one

- **TWIN-SIZE-0**, the twin's rule as its code states it (`cewp_sim::popchain`, twin/globe
  `943ab2d`): households `= round(people / 2.5)`; each draws a size
  `clamp(round(2.5 + 1.1 z), 1, 6)` with `z` standard normal; the leaf's people are
  apportioned over households by largest remainder with the drawn sizes as weights. Its
  household-size distribution is computed from 10⁶ households in leaves of 60, with a fixed
  seed, by this campaign's own implementation of that rule. The same for each class.
- **FIT-SIZE-1**, the reference read on the FIT states, per density class: sizes drawn from
  the class's own distribution, the top band at its derived mean. Nothing else is fitted.

## The match and its budget

- **Observable.** Household size, in persons. Consumer: the twin's device counts, which are
  per person, so a household's size is what the mesh reads.
- **Distance.** Exact W1 between two household-size distributions on `1 … 7+` (the top band
  placed at its derived mean), in persons; and the absolute difference in the share of
  one-person households, which is the tail the mesh's always-on nodes depend on.
- The **budget `β₀`, per class, set on FIT only.** The FIT counties of a class are split into
  two halves 1,000 times; the budget of each of the two components is its median over the
  splits times one common multiplier, the smallest at which both components pass together in
  95 % of the splits. Written to `household0/budgets.json` and committed before SCORE is
  read.
- The **sampling error.** Each class's pooled shares carry an error from the published margins
  (root of summed squares over tracts, the Bureau's rule, which it says degrades with many
  terms and which is a lower bound). It is printed beside every distance. A distance smaller
  than twice its own error is `unresolved`, not certified.

## Stakes

- **G0 (the files are the tables).** Each table's header matches the shells in `ACS_DOCS.md`;
  the national total of households from B25009 equals that from B25032 to 1 household; the
  national household population over households lies between `2.3` and `2.8`. If not,
  nothing runs.
- **S1 (the twin's rule, scored).** TWIN-SIZE-0 against SCORE, per resolved class: the
  verdict is `refused` in at least 4 of the resolved classes, with W1 at least `3 β₀`.
  *Kill:* certified in 2 or more classes. Then a single normal draw is good enough for the
  twin and the reference changes nothing.
- **S2 (the fitted rule, scored on states it never saw).** FIT-SIZE-1 against SCORE: the
  verdict is `certified` in at least 4 of the resolved classes. *Kill:* refused in 2 or more.
  Then household size varies between states more than within a density class, and density is
  the wrong key or not enough of one.
- **S3 (density is a key).** Between the lowest and the highest resolved class, persons per
  household differ by at least `0.20`, or the one-person share by at least `0.08`. *Kill:*
  both differences under half those. Then one national distribution serves all classes.
- **S4 (building type adds to density).** Within a density class, persons per household
  differ between detached houses and buildings of five or more units by at least `0.30`, in
  at least 3 resolved classes. No kill: either way it is a reading, and it decides whether
  the twin's building type should condition household size.

## What would make a stake vacuous

- S1 is expected: a symmetric draw around 2.5 cannot have the one-person share real
  households do. Its content is the SIZE of the miss, per class, and the witness.
- S2 can pass for a bad reason: FIT and SCORE are both the same survey, so they share its
  errors. It certifies the rule against this survey, not against households.
- The bars in S3 and S4 are the author's and are not sourced.
- A tract's density is not a tile's. A generator keyed by tile class that borrows these
  tables is using them outside the unit they were read on, and that is stated on the table.

## Plants

- **P-EQUAL** (carrier: a synthetic class whose FIT and SCORE halves are drawn from one
  distribution, 200 counties each; sector: the verdict). Must be `certified`.
- **P-SHIFT** (carrier: P-EQUAL with SCORE's one-person share raised by `0.10`; sector: the
  one-person component). Must be `refused`, and the witness must name that component.
- **P-MEAN** (carrier: two distributions with the same mean, `2.5`, one of them all
  households of 2 and 3, the other half of 1 and half of 4; sector: the shape). The mean
  difference is `0` and the verdict must be `refused`. The certificate's reason for being.
- **P-TOP** (carrier: a class with 5 % of households in the top band at a true mean of 9;
  sector: the open band). The derived mean must read `9.0` to 2 digits.
- **P-THIN** (carrier: a class with 12 counties in SCORE; sector: the floor). Must be
  `unresolved`, with no distance printed as a verdict.
- **P-GQ** (carrier: a synthetic tract that is 90 % dormitory; sector: the flag). It must be
  flagged and kept out of its density class's pool.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold the equal halves, the
shifted share, the equal means, the top band, the thin class or the dormitory its plant
names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** S1, S2, S3 met. The first scene certificate is issued twice: a refusal with its
  witness for the rule the twin runs, and a certification for a rule fitted to the
  reference, per density class, for one country. The table is published with its sidecar.
- **(b)** Otherwise, stated with each clause that failed. If S2 is killed the table is
  published as a reading with every verdict `unresolved` and the reason.

## What is published

Per density class (and by structure type): the household-size shares in parts per million,
the top band's mean, persons per household, the error of each, the verdict and reason for
each generator; the witness of every refusal (both distributions and the component that
broke); and a sidecar in the form `tables/uneven0/sidecar.json` uses. Envelope: the United
States, 2020–2024, households only, tract density. Everywhere and everything else:
`unresolved`.

## What this does not claim

Not device counts (the survey records only whether a household has a device); not households
per building (its building types are banded and are, in the Bureau's words, not for the
number of residential buildings); not storeys; not any other country; not the 1 km tile.

---
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (the survey gives size and structure type as margins and one joint; no device joint exists)
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.ScenePopulation.pop_total` (persons are apportioned over households; the generator under test is a section of the leaf's people)
witness: none (S1–S4 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader and runs before a table is read.
- M-BAR-FROM-THE-READ: the budget is set on FIT and committed before SCORE is read; S3's and S4's bars are the author's priors and are said to be.
- M-BASE-RATE-OMITTED: the national distribution is printed beside each class's.
- M-POPULATION-CHOICE: every tract is read; the state split is a seeded draw made before any file is opened.
- M-PROVENANCE-OVERREACH: each file's digest is recorded; that the files are the release named rests on their own headers.
- M-FLOOR-UNSTAKED: P-THIN is the plant for the resolution floor.
- M-VACUOUS-SUCCESS: the run prints the number of tracts, counties and households read; §"What would make a stake vacuous" names what a pass could fail to mean.
- M-FOREIGN-DOMAIN-CORROBORATION: a certificate here is about United States households in one survey window and is not evidence about any other country's.
