# THE dE5 TRUNCATION AUDIT — prereg

*Frozen 2026-09-01, before `engine/crates/holon-chem/examples/de5_audit.rs` existed and
before any five-body number was read. GANTT node H's rule is **MEASURE, never build**: no
five-body term is written by this campaign under any branch below. This document stakes
the estimator, the subtraction basis, the sampling rule, the bound, the plants, and the
meaning of every possible answer. The git history is the check — this file lands ALONE,
in its own commit, before the instrument.*

**misfits:** M-BUDGET-LAUNDER, M-EXIT-DISCRIMINATOR, M-MAX-OVER-SUCCESSES,
M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-POPULATION-CHOICE, M-BASE-RATE-OMITTED,
M-UNTESTED-GAP, M-PLANT-OBS, M-PLANT-SECTOR, M-HOMOG, M-DEVICE-CLASS,
M-PROVENANCE-OVERREACH, M-STALE-INSTRUMENT, M-PLACEMENT-LOTTERY,
M-CHEAPER-THAN-ITS-PRICE, M-NULL-MISSTAKE.

---

## 0. THE QUESTION

For compact five-atom clusters that real trajectories actually visit, is

```text
dE5(S)  =  E_FCI(S)  -  E_MBE4(S)          |S| = 5
```

within the ladder's own declared per-term uncertainty?

The engine's chemistry stops at four bodies. `quaternary.rs` computes
`de4_ohhh_fci = E_FCI(OHHH) - E_MBE3(OHHH)` and the runtime adds it as the last rung.
Whether that is the LAST rung is at present an assumption, and this audit is the
measurement that either certifies it or fires the seam requirement. It cannot manufacture
a five-body term and it is not permitted to: node H's receipt is a verdict, not a table.

---

## 1. THE ESTIMATOR, term by term

Let `S = {a1..a5}` be five atoms at fixed Cartesian positions. Write `E(T)` for the
total energy (electronic + nuclear repulsion) of the subsystem on atom set `T`.

```text
E_MBE4(S) = SUM_{i}      E({a_i})                         5 atom terms
          + SUM_{i<j}    dE2(a_i,a_j)                    10 pair terms
          + SUM_{i<j<k}  dE3(a_i,a_j,a_k)                10 triple terms
          + SUM_{|Q|=4}  dE4(Q)                           5 quadruple terms

dE2(P)  = E(P) - SUM_{i in P} E({a_i})
dE3(T)  = E(T) - SUM_{i in T} E({a_i}) - SUM_{P subset T, |P|=2} dE2(P)
dE4(Q)  = E(Q) - SUM_{i in Q} E({a_i}) - SUM_{|P|=2} dE2(P) - SUM_{|T|=3} dE3(T)

dE5(S)  = E(S) - E_MBE4(S)
```

This is the inclusion–exclusion ladder one rung above `quaternary.rs`'s, with the SAME
shape: `de4_ohhh_fci` is exactly the `dE4(Q)` line for `Q = (O,H,H,H)`, and this audit's
`dE5` is exactly its successor. Nothing is fitted; nothing is empirical.

### 1.1 The subtraction basis — DECLARED, per the house's law

`conformance/atomworld/s3_mesh/TRIMER_TABLE_SCHEMA.md` makes `subtraction_basis` a
required provenance axis: a dE-valued artifact that does not say which lower ladder it
subtracted **cannot be re-derived**, and the two bases in this engine agree to ~1e-10
today, which is exactly why inference from the values would succeed until it didn't.

> **`subtraction_basis` = `fci_live` — every rung of the estimator above, at every
> arity, from ONE solver: the determinant route `fci::solve_determinant`, exact in
> model, no served surface anywhere in the assembly.**

**At the pair rung this IS `pair_point_exact`, which is `quaternary.rs`'s own
convention, matched exactly.** `ohhh_mbe3_energy` builds its six pair terms as
`pair_point(A,B,r).e - E(A) - E(B)` with the isolated-atom energies from
`atom_energy_o`/`atom_energy_h`. This audit calls those same three functions for its ten
pair terms and its five atom terms, so the rung-2 numbers are bit-identical to the ones
the four-body path already subtracts.

