# holon-gpu — the branch fold on CUDA

`holon::merge` states the one law: ledgers fold associatively and commutatively,
so any sharding, ordering, or distribution of a fold is deterministic without
coordination. `holon::mesh` cuts that law across OS threads. This crate cuts it
across a GPU, and the whole point is that the GPU changes nothing about the
argument — a warp schedule is just another sharding, and the ledger does not know
it happened.

Everything below was measured on this machine on 2026-08-27. Where a number is
contaminated, it says so, and by roughly how much.

---

## 1. Design

### What was chosen, and what was rejected

**Chosen: the affine amplitude evaluation, ported to CUDA** — deliverable 1's
option (a), the deepest of the two offered. The kernel does the F2 linear algebra
on packed `u64` per branch: it solves `R u = y + h` over F2 by Gauss-Jordan on
packed augmented rows, decides consistency, reads `u` off the pivots, and
evaluates the phase polynomial `sum_a u_a d[a] (mod 4)` and
`parity{a<b : u_a u_b J[a][b]}` with `__popcll`. The batched-tableau option (b)
was **not** built; option (a) is strictly the deeper port and the time went there.

**Toolchain: cudarc 0.19 + nvcc-compiled PTX**, which is option (a) and (b) of
the brief's toolchain choice together. `cudarc` with `dynamic-loading` dlopens
`libcuda.so` at run time, so a *build* needs no CUDA libraries at all; `build.rs`
shells out to `nvcc -ptx -arch=compute_89` for the kernel. Installed: driver
580.142 (CUDA 13.0), nvcc 12.0. That mismatch is the supported direction — nvcc
12.0 emits PTX, the 13.0 driver JIT-compiles it at module load. Nothing here
needs a cubin.

`kernels/fold.ptx` is checked in so the crate builds without a toolkit, and
`build.rs` **fails the build** if nvcc is absent *and* `fold.cu` is newer than
`fold.ptx`. A checked-in artifact silently older than its source is a
stale-diagnostic trap; it is refused rather than used. That guard was exercised in
all three states rather than assumed: with `NVCC` pointed at a nonexistent path
and a touched `.cu` the build fails with the stale message; with a fresh `.ptx` it
falls back and warns; with nvcc present it recompiles.

### The exponent design (the brief's explicit question)

`Cyc::add`'s one-shot alignment is the awkward part of putting this ring on a
GPU: two elements at different denominator exponents cannot be added lane-wise.
The design takes the ring **out of the reduction entirely**, in two steps.

**Step one — the device never multiplies in the ring.** A branch's contribution
is `weight * gamma * i^ip * (-1)^sign`. Multiplying a `Cyc` by `i^ip` and by `-1`
is exactly `omega^{2 ip}` and `omega^{4 sign}`, and multiplying by `omega` is the
coefficient rotation `(c0,c1,c2,c3) -> (-c3,c0,c1,c2)`. Rotation only permutes
and negates, so it cannot change which coefficients are even, so it commutes with
`Cyc::normalize` and leaves `m` alone. Therefore

> `amplitude_of(b, y) == rot(weight_b * gamma_b, r_b)` **as a struct**, where
> `r_b = (2*ip + 4*sign) mod 8`.

The host computes `BASE_b = weight_b * gamma_b` once, at upload. The device
computes one integer `r_b in 0..8` per branch and rotates. This is not an
approximation of `amplitude_of`; `tests/ring.rs` pins `rot` against the ledger's
own `mul`, and `tests/determinism.rs` checks the per-branch rotation codes
device-against-host over every basis state of a 12-qubit register.

**Step two — one exponent per batch.** The host takes `M = max_b m_b` and aligns
every `BASE_b` to `M` before upload, using `ring::align_to`, which is `Cyc::add`'s
own alignment branch (shift by `delta/2`, and one `sqrt(2) = omega - omega^3`
multiply when `delta` is odd). This is the brief's "keep per-branch amplitudes at
a COMMON exponent by construction per batch" option. The bucket-by-exponent
alternative was not needed: the batch is aligned once at upload and folded for
arbitrarily many `y`, so the alignment cost is amortized to nothing.

After those two steps the device reduction is **four independent sums of `i128`
integers**, and nothing else.

