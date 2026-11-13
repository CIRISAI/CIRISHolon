# REGIONNORM-1 — at what level is a dwelling's make-up shared: the state, its census division, its climate zone, states of like climate, or the nation? A reading of the 2020 household energy survey. PREREGISTRATION

*Frozen 2026-11-12, committed alone. Its design is the L1 and L2 of
`BUILDGEN2_PREREG_DRAFT.md` with the must-fixes of that draft's adversarial review
(`BUILDGEN2_REVIEW.md`, findings 1, 2, 5, 10, 11) applied in the reviewer's wording; that
review is this text's review. Nothing has been downloaded for it.*

## Why, and what may be claimed

BUILDGEN-1 was killed because a dwelling's joint differs between states within a climate
zone. Before a generator is keyed by anything, the survey is asked which key carries the
most. The lead's phrase was "regional norms". What this can show is narrower: whether
the survey's census division predicts a state's tables better than its climate zone
does. A division is contiguous geography; it stands in for climate, the gas grid, prices
and the age of the stock at once. A result here names a better KEY, not a cause.

This file's columns were read on both halves of BUILDGEN-1's state split. Nothing here is
"states never seen". The comparisons are made by leaving each state out in turn.

## Carrier

The 2020 household energy survey's public microdata by its recorded digest
(`8d5f6d31…3592`); public domain as the agency states; not committed; deleted after.

## Households used

Only FIT households: those for which the low bit of the first byte of SHA-256 of the
ASCII string `<DOEID>:20261113` is 0. The other half is not parsed, and is left for a
later generator's scoring. Each statistic is a ratio of weighted sums.

## Known before the read

BUILDGEN-1's results in full (zone-level shares on its two halves of states; pooled and
within-type correlations). By this program: no state's own table and no division's. The
number of households in each state is read in the gate.

## Definitions fixed here

- **Tables, 4:** heating fuel (7) by type (5); wall material (8) by era (3); roof
  material (7), dwellings asked; type (5) given urban type (3), compared as the mixture
  weighted by the STATE's own urban shares.
- **A state's climate zone** is the zone holding most of its weighted FIT households.
- **Donor pools for a state,** each of other states alone:
  - its division (the survey's 10);
  - its zone (states whose zone, as defined, is the same);
  - like climate: the states nearest to it in (heating, cooling) degree days, each
    standardised over states, as many as the comparison needs;
  - the nation.
- **Equal pools.** For a comparison of two pools, m is the smaller count of donor states
  and h the smallest count of FIT households among the donors of both; each prediction
  is the weighted table of h households drawn without replacement from each of m donor
  states drawn without replacement, and the distance is averaged over 200 draws. A state
  with fewer than 2 donors in either pool is not scored on that comparison and is listed.
- **Distance:** total variation between two weighted tables.
- **States scored:** those with at least 100 FIT households.

## Plants (synthetic countries through the same code)

- **P-NULL.** No difference between states at all, pools of unequal size: L1 reads the
  division nearer in 45 % to 55 % of pairs, and L2 rejects in at most 10 %.
- **P-DIV.** Divisions differ and zones do not: L1 reads met.
- **P-ZONE.** Zones differ and divisions do not: L1 reads its kill.
- **P-STATE.** Each state differs from its division by a planted distance of 0.10 at 300
  households: L2 reads met.
- **P-HASH.** The split gives 45 % to 55 % FIT, the same on a second run, and is not
  the odd-or-even alternation of the identifier.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier by construction: each synthetic country holds what its plant names.

## Stakes

- **L1 (division against climate zone).** Over the scored state-table pairs, the share in
  which the division's prediction is nearer than the zone's; and a sign test over states
  (a state counts for the division if most of its 4 tables do; ties are dropped).
  - Met: the share is at least `0.60` AND the sign test favours the division at 5 %,
    one-sided.
  - *Kill:* the share is at most `0.40` AND the sign test favours the zone at 5 %. Then
    the survey's climate zone carries more than its division, and a generator's ladder
    puts zone above division.
  - Otherwise neither: the two are not told apart, and that is the result.
- **L1b (division against like climate), the same form.** The rival is the degree-day
  neighbours, a sharper climate key than 8 zones. Met, kill and neither as L1.
- **L2 (is a state its own thing?).** For each state-table pair, the weighted distance
  from the state's table to the rest of its division's, against 999 permutations of
  state labels among the division's FIT households, weights carried with the household.
  - Met: at least 50 % of pairs reject at 5 %.
  - *Kill:* under 15 % reject AND the median of (observed less the permutation mean) is
    under `0.03`. Then a division is enough and a generator need not key by state.
  - Otherwise neither. 5 % of pairs reject by chance, and a state's 4 tables are not
    independent; the count is read with that said.
- **L3 (do the dependences differ by division?), detached houses, 3 pairs:** floor area
  with total energy; vintage with storeys; floor area with electricity. The statistic is
  the weighted variance over divisions of the mid-rank Spearman; division labels are
  permuted 999 times. Met: at least 2 of 3 reject at 5 %. No kill.
- **Printed, no verdict.** For each state: its households, zone, division, and its
  distance to each pool for each table; the median distance by pool; the share won by
  each of the 4 pools; each division's 3 correlations; the state with the largest and
  smallest distance to its division for heating fuel; and, as an upper reference, states
  grouped into 10 by average-linkage clustering of their fuel-by-type tables, with the
  median distance of a state to its group.

## What would make this vacuous

- 51 areas, one survey design, one year (2020). Halves of one design share its clusters.
- L1 can only compare the keys the survey carries. A better region may exist.
- 4 tables and 3 correlations stand for "a dwelling's make-up". Energy means are not
  compared here.
- With about 160 dependent pairs a share has a standard error of 0.05 to 0.08; that is
  why L1 needs the sign test too, and why "neither" is a likely and honest outcome.

## Branches

Exactly one fires.

- **(e)** The digest differs; or a plant fails. Nothing is written.
- **(a)** Otherwise. `tables/regionnorm1/` holds each state's distances and the
  readings, and the result states L1, L1b, L2 and L3 as read. What a later generator's
  ladder takes from each: L1 met or neither, division above zone; L1 kill, zone above
  division; L2 met or neither, the state first; L2 kill, the division first.

---
witness: none (a reading of a survey by leave-one-state-out and permutation)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-POPULATION-CHOICE, M-PROVENANCE-OVERREACH, M-VACUOUS-SUCCESS. These are the registered ids this text contacts.

- M-PLANT-OBS: each plant runs through the code that reads the survey.
- M-BAR-FROM-THE-READ: bars stated here; what BUILDGEN-1 read is listed and the claims are limited to match.
- M-POPULATION-CHOICE: FIT households by a named hash; states with at least 100; pools equalised in donors and households.
- M-PROVENANCE-OVERREACH: the result names a key, not a cause; country and year in each sentence that uses it.
- M-VACUOUS-SUCCESS: §"What would make this vacuous".
