# RUNG 2 — AMENDMENT A2: the chart is the lattice-gas chart

*Frozen 2026-09-02, before a line of the A2 instrument existed and before any lattice-gas
reading was computed. `RUNG2_PREREG.md` (`aee5317`) and `RUNG2_RESULTS.md` (`ad2b6c6`) are
already banked; this amendment does NOT edit either of them. The git order is the check.*

**misfits:** M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-FINAL-VIEW-COLLISIONS,
M-ONE-MODEL-DELTA, M-NULL-MISSTAKE, M-UNTESTED-GAP, M-CONJUNCTION-MONOTONE,
M-BASE-RATE-OMITTED, M-BUDGET-LAUNDER, M-EXIT-DISCRIMINATOR, M-MAX-OVER-SUCCESSES,
M-SORTS-NOT-SEPARATES, M-TAG-AS-PROPERTY, M-PLANT-OBS, M-PLANT-SECTOR, M-POPULATION-CHOICE,
M-PRESENTATION-VERDICT, M-NONBIJECTIVE-STEP, M-HOMOG, M-VOLUME-SCALE, M-STALE-INSTRUMENT,
M-PROVENANCE-OVERREACH, M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS,
M-IDLE-CALIBRATED-TIMEOUT, M-CACHE-KIND, M-IMPORT-EXECUTES, M-MAINTENANCE-LENS,
M-FOREIGN-DOMAIN-CORROBORATION, M-PROBE-THE-RESOURCE, M-LOOP-BLIND, M-PARITY-PROTECT,
M-COND-PROBE, M-SORTS-NOT-SEPARATES.

---

## A2.0 WHY THIS AMENDMENT EXISTS, AND WHAT IT DOES NOT DO

The operator's instruction: this programme did fluid dynamics BEFORE water, and the fluid
chart must be built on that machinery rather than beside it — with the rule that **any
deviation must be argued AGAINST the existing machinery by MEASUREMENT, not by taste.**

`RUNG2_PREREG.md` staked a cell-field chart (occupancy, thermal-momentum bins,
kinetic-energy bins). That is a deviation. So this amendment does not argue; it builds the
lattice-gas chart and measures both on the same trajectories, same grids, same legs, same
controls — **one variable, the chart** — and reports which reads better.

**What A2 does not do.** It does not reopen `RUNG2_RESULTS.md`'s verdict by re-staking a
bar, it does not edit the frozen prereg, and it does not touch the trajectory set. A2's
verdict can only (i) leave branch (d) standing, (ii) move it, or (iii) show the banked chart
was the wrong instrument — and A2.5 stakes all three in advance.

---

## A2.1 THE CHART — the lattice-gas one, and its lineage

**FHP-6, because this carrier is two-dimensional.** `engine/MESH_DESIGN.md` §2.1 makes the
standing 3D choice **FCHC-24** and gives the warrant: cubic point symmetry cannot carry an
isotropic fourth-rank momentum-flux tensor, and the face-centred *hyper*-cubic 24 can.
**This carrier's scenes are `dims = 2`**, so FHP-6 is the like-for-like chart and FCHC-24 is
NOT exercised here. A2 does not adopt a cheaper mode set and does not touch the 3D choice;
a 3D instrument is a different campaign and would take FCHC-24 unchanged.

> **CREDIT**, lifted verbatim from `engine/MESH_DESIGN.md` §2.1's prior-art block, per the
> convergence rule — these are hits, not strikes:
>
> * **FHP-6, the chart this engine already wears:** Frisch, Hasslacher & Pomeau, *Lattice-gas
>   automata for the Navier–Stokes equation*, Phys. Rev. Lett. **56** (1986) 1505. The
>   hexagonal lattice's fourth-order isotropy is theirs, and it is the whole warrant of the
>   founding 64-state object.
> * **FCHC-24, the chart the 3D design adopts:** d'Humières, Lallemand & Frisch, *Lattice gas
>   models for 3D hydrodynamics*, Europhys. Lett. **2** (1986) 291.
>
> The mathematics is openly borrowed. Ours is the sector enumeration and, here, the
> molecular-to-mode map and its measurement.

**The object, and where it already lives in THIS tree.** `ciris-sim-core::regplus`
carries the six FHP directions in axial integer coordinates
`[1,0] [0,1] [-1,1] [-1,0] [0,-1] [1,-1]` and the exact local `(N, P)` label
`sector(local: u8) -> SectorLabel { occupancy, momentum }`. Its own test
`runtime_sector_table_matches_the_lean_theorem` reproduces **53 sectors with dimension
histogram 44 / 7 / 2** in-tree. That test is this amendment's resolving witness for the
object, because `Core/ModeChart.lean` (`fhpChart_injective`) and `Core/Lattice.lean` live in
the CIRISOntology tree and have no copy here; they are cited as upstream prior art and NOT
as local witnesses.

