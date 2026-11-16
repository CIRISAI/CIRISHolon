#!/bin/sh
# Build the atoms scene's browser artifact into docs/atoms-draw/, beside a copy of the draw
# backend and its text provider. The same release settings as holon-render's build-web.sh.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
workspace=$(CDPATH= cd -- "$here/../.." && pwd)
out="$workspace/../docs/atoms-draw"
CARGO_TARGET_DIR="$workspace/target/web-dist" \
CARGO_PROFILE_RELEASE_OPT_LEVEL=3 \
CARGO_PROFILE_RELEASE_LTO=true \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
CARGO_PROFILE_RELEASE_PANIC=abort \
CARGO_PROFILE_RELEASE_STRIP=true \
CARGO_PROFILE_RELEASE_DEBUG=false \
cargo build --manifest-path "$workspace/Cargo.toml" --package holon-draw-atoms --target wasm32-unknown-unknown --release
mkdir -p "$out"
cp "$workspace/target/web-dist/wasm32-unknown-unknown/release/holon_draw_atoms.wasm" "$out/"
cp "$workspace/crates/holon-draw/web/draw.js" "$out/"
cp "$workspace/crates/holon-draw/web/text.js" "$out/"
printf 'Built %s (%s bytes)\n' "$out/holon_draw_atoms.wasm" "$(wc -c < "$out/holon_draw_atoms.wasm")"
