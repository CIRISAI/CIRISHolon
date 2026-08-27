# CIRISHolon component boundary

This crate is the CEWPOS-style adapter around the same `ciris-sim-core` kernel that a
CIRISGame-style Rust/Bevy host links directly. Its WIT world imports nothing and exports
one total, typed `certify-sphere` operation.

Build and lift the component without requiring a globally installed `wasm-tools`:

```sh
cargo build --release --target wasm32-unknown-unknown
mkdir -p dist
cargo run --release --example lift -- \
  target/wasm32-unknown-unknown/release/ciris_sim_component.wasm \
  dist/ciris_sim_component.wasm
```

`examples/lift.rs` uses the same `wit-component` encoder that backs `wasm-tools
component new`, with validation enabled. The core module contains the generated
component-type custom section and has zero imports. The lifted artifact is a WebAssembly
Component suitable for a Wasmtime component host.

Do not call this component once per render frame. The canonical ABI necessarily maps
the WIT lists at the sandbox boundary. A trusted Rust/browser host should enable the
core's `alloc` feature, keep `RuntimeArena` and `RuntimeFrontier` as long-lived resources,
and call the kernel directly. That path has no JSON, JavaScript callbacks, trait objects,
or cross-component copy in the simulation loop.
