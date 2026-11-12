# HOUSEHOLD-0 (United States) — results, 2026-11-11. Branch (a): the first scene certificate verdicts are issued. The twin's household rule is certified on households per leaf and refused on composition; density class turns out to be a weak key.

*Prereg `HOUSEHOLD0_PREREG.md` (frozen at `3297bff`), the record before the parse
`household0/BEFORE_THE_PARSE.md` (`bb937d8`), instrument `household0/household0.py`, log
`household0/run3.log`, numbers `household0/results.json`, table
`tables/household0_us/household_table.json` with its sidecar. 80,708 tracts and 230,657
block groups read; 129,208,869 households; 3,144 counties.*

## What was scored

The twin's own emitted histogram (twin/hhsize `b939214`, digest on record before any table
was opened): 1,000,020 households, mean exactly `5/2`, one-person share `0.1824`. Against
two budgets stated in advance: `0.12` persons on the mean (5 % of always-on nodes per
street) and `0.03` on the one-person share.

## The verdicts (S1): expectation stands

| class (tract key) | persons per household | verdict | one-person share | verdict |
|---|---|---|---|---|
| nation | 2.532 (twin 2.500) | **certified** | 0.286 (twin 0.182) | **refused** |
| village | 2.537 | certified | 0.255 | refused |
| suburban | 2.507 | certified | 0.285 | refused |
| terraced | 2.586 | unresolved | 0.300 | unresolved |
| apartment blocks | 2.520 | unresolved | 0.344 | unresolved |
| dense fabric | 2.321 | unresolved | 0.382 | unresolved |

- The twin has two fifths too few people living alone, everywhere it can be read. Its
  mean is right because it was set to the right number.
- The same verdicts come out under the block-group key.
- W1 between the twin's distribution and the reference is `0.39` to `0.53` persons. That
  one number would have hidden a pass and a failure inside it.
- The two densest classes are unresolved: their households sit in an effective 17 and 8
  counties. Their rows are readings.
- **A fault in the frozen floor, not repaired here.** Terraced is unresolved because the
  reference's own split-half gap (`0.023` in the share) exceeds half the budget. The twin's
  miss there is `0.117`, five times that gap. The rule as frozen withholds a refusal that
  is plain. A refusal needs only the miss to exceed the budget plus the gap; a certification
  needs the stricter floor. The verdict stays `unresolved` as the frozen rule gives it.

## Is density class a useful key (S2): neither met nor killed, and close to killed

568 county-classes of at least 20,000 households in the 25 held-out states, covering 69.6 %
of those states' households.

| rule offered | households in units within both budgets |
|---|---|
| one national rule (fitted on the other 26 states) | 17.2 % |
| a rule per density class (same fit) | 18.3 % |

- The difference is `0.011`; the stake needed `0.05` and the kill needed zero.
- By class: village `0.27` against `0.13` (the class rule helps); suburban `0.21` against
  `0.22`; terraced `0.12` against `0.14` (it does not).
- The larger reading: under EITHER rule more than four fifths of these households are in a
  county-class outside one of the budgets. Counties differ from each other by more than the
  budgets, and density class does not say how.

## The key transfers between census units (S2b): yes, in all three resolved classes

Tract key against block-group key: persons per household differs by at most `0.020`, the
one-person share by at most `0.005`. This compares two census units with each other. It
does not show that either matches a 1 km tile.

## Classes differ (S3): met, on composition only

Largest difference between resolved classes: `0.044` in the one-person share (bar `0.03`),
`0.079` in persons per household (bar `0.12`, not reached). The one-person share rises
steadily with density, `0.255` to `0.382`. Persons per household does not move in one
direction: `2.54, 2.51, 2.59, 2.52, 2.32`.

## What does carry household size (S4, readings, no bar)

| nationally | persons per household | one-person share |
|---|---|---|
| owner-occupied | 2.65 | 0.229 |
| renter-occupied | 2.32 | 0.393 |
| single house, detached or attached | 2.73 | not in these tables |
| 2 to 4 units | 2.28 | not in these tables |
| 5 or more units | 1.89 | not in these tables |
| mobile home | 2.54 | not in these tables |

