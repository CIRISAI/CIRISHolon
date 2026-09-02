# CRYO-H-O — prereg

*Frozen 2026-09-02, before any instrument in this campaign existed and before any number
this campaign produces was seen. Three arms: LIQUID HYDROGEN, LIQUID OXYGEN, and the
METALLIC-HYDROGEN FENCE. Everything below is staked with its kill, and each kill takes
down its own arm and nothing beneath it.*

**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-EXIT-DISCRIMINATOR, M-SORTS-NOT-SEPARATES,
M-VACUOUS-SUCCESS, M-TAG-AS-PROPERTY, M-UNTESTED-GAP, M-MAX-OVER-SUCCESSES,
M-VOLUME-SCALE, M-HOMOG, M-PARITY-PROTECT, M-BARE-CHARGE, M-NULL-MISSTAKE,
M-BASE-RATE-OMITTED, M-FIXED-POINT-TRAJECTORY, M-PLACEMENT-LOTTERY,
M-CHEAPER-THAN-ITS-PRICE, M-STALE-INSTRUMENT, M-PROVENANCE-OVERREACH, M-DEVICE-CLASS,
M-PROBE-EIGENSTATE.

Two of those are cited because this freeze uses a word the audit's contact table watches,
and honesty is better than contorting the prose around a grep: **M-BARE-CHARGE** is about
a gauge-charged carrier and this campaign has none — the S = 0 / S = 1 distinction in ARM 2
is a SPIN sector, not a charge sector; **M-PARITY-PROTECT** is contacted only through the
H ↔ H exchange symmetry of the three-body term, which is a permutation symmetry of a
model surface and not a protected dynamical sector. The other nineteen are contacted for
cause and each is named at the gate that contacts it.

---

## 0. THE THREE STANDING MODEL FENCES

Carried on every number this campaign emits, in every arm, without exception:

1. **2D SCENE.** Every dynamics scene is the `z = depth/2` slice. A 2D box is not a thin
   3D box; coordination numbers, packing fractions and the phase diagram itself differ.
2. **CLASSICAL NUCLEI.** The nuclei are Newtonian point masses. Node E's ring-polymer
   route (`holon-chem/src/rpmd.rs`) exists and is NOT coupled to the dynamics here. This
   is a disclosed model choice, not an oversight, and it is at its most severe exactly
   where ARM 1 lives: real liquid hydrogen is the most quantum molecular liquid there is
   (de Boer parameter ≈ 1.7), so a classical-nuclei reading about liquid H₂ is a reading
   about a model, never about the substance.
3. **STO-3G MINIMAL BASIS, FULL CI.** Exact in that basis and only in it. A minimal basis
   on hydrogen carries one 1s function per centre and no p function, so it has **no
   mechanism for London dispersion at any order** — dispersion needs virtual excitation
   into a polarised function that this basis does not contain. That fact is the engine of
   ARM 1's stake and is written here, before the measurement, as its warrant.

## 0.1 WORK-UNIT PRICING, and why wall clock is not a cost here

This box is an i9-13900HX with a load average of 60.0 on 32 cores at freeze time —
roughly 2× oversubscribed, on a heterogeneous P/E-core part. Per **M-PLACEMENT-LOTTERY**
a wall-clock number measured here is a measurement of the scheduler. Every price in this
campaign is therefore quoted as **solver calls × determinants**, and the banked evidence
for that rule is in the tree already: the SAME 96-knot O–O curve, the same work, is
banked at 144.4 s, 741.8 s and 1256.8 s across three logs (an 8.7× spread). The
campaign's **cost model** is stated per arm below and per **M-CHEAPER-THAN-ITS-PRICE** a
result arriving far under its own stated price is a finding about the result, not a
bonus.

## 0.2 DISCLOSURE FIELDS

Any number in this campaign derived from a CI solve carries, adjacent to it:
`solver_exit`, `solver_budget_iterations`, `worst_residual`, `n_det`, `n_basis`,
`variational_margin`, and the solution's `device` class (**M-EXIT-DISCRIMINATOR**,
**M-DEVICE-CLASS**). A residual without its exit is not a number: per
**M-SORTS-NOT-SEPARATES** a bar can rank rather than separate, and per the banked B1b
finding a capped residual is **not monotone in effort**.

