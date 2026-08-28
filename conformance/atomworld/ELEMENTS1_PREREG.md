# Pre-registration — ELEMENTS-1: the first row emerges

*Frozen 2026-08-28, committed ALONE. The chemical tier generalizes from
H₂ to the first row: general STO-3G (s and p functions) integrals and
determinant FCI, exact-in-model, referee-gated at 50 digits. DECLARED
INPUTS, stated once: the nuclear charge Z (an integer), nuclear masses
(measured inputs, like m_e), and the STO-3G basis definition (a model
choice). Everything else — energies, bond lengths, well depths, WHICH
PAIRS BIND AT ALL — is computed, never fitted.*

misfits: contacts M-ONE-MODEL-DELTA (all gates are exact-in-model
readings against the model's own referee, never against experiment —
experimental values may appear as clearly-labelled context only),
M-STALE-INSTRUMENT (referee, engine and results committed together),
M-PLANT-OBS and M-PLANT-SECTOR (plants below, carriers asserted nonzero
in the sector each plant acts on), M-NULL-MISSTAKE (the unbound-pair
stakes are on the quantity the model controls: the in-model well depth),
M-FINAL-VIEW-COLLISIONS, M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY,
M-PROBE-EIGENSTATE, M-GAUGE-LAUNDER, M-LOOP-BLIND, M-BARE-CHARGE,
M-COND-PROBE, M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM,
M-HOMOG, M-KINEMATIC-NONLOCAL, M-VOLUME-SCALE (not otherwise contacted).

## Species staked (all neutral, exact FCI feasible in minimal basis)

Atoms H through Ne (ground-state energies, both routes). Diatomic curves:
H₂ (regression vs the banked referee), LiH, Li₂, HF, N₂, F₂, CO, plus
the two NEGATIVE CONTROLS: He₂ and Ne₂.

## Gates (all EXACT-IN-MODEL)

- **R1 — dual-route FCI per species**: determinant-space FCI
  (Slater–Condon) agrees with an independent construction (spin-adapted
  or brute-force Fock-space route) at working precision at every staked
  geometry, every species. witness: none (the H₂ instance's discipline,
  generalized)
- **R2 — the 50-digit referee gate**: the engine's f64 curves match the
  Python referee at ≤ 1e-10 Ha pointwise (staked looser than H₂'s 1e-12
  because p-integral conditioning is harder; the MEASURED residual is
  reported and becomes the successor's stake). H₂ must reproduce the
  banked referee exactly as before. witness: none (measured gate)
- **E1 — THE EMERGENT NEGATIVES**: in-model, He₂ and Ne₂ have NO well
  deeper than 1e-4 Ha anywhere on the staked grid — closed shells refuse
  to bind, with nothing telling them to. Branch (b) (either binds deeper)
  ⇒ the model or the code is wrong; find which. witness: none (measured)
- **E2 — THE EMERGENT PERIODIC PATTERN** (two-branch, structural): the
  in-model well depths order N₂ > CO > HF > Li₂ ≈ LiH > F₂ ≫ (He₂, Ne₂)
  in their broad strokes — triple bond deepest, closed shells unbound —
  with the EXACT in-model numbers reported as the finding. Branch (b):
  any gross inversion (e.g., F₂ deeper than N₂) is reported and
  investigated, not massaged. witness: none (measured; the ordering
  hypothesis is the stake, the numbers are the product)
- **E3 — the sandbox contract**: every bound species pair emits the same
  Hermite table schema (E, F, E'', envelope) the renderer consumes, with
  per-pair provenance; unbound pairs emit repulsive-only tables (the
  sandbox must show He bouncing off everything, because that is the
  in-model truth). witness: none (contract gate)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) the Z-mutation**: rerunning any species with Z off by one must
  shift its atomic energy beyond the referee tolerance by orders of
  magnitude — the pipeline provably reads Z. Carrier: the mutated run;
  sector: its energy shift, asserted large.
- **(ii) the basis-mutation**: perturbing one contraction coefficient at
  1e-6 must fire the referee gate — the pin protects the basis
  definition. Carrier: the mutated basis run; sector: the residual.
A missed plant VOIDs.

## Meaning

All gates ⇒ "the first row of chemistry — including which elements
refuse to bond — emerges exact-in-model from Z, masses, and a basis, with
every number referee-pinned and the sandbox consuming only derived
tables." What this is NOT: quantitative thermochemistry (STO-3G FCI is
exact-in-model, semi-quantitative against nature — dispersion binding of
He₂ is REAL physics the model excludes, and the record says so), and not
elements beyond the first row (p-only; d-functions are a successor).
