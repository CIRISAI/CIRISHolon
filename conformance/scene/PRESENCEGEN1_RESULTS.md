# PRESENCEGEN-1 — results, 2026-11-12. Branch (e): the gate failed and nothing ships. The frame's days are awake 0.645 of the time where the diaries are awake 0.623; the chain's are 0.623. The stakes were not read and 2024 was not opened.

*Prereg `PRESENCEGEN1_PREREG.md` with its CORRECTION of 2026-11-12, the record before the
parse `presencegen1/BEFORE_THE_PARSE.md`, generators
`engine/crates/holon-scene/src/presence.rs`, fitting and scoring
`presencegen1/presencegen1.py`, log `presencegen1/run1.log`, numbers
`presencegen1/results.json` and `presencegen1/diagnosis.json`. 16,684 diaries of 2022 and
2023. One run. The script stops at a failed gate before it scores, so no stake on spells,
far-apart hours, Saturday or the move home was computed.*

## Before the parse: a plant failed and the freeze was corrected

P-LATE failed on the generators as frozen: a two-hour cell for the hours away straddles
the stake's eight-hour line (frame `0.445` against `0.539` planted). The hours away went
to one-hour cells in a dated CORRECTION committed alone, and the plant then passed
(`0.533`). No diary had been read.

## The gate (G0)

| | budget | read | |
|---|---|---|---|
| state shares, seeded minute against all minutes, largest difference | 0.01 | 0.0002 | pass |
| travel read at the hour's start, over travel by minutes | printed | 1.21 | the bias the seeded minute removes |
| awake share, diaries | | 0.6230 | |
| awake share, chain (G1) | within 0.01 | 0.6229 | pass |
| awake share, frame (G3) | within 0.01 | 0.6451 | **fail, by 0.022** |

The sampling half of the gate passes: a seeded minute per hour gives the minute-level
shares to the fourth decimal, and reading at the hour's start would have overstated
travel by a fifth. The generator half fails for the frame. By the freeze that is branch
(e): nothing ships, the chain included.

## Why the frame is too awake (diagnosis on 2022 and 2023, after the verdict)

The frame gets the hours at home right and splits them wrongly: home asleep is `0.019`
low on the mean of the day and home awake `0.013` high. By hour the miss is largest from
6 to 9 in the morning (asleep `0.045` low) and from 21 to midnight (`0.033` low).

- **One chain for everyone at home is the main fault.** The waking rate by hour was
  fitted over all people at home, which includes those about to leave, who wake early.
  The generator then applies that rate to the people who stay, who in the diaries sleep
  later. It is a selection the design did not hold: the chain at home has to know whether
  the person leaves today and whether they have already come back.
- **Coming home asleep.** In `0.107` of days with a return, the person is asleep at the
  first hour read after the last time away. The generator sets them awake.
- **Sleeping at home between outings.** `0.032` of days have it; `0.005` of all
  person-hours. The fill within the frame had no such state and wrote it as awake.
- **Asleep elsewhere at 4 in the morning** is `0.049` low in the first hour and high at
  midday: the first state within a frame is not keyed on the frame starting at the day's
  first hour.

## What stands

- A seeded minute per hour is a fair reading of a diary; the hour's start is not.
- The chain with one bit reproduces the awake share. It is not shipped: the freeze says
  nothing ships under (e), and its stakes were not read.
- The twin's reading of presence stays PRESENCE-0's tables. No generator of a day exists.

## What follows

A successor, frozen before 2024 is opened, with the fill at home keyed on its place in
the day (no outing; before; after), a drawn state on return, home asleep within the
frame, and the frame's first state keyed on starting at 4. Stakes, budgets and kills as
frozen here. The gate's numbers and this diagnosis on the fit years are what the
successor has seen.
