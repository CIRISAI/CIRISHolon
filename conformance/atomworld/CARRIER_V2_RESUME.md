# CARRIER v2 — RESUME

*Run state for the carrier-v2 node. Untracked run-state lives beside the worktree; this
file is the map back into it. Session death must kill narration only.*

## Where things are

| | |
|---|---|
| worktree | `/tmp/claude-1000/carrier-wt` (branch `carrier-v2`, off `CIRISHolon` main) |
| build target dir | `/tmp/claude-1000/carrier-wt-target` (separate from the shared tree's) |
| freeze | `conformance/atomworld/CARRIER_V2_PREREG.md` — ADMITTED, committed BEFORE any instrument |
| format | `engine/crates/holon-lens/src/traj2.rs` |
| plants P-1..P-3 | `engine/crates/holon-lens/tests/carrier_v2_plants.rs` |
| plants P-4..P-9 | unit tests inside `traj2.rs` |
| bank gates | `engine/crates/holon-lens/examples/carrier_v2_bank.rs` |
| 3D carrier | `engine/crates/holon-md/examples/carrier3d.rs` |

## How to rebuild

```sh
cd /tmp/claude-1000/carrier-wt/engine
CARGO_TARGET_DIR=/tmp/claude-1000/carrier-wt-target cargo build --release \
    -p holon-lens --example carrier_v2_bank
CARGO_TARGET_DIR=/tmp/claude-1000/carrier-wt-target cargo build --release \
    -p holon-md --example carrier3d
CARGO_TARGET_DIR=/tmp/claude-1000/carrier-wt-target cargo test --release -p holon-lens
```

## How to re-run each gate

```sh
# G1a/G1b/G2/G5 on the real bank. Run from inside the worktree, or set HOLON_REPO.
cd /tmp/claude-1000/carrier-wt
/tmp/claude-1000/carrier-wt-target/release/examples/carrier_v2_bank \
    --bank=/home/emoore/holon-artifacts/census-traj

# The N-ladder. --dry-run first: it prints the resolved rungs and the cell arithmetic
# and computes nothing.
/tmp/claude-1000/carrier-wt-target/release/examples/carrier3d --mode=ladder --dry-run
/tmp/claude-1000/carrier-wt-target/release/examples/carrier3d --mode=ladder --workers=8
```

## Detached runs — the pattern every long run here uses

```sh
cd /tmp/claude-1000/carrier-wt
rm -f NAME.DONE
setsid nohup bash -c '<command> > NAME.log 2>&1; echo $? > NAME.DONE' >/dev/null 2>&1 &
# then: until [ -f NAME.DONE ]; do sleep 10; done
```

`.DONE` carries the EXIT STATUS, not the word "done": a run that died and a run that
finished both stop writing to the log, and only the status tells them apart.

## Known costs, measured (contended host — never a citable timing)

| step | cost |
|---|---|
| pair curve Z1–Z1, 96 knots | ~1.1 s |
| pair curve Z8–Z1, 96 knots | ~10 s |
| pair curve Z8–Z8, 96 knots | minutes — 2025 determinants per point; this dominates startup |

The three curves are generated ONCE per process and reused across every ladder rung, so a
ladder pays the curve cost once. A run that regenerated them per rung would put the whole
ladder's cost in the tables and price nothing.

## Exit codes (frozen in `CARRIER_V2_PREREG.md` §6)

`0` fine · `2` bad arguments · `3` a path did not resolve · `4` a version/format refusal ·
`5` a digest or field mismatch · `6` a worker-lease refusal · `7` an envelope refusal.

## Status

* freeze ADMITTED and committed before any instrument existed
* format v2 built; 125 tests green including all nine plants
* bank gates run: G1a 23/23, G1b 18/18, G2 18/18, G5 18/18, and the `max|dz|` column
  reproduces `CENSUS_RESULTS.md` §14.4's published 11.4899 exactly
* N-ladder: running
* production trajectories: not started; they wait on the ladder's price
