# PRESENCE-0 (United States) — results, 2026-11-11. Branch (a). People are awake 15 hours a day, not 20; half the employed are at a workplace at the peak, not all; and the working day moved home after 2020. The two transfer stakes are untestable at this budget.

*Prereg `PRESENCE0_PREREG.md` (with its dated correction), the record before the parse
`presence0/BEFORE_THE_PARSE.md`, instrument `presence0/presence0.py`, log
`presence0/run1.log`, numbers `presence0/results.json`, table
`tables/presence0_us/presence_table.json` with its sidecar. 24,353 diaries of 2022–2024;
50,649 of 2015–2019 beside them. One run; the gate passed first time.*

## The three numbers the twin can take (R1)

| the twin assumes | the diaries say | |
|---|---|---|
| a device's owner is awake `0.85` of the time (20.4 hours a day) | `0.623` (15.0 hours a day), from `0.04` at 3 a.m. to `0.98` in the early evening | **refused** |
| the employed are at work from 8 to 18 | at most `0.51` of the employed are at a workplace at once (Monday to Thursday); above half that peak from 7:00 to 17:00 | a reading |
| a phone is on Wi-Fi `0.65` of the time | of awake time, `0.61` is at home and `0.79` at home or at a workplace or school | inside the bracket |

- The protocol's wake-ups are linear in awake hours, so the twin's phone term is high by
  about a third on this count alone (20.4 against 15.0 hours), before any question of
  whether a phone is awake when its owner is.
- The Wi-Fi line is a statement about places. Nothing here measures a network.

## The table

Share of persons aged 15 and over, Monday to Thursday, by clock hour (the full table has
four day types, six states, strata and errors):

| | 3 a.m. | 8 a.m. | noon | 5 p.m. | 8 p.m. | 11 p.m. |
|---|---|---|---|---|---|---|
| at home asleep | 0.93 | 0.17 | 0.04 | 0.03 | 0.11 | 0.78 |
| at home awake | 0.03 | 0.40 | 0.43 | 0.62 | 0.72 | 0.16 |
| at a workplace or school | 0.01 | 0.29 | 0.34 | 0.12 | 0.04 | 0.02 |
| elsewhere awake | 0.00 | 0.07 | 0.12 | 0.12 | 0.09 | 0.01 |
| elsewhere asleep | 0.03 | 0.01 | 0.00 | 0.00 | 0.00 | 0.02 |
| travelling | 0.00 | 0.07 | 0.06 | 0.10 | 0.04 | 0.01 |

On a Saturday the workplace share never passes `0.09` and "elsewhere awake" reaches `0.24`.

## The regime moved (S5, a reading)

| at home, 9 to 17, Monday to Thursday | 2015–2019 | 2022–2024 |
|---|---|---|
| all persons | 0.414 | 0.501 |
| the employed | 0.267 | 0.386 |

The weekly mean awake share did not move (`0.633`, `0.623`). A presence table needs its
years; the earlier one would misplace a tenth of the employed during the working day.

## Stakes

- **S3 (two shapes of day): neither met nor killed.** At-home-awake differs between
  Monday-to-Thursday and Saturday by more than `0.05` in 11 of 24 hours; the stake needed
  12 and the kill under 4. The difference is `−0.14` at 10 a.m. and `+0.11` at 6 a.m.
- **S4 (household size beyond who is in it): neither met nor killed.** Raw, people living
  alone are at home `0.115` more of the working day than people in households of three or
  more. After standardising for age, employment and day type the difference is `0.015`
  with a gap of `0.031`. Nearly all of the raw difference is who lives alone, not the
  household's size. The stake needed `0.03`; the kill under `0.01`.
- **S6a and S6b: untestable at this budget, as the null-year plant said before the read.**
  A cell's gap is under half the `0.03` budget in 6 of 288 cells (rows by age and
  employment predicting household-size strata) and in 4 of 96 (metropolitan against
  non-metropolitan). What can be said without a verdict:
  - Rows by age and employment miss a household-size stratum's at-home share by at most
    `0.081` in a cell; no cell is refused.
  - Metropolitan respondents are at home less than non-metropolitan ones by `0.015` on
    average and `0.012` from 9 to 17 on working days; 3 cells are refused outright (a
    difference of up to `0.073`). So one table for town and country is roughly right and
    not exactly right, and this sample cannot say more. 3,605 diaries are
    non-metropolitan.

## A household's presence

For one-person households the person's row is the household's, and is in the table as
measured. For larger households the table carries a bracket (at least one member's share,
at most the sum, with the value under independence between) and says it is a bracket.
The US energy survey's household-level reading (someone home on all five weekdays in
74.7 % of households, 2020) is the only direct one and is in `DEVICES0_RESULTS.md`.

## Gates, checks and what the method cost

- G0: each diary sums to 1,440 minutes; mean sleep 9.04 hours; weighted diary-days per day
  of week even within 1.5 %; shares sum to one.
- 9.1 % of sleep minutes sat in a segment with no located episode and are at home by
  default.
- The survey's replicate weights give errors `0.95` of the bootstrap's at the median
  (`0.88` to `1.04`), so the bootstrap is not understating; no inflation was applied.
- The files were fetched through a browser because the publisher refuses plain downloads.

## What this is not

- Not children, and not anyone outside the United States.
- Not whether a device is awake, charging or on a network.
- Not whether anyone is home in a household of two or more.
- Not a table by density class: the survey has no geography below metropolitan status.

## What follows from it

- The twin's awake share and working day should be replaced by this table, keyed on day
  type and hour, and on age and employment where it generates them.
- Household size can be left out of the presence key; the bracket carries it.
- A presence generator (a person's state by hour, drawn so that a tile's people sum to
  this table) is the next freeze for this field; the German time-use file, which diaries
  every member of a household, is what would turn the bracket into a measurement.
