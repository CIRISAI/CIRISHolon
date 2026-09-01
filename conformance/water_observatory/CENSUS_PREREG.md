# THE CLOSURE CENSUS — prereg

*Frozen 2026-09-01, before any census instrument existed and before any
trajectory was dumped. This document stakes the window, the budget, the
controls, the plants, and the meaning of every possible answer. Nothing below
was written after seeing a census reading; the instrument was written after
this file and the git history is the check.*

**misfits:** M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-FINAL-VIEW-COLLISIONS,
M-PLANT-OBS, M-PLANT-SECTOR, M-BASE-RATE-OMITTED, M-TAG-AS-PROPERTY,
M-MAINTENANCE-LENS, M-STALE-INSTRUMENT, M-CHEAPER-THAN-ITS-PRICE,
M-NULL-MISSTAKE, M-EXIT-DISCRIMINATOR, M-VOLUME-SCALE, M-HOMOG,
M-DEVICE-CLASS, M-PROVENANCE-OVERREACH.

---

## 0. Why this exists

`waterquench` reports molecules by CONNECTED-COMPONENT NAMING: it reads the
final frame's bonded-pair graph, takes each component's nuclear composition,
and prints `OH2` when a component happens to hold one oxygen and two
hydrogens. That is a statement about ONE FRAME and about a FORMULA. The road's
item 5 says the product must instead be **one persistent three-member
QUOTIENT**: a set of atoms whose membership view is Closed over a trajectory
window within a stated budget.

The difference is not pedantic and it is not rhetorical. A box of twelve atoms
at 300 K under a bond criterion with **no distance cutoff** (`sim.rs`:
`bonded = e_rel < 0.0 && r < r_outer`) will produce components of every
composition at some frame or other, purely by the criterion flickering as pairs
drift across `e_rel = 0`. `OH2` at frame 20000 is, a priori, as likely to be
such a flicker as to be a molecule. This census is the instrument that tells
those two apart, and it is staked to be able to return either answer.

**This census can only wound or harden the water claim. It cannot manufacture
it.** If the census says TRANSIENT, the correct report is that the programme's
"first emergent OH2" was a naming artifact, said at full volume.

---

## 1. The view, the motion, and the two legs

From `lean/CIRISHolon/Object.lean`:

```
Closed v T  ≔  ∃ h, v ∘ T = h ∘ v        witness: `Closed`
Held   v T  ≔  v ∘ T = v                 witness: `Held`
Closed v T ↔ ∀ x y, v x = v y → v (T x) = v (T y)
                                         witness: `closed_iff_fiber_invariant`
NonFactoring (fun _ => v) (v ∘ T) ↔ ¬ Closed v T
                                         witness: `nonfactoring_iff_not_closed`
```

* **X** — the micro-state: positions and velocities of all `n` atoms, plus the
  species assignment.
* **T** — ONE GRAIN BOUNDARY, `Sim::step_frame(SUBSTEPS)` with `SUBSTEPS = 64`.
  This is the engine's own coarse clock (the holon layer runs at grain
  boundaries), so the census's motion is the engine's motion and not a second
  one invented here.
* **P(x)** — the partition of arena indices induced by the bonded-pair graph,
  read from the engine's OWN union-find (`Sim::cluster_roots`). No second
  implementation of the bond reading is written anywhere in this census; the
  trajectory dump carries the engine's `bonded` bit per pair, and the census
  reads those bits. `sim.rs` already records why: "Two implementations of a
  cluster reading is how the two of them come to disagree."
* **Arena indices, never sorted or spatial indices** (Object rule 6). A block
  is a set of ARENA indices; two frames agree about a block only if the same
  physical nuclei are in it.

### Leg A — HELD (is there a persistent quotient?)

For a candidate block `B ⊆ A`, `|B| ≥ 2`, the block view is

```
v_B : X → {0,1},   v_B(x) = 1  iff  B is EXACTLY a block of P(x)
```