`align_to` is the one piece transcribed rather than called (the alignment is not
reachable through `Cyc`'s public surface), so it is pinned by
`tests/ring.rs::align_matches_the_ledgers_own_addition`, which drives it against
`Cyc::add` across exponent gaps 0..6 on 400 random elements — including the odd
gaps where the `sqrt(2)` multiply fires.

### Getting the branch data out of `Affine`

`Affine`'s `R`, `h`, `d`, `J` are private, and adding accessors to a file another
lane is editing is not a change this crate gets to make. `Affine::canon_key()` is
public and serializes exactly those four plus `n`, `k` and the zero flag, so
`AffineDesc::from_branch` decodes it. That is a dependence on a byte layout and
is treated as one: the decoder length-checks the whole key against the layout it
expects and names both lengths if they disagree, and
`AffineDesc::agrees_with` then drives the decoded descriptor back against
`Affine::amplitude` — holon's own `Vec<Vec<bool>>` solver — on a determining set
of basis states. The conformance test runs that on **every branch** of four real
circuits. A misread field cannot survive it quietly.

### Layout and caps

Per-branch arrays are strided by the batch size `B` (element `i` of branch `b` at
`i*B + b`) so a warp's loads are contiguous.

| array | size | contents |
|---|---|---|
| `rrow` | `n*B` u64 | bit `a` of entry `row` is `R[row][a]` |
| `hbits` | `B` u64 | `h` as a bitmask over rows |
| `jrow` | `kmax*B` u64 | bit `b` of entry `a` is `J[a][b]`, strict upper triangle |
| `dpack` | `2*B` u64 | `d[a] mod 4`, two bits per column |
| `base` | `8*B` u64 | `BASE_b` aligned to `M`: 4 lanes x (lo, hi) of an `i128` |
| `kk` | `B` u32 | `k`, or `0xFFFFFFFF` for a state flagged zero |

**Caps: `n <= 64` and `k <= 63.`** Bit 63 of each augmented row carries the
right-hand side, which is what costs the 64th column. Batches past either cap are
**refused**, not truncated. The augmented-row scratch array lives in local memory
and is the occupancy limiter, so the kernel is compiled twice — `NCAP = 32`
(256 B/thread) and `NCAP = 64` (512 B/thread) — and the host picks by `n`.

The reduction is warp-shuffle (`__shfl_down_sync` on 128-bit values moved as four
32-bit words) then a 32-slot shared-memory block reduction, then a host sum over
block partials in block order. **Block size must be a nonzero multiple of 32 and
at most 1024**, because the shuffle uses the full mask; other block sizes are
refused rather than silently producing a partial-warp result.

---

## 2. The determinism argument, and its test

### The argument

1. Per-branch contributions are rotations of a host-computed `BASE_b`, so no ring
   multiplication happens on the device.
2. The whole batch sits at one denominator exponent, so the fold is four sums of
   `i128`.
3. **Two's-complement integer addition is associative and commutative
   unconditionally — on overflow as much as off it.** So the four sums are
   invariant under the shuffle tree's shape, the block size, the grid size, and
   which branch landed on which thread. There is no atomic in the accumulation
   and no completion-order reduction, and there does not need to be.
4. `ring::from_lanes` returns the result to the ledger's normal form.

Determinism therefore does **not** depend on the batch being free of overflow;
correctness does. `GpuBatch::upload` refuses a batch whose worst case
(`B * max|c|`) could exceed `i128` rather than wrapping quietly.

This is deliberately a stronger claim than the mesh's. `holon::mesh`'s own header
is careful that exact `Z[omega]` addition gives order-independence of the VALUE,
and order-independence of the REPRESENTATION only when no partial sum cancels to
zero. A device reduction has far more partial sums than a five-way shard, so this
crate removes the ring from the reduction rather than hoping.

### The test (`tests/determinism.rs`, 12 tests, all passing)

| test | what it would catch |
|---|---|
| `five_launch_shapes_return_the_same_struct` | 5 shapes x 6 `y` on 200k branches at n=28: same `Cyc` **struct**, and equal to `mesh::fold_amplitude` at 1 and 16 shards and to `cpu::fold_packed` at 8 |
| `repeated_launches_of_one_shape_do_not_drift` | run-to-run nondeterminism at a fixed shape |
| `gpu_matches_the_cpu_mesh_on_a_real_circuit` | GPU vs `mesh::fold_amplitude(&PrunedSum)` on **all 16384 basis states** of a real n=14 T=12 circuit, plus shards {1,2,3,7,16} on 64 of them |
| `per_branch_rotation_codes_match_branch_by_branch` | a sum agreeing while two branch errors cancel — every branch, every one of 4096 basis states |
| `decoded_descriptors_agree_with_the_affine_they_came_from` | a `canon_key` misread — every branch of four circuits, against `Affine::amplitude` |
| `a_single_planted_bit_moves_the_answer` | a fold that is not reading `d` |
| `a_planted_defect_in_the_kernels_own_reads_is_visible` | a fold that is not reading `h`, `base`, or `J` |
| `an_off_coset_state_folds_to_exact_zero_on_both_sides` | the zero path returning something |
| `an_exactly_cancelling_batch_returns_zero_from_every_shape` | the representation crack the mesh header names |
| `a_mixed_exponent_batch_...` | the parity fence — see section 4 |
| `a_batch_that_could_overflow_is_refused_not_wrapped` | a silent wrap |
| `a_block_size_the_shuffle_cannot_honour_is_refused` | a partial-warp full-mask shuffle |

Plus `tests/ring.rs` (4 tests): `align_to` against `Cyc::add`, value preservation
under alignment, and `rot` against the ledger's own `mul` for every `r in 0..8`.

**Launch-shape sweep, 24 configurations, 10^6 branches at n=32, k=24.** All 24
returned the identical struct; wall time spanned **5.63 ms to 22.84 ms**, a 4x
spread in schedule with zero change in the answer.

```
  block     grid    ms/fold   same struct        block     grid    ms/fold   same struct
     32       64     22.842        yes             256       64      6.761        yes
     32      256     16.557        yes             256      256      6.280        yes
     32     1024      7.260        yes             256     1024      7.830        yes
     32     4096      7.861        yes             256     4096      7.778        yes
     64       64     11.595        yes             512       64      6.543        yes
     64      256      6.480        yes             512      256      7.859        yes
     64     1024      5.629        yes             512     1024      7.506        yes
     64     4096      8.943        yes             512     4096      6.250        yes
    128       64      8.521        yes            1024       64      8.577        yes
    128      256      7.812        yes            1024      256      7.151        yes
    128     1024      6.745        yes            1024     1024      8.316        yes
    128     4096      7.497        yes            1024     4096      8.706        yes
```

Two planted-defect tests deserve a note, because the first version of one of them
**failed and was right to**. It flipped `d[0]`'s low bit and the fold did not
move — not a kernel bug: `d[a]` is only read when column `a` is in the solution
`u`, so a defect on an unselected column is silent by construction. Chasing that
turned up the load-bearing fact that **`y = h` is the `u = 0` point, where no
`d[a]` and no `J[a][b]` is read at all** — so every probe state built as "some
branch's `h`" was exercising the F2 solve and skipping the entire phase
polynomial, silently, with everything still agreeing. Both the tests and the
benchmark's probe states now go through `AffineDesc::point(u)` at a nonzero `u`,
and observability is a **checked precondition**: the host twin has to see the
plant move the rotation code before the device is asked whether it sees the fold
move.

---

## 3. Measured numbers

**Machine:** RTX 4090 Laptop GPU (16 GiB, 15.57 GiB visible, driver 580.142), 32
logical CPUs. **The CPU was heavily loaded throughout — loadavg 32 to 46 — by
other campaigns on this box.** Every CPU row below carries the loadavg it was
taken under. See the contamination note; it is not small.

Timing method: best-of-N passes on **both** arms (the GPU was already timed
best-of; the CPU arms were single-shot in the first run, which produced a table
where the strictly-better packed arm came out slower than the arm it improves on
— that was the machine, not the code, and best-of-N is the fix). The worst pass
is printed next to the best. Upload is timed separately and never amortized into
a per-fold number, because a batch is uploaded once and folded for many `y`.

### Synthetic batches

| workload | branches | GPU ms/fold | CPU serial | CPU 32-shard | vs serial | vs 32-shard | loadavg |
|---|---:|---:|---:|---:|---:|---:|---:|
| n=16, k=12 | 100,000 | **0.263** | 88.3 | 9.64 | 336x | 37x | 31–33 |
| n=32, k=24 | 1,000,000 | **6.744** | 2670 | 197.5 | 396x | 29x | 29–37 |
| n=48, k=40 | 200,000 | **3.104** | 1093 | 82.2 | 352x | 27x | 26–37 |

The CPU arms are `holon::mesh::fold_amplitude` over the same `BranchSource`, and
`cpu::fold_packed`, which is the same fold with `y` packed once instead of once
per branch — the trait's `&[bool]` signature otherwise costs an O(n) repack per
branch, and quoting a speedup against that would be quoting it against a baseline
defect. The two agree to within noise at these sizes; the tighter is quoted.

### Real circuits (random Clifford+T through `run_pruned`)

| workload | branches | GPU ms/fold | CPU packed, serial | CPU packed, 32-shard | CPU on `PrunedSum`, serial | vs packed serial | loadavg |
|---|---:|---:|---:|---:|---:|---:|---:|
| n=20, T=16, merging on | 1,024 | **0.047** | 0.424 | 17.0 | 2.285 | 9.0x | 33.6 |
| n=24, T=18, merging on | 896 | **0.059** | 0.485 | 1.406 | 1.984 | 8.2x | 38.6 |
| **n=24, T=18, merging off** | **262,144** | **0.487** | **100.5** | **14.87** | **531.8** | **207x** | **34–36** |

All rows bit-identical against all three CPU arms.

**The honest reading of the first two rows is that the pruner is very good.** At
n=24, T=18 with merging on it collapses 262,144 naive branches to 896, and 896
branches is a workload one CPU core finishes in 0.4 ms. At that size the GPU is
launch-overhead-bound and wins by 8–9x, not by hundreds. The third row is the
same circuit with merging off — `run_naive`'s branch set, 262,144 **real**
branches, and the size at which this crate is the right tool:

* **207x** against the packed CPU fold on one core,
* **1093x** against `holon`'s own `PrunedSum` fold on one core (the number a
  holon user pays today, through `Vec<Vec<bool>>` elimination),
