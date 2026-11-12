# PRESENCE-0 (United States) — who is at home, awake, at work or travelling in each hour of the week, from time diaries; the twin's flat duty cycles scored. PREREGISTRATION (DRAFT)

*REVIEWED 2026-11-11 (`PRESENCE0_REVIEW.md`): fixed in `PRESENCE0_PREREG.md`, which replaces this text. Kept as written.*

*DRAFT of 2026-11-11. NOT FROZEN; to be attacked by a second reader first. The dataset was
approved by the lead ("approved all data sources"). Nothing has been fetched or computed.
Data facts are from `SCOUT_RHYTHM_TIES.md` (documentation only, partly from archived
copies of the publisher's pages) and are re-checked against the files' own dictionaries.*

## Why

The twin's protocol wakes a device about 127 times an hour while it is awake, so hours
awake and on which network multiply phone energy more than traffic does. Every duty cycle
in the twin is flat over the week (awake share `0.85`; on Wi-Fi `0.65`; offline `0.15`;
"work hours 8 to 18"). The twin's presence-weighted locality and its energy delta both
wait on `presence.people` by hour of week.

## Carrier

- **Files.** American Time Use Survey, multi-year microdata 2003–2024: activity,
  respondent, roster and who-present files (113 MB). Public domain, no sign-up.
- **Unit.** One person aged 15 or over per household, one diary day from 4 a.m. Each
  diary is cut into the 24 clock hours it covers; the day of week is the diary day's
  (hours after midnight belong to the next day). A person's state in an hour is the state
  holding most of its minutes.
- **States:** at home asleep; at home awake; at a workplace or school; elsewhere; travelling.
- **Sleep has no recorded place.** A sleep episode is "at home" if the located episode
  before it or after it is at home, otherwise "elsewhere". The share of sleep minutes so
  placed, and the result if all sleep is taken as at home, are printed.
- **Pools.** Main: diary years 2022–2024, the survey's own final weight. Second: 2015–2019.
  The years 2020 and 2021 are not read.
- **Strata, each printed:** household size (1, 2, 3, 4, 5 or more); age (15–24, 25–64, 65
  and over); employed or not.
- **Error.** A bootstrap over respondents within year and day type (500 resamples, seed
  20261117). It ignores the survey's design and is a lower bound; said beside each value.
- **Hold-out.** Rows are fitted on 2022 and 2023 and scored on 2024.

## What is scored

- **TWIN-FLAT**: awake `0.85` in each hour; "at work" for the employed from 8 to 18 on
  weekdays and never otherwise.
- **FIT-PRESENCE-1**: the 168-hour table of the five states, fitted on 2022–2023.
- **Budget:** `0.03` in a state's share in an hour (the author's). Verdict rule as in
  `HOUSEHOLD0_DE_PREREG.md`, the gap being two bootstrap standard errors.

## Stakes

- **G0.** Dictionary names match; weighted shares over a day sum to one in each hour; mean
  sleep lies between 8 and 9.5 hours a day; the weighted count of diaries per day of week
  is within 20 % of even. If not, nothing runs.
- **S1 (flat is wrong, a reading).** The awake share's range over the 168 hours, and the
  number of hours in which `0.85` is within budget. Expected: a wide range and few hours.
- **S2 (the working day).** Among the employed on weekdays, the hours in which more than
  half of the peak at-workplace share is reached. Reported against "8 to 18". And the peak
  share itself, which says how much of the employed are at a workplace at once.
- **S3 (the week has two shapes).** At-home-awake differs between weekdays and weekend
  days by more than `0.05` in at least 12 of the 24 clock hours. *Kill:* in under 4.
- **S4 (household size is a key).** The at-home share (asleep or awake) from 9 to 17 on
  weekdays differs between one-person households and households of three or more by at
  least `0.05`. *Kill:* by under `0.02`. Then presence need not depend on the household
  generator's output.
- **S5 (the regime moved).** That same at-home share is higher in 2022–2024 than in
  2015–2019 by at least `0.03`. *Kill:* by under `0.01`.
- **S6 (a table fitted on other years).** FIT-PRESENCE-1 against 2024: certified in at
  least 150 of 168 hours for the at-home-awake share. *Kill:* refused in 20 or more.
- **S7 (network, a labelled proxy).** At home or at a workplace is read as "on Wi-Fi",
  elsewhere and travelling as "on a mobile network". The share on Wi-Fi by hour, beside
  the twin's `0.65`. No stake: the survey records no network.
- **S8 (screens, a reading).** The share whose main activity in the hour is television,
  computer use or a phone call. The survey records no secondary activity, so this is a
  floor on device use.

## What would make a stake vacuous

- One person per household: nothing here says whether ANYONE is home.
- Nobody under 15 has a diary.
- "Awake" is not "device awake", and a phone asleep in a pocket is not a person asleep.
- No geography: the same table serves each density class, which is an assumption.
- S6's three years are one regime; it certifies stability, not truth.
- The hour's majority state hides short trips.

## Plants

- **P-MIDNIGHT** (a diary whose sleep runs across midnight). The hours after midnight carry
  the next day's name.
- **P-SLEEPPLACE** (sleep between two located episodes, one at home; and between two
  away). Home; elsewhere.
- **P-WEIGHT** (weekend diaries at twice the rate with half the weight). The weighted
  table is flat across days; the unweighted is not.
- **P-FLAT** (synthetic diaries with one state in each hour). S1's range is zero and S3 is
  killed.
- **P-SHIFT** (a held-out year with at-home raised `0.10` from 9 to 17). S6 refuses those
  hours and names them.
- **P-MAJORITY** (an hour split 31 and 29 minutes). The state with 31.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** G0 or a plant fails. Nothing is read.
- **(a)** Otherwise. The 168-hour table is published by stratum with its errors and
  verdicts, as the rhythm row `presence.people`; a presence generator is the next freeze.

---
witness: `CIRISHolon.SceneRhythm.conserved` (the five states' shares sum to one in each hour)
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the order: envelope, floor, match)
witness: none (S1–S8 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through this reader before a file is read.
- M-BAR-FROM-THE-READ: budgets and bars are stated here; no value of the survey is known to the author.
- M-BASE-RATE-OMITTED: the all-persons table is printed beside each stratum's.
- M-POPULATION-CHOICE: each diary of the years named, with its weight; 2020 and 2021 left out by rule.
- M-PROVENANCE-OVERREACH: file digests are recorded; the dictionary was read from archived copies and is re-read from the files.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
