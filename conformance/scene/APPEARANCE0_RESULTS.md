# APPEARANCE-0 — results (2026-11-12)

Freeze `APPEARANCE0_PREREG.md` (committed alone at `ae50633`), review
`APPEARANCE0_REVIEW.md`, record before the parse `appearance0/BEFORE_THE_PARSE.md`,
reducer `appearance0/appearance0.py`, log `appearance0/run1.log`, numbers
`appearance0/results.json`. The archive was deleted after the run.

- **Branch (a): the three grids are written** to `tables/appearance0/`: a Köppen-Geiger
  class for each of the twin's 4,147,200 cells (1991 to 2020), the majority's share, and
  the count of classed 1 km pixels. 6 of 6 plants pass.
- **R1 met: 12 of 12 places** read the class the textbook gives (Singapore Af through
  Nuuk ET).
- **R2 met:** north of 60°S, 1,029,489 cells have a class (0.298; the bar was 0.26 to
  0.36). South of it, 0.569 (Antarctica is classed).
- **R3 met:** 0.521 of cells are classed between 30°N and 60°N against 0.054 between
  30°S and 60°S.
- **S1 met:** in 0.089 of classed cells the majority class holds under 0.8 of the cell
  (the bar was 0.15, a prior). It is highest in the temperate group (0.184) and the cold
  group (0.142), where relief and coasts mix classes; 0.041 of cells have a fifth or
  more of their pixels in another MAIN group. One class per cell is a fair reduction by
  this bar, and the share grid says where it is not.
- **By count of cells** (an equal-angle grid, so the poles weigh heavily): polar 0.369,
  cold 0.247, arid 0.183, tropical 0.118, temperate 0.083. 403 ties.
- **A hazard for a consumer:** 18,604 cells have a class from under 23 of 225 pixels.
  A class is not land; the count grid and the twin's own mask decide.

## What this does not license

- It is a key. Nothing about how a place looks follows until a table keyed by it is
  certified against something.
- The classes are a model's output from gridded climate for 1991 to 2020. They are not
  observations at a place and not a year.
- R1 cannot see a shift of one cell; the raster's tags and the plants carry that.
