# The object — the engine's contract

*Engineering statement. The research programme behind it, with its
measurement record, lives in CIRISAI/CIRISOntology; every claim here names a
machine-checked theorem in `lean/CIRISHolon/Object.lean` or a conformance
obligation testable in CI.*

## One question

A **view** is a lossy reading `v : X → C` of a state space. A **motion**
`T : X → X` is a dynamics step, a re-root, or a chart change. The contract is
the commuting square:

```
      X ──T──▶ X
      │        │        Closed v T  ≔  ∃ h, v∘T = h∘v
      v        v        Held   v T  ≔  v∘T = v
      ▼        ▼
      C ──h──▶ C
```

**A tier is a Closed view of the tier below.** That is the entire definition.
`h` is the coarse dynamics the engine runs in the tier's place; `Held` is the
special case of invariants and paid-up maintenance.

**Failure has a normal form** (`nonfactoring_iff_not_closed`): a view fails
closure exactly when there exist two states it cannot distinguish that the
motion sends to distinguishable readings — a *witness pair*. Every
conformance test in this engine is a hunt for witness pairs; every bug report
against a tier should ship one.

## The design rules, each backed by a theorem or a measured law

1. **Closure is certified, not assumed.** A tier ships with its battery run:
   construction premise (macro-matched twins read identically pre-step),
   budget (coarse divergence growth ratio ≤ 1.05 over its rise epoch —
   measured unbreached across seven geometries in the predecessor), and its
   witness-pair hunt.
2. **Exact closure is not expected; budgets are.** Deterministic contact
   dynamics leaks micro detail into every coarse view. The claim is never
   "zero leak"; it is "non-expanding leak within the stated budget."
3. **Charts declare their conditioning** (`sum_perturb_le`,
   `sum_perturb_attained`, `coherence_of_nonneg`). A near-cancelling
   aggregate amplifies per-term perturbation by 1/coherence, exactly, and
   the coherence of signed aggregates *decays as scenes settle* (measured:
   momentum-chart coherence 0.998 → 0.125 across a settling window while an
   all-nonnegative chart stays at exactly 1). Ill-conditioned charts are
   internal diagnostics, never engine state.
4. **Privilege is two-dimensional.** How "real" a coarse quantity is at a
   tier = (its chart's conditioning) × (whether the dynamics organizes
   divergence into it). Both are measurable; neither is assumed.
5. **Maintenance is rent-priced** (`rentStep`, `Ginf`, `Wstar`,
   `rent_closed_form`, `Ginf_at_Wstar`): retention under decay `lam` and
   dose `q` is `q/((1−lam)+q·lam)` at the fixed point, transient closed-form.
   LOD refresh, cache decay, and repair schedules are computed from the law.
   Two measured riders: multi-mode decay makes the single-mode law a
   *bracket* (stake floors, not equalities), and **the repair must know the
   design** — a design-blind repair holds a structure's size while its
   identity decays.
6. **Identity is arena-level and append-only.** Stable identity is the
   arena index, never a view-level or spatially-sorted index.
7. **One gate per conserved quantity, chart-relative.** Energy, momentum,
   and impulse each get their own gate, and a balance gate refuses where its
   chart has no time-translation symmetry.
8. **The quantum relation is the retract, never a bolted-on mode**
   (`bornView_diagEmbed`, `liftChannel_factors`, `lift_commutes`,
   `diag_view_closed_of_classical`): classical state embeds as the diagonal;
   Born readout is Closed with the classical step as its update; and the
   wall is a theorem (`diag_not_closed_under_coherence`) — coherence is
   precisely where the classical tier ends. Simulation strata follow:
   classical circuits at mesh cost; stabilizer circuits in a closed tableau
   view; bounded-contextuality circuits priced by their contextual fraction;
   tensor-network bulk with defect-priced bond dimension; and past the wall,
   known-exponential cost, hardware delegation, or refusal by name.
9. **Refusal is a feature.** A tier or stratum outside its certified scope
   refuses, naming the gate whose passing would lift the refusal.

## How potential-energy tables are produced

