#!/bin/bash
# THE FLUID-ELEMENT SCOUT, FULL RUN (RUNG2_AMENDMENT_5.md): 432 waters on the rigid operator,
# N seeds, 196 ps of production each = 250 windows of 785 fs at 2x2x1, priced at the pilot's
# measured 3,609 core-s/ps (~197 core-h a seed). Rigid settle by criterion (floor 3000 steps,
# cap 30000). Walks appended at every readout. Usage: replace0_scout_full.sh [n_seeds]
set -u
NSEEDS="${1:-3}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
STIFF="$BASE/transport_seed0/stiffness.json"
FLOOR_GIB=12
cd "$ROOT/engine" || exit 2
rm -f "$BASE/scout_full.KILLED" "$BASE/scout_full.DONE" "$BASE/scout_full.LOWDISK"
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
have=$(free_gib); echo "scout full: $(date -Is) pid $$; $have GiB free; $NSEEDS seeds"
[ "$have" -lt "$FLOOR_GIB" ] && { echo "REFUSED: $have GiB free" > "$BASE/scout_full.LOWDISK"; exit 3; }
pids=()
for k in $(seq 0 $((NSEEDS - 1))); do
  OUT="$BASE/scout432_full_seed$k"; mkdir -p "$OUT"
  taskset -c $((24 + k)) "$BIN" scout "$OUT" --cells 6 --settle 3000 --settle-rigid 3000 --readouts 9800 --readout-fs 20 --stiffness "$STIFF" --seed "$k" \
      > "$OUT/scout.log" 2> "$OUT/scout.err" &
  pids+=($!); echo "scout full: seed $k on core $((24 + k)) pid ${pids[$k]}"
done
( while pgrep -f "replace0 scout .*scout432_full" > /dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then echo "$(date -Is): under the floor; arms stopped" > "$BASE/scout_full.LOWDISK"; kill "${pids[@]}" 2>/dev/null; break; fi; sleep 300
  done ) &
watch=$!; fail=0
for k in $(seq 0 $((NSEEDS - 1))); do wait "${pids[$k]}"; c=$?; echo "scout full: seed $k exit $c at $(date -Is)"; [ "$c" -ne 0 ] && fail=1; done
kill "$watch" 2>/dev/null
[ "$fail" -ne 0 ] && { echo "a scout arm exited nonzero; see scout432_full_seed*/scout.err" > "$BASE/scout_full.KILLED"; exit 1; }
echo 0 > "$BASE/scout_full.DONE"
