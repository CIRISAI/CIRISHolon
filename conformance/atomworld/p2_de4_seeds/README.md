# P2 dE4-arm seed logs — the primary artifacts, banked with their conservation reading

*Banked 2026-09-01 by the lead. Task 1 of the operator's directive: upload the
sprint team's results.*

## What these are

Six completed seeds of the P2 water quench **dE4 arm** — the run family whose
physics adds the ab-initio four-body dE4(O,H1,H2,H3) dispatch on top of the
committed MBE3 arm (`conformance/atomworld/p2_waterquench.log`, which contains
NO dE4 and NO water). One seed per log, 8 H + 4 O, 20,000 boundaries, quench
3000 K -> ~300 K, fence = 4 per seed (the four ozone triples, refused — the
convicted surface was withdrawn, correctly).

These logs were captured by the sprint team's harness under
`~/.gemini/antigravity-cli/brain/dc3eab8d-*/.system_generated/tasks/` (tasks
4335/4337/4339/4343/4345/4347), copied here byte-identical (`logs.sha256`).
The earlier "no on-disk artifact exists" verdict searched the repo tree only —
the harness directory was outside every sweep. Scoped search reported as
complete, again; the artifacts existed the whole time.

## The observation

**Seed 0x53415422 formed water**: `molecules [H2 H2 OH2 O3H2]`, modal
O-molecule = OH2, T 284 K, 1,118 dE4 solves — a molecule-census line, not the
header-name collision that produced the earlier false reading. Seed 0x53415423
formed OH + O3H (and an H4). The other four read 3xH2 + O4H2. Aggregate:
**1 of 6 completed seeds with modal H2O; zero free oxygen on 6/6.**

## The gate that FAILED, reported at the same volume

**Every completed dE4 seed violates the momentum bound by 4-5 orders**
(`worst |p| / bound`: 2.9e5, 2.4e5, 5.0e4, 9.8e3, 2.5e5, 4.2e5) while energy
drift stays within its bound on all six. The MBE3 arm sat at |p|/bound ~ 6e-5.
The violation appears exactly where the new force channel does. Leading
hypothesis (stated as hypothesis, not conviction): the dE4 gradient is applied
without its full reaction partner — a Newton's-third-law / force-ledger defect
in the dispatch ("one gate per conservation law": the energy gate is green
while impulse is off by orders; a channel in the force but not the ledger).

**Consequence: the water observation is UNDER GATE, not banked as a result.**
A momentum-non-conserving force can pump local kinetic energy and bind what
should not bind. The cash-out is: fix the momentum ledger in the committed dE4
dispatch, re-run the seeds, and see whether OH2 survives. Until then this
directory is a RECORD of what an uncommitted build did, kept per the rule that
a fired gate is reported as plainly as a survival.

## Provenance

- Binary: `engine/target/release/examples/waterquench`,
  sha256 `e2259eefb31c73af3e580b699987d281030ad887ab94b0341dfe78e6467b881f` —
  PROVEN for the two in-flight seeds (hashed via /proc/<pid>/exe on both) and
  INFERRED for the six completed ones (same harness batch, same binary path;
  labelled inferred per M-PROVENANCE-OVERREACH).
- Source: the dE4 dispatch exists in NO commit. Bytes preserved at
  `refs/rescue/de4-2026-09-01` (11549dd, census-lens) and
  `rescue/de4-sim-worktree` (7480437). The reconstruction/commit path is
  saturation2-water's, after the T3 refactor lands.
- Seeds 0x53415421 (harness task-3130) and 0x53415425 (task-4341) were
  TERMINATED BY THEIR OWNER (the sprint team) 2026-09-01 as superseded by
  the exact-gradient arm, mid-run (frames ~8,000 and ~15,000 of 20,000).
  Their partial checkpoint logs remain in the harness task directory; no
  summary lines exist and none will. The same seeds run to completion in
  ../p2_de4_full/.
- The census target for census-lens's road item 5 (the OH2 closure leg) is
  seed_0x53415422.log, once a green dE4 commit exists to regenerate its
  trajectory from.
