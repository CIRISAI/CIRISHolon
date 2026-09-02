# RUNG 2 — THE FLUID-ELEMENT TIER: results

*Stakes: `RUNG2_PREREG.md`, frozen and committed (`aee5317`) before a line of the
instrument was written. Instrument: `engine/crates/holon-lens/src/field.rs` +
`examples/rung2.rs` (`7e2b8ef`). Log this document reads:
`rung2_chart.log`; G1's digest check: `rung2_g1_digests.log`. Every threshold quoted here
is one of the freeze's constants. Where a stake turned out to be wrong that is written up
as a finding (§5) and NOT repaired in place.*

---

## 0. THE HEADLINE

### THE FLUID-ELEMENT TIER IS NOT CERTIFIED, THE 1 km FACE DOES NOT FLIP, AND THE FENCE NOW HAS NUMBERS IN IT

**Branch (d) — INADMISSIBLE CARRIER.** `RUNG2_PREREG.md` G2 fails on every grid of every
trajectory, 75 of 75, and it fails by more than two orders of magnitude. The band is ~1 µm;
the carrier is 12 atoms in 1.831 × 1.101 nm.

And the reason is sharper than "too small". **The carrier's two requirements point in
opposite directions, and no grid satisfies both** — this is the finding, and it is
structural, not a matter of buying more compute:

| grid | cells | atoms/cell | transport (median) | clears transport ≥ 0.05 |
|---|---|---|---|---|
| 1×1 | 1 | 12.0 | 0.0000 | 0/15 |
| 2×1 | 2 | 6.0 | 0.0061 | 0/15 |
| 2×2 | 4 | 3.0 | 0.0153 | 0/15 |
| 4×2 | 8 | 1.5 | 0.0295 | 1/15 |
| 6×4 | 24 | 0.5 | 0.0640 | 13/15 |

Coarse cells hold enough atoms to be a fluid element but the motion never crosses their
faces, so the chart is frozen and the vacuity fence takes it. Fine cells transport, but
**the only grid that clears the fence averages half an atom per cell.** A fluid element
containing half an atom is not a fluid element, and the fluctuation column says the same
thing: σ(n)/⟨n⟩ runs 1.553–1.881 there against an admissibility bar of 0.10.

**What was measured anyway, at the nanometre scale the carrier does reach** (branch (d)'s
pre-committed item 1, run without changing a threshold):

* **The chart is NOT CLOSED.** At 6×4, the only live grid, the Leg-A defect is
  0.048 (median, occupancy rung), 0.224 (momentum), 0.891 (energy) on the fenced arm —
  against a budget of β = 0.02. 42 of 225 spatial cells scored at all; every one of them
  read `NotClosed`. **No cell in the campaign certified, strictly or at budget.**
* **But the instrument is not reading noise, and the control proves it.** At the momentum
  rung on the hydrogen arm the spatial chart beats the coherence-destroying control by
  **+0.598** (median; 0.563–0.643, 7/7 clearing the 0.05 bar) and at the energy rung by
  **+0.288**. The momentum field is spatially coherent at the 5.8 bohr cell scale — which is
  hydrodynamics' own premise — and the chart is measuring it. The defect is real and it is
  an order of magnitude above budget.
* **The density field carries nothing.** At the occupancy rung the separation is
  **−0.0007** (fenced) and **−0.0019** (hydrogen): the scrambled chart is very slightly
  BETTER, not worse. Wrong sign, 0/13 clearing. On this carrier occupancy alone holds no
  spatial structure the closure test can use, and any reading built on it is branch (e).
* **The forward prediction F1 FIRED.** It is dead and stays dead: §4.

**The vacuity theorem was observed doing exactly what the freeze said it would.** At the
1×1 grid the occupancy chart has ONE distinct reading over 20,000 frames and a defect of
exactly 0 — `Tiers.lean::exists_closed_view` in the field chart's clothes, caught by the
fence rather than reported as a certificate. Its momentum and energy charts do NOT read
zero, and that is the G9 finding arriving from the other side: those two quantities are not
conserved on this protocol, so there was never an invariant there to be vacuously closed.

---

## 1. WHAT RAN

