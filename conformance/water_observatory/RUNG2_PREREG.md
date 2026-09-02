# RUNG 2 — THE FLUID-ELEMENT TIER: prereg

*Frozen 2026-09-02, before any line of the rung-2 instrument existed and before any chart
reading was computed. This document stakes the chart, the window, the budget, the controls,
the plants, the cost model and the meaning of every possible answer. The git history is the
check: the instrument's first commit must be strictly later than this file's.*

**misfits:** M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-FINAL-VIEW-COLLISIONS,
M-ONE-MODEL-DELTA, M-NULL-MISSTAKE, M-UNTESTED-GAP, M-CONJUNCTION-MONOTONE,
M-BASE-RATE-OMITTED, M-BUDGET-LAUNDER, M-EXIT-DISCRIMINATOR, M-MAX-OVER-SUCCESSES,
M-SORTS-NOT-SEPARATES, M-TAG-AS-PROPERTY, M-PLANT-OBS, M-PLANT-SECTOR, M-POPULATION-CHOICE,
M-PRESENTATION-VERDICT, M-NONBIJECTIVE-STEP, M-HOMOG, M-VOLUME-SCALE, M-STALE-INSTRUMENT,
M-PROVENANCE-OVERREACH, M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS,
M-IDLE-CALIBRATED-TIMEOUT, M-CACHE-KIND, M-IMPORT-EXECUTES, M-MAINTENANCE-LENS,
M-FOREIGN-DOMAIN-CORROBORATION, M-PROBE-THE-RESOURCE, M-LOOP-BLIND, M-PARITY-PROTECT,
M-COND-PROBE.

Pin: this freeze is authored against worktree commit `a4da7cc` (branch `rung2-continuum`).

---

## 0. WHAT THIS RUNG IS, AND THE ONE THING IT CANNOT BE

GANTT node G asks for "upward closures: network and fluid tiers certified as Closed views,
census as referee at each rung." Rung 2 is the FLUID tier. `WORKBENCH_FSD.md` §9c places
that band at **~1 µm and up** — "fluid element (~µm+)", the continuum chart of T6 fields.

**The band is out of reach of this carrier, and of any atomistic carrier, and this freeze
says so before it measures anything.** The certified molecular arm is the frozen
`waterquench` protocol: **12 atoms, two-dimensional, box 34.6 × 20.8 bohr**. In nanometres
that is 1.8310 × 1.1007 nm, an area of 2.0154 nm², a number density of **5.954 atoms/nm²**.
A 1 µm × 1 µm patch at that density is **5.95e6 atoms** — 4.96e5 times the certified scene,
546 times its linear size. The trajectory ARTIFACT independently caps at 16 atoms
(`holon-lens/src/traj.rs`, `MAX_DUMP_ATOMS`, a `u128` bond bitset), so the µm band is not
reachable even in principle by the format this rung reads.

So this freeze does **not** stake that the µm band closes. It stakes three separable
things, and G2 below is what keeps them apart:

1. **ADMISSIBILITY** — whether the certified carrier can support a fluid-element chart at
   all, measured against a bar written down here, and expected to FAIL.
2. **THE CHART FORM** — whether the continuum chart (density, momentum, energy over cells)
   is a Closed view of the certified molecular dynamics at the scales this carrier *does*
   reach, which are nanometres, with a control that can void the reading.
3. **THE EXIT** — what a carrier would have to be for rung 2's band to become measurable,
   stated as a measured requirement rather than a guess, so the fence the site wears has a
   number in it.

**What this rung does NOT claim, staked here so no later sentence can drift into it:** it
does not claim a µm fluid element is Closed; it does not claim the nanometre reading
extrapolates to µm (M-UNTESTED-GAP forbids exactly that move — the hypothesised axis has
no measured point within five orders of the staked one); and it does not claim the
continuum chart is the *only* coarse view of this dynamics.

---

