# What the dE3 subtraction costs — priced in WORK UNITS, after wall clock lied by up to 50x

Instrument: `holon-chem/examples/s3_pair_cost.rs`. Value-only pair solves (the emitter is
fenced from emitting derivatives of the subtracted quantity, so there is nothing to spend
duals on). CPU time from `/proc/self/stat` fields 14+15; Davidson iteration count echoed per
solve; loadavg stamped at both ends per the fleet standard.

## THE VERDICT: the Cl-Cl 22.9x anomaly was SCHEDULING. The solver is fine.

3 reps each, loadavg 64.3 -> 65.3:

| pair | wall spread | **CPU spread** | CPU ms | Davidson iters |
|---|---:|---:|---:|---|
| H-H | 1.28x | *below resolution* | ~10 | 4, 4, 4 |
| O-H | 1.04x | **1.00x** | 50 | 17, 17, 17 |
| H-Cl | 10.50x | **1.00x** | 260 | 16, 16, 16 |
| O-O | **50.68x** | **1.06x** | 330–350 | 36, 36, 36 |
| Cl-Cl | 8.58x | **1.08x** | 1550–1680 | 20, 20, 20 |

**Every pair: identical iteration counts, identical energies to all printed digits, CPU
stable to within 1.08x, and wall clock varying by up to 50.68x.** A separate 5-rep Cl-Cl run
read wall/CPU ratios from 2.73 to 14.30 — the process was descheduled for 63% to 93% of its
wall time.

So the discriminator the lead asked for resolves in the benign direction: **CPU stable while
wall varies means contention, not the solver taking different paths.** The pair table is
fine. The identical `davidson_iters` settles it outright — the solve is doing the same work
every time, and no wall number was ever describing the computation.

**The banked BLAS-spin-thread control was NOT RUN, and should not be.** `holon-chem` has no
blas/lapack/ndarray/rayon dependency and no thread spawn in `fci.rs` or `pair.rs`; the solve
is single-threaded pure Rust. `OPENBLAS_NUM_THREADS` cannot discriminate anything here. A
control that cannot fire is not a control.

## The cost model, corrected by 15–35x

| pair | what I reported from wall | **work units (CPU)** |
|---|---:|---:|
| Cl-Cl | 23.6 s median, 55 s in one draw | **1.6 s** |
| O-O | 0.4–32 s | **0.34 s** |
| H-Cl | 0.28–20 s | **0.26 s** |
| O-H | 0.07–1.4 s | **0.05 s** |

My earlier "(Cl,Cl,Cl) subtraction is 30% to 7x of the trimer, DOMINANT" was built on wall
clock. **It is withdrawn.**

## What is still NOT known, and it is the number the design decision actually needs

**The ratio is not recoverable yet, because the two sides are priced in different units.**
G0's per-node trimer costs (0.21 s / 6.5 s / 39.8 s) are WALL-CLOCK numbers taken on a
loaded box. Dividing a work-unit pair cost by a wall-clock trimer cost is comparing two
different quantities and would reproduce, in miniature, exactly the defect gpu-prod found in
the GPU registry — an entry and its check agreeing with each other about a quantity nobody
receives.

**So: the trimer side must be repriced in work units before any subtraction-vs-trimer ratio
means anything.** That is one short run and it is mine; I am not reporting a ratio until
both halves are the same quantity.

What survives unchanged: the **axis cache is still a requirement**, because it is a
structural claim about how many solves are needed (`nx + ny` cached axes plus one per node,
versus three per node) and does not depend on what a solve costs.

## The instrument's own limit, announced rather than reported

H-H's CPU spread printed as **2e10x** — from readings of 0.0 and 20.0 ms. That is an
impossible value, and an impossible value is the instrument saying it has run out of
resolution rather than reporting a result: `/proc/self/stat` ticks at 10 ms and H-H takes
about that long. H-H's work-unit cost is therefore **not measured here** and needs a
different method (batch N solves and divide). Credit to the mesher/bigqvm exchange for the
detector: a quantity with a known bound announcing an impossible value is stronger than any
threshold, and free wherever the bound exists.

## How this file's first version was wrong, kept because it is the lesson twice over

1. It quoted **one wall-clock timing per pair** and reported H-Cl at 19,970 ms; the next run
   of the identical call read 216 ms. A single timing on a contended box is a draw.
2. Repaired to repeats and spread — and **still wrong**, because repeats of the wrong
   quantity give you a well-characterised distribution of the wrong quantity. The wall-clock
   spread was real and reproducible and told me nothing about the solver.

The fix was not more repetition. It was **changing what is measured**, which is the lead's
own vindicated lesson from B1b: price in work units, and keep wall clock as a
condition-stamped side channel.