**Shipped path, 2026-08-30:** every potential-energy table this engine vouches for is
generated through `holon-tables`' leased generator — probed worker leases, receipts that
accrue while the work runs, a merge-digest certificate, and a launch header carrying the
binary's sha256 and the *build's exit status*. It is bit-identical across worker counts,
across separate process invocations, and across debug and release.

The caller supplies the physics and the layer refuses to invent it: a domain is a claim
derived from a species pair's own curve and must cite the curve files it read, so the
generator exits rather than defaulting one. Details, guarantees and the full refusal list:
**`engine/RESOURCE_DESIGN.md` §11**.

## Conformance obligations (CI, per tier)

- The closure battery (construction premise, budget, witness-pair hunt).
- Chart conditioning declarations for every exposed aggregate.
- Per-conserved-quantity gates with planted-mutation sensitivity
  (a gate that cannot fire on a planted violation is refused).
- For quantum strata: the retract test (Born readout = classical step,
  exactly) and the QASM suite up to the stratum's declared boundary.

## The maximal steelman — the strongest honest case, with every kill attached

*The stance with statuses lives in CIRISAI/CIRISOntology (`Stance.lean`, claims
`closure`, `water-holon`, `upward-closure`, `object-rent`, and the cosmological
wagers). This section argues the engineering programme's maximal thesis at FULL
STRENGTH — the best case the evidence permits — and attaches to every move the
observation that would kill it. A steelman that cannot die is advocacy.*

**The thesis, at full strength.** AN OBJECT IS A SHARED PATTERN WHOSE CLOSURE PAYS
ITS OWN RENT. Some distinctions survive evolution and others do not; a thing is a
lossy summary the dynamics never splits, its law is forced rather than fitted,
things stack in tiers, conservation descends the tower for free — and every thing
above the bottom (molecule, network, fluid, law, habit, self) is the one commuting
square wearing different clothes. It persists exactly as long as its maintenance is
paid, by a payer that knows the design. The books that maintenance keeps are not a
metaphor: they are the same three-part structure (capacity, writer, record) the
universe's own ledger wears.

### The argument in five moves

**Move 1 — Existence IS closure. PROVED.** `Closed v T ≔ ∃h, v∘T = h∘v`. Failure has
a normal form (`nonfactoring_iff_not_closed`: a witness pair — two states the summary
cannot tell apart that the motion sends to different readings). The coarse law is
unique on reachable readings (`closure_determines_dynamics`) — a thing's law is
FORCED, never fitted. Closed views compose (`Tiers.lean`, `viewClosed_comp`) — which
is why chemistry and biology can both be true of one world. A reading invariant under
the coarse law is conserved by the fine dynamics (`closed_view_inherits_conservation`,
`conserved_descends`). Maximality is root-relative (`Omega.lean`); approximate
closure carries a budget growing linearly in the non-expanding case (`Budget.lean`).
The founding shape — two wholes agreeing under every partial view, differing in the
quantity — is one machine-checked object witnessed thrice in the sibling seed
(`Core/NonFactoring.lean`: parity, the CP phase, the Record), and a fourth time at the
bottom of matter (`Core/ExchangeSign.lean`: fermion vs hard-core boson).
*Kill:* a machine-checked counterexample — a definition of thing under which
something stays a thing while failing closure at every window. That breaks the
definition, not a measurement; none exists.

**Move 2 — Closure is REALIZED by nature-from-first-principles, and the instrument
can say no. MEASURED.** In a world built from nuclear charges, masses and
per-encounter exact diagonalization — no fitted potential, no chemistry written in —
a water molecule assembled itself and PASSED the closure test staked before the
instrument existed: longest closed run 893.8 fs against the pre-staked 834 fs
window, 72.3% of a 17.5 ps trajectory, momentum at 6e-5 of its roundoff bound, ZERO
of 111 look-alike groupings reaching the window, formula-reader "molecules" refused
as transient (`CENSUS_PREREG.md`, `CENSUS_RESULTS.md`). The four-body comparison then
RAN and the kill this claim carried did not fire: the exact four-body arm certifies
the same molecule strict (2599.8 fs), and the control arm — four-body provably
absent, plane held bit-exactly — certifies on its own, so the term is not what
carries hydroxyl to water (`CENSUS_RESULTS.md` §14). What the term ADDS stays open:
the treated arm alone left the declared plane and explored space the control could
not, so the attribution half was defeated by the treatment producing its own
variable — a failure mode no same-commit discipline prevents.
*Kills, separable:* regeneration from the committed protocol failing to reproduce a
certified block; the certification failing its pre-registered successor floor.

