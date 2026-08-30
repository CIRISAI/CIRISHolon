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

| combo | assemble | CI | total | class | fraction | iters | exit | var. margin |
|---|---|---|---|---|---|---|---|---|
| (H,H,Cl) | 0.18 s | 0.01 s | 0.19 s | 5 s | 0.038 | 22 | stagnated | +0.0171 |
| (H,Cl,Cl) | 1.04 s | 0.13 s | 1.17 s | 5 s | 0.233 | 14 | stagnated | +0.0189 |
| **(Cl,Cl,Cl)** | 3.75 s | 5.16 s | **8.91 s** | 5 s | **1.782** | 24 | stagnated | +0.0346 |
| (O,O,H) | 1.09 s | 1.04 s | 2.13 s | 5 s | 0.426 | 24 | stagnated | +0.0609 |
| (O,O,O) | 1.13 s | 49.23 s | 50.36 s | 300 s | 0.168 | 66 | stagnated | +0.0684 |

Residuals 8.1e−11 to 9.8e−11, inside the crate's publication bar
(`CONVERGED_RESIDUAL`, 1e−10). Nothing is within 5× of the 10× kill.

**A CORRECTION TO THIS GATE'S FIRST VERSION.** It derived an exit reason itself,
comparing the residual against 1e−10, and reported all five "CONVERGED". The
solver's own target is 1e−11 and it reaches none of them: every one of these
solves exits **`subspace stagnated`**. The column was reporting the gate's own
threshold and calling it the solve's outcome — a second copy of a rule,
disagreeing with the first. It now reports `Solution::exit`.

The mechanism matters for every table this campaign builds, and it is NOT the
one first proposed (an absolute tolerance not transferring to chlorine-scale
energies). `davidson_eigh` accepts a new expansion direction only when its norm
exceeds a hardcoded **1e−10**, and that threshold is SCALE-FREE. The
discrimination is already in the table above:

| | `n_det` | \|E\| | residual |
|---|---|---|---|
| (O,O,H) | 9,075 | 192 Ha | 9.82e−11 |
| (Cl,Cl,Cl) | 9,477 | 1651 Ha | 8.11e−11 |

Nearly equal `n_det`, 8.6× different `|E|`. An `eps·|E|·√n_det` noise floor
predicts the residuals differ by 8.6×; a scale-free threshold predicts they are
equal. **Measured ratio 1.21.** Across all five, the predicted floors span 14×
and the observed residuals span 1.21×. One threshold dominates all five.

**And it is not worth changing.** The eigenvalue error is `~resid²/gap`, about
1e−20 Ha — twelve orders below anything this campaign measures. The solves are
ACCURATE; only the label is wrong. Moving the threshold would shift every
energy's last bits, and SATURATION-2's committed table is gated on bit-identity,
so the fix would cost a 105,105-node regeneration to buy a word.

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

### THE GUARD: what neither the residual nor the exit reason can see

`saturation3-mesh` measured a deliberately wrong warm start converging cleanly
onto the WRONG EIGENVECTOR — 7.47 Ha above the ground state, reporting a residual
of 5.98e−11 against the correct solve's 5.24e−11 (indistinguishable) and an
IDENTICAL exit reason. Both of the record's discriminators are blind to it,
necessarily: **a residual is small for any eigenvector.**

The bound that is not blind is free. For any normalised trial vector
`E_0 ≤ ⟨ψ|H|ψ⟩`; a single determinant is one, giving `H_ii`; so `E_0 ≤ min_i H_ii`
rigorously. `diag` is already built for the preconditioner. It is now
`Solution::variational_margin`, `Option<f64>` rather than a default that would
read as a pass on the MPS route which never forms the diagonal.

**Verified in both directions before it went into a shared solver**, because a
guard demonstrated one way is half a guard:

* **Zero false positives**, 5 of 5 (`examples/s3_variational_guard.rs`). Correct
  solves sit 0.017 to 0.073 Ha BELOW the bound — nine orders above the residual
  scale, so it cannot drift into firing.
* **It fires**, and the sector had to be SEARCHED rather than assumed. The first
  plant, at G0's compact geometry, VOIDED on an empty sector: Davidson recovers
  the ground state there from a random start AND from the worst possible
  single-determinant start. So `examples/s3_wrongstate_hunt.rs` scanned sixty
  (H,H,Cl) geometries and found the failure at **sixteen of them — 27%** — at 7.3
  to 8.1 Ha above the ground state, independently matching the mesh lane's 7.47.
  **The guard catches all sixteen.**

The number that matters for a 34,500-node table: the wrong-eigenvector failure is
GEOMETRY-DEPENDENT and COMMON, not rare and not universal. Under warm starting,
a table built without this guard would be expected to carry silently wrong
entries at a substantial fraction of its nodes, each one passing every check the
record previously carried.

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
