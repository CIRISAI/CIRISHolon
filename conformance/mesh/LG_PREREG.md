# NODE LG — THE LATTICE-GAS TIER, CERTIFIED ON ITS OWN DYNAMICS

*Frozen 2026-09-02, before any lattice-gas instrument existed in either tree. No
collision table, no streaming step, and no lattice had been written anywhere in
CIRISHolon or CIRISOntology when this file was written; the inventory that
established that is in §1 and the two modules that would have carried such code
say so in their own headers. The closed forms in §5 are DERIVED HERE, from
lattice geometry, before the Rust instrument exists — the pump-law pattern, and
the derivation scripts freeze in the same commit as this document so the git
history is the check.*

**misfits:** M-VOLUME-SCALE, M-HOMOG, M-PLANT-OBS, M-PLANT-SECTOR,
M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-NONBIJECTIVE-STEP,
M-NULL-MISSTAKE, M-BASE-RATE-OMITTED, M-ONE-MODEL-DELTA,
M-FINAL-VIEW-COLLISIONS, M-EXIT-DISCRIMINATOR, M-TAG-AS-PROPERTY,
M-PARITY-PROTECT, M-GAUGE-UNIFORM-MOMENTUM, M-DEVICE-CLASS,
M-PLACEMENT-LOTTERY, M-CHEAPER-THAN-ITS-PRICE, M-STALE-INSTRUMENT,
M-PROVENANCE-OVERREACH, M-PRESENTATION-VERDICT, M-BUDGET-LAUNDER.

**Why `conformance/mesh/` and not `conformance/water_observatory/`.** This is
not a water campaign and it must not inherit one's frame. The crate that already
owns the FHP-6 / FCHC-24 mode-set enumerator is `holon-mesh`, whose `fchc.rs` is
the instrument control this node reuses; the tier's charts are mesh charts and
its lattice is the mesh family's geometry. Filing it under the observatory would
put the fluid tier within the molecular campaign's own directory, which is the exact
category error the first law below forbids.

---

## 0. THE FIRST LAW

**This tier is not a view of the molecular dynamics and is never composed
through `closed_comp` as if it were.** It is its own object, with its own state
space, its own motion, and its own warrant. Adopted verbatim from rung 2's flag.

Three consequences, binding on every line below:

1. No gate in this node reads a molecular trajectory, a molecular chart, or a
   molecular certificate. Nothing here is a coarsening of anything.
   witness: `closed_comp` — named to be REFUSED, not used; it is the theorem
   this node must not invoke, and no instrument in this node calls it.
2. The **molecular-to-lattice seam is a separate claim and is NOT MINE.** It
   carries no status from this node. Rung 2's measured admissibility exit will
   inform it; nothing in `LG_RESULTS.md` may be read as evidence about it, and
   the results document will say so in its own opening.
3. The certificate this node can deliver is about the lattice gas alone. If the
   lattice gas certifies and the seam does not, the fluid tier is certified and
   the *bridge to molecules* is not, and those are different sentences.

---

## 1. WHAT EXISTED BEFORE THIS FREEZE

Inventoried 2026-09-02, both trees. **No dynamics.** The state space exists
twice; the motion exists nowhere.

| artifact | what it carries | what it does not |
|---|---|---|
| `CIRISOntology/Core/Lattice.lean` | 64 local states, `np` label, `sector_count = 53`, `sector_dims` 44/7/2, `three_route_sector = {9,18,36}` | the collision law. Its own header: REG+ collisions are "DEFINED as unitaries block-diagonal in these fibers — by construction, not discovery" |
| `ciris-sim-core/src/regplus.rs` | `DIRECTIONS[6]`, `sector()`, additive `GrossState`, `transition_preserves_sector()` | Header: "It does not invent a collision law: transitions are supplied elsewhere." Nothing supplies them. |
| `holon-mesh/src/fchc.rs` | the DP enumerator; FCHC-24 → 72,047 sectors; FHP-6 → 53 / 44 / 7 / 2 through the SAME routine as its control | Header scope: "It does not implement a 3D chart, a collision rule, or a 3D ledger." |
| `holon-mesh/src/{grid,state,mesh}.rs` | a rectangular face-stencil grid and a diffusive integer ledger halving, gated bit-identical against an unsharded reference | not a lattice gas. `GrossState` is a payload there, not a fluid. No hex geometry, no streaming, no collision. |

`ciris-sim-core/src/relativity.rs:56` already records the open decision — integer
FHP sector label versus continuum `f64 P^mu`, "until that decision is made
deliberately". **This node makes it, on the integer side, and does not touch the
SR ledger.**

Prior art, reused verbatim from `engine/MESH_DESIGN.md` §2.1 and
`fchc.rs`'s header, credited generously per house pattern:

* Frisch, Hasslacher & Pomeau, *Lattice-gas automata for the Navier–Stokes
  equation*, Phys. Rev. Lett. **56** (1986) 1505 — the hexagonal lattice and its
  fourth-order isotropy.
