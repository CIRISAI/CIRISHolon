# Q7b RUN LEDGER — **CLOSED 2026-08-23. NOTHING RESUMABLE.**

Q7b kill FIRED (no candidate passes all five clauses; D3 fails only clause 1, 6 FPs of 115 wrong
region-instances). G7-FIT PASSED 22/56, so this is a verdict about certificates, not about the
sweep design. Verdict: `sim_engine/Q7B_SEAM_RESULTS.md`. Design: `sim_engine/Q7B_SEAM_PREREG.md`.

| | |
|---|---|
| Configurations | 56 of 56 gated, zero VOID |
| G7-FIT | **PASS** 22/56 (Q7: 0/84) |
| Kill | **FIRED** — soundness, not informativeness: D3 cov 0.909 / discrim 21 vs best baseline 0.436 / 0 |
| D1b | **FIRED 9 times, 9/9 correct** — the reflection anchor is live |
| Exactness | G7-E7 2.5e-14, G7-E9 9.3e-14, G-E5b 4.4e-12 |
| `q7b.DONE` | `0` — exit code of the final adjudicated run |

Reproduction only (deterministic, ~35 min; reproduces committed numbers, advances nothing):
`./target/release/q7b_run > output/q7b_seam/q7b.log 2>&1`

Pre-check artifact `spread_precheck.log` is the record that licensed the prereg (11/16 split).