"Exactly a block" means: every pair inside `B` is connected within `B`, and no
atom outside `B` is in `B`'s component. `Held v_B T` over a window is
`v_B ≡ 1` across it.

### Leg B — CLOSED (does the membership view carry its own dynamics?)

For the full partition view `v = P`, closure is fiber-invariance. On a
trajectory we can only test the observed fibers: collect every pair of frames
`(s,t)` with `P_s = P_t` and ask whether `P_{s+1} = P_{t+1}`. A pair where they
differ IS a witness pair in the exact sense of `nonfactoring_iff_not_closed`,
exhibited rather than argued.

**Leg B cannot prove closure.** Absence of a witness pair on a sampled
trajectory is a failure to refute, at the resolution sampled. It is reported as
such, never as "Closed".

---

## 2. THE STAKES

### 2.1 Window `W`

Staked in PHYSICAL TIME, not frames, because the protocol's `dt` is derived
per-scene and differs between seeds (0.5386 vs 1.0772 a.u. in the banked P2
log). Frames are converted per seed at census time via
`t_frame = dt · SUBSTEPS · 2.4188843265e-2 fs`.

> **W = 834 fs** (0.834 ps).

Why that number, stated before it is used:

| reference motion | period | periods inside W |
|---|---|---|
| O–H stretch (3657 cm⁻¹) | 9.12 fs | 91 |
| H–O–H bend (1595 cm⁻¹) | 20.9 fs | 40 |
| free rotation of H₂O at 300 K (I ≈ 3.0e-47 kg m²) | 535 fs | 1.6 |

A block that holds across 91 stretch periods and more than one full tumble is
not a threshold flicker. At the fine `dt` this is **W = 1000 frames**; at the
coarse `dt`, 500.

### 2.2 Budget `β` and flicker run `L_flick`

Exact closure is not expected (OBJECT.md rule 2); a budget is.

> **β = 0.02** — at most 2% of the window's frames may read `v_B = 0`.
> **L_flick = 8.4 fs** — no single breach run may exceed ONE O–H stretch period
> (10 frames at fine `dt`, 5 at coarse).

`L_flick` is the clause that stops β from being an escape hatch: 2% of 1000 frames
is 20 frames, and 20 consecutive frames of dissociation is a dissociation, not
a flicker. Both clauses must hold. **The strict reading (β = 0) is computed and
reported beside the budgeted one in every case**; the headline is the strict
one, and a budgeted-only pass is reported as budgeted-only.

### 2.3 Anti-vacuity, Leg A (M-FIXED-POINT-TRAJECTORY, M-VACUOUS-SUCCESS)

A trajectory-based closure gate is vacuous on a carrier that does not move. A
held block must be a MOVING carrier:

> **internal RMS displacement ≥ 0.1 bohr** across the window, measured in the
> block's own centre-of-mass frame (so translation of the whole block cannot
> pay this bill), AND
> **the block's atoms must not be a fixed point of the pair geometry**: at
> least one intra-block separation must vary by ≥ 0.05 bohr across the window.

A block failing these is reported **VOID (frozen carrier)** — not passed, not
failed.

### 2.4 Anti-vacuity, Leg B (M-VACUOUS-SUCCESS)

A functionality test over readings each visited once is empty: every reading
has exactly one successor and the defect is 0 by construction. The census
therefore asserts its WORK COUNT.

> **≥ 200 informative transitions required.** An informative transition is one
> departing from a partition reading visited at least twice in the analysed
> span. Below 200, Leg B returns **VOID**, and VOID is printed as loudly as a
> pass. The count is printed whether or not it passes.

### 2.5 Control floor — the random-block base rate (M-BASE-RATE-OMITTED)

A held-block criterion that any three atoms pass measures nothing.

> For every certified block, **200 random blocks of the SAME COMPOSITION** are
> drawn from the same trajectory's atoms and run through the identical
> criterion. If **more than 5%** of random blocks reach `W`, the census is not
> discriminating on this trajectory and the reading is **VOID (no separation)**.

