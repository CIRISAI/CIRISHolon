#!/bin/bash
# RESPONSE-1, second layout (2026-09-19 14:15 CDT). The first launcher ran eight-worker groups;
# the arms measured at ~101 % of one CPU each — the rigid operator's serial sectors dominate and
# the pool buys nothing at this size — so each eight-worker group was one core's work with
# seven idle. The three arms already running (L seed 0, T seed 0, L seed 1) are kept, their
# queue subshells stopped, and a watcher writes their run.done from the walk's row count.
# The other five arms run here SINGLE-WORKER, one per P-core thread on cores 0, 2, 4, 6 (one
# per physical core) and 1 (the sibling of 0, for the lowest-priority control). Same binary,
# same arguments but --workers 1; the executor contract makes the answer bit-identical.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
STIFF="$BASE/transport_seed0/stiffness.json"
ROWS=$((12 * 314 + 1))
cd "$ROOT/engine" || exit 2
echo "response1-b: $(date -Is) pid $$; binary $(stat -c %y "$BIN" | cut -c1-19)"
watch_orphan() {  # watch_orphan PID DIR — the first layout's arm, reparented; run.done from the walk
  local pid=$1 dir=$2
  while kill -0 "$pid" 2>/dev/null; do sleep 120; done
  local rows; rows=$(($(wc -l < "$dir/rigid.walk") - 1))
  [ "$rows" -eq "$ROWS" ] && echo 0 > "$dir/run.done" || echo "1 (walk has $rows rows, not $ROWS)" > "$dir/run.done"
  echo "response1-b: $(date -Is) first-layout arm $(basename "$dir") exited; walk rows $rows"
}
arm() {  # arm CORE KIND SEED MPS TAG
  local core=$1 kind=$2 seed=$3 mps=$4 tag=$5
  local out="$BASE/response1_${tag}_seed$seed"; mkdir -p "$out"
  echo "response1-b: $(date -Is) arm $tag seed $seed on core $core, one worker"
  taskset -c "$core" "$BIN" scout "$out" --cells 6 --settle 3000 --settle-rigid 3000 --readout-fs 10 --stiffness "$STIFF" --seed "$seed" --workers 1 \
      --kick "$kind" --kick-mps "$mps" --kick-cycles 12 --kick-relax 314 > "$out/scout.log" 2> "$out/scout.err"
  local c=$?; echo "$c" > "$out/run.done"; echo "response1-b: $(date -Is) arm $tag seed $seed exit $c"
}
watch_orphan 2442824 "$BASE/response1_L_seed0" &
watch_orphan 2442823 "$BASE/response1_T_seed0" &
watch_orphan 2731673 "$BASE/response1_L_seed1" &
arm 0 T 1 50 T &
arm 2 L 2 50 L &
arm 4 T 2 50 T &
arm 6 L 0 200 L200 &
arm 1 T 0 200 T200 &
wait
echo 0 > "$BASE/response1.DONE"
