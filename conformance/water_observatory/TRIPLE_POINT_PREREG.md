# THE TRIPLE POINT OF THE MODEL — prereg

*Frozen 2026-09-01, before the instrument exists. The ice-XI builder this design
depends on (`holon_render::lattice::ice_xi`) is UNDER CONSTRUCTION in a parallel
lane and is cited by name, as the house allows for a freeze written before its
instrument. No trajectory of a three-phase box has been dumped, no phase fraction
has been read, and no grid point has run. Everything below is stakes; the git
history is the check.*

**THE CLAIM'S DOMAIN, STATED FIRST AND NEVER RELAXED: this campaign locates the
phase diagram OF THE MODEL — nuclear charges Z, masses, the STO-3G contraction,
MBE(2,3) plus the exact four-body (O,H,H,H) term, classical nuclei on a periodic
box. It is not Earth's phase diagram, it is not a measurement of water, and no
number in it may be compared to 273.16 K / 611.657 Pa as though agreement were
evidence and disagreement were error.** Real-water values may appear in a results
document as literature anchors beside the model's own numbers, exactly as
`conformance/water_observatory/OBSERVATORY_BRIEF.md` §0 permits, and never as a
scoring reference (M-FOREIGN-DOMAIN-CORROBORATION).

**misfits:** contacts M-BUDGET-LAUNDER (budget exhaustion VOIDs a grid point
loudly and may never fall back to a scorable verdict — compute expense here is a
structural property of the subject, because a dense ordered region costs more per
frame than a dilute one, so a gate that scored exhausted points would be measuring
density and calling it a phase diagram; the VOID structure is reported as a map
over the grid so a correlated refusal pattern is visible), M-STALE-INSTRUMENT (the
physics rung is NOT frozen by this document: every run names the commit, the
binary, and the gate battery it ran under, and no results document may cite a
scratch path or a shared working tree as an instrument), M-DEVICE-CLASS (declared
below: CPU-only, x86-64, one core class, no GPU arm, and no bitwise agreement
asserted across device classes), M-PLACEMENT-LOTTERY (this campaign scores physics,
not speed — no verdict anywhere in it is a function of wall clock; timing enters
only through the CPU-second budget, and the core class is declared because an
undeclared placement is an undeclared variable), M-IDLE-CALIBRATED-TIMEOUT (no band,
floor or budget in this file is frozen from a wall-clock reading taken on a loaded
box; the cost model is in CPU-seconds and its loadavg is recorded at both ends),
M-CHEAPER-THAN-ITS-PRICE (the campaign's FIRST act is a cost measurement, and a
grid point that returns cheaper than the measured per-point price is refused, not
banked), M-VOLUME-SCALE (a lattice campaign at one N is a campaign at one N: the
boundary's LOCATION is scoped to the staked N, and a finite-size leg at 2N runs at
the boundary), M-HOMOG (a slab geometry is deliberately inhomogeneous; every
per-molecule reading is local by construction and no local reading is reported as a
box property), M-PLANT-OBS and M-PLANT-SECTOR (every plant below names its carrier
and the sector the plant acts on, and is pre-checked to fire on THIS instrument),
M-VACUOUS-SUCCESS (the moving-carrier and work-count gates are inherited from the
census unchanged, and every gate asserts its work count), M-FIXED-POINT-TRAJECTORY
(a frozen carrier VOIDs), M-NULL-MISSTAKE (the nulls are matched to the data's
generative structure — integer counts on an autocorrelated series, so the null is a
block bootstrap over whole frames, never an iid multinomial), M-BASE-RATE-OMITTED
(the phase-assignment misassignment rate is measured on pure-phase references and
printed beside every fraction), M-SORTS-NOT-SEPARATES (a count triple is not a
region triple; the contiguity statistic is what separates them and it has its own
plant), M-TAG-AS-PROPERTY (the phase of a molecule is computed from its coordinates,
never from which region it was BUILT in; the builder's region tag is carried in a
structure the classifier's signature cannot reach), M-CONJUNCTION-MONOTONE (the
coexistence criterion is a conjunction of three floors and its passing fraction is
non-increasing in the number of floors by theorem; the criterion is scored against
its own null, never quoted as though thinning were evidence),
M-FOREIGN-DOMAIN-CORROBORATION (agreement with real water is not evidence about
this model and disagreement is not error), M-EXIT-DISCRIMINATOR (every run records
WHY it stopped as a read field, not a boolean), M-MAX-OVER-SUCCESSES (the
coexistence region is defined by the floors, never by the largest passing point),
M-UNTESTED-GAP (the grid's bounds are justified against quantities this repository
has MEASURED, and where nothing has been measured on an axis the bound says so),
M-MAINTENANCE-LENS (the thermostat is a control, not a repair; no rent-clause or
maintenance reading is taken from it, and every lens declares the variable it
measures and refuses where the scene cannot carry it),
M-PROVENANCE-OVERREACH (the launch header records binary sha256, repo HEAD, build
exit status and tree-dirty state as four separate MEASURED lines and labels which
of them is an inference), M-PROBE-THE-RESOURCE (worker leasing goes through the
arena's own probe, and no liveness or health check number is read as a resource
count), M-LOOP-BLIND (the trajectory loop is the consumer, not an instrument; no
holonomy or loop quantity is read here), M-PARITY-PROTECT, M-ONE-MODEL-DELTA,
M-COND-PROBE, M-BARE-CHARGE, M-GAUGE-LAUNDER (not otherwise contacted).

---

## 0. The question, and what it is not

Does the model — this exact Hamiltonian, these exact nuclei — have a region of its
own (T, density) plane where a seeded solid, a liquid and a vapor coexist without
any of the three collapsing? The programme has certified ONE molecule
(`conformance/water_observatory/CENSUS_PREREG.md`, and the full-strength arm in
`conformance/atomworld/p2_de4_full/README.md`). A molecule is not a substance. The
gap between "this model made an OH2 that is a persistent quotient" and "this model
has water as a material" is exactly the gap this campaign measures, and it is a gap
the campaign is staked to be able to report as UNBRIDGED.

**This campaign can wound the water claim and cannot manufacture it.** If the model
holds no solid at any point of the staked grid, the correct headline is that the
programme's certified water is a molecule and not a phase, said at full volume,
with the longest-surviving solid fraction printed so the distance to the bar is
visible. That branch is written in §6 before any point runs.

### The three things this design inherits, named

