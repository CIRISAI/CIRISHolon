# GPU PRODUCTION — the FCI sigma is in holon-chem's solve path, and the artifact declares its class

*gpu-production lane, 2026-09-01. Provenance (binary/kernel/PTX sha256, repo HEAD,
build exit status, driver, device) in `LAUNCH_HEADER.txt`. Logs: `ooo_pcore.log`,
`ooo_ecore.log`.*

## Verdict

**ADOPTED.** The CUDA sigma is a production arm of `holon-chem`'s determinant
solve, reached through a declared device class rather than a build flag. The
GPU is a leasable resource with a probe that really allocates, and a card yanked
out from under a live lease is CONVICTED rather than errored. **D0 is not
relaxed**: 91.0% of the sigma entries still differ bitwise between the classes,
every committed table still declares `Cpu`, and dispatch still may not move a
bit-gated workload between them. Adoption made the choice explicit and
per-artifact; it did not make it automatic.

## What was built

| piece | where | what it is |
|---|---|---|
| the contract | `holon-chem/src/sigma_op.rs` | `SigmaOp` (one application of `H`, on a declared class), `SigmaProvider` (a source of operators bound to ONE class), `bit_identity_over_runs` (the adoption gate) |
| one definition of the class | `holon-device` (new crate) | `no_std`, zero-dep. `holon-chem` (ships to a browser) and `holon-resource` (sits under everything) both need to name the class and cannot depend on each other |
| the driver, unchanged | `holon-chem/src/tier.rs` | `davidson_eigh_from_op` — the same body, parameterised by the operator instead of by the three arguments that named the host one |
| the whole solve on one class | `holon-chem/src/fci.rs` | `solve_determinant_with`; `Solution::device` |
| the device arm | `holon-gpu/src/fci.rs`, `kernels/fci_sigma.cu` | three cuBLAS GEMMs and two atomics-free gather kernels |
| the VRAM probe | `holon-gpu/src/probe.rs` | D2's *attempt the thing*: it allocates on the device and frees it. `holon-resource` said the GPU owner must supply this; this is that |
| the GPU as a lease | `holon-gpu/src/lease.rs` | probe → lease → USE, as three steps, because the gap between them is what a lease is a receipt about |

## The measurement, reproduced three times

| quantity | G2 prototype (2026-08-30) | prototype re-run (2026-09-01) | in-crate (this entry) |
|---|---:|---:|---:|
| relative agreement with `sigma_direct` | 3.033e−15 | identical to the digit | **3.033e−15** |
| entries differing BITWISE | 188,363 / 207,025 (91.0%) | identical | **188,466 / 207,025 (91.0%)** |
| GPU kernel only | 65.7 sigma/s | — | **68.4 sigma/s, 331.6 GFLOP/s FP64** |

The 103-entry difference is the integrals arriving by different routes (exported
`f64` versus in-crate). Reported, not smoothed.

## Placement, declared (M-PLACEMENT-LOTTERY)

Both arms pinned; the pin **echoed as the process actually has it** — the binary
refuses a `--core-type` that disagrees with its real affinity. The P/E split is
read from `/sys/devices/cpu_{core,atom}/cpus` and cross-checked against the
MISFITS entry, refusing if they disagree.

| arm | device class | core class | sigma/s |
|---|---|---|---:|
| GPU, kernel only | gpu | n/a | **68.4** |
| CPU, `sigma_direct`, 1 thread, CPU-time | cpu | P (cpu 0) | 1.11 |
| CPU, `sigma_direct`, 1 thread, CPU-time | cpu | E (cpu 16) | 0.81 |

Both at loadavg 65–68. wall/CPU-time was 1.47 (P) and 1.74 (E): a wall-clock CPU
arm reads 32–43% slow here purely from descheduling. **E/P is 0.73 on this
kernel**, not the 57% the MISFITS entry records for the tableau workload — the
scaling factor is per-kernel and the entry's number is not a machine constant.

**The 32-thread CPU aggregate is NOT re-measured.** This machine is carrying a
live ozone tabulation on 27 cores; a 32-thread arm would be measuring the
neighbours. The 3.2× headline remains G2's, and quoting 68.4 against 1.11 would
be a 61× no scheduler would deliver.

## Gates, all firing

| gate | result |
|---|---|
| agreement before any timing | 3.033e−15 relative; a fast wrong answer prints nothing |
| determinism, 5 runs, per class, BITWISE | cpu YES, gpu YES — measured on the operator that runs, not inferred from the kernel being atomics-free |
| D0, mixed-class provider | REFUSED (carrier: the honest provider succeeds on the same problem) |
| D2, VRAM probe attempts | passes by allocating, refuses 4× the card, refuses a non-VRAM question |
| D3b, lease past the boundary | REFUSED, no ledger entry opened |
| D9, the yank | 5,080 MiB lease granted by a real allocation; competitor takes 13,633 MiB; the USE is **CONVICTED**, books balance |
| D12, 10× mis-registration of the LIVE rate | FIRES (688.7 claimed, 67.9 observed, tolerance 1.615); honest control `Consistent` at the same moment |

