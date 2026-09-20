#!/bin/bash
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; BIN="$ROOT/engine/target/release/examples/replace0"; BASE="$ROOT/conformance/water_observatory/replace0"; STIFF="$BASE/transport_seed0/stiffness.json"
cd "$ROOT/engine" || exit 2; OUT="$BASE/tscan1"; mkdir -p "$OUT"
i=0
for T in 293 350 400 500; do for seed in 0 1; do
  cores=(17 18 21 22 23 25 26 27); core=${cores[$i]}; i=$((i+1)); d="$OUT/T${T}_seed$seed"; mkdir -p "$d"
  ( taskset -c $core "$BIN" scout "$d" --cells 4 --settle 1000 --settle-rigid 1000 --readouts 250 --readout-fs 20 --stiffness "$STIFF" --seed $seed --workers 1 --temperature-k $T > "$d/scout.log" 2> "$d/scout.err"; echo $? > "$d/run.done" ) &
done; done
echo "tscan1: $(date -Is) eight arms launched"; wait; echo 0 > "$OUT/tscan1.DONE"
