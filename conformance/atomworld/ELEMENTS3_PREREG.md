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

## AMENDMENT A1 — 2026-08-30, before any E1/E2/F1 gate runs

*Recorded after W1, T1 and R1's heavy half landed and before a single E1, E2 or F1
gate has been evaluated. Disclosure of what was already known when this was
written, because that is what pre-registration integrity turns on: the in-model
`D_e` values for HCl, HBr and HI have been MEASURED under the corrected convention
below and appear in the lane's record. The convention was chosen on DETERMINANT
COUNTS alone, before any well depth existed, and this amendment changes no stake,
no bar and no direction. The E1 negatives, Br2, and every gate verdict are unrun.
The original text above stands unedited.*

**A1.1 — the d-shell component convention was implicit at freeze, and is now
DECLARED: five spherical components, not six Cartesian.** The engine evaluated
d shells as six Cartesian components when this freeze was written. The freeze's
own determinant arithmetic is derivable only under FIVE: "Xe's atom is ONE
determinant", "Br2 ~1.3e3", "HBr ~3.6e2", HI ~784, and "up to 54 spatial orbitals
for Xe2". Under six the engine measures Kr 361 determinants, Xe 164836, Br2
71166096, HBr 36100, HI 16483600 and Xe2 58 orbitals — not one of which this
freeze could have written down. Five is therefore the convention the freeze
declared, and implementing it executes the contract rather than revising it; the
measured six-component counts above are the discriminating evidence and are
recorded here as such.

The mechanism, because the arithmetic alone would not explain it: the six
Cartesian d functions do not span an `l = 2` space. They span the five real solid
harmonics PLUS `(x^2+y^2+z^2) exp(-a r^2)`, which is spherically symmetric and
therefore `l = 0`. Carrying it gives every d shell a spurious sixth function of
the wrong symmetry — which in a MINIMAL basis is a different model and not a
larger one, and is exactly what turned single-determinant closed shells into
361-determinant problems with no chemistry in the difference. The basis is a
DECLARED INPUT, so its component convention is part of the declaration and the
registry header now states it instead of leaving it to be inferred.

Gates on the transformation, all three demonstrated before trust: every element
below Z = 21 bit-identical (no element below scandium has a d shell, so the
projection is never built there); per-species basis dimensions asserted against
what the engine assembles; and a PLANTED wrong transform — the sixth row left in —
which must fire against this freeze's own counts. witness: none (measured gate
plus plant; the transformation is checked by the variational subspace ordering,
which is an inequality rather than a Lean statement)

**A1.2 — R1 is restated to what has a route, and the route-less set is NAMED.**
The freeze assumed mid-row species would cross on the DMRG bridge. Measured, that
bridge reaches six orbitals: `pair::MPS_MAX_ORBITALS = 6`, from the MPO-builder
logs in `engine/output/mixtures1/` — LiH at six orbitals took 528 s to build its
MPO and HCl at ten did not complete in over an hour, the construction and not the
sweep being the whole budget. Every atom in this freeze's range is 13 to 27
orbitals, so no DMRG cross-check is producible for any of them today.

R1 restated: every atom WITH A ROUTE is dual-routed and labelled; every refusal is
NAMED per the route-label discipline; and the referee-eligible set is the nine
measured at or under 3e4 determinants — Ge, As, Se, Br, Kr, Sb, Te, I, Xe. The 16
atoms with no route at all are Z = 21..27 and Z = 39..47, determinant counts 2.6e7
to 2.0e12 (yttrium is 1971493202250). witness: none (measured)

Named successor, so today's refusals become tomorrow's routes without a re-freeze:
an MPO-builder upgrade inside the `q8-mps` crate, whose acceptance test is MPO
construction for a 27-orbital atom in minutes followed by D1-grade validation.
Until that exists the refusals stand as refusals.

Recorded neutrally as a finding about a claim: a sprint report states Sc–Fe DMRG
convergence. Scandium through iron are 18 orbitals each, and the measured reach is
six, so that claim and this measurement cannot both describe the same atoms. The
discrepancy is logged here for whoever reconciles it; nothing in this freeze
depends on its resolution.

**A1.3 — P1 gains a third radius rule, for species whose homonuclear dimer is
infeasible.** P1's radius is DERIVED from each element's own homonuclear curve,
and by A1.2 most mid-row homonuclear dimers cannot be computed at all. Rather than
substitute a remembered constant, a third DECLARED rule is added: the radius is
derived from the atom's own computed electron density, as the expectation of `r`
over the 1-RDM. It is still derived and still per-element; it is a different rule
and is labelled as one. A new declared `RadiusRule` variant carries it, the
registry states it, and every surface that shows a radius — the picker included —
distinguishes dimer-derived from density-derived per species. witness: none
(contract gate plus plant)

Plant, added to the plants section's discipline above: presenting a
density-derived radius as dimer-derived must be REFUSED by the label machinery,
demonstrated firing. Carrier: the refusal, asserted nonzero in the sector the
plant acts on — a species that actually carries a density-derived radius — before
the plant is scored. A missed plant VOIDs.

## AMENDMENT A2 — 2026-08-30, declaring P1's rule as SHIPPED

*A1.3 approved a third radius rule and named the quantity: an r-expectation over the
atom's electron density. That quantity was built first, measured, and does not do the
job. The rule that shipped is a different expectation, and the declaration has to say
which one — for the same reason A1.1 exists, that a convention nobody wrote down is
how the last one went wrong. Recorded before the E1/E2/F1 gates run.*

