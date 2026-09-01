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

---

## G1 — MESH GENERATION UNDER THE MERGE LAW · **HOLDS**

*The `saturation3-mesh` lane. Instrument: `engine/crates/holon-tables`, gate in
`tests/g1_gate.rs`. **16 tests green in release AND in debug** (9 unit, 7 gate;
159 s release, 1,046 s debug — every node is a real FCI solve). Exercised on
`(H,H,Cl)` — a staked triple at 605 determinants — through
`pair::geometry_problem` into `fci::solve_determinant_from`, the same path the
tables take (M-FOREIGN-DOMAIN-CORROBORATION: never a toy).*

### The gate

| claim | result |
|---|---|
| table bit-identical at 1, 4, 8 workers over 32 nodes | ✓ digest `1504da0f…` |
| the same digest from a SEPARATE process invocation | ✓ reproduces |
| plant (iv): one flipped mantissa bit CONVICTED by the digest | ✓ |
| plant (iv): zero false positives on clean runs at 1/2/3/4/8 workers | ✓ |
| plant (iii): 12 of 32 nodes trapped, worst 7.572 Ha | 12 of 12 VOIDed |
| plant (iii): the other 20 nodes converged correctly | none falsely voided |

*The digest recorded here is `1504da0f896a57b47bb2e286ed0ee34212986adcb0a85ba3ef0821402ba057b3`,
measured twice from separate process invocations. An earlier value (`d83e5c14…`)
appears in commits 19c0060 and e40ed8c and is STALE, not wrong: it was computed
before 3d5ea03 narrowed the digest to the table's content (index, energy, both
derivatives, status), dropping the iteration counts and exit reason. Recorded
rather than quietly replaced — a digest that no longer reproduces is a stale
instrument, and the reason it moved belongs next to the number.*

### What the merge law covers here, and what it does not

`MergeLaw.lean` proves `shardedFold_invariant` and `digest_convicts` over an
`AddCommMonoid`, exactly. A table of `f64` energies is not one — float addition
is not associative, and a table is not a sum in the first place. So the theorems
are instantiated where they apply and the rest is named:

| | how it is carried |
|---|---|
| the table's CONTENTS | **not a fold at all** — a disjoint union of independent node solves. Shard-invariance is a statement that no node's value depends on its shard, and that is a CONSTRUCTION (`grid.rs`), not a theorem. |
| the table's CERTIFICATE | a genuine fold in a genuine monoid: `(Z/2^64)^4` under wrapping addition, associative and commutative unconditionally. `shardedFold_invariant` and `digest_convicts` apply literally. |

Claiming the Lean covers the `f64` table would launder an integer theorem into a
claim about floats. It does not.

### The design, and why the obvious one is wrong

The obvious generator warm-starts each node from whatever the worker solved
last. **A warm start moves the answer**: on `(H,H,Cl)`, warm and cold solves of
the same geometry were bit-identical in **0 of 5** pairs, differing by 3.4e−13 to
4.3e−12 Ha (`examples/s3_warm_probe.rs`). Under the obvious design every node's
value is therefore a function of the WORKER COUNT, and the table silently differs
between a 1-worker and a 32-worker run — precisely what G1 forbids.

So the region partition and the traversal inside a region are **canonical
functions of the grid**, fixed before any worker exists. A region is
self-contained: a cold seed, then a serpentine (boustrophedon) chain, so
consecutive nodes in the traversal are always grid-adjacent — a plain
lexicographic walk would hand one node per row the worst guess in the region.
Regions are handed out from a shared atomic counter, so which worker takes which
region genuinely varies run to run; the invariance is not a fixed assignment that
happens to reproduce. The cost is one cold seed per region.

### The mutation set, and why it is split

`holon-mesh`'s header names the trap: a reorder over an exact carrier gives the
IDENTICAL result, so "reorder the work and assert the answer moved" cannot fail
against a correct implementation. The set is therefore split, and only the pair
proves anything:

