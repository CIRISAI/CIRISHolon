#!/bin/bash
# SLOW-3 (SLOW3_PREREG.md): two FRESH rigid-operator arms, 293 K seeds 3 and 4, exactly as SLOW-1's arms
# (slow1.sh): 250 waters, 50 ps at 20 fs, the same settle and flags, one per E-core (16, 17).
# The binary: SLOW-1 ran the main checkout's engine/target/release/examples/replace0 (built 2026-09-20 10:22,
# the source of 65d4b2c, per SLOW1_PREREG.md). That target directory no longer exists (found 2026-09-25).
# It was rebuilt from `git archive 65d4b2c engine` (toolchain 1.95.0, pinned by that tree's
# rust-toolchain.toml; `cargo build --release --example replace0 -p holon-render`) with ONE line changed:
# `SEEDS` extended from 3 to 5 entries (0x...4533, 0x...4534, the same "REPLACE<k>" pattern), because the
# binary panics on `--seed 3` otherwise ("seed index 3 against 3 declared seeds", the first launch, logged in
# replace0/slow3/slow3_launch_attempt1_panic.log). Seeds 0-2 are unchanged. The same line is committed in
# engine/crates/holon-render/examples/replace0.rs. Installed as replace0/slow3/replace0_slow3
# (sha256 29cb57c9b92fb2e7...). slow3_provenance.sh checks it against SLOW-1's seed-0 walk, row for row.
set -u
ROOT="${SLOW3_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"; BASE="$ROOT/conformance/water_observatory/replace0"
BIN="$BASE/slow3/replace0_slow3"; STIFF="$BASE/transport_seed0/stiffness.json"
[ -x "$BIN" ] || { echo "slow3: no binary at $BIN"; exit 2; }
cd "$ROOT/engine" || exit 2; OUT="$BASE/slow3"; mkdir -p "$OUT"
echo "slow3: $(date -Is) binary sha256 $(sha256sum "$BIN" | cut -c1-64)"
arms=("293 3 16" "293 4 17")
for a in "${arms[@]}"; do
  set -- $a; T=$1; seed=$2; core=$3; d="$OUT/T${T}_seed$seed"; mkdir -p "$d"
  ( taskset -c $core "$BIN" scout "$d" --cells 5 --settle 1000 --settle-rigid 1000 --readouts 2500 --readout-fs 20 --stiffness "$STIFF" --seed $seed --workers 1 --temperature-k $T > "$d/scout.log" 2> "$d/scout.err"; echo $? > "$d/run.done" ) &
  echo "slow3: $(date -Is) arm T${T}_seed$seed on core $core"
done
echo "slow3: $(date -Is) two arms launched"; wait; echo 0 > "$OUT/slow3.DONE"; echo "slow3: $(date -Is) done"
