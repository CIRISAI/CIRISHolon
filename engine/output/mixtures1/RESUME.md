# MIXTURES-1 engine lane — detached runs

Everything here was launched with `setsid nohup` and writes a `.DONE` marker on
completion, so a dead session kills narration and never computation. Logs are
line-flushed, so a killed run keeps its partial results.

Campaign record: `conformance/atomworld/MIXTURES1_RESULTS.md`.
Frozen protocol: committed at `0851ccb`, **before** any P1 arm ran.

## Complete

| marker | what it measured | verdict |
|---|---|---|
| `p1_hydrogen.DONE` | P1 control, 16 H, 8 seeds | H2 modal in 8 of 8; control PASSES |
| `p1_chlorine.DONE` | P1 control, 16 Cl, 8 seeds | Cl16 in 8 of 8; control PASSES |
| `p1_mixed.DONE` | P1 product, 8 H + 8 Cl, 8 seeds | **branch (b)**: HCl modal in 0 of 8 |
| `p1_diagnose.DONE` | post-hoc branch-(b) diagnostic | 74 bonded edges; 21 of 40 H–Cl within 0.5 a₀ of `R_e` |
| `d1_overlap_H2.DONE` | D1 accuracy on H2, 16 points | worst 5.05e−13 Ha vs 1e−8 stake |
| `fci_staked_cost.DONE` | **exact FCI cost of all six staked pairs** | SiO 33.9 s/geometry — the exact side is CHEAP |
| `e2_ordering.DONE` | E2's well depths, 8 of 9 pairs | **branch (b)**: three inversions |
| `e2_byhand.DONE` | SiO's well, determinant route by hand | D_e 0.263676281 Ha — deepest in the set |
| `d1_overlap_LiH.DONE` | D1 accuracy on LiH, 12 points | worst 5.59e−12 Ha vs 1e−8 stake |
| `mps_ladder.DONE` | **MPS_MAX_ORBITALS re-derived** | no orbital threshold exists; the operative bound is `MPS_MAX_DETERMINANTS = 1024` and the route unblocks nothing |

`mpo_cost_*.log` carry the MPO-builder cost that decides D1: 0.00 s at 2
orbitals, 528.48 s at 6, did not finish in over an hour at 10. `mpo_cost_HCl.log`
has no `.DONE` by design — "did not complete" IS the reading, and finishing it
would only sharpen a number already past the point of usefulness.

## A solver constant moved, and one shipped table moved with it

`DAVIDSON_REQUESTED_TOLERANCE` is now **1e-10**, replacing an unreachable 1e-11
ask (`cd9971c`). If you regenerate anything, know that:

* **`docs/atoms/tables/Cl2.json` moved on 48 of 192 knots, worst 1.489e-11 Ha.**
  Both shipped tables were regenerated at that commit, so the tree is
  reproducible from its emitter as it stands. `HCl.json` did not move at all.
* Nothing else did. B1's bit-identity reference is untouched (the H2 path does
  not shift), Cl2's declared uncertainty is unchanged at 1.000525e-10, and the
  movement is 150x inside R2's 1e-10 pointwise stake.
* **The successor will move everything.** Lowering the expansion floor at
  `davidson_eigh_from`'s `nw > 1e-10` guard is the real fix and carries a
  re-banking campaign: B1's reference, the banked records, the referee pins. It is
  deliberately not done, and it must not be done as a side effect of anything.

`examples/exit_scope.rs` is the instrument for this: per-curve FNV digests over
raw energy bit patterns. Run it before and after any solver change — "identical"
is a claim only bits can settle, and this one was 6-of-9 identical, which is the
shape that passes a spot check.

## Nothing is running

Every detached run has landed or been deliberately stopped. The `d1_staked*` and
`mpo_cost_HCl` logs all carry a line saying they measured the PRE-REBUILD MPO
builder and were superseded; `d1_staked2_*` carry a line saying they produced zero
rows under a ladder that could not complete a grid point.

**D1's blocker moved rather than lifted.** The MPO wall fell (SiO: over twelve
hours → 0.31 s). Behind it is convergence: SiO is 1.1e−2 Ha from exact after 664 s
at chi = 32, six orders from the 1e−8 stake. `bank::D1_RECORD` stays `NONE`.
Re-running D1 is only worth it if the SWEEP implementation changes — the MPO no
longer is the cost.

