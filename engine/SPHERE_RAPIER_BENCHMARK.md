# Sparse sphere benchmark against Rapier

Date: 2026-08-22
Host: Mac mini, Apple M4 (10 cores: 4 performance + 6 efficiency), 16 GB
Build: Rust release profile, LTO, one codegen unit
Engines: PR #7 `SparseSystem<96, 270>` and Rapier 0.35.2 `f64`

## Harmonic benchmark

The primary benchmark is a sparse zero-rest-length spring network. Both engines receive
the same 96 sphere-distributed points, 270-edge graph, stiffnesses, weighted-degree
masses, initial state, timestep, and simulated duration. There are no colliders and no
boundary projection in this part.

This system obeys `M x'' = -L x`. Accuracy is measured against the independent closed
form obtained by diagonalising `M^-1/2 L M^-1/2`; neither engine supplies the reference.
The score is trajectory-wide position `Linf` over 24 checkpoints in a fixed `T = 2`.
Rapier uses its default four solver substeps and the force-based spring model. Precision
is matched at `f64`.

Each process run takes nine timing samples after a warmup and reports the median. The
ranges below span three complete process runs. No fitted or extrapolated point is used.

| Measurement | Sparse core | Rapier | Rapier / sparse |
|---|---:|---:|---:|
| Equal `dt = 0.00390625`, 512 steps | 0.442–0.445 ms | 41.811–42.513 ms | 94.08–96.12× |
| Matched `Linf <= 1e-3` | 0.007–0.009 ms | 5.166–5.265 ms | 604.74–730.29× |
| Matched `Linf <= 1e-4` | 0.027–0.028 ms | 41.811–42.513 ms | 1,482.62–1,560.11× |
| Matched `Linf <= 1e-5` | 0.055 ms | 331.390–337.199 ms | 5,998.02–6,126.55× |
| Setup from the same precomputed edges | 0.0003–0.0004 ms | 0.0205–0.0291 ms | 61.7–85.5× |

The error falls by four per timestep halving for the sparse velocity-Verlet path and by
two for Rapier: observed orders are 2.000 and 1.000. At equal timestep, removing the old
dense `O(N^2)` step restores a roughly 95× throughput lead on this sparse scene. At equal
accuracy, the order difference compounds that lead as the target tightens.

`SparseSystem<96, 270>` occupies 9.19 KiB in fixed arrays and performs no allocation.
Rapier memory is not estimated here; the benchmark does not have an allocator-level
instrument for Rapier, so quoting a guessed comparison would not be admissible.

## Solution-gated contact benchmark

The admissible contact workload is 48 isolated, simultaneous head-on sphere pairs (96
unit-mass bodies). Pair axes cycle through x, y, and z; pair centres are separated far
enough that only the intended 48 collisions can occur. Both engines receive exactly the
same initial positions and velocities, radius 0.1, restitution 0.96, `f64` precision,
timestep, 6,144 steps, and `T = 0.6`. The analytic collision time is 0.277, deliberately
between checkpoints and timesteps.

For every body at 24 synchronized checkpoints, the gate evaluates

```text
position error = max ||x_a - x_b|| / particle diameter
velocity error = max ||v_a - v_b|| / incident pair closing speed
agreement      = 1 - max(position error, velocity error)
```

A configuration is timed only if sparse/exact, Rapier/exact, and sparse/Rapier all have
position and velocity errors no greater than 0.001: at least 99.9% agreement. The
independent piecewise-linear hard-sphere solution prevents two similarly wrong solvers
from passing merely because they agree with one another.

This is a scoped validity domain, not an all-contact claim. The workload, checkpoint count,
normalizations, 99.9% threshold, and complete timestep ladder are fixed in source; the program
prints every rejected coarser rung before the first pass and panics without timing if no rung
passes. The isolated pairs cover simultaneous impacts on all three Cartesian axes, but they do
not validate coupled contact islands, friction, rotation, coincident centres, continuous
collision detection, or arbitrary stacking. Those cases remain outside the current resolver's
admissible domain rather than being silently counted as successes.

The first common timestep in the fixed halving ladder that passes all three gates is
`dt = 0.00009765625` (6,144 steps):

| Comparison | Position error / diameter | Velocity error / closing speed | Agreement |
|---|---:|---:|---:|
| Sparse / analytic | 0.0001219 | 0 | 99.98781% |
| Rapier / analytic | 0.0005391 | 0 | 99.94609% |
| Sparse / Rapier | 0.0004172 | 0 | 99.95828% |

The next coarser common timestep is rejected: at 3,072 steps Rapier/analytic agreement is
99.87009%, below the gate, even though sparse/Rapier happens to reach 99.90572%.

Only after the 6,144-step configuration passes are stepping times collected. Each
process run takes nine samples after warmup; these ranges span three complete process
runs on the M4:

| Engine | Median total time range | Time per step range |
|---|---:|---:|
| Sparse core | 15.986–16.152 ms | 2.602–2.629 µs |
| Rapier | 72.062–72.795 ms | 11.729–11.848 µs |

The admissible Rapier/sparse ratio is **4.47–4.55×** with at least 99.9% solution agreement.
Construction is outside the stepping clock for both engines. Rapier uses four solver
iterations. Its speculative-contact margin and geometric slop are set to zero because
the benchmark contract is the analytic hard-sphere surface; the default 0.02 m
prediction margin would intentionally bounce radius-0.1 bodies before contact and could
not match that solution.

## Withdrawn rendered-scene comparison

The earlier counter-shear sphere scene sent both engines the same graph, projection, and
nominal parameters, but that is not sufficient. The sparse resolver visited 730
overlapping pairs while Rapier reported 42,724 active contact-pair steps, and their
trajectories diverged. Its timing ratio is therefore withdrawn and the executable now
prints raw values only as a rejected diagnostic with `ratio WITHHELD`. No performance
claim is based on that scene.

## Reproduce

From `sim_engine/`:

```sh
cargo test --manifest-path crates/engine-compare/Cargo.toml --bin sphere_sparse
cargo run --release --manifest-path crates/engine-compare/Cargo.toml --bin sphere_sparse
cargo test --manifest-path crates/engine-compare/Cargo.toml --bin contact_matched
cargo run --release --manifest-path crates/engine-compare/Cargo.toml --bin contact_matched
```

The solution-gated contact benchmark is
`crates/engine-compare/src/bin/contact_matched.rs`; the harmonic benchmark and withdrawn
rendered diagnostic are in `crates/engine-compare/src/bin/sphere_sparse.rs`.