1. **SEEDED COEXISTENCE, NOT NUCLEATION.** `OBSERVATORY_BRIEF.md` §5 keeps v1's
   design: the seeded ice|liquid coexistence slab is the measuring instrument, and
   a phase boundary is the point where one Closed view ceases to be the
   minimal-error chart. This campaign does not wait for a phase to nucleate; it
   plants all three and asks which survive. Spontaneous nucleation is a different
   experiment with a different (and far worse) sampling problem, and it is out of
   scope by this ruling, not by convenience.
2. **THE BOX-SCALE DOOR IS A DENSITY KNOB, AND IT SCALES THE CONTENTS.**
   `holon_box_scale(f)` scales container and contents affinely
   (`engine/crates/holon-render/src/lib.rs`), so the second grid axis is density and
   the ice seed is at its own equilibrium lattice at exactly ONE value of `f`. This
   is stated here rather than discovered later: at `f ≠ 1` the seed is strained,
   and the design answers it with a relaxation window (§3.2) before any measurement
   window opens.
3. **THERE IS NO "UP" IN A PERIODIC BOX.** The inherited sketch says "vapor gap
   above". Under boundary mode 2 the gap is a z-slab of empty volume in a periodic
   cell, not an atmosphere: `holon_gravity_available` returns 0 on a periodic box
   and the field is REFUSED by name (`sim::GravityRefusal::PeriodicBox`). Nothing
   holds the vapor up and nothing needs to. Any results document describing the gap
   as gravitationally supported has described a different experiment.

---

## 1. THE MODEL, THE DOORS, AND THE PRECONDITION

### 1.1 The model

Nuclear charges, masses, the STO-3G contraction; the pair and three-body surfaces
under MBE(2,3); the exact four-body (O,H,H,H) term. Classical nuclei (tier T3).
Periodic box, boundary mode 2. Thermostat on, target in kelvin. No external
potential, no force field, no phase-specific parameter — `OBSERVATORY_BRIEF.md`'s
admissibility rule holds unchanged.

**The rung is named at launch, never here.** The physics rung current when the
sweep launches is recorded per run by commit, binary sha256, build exit status and
gate battery, as four separate measured lines. This document freezes a DESIGN. It
does not freeze a physics version, and a results document that cannot name its
rung is not banked (M-STALE-INSTRUMENT).

### 1.2 The control surface — landed doors only

Controls: `holon_reset(n)`, `holon_set_dims(3)`, `holon_set_boundary(2)`,
`holon_set_thermostat(on, target_kelvin)`, `holon_box_scale(f)`. Nothing else
touches the scene. In particular no barostat setpoint door is used, because none
ships — pressure is READ, never chased.

Readouts: `holon_temperature`, `holon_pressure` gated by `holon_pressure_defined`,
`holon_census_molecules` / `holon_census_atoms` / `holon_census_formations` /
`holon_census_dissolutions` / `holon_census_closure_rejections` /
`holon_census_global_views`, and the per-row door
(`holon_row_kind`, `holon_row_member`, `holon_row_member_count`,
`holon_row_closure_defect`, `holon_row_formed_at_frame`). The 3D lens stack is
`engine/crates/holon-lens/src/lens.rs`, driven as
`engine/crates/holon-lens/examples/census.rs` already drives it.

### 1.3 PRECONDITION P0 — the four-body term must be LEASED, not solved in the loop

The banked full-strength arm
(`conformance/atomworld/p2_de4_full/seed_0x53415422.log`) is 12 atoms over 20,000
frames at 5,101 s wall, of which 891 in-loop four-body gradient solves dominate:
`conformance/water_observatory/DE4_TABLE_PREREG.md` measures that gradient path at
a MEAN of 9,837.5 ms per evaluation. A three-phase box is 24× that atom count and
the campaign is 67 runs. In-loop solving is not expensive here, it is impossible.

> **P0: this campaign launches only when the (O,H,H,H) surface is TABULATED and
> SERVED through the door chain FSD-W2 stakes (`holon_set_de4`, with the functional
> evaluation counter). If P0 is unmet, the campaign is NOT RUN.**

NOT RUN is a state, not a verdict, and it is reported as one. The pre-committed
alternative of running an MBE3-only arm instead is **REFUSED**, and the reason is a
measurement already in the record, not a preference: on the one seed that made
water, MBE3 alone stops at hydroxyl (`p2_de4_full/README.md`, fenced row). An
MBE3-only sweep would be a phase diagram of a model that does not make the
molecule, reported under this campaign's name. That is the substitution this clause
exists to forbid.

### 1.4 Device class and core class, declared

CPU-only, x86-64, this box (i9-13900HX, 32 logical cores, P-cores carrying an SMT
sibling and E-cores not). **No GPU arm exists in this campaign and no bitwise
agreement across device classes is asserted** (M-DEVICE-CLASS). The cost
measurement of §4 is pinned with `taskset` to an E-core — no SMT sibling, therefore
reproducible — and quotes a RANGE over measured placements rather than a single
number (M-PLACEMENT-LOTTERY). Production points are NOT pinned, because pinning
restricts without reserving and the production quantity is CPU-seconds, which
descheduling does not inflate.

---

## 2. THE SCENE

### 2.1 Composition and layout

> **N = 288 atoms = 96 H₂O.** One periodic cell, three regions stacked along z:
> a solid slab from the ice-XI builder, a liquid region beside it, and an empty
> z-slab (the vapor volume). Nominal split 32 / 32 / 32 molecules, the third
> region starting empty and filled only by whatever leaves the other two.
>
> **Cell geometry at f = 1.00, staked:** cross-section **24 × 24 bohr**; the two
> condensed regions occupying **≈ 36.6 bohr** of z at the ice-XI molecular volume
> (≈ 219.4 bohr³ per molecule × 96 / 576 bohr²); the empty gap **14.2 bohr**.
> Total z extent **≈ 50.8 bohr (26.9 Å)**, which is the axis every quantity below
> that mentions "the cell" is measured along, because it is the axis the three
> regions stack on.

Why 96 and not more: the coexistence floor of §3.4 is derived FROM this number
(0.15 × 96 = 14.4 molecules, the smallest region that can have an interior at all),
and the campaign's total price scales linearly in it. Why not fewer: at 48
molecules the floor would be 7.2 molecules, which is a cluster and not a region, so
the criterion would have no meaning to measure.

> **The ice slab must be ≥ 3 molecular layers thick along z.** A two-layer slab is
> entirely surface and has no bulk interior to hold order; a solid fraction read off
> such a slab measures the interface.

