# DEVICEGEN-1 — the second generator Holon ships for the twin to call: who lives in a household of a given size, and the devices it holds, with the connection, the computer and the speaker drawn together from one table. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`DEVICEGEN1_REVIEW.md`)
whose named fixes are applied here. It replaces `DEVICEGEN1_PREREG_DRAFT.md`, kept. It
builds on `DEVICES0_RESULTS.md`. At this commit nothing is built or computed. It needs the
approved US residential energy survey files fetched again.*

*Forecasts on the record. None by the author for S7 or S8. The reviewer expects the
reproduction checks to pass where the sample can resolve them and says so.*

## What is built

In `holon-scene`, integers only, no dependencies, wasm32:
`world::devices::generate(seed, persons, building, reference) -> HouseholdDevices`.

- **Inputs.** `persons`, the household's size. `building`: house (detached, attached or
  mobile home), apartment, or unknown. `reference`: United States 2020, the only one; any
  other country returns `unresolved`, reason "outside the country read".
- **Step 1, who lives there.** Adults 18 to 64, adults over 64 and children, drawn from the
  survey's joint of the three given household size and building. They sum to `persons`
  for each seed. A size above the survey's top code is drawn as the top code and the
  remaining persons added as adults 18 to 64, and the call says so.
- **Step 2, one of 12 cells.** For each pooled household size (1, 2, 3, 4, 5 or more) and
  building, one cell of: connection (fixed line, mobile data only, none) × any laptop,
  desktop or tablet × any smart speaker or streaming device, drawn from the survey's
  weighted frequencies in parts per million. A stratum with under 30 survey records uses
  its size band over both buildings.
- **Step 3, smartphones** per person: 0, 1 or 2, by the person's group and the cell.
- **Step 4, each other kind** (laptops, desktops, tablets, televisions, set-top boxes,
  streaming devices, consoles, smart speakers): a count drawn from the survey's weighted
  histogram, 0 to 8, by pooled size, cell, building and whether there are children, made
  consistent with the cell (a household in a "no computer" cell holds none of the three).
  Thin strata fall back by dropping, in order, children, building, then the speaker part
  of the cell.
- **Step 5.** Television hours a day and weekdays someone is home, each from its
  histogram by pooled size.
- **Output.** Persons by group; smartphones by group; a count per kind; the connection;
  `gateway`: declared present with a fixed line, `can_host` unknown; `desktop`:
  `can_host` declared, `on_all_day` unknown; television hours; weekdays someone is home.
  No combined flag. One verdict per consumer (counts; nothing to host on), each reason
  carrying the year 2020 and the kind's change to 2024 from DEVICES-0.
- **The law.** Persons by group sum to `persons`; counts are whole numbers; a household of
  nobody holds nothing.

## Carrier and error

- The 2020 file. FIT and SCORE are the seeded split of states of HOUSEHOLD-0 (seed
  20261111). Each table is fitted on FIT households with their weights.
- Each SCORE household is generated under 20 seeds from its own size and building and
  pooled with its weight. Beside each generated value is printed the table's expectation
  computed with no draw.
- The gap is two standard errors of the survey's value by the agency's jackknife.
  Budgets from DEVICES-0: 15 % of a count per household; `0.05` in a share.

## Stakes

- **G0.** The section law for 100,000 calls; one digest native and wasm32; on FIT the
  12-cell frequencies are reproduced to `0.002`. If not, nothing ships.
- **R1 to R5 (reproduction checks, labelled so; pooled over the held-out half they are
  verdicts on the TABLE).**
  - R1: no cell of kind by pooled size is refused, and each kind is certified in its
    all-sizes cell.
  - R2: the generated ratio of the have-nothing share (no computer and no fixed line) to
    the product of its margins lies within the survey's value ± its gap.
  - R3: for desktops, televisions, set-top boxes and smart speakers, the generated
    house-against-apartment difference lies within the survey's ± (its gap + 15 points).
  - R4: for smartphones and smart speakers, generated variance over mean within 20 % of
    the survey's plus its gap, at each pooled size.
  - R5: shares of households with any child and with any adult over 64, by pooled size,
    by the verdict rule with a budget of `0.05`.
