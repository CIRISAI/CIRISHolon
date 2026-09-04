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

### The fold below the atom — LOCKED 2026-09-02

*The steelman's five moves say nothing about what lies under the atom. This
section says how the SAME square reaches it, and it is written as a fold of the
existing machinery, never as a new primitive. Status: WAGER, with three separable
kills, each buildable on instruments this repository already holds. The lead's
read that forced it: every rung actually built since the fluid tier found the
holon factoring away the hardest part of the problem and leaving the rest
tractable, and this is where the pattern says the next fold is.*

**The fold, in one line: GAUSS'S LAW IS THE SEAM.** Below the atom the closure of
the tier above is not a budgeted approximation, as it was for the molecule; it is
an exact conservation fiber — the one kind of closure this record has ever
measured exact (Move 3). Three moves, each standing on something already in the
lake or the record:

**Fold I — colour is the seam, and it closes exactly.** A gauge constraint is a
Held, lossy view (`gauss_held`, `gauss_is_lossy` in the sibling seed's
`Core/MatterCoupling.lean`; `Vacuum.lean`'s `vacuum_gauss_zero` here); fiber-internal
dynamics is gauge by the identity commitment (`Identity.lean`: holon identity is
the Moore quotient, exactly as gauge theory treats orbit-internal data); and a
charged pair is a state only when DRESSED by a Wilson line (`BareCharge.lean`).
Composed: the physical state space of a gauge theory is a holon quotient whose
readings are flux strings and their junctions, the electric term (`electricSq`) is
rent per unit of string, and a hadron is a closed string configuration — a colour
singlet. Confinement is the statement that the singlet view is Closed under the
dynamics below the string-breaking threshold, by Gauss's law exactly; so the hadron
tier is a Closed view of the quark–gluon tier in the strong sense the molecule tier
never reached, and its law is FORCED (`closure_determines_dynamics`): the
nucleon–nucleon table should fall out of a two-hadron exact solve the way the H–H
curve fell out of FCI, with no fitted constants. Measured banks are referees only.

**Fold II — the mass gap makes the far field free.** `Core/Locality.lean` and
`Core/Aggregation.lean` (transplanted here as `Budget.lean`) are the warrant: a
locally generated dynamics has a horizon and the state in a region factors through
its neighbourhood, with the quantitative bound the mesh already runs on. Chemistry
carries a 1/r tail, needed the B2 far sector, and still closed. Between colour
singlets the interaction decays like a Yukawa in the pion mass, so the many-body
expansion over hadrons converges exponentially and the closure defect of a hadron
chart is DERIVABLE from the gap — the shape `W(b)` took on the lattice tier, never a
fit. The consumer is the order-generic many-body machinery landed 2026-09-02
(`holon-chem/src/cluster.rs`: any arity, any order, a `PairSource`, a
`SurfaceFamily`, the census as referee), unchanged: species proton and neutron,
tables from hadron solves, the cluster solver in an oscillator basis.

**Fold III — the only priced object is one hadron in its own grain.** Water never
diagonalised the ocean; it diagonalised OHHH clusters and let tables carry the
rest. The hadron version is a box of a few fermi, and `Boundaries.lean`'s
`generic_state_table_absolute` says why that is affordable: the tier ladder exists
because physical states are not generic. The confined phase's two fixed points —
the strong-coupling vacuum (`vacuumConfig`) and the toric-code vacuum — are both
stabilizer states, so the interacting vacuum in a hadron-sized box is a low-magic,
area-law state, and this engine owns all three strata that PRICE it exactly: the
stabilizer tier, the magic tier's exact `Z[ω]` branch sums (the contextual fraction
is the price, measured), and DMRG with defect-priced bond dimension (two orders of
margin on SCHWINGER-3). The arithmetic is this repository's own: the gravity
sequence already runs exact integer gauge theory on finite groups with Gauss as a
quotient (`AdmDescent.lean`, the deficit ladder, the perfect group). Finite
subgroups of SU(3) as link groups are established prior art, credited: Petcher &
Weingarten 1980; Bhanot & Rebbi 1981 (the 1080-element group reproduces SU(3) into
the scaling region); Alexandru et al. 2019 for quantum simulation. Moving those
instruments from a torus to a three-dimensional box is a change of lattice, not of
method. `Grain.lean` supplies the time side: Floquet gauge steps at the grain are
Clifford, so coarse refreshes there carry zero defect, and the C2 real-time carrier
has landed. Staggered quarks suffice: the Nielsen–Ninomiya wall is chiral-only, and
it stays named in `LEPTON_LADDER.md` for the electroweak sector, untouched here.