If the builder's smallest supercell that is periodic in x and y AND ≥ 3 layers
thick cannot land exactly 32 molecules, the campaign takes the smallest supercell
that satisfies both structural conditions and RECORDS the resulting N in every run
header, recomputing the §3.4 floors from it by the stated formula. The rule is
frozen here; only the integer moves, and it moves in the open.

### 2.2 Blindness of the classifier

The builder knows which region each molecule was placed in. **The phase classifier
must not.** The region tag is carried in a structure the classifier's signature
cannot reach — blindness enforced by the type, exactly as `CENSUS_PREREG.md` §7
enforces it for the ice/liquid/vapor launch label (M-TAG-AS-PROPERTY). A phase
verdict computed from a builder tag is a lookup wearing a measurement's clothes,
and plant P-8 convicts it.

---

## 3. THE STAKES

### 3.1 The measurement window `W`

Staked in PHYSICAL TIME, not frames, because `dt` is derived per scene by the
governor and differs between scenes. Frames convert per run as
`t_frame = dt · SUBSTEPS · 2.4188843265e-2 fs`; at the fine `dt = 0.5386` of the
banked corpus this is 0.8338 fs per frame.

> **W = 8340 fs (8.34 ps) — exactly ten census windows.**

Why that number, stated before it is used, on the same reference motions
`CENSUS_PREREG.md` §2.1 used so the two instruments share one ruler:

| reference motion | period | periods inside W |
|---|---|---|
| O–H stretch (3657 cm⁻¹) | 9.12 fs | 914 |
| H–O–H bend (1595 cm⁻¹) | 20.9 fs | 399 |
| free rotation of H₂O at 300 K (I ≈ 3.0e-47 kg m²) | 535 fs | 15.6 |
| sound traversal of the cell's z extent (26.9 Å) at ~1500 m/s | 1.8 ps | 4.6 |
| the closure census's own certified-molecule window | 834 fs | 10 |

The binding entry is the fourth, not the first. A phase fraction is a property of a
REGION, and a region cannot be said to persist until the box has had time to
communicate with itself several times over; 4.6 sound traversals is that. The
~1500 m/s is an ORDER-OF-MAGNITUDE SCALE for a hydrogen-bonded liquid, used to
size a window and never as a reference value — **this model's own sound speed is
unmeasured**, and the campaign prints the measured longitudinal relaxation it can
extract from each run beside the window it used (M-FOREIGN-DOMAIN-CORROBORATION:
a number borrowed to size an instrument is not a number borrowed to score one). The
stretch and bend periods are carried because they are what the census's certified
rows are made of, and W must not be shorter than the window those rows were
certified over — it is ten times longer. At the fine `dt` this is **W = 10,000
frames**.

### 3.2 The relaxation window `R`, which is not measured

> **R = 4170 fs (4.17 ps), run and DISCARDED before W opens.**

Both `holon_box_scale(f)` and the thermostat set are external pushes: the affine
scale move launches a compression wave and the thermostat injects or removes energy.
R is staked at exactly W/2, which is 2.3 sound traversals of the cell's z extent — enough for
the scale move's wave to cross the box and return twice. No quantity measured
inside R appears in any verdict. R's own cost is charged to the budget in full,
because a discarded frame costs what a scored one costs.

### 3.3 The grid — Stage B, frozen here

**Temperature axis.** Six points, geometric, ratio exactly 2:

> **T ∈ {50, 100, 200, 400, 800, 1600} K.**

* Lower bound 50 K: below this the classical-nuclei chart is not a chart of this
  system at all. `C1_GATE_RESULTS.md` measured the anharmonic vibrational zero-point
  energy of the engine's own STO-3G FCI **H–H** curve (not O–H — the O–H ZPE is
  OWED, and the argument is stated at the strength the record actually supports): a
  hydrogenic stretch ZPE on this surface is of order 5e-3 Ha, more than thirty times
  kT at 50 K (kT = 1.58e-4 Ha). Below that temperature the quantum carrier is doing
  all the work and a classical solid is an artifact of the tier, not a phase of the
  model. The bound is honest scope, not convenience.
* Upper bound 1600 K: kT = 5.07e-3 Ha there, still 24× below the model's own
  measured O–H well depth D_e = 0.122901 Ha (`p2_de4_full/seed_0x53415422.log`
  header), so molecules are not thermally shredded by the pair curve — but it is far
  above every many-body residual the surface carries, so if the model has a vapor at
  all it must be vapor here. An upper bound must be a temperature at which the
  expected phase is UNAMBIGUOUS, and this is that.
