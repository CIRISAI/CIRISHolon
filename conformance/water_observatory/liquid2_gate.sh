#!/bin/bash
#
# THE LIQUID-2 GATE at the structure arm, re-run under AMENDMENT 1 (2026-09-11): the
# settling criterion's temperature band derived from the thermostat in force. The pilots are
# done and their settling frozen (pilot_1x/settling.json, 86,000 frames); this runs the gate
# alone.
#
# PREFLIGHT, paid for on 2026-09-11: the previous gate died at exit 101 after 22 hours when
# the host's root filesystem filled from unrelated work and an 8-byte write failed. A run
# that cannot write its own record should not start, and one that runs out mid-way should say
# so in a file rather than in a truncated log. So: a free-space floor before the run, and a
# watcher that stops the run and writes gate.LOWDISK if the floor is crossed while it runs.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/liquid2"
OUT="$ROOT/conformance/water_observatory/liquid2"
FLOOR_GIB=8
cd "$ROOT/engine" || exit 2
rm -f "$OUT/gate.KILLED" "$OUT/gate.EXIT" "$OUT/gate.LOWDISK"

free_gib() { df -BG --output=avail "$OUT" | tail -1 | tr -dc '0-9'; }
have=$(free_gib)
echo "gate: $(date -Is) pid $$; $have GiB free against a floor of $FLOOR_GIB"
if [ "$have" -lt "$FLOOR_GIB" ]; then
  echo "REFUSED: $have GiB free, under the $FLOOR_GIB GiB floor. Nothing run." > "$OUT/gate.LOWDISK"
  exit 3
fi

taskset -c 27 "$BIN" gate "$OUT" --arm structure > "$OUT/gate.log" 2> "$OUT/gate.err" &
run=$!
( while kill -0 "$run" 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then
      echo "$(date -Is): $(free_gib) GiB free, under the $FLOOR_GIB GiB floor; the gate was stopped so it would not die on a failed write" > "$OUT/gate.LOWDISK"
      kill "$run" 2>/dev/null
      break
    fi
    sleep 120
  done ) &
watch=$!
wait "$run"; code=$?
kill "$watch" 2>/dev/null
echo "$code" > "$OUT/gate.EXIT"
echo "gate: exit $code at $(date -Is)"
[ "$code" -ne 0 ] && echo "gate: exit $code; see gate.err and gate.LOWDISK" > "$OUT/gate.KILLED"
exit $code
