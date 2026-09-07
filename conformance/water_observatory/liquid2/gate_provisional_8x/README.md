# The gate phase, run at a PROVISIONAL step — kept, and marked

These records are a complete, passing gate phase, and they are in a subdirectory because the
step they were taken at is **NOT pre-committed by the draft freeze**.

The draft's step rule is the one it was written with: *the largest step whose measured drift
stays under L1's bar*. The sweep measured that **no step clears that bar** — not because of the
step but because serving channel 6 as CT-3's table raises the drift by three orders of
magnitude against two single-knob controls (`DRIFT_NOTE.md`). An earlier draft amended the rule
to "the largest step whose drift is under twice the 1x arm's", which selected 8x; that
amendment is WITHDRAWN on the lead's ruling that the drift is an instrument defect to be fixed
before any freeze, not a band to be widened around.

So: the step is UNCHOSEN, and everything in this directory that depends on it — the R3 window
(stride 246, max_lag 140, 137,760 counted frames, 28.72 ps), the campaign price
(`104,835.1` s), and the settling length (`11,850` frames) — is a reading at a provisional
`8x` and not a commitment. What does NOT depend on the step, and stands:

* the door's verdict on `ct3/wall_ct3.json` (the served boundedness walk returns `None`,
  G-B0W reproduced here rather than cited; the cell admitted; the truncation tail
  `7.3139e-8` hartree per water against the `1e-5` stake);
* the table term's cost, `1.006957` of LIQUID-1's own force pass on an idle host;
* the three plants' pre-checks, all PASS on carrier and on analytic reach;
* the equilibration criterion's machinery, and LIQUID-1's own settling numbers it is derived
  from.

Re-run the gate once the serving rule is smooth and the sweep has chosen a step under the rule
as written. Nothing here is a counted arm; `run` has never been executed.
