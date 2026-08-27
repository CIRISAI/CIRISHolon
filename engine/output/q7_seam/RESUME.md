# Q7 RUN LEDGER — active

## Q7 sweep
- Launch: `setsid nohup ./target/release/q7_run > output/q7_seam/q7.log 2>&1`
- Done marker: `output/q7_seam/q7.DONE` (exit code); output `output/q7_seam/q7.json`
- 2 N x 7 U x 6 a = 84 configurations, 378 region-instances. Each configuration also solves the
  MIRRORED potential for gate G7-E9, so the Lanczos cost is doubled by design.
- Deterministic (pinned Lanczos seed, pinned SCF guesses); rerun the launch line if it dies.
