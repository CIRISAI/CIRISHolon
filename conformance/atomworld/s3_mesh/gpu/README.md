# SATURATION-3 G2 — the measured instruments, rescued into the record

These are the bytes that produced the G2 numbers banked in `conformance/BENCHMARKS.md`
and `conformance/atomworld/SATURATION3_RESULTS.md` on 2026-08-30. They are kept here
because until 2026-09-01 both records cited them as `scratchpad/s3gpu/...`, **and that
citation did not resolve.**

## Why the citation did not resolve, which is the transferable part

`scratchpad/` reads as the repository's own scratch directory
(`/home/emoore/CIRISOntology/scratchpad/`, where a neighbouring lane's cited
`scratchpad/qasm/BATTLERIG.md` does resolve and still exists). These files were never
there. They lived in the **per-session** scratchpad —
`/tmp/claude-1000/<project>/<session-id>/scratchpad/s3gpu/` — which is keyed by session
id and is not durable.

So the citation was not broken by a deletion. **It never pointed at a durable location
at all**, and a reader following it would have looked in the wrong directory and
concluded the instrument was lost. The banked result was one session-cleanup away from
being unreproducible, and nothing in the record would have shown it.

This is **M-STALE-INSTRUMENT** in a form that entry does not yet cover. Its founding case
was instruments that were working-tree-only — present, runnable, merely uncommitted. These
were not in the working tree at all, and the citation named a directory they had never
been in, so the usual check (does the repo have it?) returns a clean "no such path" that
reads as a typo rather than as a lost instrument.

The rule this earns: *a citation into a scratch directory must name a durable path, and
the way to find out whether it does is to follow it from outside the session that wrote
it.* Every path in this directory is now inside the repository.

## What each file is

| file | what it measures |
|---|---|
| `probe.cu` | the device's own ceilings — FP64 FMA throughput, FP64 DAXPY bandwidth, PCIe round trip for one CI vector. Bounds the kernel before it is written |
| `sigma.cu` | **the GPU arm.** `sigma = H c` at the (O,O,O) scale as three GEMMs plus two custom gather kernels; checks agreement against the CPU reference, runs the determinism gate, then times kernel-only and with-host-round-trip |
| `gemm.cu` | the cuBLAS FP64 GEMM rate at the three shapes the reformulation actually uses |
| `cpu_fair.py` | **the fair CPU arm.** The IDENTICAL three-GEMM reformulation on the CPU through OpenBLAS, so the GPU's win is not quoted against a hand-written loop |
| `cpu_blas.py` | the same, tightened — bounds what a reformulated CPU sigma could reach |

Compiled binaries are deliberately NOT kept: they are build products, and `nvcc -O3
-arch=sm_89` on the sources reproduces them.

## The problem data is PINNED, not committed

`ooo.bin` (4.4 MB) is the exported real problem — index structures and integrals from
`pair::geometry_problem`, plus the CPU reference sigma the GPU answer is checked against.
It is not committed, because its generator is:

    engine/target/release/examples/s3_sigma_export        # holon-chem, committed

    sha256  9bd4d3523fc5b8a4397bcd0e93583fb8b2e1f33c7b1122058f1790650c0f1710

A re-export that does not match that hash is a different problem, and any number taken
against it is a different measurement. Check the hash before comparing.

## Reproduction, 2026-09-01

`./sigma ooo.bin 20` on the same host, re-run to settle a labelling inconsistency in the
record (below). The correctness figures came back **identical**, not merely close:

    max |sigma_gpu - sigma_cpu|   4.547474e-13        banked: 4.547e-13
    relative to that scale        3.033e-15           banked: 3.033e-15
    entries differing BITWISE     188363 of 207025    banked: 188,363 (91.0%)
    five repeat runs bit-identical  YES               banked: YES

So the agreement figure and the 91.0% bitwise divergence that founded M-DEVICE-CLASS are
reproduced from the instrument, a year-zero re-run rather than a citation.

The timings moved, and by different amounts — and re-running settled a real defect in
the record, though **not the one the arithmetic first suggested.**

| | 2026-08-30 | 2026-09-01 |
|---|---|---|
| kernel only | 15.22 ms (65.7 sigma/s, 318.4 GFLOP/s) | 14.820 ms (67.5 sigma/s, 327.2 GFLOP/s) |
| with host round trip | *not recoverable* — see below | 15.142 ms (66.0 sigma/s) |

### The defect, diagnosed twice — the second time correctly

The record carried two GPU rows, `whole kernel 65.7` and `incl. host round trip 69.8`.
**69.8 sigma/s is 14.33 ms and 65.7 is 15.22 ms, so the row that includes the round trip
was reported as the faster one**, which one run of the instrument cannot produce: it times
the round trip over a block containing the c upload and the sigma download.

