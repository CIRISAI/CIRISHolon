# DE4-TABLE — where the lane is, and what to do next

*State as of 2026-09-01. The prereg is `DE4_TABLE_PREREG.md`, frozen and amended (A1.1
through A1.10, all pre-data). This file is the operational hand-off: what is running, what
is proved, what is owed.*

## What is RUNNING right now

```
pid file    engine/output/de4/de4_v1.PID
log         engine/output/de4/de4_v1.log
artifact    engine/output/de4/de4_v1.json      (written only on success)
command     de4_table --nr 13 --nu 11 --r 0.9:6.0 --stretch 3.0 --u -1.0:0.9975 \
                      --region 7x7x7x6x6x6 --warm chain --workers 6
```

Launched 2026-09-01T17:59:05-05:00, binary sha256 `a2875c4f…65681`, build exit status 0,
repo HEAD `115f3c9a…` with a dirty tree (so the binary is the WORKING TREE's, and HEAD is
context, not a claim about those bytes).

**Size: 497,640 representatives of a 2,924,207-node box.** At the re-banked price (A1.3)
of ~2.0 s of core time per representative at loadavg ~77, that is ~276 core-hours of
wall-times-workers, or ~97 core-hours of true CPU once the measured 2.4x oversubscription
is taken out. It is nice'd to 19 and shares a machine that already carries `s2_ozone_table`
(~27 cores) and two `waterquench` runs.

### THE ONE THING TO KNOW: there is no resume

`generate_surface_leased` runs the whole grid in a single call. If this process dies, the
work is lost and the run restarts from zero. This is a known gap, deliberately not papered
over, and it is the top of the "what is owed" list below. Note that the existing
`s2_ozone_table` generator's resume is not a model to copy: it round-trips values through
`{:.16e}` DECIMAL, so a resumed table is not bit-identical to an uninterrupted one — the
fix is to checkpoint the whole `NodeRecord` in hex, which is what the artifact format
already does.

**Check on it with:**
```
ps -o pid=,etimes=,%cpu= -p $(cat engine/output/de4/de4_v1.PID)
tail -30 engine/output/de4/de4_v1.log
```
It prints nothing between the parameter echo and completion. Silence is expected; only
`ps` distinguishes working from dead.

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

1. **Resume for the leased generator.** Checkpoint the full `NodeRecord` in hex as each
   node lands, and replay it on restart. Without this every long run is one `kill` from
   zero. The shape to copy is the artifact's own hex-bits format, NOT `s2_ozone_table`'s
   decimal log.
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