* Ratio 2: the location of this model's melting boundary is not known to within an
  order of magnitude — nothing in this repository has measured it — so a geometric
  ladder brackets it in the fewest points. Staking a linear grid around a guessed
  centre would be M-UNTESTED-GAP: interpolating across a hole in our own data with
  confidence imported from a different axis (real water's).

**Box-scale axis.** Five points, relative to the seeded reference box at which the
ice slab sits at its own lattice constant:

> **f ∈ {0.85, 1.00, 1.20, 1.50, 2.00}.**

* f = 1.00 is the anchor: the seed at its own equilibrium lattice, the only value of
  f at which it is unstrained.
* Lower bound 0.85: the staked gap is 14.2 bohr at f = 1.00 and 0.85 × 14.2 = 12.07
  bohr, so 0.85 is the last point at which the empty z-slab still clears
  2 × R_CUT = 12 bohr. Below it the gap thins under two cutoffs, so
  the two liquid surfaces see each other through the gap and there is no vapor
  REGION left to measure — only a periodic image contact. The bound is set by the
  cutoff, which is a property of the instrument, so it is checkable.
* Upper bound 2.00: mean density 1/8 of reference. Past this the box is dilute
  enough that "all vapor" is a statement about the geometry we chose rather than
  about the model, and the grid would be answering the wrong question. 2.00 is where
  that starts, and the campaign stops there rather than buying points that cannot
  discriminate.
* Spacing deliberately non-uniform (0.15, 0.20, 0.30, 0.50): equal in log to within
  a factor of ~1.2 per step, which is the same bracketing argument as the T axis on
  an axis whose interesting structure is also unlocated.

**Stage B is 6 × 5 = 30 grid points, one seed configuration.**

### 3.4 Phase assignment, and the coexistence criterion

Assignment is per molecule, per frame, from coordinates only.

* A molecule is eligible for assignment only if it is a CERTIFIED census row —
  budgeted-or-better under the census's own inherited stakes: **β = 0.02** breach
  budget with **L_flick = 8.4 fs** maximum breach run, both taken unchanged from
  `CENSUS_PREREG.md` §2.2 so that census and this campaign read ONE instrument and
  not two.
* **VAPOR** if the molecule's oxygen has **zero** other oxygens within
  **R_nb = 6.6140 bohr** (3.5 Å — the Luzar–Chandler O···O cutoff already staked in
  the census lens stack, reused rather than reinvented). The threshold is the
  weakest possible one, so the vapor fraction is a LOWER bound: a dimer in the gap
  reads as not-vapor, which is conservative against the claim being tested.
* **SOLID** if not vapor and the q-tetrahedral reading over its four nearest oxygen
  neighbours exceeds `q_s`.
* **LIQUID** if certified, not vapor, not solid.
* **UNASSIGNED** otherwise, and the unassigned fraction `f_U` is printed for every
  point, never silently dropped.

**`q_s` is frozen as a RULE, not as a number, and the reason is that a number here
would be imported from real water.** Published tetrahedrality modes for liquid and
ice are facts about a different Hamiltonian (M-FOREIGN-DOMAIN-CORROBORATION).

> **`q_s` = the midpoint of the median q of the pure-ice-XI reference run (plant
> P-1) and the median q of the pure-liquid reference run (plant P-3)**, both
> measured in Stage 0 before any grid point runs, both printed.
>
> **SEPARATION GATE: the two medians must differ by ≥ 0.25**, and the resulting
> midpoint threshold must give a **misassignment rate ≤ 0.05** on the two reference
> runs (the fraction of P-3 molecules above `q_s` plus the fraction of P-1 molecules
> below it). Otherwise the lens does not separate THIS model's phases and the
> campaign VOIDs before spending a grid point on it.

0.25 is a quarter of the lens's full range (the lens reads exactly 1 on a perfect
tetrahedron and averages 0 on an ideal gas, both gated to 1e-12 in
`engine/crates/holon-lens/src/lens.rs`). 0.05 is the instrument's own error floor and
it is staked at exactly one third of the 0.15 coexistence floor below, because an
instrument whose assignment error is not strictly smaller than the effect it
measures is not measuring the effect.

**The coexistence criterion**, all clauses required:

> Over W, with fractions taken per frame over eligible molecules:
> **(i)** window-mean of each of f_S, f_L, f_V ≥ **0.15**;
> **(ii)** window-minimum of each ≥ **0.10**;
> **(iii)** a linear fit of each fraction over W, extrapolated ONE further W
> forward, stays ≥ 0.10;
> **(iv)** f_U ≤ **0.20**;
> **(v)** the contiguity statistic of §5 passes for S and for L.

0.15 is 14.4 molecules at N = 96 — a region with fewer members than that is entirely
surface and has no interior, so it is an interface and not a phase. 0.10 is 9.6
molecules, the point at which the region is a cluster. Clause (iii) is the one that
stops "has not collapsed yet" from reading as "coexists"; without it a monotone
decay caught mid-fall would score. Clause (iv)'s 0.20 is where the three floors stop
being separable from the unassigned pool's own variation, since 3 × 0.15 = 0.45
leaves 0.55 and an unassigned pool larger than a fifth of the box is competing with
the phases for the same molecules.

### 3.5 The thermostat must deliver

> **|mean(T_measured over W) − T_set| / T_set ≤ 0.20**, else the point is VOID
> (instrument), never a physics result.

The bar is on the WINDOW MEAN and not on frames, and the reason is measured: the
banked P2 log shows instantaneous T ranging 242–409 K around a 300 K setpoint, so a
per-frame bar would fire on a working thermostat. 0.20 on the mean is loose enough
to pass a thermostat behaving as the record shows one behaves, and tight enough that
a point silently running at half its nominal temperature cannot be scored.

---

## 4. PRICE AND BUDGET — no calendar, no duration promise

**This document contains no time estimates and no calendar framing.** The only
time-like quantities in it are physical simulation time (fs, ps) and a CPU-second
budget. Sizing is grid-points × measured per-point cost, and **the cost measurement
is the campaign's first act.**

### 4.1 The prior, labelled as one

From `p2_de4_full/seed_0x53415422.log`: 12 atoms, 20,000 frames, 5,101 s wall of
which the in-loop four-body solves dominate; the non-solve remainder is ≈ 700 s,
i.e. ≈ 0.035 s/frame at 12 atoms, at loadavg 78.22 on 32 cores. With cell lists the
cost model is O(N) at fixed density, giving ≈ 0.84 s/frame at 288 atoms, i.e. ≈ 3.5
core-hours per grid point over R+W = 15,000 frames, i.e. **≈ 235 core-hours for 67
points.**

**That number is a PRIOR, not a stake.** Its inputs are wall clock on a loaded box,
which M-IDLE-CALIBRATED-TIMEOUT and M-PLACEMENT-LOTTERY both forbid freezing a band
from. It appears here so that the budget below can be justified against something,
and so that a reader can see what would have to be wrong for the budget to fail.

### 4.2 The budget, staked

> **B = 400 core-hours of CPU time total**, measured as CPU-seconds from
> `/proc/self/stat` fields 14+15 (not wall), split:
> **G0 cost measurement ≤ 4 · Stage A ≤ 72 · Stage B ≤ 180 · Stage C ≤ 144.**

400 is 1.7× the §4.1 prior. The factor is deliberately NOT large: M-PLACEMENT-LOTTERY
measures contention on this box costing up to 3.5× in absolute terms, so 1.7× is
knowingly insufficient to absorb a full contention surprise. **The campaign is
expected to VOID rather than quietly overspend if the prior is wrong by more than
that**, and that is the point of the number. A budget with enough headroom to
survive any surprise is not a stake.

Budget exhaustion at any level **VOIDs loudly and is never scored**
(M-BUDGET-LAUNDER). The VOID structure is published as a map over the (T, f) grid,
because a refusal pattern correlated with density is exactly the failure mode this
misfit names — dense ordered points cost more per frame, so an unreported VOID map
would silently delete the solid corner of the diagram.

### 4.3 Price closure

> Every grid point's measured CPU-seconds are compared against G0's per-point price.
> A point returning below **0.5×** the price is REFUSED, not banked, and its cause is
> found before anything else proceeds (M-CHEAPER-THAN-ITS-PRICE).

The founding case in this repository is a "table" that arrived 65× cheaper than its
own banked price and turned out to be a hand-shaped fit. 0.5× is chosen because
legitimate spread from placement is measured at up to 1.4× within one core class, so
a factor-of-two shortfall is outside anything placement explains.

