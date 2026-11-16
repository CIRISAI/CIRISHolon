#!/bin/sh
# Build holon-draw's browser artifact for a consumer: holon_draw.wasm (the core and the twin's
# leaf builder, plain extern "C" exports, no imports) plus web/draw.js and web/text.js, into
# $1 (default: engine/target/holon-draw-dist). The same release settings as holon-render's.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
workspace=$(CDPATH= cd -- "$here/../.." && pwd)
out="${1:-$workspace/target/holon-draw-dist}"
CARGO_TARGET_DIR="$workspace/target/web-dist" \
CARGO_PROFILE_RELEASE_OPT_LEVEL=3 \
CARGO_PROFILE_RELEASE_LTO=true \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
CARGO_PROFILE_RELEASE_PANIC=abort \
CARGO_PROFILE_RELEASE_STRIP=true \
CARGO_PROFILE_RELEASE_DEBUG=false \
cargo build --manifest-path "$workspace/Cargo.toml" --package holon-draw --target wasm32-unknown-unknown --release
mkdir -p "$out"
cp "$workspace/target/web-dist/wasm32-unknown-unknown/release/holon_draw.wasm" "$here/web/draw.js" "$here/web/text.js" "$out/"
printf 'Built %s (%s bytes)\n' "$out/holon_draw.wasm" "$(wc -c < "$out/holon_draw.wasm")"