`rung2` over the banked census trajectories at
`/home/emoore/holon-artifacts/census-traj`, arms `fenced` and `hydrogen`, 8 seeds each,
20,000 frames each.

**G1 — carrier identity: PASSED.** `sha256sum -c` against `census_traj_manifest.sha256`:
23 files OK, exit 0 (`rung2_g1_digests.log`). Verified out of band because `holon-lens` has
zero dependencies and therefore no sha256; the pin names the files and nothing beside them.

**16 trajectories opened, 15 read, 1 REFUSED** — see §5.1.

**Cost (G11, work units, never wall clock).** Frames read: **320,000**, exactly the frozen
cost model. Chart evaluations: **18,000,000** against a modelled 9,600,000 — 1.875×, and the
excess is named: the freeze modelled 2 chart kinds and the run used 4, adding
`GlobalRelabel` (P-7 on real data) and `BlindIndex` (the freeze's own degenerate control,
kept so §5.2's finding has a witness). Arriving *over* a cost model with a named cause is
disclosed here; `M-CHEAPER-THAN-ITS-PRICE` is about arriving under one.

---

## 2. THE GATES, ONE LINE EACH

| gate | reading | verdict |
|---|---|---|
| **G1** carrier identity | 23/23 digests OK | **PASS** |
| **G2** fluid-element admissibility | 0/75 grid-trajectory cells admissible; best occupancy at ≥2 cells is 6.0 against a bar of 100; best fluctuation 0.157 against 0.10 | **FAIL — expected, and the reason the band verdict is VOID** |
| **G3** vacuity fence | 183 of 225 spatial cells VOID (1×1 on cell count; 2×1, 2×2, 4×2 on transport) | **FIRED, as designed** |
| **G4** work count ≥ 200 | met on every live spatial cell (info 5,257–19,999); the blind control voids here at the momentum and energy rungs on the fenced arm (info 2–38) | **PASS where evaluated** |
| **G5** Leg A defect | 42 live cells, all `NotClosed`; medians 0.048 / 0.224 / 0.891 (fenced) and 0.070 / 0.229 / 0.578 (hydrogen) at 6×4 | **NOT CLOSED, at 2.4×–45× the budget** |
| **G6** Leg B held out | coverage ≤ 0.006 everywhere — the chart's readings almost never recur across the halves | **UNINFORMATIVE, reported not hidden** |
| **G7** control floor | occupancy rung: −0.0007 / −0.0019, 0/13 clear, WRONG SIGN. momentum: +0.598, 7/7. energy: +0.288, 7/7 | **SPLIT — fails at density, passes at momentum and energy** |
| **G8** ladder self-check | weak form (monotone counts) true 300/300; strong form (`refines`) true 900/900 | **PASS — and never quoted as support** |
| **G9a** species/arity constant | true on all 15 read trajectories | **PASS** |
| **G9b** momentum | \|p\| grows from 2.5e-6 to 3.5–21.1 (fenced) and from ~1e-14 to 2.0–8.6 (hydrogen) | **NOT CONSERVED, as the freeze said; ledger leg UNDISCHARGED (§5.3)** |
| **G9c** energy | E_k falls 6.7e-2 → 9.0e-3 (fenced), 6.6e-2 → 1.0e-2 (hydrogen): the 3000 K → 300 K quench | **NOT CONSERVED, as the freeze said; ledger leg UNDISCHARGED** |
| **G10** bijectivity not asserted | no statement here calls `T` unitary or reversible | **HELD** |
| **G11** work-unit pricing | 320,000 frames; 18.0e6 chart evaluations vs 9.6e6 modelled | **PASS with a named 1.875× excess** |

---

## 3. THE PLANTS

Every plant of `RUNG2_PREREG.md` §7 was checked to fire before its gate was trusted. Three
of them did not fire on the first attempt, and all three failures were in the PLANT — §5.

| plant | must | fired? |
|---|---|---|
| **P-1** frozen carrier | VOID | YES — `D_A = 0` on every rung and VOID at the fence, on synthetic and on the real 1×1 grid |
| **P-2** chart closed by construction | CERTIFY | YES — `CertifiedStrict`, `D_A = 0`, and Leg B mismatches 0 |
| **P-3** hidden variable | REJECT at EVERY rung | YES — fires at Occ, Mom and Ene; refinement cannot recover a variable the chart does not carry |
| **P-5** short trajectory | REFUSE | YES — `VoidWorkCount` |
| **P-6** ladder stops refining | FIRE G8 | YES, but only against the STRONG form — §5.2 |
| **P-7** global relabelling | NOT fire | YES — bit-identical on **225/225** (arm, seed, grid, rung) cells on real data. Presentation invariance holds. |

`BlindIndex`, the freeze's literal control, read `distinct = 1` on 75/75 occupancy cells and
VOIDed everywhere — the degeneracy predicted in §5.2, witnessed on real data.

---

## 4. THE FORWARD PREDICTION FIRED

> **F1** staked that `D_A` decreases monotonically up the chart ladder on the spatial chart.

**It does the opposite, on 74 of 75 cells.** Monotone *increasing*: 39/40 (fenced), 35/35
(hydrogen). Monotone decreasing: **0/75**. F1 is dead and is kept in the record, marked
dead.

The mechanism is visible in the collision counts and it is one the freeze had already
named. Up the ladder at 6×4 on hydrogen seed `…5421`, collisions fall 970,983 → 176,675 →
12,067, an 80× collapse of the denominator; the collisions that survive refinement are
increasingly accidental near-coincidences of two binned continuous fields, which have no
reason to share a successor. That is `M-FINAL-VIEW-COLLISIONS` exactly, and its consequence
is a methodological finding for the successor campaign, stated here as a finding and NOT as
a rescue of F1:

> **`D_A` is not a valid comparator ACROSS ladder rungs**, because its denominator is not
> held fixed between them. The comparison that IS valid is within a rung and across charts,
> which is what G7 does — and G7 says the momentum field carries large, real spatial
> structure (+0.598) on the very rungs where F1's statistic was rising. The two readings do
> not conflict; F1 was staked on the wrong statistic, and it fired.

---

## 5. WHERE THE FREEZE WAS WRONG

Four defects in `RUNG2_PREREG.md`, all found by running it. None is repaired in place.

### 5.1 R1 refuses a whole trajectory for one atom-frame

Hydrogen seed `0x53415425` REFUSED at frame 32, atom 10, `y = 20.8418` against
`BOX_H = 20.8`. Measured across all 16 trajectories: **exactly one atom-frame of 3,840,000
is outside the box, by 0.0418 bohr** — 0.2% of the box height, in the opening frames while
the scene is still at 3000 K.

The rule is correctly implemented and badly scoped: `Boundary::Walls` is *soft quadratic*
walls, which permit overshoot by design, and R1 was written as though the walls were hard.
The refusal stands, the seed is excluded, and its counterfactual reading is deliberately
NOT computed — that would be running the analysis under a moved stake. A successor freeze
should carry a tolerance equal to the wall's own softness; setting that number is its work,
not this document's.

### 5.2 G8's staked form cannot catch a chart that stops refining

`ladder_monotone` — the frozen G8 — compares collision COUNTS. P-6 planted a v2 that drops
the occupancy fields entirely, which is not a refinement of v1 at all, and the counts still
fell (41,407 against 179,101). **The plant was silent and the gate would have passed a
broken ladder.**

`refinement_removes_collisions` has a hypothesis — the coarse view factors through the fine
one — and monotone counts do not establish it. `field::refines` checks that hypothesis
directly in O(F) and fires on the mutation. Both forms were run on the real data: weak true
300/300, strong true 900/900. The weak form is kept because it is what the freeze staked;
the strong form is reported beside it as the repair.

### 5.3 G9b and G9c are not computable from this artifact

Both gates ask for ledger closure — the momentum change accounted by the wall term, the
energy draw accounted by the thermostat. **The trajectory dump carries positions,
velocities, bond bits, time and temperature, and no forces and no intervention ledger**, so
neither ledger can be closed from it. The measured drift is reported instead and the ledger
leg is marked **UNDISCHARGED**. "Not computable" and "computed and failed" are different
facts (`M-EXIT-DISCRIMINATOR`) and this document does not blur them.

### 5.4 Two smaller ones

* **The freeze's own control is degenerate.** §3.5 wrote the position-blind chart as "a
  fixed permutation of ARENA INDEX", which makes membership constant in time: zero
  transport, one distinct occupancy reading, VOID at the fence before it can score. It is
  kept and run as `BlindIndex` so the finding has its witness; `BlindLabel` — a per-atom
  permutation of cell LABELS, which preserves each atom's transition times and dwell
  distribution exactly — is the control that discriminates and is reported as an ADDITION,
  never as a substitution. Every G7 number above is `BlindLabel`.
* **"Every firing collision exhibited by frame index" is not possible.** A coarse chart
  carries up to 2.0e8 collisions. The firing COUNT is exact and complete in every row; the
  LISTING is capped at 10 pairs per cell and the log says so on every line that carries one.

---

## 6. WHAT THE 1 km FACE NEEDS, AND WHAT THIS RUNG GIVES IT

**The face does not flip.** `Tiers.lean::closed_comp` is machine-checked and composes
certificates soundly, but it composes what it is given, and there is no fluid-element
certificate to give it. There is also no rung between 1.8 nm and 1 µm in the current ladder.

**What the fence can now say, which it could not before.** The band ships fenced with an
owner, an exit, and numbers:

* the certified carrier is 12 atoms in 1.831 × 1.101 nm at 5.954 atoms/nm²; a 1 µm × 1 µm
  patch at that density is **5.95e6 atoms**, 4.96e5× the scene;
* the trajectory format caps independently at **16 atoms** (`traj.rs::MAX_DUMP_ATOMS`), so
  the band is unreachable by this format before it is unreachable by compute;
* on this carrier there is **no cell size at which a fluid element both holds enough atoms
  and sees the motion cross its faces** — the occupancy/transport scissor of §0.

**The exit, as the freeze required it to be reported.** `RUNG2_PREREG.md` branch (d) item 2
asked for the occupancy `N*` at which `D_A ≤ 0.02` would be reached, computed from the
accessible points. **It is UNDETERMINED, and that is the pre-committed answer, not a
failure to compute one.** The live points are two: occupancy 0.5 (`D_A` 0.041–0.134, n=13)
and occupancy 1.5 (`D_A` 0.067, n=1). Two points, overlapping ranges, one of them a single
seed. No trend is determined, and the freeze pre-committed that UNDETERMINED is preferred to
a fitted line. `M-UNTESTED-GAP` forbids the extrapolation independently: the hypothesised
axis has no measured point within five orders of the band.

**What a successor carrier would need**, stated as a requirement and not as a prediction:
≥ 4 cells at ≥ 100 atoms per cell is ≥ 400 atoms, which is 33× this scene and past the
artifact's 16-atom cap; and the 2D force path takes `Route::Complete` unconditionally
(`cells.rs:490` — a flat scene gets `nc[2] = 1 < MIN_CELLS_PER_AXIS`), so the neighbour list
is rebuilt O(N²) every substep. A scale-up is a campaign with its own freeze, its own
conservation gates and a format v2, and it would still be nanometres.

**And the honest alternative, recorded so the results cannot invent it later.** A
continuum-native tier certified on its OWN dynamics — the FHP/REG+ chart in `holon-mesh`,
already mechanized in `CIRISOntology/Core/ModeChart.lean` — is a real object and a
reasonable node. **It is not a Closed view of the molecular dynamics and must never be
composed as though it were** (`M-FOREIGN-DOMAIN-CORROBORATION` at tier scale). Nothing in
this document discharges rung 2 through it.

---

## 7. WHAT THIS RUNG DOES NOT CLAIM

* It does not claim a µm fluid element is or is not Closed. G2 failed; the band was never
  measured.
* It does not claim the continuum chart is wrong. It claims that on this carrier, at the
  only cell size where the motion moves, the chart's defect is 0.048–0.891 against a budget
  of 0.02 — and that the momentum field's structure is real (+0.598 over control) while the
  density field's is not (−0.002, wrong sign).
* It does not claim closure anywhere. Leg A can only exhibit firing collisions or fail to
  find them, and where `D_A = 0` was read (the 1×1 chart) it is reported as the vacuity
  theorem, never as a certificate.
* It does not claim the 2D chart is the 3D one the cube face needs. The scene is
  two-dimensional and every sentence above says so.
