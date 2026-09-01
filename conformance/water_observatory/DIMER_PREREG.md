# Pre-registration — DIMER-1: the (H₂O)₂ rigid-monomer surface, and which engine is allowed to build it

*Frozen 2026-09-01, committed ALONE, before any dimer node has been solved. This
campaign has two jobs and they are separable on purpose. The FIRST is a surface:
the rigid-monomer water–water interaction on a staked coordinate grid, computed
in-engine with no imported number anywhere in the chain. The SECOND is a
DECISION: whether the crystal engine's DMRG may stand in for determinant FCI on
heavy surfaces. The decision gate is the point. The surface is the instrument it
is decided on.*

**Do not read this as chemistry.** Every node is STO-3G, and STO-3G is not a
basis in which a hydrogen-bonded dimer's binding energy means anything — basis-set
superposition error at this size is comparable to the binding itself. What is
being measured is whether two ENGINES agree on the same Hamiltonian. A basis-set
claim would need a certified transport to a larger basis, which is
`OBSERVATORY_BRIEF.md` §7's open electronic-basis gap and is not this campaign.

---

misfits: contacts **M-CHEAPER-THAN-ITS-PRICE** (G0 exists to discharge it: the
per-node price is MEASURED on the path that will actually run, before the grid is
admitted, and the arithmetic-cannot-close check is a stated refusal rather than a
habit — the ozone incident's 65× impossibility is the case this gate is shaped
by); **M-DEVICE-CLASS** (the table declares ONE device class as part of the
artifact — `holon_chem::sigma_op::DeviceClass::Cpu` — and mixed-device generation
is refused, so no node in this table may come from an accelerator arm while
another comes from the processor arm); **M-STALE-INSTRUMENT** (runner, this
freeze and the results document are committed together, and every checkpoint
carries the record); **M-PLACEMENT-LOTTERY** (the G0 price is measured with both
arms pinned to the same core, both core classes reported, the adversarial one as
the headline, and gated on processor time rather than wall clock);
**M-PROVENANCE-OVERREACH** (the launch header records the build's exit status and
labels every inferred field as inferred — a sha256 beside an unverified HEAD
claim is more confidently wrong than the timestamp it replaced);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (three plants, each re-derived for THIS
instrument, each with its carrier asserted nonzero in the sector the plant acts
on, pre-checked to fire before any verdict is trusted — see §6);
**M-VACUOUS-SUCCESS** (a node counter is reported beside every gate, and a gate
that passes on fewer than its staked node count is VOID, never a pass by
shrinkage); **M-NULL-MISSTAKE** (every convergence premise is staked on the
quantity its limit actually controls: χ on the total energy, the referee band on
the interaction energy, and they are different gates because they are different
questions); **M-EXIT-DISCRIMINATOR** (S1's two branches are named before the run
and neither is the default); **M-ONE-MODEL-DELTA** (the referee is exact
diagonalisation in the same basis, not a second approximate model, so "better
than the one thing we tried" is not available as a conclusion). NOT contacted, and
named so the absence is deliberate rather than an oversight:
**M-GAUGE-LAUNDER**, **M-PARITY-PROTECT**, **M-LOOP-BLIND**, **M-BARE-CHARGE**,
**M-HOMOG**, **M-VOLUME-SCALE**, **M-COND-PROBE**, **M-KINEMATIC-NONLOCAL**,
**M-ELECTRIC-BASIS**, **M-RING-MIXING**, **M-GAUGE-UNIFORM-MOMENTUM**,
**M-PROBE-EIGENSTATE**, **M-FIXED-POINT-TRAJECTORY**, **M-NONBIJECTIVE-STEP**,
**M-FINAL-VIEW-COLLISIONS**, **M-MAINTENANCE-LENS**, **M-IDLE-CALIBRATED-TIMEOUT**,
**M-PROBE-THE-RESOURCE**, **M-CACHE-KIND** — no gauge observable, no trajectory,
no bijectivity claim, no field-theory volume limit and no cached artifact arises in
a variational ground-state energy campaign on a fixed molecular geometry.

---

## 1. The object

Two rigid H₂O monomers. Each monomer's internal geometry is FROZEN at the value
the engine's own STO-3G minimisation returns, pinned by G1 below and recorded in
the freeze's companion pin file before a single dimer node is solved. No
experimental geometry is imported; the observatory's rule is that no number enters
the chain from outside, and a monomer bond length is a number.

Electronic model: STO-3G, closed-shell ground state at `S_z = 0`, 14 spatial
orbitals (5 per oxygen, 1 per hydrogen, ×2 monomers), 20 electrons = 10 α + 10 β.

**The determinant count at the cap, computed not estimated:** `C(14,10)² =
1001² = 1,002,001`. That is the FCI node this campaign is priced against, and it
is roughly seventy times the largest determinant space `holon-chem`'s own module
header cites as its worked example. Whether it is affordable is G0's question,
not an assumption of this freeze.

---

## 2. The coordinate grid, staked

Stage A is one dimension. Stage B adds orientations to the SAME distance grid, so
that the two stages share a referee and Stage B cannot quietly change the ruler.

**The distance axis** (oxygen–oxygen separation, ångström, converted by the
engine's own constant):

```
R_OO ∈ { 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 3.0, 3.2, 3.4, 3.6, 4.0, 4.5, 5.0, 6.0, 8.0 }
```

Fifteen nodes: dense through the region where a hydrogen bond would sit, coarse
through the tail, and one node at 8.0 Å that exists to be the dissociation
anchor rather than to be interesting.

**The orientation axis**, five staked rigid-body arrangements, each fully
determined by the monomer geometry plus the named condition — no free angle is
optimised, because an optimised angle is a different campaign with a different
gate:

| tag | condition |
|---|---|
| `LINEAR` | donor O–H bond collinear with the O···O axis; acceptor's C₂ axis at the staked tilt of 0° from the axis |
| `TILTED` | as `LINEAR`, acceptor C₂ axis tilted 60° from the O···O axis |
| `BIFURCATED` | donor's C₂ axis along the O···O axis, both donor hydrogens toward the acceptor |
| `ANTIPARALLEL` | the two monomer C₂ axes antiparallel, both perpendicular to the O···O axis |
| `REPULSIVE` | donor's C₂ axis along the O···O axis with both hydrogens pointing AWAY — the arrangement with no hydrogen bond available, kept as the control that a binding claim must NOT reproduce |

Stage A = 15 nodes (`LINEAR` only). Stage B = 75 nodes (5 × 15). The grid is
frozen here; a node may be dropped only by a gate firing on it, never added.

---

## 3. The seam law, applied per table

The seam law is per table, and this is a new table, so it gets its own scan
before it freezes as a chart. Two diagnostics, both computable on nodes we are
already solving, because a scan that needs its own campaign does not get run:

* **The reference-determinant weight** `|c_HF|²` from the FCI vector at every
  referee node. A collapse of this weight is what strong multireference character
  looks like from inside a single-reference-dominated space, and it is the tell
  that a smooth chart is about to stop being one.
* **The χ-premise's own firing pattern.** A node where DMRG cannot converge in χ
  while its neighbours can is a chart complaint, not a solver complaint, and it
  is reported as one.

If either fires, the affected orientation's distance axis is SPLIT into smooth
patches at the firing node and each patch is charted separately — or the floor is
accepted with a written reason. It is not smoothed over. Chart handover here is
the same object `Object.lean` already carries: a reading that survives a motion.
witness: `Closed`

---

## 4. The two engines

**FCI (the referee).** `holon_chem::fci`, determinant full CI, matrix-free
iterative ground state. It has an independent internal check already banked —
`FciSpace::sigma` (Knowles–Handy string factorisation) and
`FciSpace::sigma_reference` (explicit connected determinants with Slater–Condon
rules) share no loop structure — and G2 requires those two to agree on the
referee nodes rather than assuming the production route.

**DMRG (the production candidate).** `q8_mps::dmrg::solve_electronic_ground_state`
on `Mpo::from_electronic_integrals`, run under the crystal campaign's **A1
schedule**, unchanged in form from SCHWINGER-2/3 where it was certified:

* a **χ ladder** with warm start — each rung reuses the converged state of the
  rung below rather than restarting from the reference product state, because
  starting from random when a converged neighbour exists is a cost with no
  benefit;
* a **sweep-adaptive local tolerance** — loose on the first sweeps, machine
  precision after, and warm-started runs skip the loose rungs;
* a **guarded stagnation exit** — only after two consecutive machine-precision
  sweeps whose energy moved by ≤ 1e-10 relative.

The A1 clause that matters most is the one that says what the schedule is NOT:
correctness is never delegated to the warm start. The convergence premises below
are what catch a badly converged node, as VOID.

χ ladder, staked: **χ ∈ {64, 128, 256}**, with 256 the top rung. Sweeps: 20 at
each rung, subject to the stagnation exit.

---

## 5. Gates

Node counts are stated on every gate; a gate passing on fewer than its staked
count is VOID rather than a pass.

- **G0 — THE PRICE, MEASURED ON THE PATH THAT RUNS, BEFORE THE GRID IS ADMITTED.**
  Solve exactly ONE node (`LINEAR`, R = 2.9 Å) on each engine and record the
  processor time. Both arms pinned to the same core, both core classes reported,
  the slower class as the headline number. The admitted Stage-B node count is then
  `budget / measured_per_node`, computed and written down, not hoped for. **The
  refusal:** any later batch of `n` nodes that arrives in less than `0.5 · n ·
  (measured per-node price)` is VOID together with anything that consumed it. If
  the measured FCI per-node price exceeds 2 processor-hours, the referee subset
  shrinks to its staked minimum of 9 nodes and Stage B's FCI arm is dropped
  entirely rather than the grid being widened to absorb the surprise.
  witness: none (a measured price has no Lean object; the corridor rule that
  CONSUMES prices does, and it is G3's witness)
- **G1 — the monomer pin (EXACT, in the sense that it is a pin and not a band).**
  The frozen monomer geometry is the engine's own STO-3G stationary point, located
  before any dimer node, recorded with its residual gradient ≤ 1e-6 hartree/bohr,
  and pinned by content hash. Every dimer node uses that pinned geometry and the
  runner refuses a geometry whose hash differs. 1 pin.
  witness: none (an engine-internal geometry pin; no Lean object covers it)
- **G2 — the referee is checked before it referees.** On all 9 referee nodes,
  `sigma` and `sigma_reference` agree to ≤ 1e-10 hartree on the converged vector's
  Rayleigh quotient. A referee whose two independent routes disagree is not a
  referee. 9 nodes.
  witness: none (measured agreement between two in-engine routes)
- **G3 — the χ premise, per node.** `|E(χ=256) − E(χ=128)| ≤ 1e-5` hartree, else
  that node VOIDs and is reported as VOID. Two-sided by construction: DMRG is
  variational, so `E(χ=256) ≤ E(χ=128)` must also hold, and a violation is an
  instrument failure rather than a better answer. 75 nodes at Stage B.
  witness: none (measured premise)
- **G4 — the referee band, per referee node.** `0 ≤ E_DMRG(χ=256) − E_FCI ≤ 1e-4`
  hartree on all 9 referee nodes. The lower bound is not decoration: DMRG is
  variational in the same space, so a DMRG energy BELOW the FCI energy is a defect
  in one of the two engines and the campaign stops until it is found. 9 nodes.
  witness: none (measured band between two in-engine routes)
- **S1 — THE DECISION.** On the 9 referee nodes, form the interaction energy
  `ΔE_int(R) = E_dimer(R) − 2·E_monomer` on each engine separately, in the SAME
  convention (raw and counterpoise-corrected are both computed; the comparison is
  raw-to-raw and corrected-to-corrected, never crossed), and take
  `Δ = max |ΔE_int(DMRG) − ΔE_int(FCI)|`.
  **Branch (a):** `Δ ≤ 1.6e-4` hartree (0.1 kcal/mol) ⇒ the crystal engine may
  build heavy surfaces; FCI is retained as a spot referee on a staked subset of
  every future table, never dropped.
  **Branch (b):** `Δ > 1.6e-4` ⇒ it may not, and heavy surfaces either pay the
  determinant price or are refused by name. The measured `Δ`, its node, and its
  sign are reported at survival volume either way.
  Fewer than 9 posable referee nodes ⇒ VOID, and VOID is not branch (a).
  witness: `select_admissible`
- **S2 — the decision is the corridor rule, not a preference.** S1(a) is admitted
  only if the DMRG node ALSO sits inside its declared budgets — G3's χ premise and
  G4's referee band are exactly the closure and conservation budgets the corridor
  rule tests. Cheapness alone selects the dead chart and is refused by theorem: the
  cheaper engine is not selected at any price unless it is admissible first, and if
  neither engine is admissible the rule returns nothing rather than the cheaper
  wrong answer. 2 candidate carriers.
  witness: `select_min`
- **S3 — the fence, stated as a gate so it can fire.** If no candidate passes S2,
  the campaign returns a REFUSAL and not a surface. A refused surface with a named
  reason is a result; a surface built by the engine that happened to be cheaper is
  not. 1 refusal path.
  witness: `select_eq_none_iff`

---

## 6. Plants

Three plants, each re-derived for THIS instrument rather than inherited, each
pre-checked to fire before any verdict above is trusted. For each, the plant's
carrier is asserted **nonzero in** the sector the plant acts on — a carrier that is
merely nonzero overall proves nothing about a plant that acts somewhere specific.

- **P1 — FCI, the two-electron integral.** Perturb one two-electron integral
  `(pq|rs)` connecting an occupied pair to a virtual pair by +1e-3 hartree, on the
  R = 2.9 Å `LINEAR` node. Required to move the node energy by ≥ 1e-5 hartree.
  Carrier: the converged CI vector. Sector: the doubly-excited block that integral
  connects, whose summed weight is asserted **nonzero in** that block (≥ 1e-4)
  before the plant is read — a plant on a sector the state does not occupy is a
  null mutation wearing a mutation's name, and the crystal campaign has already
  been caught by exactly that (its first MPO plant was identically zero on the
  physical sector).
- **P2 — DMRG, the MPO coefficient.** Introduce an off-by-one in one hopping
  coefficient of the electronic MPO, on the same node. Required to move the DMRG
  energy by ≥ 1e-4 hartree at χ = 256. Carrier: the MPS bond spectrum. Sector: the
  bond that term crosses, asserted **nonzero in** that bond's retained spectrum
  (kept-spectrum floor > 1e-8) before the plant is read.
- **P3 — the interaction-energy pipeline.** Substitute the dimer-geometry monomer
  energy where the isolated-monomer energy belongs — the classic interaction-energy
  bug, and the one that produces a plausible number rather than an obviously wrong
  one. Required to move `ΔE_int` by ≥ 1e-4 hartree. Carrier: the interaction
  energy. Sector: the monomer-relaxation contribution, asserted **nonzero in** it
  (the rigid monomers make this exactly zero by construction, so the assertion is
  that the substitution is DETECTABLE, and if it is not, P3 is reported as an
  inapplicable plant rather than a passed one).

All three must fire before S1 is read. A plant that does not fire is not a
reassurance; it means the sensor is dead.

---

## 7. What each outcome means

**S1(a).** The crystal engine's DMRG reproduces determinant FCI on a real
molecular Hamiltonian to better than a tenth of a kilocalorie, and heavy surfaces
— the ones whose determinant spaces are out of reach — may be built on it with FCI
retained as a spot referee. This is the transport claim in the tower's own terms:
two carriers reading the same number on the same object, which is what a certified
transport IS. The Lean says that a certificate composing to any height leaves the
reading unchanged; this campaign is the measurement of one rung's certificate, and
nothing here proves the square commutes — it MEASURES how far from commuting it is.
witness: `climb_total`

**S1(b).** The two engines disagree past the band on a system where the exact
answer is available, and the disagreement is banked with its node, its sign and its
size. Heavy surfaces then either pay the determinant price or are refused by name.
This is a result, not a setback, and it fires the tower's own acceptance law
(WB-8.4) if no reachable carrier can discharge the refusal.

**VOID.** Fewer than 9 posable referee nodes, or G0's arithmetic failing to close.
VOID is reported as loudly as either branch and is never read as branch (a).

---

## 8. Provenance and running discipline

Detached compute with done-markers and a `RESUME.md`; session death may kill
narration and never computation. The launch header records the build's exit status
and labels inferred fields as inferred. Checkpoints per node, resumable. One
declared device class for the whole table. No calendar language anywhere: the
campaign is sized by processor time and node count, both measured at G0.

The runner, this freeze, and the results document are committed together. A
results document without its instrument's commit is not banked.
