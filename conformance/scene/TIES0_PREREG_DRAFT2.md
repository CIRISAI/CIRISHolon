# TIES-0 (Germany) — how far friendship ties reach between areas, from a connectedness index: what is identified, a check of the index's own conventions, and whether a law fitted between areas predicts ties within one. PREREGISTRATION (SECOND DRAFT)

*SECOND DRAFT of 2026-11-11, after `TIES0_REVIEW.md` (REDESIGN). NOT FROZEN; it goes back to
the same reader first. The dataset was approved by the lead. Nothing has been fetched or
computed.*

## Why, and what this can and cannot give the twin

The twin's "about 92 % of acts stay local" rests on where it places a person's groups
(`TWIN_THEORY_INPUTS.md`). This index is one undifferentiated count of friendship links
between areas. It can give an outer envelope on how far kin and acquaintance ties reach,
at the grain of the twin's localities (about 600 km across). It cannot give ties by kind,
anything about the household, messages, or anything inside an area. The number the twin
most wants, the share within a kilometre, is not measured here by anyone.

## Carrier

- **Files.** Meta's Social Connectedness Index, 2026 release, the European file (14 MB):
  pairs of regions at three nested levels (NUTS 1, 2, 3). Auxiliary, from the statistical
  office: each region's population, land area and centre point.
- **What is identified.** The index is links between two areas over the product of their
  user counts. Ratios of the index from one area to two others need no assumption, so the
  distance exponent is identified. A SHARE of an area's ties needs the users of each
  destination, which are not published.
- **The assumption, and its one testable parameter.** Users are taken as
  `population^(1 + g)`. `g = 0` is "in proportion to residents". The release's own rule for
  nested levels (a parent's index is the user-weighted mean of its children's) fixes `g`:
  it is fitted so that NUTS 2 pairs are reproduced from NUTS 3 pairs. Each share is
  printed at `g = 0` and at the fitted `g`.
- **Bands:** the area itself; within 100 km; within 300 km (the twin's locality); farther
  in Germany; foreign Europe (from the same file); beyond Europe, not identified. An area
  is taken as a disc of its own land area and a band holds the share of the other area's
  population within the distance of a resident; the centre-point version is printed beside.
- **Strata:** German NUTS 3 areas in population-weighted fifths of density, cut points
  printed. The twin's tile bounds are not applied to areas.
- The **blocks.** A NUTS 2 region. FIT is a seeded half of Germany's (seed 20261118),
  SCORE the rest.
- **Gap.** The 95th percentile of the difference between random halves of the areas
  concerned, plus the difference between the reading at `g = 0` and at the fitted `g`.
  Verdict: certified if miss + gap ≤ budget; refused if miss − gap > budget; else
  unresolved.

## Stakes

- **G0.** Each area pairs with itself; values positive; the three levels' scales are
  reconciled on off-diagonal parent pairs; and THE CONVENTION: (a parent's own index less
  its children's off-diagonal part) over (its children's diagonal part) has a median
  within `0.15` of `1`. If it sits near `0.5` or `2`, the own term's convention is
  changed, recorded as a dated correction, before any share is read. If it sits elsewhere,
  nothing runs.
- **S0 (are users in proportion to residents).** The fitted `g` with its spread over
  random halves of parent pairs. Reported. A stake below stands only if it holds at both
  values of `g`.
- **S1 (how far, a reading).** Per density fifth: the share of ties in each band, with the
  10th and 90th percentiles across areas, at both values of `g`.
- **S2 (the exponent, and whether grain moves it).** The exponent fitted on the index
  itself (log index on log distance with a term for each area, by alternating demeaning)
  on German NUTS 3 pairs 100 to 400 km apart, and on German NUTS 2 pairs over the same
  range, differ by under `0.3`. *Kill:* by over `0.6`. Then the exponent is a property of
  the units and cannot be carried down.
- **S3 (is density a key).** The own share and the share within 100 km by density fifth.
  Direction not known. *Kill of "density is a key":* the top and bottom fifths differ by
  under `0.03` in both quantities at both values of `g`.
- **S4 (a law fitted between areas predicts ties within one).** A kernel
  `A (b + d)^(−c)` fitted on off-diagonal pairs with both ends in FIT areas predicts the
  own share of each SCORE area, by integrating the kernel over the area's disc, within
  `0.05` for at least 70 % of SCORE's population. *Kill:* under 40 %. Between: "not
  established", printed with the median miss. The rival it must beat is the FIT mean own
  share of the area's density fifth.
- **S4b (the ladder, a reading).** The same step one and two levels up: a kernel fitted on
  NUTS 1 pairs predicting NUTS 2 own shares, and on NUTS 2 pairs predicting NUTS 3. The
  median miss at each step says whether the error grows going down.
- **S5 (the inner border, a reading).** Ties between areas on opposite sides of the former
  border, over the kernel's prediction for their distance.
- **S6 (below the area: a bracket, not a value).** The share of an area's own ties within
  1 km, by carrying the fitted kernel inside the area. Endpoints from the floor (zero to
  the fitted `b`), the exponent (`0.9` to the fitted `c`) and uniform population. Labelled
  "extrapolation bracket; no population measured below about 20 km here". Printed only
  with S4's verdict beside it; if S4 is killed the bracket is printed as having no licence.
  It is a share of friendship links and must not enter the twin as a share of acts.
- **S7 (beside the twin's guesses, a reading).** The share of ties beyond 300 km and the
  share in foreign Europe, beside the twin's "diaspora 0.12" and "interest groups local
  0.5", with the mismatch of kind said.

## What would make a stake vacuous

- The one parameter `g` cannot see reach that varies with age or between town and country
  at equal population.
- Links among users active in 30 days with more than 20 friends; a stale tie counts as a
  daily one; privacy noise of unstated size biases small far values.
- S4's kernel is fitted without the per-area terms S2 uses, so its level rests on the
  assumption as well.
- S4 one scale up is evidence about S6's step, not a measurement of it.
- A disc is not an area's shape, and population is not uniform inside it.
- One country.

## Plants

- **P-UNIFORM** (ties in proportion to population alone). Exponent zero within `0.05`.
- **P-CENTROID** (one law for individuals on a plane, units of two sizes). S2 is not
  killed; the two band methods are printed and differ.
- **P-DIAG** (own terms halved). G0's convention check reads near `0.5` and fires.
- **P-REACH** (users per resident rising with population, one kernel). The fitted `g` is
  within `0.1` of the planted one, and S3 is reported as not standing at both values.
- **P-KERNEL** (individuals' ties from a known kernel, aggregated to areas). S4 is met and
  beats the band-mean rival.
- **P-LEAK** (SCORE areas' own shares set at random). S4 is killed.
- **P-NOISE** (additive noise with a floor on far pairs). The far band's bias is printed.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built from individuals' ties and
then aggregated, so no plant recovers a law by construction at area level.

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
