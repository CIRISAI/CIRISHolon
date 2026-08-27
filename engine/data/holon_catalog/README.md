# CIRISHolon chemistry/material catalog

Machine-readable source data for descriptor holons. This directory is intentionally
**not** a second material ontology: element/species/material records become descriptor
holons, and realization-specific charts decide which warranted properties they consume.

## Files

- `elements.csv` — all 118 named elements. CIAAW 2024 abridged standard atomic weights
  are numeric where a standard atomic weight exists; blanks are semantically `unresolved`,
  especially for radioactive elements. Never substitute atomic number or a guessed isotope mass.
- `electron_configurations.csv` — NIST neutral ground configurations H through U. The
  source itself scopes the table to H-U; Z>92 is deliberately absent rather than extrapolated.
- `species.csv` — species identity, stoichiometric formula and molar-mass reference data.
  These records may identify what inventory is being tested; they do **not** prescribe its bond graph.
- `chemistry_validation_targets.jsonl` — known atom/bond graphs and qualitative geometries
  used only as withheld validation targets. They carry `must_not_parameterize_dynamics=true`.
  A chemistry realization earns these structures by producing them from holonic dynamics.
- `materials.jsonl` — specimen/grade/composition/property records. Properties carry their
  own source and conditions. A composition record is not automatically a constitutive law.
- `mixture_rules.json` — explicit conversions and effective-property models. Definitions,
  bounds and estimates are labeled separately. Strength/fracture fields have no generic
  mixing rule and therefore fail closed.
- `provenance.json` — source registry. Every `source_id` in the catalog must resolve here.
- `manifest.json` — version, unit convention and fail-closed behavior.

## Simulator contract

The runtime should compile only **permitted inputs** into static Rust tables; it should not
parse CSV/JSON inside the `no_std` stepping loop. A chemistry/material descriptor compiler may:

1. resolve elemental identity/electronic-state data and a stoichiometric inventory;
2. select conditions (T, P, phase, porosity, heat treatment, loading mode);
3. resolve only properties warranted at those conditions;
4. apply a named mixing/homogenization model when one is explicitly selected;
5. run holonic dynamics to determine stable relations/configurations;
6. compare the resulting structure against withheld chemistry/crystal validation targets;
7. return an unresolved/refinement error rather than manufacture a missing property or bond.

**Atom/bond topology is an output.** A known molecule graph, crystal unit cell, bond order or
coordination number must not be copied into the initial holon decomposition merely because the
reference catalog knows the answer. Doing so would reduce the chemistry lane to a renderer.

The intended mapping is: elemental/electronic identity is `Identity/Facts`; stoichiometric
inventory is the gross/input ledger; emergent bonding is derived `Structure`; conditions are
`Circumstances`; the dynamical/constitutive law is `Model/Rules`; source/specimen identity is
`Record/Warrant`.

## Important scope boundaries

Atomic weight is not isotope mass. CIAAW standard atomic weights describe normal-material
isotopic composition and can be intervals in the full table. The abridged values here are
convenient recommended values with uncertainties for ordinary molar-mass work; isotope-resolved
quantum/nuclear simulations need an isotope catalog, not a replacement value in this table.

Likewise, alloy composition does not uniquely determine strength, fracture energy, fatigue,
damping or friction. Those depend strongly on processing, microstructure, geometry and loading.
The material records therefore mark such simulator fields `unresolved` until a warranted
specimen/grade/property source is added.

## Next data layers

The schema can extend without changing the holon type: isotope masses/abundances,
oxidation/ionic states, phase diagrams, thermochemical tables, spectroscopic targets,
crystal/unit-cell **validation targets**, and specimen-level fracture/fatigue datasets.
Force-field or bond parameters may be added only as explicitly named comparison/control models;
they must not silently become the fundamental holonic dynamics whose success is being tested.