**What the fold does NOT claim.** No hadron has been computed. The three moves are
a wager about a NEW tier; they move none of the weights in the table below. The
lepton is an input to QED, not an emergent object, and no move above says otherwise.
"Exact-first" is a statement about arithmetic and Hamiltonian formulation (no sign
problem is posable here at all), never about affordability: Fold III's price is
measured by its own kill.

*Kills, separable, each buildable now (GANTT nodes GF0–GF2):*
- **GF0 — SCHWINGER-4.** The residual interaction between two screened static
  charge pairs in QED₂ must decay exponentially at the banked vector-meson mass —
  a defect DERIVED like `W(b)`. Slower kills Fold II in one dimension before any
  three-dimensional cycle is spent; faster kills only the identification of the
  exchanged state and is reported. **Read 2026-09-02 on the engine arm: branch (a)
  on both columns, the rate at the gap to 0.6 % (`SCHWINGER4_RESULTS.md`); the
  Python cross-check per amendment A1 is the one open condition.**
- **GF1 — the magic price of gauge vacua.** The stabilizer extent (and its
  measurable proxies) of Z₂ and Z₃ lattice-gauge ground states across the coupling,
  on the exact tiers, in boxes growing toward a hadron's grain. If the price escapes
  with volume, Fold III dies.
- **GF2 — the finite-subgroup hadron box.** Σ(1080)-valued links with staggered
  quarks in a small three-dimensional box on the finite-group instruments, against
  the proton, neutron and pion masses and the deuteron. If the DERIVED hadron pair
  table misses its referees at the staked band, Fold I's "forced, not fitted" dies
  for this carrier.
- **GF2a — the 1+1D rehearsal of GF2's instrument (E7 → E14), READ 2026-09-04.**
  The labelled MPS arm met the exact colour-lane referee on all six N = 8 sectors at
  a χ set by the cut's RANK (misses ≤ 1.6e-9, cold = warm-mixed to 1e-9); the plants
  fired as staked — the labels are load-bearing by the sharpest route, the unlabelled
  mutant LEAVING its sector to sit 3.93 below its own variational bound on the
  neighbouring sector's ground state to 3.1e-6 — and the two-site variance is the
  arm's own error bar (108× between two states whose residuals agree to 2 %). The
  volume ladder past N = 8 is **CLOSED AS PRIOR ART, not as a result** (Silvi, Sauer,
  Tschirsich, Montangero, PRD 100 074512, 2019; Hayata, Hidaka, Nishimura,
  arXiv:2311.11643; neither reports runtime, none is compared): G1′, G2, G3 unread by
  A1.6 and withdrawn. What it banks for Fold III's PRICE: χ is the sector's rank at
  the middle cut and the label count grows as (N/2+1)³, so a hadron sector's price is
  the number of labels that CARRY WEIGHT — the measurement GF2 owes before its box is
  priced. And the DRY register's newest entry got its first exact-vs-budgeted reading:
  the penalty-enforced sector (a budgeted closure) failed where the exact label fixed
  it (`conformance/crystal/GF2A_QCD2_RESULTS.md`, closing section).

### The surface, audited — what of the workbench is the holon (2026-09-04)

*The operator's question, asked after the zoom went fluid: "how much of this has been
tacked on vs using the holon maximally?" Answered per band from the doors, not from
the cards. The rule applied: a band's LIVE means its doors resolve, never a
certificate; the band-flip law (a band is a tier only on a closure certificate against
the dynamics beneath it) is untouched, and none of the three fine bands passes it.*

