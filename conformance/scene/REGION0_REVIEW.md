# REGION-0, the adversarial review of the draft (2026-11-12)

One reviewer subagent read the draft, the frozen parent and Natural Earth's two pages.
Nothing was downloaded. Verdict: freeze after named fixes; each is applied in
`REGION0_PREREG.md`.

Must-fix:

- **R1 was circular on codes and covered 14 of 50 labels.** The comparison went through
  the script's own table of postal codes. Now 50 points, one in each state, each with its
  code written as a number in the freeze, at the centre of its cell.
- **R2 cannot see a flip and the draft said it could.** Now each check says what it
  catches; the checks are read from the packed bytes as they will ship.
- **R2's reference and bars.** The shapes include inland water, so a correct grid reads
  high against land area. Bars are now −3 % to +6 %, Alaska −8 % to +10 %; the area
  formula is written out; a plant sums the globe.
- **The antimeridian.** A ring spanning over 180° of longitude or a vertex out of range
  now stops the run; a containment check R4 is added.
- **The fill rule and overlaps were unstated.** Now written: even-odd over the rings of
  one record; a point in two states goes to the lower code and is counted.

Should-fix, applied: points near cell edges (each is now a cell's centre); the sidecar's
warning about cells that are mostly another country or sea; the loss of small islands at
this scale; what was read where (version, size, address); the census division dropped
from this sidecar, since it is a second source.