**Inherited disclosure, carried on every ARM 2 number:** the banked O–O curve exits
`IterationCap` at `solver_budget_iterations = 5000` with `worst_residual = 4.809e-6 Ha`,
`n_det = 2025`, `n_basis = 10` (`B1B_RESULTS.md` W1). ARM 2's staked bands are all wider
than that residual, and where one is not, the reading is VOID rather than close.

## 0.3 INVENTORY READ BEFORE STAKING (banked, not produced here)

Stated so that nothing below can be mistaken for this campaign's own output.

| banked fact | value | source |
|---|---|---|
| H–H curve, 96 knots | `R_e = 1.3887 bohr`, `D_e = 0.204142 Ha`, residual 8.7e-11, `Converged` | `s2_runs/p1_hydrogen.log` |
| H–H referee, 50 digits | `R_e = 1.3886940180177763`, `D_e = 0.20414235210759105`, `E_asym = -0.9331636991145509` | `h2_potential.json` |
| **H–H pair excess in the tail** | −2.301e-4 Ha at 6.02 bohr; −2.115e-6 at 8.10; −1.289e-8 at 10.00 | same, read at freeze time |
| O–O curve, 96 knots | `R_e = 2.4421 bohr`, `D_e = 0.147621 Ha`, `IterationCap`, budget 5000, residual 4.809e-6 | `p2_de4_full/*.log`, `B1B_RESULTS.md` |
| pure-H quench, 12 atoms, 300 K, 8 seeds | 44 × H2, 2 × H4; zero free H in 8/8; fence 0 | SATURATION-2 P1 |
| pure-O quench, 12 atoms, 300 K, 8 seeds | 8 × O12 (one aggregate every seed); zero free O in 8/8; fence exactly 220 = C(12,3) | SATURATION-2 P1 |
| the `(O,O,O)` surface | UNTABULATED — `FENCES.md` P1/P2, `ozone.rs:412` returns `None` | `FENCES.md` |
| spin rule at the seam | `sz2_sector(n) = n % 2`: the MINIMAL \|S_z\| sector, containing every state of every S | `elements.rs:2649` |

**The H–H tail is the single most important banked number in this freeze.** Between 6 and
10 bohr the pair excess falls by 4.25 decades over 3.98 bohr — a decay constant of
2.46 bohr⁻¹, i.e. EXPONENTIAL, with no algebraic tail. That is the minimal basis having no
dispersion, measured rather than asserted, and it is what ARM 1 stakes against.

---

# ARM 1 — LIQUID HYDROGEN

## 1.0 The question, and why it is not the obvious one

The engine's dynamics is a many-body expansion over ATOM pairs and triples. The H–H pair
term is a **covalent bonding curve**, and it is applied to every H–H distance in the box
including the four cross distances between two already-saturated H₂ molecules. So the
engine has TWO different H₂–H₂ interactions and they are not the same object:

* **the model's own** — full CI on all four hydrogens at once, which is what the model
  actually says;
* **the engine's** — the truncated expansion the dynamics integrates.

An arm that measured only the second would be reporting an expansion artifact as
chemistry. An arm that measured only the first would not describe the engine anybody runs.
Both are staked.

## 1.1 The instrument

`engine/crates/holon-chem/examples/cryo_h2_dimer.rs`. Two rigid H₂ molecules, each at the
referee's `r = 1.3886940180177763 bohr`, centre-to-centre separation `R` swept over
**{2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0} bohr** in three
frozen orientations, all coplanar (the 2D fence):

* **H** — parallel, both molecular axes along x, centres offset along y;
* **T** — one axis along x, the other along y;
* **L** — collinear, both axes along x, end to end.

Read at each point:

```
E_int(R)      = E_FCI(4 H) − 2 · E_FCI(H2 at r_e)          the MODEL's answer
E_int_MBE2(R) = Σ_{6 pairs} V2_HH − (same for the two isolated molecules)
E_int_MBE3(R) = E_int_MBE2 + Σ_{4 triples} dE3_exact − (same for the isolated molecules)
```

Every sub-cluster energy is an **exact FCI solve**, not a table lookup: `dE3` here is
`E_FCI(3 H) − 3 E(H) − Σ V2` computed on the spot, so ARM 1 carries **no interpolation
error at all** and a residual it reports is the expansion's, never a table's. Prices:
4 H = 4 orbitals, 4 electrons, `C(4,2)² = 36` determinants; 3 H = `C(3,2)·C(3,1) = 9`;
2 H = 4. **Cost model: 13 separations × 3 orientations × (1 + 4 + 6) solves ≈ 429 solves,
none above 36 determinants.**