| mutation | must the table move? | measured |
|---|---|---|
| `ReverseRegionOrder` | **no** | does not |
| `WorkerLocalWarmStart` | **yes** | does, at 1 vs 4 workers |
| `CorruptNode` | convicted | convicted |
| `WrongWarmStartAll` | the trapped nodes VOID | 12 of 12 |

### The clause that no design could satisfy — AMENDED, NOT REINTERPRETED

The freeze asked for "the WARM result bit-identical to cold at every node (a warm
start may change the path, never the answer)". The parenthetical is a factual
claim about f64 Davidson and it is measured **FALSE at every scale tested**:

| table | `n_det` | warm/cold pairs bit-identical | worst \|dE\| |
|---|---|---|---|
| (H,H,Cl) | 605 | 0 of 5 | 4.320e−12 Ha |
| (O,O,O) | 207,025 | 0 of 5 | 3.126e−12 Ha |

In finite precision the trailing bits of "the answer" ARE a function of the
iteration path, so no design satisfies the clause as written.

This was escalated rather than re-scored, and the lead ruled: **AMENDMENT A2**
(prereg, 2026-08-30, post-data). A clause that fails EVERY design separates
nothing, so re-wording it cannot have been selected by a favourable result —
none was available to select. The cited precedent is the retired 1e-11 Davidson
ask. The amendment records explicitly that had the clause been achievable by some
designs and not others, it would have been **REFUSED as a rescue** and the
campaign re-frozen. The primary clause — bit-identical across shard counts, a
corrupted shard convicted — is untouched, and is what this gate measured.

### Plant (iii), scored against A2's two-sided gauge

A2 replaced this lane's judgement-based three outcomes with **numbers**, and the
gate now scores against those rather than against a reading:

| outcome | boundary | measured at discharge |
|---|---|---|
| **BENIGN** | \|E_node − E_cold\| ≤ **1e−9 Ha** (one order above the 1e−10 convergence bar) | 20 of 32, worst **4.3e−12 Ha** |
| **REFUSED to classify** | the dead band between — named, never silently absorbed | **0 of 32; the band is empty on this data** |
| **TRAPPED** | ≥ **1e−3 Ha** — must VOID via the variational bound or the plant fires AS A FAILURE | 12 of 32, worst **7.572 Ha**, **12 of 12 VOIDed** |

**Separation: twelve orders**, with zero false VOIDs among the benign. The dead
band being empty is the reading that matters — it means the gauge is not
straddling the data, and the classification is not a threshold choice dressed as
a result.

### The trap is geometry-dependent, which cost this gate a false alarm

The same random start vector that traps a `(H,H,Cl)` solve 7.47 Ha above the
ground state at one geometry converges to within 3.3e−12 Ha at another on the
same grid. Whether a wrong start gets lost is a property of the level spacing
where it is dropped. Planting ONE node therefore samples the trap rather than
testing it; the gate plants every node and asserts the guard fires on every
occasion the sector is non-empty (M-PLANT-SECTOR — a plant on an empty sector
VOIDs rather than passes, and the gate says so instead of going green).

### The warm start's value GROWS WITH THE SPACE, and changes sign

The prereg asks for "cold vs warm Davidson iterations along a locality sweep,
speedup reported". Measured, same instrument, same grid shape, one cold seed per
region:

**Instrument A — the generator**, warm chains with one cold seed per region, so the
saving is DILUTED by the seeds:

| table | `n_det` | cold iters | warm iters | saving | cold-seed fraction |
|---|---|---|---|---|---|
| (H,H,Cl) | 605 | 744 | 771 | **−3.6%** (a slowdown) | 4/32 = 0.12 |
| (Cl,Cl,Cl) | 9,477 | 509 | 465 | **+8.6%** | 4/18 = 0.22 |

**Instrument B — the pairwise probe** (`s3_warm_probe`), cold vs warm at the SAME
five geometries with no seeds in the average, so this is the UNDILUTED per-node
saving and is not directly comparable to the rows above:

