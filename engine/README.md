# CIRISHolon (incubating in CIRISOntology)

CIRISHolon is CIRIS's AGPL-3.0-or-later dynamics engine: one recursively compositional
holon, exact REG+ gross composition, separate whole-only state, and certificate-directed
refinement to the minimum viable grain for a declared macroscopic claim.

PR #7 is the Newtonian architectural slice. It includes the deterministic `no_std`
Rust kernel, runtime-sized and allocator-free storage policies, adaptive sphere-contact
certification, a zero-import WebAssembly Component boundary, a rendered particle-on-sphere
pipeline, and solution-gated analytic/Rapier controls. Einstein is next; the
metaphysical/linguistic closure is already substrate.

The first material/fracture gate is executable too: a browser tap launches a
10,000-holon ball into a 1,000,000-holon stone wall. Rust/WASM owns contact, cohesive
damage, and crack formation while Canvas owns only input and pixels. See
[`MATERIALS_AND_FRACTURE.md`](MATERIALS_AND_FRACTURE.md) for how "made of stone" and
"weakly connected" remain relations inside the common holon model.

Start here:

- [`MISSION.md`](MISSION.md) — why the engine exists, mission inheritance, AGPL, and the
  Newton → Einstein order;
- [`CIRIS_HOLON_ENGINE.md`](CIRIS_HOLON_ENGINE.md) — executable architecture, current
  guarantees, measurements, and explicit remaining work;
- [`SPHERE_RAPIER_BENCHMARK.md`](SPHERE_RAPIER_BENCHMARK.md) — exact-reference parity and
  withdrawn divergent-trajectory comparison;
- [`CIRIS_HOLON_HOME.md`](CIRIS_HOLON_HOME.md) — dedicated-home layout and history-preserving
  extraction plan;
- [`PRIOR_ART.md`](PRIOR_ART.md) — public defensive publication and prior-art boundary;
- [`H3ERE2_RESTART.md`](H3ERE2_RESTART.md) — recovered Qwen3-0.6B work and restart gate.
- [`MATERIALS_AND_FRACTURE.md`](MATERIALS_AND_FRACTURE.md) — constitutive descriptor
  holons, cohesive relation holons, cracks, and the interactive Rust/WASM gate.

Core verification:

```sh
./ci-gates.sh
cargo test -p ciris-sim-core --features alloc
cargo check --workspace --all-targets --features alloc
```

Build and serve the interactive fracture gate:

```sh
./crates/holon-ball-game/build-web.sh
cd crates/holon-ball-game/viewer
python3 -m http.server 4177
```

The rendered demo and Rapier reproduction commands are documented in their respective
reports. The runtime remains here until the extraction gates in `CIRIS_HOLON_HOME.md` pass
and a canonical `CIRISAI/CIRISHolon` repository is created with preserved history.