**At the triple and quadruple rungs it deliberately DIFFERS from the runtime path, and
the reason is numeric, not stylistic.** `ohhh_mbe3_energy` serves its triples from
`WaterTable` and `TrimerTable`. Their measured held-out interpolation errors are
**6.3e-5 Ha** (`trimer.rs`, 33 x 33 x 13) and **at most a third of T1's 1e-3 kill**,
i.e. ~3.3e-4 Ha (`water.rs`). A ten-triple assembly on that basis would carry table
error of the order of **1e-3 Ha** into a quantity whose "terminates" bar is **5e-5 Ha** —
twenty to sixty times larger than the signal. Served tables would measure their own grids
and call it physics. There is a second, independent reason: this audit's clusters contain
`(O,O,H)` and `(H,H,H)` triples whose side lengths can fall outside `ooh.rs`'s
`[0.8, 14.0]` and `trimer.rs`'s `[0.7, 9.0]` boxes, where a table CLAMPS; a clamped value
is not the geometry's energy.

So the deviation is declared here rather than discovered later, and the audit's dE4
values are recomputed live rather than read from `de4_ohhh_fci` for the same reason: a
telescope whose rungs come from two bases does not telescope, and the residue would be
table error wearing five-body clothes.

### 1.2 Every subsystem goes through `solve_determinant`, never `solve`

`fci::solve` routes past `MPS_ROUTE_THRESHOLD = 50,000` determinants into the DMRG path.
`conformance/atomworld/s3_runs/RESUME.md` records what that costs a careless caller:
"an MPO builder that reaches six orbitals and HANGS rather than erroring... Call
`solve_determinant` explicitly anywhere the space size is not statically obvious."

**CORRECTION C-1, landed pre-data, before the instrument existed and before any five-body
number was read.** This paragraph first said "Two of this audit's subsystem classes are
over that line (`O2H3` at 204,490 determinants; `OHHH` at 52,920 in the four-hydrogen
case)". The second number was arithmetic error: `OHHH` is 8 orbitals and 11 electrons,
`C(8,6)*C(8,5) = 1,568` — the count `quaternary.rs`'s own header states, which is the
check that should have caught it. Recomputed over every composition this audit can meet:

> **Exactly ONE subsystem class is over the 50,000 line: the `O2H3` pentamer, at
> 204,490 determinants. The next largest subsystem in scope is the `O2H2` quadruple at
> 48,400 — only 3.2% under the threshold**, which is far too close to assume, so the
> instrument reads `Solution::route` on EVERY solve and refuses a `Dmrg` route rather
> than inferring the route from a size it believes it knows.

The stake is unchanged and is if anything tightened: routing was already asserted per
solve, and the corrected number says the assertion is not decoration. So:

> **Every solve in the estimator uses `fci::solve_determinant`. A mixed-exactness
> telescope — some rungs exact, some DMRG-64 — would put an approximation error of
> unknown size in the residue and call it a five-body term.**

witness: `exact_never_degraded` — under an exactness hold, whatever the selector returns
is exact; approximation cannot leak past the policy. This audit holds exactness and
degrades nothing; a subsystem that cannot be solved exactly is REFUSED, never
substituted.

### 1.3 The spin-sector fence, inherited and stated

`pair::electron_counts` puts every subsystem in its lowest-|Sz| sector
(`sz2_sector(n) = n % 2`). The fragments' sectors therefore do not sum to the parent's:
`O2H3` is solved at (10a, 9b) while its five atoms sum to (11a, 8b). This is
`quaternary.rs`'s convention exactly — `OH3` at (6a, 5b) against its atoms' (7a, 4b) —
and it is inherited rather than overridden, because the quantity being audited is the
residue of the expansion **the engine actually runs**. It is NOT a spin-adapted
fragmentation, and no claim below is a claim about one.

### 1.4 Provenance axes, all three hats

Per the schema's law that device class, solver budget and subtraction basis are one law
wearing three hats, every CSV row and the results document carry:

| axis | value |
|---|---|
| `device_class` | `cpu` (host provider; no accelerator path is used) |
| `solver_budget` | 5000 (`fci::DAVIDSON_DEFAULT_BUDGET`, unmodified) |
| `subtraction_basis` | `fci_live` (section 1.1) |

---

## 2. SCOPE, and the fences that are NOT negotiable

### 2.1 Which cluster COMPOSITIONS are in scope

Not by taste: by an arithmetic rule stated here and applied by the instrument. In STO-3G
a cluster of `nO` oxygens and `nH` hydrogens has `n_orb = 5*nO + nH` spatial orbitals and
`n_elec = 8*nO + nH` electrons, and the determinant space is
`C(n_orb, na) * C(n_orb, nb)` with `na = ceil(n_elec/2)`, `nb = floor(n_elec/2)`.

| composition | n_orb | n_elec | determinants | in scope |
|---|---:|---:|---:|:--|
| `H5`    |  5 |  5 |         100 | **yes** |
| `OH4`   |  9 | 12 |       7,056 | **yes** |
| `O2H3`  | 13 | 19 |     204,490 | **yes** |
| `O3H2`  | 17 | 26 |   5,664,400 | no |
| `O4H`   | 21 | 33 | 121,788,765 | no |

