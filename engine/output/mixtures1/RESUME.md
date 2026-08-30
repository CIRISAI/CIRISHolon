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

`mpo_cost_*.log` carry the MPO-builder cost that decides D1: 0.00 s at 2
orbitals, 528.48 s at 6, did not finish in over an hour at 10. `mpo_cost_HCl.log`
has no `.DONE` by design — "did not complete" IS the reading, and finishing it
would only sharpen a number already past the point of usefulness.

## Still running

| log | what | how to read it |
|---|---|---|
| `d1_staked_SiO_cost.log` | **the staked D1 refusal, measured on the staked species** | SiO's MPO build against a 12-hour budget. Replaces an extrapolation from HCl. |
| `d1_staked_S2_cost.log` | same for S2 (18 orbitals) | queued behind SiO; S2 is strictly larger, so an SiO refusal settles it |
| `mpo_cost_HCl.log` | the 10-orbital MPO build, still going | **past two hours** as of the last check, which is a sharper reading than the "over an hour" the results doc quotes. If it ever completes, quote the real number |

Every gate's verdict is already committed and none of these can change one. The
SiO and S2 builds would replace an extrapolated D1 refusal with a measured one;
if either MPO *does* build, the refusal weakens from structural to a scheduling
question and the D1 section should say so.

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