## 1.2 THE STAKE, and the meaning of every possible answer

Written from § 0 fence 3 — a basis with no p function on hydrogen cannot produce
dispersion — and from the measured 2.46 bohr⁻¹ exponential tail of § 0.3.

- **G1 — the model has NO H₂–H₂ well.** `E_int(R) > −1.0e-5 Ha` at every `R ≥ 3.0 bohr`
  in all three orientations. 1.0e-5 Ha is 3.16 K, and it is chosen as a bar this
  instrument can resolve (the solves are converged to ~1e-11) rather than as a physical
  threshold. **If G1 HOLDS: this model has no liquid hydrogen, and the number is the
  answer** — a measured-no-well with a bound, which is a publishable reading about the
  model. **KILL: any orientation showing a well of 1.0e-5 Ha or deeper.** That falsifies
  "minimal basis carries no dispersion" for this model and ARM 1's verdict inverts. The
  kill is separable: it takes down G1 and G3 and touches no other arm.
  witness: `none (a claim about STO-3G FCI on four protons; there is no Lean object here and inventing one would be decoration)`
- **G2 — the ENGINE's expansion invents an attraction the model does not have.**
  `E_int_MBE2(R) < E_int(R) − 1.0e-4 Ha` somewhere in `R ∈ [3.0, 6.0] bohr`. The
  mechanism, named in advance: the pair term is a bonding curve and knows nothing of
  valence saturation, so four cross pairs at ~4 bohr contribute ~4 × (−1.06e-2) Ha of
  spurious binding. **If G2 holds, the two H4 components in SATURATION-2's own banked
  hydrogen control are explained, not anomalous.** KILL: MBE2 tracks the model to within
  1.0e-4 Ha across that band — then the expansion is faithful here and the H4 artifact
  needs a different explanation.
  witness: `none (a statement about a truncation error, measured; no theorem is claimed)`
- **G3 — the three-body term corrects most of it, and the residual is reported not
  claimed.** Report `E_int_MBE3 − E_int` at every point. NO band is staked on the size of
  the correction, deliberately: staking one would be a number chosen to be met. What IS
  staked is the SIGN and the direction of the correction — `|E_int_MBE3 − E_int| < |E_int_MBE2 − E_int|`
  at the MBE2 worst point. KILL: MBE3 is further from the model than MBE2 at that point,
  which would say the expansion is not converging at ARM 1's own geometries.
  witness: `none (an ordering of two measured residuals)`

## 1.3 The quench ladder

`waterquench`'s `hydrogen` arm, protocol UNCHANGED (12 atoms, 34.6 × 20.8 bohr, 20,000
grain boundaries × 64 substeps, `T_init = 3000 K`, Berendsen `tau = 2000`), with the only
change being `T_target`. A logarithmic ladder in steps of ~3.16, **chosen as a log ladder
and not aimed at any physical transition temperature**:

    T_target ∈ {300, 100, 30, 10, 3} K,  seeds 0x53415421, 0x53415422, 0x53415423

The classifier reads the FINAL QUARTER of each run (grain boundaries 15,000–20,000,
sampled every 25 → 200 frames), so it reads the quenched state and not the quench.

