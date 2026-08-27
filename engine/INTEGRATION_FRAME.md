# INTEGRATION FRAME — one holon, values only
Status: binding integration constraints for the descriptor-chain program.
Role: the tier specs (workflow `wf_29f9c722-2bd`) are integrated INTO this frame; any
tier content that cannot be expressed inside it is a **misfit to cash**, not a reason to
extend the frame.
Date: 2026-08-23.

## The three hard rules (Eric)

1. **All tiers use THE holon.** `Holon<W>` / `RuntimeArena` as they exist: `parent, depth,
   grain_units, gross (REG+), whole[W], channels, boundary, decomposition`. No per-tier
   structs, no new fields.
2. **Same frame and objects precisely.** Descriptor holons, relation holons,
   `MaterialBinding`, `CohesiveBond`, certificates, `GrainFloor`/`RefinementUnavailable` —
   identical at every tier. **The only difference between tiers is VALUES.**
3. **A stone holon must describe containing the three levels below it accurately enough
   to decompose** — grain, crystal, atomic — on certificate demand.

## What the code already has, and the one structural gap

`Decomposition::{Leaf, Latent, Expanded}` and transactional `materialize()` already give
us containment-without-enumeration: a Latent holon "represents recursively implied
children without enumerating them," and no materialization commits unless children
compose exactly back to the parent's REG+ ledger.

**The gap: the demo materializer is structurally empty.** `split_gross(parent.gross)`
halves counts. The children carry no mineral identity, no size distribution, no flaws —
because the parent carries no information about what its interior IS. Accurate
decomposition needs a **generator**, and the rules say it may not be a new object.

## The resolution: the descriptor holon IS the generator, and containment statistics are
## its WHOLE-ONLY state

No new machinery is needed — the frame already contains the answer if we use it fully:

- **Descriptors are holons**, so they decompose like everything else. The stone
  descriptor's children are the mineral descriptors; the mineral descriptor's children
  are bond/force-constant descriptors; theirs are atomic descriptors. "Three levels
  below" is the descriptor holon's OWN depth-3 subtree.
- **Mineral fractions are the descriptor's gross ledger.** Quartz 30% / feldspar 60% /
  mica 10% of a million-constituent wall is literally `GrossState` arithmetic on the
  descriptor's children: 300k/600k/100k constituent counts that must compose exactly.
  The existing conservation check enforces modal composition for free.
- **Distribution parameters are whole-only state on the descriptor** — and this is not a
  storage convenience, it is the metaphysics landing exactly where it should: a
  grain-SIZE distribution, an orientation texture, a Weibull flaw modulus are facts about
  the ENSEMBLE that do not factor through any single materialized child. Information
  that cannot be reconstructed from partial views is precisely what `whole[W]` exists to
  carry. The variable-width whole-state pool in `RuntimeArena` already supports a
  different W per holon, so a stone descriptor can carry {modal weights, grain-size
  log-normal (mu, sigma), Weibull (m, sigma_0, flaw density), texture parameters} while
  a bond descriptor carries {stiffness, energy} — same object, different values and W.
- **"Made of" stays `MaterialBinding`** — subject holon → descriptor holon. Unchanged.
- **Materialization becomes: read the binding, walk the descriptor subtree, draw
  children deterministically from the descriptor's distributions.** Seeded from the
  parent holon ID (the engine's `EntropyProvenance::Seeded` idiom), so replay is
  bit-identical — decomposing the same wall twice yields the same grains. The existing
  transactional commit then enforces ledger exactness.

## The new certificate this forces (and it is a check, not a new object)

Ledger exactness is already enforced. Accurate decomposition adds ONE obligation:
**statistical composition** — the empirical distribution of a materialized ensemble must
converge to the descriptor's declared distributions (KS-distance or moment bounds at the
materialized count, with the tolerance scaled to how many children were drawn: 288
resident nodes cannot and need not match a Weibull tail that only 10^6 grains would
show). A materialization that passes the ledger but fails the statistics is REJECTED the
same way a ledger violation is. This is the per-edge homogenization certificate of the
descriptor DAG, run in the DOWNWARD direction.

## What each tier is, under the rules

One schema, six value-sets. Per tier: (a) which `whole[W]` fields the descriptor carries
(values), (b) which channel evaluates it, (c) the certificate values (validity domain,
tolerance), (d) the gate values (reference sim, observables, thresholds). The tier table
comes from the workflow output, integrated under this frame — any tier demanding a new
FIELD rather than new VALUES is a misfit.

## Immediate consequences worth noting

- The mechanical channel almost never refines past crystal scale: the chain terminates
  at `GrainFloor` for mechanics at grain/crystal, by design. The atomic and floor tiers
  exist to CERTIFY the crystal descriptor's values (DFT-derived force constants), not to
  be materialized in a game frame.
- `deterministic_flaw`'s mod-43 pattern is the degenerate ancestor of the Weibull draw:
  same role, hand-tuned values. Replacing it is a VALUES change, which is the point.
- The demo's similarity-scaled cohesive law becomes a DERIVED value: descriptor
  properties + bond length + node spacing → law constants, with the derivation being
  T4's homogenization certificate run downward.

## Integrator decisions from the programme review — 2026-08-23

