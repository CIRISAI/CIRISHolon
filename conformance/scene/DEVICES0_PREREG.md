# DEVICES-0 (United States) — devices per household by kind and by who lives there, what a household already has that is on all day and could host, and the twin's device censuses scored. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`DEVICES0_REVIEW.md`)
whose named fixes are applied here. The lead approved the dataset the same day ("approved
all data sources"). It replaces `DEVICES0_PREREG_DRAFT.md`, kept. At this commit nothing
has been fetched or computed. Data facts are from `SCOUT_DEVICES_POWER.md` (documentation
only) and are re-checked against the files' own codebooks in the build.*

*Forecasts on the record. None by the author for S2, S5 or S7. The reviewer thinks rural
homes plausibly differ by more than 10 % in set-top boxes, desktops and connection type.*

## Why

The twin's power question is a delta per household: what the mesh adds on devices people
already run, plus devices added for it, less what it removes upstream
(`TWIN_THEORY_INPUTS.md`). The second term turns on whether a household already has
something on all day that could host a node. The twin holds three device censuses that
disagree, none with a source.

## Carrier

- **Files.** US Energy Information Administration, Residential Energy Consumption Survey,
  public-use microdata of 2020 (18,496 households) and of 2024 (a first release), with
  their codebooks. Public domain. The 2015 file is read only if its codebook carries the
  same device counts, and then only for the reading S4.
- What is **read per household:** counts of smartphones, other mobile phones, laptops, desktops,
  tablets, televisions, set-top boxes (three kinds summed), streaming devices, games
  consoles, smart speakers; home internet and its type; members, children, adults over
  64 (adults 18 to 64 by subtraction); weekdays someone is home; television hours;
  housing type; urban type; the final weight and the replicate weights.
- **Two declared attributes per kind, neither measured:**

  | kind | power state | can host a node |
  |---|---|---|
  | gateway (declared present with fixed broadband) | always on | unknown |
  | set-top box, smart speaker | always on | no |
  | desktop | plugged in, state unknown | yes |
  | television, console, streaming device | plugged in, state unknown | no |
  | laptop, tablet, phone | duty-cycled | yes, while awake |

  A household with cellular data only is a third state: the survey does not say whether
  it has a fixed-wireless gateway or only a phone.
- **Error.** Each estimate and each difference is computed inside each replicate, and its
  standard error taken by the agency's own published method (read at the build and
  quoted). The gap in a verdict is two standard errors.
- **Pooling.** A cell is published where its relative standard error is at most half its
  budget; household sizes are pooled upward (5 or more, then 4 or more) until it is.
- **No state hold-out.** The table is fitted on each record of 2020. The out-of-sample
  test is the later year.

## What is scored

- **TWIN-DEVICES-PAGE**: 2.25 devices per person. **TWIN-DEVICES-FINE**: 1.0 phone and 0.5
  laptop per person. Both are world figures in the twin; a verdict here is about the US.
- **FIT-KIND-2020**: per kind and pooled household size, the weighted mean count and the
  share of households with none, from 2020.
- **Budgets:** 15 % of the reference's count per household (the count multiplies a device
  power known far less well than that); `0.05` in the share with none. The author's.
- **Verdict:** certified if miss + gap ≤ budget; refused if miss − gap > budget; else
  unresolved.

## Stakes

- **G0.** The codebook names each column read; the weighted household total within 1 % of
  the agency's stated total for the year; weighted persons per household between `2.3`
  and `2.8`; members equal children plus the two adult groups in at least 99 % of
  households below the top code. If not, nothing runs.
- **S1 (the twin's censuses, a reading).** Smartphones per person against `1.0`; laptops
  per person against `0.5`; phones, laptops, desktops and tablets per person against
  `2.25`; households of 7 or more left out. Verdicts printed; no kill.
- **S2 (who lives there, not how many).** Three rates per kind (per adult 18 to 64, per
  adult over 64, per child), fitted by weighted least squares to smartphones, laptops and
  tablets. Met if they predict the mean count at each household size from 1 to 6 within
  budget for all three kinds. *Kill:* outside budget at 2 or more sizes for smartphones.
  Then household size matters beyond who is in it.
- **S3 (a table fitted in another year).** FIT-KIND-2020 against 2024, by kind and pooled
  size. Met if certified on the count for smartphones, laptops and televisions at each
  pooled size. *Kill:* refused at some size for 2 of the 3. The first year is a pandemic
  year and the second a first release; both are said beside the result.
- **S4 (drift, a reading).** Per kind, the change in the count per household from 2020 to
  2024, with 2015 as a third point if it can be read: the sign over each interval and
  2020's distance from the line between the other two.
- **S5 (does the building matter).** At equal household size (standardised to 2020's size
  mix), the count per household in detached houses against buildings of five or more
  units differs by under 10 % plus its gap in at least 7 of 10 kinds. *Kill:* by over
  20 % in 4 or more. The same contrast by urban type is printed.
- **S6 (what is already there, a reading).** By pooled household size: the share with
  fixed broadband; with cellular data only; with a desktop; with no laptop, desktop or
  tablet; weekdays someone is home; television hours.
- **S7 (do the have-nots cluster).** The share of households with no laptop, desktop or
  tablet AND no fixed broadband, over the product of the two separate shares. Met at
  `1.5` or more: kinds must be drawn jointly. *Kill:* under `1.2`: they may be drawn
  independently.
- **S8 (the family, a reading).** Per kind and household size, the variance of the count
  over its mean. Under one it is narrower than a Poisson, over one wider.

## What would make a stake vacuous

- Counts are of devices "used", self-reported; the survey never asks about a router.
- The declared attributes decide what S6 means. They are the author's.
- S3's two years differ by a pandemic and by release status as well as by time.
- S2's adults by subtraction fail where members are top-coded.
- S5's housing type is not the twin's density class, and urban type is settlement size.
- Nothing here is outside the United States.
- 15 % is wide. A consumer that needs better reads the printed errors.

## Plants

- **P-EQUAL, P-SHIFT**: as in HOUSEHOLD-0, on synthetic weighted households.
- **P-WEIGHT** (a kind held by a group whose weights are ten times the rest's). The
  weighted mean is recovered; the unweighted is not.
- **P-REPLICATE** (synthetic replicate weights of known variance). The standard error is
  recovered within 10 %.
- **P-POOL** (a household size with 40 records). It is pooled upward and the pooled cell
  is what is published.
- **P-RATES** (households made with rates 0.95, 0.70 and 0.30). S2's fit recovers each
  within `0.03` and S2 is met; with a size effect added, S2 is killed.
- **P-CLUSTER** (have-nots made independent; then made to coincide). S7 reads near `1.0`,
  then above `1.5`.
- **P-TOPCODE** (households of 9 recorded as 7). They are left out of S1 and S2.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. The table is published by kind and pooled size with its errors, the
  declared attributes, the three rates and the joint; a device generator is the next freeze.

---
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (S7: the share with none of each kind does not fix the share with none of any)
witness: none (S1–S8 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a file is read.
- M-BAR-FROM-THE-READ: budgets and bars are stated here; no value of the survey is known to the author beyond the codebook ranges the scout quoted.
- M-BASE-RATE-OMITTED: national values are printed beside each household size's.
- M-POPULATION-CHOICE: each household in each file, with its weight.
- M-PROVENANCE-OVERREACH: file digests are recorded; the agency's acknowledgement travels with the table.
- M-FLOOR-UNSTAKED: P-POOL is the plant for the pooling floor.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