The random-block pass rate is printed for every trajectory, pass or fail. This
is the eligible-pool rate the misfit demands, not an afterthought.

### 2.6 Protocol equality (M-STALE-INSTRUMENT, M-CHEAPER-THAN-ITS-PRICE)

The trajectories are REGENERATED from the frozen seeds by a separate example
(`waterquench_traj`) which does not touch `waterquench.rs`. Two gates guard the
claim that they are the same trajectories:

> **The frozen-protocol block of both example files must be byte-identical**,
> enforced by a test that reads both sources — so any constant another lane
> changes fires it.
> **The regenerated final-frame reading must equal the reference run's
> reading** for the same seed, molecule list and all.

And the price must close: a regenerated seed that finishes far below the
reference run's measured per-seed cost is not that seed's trajectory. The
per-seed wall time is recorded and compared.

---

## 3. THE GATES

- **G1 — instrument identity.** The frozen-protocol source block shared by
  `waterquench.rs` and `waterquench_traj.rs` is EXACT byte equality; 0 bytes of
  permitted difference. witness: `none (a source-equality gate has no theorem; it is a CI test)`
- **G2 — trajectory equality.** For each regenerated seed, the final-frame
  molecule census equals the reference run's line for that seed: EXACT string
  equality on the composition multiset, free-O count, free-H count, and largest
  component. witness: `none (an equality-of-runs gate has no theorem; it is a measured reproduction)`
- **G3 — Leg A strict.** A block is CERTIFIED-STRICT iff `v_B ≡ 1` over ≥ 1
  window of `W = 834` fs. witness: `Held`
- **G4 — Leg A budgeted.** CERTIFIED-BUDGETED iff `v_B = 1` on ≥ 98% of a
  window's frames and every breach run ≤ 8.4 fs. witness: `Held`
- **G5 — moving carrier.** Internal RMS displacement ≥ 0.1 bohr and ≥ 1
  intra-block separation varying by ≥ 0.05 bohr, else VOID.
  witness: `none (an anti-vacuity gate on a measurement; M-FIXED-POINT-TRAJECTORY is its warrant)`
- **G6 — Leg B work count.** ≥ 200 informative transitions or VOID.
  witness: `closed_iff_fiber_invariant`
- **G7 — Leg B defect.** The witness-pair rate `D` over the analysed span, with
  every witness pair exhibited by frame index. `D = 0` is reported as
  "no witness pair found at this resolution", never as closure.
  witness: `nonfactoring_iff_not_closed`
- **G8 — control floor.** Random-block pass rate ≤ 5% over 200 draws per
  certified block, else VOID. witness: `none (an empirical base-rate floor; M-BASE-RATE-OMITTED is its warrant)`
- **G9 — budget non-expansion.** `D` measured on the window's second half must
  not exceed 1.05 × `D` on its first half (OBJECT.md rule 1's non-expansive
  leak, in its discrete form). witness: `Closed`

---

## 4. THE BRANCHES — every answer's meaning, staked in advance

Let `B*` be the block whose final-frame composition is (1 O, 2 H) in the run
that reported an emergent OH2.

* **BRANCH (a) — QUOTIENT.** `B*` passes G3 (strict) and G5 and G8.
  → The OH2 is a persistent quotient. Road item 5 is MET for that seed.
  The claim is hardened, and the report says which window and how many.
* **BRANCH (b) — BUDGETED QUOTIENT.** `B*` fails G3 but passes G4, G5, G8.
  → Held within the stated budget only. Road item 5 is MET AT BUDGET,
  and the budget is named in the same sentence as the claim, every time. This
  is a weaker result than (a) and is never reported as (a).
* **BRANCH (c) — TRANSIENT.** `B*` fails G4.
  → The OH2 was a graph component whose formula happened to be H₂O. Road item 5
  is NOT met. The water claim is NOT hardened; the correct headline is
  that connected-component naming produced a molecule the closure test rejects.
  Reported at full volume, and the longest held run is reported as the measured
  quantity so the distance to the bar is visible.
