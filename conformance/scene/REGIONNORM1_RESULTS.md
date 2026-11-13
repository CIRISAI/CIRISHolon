# REGIONNORM-1 — results (2026-11-12): region carries a dwelling's make-up better than a climate zone does, and a state is its own thing

Freeze `REGIONNORM1_PREREG.md` (committed alone at `28a2bf1`), its review
`BUILDGEN2_REVIEW.md`, script `regionnorm1/regionnorm1.py`, plants 5 of 5
(`regionnorm1/plants.log`), numbers `regionnorm1/results.json`, table
`tables/regionnorm1/`. The file carried the recorded digest and was deleted after the run.
9,238 of 18,496 households were read (the named half); the other half was not parsed.

## The readings

- **L1 MET: the census division predicts a state better than its climate zone.** With
  pools equalised in donor states and households, the division is nearer in 0.678 of 152
  state-table pairs (the bar was 0.60), and 23 states favour the division against 7 for
  the zone (sign test p = 0.003). By table: heating fuel by type 0.74, type given urban
  type 0.71, wall material by era 0.68, roof material 0.58. Alaska, North Dakota, Oregon
  and Washington had too few donors in one pool and were not scored.
- **L1b NEITHER: the division against states of like degree days.** The division is
  nearer in 0.619 of 168 pairs and 21 states to 11, a sign test of p = 0.055 against a
  bar of 0.05. A sharper climate key than eight zones closes most of the gap. This is
  the honest edge of the result: region beats the survey's climate ZONES; it is not
  shown to beat climate measured finely.
- **L2 MET: a state differs from the rest of its division by more than chance.** 0.524
  of 168 pairs reject at 5 % (the bar was 0.50; 0.05 would do so by chance). It is the
  materials and the fuel that are a state's own: wall material by era 0.74, heating fuel
  by type 0.69, roof material 0.48, type given urban type 0.19. The median distance
  above its permutation mean is 0.049.
- **L3 met: two of three dependences differ by division.** Among detached houses the
  rank correlation of vintage with storeys runs from −0.18 in one division to +0.28 in
  another (p = 0.001): in some regions newer houses are taller, in others lower. Floor
  area with total energy runs 0.29 to 0.54 (p = 0.001). Floor area with electricity does
  not differ (0.28 to 0.43, p = 0.38).
- **The four pools on one footing (printed):** the division is the nearest of the four
  in 0.54 of pairs, like climate in 0.27, the zone in 0.14, the nation in 0.05. Median
  distances: division 0.149, like climate 0.181, zone 0.193, nation 0.247.
- **Who is nearest and farthest from its division, heating fuel by type:** Ohio 0.116,
  Tennessee 0.119, Texas 0.120; Arizona 0.416, Florida 0.441, the District of Columbia
  0.643, Hawaii 0.857.

## What it says, in the terms the lead used

"Regional norms" are visible in this survey as a key: knowing a dwelling's census
division tells more about its fuel, its walls and its type than knowing its climate
zone, and knowing its state tells more again for fuel and walls. Even the way a
dwelling's attributes hang together (which era built tall) differs by region. BUILDGEN-1
was killed for lacking exactly this key.

## What it does not say

- Not a cause. A division is contiguous geography: climate, the gas grid, prices and the
  age of the stock ride on it together. L1b shows fine climate explains much of it.
- Not prediction of unseen states in the strict sense: these columns were read before on
  a split by state, and the stakes were fixed after that.
- Nothing about energy means, which were not compared here, and nothing outside the
  United States or 2020.

## What a generator takes from it

A pooling ladder with the state first where it has the households, then the division,
then like climate or the zone, then the nation; and correlation matrices that are
allowed to differ by division for the pairs shown to differ. That generator is its own
campaign (the remaining must-fixes of `BUILDGEN2_REVIEW.md`), not started.
