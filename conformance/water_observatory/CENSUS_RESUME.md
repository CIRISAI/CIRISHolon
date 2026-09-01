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
| mixed `--ozone=served` | 0 of 8 | **REFUSED** — panicked at the ozone load, exit 101 | `census_mixed_served.log`, `CENSUS_RESULTS.md` §10.4 |
| mixed `--ozone=fenced` | **8 of 8, COMPLETE** | rc 0 | `census_mixed_fenced.log`; 15 certified-strict incl. the OH₂, 14 budgeted, 262 transient, 15 void |

**The served arm refused and that is a result, not a failure.** At this pin
`ozone::generate()` is a hard `None` — the convicted surface was WITHDRAWN, not patched —
so the run died at the table load rather than censusing an empty surface. Its consequence
is that `p2_waterquench.log` is not reproducible from any current commit. §10.4.

**A forward prediction is staked on the fenced arm** at `0a6d363`, before seeds 2–8
printed: ≥ 5 of 8 seeds reproducing the banked molecule multisets. Its premise was then
found FALSE at `d2533b0`, still before the data — the O–O curve also moved (6.7e-6 → 2.7e-6,
from a `tier.rs` sparsity optimisation), so the comparison is confounded and the stake is
expected to fail. Both commits precede the result; git is the check.

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

## THE THREE CHECKS, answered

**(1) THE BUILD PIN — my build carries NO dE4 dispatch, and that is verified by symbol
table rather than by mtime.** `nm -C` on my `waterquench_traj` finds **zero**
`quaternary::de4_ohhh_fci` symbols; the same command on the running `waterquench` finds
**one**. Neither binary carries any T3 marker (`DEFAULT_SCENE_ATOMS`, `complete_pairs`,
`ExternalWork`, `Periodic` — all absent), so both predate that refactor. My worktree is
pinned at `a3b3d4b`.

The consequence is that **neither of my arms can reproduce the completed dE₄ seeds, and
neither is trying to.** They are a different experiment, not a different build of the same
one, so a protocol-equality gate between them would fail for a reason that has nothing to
do with trajectories. I did not build from `rescue/de4-sim-worktree` minus the T3 hunks:
that reconstruction is the water lane's to own and to certify, and a census standing on my
guess at someone else's diff would be worth less than no census.

**(2) THE SERVED ARM IS PREP-ONLY, and was designed that way.** `--ozone=served` loads a
surface that is mid-generation and uncertified, and whose predecessor was convicted by
M-CHEAPER-THAN-ITS-PRICE. That arm exists solely to test whether my runner reproduces the
committed `p2_waterquench.log` — an INSTRUMENT gate — and nothing from it banks as physics
under any outcome. Said in `CENSUS_RESULTS.md` §5 as well, so a reader of either document
meets the fence.

**(3) PARKED, out of the session scratchpad.** The hydrogen arm now lives at
`/home/emoore/holon-artifacts/census-traj/` (95 MB, 8 seeds) and its sha256 manifest is
COMMITTED at `conformance/water_observatory/census_traj_manifest.sha256`, so a regeneration
can be verified bit-for-bit rather than merely repeated. The two mixed arms are still
writing, so they cannot be moved from under their own file handles; `park_and_census.sh`
is detached and does it for them — census into the repo, move to the durable path, append
to the manifest — the moment each done-marker lands.

## ROAD ITEM 5 IS MET — and the OH₂ framing here was corrected twice

**A certified-strict OH₂ quotient exists, on conservation-clean physics:** seed
`0x53415425`, fenced arm, block `0x0a08`, atoms [3, 9, 11]. 893.8 fs strict against an 834
fs window, a block for 72.3% of a 17.5 ps run, control rate 0.000, momentum 5.70e-5 of
bound. `CENSUS_RESULTS.md` §0 and §11.

The framing in this file went through two corrections and both are worth carrying:

1. The brief said seed 2 produced the first emergent OH₂. On the **MBE3 banked log** that
   was a header grep — `OH2` there names the (O,H,H) surface — and the parser gate
   (`holon_lens::quenchlog`) exists because of it.
2. Then the **dE₄ arm's** logs were found outside the repo tree and banked
   (`p2_de4_seeds/`, `1ae2c1b`): seed `0x53415422` really does carry an OH₂ molecule line.
   So "the OH₂ does not exist" was true of one artifact and false as a general claim.

**But the dE₄ observation is UNDER A FIRED GATE** — all six banked dE₄ seeds breach the
momentum bound by 4–5 orders with energy in bound. The conservation-clean OH₂ is the one
measured above, from MBE3 with the OOO triples fenced, at 4.7e-5–7.9e-5 of bound. **So the
water adjudication did not have to wait on the dE₄ repair.** What still waits on it is a
different question: whether the four-body term changes the answer.

## WHAT THIS LANE LEAVES READY FOR THE NEXT ONE

The blocker that opened this lane is cleared: road item 5 did not need the dE₄ repair. What
remains is two experiments, both SPECIFIED so neither gets answered by inference:

| question | held fixed | the variable | who |
|---|---|---|---|
| does the OOO fence move the endpoint? | everything at `45a513a` | served vs fenced | unassigned; design in `CENSUS_RESULTS.md` §10.5 |
| does the four-body term change the answer? | **this fenced arm as the baseline** | repaired full-strength dE₄ | water lane owns the rerun; this census adjudicates its trajectories |

The second is the live one. The banked dE₄ seeds were near-MBE3 physics — the momentum
defect was a double mass division that also weakened the dE₄ forces by 3–4 orders — so the
first true four-body experiment is the repaired rerun, and its comparison baseline is the
eight fenced trajectories parked at `/home/emoore/holon-artifacts/census-traj/fenced/` and
hashed in `census_traj_manifest.sha256`.

**To adjudicate them when they land**, from a clean checkout:

```
cargo run --release -p holon-lens --example census -- \
    --reference=conformance/atomworld/p2_waterquench.log <their-traj-dir>
cargo run --release -p holon-lens --example block_probe -- <traj> <block-hex>
```

The census needs trajectory DUMPS, not logs — `waterquench_traj` produces them and its
frozen-protocol block is byte-gated against `waterquench.rs`.

## Two corrections this lane made to itself## Two corrections this lane made to itself, both worth carrying forward

1. **A window staked in TIME must be measured against timestamps, never against `dt`.** The
   engine's timestep adapts mid-run; on hydrogen seed `0x53415421` it halves after eleven
   frames. The first census converted the window once from the header `dt` and was
   therefore enforcing 417 fs while claiming 834. `Header::frame_fs` and
   `Header::frames_in` are now REMOVED, so nothing can make that mistake again.
2. **A control floor must be a shuffle floor, not a constant.** The staked 5% pool rate
   voids all 48 genuine H₂ molecules at exactly 0.077, because six molecules in a pool of
   66 pairs means each sees five peers pass. The stake was not moved; the successor is
   staked in `CENSUS_RESULTS.md` §4 for the next freeze.
