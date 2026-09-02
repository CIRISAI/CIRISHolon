# SCHWINGER-4 — resume notes (GF0)

Three detached runs, launched 2026-09-02 from `conformance/crystal/`:

| run | command | log | done marker |
|---|---|---|---|
| x = 4 column | `OMP_NUM_THREADS=8 python3 instrument/schwinger4.py staked4 4.0` | `schwinger4_x4.0.log` | `schwinger4_x4.0.DONE` |
| x = 9 column | `OMP_NUM_THREADS=8 python3 instrument/schwinger4.py staked4 9.0` | `schwinger4_x9.0.log` | `schwinger4_x9.0.DONE` |
| plant (ii) | `OMP_NUM_THREADS=6 python3 instrument/schwinger4.py plant-coulomb-off` | `schwinger4_plant.log` | `schwinger4_plant.DONE` |

Every configuration checkpoints to `ckpt4_<tag>.npz` beside this file; re-running the
same command resumes from the checkpoints (a `[checkpoint]` tag marks replayed points in
the log). Column outputs land in `schwinger4_x<x>.json`; then
`python3 instrument/schwinger4.py analyze` prints the gates and writes
`schwinger4_analysis.json`. The prereg is `SCHWINGER4_PREREG.md` (frozen alone,
36fd4c1); the instrument's gauge receipt is in commit bba3ad0. The record prints
loadavg beside every point; no wall-clock figure is a gate.