* **30.6x** against the packed CPU fold across all 32 shards — and see the
  contamination note, because that last one is the inflated column.

Worth recording: `run_naive` at n=24, T=18 took **2.41 s** to produce 262,144
branches, while the merged run producing 896 took **100 s**. The merge pass, not
the expansion, is where a `run_pruned` call spends its time.

### GPU scaling — where the floor is

| branches | ms/fold | ns/branch |
|---:|---:|---:|
| 1,000 | 0.115 | 114.5 |
| 10,000 | 0.125 | 12.5 |
| 100,000 | 0.864 | 8.6 |
| 300,000 | 2.100 | 7.0 |
| 1,000,000 | 6.558 | 6.6 |
| 3,000,000 | 16.233 | 5.4 |

The launch-plus-copy floor is **~0.115 ms**. Marginal cost settles at
**5.4–6.6 ns per branch** at n=32, k=24.

### CONTAMINATION — read before quoting any of the above

The CPU was at loadavg 32–46 on 32 logical cores for every measurement. Concretely:

* The 32-shard CPU arm gets only about **13.5x** over its own serial arm at 10^6
  branches (2670 ms to 197.5 ms), where an idle 32-core box would plausibly give
  something closer to 25–32x. **The "vs 32-shard" column is therefore inflated by
  roughly 2x.** A defensible idle-machine estimate for GPU vs a fully parallel
  CPU on the 10^6-branch workload is **13–20x, not 29x.**
