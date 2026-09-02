# NODE_E_PREREG — ring-polymer dynamics coupled into the engine's own `Sim`

**Frozen 2026-09-02.** Committed ALONE, before `RingSim` or any part of it exists. Nothing
in this document was written with a reading in hand: the four staked scenes are existing
conformance fixtures, the only numbers below are either EXACT, derived here from
arithmetic already in the tree, or counted integers, and every criterion that could have
been fitted is instead stated as a closed form with its derivation beside it.

**Node:** GANTT.md row E — "NQE in dynamics: ring-polymer propagation coupled into Sim, not
merely the C1 carrier". Receipt the graph stakes: *per-law gates under RPMD; P=1
bit-identical to classical Sim (the C1 pattern, in-engine)*.

**misfits:** M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-PLANT-OBS, M-PLANT-SECTOR,
M-DEVICE-CLASS, M-PLACEMENT-LOTTERY, M-CHEAPER-THAN-ITS-PRICE, M-STALE-INSTRUMENT,
M-NONBIJECTIVE-STEP, M-UNTESTED-GAP, M-VOLUME-SCALE, M-IDLE-CALIBRATED-TIMEOUT,
M-PROVENANCE-OVERREACH, M-EXIT-DISCRIMINATOR.

---

## 0. What this campaign claims, and what it refuses to claim

### 0.1 The physics honesty line, stated before any gate

Ring-polymer molecular dynamics is **approximate quantum dynamics**, and the approximation
is not a detail to be discovered later. Written down here so no reading of this campaign
can be quoted past it:

**RPMD licenses.** Exact quantum *statistics* in the `P → ∞` limit: the ring-polymer
configurational distribution is the exact quantum Boltzmann distribution, so equilibrium
and structural properties — zero-point-broadened bond length distributions, isotope
effects on structure and on free energies, quantum contributions to a radial distribution
function — are RPMD's own sector and it is a strong instrument there. Short-time
dynamics and rate constants at the level RPMD is known good for: it is exact in the
classical limit, exact in the harmonic limit for the position autocorrelation function's
short-time behaviour, and it conserves the exact quantum Boltzmann distribution.

**RPMD does NOT license.** Real-time quantum coherence. There is no interference in a ring
polymer; it is classical dynamics in an extended phase space. Anything that lives in a
phase relationship — coherent tunnelling splittings, recurrences, zero-point energy
leakage between modes over long times, spectral lineshapes in the deep quantum regime — is
outside it, and this campaign will not report such a quantity even if the code emits one.
RPMD also has the well-known **spurious-resonance** pathology: internal ring modes at
`ω_k` can beat against physical frequencies and put fictitious peaks in a spectrum.

**The fence this campaign therefore carries, with its own exit.** `E-1: RPMD-REAL-TIME` —
no claim about real-time quantum coherence is admissible from this instrument, at any bead
count. Class: PHYSICS-HONESTY. Owner: this node. Exit: a genuinely coherent carrier, which
is `tower.rs`'s C2 rung (`C2_MpsTdvp`), already fenced in FENCES.md as P11 — the exit is
not more beads and no amount of compute reaches it. Node E does not lift P11 and must not
be read as lifting it.

### 0.2 What the node is, structurally

C1 is banked (`conformance/water_observatory/C1_GATE_{PREREG,RESULTS}.md`): the ring-polymer
physics is real, hits an exact spectral referee to −0.0805% on the engine's own H–H curve,
and reproduces the classical trajectory bit for bit at one bead in
`holon-chem/src/rpmd.rs`'s own 3D machinery. **None of that is re-litigated here and none
of it is re-claimed.** C1 proved the physics on `tower.rs`'s carrier types. Node E is an
INTEGRATION node: it asks whether the engine that actually simulates scenes — `Sim`, with
its pair bank, its three- and four-body surfaces, its cell decomposition, its periodic
boundary, its energy, momentum and angular ledgers — can be driven as a ring polymer
without any of that machinery being forked, weakened, or silently bypassed.

