# Pre-registration — ELEMENTS-3: the table to xenon, from the same three inputs

*Frozen 2026-08-30, committed ALONE. The registry extends from Ar (Z = 18)
to Xe (Z = 54): potassium through xenon, STO-3G, s/p/d shells only — the
Cartesian d integrals already landed, so NO new integral machinery is
required below the lanthanides (f, l = 3, stays the named successor). The
declared inputs remain exactly three — Z, masses, the basis — and every
other number is computed. Two structural facts of the determinant map are
stated up front as model properties: near-noble species are hole-cheap
(Xe's atom is ONE determinant; Br2 ~1.3e3; HBr ~3.6e2) while mid-row
species are exponential (K2 ~4e11) and cross only on the DMRG bridge under
MIXTURES-1's D1 admission rule; and the FCI string masks are u32 today, so
heavy dimers (up to 54 spatial orbitals) require a u64 widening, gated by
bit-identity below.*

misfits: contacts M-ONE-MODEL-DELTA (exact-in-model throughout;
experimental values appear as labelled context only, including the
relativistic fence gauge below), M-STALE-INSTRUMENT (referee source
committed and extended in place; every table names producer, grid rule,
route), M-PLANT-OBS and M-PLANT-SECTOR (plants below), M-NULL-MISSTAKE
(negative stakes are on in-model well depths), M-PARITY-PROTECT (the spin
audit — multiplicity asserted only where the gap resolves, 2S matching
electron count mod 2, degeneracy reported — carries to every new species),
M-HOMOG, M-FINAL-VIEW-COLLISIONS, M-NONBIJECTIVE-STEP,
M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE, M-GAUGE-LAUNDER,
M-LOOP-BLIND, M-BARE-CHARGE, M-COND-PROBE, M-ELECTRIC-BASIS,
M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM, M-KINEMATIC-NONLOCAL,
M-VOLUME-SCALE (not otherwise contacted).

## Scope, stated before the gates

Z = 19..54. Nonrelativistic Schrodinger throughout: relativistic physics is
GENUINELY ABSENT from the model and the record says so — gate F1 measures
that fence rather than hiding it. No f functions. Mid-row FCI-infeasible
species enter only as DMRG-labelled objects per MIXTURES-1 D1; nothing
DMRG-produced is ever presented as exact. The four standing questions
apply to every gate built here (connected in the place it runs; sweep
reaches where the claim fails; nothing-reads audit with the writer outside
the reader set; a demonstrated failing case before trust).

## Gates

- **W1 — the mask widening costs nothing**: FCI string masks go u32 to
  u64; every previously computed species (H through Ar, all banked pairs)
  reproduces BIT-IDENTICALLY under the widened masks. witness: none
  (regression, exact equality)
- **T1 — the transcription gates, generalized**: per-shell exponent-ratio
  bands with DERIVED tolerances (each ratio's own rounding bound, judged
  against the best-determined element — the oxygen lesson) across ALL
  rows, plus contraction-coefficient universality per shell type. A
  planted single-digit typo in any new element must fire. witness: none
  (measured gate + plant)
- **R1 — atoms, dual-route and referee-pinned by declared tier**: every
  atom K..Xe solved by two independent routes at working precision. The
  50-digit referee covers every atom whose determinant count is <= 3e4
  (the threshold staked here, result-blind); heavier atoms carry f64
  dual-route agreement plus a DMRG cross-check at <= 1e-8 Ha, and every
  atom's ROUTE is labelled in its record. witness: none (measured)
- **E1 — the emergent nobles**: Kr and Xe are single-determinant closed
  shells in-model (asserted exactly), and Kr2 and Xe2 have NO well deeper
  than 1e-4 Ha on their staked grids. Branch (b) = investigate, never
  massage. witness: none (measured)
- **E2 — the emergent column trend**: in-model D_e ordering HCl > HBr >
  HI (nature's halide-hydride trend) with the numbers reported as the
  product; the staked exact dimers are HBr, HI, Br2, plus negatives Kr2,
  Xe2. Any inversion is branch (b), reported and investigated. witness:
  none (measured; the ordering hypothesis is the stake)
- **F1 — THE RELATIVISTIC FENCE, MEASURED**: the in-model vs experimental
  D_e gap down the column HCl -> HBr -> HI is computed and REPORTED as
  labelled context, staked to GROW down the column (relativity and core
  correlation are absent and increasingly missed). This is a gauge of the
  model's edge, two-sided: if the gap does NOT grow, the fence claim as
  stated dies and is investigated. witness: none (measured context gauge)
- **P1 — the display tier**: palette radii, masses, and timescales for
  all new species derived by the existing machinery, no remembered
  numbers; the sandbox provenance shows each species' route label.
  witness: none (contract gate)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) the transcription typo**: one digit changed in a staked new
  element's exponent must fire T1's ratio band (carrier: the band
  deviation, asserted large).
- **(ii) the mask plant**: a deliberate 32-bit truncation reintroduced
  behind the u64 path must fire W1's bit-identity on a >32-orbital
  species while leaving <=32-orbital species silent — proving the
  regression watches the sector the widening changed (carrier: the
  >32-orbital divergence).
- **(iii) the route-label plant**: presenting a DMRG-routed atom as
  exact must be REFUSED by the provenance gate, demonstrated firing
  (carrier: the refusal).
A missed plant VOIDs.

## Meaning

All gates => "the periodic table through xenon emerges from Z, masses,
and a basis — transcription machine-guarded, every route labelled, the
nobles closing themselves, a column trend of nature reproduced in kind,
and the model's relativistic edge measured and displayed rather than
denied." NOT claimed: lanthanides or anything needing f functions;
relativistic fidelity; quantitative thermochemistry; mid-row exactness
(DMRG-labelled means DMRG-labelled).