* On the small real-circuit workloads the 32-shard arm is *far slower than
  serial* (17.0 ms vs 0.424 ms at 1024 branches) — thread spawn against a
  saturated run queue, 40x of pure contention. Those rows quote serial, and the
  32-shard column there should be read as a measurement of the machine.
* The serial arm is less affected than the parallel one but is not clean either:
  worst-pass/best-pass spreads reached 3975/2774 ms.
* The **GPU numbers are not contaminated** — the GPU was idle apart from this
  work — and neither is anything in section 2. Determinism is a bit-comparison
  and does not care about load at all.

---

## 4. The parity fence — measured, and it fires

`Cyc::normalize` divides out only *even* powers of two, so a nonzero value has
**two** normalized faces, one per parity of `m`: `1 = ([1,0,0,0], m=0)` and
`1 = ([0,1,0,-1], m=1)` are both fixed points. The sequential CPU fold's final
`m` carries the parity its own path reached; this crate's carries the parity of
`M = max_b m_b`.

**The sufficient condition is parity-uniformity of the `m_b`, not
exponent-uniformity.** This crate first stated the stronger condition and was
corrected by its own measurement: a real `run_pruned` batch (n=24, T=18) came
back with **mixed exponents and bit-identical results**, which the stronger claim
could not account for. `GpuBatch::parity_uniform` is the flag that matters;
`exponent_uniform` is kept because it is cheap and informative, not because
anything rests on it.