**A2 reuses `regplus::sector` and does not reimplement it.** Two implementations of one
label is how the two of them come to disagree — the same rule `sim.rs` states about the bond
reading and the census obeyed.

### A2.1.1 The chart ladder

Three rungs, each a refinement of the last, and the ladder is the machinery's own structure
rather than a new invention: the 64-state local word, its 53-sector quotient, and the
occupancy marginal.

| chart | per cell | what it is |
|---|---|---|
| **w1** | `N` — the number of OCCUPIED MODES, 0…6 | the density field, capped by exclusion |
| **w2** | `(N, P)` — the full sector label | **the operator's chart**: the conserved-fields view |
| **w3** | the 6-bit local word | the founding 64-state object, pre-quotient |

`w2` is a function of `w3` (`sector`), and `w1` is a function of `w2`, so
`refinement_removes_collisions` applies and G8 (both forms) carries over unchanged.
**Note `w1` is NOT `RUNG2_PREREG.md`'s `v1`**: `v1` counts ATOMS per cell (0…12), `w1`
counts occupied MODES (0…6). They are different fields and the results must never conflate
them.

---

## A2.2 THE MAP — molecular scene to mode occupancy

This is the one genuinely new piece and it is stated precisely enough for a bit-identical
reimplementation.

The six axial directions are the hexagonal unit vectors at 60° spacing. In Cartesian
components, axial `[p, q] ↦ (p + q/2, q·√3/2)`, giving
`(1,0) (½,√3/2) (−½,√3/2) (−1,0) (−½,−√3/2) (½,−√3/2)` — all unit length, 60° apart.

> **THE MAP.** For each cell `c` and each atom `a` assigned to `c` by
> `RUNG2_PREREG.md` §2.1's cell rule: let `u = (vx, vy)` be the atom's planar velocity.
> Mode `d` is the direction MAXIMISING the dot product `u · e_d`, ties broken to the LOWEST
> mode index. Mode `d` of cell `c` is OCCUPIED iff at least one atom in `c` maps to `d`.
> The cell's local word is the 6-bit occupancy; its label is `regplus::sector` of that word.

**No rest mode, stated rather than smuggled.** FHP-6 has no rest particle. An atom with
exactly zero planar velocity has no defined direction; it is assigned to NO mode and
COUNTED, and the count is printed. This is a loss and it is reported, never patched with an
invented seventh mode — adopting FHP-7 to absorb it would be changing the mode set to
flatter the map, which is the move §A2.1 refuses.

**THE EXCLUSION FENCE, which is the map's real cost.** FHP is an exclusion automaton: at
most one particle per mode per cell. The map can send two or more atoms to one mode, and
Boolean occupancy then loses them. `Core/ModeChart.lean` states this fence exactly —
Boolean occupancy is exact only for DETERMINATE states, and over mixtures the exact
invariant is the CAP, mean occupancy in `[0,1]`. So:

> **The SATURATION RATE is a first-class reported quantity**: the fraction of (cell, frame)
> pairs in which some mode carries ≥ 2 atoms, and the fraction of ATOMS thereby lost to the
> Boolean word. Both are printed for every grid, whatever the verdict.

A high saturation rate does not invalidate the chart; it bounds what the chart can be
claimed to be a view OF, and the results document must quote it beside every A2 defect.

---

## A2.3 THE GATES — inherited unchanged, plus what is new

Every gate of `RUNG2_PREREG.md` §3 applies verbatim to the lattice-gas chart: G1 (digests),
G2 (admissibility), G3 (vacuity fence), G4 (work count ≥ 200), G5 (Leg A, β = 0.02),
G6 (Leg B held out), G7 (control floor, separation ≥ 0.05, on `BlindLabel`), G8 (ladder
self-check, BOTH the weak monotone form and the strong `refines` form that `RUNG2_RESULTS.md`
§5.2 found necessary), G9a/b/c, G10, G11. The grid list of §2.5 is unchanged, so the
comparison is one-variable.

New to A2:

- **A1 — map non-degeneracy.** The mode occupancy must actually vary: ≥ 8 distinct local
  words must occur across the analysed span, per grid, else the map is degenerate and the
  reading is **VOID (degenerate map)**. A chart whose word never changes is
  `exists_closed_view` again, one level down.
  witness: `exists_closed_view`