**A2.1 — the shipped rule is the OUTERMOST OCCUPIED ORBITAL's r-expectation, not the
whole density's.** The all-electron quantity `sqrt(<r^2>/N)` is dominated by the tight
core: measured across the registry it is flat at about one bohr from hydrogen to xenon
and is not monotone — xenon reads 1.026 against hydrogen's 1.396. That is the correct
value of a quantity that does not mean what a drawn radius has to mean, and it is the
reason for the substitution rather than a defect to be repaired.

The shipped rule is `sqrt(<phi|r^2|phi>)` for the highest occupied orbital of the SCF
that fixes the CI's orbital basis. It reproduces two facts about the periodic table
that are not inputs to it: size FALLS across a period (Na 2.261, Ar 1.640; K 3.499,
Kr 1.901 bohr) and JUMPS when a new shell opens (Na > Ne, K > Ar).

Two declarations travel with it. The orbital is the SCF's, so "which orbital is
outermost" is a property of that reference and not of the correlated state — adequate
for a drawn radius, and NOT a physical observable. And the quantity is not on the same
axis as the other two rules: those measure where two atoms sit relative to each other,
this measures how far one atom's valence electron sits from its own nucleus, which is
why every surface carries `radius_from_dimer` rather than a radius alone. The
all-electron function is KEPT in the crate under a gate asserting it still has the
defect it was rejected for. witness: none (measured rule plus the A1.3 label plant,
which is unchanged and fires on selenium)

**A2.2 — the referee leg reaches TWO of the nine, and the successor is shared.**
A1.2 named the referee-eligible set as the nine atoms at or under 3e4 determinants.
The threshold stands; the arithmetic does not reach it. A 50-digit referee needs an
eigensolve over the determinant space, and germanium's 23409 is far past what mpmath
does. Krypton and xenon are reachable for a structural reason rather than a size one —
every orbital doubly occupied, so the determinant is unique up to a phase, its energy
is invariant under orbital rotation, and it is a closed expression in the AO integrals
with `D = 2 S^-1`, needing no eigensolve and no SCF. Measured against the engine:
1.0e-11 and 5.3e-11 hartree. The remaining seven are OWED, not delivered.

The successor that unlocks them is not a new one. `conformance/atomworld/
mixtures1_referee/FEASIBILITY.md` records the same wall from the other side —
SiO's 196889056 nonzeros re-walked per matvec, measured rather than projected — and
its string-driven sigma rewrite is what both need. One successor, two campaigns'
owed items, and when it is built both discharge without a re-freeze.

## AMENDMENT A3 — 2026-08-30, two corrections to this lane's own record

*Both prompted by findings from sibling lanes rather than by my own review, which is
worth stating: neither would have been caught by re-reading what I wrote.*

**A3.1 — A1.2 over-claimed. "No route at all" should read "no AUTOMATIC route".**
A1.2 lists sixteen atoms as having "no route at all", Z = 21..27 and 39..47. That is
too strong and it is the exact mistake the MIXTURES-1 lane named the same day: reading
`pair::feasibility`'s refusal, which is a statement about the AUTOMATIC route through
`fci::solve`, as a statement about reachability.

The measurement that exposes it: SiO at 132496 determinants solves in 33.9 s through
`solve_determinant`, which has no threshold — against the thirty hours implied by
treating the automatic route's refusal as the answer.

What is actually true, split three ways because they are three different facts:

* there is no AUTOMATIC route for anything past `MPS_ROUTE_THRESHOLD`, since `solve`
  sends it to a DMRG measured to reach six orbitals — that covers essentially every
  atom in this freeze's range above germanium;
* `solve_determinant` has no threshold and is bounded by MEMORY, the CI vector being
  `n_det * 8` bytes with a Davidson subspace on top;
* the sixteen span 2.6e7 to 2.0e12 determinants, so they do NOT share a verdict.
  Cobalt at 2.6e7 is plausibly reachable with patience; yttrium at 1.97e12 needs a
  sixteen-terabyte vector and is not reachable by any arrangement of this machine.

The boundary between those last two is NOT measured here and is not asserted. It is
owed, and the honest form of A1.2's list is "no automatic route, and unmeasured
reachability by the determinant route above about 1e7 determinants". The
`elements3_atoms` record already labelled its own cut as OVER BUDGET — a spending cap,
explicitly not a claim that no route exists — so the record was right and the amendment
was wrong; they are now consistent. witness: none (correction of a scope claim)

**A3.2 — E1's "staked grids" had no referent, and one is named.**
E1 stakes that Kr2 and Xe2 have no well deeper than 1e-4 Ha "on their staked grids",
and this freeze never declared what those grids are. The MIXTURES-1 lane hit the same
gap in its own E1 and deferred discharge rather than measure on a grid its freeze did
not name.

The grid is hereby declared as the engine's standing rule, which is what every curve in
this crate already uses: the range from `pair::derive_range` — inner end where the
repulsion reaches `WALL_CEILING`, outer end where the interaction falls inside
`TAIL_TOLERANCE`, both bisected — with knots placed by `table::grid_point`, uniform in
`R^{-1/4}`.

DISCLOSURE, because the ordering matters: Kr2 had already been measured on that grid
when this was written (no well over R = 3.278..10.240 bohr). What makes the declaration
legitimate rather than fitted is that the rule is not being chosen now — it predates
this freeze, it is used by every pair curve the crate produces, and it is computed from
the asymptote and two declared energy thresholds BEFORE any well is located, so it
cannot have been steered by the result. What was missing was a pointer, not a decision.
A reader who disagrees should treat E1 as this lane's E1 measured on a rule named after
the fact, and weigh it accordingly. witness: none (declaration of an existing rule)