* **BRANCH (d) — VOID.** `B*` fails G5 or G8, or G2 fails.
  → The census did not measure what it set out to measure. No verdict about the
  OH2 either way. The failing gate is named.
* **BRANCH (e) — NO CANDIDATE.** No (1 O, 2 H) component exists in any
  regenerated trajectory's final frame.
  → The regeneration disagrees with the reference run; G2 has fired; this is
  branch (d) with a specific cause, and the census is void until the
  disagreement is explained.

**Pre-committed follow-ups** (a branch is design; a rescue is post-hoc):

* If (c), the SAME instrument is run on every H₂ block in every trajectory
  without changing a threshold. H₂ is the banked, uncontroversial molecule of
  SATURATION-1. If H₂ also fails G3, the instrument's bar is too high and the
  reading is VOID rather than negative — this is the instrument's own control
  and it is committed here, not invented afterwards.
* If (a) or (b), the census is run on the OTHER seeds' components with no
  threshold change, and the pass rate across compositions is reported. A census
  that certifies O₄H₄ droplets as readily as OH₂ has certified nothing about
  water.

---

## 5. PLANTS

Every plant names its carrier and the sector it must be nonzero in
(M-PLANT-SECTOR), and each is checked to FIRE on this instrument before it is
trusted (M-PLANT-OBS). The sector the plant acts on is stated per plant.

* **C-1 — must CERTIFY.** Carrier: a synthetic 3-atom trajectory whose three
  atoms are bonded to each other and to nothing else in every frame, with
  thermal internal motion. Sector the plant acts on: the MEMBERSHIP sector; the
  planted signal must be nonzero in the block-view time series `v_B`, which
  reads 1 in every frame. Expected: CERTIFIED-STRICT. A census that cannot
  certify this cannot certify anything.
* **C-2 — must REJECT (dissociation).** Carrier: same block, bonded for
  `W/2` frames then permanently split. Nonzero in the membership sector: `v_B`
  transitions 1 → 0 and never returns. Expected: FAIL G3 and G4.
* **C-3 — must REJECT (budget abuse).** Carrier: `v_B = 1` except for one
  breach run of `L_flick + 1` frames placed mid-window, total breach fraction
  below β. Nonzero in the membership sector by construction: the breach run is
  the plant. Expected: FAIL G3 (any breach) and FAIL G4 (run too long) even
  though the 2% budget alone would pass it. This plant exists solely to prove
  the budget is not an escape hatch.
* **C-4 — must REJECT (the naming artifact itself).** Carrier: a trajectory
  where a (1 O, 2 H) component exists at the FINAL frame but its atom
  membership is reshuffled every 3 frames. Nonzero in the membership sector:
  the composition series is constant while the block series flickers — the
  plant acts on membership, not on formula. Expected: the formula reader says
  H₂O, the census says TRANSIENT. This is the defect the census exists for and
  it must fire.
* **C-5 — must REFUSE.** Carrier: a trajectory shorter than `W`. Expected:
  REFUSAL naming the window, not a pass and not a fail (Object rule 9).
* **C-6 — must VOID (frozen carrier).** Carrier: a block held perfectly but
  with all velocities zero. Nonzero in the membership sector (`v_B ≡ 1`) while
  ZERO in the motion sector — which is precisely the vacuity G5 catches.
  Expected: VOID, not CERTIFIED.

---

## 6. THE LENS STACK — scope and refusals, staked here

Six lenses (WP-4). Each declares the variable it claims to measure, and
REFUSES where the scene cannot carry it (M-MAINTENANCE-LENS: a lens that hides
the variable it claims to measure cannot measure it).

