# DM gauge-vacuum + emergent chemistry handoff

Status snapshot: 2026-08-23. Research branch only. No CI result is used as evidence here.

## Banked into Lean

- `Core.DMGauge`
  - reuses a three-state local carrier as spin-1 electric flux `-1,0,+1`;
  - truncated raising changes flux by exactly one;
  - a plaquette magnetic move preserves every lattice Gauss charge;
  - the uniform closed-flux one-plaquette physical sector has exactly three states.
- `Core.DMModular`
  - for a charge-conjugation-symmetric link reduction `(p,p0,p)`, the modular energy is exactly
    `K = -log(p0) I + log(p0/p) E^2`;
  - therefore the local modular generator needs no operator beyond the gauge electric-energy
    operator once the reduced state has that symmetry;
  - a flat spectrum gives zero modular gauge coupling, preserving the C5 failure as a useful kill.

These statements do **not** identify the existing route Hamiltonian with the spin-1 ladder.
They establish that the same finite holonic carrier admits a gauge realization; an operator-level
route -> gauge derivation remains an explicit target.

## Banked into the simulator

`ciris-sim-core::quantum_link` now provides an executable, no_std-compatible probe:

- spin-1 flux and truncated raising/lowering;
- Gauss-law evaluation and exact plaquette-move preservation tests;
- one-plaquette electric + magnetic Hamiltonian solved with the existing deterministic Jacobi path;
- one-link modular `a I + beta E^2` residual;
- GF(2) boundary-channel rank, kept distinct from raw cut-edge count.

A local sanity evaluation at `g^2 = kappa = 1` gave one-link probabilities approximately
`(0.0458759, 0.908248, 0.0458759)`, `beta ~= 2.98558`, with modular-electric residual at numerical
roundoff. This is a diagnostic, not a promoted physical claim.

## Banked chemistry/material source layer

`sim_engine/data/holon_catalog/` now contains:

- all 118 element identities with CIAAW abridged standard atomic weights where defined;
- NIST neutral ground electronic configurations H through U;
- starter species/formula/molar-mass records;
- provenance-backed common material composition/property records;
- explicit mixture/homogenization models with assumptions and fail-closed fields;
- a provenance registry and manifest;
- a stdlib-only validator.

### Critical chemistry rule

**Atom/bond topology is not an input.**

Known molecular graphs are stored only in `chemistry_validation_targets.jsonl` with
`role = withheld_validation_target` and `must_not_parameterize_dynamics = true`.
The validator rejects a target that does not carry that fence.

Allowed inputs may include elemental/electronic identity, stoichiometric inventory, external
circumstances, and explicitly named control/model parameters. Stable atom/bond relations,
bond order, molecular geometry, coordination and eventually crystal topology must be outcomes
of the holonic dynamics. Otherwise the chemistry lane has only rendered a lookup table.

## Next decisive tests

1. **Route -> gauge operator derivation:** derive or kill a principled map from the existing
   three-route dynamics to the spin-1 link algebra, rather than relying only on equal dimension.
2. **Gauge-vacuum scaling:** move beyond one/two plaquettes and test whether modular locality,
   boundary-channel entropy and an emergent causal scale survive increasing lattices.
3. **Emergent chemistry:** given only elemental/electronic state + stoichiometric inventory,
   recover withheld targets such as H2O/CO2/CH4/NH3 topology and qualitative geometry without
   injecting their bonds.
4. **Cross-scale chemistry/material bridge:** derive material composition/property behavior from
   emerged molecular/crystal structure where possible; retain specimen-level empirical properties
   as Record-backed facts where derivation is not yet earned.

Kill conditions remain simple: if the required known bond graphs, gauge constraints, modular
structure or macroscopic observables appear only after inserting the answer as chart-specific
state/law, the claimed unification has not been gained.
