# DEVICEGEN-1 — the second generator Holon ships for the twin to call: who lives in a household of a given size, and the devices it holds, drawn jointly, with what is already on all day and could host. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-12. NOT FROZEN; to be attacked by a second reader first. It builds on
`DEVICES0_RESULTS.md`. Nothing below is built or computed. It needs the approved US
residential energy survey files fetched again (2020 and 2024), because the generator's
tables must be fitted on part of the sample and scored on the rest.*

## What is built

In `holon-scene`, integers only, no dependencies, wasm32:

`world::devices::generate(seed, persons, building, reference) -> HouseholdDevices`

- **Inputs.** `persons`: the household's size, from the household generator. `building`:
  detached house, attached house, small apartment building, large apartment building,
  mobile home, or unknown. `reference`: United States 2020, the only one.
- **Step 1, who lives there.** Adults 18 to 64, adults over 64 and children, drawn from the
  survey's joint distribution of the three given the household's size. The three sum to
  `persons` for each seed.
- **Step 2, a latent class.** DEVICES-0 found that households with no computer are the
  households with no fixed line, 2.9 times more often than chance. The generator draws
  one of a few classes per household (the number fixed below) and draws each kind given
  the class, so that the kinds' joint is carried and no kind is drawn alone.
- **Step 3, personal devices** (smartphones, laptops, tablets): for each person, by that
  person's group and the household's class.
- **Step 4, fixed devices** (desktops, televisions, set-top boxes, streaming devices,
  consoles, smart speakers) and the connection (fixed line, mobile data only, none): by
  household size, building and class.
- **Output.** A count per kind; the connection; a gateway declared present with a fixed
  line; each kind's two declared attributes (power state; can host); and
  `has_host_capable_always_on`, true where a desktop or a gateway is present, labelled as
  a declaration. One verdict per consumer: counts, and "has nothing to host on".
- **The law.** Persons by group sum to `persons`; each count is a whole number; a
  household of nobody holds nothing.

## The classes

A latent-class model with `K` classes over: fixed line (yes, mobile only, none); any
laptop, desktop or tablet (yes, no); any smart speaker or streaming device (yes, no).
`K` is chosen on FIT among 2, 3 and 4 as the smallest whose fitted share of households
with no computer AND no fixed line is within `0.01` of FIT's own. Fitted by a fixed
number of expectation-maximisation steps from a seeded start; the tables are then frozen
as integers in parts per million.

## Carrier and hold-out

- The 2020 file. FIT and SCORE are the seeded split of states used in HOUSEHOLD-0 (seed
  20261111). Each table is fitted on FIT households with their weights.
- The generator under test is run once per SCORE household, given that household's own
  size and building, seed its row number; its output is pooled with the household's weight.
- Errors: the agency's jackknife on the survey side; the gap is two standard errors of the
  survey's own value.
- Budgets, from DEVICES-0: 15 % of a count per household; `0.05` in a share. Cells pooled
  upward until the survey's relative error is at most half the budget.

## Stakes

- **G0.** The section law for 100,000 calls; one digest native and wasm32; the fitted
  tables reproduce FIT's own margins to `0.005`. If not, nothing ships.
- **S1 (counts, on states it never saw).** For each of the nine kinds and pooled household
  size, generated against SCORE's mean count. Met if certified in at least 80 % of cells.
  *Kill:* refused in 20 % or more.
- **S2 (the joint that decides "devices added").** The share of households with no
  laptop, desktop or tablet and no fixed line: generated against SCORE's, within `0.02`.
  And the ratio to the product of the two separate shares at least `2.0`. *Kill:* the
  ratio under `1.5`: the classes did not carry the clustering.
- **S3 (the building).** For desktops, televisions, set-top boxes and smart speakers, the
  generated difference between detached houses and large apartment buildings at equal
  household size has the survey's sign and at least half its size, in each of the four.
  *Kill:* in 2 or more it does not.
- **S4 (the shape).** For smartphones and for smart speakers, the generated variance over
  mean at household sizes 1 to 4 is within `0.25` of the survey's (smartphones are far
  narrower than a Poisson; speakers far wider). *Kill:* off by more than `0.5` at 2 or
  more sizes for either.
- **S5 (who lives there).** Generated shares of households with any child, and with any
  adult over 64, by household size, within `0.03` of SCORE's.
- **S6 (another year, a reading).** The same generator against the 2024 file. DEVICES-0
  already killed "a 2020 table holds in 2024"; this prints by how much per kind, as the
  generator's known drift.
- **S7 (against independence, a reading).** The same scoring for a generator that draws
  each kind alone from its own margin by household size. It must lose S2 and S3; if it
  does not, the classes and the building earn nothing.

## What would make a stake vacuous

- FIT and SCORE are halves of one survey in one year.
- The survey's own building type and household size are passed in. The twin will pass
  generated ones; a wrong household size in gives wrong devices out.
- "Can host" and "on all day" are declarations. The survey never asks about a router.
- A latent class is a device for carrying a joint, not a kind of household anyone measured.
- 2020 was a pandemic year; United States only.

## Plants

- **P-INDEP** (synthetic households whose kinds are independent). `K = 2` reproduces the
  joint and the ratio reads near `1`.
- **P-CLUSTER** (synthetic households with a have-nothing class of 9 %). The fitted
  classes recover its share within `0.01` and S2's ratio reads above `2`.
- **P-PERSONS** (households of 1 to 7). Persons by group sum to the size for each seed.
- **P-BUILDING** (desktops made twice as common in houses). S3's scoring sees the sign
  and size.
- **P-NARROW** (one smartphone per adult exactly). Variance over mean reads below `0.1`.
- **P-SEED** (one seed twice; two seeds). Identical; different.
- **P-BROKEN** (personal devices drawn per household, not per person). S4 fails for
  smartphones.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing ships.
- **(a)** S1 and S2 met. The generator ships with its tables, verdicts and its year.
- **(b)** Otherwise. It ships with each failed consumer marked `refused` or `unresolved`
  and the witness; the twin's legacy device rule stays the fallback there.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (persons by group sum to the household's size for each seed)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (kinds drawn alone match each margin and miss the joint: S7)
witness: none (S1–S7 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: budgets are DEVICES-0's. Known at this commit: the all-sample tables of DEVICES-0. Not known: any FIT or SCORE value, or any class.
- M-BASE-RATE-OMITTED: S7 scores the independent generator beside the one shipped.
- M-POPULATION-CHOICE: each SCORE household once, with its weight; the split is HOUSEHOLD-0's.
- M-PROVENANCE-OVERREACH: tables carry the agency's acknowledgement, the file digests and the year.
- M-FLOOR-UNSTAKED: cells are pooled until the survey's error is at most half the budget; P-NARROW and P-BROKEN are the plants for the shape.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
