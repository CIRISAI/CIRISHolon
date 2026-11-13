# REGION-0 — results (2026-11-12)

Freeze `REGION0_PREREG.md`, review `REGION0_REVIEW.md`, record before the parse
`region0/BEFORE_THE_PARSE.md`, rasteriser `region0/region0.py`, log `region0/run1.log`,
numbers `region0/results.json`. The archive was deleted after the run.

- **Branch (a): the three grids are written** to `tables/region0/`: a US state code for
  each of the twin's cells, the winner's share, and the count of sample points in any
  state. 8 of 8 plants pass. Natural Earth version 5.1.1; its federal-code column agrees
  with the script's table for each of the 51 US shapes.
- **R1 met: 50 of 50** labelled points, one in each state, carry the code written in the
  freeze, read from the packed bytes.
- **R2 met: 10 of 10** areas inside their bars. Each reads a little high against land
  area, as the review expected of outlines that include inland water: Alaska +0.68 %,
  Texas +1.40 %, California +1.46 %, down to Arizona +0.26 %.
- **R3 met:** each of the 50 states has a cell (Rhode Island 21, Delaware 41). The
  District of Columbia has 1.
- **R4 met:** no coded cell outside the box.
- **Borders:** 73,425 cells carry a code; 67,383 lie wholly in one state; in 1.4 % the
  winner holds under 0.8 of the points in any state; 2,064 have under 13 of 25 points in
  any state (coast, or the Canadian and Mexican borders).
- **Missing at this map scale:** no Alaskan cell lies east of 172°E, so the westernmost
  Aleutians are absent; small islands and keys are dropped.

## What this does not license

A state is a key. A code is not land and not "mostly this state": the count grid and the
consumer's own masks decide. Nothing about a state's dwellings follows.
