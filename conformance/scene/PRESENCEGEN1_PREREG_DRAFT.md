# PRESENCEGEN-1 — the third generator Holon ships for the twin to call: where a person is in each hour of a day, drawn so that a tile's people add up to the measured table AND a person's day looks like a day. PREREGISTRATION (DRAFT)

*REVIEWED 2026-11-12 (`PRESENCEGEN1_REVIEW.md`): fixed in `PRESENCEGEN1_PREREG.md`, which replaces this text. Kept as written.*

*DRAFT of 2026-11-12. NOT FROZEN; to be attacked by a second reader first. It builds on
`PRESENCE0_RESULTS.md`. Nothing below is built or computed. It needs the approved US time
diaries fetched again, through a browser as before.*

## Why a table is not enough

PRESENCE-0 gives the share of people in each of six states by day type and hour. The twin
needs more than shares: a phone is on a home network for a run of hours, a household is
empty or not, a person either went to work today or did not. Drawing each hour on its own
from the table reproduces every share and gets every person's day wrong.

## What is built

In `holon-scene`, integers only, no dependencies, wasm32:
`world::presence::day(seed, person, day_type) -> [State; 24]`, with the person given as
age band (15 to 24, 25 to 64, 65 and over) and employed or not, and the six states of
PRESENCE-0 (home asleep, home awake, workplace or school, elsewhere awake, elsewhere
asleep, travelling). The state returned for an hour is the person's state at the start of
that clock hour.

Three generators are built and compared; one ships.

- **G0, each hour alone.** The state at each hour drawn from the table's share for the
  person's stratum, day type and hour.
- **G1, a chain.** The state at 4 a.m. from the table; each next hour from a transition
  table (from state, to state) for the stratum, day type and hour.
- **G2, a kind of day first, then a chain.** One of three kinds of day is drawn for the
  person: a day at home (never leaves), a day with a workplace or school in it, a day out
  without one. Then a chain with that kind's own transition tables.

**The law.** Each person is in exactly one state in each hour, so over any set of people
the six counts sum to the people, in each hour (`SceneRhythm.conserved`).

## Carrier

- The diaries of 2022 to 2024, cut as in PRESENCE-0; a person's state at the start of each
  clock hour is read from the diary. FIT is 2022 and 2023; SCORE is 2024.
- Each table is fitted on FIT with the survey's weights, per stratum (six) and day type
  (four), in parts per million. A transition row seen in under 30 diaries uses the row
  for the stratum over all day types, then for all persons.
- Each generator is run for each SCORE diary's stratum and day type under 20 seeds.
- Errors: a weighted bootstrap over SCORE respondents; the gap is two standard errors.

## Stakes

Budgets: `0.03` in a share; 10 % of a mean duration. All scored on 2024.

- **G0 (gate).** One state per hour for 100,000 generated days; one digest native and
  wasm32; on FIT each generator reproduces the hourly shares to `0.01`. If not, nothing
  ships.
- **R1 (the shares, a reproduction check).** Each generator's hourly shares on SCORE's
  strata against SCORE's, by day type: the share of the 96 cells certified. Expected the
  same for the three, since each is built on the same margins.
- **S1 (a day at home).** The share of persons who are at home in each of the 24 hours.
  Stake: G2 is within budget of the diaries and G1 is not. *Kill:* G1 is within budget.
  Then a chain alone already knows who stays in, and the kinds of day earn nothing.
- **S2 (a working day).** Among the employed on Monday to Thursday, the share at a
  workplace or school for at least 6 of the 24 hour-starts, and the share for none.
  Stake: G2 within budget on both; G1 outside on at least one. *Kill:* G1 within budget
  on both.
- **S3 (how long a spell lasts).** The mean length in hours of an unbroken spell away from
  home, and of an unbroken spell at home awake. Stake: G2 within 10 % on both. *Kill:*
  G2 outside 20 % on either. Then even with kinds of day a first-order chain does not
  carry dwell, and durations must be drawn directly.
- **S4 (leaving and returning, a reading).** Mean number of departures from home a day,
  and the hour of the first, per generator and in the diaries.
- **S5 (the household, a reading that the generator cannot certify).** For households of
  two adults drawn independently by the shipped generator: the share of hours in which
  nobody is home, beside PRESENCE-0's bracket. No diary can score it; said in the output.

## What would make a stake vacuous

- Three years of one survey in one regime; SCORE is the last of them.
- A state at the start of each hour misses a trip that begins and ends inside the hour.
  The departures in S4 are therefore undercounted in the diaries and the generators alike.
- People in one household are drawn independently. They are not independent, and nothing
  here measures how they are not.
- The kinds of day are three because three can be named; a finer set is not tested.
- Nobody under 15; United States; a person's state is not a device's.

## Plants

- **P-ALONE** (synthetic days that never change state). G0 breaks them; G1 and G2
  reproduce them; S3's scoring refuses G0.
- **P-CHAIN** (synthetic days from a known chain with no kinds). G1 recovers spell
  lengths within 5 %; S1 and S2 are killed, as they should be.
- **P-KINDS** (synthetic days of two kinds, home all day or eight hours out, from one
  stratum). G1 gets the stay-home share wrong by more than `0.05`; G2 within `0.02`.
- **P-ONE** (any generated day). One state in each hour.
- **P-SEED** (one seed twice; two seeds). Identical; different.
- **P-THIN** (a transition row seen in 12 diaries). It falls back and says so.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic carrier is built to hold what its plant names.

## Branches

- **(e)** The gate or a plant fails. Nothing ships.
- **(a)** S1 or S2 met and S3 not killed: G2 ships.
- **(b)** S1 and S2 killed: G1 ships; the kinds of day earned nothing.
- **(c)** S3 killed: the best of G1 and G2 ships marked `unresolved` for durations, and
  drawing durations directly is the next design.

---
witness: `CIRISHolon.SceneRhythm.conserved` (one state per person per hour: the six counts sum to the people)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (each hour's shares do not fix a person's day: G0 against G1 and G2)
witness: none (R1, S1–S5 are measured)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-BASE-RATE-OMITTED, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-FLOOR-UNSTAKED, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the shipped function or the scoring that judges it.
- M-BAR-FROM-THE-READ: budgets are stated here. Known at this commit: PRESENCE-0's table of shares. Not known: any transition, spell length, or share of days at home.
- M-BASE-RATE-OMITTED: G0 and G1 are scored beside G2 on each stake.
- M-POPULATION-CHOICE: each diary of 2024 with its weight; 2020 and 2021 not read.
- M-PROVENANCE-OVERREACH: file digests are recorded; tables carry the survey's notice and years.
- M-FLOOR-UNSTAKED: P-THIN is the plant for thin transition rows.
- M-VACUOUS-SUCCESS: §"What would make a stake vacuous".