### 4.4 Run conditions

Every run's log carries `run_conditions "at launch"` and `run_conditions "at exit"`
from `conformance/lib/run_conditions.sh` — loadavg, per-core clock, and derived core
class at BOTH ends — plus the four-line launch header (binary sha256, repo HEAD,
build exit status, tree-dirty state) with the inference lines labelled as inferences
(M-PROVENANCE-OVERREACH). A record without its conditions is not a record, and a
baseline without them is worse than none.

---

## 5. NULLS, MATCHED TO THE DATA'S GENERATIVE STRUCTURE

The data are a time series of INTEGER count triples over an autocorrelated
trajectory. An iid multinomial null is wrong twice over — it ignores the
autocorrelation and it ignores that a frame's triple is a whole-frame object.

* **N1 — persistence null (circular block bootstrap).** Resample the per-frame
  (f_S, f_L, f_V) triple in whole blocks of length **L_b = 2 τ_int** frames, τ_int
  the integrated autocorrelation time of f_S measured per run and PRINTED. **10,000
  resamples.** Blocks are whole frames, so every resampled frame is a real integer
  triple and the discreteness survives the null; 2 τ_int is the standard block length
  at which the resampled series carries the measured correlation rather than
  destroying it. The coexistence criterion is scored against this distribution, and
  τ_int is reported whether or not the point passes.
* **N2 — assignment null (the count-is-not-a-region null).** Permute phase labels
  across molecules WITHIN each frame. This preserves the per-frame count triple
  EXACTLY and destroys spatial contiguity. **A coexistence reading that survives N2
  is a statement about counts only, and is reported as one, never as coexistence.**
* **N3 — the contiguity statistic, and it is a criterion clause, not a diagnostic.**
  For each of S and L, the largest connected component under the R_nb edge set,
  divided by that phase's molecule count, must be **≥ 0.60**. A phase whose members
  are scattered in pieces none larger than 60% of the set is a dispersion, not a
  region; at the 0.15 floor this means the largest solid component holds ≥ 8.6
  molecules. **Vapor is EXEMPT from N3 by definition — a vapor is dispersed — and
  that exemption is stated here rather than left silent**, because an unstated
  exemption is how a criterion becomes a criterion for two different things.

---

## 6. THE PRE-COMMITTED DECISION TREE

Every outcome has its meaning written now. **A follow-up run is legitimate only if
it is a branch below; anything else is a rescue and is refused by this document.**

* **BRANCH (A) — THREE-PHASE REGION.** The criterion (§3.4, all five clauses) passes
  at ≥ 2 ADJACENT grid points. → The model has a triple-point region, located within
  the staked grid at the staked N. The report names the cell, the fractions, the
  contiguity numbers, the null p-values and the finite-size leg's result. This is the
  strongest reading available and it is still scoped to one N.
* **BRANCH (A′) — ONE POINT ONLY.** The criterion passes at exactly one point.
  → Reported as ONE POINT, unreplicated. Not called a triple point. Stage C refines
  around it under §7's frozen rule; if refinement produces no passing neighbour, the
  final reading is "one point, unreplicated", and it stays that way. A single point
  promoted to a region is M-MAX-OVER-SUCCESSES.
* **BRANCH (B) — TWO-PHASE ONLY.** Some points hold exactly two fractions above
  floor; none holds three. → The model HAS coexistence, and the two-phase boundaries
  found ARE the result: they locate this model's melting and boiling lines within the
  grid, which is a real measurement and is reported as the campaign's product. The
  triple-point question is **OPEN, not killed** — the locus is outside the staked
  grid or does not exist at this N. Extending the grid is a branch ONLY if the two
  two-phase boundaries are measured to be converging toward each other WITHIN the
  grid, in which case the extension direction is determined by that convergence and
  by nothing else. If they are not converging, extension is a rescue and is refused.
* **BRANCH (C) — NO STABLE SOLID.** f_S falls below 0.10 within R at every point of
  the grid. → **THIS WOUNDS THE WATER CLAIM, and the wound is reported at full
  volume.** The model that made one certified OH2 does not hold a crystal of them at
  any temperature or density in the staked grid; the programme's water is a molecule
  and not a substance, at this tier and this N. The measured quantity published
  beside the verdict is the longest surviving f_S ≥ 0.15 run over the whole grid, in
  fs, so the distance to the bar is visible rather than asserted. What the branch
  does NOT touch: the census's certification of the molecule (a different claim with
  a different instrument) and the four-body term's causal role. Separability is the
  point.
* **BRANCH (D) — MELTS EVERYWHERE, liquid and vapor alive.** A special case of (C)
  in which f_L and f_V pass their floors somewhere. → The model has a fluid and a gas
  and no solid in the grid. Wounds the crystal claim ONLY; the molecule claim and the
  liquid claim are untouched, and the report says which of the three it wounded.
* **BRANCH (E) — FREEZES EVERYWHERE.** f_S ≥ 0.85 at every point including T = 1600 K.
  → Before this is read as physics, §3.5 fires: if the thermostat mismatch exceeds
  0.20 at the hot points, the branch is **VOID (instrument)**, not a result, and the
  thermostat is the finding. If the thermostat IS delivering 1600 K and the seed
  still holds, the reading is that the model's cohesion is far stronger than its
  temperature scale suggests, which is a statement about the model worth publishing
  and worth attacking — the pre-committed attack is the vapor plant P-2, run at the
  same T, which must still read vapor.
* **BRANCH (F) — VOID.** Any of: budget exhausted (§4.2), f_U > 0.20, thermostat
  mismatch > 0.20, lens separation < 0.25 or misassignment > 0.05, informative
  transitions < 200, price shortfall (§4.3), or the Stage-A spread gate (§7). → The
  campaign did not measure what it set out to measure. No verdict about the triple
  point either way; the failing gate is named, and VOID is printed as loudly as a
  pass. **VOIDs are never scored and never averaged into anything.**

**Pre-committed follow-ups (a branch is design; a rescue is post-hoc):**

* If (C) or (D), the IDENTICAL instrument is run on the pure ice-XI seed at T = 50 K
  with no liquid and no gap — plant P-1's carrier. If the pure seed also fails to
  hold f_S, the instrument's bar is wrong and the reading is VOID rather than
  negative. This is the instrument's own control and it is committed here.
