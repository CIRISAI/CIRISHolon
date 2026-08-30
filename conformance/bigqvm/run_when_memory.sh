#!/usr/bin/env bash
# THE FLAGSHIP AT THE LARGE DISTANCES, run when the box can carry it.
#
# d=221 needs a 9.54 GB working set (column engine plus row-major reference).
# This box is shared and its free memory swings by more than that within the
# hour, so rather than either OOM-killing a sibling or abandoning the run,
# this waits for a window and takes it.
#
# THE AUTHORITY ON WHETHER THERE IS ROOM IS THE BINARY, NOT THIS SCRIPT. A
# first version pre-checked memory in shell, lost a race (free memory fell
# between the check and the run), and would have written a DONE marker for a
# run the binary had correctly REFUSED. So: shell polls only to avoid
# hammering, the binary decides, exit code 2 means "no room, try later", and
# the marker is written ONLY on a real success. A done-marker that can lie is
# worse than no done-marker.
#
# Usage: setsid nohup ./run_when_memory.sh > waiter.log 2>&1 &

set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/surface_flagship"
STIMPY="/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/stimvenv/bin/python"
H2H="$ROOT/conformance/qasm/surface_h2h.py"

MAX_WAIT=${MAX_WAIT:-86400}
POLL=${POLL:-180}
TARGETS=${TARGETS:-221}

avail_gb() { awk '/MemAvailable/ {printf "%.2f", $2/1048576}' /proc/meminfo; }

run_target() {
  local d=$1
  local done="$HERE/d${d}.DONE"
  local log="$HERE/d${d}.log"
  [ -f "$done" ] && { echo "[$(date -Is)] d=$d already done"; return 0; }

  local waited=0 rc=99
  while [ "$waited" -lt "$MAX_WAIT" ]; do
    echo "[$(date -Is)] attempting d=$d (MemAvailable $(avail_gb) GB)"
    {
      echo "=== attempt $(date -Is): d=$d QEC demo (full verification) ==="
      nice -n 15 "$BIN" --d "$d" --seed 1 --json "$HERE/flagship_d${d}.json"
    } >> "$log" 2>&1
    rc=$?
    if [ "$rc" -eq 0 ]; then
      echo "[$(date -Is)] d=$d QEC demo SUCCEEDED; running the head-to-head"
      {
        echo "=== d=$d head-to-head vs stim (min of 3, identical circuit) ==="
        nice -n 15 "$STIMPY" "$H2H" --d "$d" --rounds 3 --reps 3 \
             --tmpdir /tmp --out "$HERE/h2h_d${d}.json"
        echo "h2h rc=$?"
      } >> "$log" 2>&1
      date -Is > "$done"
      echo "[$(date -Is)] d=$d COMPLETE -> $done"
      return 0
    elif [ "$rc" -eq 2 ]; then
      # The binary's own guard refused: no room right now. Wait and retry.
      echo "[$(date -Is)] d=$d refused for memory; retrying in ${POLL}s"
      sleep "$POLL"
      waited=$((waited + POLL))
    else
      # A real failure (verification, crash) — do NOT mark done, do NOT loop.
      echo "[$(date -Is)] d=$d FAILED with rc=$rc — see $log" | tee -a "$log"
      echo "rc=$rc" > "$HERE/d${d}.FAILED"
      return 1
    fi
  done
  echo "[$(date -Is)] d=$d GAVE UP after ${MAX_WAIT}s: never saw a window" \
       | tee -a "$log"
  echo "no memory window within ${MAX_WAIT}s" > "$HERE/d${d}.GAVEUP"
  return 1
}

echo "[$(date -Is)] flagship memory-waiter starting; MemAvailable $(avail_gb) GB"
for d in $TARGETS; do
  run_target "$d"
done
echo "[$(date -Is)] waiter finished"
