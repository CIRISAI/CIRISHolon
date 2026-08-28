# The atom world — the physics/graphics tier's design, banked

*2026-08-28. The design record for RENDER-1/ATOM-1 and the substrate for
SELECTOR-1. This is the tier where crystals face reality: exact chemistry
below, ledger-gated rendering above, and the holon layer in between —
which is where molecules live.*

## The three clocks (never conflated)

1. **Physics dt** — derived from the model, not chosen: ω = √(E″(R_e)/μ)
   from the exact curve's own curvature; dt = period/64; the Verlet drift
   bound derived from (dt, ω) and displayed. A changed dt with a stale
   bound is a defect.
2. **Frame rate** — whatever the device gives, consuming substeps through
   a fixed-timestep accumulator. dt is never stretched to fit a frame.
3. **Sim-speed** — fs of sim-time per wall-second, user-visible; default
   makes one vibration watchable (~2 s).

Degradation is the tuner's contract (Policy{hold, degrade}): Hold =
Exactness; shortfall dilates time first; dt grows only under explicit
toggle WITH the re-derived bound shown. Silent substep-dropping forbidden.
The mouse is a zero-order-hold intervention whose work is integrated on
the ledger — E − W_ext = const through every push.

## The holon census (the recursive architecture, priced)

| level | holon | count | per-frame cost |
|---|---|---|---|
| micro | atom (position/momentum; view; transport) | N | the O(N²) force loop — the ENTIRE budget |
| composite | bonded pair / molecule — a maintained closure paying rent against kinetic noise | dynamic | ~free: closure checks at grain boundaries |
| candidate | every pair as potential closure — bond DETECTION is per-pair closure evaluation | O(N²) | rides the force loop free (~10 flops on numbers already in hand) |
| global | the energy ledger, momentum, external work | a handful | tens of flops at grain boundaries |

**The structural facts this banks:**
- The holon layer evaluates at FRAME boundaries (grain.rs's closure-
  aligned scheduling), not substeps: a thousand live molecules cost ~1e5
  flops/frame against a force budget of 1e7–1e8. Being matter is
  expensive; being a holon is cheap.
- **A molecule IS a composite holon**: formation = closure acquisition
  (pair energy below the in-model asymptote inside the outer turning
  point — read off the exact curve, never a distance heuristic);
  dissolution = rent unpaid; each molecule carries its own ledger row
  (bond energy, vibrational action) — so MOLECULAR DYNAMICS is the holon
  layer's dynamics, not an add-on.
- Capacity (projections pending the on-load calibration burst, which is
  authoritative per device): watchable speed ≈ 2,500–5,000 micro-holons
  pairwise-exact on a flagship phone (scalar → SIMD128), ≈ 300 on
  low-end; fast dynamics (~1 ps/s) ≈ 180–350 flagship; cell lists (the
  force loop is structured for the drop-in) push watchable to tens of
  thousands. N_max ≈ √(pair-throughput / substep-rate), computed and
  DISPLAYED by the calibration on every device.

## The SELECTOR-1 nesting

The payer-builder's subsystem P is a composite holon with its own ledger
view — a molecule that spends bond energy to maintain something. The
per-composite ledger rows at grain boundaries are exactly the instrument
SELECTOR-1 needs; its candidate subsystems are additional rows in a layer
that costs nothing measurable. The same box the user pushes atoms in is
the arena where the selection principle runs.

## Publishing

GitHub Pages (workflow-deployed from docs/): the README cannot execute
WASM, so the live demo publishes at cirisai.github.io/CIRISHolon and the
README carries a GIF linking to it.

## The three fences (external review, adopted before implementation lands)

1. **Boundness must imply MEASURED closure.** The energy threshold proves
   a bound pair, not an autonomous molecular view. At formation the
   molecular view's own one-step closure defect is SCORED (and must be
   zero/bounded); at dissolution the defect/rent must RISE. Formation =
   closure acquisition is a measurement, not an interpretation.
2. **Formation is accounting-only.** Creating or dissolving a molecule
   row redistributes ledger LABELS without touching the state:
   E_before = E_after exactly across the event, and (E − W_ext) drift
   stays within the derived bound. THE CAPTURE PLANT: an isolated,
   initially unbound two-body system with W_ext = 0 must NEVER form a
   stable molecule — pair energy is conserved, so capture requires a
   third body or an extraction channel (the user's spring doing negative
   work counts; nothing else does). A bond formed in that configuration
   convicts the integrator or the predicate.
3. **The dt bound must use the curvature ENVELOPE, not E″(R_e).** The
   repulsive wall's curvature exceeds the equilibrium value; dt derives
   from the largest Hessian accessible at the current energy (or adaptive
   substepping with a refusal), else the displayed Verlet bound can read
   green through a collision that violates it.

Plus: deterministic hysteresis/dwell for formation-dissolution at grain
boundaries with canonical multi-eligibility resolution; the census's
"cheap" claim MEASURED (frame cost with census on vs off); any future
cell-list cutoff certified against the bond predicate's reach; and the W3
note adopted verbatim — an extensible `kind` enum is schema compatibility,
NOT lawful extension: a payer-builder kind must be constructed by a
predicate definable from existing Ω views, interventions and ledgers
under the frozen extension grammar.
