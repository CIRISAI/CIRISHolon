# SATURATION-2 (water) — detached runs

Lane: `saturation2-water`. Contract: `../SATURATION2_PREREG.md`.
Every long run here is `setsid`-detached with a `.DONE` marker holding its exit
status, so a dead session kills narration and never computation.

## What is running / has run

| marker | producer | what it is | how to restart |
|---|---|---|---|
| `s2_grid.DONE` | `holon-chem --example s2_grid` | stretch sizing: one fine 49x49x25 table per `a in {2,3,4}`, held-out error of every coarser subgrid inside it | `engine/target/release/examples/s2_grid 8 > s2_runs/s2_grid.log` |
| `s2_build.DONE` | `holon-chem --example s2_build` | the fine 65x65x49 node set at `a = 3`, the held-out tableau of every candidate table inside it, and the cache the `--emit` step reads | `engine/target/release/examples/s2_build 8 > s2_runs/s2_build.log` |
| `s2_referee.DONE` | `saturation2_referee.py --grid` | the 50-digit mpmath referee over the 84 staked geometries, into `engine/crates/holon-chem/tests/data/s2/water_referee.json` | `python3 saturation2_referee.py --grid --out <that path>` |

Both producers CACHE and resume: the referee keys each geometry by its exact
coordinates under `referee_cache/` and stamps it with the basis fingerprint
(a record written against a different basis is REFUSED, not silently used), and
`s2_build` writes its fine node set to `s2_fine_65x49.txt` so `--emit NR NU`
costs no solves. Both caches are gitignored: they are re-derivable intermediates,
and what is committed is the product.

**The referee holds a RUN LOCK** (`<out>.lock`). It exists because it was needed:
a `--grid` run was relaunched against a corrected domain while the previous one
was still going, and for twenty minutes two processes wrote one log and were both
aimed at one output file — so the stale run, finishing later, would have
overwritten the corrected artifact with a staked set built to the wrong `R_HI`,
and nothing would have looked broken. The lock refuses a live holder, takes over
a stale one with a note, and `--force-lock` is the deliberate override. The
refusal has been demonstrated firing against a live process.

## Design readings banked (inputs to everything below)

**The G1 reference**, from `examples/s2_design.rs` — the model's own full-FCI water
optimum, computed BEFORE the MBE3 table existed:

    E(H2O)     = -75.023291531289 Ha
    r_OH       =   1.9435740105 bohr
    theta_HOH  =  96.75788837 deg
    d2E/ds2 (antisymmetric) = +1.036 Ha/bohr^2  -> a minimum, not a saddle
    relaxed linear (theta = 180): r = 1.7885 bohr, E = -74.888555515412 Ha
    [labelled context, never compared against: nature's 104.5 deg, 0.957 A]

**The domain**, from `examples/s2_domain.rs` and `examples/s2_dispersion.rs`:

    R_HI = 15.0 bohr, truncation systematic 3.54e-6 Ha (stake 1e-5, 2.8x margin)
    C_LO = 0.05, C_HI = sqrt(2), STRETCH_A = 3.0, R_LO = 0.7

    R_HI = 14 was the first answer and it was wrong TWICE:
      * a grid maximum understates its own supremum — re-swept at 5x resolution
        with the angle carried past the table's fence, b = 14 read 1.0091e-5,
        ABOVE the stake, where the coarse sweep had said 9.71e-6;
      * the tail is ALGEBRAIC, not exponential. `s2_dispersion.rs` staked R^-6
        (dispersion) and MEASURED -5.01 — the quadrupole-quadrupole law. The
        stake FIRED and is kept fired. Its discriminator came back sharper than
        the prediction: swapping oxygen for closed-shell NEON removes the
        algebraic sector ENTIRELY rather than leaving an R^-6 behind.

**The third coordinate**, from `examples/s2_third.rs`: `c` beats `u` on the value
AND on `dF/du` at 25 nodes and ties at 49, so the grid is in `c` and the `u = 1`
singularity is routed around in the chain rule (every derivative converted to `u`
at the CLAMP point; the sliver inside the fence extended linearly in `u`).

**Engine vs referee**, spot-checked before the full referee run was paid for:
every energy at `(x, y, c) = (1.5, 2.0, 1.0)` agrees to <= 5.2e-13 Ha, against
R1's 1e-10 stake.

## Gate order (prereg)

R1 -> T1/T2 -> G1 -> G2 -> C1 -> P1. C1 and P1 additionally need MIXTURES-1's pair
bank (the `mixtures-engine` lane); the table, the referee and G1/G2 do not.
