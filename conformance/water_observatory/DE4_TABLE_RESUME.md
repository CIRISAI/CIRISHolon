# DE4-TABLE — where the lane is, and what to do next

*State as of 2026-09-01. The prereg is `DE4_TABLE_PREREG.md`, frozen and amended (A1.1
through A1.10, all pre-data). This file is the operational hand-off: what is running, what
is proved, what is owed.*

## What is RUNNING right now

```
pid file    engine/output/de4/de4_v4.PID
log         engine/output/de4/de4_v4.log
checkpoint  engine/output/de4/de4_v4.ckpt      <- the resume AND the live progress readout
artifact    engine/output/de4/de4_v4.json      (written only on success)
command     de4_table --nr 13 --nu 11 --r 0.9:6.0 --stretch 3.0 --u -1.0:0.9975 \
                      --region 5x5x5x4x4x4 --warm chain --workers 6 --device cpu \
                      --checkpoint <ckpt>
nice        5     <- NOT 19; see "why v4" below
```

**Size: 497,640 representatives of a 2,924,207-node box.** Re-banked price (prereg A1.3)
~2.0 s of core time per representative at loadavg ~77.

### Why there are four versions, one line each

Three kills, each for a different defect, and none of them a false start — the run is the
instrument that found them:

| | killed because | cost |
|---|---|---|
| **v1** | no resume, no live progress: a 78-hour run whose only health signal was that the pid existed | 0.96% of budget |
| **v2** | the sink was correct, its GRANULARITY was not — 64 regions put the first commit ~21 h out, and v2 then died at 52 min with **zero** regions committed, demonstrating it | nothing replayable |
| **v3** | nice 19 against a nice-0 machine: 0.43 effective cores, a 9.4-day run. `renice` downward is refused to an unprivileged process, so the only fix was a relaunch | nothing replayable |
| **v4** | — running — nice 5, 729 regions, first region committed at 15 min | — |

v4 also carries what v3 could not: the checkpoint opens BEFORE the ~370 s surface build (so
a death in setup leaves a diagnosable log), every regime axis is READ rather than asserted,
and it is main's merged bytes rather than a branch's.

### v1 was KILLED, and that is a recorded decision

The first attempt (pid 4186424) ran 2698 s, consumed 3357 s of CPU at **1.24 effective
cores** against six workers requested, and was killed at **0.96% of its priced budget** on
the lead's ruling. It projected to 348,000 / 1.24 = **77.9 hours** with no resume and no
live progress — `LeasedRun::progress()` is readable only after the join, so pid-existence
was its only health signal. Both violate the detached-compute rule; M-PROBE-THE-RESOURCE
names the blindness half. The full arithmetic is in `engine/output/de4/de4_v1.log` and is
repeated in v2's launch header, so the kill is a decision in the record rather than a gap.

### Resume: how it works, and the trap it avoids

The checkpoint is **region-granular**, and that is load-bearing rather than convenient.
Under `CanonicalChain` a region's first solved node is cold and every later one warm-starts
from its canonical predecessor in that same region — so replaying INDIVIDUAL NODES would
hand the next node a different starting vector, and the table would come back with
different last bits: correct physics, different artifact. A region is therefore replayed
**whole or re-solved from cold**, never partially. A crash loses at most one in-flight
region per worker.

`tests/checkpoint.rs` proves rather than asserts it: a resumed run's `table_bytes` and
digest equal an uninterrupted run's exactly, at 1, 4 and 8 workers; a fully committed log
solves ZERO nodes; a region whose `END` line never landed is reported torn and re-solved;
and a region whose bytes were corrupted fails its own digest. The control is there too — 
merely attaching a checkpoint must not move the table, or the equality would be comparing
two altered runs.

**To resume after a death: relaunch the identical command.** The checkpoint path is the
same; it replays what committed and re-solves the rest. The binary prints, at startup, how
many regions replayed and how many were torn.

### Live progress

One `END` line per committed region, so:

```
grep -c '^END ' engine/output/de4/de4_v4.ckpt      # regions done, out of 729
./target/release/ckpt_verify engine/output/de4/de4_v4.ckpt   # and: is it replayable?
```

The binary also prints a progress line every 60 s carrying regions-committed, percent,
elapsed and loadavg.

**When it lands**, the gates are one command:
```
cargo run --release -p holon-chem --example de4_certify -- engine/output/de4/de4_v1.json
```

## What is PROVED, and where

