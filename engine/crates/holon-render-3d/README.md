# holon-render-3d

The atom world in three dimensions. Hydrogen atoms in a box under the exact STO-3G pair
potential — solved by full CI in the browser at load, not fetched — integrated with
velocity Verlet, with every energy and momentum flow on a gated ledger. Drawn with Bevy
0.19; pushed around with a finger.

Sibling of `docs/atoms`, the 2D canvas shell. Same core, two views.

## What is here and what is not

The physics is **not** here. `holon-render` owns the potential table, the integrator, the
ledger and its two gates, the bond predicate, the three clocks and the composite-holon
census. This crate owns a camera, some spheres, some cylinders, an overlay and a finger.

Every number the overlay shows is *read* from the `Sim`, never recomputed. That is the
point of the arrangement rather than a detail of it: if a gate reads PASS in the browser
and FAIL here, the difference is a difference in the run, not in the arithmetic of two
HUDs.

## What the 3D lift changed in the physics

`holon-render`'s `sim.rs` now carries three components per atom and six faces on the box.
Almost nothing needed re-deriving, because almost nothing was ever two-dimensional: the
curve, the force law, the bond predicate, the outer turning point, the drift bound and
all three clocks are functions of the scalar separation alone.

Exactly two things were genuinely dimension-dependent, and both are named at their
definitions rather than inferred:

- **the equipartition denominator** behind the temperature reading — two translational
  degrees of freedom per atom against three;
- **the opening arrangement** of the atoms — a ring becomes a Fibonacci sphere.

One more changed shape without changing meaning: the centrifugal term in the
turning-point solve now uses `|L|²` from the full cross product rather than `L_z²`. On the
mid-plane the two transverse components are exact zeros, so the 2D value is unchanged bit
for bit.

The 2D scene is now the exact `z = depth/2` **slice** of this one. It is not a separate
code path and not an approximation: the pair force along z is `(slope/r)·dz` with `dz = 0`,
the z faces are never reached, and every sum that grew a third term grew it in the order
`(xx + yy) + zz`, so a float times zero stays zero and adding zero changes no bit. That is
why the canvas shell, the browser ABI and all 40 of `holon-render`'s existing gate tests
were untouched by the lift.

Both claims are tested rather than asserted, in `holon-render/tests/three_dimensions.rs`:
the mid-plane invariance is checked **on the bits** through walls, a collision and a
dragged spring, and the lift's rotational covariance is checked by running one scene flat
and the same scene rotated into a generic plane and comparing every scalar the gates read.

## Interaction

- **Orbit / zoom / pan** — `bevy_panorbit_camera`, mouse or touch.
- **Drag an atom** — press on it and move. The pointer does not move the atom; it moves
  the anchor of a finite spring whose energy is a term in the Hamiltonian, and the work
  that anchor motion does is posted to `W_ext` in the same breath. This is why `E − W_ext`
  stays constant through a drag, and why the hand can be used as a *brake* to make H₂ —
  in a two-atom scene it is the only channel by which a bond can form at all.
- **Depth** — the drag follows the screen-parallel plane through the atom, frozen at the
  moment of the grab. To move an atom along the view axis, orbit first and then drag: the
  depth you cannot reach is the depth you cannot see.
- **Two or more fingers** is a camera gesture and never a drag, so a pinch cannot pick up
  an atom by accident.

## Build and run

```sh
# native (default feature = `native`)
cargo run --release

# headless: the gates, no GPU, no window. What CI runs.
cargo test --no-default-features --features headless

# both browser artifacts, staged into docs/atoms3d/
cargo install wasm-bindgen-cli --version "$(awk '/^name = "wasm-bindgen"$/ {getline; gsub(/[",]/,"",$3); print $3; exit}' Cargo.lock)" --locked
./build-web.sh            # needs wasm-opt (binaryen) on PATH for the size pass
```

The crate is **outside** the engine workspace (`exclude` in `engine/Cargo.toml`) and
carries its own `[workspace]` table, so build and test it with `--manifest-path` from the
engine root. Bevy brings ~400 crates and a large optional-feature surface, and Cargo
unifies features across a workspace: joining would put wgpu and winit within reach of
`ciris-sim-core`'s graph and falsify the isolation gates in `ci-gates.sh`. The dependency
on `holon-render` runs one way only, so the engine workspace's own resolution is
untouched.

## Two wasm artifacts, not one

Bevy issue #13168 blocks a single wasm binary from selecting its wgpu backend at runtime,
so `webgpu` and `webgl2` are separate builds. `docs/atoms3d/index.html` picks between them
at load by asking `navigator.gpu`, falls back on a start failure, and reports which
backend it ended up on rather than falling back silently. `?backend=webgl2` forces the
fallback for testing.

They are **not** the same picture, and the difference is deliberate:

| | webgpu | webgl2 |
|---|---|---|
| HDR | yes | **no** — WebGL2 cannot do HDR in the browser (bevy #7352); a camera marked `Hdr` on that backend renders magenta |
| Bloom | yes | no — nothing above 1.0 to bloom, so the pass is dropped and `bevy_post_process` is left out of the artifact |
| geometry, physics, overlay, numbers | — | identical |

## Size

A Bevy app is megabytes; `docs/atoms`' canvas shell is 111 KB. Both numbers are honest and
they are not comparable as they stand, because the two artifacts are different kinds of
thing. The 2D shell is a raw `extern "C"` cdylib with **no** wasm-bindgen glue, no
renderer, no ECS, no window: the browser owns input and pixels and the wasm owns the
physics. This artifact carries the ECS, the PBR renderer, the wgpu backend, the winit
event loop, the UI stack and an embedded font, and it draws its own pixels.

See `SIZES.md` for the measured numbers, what was built to produce them, and the split.

## Layout

```
src/world.rs     the Sim as a Bevy resource, the frame advance, the calibration burst
                 (compiles headless — the gate tests link this and nothing else)
src/render.rs    the app, the camera, the per-frame advance
src/scene.rs     the box, the atoms, sim↔world coordinates, the palette
src/bonds.rs     bond cylinders, drawn from the pair predicate's own answer
src/pick.rs      pointer and touch drag through the spring
src/hud.rs       the ledger, both gates, the clocks, the census, the controls
src/lighting.rs  key / fill / rim
tests/headless.rs the gates, run through the shell rather than beside it
```
