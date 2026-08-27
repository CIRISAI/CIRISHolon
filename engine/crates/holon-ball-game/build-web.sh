#!/bin/sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
workspace=$(CDPATH= cd -- "$here/../.." && pwd)

cargo build \
  --manifest-path "$workspace/Cargo.toml" \
  --package holon-ball-game \
  --target wasm32-unknown-unknown \
  --release

cp "$workspace/target/wasm32-unknown-unknown/release/holon_ball_game.wasm" \
  "$here/viewer/holon_ball_game.wasm"

printf 'Built %s\n' "$here/viewer/holon_ball_game.wasm"
