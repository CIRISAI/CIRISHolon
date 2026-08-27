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

## Conformance obligations (CI, per tier)

- The closure battery (construction premise, budget, witness-pair hunt).
- Chart conditioning declarations for every exposed aggregate.
- Per-conserved-quantity gates with planted-mutation sensitivity
  (a gate that cannot fire on a planted violation is refused).
- For quantum strata: the retract test (Born readout = classical step,
  exactly) and the QASM suite up to the stratum's declared boundary.
