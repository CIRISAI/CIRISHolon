# Q8 grid re-adjudication on the repaired SVD — RESUME

Launched 2026-08-25 from main @ 4bcf0d2 (the SVD canonicality repair).

- **Runner:** `output/q8_mps/run_regrid.sh` (detached via `setsid`)
- **Log:** `output/q8_mps/regrid.log`
- **Done marker:** `output/q8_mps/regrid.DONE` — contains the exit code. 124/137 = hang.
- **Wall budget:** 4h hard timeout. Expected ~30 min (5 sweeps/config, not 20).

## What it decides

Adjudicated against the FOUR OUTCOMES committed at `3123000` — no new outcome.
The pre-registration is the ADDENDUM already written in `Q8_MPS_RESULTS.md`
(committed 4bcf0d2, before this run was launched).

Recorded table had 5 of 8 VOID, §7 sweep kill FIRING (threshold 2).
Two of those five already invert on replay at χ=256. This run decides the
remaining three (N=10 at U=0, 1, 4) on the original harness.

## On completion

Append the reading to `Q8_MPS_RESULTS.md` under the addendum — WHICHEVER WAY IT
FALLS, including if it leaves the kill standing. Do not edit the recorded table.