- **A2g — saturation disclosure.** The saturation rate and the lost-atom fraction are
  printed for every grid. This is a DISCLOSURE, not a bar: no threshold on it can fire,
  because the honest use of the number is to scope the claim, and a bar would tempt the
  reading toward grids that flatter the map. If saturation exceeds 0.50 the results document
  must say in the same sentence as any A2 defect that the chart is then a view of the
  MODE SET and not of the atoms.
  witness: `none (a disclosure of the map's lossiness; M-MAINTENANCE-LENS is its warrant — a lens that hides the variable it measures cannot measure it)`
- **A3 — the phase-resolved defect, for door (c).** For `p ∈ {1, 2, 3, 4, 6, 8}` and each
  residue `r < p`, `D_A` restricted to collisions whose BOTH frames satisfy `i ≡ r (mod p)`,
  reported with its own work count; a `(p, r)` cell with fewer than 200 informative
  transitions is VOID and cannot contribute. A grain boundary exists iff some `(p, r)`
  reaches `D_A = 0` EXACT with its work count met.
  witness: `recurrence_eight`
- **A4 — the one-variable comparison.** `D_A` for the lattice-gas chart and for the banked
  cell-field chart, on the same (arm, seed, grid, kind) cells, reported side by side with
  their collision counts. Charts are compared WITHIN a rung of comparable arity, never
  across the two ladders' rung indices — `RUNG2_RESULTS.md` §4 established that `D_A` is not
  a valid cross-rung comparator, and A2 does not repeat that error.
  witness: `rate_unique_on_range`

---

## A2.4 THE LIVE READOUT — the door this certificate implies

Required in the freeze, chosen among (a) an aggregate defect over the band's rows, (b) a
defect against a specific coarse VIEW, (c) a grain-schedule readout. **A2 names TWO, with
the reason.**

> **DOOR (b) — PRIMARY. The closure defect of the fluid band against the lattice-gas
> `(N, P)` sector chart (`w2`), displayed beside its budget β = 0.02 and its measured
> saturation rate.**
>
> Why (b) and not (a): an aggregate over the band's rows would average a quantity whose
> denominator differs per row, and `RUNG2_RESULTS.md` §4 measured exactly how badly that
> misleads — the defect ratio's denominator collapsed 80× under refinement and the ratio rose
> while the chart got more informative. A single named view has one denominator and one
> meaning. The page must show the defect, the budget and the saturation together; the defect
> alone would be a number without its scope.

> **DOOR (c) — SECONDARY, and staked to come back a NULL.** The grain schedule of this
> chart: the steps at which the coarse view is exact and a refresh is free.
>
> `engine/crates/holon/src/grain.rs` states the fence this door must respect — the period
> belongs to the measured coupling, NEVER to nature and never to the engine, and
> `Grain::measured` refuses an unprovenanced schedule. **There is no measured schedule for
> this chart**, and A3 is the measurement that would find one.
>
> **Staked in advance: A3 returns EMPTY** — no `(p, r)` with an adequate work count reaches
> `D_A = 0`. If that is what A3 measures, door (c) displays **"no free refresh point
> measured"** and NOT a period; an empty `exact_at` is not even constructible-usable, since
> `Grain::steps_to_close` panics on it. If A3 instead finds a boundary, that is a positive
> result and door (c) displays the measured `(p, r)` with its work count and its provenance.

**Neither door may display a number implying the band is live.** Rung 2 did not certify;
both doors are readouts of a FENCED band, and the page must render them beside the fence,
its owner and its exit.

---

## A2.5 THE BRANCHES — staked before the measurement

* **A2-(i) — THE VERDICT STANDS.** The lattice-gas chart also fails G2 and also reads
  `NotClosed` where live. → `RUNG2_RESULTS.md`'s branch (d) is unchanged and is now
  chart-independent, which strengthens it: the scissor is a property of the CARRIER.
* **A2-(ii) — THE LATTICE-GAS CHART READS BETTER, AND THE BANKED CHART WAS THE WRONG
  INSTRUMENT.** The lattice-gas chart certifies (strict or budgeted) on some live cell where
  the cell-field chart did not. → Reported at full volume as a correction to
  `RUNG2_RESULTS.md`, the operator's instruction is vindicated by measurement, and the door
  (b) number becomes the certifying one. This branch is why A2 is a measurement and not a
  paragraph.
* **A2-(iii) — THE LATTICE-GAS CHART READS WORSE.** Higher `D_A` at comparable arity, or it
  VOIDs where the cell-field chart scored. → Reported as such, WITH the saturation rate,
  because a lossier map reading worse is the expected shape of that loss and must not be
  presented as a verdict about lattice gases. The deviation in `RUNG2_PREREG.md` is then
  argued against the machinery BY MEASUREMENT, which is what the operator's rule asks for.