## The defect this benchmark found in itself

The first version registered the WARM mean and spot-checked it against a COLD
reading — 26% apart, from GPU clock state and nothing in the kernel — and D12
**convicted a correct entry**. That is the registry's own version of a false
reap. The fix is the rule D12 already states and the benchmark was not
following: a spot-check RE-TIMES the workload rather than comparing the
registration to whatever number is lying around. Both readings are now warm, the
warm-up discard is declared, and the cold number is printed rather than dropped.

## The question the lane was asked to evaluate: should the Davidson loop move device-side?

**Measured, and the answer is NO — not because the port is hard, but because the
port is not the lever.** `holon-gpu/examples/fci_bench.rs --davidson 60`, on
`(O,O,O)`, pinned to one P-core at loadavg 66:

| quantity | measured |
|---|---:|
| 60 Davidson iterations | 24.602 s |
| one iteration | **410.0 ms** |
| one device sigma | **14.7 ms** |
| the sigma's share of an iteration | **4%** |
| the HOST-SIDE driver's share | **96%** |

The estimate this replaces was "PCIe is 0.5 ms against 15 ms of compute, so
moving the driver buys about 3%". That counted the transfer and forgot the
driver. The device made the sigma so fast that the host loop now dominates it by
a factor of 25.

**The mechanism, located.** `tier::davidson_eigh_from_op` rebuilds the ENTIRE
`m × m` subspace matrix every iteration:

```rust
for i in 0..m { for j in 0..m { sub[i*m+j] = dot_t(&basis[i], &hbasis[j]); } }
```

Only the new row and column changed. At `m = 48` and 207,025 determinants that
is 2,304 dot products over a 1.6 MB vector — 477 M multiply-adds per iteration,
single-threaded, to recompute values that were already correct.

**It is a RATE gap, not a work gap, and that distinction picks the fix.** The
sigma is the bigger computation (4.85 GFLOP against ~0.95 GFLOP for the subspace
rebuild at `m = 48`); it finishes first because it runs at 331 GFLOP/s on the
device while the dots run at about 5 GFLOP/s on one contended core. So there are
three levers and porting the loop is the last of them:

1. **cache the subspace matrix** — carry the previous iteration's entries and
   compute only the new row and column. Expected bit-identical: the entries are
   the same dot products of the same operands, and re-symmetrising an
   already-symmetric pair is exact in IEEE (`(a+a)*0.5 == a`). Expected
   ~25× on the host share at this size;
2. **vectorise or parallelise the dots** — they are embarrassingly parallel and
   the driver is single-threaded by construction (`holon-chem` has no thread
   dependency, deliberately);
3. **only then** consider moving the loop device-side.

**None of the three is done here, and (1) must not be done casually.** It is a
solver change, and every committed table is keyed on the Davidson path's trailing
bits — `w1_masks`'s banked Be bit pattern and `water`'s committed table are the
gates that would catch it. "Expected bit-identical" is a prediction, and the
gates are what would turn it into a fact.

**The adoption consequence, which is the decision-relevant part.** The table
generator's parallelism is at the NODE level: 32 concurrent single-threaded
solves. One GPU serialises across all 32 of them. So the device arm helps a
SINGLE large solve and does not help a table — which confirms G2's "adopting the
GPU idles 32 cores rather than adding to them" at the SOLVE level, not just at
the sigma level, and it is a stronger statement than the sigma ratio alone
supports.

**Measurement caveat, stated rather than buried.** The 96% is the single-core
figure, taken pinned per M-PLACEMENT-LOTTERY. Pinning does not change the
driver's parallelism — it has none — but it does expose it to contention:
wall/CPU-time on the CPU arm in the same run was 1.47, so roughly a third of the
host time is descheduling on a machine at loadavg 66.

## What is NOT claimed

* **Not that GPU dispatch is safe for bit-gated work.** It is not, and D0 says so.
  The class is now declarable and checked; it is not chosen by a crossover.
* **Not that the full Davidson loop belongs on the device.** It does not, and the
  reason is measured above rather than estimated: the host driver is 96% of an
  iteration and the sigma is 4%, so the lever is the driver's quadratic subspace
  rebuild, not a port.
* **Not that the CPU baseline was re-measured at 32 threads.** See above.
* **Not that any committed table should be regenerated.** They declare `Cpu` and
  they stay that way; a GPU-built table would be a different artifact.

## To re-run

```bash
cd engine/crates/holon-gpu
cargo test --release --test fci_sigma --test gpu_lease -- --test-threads=1
taskset -c 0  ./target/release/examples/fci_bench --species O,O,O --core-type P --reps 20
taskset -c 16 ./target/release/examples/fci_bench --species O,O,O --core-type E --reps 20
taskset -c 0  ./target/release/examples/fci_bench --species O,O,O --core-type P --reps 20 --davidson 60
```

The crate is outside the workspace and `ci-gates.sh` cannot reach it, deliberately:
a gate that needs hardware CI does not have must not be able to silently not-run
inside a green workspace build.