| | |
|---|---|
| the fold is bit-identical (**G1**) | 3-body digest `43bfa8f5…b6d7` reproduces at 1/4/8 workers after the whole 3-axis path was re-expressed through the folded generator. `holon-tables/tests/nd_bit_identity.rs` |
| the orbit reduction is exact | 5.876x fewer solves on the production shape; every mirrored node's bits equal its representative's. `holon-tables/tests/symmetry_orbits.rs` |
| the 6-D interpolant is bit-exactly S3-symmetric | on the trivial-stabiliser orbit. `quaternary_table.rs`, 7 unit tests |
| the certification harness works | L0, N0, C1, S3, B1 all PASS against a synthetic artifact on the real 13/11 grid, and each was shown to FIRE under a planted defect |
| the corners are real (**S1**) | `d3[dE4]` grew 3.27x and 5.14x when `h` halved; branch (a) |
| the seam scan is variationally clean | no warm start beat its cold solve on any of six slices, worst -1.19e-12 Ha |

## What is OWED, in priority order

1. ~~Resume for the leased generator.~~ **DONE** — `holon-tables/src/checkpoint.rs`, region
   granular, bit-identity proven at 1/4/8 workers. Note the correction: I first justified the
   hex format by claiming `s2_ozone_table`'s `{:.16e}` decimal log loses bits. **That is
   false** — measured over 2,999,033 values, `{:.16e}` round-trips f64 exactly, because
   seventeen significant digits is the round-trip width. The real defect in that generator is
   worse and is the one to carry: its warm-start carrier is updated only inside the branch
   that solves, so a knot replayed from the log leaves the carrier stale and the next knot on
   that ray starts from the wrong vector. A resumed ozone table is not bit-identical to an
   uninterrupted one, and the cause is the warm chain, not the number format. Measured on the
   live job: 4,627 KNOT lines, **zero duplicate `idx`**, so it has never resumed and its
   bit-identity is currently intact — the obligation is contingent, not owed.
2. **Wire the table into the trajectory loop.** NOT DONE, and deliberately not started:
   `holon-render/src/sim.rs`'s four-body block was rewritten by another lane mid-session
   (commit 21e6be3, `ohhh_fci_grad`, nine seeded dual solves), a `waterquench` process is
   running against it, and there is no table to wire yet. The wiring is
   `QuaternaryTable::eval_cartesian`, which already returns the value and the Cartesian
   force on all four atoms; the two-route discipline wants `ohhh_fci_grad` KEPT as the
   referee behind a flag, not deleted.
3. **Run the gates.** `de4_certify` on the real artifact. Expect T1 to be informative
   either way — A1.4 pre-commits both readings and names the discriminator (whether the
   firing witnesses are the ones near a reactive channel).
4. **Migrate the four hand-rolled `holon-chem` generators** (trimer, water, ooh, ozone)
   onto the folded pipeline. The capability exists; the migration does not. This is DRY
   residual entry (ii) and it is the largest one.
5. **`geometry_problem`'s `expect` is a live hazard.** Two coincident centres make
   `cholesky_orthonormaliser(...).expect("overlap not positive definite")` fire INSIDE a
   worker, which takes down a whole generation. `embed_tetramer` and `OhhhSurface::realise`
   guard it at `MIN_SEPARATION = 0.1` bohr, but the upstream panic is still there for any
   surface that reaches a degenerate geometry.
6. **The sum-parity serpentine's documented invariant is false**, and the 3-body production
   region shape `[2,2,2]` sits in the failing case (4 measured non-adjacent steps on a
   4x4x2 grid). Not repaired, because repairing it moves every committed table. Pinned by
   `nd_bit_identity.rs::the_sum_parity_serpentine_is_not_unconditionally_adjacent`. The
   4-body path uses `Serpentine::Reflected`, which is adjacent unconditionally.
7. **`live_reaper::the_reaper_never_reaps_a_worker_that_is_working` flakes** ~60% on a
   loaded machine, on identical binaries. It measures scheduler latency, not solve time.
   Not retuned — a gate's threshold is not a passing lane's to move.

## Things that will bite the next person

- **`sort_ohhh_internals` is NOT a canonical form** and is still exported and still has a
  passing test. It sorts the O-H and H-H triples independently, which is invariance under
  S3xS3 (order 36) where the group is S3 (order 6), so it hands six geometries one address.
  Use `quaternary_table::canonical_ohhh`. The old function should be deprecated with a
  pointer; that edit is not made here only because `tests/quaternary.rs` asserts on it and
  the fix belongs with whoever owns that test.
- **`OhhhSurface::canonical` permutes indices BETWEEN axes**, so it silently requires the
  three radial axes to be identical to each other and the three cosine axes likewise. The
  mesh cannot check that — it cannot see the physics. A ragged grid produces a complete
  table of entirely plausible wrong numbers. Asserted in
  `symmetry_orbits.rs::the_orbit_grids_axes_are_uniform`.
- **`OhhhSurface::new` costs ~370 s** before a single node is solved: it samples two
  1024-knot pair curves, 2048 two-centre solves. That is amortised over the run and worth
  it (it removes six fresh pair solves per node), but it makes every small test using a
  real `OhhhSurface` slow, and it is why the smoke run's log shows nothing for six minutes.
- **The artifact is large.** The production JSON carries 2,924,207 hex values, ~67 MB. It
  is gitignored territory; do not commit it.
