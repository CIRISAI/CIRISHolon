# TIER-TREND-0 — adversarial review of the draft, 2026-11-23: REDESIGN

*An independent reader, given `TIER_TREND0_PREREG_DRAFT.md` at `b29c75a`, told to break it.
It wrote nothing in the repository and opened no trace directory; this is its report as the
lead's agent recorded it, shortened. Both tools admit the draft (`prereview_check.py`
PASSED, `prereg_audit.py` ADMITTED); neither can see what follows.*

**Verdict: REDESIGN. Both staked cells are fixed by banked arithmetic, and the share `O` is
not comparable across tiers.**

## 1. The three findings, in order

1. **The reasoning stake reads RISES for any autocorrelated block.** One static column
   predicted exactly and two organised columns with autocorrelation `φ`, with nothing above
   reading them, give `O = 2φ²/(1 + 2φ²)`: `0.12` at `φ = 0.3`, `0.28` at `0.5`, `0.53` at
   `0.8` (synthetic, checked), with a ceiling of `2/3` that is only the column count. RISES
   (`≥ 0.15`) fires at `φ ≳ 0.35` with zero carriage. The bank already fixes the cell near
   `0.36–0.41`. The stake cannot tell CARRIED from PERSISTENT. A real target needs a level
   above the chains, which the record does not have.
2. **`O` is not a unit.** The same system and the same split read `0.000` or `0.636` by
   the target; `0.010`, `0.505` or `0.991` by one target column's scale; `0.15` to `0.60`
   when a second noisy chart of the conserved variable is filed as organised; and, on the
   record's own seed 0, `0.00` static-first against about `0.79` counts-first
   (`REPLACE1_RESULTS.md`, lines 78 and 81 as read). No cross-tier comparison of `O` is
   meaningful.
3. **The calibration table breaks the draft's own definition.** The fluid's `0.031` uses
   one step of the count block, not the joint increment; jointly the three seeds read
   `−0.0025, −0.0048, +0.0003`, so `O = 0, 0, 0.003`. And the count block is, in the
   record's own words, "a second chart of the density mode": by the draft's rule it is
   STATIC, not organised. `OBJECT.md`'s tier table of 2026-11-23 carries the same misfiling.

## 2. The molecule stake is not a prediction

Intramolecular forces cannot change a molecule's own centre-of-mass momentum, so the floor
is near a theorem; the banked cross-block (`≤ 0.023`) puts the share at `≲ 0.005`. Both
rival readings predict LOW. It is an instrument check of the gate against the search.

## 3. The counterexample

The record supports "target, temperature and lag first, tier unmeasured": on one tier and
one dictionary the price moves `+0.0283 → +0.0002` with temperature and is `≤ 0` on the
density target; that within-tier range exceeds the whole between-tier difference
(`0.003` against `0.028 ± 0.014`). The rescue is to fix the target as the level above's
state, which the one high tier lacks. **The whole trend today is one number at 2.2 standard
errors** (the conscience block's price on the chains; the record's "one SE from its null"
is 2.2 by the numbers).

## 4. The better-posed claim, and the stake that would decide it

The organised block's increment on a target that contains none of its own columns, that
target being the level above's state, as a fraction of the target's predictable variance.
Read: fluid `0.000–0.003`; chains `0.028 ± 0.014`. Unread: the molecule (arithmetic says
`≤ 0.005`) and a task-level target for reasoning.

- **First:** a REPLICATION of the conscience block's price on chains never read. Kill: the
  price within one standard error of its permutation null on the fresh chains.
- **Then, on the lead's word only:** a task-level target from the trace directories. Kill:
  the share `≤ 0.05` at a standard error `≤ 0.02`.

Other fixes: bars derived and not chosen (`0.05` is the wrong number plus a margin); a
plant-failure branch separate from NOT COMPARABLE; a label-shuffle plant (moving tokens
across the static/organised label takes `0.028` to about `0.4`: M-TAG-AS-PROPERTY); the
QVM views admitted or excluded on a rule that also applies to reasoning.

Misfits contacted and not named: M-TAG-AS-PROPERTY, M-EMPTY-SECTOR, M-FLOOR-UNSTAKED,
M-MAX-OVER-SUCCESSES, M-UNTESTED-GAP, M-BASE-RATE-OMITTED, M-PRESENTATION-VERDICT,
M-KILL-DISSOLVED.

Not verified: anything in the release or the trace directories; whether chain targets are
standardised; whether unread scrubbed chains exist.
