# WATER PHASE OBSERVATORY — the build brief

*2026-08-31. Commissioned by the operator. The flagship pivots: not a generic
universe sandbox, not 118 differently colored atoms — 128 H₂O molecules, 384
atoms, ONE Hamiltonian across every view and state, and the phases of water
inferred from the trajectory by a classifier that has never seen a preset
button. The three-atom water scene the programme already owns becomes the
observatory's diagnostic panel. SELECTOR-7 is parked, its commission note
preserved; this brief supersedes it in priority.*

*Style: this is a sprint brief, so it is complete rather than terse. Every
hazard is stated as a design requirement with its gate, never as a refusal.
Sizes are in compute and scope; no calendar estimates anywhere, per standing
order.*

---

## 0. The mission, and the three principles that are the mission

**Build four reproducible experiments over one Hamiltonian:**

1. **VAPOR** — expanding molecular gas; droplet evaporation.
2. **LIQUID** — stable density, molecular diffusion, hydrogen-bond turnover.
3. **ICE Ih** — persistent tetrahedral/crystalline order.
4. **DIRECT TRANSITION** — heat ice until the interface retreats; heat a
   liquid slab into vapor; walk temperature and pressure through the full
   state diagram, including the ice polymorph ladder on the reference
   backend.

**Principle 1 — one Hamiltonian.** The same energy function runs every
experiment and every lens. A phase is a *state* of one system, never a
different simulation. Any per-experiment special-casing in the force path is
a defect by definition.

**Principle 2 — the classifier is blind.** The phase label is inferred from
the trajectory (order parameters over sliding windows), never from which
preset was clicked. This is enforced by a plant, not a promise (§6, P-1):
feed the classifier a liquid trajectory launched from the "ice" preset
button and it must say LIQUID.

**Principle 3 — the macro lenses are the holon demonstration.** Each lens —
density, tetrahedral order, diffusion, H-bond network, largest domain,
energy, and the **closure defect** — is a lossy view `v : X → C` in exactly
`OBJECT.md`'s sense, and the observatory *measures* whether each view is
Closed under the dynamics (witness-pair search on the trajectory). "A phase
is a Closed view of molecular dynamics, and here is its measured closure
defect" is the claim that makes this a holon flagship rather than a demo:
quantum-informed molecular structure → collective organization →
macroscopic phase, with the commuting square checked at each rung.

---

## 0.5 Emergence scope — what is and is not claimed, so nobody conflates rungs

**Claimed as emergent here:** collective order from molecular terms. No
phase, network, or crystal structure is programmed in; ice Ih's stability,
liquid structure, and the transitions must come OUT of pair + few-body
surfaces under statistical mechanics, and the blind classifier finds them
or does not. This is chemistry-rung emergence: classical dynamics over
quantum-INFORMED surfaces (every table point is an exact-in-model
electronic-structure solve; the dynamics on top is classical, per
Born–Oppenheimer).

**Not claimed, and not assumed:** that ice is derived from or predicted by
the CRYSTAL TIER. SCHWINGER-3 and the crystal bank live on a different rung
— quantum lattice field theory, DMRG/MPS, mass-from-vacuum — with its own
provenance. Schwinger predicts a meson mass from a vacuum; it does not
predict ice, and this observatory's ice inherits nothing from it. The two
share the holon SHAPE (a macro quantity as a Closed view of a lower rung,
which is why the closure-defect lens runs at both rungs) and share no
machinery. The observatory neither waits on the crystal tier nor claims
its authority.

**The one real quantum gap, fenced now rather than found later:** nuclear
quantum effects. Real water's protons delocalize enough to shift bulk
properties, melting point included. Backend A absorbs NQE implicitly (its
parameters are experiment-tuned); Backend B, fit purely to
electronic-structure solves, will MISS them — so a backend-B disagreement
with experiment on absolute numbers is the missing quantum nuclei, not a
defect, and it is read that way. The named successor if the gap ever
matters to a claim: path-integral dynamics. Until then, backend-B claims
are about THE MODEL's phases, exactly as the rent clause's theorems are
about the model.

## 1. What already exists and is REUSED, not rebuilt