* d'Humières, Lallemand & Frisch, *Lattice gas models for 3D hydrodynamics*,
  Europhys. Lett. **2** (1986) 291 — FCHC-24.
* Hardy, Pomeau & de Pazzis (1973) — HPP-4, whose spurious per-line momentum
  invariant is used below as this node's positive control for a NON-vacuous
  exact closure, and which is historically why FHP exists.

**Ours is the classification, the exact defect law, and the certificate.**
Nothing in §5 is claimed as new lattice-gas physics; §5.1's classification and
§5.3's defect law are, as far as this node's search goes, not stated in that
form in the cited literature, and the results document will say "not found,
searched" rather than "first".

---

## 2. THE OBJECT

* **Local state** — `s ∈ {0,…,63}`, bit `d` = a particle moving in direction
  `d`, in `Core/Lattice.lean`'s axial integer coordinates
  `DIR = [(1,0),(0,1),(−1,1),(−1,0),(0,−1),(1,−1)]`.
* **Lattice** — an `L × L` axial torus; cell `c = (i,j) ∈ Z_L × Z_L`. Periodic
  in both axial directions. Micro-state `x ∈ {0,…,63}^(L²)`.
* **Collision `C`** — a permutation of the 64 local states that fixes the `np`
  label. §5.1 classifies every such map.
* **Streaming `S`** — the particle occupying direction `d` at cell `c` moves to
  cell `c + DIR[d]`, modulo `L`.
* **The motion `T = S ∘ C`** — collide, then stream. One application is one
  step, and it is the tier's own clock. There is no second clock, and no
  molecular clock anywhere in this node.

**Charts — the tier's own, one family with the two degenerate ends inside it.**
For `b | L`, `v_b` sends a micro-state to the field of block gross states:

```
block(c) = (⌊i/b⌋, ⌊j/b⌋)
v_b(x)[B] = Σ_{c ∈ B} (N(x_c), P_x(x_c), P_y(x_c))     — GrossState.combine
```

`v_1` is the per-cell sector field. `v_L` is the single global conserved label.
The whole certificate is read along `b`, which is why the door is
defect-against-view and not an aggregate.

**The axial ledger is exact; isotropy is Euclidean.** `P` is carried in axial
integers, where conservation is integer identity with no tolerance. Isotropy is
a metric statement and is read in the Euclidean embedding
`M = [[1, 1/2], [0, √3/2]]`, `det M = √3/2 ≠ 0`. Because `M` is linear and
invertible, conservation in axial coordinates and conservation in Euclidean
coordinates are the same statement — stated here so that the one float
computation in this node cannot be mistaken for a weakening of the ledger.

---

## 3. WHAT IS BEING CERTIFIED, AND WHAT IS NOT

**Certified here (integer-exact, zero tolerance, no epsilon anywhere):** that
the tier runs on its own dynamics; that its motion is a bijection; that each
conservation law holds separately as an integer identity; that the census
classifies the dynamics; and the closure reading along `b`.

**NOT claimed: the Navier–Stokes limit.** This node measures the *necessary*
lattice condition — fourth-rank isotropy of the direction set — and stops.
**Exit, named:** the sufficient conditions are a measured kinematic viscosity
against the model's own prediction, semi-detailed balance of the collision
table, and the `g(ρ) ≠ 1` Galilean defect. None is measured here. A later node
that wants the limit runs those three; until it does, no document in this
programme may say this tier has a Navier–Stokes limit.

**NOT claimed: the seam.** §0.2.

**NOT run here: FCHC-24.** Priced in §8, gated on FHP-6 banking first.

---

## 4. THE INSTRUMENT

`engine/crates/holon-lattice/` (new crate), plus a frozen Python reference.

**Two independent implementations, as `holon-mesh` already does it.** The
reference (`ref_lattice.py`, frozen in this commit) and the Rust instrument are
written from this document, not from each other, and G7 is their bit-identical
agreement. A closed form that only its own implementation reproduces is not a
closed form.

**Determinism.** The `±60°` choice in FHP-I-R is a counter hash of
`(i, j, step, seed)` — no hidden state, no sequential RNG, so a run is
reproducible bitwise and is independent of traversal order and of any sharding.

**Termination (M-EXIT-DISCRIMINATOR).** Every run records WHY it stopped:
`STEPS_COMPLETED`, `GATE_TRIPPED(name)`, or `BUDGET_EXHAUSTED`. A budget
exhaustion is VOID and is never scored (M-BUDGET-LAUNDER); it may not fall back
to a verdict.

**Work counts (M-VACUOUS-SUCCESS).** Every gate prints the number of checks it
performed alongside its verdict. A gate reporting PASS with a zero work count is
a FAILURE of that gate, not a pass. Specifically: the collision-fire count per
run (how many cells actually landed in an acting sector and were permuted), the
number of probes, the number of steps, and the number of cells visited.