- **S7 (the rival: do the cells earn anything).** A rival draws each kind ALONE from its
  own histogram given pooled household size, building and who lives there (children or
  not; any adult over 64 or not), with no cell. Stake: the rival's have-nothing ratio is
  under `1.5` while the shipped generator's is within the survey's gap. *Kill:* the
  rival's ratio is within the survey's gap. Then size, building and composition carry the
  joint, the cells are dropped, and the simpler generator ships.
- **S8 (the chain, a reading).** Sizes drawn from the household generator's US row;
  building drawn (a) from the survey's distribution given size and (b) at the survey's
  overall share whatever the size. The change in mean count per household for desktops,
  televisions, set-top boxes and smart speakers between (a) and (b). No bar: it is what a
  consumer loses by drawing building and size apart.
- **S9 (composition against the census, a reading).** After the chain in (a): the share
  of all persons who are children and who are over 64, beside the census's national
  figures; and the any-senior share of one-person households by building.
- **S6 (another year, a reading).** The generator against the 2024 file, per kind.

## What would make a stake vacuous

- R1 to R5 can pass because FIT and SCORE are halves of one survey; where a cell reads
  `unresolved` the sample could not say.
- S7's rival is given composition, which the shipped generator also uses; if the survey's
  composition carries the clustering the kill fires, and that is a finding.
- The twin will pass a generated size and building. S8 measures one way that goes wrong.
- "Can host" and "on all day" are declarations. The survey never asks about a router.
- 2020 was a pandemic year; United States only; the twin's "home node in 0.60 of
  households" still has no reference.

## Plants

- **P-CELLS** (synthetic households with a known 12-cell table). Reproduced to `0.002`.
- **P-CLUSTER** (a have-nothing share three times independence, unrelated to size,
  building or composition). The shipped generator's ratio is within `0.2` of 3; the
  rival's is under `1.5`.
- **P-COVARIATE** (a have-nothing share explained wholly by building). The rival's ratio
  matches and S7's kill fires.
- **P-PERSONS** (sizes 1 to 9). Persons by group sum to the size for each seed; sizes 8
  and 9 are flagged as above the top code.
- **P-CONSISTENT** (a "no computer" cell). No laptop, desktop or tablet is drawn.
- **P-THIN** (a stratum with 12 records). It falls back and says which level it used.
- **P-SEED** (one seed twice; two seeds). Identical; different.
- **P-TWOPHONES** (persons who each hold two phones). The generated count exceeds the
  number of persons.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing ships.
- **(a)** S7 met. The generator with cells ships, with its tables, verdicts and year.
- **(b)** S7 killed. The simpler generator (no cells) ships, and the results say the
  covariates carried the joint.
- **(c)** S7 neither. The generator with cells ships, marked as not shown to beat the
  rival.

---
witness: `CIRISHolon.ScenePopulation.pop_total` (persons by group sum to the household's size for each seed)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (kinds drawn alone match each margin and may miss the joint: S7)
witness: none (R1–R5, S6–S9 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: budgets are DEVICES-0's. Known at this commit: DEVICES-0's all-sample tables, including the ratio 2.9. Not known: any FIT or SCORE value, the rival's ratio, or the chain's effect.
- M-BASE-RATE-OMITTED: S7 scores a rival beside the generator; each generated value has the table's no-draw expectation beside it.
- M-POPULATION-CHOICE: each SCORE household, 20 seeds, with its weight; the split is HOUSEHOLD-0's.
- M-PROVENANCE-OVERREACH: tables carry the agency's acknowledgement, the file digests and the year.
- M-FLOOR-UNSTAKED: P-THIN is the plant for thin strata; the fallback level used is recorded per stratum.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
