#!/bin/bash
# RESPONSE-1, third layout (2026-09-20 10:10 CDT): the five arms killed at 09:02 by a transient
# disk-full event (not this tree's) relaunched fresh, single-worker one per P-core, plus a
# DISK WATCHDOG that SIGSTOPs every arm (these and the three survivors) while free space is
# under the floor and SIGCONTs them when it recovers - an append into a full disk is a panic
# the binary cannot retry, and a paused arm loses nothing.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
STIFF="$BASE/transport_seed0/stiffness.json"
FLOOR_GIB=12
cd "$ROOT/engine" || exit 2
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
echo "response1-c: $(date -Is) pid $$; $(free_gib) GiB free; binary $(stat -c %y "$BIN" | cut -c1-19)"
arm() { local core=$1 kind=$2 seed=$3 mps=$4 tag=$5
  local out="$BASE/response1_${tag}_seed$seed"; mkdir -p "$out"
  echo "response1-c: $(date -Is) arm $tag seed $seed on core $core"
  taskset -c "$core" "$BIN" scout "$out" --cells 6 --settle 3000 --settle-rigid 3000 --readout-fs 10 --stiffness "$STIFF" --seed "$seed" --workers 1 \
      --kick "$kind" --kick-mps "$mps" --kick-cycles 12 --kick-relax 314 > "$out/scout.log" 2> "$out/scout.err"
  local c=$?; echo "$c" > "$out/run.done"; echo "response1-c: $(date -Is) arm $tag seed $seed exit $c"; }
arm 0 L 0 50 L & arm 2 T 1 50 T & arm 4 L 2 50 L & arm 6 T 2 50 T & arm 1 T 0 200 T200 &
sleep 5
( paused=0
  while pgrep -f "replace0 scout" > /dev/null; do
    pids=$(pgrep -f "replace0 scout")
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ] && [ "$paused" = 0 ]; then kill -STOP $pids; paused=1; echo "response1-c: $(date -Is) PAUSED all arms: $(free_gib) GiB free"
    elif [ "$(free_gib)" -ge "$FLOOR_GIB" ] && [ "$paused" = 1 ]; then kill -CONT $pids; paused=0; echo "response1-c: $(date -Is) resumed: $(free_gib) GiB free"; fi
    sleep 15
  done ) &
wait
