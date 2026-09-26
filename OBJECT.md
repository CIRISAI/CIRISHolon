# The object — the engine's contract

*Engineering statement, current stance only. The research programme behind it, with its
measurement record, lives in CIRISAI/CIRISOntology; every claim here names a
machine-checked theorem in `lean/CIRISHolon/Object.lean` (or the sibling seed's `Core/`) or
a conformance obligation testable in CI. History lives in git and is not repeated here.
Rewritten 2026-09-04 at the operator's order — the lock below was opened for this
clarification pass and closes again with this commit; no claim's strength moved except
where a measurement moved it, and each such move names its record.*

## One question

A **view** is a lossy reading `v : X → C` of a state space. A **motion** `T : X → X` is a
dynamics step, a re-root, or a chart change. The contract is the commuting square:

```
      X ──T──▶ X
      │        │        Closed v T  ≔  ∃ h, v∘T = h∘v
      v        v        Held   v T  ≔  v∘T = v
      ▼        ▼
      C ──h──▶ C
```

**A tier is a Closed view of the tier below.** That is the entire definition. `h` is the
coarse dynamics the engine runs in the tier's place; `Held` is the special case of
invariants and paid-up maintenance.

**Failure has a normal form** (`nonfactoring_iff_not_closed`): a view fails closure exactly
when two states it cannot distinguish are sent by the motion to distinguishable readings — a
*witness pair*. Every conformance test in this engine hunts witness pairs; every bug report
against a tier should ship one.

**The gap is the value.** The witness pair's difference is not an error term; it is the
quantity the next view must carry (`viewClosed_comp`: closed views compose). Measured on
2026-09-04 in chemistry: the residual of an embedded two-body view IS the three-body
dispersion energy, harvested to a `1e-12` floor (`EMBED2_RESULTS.md`). This is the same fact
the ladder of tables has always used: a table is a harvested residual.

## CORRECTION 2026-09-25 — the philological backpass (the owner's order; locked text above and below kept legible)

*After five carriers, one head-to-head (`SLOW2_RESULTS.md`) and two re-findings with prior art
found late (`GF1_RESULTS.md`, `SLOW1_RESULTS.md`), four terms above no longer carry what they
were written to carry. Each is corrected here, dated, beside the sentence it corrects.*

1. **"A tier is a Closed view of the tier below. That is the entire definition."** It is not the
   entire definition, and the record says so twice: the hydrogen-bond network is a Closed
   view the fluid does not carry (`TSCAN_HBOND_SEARCH_RESULTS.md`, three walks, increment
   exactly zero), and a learned coordinate slower than tetrahedral order carries less of the
   mobility than tetrahedral order does (`SLOW2_RESULTS.md`). Closure is cheap, as
   `exists_closed_view` always said. **Corrected: a tier is a Closed view of the tier below THAT
   THE TIER ABOVE CARRIES** — one its conservation law cannot remove within its budget (design
   rule 11). Move 1 below, "Existence IS closure", reads henceforth "existence is closure;
   thinghood is closure the level above carries": the theorem proves the first half; the
   second is measured, separately, on the fluid (the face chart chosen by the momentum law
   over the closure score, `RESPONSE1_AMENDMENT_5.md`) and on the network (rejected by the same
   law). The steelman's Move 1 is not dead; it was over-read.
2. **"The object is FOUND rather than named."** True as arithmetic, false as a distinction. The
   search that finds the closed sectors is TICA/VAMP (Pérez-Hernández–Noé 2013, Wu–Noé 2020):
   on the same features, lags and target it returns the same direction as ours to `10⁻⁴`
   (`SLOW2_RESULTS.md`, C1). **Corrected: the object is found by the standard slow-mode search
   and SELECTED by the level above.** What is ours is the selection, the plants, the stakes
   frozen before the run, and the closed-versus-carried split measured as two numbers. The
   search added nothing over TICA on any carrier where the comparison was run.
3. **"Rent."** The word does two jobs above: the maintenance flux persistence pays (Move 4,
   `rent_closed_form`) and, since the QVM campaigns, the computational price of the hard part.
   The second is a COST, not a rent; nothing this month tested the first. **Corrected: PRICE for
   what a certificate charges (`2^{M₂}`, the remainder, the discarded weight); RENT for what
   persistence pays.** Where "rent" appears in a QVM record it means price.
4. **The calibrated read** ("Where the weight sits", below) was written on 2026-09-21. Readings A
   and B are not rivals: B is A with one named unknown — the selection principle — and the
   month's evidence is that closure is cheap and selection is the whole game. **Corrected table
   (the 2026-09-21 table kept beside it):**

   | reading | p | what moved it |
   |---|---|---|
   | **A′.** The thesis, with the selection principle as its named open item: closure the level above carries, paid rent, physical books | **0.55** (A 0.40 + B 0.10 merged, raised) | the network and the slow-but-uncarried coordinate are direct evidence that closure alone is not thinghood; the face chart is one measurement that the law above selects; the principle has a candidate and no test |
   | **C.** Thinghood is primarily conservation; rent is thermodynamics repackaged | 0.32 | the face chart was chosen by the conservation law and was right — C and A′ read that the same way; the tie stands |
   | **D.** Closure is observer-indexed with no observer-free fact | 0.08 | cadence and sample dependence (one seed of three breaks isotropy at one-cycle cadence) is the acuity law, not observer relativity |
   | **E.** The world-level rent is metaphor | 0.05 | untouched; the QVM's "price" is not rent |

   **The operational reading, at the strength the evidence carries** (not a rival row; the
   sentence every measurement this month supports): *a holon is a certified sufficient
   statistic at an acuity — the smallest closed view the level above cannot remove within its
   budget, with a price for what it drops.* Under it the tools are TICA, SPIB and Mori–Zwanzig
   with a stricter discipline, and the instrument, after five carriers, re-finds and has not
   discovered.

5. **Move 4's join was MEASURED on the fluid this month and I did not count it** (2026-09-25,
   later the same day). RESPONSE-1's R3: the driven shear rent — the decay of a driven closed
   mode — reads `η = 4.4 × 10⁻⁴` Pa s, equal to the equilibrium read on the same box: the
   maintenance rate of a driven closure is the equilibrium price, the fluctuation–dissipation
   relation in the object's clothes, and the physics face of "the rate is not a free
   parameter". And R1: the Record — what the cell chart cannot generate — leaked through the
   faces and the faces carried it (`0.18` on the graded average). The 5 + 1's join held on
   water; the 11 was never in play there (no trajectory of oxygens distinguishes a Fact from a
   Process) and is in play only on the reasoning carrier, where the kinds are native and the
   test waits on thought text. **The weak carried direction — the non-conserved variable the
   level above carries anyway (tetrahedral order; the conscience sector) — is the ORDER
   PARAMETER, Landau's, and its blind hunt on the other tiers is `ORDER-1` (frozen 2026-09-25).**

