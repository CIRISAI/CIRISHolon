# GF2a A2 — resume notes (detached-compute rule)

Everything here is resumable. A killed run loses at most the sweep it was in.

## Layout
- `exam.sh` — the N = 8 exam (14 runs: 6 mixed ladders, 6 cold χ=256, plant vi unmixed
  ladder, plant vii mutant). `rows/<name>.json` is a completed run; `rows/<name>.err` its
  stderr; `ckpt/<arm>/<tag>_chi<c>.state` the in-rung state (per sweep),
  `<tag>_chi<c>.done.state` a completed rung's final state, `<tag>.rungs.jsonl` the
  completed rows of a ladder. Re-running `exam.sh` skips completed runs and resumes partial
  rungs from their last sweep (bit-identical: R1, `tests/qcd2_gauge.rs`).
- `ladder.sh` — the volume ladder of A2.6 (written when the exam passes), same layout under
  `ckpt/ladder` and `rows/ladder_*`.
- `score.py` — reads the rows and prints the verdicts against the exact referees.

## Restart
```
cd conformance/crystal/qcd2_a2
QCD2_WORKERS=4 Q8_THREADS=4 setsid nohup bash exam.sh > exam.log 2>&1 < /dev/null &
python3 score.py
```
Binaries: `engine/target/release/examples/qcd2_dmrg` (host), built with
`cargo build --release -p q8-mps --example qcd2_dmrg` from `engine/`;
`engine/crates/holon-gpu/target/release/examples/qcd2_sym_device` (device), built with
`cargo build --release --example qcd2_sym_device` from `engine/crates/holon-gpu/` (its own
workspace). Never edit a running script; copy, edit the copy, run that.

## Exact referees (the colour-lane arm, `GF2A_QCD2_RESULTS.md`)
x=4: B0 −51.9229999638, B1 −47.9964825669, B2 −36.6401053164;
x=9: B0 −123.0642401146, B1 −113.9136751337, B2 −87.5269948585.