## 1. THE VACUITY TRAP, NAMED FIRST BECAUSE IT IS THE WHOLE DANGER

`lean/CIRISHolon/Tiers.lean` proves `exists_closed_view (T) : Closed T T` — **every step
closes some view, namely itself.** Its docstring states the consequence: *"'Is a tier' is
empty as a unary predicate; the engineering content is always the RELATION (v, T,
budget)."*

The continuum chart walks straight into that theorem if it is not fenced. Take the chart
with **one cell**: its fields are total mass, total momentum, total energy — and a chart
whose readings are the motion's own invariants is Closed with `h = id`, exactly, for free,
forever. A rung-2 report showing a clean certificate at one cell would be reporting
`exists_closed_view`, not a measurement of water.

Therefore: **the one-cell chart is run in every arm as a positive control that MUST be
declared VOID** (G3). A run in which it certifies convicts the instrument.

The same trap has a second door, and `M-CONJUNCTION-MONOTONE` is its name. Refining a
chart can only remove collisions — `ClosureLadder.lean::refinement_removes_collisions`
proves it for *any* views whatsoever. So "the finer chart has fewer firing collisions" is
a theorem, not evidence. G8 uses that monotonicity **only as an instrument self-check**,
never as support, and §4's forward prediction is staked on the defect RATIO, which the
theorem does not constrain in either direction.

---

## 2. THE VIEW, THE MOTION, AND THE CHART

From `lean/CIRISHolon/Object.lean` and `lean/CIRISHolon/ClosureLadder.lean`:

```
Closed v T        ≔  ∃ h, v ∘ T = h ∘ v                 witness: `Closed`
Closed v T        ↔  ∀ x y, v x = v y → v (T x) = v (T y)
                                                        witness: `closed_iff_fiber_invariant`
Collision x v i j ≔  v (x i) = v (x j)                  witness: `Collision`
Firing x v i j    ≔  Collision ∧ v (x (i+1)) ≠ v (x (j+1))
                                                        witness: `Firing`
ClosedOn x v      ≔  ∀ i j, Collision x v i j → v (x (i+1)) = v (x (j+1))
                                                        witness: `ClosedOn`
```

`ClosureLadder.lean`'s definitions are already trajectory-level — *"a trajectory is a
function `x : ℕ → S`, a view `v : S → V`, a collision a pair of times with equal views,
FIRING when the successors' views differ"* — which is precisely this instrument. Nothing
new is mechanized for this rung; the existing bricks are the scaffolding and this campaign
supplies the measurement.

* **X** — the micro-state: positions and velocities of all 12 atoms, plus species.
* **T** — ONE GRAIN BOUNDARY, `Sim::step_frame(64)`. **The same T the census used**, not a
  second motion invented here. That is what keeps rung 1 and rung 2 certificates composable
  under `Tiers.lean::closed_comp`: two independent charts over one motion.
* **the trajectory** — the frames of the banked `.traj` dumps, in file order.

### 2.1 The cells

An `n_x × n_y` Eulerian grid over the box, cells of equal size, fixed in space. The
protocol's boundary is `Boundary::Walls` (`waterquench_traj.rs:507`), so atoms are confined
and the grid tiles the reachable region exactly; an atom outside the box on any frame is a
REFUSAL (§6, R1), never a clamp.

Cell of an atom = `(floor(x / (BOX_W/n_x)), floor(y / (BOX_H/n_y)))`. Arena indices
throughout, never sorted or spatial indices — the same Object rule 6 the census obeys.

### 2.2 The three fields, and one refusal

Per cell `c`:

| field | definition | why it is here |
|---|---|---|
| **occupancy** `n_c^Z` | count of atoms of nuclear charge `Z` in `c` | the density field; EXACTLY discrete, so it needs no binning and carries no quantisation choice |
| **momentum** `p_c` | `Σ_{i∈c} m_i v_i`, two components | the momentum field |
| **kinetic energy** `e_c` | `Σ_{i∈c} ½ m_i |v_i|²` | the energy field, KINETIC PART ONLY |