- Building type moves the mean by `0.84` persons and tenure moves the one-person share by
  `0.16`. Density class moves them by `0.08` and `0.04` among the resolved classes.
- In the densest class the pattern reverses for houses: a single house there holds `3.10`
  persons and a flat in a large building `2.07`.

## The open top band: the review was right, and by more than it guessed

The residual mean of "7-or-more" is about `12` in each class (`10.3` to `13.8` nationally
when the household population is moved by ±1 %). A true mean near 8 is expected. So the
published household population exceeds what the household counts imply by about 2 %. Why
is not established here; the review's suggestion (different weights for persons and for
housing units) is unchecked. It entered no verdict. It does mean "persons per household"
has two values about `0.06` apart depending on which table it is taken from, which is half
the mean's budget.

## Gates, plants and what went wrong on the way

- All eleven plants passed before each run.
- **G0 stopped the run twice, both times on the author's reader.**
  - Run 1 (`run1_G0_failed.log`): the renter columns were sliced one place early, so the
    renter total was counted as a household size. The gate read 173.6 million households
    in the cells against 129.2 million in the total, and nothing further ran.
  - Run 2 (`run2_G0_failed.log`): a label check added after run 1 expected a label the
    author had guessed ("Renter occupied:" where the file says "Renter-occupied housing
    units:"). The layout was right; the expected string was corrected.
  - No class reading was printed by either failed run. The plants did not cover the column
    layout; the gate did. G0 now also requires each unit's cells to sum to its total.
- G0 as passed: households 129,208,869 from both tables; block groups sum to 129,209,336
  (not exactly additive, within the stated 0.1 %); national persons per household `2.528`.
- Two departures from the freeze were stated before the parse (`BEFORE_THE_PARSE.md`):
  block-group land area from the cartographic boundary file, and what P-TOP can check.
- Excluded and counted (tract key): 2,322 tracts over 20 % group quarters, 1,042 with no
  households, 320 with no land, 578 under the floor; 98.1 % of households kept. Group
  quarters hold 2.4 % of people. Connecticut is carried as its planning regions (the
  release has 3,144 counties in all).

## What this is and is not

- It is a verdict on the twin's rule as a description of US households pooled by the
  density of a census unit. A twin leaf has no census unit, so for the twin's own leaves
  these verdicts are outside the envelope.
- The twin's file is for leaves at exactly 2.5 people per household; real leaves round
  people and households separately and were not scored.
- Both budgets are the author's. A consumer with another tolerance reads the differences.
- One survey, one country, one window that includes 2020.

## What follows from it

- A better generator should key household size on building type and tenure before density,
  and needs something regional. That is a new rule to state and score, not a tuning of this
  one, and the SCORE states have now been read once.
- The certificate's error floor should be split into one for certifying and one for
  refusing before the next campaign uses it.
- The European campaign inherits the shape but not the blocks, the bands or the margins.

## Addendum, 2026-11-11: the twin's histogram over real leaves (as the twin session reports it)

Not scored here; recorded because it answers the caveat above. The twin emitted the same
histogram from the page's own path over every leaf under twelve places (twin/hhsize
`165e0bc`, `docs/geo/household_hist_real.json`, digest `5526d2d6…`, recomputed here).

- 356,507 households in 16,041 leaves: mean `2.500001`, one-person share `0.1806`. Real
  leaves sit close to 2.5 people per household (2.44 to 2.60), so the synthetic file that
  was scored stands for the page. The verdicts above are not changed by it.
- 98 % of those households are in dense-fabric leaves. The US reference resolves village,
  suburban and terraced. The two barely overlap, which is a second reason the verdicts are
  outside the envelope for the twin's own leaves.
- **By units per building the twin's rule is flat:** `2.45` in single units, `2.51` in 2 to
  4, `2.51` in 5 or more. The census moves `0.84` across the same bands. That is the
  plainest statement of what the next generator has to add.
- Two cautions from the twin's code for such a generator: in dense fabric a building's unit
  count is a by-product of plot count, not yet a building type; and the apportionment
  rescales every drawn size after the draw, so a per-type shape survives only up to that
  rescaling. Persons should be apportioned by type within the leaf, not drawn and rescaled.
