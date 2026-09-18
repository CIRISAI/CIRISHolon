#!/bin/bash
# THE FLUID-ELEMENT SCOUT, PILOT STAGE (RUNG2_AMENDMENT_5.md): 432 waters (n = 6, the first
# admissible box), three seeds on the RIGID OPERATOR, 5 ps of production each. The pilot's job
# is to MEASURE the price per picosecond at this size and the averaged fields' spreads at the
# cadence, so the full run (~200 ps a seed) is launched at a known cost, not an extrapolated
# one. Stiffness envelope read from the 128-water record (an intensive property, declared).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
STIFF="$BASE/transport_seed0/stiffness.json"
FLOOR_GIB=8
cd "$ROOT/engine" || exit 2
rm -f "$BASE/scout_pilot.KILLED" "$BASE/scout_pilot.DONE" "$BASE/scout_pilot.LOWDISK"
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
have=$(free_gib); echo "scout pilot: $(date -Is) pid $$; $have GiB free"
[ "$have" -lt "$FLOOR_GIB" ] && { echo "REFUSED: $have GiB free" > "$BASE/scout_pilot.LOWDISK"; exit 3; }
pids=()
for k in 0 1 2; do
  OUT="$BASE/scout432_seed$k"; mkdir -p "$OUT"
  taskset -c $((24 + k)) "$BIN" scout "$OUT" --cells 6 --settle 3000 --settle-rigid 3000 --readouts 250 --readout-fs 20 --stiffness "$STIFF" --seed "$k" \
      > "$OUT/scout.log" 2> "$OUT/scout.err" &
  pids+=($!); echo "scout pilot: seed $k on core $((24 + k)) pid ${pids[$k]}"
done
( while kill -0 "${pids[0]}" 2>/dev/null || kill -0 "${pids[1]}" 2>/dev/null || kill -0 "${pids[2]}" 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then echo "$(date -Is): under the floor; arms stopped" > "$BASE/scout_pilot.LOWDISK"; kill "${pids[@]}" 2>/dev/null; break; fi; sleep 120
  done ) &
watch=$!; fail=0
for k in 0 1 2; do wait "${pids[$k]}"; c=$?; echo "scout pilot: seed $k exit $c at $(date -Is)"; [ "$c" -ne 0 ] && fail=1; done
kill "$watch" 2>/dev/null
[ "$fail" -ne 0 ] && { echo "a scout arm exited nonzero; see scout432_seed*/scout.err" > "$BASE/scout_pilot.KILLED"; exit 1; }
echo 0 > "$BASE/scout_pilot.DONE"