| table | `n_det` | cold iters | warm iters | saving |
|---|---|---|---|---|
| (O,O,O) | 207,025 | 579 | 539 | **+6.9%** |

per step: 117→96, 113→111, 97→96, 121→96, 131→**140** — the last one is a
*slowdown*, at the largest step.

**THE WARM START DOES NOT RESCUE THE EXPENSIVE TABLE.** This gate's warrant
estimated the warm start could save "up to 99% of `(O,O,O)`'s" cost. Measured, it
saves **6.9%** of Davidson iterations — undiluted, on the table it was supposed to
pay on. Backing the cold seeds out of `(Cl,Cl,Cl)`'s 8.6% puts that one near 11%
undiluted, so the benefit does **not** keep growing with the space: it is roughly
7–11% on both heavy tables and negative on the cheap one. Against G0's ~380
core-hours for a full-resolution `(O,O,O)` table, 6.9% is about 26 core-hours —
real, worth keeping, and not a reason to build a table that was otherwise
unaffordable.

The sign changes with the size of the determinant space, which is why this must
be sized on the expensive table and not on chlorine — the tables lane's point,
and the measurement agrees with it. On (H,H,Cl) the cold start (lowest-diagonal
determinant plus a deterministic perturbation) is already as good as a
neighbour's converged vector, and the warm start's dense first basis vector
costs slightly more to refine than it saves.

Two things keep this honest. The saving is quoted per TABLE and is diluted by the
cold seeds — on (Cl,Cl,Cl), backing the four cold seeds out puts the per-warm-node
saving nearer 11%. And iterations are not wall-clock: on (H,H,Cl) the CI is only
3.8% of node cost (G0), so neither sign matters there at all.

### NEGATIVE RESULT: neither the residual nor the exit reason can see a wrong solve

| start | E (Ha) | error | residual | exit |
|---|---|---|---|---|
| cold | −467.207401633682 | — | 5.240e−11 | subspace stagnated |
| neighbour (good) | −467.207401633683 | 8.5e−13 | 5.999e−11 | subspace stagnated |
| **random (planted)** | **−459.735448873** | **7.472 Ha** | **5.984e−11** | subspace stagnated |

It converged CLEANLY onto the wrong eigenvector. A residual is small for any
eigenvector, so no residual threshold can separate these, and the exit reasons
are identical. This is the carbon incident's shape reached through the
warm-start channel.

The tables lane swept this rather than leaving it at one instance: **16 of 60
`(H,H,Cl)` geometries, 27%, at 7.3–8.1 Ha**, independently matching the 7.47 Ha
found here. Two instruments, two lanes, one number. Geometry-dependent and
COMMON — not rare, not universal — which is what makes the guard mandatory
rather than prudent on a 34,500-node warm-started table.

**And the trap did NOT occur at all on `(O,O,O)`**, which is worth recording
because the probe's first verdict line got it wrong. The random start there
reached the reference energy to **2.103e-12 Ha** — it found the right
eigenvector, took 361 iterations against 97 to do it, and the guard correctly did
not fire. The probe printed "NOT SUFFICIENT: the plant slips past it; a second
guard is owed", which conflates *the guard missed a trapped solve* with *there
was nothing to catch*. The plant's sector was EMPTY at that geometry. The probe
now measures the carrier before judging the guard, and says so; nothing is owed.

*(This lane's first diagnosis of the universal `stagnated` exit — an absolute
1e-11 tolerance not transferring to chlorine-scale energies — was WRONG, and the
tables lane's G0 data discriminates it: `(O,O,H)` and `(Cl,Cl,Cl)` sit at nearly
equal `n_det` with 8.6× different `|E|`, and a scale-dependent floor predicts
their residuals differ by 8.6× where the measured ratio is 1.21. The real cause
is scale-free — `davidson_eigh` accepts an expansion direction only when its norm
clears a hardcoded 1e-10 after Gram-Schmidt. It is deliberately not changed:
the eigenvalue error is ~1e-20 Ha, so only the label is wrong, and moving the
threshold would shift every energy's last bits and cost SATURATION-2 a
105,105-node regeneration.)*