| lens | variable | refusal |
|---|---|---|
| q-tetrahedral (Errington–Debenedetti) | 3D angular order over 4 nearest neighbours | REFUSES on a 2D scene — the P2 quench scenes are `Dims::Two`, where the tetrahedral angle distribution is degenerate and `q` is not the quantity its name claims |
| Steinhardt `q6` | 3D bond-orientational order, `l = 6` | REFUSES on a 2D scene, same reason |
| hexatic `ψ6` | 2D bond-orientational order | REFUSES on a 3D scene |
| MSD / diffusion | single-particle displacement | REFUSES to report a diffusion constant where wall collisions dominate the fitted window |
| H-bond census | Luzar–Chandler geometry | REFUSES where the scene has no O–H pair |
| closure defect | witness-pair rate of a binned macro view | REFUSES below the staked informative-transition count |

**The 2D refusals are the point, not an inconvenience.** The banked P2 scenes
are two-dimensional. Reporting a tetrahedrality number on them would be a
lens that does not contain its own variable — the exact shape of
M-MAINTENANCE-LENS. The 3D lenses are implemented, gated against exact
reference lattices, and stand ready for the 3D tier; they refuse the 2D data
rather than fabricate a reading from it.

Stated criteria, so the lenses are reproducible:

* **q-tet:** `q = 1 − (3/8) Σ_{j<k} (cos ψ_jk + 1/3)²` over the four nearest
  neighbours. Gate: EXACT tetrahedral neighbour set → `q = 1` to 1e-12; ideal
  gas → `q ≈ 0`.
* **q6:** `q6 = sqrt(4π/13 · Σ_m |q6m|²)`, `q6m` averaged over the neighbour
  set. Gates against the published lattice values: FCC 0.5745, BCC 0.5106,
  simple cubic 0.3536, each to 1e-3.
* **ψ6:** `ψ6 = |N⁻¹ Σ_j exp(6iθ_j)|` over the six nearest neighbours. Gate:
  perfect triangular lattice → 1 to 1e-12; square lattice with 6 neighbours →
  below 0.5.
* **H-bond (Luzar–Chandler):** `r(O···O) < 6.6140 bohr` (3.5 Å),
  `r(O···H) < 4.6298 bohr` (2.45 Å), and `∠(H–O_donor···O_acceptor) < 30°`.
  All three, stated in bohr because the engine's unit is bohr.
* **largest domain:** largest component of the stated edge set, reported with
  WHICH edge set (bonded-pair or H-bond) — never with the edge set implicit.

---

## 7. THE BLIND CLASSIFIER — plants P-1 and P-5

The classifier reads TRAJECTORY ONLY. The launch label (`ice`, `liquid`,
`vapor`) is carried in a separate structure the classifier's signature cannot
reach — blindness enforced by the type, not by discipline
(M-TAG-AS-PROPERTY: a verdict computed from construction metadata is a lookup
wearing a measurement's clothes).

* **P-1 — preset blindness.** Carrier: a liquid trajectory whose launch label
  says `ice`. The plant acts on the LABEL sector and must be nonzero there (the
  label differs from the truth) while being exactly zero in the trajectory
  sector (the coordinates are a liquid's, unmodified). Expected: LIQUID. A
  classification of ICE convicts the classifier of reading its label.
* **P-5 — vapor never reads ICE.** Carrier: dilute-gas trajectories. The plant
  acts on the DENSITY sector and must be nonzero there. Staked in advance:
  over **200 synthetic vapor trajectories, 0 may classify ICE**; at 0/200 the
  Clopper–Pearson 95% upper bound on the false-crystal rate is **1.5%**, and
  that bound is the number this campaign publishes. Any ICE verdict on vapor
  fires the plant and the classifier is refused until the cause is named.

---

## 8. What this census does NOT claim

* It does not claim water formed. It claims a membership view is or is not
  Closed on a regenerated trajectory of a frozen protocol.
* It does not claim the bond criterion is correct. It inherits the engine's
  criterion deliberately, so that census and headline read one instrument.
* It does not claim 2D quench scenes are liquid water. They are twelve atoms in
  a plane; the lens stack's refusals say so mechanically.
* It does not claim closure. Leg B can only exhibit witness pairs or fail to
  find them, and the second is reported as a failure to refute.