| Asset | State | Role here |
|---|---|---|
| OHH three-body surface, 65×49×49, regenerated under the current solver | committed, gate green (`tests/data/s2/s2_water_table.txt`) | the flexible **intramolecular** term of every water molecule |
| O-O, O-H, H-H pair curves | committed (SATURATION-2 machinery) | intramolecular pair terms; O-O's iteration-cap tail knots are documented (budget case, energies accurate) |
| The two SEAM findings (state crossings at θ≈174.9° and θ≈36°) | measured, 9c2ac72 | grid-design law for ANY new surface: locate seams first, split into smooth patches or accept the floor with a reason (§5, WP-2) |
| `SATURATION3/trimer-table/v1` artifact class | shipped (a40209b) | the manifest discipline (producer, route, grid rule, weighed uncertainty, REQUIRED seam record, no top-level converged boolean) — the intermolecular tables ship under a sibling schema |
| The leased mesh generator + launch discipline | shipped, CI-enforced | ALL table generation for this campaign runs through it: leases, receipts, digests, binary hash + build exit status in every log |
| DD overflow tier (`refine_determinant_dd`) | shipped, calibrated | hard dimer/trimer reference solves that cap at f64 get resolved, not refused |
| Provenance gates (uncertainty read as a NUMBER; refusal demonstrated both doors) | shipped (mixtures-engine) | the loader-side standard the water tables must meet from birth |
| The self-lifting fence + trimer loader | render-3d, in progress | the sandbox path that puts OXYGEN on screen — WP-0 rides it |
| 2D/3D sandbox + pages pipeline | live | the observatory's delivery surface |
| Misfit registry, 41 entries, forward-armed grep audit | live | every prereg in this campaign cites what it contacts (§7) |

## 2. What main lacks, stated as the work

Bulk water physics is not a scaling-up of the atomic model. Releasing
hundreds of O and H atoms under the present atomic MBE3 would be
misleading: the known missing four-body contribution can overbind extra
atoms (the MBE4 instrument was removed for exactly this), and OOH/OOO
coverage is incomplete. Therefore:

* **molecules are instantiated, not assembled** — 128 flexible waters are
  placed as molecules; spontaneous O/H assembly is not relied on and
  cross-molecule energies never route through the atomic MBE3;
* the missing physics is added as a separately provenance-gated
  **intermolecular layer**: water–water attraction/repulsion, molecular
  dipoles with long-range electrostatics, polarization/many-body effects;
* the missing infrastructure is added as engine work: periodic boundaries,
  neighbor/cell lists, a long-range solver, deterministic checkpointing,
  and a controlled-volume phase protocol (barostat later, §5 WP-6).

## 3. The Hamiltonian: two backends, one analysis stack

**Backend A — the pinned reference (first, fastest honest route).**
A published water potential implemented EXACTLY as published, clearly
marked as the reference backend: **TIP4P/2005** (rigid, 4-site, Ewald
electrostatics; pinned parameter block with citation in the manifest).
Chosen because its phase diagram is the best-mapped of any classical water
model — melting point ≈252 K, density maximum, and a published ice ladder
(Ih, Ic, II, III, V, VI) to compare the observatory's readings against.
The model-superset law applies with force: Backend A is TIP4P/2005 *as
published* — rigid, its own geometry — never a hybrid with our OHH surface.
A mixed model is a THIRD model and does not exist in this campaign.

**Backend B — the repository-native surface (in parallel).**
`NativeWater-1`: our flexible OHH intramolecular surface per molecule, plus
an intermolecular layer built from engine-computed water-dimer
configurations (a staked grid of O–O distance × mutual orientations, each
point an engine solve with counterpoise correction — the ELEMENTS-3 F1
machinery exists), a polarization term (staked functional form, fit only on
the dimer/trimer data, held-out validation mandatory), and long-range
electrostatics from the surface's own fitted dipoles. Ships as tables under
the trimer-class manifest discipline WITH seam records: the dimer surface
gets the seam treatment before its grid freezes (the θ-crossing lesson says
reactive/rearrangement channels put corners inside tabulated domains).

**Both backends run the identical phase-analysis stack, lenses, classifier,
and CI gates.** Where they disagree, the disagreement is a *reading* (model
difference), displayed, never averaged.

## 4. Physics scope and honest feasibility

