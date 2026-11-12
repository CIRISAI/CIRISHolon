# HOUSEHOLD-0 (United States) — persons per household by neighbourhood density, read under two keys, and the first scene certificate issued: the twin's own household-size output against two stated consumer budgets. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`HOUSEHOLD0_REVIEW.md`)
whose named fixes are applied here, and on the lead's approval of the same day ("yes, run
the campaign for the US and for whatever european country has the most data available"). It
replaces `HOUSEHOLD0_PREREG_DRAFT.md`, kept. At this commit nothing has been fetched, no
instrument exists, and nothing has been computed on the survey by anyone; the review ran one
synthetic script (`household0/review_synthetic.py`). Data facts are from `ACS_DOCS.md`
(documentation only) and are re-checked against the files' own headers in the build.*

*What the review forecast, stated so that it is on the record before the read: the twin's
rule will be certified on the mean and refused on composition. The author has no forecast
for S2 or S2b, and those are the stakes this campaign is run for.*

## Why

The CEWP twin's models read how many persons a household holds. The twin sets households at
people over `2.5`, draws each a size from a normal (mean `2.5`, spread `1.1`, rounded,
clamped to 1 to 6), the same everywhere, and apportions the leaf's people over households
with those sizes as weights. No reference stands behind that rule. This campaign reads the
reference that exists for one country and issues the first verdicts of the scene
certificate on output the twin itself emits.

## Carrier

- **Tables.** American Community Survey, 5-year estimates 2020–2024, from the table-based
  summary file (no key, no agreement), at census tract (summary level 140) and block group
  (150), for the 50 states and DC. B25009 (occupied housing units by tenure by household
  size, 1 to 6 and "7-or-more"), B25008 (population in occupied housing units), B25032 and
  B25033 (occupied units, and population in them, by units in structure), B26001
  (group-quarters population; tract only), the geography labels, and land area per tract
  and per block group from the Bureau's own geography files of the same vintage (the
  gazetteer where it carries the unit, otherwise the land-area attribute of the TIGER/Line
  file for it). Puerto Rico is a separate survey and is not read.
- **Terms.** No licence text is attached; the Bureau asks to be cited as the source of the
  original data and forbids re-identification. The notice that travels with every table
  produced here is fixed in the build from the Bureau's own citation page and quoted there.
  Raw files are not committed; their digests are.
- **Two keys.** Every table is produced twice: by the tract's own density and by the block
  group's own density, each as household population (B25008) over land area, at the twin's
  five lower bounds (`0 / 250 / 1,500 / 6,000 / 12,000` per km²) with the twin's floor of
  `1`. The difference between the two keys, per class, is printed beside each reading. No
  unit is joined to any grid tile. A table row for class X describes households living in
  units whose AVERAGE density is X; neither key is the tile class of UNEVEN-0.
- **Exclusions, each counted and printed.** A unit with zero land area; a unit with zero
  households; a unit carrying a negative value (the file's codes for "not computed") in any
  cell read; a tract whose group-quarters population exceeds 20 % of its total, and the
  block groups of such a tract.
- The **blocks.** A county, as the release's own identifiers define one (Connecticut's
  planning regions if that is what the release carries; the build prints which). The states
  and DC, sorted by their two-digit codes, are permuted by
  `numpy.random.default_rng(20261111).permutation`; the first 26 are FIT, the other 25
  SCORE. The draw is made and printed before any table is opened.
- A class being **resolved.** With `w` a county's households in the class, the effective number of
  counties is `(Σw)² / Σw²`. A class is resolved for a comparison when that number is at
  least `30` in each set of counties the comparison uses.
- What is **read per class and key.** Shares of households at `1, 2, 3, 4, 5, 6, 7+`; persons
  per household as the published household population over households; the same by tenure
  and by the structure bands B25033 carries (named from its header). National values are
  printed beside each.
- **The open top band.** Placed at `8` persons on both sides of every W1. The residual
  mean, (household population − Σ k × households of size k, k = 1 to 6) over households of
  7 or more, is printed per class with its value when the household population is moved by
  ±1 %. It enters no verdict.

## The generator under test, and the fitted one

