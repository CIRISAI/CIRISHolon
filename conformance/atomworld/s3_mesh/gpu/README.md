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

### The defect: a with-round-trip figure faster than the kernel it contains

The record carried two GPU rows, `whole kernel 65.7` and `incl. host round trip 69.8`.
**69.8 sigma/s is 14.33 ms and 65.7 is 15.22 ms, so the row that includes the round trip
was reported as the faster one.** The instrument times the round trip over a block
containing the c upload and the sigma download (the second `cudaEventRecord` pair in
`sigma.cu`), so it cannot produce that ordering. One of the two rows was wrong.

The obvious reading is that the labels were swapped. **That reading is wrong**, and the
thing that discriminates is which numbers came off one printed line. `sigma.cu` prints
sigma/s *and* GFLOP/s together on the kernel-only line, and prints sigma/s alone on the
round-trip line. The banked pair ties:

    65.7 sigma/s  ->  15.221 ms  ->  318.6 GFLOP/s      banked GFLOP/s: 318.4
    69.8 sigma/s  ->  14.327 ms  ->  338.5 GFLOP/s      not banked anywhere

and today the same line prints the same three-way tie (67.5 / 14.820 / 327.2). So the
headline row is the kernel-only line, transcribed correctly and labelled correctly, and
**the single defective datum is 69.8** — a number the instrument could not have printed
next to 65.7, and whose provenance is not recoverable. Aug 30's true round trip would
have been near 15.5 ms (about 64.4 sigma/s), by today's measured 0.32 ms delta.

Both readings explain the impossible ordering; only one explains the GFLOP/s. Taking the
first would have moved a correct headline and left the real defect standing. **A check
that cannot separate two causes says look, not conclude** — the impossible ordering was a
detector, and the GFLOP/s tie was the discriminator.

**No claim moves.** The headline and its GFLOP/s are sound, the ratio is 65.7/20.8, and
the 3.2x is untouched. The 69.8 row is replaced by today's measured round trip with its
date, rather than by a reconstruction of a number nobody can source.

### What this says about contention, which was an open question

The GPU arm was re-measured at **loadavg 61**; the banked run was taken at loadavg 18–32.
The kernel-only figure moved 65.7 → 67.5, i.e. **+2.7%, and in the wrong direction for a
contention story** — the busier box read faster. The GPU arm is essentially not exposed to
host contention, which is what the 14.8 ms device / 0.32 ms host split predicts.

That matters because it converts a borrowed argument into a measured one. bigqvm-demo's
third M-PLACEMENT-LOTTERY rung is that contention is **a bias with a sign, not noise**,
because two arms lose different amounts to it. For this comparison the asymmetry is now
measured rather than assumed: the GPU arm loses ~nothing to host load, the CPU arm is
fully exposed to it, so **the whole of any contention bias sits in the CPU arm and its
sign inflates the ratio.** 3.2x is therefore an upper bound on the quiet-box ratio. How
much of an upper bound is unmeasured and needs a quiet window; the CPU arm's own two
banked readings (20.8 at loadavg 32, 17.2 at loadavg 18) are 1.21x apart and also ordered
against a simple load story, so they do not bound it either.