**E4 (n-ary relations / M23): DECIDED — no new object class, two moves inside the frame.**
1. *Relation descriptors* (grain-boundary cohesion, the place granite actually fails) need
   no extension at all: a `CohesiveBond` already carries its own relation-holon ID, and a
   `MaterialBinding` whose subject is that relation holon points it at a descriptor. A
   binding from a relation to a descriptor is values, not a new kind — the frame already
   contains it.
2. *Angle terms* (Si–O–Si, arity ≥ 3) are parked as chart state of a 3-atom PARENT holon,
   deferred until the T1 tier actually materializes — the Newton build needs no angle
   terms. Revisit only if the parent-holon chart creaks in practice; if it does, that is a
   misfit to record, not a silent generalization.

**A5 clarification:** splitting `IsotropicMaterial`'s dissipation into separately-warranted
fields is a DESCRIPTOR-schema change, which the frame permits — descriptors carry values,
and the frame rule protects the HOLON object. The rule violated today is warrant, not
shape: solver-stabilization numbers wearing material-constant costume.

**Build order (Lane E):** E3 (quenched-flaw Record + whole-only eviction + single friction
owner) and the descriptor-as-generator materializer run first, in parallel; E1 (adaptive
crack-tip) follows on the materializer's API; E2 (rigid chart + Record tag) rides with E3.
Every new gate added must be MUTATION-TESTED — a gate that cannot fail proves nothing.

## Integrator decisions from the materializer build — 2026-08-23

**Error surface: statistical rejection IS a composition failure.** The certificate rejects
through the trait as `GrossStateDoesNotCompose`, with the distinguishing
`StatisticalReport` retained on `last_report()`/`materialize_described()`. This is the
frame's own sentence made literal — "a materialization that passes the ledger but fails
the statistics is REJECTED the same way a ledger violation is" — so no new error variant
is added, and the frozen exhaustive match in the component adapter stays frozen.

**Descriptor libraries are their own holarchy.** A descriptor cannot live inside the scene
tree without inflating the scene's constituent ledger (RuntimeArena is single-rooted with
exact composition). Two arenas — scene and descriptor library — related through
`MaterialBinding` indices is frame-faithful: an arena is a resident refinement window, and
the library is a different window. One-arena-for-everything is REJECTED as the misfit it
would create, not the purity it would look like.

**Flagged and deferred, on the record:** coarse-to-fine label consistency (a coarse
"quartz" region may currently contain feldspar sub-grains on refinement — fresh draws per
level, documented; a majority-consistent draw rule is a values-only change if a tier needs
it); and per-child boundary selection on the materializer (tip-side children only), which
the crack-tip lane needs and the materializer's author is adding as a follow-up.

## Frame decision: gravity is CHART data — 2026-08-23

Uniform gravity lives on the scene's chart, never on holons: the equivalence principle
makes uniform g frame-equivalent to an accelerated chart, and universality of free fall
forbids per-holon g. The demo's honest statement, measured: **uniform chart gravity, one
named stage deviation** (`STAGE_WALL_GRAVITY_FACTOR = 0.035` on standing-wall nodes),
load-bearing for the CURRENT bond values — the factor-1.0 experiment tore all 272 free
nodes off the anchor within 2 s, tension fibers at the cantilever root exceeding the bond
peak ~3x. Deleting the knob is a VALUES change (a bond law strong enough to carry the
wall), and the experiment is worth rerunning when descriptor-driven T4-scaled bond laws
land — a wall that carries its own weight would let the knob be deleted as a values
consequence, which is the frame working. Self-gravity (mass sourcing the field) is the
curved tier's, per Gantt A3; the SR→Newton certificate does not cover weighted scenes
until A3 closes, and says so.

## DIRECTIVE: close all papering-over BEFORE game-level work — 2026-08-23 (Eric)

Game/DX integration is POSTPONED until the foundations carry no stand-ins. Rationale:
retrofitting foundations after the DX hardens costs 5–10x — every fix becomes an API break.

The papering-over inventory, each with its closure lane:
| # | stand-in | closure |
|---|---|---|
| P1 | gravity as bolted-on chart constant; A3 open; EP by citation only | **curvature lane**: SR realization + closed-form gates (A1/A2), then the weak-field curved chart — geodesic integration, Newtonian limit DERIVED with bounded remainder, EP entering once as the metric-theory premise |
| P2 | cohesive law "similarity-scaled" by hand | derive K/peak/failure-opening from continuum values (E, G_F, f_t) + lattice spacing — T4 homogenization run downward |
| P3 | STAGE_WALL_GRAVITY_FACTOR = 0.035 | rerun factor-1.0 with P2's derived laws; delete if the wall carries its weight |
| P4 | DEMO_CALIBRATION matches no specimen | pin to a published granite record (full field set, Record provenance) or keep the honest label with the gap named |
| P5 | C1 (Poisson direction) and C3 (McClintock–Walsh arithmetic) unfixed in the T4 values | fix with sources |
| P6 | fixed 288-node frontier | E1 (in flight) |
| P7 | MEET-1 keystone incomplete: mode-chart parameter + Boolean-vs-correlation fence unbuilt | Lean lane |
| P8 | T5 scope | first slice is free particles + geodesics + contact collisions ONLY — no long-range forces, so M16's relation-momentum build is not required yet and is not smuggled |