**FIRST DIAGNOSIS (wrong): the labels were swapped.** Rejected, correctly, because
`sigma.cu` prints sigma/s *and* GFLOP/s together on the kernel-only line and sigma/s alone
on the round-trip line, and the banked pair ties — 65.7 sigma/s is 15.221 ms is 318.6
GFLOP/s, which is the banked 318.4, while 69.8 would be 338.5 and is banked nowhere. So
65.7 and 318.4 did come off the kernel-only line.

**SECOND DIAGNOSIS (also wrong): 69.8 was an unsourceable datum.** That is what the
GFLOP/s tie appears to imply, and it is what this file said for about an hour. It is
wrong, and what refuted it was a neighbour's independent instrument plus five repeats.

**WHAT IS ACTUALLY TRUE: on a loaded box neither timing is stable, and the ordering
between them carries no information.** Six runs of the same binary against the same
`ooo.bin`, minutes apart, at loadavg 61–72:

| run | kernel only | with host round trip |
|---|---|---|
| 1 | 14.820 ms (67.5) | 15.142 ms (66.0) |
| 2 | 18.168 ms (55.0) | **89.822 ms (11.1)** |
| 3 | 17.419 ms (57.4) | 14.903 ms (67.1) |
| 4 | 17.686 ms (56.5) | **80.959 ms (12.4)** |
| 5 | 17.007 ms (58.8) | **81.067 ms (12.3)** |
| 6 | 14.867 ms (67.3) | 14.904 ms (67.1) |

Kernel-only spans **1.23x** (14.820–18.168 ms). The round trip is **bimodal**: three runs
at ~14.9 ms and three at 81–90 ms, a **6.0x** spread. The slow mode's excess over the fast
mode is **66.1 ms against a measured PCIe round trip of 0.50 ms — 132x.** No device-side
mechanism moves 1.58 MiB that slowly. That is the host thread being descheduled at loadavg
72, and the same cause inflates kernel-only timings whenever the host cannot keep the
launch queue fed, because `cudaEventElapsedTime` across 20 back-to-back reps charges host
gaps to the device clock.

So 65.7 and 69.8 are **two draws from a contaminated distribution**, not one good number
and one corrupt one, and an impossible-looking ordering between two independently
contaminated blocks is expected rather than diagnostic.

Independent corroboration, same day, different instrument: gpu-production's `fci_bench`
(the holon-gpu production path, pinned with `taskset`, its own launch header) reported
`kernel only 58.9` / `incl. host round trip 65.2` — **the same impossible ordering** — and
in the same run measured the GPU rate over five timing runs at **69.40 ± 1.01**. Its
registry spot-check then convicted its own 58.9 reading as outside tolerance of that mean.
Two instruments, one pattern.

### What this does to the contention argument, which was the question that started it

bigqvm-demo's third M-PLACEMENT-LOTTERY rung says contention is a **bias with a sign**, not
noise, because two arms lose different amounts to it, and that a margin argument cannot
answer a bias. This file briefly claimed the asymmetry had been measured in the GPU arm's
favour — one re-run at loadavg 61 read 67.5 against the banked 65.7, "+2.7% and in the
wrong direction for a contention story", concluding the GPU arm was essentially unexposed
to host load and therefore that 3.2x was an upper bound on the quiet-box ratio.

**That claim was built on a single draw from the distribution above and does not survive
five more.** Runs 2–5 read 55.0–58.8 sigma/s kernel-only, below the banked 65.7, and the
round trip loses 66 ms outright in half the runs. The correct statement is the opposite in
spirit:

* The **device compute** is unexposed to host load — 4.849 GFLOP at a device-set rate.
* The **measured GPU throughput** is host-exposed twice over, through launch scheduling and
  through pageable-memory copies, and on this box that exposure is larger in relative terms
  than anything measured on the CPU arm (6.0x vs the CPU arm's 1.21x between its two
  banked readings).

So the direction of the bias between the two arms is **not established**, and the honest
position is that **both arms need the quiet window**, not just the CPU one. bigqvm's
warning applies more sharply than when they gave it, not less. What survives untouched is
that the ratio was quoted against the *faster* of the two CPU readings, and that the G2
verdict (adoption-condition MET, adoption DEFERRED) rests on device-class grounds rather
than on the size of the win.

**The correctness figures are unaffected by any of this** — they are bit comparisons, not
timings, and they reproduced identically. M-DEVICE-CLASS's founding measurement stands.

**The lesson, which is the transferable part:** a timing taken once on a loaded box is a
draw, not a measurement, and two such draws will sometimes order impossibly. The tell was
available before any of this — the record carried a *single* number per row with no spread
beside it. RESOURCE_DESIGN section 9 Q4 already requires mean AND spread for a registry
entry; the benchmark record did not hold itself to its own crate's rule.
