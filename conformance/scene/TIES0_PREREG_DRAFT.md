# TIES-0 (Germany and the United States) — what share of a place's friendship ties lie within its own area, within 25, 50 and 200 km, in the country and abroad, from a connectedness index between areas; and whether a law fitted on half the places predicts the rest. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. The dataset was
approved by the lead ("approved all data sources"). Nothing has been fetched or computed.
Data facts are from `SCOUT_RHYTHM_TIES.md` (documentation only).*

## Why

The twin's "about 92 % of acts stay local" is `0.65 + 0.30 × 0.90` on its own inputs
(`TWIN_THEORY_INPUTS.md`). The only field that moves it is where the members of a person's
groups live. No reference stands behind the twin's placements (family all in one locality;
community groups 90 % local; interest groups half local).

## Carrier

- **Files.** Meta's Social Connectedness Index, the 2026 release (friendships as of
  25 January 2026; marked CC0 by its catalogue): European regions at NUTS 3 (14 MB), US
  counties (245 MB), and the country-pair file (0.6 MB). Auxiliary, each small and from a
  public statistical office: region and county populations, land areas and centre points.
- **What the index is.** Friendships between two areas over the product of their user
  counts, scaled within each file. It is a relative probability. User counts are not
  published.
- **The stated assumption that turns it into shares.** An area's users are taken in
  proportion to its resident population. Then the share of area `i`'s ties that lie in
  area `j` is `index(i, j) × population(j)` over the sum of that across `j`.
- **Bands,** by distance between centre points: the area itself; other areas within 25 km,
  50 km, 200 km; farther in the country; abroad. The abroad share comes from the
  country-pair file by the same assumption and is printed apart: it is the weakest number
  here.
- **What it is not.** Not kinds of tie (kin, friends, colleagues are one). Not messages.
  Not anything inside an area: a county or a NUTS 3 region is hundreds of square
  kilometres. Users with 20 friends or fewer are left out; areas under 500 users are
  missing; the published values carry privacy noise of unstated size.
- The **blocks.** An area. FIT and SCORE: Germany's three bands of six, US states by the
  seeded split of HOUSEHOLD-0, so the hold-outs are those of the household campaigns.
- **Strata:** the area's population density in five bands by the twin's class bounds.

## What is scored

- **TWIN-LOCAL** (a reading only): the twin's localities are about 600 km across, so its
  "90 % of community groups local" is compared with the share of ties within 200 km and
  within the country, with the mismatch of scale said.
- **FIT-GRAVITY-1**: `ties(i, j) ∝ population(j) × distance^(−a)` beyond the own area, with
  one exponent per country and one own-area share per density band, fitted on FIT.
- **Budget:** `0.05` in the share of ties within 50 km (the author's; a tenth of the
  quantity's likely size).

## Stakes

- **G0.** Each area pairs with itself; index values positive; the US population-weighted
  mean share of ties within 50 miles lies between `0.45` and `0.65` (a published analysis
  of the 2016 release found `0.554`). If not, nothing runs.
- **S1 (how local, a reading).** Per country and density band: the share of ties in each
  band, with the 10th and 90th percentiles across areas.
- **S2 (one law for two countries).** The fitted exponent under 200 km differs between
  Germany and the United States by under `0.3`. *Kill:* by over `0.6`. Then no exponent
  travels and each country needs its own.
- **S3 (dense places reach farther).** The share of ties within 50 km is lower in the two
  densest bands than in the two sparsest by at least `0.05`, in both countries. *Kill:*
  higher in either. Direction not known to the author.
- **S4 (a law fitted elsewhere).** FIT-GRAVITY-1 predicts the share of ties within 50 km
  of SCORE areas within `0.05` for at least 70 % of their population. *Kill:* under 40 %.
- **S5 (the inner border, a reading).** For German areas within 100 km of the former
  border, the share of ties across it against the gravity law's prediction.
- **S6 (below the area, a labelled prior).** The own-area share is spread inside the area
  by a `1/distance` law with a floor (two published studies of other populations), giving
  a share within 1 km. Printed with both studies' ranges. No verdict can attach to it.

## What would make a stake vacuous

- Users in proportion to residents is false where the platform's reach varies (by age,
  between town and country). Each share inherits it.
- The index weights a stale tie and a daily one alike.
- S4's hold-out areas exchange ties with the fitted ones; the two halves are not
  independent.
- Centre-point distance misstates large areas and neighbours.
- The privacy noise is of unstated size; small areas' values are the noisiest.
- S6 moves a law from one population and platform to another, and decides the number the
  twin most wants.

## Plants

- **P-UNIFORM** (ties in proportion to population alone). The fitted exponent is zero
  within `0.05`.
- **P-GRAVITY** (ties made with an exponent of `1.3`). Recovered within `0.05`; S4 met.
- **P-TWO-LAWS** (two countries made with exponents `1.0` and `1.8`). S2 is killed.
- **P-SCALE** (one file's values multiplied by a constant). No share changes.
- **P-MISSING** (an area absent from the file). Its ties are not counted as zero: shares
  are over the areas present, and the population missing is printed.
- **P-DENSE** (dense areas made with a smaller own-area share). S3 met.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. The table of ties by distance band is published per country and
  density band with its percentiles, the assumption it rests on, and S6's prior labelled
  as one.

---
witness: `CIRISHolon.ScenePopulation.edge_law_does_not_fix_wiring` (shares of ties by band fix no one's friends)
witness: `CIRISHolon.World.outflow_commutes` (an area's ties to each band sum to its ties)
witness: none (S1–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS, M-FOREIGN-DOMAIN-CORROBORATION. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a file is read.
- M-BAR-FROM-THE-READ: bars are stated here. Known to the author: the published 2016 US figures quoted in the scout report, used only as G0's range.
- M-BASE-RATE-OMITTED: P-UNIFORM is the null in which distance does nothing, and S1 prints population-only shares beside the read ones.
- M-POPULATION-CHOICE: each area present in the files; hold-outs are the household campaigns' seeded splits.
- M-PROVENANCE-OVERREACH: file digests are recorded; the licence is the catalogue's statement and is quoted as that.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
- M-FOREIGN-DOMAIN-CORROBORATION: S6 carries a law from other populations and is labelled a prior; S2 is the test of whether one country's exponent says anything about the other's.
