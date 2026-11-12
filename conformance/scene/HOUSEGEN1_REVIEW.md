# HOUSEGEN-1 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `HOUSEGEN1_PREREG_DRAFT.md`. No data was fetched or opened.
Synthetic checks only (`housegen1/review_synthetic_1.py`, `_2.py`), single-threaded, with
two made-up rows at leaves of 15, 30, 60, 120 and 300 people, none of them the draft's
sizes. The author's condensation; the draft is kept as written.*

**Verdict.** Three faults would have been frozen.

1. **The draw is biased.** Truncating the last household always adds one small household,
   about 0.4 to 0.5 households per leaf whatever its size. On a large-household row the
   mean is low by 7.8 % at 15 people, 4.1 % at 30, 2.1 % at 60. The draft's expectation
   ("inside both budgets from 20 people") would probably fail. **Fix:** when the drawn size
   `s` exceeds the `r` people left, keep a household of `r` with probability `r/s`;
   otherwise the `r` people join an earlier household. Bias falls by about three; the cost
   is a household above the top band in about 0.1 % of households. Drawing the household
   count first gives an exact mean but the reviewer's repair put the error into the shape
   and removed nearly all variation between leaves; not shipped.
2. **Borrowing a neighbour's row is worse than the class's own reading.** Germany's two
   densest classes are read from a full count; they are unresolved because too few 50 km
   blocks hold such tiles, not because the reading is noisy. Borrowing terraced for dense
   fabric misses by about 15 % on the mean and 0.16 on the one-person share. **Fix:** use
   the class's own reading, return `unresolved`, reason "read, not certified".
3. **The household count conflicts with the twin's.** The twin apportions households down
   its tree as people over 2.5; a random count per call no longer sums from child to
   parent. **Fix:** settle with the twin before the freeze. (Settled: the twin carries
   people as the exact quantity and takes households from the call; an expected count is
   also exposed.)

**Also.**
- **S1 composes two things**, the table against the held-out half and the draw against its
  own row. Report them apart; certify only if both pass.
- **150 people is the best case.** Score at the sizes the consumer calls with.
- **The key.** The call took a region; the rows are by country. The US rows are keyed by
  tract, which a leaf does not have: US calls are outside the envelope.
- **One verdict for two consumers** cannot say what the reference campaigns found. Return
  two.
- **The top band:** make the mean verdict the worst of the band at one less, as declared,
  one more.
- **Shipping the all-data table** is acceptable if labelled: it is a mixture of the two
  halves' rows. Gate: each shipped share lies between the two halves'.
- **Missing for a consumer, to be said in the sidecar:** who is home; adults against
  children; which households share a building; vacant dwellings; communal residents, who
  are in the people passed and become household members here.
- **Two stakes with unknown outcomes:** per tile (the grain the twin reads), and the spread
  between tiles against the generator's.
- **Plants:** the draft's tested the law, not the bias. S3's `0.01` was close to the
  draw's own bias.

**What the author took and did not take.** Each fix is applied in `HOUSEGEN1_PREREG.md`.
The "smallest honest version" is taken whole: Germany only is certified; the US rows ship
`unresolved` from the published table with no new fetch. The carrier is the held-out
tiles' own people, not a twin histogram, because the twin now calls once per tile.
