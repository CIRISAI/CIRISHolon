# Particles on a sphere

This crate is the first rendered demonstration of CIRISHolon's sparse Newtonian mechanics
path, introduced in PR #7. Native Rust generates deterministic frame data; a
dependency-free Canvas 2D viewer renders those recorded positions in a browser.

From `sim_engine/`:

```sh
cargo run --release -p sphere-demo
python3 -m http.server 4173 --directory crates/sphere-demo/viewer
```

Then open `http://127.0.0.1:4173/`.

The generated scenarios are:

- coherent spin — a smooth near-rigid flow;
- counter-rotating shear — opposite hemispheric flows that exercise contacts;
- standing surface wave — a deterministic three-lobed tangential impulse.

The sparse core supplies spring forces, weighted-degree masses, velocity Verlet, and
frictionless sphere impulses with restitution `e = 0.96`. This crate supplies the explicit
spherical boundary choice: positions are projected to radius one and velocities to the
tangent plane after each step. That constraint is a demo boundary condition, not an
ontology-derived quantity.