- **G4 — no rung condenses.** In every rung and every seed, the fraction of atoms in
  components larger than 2 is `≤ 1/6` (i.e. at most one H4 in twelve atoms, the banked
  300 K artifact rate), and the largest component is `≤ 4`. **If G4 holds across the whole
  ladder down to 3 K, the model does not condense hydrogen at any temperature this
  protocol reaches, and G1 is why.** KILL: a rung where the majority of atoms sit in one
  component — the model condenses, and G1's no-well reading would then have to explain
  what binds it (the candidate is already named: G2's spurious pair attraction).
  witness: `none (a component-size criterion on the engine's own union-find)`

**A PRE-STAKED INSTRUMENT FENCE, and it is not a hedge.** The blind classifier's VAPOR
clause is `free_fraction > 0.50`, and `free_fraction` counts atoms in **singleton**
components. A box of six H₂ molecules has `free_fraction = 0` however dilute it is, so the
VAPOR branch **cannot fire on a molecular gas** and the classifier will report ICE or
LIQUID on a scene that is neither. This is stated here, before the run, because a
verdict-shaped output from an instrument outside its domain is exactly the shape of
**M-TAG-AS-PROPERTY**. The classifier's verdict is therefore reported **verbatim and
unscored** on ARM 1; G4 is the criterion that decides ARM 1, and it is a molecular
criterion because the scene is molecular. The fence's exit is named: a classifier whose
free clause counts MOLECULAR components rather than singleton atoms.

---

# ARM 2 — LIQUID OXYGEN

## 2.0 THE DISCLOSURE GATE, first, before any dynamics

Physical O₂ is a triplet ground state (³Σg⁻) and liquid O₂ is paramagnetic. The engine
solves every even-electron system in `sz2_sector(16) = 0`, the `S_z = 0` block. The
warrant written into `elements.rs` is that a multiplet of total spin `S` has a component
in every sector with `|S_z| ≤ S`, so the minimal sector contains every state and the solve
cannot miss the ground state whatever its spin. That warrant is a claim, and this gate
measures it rather than repeating it.

- **G5 — which spin sector the banked O–O curve actually returned.** Solve O₂ at
  `R = 2.4421 bohr` in `S_z = 0`, take `⟨S²⟩` of the converged CI vector
  (`fci::s_squared`) and `multiplicity(⟨S²⟩, 16, 1e-6)`. **STAKED: `⟨S²⟩ = 2.000 ± 0.01`,
  multiplicity 3 — the banked curve IS a triplet curve and the paramagnetic fence does
  not need to be raised.** **KILL: `⟨S²⟩ = 0` (multiplicity 1).** Then the banked O–O
  curve is an S = 0 curve, physical liquid O₂ is not, and that becomes a named MODEL FENCE
  carried on every ARM 2 number and written into `FENCES.md` — not a silent substitution.
  A third branch is pre-committed rather than left to be improvised: **⟨S²⟩ resolving to
  neither 0 nor 2** (a spin-contaminated vector, `multiplicity` returning `None`) is
  reported as `UNRESOLVED` with the value printed, and ARM 2 then carries the fence in its
  weaker form ("the sector is not established") rather than either verdict.
  witness: `none (an expectation value of the converged vector; the warrant it tests is a code comment, not a theorem in this tree)`
- **G6 — the sector plumbing is two-sided.** Also solve at `S_z = 1` (`n_α = 9, n_β = 7`)
  and at `S_z = 2` (`n_α = 10, n_β = 6`). STAKED: `E(S_z=2) > E(S_z=0)` by more than
  1.0e-4 Ha (twenty times the inherited 4.809e-6 residual); and IF G5 returns a triplet,
  `|E(S_z=1) − E(S_z=0)| < 1.0e-6 Ha`, because the M_s components of one multiplet are
  exactly degenerate in a spin-free Hamiltonian. KILL: either fails — the sector machinery
  is not doing what the code says and every energy that came through it is in question.
  This gate is the reason G5 is a measurement and not a lookup.
  witness: `none (degeneracy of M_s components of one multiplet under a spin-free Hamiltonian — standard, not mechanized here)`

## 2.1 What ARM 2 can and cannot reach, priced

**The ARM 1 trick does not transfer, and here is the number.** The exact O₂–O₂
interaction needs FCI on four oxygens: 20 orbitals, 32 electrons, `C(20,16)² =
4845² = 23,474,025` determinants — **11.7× past `HARD_DETERMINANT_CAP = 2,000,000`**
(`fci.rs:1085`) and 469× past `MPS_ROUTE_THRESHOLD = 50,000`. So ARM 2 gets no
exact-in-model intermolecular reference. This is a COMPUTE-PRICED fence with a stated
exit (`FENCES.md` C5/C6, GANTT node F), and its consequence is stated in advance: **ARM 2
cannot decide whether the model has a molecular O₂ liquid, only what the pair-only
expansion does with twelve oxygens.**

Worse, and also stated in advance: the `(O,O,O)` three-body surface does not exist
(`FENCES.md` P1), so the oxygen scene runs **MBE2-only** with the fence counted at
`C(12,3) = 220` per force evaluation. The model has no valence-saturation term for
oxygen at all. SATURATION-2's banked 8 × O12 result is what that produces.

## 2.2 The quench ladder

Same ladder, same seeds, `waterquench`'s `oxygen` arm, protocol unchanged. **The O–O
curve is generated ONCE (96 knots, 2025 determinants each) and reused across all fifteen
runs** — the arm's whole cost model is that one curve.

- **G7 — the model's cold oxygen is ONE AGGREGATE, not a molecular liquid, at every
  rung.** STAKED: largest component ≥ 10 of 12 in ≥ 2 of 3 seeds at every rung, fence
  exactly 220 every seed, zero free O. **Physical liquid O₂ boils at 90.2 K — that number
  is LABELLED CONTEXT and nothing here is scored against it, nothing is tuned toward it,
  and the ladder was chosen before it was looked up.** KILL: a rung where the scene
  breaks into O₂ molecules (modal component size 2). That would be the model exhibiting
  valence saturation for oxygen with no three-body term, which the expansion has no
  mechanism to do, and it would be a finding about the pair curve.
  witness: `none (a component-size criterion; see G4)`
- **G8 — the classifier's reading of that aggregate, and where it turns.** Report the
  blind classifier's `order`, `mobility`, `free_fraction`, `ice_criterion_fired` and
  verdict per rung. STAKED, one-sided and weak on purpose because this is a five-point
  ladder on three seeds: `order` is **monotone non-decreasing** as `T_target` falls,
  within a tolerance of 0.05 per step. KILL: `order` falls by more than 0.05 between two
  adjacent rungs as the scene is cooled. **NO transition temperature is staked** — per
  **M-UNTESTED-GAP**, this campaign has no prior points on this axis and a staked
  crossing would predict nothing. If a turn is seen it is reported as located between two
  rungs, never as a number.
  witness: `none (a monotonicity criterion on a measured order parameter)`

---

# ARM 3 — THE METALLIC-HYDROGEN FENCE

## 3.0 This arm claims no phase, and says so first

Metallization is electron delocalization across many centres. The engine's entire picture
is fragment-local: energies assembled from clusters of two, three and four atoms, each
solved with its own electrons. **A metal is the exact breakdown of that picture, and this
engine cannot exhibit one.** What it can do is measure WHERE its own picture fails, in
its own units, and name the exit. That is the whole of ARM 3. Any sentence in the results
document that reads as a claim about metallic hydrogen is a defect in the results
document.

## 3.1 The instrument

`engine/crates/holon-chem/examples/cryo_h_compress.rs`. Eight hydrogens as **four H₂
molecules on a 2 × 2 planar lattice**, every bond frozen at the referee's `r_e`, molecular
axes all along x, nearest-neighbour centre separation `a` stepped DOWN:

    a ∈ {8.0, 6.5, 5.5, 4.5, 4.0, 3.5, 3.0, 2.6, 2.2, 1.9} bohr

At every rung, all four levels of the expansion built from **exact FCI sub-clusters**:

| level | what it sums | price per rung |
|---|---|---|
| `E_exact` | FCI on all 8 H | 1 solve, `C(8,4)² = 4,900` det |
| `E_MBE2` | atoms + `C(8,2) = 28` pair excesses | 28 solves, 4 det |
| `E_MBE3` | + `C(8,3) = 56` exact three-body terms | 56 solves, 9 det |
| `E_MBE4` | + `C(8,4) = 70` exact four-body terms | 70 solves, 36 det |

**Cost model: 10 rungs × 155 solves = 1,550 solves, the largest 4,900 determinants.** No
table, no interpolation, no fitted surface anywhere in this arm.

Recorded per rung, all of them disclosure fields: `solver_exit` at every level,
`davidson_iters`, `worst_residual`, `variational_margin` (a NEGATIVE margin VOIDs the
rung — the answer cannot be the ground state), SCF convergence, and the number of
sub-cluster solves that did not exit `Converged`.

Also per rung, and both quoted because neither alone is honest:

* **the model's own 2D pressure**, `P_2D = −dE_exact/dA` by centred difference on the
  lattice scale, in **Ha/bohr²**;
* **the engine's virial pressure**, `Sim::pressure()` on the same configuration at zero
  velocity (so the kinetic term is exactly zero and the reading is purely
  configurational), in Ha/bohr³ and in Pa via `AU_PRESSURE_PA = 2.9421015697e13`.

**THE UNIT FENCE, stated before the numbers exist.** `Sim::pressure` computes
`(2K − Σ virial) / 3V` with `V = width · height · depth`, and a 2D scene carries
`depth = 24.0 bohr` by default (`sim.rs:725`). So **the engine's pascal number for a 2D
scene is a three-dimensional pressure on a slab of arbitrary assumed thickness**, and the
`3` in the denominator is the 3D virial factor where a 2D scene wants a `2`. Both defects
are in the printed number and neither is a bug in this campaign. The results document
quotes `P_2D` in Ha/bohr² as the primary reading, quotes the engine's pascal number beside
it, and states the assumed thickness every time. Converting either to GPa and comparing
against the ~500 GPa metallization pressure of the literature is **not** performed:
the comparison would be a number with an invented thickness in it.

## 3.2 THE STAKE

- **G9 — the expansion's error grows under compression, and the fence has a location.**
  Report `|E_exact − E_MBE_k| / 8` (hartree per atom) for k = 2, 3, 4 at every rung.
  STAKED, from the fragment-local picture itself: (i) at `a = 8.0 bohr` the MBE3 error is
  **below 1.0e-3 Ha/atom**; (ii) that error is **monotone non-decreasing** as `a` falls;
  (iii) it **crosses 1.0e-3 Ha/atom** somewhere on this ladder. 1.0e-3 Ha/atom is stated
  as the bar because it is below chemical accuracy (1.6e-3 Ha) and because the fence
  should be located where the model stops being usable, not where it becomes absurd.
  **The fence's location is the first rung where the crossing happens, quoted in `a`
  (bohr), in number density `N/A` (bohr⁻²), and in `P_2D` (Ha/bohr²).**
  KILL, and each is separable: (i) fails → the expansion is already broken at the loosest
  lattice and this arm has no baseline; (ii) fails → the error is not a monotone function
  of density and "the fence has a location" is the wrong shape of claim; (iii) fails →
  the expansion survives the whole ladder and the fence is BEYOND `a = 1.9 bohr`, which is
  reported as a bound, not as an absence.
  witness: `none (a truncation-error measurement on one model system)`
- **G10 — the ladder's own non-convergence signature, reported beside G9.** The MBE
  ladder is CONVERGING at a rung iff `|E_exact − E_MBE4| < |E_exact − E_MBE3| <
  |E_exact − E_MBE2|`. Report the first rung where that chain breaks. No band is staked
  on where it breaks — per **M-UNTESTED-GAP** there are no prior points on this axis —
  but the two locations (G9's crossing and G10's break) are reported together, and if
  they disagree the disagreement is the finding.
  witness: `none (an ordering of three measured residuals)`

**Scope, stated as narrowly as the measurement supports.** Eight atoms in one geometry in
two dimensions. Per **M-VOLUME-SCALE** the fence's location is an 8-atom location and this
campaign does not establish that it survives N — the exit is a larger scene, which is
`HARD_DETERMINANT_CAP`-priced (12 H is `C(12,6)² = 853,776` determinants, below the cap
but past `MPS_ROUTE_THRESHOLD`, so it would arrive by a different route and per
**M-DEVICE-CLASS**/`SolverRoute` would not be the same artifact). Per **M-HOMOG** a
regular lattice is a spatially homogeneous carrier, so this arm measures the fence's
location on an ordered scene and says nothing about a disordered one.

**The exit, named as the law requires.** Past the fence this engine stops being able to
speak, and the exit is **delocalized / periodic electronic structure** — a band or
plane-wave solver with k-point sampling, a different solver class entirely, out of scope
for this campaign and for this crate.

---

# PLANTS

Four, each re-derived for THIS instrument (**M-PLANT-OBS**), each with the sector it acts
on named and its carrier required **nonzero in** that sector (**M-PLANT-SECTOR**). Every
plant is run and verified to FIRE before its arm's verdict is read; a plant that does not
fire VOIDs its arm's gate rather than being tuned until it fires.

- **P1 — the sign plant (ARM 1, G1).** Negate `E_int(R)`. Carrier: the intermolecular
  energy channel, which is **nonzero in** that channel over `R ∈ [3.0, 6.0] bohr` where
  `|E_int| > 1.0e-4 Ha` by the banked pair tail. MUST fire: G1's no-well verdict must
  invert to a well of at least 1.0e-4 Ha. A plant that leaves G1 reading NO WELL means
  G1's finder cannot see a well at all and G1 is VOID.
- **P2 — the three-body deletion plant (ARM 1 G3, ARM 3 G9).** Drop every three-body term
  and present `E_MBE2` as `E_MBE3`. Carrier: the three-body channel, which must be shown
  **nonzero in** that channel — printed as `Σ dE3` at the rung — before the plant is
  scored. MUST fire: the reported MBE3 residual changes by ≥ 10× at ARM 3's loosest rung.
  This is the plant that proves ARM 3 measures the expansion and not a constant offset.
- **P3 — the spin-sector plant (ARM 2, G5/G6).** Force the O₂ solve into `S_z = 2`. The
  quintet block is **nonzero in** the `S_z = 2` sector and contains no `M_s = 0` component
  at all, so the plant's carrier is disjoint from the honest solve's by construction. MUST
  fire: `E(S_z=2) − E(S_z=0) > 1.0e-4 Ha`, i.e. G6's own criterion IS the plant's firing
  condition, which is why G6 is written two-sided.
- **P4 — the scrambled-scene plant (ARM 1 G4, ARM 2 G8).** Take the trajectory that reads
  highest `order` and randomly permute each frame's positions across atoms, independently
  per frame, leaving the bonded bitset and the per-frame position multiset untouched.
  Carrier: the bond-orientational order channel, **nonzero in** that channel — the
  unscrambled reading is printed first and must exceed 0.10, or the plant is unobservable
  on this scene and is reported as such rather than passed. MUST fire: `order` falls below
  `STAKE_ORDER = 0.45` and the verdict leaves ICE. Per **M-VACUOUS-SUCCESS** the plant
  reports its own work count — frames permuted, atoms moved — and a plant that permuted
  nothing is a failure, not a pass.

---

# VOID CONDITIONS

Distinct from kills. A VOID says the instrument did not run, not that the world answered.

- **V1 — instrument identity.** Any curve regenerated in this campaign that does not
  reproduce the banked `R_e`/`D_e` to their printed digits (H–H 1.3887 / 0.204142; O–O
  2.4421 / 0.147621) VOIDS the whole campaign: it is not the banked instrument.
- **V2 — ledger.** Any quench rung with `drift/bound > 1` or `|p|/bound > 1` is VOID and
  reported, never averaged in.
- **V3 — classifier refusal.** A rung on which the classifier returns `Verdict::Refused`
  reports the refusal and its gate, and is not scored in G8's monotonicity.
- **V4 — solver exit.** Any ARM 3 rung whose `E_exact` solve does not exit `Converged`,
  or whose `variational_margin` is negative, is VOID for the residual comparison and is
  printed with its exit. Per **M-NULL-MISSTAKE** a VOID is staked on the quantity the
  gate constrains — the exact reference — and not on the sub-cluster solves, which have
  their own count.
- **V5 — basis linear dependence.** ARM 3's ladder descends into geometries where the
  overlap matrix stops being positive definite and `cholesky_orthonormaliser` refuses.
  That refusal is a legitimate terminus, is caught rather than allowed to abort the run,
  and the rung at which it fires is REPORTED as the basis's own limit — which is a
  *different* fence from G9's and must never be presented as G9's answer.
- **V6 — a plant that does not fire** VOIDs the gate it guards, per **M-PLANT-OBS**.

# THE LOCALIZATION CLAUSE

Where any arm's error is concentrated, the results document names where. Specifically:
ARM 1 names the orientation and separation of its worst MBE residual; ARM 3 names which
sub-cluster class (pair, triple, quadruple) carries the largest share of the MBE3 error at
the fence, and whether the error is spread over the lattice or sits on one local pair.
An error reported only as a total is not reported.

# WHAT THIS CAMPAIGN CANNOT DECIDE, listed before it runs

1. Whether the SUBSTANCES liquid H₂ and liquid O₂ behave as this model does. Three
   standing fences (§ 0), and for hydrogen the nuclear-quantum one is severe.
2. Whether the model has a molecular O₂ liquid — priced out at 23,474,025 determinants
   (§ 2.1).
3. Where the metallization pressure of hydrogen is. ARM 3 locates a fence in the model's
   own picture and refuses the pressure comparison for the unit reason in § 3.1.
4. Whether ARM 3's fence location survives N or survives disorder (§ 3.2).

The result is banked in `conformance/atomworld/CRYO_HO_RESULTS.md`, verdict first, with
the instrument commit named beside every number (**M-STALE-INSTRUMENT**) and with no
inference attached to that pin beyond what it measures (**M-PROVENANCE-OVERREACH**).