* N = 384 atoms (128 molecules). Ewald at this size is cheap; a real-space
  cutoff + reaction-field variant is the wasm fallback, declared per build,
  never silently swapped (M-DEVICE-CLASS's shape: the electrostatics
  treatment is part of the artifact).
* Timescales: diffusion and H-bond turnover are ps-scale — interactive.
  Interface motion (melting/freezing direction) needs long runs —
  detached campaign jobs with checkpoints, replayed in the observatory.
  **Spontaneous nucleation of ice from cooled liquid is NOT the primary
  test** — nucleation can exceed any interactive timescale. The **seeded
  coexistence slab** is the stronger experiment: build ice|liquid in one
  box, run at a ladder of temperatures, and the interface's direction of
  motion measures which phase the model favors; the crossing estimates the
  model's melting point, compared against TIP4P/2005's published 252 K as
  the backend-A validation gate.
* Ice Ih seeds are generated proton-disordered under the Bernal–Fowler ice
  rules with near-zero cell dipole (gate: ice rules satisfied exactly on
  the seed; plant: a proton-ORDERED slab fed to the ice-rules gate must be
  flagged as ordered). Ice Ic and the high-pressure ladder (II, III, V,
  VI) are backend-A experiments — the native surface earns high-pressure
  trust later or not at all, and says so on its manifest.

## 5. Build order — work packages

Mirrors the operator's build order exactly; each WP names deliverables,
gates, plants, and kills. Preregs: one short freeze per WP with a measured
gate (this campaign's discipline is CI-gate-first rather than
hypothesis-first — the hypotheses live in WP-7's experiments).

**WP-0 — OXYGEN IN THE SANDBOX (the diagnostic panel).** Land the trimer
loader + fence lift (render-3d, in flight), ship the regenerated OHH
surface + O-H/O-O/H-H pairs to the page, and the three-atom water scene
becomes the observatory's diagnostic panel: one molecule, bend/stretch
modes visible, the intramolecular surface inspectable. DONE-when: the
sandbox renders H₂O with the fence lifted and the provenance panel showing
the table's manifest. *(Mostly already in flight — this WP is the
integration.)*

**WP-1 — PERIODIC WORLD.** Periodic boxes (orthorhombic first), minimum-
image convention, cell lists sized to the largest cutoff, and
DETERMINISTIC CHECKPOINTING: fixed-seed, fixed-order reductions,
checkpoint = bit-exact state, replay identity as a CI gate (same seed +
checkpoint → bit-identical trajectory segment; the debug/release
bit-identity check from the tables work is the precedent and applies
here). Kill for the WP: any gate that passes with the box un-wrapped
(plant: an atom translated by one box vector must produce bit-identical
energies).

**WP-2 — THE INTERMOLECULAR LAYER.** Backend A implemented and validated
against published TIP4P/2005 numbers (energy of published dimer geometry,
density at 298 K/1 bar within the published model's value, RDF g_OO peak
positions). Backend B's dimer campaign: staked configuration grid → engine
solves through the leased generator (launch discipline, DD tier for
cap-cases) → seam scan on the orientation axes BEFORE the grid freezes →
fit with held-out validation → tables under the manifest discipline.
Gates: provenance loader refuses missing/oversized uncertainty and missing
seam record (both doors demonstrated firing). Kill: held-out dimer error
above the staked bound re-scopes the fit, never widens the bound.

**WP-3 — 128 FLEXIBLE WATERS INSTANTIATED.** Molecule objects (Backend B:
flexible via OHH surface; Backend A: rigid constraints as published —
SETTLE/RATTLE, constraint residual gated). NVT thermostat, deterministic
(seeded, reproducible; thermostat choice documented with its known
artifacts). Energy conservation gated PER CONSERVED QUANTITY (the
one-gate-per-law lesson): energy drift bound on NVE segments, momentum
zero, and the energy ledger's zero-point from the force law's own zero.

**WP-4 — BLIND CENSUS LAYERS.** Molecule census (O-H connectivity by
distance criterion, gated against instantiation count — with the fence
that it must also RUN on trajectories where molecules could dissociate on
Backend B, and report, not assume); H-bond census (geometric criterion,
stated; turnover rate lens); tetrahedral order q per O (Errington–
Debenedetti); Steinhardt q6/local-structure for crystal recognition; MSD →
diffusion D; density field and largest connected domain; energy per
molecule; and the CLOSURE-DEFECT lens: for each macro view, the measured
witness-pair defect over trajectory windows — the holon reading.