When parities do differ, the fence is real and it is **not** hypothetical.
`a_mixed_exponent_batch_is_value_equal_and_the_struct_question_is_measured`
builds batches mixing exponent *parities* deliberately and reports:

> **struct-equal on 7 of 18 probes; value-equal on 18 of 18; equal under
> `mesh::canonicalize` on 18 of 18.**

So on a mixed-parity batch the GPU and the CPU agree on the number and can
disagree on which of the ring's two faces they wear. Three things stay true
regardless:

* every launch shape still agrees with every other launch shape, bit for bit —
  that is a property of the GPU reduction alone and the fence does not touch it;
* the values are equal, checked by posting the credit against the debit through
  the ledger's own addition;
* `holon::mesh::canonicalize` — which already exists, and which the mesh's own
  header offers as the durable remedy — reconciles the faces.

Nothing in this crate applies `canonicalize` silently. The tests compare structs
directly and report the parity flag alongside, because canonicalizing the
comparison would hide the question rather than answer it.

---

## 5. VRAM limits

Device footprint is `8*(n + 1 + kmax + 2 + 8) + 4` bytes per branch. Measured
against 15.32 GiB free:

| n | k | bytes/branch | predicted max branches | upload verified at |
|---:|---:|---:|---:|---:|
| 8 | 8 | 220 | 74,750,361 | 4,000,000 |
| 16 | 16 | 348 | 47,255,975 | 4,000,000 |
| 32 | 32 | 604 | 27,226,952 | 4,000,000 |
| 48 | 48 | 860 | 19,122,185 | 4,000,000 |
| 64 | 63 | 1,108 | 14,842,129 | 3,710,532 |

The verified column is a quarter of the prediction, which is what a caller should
actually target: the host-side transpose buffers cost the same again in system
RAM, and the driver needs headroom. **At the representation's bounds (n=64 orbitals
is one `u64` occupation string, k=63 the widest half-filling it admits) roughly 3.7M
branches upload comfortably and ~14.8M is the arithmetic ceiling.** Those two are
representation choices, not budgets: a wider string type moves them, and nothing else
here refuses on them. Nothing here
was pushed to an out-of-memory failure; the ceiling column is arithmetic, and it
is labelled as such.

---

## 6. What is owed

1. **Tableau kernels were not built.** Deliverable 1's option (b) — batched
   packed-tableau H/S/CX with `__popcll` measurement-sign accumulation — does not
   exist here, and therefore neither does deliverable 3(b). Option (a) was the
   deeper port and took the time.
2. **Only one real circuit family was measured** — random Clifford+T from
   `random_circuit`, at three sizes. Structured circuits (the ones with the
   interesting merge behaviour) were not swept, and the branch-count regime where
   a real merged batch stays large was not found; the 262,144-branch row gets
   there by turning merging off.
3. **Clean-machine numbers.** Every CPU figure here was taken at loadavg 32–46.
   The whole comparison should be re-run on an idle box before any speedup is
   quoted outside this document.
4. **`AffineDesc::from_branch` depends on `Affine::canon_key`'s byte layout.**
   The decoder length-checks and the conformance test drives it against
   `Affine::amplitude`, so a format change fails loudly rather than silently —
   but the durable fix is accessors on `Affine`, which is another lane's file.
5. **Ragged `k` wastes device memory.** `jrow` is allocated at `kmax` for every
   branch. Batches with a wide spread of `k` pay for the worst one.
6. **The kernel is at roughly 1% of the device's integer peak.** It is
   local-memory bound on the augmented-row scratch array. Forward-elimination
   plus back-substitution instead of full Gauss-Jordan would roughly halve the
   inner loop; a register-resident variant for small `n` would do better still.
   Neither was attempted — correctness and the determinism certificate came
   first.
7. **`ci-gates.sh` cannot reach this crate**, deliberately (its own empty
   `[workspace]` table), because its tests need a CUDA device. The suite is run
   by hand and its result is recorded here. That is a recorded commitment, not a
   machine-enforced gate, and it is not advertised as one.

---

## Running it

```bash
cd engine/crates/holon-gpu
cargo test --release -- --test-threads=1     # 16 tests; needs a CUDA device
cargo run --release --bin gpu-bench sweep    # the table in section 3
cargo run --release --bin gpu-bench shapes   # the 24-configuration determinism sweep
cargo run --release --bin gpu-bench scale    # the scaling curve
cargo run --release --bin gpu-bench caps     # the VRAM table
```

`tests/ring.rs` needs no GPU. Everything else does.
