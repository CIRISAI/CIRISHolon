#!/bin/bash
#
# REPLACE-0's arms re-run with VELOCITIES BANKED (2026-09-16), for RUNG2_AMENDMENT_1's
# momentum and energy rungs. Same three branch points as the transport arms (branch.ckpt
# reused, no re-settling), same 3 ps, but 300 readouts at 10 fs instead of 150 at 20 fs so
# the chart has >= 200 transitions to grade on (G4) at the same physical length and cost.
# The flexible arm re-runs (its banked series is at 150 readouts and is refused by count);
# the rigid arm re-runs (an hour). Writes flexible.vwalk and rigid.vwalk beside the walks.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
FLOOR_GIB=8
cd "$ROOT/engine" || exit 2
rm -f "$BASE/momentum.KILLED" "$BASE/momentum.DONE" "$BASE/momentum.LOWDISK"
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
have=$(free_gib); echo "momentum: $(date -Is) pid $$; $have GiB free against a floor of $FLOOR_GIB"
[ "$have" -lt "$FLOOR_GIB" ] && { echo "REFUSED: $have GiB free" > "$BASE/momentum.LOWDISK"; exit 3; }
pids=()
for k in 0 1 2; do
  OUT="$BASE/transport_seed$k"
  [ -f "$OUT/branch.ckpt" ] || { echo "seed $k: no branch.ckpt to reuse" > "$BASE/momentum.KILLED"; exit 3; }
  taskset -c $((24 + k)) "$BIN" run "$OUT" --seed "$k" --settle 102100 --frames 115105 --readouts 300 --reuse \
      > "$OUT/momentum.log" 2> "$OUT/momentum.err" &
  pids+=($!); echo "momentum: seed $k on core $((24 + k)) pid ${pids[$k]}"
done
( while kill -0 "${pids[0]}" 2>/dev/null || kill -0 "${pids[1]}" 2>/dev/null || kill -0 "${pids[2]}" 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then
      echo "$(date -Is): under the floor; arms stopped" > "$BASE/momentum.LOWDISK"; kill "${pids[@]}" 2>/dev/null; break
    fi; sleep 120
  done ) &
watch=$!; fail=0
for k in 0 1 2; do wait "${pids[$k]}"; c=$?; echo "momentum: seed $k exit $c at $(date -Is)"; [ "$c" -ne 0 ] && fail=1; done
kill "$watch" 2>/dev/null
[ "$fail" -ne 0 ] && { echo "an arm exited nonzero; see transport_seed*/momentum.err" > "$BASE/momentum.KILLED"; exit 1; }
echo 0 > "$BASE/momentum.DONE"
