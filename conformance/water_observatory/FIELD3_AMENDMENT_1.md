# FIELD-3 — AMENDMENT 1: the price node's ceiling in core-seconds, not wall-seconds

*Frozen 2026-09-05, committed alone, BEFORE the first node's record exists: at the time of
this commit `field3/solve.log` carries the launch line only and no node JSON has been
written (the git timestamp of this commit against the file times of `field3/*.json` is the
check). Written because the lead launched the harvest on 24 threads pinned to cores 0–23 —
to keep eight cores for the engine build the same freeze requires — while G-C0's ceiling
of `1800` wall-seconds was priced from EMBED-3's record on 32 threads. A wall-clock ceiling
that moves with the thread count is the M-PLACEMENT-LOTTERY shape: the gate would refuse a
solve for where it ran, not for what it cost.*

misfits: contacts **M-PLACEMENT-LOTTERY** (the wall clock is placement; the price is not);
**M-CHEAPER-THAN-ITS-PRICE** (the cost model is unchanged — EMBED-3's `14,500–26,150`
core-seconds per node at `1,002,001` determinants — and the too-cheap refusal stands at a
tenth of it); **M-STALE-INSTRUMENT** (this amendment alone).

## The change

- **G-C0 — the price (amended).** The 2.9 Å node's `cpu_seconds` (user + system, the
  runner's own reading from `/proc/self/stat`) is at most `57600` core-seconds — the frozen
  `1800` s × the 32 threads the ceiling was priced on — and at least `1450` (a tenth of the
  record's floor), else the harvest is REFUSED. `1,002,001` determinants (EXACT), the
  Davidson iteration count and the residual `≤ 1e-9` are unchanged. `wall_seconds` and
  `threads` are still recorded on every node.
  witness: none (a price, recorded)

Nothing else in the freeze moves. The admission flag is computed from `cpu_seconds` by the
`fit` phase, where the record is read; the `solve` phase's `admitted` field (computed on
wall time before this amendment) is superseded and reported beside it.