## A FOURTH exposure, and I caused it: NEVER EDIT A RUNNING SHELL SCRIPT

Bash reads a script INCREMENTALLY, resuming from a byte offset. Rewriting a
launcher while its shell is still executing it makes bash resume at that offset
inside the NEW contents — which re-ran the whole `mps_ladder` invocation,
truncated its log with `>`, and lost the completed SiO row.

I did this while adding the marker-removal fix below, i.e. while writing a note
about stale artifacts. The data was not corrupted, only lost and recomputed, but a
run that silently restarts is indistinguishable from one that is merely slow, and
the numbers changed slightly between invocations (HCl 55.9 s then 63.0 s) which is
exactly the tell that would make a reader distrust both.

**Rule: a launcher script is immutable while its shell is alive.** Write a new file
under a new name, or wait. `ps -o ppid=` on the running binary tells you which
shell owns it.

## A third exposure, found by tripping over it: STALE DONE-MARKERS

The detached-compute discipline here is `setsid` + a `.DONE` marker + this file.
The marker is written AFTER a run. Nothing was removing it BEFORE one — so a
relaunch left the previous invocation's marker in place, and a waiter watching for
it reported completion instantly for a run that had barely started. That happened
to the `mps_ladder` run and I read a three-row partial log as a finished ladder
until the process list disagreed with the marker.

Same shape as the run-lock hazard: a stale artifact from a previous run being read
as this run's result, with nothing looking broken — the file exists, it parses, and
it is wrong. **Every launcher here now removes its marker as its first action**, and
a waiter that fires suspiciously fast should check `ps` before believing it.

## Two exposures, one closed and one open

**The run lock, OPEN.** `p1.sh` runs its three arms sequentially, one writer per
output path, so there is no concurrent-writer exposure *within* a run. There is
one *across* runs: relaunching a script while one is live gives two processes and
one path, and the stale one finishing last silently overwrites the good artifact
with a file that exists, parses, and is wrong. Every P1 arm has landed and its
logs are committed, so a lock now would guard nothing — but any NEW multi-arm
campaign here should take one (the referee lane's `elements1_referee/test_runlock.py`
is the pattern: refuse a live holder, take over a dead one with a note, and
demonstrate the refusal against a live process). Also: `pkill -f <pattern>`
matches your own shell's command line, which is how the water lane killed the
wrong process.

**The grid maximum, CLOSED.** A max over a grid is a LOWER bound on its own
supremum, so a number scored against a stake with under ~2× margin deserves a
re-sweep at higher resolution. Checked: nothing here is inside that band. C1
energy 4.047e−5 against a 4.058e−3 bound (ratio 0.0100), C1 momentum 0.0003,
D1's H2 overlap 5.05e−13 against 1e−8, plant (i)'s carriers 8.6 orders above the
referee tolerance. B1 is bit-exact and has no margin to be wrong about. P1's
headline is a COUNT, not a supremum.

## Re-running any of it

```
cd engine
./target/release/examples/mixquench <mixed|hydrogen|chlorine|cost|diagnose>
./target/release/examples/e2_ordering [PAIR ...]
./target/release/examples/e2_byhand  [PAIR ...]     # determinant route, by hand
./target/release/examples/fci_cost   [PAIR ...]     # one geometry, exact, timed
D1_CHI=8,16,32,64 ./target/release/examples/d1_bridge <cost|probe|overlap|curve> <PAIR> [n]
node crates/holon-render/viewer/smoke.mjs           # the wasm, not the library
```

Every parameter these accept is echoed in the run's own header, so a log says
which run it is. P1's protocol constants are NOT settable — they are `const` in
`mixquench.rs`, which is what makes a reported run re-runnable byte for byte.

## The wasm regression: CLOSED, and my attribution of it was wrong

The browser artifact trapped when rebuilt; it no longer does. Fixed at `3b37b8e`,
artifact rebuilt at `4536244` (double-built from a deleted target dir to identical
sha256, all three copies hashing the same), and gate 17 runs green at the DEFAULT
1 MiB stack. Nothing here is owed.

