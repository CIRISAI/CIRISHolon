# TIES-0 (Germany) — how far friendship ties reach between areas, from a connectedness index: what is identified, a check of the index's own conventions, and whether one inner scale explains the ties within an area. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after a review that ordered a redesign and a second
pass on the redesign (`TIES0_REVIEW.md`), whose named fixes are applied here. The lead
approved the dataset ("approved all data sources"). It replaces both drafts, kept. At this
commit nothing has been fetched or computed.*

*Forecasts on the record. The reviewer guesses the tilt between −0.1 and +0.05, the
band-mean rival near 40 to 45 %, and a visible deficit of ties across the former border.
The author has no forecast for S0 or S4.*

## What this can and cannot give the twin

The twin's "about 92 % of acts stay in the signer's area" rests on where it places a
person's groups (`TWIN_THEORY_INPUTS.md`). This index is one undifferentiated count of
friendship links between areas. It can give an outer envelope on how far kin and
acquaintance ties reach, at the grain of the twin's areas (about 600 km across). It gives
nothing by kind of tie, nothing about the household, nothing about messages, and nothing
measured below the size of an area. The share of ties within a kilometre is measured here
by nobody.

## Carrier

- **Files.** Meta's Social Connectedness Index, 2026 release, the European file (14 MB):
  pairs of regions at three nested levels (NUTS 1, 2, 3). Auxiliary, from the European
  statistical office: each region's population, land area and centre point.
- **What is identified.** The index is links between two areas over the product of their
  user counts. Ratios from one area to two others need no assumption, so the distance
  exponent is identified. A SHARE of an area's ties needs the users of each destination,
  which are not published.
- **The assumption and its one parameter.** Users are taken as `population^(1 + g)`. The
  release's own rule for nested levels (a parent's index is the user-weighted mean of its
  children's) fixes `g`. Each share is printed at `g = 0` and at the fitted `g`.
- The **bands:** the area itself; within 100 km; within 300 km (half the width of one of
  the twin's areas); farther in Germany; foreign Europe (same file; it assumes equal reach
  across countries and is the weakest band); beyond Europe, not identified. An area is
  taken as a disc of its own land area, and a band holds the share of the other area's
  population within the distance of a resident; the centre-point version is printed beside.
- The **strata:** German NUTS 3 areas in population-weighted fifths of density, and in
  thirds of own population; cut points printed.
- The **blocks.** A NUTS 2 region. FIT is a seeded half of Germany's (seed 20261118),
  SCORE the rest.
- **Verdict.** Per density fifth: miss = |predicted − observed| population-weighted mean
  own share on SCORE; budget `0.05`; gap = the 95th percentile of the difference between
  random halves of the fifth's areas, plus the difference between the readings at `g = 0`
  and at the fitted `g` (added linearly, which is conservative), plus the measured privacy
  noise if it exceeds `0.01` in a share. Certified if miss + gap ≤ budget; refused if
  miss − gap > budget; else unresolved.

## Stakes

- **G0.** Each area pairs with itself; values positive.
  - *Scale:* the median log ratio between levels over pairs of regions that are a single
    unit at both levels, checked against the median over off-diagonal parent pairs (agree
    within 2 %). The spread over the single-unit pairs is printed as the privacy noise.
  - *Convention:* over parents whose off-diagonal part is at least half their diagonal
    part, at the fitted `g`: `R_A` = (parent's own index − children's off-diagonal part) /
    children's diagonal part, and `R_B` the same with half the off-diagonal part. The
    convention whose median is within `0.15` of `1` is used; under B the own term is
    doubled in each share, recorded as a dated correction. Both or neither: nothing runs.
- **S0 (are users in proportion to residents).** `g` fitted twice, NUTS 3 into 2 and NUTS 2
  into 1, each with its spread over random halves of parent pairs. Reported. If the two
  disagree by more than their spreads the one-parameter form is said to fail. A stake
  below stands only if it holds at both `g = 0` and the fitted `g`.
- **S1 (how far, a reading).** Per density fifth: the share of ties in each band, with the
  10th and 90th percentiles across areas, at both values of `g`.
- **S2 (the exponent, and whether grain moves it beyond geometry).** The exponent fitted on
  the index itself (log index on log distance with one term per area, on the doubled list
  of pairs, by alternating demeaning) for German pairs 100 to 400 km apart, at NUTS 3 and
  at NUTS 2. The difference expected from unit size alone is computed by P-CENTROID at
  Germany's unit sizes. Met: the observed difference is within `0.15` of that. *Kill:* more
  than `0.3` from it. Printed with and without terms for same Land and for pairs across
  the former border.
