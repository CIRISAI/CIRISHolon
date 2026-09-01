# Closure-census lane — RESUME

*Written while the runs were detached, so a dead session kills narration and never
computation.*

## What is banked (committed, green)

| thing | where |
|---|---|
| the stakes, frozen before the instrument | `conformance/water_observatory/CENSUS_PREREG.md` (commit `2f389e7`) |
| the instrument | `engine/crates/holon-lens/` — 44 unit tests + 9 plants |
| the trajectory runner | `engine/crates/holon-render/examples/waterquench_traj.rs` |
| the identity gate | `engine/crates/holon-render/tests/protocol_identity.rs` |
| the report generator | `engine/crates/holon-lens/examples/census.rs` |

Build and test, from a clean checkout:

```
cargo test -p holon-lens                      # instrument + plants, no holon-render needed
cargo test -p holon-render --test protocol_identity
```

`holon-lens` has ZERO dependencies and is a DEV-dependency of `holon-render`, never the
reverse. That is load-bearing: it was written while `holon-render` did not compile, and
the suite ran green throughout.

## The detached runs

Two arms, one process each, `setsid` + done-marker, one writer per output path:

```
$SP/census-target/release/examples/waterquench_traj mixed --ozone=served --out=$SP/traj/served
$SP/census-target/release/examples/waterquench_traj mixed --ozone=fenced --out=$SP/traj/fenced
```

where `$SP` is this session's scratchpad. Logs at `$SP/traj/{served,fenced}.log`, exit
codes at `$SP/traj/{served,fenced}.DONE`. Each arm: all eight staked seeds, 20,000 frames,
about 20 s a seed after a ~330 s setup (the O–O curve is 96 knots of CI).

Reading them:

```
cargo run --release -p holon-lens --example census -- $SP/traj/fenced
```

**Both arms are built from commit `a3b3d4b` in an isolated worktree**, not from the shared
working tree. The shared tree did not compile when this lane started (a T3 dynamic-storage
refactor had removed `MAX_ATOMS` from `sim.rs` while ten call sites still used it), and a
census computed against a tree that changes under it is not reproducible.

## THE BLOCKER, and it is not mine to clear

The run that reported the programme's first emergent OH₂ — `waterquench mixed` with
`dE4(O,H,H,H)` riding — **was built from source that is in no commit**:

* `git show HEAD:engine/crates/holon-render/src/sim.rs | grep -c de4_enabled` → 0.
* The working-tree `sim.rs` that had it has since been overwritten in place by the T3
  refactor. No stash, no backup, no worktree holds the pre-refactor bytes.

So the census cannot answer the OH₂ question yet, and saying it could would be inventing a
verdict from different physics. What is needed, in order:

1. The dE₄ `sim.rs` committed (red is fine — a red commit is recoverable, an overwritten
   buffer is not).
2. One completed seed re-run on that commit, its per-seed line diffed against the running
   job's line for the same seed. Match ⇒ the physics is reproducible and bankable.
3. Then: `waterquench_traj mixed --ozone=fenced` on that commit for seeds
   `0x53415422/23/24/26/27/28`, and the identical census over the dumps.

Step 3 is a copy-paste once step 1 lands. Nothing else in this lane is waiting on anything.

## What the arms can and cannot settle

* **`served`** reproduces the configuration of the banked `conformance/atomworld/p2_waterquench.log`,
  so it is the only arm whose G2 (trajectory equality against a banked reference) is
  checkable. Its physics is the (O,O,O) surface M-CHEAPER-THAN-ITS-PRICE convicted, so it
  is used as an INSTRUMENT gate and never as evidence about water.
* **`fenced`** is the last valid P2 configuration (OOH-complete MBE3, the four OOO triples
  honestly fenced) minus dE₄. Its physics stands; it has no banked reference log, so G2 is
  unverifiable on it and the report says so.

Neither arm contains an OH₂. The census's headline on them is therefore about the
INSTRUMENT and about what the engine's components actually do, not about water formation.
