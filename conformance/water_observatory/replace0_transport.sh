#!/bin/bash
#
# REPLACE-0's TRANSPORT arms (2026-09-12): the question that gates the next tier — does the
# coarse operator's water WALK like the fine model's? Three seeds, each its own branch point
# and its own bundle, both arms NVE from it, ~3 ps counted.
#
# Sized so the MSD ladder clears the gate's anti-vacuity legs: 150 readouts puts the top lag
# at 37 x 20 fs = 740 fs, about 7 crossovers, on 8 lags of the lens's own x1.5 ladder. NO
# diffusion constant is claimed at this length and the gate says so.
#
# Same disk preflight as the LIQUID-2 launchers.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
FLOOR_GIB=8
cd "$ROOT/engine" || exit 2
rm -f "$BASE/transport.KILLED" "$BASE/transport.DONE" "$BASE/transport.LOWDISK"

free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
have=$(free_gib)
echo "transport: $(date -Is) pid $$; $have GiB free against a floor of $FLOOR_GIB"
if [ "$have" -lt "$FLOOR_GIB" ]; then
  echo "REFUSED: $have GiB free, under the $FLOOR_GIB GiB floor." > "$BASE/transport.LOWDISK"; exit 3
fi

pids=()
for k in 0 1 2; do
  core=$((24 + k)); OUT="$BASE/transport_seed$k"; mkdir -p "$OUT"
  taskset -c "$core" "$BIN" run "$OUT" --seed "$k" --settle 102100 --frames 115105 --readouts 150 \
      > "$OUT/run.log" 2> "$OUT/run.err" &
  pids+=($!); echo "transport: seed $k on core $core pid ${pids[$k]}"
done
( while kill -0 "${pids[0]}" 2>/dev/null || kill -0 "${pids[1]}" 2>/dev/null || kill -0 "${pids[2]}" 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then
      echo "$(date -Is): $(free_gib) GiB free, under the floor; the arms were stopped" > "$BASE/transport.LOWDISK"
      kill "${pids[@]}" 2>/dev/null; break
    fi; sleep 120
  done ) &
watch=$!
fail=0
for k in 0 1 2; do
  wait "${pids[$k]}"; code=$?
  echo "transport: seed $k exit $code at $(date -Is)"
  [ "$code" -ne 0 ] && fail=1
done
kill "$watch" 2>/dev/null
[ "$fail" -ne 0 ] && { echo "a transport arm exited nonzero; see transport_seed*/run.err" > "$BASE/transport.KILLED"; exit 1; }
echo "0" > "$BASE/transport.DONE"
