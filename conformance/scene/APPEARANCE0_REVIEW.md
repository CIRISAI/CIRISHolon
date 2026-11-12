# APPEARANCE-0, the adversarial review of the draft (2026-11-12)

One reviewer subagent read the draft and the repository record's metadata. No archive was
downloaded. Verdict: freeze after named fixes; each is applied in `APPEARANCE0_PREREG.md`.

Must-fix:

- R1's places had no coordinates, so "the cell containing each place" could be chosen
  after the read; three allowed sets were too tight (Yakutsk, Sydney, Phoenix).
  Coordinates and sets are now fixed; a cell of class 0 is a miss.
- R1 cannot see a shift of one cell or a wrong legend offset beyond 4 of 12. Added: the
  raster's size and corner must nest 15 by 15 pixels in a cell; the legend must hold 30
  codes with 5 named symbols.
- R2's lower edge (0.25) sat on a plausible outcome if Antarctica is masked. Now counted
  north of 60°S alone, with a legend-free orientation stake R3 (north against south).
- A coastal cell with 1 land pixel in 225 carried a class at full share. A third grid of
  classed-pixel counts is added and the sidecar says a class is not land.
- The coarser-raster fallback left a choice open after the read. Deleted: if the 1 km
  raster cannot be decoded, branch (e).
- `holds_no_real_feature` was asserted without the twin's word. The twin's lead gave the
  grid's order, the key name and the zero convention on 2026-11-12; cited.

Should-fix, applied: the full md5 and the record's January 2026 correction; S1 was
vacuous ("under half" needs three classes in one cell) and is now "under 0.8" with bars
that are priors; period in the file names; deterministic gzip and both digests; branch
(e) and (r) widened; limits stated.
