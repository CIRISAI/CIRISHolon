# PRESENCE-0 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `PRESENCE0_PREREG_DRAFT.md`. No data fetched or opened. One
single-threaded synthetic script (`presence0/review_synthetic.py`). Sizes it marks as
memory are its recollection of published figures, not read that day. The author's
condensation; the draft is kept as written.*

**Verdict.** Three faults would have been frozen.

1. **The main stake returned "unresolved" by arithmetic.** One weekday's hour has roughly
   700 scored and 1,500 fitted diaries. With no change between years and a budget of
   `0.03`, a cell with a share near `0.35` certified in 0 % of synthetic trials and a
   refusal never fired. "Certified in 150 of 168 hours" was unreachable with nothing
   wrong. The budget was below the instrument's resolution at that grain. **Fix:** verdicts
   on four day types by clock hour (96 cells), strata printed without verdicts where the
   gap is too wide, and a null-year plant that prints the ceiling the bar must sit under.
2. **The five states could not give an awake share, and the sleep rule misplaced sleep.**
   Sleep away from home was folded into "elsewhere". Grooming is also unlocated, and most
   diaries open with sleep at 4 a.m.: a morning of sleep, grooming and a drive has "car"
   as its next located episode, so the drafted rule files that night as away. **Fix:** a
   sixth state (elsewhere asleep) and a segment rule: cut the day at travel; an unlocated
   episode takes the place of the located episodes in its own travel-free segment; a
   segment with none is home by default, and that share is printed.
3. **"Most of its minutes" is the wrong operator for a table of shares.** It deletes short
   states: synthetic travel was `0.045` of person-time by minutes and `0.030` by the hour's
   largest state. **Fix:** average the minutes. Assigning one state per hour is a
   generator's job.

**Also.**
- **Person rows are offered where a consumer needs household rows.** One respondent per
  household gives no joint. For one-person households the person's row is the
  household's. For others only a bracket exists. Phones on home Wi-Fi need person rows;
  an always-on node needs neither.
- **The household-size stake was mostly age and employment** (people living alone skew
  retired). **Fix:** standardise to the all-persons mix of age, employment and day type
  first.
- **A hold-out by year tests sampling noise.** What tests something: rows by age and
  employment predicting each household-size stratum; and a table from metropolitan
  respondents scoring non-metropolitan ones, which is the assumption the twin leans on.
- **The number the energy delta reads was not in the draft:** weekly mean awake hours
  against the twin's `0.85` (wake-ups are linear in it). Expected near `0.63` (memory).
- **The Wi-Fi proxy needs a denominator and a bracket**, or it will look like a
  confirmation of `0.65` by accident.
- **The error:** the weighted bootstrap carries the main design effect; run the survey's
  replicate weights on the all-persons table and print the ratio.
- **Cut diaries on the 24-hour duration fields** or one diary covers more than 1,440
  minutes.
- **Screens (S8):** no consumer reads it; dropped.
- **The budget was by feel.** Price it: 5 % relative on the weekly mean awake share.

**What the author took and did not take.** Each fix is applied in `PRESENCE0_PREREG.md`.
The 168-hour table is still printed, without verdicts. Both extra files the review names
(metropolitan status; replicate weights) are read, within the lead's approval.
