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

## What is NOT claimed

* **Not that GPU dispatch is safe for bit-gated work.** It is not, and D0 says so.
  The class is now declarable and checked; it is not chosen by a crossover.
* **Not that the full Davidson loop belongs on the device.** At this size one
  sigma is ~15 ms of compute against ~0.5 ms of PCIe, so moving the driver buys
  about 3% and costs the ability to run one engine under two devices. The
  question is open for larger spaces and is not answered here.
* **Not that the CPU baseline was re-measured at 32 threads.** See above.
* **Not that any committed table should be regenerated.** They declare `Cpu` and
  they stay that way; a GPU-built table would be a different artifact.

## To re-run

```bash
cd engine/crates/holon-gpu
cargo test --release --test fci_sigma --test gpu_lease -- --test-threads=1
taskset -c 0  ./target/release/examples/fci_bench --species O,O,O --core-type P --reps 20
taskset -c 16 ./target/release/examples/fci_bench --species O,O,O --core-type E --reps 20
```

The crate is outside the workspace and `ci-gates.sh` cannot reach it, deliberately:
a gate that needs hardware CI does not have must not be able to silently not-run
inside a green workspace build.
