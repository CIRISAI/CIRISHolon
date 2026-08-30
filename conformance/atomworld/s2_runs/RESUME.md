# SATURATION-2 (water) — detached runs

Lane: `saturation2-water`. Contract: `../SATURATION2_PREREG.md`.
Every long run here is `setsid`-detached with a `.DONE` marker holding its exit
status, so a dead session kills narration and never computation.

## What is running / has run

| marker | producer | what it is | how to restart |
|---|---|---|---|
| `s2_grid.DONE` | `holon-chem --example s2_grid` | grid sizing: one fine 49x49x25 (O,H,H) table per stretch `a in {2,3,4}`, held-out error of every coarser subgrid inside it | `nice -n 15 engine/target/release/examples/s2_grid 8 > s2_runs/s2_grid.log` |
| `s2_referee.DONE` | `saturation2_referee.py --grid` | the 50-digit mpmath referee over the 84 staked geometries, into `../water_referee.json` | `python3 saturation2_referee.py --grid --out ../water_referee.json` |

The referee CACHES every geometry under `s2_runs/referee_cache/`, keyed by the exact
geometry string and stamped with the basis fingerprint, so a restart resumes
rather than recomputes. A cache record written against a different basis is
REFUSED, not silently used.

## Design readings already banked (they are inputs to everything below)

From `examples/s2_design.rs` — the model's own full-FCI water optimum, which is
gate G1's reference and was computed BEFORE the MBE3 table existed:

    E(H2O)     = -75.023291531289 Ha
    r_OH       =   1.9435740105 bohr
    theta_HOH  =  96.75788837 deg
    d2E/ds2 (antisymmetric) = +1.036 Ha/bohr^2  -> a minimum, not a saddle
    relaxed linear (theta = 180): r = 1.7885 bohr, E = -74.888555515412 Ha
    [labelled context, never compared against: nature's 104.5 deg, 0.957 A]

From `examples/s2_domain.rs` — the domain:

    worst |dE3| on the shell max(O-H) = b:
      b = 9  -> 2.29e-3    b = 12 -> 1.02e-4
      b = 10 -> 8.54e-4    b = 13 -> 3.25e-5
      b = 11 -> 3.03e-4    b = 14 -> 9.71e-6   <- first shell inside the 1e-5 stake
    => R_HI = 14.0 bohr on the LARGER O-H side.

    the closed-angle corner saturates rather than diverging (the 1/z nuclear
    repulsion cancels between E(OHH) and E(HH)), smooth down to c = 0.01
    => C_LO = 0.05.

Engine-vs-referee spot check at (x, y, c) = (1.5, 2.0, 1.0), before the full
referee run was paid for: every energy agrees to <= 5.2e-13 Ha, against R1's
1e-10 stake.

## Gate order (prereg)

R1 -> T1/T2 -> G1 -> G2 -> C1 -> P1. C1 and P1 additionally need MIXTURES-1's
pair bank (the `mixtures-engine` lane); the table, the referee and G1/G2 do not.
