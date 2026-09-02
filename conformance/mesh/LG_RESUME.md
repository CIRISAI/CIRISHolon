# NODE LG — RESUME

**Owner:** lattice-tier lane. **Worktree:** `/tmp/claude-1000/lg-wt`, branch `lattice-tier`.
**Never push.** Pathspec commits only; cirisontology-b4 integrates.

## State

| | |
|---|---|
| prereg | `conformance/mesh/LG_PREREG.md`, **ADMITTED**, frozen and committed at `ce392f3` |
| instrument | `engine/crates/holon-lattice/` — built, 28 lib tests + 2 FCHC integration tests green |
| campaign | detached via `conformance/mesh/lg_detached.sh`; markers `lg_full.DONE`, `lg_tests.DONE` |
| results | `LG_RESULTS.md` — written from `lg_full.log` once `lg_full.DONE` exists |

## If the session died

```
ls /tmp/claude-1000/lg-wt/conformance/mesh/*.DONE     # both present => the run finished
tail -80 /tmp/claude-1000/lg-wt/conformance/mesh/lg_full.log
```

Re-launch is idempotent — the script clears its own markers first:

```
setsid nohup /tmp/claude-1000/lg-wt/conformance/mesh/lg_detached.sh >/dev/null 2>&1 </dev/null &
```

## What must not drift

1. **The first law.** This tier is NOT a view of the molecular dynamics and is never
   composed through `closed_comp`. The molecular-to-lattice seam takes no status from
   this node, in any branch. `LG_PREREG.md` §0 is binding on every document written here.
2. **The Navier–Stokes limit is NOT claimed.** Only the necessary lattice condition
   (fourth-rank isotropy) is measured. The exit is named in §3 and stays named.
3. **The `b = L` point is the VACUOUS end of the curve**, never the curve's success. If
   the workbench band ever reads "certified closed", that is the failure this node exists
   to prevent.
4. **The door is DEFECT-AGAINST-VIEW**, named in §12 before the page existed. No aggregate.

## Findings the instrument made about its own design, to be banked

* The prereg's G7 did not name the probe's POPULATION. It is pinned by the second clause
  ("agrees with the frozen Python reference"), whose population is every
  `(position, movable state)` pair — but the ambiguity was real and is reported.
* Two sampler defects, both found by the derived law refusing to be reproduced:
  a forward-scan for the next movable cell biased the POSITION within the block
  (read 0.7025 against 0.75, and exceeded the bound at another size); and 20,000 draws
  with replacement from ~300 distinct cells were given a binomial band on 20,000, turning
  a 0.65σ agreement into a 5σ disagreement.
* `line_momenta` was first written summing each momentum component along its OWN axis,
  which is a quantity nothing conserves — the Leg-A gauge then had no sides at all.
  The line a component is summed along is the one that component's movers do not leave.
* HPP-4's census was guessed at 12 in a test; it is 15, with exactly ONE fiber of
  dimension above 1. So HPP admits a collision group of order 2 where FHP admits 4608.