---

## 5. THE DERIVED CLOSED FORMS

Derived in this freeze, from geometry, before the Rust exists. Scripts
`lattice_common.py`, `ref_census.py`, `ref_isotropy.py`, `ref_plants.py`,
`ref_defect.py`, `ref_defect_allmodels.py` freeze alongside this file, and each
prints the table it derives.

### 5.1 The census classifies the dynamics

A sector-preserving collision permutes within `np` fibers and can do nothing
else. `Core/Lattice.lean`'s `sector_dims` therefore states exactly where a
collision law may act: it is the identity on all **44** sectors of dimension 1, and
the full group of REG+ collision laws on FHP-6 is

> **S₃ × (S₂)⁷ × S₃, of order 4608.**

The two dimension-3 sectors are `(N=2, P=0) = {9,18,36}` — which is precisely
`Lattice.lean`'s `three_route_sector` — and its particle-hole dual
`(N=4, P=0) = {27,45,54}`. All seven dimension-2 sectors have `N = 3`; the one
FHP-I acts on is `(N=3, P=0) = {21,42}`.

FHP-I is one named element: the 3-cycle `9→18→36→9` and the swap `21↔42`.

### 5.2 Fourth-rank isotropy, exact, two-sided

`T⁴_αβγδ = Σ_i c_iα c_iβ c_iγ c_iδ` in the Euclidean embedding, against
`A(δδ + δδ + δδ)`:

| direction set | `T²` | `T⁴_xxxx` | `T⁴_xxyy` | `A` | `max|T⁴ − A·iso|` |
|---|---|---|---|---|---|
| FHP-6 hex | `3·δ` | 2.25 | 0.75 | 0.75 | ≤ 1e-14 (machine zero) |
| HPP-4 square | `2·δ` | 2.00 | 0.00 | 0.6667 | 0.6667 |

