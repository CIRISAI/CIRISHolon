# PRESENCEGEN-1 — the third generator Holon ships for the twin to call: where a person is in each hour of a day. A chain with one bit of memory against a frame (when the person left and came back) with a chain in it, judged on what neither holds as a parameter. PREREGISTRATION

*Frozen 2026-11-12, committed alone, after one adversarial review (`PRESENCEGEN1_REVIEW.md`)
whose named fixes are applied here. It replaces `PRESENCEGEN1_PREREG_DRAFT.md`, kept. It
builds on `PRESENCE0_RESULTS.md`. At this commit nothing is built or computed. It needs
the approved US time diaries fetched again, through a browser as before.*

*Forecasts on the record. The reviewer expects a plain chain to give too many short spells
away and too many people home at three far-apart hours. Nobody has a forecast for S3 or S4.*

## What is built

In `holon-scene`, integers only, no dependencies, wasm32:
`world::presence::day(seed, employed, day_type, person_seed, week) -> Day`.

- **The six states** of PRESENCE-0. A person's state in a clock hour is their state at one
  minute of that hour.
- **Kinds of day,** read from a diary's minutes: HOME (never away from home); WORK-EARLY
  (120 minutes or more at a workplace or school, first there before noon); WORK-LATE (the
  same, first there at noon or later); OUT (away, without that). Working at home is a day
  at home.
- **G1, a chain with one bit.** Over the six states and whether the person has left home
  today. Transitions by hour, by employed or not and by day type; a row seen in under 30
  diaries uses the row over all day types, and the day is flagged.
- **G3, a frame and a fill.** The kind is drawn from the shares for employed-or-not and
  day type. For a day that is not HOME, the hour first left and the hours until last home
  are drawn from one aggregated table per kind and employed-or-not, in two-hour bins; a
  first-left row with under 30 diaries uses the kind's durations over all rows. Before
  and after the frame the person is at home, asleep or awake by a two-state chain. Within
  it a chain over workplace or school, elsewhere awake, elsewhere asleep, travelling and
  home awake. The within-kind tables are pooled over day types.
- **Returned:** 24 states; the kind; the number of changes of place in the day, drawn from
  the diaries' minute-level count for that kind; a fallback flag.
- **A person's week** is not measured (one diary day each). `week` is a declared knob with
  two ends: each day drawn afresh, or one kind per person for the working week from
  `person_seed`.
- Also shipped, **beside it:** the share employed by age band, so a caller holding only adults
  and adults over 64 can draw the key. Nobody under 15 has a diary; a child gets no day,
  and the call says so.
- **The law.** One state per person per hour (`SceneRhythm.conserved`).

## Carrier

- Diaries of 2022 to 2024 cut as in PRESENCE-0, plus 2015 to 2019 for S4. For each diary
  and clock hour one minute is fixed by a seeded draw, and the state at that minute read.
- FIT is 2022 and 2023; SCORE is 2024. Tables in parts per million, with the survey's
  weights. Each generator is run for each SCORE diary's key under 20 seeds.
- Errors: a weighted bootstrap over SCORE respondents; the gap is two standard errors.
- Budgets: `0.03` in a share; one hour in a quartile.

## Stakes

- **G0 (gate).** One state per hour for 100,000 generated days; one digest native and
  wasm32. On FIT the sampled-minute shares match the minute shares of the same diaries
  within `0.01` in each state, travelling included; and the generated weekly mean awake
  share is within `0.01` of the diaries'. If not, nothing ships.
- The **reproduction checks R,** labelled so. Hourly shares by day type; the share staying
  home; the mean spell away; the share of the employed with no workplace hour. Each
  generator holds these or is fitted on them. Printed with the share of generated
  transitions that came from a fallback row, per state.
- **S1 (how long a spell away lasts).** The share of spells away of 2 hours or less, and
  of 8 hours or more, over all persons. Met: G3 within `0.03` on both and G1 outside on
  at least one. *Kill:* G1 within on both. Then one bit is enough and the frame earns
  nothing; G1 ships.
- **S2 (the shape of a day).** The share at home at 9, at 14 and at 18, all three; and the
  quartiles of hours away among those who leave. Met: G3 within `0.03` and within one hour
  on each quartile. *Kill:* G1 is, and G3 is not better than G1 by more than the gap.
- **S3 (day type beyond the kind of day).** G3's within-kind tables fitted on Monday to
  Thursday, with Saturday's kind shares and 4 a.m. state, predict Saturday's hourly
  at-home-awake share within `0.03` in at least 18 of 24 hours. *Kill:* under 12. Then
  the tables must not be pooled over day types.
