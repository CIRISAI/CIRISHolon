#!/bin/sh
# Build BOTH browser artifacts for the 3D atom world and stage them under docs/atoms3d/.
#
# Two artifacts, not one, and that is a property of the engine rather than a choice:
# Bevy issue #13168 blocks a single wasm binary from selecting its wgpu backend at
# runtime, so `webgpu` and `webgl2` are separate builds of separate binaries. The page
# picks between them at load (see docs/atoms3d/index.html) by asking `navigator.gpu`.
#
#   webgpu   primary. HDR + Bloom; the emissive bonds glow.
#   webgl2   the phone-compatible fallback. NO Hdr and NO Bloom — WebGL2 cannot do HDR
#            in the browser (bevy #7352) and a camera marked Hdr on that backend renders
#            magenta. Same geometry, same physics, same overlay, plain LDR. `render.rs`
#            carries the cfg that enforces it.
#
# This is NOT the shape holon-render/build-web.sh uses, and the difference is worth
# stating: that artifact is a raw `extern "C"` cdylib with no bindgen step at all, which
# is how it fits in 111 KB. A Bevy app needs wasm-bindgen's JS glue (canvas, winit event
# loop, wgpu surface), so this script runs `cargo build`, then `wasm-bindgen`, then
# `wasm-opt`, and the result is megabytes. See README.md on what the megabytes buy.
#
# Requirements:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <the version in Cargo.lock> --locked
#   wasm-opt (binaryen) on PATH -- OPTIONAL; skipped with a warning, never faked.
#
# The wasm-bindgen CLI version MUST match the `wasm-bindgen` crate in Cargo.lock or
# wasm-bindgen hard-errors with "expected version X, found Y". This script reads the
# expected version out of the lockfile and checks it, rather than pinning a number here
# that would go stale the next time the lock moves.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo=$(CDPATH= cd -- "$here/../../.." && pwd)
out="${ATOMS3D_OUT:-$repo/docs/atoms3d}"

want=$(awk '/^name = "wasm-bindgen"$/ {getline; gsub(/[",]/, "", $3); print $3; exit}' \
  "$here/Cargo.lock")
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen not on PATH. Install it with:" >&2
  echo "  cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi
have=$(wasm-bindgen --version | awk '{print $2}')
if [ "$have" != "$want" ]; then
  echo "wasm-bindgen CLI is $have but Cargo.lock wants $want; they must match." >&2
  echo "  cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi

if command -v wasm-opt >/dev/null 2>&1; then
  opt=1
else
  opt=0
  echo "WARNING: wasm-opt not on PATH; shipping UNOPTIMISED wasm. Sizes below are not" >&2
  echo "         the sizes a release deploy would produce." >&2
fi

build_one() {
  backend=$1
  echo "=== $backend ==="
  # --no-default-features because `default = native`, which pulls x11/wayland and will
  # not cross-compile.
  #
  # Plain `--release` at opt-level 3, NOT holon-render's `opt-level = z`. That crate is a
  # numeric kernel whose whole claim is 111 KB, and size is the right thing to optimise
  # for it. This one has a per-frame O(N^2) force loop and a renderer, and `z` would buy
  # a few percent of a multi-megabyte artifact by making the physics slower — which the
  # calibration would then report back as a lower N_max on every device. wasm-opt -Oz
  # below does the size pass on the finished module instead, where it costs no speed.
  # The profile itself is in Cargo.toml, which applies because this crate is its own
  # workspace root.
  cargo build \
    --manifest-path "$here/Cargo.toml" \
    --no-default-features --features "$backend" \
    --release --target wasm32-unknown-unknown

  raw="$here/target/wasm32-unknown-unknown/release/holon-atoms-3d.wasm"
  dest="$out/$backend"
  mkdir -p "$dest"
  # --no-typescript: nothing here consumes the .d.ts files, and a deploy directory
  # should not carry 18 KB of type declarations for a page that imports one function.
  wasm-bindgen --target web --no-typescript --out-dir "$dest" --out-name atoms3d "$raw"
  if [ "$opt" = "1" ]; then
    # --enable-bulk-memory / --enable-nontrapping-float-to-int: rustc emits both, and
    # wasm-opt REFUSES the module rather than miscompiling it if they are not named.
    # --strip-debug / --strip-producers: measured at 35 KB on a 30 MB module, so this is
    # tidiness rather than a size lever — the module is genuinely that much code.
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
      --strip-debug --strip-producers \
      -o "$dest/atoms3d_bg.wasm" "$dest/atoms3d_bg.wasm"
  fi
  bytes=$(wc -c < "$dest/atoms3d_bg.wasm")
  # Report the COMPRESSED size next to the raw one. A browser downloads the compressed
  # bytes, so the raw figure alone overstates what a phone on a slow link actually waits
  # for by about 3.3x.
  gz=$(gzip -9 -c "$dest/atoms3d_bg.wasm" | wc -c)
  printf '%s: %s bytes raw, %s bytes gzip -9 (%s)\n' \
    "$backend" "$bytes" "$gz" \
    "$([ "$opt" = 1 ] && echo 'wasm-opt -Oz' || echo 'UNOPTIMISED')"
}

build_one webgpu
build_one webgl2

echo
echo "staged in $out"
ls -l "$out"/webgpu/atoms3d_bg.wasm "$out"/webgl2/atoms3d_bg.wasm