The instrument must reproduce both rows. **The failing row is the point**: a
tensor routine that only ever prints "isotropic" has not been shown able to
print anything else (M-TAG-AS-PROPERTY — the verdict must be computed from the
direction set, never from the model's name).

### 5.3 THE CLOSURE DEFECT IS THE BLOCK'S BOUNDARY FRACTION — exactly

For the fiber move of §6.2 at one step, the block chart `v_b` on an `L`-torus
has witness rate exactly

> **W(b) = 1 − max(0, b−2)² / b²  for b < L, and W(L) = 0.**

Derived, not fitted: `C` is a bijection fixing `np`, so `C(s) ≠ C(σ(s))` lie in
one fiber; `S` moves each particle exactly one cell; therefore `v_b(Tx)` and
`v_b(Ty)` can differ only if some particle crosses a block boundary, and a cell
whose six neighbours all lie in its own block cannot produce a difference. The
enumeration says the bound is **saturated**: every boundary-layer cell produces
a witness, for every movable state.

| `b` | 1 | 2 | 4 | 8 | 16 | 32 | 64 = L |
|---|---|---|---|---|---|---|---|
| `W(b)` | 1.0000000000 | 1.0000000000 | 0.7500000000 | 0.4375000000 | 0.2343750000 | 0.1210937500 | 0.0000000000 |

`W(b)` is independent of `L` (checked at `L` = 64, 128, 256: 0.4375 at `b = 8`
in all three). The `b = L` zero is **VACUOUS BY CONSERVATION** and the same
geometric formula says so: on a torus a single block has no inter-block edges,
so the boundary layer is empty. That is rung 2's flag arriving as arithmetic —
the one cell that closes by conservation alone.

### 5.4 The defect belongs to the LATTICE, not to the collision law

M-ONE-MODEL-DELTA earns "worse than the one model you chose" unless the
alternative is exhausted. It is exhausted here. **All 4608 sector-preserving
collision laws were enumerated and every one gives `W(8) = 0.4375` exactly** —
one distinct value over the whole group, identical to the geometric bound. The
identity collision is in that enumeration, so the reading survives the removal
of the collision entirely.

Stated for the results document, and staked now: the closure failure of the
coarse charts is a property of streaming on a lattice, and no choice of REG+
collision law removes it.

---

## 6. THE GATES

Each gate stands alone and takes down its own claim (rule 2). Criteria are
integer identities where the quantity is an integer; there is no tolerance
anywhere except G3, whose float nature is stated.

### 6.1 Dynamics and conservation

- **G1 — mass, alone.** Σ_cells N is integer-identical at every one of the
  20,000 steps of the reference run. Per-carrier invariance across steps, never
  equality between two different carriers (M-NULL-MISSTAKE). Criterion: EXACT.
  Work count printed. witness: none (the conserved label is
  `Core/Lattice.lean`'s `np`, which lives in CIRISOntology and cannot resolve in
  `lean/CIRISHolon`; this gate is engine-checked by exhaustion over the run)
- **G2 — momentum x, alone.** Σ_cells P_x integer-identical at every step.
  Criterion: EXACT. witness: none (same reason as G1)
- **G3 — momentum y, alone.** Σ_cells P_y integer-identical at every step.
  Criterion: EXACT. witness: none (same reason as G1)
- **G4 — the wall ledger.** In the obstacle configuration (§6.4), mass is EXACT
  and momentum equals initial momentum plus the accumulated bounce-back impulse,
  as an integer identity at every step. A channel present in the dynamics and
  absent from the ledger reads as unexplained loss; the impulse is a term, not a
  tolerance. Criterion: EXACT, 2 components accounted separately.
  witness: none (engine-checked; the wall impulse has no Lean statement)
- **G5 — bijectivity.** The 64→64 collision table is injective, checked
  exhaustively over all 64 states, for every collision law the node runs
  (M-NONBIJECTIVE-STEP). Streaming is a bijection on the torus, checked by
  round-tripping the full micro-state through `S` and `S⁻¹` on 1 lattice of
  4,096 cells. Criterion: EXACT, 64 states + 4096 cells.
  witness: none (engine-checked by exhaustion)

### 6.2 Closure — the census two-leg form, at the tier's own charts

**Leg A — HELD.** Which charts are exactly held under `T`?
- **G6 — the two-sided Leg-A gauge.** On FHP-6, the global chart `v_L` is HELD
  (that is G1–G3) and is LABELLED VACUOUS-BY-CONSERVATION, not counted as a
  result. On **HPP-4**, the per-line momentum chart — momentum summed along each
  row and each column separately, a chart `L` times finer than global — is
  HELD EXACTLY, and on **FHP-6 the same chart is NOT held**, with the witness
  step exhibited. Criterion: HPP-4 per-line drift EXACT zero over 2,000 steps;
  FHP-6 per-line drift nonzero within 100 steps. Both required.
  witness: `Held`
- Why G6 and not just G1–G3: a Leg A that has only ever returned "held for the
  globally conserved thing" has not been shown able to return a NON-vacuous
  held chart, nor to return "not held". HPP-4 supplies both, at one chart
  granularity, on one instrument. This is the instrument control that makes
  Leg A mean something (M-VACUOUS-SUCCESS).

**Leg B — CLOSED.** Fiber-invariance, probed **by construction, never by
trajectory coincidence.**
- The census's observed-fiber pairing cannot be used here: on a moving lattice
  gas the coarse view essentially never repeats between frames, so the pairing
  would return "no witness found" and that would be a vacuous pass.
  M-FIXED-POINT-TRAJECTORY says the same thing from the other side and instructs
  staking closure over configurations rather than over one orbit. So:
- **The fiber move.** Given `x`, pick a cell whose state lies in a sector of
  dimension ≥ 2 (there are 20 such states) and replace it by that sector's
  cyclic successor. `np` is unchanged, so `v_b(y) = v_b(x)` for EVERY `b`
  simultaneously — one identical perturbation serves the whole chart family, so
  no confound enters the defect curve from the probe changing with `b`.
- **G7 — the defect law.** The measured witness rate at `k = 1` equals §5.3's
  `W(b)` for `b ∈ {1,2,4,8,16,32,64}` at `L = 64`, and the Rust instrument
  agrees with the frozen Python reference **bitwise** on every entry.
  Criterion: EXACT agreement on all 7 entries, 2 implementations.
  witness: `closed_iff_fiber_invariant`
- **G8 — witness pairs exhibited, not argued.** For every `b < L` the results
  document carries at least 1 explicit witness pair — the two micro-states, the
  agreeing coarse view, and the two disagreeing stepped views — in the exact
  sense of the `¬Closed` equivalence. Criterion: ≥ 1 exhibited pair per `b`, 6
  values of `b`. witness: `nonfactoring_iff_not_closed`
- **G9 — the refined chart's OWN witnesses.** M-FINAL-VIEW-COLLISIONS: a coarser
  chart is not credited with restoring closure because it separates the finer
  chart's witnesses. Each `b` is probed independently with its own fiber moves
  and its own witness search; no `b`'s verdict is inferred from another's.
  Criterion: 7 independent probe sets, ≥ 4,096 probes each.
  witness: `Closed`
- **G10 — the two-sided probe gauge (M-BASE-RATE-OMITTED, M-PLANT-OBS).** The
  probe must be shown able to return both answers on the same instrument.
  Negative control: `y = x` (no perturbation) must give witness rate EXACTLY 0
  at every `b` — a probe that fires on nothing is measuring itself. Positive
  control: a perturbation to a state in a DIFFERENT sector (which changes `v_b`
  by construction) must give witness rate 1.000 at `b = 1`. Criterion: 0.000 and
  1.000, EXACT, ≥ 4,096 probes each. witness: `closed_iff_fiber_invariant`

### 6.3 Instrument control

- **G11 — the sector census.** The instrument's own enumerator returns 53
  sectors with dimension histogram 44 / 7 / 2 on FHP-6, reproducing
  `Core/Lattice.lean` and `regplus.rs` and `holon-mesh/fchc.rs`. The same routine
  returns the FCHC-24 numbers 16,777,216 / 72,047 / 11,740 unchanged. Criterion:
  EXACT, 4 numbers. witness: none (`sector_count`/`sector_dims` are
  CIRISOntology theorems and cannot resolve in `lean/CIRISHolon`; the engine
  reproduction is the check, and it is the pre-existing one)
- **G12 — isotropy, two-sided.** §5.2's two rows reproduced. FHP-6 residual
  ≤ 1e-12; HPP-4 residual ≥ 0.66. The verdict is computed from the direction set
  and never from the model's name (M-TAG-AS-PROPERTY): a presentation regression
  feeds the SAME six hex directions under a permuted ordering and under the
  axial-vs-Euclidean relabelling, and both must return bit-identical verdicts
  (M-PRESENTATION-VERDICT). Criterion: 2 rows + 2 re-presentations.
  witness: none (a metric statement about a direction set; no Lean statement
  exists in either tree)
- **G13 — the carrier moves (M-FIXED-POINT-TRAJECTORY).** The reference run's
  micro-state is not a fixed point: Hamming distance from step 0 exceeds 0.30·L²
  by step 100, the per-cell sector field changes on ≥ 0.10·L² cells per step,
  and the collision-fire count per step is ≥ 1 for all 20,000 steps. A run
  failing any of these VOIDs every closure reading taken on it. Criterion: 3
  counters, thresholds as stated.
  witness: none (a property of a particular run, engine-checked)

### 6.4 The inhomogeneity discharge

M-HOMOG: a periodic torus is spatially homogeneous, so a locality-shaped result
on it may hold for a homogeneity reason rather than a locality reason.

- **G14 — the defect law survives a structurally inhomogeneous graph.** A
  bounce-back wall segment of 32 cells is inserted, breaking translation
  invariance. `W(b)` is remeasured for cells whose block does not touch the
  wall, and must reproduce §5.3 exactly; blocks touching the wall are reported
  separately and are NOT averaged into the curve. Criterion: EXACT on
  wall-free blocks, 6 values of `b`; wall-adjacent blocks reported, not scored.
  witness: none (engine-checked)

### 6.5 The collision-law sweep

- **G15 — the defect is the lattice's.** All 4608 sector-preserving collision
  laws enumerated at `b = 8`; the number of DISTINCT witness rates is 1 and its
  value is 0.4375 (M-ONE-MODEL-DELTA). Criterion: EXACT, 4608 laws.
  witness: none (engine-checked by exhaustion over the group)

---

## 7. PLANTS

Every plant is re-derived for THIS instrument and pre-checked to fire
(M-PLANT-OBS). Each plant's carrier is state **9** — the head-on pair, which is
**nonzero in** the `(N=2, P=0)` sector the plant acts on, and whose population is
asserted `≥ 1` per step by G13's collision-fire counter before any plant is run
(M-PLANT-SECTOR). The three conservation plants are mutually isolating: each
moves exactly one conserved quantity, verified against the `np` table in this
freeze, so a plant that fires two gates is itself a finding.

| plant | edit | `np` before → after | must FIRE | must NOT fire |
|---|---|---|---|---|
| **P1 mass** | `C: 9 → 0` | `(2,0,0) → (0,0,0)` | G1 | G2, G3 |
| **P2 momentum-x** | `C: 9 → 34` | `(2,0,0) → (2,1,0)` | G2 | G1, G3 |
| **P3 momentum-y** | `C: 9 → 5` | `(2,0,0) → (2,0,1)` | G3 | G1, G2 |
| **P4 bijectivity** | `C: 18 → 9` (two states share an image) | label-preserving | G5 | G1, G2, G3 |
| **P5 census** | one direction vector set to `(1,1)` | — | G11 | — |
| **P6 isotropy** | FHP-6 run through the HPP-4 tensor path | — | G12 | — |
| **P7 wall ledger** | wall impulse dropped from the ledger, kept in the dynamics | — | G4 | G1 |
| **P8 probe** | fiber move replaced by a same-state no-op | — | G10 negative control stays 0; **G7 must FIRE** as a failure to reproduce `W(b)` | — |
| **P9 fixed point** | run seeded with a state `C` and `S` both fix | — | G13 | — |

P8 is the one that matters most: it is the plant against a closure result
produced by a probe that never perturbed anything, which is the shape
M-VACUOUS-SUCCESS names. P9 is the plant against the closure gate that is
vacuous because the carrier never moved.

**Every plant is run and its firing recorded before the unplanted verdict is
read.** A plant that does not fire VOIDs its gate.

---

## 8. PRICING

Unit: **cell-updates (cu)**, `1 cu` = one cell advanced one step. No wall clock
is used in any criterion, so scheduler placement on this heterogeneous core
class cannot move a verdict (M-PLACEMENT-LOTTERY); the prices below are the
falsifiable cost model (M-CHEAPER-THAN-ITS-PRICE), and a result arriving at less
than a tenth of its priced work is refused as not that result.

| stage | size | price |
|---|---|---|
| census + classification + bijectivity | 64 states, 4608 laws | < 1e6 cu-equivalent |
| isotropy, both lattices | exact | < 1e4 |
| reference run, FHP-6 | `L = 256`, 20,000 steps | 1.31e9 cu |
| HPP-4 spurious-invariant control | `L = 128`, 2,000 steps | 3.3e7 cu |
| closure probe, all `b`, `k ∈ {1,2,4,8,16}` | `L = 64`, 4,096 probes | ~9.1e8 cu |
| collision-law sweep, 4608 laws | `L = 16`, `k = 1` | ~1.5e8 cu |
| obstacle configuration | `L = 128`, 5,000 steps | 8.2e7 cu |
| **total** | | **≤ 1e10 cu, one CPU device class** |

Everything is integer arithmetic, so the artifact is bitwise identical across
device classes by construction rather than by measurement — but only CPU is run
here and only CPU is claimed (M-DEVICE-CLASS). The single float computation is
§5.2's isotropy tensor, which carries a stated tolerance and gates nothing else.

**FCHC-24, priced and NOT run in this freeze.** `2^24` local states, 72,047
sectors; the collision group is the product of the sector symmetric groups, far
past enumeration. A 3D run at `L = 64` for 10,000 steps is 2.6e9 cu with 24
lanes per cell — roughly 4× the 2D reference — and the ledger's momentum arity
must go from 2 to 4 first, which `fchc.rs`'s own momentum-arity flag names and this node does not
touch. **Gate: FCHC-24 does not start until FHP-6 banks.** The spurious fourth
momentum component is already named in `fchc.rs` and inherits that naming.

`L` scaling (M-VOLUME-SCALE): a single `L` would leave `W(b)`'s `L`-independence
untested and would let a block chart be confused with the global one. `W(b)` is
measured at `L ∈ {64, 128}` and every `b` in the curve satisfies `b ≤ L/4`,
except the deliberately included `b = L` endpoint, which is labelled vacuous.

---

## 9. VOID CONDITIONS

VOID is not KILL. A VOID says the instrument did not measure what it claimed.

- **V1** — any conservation gate fails on the UNPLANTED run: the instrument is
  wrong, the tier is not being measured, and no closure reading may be reported.
- **V2** — any plant fails to fire its gate: that gate is unarmed and its
  verdict is withdrawn.
- **V3** — measured `W(b)` EXCEEDS §5.3's geometric bound: information moved
  further than one cell in one step, which is an instrument defect (a streaming
  or indexing error), not a physics result.
- **V4** — G13's carrier-motion counters fail: every closure reading on that run
  is vacuous and is withdrawn.
- **V5** — the Rust and the frozen Python reference disagree anywhere: neither is
  reported until the disagreement is resolved and named.
- **V6** — budget exhaustion. Reported VOID and never scored.
- **V7** — G10's negative control returns anything but exactly 0: the probe fires
  on nothing and every rate it produced is meaningless.

---

## 10. KILLS — separable, each taking down its own claim and nothing beneath it

- **K1 — the classification.** *Claim:* the space of REG+ collision laws on FHP-6
  is exactly `S₃ × (S₂)⁷ × S₃`, order 4608, acting only on the 9 sectors of dimension above 1
  sectors. *Kill:* exhibit a mass- and momentum-conserving local collision law on
  the 6-direction state space outside that group. Takes down §5.1 alone.
- **K2 — the defect law.** *Claim:* `W(b) = 1 − max(0,b−2)²/b²`, exactly, at
  `k = 1`. *Kill:* a measured `W(b)` differing from that value at any `b`, on an
  instrument passing G1–G5 and G10. Takes down §5.3 and the door's shape; leaves
  the conservation certificate standing.
- **K3 — the law's lattice-locality.** *Claim:* no REG+ collision law changes
  `W`. *Kill:* one of the 4608 laws giving a different `W(8)`. Takes down §5.4;
  leaves K2's statement about FHP-I standing, downgraded to one model.
- **K4 — the vacuity reading.** *Claim:* the only exactly-closed chart in the
  family is `v_L`, and it closes by conservation alone. *Kill:* a `b < L` with
  witness rate exactly 0 over ≥ 4,096 probes at `k = 1` AND at `k = b`. Takes
  down §5.3's reading; would be a real closed fluid chart and the node's most
  interesting possible failure.
- **K5 — the isotropy warrant.** *Claim:* FHP-6's fourth-rank tensor is
  isotropic and HPP-4's is not. *Kill:* either row failing. Takes down
  `engine/MESH_DESIGN.md` §2.1's warrant as measured here; takes down nothing else, because
  no claim in this node depends on the Navier–Stokes limit.
- **K6 — the Leg-A gauge.** *Claim:* HPP-4 carries an exactly-held non-global
  chart that FHP-6 does not. *Kill:* HPP-4's per-line momentum drifting, or
  FHP-6's holding. Takes down G6's status as a control; the closure curve is
  then reported with its Leg-A gauge missing, and said so.

---

## 11. THE MEANING OF EVERY POSSIBLE ANSWER

Written before the instrument, so no reading can be re-interpreted after.

1. **Conservation exact + `W(b)` as derived** (the expected outcome). The tier is
   certified as its own object: exact integer conservation on its own dynamics,
   a bijective motion, a census that classifies the law — and its coarse charts
   are **NOT closed views**, with the defect an exact boundary fraction and the
   only closed chart the vacuous conserved one. **This is a certificate with an
   honest negative in it**, and the negative is the content: hydrodynamics on
   this tier is a measured approximation, not a closed view. The workbench band
   must say that, not "certified closed".
2. **Conservation exact + `W(b)` differs.** K2 fires. The conservation
   certificate stands alone; the door loses its shape and the results document
   reports the measured curve with the derivation marked wrong, in the same
   sentence as the survival.
3. **Some `b < L` closes** (K4). The most interesting failure available: a
   genuinely closed non-vacuous fluid chart. It would be checked against the
   HPP-4 spurious-invariant pattern first, because a chart that closes for a
   spurious-invariant reason is a defect of the model, not a tier.
4. **A conservation gate fails unplanted.** V1. Nothing is reported.
5. **The classification is wrong** (K1). §5.1 falls and G11's control is
   unaffected; the census is still the census, it just would not classify.

In no branch does this node say anything about the molecular-to-lattice seam.

---

## 12. THE LIVE READOUT FOR THE WORKBENCH

Named here, before the page exists, so the page cannot shape the claim.

> **DOOR SHAPE: DEFECT-AGAINST-VIEW.**

The readout is `W(b)` against `b`: the closure defect as a function of
coarse-graining scale, with the derived closed form drawn as the curve and the
measured points on it.

Three requirements on whatever `workbench-engine` builds, which are part of this
freeze and not of the page's design freedom:

1. **The `b = L` point is drawn as the VACUOUS end of the axis, labelled as
   closing by conservation alone** — never as the curve's success. It is rung 2's
   flag and the page must carry it as such.
2. **No aggregate.** A single scalar "defect" collapses the only axis this tier's
   reading has and would let `b = L`'s exact zero be averaged into a pass.
3. **The band text says "not closed at any intermediate chart"** if outcome 1
   lands. The band may not say the fluid tier is certified closed under any
   phrasing, and it may say nothing at all about the molecular seam.

Fence/flip wiring belongs to `workbench-engine`; the certificate and the honest
seam statement are this node's deliverable and the band flips on the resolving
certificate, not on this document.

---

## 13. BANKING

`LG_RESULTS.md`, verdict first, carrying: the instrument's commit
(M-STALE-INSTRUMENT — a results document without it is not banked); the sha256
of the reference run's final micro-state and of the derivation scripts, naming
only what was measured and nothing inferred beside it (M-PROVENANCE-OVERREACH);
every gate's verdict WITH its work count; every plant's firing; every VOID; and
the fired kills as plainly as the survivals. Run-state markers stay untracked;
cited logs are committed. Nothing is pushed from this node.

---

## APPENDIX — POST-FREEZE ANNOTATIONS

**Nothing in this appendix moves a gate, a stake, a criterion, a VOID condition or a kill.**
It corrects two pointers, records one scope correction from the operator, and records one
addition the lead asked for. Each is dated. An annotation must never weaken a gate, and none
of these touches one; the frozen body above is unchanged except where noted in A1, which
changes only file paths.

### A1 — path correction, 2026-09-02

`MESH_DESIGN.md` exists in **both** trees. Every citation in this document and in the
instrument now names the LOCAL copy, `engine/MESH_DESIGN.md` (§2.1; FHP prior art at ~line
101, FCHC-24 at ~line 105). The previous text cited
`/home/emoore/CIRISOntology/sim_engine/MESH_DESIGN.md`, which is the sibling repository's
twin — identical content, but not a tracked file here, so it fails this tree's citation gate.
**Content identical, path only.** Caught by `workbench-engine`'s citation gate.

### A2 — corroboration for the chart choice, 2026-09-02

`RUNG1_RESULTS.md` (merged) measured that their **geometric-predicate** charts — H-bond
networks built from distance and angle criteria — factored through **nothing**, their Leg F.
This node's chart's fibers are conserved-label classes, closed by construction for
sector-preserving collisions. Their stated lesson, written to this node by name: a chart
meant to compose into a conserved-label fiber census should be **built from conserved labels,
not from geometry.**

Recorded here so the `(N,P)` chart choice does not read as taste. **It is corroboration, not
a stake**: it was measured by an independent instrument that tried the other kind of chart and
watched it fail, and it arrived after this freeze. Nothing in §6 depends on it, and §11's
meanings are unchanged. Rung 1 also confirms that nothing composes upward into this node —
they certified nothing — so §0.2's standalone framing is untouched.

### A3 — scope correction from the operator, 2026-09-02

**This node's certificate confers NO workbench band state.** The band-flip law is restated in
the FSD (`b374773`): a band goes live only on a **node-G closure certificate**, a certified
coarse view of the dynamics beneath it. This tier is certified on **its own** dynamics, which
is a different thing, and running it under a band would be running physics that is not the
certified coarse truth of that scene — the fake the FSD bans, and what §0's first law already
forbade from the other direction.

§12's door requirements stand exactly as written, now as requirements on **research content
the page may cite**, never on a band state. The earlier framing that upper bands could go live
running this tier's physics is retracted, by the lead, before it reached any artifact.

### A4 — the invariant question, raised by the lead, answered by measurement, 2026-09-02

The lead suggested staking, in advance, that the **staggered (Zanetti) momentum invariants**
would appear as extra closed views at a staggered chart — rule-6 material if confirmed — and
said to verify the literature before staking it.

**Neither staked nor dismissed: solved.** Staking a half-remembered formula would have made
the answer depend on the recall. Instead `ref_invariants.py` solves for the **complete space
of linear invariants** of `T = S ∘ C`, so that any spurious invariant of any form must appear
in the dimension count. The derivation and the two gauges are in that file's header; the
reading is in `LG_RESULTS.md` §8.

This is **post-freeze and gates nothing.** It bears on the results document's wording — a
spurious invariant would be an extra exactly-closed view, and §11's outcome 1 says "the only
exactly-closed chart is the global one" — so it is reported as a measurement in its own right
with its own gauges, and the wording in the results follows what it measured.

**The literature check the lead asked for could not be completed**: this session's web-search
budget was exhausted before the Zanetti citation could be read, and the one page reachable by
direct fetch does not discuss spurious invariants. The measurement below is therefore stated
as a fact about **this configuration**, gauged on two systems whose invariant spaces are known
independently, and it is NOT stated as agreeing or disagreeing with Zanetti, whose exact
statement and scope this node has not read. That check is **owed**, and named here as owed
rather than quietly dropped.

### A5 — the ansatz scope of A4's solve, 2026-09-02

The lead asked, correctly, which space `ref_invariants.py` searched — because "zero spurious"
means different things over a site-dependent space and over a translation-invariant one, and
the difference is *we looked and they are absent* versus *we looked where they could not be.*

**The search is over the FULL site-dependent space: one free weight `w[c][d]` per cell and
direction, all `6L²` of them, with nothing assumed translation-invariant.** The collapse to
one weight per (direction, line) is a **derived consequence of the dynamics, not a restriction
on the search**: put a single particle at cell `c₀` in direction `d₀` and conservation reads
`w[c₀+DIR[d₀]][d₀] = w[c₀][d₀]` immediately, because single-particle states are alone in their
`(N,P)` fiber and every sector-preserving collision fixes them.

A staggered momentum is exactly a **position-dependent** weight, and it survives that collapse
whenever its sign pattern is constant along each direction's own lines — which is how HPP's
per-line momenta appear. So such functionals are **inside** the searched space, and
`ref_ansatz_scope.py` measures how much of each system's invariant space is genuinely
position-dependent, basis-independently, by re-solving with the weights forced flat:

| system | `dim` | `dim_TI` | **position-dependent** |
|---|---:|---:|---:|
| identity collision (streaming), `L` = 4, 6, 8 | 24, 36, 48 | 6 | **18, 30, 42** |
| HPP-4, `L` = 4, 6, 8, 12 | 9, 13, 17, 25 | 3 | **6, 10, 14, 22** |
| **FHP-I**, `L` = 4, 6, 8, 10, 12, 16 | 3 | 3 | **0** |

So the solver finds position-dependent invariants in quantity when they exist — `2L−2` of them
on HPP-4, which is the staggered shape and the historical reason FHP exists — and for FHP-I
that sector is **empty**. `A4`'s reading is therefore *looked and absent*, over the space where
staggered invariants live.

**This does not discharge the literature check**, which stays owed at A4 and is now owed
session-wide (the web-search budget is a shared pool and was exhausted). What it does is fix
what the check will adjudicate: no longer whether the measurement was aimed at the right
space, but whether Zanetti's scope — rest particles, boundary conditions, `L` parity, model
variant — differs from this configuration's.