* If (A) or (A′), the finite-size leg runs at the passing cell only: the same point
  at **2N = 576 atoms**, one configuration. The leg does not confirm or refute the
  region; it MEASURES the N-dependence and publishes it beside the location
  (M-VOLUME-SCALE). A triple-point location quoted without it is quoted at one N and
  says so.
* If (B), no extension without the convergence condition above. Written now so it
  cannot be invented later.

---

## 7. BOUNDARY SPREAD FIRST — Stage A, and it can only VOID

*"The physics varies along this axis" does not imply "the boundary varies along it."*
`engine/Q10_PREREG.md` §9 records the campaign that VOIDed at its family gate because
nobody measured the spread first, and this design pays that cost up front.

> **Stage A: the same T ladder {50, 100, 200, 400, 800, 1600} K at f = 1.00 only, on
> TWO independent seed configurations** — different proton-disorder realisation in
> the ice-XI builder AND different velocity seed, so both the configurational and the
> kinetic sectors differ. **12 runs.**

Each configuration yields a bracketing pair: the adjacent T points between which f_S
crosses the 0.15 floor.

> **THE SPREAD GATE: if the two configurations' bracketing pairs do not overlap —
> i.e. the boundary's location differs by more than one full grid spacing (a factor
> of 2 in T) — Stage B is VOID-NOT-KILLED.** The grid cannot resolve the boundary at
> this spacing, no verdict about the triple point is issued, and the campaign reports
> the measured spread as its product.

**VOID, never KILL, and the distinction is load-bearing**: a boundary that varies
more than the grid spacing is a fact about our resolution, not about whether the
model has a triple point. Reading it as a kill would be scoring an instrument
failure as physics.

**Stage A cannot move Stage B's grid.** The bounds and spacing of §3.3 are frozen in
this file. Stage A decides only whether that grid is ADMISSIBLE. Widening or shifting
the grid on Stage A's result would be exactly the post-hoc reinterpretation this
document exists to prevent — the legitimate place for spread-derived structure is
Stage C, whose RULE (not whose numbers) is frozen here:

> **Stage C — refinement, rule frozen:** if Stage B yields (A) or (A′), refine inside
> the passing cell (and its immediate neighbours) on a **5 × 5** grid whose spacing is
> the bracketing cell's edge divided by **4**, in both axes. **≤ 25 points.** No other
> refinement is authorised, and no second refinement round exists.

---

## 8. THE GATES

- **G0 — the price, first act.** Before any grid point runs, one full R+W point at
  T = 400 K, f = 1.00 is run to measure CPU-seconds per point, pinned with `taskset`
  to an E-core, reported as a RANGE over ≥ 3 placements with n and spread on the row.
  The grid's total price is `67 × measured per-point cost` and must be ≤ B = 400
  core-hours, else the pre-committed cut applies (§8 note) or the campaign VOIDs.
  witness: `none (a price measurement has no theorem; M-CHEAPER-THAN-ITS-PRICE is its warrant)`
- **G1 — lens separation.** |median q(P-1) − median q(P-3)| ≥ 0.25 AND misassignment
  ≤ 0.05, both measured on the pure-phase references before any grid point.
  witness: `none (an empirical separation floor on an instrument; M-SORTS-NOT-SEPARATES is its warrant)`
- **G2 — boundary spread.** Stage A's two configurations' bracketing pairs overlap
  (spread ≤ 1 grid spacing, a factor of 2 in T), else VOID-not-killed.
  witness: `none (a resolution gate on a measurement; the Q10/Q7 family-gate precedent is its warrant)`
- **G3 — eligibility.** A molecule counts toward a fraction only as a certified census
  row at β = 0.02 with every breach run ≤ 8.4 fs. witness: `Held`
- **G4 — coexistence.** All five clauses of §3.4: means ≥ 0.15, minima ≥ 0.10, the
  one-window forward extrapolation ≥ 0.10, f_U ≤ 0.20, and G6.
  witness: `none (a conjunction of measured floors; M-CONJUNCTION-MONOTONE is its warrant and it is scored against N1)`
- **G5 — persistence null.** The criterion is scored against N1's 10,000-resample
  block bootstrap at block length 2 τ_int, with τ_int printed.
  witness: `none (a resampling null; M-NULL-MISSTAKE is its warrant)`
- **G6 — contiguity.** Largest connected component ≥ 0.60 of the phase's count, for
  S and for L; vapor exempt by definition and the exemption printed.
  witness: `none (a spatial-separation floor; M-SORTS-NOT-SEPARATES is its warrant)`
- **G7 — thermostat delivery.** |mean(T) − T_set| / T_set ≤ 0.20 over W, else VOID.
  witness: `none (a control-delivery gate on a measurement; M-VACUOUS-SUCCESS is its warrant)`
- **G8 — moving carrier.** The census's anti-vacuity clauses, inherited unchanged:
  internal RMS displacement ≥ 0.1 bohr and ≥ 1 intra-block separation varying by
  ≥ 0.05 bohr, else VOID (frozen carrier).
  witness: `none (an anti-vacuity gate; M-FIXED-POINT-TRAJECTORY is its warrant)`
- **G9 — work count.** ≥ 200 informative transitions in the closure leg over the
  analysed span, or the closure reading is VOID. The count is printed pass or fail.
  witness: `closed_iff_fiber_invariant`
- **G10 — closure defect non-expansion.** W is split into 4 equal quarters; the
  witness-pair rate D on quarter 4 must be ≤ **1.10 ×** D on quarter 1, and all four
  quarters are printed individually so a monotone climb is visible inside the budget.
  witness: `horizonBudget_le_of_nonexpansive`
- **G11 — defect exhibition.** Every witness pair is exhibited by frame index. D = 0
  is reported as "no witness pair found at this resolution", never as closure.
  witness: `nonfactoring_iff_not_closed`
- **G12 — price closure.** No grid point is banked whose measured CPU-seconds fall
  below 0.5 × G0's per-point price. witness: `none (a price-closure gate; M-CHEAPER-THAN-ITS-PRICE is its warrant)`
- **G13 — budget.** Cumulative CPU-seconds ≤ B = 400 core-hours across all stages,
  with the per-stage splits 4 / 72 / 180 / 144. Exhaustion VOIDs the affected points
  loudly and the VOID map over the grid is published.
  witness: `none (a budget gate; M-BUDGET-LAUNDER is its warrant)`
- **G14 — exit reason.** Every run records WHY it stopped as a read field (completed
  W · budget · thermostat VOID · price refusal · crash), shipped as a histogram over
  the 30+ points; no `converged: true` boolean anywhere.
  witness: `none (a record-completeness gate; M-EXIT-DISCRIMINATOR is its warrant)`
