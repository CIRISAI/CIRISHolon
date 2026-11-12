# PRESENCEGEN-1 — adversarial review of the draft, 2026-11-12: freeze only after named fixes

*A second reader attacked `PRESENCEGEN1_PREREG_DRAFT.md`. No data fetched or opened. Two
synthetic scripts, single-threaded (`presencegen1/review_synthetic_1.py`, `_2.py`). The
author's condensation; the draft is kept as written.*

**Verdict.** The generators can stand; the stakes could not.

1. **No stake tested what a chain gets wrong.** On synthetic days a first-order chain
   reproduced the share who stay home (`0.247` against `0.247`) and the MEAN spell away
   (`5.12` against `5.13` hours): the mean is hours in a state over entries into it, and
   both are what a chain is fitted on. So the duration stake was met by construction and
   its kill could not fire; and the "stays home" stake was a parameter of the generator
   meant to win it. What a chain does get wrong is the DISTRIBUTION of spell lengths
   (spells of two hours or less: `0.425` against `0.355`) and the joint of hours far apart
   (home at 9, 14 and 18: `0.455` against `0.396`). No stake read either.
2. **The state at the start of an hour is not PRESENCE-0's quantity.** Diary times sit on
   the hour and half hour and a trip ends on the mark. With 80 % of times so placed,
   travel was a fifth low at hour-starts. A seeded random minute in each hour is unbiased.
3. **The rows were thin, the fallback crossed day types, and the call's key is not what
   the chain upstream emits.** Over half the transition rows would fall back, and they are
   the short states; borrowing Saturday's destinations after a Monday commute breaks the
   margins. And nothing upstream emits "employed" or splits 15–24 from 25–64.

**Also.**
- **Three kinds miss when a working day starts**, which is what breaks a chain: a late
  arrival meets the five o'clock leaving rate.
- **A design that carries durations without shipping anyone's day:** draw the kind, then
  the hour first left and the hour last home from one aggregated table, then a short
  chain between them. The frame carries staying home, hours away and who is home in the
  evening; only the fill is approximate.
- **Fair rival:** a chain with one extra bit, "has left home today".
- **Pool transitions over day types within a kind;** let day type set only the share of
  each kind. Whether that holds is itself a stake with an unknown outcome.
- **A hold-out that tests the design:** within-kind tables from 2015–2019 with kind shares
  from 2022–2023, scored on 2024. If it passes, the move home after 2020 is one knob.
- **The household:** independence cannot be certified; print its floor and ceiling.
- **A person's week is not identified** (one diary day per person): a declared knob.
- **Missing from the return:** place changes a day; the kind; a fallback flag;
  employment by age so the chain from the household generator closes.

**What the author took.** The "smallest honest version": two generators (the chain with
the has-left bit; the frame-then-fill), seeded-minute sampling, tables pooled over day
types within kind, stakes on what no generator holds as a parameter.
