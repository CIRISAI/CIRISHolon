# Pre-registration — SATURATION-1: valence emerges when the third body pays

*Frozen 2026-08-28, committed ALONE. The sandbox's force loop is
pairwise-additive, so 16 hydrogens condense into a noble-gas droplet
(the field screenshots of 2026-08-28) instead of eight molecules. Real
hydrogen saturates because valence is a MANY-BODY fact. This campaign
adds the three-body term of the many-body expansion (MBE), exact-in-model
and constant-free: dE3(r12, r13, r23) computed by STO-3G FCI on H3
(9 determinants per point), tabulated at load by the engine itself,
differentiated analytically into the force loop. DECLARED INPUTS: none
beyond ELEMENTS-1's (Z, masses, the basis). Everything else — the sign,
size, range and consequence of the three-body law — is computed.*

*DISCLOSED, seen before this freeze (the feasibility probe,
`examples/mbe3_probe.rs`, committed 58cf9e1): dE3 at compact trimers is
repulsive (+0.858 Ha equilateral at r_e, +0.355 linear, +0.217 for H2+H
at 2 bohr), machine-zero (-2.3e-15 Ha) at 20 bohr, and two separated
dimers beat the r_e-edge H4 tetrahedron by +0.426 Ha (singlet block).
These numbers are PRIORS here, not results; every gate below stakes a
quantity not yet computed.*

misfits: contacts M-ONE-MODEL-DELTA (every gate is exact-in-model against
the model's own referee; nature's H3 barrier appears as labelled context
only), M-STALE-INSTRUMENT (referee, engine table and results committed
together), M-PLANT-OBS and M-PLANT-SECTOR (plants below, carriers
asserted nonzero in the sector each plant acts on), M-NULL-MISSTAKE (the
saturation stakes are on in-model quantities the truncation controls),
M-HOMOG (the quench stake D1 is scored on cluster-size distributions,
never on a homogeneity assumption), M-FINAL-VIEW-COLLISIONS,
M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY, M-PROBE-EIGENSTATE,
M-GAUGE-LAUNDER, M-LOOP-BLIND, M-BARE-CHARGE, M-COND-PROBE,
M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM,
M-KINEMATIC-NONLOCAL, M-VOLUME-SCALE (not otherwise contacted).

## Scope, stated before the gates

Hydrogen only (the homonuclear trimer table; heteronuclear three-body
tables are a successor). MBE truncated at order 3, with the truncation
GAUGED at order 4 (F1) rather than assumed. Each MBE subsystem energy is
its electronic ground state in the subsystem's own minimal Sz sector
(H: doublet; H2: singlet; H3: doublet; the H4 comparisons: singlet
block, stated). Born–Oppenheimer throughout. The staked table domain is
sides in [0.9, 7.0] bohr under the triangle inequality; outside it dE3
is taken as zero and the truncation is MEASURED (T2), never assumed.

## Gates (all EXACT-IN-MODEL)

- **R1 — the 50-digit trimer referee**: the engine's f64 H3 FCI energies
  match an independent Python/mpmath referee at ≤ 1e-10 Ha on a staked
  set of ≥ 64 geometries spanning the domain (compact, scalene, near-
  linear, near-boundary). The referee shares NO code with the engine.
  witness: none (measured gate)
- **T1 — interpolant fidelity, held out**: the tabulated/interpolated
  dE3 vs direct FCI at 256 pseudo-random geometries (staked seed, drawn
  inside the domain, none on grid nodes) — max |error| REPORTED; kill
  if > 1e-3 Ha (that would corrupt wells at the percent level); the
  measured value becomes the successor's stake. Two-sided: the same 256
  must show a NONZERO error (an exact zero means the draw hit nodes and
  the check tested nothing — VOID, redraw). witness: none (measured)
- **T2 — the boundary gauge**: max |dE3| on the domain boundary shell
  (any side at 7.0 bohr) REPORTED as the tail-truncation systematic;
  kill if > 1e-5 Ha (the tail would then be dynamics-visible and the
  domain must grow). witness: none (measured)
- **C1 — conservation with the third body paying**: the sandbox's energy
  gate holds (drift ≤ derived bound) through the staked scenes with
  three-body forces ON, the drift bound's curvature envelope extended by
  the three-body stiffness the same way k_pair_max extended it; the
  momentum gate holds unchanged (the triple force is translation-
  invariant by construction — equal and opposite in every triple).
  One gate per conservation law, never combined. witness: none (measured)
- **D1 — THE PRODUCT: molecules, two-branch**: the staked quench
  protocol (16 atoms, 8 staked seeds, thermostat on at the staked
  strength, staked frame count — all frozen in the results doc BEFORE
  the MBE3 runs are looked at) is run twice: pair-only and MBE3.
  CONTROL: pair-only must reproduce the field droplet (largest final
  cluster ≥ 8 in ≥ 6/8 seeds); if the control fails the gate is VOID
  (protocol, not physics — per the detector-not-verdict rule). Branch
  (a): with MBE3, the modal final cluster size is 2 and no cluster > 4
  survives, in ≥ 6/8 seeds — the gas becomes molecules. Branch (b): any
  other outcome is reported and investigated as a finding about the
  in-model three-body surface, not massaged. witness: none (measured;
  the branch structure IS the prereg's decision tree)
- **F1 — the truncation gauge at order 4**: dE4 (the four-body term)
  measured at the staked compact H4 set (r_e tetrahedron, r_e square,
  r_e rhombus, and the same three at 1.5 r_e) and REPORTED as
  |dE4|/|dE3| per geometry; kill if |dE4| > |dE3| at a majority of the
  compact set (MBE3 would then not be the converged tier where
  saturation lives, and the tier's label dies). witness: none (measured)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) the sign-flip plant**: negating the tabulated dE3 must invert
  the two-dimers-vs-tetrahedron comparison (carrier: the +0.426 Ha gap,
  disclosed above, asserted nonzero at run time). A saturation gate that
  cannot see the sign of the term it credits is not a gate.
- **(ii) the symmetry plant**: dE3 is totally symmetric in its three
  sides. Evaluating the table at a staked scalene geometry under all six
  permutations must agree EXACTLY (machine); a deliberately
  desymmetrized table must show the disagreement ≥ 1e-6 Ha (carrier:
  the mutated table's asymmetry, asserted nonzero).
- **(iii) the far-field plant**: zeroing dE3 INSIDE the domain (below
  4 bohr perimeter) must flip D1's MBE3 arm back toward the droplet in
  a staked 2-seed spot check — the dynamics provably reads the table
  where the physics lives (carrier: the D1 outcome shift).
A missed plant VOIDs.

## Meaning

All gates ⇒ "valence — the fact that hydrogen makes molecules and stops —
emerges exact-in-model from the same three declared inputs, enters the
sandbox as a computed-at-load three-body law with no new constant, and
the ledger still balances while it acts." What this is NOT: quantitative
H3 reaction dynamics against nature (STO-3G FCI's in-model barrier is
not nature's 9.6 kcal/mol, and the record will print both, labelled);
not a claim that order 3 suffices beyond the gauged domain; not
heteronuclear; not a claim about liquid hydrogen at scale (MAX_ATOMS is
16 and the box is a pedagogy).