- **G15 — finite size.** If (A) or (A′), the passing cell is re-run at 2N = 576 atoms
  and the N-dependence of every fraction is published beside the location.
  witness: `none (an N-scaling leg; M-VOLUME-SCALE is its warrant)`

**Pre-committed cut, if G0's price puts the grid over B:** drop Stage C entirely
(−144 core-hours), then halve Stage A to one T ladder point per decade (−36). **A
BOUND IS NEVER MOVED AND A FLOOR IS NEVER LOWERED TO FIT A BUDGET.** If the campaign
still does not fit, it VOIDs before running, which is a cheaper and more honest
outcome than a grid whose bounds were chosen by its price.

---

## 9. SEPARABLE KILLS — each falsifier takes down its own claim and nothing beneath it

| claim | its falsifier | what else it takes down |
|---|---|---|
| **K1** the model has a solid phase somewhere in the staked grid | branch (C) or (D): f_S < 0.10 within R at every point | nothing — K2, K3 and the census's molecule certification stand |
| **K2** the model has a liquid phase somewhere in the staked grid | f_L never reaches 0.15 at any point | nothing — K1 and K3 stand |
| **K3** three-phase coexistence exists in the staked window at the staked N | branch (B): no point holds all three floors | nothing — K1 and K2 are read off the same runs and survive independently |
| **K4** the census lenses separate THIS model's phases | G1: separation < 0.25 or misassignment > 0.05 | VOIDs K3 only; K1 and K2 fall back to the raw printed quantities (median q, neighbour count), which are published regardless |
| **K5** the boundary is resolvable at the staked spacing | G2: Stage A spread > one grid spacing | **VOID, never KILL** — takes down no claim at all; it retires the grid |
| **K6** the coexistence reading is a REGION statement, not a count statement | N2 survives, or G6 fails | downgrades K3 to a count claim, published as one; K1, K2, K4, K5 untouched |
| **K7** the triple-point LOCATION is an N-independent property | G15: fractions move materially at 2N | scopes K3's location to one N; does not kill the existence reading |

No kill implies another. All fired kills are reported in the results document's title
line, as loudly as any survival.

---

## 10. PLANTS

Every plant names its carrier and **the sector the plant acts on**, and each is
pre-checked to FIRE on THIS instrument before it is trusted (M-PLANT-OBS —
observability is instrument-relative and a plant re-used from another instrument is
not a plant here). Plants run in Stage 0, before any grid point.

* **P-1 — must CLASSIFY SOLID.** Carrier: the pure ice-XI seed at T = 50 K, no
  liquid region, no gap, at f = 1.00. The sector the plant acts on is the ORDER
  sector, and the carrier must be nonzero in it: q-tetrahedral at the ice reference,
  far from the ideal-gas floor. Expected: f_S ≥ 0.90. **An instrument that cannot see
  a perfect crystal cannot see a coexisting one**, and this plant also supplies one
  of the two medians that define `q_s`.
* **P-2 — must CLASSIFY VAPOR.** Carrier: the same 96 molecules at f = 4.00 (1/64 the
  reference density) and T = 800 K. The sector the plant acts on is the DENSITY
  sector, nonzero there (neighbour counts ≈ 0) while the COMPOSITION sector is
  unchanged — the molecules are the same molecules. Expected: f_V ≥ 0.90. Note f =
  4.00 lies OUTSIDE the staked grid on purpose: a plant must be unambiguous, and the
  grid's own upper bound is where ambiguity begins.
* **P-3 — must CLASSIFY LIQUID.** Carrier: the reference-density box equilibrated at
  T = 400 K with NO ice seed. The sector the plant acts on is the ORDER sector, and
  the carrier must be nonzero there but intermediate — neither crystal nor gas.
  Expected: f_L ≥ 0.80 and f_S ≤ 0.10. Supplies the second median defining `q_s`.
* **P-4 — must REFUSE (2D).** Carrier: a `Dims::Two` scene. The sector the plant acts
  on is the DIMENSION sector, nonzero there by construction. Expected: the
  q-tetrahedral lens REFUSES by name with the gate `dims == 3`, so no phase diagram
  can be reported from a 2D scene. The refusal is the finding, not an inconvenience.
* **P-5 — must VOID (frozen carrier).** Carrier: the reference lattice with all
  velocities exactly zero. The sector the plant acts on is the MOTION sector, in
  which the carrier is ZERO, while the membership sector reads perfect. Expected:
  VOID via G8, not a spectacular f_S = 1.00.
* **P-6 — must FAIL CONTIGUITY (the count-is-not-a-region plant).** Carrier: a
  synthetic frame set carrying EXACTLY a passing count triple (f_S = f_L = f_V = 1/3)
  with the three labels distributed spatially at random through the cell. The sector
  the plant acts on is the SPATIAL sector, nonzero there (contiguity destroyed) while
  the COUNT sector is exactly unchanged. Expected: §3.4 clauses (i)–(iv) PASS and
  G6 FAILS. **This plant exists solely to prove the count criterion is not the whole
  verdict**, and it is the direct analogue of the census's C-3 budget-abuse plant.
* **P-7 — must VOID (budget).** Carrier: one grid point launched with its budget set
  below G0's measured per-point price. The sector the plant acts on is the BUDGET
  sector, nonzero there while the physics sector is untouched. Expected: a loud VOID
  that appears in the published VOID map and scores nothing.
