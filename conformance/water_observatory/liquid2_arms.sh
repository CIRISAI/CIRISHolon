#!/bin/bash
#
# THE LIQUID-2 COUNTED ARMS, structure (2026-09-11): three confirmation seeds behind an
# admitted gate (gate.done, Sprice 0.712 of the ceiling). Each arm re-derives its design from
# the CAMPAIGN root and is refused by the BIND gate unless its floor, cap, counted length,
# stride and arm kind are the gate's exactly.
#
# Same disk preflight as the gate: a free-space floor before the run, and a watcher that
# stops every arm and writes arms.LOWDISK if the floor is crossed while they run.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/liquid2"
OUT="$ROOT/conformance/water_observatory/liquid2"
FLOOR_GIB=8
cd "$ROOT/engine" || exit 2
rm -f "$OUT/arms.KILLED" "$OUT/arms.DONE" "$OUT/arms.LOWDISK"

free_gib() { df -BG --output=avail "$OUT" | tail -1 | tr -dc '0-9'; }
have=$(free_gib)
echo "arms: $(date -Is) pid $$; $have GiB free against a floor of $FLOOR_GIB"
if [ "$have" -lt "$FLOOR_GIB" ]; then
  echo "REFUSED: $have GiB free, under the $FLOOR_GIB GiB floor. Nothing run." > "$OUT/arms.LOWDISK"
  exit 3
fi
if [ ! -f "$OUT/gate.done" ]; then
  echo "REFUSED: gate.done is absent; the counted arm runs only behind the gate phase." > "$OUT/arms.KILLED"
  exit 3
fi

pids=()
for k in 0 1 2; do
  core=$((24 + k))
  taskset -c "$core" "$BIN" run "$OUT" --seed "$k" --arm structure > "$OUT/seed$k.log" 2> "$OUT/seed$k.err" &
  pids+=($!)
  echo "arms: seed $k on core $core pid ${pids[$k]}"
done
( while kill -0 "${pids[0]}" 2>/dev/null || kill -0 "${pids[1]}" 2>/dev/null || kill -0 "${pids[2]}" 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then
      echo "$(date -Is): $(free_gib) GiB free, under the $FLOOR_GIB GiB floor; the arms were stopped so they would not die on a failed write" > "$OUT/arms.LOWDISK"
      kill "${pids[@]}" 2>/dev/null
      break
    fi
    sleep 120
  done ) &
watch=$!
fail=0
for k in 0 1 2; do
  wait "${pids[$k]}"; code=$?
  echo "arms: seed $k exit $code at $(date -Is)"
  [ "$code" -ne 0 ] && fail=1
done
kill "$watch" 2>/dev/null
if [ "$fail" -ne 0 ]; then
  echo "a counted arm exited nonzero; see seed*.err" > "$OUT/arms.KILLED"
  exit 1
fi
"$BIN" read "$OUT" > "$OUT/read.log" 2> "$OUT/read.err"; code=$?
echo "arms: read exit $code at $(date -Is)"
echo "$code" > "$OUT/arms.DONE"
exit $code