**A POSTSCRIPT ON THIS DISCRIMINATOR, recorded because it is the same lane
failing the same test later the same day.** Exit reason and residual carry
different facts, and this section is built on reading them apart. Hours after
writing it, in RESOURCE-1, this lane read the DD tier's calibration numbers
(Sb 3.37e-15, Te 2.97e-13) as *floor* readings and reported to the lead that
`DD_EXPANSION_FLOOR` was contradicted by nine to eleven orders and owed a move.
Both solves had exited **`IterationCap` while still descending** — they ran out
of iterations, not of arithmetic. Capped-while-descending is a budget reading;
stagnated-at-the-floor is a floor reading; nothing was owed. Retracted at
`613404c`, settled independently by the lead at `665a342`.

M-EXIT-DISCRIMINATOR, self-inflicted, one lane over from where it was first
applied: **the tool in hand, not turned on one's own claim.** The repair is in
the type rather than in a resolution — `Limit::{Floor, Budget}` now rides on
every measured boundary, because a budget-limited reading wearing a wall's
clothes would make the lease layer refuse work that a larger budget would
finish.

The guard that works is free and rigorous: `E <= min_i H_ii`, since a single
determinant is itself a trial vector. It fires on the plant by 7.4 Ha and passes
both good solves by 5.4e−2. The tables lane has since put it on
`Solution::variational_margin`; `holon-tables` reads that field rather than
keeping a second copy of the rule, and inherits its scope honestly — **necessary,
not sufficient**: an excited state BELOW `min_i H_ii` still passes.

---

## G2 — GPU ADOPTION, MEASURED · **CONDITION MET, ADOPTION DEFERRED**

*Measured on the real `(O,O,O)` problem — 207,025 determinants, the scale the
prereg names — with the index structures and integrals exported from
`geometry_problem` (`examples/s3_sigma_export.rs`) and the answer checked against
`sigma_direct`'s own. Device: RTX 4090 Laptop (Ada, sm_89, 76 SMs).*

### The kernel, and why it is three GEMMs

Written out, two of `sigma_direct`'s three blocks are dense GEMMs against
matrices that do not depend on `c` at all, so they are built once per geometry:

```
  beta  same-spin :  Sigma += C · F_b            F_b (nb × nb)
  alpha same-spin :  Sigma += F_aᵀ · C           F_a (na × na)
  mixed           :  D[ja] = A[ja] · T[ja],  then a gather
```

Only `T` depends on `c`, and `T[ja][kl][ib] = sign · c[ja][jb(kl,ib)]` is a pure
GATHER, because for a fixed excitation and destination the source string is
UNIQUE (`a⁺_p a_q |jb⟩ = s|ib⟩` inverts).

### The determinism gate: PASSES, structurally

There is **not one atomic in the kernel**. The obvious implementation scatters
through both excitation lists, both scatters collide, and floating-point
`atomicAdd` accumulates in completion order — a table built that way would not be
bit-identical even to itself across two runs. Both scatters are inverted into
gathers, which the invertibility above makes possible, so every sum is over a
fixed range in a fixed order. **Five repeat runs bit-identical.**

### The measurement

| arm | sigma/s | GFLOP/s FP64 | note |
|---|---|---|---|
| **GPU, whole kernel** | **65.7** | 318.4 | 15.2 ms/sigma |
| GPU, kernel only, WARM | 68.2 | 331 | the honest device rate (2026-09-01) |
| GPU + host round trip | 28.9–56.9 | — | genuinely host-exposed, 1.97× spread |
| CPU, `sigma_direct`, 32 threads | 17.2 – 20.8 | ~97 | two runs, loadavg 18 and 32 |
| CPU, same GEMM reformulation, OpenBLAS 1 thread | 1.40 | 6.8 | **slower than the hand-written kernel** |

**Speedup 3.2× against the best CPU number.**