**The refused field, named rather than fudged (M-MAINTENANCE-LENS).** The potential energy
is not cell-local: a pair straddling a face has no share of its interaction energy that the
dynamics forces onto one side. A half-and-half split is a convention available in the
literature, and adopting one silently would put a free parameter inside the chart. This
rung therefore reads the KINETIC field and says so in every sentence that mentions the
energy chart. The consequence is stated with it: the energy field is **not** the conserved
energy, so G9c tests the kinetic field's closure and nothing more.

### 2.3 Quantisation — derived, never fitted

A chart is a finite-resolution reading. Occupancy is already integral. The two continuous
fields are binned at scales **derived from protocol constants only**, so that no number in
the chart is chosen after seeing data:

> **Δp = √(m_H · k_B · T_target) = 1.3211** (atomic units), the thermal momentum of a
> hydrogen atom at the thermostat's own target. `m_H = 1.00782503207 u × 1822.888486 m_e/u
> = 1837.1527 m_e`; `k_B = 3.166811563e-6 Ha/K`; `T_target = 300 K`.
>
> **Δe = k_B · T_target = 9.500435e-4 Ha**, one thermal quantum.

Bin index is `floor(value / Δ)`, ties to `-∞`, symmetric about zero for the signed momentum
components. The instrument must ASSERT both constants against the formula at start-up and
refuse if they drift (M-STALE-INSTRUMENT).

### 2.4 The chart ladder

Three rungs, each a refinement of the last, so `refinement_removes_collisions` applies:

| chart | fields |
|---|---|
| **v1** | occupancy only |
| **v2** | occupancy + binned momentum |
| **v3** | occupancy + binned momentum + binned kinetic energy |

Every rung's OWN collisions are enumerated separately. `M-FINAL-VIEW-COLLISIONS` is the
misfit that fires if a report claims "the refined chart restores closure" from the coarse
chart's separated pairs alone; it is the reason each of v1, v2, v3 carries its own work
count and its own D, and no rung's verdict is inferred from another's.

### 2.5 The cell grids tested

`(n_x, n_y) ∈ {(1,1), (2,1), (2,2), (4,2), (6,4)}`, giving mean occupancies
12, 6, 3, 1.5 and 0.5 atoms per cell. `(1,1)` is the vacuity control of G3. The list is
frozen here so the grid is not a free parameter chosen to flatter a curve
(M-POPULATION-CHOICE).

---

## 3. THE STAKES AND THE GATES

Every gate names its witness. `none` is permitted only with a reason on the same line.

### 3.1 The trajectory set and its digests

- **G1 — carrier identity.** Every `.traj` file read must match its digest in
  `conformance/water_observatory/census_traj_manifest.sha256` EXACTLY, all 64 bytes of hex.
  Arms: `fenced` (8 seeds — the arm carrying the census's certified-strict OH₂) and
  `hydrogen` (8 seeds — the control arm). The digest is a provenance line and it names what
  it measured and nothing beside it (M-PROVENANCE-OVERREACH): it pins the FILE, not the
  physics that produced it, and the physics is cited to `CENSUS_RESULTS.md` rather than
  re-derived here.
  witness: `none (a digest gate has no theorem; it is a byte comparison)`

### 3.2 Admissibility — the µm bar, staked to be able to fail

A "fluid element" is a cell whose continuum fields mean something. That requires the cell's
particle number to be large enough that relative density fluctuation is small, and it
requires more than one cell.

- **G2 — fluid-element admissibility.** ALL THREE of: mean occupancy ≥ 100 atoms per cell;
  cell count ≥ 4; relative density fluctuation `σ(n_c)/⟨n_c⟩ ≤ 0.10` measured over frames
  and cells. The 100 follows from the 0.10 without a second choice: Poisson fluctuation
  `1/√N ≤ 0.10` is `N ≥ 100`.
  **Expected on this carrier: FAIL**, by construction and not by surprise — the maximum
  mean occupancy at ≥ 2 cells is 6. The reading is computed and printed anyway, for every
  grid, because a bar quoted without its measured value is not a bar.
  On FAIL the µm verdict is **VOID (inadmissible carrier)** and branch (d) of §5 runs.
  witness: `none (an admissibility precondition on a measurement; M-VACUOUS-SUCCESS and M-VOLUME-SCALE are its warrants)`

### 3.3 Anti-vacuity

- **G3 — the vacuity fence.** A chart reading is admitted only if BOTH: the grid has ≥ 2
  cells, AND ≥ 5% of grain boundaries carry at least one atom crossing a cell face. A chart
  failing either is **VOID (frozen or trivial chart)** — not passed, not failed. The `(1,1)`
  grid MUST land here in every arm; if it certifies instead, the run is refused and the
  instrument is convicted. This is `M-FIXED-POINT-TRAJECTORY` in the field chart's clothes:
  a closure gate is vacuous on a carrier the motion does not move.
  witness: `exists_closed_view`
- **G4 — work count.** ≥ 200 informative transitions per (arm, grid, chart rung), where an
  informative transition departs from a chart reading visited at least twice in the analysed
  span. Below 200 the reading is **VOID**, and VOID is printed as loudly as a pass. The count
  is printed whether or not it passes. Carried over unchanged from `CENSUS_PREREG.md` G6
  rather than re-invented, so the two rungs share one bar.
  witness: `closed_iff_fiber_invariant`

### 3.4 The two legs

- **G5 — LEG A, the collision form.** `D_A = (firing collisions) / (all collisions)` per
  (arm, grid, chart rung), with every firing collision exhibited by frame index pair.
  **CERTIFIED-STRICT** iff `D_A = 0` EXACT with G3 and G4 met; **CERTIFIED-BUDGETED** iff
  `D_A ≤ 0.02` — the census's own β, carried over unchanged. `D_A = 0` is reported as *"no
  firing collision found at this resolution"* and NEVER as "Closed"; absence of a witness
  pair on a sampled trajectory is a failure to refute, exactly as `CENSUS_PREREG.md` §1
  requires. The collision form is chosen deliberately over a fitted-model residual:
  `M-ONE-MODEL-DELTA` says a defect against one chosen model earns only "worse than that
  model", while the collision form earns "best memoryless" — which is the claim this rung
  needs, because `h` is quantified existentially in `Closed`.
  witness: `Firing`
- **G6 — LEG B, held out.** The empirical coarse law `h` is built on the first half of each
  trajectory (for each chart reading, its modal successor) and applied to the second half;
  `D_B` is the mismatch rate over second-half frames whose reading was seen in the first
  half, with COVERAGE (the fraction of second-half frames so seen) printed beside it. Same
  bars as G5. This leg exists because a low `D_A` can be produced by a chart whose fibers
  are visited only in one short stretch; generalisation is what that cannot fake. It is a
  ONE-MODEL delta by construction and is reported with that scope, never as the headline.
  **Deviation from the census form, named rather than silent:** the census's Leg A was HELD
  (a block persists). A field chart has no persistent block — the fields evolve — so rung 2's
  second leg tests generalisation instead of persistence. The two-leg *shape* is kept; the
  second leg's question is different and this sentence is the disclosure.
  witness: `rate_unique_on_range`

### 3.5 The control floor — the discriminator

- **G7 — the position-blind chart.** For every certified or near-certified reading, the
  identical instrument is run on a chart whose cell membership is assigned by a FIXED
  permutation of ARENA INDEX rather than by position — same cell count, same field arity,
  same occupancy statistics, no spatial meaning whatsoever. Required separation:
  `D_A(blind) − D_A(spatial) ≥ 0.05` absolute. Below that the reading is **VOID (no
  separation)**: an instrument that scores a position-blind chart as well as a spatial one
  is not measuring continuum structure, whatever number it prints. The blind reading is
  printed for every grid, pass or fail — this is the eligible-pool rate `M-BASE-RATE-OMITTED`
  demands, not an afterthought. **Selection is on the difference, never on `D_A` alone.**
  Before reading the 0.05 as separation, whether ANY grid clears it is checked and reported
  (M-SORTS-NOT-SEPARATES): a bar that ranks rather than separates is reported as ranking.
  witness: `none (an empirical separation floor; M-BASE-RATE-OMITTED is its warrant)`

### 3.6 The instrument's self-checks

- **G8 — ladder monotonicity, EXACT.** Collision counts must be non-increasing from v1 to
  v2 to v3 on every (arm, grid). This is `refinement_removes_collisions`, a theorem, so a
  violation convicts the CODE and never the physics; the run is refused, not reported.
  **It is a self-check and is never quoted as support** — `M-CONJUNCTION-MONOTONE` is
  precisely the error of reading a for-any-predicate monotonicity as a finding.
  witness: `refinement_removes_collisions`
- **G9 — conservation, ONE GATE PER LAW, and the null staked on what the law constrains.**
  `M-NULL-MISSTAKE` is the governing misfit and it bites hard here, because on THIS protocol
  two of the three fields are not conserved and staking a conservation null on them would be
  staking it on a quantity the law never constrained:
  - **G9a — species count.** `Σ_c n_c^Z` constant across every frame, EXACT, per species.
    This one IS constrained: nothing creates or destroys nuclei. A violation is a REFUSAL.
    witness: `closed_view_inherits_conservation`
  - **G9b — momentum.** `Boundary::Walls` delivers impulse, so total momentum is NOT
    conserved and no null is staked on its constancy. What is staked instead is the LEDGER:
    the frame-to-frame change in `Σ_c p_c` must be accounted by the wall term to within
    1e-6 relative, and the unaccounted residual is reported as a number.
    witness: `none (a ledger-closure reading, not a conservation law; M-NULL-MISSTAKE is its warrant)`
  - **G9c — energy.** The Berendsen thermostat runs for all 20000 frames
    (`thermostat_on = true`, `T_target = 300 K`, `τ = 2000` a.u.), so energy is removed
    continuously and is NOT conserved. No null is staked on its constancy; the kinetic
    field's closure is tested by G5/G6 like any other field, and the thermostat's draw is
    reported from the engine's own intervention ledger.
    witness: `none (a ledger-closure reading, not a conservation law; M-NULL-MISSTAKE is its warrant)`

  **This is a finding of the freeze, not a nuisance:** the brief's argument that the
  conserved quantities are the natural chart *because* `conserved_descends` forces them to
  descend does NOT hold on this carrier, because on this carrier they are not conserved.
  The chart is still well defined — the fields are readings whatever the dynamics does with
  them — but its warrant is different and weaker, and that is recorded here before any
  measurement rather than discovered in a postmortem.
- **G10 — the step is not asserted bijective.** `M-NONBIJECTIVE-STEP` requires any map
  called dynamics to be verified bijective. The thermostat's velocity rescaling is not, and
  this rung does not claim otherwise: `T` is a trajectory-generating map, closure is tested
  in the trajectory-level `ClosedOn` form which never needs bijectivity, and no statement in
  the results may call `T` unitary or reversible.
  witness: `ClosedOn`

### 3.7 Cost

- **G11 — work-unit pricing.** The instrument's price is quoted in FRAMES READ and CHART
  EVALUATIONS, never in wall clock: this box is heterogeneous (P-cores 0–15, E-cores 16–31)
  and carries a load average above 60 throughout, so a wall-clock baseline is confounded by
  placement (M-PLACEMENT-LOTTERY) and a wall-clock budget would be uncalibrated for the
  regime it runs in (M-IDLE-CALIBRATED-TIMEOUT). The banked cost model is: 16 trajectories
  × 20000 frames = 320,000 frames read; 5 grids × 3 chart rungs × 2 chart kinds
  (spatial, blind) = 30 chart evaluations per frame; 9.6e6 chart evaluations total. A run
  finishing at a small fraction of its own model is not that run (M-CHEAPER-THAN-ITS-PRICE)
  and is refused. Budget exhaustion VOIDs loudly and may never fall back to a scorable
  verdict (M-BUDGET-LAUNDER). The instrument is single-threaded integer/float work on one
  core class; no accelerator and no bitwise-variant arithmetic route is used, so no
  device-class dependence is claimed or needed (M-DEVICE-CLASS).
  witness: `none (a pricing gate has no theorem; it is a counted budget)`

---

## 4. THE FORWARD PREDICTION — one, and deliberately qualitative

Rule 6 support comes only from confirmed advance predictions, so one is staked here,
before the instrument exists.

> **F1.** On the real (spatial) chart, the Leg-A defect RATIO `D_A` decreases monotonically
> up the chart ladder — `D_A(v1) ≥ D_A(v2) ≥ D_A(v3)` — on a majority of admitted (arm,
> grid) cells; and on the position-blind chart it does NOT (no majority-monotone decrease).
>
> Why this is a real prediction and not the theorem: `refinement_removes_collisions`
> constrains the collision COUNT (numerator and denominator both fall) and says nothing
> whatever about the RATIO, which may rise, fall or stay flat. F1 says the momentum and
> energy fields carry the information that decides where matter goes next — the physical
> content of a continuum chart — and that a chart with no spatial meaning has no such
> information to add.
>
> Firing F1 does not kill rung 2; it kills the reading that the continuum fields are the
> right refinement, which is a separable claim and is the one F1 owns.

**No numeric band is staked forward, and that is a decision, not an omission.** Two
campaigns in this programme died staking that a boundary varies along an axis without first
measuring the spread on two configurations. The spread of `D_A` across seeds is unmeasured
at freeze time, so a numeric band here would be a blind stake. The results document reports
the across-seed spread as its first table, and any numeric band for a successor campaign is
gauged from it — after this campaign, never inside it.

**And the extrapolation is refused in advance.** `M-UNTESTED-GAP`: before staking a
prediction, plot the existing points on the hypothesised axis and check the hypothesis
predicts anything at the staked point. On the occupancy axis, this carrier's points are
{0.5, 1.5, 3, 6} atoms per cell and the µm band's point is 5.95e6. Five orders separate the
data from the target. **No fit over the accessible range may be reported as evidence about
the band**, and §5's exit is written as a REQUIREMENT ("a carrier would need N") rather than
as a prediction ("the defect at the band will be D").

---

## 5. THE BRANCHES — every answer's meaning, staked in advance

* **BRANCH (a) — TIER CERTIFIED.** G2 passes AND v3 reaches CERTIFIED-STRICT with G3, G4,
  G7 met. → The fluid-element tier is a Closed view of the certified molecular dynamics.
  `Tiers.lean::closed_comp` composes it with rung 1 and the band's face may go live on a
  banked, citation-gated certificate. *Not expected: G2 cannot pass on this carrier.*
* **BRANCH (b) — TIER CERTIFIED AT BUDGET.** As (a) but `D_A ≤ 0.02` rather than exact. →
  Certified at budget, and the budget is named in the same sentence as the claim, every
  time. Weaker than (a) and never reported as (a).
* **BRANCH (c) — ADMISSIBLE BUT NOT CLOSED.** G2 passes, v3 fails G5 and G6. → The tier
  does not close at the staked cell size. Reported at full volume; the measured quantity is
  `D_A` per rung and grid with the ladder trend, and the exit of (d) is computed anyway.
* **BRANCH (d) — INADMISSIBLE CARRIER (expected).** G2 fails. → **The µm band is NOT
  certified, cannot be certified from this carrier, and the site's fence stays up.** No
  verdict about the band either way; the failing gate is named on the page. The campaign
  then delivers, without changing a single threshold:
  1. the chart-form reading at every admitted grid, both legs, with the position-blind
     control and the across-seed spread — a measurement of whether the continuum chart is
     the right SHAPE for this dynamics, scoped to nanometres in every sentence;
  2. **THE EXIT**: the occupancy `N*` at which `D_A ≤ 0.02` would be reached, computed from
     the accessible points and reported as an EXTRAPOLATION with its five-order span stated
     beside it, at wager strength, never as a certificate. If the accessible points do not
     determine a trend (no monotone dependence on occupancy, or a spread wider than the
     trend), the exit is reported as UNDETERMINED — which is a real answer and is preferred
     to a fitted line through four points. `M-MAX-OVER-SUCCESSES` applies: the exit may not
     be defined by the best-scoring grid alone.
* **BRANCH (e) — VOID, NO SEPARATION.** G7 fails: the position-blind chart scores as well
  as the spatial one. → The instrument does not discriminate spatial structure on this data
  and NOTHING is concluded about the tier, in either direction. The blind and spatial
  numbers are both published. This branch outranks (c) and (d): a reading that cannot tell
  a real chart from a scrambled one is not a reading.
* **BRANCH (f) — REFUSED.** G1, G8, G9a or G11 fires. → The run did not measure what it set
  out to measure and the cause is named. No verdict.

**Pre-committed follow-up, so a branch is design and not a rescue.** If (d), the SAME
instrument is run unchanged on the `hydrogen` control arm beside the `fenced` arm, and the
two arms' readings are reported side by side. The hydrogen arm has no water in it at all
(`CENSUS_RESULTS.md` §0), so a continuum chart that reads the same on both is reading
something other than water's structure — and that comparison is committed here rather than
invented after seeing a number.

---

## 6. REFUSALS

* **R1 — an atom outside the box.** Any frame with an atom outside `[0,BOX_W] × [0,BOX_H]`
  is a REFUSAL naming the frame, never a clamp into an edge cell. A clamp would manufacture
  occupancy at the boundary.
* **R2 — a trajectory shorter than the work count.** REFUSAL naming G4, not a pass and not
  a fail (Object rule 9).
* **R3 — a digest mismatch.** REFUSAL naming G1 and the file.
* **R4 — 3D lenses.** Nothing in this rung reports a tetrahedral or `q6` reading; the scene
  is 2D and those lenses refuse on it by construction, as `holon-lens` already enforces.
  The chart here is a 2D chart and every sentence about it says so — a 2D continuum chart is
  a real object (this is the FHP lattice's own setting), but it is not the 3D one the 1 km
  cube face would need, and that gap is named, not bridged.

---

## 7. PLANTS

Every plant names its carrier and the sector it must be nonzero in (M-PLANT-SECTOR), and
each is checked to FIRE on THIS instrument before the gate it guards is trusted
(M-PLANT-OBS — observability is instrument-relative and a plant re-used from another
instrument proves nothing here). All plants are synthetic trajectories written through the
same `.traj` reader the real arms use, so a plant exercises the real code path
(M-FOREIGN-DOMAIN-CORROBORATION: a plant passing through a different path would not be
evidence about this one). Plant construction happens at module scope nowhere: the fixture
writer opens no file on import (M-IMPORT-EXECUTES).

* **P-1 — must VOID (vacuity).** Carrier: a `(1,1)` grid on any real arm, and separately a
  synthetic trajectory whose atoms never leave their starting cells. Sector the plant acts
  on: the TRANSPORT sector, which is exactly ZERO by construction, while the OCCUPANCY
  sector is nonzero (a nontrivial multi-cell reading exists). Expected: `D_A = 0` on every
  chart rung AND **VOID at G3**. This plant is what proves the vacuity fence fires; if it
  certifies, `exists_closed_view` is being reported as a result.
* **P-2 — must CERTIFY.** Carrier: a synthetic trajectory generated by a deterministic
  cellular rule in which each atom's next cell is a function of the current cell occupancy
  alone, with velocities set consistently. Sector: the TRANSPORT sector is nonzero (atoms
  cross faces every frame) and the chart is closed BY CONSTRUCTION. Expected:
  CERTIFIED-STRICT, `D_A = 0`, G3 and G4 met. An instrument that cannot certify a chart
  built to be closed cannot certify anything.
* **P-3 — must REJECT.** Carrier: P-2's trajectory with one hidden bit per atom that decides
  its move and is invisible to every field of the chart. Sector: the HIDDEN sector is
  nonzero while the chart-visible sectors carry the same marginal distributions as P-2.
  Expected: firing collisions at `D_A` near 0.5 on v1, and — the point of the plant —
  **still firing on v3**, because refinement cannot recover a variable the chart does not
  carry. This is the defect the rung exists to detect.
* **P-4 — must VOID (no separation).** Carrier: a trajectory whose positions are drawn
  independently and uniformly every frame, so position carries no dynamical information.
  Sector: the POSITION sector is nonzero (atoms have positions and cells are populated) but
  the TRANSPORT-INFORMATION sector is exactly zero. Expected: spatial and position-blind
  charts score within 0.05 of each other and **G7 VOIDs**. This proves the discriminator can
  actually fire.
* **P-5 — must REFUSE.** Carrier: a trajectory truncated below the work count. Sector: the
  SAMPLE-SIZE sector, nonzero by construction (frames removed). Expected: REFUSAL naming
  G4, not a pass and not a fail.
* **P-6 — must FIRE the self-check (instrument mutation).** Carrier: the instrument with the
  momentum bin index taken from the unrounded float, so v2 no longer refines v1. Sector: the
  QUANTISATION sector, nonzero by construction. Expected: **G8 fires** and the run is
  refused. A planted defect that stays silent is a defect in the plant, and this one is
  checked to fire before G8 is trusted.
* **P-7 — must NOT fire (the paired negative).** Carrier: the instrument with the cell
  enumeration order reversed, which is a relabelling and must change nothing. Sector: the
  PRESENTATION sector, nonzero by construction while the chart's partition is identical.
  Expected: every reading bit-identical. `M-PRESENTATION-VERDICT` is its warrant — a
  criterion on the chart must be invariant under re-presentation, demonstrated on a
  re-presented instance. P-6 and P-7 are only meaningful as a pair: one must fire and one
  must not.

**No plant reads a construction tag.** The chart's signature cannot reach the arm label, the
seed, or the plant's own name (M-TAG-AS-PROPERTY): a verdict computed from construction
metadata is a lookup wearing a measurement's clothes, and blindness here is enforced by
what the chart function is given, not by discipline.

---

## 8. WHAT THE 1 km FACE WOULD NEED BEYOND COMPOSITION

Recorded in the freeze so the results document cannot invent it later.

`Tiers.lean::closed_comp` is machine-checked and composes certificates soundly — but it
composes what it is given, and it cannot manufacture a µm tier from a 1.8 nm one. If branch
(d) lands, as expected, the face does NOT become live by composition, and the honest routes
are exactly two:

1. **Ship the band fenced, with the measured exit** from branch (d)'s item 2 — an owner, an
   exit, and now a number, which is what `WORKBENCH_FSD.md` §9c's own fence law asks for.
2. **Certify a continuum-native tier on its own dynamics.** The FHP/REG+ lattice chart in
   `holon-mesh` is that object and `CIRISOntology/Core/ModeChart.lean` already mechanizes
   it. **It is NOT a Closed view of the molecular dynamics** and composing it as though it
   were would be `M-FOREIGN-DOMAIN-CORROBORATION` at tier scale — a result from a different
   domain read as evidence about this one. It is a separate node with a separate warrant,
   and this freeze neither opens nor forecloses it.

Nothing in the results document may report route 2 as a discharge of rung 2.