> **`FCI_DET_MAX = 250,000`.** A five-cluster is admitted iff its own determinant count
> is at or below it. `OH4` and `O2H3` — the two the brief names, and the two the water
> scenes visit — are both in; `H5` joins them for free.

**The threshold is not a free parameter, and this is checkable rather than asserted:**
the largest admitted composition is 204,490 determinants and the smallest refused is
5,664,400, a gap of **27.7x** with nothing inside it. Every threshold in that interval
produces the identical partition, so the number cannot be tuned to move a verdict
(M-UNTESTED-GAP: do not stake a value in a hole in your own axis — here there is no hole
to stake in, and the engine's own `HARD_DETERMINANT_CAP = 2,000,000` refuses `O3H2`
independently).

**`O3H2` and `O4H` are OUT OF SCOPE for this audit and the verdict does not speak for
them.** They are counted, by composition, in the results document's landscape table.

### 2.2 Quadruples with no dE4 machinery — the fence, settled by construction

`de4_ohhh_fci` exists only for `(O,H,H,H)`. The five-clusters in scope contain quadruples
of shapes `H4`, `OH3`, `O2H2`:

| 5-cluster | its five quadruples |
|---|---|
| `H5`   | 5 x `H4` |
| `OH4`  | 4 x `OH3` + 1 x `H4` |
| `O2H3` | 2 x `OH3` + 3 x `O2H2` |

Only `OH3` has runtime machinery. **This audit does not need it**: section 1.1's
`fci_live` basis computes `dE4(Q)` for every shape from the same live determinant solves,
and `H4` (36 determinants), `OH3` (1,568) and `O2H2` (48,400) are all far inside
`FCI_DET_MAX`. The fence that remains is a fence on the *runtime*, not on this
measurement, and it is stated so nobody reads the two as the same thing:

> **The engine's four-body FORCE path covers `OHHH` only. A cluster whose quadruples
> include `O2H2` or `H4` is audited here but is NOT four-body-corrected in dynamics.**
> That gap is a finding for GANTT node A (species-generic MBE), not a defect of this
> audit, and it is reported in the results document rather than left implicit.

Cross-check, run and reported on every `OH3` quadruple this audit touches: the live
`dE4(Q)` must agree with `quaternary::de4_ohhh_fci` on the same four centres to within
the served-table error budget of section 1.1. Disagreement beyond it is a finding about
the two bases, printed, and never silently averaged.

### 2.3 The geometry source is PLANAR, and the verdict inherits that

The parked trajectories are `Dims::Two` (header field measured: `dims = 2`, box
34.6 x 20.8 bohr, 12 atoms, `z = [8,8,8,8,1,1,1,1,1,1,1,1]`). Every sampled five-cluster
is therefore coplanar with `z = 0`.

> **SCOPE: planar configurations only.** The verdict below is about the ladder at the
> two-dimensional geometries the engine's own water scenes visit. A three-dimensional
> successor is OWED and is named here so it cannot be forgotten; no branch below may be
> restated without the word "planar" in the same sentence.

### 2.4 The dynamics that generated the geometries carried no four-body term