**CORRECTED 2026-09-01, with the instrument rescued.** These GPU numbers were taken
with instruments cited as `scratchpad/s3gpu/...` — a path that never resolved, because
they lived in a per-session scratchpad rather than the repository's. They are now at
`conformance/atomworld/s3_mesh/gpu/` (README there carries the reproduction and the
`ooo.bin` sha256 pin), and re-running reproduced the correctness figures **identically**:
4.547474e-13 absolute, 3.033e-15 relative, 188,363 of 207,025 bitwise, five repeats
bit-identical. The 91.0% divergence that founded M-DEVICE-CLASS is a re-run result now.

The re-run exposed a defect I then diagnosed wrongly three times; the fourth account is
cross-validated. `69.8` was reported as the WITH-round-trip figure while being faster than
the 65.7 it contains. **Cause: a cold-clock ramp on whichever block runs first.** `sigma.cu`
times kernel-only first, so it pays the ramp — banked 65.7 = 15.221 ms is SLOWER than the
warm kernel measured today at 14.60–14.76 ms, while banked 69.8 = 14.327 ms is the warm
second block. **The kernel block is stable and my earlier 1.23× was my own instrument**: at
200 reps rather than 20 the ramp amortises and the spread collapses to 1.010× (mean 68.22),
against gpu-prod's twelve separate invocations of a different instrument at 1.019× (mean
68.253) over loadavg 67.5–86.6 — two methods agreeing to 0.05%. **So the banked 65.7
understates the kernel by 3.7%; the honest warm rate is 68.2.** What survives as host
exposure is the round trip alone, and it worsens with reps (1.97× at 200), the signature of
per-rep pageable copies rather than a ramp. Previously this record claimed gpu-prod's log as
independent corroboration of the ordering; that was a transient working-tree state of their
in-flight file and is WITHDRAWN — their committed logs order correctly, and their
twelve-invocation series is the better corroboration that replaces it.

**RETRACTED, one hour after I posted it:** that the GPU arm's contention exposure had been
measured and was negligible (65.7 → 67.5, "+2.7% in the wrong direction"), and that 3.2×
was therefore an upper bound on the quiet-box ratio. That rested on a single draw; five
more read 55.0–58.8, below the banked 65.7. The device compute is unexposed, but measured
GPU throughput is host-exposed through launch scheduling and pageable copies — relatively
more so than the CPU arm (6.0× vs 1.21×). **M-PLACEMENT-LOTTERY's third rung applies and
the direction of the bias is NOT established: both arms need the quiet window.** The 3.2×
is a loaded-box figure with unmeasured spread on both arms. It does not touch the G2
verdict, which was adoption-condition-MET / adoption-DEFERRED on device-class grounds
rather than on the size of the win, and it does not touch the correctness figures, which
are bit comparisons rather than timings.

### A property banked rather than assumed: the table is profile-independent

Checked while wiring the generator through the resource layer, and it had never
been checked before: the same table generated in DEBUG and in RELEASE has the
**identical digest** (`e763ba08…` on the H3 smoke grid, both profiles). No
reassociation, no fast-math drift, no optimisation-level dependence.

That matters more than its size suggests. Every digest in this record — G1's
`1504da0f…` included — is a bit-identity claim, and had this gone the other way
every one of them would have been silently profile-dependent: green in whichever
profile it was recorded under, red in the other, for a reason nothing in the
gate would have named.

### The honest-baseline check — what makes the 3.2× a DEVICE result