So the deliverable is one new type in `holon-render`, `RingSim`, and the gates below.
`holon-chem` cannot host it (`holon-render` depends on `holon-chem`, not the reverse), and
`RingSim` will import `holon_chem::rpmd::{NormalModes, FreeRingPropagator}` **unmodified**.
That is a deliberate constraint with two reasons: `rpmd.rs` states in its own header that a
second copy of the propagator is a second place for it to be wrong, and any edit to it
would move C1's banked instrument (M-STALE-INSTRUMENT). If this campaign finds a defect in
`rpmd.rs` it will report it and stop, not patch it under a different node's name.

---

## 1. The design, fixed here so the gates cannot be fitted to it

### 1.1 One `Sim`, P bead states

`Sim` is not `Clone` (it owns `Option<Box<dyn ForceExecutor + Send + Sync>>`), so a
P-replica-of-`Sim` design is unavailable and is not wanted anyway: P copies of the pair
bank and the three-body surfaces would be P copies of a thing that must be identical.
`RingSim` holds ONE `Sim` plus `P` bead coordinate sets and `P` cached force sets, and
drives the physical force machinery one bead at a time by writing that bead's coordinates
into the `Sim` and calling the `Sim`'s own `compute_forces`.

The consequence, which is a feature: **every physical term the engine has — pair bank,
`trimer`, `water`, `ooh`, `ozone`, the shipped `trimers` bank, walls, gravity, the far
sector — acts on the ring for free and by construction, with no term needing to know that
beads exist.** No force code is duplicated and no force code is modified.

### 1.2 The step

`Sim::step` is velocity Verlet in three passes — half kick with cached forces (accumulating
external impulse), drift, THE WRAP, `compute_forces`, half kick — and it will be factored
into exactly those pieces with `Sim::step` itself becoming their caller, so there is one
statement of each. `RingSim::step` is then:

1. half kick every bead from its cached forces, accumulating external impulse per bead;
2. **exact free-ring evolution** of all beads in normal modes (`FreeRingPropagator`),
   which at `P = 1` is arithmetically the drift and nothing else;
3. THE WRAP, by the rule in §1.3;
4. for each bead: write it into the `Sim`, `compute_forces`, cache the result;
5. half kick every bead from the new cached forces, accumulating external impulse;
6. close the ledgers.

The free propagator is written on VELOCITIES, never momenta, because `rpmd.rs` records
that the momentum form's multiply-then-divide by the mass is not the identity in `f64` and
cost the `P = 1` gate its exactness once already.

### 1.3 THE WRAP RULE — the one real physics decision, and why it is exactly reducible

`Sim::step` folds every atom back into the box on the drift. Folding **beads**
independently would tear the ring: two beads either side of a face would sit a box length
apart and the spring term would read that as a real displacement, which is a physics bug
that no conservation gate is guaranteed to catch because it can be energy-conserving in
the torn configuration.

The rule adopted, stated before the instrument exists:

> **A ring translates as one object, by whole box vectors, driven by its centroid.**
> Per atom and per axis, compute the centroid `c` of that atom's beads. Then run
> `BoxGeom::wrap1`'s own subtraction on `c` — `while c >= L { c -= L; … }`,
> `while c < 0 { c += L; … }` — and apply **the same subtraction, in the same order, to
> every bead of that atom** within the same iteration.

Two properties, both consequences and neither of them tuned:

**(a) It is bit-exact at `P = 1`, honestly.** At one bead the centroid is the bead (the sum
is initialized to bead 0 and no further term is added, and the division is by `1.0`), so
the rule performs literally the same sequence of `x -= L` subtractions `wrap1` performs on
the same value. The identity is not a branch on `P` and there is no `if P == 1` anywhere in
the ring path; it is the same arithmetic reaching the same place.

**(b) It is right above `P = 1`.** Every bead of an atom moves by the identical integer
number of box vectors, so no intra-ring separation changes and no minimum-image pair
separation changes — `BoxGeom::minimum_image` already reduces every pair displacement, so
beads sitting up to a radius of gyration outside the box are correctly interacted.

