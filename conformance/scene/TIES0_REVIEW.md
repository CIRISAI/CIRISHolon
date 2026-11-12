# TIES-0 — adversarial review of the draft, 2026-11-11: REDESIGN the scored part

*A second reader attacked `TIES0_PREREG_DRAFT.md`. No data fetched or opened. One
documentation page read (the index's methodology note). Synthetic scripts, single-threaded
(`ties0/review_synthetic_centroid.py`, `_nested.py`). The author's condensation; the draft
is kept as written and marked.*

**Verdict.** The carrier stands. The bands, the strata, three of the stakes and the label
on the sub-kilometre number do not.

1. **The bands and strata did not fit the instrument.** The twin's density bounds are for
   1 km tiles. At an area's average density the two densest bands are empty in Germany and
   hold about five US counties. And the distance bands read the size of the unit, not
   anyone's ties: with one law for individuals and only the unit's side changed from 22 to
   110 km, the share "within 50 km" by centre points moved by `0.08` (the budget was
   `0.05`) and the fitted exponent by up to `0.28` (the bar was `0.3`).
2. **The assumption's error is the size of the budget, and the own-area convention was
   unread.** If users per resident are 1.3 times the average in an area, an own share of
   `0.40` reads `0.46`. The methodology note does not say whether a link within an area
   counts once or twice: a factor of two on the own term, which the draft's gate could not
   see. The release's nested levels (regions inside regions) make both testable: the
   convention cleanly, and one parameter of "users are not proportional to residents".
3. **The hold-out was not one.** The index is symmetric, so a held-out area's ties to
   fitted areas are numbers the fit read; and the share within 50 km is mostly the own
   share, which the draft's law took as a free number per band. **Fix:** fit a kernel on
   off-diagonal pairs only and predict each area's OWN share by integrating it over the
   area. No diagonal is ever fitted. It is also the consumer's own step, carrying a law
   below the scale it was fitted on, taken one scale up, where it can be checked.

**Also.**
- **Shares are not identified without the assumption; the exponent is.** Fit it on the
  index directly, on pairs far enough apart that unit size does not matter.
- **The abroad share from the country file** multiplies an index by populations where the
  platform's reach differs by multiples. Use foreign Europe from the same file; beyond
  Europe is not identified.
- **The sub-kilometre number** may be printed only as a bracket. Of the "two studies",
  one is at city grain; only one reaches below a mile, and it finds the curve flatter
  there. Its licence is the result of the kernel stake one scale up.
- **Ties by kind:** nothing is identified. A friend list is an accumulation over a life.
  It is nearest to extended kin and to the pool interest groups draw on; it says nothing
  for the household term that carries 0.65 of the twin's headline, and its stale ties
  reach farther than active contact does.
- **The twin's locality is about 600 km across;** the comparison band is 300 km, not 200.
- **No gap or verdict rule was named.**
- **Plants:** two recovered by construction and two could not fail.

**What the author took.** The "smallest honest version": Germany and Europe only, from the
14 MB file. The US county file is left for a later freeze. A second draft
(`TIES0_PREREG_DRAFT2.md`) goes back to the same reader before any freeze.
