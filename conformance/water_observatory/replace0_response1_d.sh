#!/bin/bash
# RESPONSE-1, two more 200 m/s LONGITUDINAL seeds (2026-09-20, on the partial arms' read):
# R1 at 50 m/s sits at its floor by arithmetic, so the stake can be met only on the 200 m/s
# control, and one seed is not a reading. Seeds 1 and 2 on E-cores 16 and 20 (the P-cores
# carry the eight arms); ~38 h each. Same binary and arguments as the control.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
STIFF="$BASE/transport_seed0/stiffness.json"
cd "$ROOT/engine" || exit 2
arm() { local core=$1 seed=$2; local out="$BASE/response1_L200_seed$seed"; mkdir -p "$out"
  echo "response1-d: $(date -Is) arm L200 seed $seed on core $core"
  taskset -c "$core" "$BIN" scout "$out" --cells 6 --settle 3000 --settle-rigid 3000 --readout-fs 10 --stiffness "$STIFF" --seed "$seed" --workers 1 \
      --kick L --kick-mps 200 --kick-cycles 12 --kick-relax 314 > "$out/scout.log" 2> "$out/scout.err"
  local c=$?; echo "$c" > "$out/run.done"; echo "response1-d: $(date -Is) arm L200 seed $seed exit $c"; }
arm 16 1 & arm 20 2 & wait