- **S4 (the regime as one knob).** Within-kind tables fitted on 2015–2019, kind shares
  from 2022–2023, scored on 2024, Monday to Thursday: the hourly at-home share within
  `0.03` in at least 18 of 24 hours. *Kill:* under 12. If met, the move home after 2020
  is a change in how many days are of each kind, and the generator carries to another
  year by changing those shares.
- **S5 (the household, unresolved by design).** Members drawn independently. Printed for
  two-adult households: nobody home by hour under independence, and its ceiling (the
  smaller of two away shares). The call returns `household_joint: unresolved`.

## What would make a stake vacuous

- Three years of one survey; SCORE is the last. S4 is the stake that leaves that regime.
- S1 and S2 are read on sampled hours: a spell shorter than an hour may be missed in the
  diaries and the generators alike.
- The frame is read from sampled hours too, so "never left" means "at home at each of 24
  sampled minutes".
- A person's week, a household's joint and a child's day are not in the survey.
- Employed-or-not is the only key; age enters through the share employed alone.

## Plants

- **P-HEAP** (trips ending on the hour). Sampling at the hour's start loses a fifth of
  travel; the seeded minute is within `0.003`.
- **P-MEAN** (any chain on synthetic days). Mean spell within 2 %: recorded as the reason
  the mean is a reproduction check and not a stake.
- **P-LATE** (a fifth of working days starting at 14). G1's share of long spells is short
  by more than `0.02`; G3's is within `0.02`.
- **P-CHAIN** (days made by a chain with one bit). G1 reproduces spell lengths and S1 is
  killed, as it should be.
- **P-BORROW** (a row borrowed across day types). The day is flagged.
- **P-ONE** (any generated day). One state per hour; a HOME day never leaves home.
- **P-SEED** (one seed twice; two seeds). Identical; different.
- **P-WEEK** (the knob at "one kind per person"). Five weekdays of one person share a kind.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** The gate or a plant fails. Nothing ships.
- **(a)** S1 or S2 met. G3 ships, pooled over day types if S3 is not killed.
- **(b)** S1 killed and S2 not met. G1 ships.
- **(c)** Neither generator is within budget on S1 and S2. The better one ships marked
  `unresolved` for durations.

---
witness: `CIRISHolon.SceneRhythm.conserved` (one state per person per hour)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (each hour's shares do not fix a person's day; S1 and S2 read the joint)
witness: none (R, S1–S5 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: budgets are stated here. Known at this commit: PRESENCE-0's table of shares. Not known: any transition, frame, spell length or kind share.
- M-BASE-RATE-OMITTED: G1 is scored beside G3 on each stake; P-MEAN records why the mean is no stake.
- M-POPULATION-CHOICE: each diary of 2024 with its weight; 2020 and 2021 not read.
- M-PROVENANCE-OVERREACH: file digests are recorded; tables carry the survey's notice and years.
- M-FLOOR-UNSTAKED: P-BORROW is the plant for thin rows; the fallback share is printed per state.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".

## CORRECTION 2026-11-12, before the parse: the hours away are held in one-hour cells

**What failed.** P-LATE, run on the generators as frozen and on synthetic days alone, before
any diary was parsed. On a carrier whose working days are 8 to 10 hours away, G3 put
`0.445` of spells at eight hours or more against `0.539` planted (G1: `0.412`). The plant
asks G3 to be within `0.02`.

**Why.** The freeze holds the hours until last home in two-hour cells and draws evenly
within a cell. The cell for 7 and 8 hours straddles S1's eight-hour line, so half of the
days of exactly 8 hours come back as 7. The stake's line and the table's cell edge were
chosen apart, and no two-hour grid holds both of S1's lines (2|3 and 7|8) on an edge.

**Amendment.** The hours until last home are held in one-hour cells (24), still one table
per kind and employed-or-not, still conditioned on the two-hour bin of the hour first left,
with the same fallback under 30 diaries. The hour first left stays in two-hour bins.
Nothing else in the freeze changes: stakes, budgets, kills, plants, branches, the split
of years and the seeds are as frozen.

**After the amendment, same carrier.** G3 `0.533` against `0.539`; G1 `0.412`. P-HEAP,
P-MEAN and P-CHAIN pass as before.

**What this costs.** A table with 24 cells per row in place of 12 is thinner per cell; the
share of generated days that use the fallback is printed, as frozen. The amendment was
made knowing the plant's result and nothing of the diaries; the raw files were fetched
and their digests checked, and no row of them had been read by the scoring script.
