# C1 quantum-nuclei campaign — resume notes

Prereg: `conformance/water_observatory/C1_GATE_PREREG.md` (frozen 2026-09-01).
Driver: `engine/crates/holon-chem/examples/c1_campaign.rs`.
Fast gates (CI): `engine/crates/holon-chem/tests/c1_quantum_nuclei.rs`.

## How the stages are launched

Detached, so a dead session kills narration and never computation:

```
cd /home/emoore/CIRISHolon
setsid nohup ./engine/target/release/examples/c1_campaign <stage> [steps] \
  > engine/output/c1/<stage>.log 2>&1 < /dev/null &
# and on completion the launcher touches engine/output/c1/<stage>.DONE
```

Stages and what they produce:

| stage | gates | output |
|---|---|---|
| `dvr` | G0, G2, G6 | `dvr.log` — spectral references on both surfaces, residuals, work counts |
| `ladder <steps>` | G3 | `ladder.log` — E_cv(P) over the bead ladder |
| `production <steps>` | G1, G4, G7 | `production.log` — the headline and the isotope shift |
| `square` | G5, gate (c) | `square.log` — classical limit, commuting-square budget |
| `price` | G7 | `price.log` — unit costs and the cost-model check, run PINNED |

A stage is finished when `<stage>.DONE` exists AND the log's last line is not a panic.
`<stage>.DONE` absent with no process running means the stage died and must be relaunched;
the logs are append-safe to inspect but the runs are NOT resumable mid-stage — each stage
is deterministic in its seeds, so relaunching reproduces it exactly.

The `price` stage is the one that must be pinned, and it is run twice:

```
( echo "### pinned to CPU 0 (P-core)";  taskset -c 0  ./engine/target/release/examples/c1_campaign price
  echo "### pinned to CPU 24 (E-core)"; taskset -c 24 ./engine/target/release/examples/c1_campaign price
) > engine/output/c1/price.log 2>&1
```

Both core classes are reported and the E-core is the headline: this box is heterogeneous and
an unpinned wall clock is an undeclared variable.

## Machine note

This box is an i9-13900HX with heterogeneous cores and was at load average 50-85 while this
campaign ran (other lanes). No gate here is a timing comparison; the ONE timing number
(G7's price) is taken under `taskset` on a declared core, per M-PLACEMENT-LOTTERY.
