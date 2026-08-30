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

## One artifact owes a rebuild, and the rebuild is currently BROKEN

`crates/holon-render/viewer/holon_render.wasm` and its two `docs/` copies are the
build from `1a13c49`'s source. They **work**, and they are what ships. What they
are not is reproducible from HEAD.

**Do not rebuild and commit until the regression below is fixed.** A rebuild from
current source produces a wasm that TRAPS.

### The regression: a wasm stack overflow, invisible to every native test

Rebuilding after W1 landed (`MAX_ORB` 32 → 64, `Mask` → `u64`, `Det` → `u128`)
gives a 299,007-byte artifact that traps with `RuntimeError: memory access out of
bounds` on the first in-browser pair solve — `holon_bank_generate_pair(1, 2, 96)`,
which is helium, two orbitals, the cheapest solve the sandbox does.

Diagnosed, not guessed:

| build | wasm stack | result |
|---|---|---|
| current source, default | 1 MiB (wasm-ld default) | **TRAP** |
| current source, `-C link-arg=-zstack-size=8388608` | 8 MiB | works, 1,204 ms |
| `1a13c49` source, default | 1 MiB | works, 958 ms |

So it is the stack, and 8 MiB is enough. It is invisible natively because a native
main thread gets 8 MiB anyway — `cargo test -p holon-chem` and `-p holon-render`
are both fully green against the same source.

Ruled out: `trimer.rs`'s `MAX_ORB^4` stack arrays, which look alarming and are
not involved — that module has its own private `const MAX_ORB: usize = 3`.
`fci.rs` has no large fixed stack arrays either (`Strings::masks` is a `Vec`), so
the growth is in accumulated frames from the widened `Mask`/`Det` under
`opt-level=z` + LTO, not in one obvious object.

Two possible fixes, and the choice belongs to the lane that owns the widening:
raise the stack in `build-web.sh` with a declared `-zstack-size` (honest, and
masks future growth), or find and shrink the frame. Reported to `elements3-heavy`
and the lead rather than decided here.

**`viewer/smoke.mjs` is what caught this**, which is the gap it was written for:
every native gate passes, and the thing the page actually loads does not run.

### When it is fixed

```
cd engine && bash crates/holon-render/build-web.sh
cp crates/holon-render/viewer/holon_render.wasm ../docs/atoms/holon_render.wasm
cp crates/holon-render/viewer/holon_render.wasm ../docs/unified/holon_render.wasm
node crates/holon-render/viewer/smoke.mjs   # must reach the end without trapping,
                                            # and still refuse Cl2 with code 21
```

## Blocked on another lane, not on a run

R2's engine half is built and `#[ignore]`d until `mixtures-referee` commits
`engine/crates/holon-chem/tests/data/mixtures1/`. Its exact side is measured and
cheap — SiO is 33.9 s per geometry, so a twenty-point grid is about eleven
minutes — so when the drop lands, R2 is a short run and a digest re-pin, done
field-diff-first.
