# dE5 audit — resume record

*Session death must only kill narration, never computation. This file is what a fresh
narrator needs to pick the lane up without re-deciding anything.*

## State

| | |
|---|---|
| branch | `lane/de5-audit` (worktree, not pushed) |
| freeze | `DE5_PREREG.md` — ADMITTED by `Audit/prereg_audit.py`, committed ALONE at `5c27f42`, pre-data corrections at `6654405` |
| instrument | `engine/crates/holon-chem/examples/de5_audit.rs`, landed `2c8243d` with the plants log |
| plants | **all three FIRE** — see `de5_plants.log`. P-1 2.109e-15 vs 1e-12; P-2 9.35e-12 vs 1e-8; P-3 VOIDs as staked |
| SCF probe | `de5_scf_probe.log` — the evidence base for amendment A-1 |
| scoring | detached, `setsid`, `nice -n 10`; log `de5_score.log`, CSV `de5_audit.csv`, marker `de5_score.DONE` |

## How to check whether the run is still alive

**Do NOT trust `de5_score.DONE` alone until you have checked the runner's version.** The
first version of `de5_run.sh` used `wait $pid` on a `setsid` job, which returns "not a
child" immediately, so it wrote its marker the instant the run STARTED. One false marker
was produced and removed that way. The current version polls `/proc/$pid` instead. If in
doubt:

```
pgrep -f 'de5_audit --traj-dir'          # the process itself is the fact
tail -3 conformance/water_observatory/de5_score.log
```

The log's last line is `# exit code ...` only when the run really ended.

## To resume

```
cd <worktree>
./conformance/water_observatory/de5_run.sh score      # relaunch (idempotent; rewrites the log)
```

Nothing in the sampling depends on run order or on wall clock: the draw is a deterministic
function of the pinned trajectories, so a relaunch reproduces the same 24 configs.

## To finish the results document

`DE5_RESULTS.md` is written except for two placeholders, `<!--MEASURED-->` and
`<!--WORST-->`. Fill them from the run's own artifacts, never by hand:

```
python3 <scratchpad>/mkresults.py     # emits the §5 tables straight from de5_audit.csv
sed -n '/=== THE READING (amended/,$p' conformance/water_observatory/de5_score.log
```

The worst-config block (full geometry to 12 significant figures, rung sums, all five dE4
terms) is printed by the instrument at the end of a completed run.

## The verdict, as it stood at 5 of 24 configs

Stable and one-sided: every scored config is LIVE and every one is OVER the 5.0e-5 Ha
bound, by 36x to 1,501x, with the worst convergence ratio `|dE5| / max|dE4|` at 1.1285.
The expected branch is **(b) DOES NOT TERMINATE**, firing GANTT's `MPS` seam node. If the
remaining configs do not change that, the document needs only its two placeholders filled.

**Do not restate the verdict without its scope** (planar, STO-3G, `O2H3` only, diameter
< 6.0 bohr, atom-based MBE) **and without the strict reading** (G2 as frozen returns
BRANCH (d) VOID) beside it. Both requirements are staked in the freeze, §2.3 and §5b.
