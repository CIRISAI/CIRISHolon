# Pre-registration — MIXTURES-1: arbitrary species share one box, and every pair pays its own curve

*Frozen 2026-08-30, committed ALONE. The sandbox holds one PotentialTable, so
a scene is one pair-type at a time even though atoms already carry species
and masses. This campaign gives the sim a PAIR-TABLE BANK keyed by unordered
species pair — the force loop, the bond criterion, the drift bound, and the
ledger dispatching per pair — plus the chemistry that fills it: second-row
and mixed-pair curves, exact-in-model where FCI is feasible and DMRG-bridged
with declared uncertainty where it is not. DECLARED INPUTS unchanged from
ELEMENTS-1: Z, masses, the STO-3G basis. The determinant-count feasibility
map is stated in the campaign notes and is a property of the model, not a
choice: near-full shells are cheap (Ar2 is ONE determinant), half-filled
shells are exponential (Na2 ~1e9), and the q8-mps DMRG bridge exists for the
mountain.*

misfits: contacts M-ONE-MODEL-DELTA (all gates exact-in-model; experimental
values labelled context only), M-STALE-INSTRUMENT (referee source is
committed — conformance/atomworld/elements1_referee/ — and every shipped
table names its producer and grid rule), M-PLANT-OBS and M-PLANT-SECTOR
(plants below, carriers asserted nonzero), M-NULL-MISSTAKE (negative stakes
are on in-model well depths), M-PARITY-PROTECT (the spin audit's parity
condition — 2S matching electron count mod 2 — is carried per geometry
exactly as R2 of ELEMENTS-1 hardened it), M-HOMOG (mixture scenes are scored on
per-pair-type readings, never a homogeneity assumption),
M-FINAL-VIEW-COLLISIONS, M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY,
M-PROBE-EIGENSTATE, M-GAUGE-LAUNDER, M-LOOP-BLIND, M-BARE-CHARGE,
M-COND-PROBE, M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM,
M-KINEMATIC-NONLOCAL, M-VOLUME-SCALE (not otherwise contacted).

## Scope, stated before the gates

Species H through Ar. The three-body term stays H3-ONLY and the sandbox
DISPLAYS that fence (heteronuclear trimer tables are a named successor with
a named cost); mixed scenes therefore run MBE2-exact plus H-only MBE3, and
no reading is presented as beyond-pair-complete for non-H triples. The DMRG
route (q8-mps) is admitted ONLY through gate D1 below. Standing questions 1
and 4 apply to every new gate: each must be exercised in the place it runs
and must demonstrate a failing case before it is trusted.

## Gates

- **R1 — dual-route atoms, second row**: Na through Ar ground-state
  energies agree between the determinant route and an independent
  construction at working precision, and against the committed Python
  referee at <= 1e-10 Ha. witness: none (measured)
- **R2 — staked-pair referee gate**: the engine's curves for the staked
  EXACT set — Cl2, S2, Ar2, HCl, ClF, NaH, SiO — match the referee at
  <= 1e-10 Ha pointwise on each pair's declared grid (sparse grids allowed,
  rule declared in the file, result-blind), with per-column declared
  uncertainties and the spin audit (multiplicity, parity, degeneracy
  reported not asserted) carried per geometry. Coverage manifest-declared;
  present + owed = staked, enforced. witness: none (measured)
- **D1 — the DMRG bridge earns admission**: q8-mps ground energies match
  exact FCI on at least two overlap species where BOTH are feasible (staked:
  S2 and SiO) at <= 1e-8 Ha across their grids; only then may DMRG-only
  curves (staked: Si2, Na2) enter the sandbox, each labelled DMRG with its
  own convergence-derived uncertainty, never presented as exact. Kill: an
  overlap disagreement past 1e-8 kills the bridge's admission, not the FCI.
  witness: none (measured)
- **E1 — the emergent negatives, second row**: Ar2 and NeAr have NO well
  deeper than 1e-4 Ha on their staked grids (closed shells refuse across
  rows). Branch (b) = investigate, never massage. witness: none (measured)
- **E2 — the emergent chemical contrast** (two-branch, structural): in-model
  D_e ordering N2 > SiO > HCl > ClF > S2 > Cl2 > NaH >> (Ar2, NeAr) in its
  broad strokes, numbers reported as the product; any gross inversion is
  branch (b), reported and investigated. witness: none (measured; the
  ordering hypothesis is the stake)
- **B1 — the bank is exact where the single table was**: a two-species
  scene through the bank reproduces the single-table H2 scene BIT-FOR-BIT
  when both species are H (the bank must not cost the banked physics), and
  every pair's bonded/e_rel reading uses that pair's own curve (asserted on
  a mixed fixture where the H-H and X-X criteria provably differ).
  witness: none (regression + fixture)
- **C1 — conservation in a mixed box**: energy drift <= the derived bound
  with the curvature envelope taken over ALL active tables and per-species
  masses in every mode; momentum gate unchanged. One gate per law.
  witness: none (measured)
- **P1 — THE PRODUCT: emergent hetero-chemistry**: the frozen quench
  protocol (staked before the mixed arm runs, D1-of-SATURATION-1 style:
  8 seeds, control arms) on an 8 H + 8 Cl gas ends with HCl as the modal
  molecule (branch a), the H-only and Cl-only controls reproducing their
  own banked behaviours. Branch (b) reported and investigated. VOID if a
  control fails. witness: none (measured)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) the swapped-table plant**: serving pair (A,B) the (A,A) curve must
  move a staked mixed dimer's R_e beyond referee tolerance by orders
  (carrier: the R_e shift, asserted large).
- **(ii) the mass plant**: running Cl atoms at H's mass must shift the
  mixed scene's derived timescale by the mass ratio's square root (carrier:
  the dt shift, asserted, computed not assumed).
- **(iii) the DMRG-label plant**: presenting a DMRG curve as exact must be
  REFUSED by the provenance gate (carrier: the refusal, demonstrated
  firing per standing question 4).
A missed plant VOIDs; a plant on an empty sector VOIDs.

## Meaning

All gates => "arbitrary first- and second-row atoms share one box, every
pair pays its own referee-pinned curve, the mountain in the determinant map
is crossed only over a validated bridge wearing its label, and
hetero-chemistry — which molecules FORM — emerges from Z, masses, and a
basis." NOT claimed: beyond-pair completeness for non-H triples (fence
displayed), relativistic fidelity (absent from the model and said so),
elements past Ar, quantitative thermochemistry against nature.
