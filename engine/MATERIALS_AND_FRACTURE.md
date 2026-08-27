# Materials and fracture in CIRISHolon

Status: executable Newtonian slice. Date: 2026-08-22. License: AGPL-3.0-or-later.

## The two statements

"This holon is made of stone" and "this holon is weakly connected to the next" are
different assertions.

The first is a constitutive description of one holon. The second is a description of the
relation between two holons. Neither requires a new base class, entity enum, or ontology.
The executable forms are in `ciris-sim-core/src/material.rs`:

```rust
MaterialBinding {
    subject_holon: WALL_HOLON,
    descriptor_holon: STONE_DESCRIPTOR_HOLON,
    properties: IsotropicMaterial::DEMO_CALIBRATION,
}
```

Both IDs address ordinary holons. The descriptor holon can carry the full common CIRIS
coordinates, Record history, provenance, warrant, uncertainty, specimen, grade, and
applicability conditions. `IsotropicMaterial` is the Newtonian chart that reads the subset
needed for this update: density, elastic modulus, Poisson ratio, damping, restitution,
tensile/compressive strength, and fracture energy.

The demo preset is explicitly illustrative. Production code should resolve a warranted
descriptor holon for a specific stone, orientation, moisture/temperature condition, test
method, and uncertainty. Changing that record changes the chart parameters; it does not
change what kind of object the wall is.

## Weak connection

A connection is itself addressable as a relation holon:

```rust
CohesiveBond::new(
    relation_holon,
    left_holon,
    right_holon,
    rest_length,
    CohesiveLaw {
        stiffness_n_m,
        damping_n_s_m,
        peak_force_n,
        fracture_energy_j,
    },
)
```

The discrete bilinear cohesive law is elastic up to
`opening_at_peak = peak_force / stiffness`. It then softens irreversibly to zero at
`opening_at_failure = 2 * fracture_energy / peak_force`. Damage `D` is monotone from zero
to one. Lower peak force means the interface starts failing sooner; lower fracture energy
means it requires less work to separate. `CohesiveLaw::weakened` makes those two choices
explicit instead of hiding a "crack chance" in rendering code.

Compression/contact after separation belongs to the contact solver, not to the broken
bond. That distinction prevents a failed relation from continuing to transmit tensile
force merely because its endpoints remain close.

## What a crack is

A crack is the boundary induced by failed relations:

```text
crack = { relation holon r | damage(r) = 1 }
```

It is an observable over the common holarchy, not a new object pasted over it. A crack can
still be re-rooted and described as a holon when an application needs its extent, history,
warrant, gameplay meaning, or narrative consequences. The mechanical evaluator does not
need a special metaphysical exception to find it.

## Gross count and resident grain

The browser scene defines exact REG+ gross states:

- ball: 10,000 terminal holons;
- stone wall: 1,000,000 terminal holons;
- encounter root: 1,010,000 terminal holons by exact `GrossState::combine`.

The visible wall is a resident frontier of 288 node holons joined by 797 relation holons.
Each node represents either 3,472 or 3,473 terminal holons, distributed so the sum is
exactly one million. This is composition, not a claim that 288 particles literally are a
stone specimen.

This gate deliberately holds the resident fracture frontier fixed. It proves the missing
semantics—material binding, weak relation, irreversible damage, deterministic trajectory,
and browser execution—before introducing another adaptive variable. The next step is to
make bond damage and crack-tip residual the `BoundaryModel` priority: begin coarse, split
only relations whose uncertainty can change the declared macro observables, and stop at the
coarsest frontier meeting crack-path, impulse, conservation, and render tolerances.

## Run the gate

```sh
cd sim_engine
cargo test -p holon-ball-game
./crates/holon-ball-game/build-web.sh
cd crates/holon-ball-game/viewer
python3 -m http.server 4177
```

Open `http://127.0.0.1:4177`, then tap or click the wall. JavaScript sends only the target
height and throw speed into the compiled Rust module. Rust owns the trajectory, contacts,
cohesive forces, damage, and cracks; JavaScript reads state back for Canvas rendering.
The native gate sweeps five aim heights across the full wall at 8.0, 13.8, and 18.0 m/s;
all 15 combinations must remain finite and produce an impact, so the rendered centre throw
is evidence, not a uniquely passing trajectory.

The verified full-pipeline artifact is
[`output/playwright/cirisholon-ball-wall-fracture.png`](output/playwright/cirisholon-ball-wall-fracture.png).