- **TWIN-SIZE-0** is the histogram emitted by the twin's own library code on about 10⁶
  households in leaves of 60 households and 150 people, with a stated seed, committed in
  the twin's tree as `docs/geo/household_hist.json`, its commit and sha256 recorded in this
  campaign's log BEFORE any survey table is parsed. Requested from the twin's session
  2026-11-11 and agreed. Its mean is its persons over its households; its one-person share
  is its count at size 1 over its households. If the twin states that size depends on
  anything besides the leaf's people and household count, each emitted histogram is scored.
- **The port.** This campaign's own implementation of the rule
  (`clamp(round(2.5 + 1.1 z), 1, 6)`, largest-remainder apportionment with the drawn sizes
  as weights) is a cross-check. Its agreement with the twin's file, to `0.002` in each
  share, is printed. It is not what any verdict is about.
- **FIT-CLASS-1**: per class under the tract key, the FIT states' pooled persons per
  household and one-person share. **FIT-NATION-1**: the same two numbers pooled over all
  FIT units regardless of class. Nothing else is fitted.

## The match: two consumers, two budgets, stated before any file is read

- **Households per leaf.** |persons per household, generator − reference| ≤ `0.12`. The
  twin places one always-on router per household and fixes households at people over the
  mean, so this is a 5 % change in always-on nodes per street. The 5 % is the author's
  tolerance on the consumer's behalf, not a measured requirement.
- **Composition.** |one-person share, generator − reference| ≤ `0.03`. Who is home, and
  which households hold a second device, follow the share of people living alone. The
  `0.03` is the author's and is not sourced.
- The **reference's own error.** For each class and key the counties are split at random
  into two halves 1,000 times (seed 20261112); the 95th percentile of the gap between
  halves, in each of the two quantities, is printed. A class whose 95 % gap exceeds half a
  budget is `unresolved` for that consumer. The published margins of error are not used.