The cost is that bead coordinates can leave `[0, L)` by up to `R_g`. `cells.rs::cell_of`
takes `rem_euclid` of the cell index in a periodic box and clamps in an open one, so it
tolerates that — but tolerating is not the same as being right, and **B4 below gates it
rather than assuming it.**

### 1.4 What the ring path REFUSES, by name

- **The thermostat.** RPMD dynamics is Hamiltonian; a thermostat on the dynamics is a
  different claim (that is PIMD, which samples and does not propagate). `rpmd.rs` carries
  the same fence, and a PILE thermostat exists there for the sampling case.
  Exit: a declared PIMD sampling mode with its own freeze, which this node does not build.
- **The barostat.** NPT in this engine is the MTK Trotter factorization on an extended
  Hamiltonian whose box is a degree of freedom; composing that with a ring polymer is a
  third integrator and not a flag. Exit: a freeze of its own.
- **A grabbed atom (the user's hand).** The hand acts on one atom; on a ring it is
  ambiguous whether it acts on the centroid or on every bead, and the two differ. Exit:
  declare which, with the work column that follows from the choice.
- **A mixed executor class.** Both arms of any identity gate run on `SerialExecutor`, the
  reference. A bit-identity gate whose arms could have run on different devices is not a
  bit-identity gate (M-DEVICE-CLASS): SATURATION-3 measured two correct answers differing
  bitwise on 91.0% of entries.

These refusals are not hedges; they are gated. See **B5**.

### 1.5 The admissibility condition, derived not tuned

The scheme is a Strang splitting of `H = H_free + V_phys`. The free part is integrated
EXACTLY, so the ring's stiff modes never limit the step the way they would under Verlet —
that is the whole reason a large bead count is affordable. But the splitting is not
unconditionally stable: the free rotation advances internal mode `k` by phase `ω_k dt` per
step, with

```
ω_k = 2 (P / (β ħ)) sin(k π / P),    ω_max = ω_{P/2} ≈ 2P / (β ħ)
```

and the physical kick resonates with a mode whose phase advance approaches `π`. This is a
KNOWN pathology of the exact-free-ring scheme, not a discovery of ours (the standard
treatment is the Cayley-modified propagator of Korol, Bou-Rabee and Miller, JCP 151,
124103 (2019)).

**Therefore:** `RingSim` computes `ω_max · dt` at construction and **REFUSES** any
configuration with

```
ω_max · dt  ≥  1
```

naming the quantity, the bead count and the temperature in the refusal. The bar is `1` and
not `π`: `π` is where the map becomes unstable, and running up to the edge of instability
is not a regime, it is a dare. `1` is a round number chosen for being conservative and is
declared as a CHOICE, not derived — a configuration refused at `0.9 ≤ ω_max dt < π` is
refused by our conservatism and not by arithmetic, which is why the refusal message prints
the value. Exit for the fence: the Cayley propagator, which removes the resonance and would
let the bar move; this node does not build it.

`dt` in this engine is derived from the curve by `Timescale`, so the condition binds `P`
and `T` jointly against a step nobody sets by hand. **This is a real constraint on how far
node E can be pushed and it is stated before any bead count is chosen, not after a run
misbehaves.**

### 1.6 P and T are part of the artifact's identity

A ring-polymer reading is not a classical reading with a setting attached: `β` enters the
spring frequency, so the *arithmetic* is a function of `(P, T)` the way a table's trailing
bits are a function of its device class (M-DEVICE-CLASS) and a timing is a function of its
core class (M-PLACEMENT-LOTTERY). **`P` and `T` join the regime identity**: every reading
this campaign emits carries both, a checkpoint taken under one `(P, T)` is not the same
artifact as one taken under another, and **B3** gates that the engine agrees.

---

## 2. The staked scenes

Four, fixed here. Two are existing conformance fixtures carried in VERBATIM, byte-compared
against their originals by a gate (the `tests/protocol_identity.rs` pattern) so the copy
cannot drift from the thing it stands in for — M-STALE-INSTRUMENT's discipline, applied to
a scene rather than to a runner. Two are specified completely below.

- **S1 — `staked_nve`**, verbatim from `engine/crates/holon-render/tests/ledger.rs`. 2D,
  `Boundary::Open`, 2 hydrogens, a bound vibrating pair at `R = 2.2` bohr with the centre
  of mass drifting at `(0, 0.001)`. Chosen because it is the fixture the classical energy
  and momentum gates are already staked on, and because a 2D scene holds `vz` at an exact
  zero, which is where the signed-zero hazard of §3.2 lives if it lives anywhere.
- **S2 — `staked_nve_3d`**, verbatim from
  `engine/crates/holon-render/tests/three_dimensions.rs`. The same pair rotated out of
  plane so every coordinate, every velocity component and the angular momentum vector
  carry all three components. This is the scene the ANGULAR law is gated on, because it is
  the one with a nonzero angular momentum to conserve.
- **S3 — walls-3.** `Boundary::Walls`, `Dims::Three`, `reset(3)`, positions
  `(cx − 1.4, cy, cz)`, `(cx + 1.4, cy, cz)`, `(cx, cy + 3.0, cz)` and velocities
  `(0.004, 0, 0)`, `(−0.004, 0, 0)`, `(0, −0.004, 0)` at the box centre, then `rebase()`.
  No grab: the hand is refused by §1.4, so the scene that exercises EXTERNAL work is the
  walled one. Chosen because the wall term is external, so `w_ext` and `j_ext` are nonzero
  and the identity gate compares LEDGERS that are actually carrying something rather than
  three zeros. On S3 the angular row is VOID BY CONSTRUCTION — walls break rotational
  symmetry and `Sim::angular_gate` returns `None` there — and that is a stated expectation
  recorded in advance, not a failure found later.
- **S4 — periodic-27.** `Boundary::Periodic`, `Dims::Three`, a `3 × 3 × 3` cubic lattice at
  spacing `3.0` bohr (box `9 × 9 × 9`), built exactly as `tests/t3_scale.rs::lattice`
  builds one — and then given velocities, because **a symmetric lattice is a force-balanced
  fixed point of the dynamics and a conservation gate on a scene that does not move is
  vacuous** (M-FIXED-POINT-TRAJECTORY, M-VACUOUS-SUCCESS). The velocities are deterministic
  and RNG-free: atom `i` gets
  `v = 0.002 · (sin(i + 1), sin(2i + 1), sin(3i + 1))` in atomic units. Chosen because it is
  the only scene that exercises THE WRAP, the cell decomposition and the minimum image, and
  therefore the only one on which the §1.3 rule and B4 mean anything.
  **This campaign makes NO claim about how anything here scales with `N` or with box
  volume** (M-VOLUME-SCALE): S4 is one fixed 27-atom box, and a reading taken on it is a
  reading on it.

**Anti-vacuity, required of every scene before any row on it is graded** (M-VACUOUS-SUCCESS,
M-FIXED-POINT-TRAJECTORY):

- **S0** Each scene's classical arm must move: total path length summed over atoms
  `≥ 1.0` bohr over the graded run, and the checkpoint must be `≥ 200` bytes. S4
  additionally requires `≥ 1` face crossing, counted, or its wrap rows are VOID rather
  than passed. Reported as counts, per scene, whatever the verdict.

---

## 3. THE BOTTOM GATE — `P = 1` bit-identity

### 3.1 The gate

- **G0 (EXACT).** For each staked scene `S ∈ {S1, S2, S3, S4}`, run the classical arm
  `Sim::step_frame(64)` for 156 frames and the ring arm `RingSim::step_frame(64)` at
  `P = 1` for 156 frames from the identical initial condition, both on `SerialExecutor`,
  and compare `Checkpoint::of(...)` **byte for byte**. Criterion: EXACT — every byte
  equal, on all four scenes, and `steps`, `time` and `drift_peak` equal as bits alongside.
  A failure reports the first differing byte offset and the field it lands in.
  witness: `carry_path_independent`

  The checkpoint is the right identity object rather than a positions-only digest: it
  carries `w_ext`, the work columns, `l0`, `p0`, `j_ext` and `time` as well as every atom's
  six coordinates, so a ring path that got the IMPULSE BOOKKEEPING wrong fails G0 even if
  every position matched. 156 frames × 64 substeps is the classical NVE gate's own staked
  run length, taken rather than invented.

- **G0b (EXACT).** The ring path contains no branch on `P`. Criterion: a source gate over
  the ring module refuses any `if p == 1`, `if self.p == 1`, `match p { 1 =>` or
  equivalent special case, and the gate asserts the number of files and llines it scanned
  so it cannot pass by scanning nothing (M-VACUOUS-SUCCESS). Without this, G0 could be
  passed by a path that simply calls the classical stepper at one bead, which would prove
  nothing about the arithmetic.
  witness: none (a source property, mechanically checked, not a theorem)

### 3.2 The pre-registered branch on signed zero — decided now, not after the reading

`NormalModes::to_modes` at `P = 1` computes `s = 0.0; s += 1.0 * x[0]`. In IEEE
round-to-nearest `0.0 + (−0.0) = +0.0`, so a coordinate or velocity component that is
NEGATIVE ZERO would come back POSITIVE ZERO and change a checkpoint byte while changing no
physics. The branches, fixed in advance:

- **(a)** Every byte equal → G0 passes as stated.
- **(b)** The ONLY differing bytes are the sign bit of `f64` fields whose value is zero →
  **G0 FIRES**, and stays fired. Bit-identity is bit-identity, and this freeze will not
  redefine the gate around the first thing that fails it. The claim is then published in
  its weakened form — identical except on signed zeros, with the mechanism named — and the
  weakened form is what goes in the results, marked as a fired clause. `rpmd.rs` is NOT
  edited to fix it (that would move C1's banked instrument); the finding is reported to
  C1's owner.
- **(c)** Bytes differ anywhere else → **K1 fires** (§6).

Because branch (b) is cheap to anticipate, the campaign will additionally COUNT, on the
classical arm alone and before the ring path exists, how many negative-zero coordinate and
velocity components each staked scene produces over its graded run. That count is a
scouting measurement on an existing instrument, it is reported whatever it says, and it
cannot change any criterion above.

### 3.3 The plants for G0 — each must be shown to FIRE before it arms anything

M-PLANT-OBS: a plant is re-derived for THIS instrument and PRE-CHECKED to fire; a plant
that does not fire disarms the gate it was supposed to arm rather than passing it.
M-PLANT-SECTOR: each plant below names **the sector the plant acts on**, and the carrier
must be nonzero in that sector or the plant is VOID.

- **B1 (EXACT).** *The mass round trip.* Rewrite the free propagator's velocity update as
  a momentum update — multiply by the mass, propagate, divide back — which is the exact
  defect that cost `rpmd.rs`'s classical-limit gate its exactness once. **Sector: the
  mass division.** Carrier requirement: at least one atom with `mass ≠ 1` in electron-mass
  units, which every scene here satisfies (hydrogen is `M_H`), and the requirement is
  asserted rather than assumed. Must change at least one checkpoint byte on `≥ 3` of the 4
  scenes.
- **B2 (EXACT).** *One unit in the last place.* Perturb a single velocity component of a
  single atom of a single bead by one ULP at step 1. **Sector: the velocity of that atom
  on that axis.** Carrier requirement: the chosen component must be NONZERO at the moment
  of the plant — a one-ULP perturbation of an exact zero is a denormal and is a different
  and much weaker plant — and the instrument asserts nonzero-ness before planting, on
  every scene. Must change at least one checkpoint byte on all 4 scenes.
- **B3 (EXACT).** *The bead count is not a decoration.* Run the ring arm at `P = 2` against
  `P = 1` on all four scenes. **Sector: the internal ring modes**, which are identically
  absent at one bead and present at two. Carrier requirement: `ω_1 > 0` at `P = 2`, which
  the freeze asserts arithmetically. The checkpoints must DIFFER on all 4 scenes, and the
  centroid must separate from the `P = 1` trajectory by `≥ 1e-3` bohr on at least one
  scene — C1's own two-bead control separated by 0.234 bohr, so `1e-3` is three orders
  inside a measured precedent and is a floor, not a prediction. A `P` that changed no byte
  would be a `P` the engine was ignoring, which is exactly how a run comes to mean
  something other than its label.

---

## 4. THE PER-LAW GATES UNDER RPMD

One gate per conservation law, never combined, each against a SEPARATELY DERIVED bound,
each with its own plant, each falsifiable without touching the others. The bounds are not
new: they are `Sim`'s own derived bounds with the sum extended over beads, and each
reduces exactly to `Sim`'s at `P = 1`.

The conserved quantity is the ring-polymer Hamiltonian

```
H_P = Σ_k [ V_phys(x_k) + Σ_i ½ m_i |v_{k,i}|² ] + Σ_k Σ_i ½ m_i ω_P² |x_{k,i} − x_{k+1,i}|²
```

with `ω_P = P / (β ħ)`, the physical term being whatever the `Sim` has loaded.

- **G1 — ENERGY (derived).** Measure `|H_P(t) − H_P(0)|` at every substep and take the
  peak. Criterion: `≤ DRIFT_SAFETY · 0.25 · ω_phys² · dt² · Σ_k e_ref,k`, which is
  `Sim::drift_bound`'s closed form with the reference energy summed over beads. Two things
  are fixed here and neither is negotiable later. **(i)** `ω_phys` is the PHYSICAL
  stiffness envelope only — the ring spring is NOT folded into it, and the reason is that
  the free ring is propagated exactly rather than by Verlet, so it contributes no `O(dt²)`
  Verlet drift. **(ii)** the reference energy is summed over beads rather than taken as the
  whole of `H_P`, and that direction is deliberate: taking the whole of `H_P` would let the
  huge spring energy inflate the budget and make the gate EASIER at larger `P`, which is a
  bound that loosens along the axis being tested. Graded at `P ∈ {1, 2, 4, 8}` on S1–S4,
  subject to §1.5.
  witness: `eval_total`
- **G2 — MOMENTUM (derived).** Measure `|P_ring(t) − P_ring(0) − J_ext(t)|` peak, where
  `P_ring = Σ_k Σ_i m_i v_{k,i}`. Criterion:
  `≤ 8 · steps · ε · Σ_k Σ_i m_i |v_{k,i}|`, which is `Sim::momentum_bound` with the scale
  summed over beads — the same worst-case one-ULP-per-posting accounting, over the `P · N`
  postings the ring commits instead of `N`. Total momentum is conserved EXACTLY in exact
  arithmetic by two independent mechanisms and the gate tests both at once: the physical
  kicks are equal and opposite pairwise per bead, and the free propagator leaves mode 0's
  velocity untouched while `Σ_k v_k = √P · v̂_0`. Graded at `P ∈ {1, 2, 4, 8}` on S1–S4.
  witness: none (a floating-point accounting bound, arithmetic, derived in place)
- **G3 — ANGULAR (derived).** Measure `|L_ring(t) − L_ring(0) − (angular impulse)|` peak,
  where `L_ring = Σ_k Σ_i m_i x_{k,i} × v_{k,i}`, against `Sim::angular_bound`'s form with
  the scale summed over beads. Graded on **S2 only** at `P ∈ {1, 2, 4, 8}`: S1 is planar,
  S3's walls break rotational symmetry (`angular_gate` returns `None`, VOID by construction
  as declared in §2), and S4's periodic box does not conserve angular momentum at all.
  Naming where a law does NOT apply is half of gating it — B2's own campaign gated three
  laws independently and had angular fire while momentum stayed green, which is only
  visible because they were separate. The non-obvious content: the free ring evolution
  conserves `L` **only because the ring Hamiltonian depends on squared inter-bead distances
  and every Cartesian component is propagated with the same `ω_k`**; a propagator that
  treated an axis differently would conserve energy and momentum and silently destroy `L`.
  witness: none (a conservation reading, measured against a derived roundoff bound)
- **G4 — SYMPLECTICITY / BIJECTIVITY (EXACT).** M-NONBIJECTIVE-STEP: any map called
  dynamics must be verified bijective. The free ring step is a linear map on `(q̂_k, v̂_k)`
  per mode; the gate asserts its determinant is `1` to `≤ 1e-14` for every mode at every
  graded `P`, and asserts the number of modes it checked so it cannot pass by checking
  none. This is arithmetic on the propagator's own coefficients, not a trajectory reading.
  witness: none (a determinant identity, checked numerically over the built coefficients)

### 4.1 The plants for the per-law gates

Each is verified to FIRE before its gate is believed, and each names the sector it acts on.

- **B4 (EXACT).** *The torn ring.* Replace the §1.3 centroid wrap with an independent
  per-bead wrap. **Sector: the ring spring across a periodic face.** Carrier requirement:
  a scene where a ring actually crosses a face during the run — S4, with the crossing
  COUNTED and the plant VOID if the count is zero — and `P ≥ 2`, since at one bead the
  centroid is the bead and the defect is invisible by construction. That invisibility is
  stated here rather than discovered later: **B4 is a plant for the conservation gates, not
  for G0.** Must fire G1 (energy) on S4 at `P ∈ {2, 4, 8}`.
- **B5 (EXACT).** *The refusals are real.* Present the ring path with a thermostatted
  configuration, a barostatted one, a grabbed one, and one violating `ω_max · dt < 1`.
  Criterion: 4 presented, 4 refused by name, 0 silently run — counted and reported as
  `4/4`, because a refusal nobody counts is indistinguishable from a refusal that never
  happened (M-VACUOUS-SUCCESS). **Sector: the configuration check at construction.**
- **B6 (EXACT).** *The dropped spring.* Zero the free propagator's `ω_k` for `k ≥ 1`,
  making every ring mode free. **Sector: the internal ring modes.** Carrier requirement:
  `P ≥ 2` and `ω_1 > 0` unplanted. Must fire G1 on at least S1 and S2 at `P ∈ {4, 8}`.
  This plant is the discriminator that separates "the spring is integrated exactly" from
  "the spring is not integrated at all" — two states an energy gate alone cannot tell
  apart, since a ring with no spring conserves its own smaller Hamiltonian perfectly
  (M-EXIT-DISCRIMINATOR).

---

## 5. THE PRICE

Work-unit priced, not wall-clock priced, because the box is loaded and shared.

- **G5 (EXACT).** The ring arm's count of physical force evaluations per step is exactly
  `P` times the classical arm's. Criterion: EXACT integer equality, `count_ring = P ×
  count_classical`, on all four scenes at `P ∈ {1, 2, 4, 8}`, with the counts printed. This
  is the campaign's cost model and it is a falsifying check in M-CHEAPER-THAN-ITS-PRICE's
  sense: a ring result arriving for fewer force evaluations than `P` per step is not a ring
  result, and one costing more has a term nobody declared.
  witness: none (an integer count, exact)
- **G6 (reported, not gated).** Wall-clock ratio ring-to-classical, reported with `n`, the
  SPREAD over repetitions, the core class (`taskset`-pinned to an E-core, which on this
  i9-13900HX part has no SMT sibling and therefore repeats), and `loadavg` at start AND
  end. **No verdict in this campaign is a function of it** (M-PLACEMENT-LOTTERY,
  M-IDLE-CALIBRATED-TIMEOUT). A launch header records the build's exit status alongside
  the commit, and any field inferred beside a pin is labelled inferred
  (M-PROVENANCE-OVERREACH).

---

## 6. KILLS — separable, each taking down its own row and nothing beneath it

- **K1.** G0 fails on any staked scene by branch (c) of §3.2 → **the `P = 1` in-engine
  identity claim is dead.** It does NOT touch C1: `rpmd.rs`'s own classical-limit result
  stands on its own carrier and its own gate, and a failure here convicts the COUPLING.
- **K2.** G1 fails inside §1.5's admissible region on a scene whose classical arm's own
  `energy_gate` is green → **the energy claim under RPMD is dead**, and only it. Momentum,
  angular and the identity claim are untouched.
- **K3.** G2 fails likewise → the momentum claim under RPMD is dead, and only it.
- **K4.** G3 fails on S2 likewise → the angular claim under RPMD is dead, and only it.
- **K5.** B3 shows the checkpoint unchanged between `P = 1` and `P = 2` → **`P` is not
  reaching the arithmetic**, the regime-identity claim of §1.6 is dead, and every reading
  at `P > 1` in this campaign is VOID with it.
- **K6.** G4 fails → the free step is not the symplectic map it is claimed to be, and G1's
  reading is VOID (not merely failed) because its bound was derived assuming it.
- **K7.** G5 fails → the cost model is wrong and every price in the results document is
  withdrawn. Does not touch any physics row.

**A fired kill stays fired and stays in the record marked dead.** C1's freeze had two
clauses fire — G4's second clause and G6 — and both are published beside the passes; this
freeze expects the same treatment of its own.

---

## 7. VOID CONDITIONS — where a row is UNGRADED rather than failed

Named in advance, because the difference between "the instrument could not read this" and
"the claim is false" is the difference this programme most often gets wrong.

- **V1.** A scene failing S0's anti-vacuity floor: every row on it is VOID.
- **V2.** A scene whose CLASSICAL arm's own `energy_gate` is already red at the derived
  `dt`: G1 on that scene is VOID. RPMD's energy conservation cannot be graded where the
  classical integrator is already outside its own budget.
- **V3.** Any configuration refused by §1.5's `ω_max · dt ≥ 1`: VOID, not failed, and the
  refused `(P, T)` pairs are listed. This is the honest reading of a fence: the region was
  not entered, so nothing was learned about it.
- **V4.** A plant that does not fire for a NUMERICAL reason the campaign can NAME (below
  the carrier's resolution, an exact-integer path, a saturating term): that plant is VOID
  and **the gate it was arming is VOID with it**, never passed. Three of seven mutations in
  a prior campaign stayed silent for exactly such reasons, so the presumption is that a
  silent plant convicts the plant first.
- **V5.** Arms that ran under different executor classes, different builds, or different
  `(P, T)`: VOID (M-DEVICE-CLASS, §1.6).
- **V6.** S4's wrap rows if the face-crossing count is zero, or if the periodic box refuses
  the configuration on a legality condition the engine already enforces.

---

## 8. WHAT LANDS, AND IN WHAT ORDER

Bottoms-up, and each step banked before the next begins.

1. This freeze, ADMITTED by `Audit/prereg_audit.py`, committed alone.
2. The factoring of `Sim::step` into its three passes, with `Sim::step` as their caller,
   landed with the whole existing `holon-render` suite green and unchanged — a refactor
   that moves a bit is not a refactor, and the existing gates are what say so.
3. `RingSim`, G0, G0b and plants B1–B3. **This is the bottom, and no `P > 1` physics is
   claimed until G0 is green with its plants firing.**
4. The per-law gates G1–G4 and plants B4–B6; the price G5–G6; the refusal count B5.
5. `NODE_E_RESULTS.md`, naming the instrument's commit and carrying every fired clause
   beside every pass.

## 9. WHAT THIS UNBLOCKS — named, and deliberately not run

`conformance/atomworld/CRYO_HO_RESULTS.md` carries a classical-nuclei fence on every arm
and says of it: *"Node E's ring-polymer route exists and is not coupled."* Landing node E
is that fence's exit. The re-run it makes possible is **the cryo campaign's H₂ arm with
quantum nuclei** — the arm whose binding was invisible to MBE3 and where hydrogen's mass
makes zero-point motion largest, so it is exactly the arm where classical nuclei are least
defensible.

**This campaign names that re-run and does not run it.** Node E's receipt is the coupling
and its gates; a physics re-reading is a separate freeze with its own kills, and running it
within this one would let a coupling gate be graded on whether its physics answer looked
right. The results document will state the exit as available and leave it to cryo's owner.