**Move 3 — Closure above the bottom is NOT FREE, and the steelman's strength is
that it predicted the price's SHAPE before measuring it. MEASURED.** Design rule 2
says exact closure is not expected, budgets are. The first upward campaign measured
exactly that: on the certified carrier no coarser summary is both dynamic and inside
its budget — rung 1's 70 readings split 36 vacuous-in-budget / 32 dynamic-out /
ZERO both, the boundary being ALIGNMENT (molecules present and within H-bond
distance 84–99.8% of frames, inter-molecular H-bonds in 0–18 of 20,000); rung 2's
occupancy/transport scissor is CHART-INDEPENDENT (A2: bit-identical verdict census
under a wholly different chart). On the founding lattice tier the closure defect is
DERIVED, not fitted — the block's boundary fraction `W(b) = 1 − max(0,b−2)²/b²`,
exact at every measured point, saturated, identical across all 4,608 lawful
collision laws (the defect belongs to the lattice, not the law), with the light cone
saturating precisely when it crosses the block. And the ONLY exact closures anywhere
in the record are conservation fibers: the lattice's global chart, HPP's per-line
momenta, FHP-I's exactly three invariants over the full site-dependent space
(`conformance/mesh/LG_RESULTS.md`). So the maximal thesis reads: above the bottom,
thinghood is closure WITHIN A BUDGET, and the budget is a measurable property of the
carrier and the chart. The carrier that limited both rungs (12 atoms, declared-2D, a
16-atom format cap) was stale engineering — now removed — not physics.
*Kills:* a summary on the same trajectories both dynamic and inside budget kills the
disjointness reading; a third chart family disagreeing on the same cells kills
chart-independence; `W(b)` failing at any off-grid block kills the derived law.

**Move 4 — Persistence IS rent, and the payer must know the design. PROVED on the
model, MEASURED on three substrates.** Under decay, paying the decay holds an entry,
underpaying strictly loses, paying nothing tends to zero (`rentStep`,
`rent_closed_form`, `Ginf_at_Wstar`; the sibling seed's `rent_holds`,
`underpaid_shrinks`, `unpaid_decays`). Maintenance CREATES what it maintains — one
repair step on pure noise mints the code's whole-only share exactly, the
flip-symmetric repair mints zero (`Core/Creation.lean`; the sawtooth campaign planted
and found it, dose-response 1.9847 vs 2.000). Maintained holonomy holds a
structure's SIZE exactly and forever — 0.435 of design transport constant to six
decimals to R = 4001 while unpaid decays 65 orders — and loses its IDENTITY
completely unless the repair KNOWS THE DESIGN (fidelity 0.9909 flat vs a power-law
collapse to chance). The one-way valve: under per-cell noise, order flows only UP,
and the pump is asymmetry (`Core/Valve.lean`).
*Kills, one already fired and kept:* closure held at zero maintenance flux; identity
retained under design-blind repair. And on hardware the rent clause's RATE survived
parameter-free within 7% while its exponential SHAPE DIED (χ² 153 vs 26.5, the
substrate is stretched-exponential) — kept dead: the law is a bracket, not an
equality, on real substrates.