| band | what is drawn or read | holon-coupled? | verdict |
|---|---|---|---|
| molecular | the certified scene: pair curves from FCI, the (O,H,H) surface, the census, the closure certificate, ACUITY-B's allocation as the cut | yes — it IS the holon | maximal |
| atom | the SOLVE (`holon_atom_band_solve`): STO-3G FCI of the CENSUS MOLECULE at the scene's own geometry — energy, electron count, residual, exit | yes: the census picks the members, the scene supplies the geometry | the engine's bottom exposed, honestly — a substrate, not a tier |
| atom | the drawn cloud SIZE: the pinned molecule's atoms at the molecular solve's OWN density, Mulliken-partitioned to each about its nucleus (`holon_atom_band_coupled_rms_bohr`, on EMBED-1's density, DONE 2026-09-04: H in H₂ reads 1.506 bohr against the free atom's 1.396); every other atom at its free-atom size, labelled | yes for the pinned molecule; no for the rest, and the ring says which | step 1 of the path below, closed the same day; what remains decoupled is labelled |
| nucleus | Z, isotope, mass, spin, charge radius | no — DECLARED inputs (WB-1.7), labelled so | honest input; nothing here is a coarse view of anything |
| nucleus | the thermal de Broglie wavelength | half: the holon's measured temperature into a closed form | a formula standing in for the MEASURED spread node E's ring polymer would give; **tacked on until node E is wired** |
| the fold | one baryon's quark density on a 6-site 1+1D SU(3) chain, exact, with a grab that quenches and a unitary clock | the SOLVER is the holon's lane kernel (the shard law); the OBJECT has no seam to the nucleus above it and produces no table | **a model, credited as prior art on its face**; it becomes a tier only through GF2 |
| the view (two-box, fluid zoom, descent glide, filmstrip) | presentation of the cut ACUITY-B defined | it draws the holon's own view `v` and moves nothing in the physics | machinery, correctly placed — the one place the day's work was maximal by construction |

Count: of the seven bands three are fenced on node G, one is the holon, one is the
substrate exposed with a decoupled picture, one is declared, one is a model. The
surface below the molecule is therefore mostly NOT the holon yet, and the page now
says so on each card. The maximal-use path is three named steps, none new machinery:
(1) the atom's drawn size from the molecular solve's density (Build 1's one-body
density, reused — DONE the same day); (2) node E's ring-polymer spread into the nucleus band in place of
the formula; (3) the fold stays a model until GF2 gives it a seam — and is labelled a
model until then.

### The lock — how this steelman may be extended, and how it may not

The steelman is LOCKED as of this section. It changes in exactly two ways:

1. **A move's own kill fires.** The move is marked dead and kept, its weight moves,
   nothing is rewritten around it (rule 7 of the seed's discipline).
2. **A FOLD is added.** A fold is the same square applied to a carrier or tier it has
   not yet been applied to. To be admitted it must name, in this document, all four
   of: the EXACT fiber it closes on (or the budget it closes within), the DERIVED
   form of its defect (never a fit), the one object whose solve is PRICED and the
   stratum that prices it, and a separable kill buildable on instruments this
   repository holds. A fold that needs a primitive not derivable from the existing
   signature is a format replacement under W3's frozen extension grammar
   (`STANCE.md`) and is refused as a fold — it may be proposed only as a new move,
   at wager strength, with its kill.

**The DRY register — where the object has already folded into itself.** Kept so the
next fold is found by looking here first, not by re-deriving: closure ≡ never-split
(`viewClosed_iff_never_splits`); conservation ≡ the only exact closure (Move 3);
curvature ≡ paid-up rent on the transport map (`curvature_iff_held`); back-reaction
≡ mutual non-closure (`MatterCoupling`); gauge ≡ the Moore quotient's fiber
(`Identity.lean`); the classical tier ≡ the diagonal retract of the quantum carrier
(`DiagonalLift`); magic ≡ the sixth wall, non-closure of the tableau view
(`Stabilizer.lean`); the grain ≡ the Clifford angle of a Floquet gauge step
(`Grain.lean`); the string tension ≡ rent per link (`electricSq`, Fold I); the mass
gap ≡ the far field's closure (Fold II); the hadron ≡ the closed string (Fold I);
the observer's frame ≡ allocation, not thinghood (ACUITY-B); **a conserved integer
lane ≡ a shard** (the swarm's arena law at the hadron tier: a many-fermion sector with
`k` commuting counts is a product of `k` occupation strings, every Hamiltonian term
moves within a lane or couples two by one single each, so the solver shards on a lane
with no halo — chemistry's alpha/beta is `k = 2`, SU(3)'s Cartan-neutral colour
block is `k = 3`, and the ENTIRE determinant engine now runs on that one kernel, host
and device bit-identical; `holon-chem/src/lanes.rs`, 2026-09-02). **The embedding field ≡ the Record** (the operator's squint, 2026-09-04, on EMBED-1's
logic): the sibling's taxonomy of change is eleven ARTIFACT-LOCAL kinds plus ONE
frame-relation, Record, provably not generated by any site (`record_not_site_generated`)
and not factoring over the parts (`repairable_does_not_factor`) — whether the past can be
proven depends on what survives AROUND the artifact. The embedded expansion has that exact
shape: every fragment solved in vacuum generates its own local kinds (its Facts, its
Model, its Structure, its Process), and no fragment can generate the field it sits in;
the field is the one relation to what surrounds it, counted ONCE as the cross term the
fragments share (`E_qq`), which is why the bare ladder never terminated (dE5: the eleven
without the +1) and the embedded one closes at two-body (Gillan 2013: the +1 supplied).
Two sharper correspondences, both MEASURED on 2026-09-04: plant (ii) — counting the frame
term inside EACH fragment — reads `ρ ≈ 1` on every far node, which is the taxonomy's
retracted 10+1+1 coordinate (Record demoted to a twelfth local kind, counted per artifact)
failing by arithmetic rather than by panel; and the self-consistent fixed point is the
declaration corner of the fit square (`declaration_is_double`): each fragment's charges
are at once a reading of the field and its source, both directions of fit at zero depth,
and G4 measures that corner as start-independent to 2e-12. The Mulliken control is the
Model/Facts twin confusion priced: it carries the manner of assigning charge and not the
fact (the dipole), and pays `ρ = 0.10` flat for it. Philology, since the operator asked:
em-BED (a thing laid in a ground), re-CORD (brought back to the heart from OUTSIDE the
utterance), FIELD (the open ground a SITE stands on) — the words already carry the split
the theorems draw: the thing, and the ground it is proven in. A reading, not a fold: it
adds no primitive and names no new kill. Every entry is one
square wearing different clothes; a candidate fold that is not on this register and
cannot be written as one of its entries composed is the signal to look harder, not
to add a primitive.

### What the record does NOT license, stated so the steelman cannot be read past it

No formation-rate claim. A model system: minimal basis, classical nuclei (the
ring-polymer coupling is the exit, node E), a two-dimensional certified scene (the
genuine-3D carrier is being built). Closure is statistical over a staked window
with a declared budget, never trajectory-exact, which chaos forbids. The many-body
ladder does not terminate at four (dE5: 24/24 over bound, worst 1,572×) — the design
is arbitrary order with exact cluster solves, and the four-specific assembly in this
tree is a residue being removed, not a claim. **Measured 2026-09-04 (EMBED-1, SEAM-1):
the ladder that did not terminate was BARE.** Solved inside the mutual point-charge field
of its partners, the pairwise expansion carries 99.93 % of the three-body term on the HF
chain's far sector against the exact trimer (S1′ branch (b) on measured ground: the part it
misses, one in 1,300 to 2,000, decays more slowly than the term at 5–6 Å, and the next
freeze stakes the field as the partners' densities). At the hydrogen-bond distance a third
of the term is uncarried — non-additive exchange, which is what exact cores are for. Every constant in this engine is a
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
measured the allocation half of B and moved 0.13 of its weight into A. **MOVED AGAIN
the same day by the three folds** (the colour lanes, the port, the Davidson on a vector
space under one reduction law): in each, the frame was selected by a conserved label
and by nothing else, and the one BUDGETED closure run that day — the MPS arm's penalty-
enforced sector — failed where the exact label is the fix. Read now: A 0.36, B 0.20,
C 0.28, D 0.08, E 0.08. B keeps 0.20 on rung 1's alignment finding, a frame puzzle
still unsolved; A still leads C on the one measured non-conservation closure that held
with controls (the certified molecule). The programme's next measurements that would MOVE this table
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

**2026-09-04, the table UNCHANGED.** The season since 09-02 produced base machinery
(E14), a rehearsal closed as prior art (GF2a), and a surface audited above — none is a
measurement that separates the readings. The one reading added, "four physics, one
object test, four different answers" (`TIERS.md`; the sibling's `four-from-one` claim),
sharpens Move 1's INSTRUMENT — the same closure test discriminates across carriers —
without moving any weight. A 0.36, B 0.20, C 0.28, D 0.08, E 0.08 stand.
