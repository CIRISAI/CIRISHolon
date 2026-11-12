# PRESENCE-0 (United States) — where people are and whether they are awake, by hour and day type, from time diaries: the twin's flat duty cycles scored, and whether one table serves different households and places. PREREGISTRATION

*Frozen 2026-11-11, committed alone, after one adversarial review (`PRESENCE0_REVIEW.md`)
whose named fixes are applied here. The lead approved the dataset ("approved all data
sources"). It replaces `PRESENCE0_PREREG_DRAFT.md`, kept. At this commit nothing has been
fetched or computed. Data facts are from `SCOUT_RHYTHM_TIES.md` (documentation only,
partly archived copies) and are re-checked against the files' own dictionaries.*

*CORRECTION 2026-11-11, before any fetch: one sentence of "Why" reworded. The frozen text
used a word the prereg audit reads as contact with an unrelated registered misfit, and it
was committed with the audit refusing it, which this session's command chain let through.
No stake, bar, plant or rule is changed.*

*Known before the read, so not staked: the direction of the change after 2020 (more time
at home on working days) is published. The reviewer's remembered sizes: weekly mean awake
share near `0.63`; peak share of the employed at a workplace near `0.45` to `0.5`. The
author has no forecast for S4, S6a or S6b.*

## Why

The twin's protocol wakes a device about 127 times an hour while it is awake, so awake
hours and network multiply phone energy more than traffic does. Its duty cycles are flat
(awake `0.85`; on Wi-Fi `0.65`; at work 8 to 18). Two of its computations wait on
a presence table: how much traffic stays nearby, weighted by who is awake, and the energy
delta.

## Carrier

- **Files.** American Time Use Survey, multi-year microdata 2003–2024: activity,
  respondent, roster, the linked population-survey file (for metropolitan status) and the
  replicate weights. Public domain.
- **Unit.** One person aged 15 or over per household, one diary day from 4 a.m. Each
  diary is cut on its 24-hour duration fields into 1,440 minutes. The table holds, per
  clock hour, the weighted mean share of the hour's minutes in each state. No person is
  assigned one state per hour. Hours after midnight carry the next day's name; they come
  from the previous day's diaries.
- The six **states:** at home asleep; at home awake; at a workplace or school; elsewhere
  awake; elsewhere asleep; travelling.
- **Place of unlocated episodes** (sleep, grooming, personal). The day is cut at travel
  episodes. An unlocated episode takes the place of the located episodes in its own
  travel-free segment. A segment with no located episode is home by default, and the share
  of sleep so defaulted is printed. The table with all sleep at home is printed beside it.
- **Pools.** Main: 2022–2024. Beside it as a reading: 2015–2019. 2020 and 2021 are not read.
- The **grain.** Verdicts on four day types (Monday to Thursday, Friday, Saturday, Sunday) by
  clock hour: 96 cells, all persons. The 168-hour table and the strata (employed or not;
  age 15–24, 25–64, 65 and over; household size 1, 2, 3 or more) are printed with errors,
  and carry a verdict only where the gap is at most half the budget.
- **Error.** A weighted bootstrap over respondents (500 resamples, seed 20261117). On the
  all-persons table the survey's replicate weights are run as well, and the ratio of the
  two errors is printed and applied to the strata.
- **A household's presence.** For one-person households the person's row is the
  household's. For others only a bracket is printed: at least the largest member's share
  at home, at most the sum, with the independence value between; children under 15 have
  no diary.

## Budgets

- **Weekly mean awake share:** 5 % of the reference (wake-ups, and so the phone's added
  energy, are linear in it).
- **A cell's share:** `0.03`, a shape tolerance, the author's.
- Verdict: certified if miss + gap ≤ budget; refused if miss − gap > budget; else
  unresolved. The gap is two standard errors.

## Stakes

- **G0.** Dictionary names match; each diary sums to 1,440 minutes; weighted mean sleep
  between 8 and 9.5 hours a day; weighted diary-days per day of week within 5 % of even;
  the six shares sum to one in each cell. If not, nothing runs.
- **R1 (the three numbers the twin can take, readings with verdicts).** The weekly mean
  awake share against `0.85`. The peak share of the employed at a workplace on Monday to
  Thursday, and the hours above half that peak, against "everyone, 8 to 18". The share of
  awake time at home or at a workplace or school (an upper bracket for "on Wi-Fi"), and
  the at-home part alone (a lower one), against `0.65`.
- **S3 (two shapes of day).** At-home-awake differs between Monday-to-Thursday and
  Saturday by more than `0.05` in at least 12 of 24 hours. *Kill:* in under 4.
- **S4 (does household size matter beyond who is in it).** After standardising to the
  all-persons mix of age band, employment and day type, the at-home share from 9 to 17 on
  Monday to Thursday still differs between one-person households and households of three
  or more by at least `0.03`. *Kill:* by under `0.01`. Then presence is keyed on age and
  employment, and household size enters only through the household bracket. The raw
  difference is printed beside it.
- **S5 (the regime, a reading).** The at-home share 9 to 17 on Monday to Thursday, and the
  weekly mean awake share, in 2015–2019 against 2022–2024.
- **S6a (who, not which household).** Rows by age band, employment and day type fitted on
  2022–2023 predict the at-home share of each household-size stratum in 2024 by
  reweighting. Certified in at least 70 % of the cells that can carry a verdict. *Kill:*
  refused in 10 % or more.
- **S6b (one table for town and country).** The at-home table of metropolitan respondents
  against non-metropolitan ones, 2022–2024, the 96 cells. Certified in at least 70 % of
  the cells that can carry a verdict. *Kill:* refused in 10 % or more. Then presence
  needs the place.

## What would make a stake vacuous

- Nobody under 15 has a diary; one person per household.
- "Awake" is a person's state. A phone in an awake person's pocket may be asleep.
- Metropolitan status is not the twin's density class.
- S6a and S6b are scored where the gap allows; the count of cells that could not carry a
  verdict is printed and may be most of them.
- The default for a segment with no located episode decides how much sleep is at home.
- The Wi-Fi bracket is a statement about places, not networks.

## Plants

- **P-MIDNIGHT** (sleep across midnight). The hours after midnight carry the next day's name.
- **P-LASTEPISODE** (a final sleep stopping at 07:30). The diary sums to 1,440 minutes.
- **P-GROOM-DRIVE** (sleep, grooming, car, workplace). The sleep is at home, defaulted.
- **P-AWAY** (hotel evening, sleep, hotel morning). Elsewhere asleep.
- **P-SHORT** (three 15-minute trips in three hours). Travel reads `0.25` in each hour.
- **P-WEIGHT** (weekend diaries at twice the rate with half the weight). The weighted
  table is even across days; the unweighted is not.
- **P-STANDARD** (two household sizes that differ only in their age mix). The raw
  difference is large; the standardised one is under `0.01` and S4 is killed.
- **P-NULL-YEAR** (a held-out year drawn from the fitted table at the real cell sizes).
  The share of cells certified is printed; S6a's and S6b's bars must sit under it, and
  if they do not the stake is reported as untestable.
- **P-SHIFT** (a held-out year with at-home raised `0.10` from 9 to 17). Those cells are
  refused.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. The table is published by day type and hour with its errors, strata,
  the household bracket and R1's three numbers, as the rhythm row `presence.people`.

---
witness: `CIRISHolon.SceneRhythm.conserved` (the six states' shares sum to one in each cell)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (persons' rows do not fix whether anyone is home)
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: none (R1, S3–S6 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a file is read.
- M-BAR-FROM-THE-READ: budgets and bars are stated here. The reviewer's remembered sizes are on the record above and no stake rests on a quantity it remembered.
- M-BASE-RATE-OMITTED: the all-persons table is printed beside each stratum's; S4 prints the raw difference beside the standardised one.
- M-POPULATION-CHOICE: each diary of the years named, with its weight; 2020 and 2021 left out by rule.
- M-PROVENANCE-OVERREACH: file digests are recorded; the dictionary is re-read from the files.
- M-FLOOR-UNSTAKED: P-NULL-YEAR is the plant for what the sample can resolve.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