* **A2-(iv) — VOID.** A1 fires (degenerate map), or G7 fails for both charts. → Nothing is
  concluded about which chart is better.
* **A2-(v) — REFUSED.** G8's strong form fires on the lattice-gas ladder, or `regplus::sector`
  disagrees with its own banked test. → The instrument is convicted, not the physics.

**Pre-committed, so it is design and not a rescue:** whichever branch lands, the SATURATION
rate and the ZERO-VELOCITY count are reported for every grid, and the admissibility reading
(G2) is reported for the lattice-gas chart even though it cannot differ — because a bar
quoted without its measured value is not a bar, and because a reader must be able to check
that it did not differ.

**A staked prediction, separable from the above.** The G2 admissibility verdict does NOT
move: the occupancy/transport scissor is a property of the carrier — 12 atoms in
1.831 × 1.101 nm — and no choice of mode set changes how many atoms sit in a cell. If it
moves, the scissor argument in `RUNG2_RESULTS.md` §0 is wrong and this campaign says so.

---

## A2.6 PLANTS FOR THE MAP

The operator's instruction was to plant a defect in the map specifically. Each names its
carrier and the sector it must be nonzero in, and each is checked to FIRE on THIS instrument
before the gate it guards is trusted.

* **MAP-1 — must REJECT (the map stops reading velocity).** Carrier: the identical
  trajectories with the mode chosen from the atom's ARENA INDEX instead of its velocity
  direction. Sector the plant acts on: the VELOCITY-DIRECTION sector, which the mutated map
  reads as exactly zero while the occupancy sector is unchanged. Expected: `D_A` rises and
  the G7 separation against `BlindLabel` collapses — a map that does not read velocity
  cannot carry the momentum field, and if this scores as well as the real map then A2's
  chart is not measuring the lattice-gas structure at all.
* **MAP-2 — must NOT fire (the mode set is a presentation).** Carrier: the six directions
  cyclically relabelled by one position, which is a 60° rotation of the mode SET. Sector:
  the PRESENTATION sector, nonzero by construction while the partition of velocities into
  modes is identical. Expected: collisions, firings, work counts **bit-identical**. The
  `(N, P)` labels rotate with the frame; the chart's collision structure may not.
  MAP-1 and MAP-2 are only meaningful as a pair: one must fire and one must not.
* **MAP-3 — must REJECT (the momentum field carries something).** Carrier: the real map with
  the sector's `P` discarded, keeping only `N` — that is, `w1` used where `w2` is claimed.
  Sector: the MOMENTUM sector, exactly zero by construction. Expected: strictly more
  collisions than `w2` and a different `D_A`. If `w1` and `w2` read identically then `P` is
  carrying nothing on this carrier, which is itself a reportable finding about the map.
* **MAP-4 — must VOID (degenerate map).** Carrier: a synthetic trajectory in which every atom
  has the same velocity direction, so exactly one mode is ever occupied. Sector: the
  DIRECTION-DIVERSITY sector, exactly zero. Expected: fewer than 8 distinct local words and
  **VOID at A1**.
* **MAP-5 — must fire the exclusion fence.** Carrier: a synthetic cell holding six atoms whose
  velocities all point along one direction. Sector: the SATURATION sector, nonzero by
  construction. Expected: saturation rate 1.0 and a lost-atom fraction of 5/6, both printed,
  proving the disclosure of A2g can actually report a loss rather than always reading zero.

**Reuse gate (M-PLANT-OBS).** No plant is inherited from `RUNG2_PREREG.md` §7 on the
strength of having fired there: observability is instrument-relative and the A2 chart is a
different instrument. The inherited plants P-1, P-2, P-3, P-5, P-7 are re-run against the
lattice-gas chart and must fire again, on this chart, before any A2 gate is trusted.

---

## A2.7 COST

Work units, never wall clock (M-PLACEMENT-LOTTERY, M-IDLE-CALIBRATED-TIMEOUT). The banked
run read 320,000 frames at 18.0e6 chart evaluations in about a minute on one core class.
A2's model: the same 320,000 frames, 5 grids × 3 lattice-gas rungs × 4 chart kinds =
60 evaluations per frame = **19.2e6 chart evaluations**, plus A3's phase-resolved pass over
`p ∈ {1,2,3,4,6,8}` (24 residue cells) on the `w2` chart only, which re-groups already
computed readings and adds no chart evaluations. A run arriving at a small fraction of that
model is not that run (M-CHEAPER-THAN-ITS-PRICE). Budget exhaustion VOIDs loudly and never
falls back to a scorable verdict (M-BUDGET-LAUNDER). Single-threaded, one core class, no
accelerator and no bitwise-variant arithmetic route (M-DEVICE-CLASS).
