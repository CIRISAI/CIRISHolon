# TIES-0 (Germany) — results, 2026-11-11. Branch (a). Three quarters of friendship ties lie within 300 km and a third within the home district; a law fitted between areas does NOT predict ties within one (KILLED), so the share within a kilometre has no licence; and users are not in proportion to residents.

*Prereg `TIES0_PREREG.md`, the record before the parse `ties0/BEFORE_THE_PARSE.md` (which
says three plants are weaker as coded than as frozen), instrument `ties0/ties0.py`, log
`ties0/run1.log`, numbers `ties0/results.json`, table `tables/ties0_de/ties_table.json`
with its sidecar. 400 German districts as origins, 1,323 European areas as destinations.
One run; the gate passed first time.*

## What it is

One undifferentiated count of friendship links between areas, on one platform, among
users active in a month with more than 20 friends. Not kinds of tie, not the household,
not messages, and nothing measured below the size of a district (median radius 16 km).

## The gate: the index's own conventions (G0)

- The three nested levels reconcile: the scale between them read from 52 single-unit
  regions agrees with the one read from parent pairs to `0.4 %`.
- **The own-area convention is the one the freeze assumed** (`R_A` median `0.94`, `R_B`
  `1.22`). No doubling was needed.
- **The privacy noise is about 2 %** at the finest level (read from regions that appear
  unchanged at two levels), and about 8 % one level up.

## Are users in proportion to residents (S0): NO, and one parameter does not capture how

The tilt `g` in users ∝ population^(1+g) is `0.12 ± 0.02` from one pair of levels and
`0.04 ± 0.02` from the other. They disagree by more than their spreads, so by the freeze
the one-parameter form fails. More populous areas have more users per resident; how much
depends on the level. Each share below is given at `g = 0` and at `g = 0.12`.

## How far ties reach (S1, a reading under that assumption)

Share of a district's friendship ties, population-weighted, by density fifth (`g = 0`):

| density fifth | own district | within 100 km | 100–300 km | farther in Germany | foreign Europe |
|---|---|---|---|---|---|
| sparsest | 0.44 | 0.26 | 0.13 | 0.09 | 0.09 |
| 2 | 0.41 | 0.28 | 0.12 | 0.08 | 0.12 |
| 3 | 0.38 | 0.27 | 0.12 | 0.08 | 0.15 |
| 4 | 0.28 | 0.32 | 0.13 | 0.10 | 0.16 |
| densest | 0.23 | 0.24 | 0.14 | 0.15 | 0.23 |
| **all** | **0.35** | **0.27** | **0.13** | **0.10** | **0.15** |

- **Three quarters (0.75) of ties lie within 300 km**, the half-width of one of the twin's
  areas. A quarter do not. Ties beyond Europe are not in the file, so the true share
  within 300 km is lower.
- **Dense places reach farther.** The own-district share falls from `0.44` to `0.23` and
  the foreign share rises from `0.09` to `0.23`.
- At `g = 0.12` the columns move by at most `0.02`. Centre-point bands differ from disc
  bands by under `0.01` at these distances.
- Foreign Europe assumes equal reach across countries and is the weakest column.

## The exponent (S2): met, narrowly, and it is not one number

- Between 100 and 400 km: `1.02` at the district grain, `1.18` at the regional grain. The
  difference is `0.16`; unit size alone accounts for `0.03`; the stake allowed `0.15`
  beyond geometry and the observed excess is `0.13`.
- With terms for the same Land and for the former border the exponent falls to `0.78`:
  **being in the same Land multiplies ties by 1.7; a pair across the former border has
  0.73 of the ties its distance predicts.**
- Over all distances beyond 25 km the fitted exponent is `1.24`. The curve steepens at
  short range. Any single exponent is a price in a regime.

## Density beyond size (S3): not killed

Within each third of own population the own-district share still falls with density, by
`0.15` to `0.24` between the sparsest and densest fifths present. Density is a key beyond
the size of the place.

## One inner scale (S4): KILLED

A kernel fitted on ties between districts in half of Germany's regions, with one term per
district, was asked to predict each held-out district's own share by carrying the kernel
into the district.

| | held-out population within 0.05 | median miss |
|---|---|---|
| the kernel, floor fitted on own indices | 2.8 % | 0.37 |
| the rival: the mean own share of the density fifth | 53.5 % | 0.05 |

- The stake needed 70 % and the kill was under 40 % or not above the rival. Refused in
  each of the five density fifths, at both values of `g`.
- The fitted floor ran to the bottom of its range and the law still under-predicts: ties
  within a district are far more numerous than any continuation of the between-district
  law allows. **Being in the same district carries more than distance does**, as being in
  the same Land does.
- The same failure one level up: the floor is at the edge of its range at both levels.

## Below the district (S6): NO LICENCE

By the freeze, the bracket for "share of own-district ties within 1 km" has no licence
because S4 was killed. It is printed so its width is on record: for the median district,
from `0.4 %` (uniform population) to `48 %` (a rank law); for a city of radius 4 km, from
`6 %` to `68 %`. That is two orders of magnitude. Nobody measured it, and no value from it
should enter the twin.

## Readings

- **The former border (S5):** pairs across it have `0.93` of the ties the plain kernel
  predicts and pairs on one side `1.03`; with the explicit term, `0.73`.
- **Beside the twin's guesses (S7):** `0.10` of ties are farther than 300 km within
  Germany and `0.15` are in foreign Europe: `0.25` outside one of the twin's areas, against
  its "diaspora 0.12". Its "community groups 90 % in the signer's area" is not the same
  quantity; friendship links are more scattered than that at `0.75`, and they count stale
  ties.

## What this is not, and what follows

- Not the number the locality claim most needs. The household and close kin, which carry
  most messages, are a handful of a person's hundreds of links.
- The step the twin wants (carry a distance law down to the street) was tested one scale
  up and FAILED. So the twin's sub-area placement must stay a labelled input with a sweep,
  and the honest statement of the locality claim is at the grain of districts and above.
- What the table gives the twin: per density fifth, the share of acquaintance-and-kin
  links in its own district, within 100 km, within 300 km, in the rest of the country and
  abroad; a same-Land factor of 1.7; and the finding that dense places reach farther.
- What would give ties by kind and near distances: surveys that ask how far away relatives
  and close friends live. Those need registrations and are the lead's decision.