* **P-8 — must not read its tag (blindness).** Carrier: a fully liquid trajectory
  whose builder region tags say "solid" for every molecule. The sector the plant acts
  on is the TAG sector, nonzero there (the tags are wrong) while being exactly zero in
  the coordinate sector (the positions are an unmodified liquid's). Expected: LIQUID.
  A verdict of SOLID convicts the classifier of reading its construction metadata
  (M-TAG-AS-PROPERTY).

---

## 11. THE STAKED NUMBERS — this freeze's own honesty table

| # | number | justification, one line |
|---|---|---|
| 1 | W = 8340 fs | ten census windows; 4.6 traversals of the cell's 26.9 Å z extent at an order-of-magnitude ~1500 m/s, the shortest time in which a REGION can be said to persist |
| 2 | R = 4170 fs, discarded | W/2 = 2.3 traversals — the scale move's compression wave crosses and returns twice before measurement opens |
| 3 | T ∈ {50,100,200,400,800,1600} K | geometric ratio 2 because this model's melting boundary is unlocated in this repository; a linear grid around a guessed centre would be M-UNTESTED-GAP |
| 4 | T lower bound 50 K | kT = 1.58e-4 Ha is >30× under a hydrogenic stretch ZPE on this surface (C1 measured the H–H curve's; the O–H ZPE is OWED), so a classical solid below it is a tier artifact, not a phase |
| 5 | T upper bound 1600 K | kT = 5.07e-3 Ha is 24× below the model's measured D_e(O–H) = 0.122901 Ha but far above every many-body residual — vapor must be unambiguous here |
| 6 | f ∈ {0.85,1.00,1.20,1.50,2.00} | f = 1.00 is the seed's own lattice; spacing is ~1.2× per step in log, the same bracketing argument on an equally unlocated axis |
| 7 | gap = 14.2 bohr at f = 1.00 | on a 24 × 24 bohr cross-section with 96 molecules at the ice-XI molecular volume, giving a 26.9 Å z extent — the geometry every window justification is measured against |
| 8 | f lower bound 0.85 | 0.85 × 14.2 = 12.07 bohr, the last point clearing 2·R_CUT = 12 bohr; below it the two liquid surfaces see each other and no vapor region is left to measure |
| 9 | f upper bound 2.00 | mean density 1/8 of reference; past it "all vapor" is a fact about our geometry, not about the model |
| 10 | N = 288 atoms (96 H₂O) | sets the 0.15 floor at 14.4 molecules (the smallest region with an interior); at 48 molecules the floor would be a cluster |
| 11 | slab ≥ 3 molecular layers | a two-layer slab is entirely surface; its solid fraction measures the interface |
| 12 | coexistence floor 0.15 | 14.4 molecules at N = 96 — below it a "phase" is an interface |
| 13 | window-minimum floor 0.10 | 9.6 molecules — the point at which a region is a cluster |
| 14 | forward-extrapolation clause, one W | stops "has not collapsed yet" from scoring as "coexists"; a monotone decay caught mid-fall is caught |
| 15 | f_U ≤ 0.20 | above it the unassigned pool competes with the three 0.15 floors for the same molecules |
| 16 | lens separation ≥ 0.25 | a quarter of the lens's own gated range (exactly 1 on a perfect tetrahedron, 0 on an ideal gas) |
| 17 | misassignment ≤ 0.05 | one third of the 0.15 floor: an instrument's error must be strictly smaller than the effect it measures |
| 18 | R_nb = 6.6140 bohr | the Luzar–Chandler O···O cutoff already staked in the census lens stack — reused so both instruments read one ruler |
| 19 | vapor threshold = ZERO O-neighbours | the weakest possible criterion, so f_V is a LOWER bound and cannot be inflated by threshold choice |
| 20 | contiguity ≥ 0.60 | a phase in pieces none larger than 60% of its members is a dispersion; at the floor this is ≥ 8.6 molecules in one component |
| 21 | vapor exempt from contiguity | a vapor is dispersed by definition; stated rather than left silent so the criterion means one thing |
| 22 | β = 0.02, L_flick = 8.4 fs | inherited unchanged from CENSUS_PREREG so census and campaign read ONE instrument |
| 23 | informative transitions ≥ 200 | inherited unchanged; below it the closure leg is vacuous by construction |
| 24 | defect non-expansion 1.10 | the census's 1.05 loosened for a 10× longer window on a genuinely moving interface; the sign of the law (defects must not grow) is untouched and all four quarters print |
| 25 | thermostat mismatch ≤ 0.20 on the MEAN | the banked P2 log shows instantaneous T at 242–409 K around a 300 K setpoint, so a per-frame bar would fire on a working thermostat |
| 26 | Stage A: 2 seed configurations, 12 runs | proton-disorder AND velocity seed both differ, so configurational and kinetic sectors are both varied |
| 27 | spread gate = 1 grid spacing (factor 2 in T) | a boundary varying more than the grid spacing is a resolution fact, so it VOIDs and never kills |
| 28 | Stage B = 30 points (6 × 5) | the frozen envelope; Stage A can only VOID it, never move it |
| 29 | Stage C = 25 points (5 × 5), spacing = cell edge / 4 | the only authorised refinement, rule frozen here and numbers derived from the bracketing cell; no second round exists |
| 30 | B = 400 core-hours CPU | 1.7× the §4.1 prior — knowingly insufficient against the measured 3.5× contention penalty, so the campaign VOIDs rather than overspends |
| 31 | budget split 4 / 72 / 180 / 144 | G0 / Stage A / Stage B / Stage C, summing to B |
| 32 | price shortfall refusal 0.5× | measured placement spread within one core class reaches 1.4×, so a factor-of-two shortfall is outside anything placement explains |
| 33 | N1: 10,000 resamples, block 2 τ_int | whole-frame blocks keep the counts integral; 2 τ_int keeps the measured correlation instead of destroying it |
| 34 | finite-size leg at 2N = 576 atoms | one doubling at the passing cell only — enough to MEASURE N-dependence, not enough to claim independence of it |
| 35 | 67 runs total (12 + 30 + 25) | the sizing multiplicand; the multiplier is G0's measured per-point cost and nothing else |
| 36 | ≈ 235 core-hours | **NOT A STAKE — a labelled PRIOR** extrapolated from wall clock on a loaded box, printed so the budget has something to be justified against |

---

## 12. WHAT THIS CAMPAIGN DOES NOT CLAIM

* It does not claim the model's triple point is near water's. It does not compare to
  273.16 K or 611.657 Pa as a score. Real-water numbers may sit beside the model's in
  a results document as literature anchors and never as a reference
  (M-FOREIGN-DOMAIN-CORROBORATION).
* It does not claim a phase diagram. It claims — or fails to claim — the survival of
  three seeded regions over one staked window on one staked grid at one staked N.
* It does not claim thermodynamic phases. Closure certification of a phase as a
  Closed view of the fine dynamics is `OBSERVATORY_BRIEF.md` §5's admission half and
  is a separate instrument; this campaign is the discovery half.
* It does not claim closure. The closure leg can exhibit witness pairs or fail to
  find them, and the second is reported as a failure to refute at the resolution
  sampled, exactly as `lean/CIRISHolon/Object.lean`'s
  `nonfactoring_iff_not_closed` requires.
* It does not claim the boundary is sharp. Stage A measures its spread first, and a
  spread larger than the grid spacing retires the grid rather than the question.
* It does not claim N-independence of anything. The finite-size leg measures the
  N-dependence and publishes it; a location quoted without it is quoted at one N and
  says so.