**Move 5 — The books are the world. WAGER, and the ceiling of this steelman.**
The banked tables, warm-start carriers and composite rows are PRECEDENT CARRIED AS
CLASSICAL BITS — habit's machine-checked substrate requirement — which is the
dark-matter ROLE (capacity holding the pattern); the receipts, `w_ext` and the
append-only ledgers are the RECORD (dark energy's role); the atoms are the WRITER
(luminous matter). This engine is building the same three-part structure in
miniature on purpose (`TIERS.md`, top rung). It is argued at wager strength and no
higher, because two of its legs are DEAD AND KEPT: the Landauer-normalisation leg
fired at 3–5 dex, and the flow/maintenance rescue fired harder (`flow/stock = λ/3H`,
the failure conserved). What survives is the SHAPE (DESI DR2: Δχ² = −2.13 against
ΛCDM with no ΛCDM limit), with DESI DR3 as the standing kill.

### What the record does NOT license, stated so the steelman cannot be read past it

No formation-rate claim. A model system: minimal basis, classical nuclei (the
ring-polymer coupling is the exit, node E), a two-dimensional certified scene (the
genuine-3D carrier is being built). Closure is statistical over a staked window
with a declared budget, never trajectory-exact, which chaos forbids. The many-body
ladder does not terminate at four (dE5: 24/24 over bound, worst 1,572×) — the design
is arbitrary order with exact cluster solves, and the four-specific assembly in this
tree is a residue being removed, not a claim. Every constant in this engine is a
PRICE measured in a regime, and four were caught this season being inherited across
regimes; the arithmetic-regime law (device class, solver budget, subtraction basis,
and now bead count and bond dimension) is what keeps a price from becoming a wall.

### Where the weight sits — the lead's calibrated read, dated 2026-09-02, MOVED the same day by ACUITY-B

Not a measurement; a judgement over MUTUALLY EXCLUSIVE readings of what a holon most
likely IS, given everything above. Probabilities sum to one and each names the
evidence that moves it.

| reading | p | what moves it |
|---|---|---|
| **A.** The full thesis: closure + paid rent IS thinghood, at every tier, and the books are physical (Move 5 true) | 0.35 | Moves 1–4 all stand; but Move 5's two dead legs and the fact that the only EXACT closures measured are conservation fibers cap it here |
| **B.** Closure + rent is the right account of objects, but a FRAME-SELECTION principle is missing: `exists_closed_view` makes closure cheap (every step closes some view — itself), so what picks the frame in which water is a thing and a fluid cell is not remains unaccounted; the observer's acuity is doing unacknowledged work — **TESTED (ACUITY-B, `conformance/water_observatory/ACUITY_B_RESULTS.md`): the frame selects ALLOCATION, not thinghood; carried-coarse cost the observed thing 0.018 bohr and 4% of a well at 76% of pair work saved, with a measured density crossover where the unobserved region becomes load-bearing** | 0.27 | The upward campaign's vacuity trap; the acuity law's own arithmetic; `frames_are_not_gauge` cuts AGAINST full relativism (frames are an order), which is why this is B and not D |
| **C.** Thinghood is primarily CONSERVATION — closed views are fibers of conserved labels and everything else is budgeted approximation to that; rent is thermodynamics repackaged | 0.22 | LG: the global chart closes by conservation alone and nothing else closes exactly; rung 1's conserved-label lesson; `conserved_descends` reads as the primitive, not a corollary |
| **D.** Closure is real but observer-indexed with no observer-free fact of thinghood | 0.08 | The acuity law's spirit; against it, `FrameOrder.lean` (frames are an ORDER, gauge is only presentation) and the certified molecule's controls (0/111) |
| **E.** The frame is wrong in a way the record already shows: the world-level "rent" is metaphor, and the dead cosmology legs are the tell | 0.08 | The Landauer and flow legs fired; against it, the rent clause's rate surviving on hardware and the design-knowing repair result |

The steelman's own verdict on itself: Moves 1–4 are load-bearing and would survive
E; Move 5 is the bet. The first lever was pulled the day this was written: ACUITY-B
measured the allocation half of B and moved 0.13 of its weight into A. The programme's next measurements that would MOVE this table
are named, not implied — the genuine-3D carrier re-running rungs 1 and 2 (B and C
separate on whether a dynamic in-budget chart appears at scale), node E's quantum
nuclei on the H₂ arm (A and C separate on whether persistence needs more than
conservation), and DESI DR3 (E fires or Move 5 survives).

**The join, wagered:** an object is a shared pattern whose closure pays its own
rent — existence from the square, persistence from the paid step, the receipts
where the books are kept. Each half is backed at its own strength; the join is the
bet, and its kills are separable: closure held at zero maintenance flux, or identity
retained under design-blind repair — either fires alone, leaving both halves
standing.
