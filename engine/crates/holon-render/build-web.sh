#!/bin/sh
# Build the browser module for the atom renderer.
#
# The size profile is set through environment variables rather than a
# `[profile.release]` section in the crate's own manifest: a workspace member's profile
# section is silently IGNORED by Cargo, so declaring one there would look like it worked
# and would not. This mirrors holon-sandbox/build-web.sh, including the reasons in its
# header, and those reasons are load-bearing here for the same reason they are there.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
workspace=$(CDPATH= cd -- "$here/../.." && pwd)

out="${HOLON_RENDER_WASM_OUT:-$here/viewer/holon_render.wasm}"

# Dedicated target dir: the shipped artifact must be a function of (source, toolchain)
# alone. Sharing the workspace target/ with plain-release builds makes the bytes depend
# on BUILD ORDER, because cargo reuses artifacts fingerprinted under other flag sets.
#
# CARGO_PROFILE_RELEASE_DEBUG is pinned explicitly rather than inherited: the workspace
# root carries `[profile.release] debug = true` for other members' bench binaries, and
# `-C debuginfo=2` changes codegen decisions that survive stripping (stripping happens
# after codegen, not before) — which cost holon-sandbox its reproducibility once already.
CARGO_TARGET_DIR="$workspace/target/web-dist" \
CARGO_PROFILE_RELEASE_OPT_LEVEL=3 \
CARGO_PROFILE_RELEASE_LTO=true \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
CARGO_PROFILE_RELEASE_PANIC=abort \
CARGO_PROFILE_RELEASE_STRIP=true \
CARGO_PROFILE_RELEASE_DEBUG=false \
cargo build \
  --manifest-path "$workspace/Cargo.toml" \
  --package holon-render \
  --target wasm32-unknown-unknown \
  --release

mkdir -p "$(dirname -- "$out")"
cp "$workspace/target/web-dist/wasm32-unknown-unknown/release/holon_render.wasm" "$out"

printf 'Built %s (%s bytes)\n' "$out" "$(wc -c < "$out")"
