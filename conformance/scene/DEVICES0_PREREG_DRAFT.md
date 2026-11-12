# DEVICES-0 (United States) — devices per household by kind and household size, which of them are on all day anyway, and the twin's three device censuses scored. PREREGISTRATION (DRAFT)

*REVIEWED 2026-11-11 (`DEVICES0_REVIEW.md`): fixed in `DEVICES0_PREREG.md`, which replaces this text. Kept as written.*

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. NOT APPROVED:
the dataset below has not been approved by the lead by name. Nothing has been fetched or
computed. Data facts are from `SCOUT_DEVICES_POWER.md` (documentation only) and are
re-checked against the files' own codebooks in the build.*

## Why

The twin's power question is a delta from the status quo (`TWIN_THEORY_INPUTS.md`): what
the mesh adds on devices people already run, what it adds in new devices, less what it
removes upstream. The first term needs the devices a household already has, by kind, and
which are on all day. The twin holds three censuses that disagree (2.25 devices per person
on its page; one phone and half a laptop per person in its fine tier; tier and fleet
shares in its scaling model), none with a source.

## Carrier

- **Files.** US Energy Information Administration, Residential Energy Consumption Survey,
  public-use microdata: 2020 (18,496 households, 56 MB) and 2024 (about 17,000 households,
  25 MB, a first release of household characteristics), with their codebooks. Public
  domain; no sign-up; the acknowledgement the agency asks for travels with each table.
- **Read per household:** counts of smartphones, other mobile phones, laptops, desktops,
  tablets, televisions, set-top boxes (three kinds summed), streaming devices, games
  consoles, smart speakers; home internet access and its type; household members
  (top-coded at 7), children, adults over 64; housing type; urban type (urban area, urban
  cluster, rural); state; the final weight and 60 replicate weights.
- **"On all day anyway" is a declared class, not a measurement.** Set-top boxes, smart
  speakers and a home internet connection (standing for a modem or router, which the
  survey does not ask about) are classed always-on; desktops, televisions, consoles and
  streaming devices as "plugged in, state unknown"; phones, laptops and tablets as
  duty-cycled. The survey does not say which devices are on.
- The **blocks.** A state. FIT and SCORE are the seeded split of states used in
  HOUSEHOLD-0 (seed 20261111), so the two campaigns hold out the same places.
- **Error.** Each estimate's standard error from the replicate weights, by the agency's
  own published method (read at the build and quoted). The gap in a verdict is two
  standard errors of the difference.

## The generators under test

- **TWIN-DEVICES-PAGE**: 2.25 devices per person.
- **TWIN-DEVICES-FINE**: 1.0 phone and 0.5 laptop per person; a home node in 0.60 of
  households (no reference here can score the last, and it is said so).
- **FIT-KIND-1**: per kind and household size (1 to 6, and 7-or-more), FIT's weighted mean
  count per household and its share of households with none.

## The match

- **Budgets.** Per kind: 5 % of the reference's count per household (a device count
  multiplies that kind's energy linearly); and `0.03` in the share of households with none
  of the kind (who cannot host on it). Both the author's.
- **Verdict:** certified if miss + gap ≤ budget; refused if miss − gap > budget; else
  unresolved.

## Stakes

- **G0.** The codebook names each column read; the weighted households total within 1 % of
  the agency's stated total for the year; weighted persons per household between `2.3` and
  `2.8`. If not, nothing runs.
- **S1 (the twin's censuses).** Smartphones per person against `1.0`; laptops per person
  against `0.5`; phones, laptops, desktops and tablets per person against `2.25`. Persons
  are the household's members with the top code taken as 7. Reported with verdicts. No
  forecast and no kill: each is read once.
- **S2 (per person is the wrong key).** Smartphones per person fall by at least 10 % from
  two-person to five-person households. *Kill:* by under 5 %. Then one rate per person
  serves and household size adds nothing.
- **S3 (a rule fitted elsewhere).** FIT-KIND-1 against SCORE: certified on the count in at
  least 6 of the 10 kinds at household sizes 1 to 4. *Kill:* refused in 4 or more kinds.
- **S4 (a device table needs a year).** Between 2020 and 2024 the count per household
  moves by more than 5 % in at least 3 of the 10 kinds. *Kill:* in none. The first is a
  pandemic year and the second a first release; both are said beside the result.
- **S5 (density is not needed as a key).** At equal household size, counts per household
  in rural and in urban-area households differ by under 5 % in at least 7 of 10 kinds.
  *Kill:* by over 10 % in 4 or more. Then the device generator needs the tile's class.
- **S6 (what is already on, a reading).** By household size: the share of households with
  at least one device of the always-on class, and the mean count of each such kind. No
  stake; it is the column the twin's energy delta asked for.
- **S7 (kinds together, a reading).** Within a household size, the correlation between
  counts of each pair of kinds, and the share of households with no phone, laptop,
  desktop or tablet at all.

## What would make a stake vacuous

- Counts are of devices "used", self-reported.
- The top code on members makes per-person rates in the largest households too high.
- 2020 was a pandemic year (telework and schooling at home); 2024 is preliminary.
- Urban type is not the twin's density class; S5 answers for three bands, not five.
- Nothing here is outside the United States. Germany publishes means per 100 households
  only; a German check is a later, smaller freeze.
- The always-on class is the author's and decides S6.

## Plants

- **P-EQUAL, P-SHIFT, P-THIN**: as in HOUSEHOLD-0, on synthetic households with weights.
- **P-WEIGHT** (a kind held by a group whose weights are ten times the rest's). The
  weighted mean is recovered; the unweighted one is not.
- **P-REPLICATE** (synthetic replicate weights with a known variance). The standard error
  is recovered within 10 %.
- **P-TOPCODE** (households of 9 recorded as 7). The per-person rate is flagged as an
  upper bound for the top band.
- **P-IMPUTED** (a column whose imputation flag marks half the rows). The share imputed is
  printed beside the estimate.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. The table is published by kind and household size with its errors,
  years and verdicts; a device generator drawing from it is the next freeze.

---
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (S7: counts by kind are margins; the household holds a joint)
witness: none (S1–S7 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a file is read.
- M-BAR-FROM-THE-READ: budgets and bars are stated here; no value of the survey is known to the author beyond the scout's quotation of codebook ranges.
- M-BASE-RATE-OMITTED: national values are printed beside each household size's.
- M-POPULATION-CHOICE: each household in the file, with its weight; the state split is HOUSEHOLD-0's.
- M-PROVENANCE-OVERREACH: file digests are recorded; the agency's acknowledgement travels with the table.
- M-FLOOR-UNSTAKED: P-THIN is the plant for a thin cell (a household size with few records in SCORE).
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