**WP-5 — REFERENCE STATES SHIPPED.** Vapor, liquid, ice Ih equilibrated
states committed as checkpoints with manifests (backend, T, density, seed,
binary hash, generation log). The observatory loads them instantly;
regeneration is a leased detached job, never a page load.

**WP-6 — SWEEPS.** Heating and density sweeps as scripted protocols
(controlled-volume first: fixed-V temperature ladders, slab geometries;
NPT barostat is a later increment and its own prereg). Each sweep writes a
trajectory + census record the classifier and lenses replay.

**WP-7 — THE FOUR EXPERIMENTS + COEXISTENCE.** The four reproducible
experiments wired as observatory presets over the SAME Hamiltonian, plus
the ice|liquid coexistence slab ladder → interface-direction readout →
model melting point (backend A gate: brackets 252 K within a staked
window; backend B: reported as the native surface's measured melting
point, whatever it is — that number is a RESULT, not a target).

**WP-8 — CI GATES.** RDF against reference (backend A), diffusion in
liquid window, density stability, per-law conservation, phase separation
(the slab stays separated below melting, mixes above), replay identity,
ice-rules on seeds, classifier blindness plant P-1, and the provenance
refusals — all in ci-gates.sh with the affordable-half discipline (short
gates in CI; long campaigns detached with committed logs).

## 6. Plants (the gates must be seen to fire)

* **P-1 preset-blindness**: liquid trajectory from the "ice" button →
  classifier says LIQUID.
* **P-2 box plant**: one-box-vector translation → bit-identical energy.
* **P-3 ice-rules plant**: proton-ordered slab → flagged.
* **P-4 provenance plants**: oversized uncertainty and missing seam record
  → loader refuses (both doors, positive controls beside them).
* **P-5 classifier null**: high-T vapor windows → never ICE (false-crystal
  rate bound staked).
* **P-6 conservation mutation**: the old accounting (a known-wrong force
  zero-point) planted → energy gate fires.
* **P-7 backend integrity**: a hybrid configuration (flexible molecule
  under backend A) must REFUSE — the third-model trap made structural.

## 7. Misfit contacts (forward-armed audit will demand these)

M-DEVICE-CLASS (electrostatics treatment + device class are part of the
artifact); M-PLACEMENT-LOTTERY (any perf ratio pinned + both core types);
M-IDLE-CALIBRATED-TIMEOUT (every timeout/patience constant learned in the
running environment); M-VACUOUS-SUCCESS (census gates assert work counts);
M-PLANT-SECTOR (every plant's carrier asserted non-empty); M-CACHE-KIND
(state/table registries key kind+backend+size); M-MAINTENANCE-LENS (any
equilibration/annealing "repair" claim names the lens and runs the
issue-the-command-restore-nothing control); M-STALE-INSTRUMENT +
M-PROVENANCE-OVERREACH (launch headers: hash, HEAD, build exit status);
M-EXIT-DISCRIMINATOR (every solver/fit records why it stopped); the seam
ruling (no grid freezes before its seam scan or an accepted-floor note).

## 8. Sequencing and the two teams

Sprint team takes WP-1 through WP-3 plus backend A end-to-end — it is
self-contained engine work with published validation targets. Local lanes,
when they return: render-3d finishes WP-0's loader (in flight); the water
lane owns backend B's dimer campaign design and seam scans (their
instrument, their state-crossing law); the mesh lane's leased generator
runs all of backend B's solves; mixtures-engine's provenance-gate pattern
is the WP-2 loader standard. WP-4's closure-defect lens is lead work (it
touches OBJECT.md's contract). Nothing in WP-1..3 waits on anything local.

**The demo, when it stands:** drag the temperature slider on a seeded
ice|liquid box and watch the interface choose a direction; the lenses show
tetrahedral order collapsing, diffusion switching on, the H-bond network
fragmenting, density stepping — and the classifier, which has never seen
the slider, calls the phase from the trajectory alone, with the closure
defect of each macro view printed beside it. From a quantum-informed
molecular surface to a macroscopic phase, one Hamiltonian, every rung
measured. That is the holon claim with something genuinely difficult to
predict and measure.
