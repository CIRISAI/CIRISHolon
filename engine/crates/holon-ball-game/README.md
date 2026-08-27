# CIRISHolon ball / wall fracture gate

This crate is a dependency-light Rust/WASM game: click or tap a one-million-holon stone
wall to launch a 10,000-holon ball. Rust owns integration, contact, cohesive damage, and
crack state. The browser owns input and Canvas pixels.

```sh
cd sim_engine
cargo test -p holon-ball-game
./crates/holon-ball-game/build-web.sh
cd crates/holon-ball-game/viewer
python3 -m http.server 4177
```

Then open <http://127.0.0.1:4177>.

The material model and its relationship to common holon identity are documented in
[`../../MATERIALS_AND_FRACTURE.md`](../../MATERIALS_AND_FRACTURE.md).