**The cause was not what I said it was.** I named W1's mask widening (`MAX_ORB`
32 → 64, `Mask` → `u64`, `Det` → `u128`). It was another lane's uncommitted
f-shell constants (`RMAX` 8 → 12), and the mechanism is
`Box::new(<big array literal>)` — a stack allocation wearing a heap allocation's
clothes. `RTensor::work` materialised 685 KB on a 1 MiB stack before moving it;
building a boxed slice through `vec!` instead drops that to 52 KB.

**The flaw in my method, which is the part worth keeping:** my three builds were
"current source", "current source + 8 MiB stack", and "1a13c49". The first two
isolate the stack correctly — that finding stands. The third does not isolate the
CHANGE, because `1a13c49` predates W1 *and* the f-shell work, so "pre-W1 source
works" was equally consistent with "pre-f-shell source works". A control that
predates two variables cannot attribute to one. elements3-heavy settled it with
three builds and one variable between them: clean HEAD green, HEAD + only the four
f-shell constants trapping.

I reported the mechanism as measured, and the measurement I had did establish the
stack — but I attached it to a named change my experiment could not distinguish.
That is a stronger claim than the evidence carried, made in the same breath as
"measured, not guessed".

## Blocked on another lane, not on a run

R2's engine half is built and `#[ignore]`d until `mixtures-referee` commits
`engine/crates/holon-chem/tests/data/mixtures1/`. Its exact side is measured and
cheap — SiO is 33.9 s per geometry, so a twenty-point grid is about eleven
minutes — so when the drop lands, R2 is a short run and a digest re-pin, done
field-diff-first.

## The debug profile was not being run, and it had been red since 09:52

`cargo test --manifest-path crates/holon-render-3d/Cargo.toml --no-default-features
--features headless` was passing for me in RELEASE and aborting in DEBUG, and I was
reporting the release run as the contract's 3D check. The abort is a stack overflow in
`all_presets_load_and_conserve_energy`, and it is MINE: bisected to `a0a665b` — the bank
commit — which grew `Sim` from one potential table to six, 331,656 bytes, of which the
bank is 197,504. `AtomWorld` carried it BY VALUE, and `new_with_preset` builds one, moves
it into the struct and returns the struct, none of which the debug profile elides.

Fixed by boxing the field (`AtomWorld` is now 32 bytes) and gated on the SIZE rather than
on the overflow, because an overflow aborts the process and takes the rest of the suite
with it — a defect that destroys the evidence of its own occurrence. The gate is
mutation-tested: un-boxing reports 331,680 bytes and fires.

Two rules out of it. **Run both profiles or say which one you ran** — the release run
cannot see a stack defect, and this is the second time the debug profile has caught a
`Sim`-size problem the release profile could not (MAX_SPECIES=4 was the first). And **a
big value that will live somewhere else is still assembled on the stack unless the type
says otherwise** — the same shape as `Box::new(<big array literal>)` in the wasm build,
now twice.

## The wasm artifact was rebuilt

`crates/holon-render/viewer/holon_render.wasm` is rebuilt from `build-web.sh` (300,756
bytes) because plant (iv) added two refusal codes to the ABI. Gate 17 exercises both
through the real ABI: 23 and 24 fire on mutated copies of the shipped Cl2 file, and the
unmutated file loads as the positive control. A shipped-table change without a wasm
rebuild leaves smoke.mjs grading a stale engine.

## A shipped artifact stopped reproducing from its emitter, and the check is manual

The tier ruling moved `CONVERGED_RESIDUAL` a decade, which flips `Cl2.json`'s published
`converged` field with no energy moving. Both shipped copies are regenerated. Note for
whoever checks this next: **the tables can never be bit-compared against a fresh emitter
run**, because `generation_ms` is a wall clock written into the artifact. Diff every
field except that one; a bare `diff` always shows a difference and will train you to
ignore it. Regenerating Cl2 costs 390 s.

The pre-ruling artifact is kept at `engine/output/mixtures1/Cl2.PRE_TIER_RULING.json`
(sha256 88ad657d0ffbe854a1c6535d700c9d7c3ec5cd5da26080d4a501dfb2e8872f37). It is the
evidence for the plant (iv) section and regenerating it away would delete the warrant.