- **S3 (density beyond size).** The own share by density fifth within thirds of own
  population. *Kill of "density is a key beyond size":* the top and bottom density fifths
  differ by under `0.03` within each population third, at both values of `g`.
- **S4 (one inner scale).** A kernel `(b + d)^(−c)` times one term per area. The exponent
  and the terms come from off-diagonal pairs with both ends in FIT (pairs closer than the
  sum of their radii left out and counted); a SCORE area's term from its own off-diagonal
  pairs. The floor `b`, one number, from FIT areas' own indices. Met: the predicted own
  share of each SCORE area is within `0.05` for at least 70 % of SCORE's population.
  *Kill:* under 40 %, or not above the rival (the FIT mean own share of the area's density
  fifth). Printed beside it as a rung, no stake: the same with `b` from the off-diagonal
  fit alone, which is expected to fail.
- **S4b (the ladder, a reading).** `b` fitted on NUTS 2 own indices beside `b` on NUTS 3:
  whether one inner scale holds across levels.
- **S5 (the inner border, a reading).** Ties between areas on opposite sides of the former
  border, over the kernel's prediction for their distance.
- **S6 (below the area: a bracket, not a value).** The share of an area's own ties within
  1 km, by carrying the kernel into the area's disc. Endpoints: `b` from S4 and from S4b;
  the exponent from `0.9` to the fitted; a rank law; uniform population. Labelled
  "extrapolation bracket; no population measured below about 20 km here". Printed only
  with S4's verdict; if S4 is killed, or `b` drifts between levels, the bracket is printed
  as having no licence. It is a share of friendship links and must not enter the twin as a
  share of acts.
- **S7 (beside the twin's guesses, a reading).** The share of ties beyond 300 km and in
  foreign Europe, beside the twin's "diaspora 0.12" and "interest groups half in the
  signer's area", with the mismatch of kind said.

*Left for the US freeze:* whether a law in rank (people nearer than the destination)
predicts better than a law in kilometres.

## What would make a stake vacuous

- One parameter `g` cannot see reach that varies with age or between town and country at
  equal population.
- Links among users active in 30 days with more than 20 friends; a stale tie counts as a
  daily one.
- S4 one scale up is evidence about S6's step, not a measurement of it.
- A disc is not an area's shape, and population is not uniform within it.
- The former border and Land borders are pair effects that area terms do not absorb.
- One country.

## Plants

Each synthetic carrier is built from individuals' ties and then aggregated to areas.

- **P-UNIFORM** (ties in proportion to population alone). Exponent zero within `0.05`.
- **P-CENTROID** (one law for individuals, Germany's two unit sizes). The NUTS 3 to NUTS 2
  difference in exponent equals the computed artefact within `0.05`.
- **P-DIAG** (children and parents both published counting a pair once). `R_B` reads `1`,
  `R_A` below `0.9`, and the own term is doubled.
- **P-REACH** (users per resident rising with population, one kernel). The fitted `g` is
  within `0.1` of the planted one.
- **P-FLOOR** (individuals' ties with a floor of 4 km). The off-diagonal fit alone misses
  the own share by more than `0.05`; `b` from FIT own indices recovers it within `0.02`
  and S4 is met.
- **P-LEAK** (SCORE areas' own indices set at random). S4 is killed.
- **P-NOISE** (noise added to each published value). The noise read from single-unit
  pairs is within a fifth of the planted.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each carrier is built to hold the uniform ties, the unit
sizes, the convention, the tilt, the floor, the random own terms or the noise its plant
names, at the level of individuals, so no plant recovers a law by construction at area
level.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. The table of ties by band is published per density fifth at both
  values of `g`, with the exponent, S4's verdict and S6's bracket labelled as one.

---
witness: `CIRISHolon.ScenePopulation.edge_law_does_not_fix_wiring` (shares of ties by band fix no one's friends)
witness: `CIRISHolon.World.outflow_commutes` (a parent's ties are its children's, summed: the nested-level rule G0 and S0 use)
witness: none (S0–S7 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a file is read.
- M-BAR-FROM-THE-READ: bars are stated here. Known to the author: published US figures of an earlier release (an exponent near −2 under 200 miles); no German value.
- M-BASE-RATE-OMITTED: P-UNIFORM is the null in which distance does nothing; S4 must beat the band-mean rival.
- M-POPULATION-CHOICE: each area present in the file; FIT and SCORE by a seeded draw of regions.
- M-PROVENANCE-OVERREACH: file digests are recorded; the licence is the catalogue's statement and is quoted as that.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: S6 carries a law below any scale measured here and is a bracket whose licence is S4; nothing by kind of tie is claimed.