The `fenced` arm ran MBE3 only (`conformance/atomworld/p2_de4_full/README.md`: "the
fenced arm ran no four-body term at all"). The trajectory is used here as a GEOMETRY
SAMPLER and nothing else — dE5 is a property of the electronic structure at a fixed
nuclear configuration, not of the propagator that produced it. What the fence does buy
is a stated ensemble: the sampled configurations are the ones MBE3 dynamics visits, which
is a spatially different population from the ones MBE4 dynamics would visit
(M-HOMOG: a population measured in one local regime does not speak for another). The
`de4_off`/`de4_on` arms are NOT used, because only the `fenced` rows are pinned in the
committed manifest (section 2.5).

### 2.5 The trajectory pin

The instrument REFUSES any trajectory whose sha256 is not the committed one. Pinned BY
BLOB — `f62486aa908ba8f382099049853f28d5a04f1b27`, the git object for
`conformance/water_observatory/census_traj_manifest.sha256` — and not from the working
tree, where another lane's uncommitted rows were present when this was written. A blob
hash is the durable pin: the same object is reachable from commit
`c5919a22cac08d40819afce3a0425ac1695a2868` (read first) and from this lane's base commit
`151362c85e4f694f8efe312f5b6cb079431feb5e` (read again after the shared tree advanced),
which is the check that the pin was not poisoned between the two reads:

```text
0441aeef8797d37e22e842ec577194d0649a2dbfbd08bafa064a902e79e6cb02  fenced/seed_0x0000000053415421.traj
020c342784f6720d80e374d247b90412914052f79d385056062a683031759351  fenced/seed_0x0000000053415422.traj
035e1950cb7b222dbe8b3f471bf1fbb68218f906a2bc16aef0dad1f7370a01fe  fenced/seed_0x0000000053415423.traj
d22bde01fb3df238f0079f14548c4dc9d52dedf70119c73ee553bd264e3eaf65  fenced/seed_0x0000000053415424.traj
0a576fe10e589469a26ce61206bd813bb211861612768227cb307ce8c357a619  fenced/seed_0x0000000053415425.traj
5cc80583f760118933b2cb999cc0c9ede04cc7e2af2045fa78e3621d01568fab  fenced/seed_0x0000000053415426.traj
b4cb069b0be1986f219e12aaae25e54b40f23d72733005cd8bfac955acb77518  fenced/seed_0x0000000053415427.traj
e62c326283d4fc82da00c473d2bf26ddd5d318f0bb9e09b64968d7d347e0bdee  fenced/seed_0x0000000053415428.traj
```

The provenance claim is exactly this and no more (M-PROVENANCE-OVERREACH): the bytes are
the bytes the census banked. It is not a claim that the dynamics that produced them was
correct, and it is not a claim about any other arm.

---

## 3. THE SAMPLING RULE — staked, deterministic, no cherry-picking

Every clause below is fixed here and the instrument implements it literally. The
POPULATION is declared, not left as a silent free parameter (M-POPULATION-CHOICE), and
the rule's yield is reported whether or not it is convenient.

### 3.1 Compactness radius

> **`R_COMPACT = 6.0` bohr**, applied to the cluster DIAMETER: a five-set is a candidate
> iff `max_{i<j} |r_i - r_j| < 6.0`.

Justified from the engine, not chosen: `quaternary::R_CUT = 6.0` is documented as the
"measured far-field cutoff (bohr) where dE4 decays below T1 interpolation tolerance
(~5e-5 Ha)". Taking the five-cluster diameter below the SAME radius means every scored
cluster is one where the ladder's previous rung is still live. A larger radius would fill
the sample with configurations where dE4 has already vanished and dE5 is therefore small
for a reason that has nothing to do with the ladder terminating — which is the vacuity
section 3.5 refuses outright.

### 3.2 Frames

> **`STRIDE = 250` frames**, starting at frame 0, over all 20,000 frames of all eight
> `fenced` seeds: 80 frames per seed, 640 frames, `C(12,5) = 792` five-sets each,
> 506,880 candidate tests.

At the run's measured cadence (`dt = 0.5386` a.u. x 64 substeps) one frame is 0.834 fs,
so the stride is **208 fs** — 22 O–H stretch periods (9.12 fs) and 10 H–O–H bend periods
(20.9 fs).

### 3.3 The near-duplicate rule (M-FIXED-POINT-TRAJECTORY)

208 fs is shorter than a free H2O rotation (535 fs), so a persistent molecule would
otherwise be sampled many times as if it were many configurations.

> **For a given (seed, five-set of ARENA indices), at most ONE candidate is accepted per
> 2,000-frame block** (1.67 ps, more than three rotation periods). Arena indices, never
> sorted or spatial indices — two candidates are "the same five-set" only if the same
> physical nuclei are in it.

### 3.4 Selection, the cap, and the excess

Candidates are enumerated in a fully determined order: seed index ascending, then frame
index ascending, then five-set in lexicographic arena-index order. Then:

> **Per-composition quota 8**, over the three in-scope compositions, taken as a SYSTEMATIC
> sample of that composition's candidate list: with `m` candidates and quota `q`, take
> indices `0, floor(m/q), 2*floor(m/q), ...` so the draw spans the whole run rather than
> its first minutes.
>
> **Shortfall redistribution, in this fixed priority order and no other: `O2H3`, then
> `OH4`, then `H5`.** `O2H3` leads because it is the composition the `fenced` seed
> `0x53415422` actually formed (banked line: `molecules [H2 H2 O2H O2H3]`).
>
> **`N_TARGET = 24`, and the cap is DECLARED, not silent:** the results document prints,
> per composition, the number of candidates enumerated, the number drawn, and the excess
> left undrawn.

### 3.5 Anti-vacuity: the sample must be where rung four is still alive

A five-cluster in which the four-body term has already died is a configuration where the
ladder terminated one rung earlier, and a small `dE5` there says nothing about whether it
terminates at five.

> **A scored config is LIVE iff `max_{|Q|=4} |dE4(Q)| >= 5.0e-5` Ha** — at least one of
> its five quadruple terms is at or above the same bar the verdict uses.
> A config that is not LIVE is reported **VACUOUS**, counted, and is EXCLUDED from the
> verdict. It is never reported as a "terminates" success (M-VACUOUS-SUCCESS).

The verdict's `N` is the LIVE count, and it is the number the `>= 20` requirement is
applied to.

### 3.6 The landscape base rate (M-BASE-RATE-OMITTED)

> The results document prints the composition histogram of **ALL** compact candidates —
> the out-of-scope `O3H2` and `O4H` rows included, with counts — so the fraction of the
> visited landscape this audit actually covers is visible rather than implied.

---

## 4. THE BOUND

> **`BOUND = 5.0e-5` Ha.** `|dE5| < BOUND` reads TERMINATES for that config;
> `|dE5| >= BOUND` reads DOES NOT.

The reasoning, written before any reading:

1. `quaternary::R_CUT`'s own documentation names **~5e-5 Ha** as the T1 interpolation
   tolerance, and the four-body term is declared negligible — cut off entirely — once it
   falls below it. That is the ladder's DECLARED per-term uncertainty, in the engine's
   own words, and it is the only such number the ladder carries.
2. A five-body residue below it is invisible to the ledger the four-body term is served
   into: it is smaller than the error already accepted on the rung above it, so including
   it could not change any energy the engine reports.
3. A five-body residue at or above it is LARGER than the uncertainty at which the
   previous rung is already carried, which is precisely the condition under which
   truncating at four is not justified.
4. Corroborating the same order from a different direction: `TrimerTable`'s measured
   held-out maximum is 6.3e-5 Ha, so 5e-5 Ha is also the scale at which this engine's
   served three-body surfaces stop resolving anything.

The bound is a bound on the TERM, not on a total energy, and it is not converted into one.

**Secondary readings, staked here so they cannot be invented afterwards** (reported, not
gated): the distribution of `|dE5|`; its median and worst; the per-config ratio
`|dE5| / max_Q |dE4(Q)|`, which is the ladder's own convergence ratio at that geometry;
and the same quantities split by composition.

---

## 5. THE GATES

- **G1 — exactness of every rung.** Every one of the 26 subsystem solves per config
  (1 pentamer + 5 quadruples + 10 triples + 10 pairs) is `fci::solve_determinant`, and
  the instrument asserts `route` is the determinant route on all 26. 0 DMRG solves
  permitted. witness: `exact_never_degraded`
- **G2 — convergence of every rung.** Every one of the 26 solves must exit
  `SolveExit::Converged` or `SolveExit::Trivial`, AND carry `residual <= 1.0e-9`
  (`pair::CONVERGED_RESIDUAL = 10 * DAVIDSON_EXPANSION_FLOOR`). Any failure VOIDs the
  config; see section 6.
  **CORRECTION C-2, landed pre-data**: this row first also required `scf_converged = true`
  on all 26. It cannot be delivered on all 26, so the stake is amended here rather than
  quietly under-delivered. `pair::geometry_problem` — the only public entry point that
  hands back the `(space, mo, nuc)` triple `solve_determinant` needs, and therefore the
  only route to G1's exactness — DISCARDS the SCF convergence flag (`let (u, _, _) =
  orbital_rotation(...)`). `solve_geometry` reports the flag but chooses its own route,
  which G1 forbids. So the flag is required, and CHECKED, on every solve that goes through
  `solve_geometry` — every subsystem at or below `MPS_ROUTE_THRESHOLD`: 26 of 26 for `H5`
  and `OH4`, 25 of 26 for `O2H3` — and is recorded as UNOBSERVABLE, by name, on the one
  solve that cannot supply it. That solve is not left unguarded: the FCI energy is
  invariant under the orbital rotation for every geometry (`pair.rs`: "the FCI energy is
  invariant under U for every R"), so SCF convergence there changes Davidson's conditioning
  and not the answer, and the answer's own guard is the residual bound in this same row.
  The dropped flag is itself a finding — a diagnostic one entry point carries and its
  sibling does not — and it is reported to the crate rather than absorbed.
  witness: `none (a solver-convergence gate has no theorem here; SolveExit is the recorded fact and M-EXIT-DISCRIMINATOR is its warrant)`
- **G3 — the size fence.** No subsystem exceeds `FCI_DET_MAX = 250,000` determinants;
  a candidate whose pentamer exceeds it is never sampled, and the exclusion is counted by
  composition. witness: `none (an arithmetic scope fence on determinant counts; section 2.1 is its derivation)`
- **G4 — the trajectory pin.** Each trajectory's sha256 EXACT-matches section 2.5's line
  or the run refuses, naming the file. witness: `none (a byte-identity refusal, not a theorem)`
- **G5 — anti-vacuity.** A config is scored only if `max_Q |dE4(Q)| >= 5.0e-5` Ha;
  otherwise VACUOUS and excluded. witness: `none (an anti-vacuity gate on a measurement; M-VACUOUS-SUCCESS is its warrant)`
- **G6 — work count.** At least **20** LIVE scored configs, or the audit returns VOID
  with no verdict either way. witness: `none (a sample-size floor; the count is printed whether or not it passes)`
- **G7 — the bound.** `|dE5| < 5.0e-5` Ha per LIVE config; the branch is decided on the
  set of LIVE configs and on nothing else. witness: `lipDependsWithin_comp`
- **G8 — the error budget composes.** The 26 solves' residuals are each `<= 1.0e-9` Ha
  and there are 26 of them, so arithmetic error in `dE5` is bounded by `2.6e-8` Ha —
  **1,900x** below the bound. Reported per config as the assembly's residual sum, so the
  claim that the measurement resolves the bound is a measured number and not a hope.
  witness: `horizonBudget_le_of_nonexpansive`
- **G9 — the cross-check on `OH3`.** For every `OH3` quadruple in the sample, the live
  `dE4(Q)` and `quaternary::de4_ohhh_fci` on the same four centres are both printed and
  their difference reported; a difference above 1.0e-3 Ha (the served-table budget of
  section 1.1) is a FINDING about the two bases, printed at full volume, never averaged.
  witness: `none (a two-basis comparison; TRIMER_TABLE_SCHEMA.md's subtraction_basis law is its warrant)`
- **G10 — the wall clock budget.** 3,600 s per config. Exhaustion VOIDs that config,
  loudly, and may never fall back to a scorable verdict (M-BUDGET-LAUNDER). Placement is
  recorded with the reading (`nice -n 10`, host, core class) so a budget VOID cannot be
  confused with a slow machine (M-PLACEMENT-LOTTERY). witness: `cheap_but_over_budget_not_selected`

---

## 5b. AMENDMENT A-1 — POST-DATA, and labelled that way in every restatement

**This amendment was made AFTER scoring began. It is not a correction and it is not
pre-registered; it is a stake changed in the light of a reading, which is the weakest kind
of change this programme permits, and it is recorded at full volume rather than absorbed.**

### What fired it

The first four drawn configs VOIDed. All four VOIDed on the SAME clause and the SAME rung:
an `(O,H,H)` triple at an ordinary water geometry (`r_min` 2.02–2.16 bohr, 441 determinants)
reporting `scf_converged = false`. Extended to the full draw, the rate is **68 of 240
triple solves (28%)**, and because every config contains ten triples, **24 of 24 configs
VOID**. A gate that refuses 100% of its sample is measuring itself, not its subject — the
same shape M-BUDGET-LAUNDER names, arriving through the convergence clause instead of the
budget one.

### The measurement that decided it, not the inconvenience

A gate is not amended because it fires. It is amended only if it can be shown to be reading
a quantity other than the one it claims. So the question was put to an INDEPENDENT
reference — the committed `(O,H,H)` surface in
`engine/crates/holon-chem/tests/data/s2/s2_water_table.txt`, generated by another campaign,
on another day, through `examples/s2_table.rs` — and answered by measurement
(`conformance/water_observatory/de5_scf_probe.log`, instrument mode `--scf-probe`):

| SCF flag on the live solve | n | worst \|live − served\| |
|---|---:|---:|
| `scf_converged = true` | 76 | 5.386e-4 Ha |
| `scf_converged = false` | 68 | 6.678e-4 Ha |

**The flag does not predict disagreement.** The two worst cases differ by a factor of 1.24,
both land on the same stretched near-linear `H–O···H` shape (`r_OH` ≈ 3.8/2.0, `r_HH` ≈ 5.7),
and the largest disagreements occur on BOTH sides of the flag. Pairs, meanwhile, are 240
converged and 0 not.

This is what the freeze already said in correction C-2, now measured instead of argued: the
FCI energy is invariant under the orbital rotation, so a rotation the convenience-SCF failed
to iterate to 1e-10 in 200 damped steps changes Davidson's conditioning and not the answer.
G2 as frozen therefore contradicted the freeze's own stated reasoning, and the contradiction
was invisible until it refused everything.

### The change, and what is NOT changed

> **`scf_converged = false` is RECORDED per config — count, arity, and geometry — and is no
> longer a VOID condition.** VOID keeps its other three clauses unchanged:
> `SolveExit::IterationCap`, `SolveExit::Stagnated` above the residual bound, and the G10
> budget. The residual bound is untouched at 1.0e-9 Ha and remains the guard on the answer.

Nothing else moves. The bound stays 5.0e-5 Ha, the sample stays the 24 already drawn by the
frozen rule, the compositions stay fenced, and no branch is re-worded.

### The discipline that comes with a post-data amendment

> **Both readings are computed and published side by side, in every restatement, with the
> strict one first.** The STRICT reading is G2 exactly as frozen: 24 of 24 VOID, branch (d),
> no verdict about the ladder. The AMENDED reading is whatever the scoring returns under the
> clause above. A headline that quotes the amended reading without the strict one beside it
> is a headline this document forbids in advance.

Owner of the residual obligation: the crate. A 28% SCF failure-to-converge rate on `(O,H,H)` at
trajectory geometries is a fact about `pair::orbital_rotation`, not about this audit, and
the served water table was built through those same solves. It is reported to the crate and
not fixed here.

---

## 6. VOID — what it is, how it is counted, and why it is never scored

> **A config VOIDs if any of its 26 solves exits `SolveExit::IterationCap`; or exits
> `SolveExit::Stagnated` with `residual > 1.0e-9`; or exceeds G10's budget.**
>
> The `scf_converged = false` clause that stood here was REMOVED by amendment A-1
> (section 5b) after it VOIDed 24 of 24 configs and was measured not to predict any
> disagreement with an independent reference. It is recorded per config instead. The
> STRICT reading — this list with that clause restored — is computed and published beside
> the amended one every time.

A VOID config is counted, its reason named, and it is **never scored** — not as a pass,
not as a fail. Budget exhaustion is a structural property of the subject and can
correlate with the label: `O2H3` is 204,490 determinants and `H5` is 100, so a gate that
scored exhausted cases would be measuring determinant count and calling it physics
(M-BUDGET-LAUNDER). The results document therefore prints the **VOID STRUCTURE** — VOIDs
by composition and by reason — so a correlated refusal pattern is visible instead of
buried in an aggregate.

`IterationCap` and `Stagnated` are recorded separately and both are READ, not merely
carried (M-EXIT-DISCRIMINATOR): they are different facts with opposite remedies
(more budget vs. the arithmetic tier is exhausted), and a residual alone cannot tell them
apart.

**And the maximum is a bound on the SCORED set only** (M-MAX-OVER-SUCCESSES). The
headline "worst |dE5|" is reported in the same sentence as the VOID count, every time; a
maximum taken over a set from which the hard cases were removed is not a bound on the
ladder, and this document refuses in advance to let it be quoted as one.

---

## 7. THE BRANCHES — every answer's meaning, staked in advance

Evaluated in this precedence, top to bottom, and no other:

* **BRANCH (d) — VOID.** Fewer than 20 LIVE scored configs after G2/G5/G6/G10.
  → The audit did not measure what it set out to measure. **No verdict about the
  ladder either way.** The failing gate is named, the VOID structure is printed, and
  GANTT node H stays LAUNCHED rather than flipping to a receipt.

* **BRANCH (c) — MIXED BY COMPOSITION.** At least two in-scope compositions each carry
  `>= 6` LIVE configs and they disagree: every LIVE config of one composition is under the
  bound and at least one LIVE config of another is at or above it.
  → The ladder terminates at four for the passing compositions and does not for the
  failing ones, at planar geometries. The seam requirement fires **for the failing
  compositions only**, named. The fence is by composition, stated in those words.

* **BRANCH (b) — DOES NOT TERMINATE.** At least one LIVE config has
  `|dE5| >= 5.0e-5` Ha, and (c) does not apply.
  → **The four-body truncation is not justified at planar compact geometries.** The
  DMRG-cluster seam requirement of GANTT's `MPS` node fires — "DMRG for compact cores,
  MBE far-field, seam defect-audited", which that node lists as gated on precisely this
  verdict. The worst config's full geometry is printed in bohr to 12 significant figures,
  with its composition, seed, frame, arena indices, all five `dE4(Q)`, and the ratio
  `|dE5| / max_Q |dE4(Q)|`.
  **No five-body term is written.** Node H is "measure, never build", and a lane that
  answers a truncation question by extending the truncation has answered a different
  question.

* **BRANCH (a) — TERMINATES.** Every LIVE config has `|dE5| < 5.0e-5` Ha, with
  `N_live >= 20`.
  → The ladder's termination certificate, **with its scope in the same sentence every
  time it is stated**: planar (2D) geometries, STO-3G, compositions `{H5, OH4, O2H3}`,
  cluster diameter below 6.0 bohr, `N_live` configs, subtraction basis `fci_live`.
  It is NOT a certificate for `O3H2`, for `O4H`, for three-dimensional geometries, or
  for any basis but this one.

**Pre-committed follow-ups — a branch is design, a rescue is post-hoc:**

* If **(b)** or **(c)**: the SAME instrument, unchanged in every threshold, is run on the
  worst config's cluster at 1.25x and 1.5x its diameter, to report whether `|dE5|` falls
  with separation as a genuine five-body term must. A `|dE5|` that does NOT fall with
  separation is an ARITHMETIC finding about the assembly, not a physics finding about the
  ladder, and would be reported as such.
* If **(a)**: the same instrument is run on the `hydrogen` arm's eight seeds (pinned in
  the same manifest) with no threshold changed, and the `H5` pass rate reported. A
  certificate that passes as readily on a hydrogen-only scene as on the mixed one has
  certified less about water than it appears to.
* Under **every** branch: the composition histogram of section 3.6 is published, so the
  reader can see how much of the visited landscape the audit's determinant fence excluded.

**What no branch does:** promote a stance claim, write a five-body term, or restate the
verdict without its planar/STO-3G/composition scope.

---

## 8. PLANTS

Each plant names its carrier and the sector it must be nonzero in (M-PLANT-SECTOR), and
each must be shown to FIRE on this instrument before any reading is trusted
(M-PLANT-OBS). All three run on a config drawn by the rule of section 3, not on a
synthetic geometry, so the plant exercises the production path.

* **P-1 — the dropped quadruple, must shift EXACTLY.** Carrier: the single quadruple term
  `dE4(Q*)` with the largest `|dE4|` in the config. The plant deletes it from the
  `E_MBE4` assembly and from nothing else. Sector the plant acts on: the FOUR-BODY
  sector; the planted signal must be nonzero in that sector, and the instrument REFUSES
  to run P-1 on a config where `|dE4(Q*)| < 1.0e-6` Ha, because a plant smaller than the
  arithmetic could not be seen and would prove nothing.
  Expected, and staked as an identity rather than a tolerance:
  `dE5_planted - dE5_true = +dE4(Q*)` to within **1.0e-12** Ha. Anything else means the
  instrument cannot see what it measures.
* **P-2 — the separated atom, must read ZERO.** Carrier: one atom of the config,
  translated to 40 bohr from the cluster's centre. Sector the plant acts on: the GEOMETRY
  sector, where it must be nonzero (the displacement is tens of bohr) while the
  five-body sector must read exactly zero. Expected: `|dE5| < 1.0e-8` Ha. The zero is a
  fact about the SCENE and not about the instrument's coverage — the MBE is exactly
  size-consistent, so a four-atom cluster plus a distant atom has an identically
  vanishing five-body term, and an instrument that cannot reproduce that identity has an
  assembly error, not a physics result.
* **P-3 — the exhausted budget, must VOID.** Carrier: `fci::DAVIDSON_MAX_ITER`, lowered
  so a real subsystem solve genuinely fails. Sector the plant acts on: the SOLVER-EXIT
  sector, where it must be nonzero (`SolveExit::IterationCap` where the unplanted run
  reports `Converged`). Expected: the config is reported **VOID (iteration cap)**, is
  absent from the scored table, and appears in the VOID structure with its reason. This
  plant exists solely to prove M-BUDGET-LAUNDER's refusal is mechanized rather than
  promised: a run that scored the exhausted case would be caught here and nowhere else.

---

## 9. WHAT THIS AUDIT DOES NOT CLAIM

* It does not claim the many-body expansion converges. It measures one residue, at one
  arity, on one basis, at planar geometries drawn by one stated rule.
* It does not claim a five-body term is unnecessary in general. `O3H2` and `O4H` are
  refused by the determinant fence and the verdict is silent about them.
* It does not claim the engine's four-body FORCE path is complete: that path covers
  `OHHH` only, and section 2.2 records the `O2H2`/`H4` gap as a node-A finding.
* It does not claim anything about three-dimensional water. The source trajectories are
  two-dimensional and every statement above inherits that.
* It does not claim the `fenced` arm's dynamics was correct. The trajectory is a geometry
  sampler; the pin claims byte identity and nothing more.
* It does not claim a null. A `|dE5|` below the bound is "smaller than the ladder's own
  declared per-term uncertainty on this sample", which is a bounded statement about a
  measured set, not an assertion that the quantity is zero (M-NULL-MISSTAKE).
