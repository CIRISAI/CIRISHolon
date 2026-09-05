# Pre-registration — EWALD-1: the field in the wrapped box — the Ewald sum as a pure module, gated on its own invariances, then served to the field where FIELD-1 refused

*Frozen 2026-09-05, committed ALONE, before the module existed. Built by the lead's
delegate under the lead's integration. FIELD-1 refused the field under a wrapping boundary
by name (`FieldRefusal::PeriodicNeedsEwald`): a bare Coulomb sum over minimum images is not
a potential in a periodic cell, and the ledger would not close. This freeze builds the exit
it named. The sum is Ewald's (1921): the `1/r` lattice sum split by `erfc(αr)/r` (real space,
short-ranged, minimum image inside `r_c ≤ L/2`) plus `erf(αr)/r` summed in reciprocal space
over the cell's wave-vectors, minus the self term `α/√π · Σq²`, with each unit's own
minimum-image pairs EXCLUDED from interacting (their reciprocal-space part removed by
`−qq·erf(αr)/r`) exactly as the open-box field skips pairs within one row; a unit's
interaction with its own periodic images is kept, because in a periodic cell it is real.
Prior art: Ewald 1921; de Leeuw, Perram and Smith 1980 (the conditionally convergent sum and
the conducting-boundary convention used here, no surface dipole term); Essmann et al. 1995
for the parameter estimates. Not compared to any other code's runtime or numbers.*

misfits: contacts **M-VACUOUS-SUCCESS** (every gate asserts its work count — pairs in the
real sum, wave-vectors in the reciprocal sum — before its tolerance); **M-PLANT-OBS** and
**M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the sector the plant acts on —
§3); **M-STALE-INSTRUMENT** (this freeze alone; module, gates and the results document
together); **M-DEVICE-CLASS** (native `f64`, one class); **M-COND-PROBE** ("inside the"
appears; a force term, not a post-step operator); **M-BARE-CHARGE**, **M-HOMOG** (the words
"charge", "uniform" appear; classical charges on units; the neutralising background is the
one uniform object and it is named); **M-FLOOR-UNSTAKED** (every tolerance below is a
staked number; the α-invariance gate's floor is the accuracy parameter itself);
**M-VOLUME-SCALE** (contacted by keyword — the reciprocal grid's `k_max` is DERIVED from
`α·L` per axis rather than fixed, so the grid scales with the cell it resolves). Not
contacted: the rest of the registry.

## 0. What is built

`holon-render/src/ewald.rs`, a PURE module: positions, charges, unit ids and the cell →
energy, per-atom forces, the scalar virial (the engine's convention `Σ r·dU/dr`, so
`pressure()` reads `(2K − W)/3V` unchanged), and the two work counts. Parameters `(α, r_c,
k_max)` derived from the cell and ONE accuracy target `ε` by the standard estimates
(`r_c ≤ L_min/2`, `α = √(−ln ε)/r_c`, `k_max = ⌈α·L·√(−ln ε)/π⌉` per axis); `ε = 1e-8` is
the engine's default and is a declared number, not a fit. Then the integration, by the
lead: `Sim::accumulate_field` dispatches to the module when the boundary wraps, the refusal
`PeriodicNeedsEwald` is retired, and FIELD-1's gate G5 ("the wrapped box is refused") is
SUPERSEDED by this freeze's G6 ("the wrapped box is served"), recorded as such in both
results documents. The seam's wall is short-ranged and needs no lattice sum; it uses the
minimum image already.

## 1. Gates — the module

- **E1 — α-invariance.** On a neutral scene of 4 water units (8 charges, FIELD-1's pin
  charge `q_H = 0.231380372`) in a 20-bohr cubic cell: energy and every force component
  agree across three splits (`α` at 0.7×, 1.0× and 1.4× the derived value, `k_max` and
  `r_c` re-derived for each) to `1e-7` hartree and `1e-7` hartree/bohr respectively, at
  `ε = 1e-8`. The work counts: real pairs `≥ 1`, wave-vectors `≥ 1` on every split.
  witness: none (an invariance of the split)
- **E2 — the lattice sum's known value.** The rocksalt lattice: 8 unit charges `±1` on the
  sites of a cubic cell of edge `2a`, each charge its own unit; the energy per ion pair is
  `−M/a` with `M = 1.747564594633` (Madelung's constant, the defined limit of this lattice's
  Coulomb sum — a mathematical constant, declared as the one number this freeze takes from
  outside the engine, and from mathematics rather than from a measurement) to `1e-6`.
  witness: none (a lattice sum against its limit)
- **E3 — the force is the derivative.** Central differences at `h = 1e-4` bohr on every
  atom of the E1 scene: `|F − (−∂E)| / |F| ≤ 1e-7` per component where `|F| > 1e-10`; the
  forces sum to `≤ 1e-12` of the largest force magnitude (the sum runs over a neutral cell).
  witness: none (finite difference)
- **E4 — the open box is the large-cell limit.** The dimer of FIELD-2's start (two units,
  `R_OO = 5.5` bohr) centred in cubic cells `L = 20, 40, 80` bohr: `|E_ewald(L) − E_open|`
  decreases with `L`, and the exponent of the decrease between 40 and 80 bohr is `≤ −2.5`
  (the leading image term of a neutral dipolar pair is `L⁻³`); `E_ewald(80)` within `1e-6`
  hartree of `E_open` (the direct sum of the same charges in the open box, the engine's
  `field_energy_of`).
  witness: none (a limit)
- **E5 — the virial is the volume derivative.** On the E1 scene: `W = 3V·dE/dV` at fixed
  scaled coordinates, `dE/dV` by central difference with the cell scaled by `1 ± 1e-5`,
  agreeing to `1e-6` relative.
  witness: none (finite difference)

## 2. Gates — the integration

- **G6 — the wrapped box is served.** FIELD-1's four-water scene under `Boundary::Periodic`:
  `set_field(true, ·)` returns `Ok`, `e_field` is nonzero, and over 2,000 steps the receipt
  columns sum to `w_ext`, the honest drift peak is under a tenth of the enabling transition,
  and the momentum residual is under its bound. Supersedes FIELD-1 G5.
  witness: none (engine ledger and conservation gates)
- **G7 — the open box is untouched.** `Boundary::Open` and `Boundary::Walls` scenes with the
  field on produce checkpoint BYTES identical before and after this change over 2,000 steps
  (EXACT): the dispatch adds nothing to the direct-sum path.
  witness: none (bytes)

## 3. Plants

- **(i) The self term dropped.** E2 must miss `−M/a` by exactly the self term
  `α/√π · Σq²` (to `1e-9`); carrier: `α/√π · Σq² ≥ 1e-2` hartree, asserted nonzero in the
  sector the plant acts on (the self term).
- **(ii) The excluded pairs not corrected.** On the E4 dimer at `L = 40`: E4's agreement with
  the open box must fail by at least `1e-3` hartree; carrier: `Σ_intra qq·erf(αr)/r ≥ 1e-3`
  hartree, asserted nonzero in the sector the plant acts on (the exclusion correction).

## 4. Discipline

Module `holon-render/src/ewald.rs`; gates `holon-render/tests/ewald.rs` (E1–E5, the plants)
and `tests/field.rs` (G6 replacing G5, G7); results `EWALD_RESULTS.md` committed with the
module. No number enters from outside the engine except Madelung's constant, declared
above. The module is built and gated as a pure function before a line of `sim.rs` changes.