**The known unknown, named as a hunt.** A′ and C differ on one thing: whether "the level above
carries it" is a fact about conservation alone or about paid maintenance. And A′ itself owes
the selection principle as ARITHMETIC: a rule that, given the closed sectors of a tier and the
law of the tier above, names the object BEFORE the tier above is run — measured once (the face
chart) and asserted everywhere else. The freeze that would settle it (`SELECT-1`, not yet
written): state the rule (carried ⇔ not removable within the level-above budget under its
law), apply it blind to the six closed sectors already on disk (the two molecule sectors, the
network, the cell and face charts, the VAMPnet's coordinate) and to the Clifford light cone,
with the network and the uncarried coordinate as the plants it must REJECT and the face chart
and the light cone as the ones it must KEEP; kill: one wrong call. It costs no trajectory.

## The procedure, as arithmetic — the one fold the three measurements share (2026-09-21, locked at the owner's order)

*Provenance, line by line: `Closed` and composition are PROVED (`Object.lean`, `StatClosure.lean`);
the bound is ADOPTED (the variational principle for Markov processes, Wu & Noé 2020, with TICA and
Markov-state models before it) and credited; relevance and the +1 are DEFINED here and MEASURED
on three rungs. The prose that preceded this section on 2026-09-20/21 is folded into the instance
table below; nothing it claimed has moved.*

```
SETTING
  X            the fine state space; T : X → X the step; τ the cadence (a lag)
  Φ : X → R^d  the dictionary — DECLARED, one of the two human choices (τ is the other)
  a view       v = Wᵀ Φ,  W ∈ R^{d×k};   its law h = least squares  v(x) ↦ v(T x)
               [Closed v T ≔ ∃ h, v∘T = h∘v]

CLOSURE  (adopted)
  C₀₀ = E[Φ Φᵀ],  C₀τ = E[Φ (Φ∘T)ᵀ],  Cττ = E[(Φ∘T)(Φ∘T)ᵀ]         ridge on the diagonals
  K   = C₀₀^{-½} C₀τ Cττ^{-½},   singular values σ₁ ≥ σ₂ ≥ …
  score(v) = Σᵢ σᵢ(K_v)²           K_v the same construction on v's own coordinates
  bound(k) = Σ_{i≤k} σᵢ(K)²        attained by the top-k left singular subspace of K
  D_v²     = 1 − score(v)/k        the relative residual of v's law, whitened;  closed within budget: D_v ≤ β
  fraction(v) = score(v)/bound(k), HELD OUT (train covariances; the test data's own)
  sectors  blocks A, B of Φ with ‖K_AB‖/‖K‖ < ε are decoupled — each closed on its own
  null     the time-shuffled (or cross-chain re-paired) Φ: score → 0

RELEVANCE  (defined; the level above declares it)
  the level above carries Q (conserved densities) under a law L(Q, J) = 0
     e.g. continuity   ΔN_c = − Σ_{faces f of c} dir_f ∫_window J_f dt
  for each representation R of J in the dictionary:
     ρ_R = held-out R² of L's prediction of ΔQ from R;   α_R = ⟨ΔQ·pred_R⟩ / ⟨pred_R²⟩
  carrier of J = argmax_R ρ_R          (NOT argmax closure — the two rank oppositely on the fluid)

THE OBJECT at scale (a, τ)
  (Q, M, J):  Q the densities the level above carries;  M the search's closed sector that
              carries the momentum law;  J the flux the conservation law selects
  the +1      J ∉ I(M), I the reconstruction of faces from slabs: ‖J − I(M)‖/‖J‖ is the part the
              local pieces do not generate, counted once — measured as 1 − α_{I(M)}
  the price   of dropping a sector = ‖K_AB‖/‖K‖;   of a coarser chart = 1 − α

COMPOSITION  (proved; locality on the cell graph is the hypothesis Leg B tests)
  pooled D² over cells = Σ O_c D_c² / Σ O_c ≤ max_c D_c²
  ‖Tⁿx − hⁿx‖ ≤ ε Σ_{i<n} Kⁱ ≤ n ε   for a non-expansive law
```

| rung | Φ, τ | the search found | the law selected | fraction of the bound / price |
|---|---|---|---|---|
| molecule (`MOLSEARCH1_RESULTS.md`) | internal coordinates and rates, COM and angular velocity; 5–100 fs | two sectors, cross `0.02`; the vibrations the MORE closed | the rigid unit — mass and momentum are what the fluid carries | rigid view `0.09` of the bound at 50 fs; price `0.02` |
| fluid, the chart (`VIEW_SEARCH_RESULTS.md`) | cell fields and first-harmonic modes; 100 fs | the density and current modes, at the bound, no chart named | the cell chart, because it sums to the fields | cell chart `0.70–0.78` |
| fluid, the flux (`RESPONSE1_AMENDMENT_5.md`, `auto_dictionary.py`) | occupancies, slab, face and lagged momenta; 200 fs | the occupancy pattern and the LAGGED slab drift; slab at `1.00` of the bound | the `0.25 Å` face flux: `ρ = 0.69` against the slab's `0.07`, monotone in the face width | face view `0.78–0.85` of the bound; price of reconstructing it from slabs `0.55` at one diameter, `0.8` at half, `0` at two |

**What the table says in one line.** Closure is found by the search; the object is the closed
sector the level above carries; the two rankings can oppose each other within one tier, and
when they do the law decides (the fluid's flux: the bound ranked the slab first and the
conservation law ranked it last). The residual of a chart names its successor: the cell chart's
leak was the layering, and the chart that carries the leak closes (`D 0.735 → 0.164` on the
200 m/s control). **Two gaps the formula exposes, open:** relevance is defined per conservation
law — a level whose law is a CONSTRAINT rather than a conservation (the reasoning tier: the
covenant, the coherence ratchet's target) needs `ρ` redefined as the carrier's held-out
prediction of constraint satisfaction, and that is the A3+ tier's object; and whether the
selected carrier must also sit within closure budget is open — the fluid says no (`0.78`).

## The shape, stated four times

The object is one shape wearing four sets of clothes. Each is machine-checked where it says
so and a reading where it says so.

| the shape | as | the split | where |
|---|---|---|---|
| **the square** | closure | what the view keeps · what the motion does to it | `Closed`, `nonfactoring_iff_not_closed` |
| **11 + 1** | the taxonomy of change | eleven ARTIFACT-LOCAL kinds (Priorities, Rules, Manner, Identity, Confidence, Facts, Circumstances, Process, Model, Structure, Premises) · ONE frame-relation, Record — whether the past can still be proven depends on what survives AROUND the artifact | `record_not_site_generated`, `repairable_does_not_factor` (sibling `Core/Generator.lean`, `Core/WrongKind.lean`) |
| **5 + 1** | the steelman | five moves a reader can check on the artifact alone · one join that relates them to what surrounds — the rent that pays for the closure | this document, below |
| **non-trivial holonomy** | transport | carry a pattern around a loop; the return is not the identity · the difference is not noise, it is the curvature — paid-up rent on the transport map — and the value the next tier carries | `curvature_iff_held`; the maintained-holonomy measurement (Move 4); the harvest (above) |

The three "+1"s are the same object: the frame relation of the taxonomy, the join of the
argument, and the holonomy of the transport are each *the part that is not generated by the
local pieces and does not factor over them, counted once*. EMBED-1 measured that
arithmetic directly: count the field inside each fragment (the retracted 10+1+1 coordinate)
and the expansion reads `ρ ≈ 1`; count it once and it reads `1e-2` (`EMBED_RESULTS.md`,
plant ii). The self-consistent fixed point is the declaration corner of the fit square —
both directions of fit at zero depth (`declaration_is_double`) — measured start-independent
to `2e-12`. A reading, marked as one: it adds no primitive and names no new kill.

## The design rules, each backed by a theorem or a measured law

1. **Closure is certified, not assumed.** A tier ships with its battery: construction premise,
   budget (coarse divergence growth ratio ≤ 1.05 over its rise epoch), witness-pair hunt.
   *Added 2026-09-20:* and the certified view is found and selected, never guessed (rule 11).
2. **Exact closure is not expected; budgets are.** The claim is never "zero leak"; it is
   "non-expanding leak within the stated budget" (`Budget.lean`).
3. **Charts declare their conditioning** (`sum_perturb_le`, `sum_perturb_attained`,
   `coherence_of_nonneg`): a near-cancelling aggregate amplifies per-term perturbation by
   1/coherence, exactly; ill-conditioned charts are diagnostics, never state.
4. **Privilege is two-dimensional**: a coarse quantity's reality = chart conditioning ×
   whether the dynamics organises divergence into it. Both measurable, neither assumed.
5. **Maintenance is rent-priced** (`rentStep`, `Ginf`, `Wstar`, `rent_closed_form`,
   `Ginf_at_Wstar`): retention under decay `lam` and dose `q` is `q/((1−lam)+q·lam)` at the
   fixed point. Two measured riders: multi-mode decay makes the law a *bracket*; **the repair
   must know the design**.
6. **Identity is arena-level and append-only.**
7. **One gate per conserved quantity, chart-relative**; a balance gate refuses where its chart
   has no time-translation symmetry.
8. **The quantum relation is the retract, never a bolted-on mode** (`bornView_diagEmbed`,
   `liftChannel_factors`, `lift_commutes`, `diag_view_closed_of_classical`,
   `diag_not_closed_under_coherence`): the classical tier is the diagonal, Born readout is
   Closed with the classical step, and coherence is exactly where the classical tier ends.
   Simulation strata follow: classical, stabilizer, bounded-contextuality (priced by the
   contextual fraction), tensor-network (defect-priced bond dimension), and past the wall,
   known-exponential cost, delegation, or refusal by name.
9. **Refusal is a feature.** A tier outside its certified scope refuses, naming the gate
   whose passing would lift the refusal.
10. **The interaction is a LEDGER OF CHANNELS, each with a derived rate, and precision is an
    allocator over them** (locked 2026-09-04 at the operator's order, read off EMBED-1/2/3 and
    SEAM-1). Five channels on the record, five rates: fixed-multipole electrostatics (the
    field, `R⁻¹`/`R⁻³`), induction (the field's fixed point, `R⁻⁴`/`R⁻⁶`), pair dispersion
    (inside the exact pair, `R⁻⁶`), three-body dispersion (the harvested residual, `−C/R⁹`),
    and exchange with penetration (the exact core, exponential, gone by 3 Å). COMPLETENESS is
    the field counted once (the +1 that no fragment generates and that does not factor):
    with it the channels sum to the exact energy, so a residual is ASSIGNED by its exponent
    rather than hunted. SEPARABILITY is a channel's coefficient not depending on the others
    beyond a measured coupling — MEASURED 2026-09-05 for channel 4 inside channel 1: three
    parts in a thousand at 6–12 Å against the frozen reference, at the arithmetic floor
    beyond 8 Å against the in-process one (`EMBED3_RESULTS.md`, System A, branch (a)). Under both, the dynamics is the gradient of known transfers between fragments —
    forces from Hellmann–Feynman with no second solve — and the cost of a scenario is fixed
    by the digits it asks for: at precision `ε` evaluate exactly the channels with
    `C_k R^−n_k > ε`, and solve a core only inside channel 5's reach. The engine's energy
    rows are the channels and its receipt columns are the transfers; FIELD-1 lands the
    field as channel 1 of that ledger. *Kills:* a residual whose exponent matches no
    channel; a channel coefficient that depends on another's field at more than one part in
    ten where the freeze staked less; and contact, where the channels do not separate and
    the core stays irreducible — which is a bound, not a defect.

**Tables** are produced only through `holon-tables`' leased generator — probed leases,
receipts, a merge-digest certificate, a launch header with the binary's sha256 and the
build's exit status; bit-identical across worker counts and profiles; the caller supplies the
physics and the layer refuses to invent it (`engine/RESOURCE_DESIGN.md` §11).

**Conformance obligations (CI, per tier):** the closure battery; chart-conditioning
declarations for every exposed aggregate; per-conserved-quantity gates with planted-mutation
sensitivity (a gate that cannot fire on a plant is refused); for quantum strata, the retract
test and the QASM suite to the stratum's declared boundary.

11. **The view is searched before it is staked, and the level above selects it** — "The
    procedure, as arithmetic" above: search on a declared dictionary at the tier's cadence,
    a named chart banked with its fraction of the bound, the carrier of each law chosen by
    `ρ`, the certificate (held out, the time-shuffled null, composition) separate from both.

## The maximal steelman — five moves and the join, each with its kill

*The strongest honest case, at full strength. Statuses: PROVED (machine-checked here or in
the sibling seed), MEASURED (named record), WAGER. Every move carries the observation that
would kill it; a steelman that cannot die is advocacy.*

**The thesis.** An object is a shared pattern whose closure pays its own rent. A thing is a
lossy summary the dynamics never splits; its law is forced, not fitted; things stack in
tiers; conservation descends the tower for free; it persists exactly as long as its
maintenance is paid, by a payer that knows the design.

**Move 1 — Existence IS closure. PROVED.** `Closed v T ≔ ∃h, v∘T = h∘v`; failure is a witness
pair (`nonfactoring_iff_not_closed`); the coarse law is unique on reachable readings
(`closure_determines_dynamics`); closed views compose (`viewClosed_comp`); an invariant of
the coarse law is conserved by the fine dynamics (`closed_view_inherits_conservation`,
`conserved_descends`); maximality is root-relative (`Omega.lean`). The founding shape — two
wholes agreeing under every partial view, differing in the quantity — is one object witnessed
four times (`Core/NonFactoring.lean`: parity, the CP phase, the Record; `Core/ExchangeSign.lean`:
fermion vs hard-core boson).
*Kill:* a machine-checked definition of thing under which something stays a thing while
failing closure at every window. None exists.

**Move 2 — Closure is REALISED from first principles, and the instrument can say no.
MEASURED.** From nuclear charges, masses and per-encounter exact diagonalisation — no fitted
potential — a water molecule assembled itself and passed the closure test staked before the
instrument existed: longest closed run 893.8 fs against a pre-staked 834 fs window, 72.3 % of
the trajectory, 0 of 111 look-alike groupings reaching the window
(`CENSUS_PREREG.md`, `CENSUS_RESULTS.md`); the exact four-body arm certifies the same molecule
strict and the four-body-absent control certifies on its own (§14).
*Kills, separable:* regeneration from the committed protocol failing to reproduce a certified
block; the certification failing its pre-registered successor floor.

**Move 3 — Closure above the bottom is NOT free, and the price has a DERIVED shape.
MEASURED, twice.** Rule 2 says budgets, and the record now carries two defect laws that were
derived, not fitted: on the founding lattice tier the block's boundary fraction
`W(b) = 1 − max(0,b−2)²/b²`, exact at every measured point across all 4,608 lawful collision
laws (`conformance/mesh/LG_RESULTS.md`); and in chemistry the far-field defect of an embedded
two-body expansion, `r = −C/R⁹` with `C = 8.5 Ha·bohr⁹` on the HF chain — three-body
dispersion, one constant fitting three nodes to a few per cent, floor `1e-12`
(`EMBED2_RESULTS.md`). The bare many-body ladder that did not terminate (dE5: 24/24 over
bound, worst 1,572×) was bare: solved inside the field of its partners the pairwise expansion
carries 99.93 % of the three-body term on the far sector (`SEAM_RESULTS.md`), and what remains
is fast-decaying and tabulable. The only EXACT closures in the record are conservation fibers
(the lattice's global chart, HPP's per-line momenta, FHP-I's three invariants); the first
upward rungs found in-budget and dynamic charts exactly disjoint, the boundary being
alignment (`RUNG1_RESULTS.md`, `RUNG2_RESULTS.md`).
*Kills:* a summary on the same trajectories both dynamic and in budget kills the disjointness
reading; `W(b)` failing off-grid, or the `R⁻⁹` law failing on a second carrier, kills the
derived-defect reading for that tier.

**Move 4 — Persistence IS rent, and the payer must know the design. PROVED on the model,
MEASURED on three substrates.** Paying the decay holds an entry, underpaying strictly loses,
paying nothing tends to zero (`rent_holds`, `underpaid_shrinks`, `unpaid_decays`; here
`rent_closed_form`, `Ginf_at_Wstar`). Maintenance CREATES what it maintains (`Core/Creation.lean`;
the sawtooth campaign planted it: dose-response 1.9847 vs 2.000). **Non-trivial holonomy,
maintained:** upkeep holds a structure's SIZE exactly and forever — 0.435 of design transport
constant to six decimals to `R = 4001` while unpaid decays 65 orders — and loses its IDENTITY
unless the repair knows the design (fidelity 0.9909 flat vs a power-law collapse to chance).
Curvature is paid-up rent on the transport map (`curvature_iff_held`). The one-way valve: under
per-cell noise order flows only up, and the pump is asymmetry (`Core/Valve.lean`).
*Kills, one fired and kept:* closure held at zero maintenance flux; identity retained under
design-blind repair; and on hardware the rent clause's RATE survived parameter-free within 7 %
while its exponential SHAPE died (stretched-exponential substrate) — the law is a bracket.

**Move 5 — The books are the world. WAGER, the ceiling.** Banked tables and warm-start
carriers are precedent carried as classical bits (the dark-matter ROLE); receipts and
append-only ledgers are the RECORD (dark energy's); the atoms are the WRITER. Two legs are
dead and kept (Landauer normalisation at 3–5 dex; the flow/maintenance rescue); what survives
is the shape (DESI DR2 Δχ² = −2.13 against ΛCDM), with DESI DR3 as the standing kill.

**The join — WAGER.** An object is a shared pattern whose closure pays its own rent: existence
from the square, persistence from the paid step, the receipts where the books are kept. Its
kills are separable and do not touch the halves: closure held at zero maintenance flux, or
identity retained under design-blind repair.

## The fold below the atom — three folds, LOCKED 2026-09-02

*How the same square reaches under the atom, written as a fold of existing machinery, never a
new primitive. WAGER with three separable kills; no hadron has been computed.*

- **Fold I — colour is the seam, and it closes exactly.** A gauge constraint is a Held, lossy
  view (`gauss_held`, `gauss_is_lossy`; `vacuum_gauss_zero`); fiber-internal dynamics is gauge
  by the identity commitment (`Identity.lean`); a charged pair is a state only when dressed
  (`BareCharge.lean`). A hadron is a closed string; the electric term (`electricSq`) is rent per
  unit of string; confinement is the singlet view Closed below string-breaking, by Gauss's law
  exactly — so the hadron tier's law is FORCED (`closure_determines_dynamics`).
- **Fold II — the mass gap makes the far field free.** A locally generated dynamics has a
  horizon (`Core/Locality.lean`, `Budget.lean`); between singlets the interaction decays like a
  Yukawa in the pion mass, so the hadron expansion converges exponentially and its defect is
  derivable from the gap — the shape `W(b)` and `C/R⁹` take elsewhere.
- **Fold III — the only priced object is one hadron in its own grain.** Both confined fixed
  points are stabilizer states (`vacuumConfig`, the toric code), and the interacting vacuum's
  magic is EXTENSIVE with a converged density `c(x)`: zero at the fixed points, leaving them
  as `x²` (`≈ 5.7 x²` in the Schwinger rehearsal), saturating near `0.44` bits per site at
  physical coupling. The magic INSIDE a box of fixed size is independent of the volume around
  it, so a hadron-sized box of `L` sites is priced at `2^{c(x) L}` stabilizer terms — exactly,
  by the strata this engine owns, and affordably only for boxes of a few dozen sites. *(Reworded
  2026-09-19 on GF1's read, the owner's call; the locked text said "low-magic and area-law,
  priced exactly", and its area-law clause named no quantity — the non-local magic is `M₂`
  itself on any real vacuum of definite parity, by theorem. The price clause was measured and
  kept; the adjectives were wrong at physical coupling and are gone. `GF1_RESULTS.md`.)*
  Finite subgroups of SU(3) as link groups are prior art (Petcher–Weingarten 1980,
  Bhanot–Rebbi 1981, Alexandru et al. 2019); Floquet gauge steps at the grain are Clifford
  (`Grain.lean`); staggered quarks suffice (Nielsen–Ninomiya is chiral-only, `LEPTON_LADDER.md`).

*Kills and their status:*
- **GF0 — SCHWINGER-4:** two screened pairs' residual interaction must decay at the banked
  vector-meson mass. **READ, branch (a):** the rate at the gap to 0.6 % on both columns
  (`SCHWINGER4_RESULTS.md`).
- **GF1 — the magic price of gauge vacua:** if the log-price grows with volume, Fold III dies.
  **READ 2026-09-19** (`GF1_RESULTS.md`): the price clause is measured — a ten-site box of the
  Schwinger vacuum carries under four bits of magic at every coupling read, volume-independent
  — and the magic density `c(x)` is a converged function of the coupling (`≈ 5.7 x²` near the
  fixed point, `0.44` at `x = 1–4`). But the "area-law" clause names no quantity: the non-local
  magic, over local Cliffords or over any product of single-site unitaries, IS `M₂` on a real
  vacuum of definite parity, by theorem — and `M₂` is extensive, `c(x) N`. **Fold III reworded
  the same day around `c(x)` and the box price** (the owner: "this is just clarity"). The
  weak-coupling vacuum (`x = 16`) is outside the exact instrument's lease.
- **GF2 — the Σ(1080) hadron box** against the proton, neutron, pion and the deuteron: if the
  derived NN table misses its referees, Fold I's "forced, not fitted" dies. **GATED.** Its 1+1D
  rehearsal (GF2a, E7 → E14) met the exact colour-lane referee on all six N = 8 sectors at a χ
  set by the cut's rank, and was **closed as prior art** at volume (Silvi et al. 2019;
  Hayata–Hidaka–Nishimura 2023; no runtime compared). Banked for Fold III's price: χ is the
  sector's rank at the middle cut, labels grow as `(N/2+1)³`, and the price is the number of
  labels that carry weight (`GF2A_QCD2_RESULTS.md`).

**What the fold does NOT claim.** No hadron computed; the lepton is an input to QED, not an
emergent object; "exact-first" is about arithmetic and formulation, never affordability.

## The DRY register — where the object has folded into itself

*Kept so the next fold is found by looking here first. Every entry is one square in different
clothes; a candidate fold that cannot be written as entries composed is the signal to look
harder, not to add a primitive.*

- **2026-09-21 — search, select, price.** Five prose statements of 09-20/21 ("found, not
  named"; "closed is not object"; "the residual is the value"; rule 11's long form; W4's first
  wording) folded into "The procedure, as arithmetic": one formula block, one instance table.
  The +1 is the flux the slabs cannot reconstruct; the join is the law the level above owes.

| entry | witness |
|---|---|
| closure ≡ never-split | `viewClosed_iff_never_splits` |
| conservation ≡ the only exact closure measured | Move 3 |
| curvature ≡ paid-up rent on the transport map | `curvature_iff_held` |
| back-reaction ≡ mutual non-closure | `MatterCoupling` |
| gauge ≡ the Moore quotient's fiber | `Identity.lean` |
| the classical tier ≡ the diagonal retract of the quantum carrier | `DiagonalLift` |
| magic ≡ the sixth wall, non-closure of the tableau view | `Stabilizer.lean` |
| the grain ≡ the Clifford angle of a Floquet gauge step | `Grain.lean` |
| the string tension ≡ rent per link; the mass gap ≡ the far field's closure; the hadron ≡ the closed string | Fold I, II |
| the observer's frame ≡ allocation, not thinghood | ACUITY-B (`ACUITY_B_RESULTS.md`: the frame selects allocation; carried-coarse cost 0.018 bohr at 76 % of pair work saved) |
| a conserved integer lane ≡ a shard | `holon-chem/src/lanes.rs`: the whole determinant engine runs on one kernel, host and device bit-identical |
| the embedding field ≡ the Record | the +1 that no fragment generates and that does not factor, counted once (`EMBED_RESULTS.md` plant ii; `record_not_site_generated`, `repairable_does_not_factor`) |
| the residual of an embedded view ≡ the value the next view carries | `EMBED2_RESULTS.md`: the two-body residual is the three-body dispersion, `−C/R⁹`; separation by decay rate is what makes a residual tabulable |
| the self-consistent fixed point ≡ the declaration corner | `declaration_is_double`; measured start-independent to `2e-12` |
| a residual's exponent ≡ its channel; precision ≡ an allocator over channels | design rule 10: the sum of channels is exact once the field is counted once, so a residual is assigned by its decay law (`9.34 → R⁻⁹`, EMBED-2) and a scenario evaluates only the channels above the digit it asks for |

## The surface, audited — what of the workbench is the holon (2026-09-04)

*A band's LIVE means its doors resolve, never a certificate; none of the fine bands passes the
band-flip law (a tier only on a closure certificate against the dynamics beneath).*

| band | what is drawn or read | holon-coupled? | verdict |
|---|---|---|---|
| molecular | the certified scene: pair curves from FCI, the (O,H,H) surface, the census, the closure certificate, ACUITY-B's cut | it IS the holon | maximal |
| atom | the census molecule solved at the scene's geometry; the pinned molecule's atoms DRAWN at that solve's own density, Mulliken-partitioned (`holon_atom_band_coupled_rms_bohr`) | yes for the pinned molecule; every other atom at its free size, labelled | substrate exposed, honestly |
| nucleus | Z, isotope, mass, spin, charge radius DECLARED; the thermal wavelength on the holon's temperature | half | declared inputs; node E's measured spread not yet wired |
| the fold | one baryon's quark density on a 1+1D chain, exact, on the lane kernel | the solver, not the object | a model, credited as prior art on its face; a tier only through GF2 |
| the view | two-box law, fluid zoom, descent glide, filmstrip | draws the holon's own cut and moves nothing in the physics | machinery, correctly placed |

## The lock

The steelman changes in two ways only: a move's own kill fires (marked dead and kept), or a
FOLD is added — naming the exact fiber or budget it closes on, the DERIVED form of its defect,
the one priced object and its stratum, and a separable kill buildable here. A fold needing a
primitive outside the signature is a format replacement under W3's frozen grammar and is
refused as a fold. Opened 2026-09-04 for this clarification pass by the operator; closed with
it.

## What the record does NOT license

No formation-rate claim. A model system: minimal basis, classical nuclei (node E is the exit), a
two-dimensional certified scene (the 3D carrier in build). Closure is statistical over a staked
window with a declared budget, never trajectory-exact. The seam's far-field law is measured on
one carrier and one basis; its dependence on the field a core sits in is unmeasured and is the
next freeze. Every constant is a PRICE measured in a regime; the arithmetic-regime law is what
keeps a price from becoming a wall.

- **That the liquid carrier is water above the molecule tier.** MEASURED 2026-09-20 evening
  (`TSCAN1_RESULTS.md`): it is a LIQUID that diffuses, `D = 6.5 × 10⁻¹⁰` m²/s at 312–317 K —
  a quarter of water's — with water's activation energy (`19.0` kJ/mol, Arrhenius over
  312–526 K). Not a glass: the morning's "does not rearrange in 40 ps" was the lead's
  overreach from a 5 ps offset whose own arithmetic says `τ_c ≈ 1–2` ps, retracted the same
  day (the fifteenth instance). Every fluid reading is of a driven element of a liquid
  three to four times more sluggish than water at the same temperature, with a
  Stokes–Einstein ratio `~0.13` of water's at `k = 0.27 Å⁻¹`.

## Where the weight sits — the lead's calibrated read (2026-09-21; SUPERSEDED by the corrected table in the CORRECTION of 2026-09-25 above, kept legible here)

*Mutually exclusive readings of what a holon most likely IS; probabilities sum to one.*

| reading | p | what moves it |
|---|---|---|
| **A.** The full thesis: closure + paid rent IS thinghood at every tier, and the books are physical | 0.40 (0.36 before 2026-09-20) | Moves 1–4 stand; Move 5's dead legs and "the only exact closures are conservation fibers" cap it |
| **B.** Right account of objects, but a frame-selection principle is missing (`exists_closed_view` makes closure cheap) | 0.10 (0.20 before 2026-09-20: the missing principle has a candidate — the view is the variational optimum at the tier's cadence, measured once to find the staked charts unprompted; B keeps the weight of "once, one tier, linear views") | ACUITY-B measured the allocation half; rung 1's alignment finding is the frame puzzle still unsolved; `frames_are_not_gauge` cuts against full relativism |
| **C.** Thinghood is primarily CONSERVATION; rent is thermodynamics repackaged | 0.34 (unchanged 2026-09-21: the molecule tier says the object is the sector the next view carries — mass and momentum, conserved quantities — which C reads as conservation selecting and A reads as rent selecting; the tie is not broken) (0.28 before 2026-09-20: the search's slowest closed directions ARE the conserved densities at the longest wavelength — hydrodynamics — which is C's claim read off the data as much as A's) | the only exact closures are fibers; the exact label fixed what the budgeted sector could not |
| **D.** Closure is observer-indexed with no observer-free fact | 0.08 | `FrameOrder.lean` (frames are an order) and the certified molecule's 0/111 controls cut against it |
| **E.** The world-level rent is metaphor; the dead cosmology legs are the tell | 0.08 | the rate surviving on hardware and the design-knowing repair cut against it |

What moves the table next, named: the 3D carrier re-running rungs 1 and 2 (B and C separate on
whether a dynamic in-budget chart appears at scale); node E's quantum nuclei on the H₂ arm
(A and C separate on whether persistence needs more than conservation); DESI DR3 (E fires or
Move 5 survives). The two derived defect laws and the harvest sharpen Move 3's instrument
without moving a weight.
