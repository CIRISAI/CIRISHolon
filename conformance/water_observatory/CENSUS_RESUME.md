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
| the measured results | `conformance/water_observatory/CENSUS_RESULTS.md` |
| the census output they summarise | `conformance/water_observatory/census_hydrogen.log` |

Build and test, from a clean checkout:

```
cargo test -p holon-lens                      # instrument + plants, no holon-render needed
cargo test -p holon-render --test protocol_identity
```

`holon-lens` has ZERO dependencies and is a DEV-dependency of `holon-render`, never the
reverse. That is load-bearing: it was written while `holon-render` did not compile, and
the suite ran green throughout.

## What has RUN

| arm | seeds | state | verdict |
|---|---|---|---|
| hydrogen control, `--ozone=fenced` | 8 of 8 | **COMPLETE** | banked in `CENSUS_RESULTS.md` and `census_hydrogen.log` |
| mixed `--ozone=served` | 0 of 8 | in flight, still generating the O–O curve | — |
| mixed `--ozone=fenced` | 0 of 8 | in flight, same | — |

The two mixed arms each pay a ~320 s O–O curve before their first seed, and the box has
been at load 65–85 on 32 cores throughout (a 27-core ozone tabulation holds the critical
path). They were launched at `nice -n 12` deliberately and have NOT been renice'd: the
ozone table is the campaign's critical path and this lane is not.

## The detached runs

Three arms, one process each, `setsid` + done-marker, one writer per output path:

```
$SP/census-target/release/examples/waterquench_traj mixed    --ozone=served --out=$SP/traj/served
$SP/census-target/release/examples/waterquench_traj mixed    --ozone=fenced --out=$SP/traj/fenced
$SP/census-target/release/examples/waterquench_traj hydrogen --ozone=fenced --out=$SP/traj/hydrogen
```

where `$SP` is this session's scratchpad. Logs at `$SP/traj/{served,fenced}.log`, exit
codes at `$SP/traj/{served,fenced}.DONE`. Each arm: all eight staked seeds, 20,000 frames,
about 20 s a seed after a ~330 s setup (the O–O curve is 96 knots of CI).

Reading them:

```
cargo run --release -p holon-lens --example census -- $SP/traj/fenced
```

**The mixed arms census THEMSELVES.** `$SP/auto_census.sh` is detached with its own
done-markers and waits on the two `.DONE` files, then writes
`conformance/water_observatory/census_mixed_{served,fenced}.log`. So the reading happens
whether or not any narration session is still alive — the discipline is that a dead session
kills narration and never computation, and a computation whose reading depends on someone
watching has the same exposure one level up.

**All arms are built from commit `a3b3d4b` in an isolated worktree**, not from the shared
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

## Two corrections this lane made to itself, both worth carrying forward

1. **A window staked in TIME must be measured against timestamps, never against `dt`.** The
   engine's timestep adapts mid-run; on hydrogen seed `0x53415421` it halves after eleven
   frames. The first census converted the window once from the header `dt` and was
   therefore enforcing 417 fs while claiming 834. `Header::frame_fs` and
   `Header::frames_in` are now REMOVED, so nothing can make that mistake again.
2. **A control floor must be a shuffle floor, not a constant.** The staked 5% pool rate
   voids all 48 genuine H₂ molecules at exactly 0.077, because six molecules in a pool of
   66 pairs means each sees five peers pass. The stake was not moved; the successor is
   staked in `CENSUS_RESULTS.md` §4 for the next freeze.
