# Q-SEAM RUN LEDGER — **CAMPAIGN CLOSED 2026-08-23. NOTHING HERE IS RESUMABLE.**

Both legs adjudicated and cashed. **Do not rerun anything in this directory expecting to continue
work** — there is no continuation. This file is a record of how the finished artifacts were
produced, kept so they can be *reproduced*, not resumed.

Verdicts live in `sim_engine/Q_SEAM_RESULTS.md`. The frozen design and its three amendments live
in `sim_engine/Q_SEAM_PREREG.md`. No further work is commissioned in this lane.

## Final state

| | |
|---|---|
| Q5 | **COMPLETE.** 70/70 configurations, zero VOID. Kill did **not** fire (C4 passes); headline **CORRECT BUT UNINFORMATIVE** (M3 beats it 1.000 to 0.667). |
| Q6 | **COMPLETE.** Kill **FIRED** on clauses (a) and (c). A falsification, not a VOID — the derived plumb line passed at 1.213e-13. |
| Robustness | Both kills re-adjudicated under the frozen G-E4b reading (N=6 VOID). **Adjudications agree.** Nothing UNADJUDICATED. |
| Gates | 20/20 green, zero warnings. |
| `q5.DONE` / `q6.DONE` | both `0` — **exit codes of the final, adjudicated runs.** Present and complete; their presence means finished, not in-flight. |

## Artifacts (final, not intermediate)

`q5.json`, `q5.log` — the Q5 sweep, joint gate, mutants, severity baselines, robustness block.
`q6.json`, `q6.log` — the Q6 statistics, plumb line, kill clauses, robustness block.

## Reproduction (only if an artifact must be regenerated from scratch)

Both binaries are **fully deterministic** — pinned Lanczos seed (`0x515F_5EA0_0000_0001`), pinned
SCF guesses, pinned permutation seeds — and each overwrites its own output. From `sim_engine/`:

```
cargo build --release -p q-seam
./target/release/q_seam_run > output/q_seam/q5.log 2>&1     # ~3 min
./target/release/q_seam_q6  > output/q_seam/q6.log 2>&1     # ~4 min
```

Rerunning reproduces the committed numbers exactly. It does **not** advance the campaign, and any
divergence from the committed artifacts is a regression to investigate, not a new result.
