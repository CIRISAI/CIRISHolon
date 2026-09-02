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

## The maximal reading — and its first certified instance

*The stance with statuses and kill conditions lives in CIRISAI/CIRISOntology
(`Stance.lean`, claims `closure`, `water-holon`, `object-rent`). This section
states the engineering programme's own maximal interpretation of the contract
above, at the strength the evidence carries — no stronger.*

**The premise, in one line:** some distinctions survive evolution and others
do not. An OBJECT is a lossy summary the dynamics never splits, and
everything above the bottom — object, law, scale, gauge, conservation,
tier — is the one commuting square wearing different clothes.

Two primitives, not one, and the distinction is load-bearing:

1. **The square** (`Closed v T`) gives EXISTENCE, and its algebra is
   mechanized: the induced coarse law is unique on reachable readings
   (`closure_determines_dynamics`), closed views compose into tiers
   (`Tiers.lean`), a reading invariant under the coarse law is conserved by
   the fine dynamics (`closed_view_inherits_conservation`), maximality is
   root-relative (`Omega.lean`), and approximate closure carries a budget
   that grows linearly in the non-expanding case (`Budget.lean`).
2. **The paid step** gives PERSISTENCE, and it is not a corollary of the
   square: under noise a closed view decays unless maintained (`rentStep`,
   `rent_closed_form`), and the repair must know the design — design-blind
   maintenance holds a structure's size while its identity decays (rule 5).

The physics dictionary — object as persistent closed quotient, gauge as
fiber motion, entropy as fiber multiplicity, interaction as
nonfactorization, conservation as a held reading, life as closure paying
rent — is carried at mixed strength: the rows through conservation are
theorem-backed here or in the sibling seed; particle, field, spacetime and
curvature are wagers with finite-model witnesses only. The statuses are not
decoration. A dictionary row without its status is how a programme misleads
itself.

**The first certified instance (2026-09-01).** In this engine's own
first-principles world — nuclear charges, masses, and per-encounter exact
diagonalization; no fitted potential anywhere; twelve atoms in a
two-dimensional box, quenched from hot gas on the conservation-audited
arm — a water molecule assembled itself and PASSED the closure test as
staked before the instrument existed: longest unbroken closed run 893.8 fs
against the pre-staked 834 fs window, 72.3% of a 17.5 ps trajectory, a
genuinely vibrating and tumbling carrier, zero of 111 control groupings
reaching the window, momentum at 6e-5 of its roundoff bound
(`conformance/water_observatory/CENSUS_PREREG.md`, `CENSUS_RESULTS.md`,
`census_mixed_fenced.log`; trajectory under a committed sha256 manifest).
The instrument says no far more often than yes — formula-reader "molecules"
are refused as transient, and most candidate summaries the campaign has
tested fail one leg or the other — which is what makes the yes evidence.
One certified-strict molecule in eight seeds; no formation-rate claim;
closure is statistical over the staked window with a declared defect
budget, never trajectory-exact, which chaos forbids.

**The four-body comparison, RULED (2026-09-02) — and half-defeated, which
is itself the result** (`CENSUS_RESULTS.md` §14). Both arms certify the
same three atoms strict. The control arm alone — dE4 provably absent,
momentum clean, plane held bit-exactly — refutes the causal claim: MBE3
does not stop at hydroxyl; water forms without the four-body term. What
dE4 ADDS remains open, because the comparison's other half was defeated by
the treatment producing its own variable: the dE4 arm alone leaves the
declared 2D plane (frame 4230, |z−z₀| → 11.49 bohr) and so explored space
the control could not reach. The one-variable design failed in a mode no
same-commit discipline prevents — the treatment opened a dimension — and
the clean successor needs genuinely-3D arms on both sides (carrier-v2).
Separately, node H measured the ladder itself: dE5 exceeds its declared
per-term bound on 24 of 24 sampled clusters (worst 1,572×), so the
truncation does not terminate at four and the cluster seam is fired.

**The first upward campaign (2026-09-02): closure above the certified
tier is not free, and the cost is now measured.** No coarse chart on the
certified carrier is both dynamic and inside its budget — rung 1's 70
readings split 36 vacuous-in-budget / 32 dynamic-out / 0 both, and rung
2's occupancy/transport scissor is chart-independent (A2: the verdict
census is bit-identical across a cell-field and a lattice-gas chart). Two
riders join design rule 2 from this campaign: BUDGET-COMPLIANCE WITHOUT
DYNAMICS IS VACUOUS — the battery must pair the defect with
reading-changes and distinct-readings, because on real carriers the two
conditions came apart everywhere (a band-aggregate defect is refused as a
readout for exactly this reason); and a chart meant to compose into a
conserved-label census must be BUILT from conserved labels — geometric
predicates measured as factoring through nothing. On the founding lattice
tier the closure defect is DERIVED, not fitted: the block's boundary
fraction, saturated, identical across all 4,608 lawful collision laws —
the defect belongs to the lattice, not the law — and a Boolean FHP word
discards ~94% of a fluid element's atoms, naming the fractional
mean-occupancy chart (`Core/ModeChart.lean`'s cap fence) as the only
bridge that can carry the band.

**The join, wagered:** an object is a shared pattern whose closure pays its
own rent — existence from the square, persistence from the paid step, the
receipts where the books are kept. Each half is backed at its own strength;
the join is the bet, and its kills are separable: closure held at zero
maintenance flux, or identity retained under design-blind repair — either
fires alone, leaving both halves standing.