- **Verdict order**, per consumer, as `verdict_certified_iff` has it: envelope (United
  States, 2020–2024, households, the stated key), then the floor (resolved, and the
  reference's error under half the budget), then the match.
- **W1** on `1 … 7+` is printed for each comparison as a reading, with both distributions.

## Stakes

- **G0 (the files are the tables).** Each table's header matches the shells in
  `ACS_DOCS.md`; national households from B25009 equal those from B25032 to 1; block-group
  households sum to tract households nationally within 0.1 %; national persons per
  household lies between `2.3` and `2.8`. If not, nothing runs.
- **S1 (the twin's output, per consumer).** Reported nationally and per resolved class
  under each key: the verdict for households per leaf and the verdict for composition.
  Expected: certified on the first nationally; refused on the second in each resolved
  class. *Kill of the expectation:* composition certified in 2 or more resolved classes
  under the tract key. Then a single symmetric draw is good enough for both consumers.
- **S2 (is density a useful key).** Units: each SCORE county's households in one class
  under the tract key, where they number at least `20,000` (about 1,600 interviews, a
  sampling error near `0.01` in the share; the households so covered are printed as a share
  of all SCORE households). A unit is WITHIN when both of its quantities are inside both
  budgets of the rule offered. The household-weighted share of units WITHIN under
  FIT-CLASS-1 exceeds that under FIT-NATION-1 by at least `0.05`. *Kill:* not higher. Then
  density class adds nothing a consumer can see. Between `0` and `0.05`: neither met nor
  killed, and said so.
- **S2b (the key transfers).** Persons per household by class differs between the tract key
  and the block-group key by less than `0.12`, and the one-person share by less than
  `0.03`, in each class resolved under both. No kill: a class where it does not is marked
  on the table as not transferable to a leaf.
- **S3 (classes differ).** The largest difference between any two resolved classes under
  the tract key is at least `0.12` in persons per household, or at least `0.03` in the
  one-person share. *Kill:* both under half those.
- **S4 (structure and tenure).** Reported, no bar: the two quantities by structure band
  and by tenure within each class.

## What would make a stake vacuous

- S1's expected outcome is known from a simulation and a remembered national figure. Its
  content is the witness, per class, and that the twin's own output is what was scored.
- S2 and S1 use one survey. A unit WITHIN is within this survey's estimate of it.
- S2's unit floor keeps only large counties' classes. What it says about small counties is
  nothing, and the covered share is printed for that reason.
- Both budgets are the author's. A consumer with a different tolerance reads the printed
  differences and not the verdicts.
- A twin leaf has no tract and no block group. For the twin's own use each verdict here is
  outside the envelope until S2b says the key transfers, and S2b compares two census units
  with each other, not either with a tile.

## Plants

Each runs on a synthetic carrier through the same code, before any table is opened.

- **P-EQUAL** (two halves of 200 counties drawn from one distribution; sector: the
  verdict). `certified` for both consumers.
- **P-SHIFT** (P-EQUAL with one side's one-person share raised by `0.10`; sector:
  composition). `refused`, witness naming the one-person share.
- **P-MEAN** (all households of 2 and 3 against half of 1 and half of 4, both of mean
  `2.5`; sector: the shape). Households per leaf `certified`, composition `refused`, W1
  printed as `1.0`.
- **P-TOP** (5 % of households in the top band at a true mean of 9, overall mean `2.75`;
  sector: the open band). The residual reads `9.0`; with the household population raised
  1 % it reads `9.55`; W1 against a copy of itself is `0` in both cases.
- **P-THIN** (one side with an effective 12 counties; sector: the floor). `unresolved`, no
  verdict printed.
- **P-NEFF** (100 counties, one holding 99 % of the households; sector: the effective
  count). `unresolved`.
- **P-NOISY** (a class whose halves differ by `0.02` in the one-person share at the 95th
  percentile; sector: the error floor). `unresolved` for composition only.
- **P-GQ** (a tract that is 90 % dormitory; sector: the flag). Excluded and counted, with
  its block groups.
- **P-SENTINEL** (a tract carrying a negative code in one cell; sector: the read).
  Excluded and counted; no pooled value moves by more than that tract's households.
- **P-ZEROLAND** (a tract with zero land area; sector: the class). Excluded and counted,
  no division.
- **P-PORT** (the port with its apportionment step removed; sector: the cross-check). It
  must DISAGREE with the port as specified by more than `0.002` in some share.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold the equal halves, the
shifted share, the equal means, the top band, the thin side, the dominant county, the noisy
halves, the dormitory, the coded cell, the zero area or the missing step its plant names.

## Branches

- **(e)** G0 or a plant fails, or the twin's file is not on record before the first parse.
  Nothing is read.
- **(a)** S1's expectation stands. Two verdicts per consumer are issued for output the twin
  emitted, on the reference's own unit, with witnesses. S2, S2b and S3 are each reported as
  met, killed or neither; none of them gates the issue.
- **(b)** S1's expectation is killed. Stated with the classes where composition certified.

## What is published

Under `conformance/scene/tables/household0_us/`: per class and key, the shares in parts per
million, persons per household, the 95 % split-half gap of each, the residual top mean and
its sensitivity, the same by tenure and structure band; the verdicts with reasons and
witnesses; S2's two shares and coverage; S2b's per-class differences and transfer marks;
and a sidecar in the form `tables/uneven0/sidecar.json` uses.

## What this does not claim

Not device counts; not households per building; not storeys; not any other country; not the
1 km tile. Not group quarters: they are 2 to 3 % of people (the review's figure, to be read
from B26001 and printed), the population grid holds them, and the twin makes each a
household member, so its household count is high by about that share. Not vacant units.
The window includes 2020, collected at about two thirds of the usual responses.

---
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (P-MEAN: equal means, different shapes; and the survey gives size and structure as margins)
witness: `CIRISHolon.ScenePopulation.pop_total` (persons are apportioned over households; the generator under test is a section of the leaf's people)
witness: none (S1–S4 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant is built for this reader and runs before a table is read.
- M-BAR-FROM-THE-READ: both budgets, the unit floor and each bar are stated here, before any file exists on this machine; none is set from the survey.
- M-BASE-RATE-OMITTED: the national values are printed beside each class's, and S2 scores the class rule against the national rule.
- M-POPULATION-CHOICE: the tracts and block groups of the 50 states and DC are read whole; exclusions are rules stated above and counted; the state split is a seeded draw made before any file is opened.
- M-PROVENANCE-OVERREACH: each file's digest is recorded; that the files are the release named rests on their own headers. The twin's histogram is the twin session's statement of what its code emits, recorded with its commit and digest.
- M-FLOOR-UNSTAKED: P-THIN and P-NEFF are the plants for the count floor, P-NOISY for the error floor.
- M-VACUOUS-SUCCESS: the run prints the units, counties and households read and excluded; §"What would make a stake vacuous" names what a pass could fail to mean.
- M-FOREIGN-DOMAIN-CORROBORATION: the generator scored is the twin's own emitted histogram; this campaign's port of the rule is a cross-check and is not what a verdict is about.
