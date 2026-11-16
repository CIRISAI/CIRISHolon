#!/bin/sh
# Build the ladder's browser artifact into docs/ladder/, beside copies of draw.js and text.js.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
workspace=$(CDPATH= cd -- "$here/../.." && pwd)
out="$workspace/../docs/ladder"
CARGO_TARGET_DIR="$workspace/target/web-dist" \
CARGO_PROFILE_RELEASE_OPT_LEVEL=3 \
CARGO_PROFILE_RELEASE_LTO=true \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
CARGO_PROFILE_RELEASE_PANIC=abort \
CARGO_PROFILE_RELEASE_STRIP=true \
CARGO_PROFILE_RELEASE_DEBUG=false \
cargo build --manifest-path "$workspace/Cargo.toml" --package holon-draw-ladder --target wasm32-unknown-unknown --release
mkdir -p "$out"
cp "$workspace/target/web-dist/wasm32-unknown-unknown/release/holon_draw_ladder.wasm" "$out/"
cp "$workspace/crates/holon-draw/web/draw.js" "$workspace/crates/holon-draw/web/text.js" "$out/"
printf 'Built %s (%s bytes)\n' "$out/holon_draw_ladder.wasm" "$(wc -c < "$out/holon_draw_ladder.wasm")"