This is the load-bearing control for the whole gate, so it is stated separately
rather than left in a caveat. The GPU arm does not merely run on a GPU: it also
**reformulates** sigma as three GEMMs and hands them to a **tuned library**.
Quoting that against `holon-chem`'s hand-written loop would credit the device
with all three, which is the defect `holon-gpu/src/cpu.rs`'s own header warns
about ("so the GPU's speedup is quoted against the best CPU and not against the
most convenient one").

So the identical reformulation was run on the CPU through OpenBLAS. **It came
back SLOWER than `sigma_direct`** — 1.40 against 2.20 sigma/s single-threaded.
The reason is bandwidth, not tuning: materialising the intermediate `T` costs
**372 MB per sigma**, which the GPU has (462.7 GB/s measured) and the CPU does
not. `sigma_direct` avoids it by building `t` one alpha string at a time (819 KB,
cache-resident) and reusing it across that string's 48 excitations.

Two things follow. `sigma_direct` is a good cache-blocked algorithm and IS the
honest CPU arm — there was no slow baseline to beat. And the 3.2× is attributable
to the **device**, specifically to its memory bandwidth, rather than to the
reformulation or the library; the reformulation on its own is a *pessimisation*
on CPU.

### Agreement, and what adoption would cost G1

| | |
|---|---|
| max abs difference vs `sigma_direct` | 4.547e−13 |
| relative to max abs sigma (1.499e2) | **3.033e−15** |
| entries differing **BITWISE** | **188,363 of 207,025 (91.0%)** |

That last row is the one that matters. The two sigmas are numerically the same
answer and are not the same BITS, so a Davidson driven by one follows a different
path from the other and **a GPU-built table is a different artifact from a
CPU-built one**. G1's guarantee would survive only WITHIN a device class, and CPU
and GPU nodes could never be mixed inside one table — which also means adopting
the GPU idles the 32 cores rather than adding to them.

### The verdict, with gate and judgment kept apart

The prereg's adoption condition — "it wins, with a determinism gate stated" — is
**MET**: 3.2× with a fixed reduction order that is a property of the
construction. This is not the refusal branch; the kernels fit these sizes.

The recommendation to DEFER is engineering judgment, not the gate:

* the whole benefit lands on `(O,O,O)`, the one table P2's fence counter may
  demote without ever building — G0 prices it at ~380 core-hours, which the GPU
  would take to roughly a fifth;
* adoption makes the table device-dependent (91% of entries differ bitwise);
* the kernel is measured but not integrated — integration means `cudarc`
  plumbing plus pinning a cuBLAS version, since determinism rests on stable
  kernel selection.

Hold it until P2 branch (b) fires; spend nothing on integration before then.

---

## THE ANGLE-AXIS OBLIGATION — **DISCHARGED: it is two state crossings, and no grid buys them back**

*The freeze makes this a design-time obligation: "the angle-axis convergence
anomaly from SATURATION-2's hand-off (5x per doubling where C1 cubic owes 16x) is
a STAKED INVESTIGATION at design time: the builder must either explain it or show
the chosen coordinate beats it before committing any grid." Explained. Nothing
below required a new electronic-structure campaign — the gauge costs no solves at
all, and the two slices cost about four hundred (O,H,H) points between them.*

### The owed rate was wrong, and it is now gauged rather than argued

"A C1 cubic should give 16x" is `h^4`, and it was asserted. Catmull-Rom takes its
node slopes from centred differences, whose `O(h^2)` error enters the Hermite
form multiplied by one factor of `h`, so the scheme is third order — Keys' cubic
convolution result for `a = −1/2` says the same. Rather than argue it, `s2_build
--gauge` plants two analytic functions on the same fine grid, interpolates them
with the SAME `eval` on the SAME subgrids at the SAME 384-point held-out draw,
and reads the rate off:

| planted | 13 → 25 | 25 → 49 |
|---|---|---|
| `exp-cos` | 10.13× | 6.68× (floored by the radial axis) |
| `c-bump` (curvature deliberately on the angle axis) | 10.92× | 9.32× |

**So the interpolator delivers 9.3–10.9× per doubling of the angle axis, not 16×.**
The shortfall being explained is therefore 5.05 against ~9.3, not against 16.

### It is not the end condition

`slope_weights` uses a one-sided three-point slope at both ends of every axis, so
the first and last intervals were the obvious suspects — and the tableau's worst
point does sit in an end interval on three of the four angle rungs. But on
planted data the "all points" and "no end interval" columns are **identical to
every digit on every row**. The one-sided slope costs nothing on smooth data. It
is not the mechanism.

### It is two electronic state crossings, inside the tabulated domain

At the shipped 65 × 49 both axes stall at the same value, 7.68e-4, at the same
held-out point: refining the radial axis 33 → 65 buys 1.38× and refining the
angle axis 25 → 49 buys 1.35×, where the planted functions buy 9× on those rungs.
A feature that survives refinement in BOTH directions is not a resolution
problem. Walking the surface through it (`s3_angle_slice`) finds why:

* **Seam 1, near collinear.** On the worst point's slice (x = 1.766, y = 2.576)
  `dE3` has a corner at `c ≈ 1.4128`, `theta ≈ 174.9°`: the bending slope changes
  by a factor of about 400 across roughly `1e-4` of the `c` axis, against a grid
  spacing there of `0.0284`. The corner sits inside the LAST cell, at 98% of the
  way across it, and the Catmull-Rom stencil for the worst held-out point
  (c = 1.373) reaches the endpoint node ACROSS it — which is why refining either
  axis changes nothing.
* **Seam 2, where H₂ forms.** Masking out every point whose stencil touches the
  last node leaves a floor of 4.90e-4 converging at 2.11×, and its worst point is
  (2.621, 2.703, 0.461) — `theta ≈ 38°`, H-H side 1.74 bohr, the H₂-formation
  region. That slice has its own corner at `c ≈ 0.4375`, where the slope in `c`
  changes by about 4× and the energy turns over.

**Both are physics, not the eigensolver, and that was measured rather than
assumed.** `s3_cross_check` carries a converged eigenvector across each corner and
re-solves from it: if cold Davidson had been sitting on the upper branch anywhere,
a warm start would find a lower energy at the same geometry and the variational
bound would say so without appeal. Across both corners, **no warm start beat its
cold solve at any geometry** (differences ≤ 1e-12, i.e. noise), with variational
margins healthy throughout (0.062–0.071 Ha). The ground state is the lower
envelope of two branches, so the corner is where they cross.

### The coordinate is complicit, and this is why the bend angle won

`c = sqrt(1 − cos θ)` compresses the near-collinear region: `dθ/dc → ∞` as
`c → √2`, so near the top of the axis one grid cell of width 0.0284 in `c` spans
about 6° of θ, while the same width near `c = 1.37` spans under a third of a
degree. Seam 1 is squeezed into the last cell by the coordinate itself. That is
the mechanism behind SATURATION-2's measurement that the bend angle beats `u` and
`c` on the worst slice by 2.2× — a coordinate uniform in θ gives the collinear
region roughly an order of magnitude more grid. It does not remove the corner; it
stops hiding it inside one cell.

### What this rules for SATURATION-3's grids

1. **Uniform refinement of an angle axis cannot beat a corner.** A cubic
   interpolant across a slope discontinuity has an error floor set by the jump,
   and the 5×-per-doubling reading was that floor arriving. Paying for angle
   resolution past the point where the seam dominates buys nothing, and the
   held-out tableau will say so by stalling.
2. **The design question is where the seams are, not how fine the grid is.** The
   options are to place a grid line ON a seam (splitting the domain into two
   smooth patches, which restores the gauged rate on each), or to accept a floor
   and stop paying. Both need the seam LOCATED first, which is now a cheap
   instrument (`s3_angle_slice` is ~60 points a slice).
3. **This is not a water-specific hazard.** Every trimer type this campaign
   tabulates has a reactive channel — (H,H,Cl) and (H,Cl,Cl) most obviously — so
   a seam scan belongs in each table's design step BEFORE its grid is frozen,
   which is what the obligation was protecting against.

### What is still owed

The seam LOCUS, as opposed to two points on it. Both seams were found by walking
one slice each, chosen because the held-out tableau pointed at them. Whether they
are surfaces cutting the whole domain, and where, is a scan this has not paid for
— and the grid ruling in (2) needs it before any angle axis is frozen.

