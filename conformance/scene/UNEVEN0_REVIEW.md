# UNEVEN-0 — adversarial review of the draft, 2026-11-11: freeze only after named fixes

*A second reader attacked `UNEVEN0_PREREG_DRAFT.md`. No data was fetched; one single-threaded
synthetic script checked the grid's index arithmetic and the plant values. The author's
condensation; the draft is kept as written.*

**Verdict.** Right to run first, and cheap. Three things must change before a freeze.

1. **Whether an empty pixel is "village" was undefined, and it decides S1.** The twin's
   village class starts at 0 and its floor is stated for cells only. If empty pixels count
   as village, a village cell is nearly all "same class" and the stake fails; if empty is
   its own class, the same cell reads near zero. On the synthetic town the reading is
   `0.960` one way and `0.000` the other. **Fix:** six pixel classes, empty first; stake S1
   on the suburban class only and report the village class.
2. **"Exactly 225 pixels per cell" is false at the grid's edges.** The file's origin puts
   pixel edges 0.05 pixel off cell edges. Under the twin's rule (a pixel goes whole to the
   cell holding its centre) interior cells do hold 225, but the grid is 2 columns wider than
   2,880 × 15, so the first and last cell columns hold 240, and two cell rows hold 180. The
   date-line columns include populated places. **Fix:** state the assignment rule and those
   counts, print the people in the two extra columns, and add a plant for the convention.
3. **G0 gated nothing.** Its first half was true by definition and its second passed with
   nothing to spare. **Fix:** three gates from what the twin ships: the world total
   (`7,840,952,947.01`); every half-degree total against the twin's float values to `10⁻⁶`;
   and the campaign's cell sums re-encoded by the twin's own formula reproducing its shipped
   codes in all but a handful of cells. With plants for a one-pixel shift and for a
   north-south flip, which the world total alone cannot see.

**Also.**
- The better deliverable is the full TRANSITION TABLE: for each cell class, the share of
  tiles and of people in each of the six pixel classes, at 1, 2 and 4 pixel grain. It is the
  mixture any cell-keyed reference would average over.
- S3 (unevenness falling with density) is partly forced arithmetic: a cell can only average
  12,000 per km² if most of its pixels are full. Report it; do not read it as a finding.
- Coastal cells read as uneven because sea pixels are zero under the whole-area rule, and
  the file carries no land mask.
- The reviewer advises AGAINST a pilot: any block of the same grid makes the rest a
  replication. State the bars as the author's priors with their reasoning.
- "The first scene certificate with a reference in hand" was not honest: there is no budget,
  consumer or envelope, and the "generator" was never fitted. It is a reading of the
  reference's own structure within a cell.
- Branch (a) overclaimed: the stakes show the cell's class mislabels tiles, not that the
  tile's own class is a sufficient key.
- Back-of-envelope expectation for a suburban cell: about a fifth of tiles and a quarter of
  people in the cell's own class, two thirds of people in a higher one. So the stakes are
  expected to be met by a wide margin, which makes this, too, close to a confirmation; its
  value is the table of numbers, not the verdict.
