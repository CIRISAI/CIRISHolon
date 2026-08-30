# SATURATION-3 — results

*The record for `SATURATION3_PREREG.md` (frozen 2026-08-30, commit 7bc8d50).
This lane owns the TABLES AND PHYSICS half: G0, the species-general trimer
machinery, R1, T1/T2, and both product gates. The `saturation3-mesh` lane owns
G1 and G2. Everything here is EXACT-IN-MODEL — all-electron STO-3G, full CI, no
frozen cores.*

---

## G0 — THE HARDEST COMBOS ARE TRIVIAL AS PREDICTED · **HOLDS**

*First and blocking. Instrument: `examples/s3_g0.rs`. Run twice; the two runs
agree exactly on every physical quantity and differ only in wall time.*

### The committed arithmetic is exact, five of five

| combo | orbitals | `n_det` | committed | match |
|---|---|---|---|---|
| (H,H,Cl) | 11 | 605 | 605 | ✓ |
| (H,Cl,Cl) | 19 | 3,249 | 3,249 | ✓ |
| (Cl,Cl,Cl) | 27 | 9,477 | 9,477 | ✓ |
| (O,O,H) | 11 | 9,075 | 9,075 | ✓ |
| (O,O,O) | 15 | 207,025 | 207,025 | ✓ |

The arithmetic was also re-derived independently before the gate ran, from hole
counting: every one of these triples is near-closed-shell, so the space is
`C(n_orb, n_α)·C(n_orb, n_β)` with one to three HOLES. A chlorine trimer is 27
orbitals and 51 electrons — one α hole and two β holes, hence `27 × 351`. That
is why the freeze could predict "trivial" for a system whose electron count is 51.

### The cost, and the one number the freeze got wrong

| combo | assemble | CI | total | class | fraction | iters | exit |
|---|---|---|---|---|---|---|---|
| (H,H,Cl) | 0.20 s | 0.01 s | 0.21 s | 5 s | 0.042 | 22 | CONVERGED |
| (H,Cl,Cl) | 1.12 s | 0.14 s | 1.25 s | 5 s | 0.251 | 14 | CONVERGED |
| **(Cl,Cl,Cl)** | 4.72 s | 1.75 s | **6.47 s** | 5 s | **1.294** | 24 | CONVERGED |
| (O,O,H) | 0.18 s | 0.20 s | 0.38 s | 5 s | 0.076 | 24 | CONVERGED |
| (O,O,O) | 0.48 s | 39.35 s | 39.83 s | 300 s | 0.133 | 66 | CONVERGED |

Every solve converged to the crate's own declared bar (`CONVERGED_RESIDUAL`,
1e−10); residuals 8.1e−11 to 9.8e−11. Nothing is within 8× of the 10× kill.

**(Cl,Cl,Cl) is over its stated class** — 6.47 s against the freeze's "≤ 5
seconds per point". It does not trigger the kill and does not re-scope anything;
what needs amending is the CLASS STATEMENT, not the campaign. And the honest
reading is that the boundary sits inside run-to-run variance: the two runs
measured 5.67 s and 6.47 s for the same solve, a 14% spread from machine load,
with identical residuals and identical iteration counts. **A 5-second class
boundary cannot be resolved by this instrument on a shared machine.** The
defensible restatement is that the four cheap types are all under ten seconds a
point and (O,O,O) is under a minute.

### THE STRUCTURAL FINDING: the two cost classes are bound by different things

This is the number that should shape the engineering, and it was not predicted:

    (Cl,Cl,Cl)   4.72 s assemble / 1.75 s CI   —  73% ASSEMBLE, basis-bound
    (O,O,O)      0.48 s assemble / 39.35 s CI  —  99% CI, determinant-bound

The chlorine tables are limited by the integral assemble and transform over 27
orbitals; (O,O,O) is limited by Davidson over 207,025 determinants. They are
different problems and they want different optimisations:

* **the warm start** (this lane's step 2) accelerates Davidson. It can save at
  most 27% of (Cl,Cl,Cl)'s cost and up to 99% of (O,O,O)'s. It should be sized
  and gated on (O,O,O), not on chlorine.
* **the GPU kernel** (`saturation3-mesh`'s G2, targeting the sparse H matvec) has
  the same profile: it can only pay on (O,O,O). A GPU win measured on chlorine
  would be measuring 27% of the wrong thing.
* **what would speed the chlorine tables** is the assemble path — the same
  74%-assemble / 5%-CI split SATURATION-2 measured for water, and the same
  conclusion `mixtures-engine` reached from nonzero-element counts: **size by
  BASIS SIZE, not by determinant count.** Three campaigns, three instruments,
  one answer.

### What G0 says about the (O,O,O) table, before it is built

At 39.8 s a point, an S3-symmetric (O,O,O) table at SATURATION-2's per-axis
resolution (65 × 65 × 49, fundamental domain one sixth) is about 34,500 solved
nodes — **roughly 380 core-hours, or sixteen days single-threaded.** That is the
whole warrant for G1's sharding and for P2's decision tree: the campaign should
build this table only if the fence counter says the physics needs it, and if it
does, only on the mesh.

The four cheap tables together, by contrast, are hours rather than days.

### The trap this gate had to route around, recorded because it is invisible

`fci::solve` routes any space past `MPS_ROUTE_THRESHOLD` (50,000 determinants) to
the MPS/DMRG path, whose MPO builder reaches six orbitals and then HANGS rather
than erroring. **(O,O,O) at 207,025 determinants over fifteen orbitals is the one
staked combo that crosses that threshold** — G0's single expensive measurement is
exactly the one the shortcut would have swallowed, with no error and no return.
Every solve in this gate calls `solve_determinant` explicitly.

### Two things about the instrument, disclosed

**The staked geometry rule**, fixed before any solve: each side at 0.75 × that
pair's own located `R_e`, compact because the CI is hardest where overlap is
greatest, and 0.75 because it is inside every pair's repulsive wall while staying
two decades clear of the linear-dependence corner SATURATION-2 measured. The
located equilibria, printed so the geometry can be re-derived:

    H-H 1.3887   H-Cl 2.5369   Cl-Cl 4.0241   O-O 2.4421   O-H 1.9909  bohr

**`homonuclear_radius` was NOT used**, and nearly was. It is declared only for
Z ≤ 10 and silently returns HYDROGEN's 0.694 bohr for chlorine — so a geometry
rule built on it would have staked the three chlorine combos at a hydrogen length
and reported the resulting speed as a chlorine measurement. A fallback that
returns a plausible number for an out-of-range input is the same defect shape as
a stagnated solve reported as converged.
