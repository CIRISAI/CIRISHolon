#!/usr/bin/env bash
# THE CITABLE PERFORMANCE TABLE, taken when the box is actually quiet.
#
# Ruling standing over this lane: nothing measured at load 33-54 with 2-4x
# repetition spread is bankable against CI-runner baselines. The verification
# claims do not care (correctness is load-independent); the RATIOS do. So the
# performance table that goes into BENCHMARKS as citable is the one taken in a
# quiet window, and this waits for one.
#
# HOW IT DECIDES THE MACHINE IS QUIET, and why not loadavg alone: loadavg is a
# proxy for what we actually care about, which is "does our process get a core
# when it asks". So the gate is a CALIBRATION RUN of known cost, and the sweep
# proceeds only when that job comes in near its own record. A loadavg threshold
# is kept as a cheap pre-filter so we are not calibrating every minute.
#
# THE CALIBRATION JOB IS d=101, AND THE FIRST CHOICE WAS WRONG. d=45 was tried
# first and REJECTED by measurement: at loadavg 46 it read 0.055 s against a
# 0.052 s record — 1.06x, comfortably inside any sane gate — because a 55 ms
# job gets scheduled onto an idle core even on a saturated box. It could not
# distinguish quiet from loaded, which is the only thing it was there to do,
# and it would have certified a load-46 machine as quiet and produced exactly
# the mislabelled citable table this script exists to prevent. d=101 runs long
# enough to compete: measured at loadavg 45 it reads 1.79-2.02 s against a
# 1.319 s record, 1.36-1.53x, so the gate below actually separates.
#
# AND IT CHECKS THE WINDOW HELD. The sweep records loadavg at both ends; a run
# that starts quiet and finishes loaded is not a quiet-machine measurement, and
# this refuses to label it one. The result is written either way, with a
# verdict line saying which it is — a mislabelled quiet number is worse than an
# honestly-labelled loaded one.
#
# Usage: setsid nohup ./run_when_quiet.sh > quiet_waiter.log 2>&1 &

set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/surface_flagship"
STIMPY="/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/stimvenv/bin/python"
H2H="$ROOT/conformance/qasm/surface_h2h.py"

# Best observed d=101 engine time on this box (min over every sweep taken).
# The calibration passes when we get within CAL_FACTOR of it.
CAL_D=${CAL_D:-101}
CAL_BEST=${CAL_BEST:-1.319}
CAL_FACTOR=${CAL_FACTOR:-1.15}
# Cheap pre-filter so calibration is not run every minute.
LOAD_MAX=${LOAD_MAX:-8}
# Load must not exceed this at the END, or the window did not hold.
LOAD_MAX_END=${LOAD_MAX_END:-12}
POLL=${POLL:-300}
MAX_WAIT=${MAX_WAIT:-172800}
DS=${DS:-21,45,101,141,221}
REPS=${REPS:-5}

load1() { awk '{print $1}' /proc/loadavg; }

# Is the machine giving us a core? Run the known-cost job and compare.
calibrate() {
  local t
  t=$("$BIN" --d "$CAL_D" --mode bench --rounds 3 --json /dev/null 2>/dev/null \
      | python3 -c "import json,sys; print(json.load(sys.stdin)['results'][0]['metadata']['timing_seconds']['wall'])" 2>/dev/null)
  [ -z "$t" ] && { echo "999"; return; }
  echo "$t"
}

echo "[$(date -Is)] quiet-window waiter starting (load $(load1), gate: d=$CAL_D within ${CAL_FACTOR}x of ${CAL_BEST}s)"

waited=0
while [ "$waited" -lt "$MAX_WAIT" ]; do
  l=$(load1)
  if awk "BEGIN{exit !($l < $LOAD_MAX)}"; then
    cal=$(calibrate)
    if awk "BEGIN{exit !($cal < $CAL_BEST * $CAL_FACTOR)}"; then
      echo "[$(date -Is)] QUIET: load $l, calibration ${cal}s vs best ${CAL_BEST}s — running the sweep"
      {
        echo "=== quiet-window sweep $(date -Is), load $l, calibration ${cal}s ==="
        nice -n 5 "$STIMPY" "$H2H" --d "$DS" --rounds 3 --reps "$REPS" \
             --tmpdir /tmp --out "$HERE/h2h_quiet.json"
        echo "rc=$?"
      } >> "$HERE/quiet.log" 2>&1

      end_load=$(python3 -c "
import json
try:
    j=json.load(open('$HERE/h2h_quiet.json'))
    print(j.get('loadavg_end',[99])[0])
except Exception:
    print(99)
")
      if awk "BEGIN{exit !($end_load < $LOAD_MAX_END)}"; then
        echo "[$(date -Is)] WINDOW HELD (load ended $end_load) — this table is citable" \
          | tee -a "$HERE/quiet.log"
        date -Is > "$HERE/quiet.DONE"
      else
        echo "[$(date -Is)] WINDOW BROKE (load ended $end_load > $LOAD_MAX_END) — result kept but NOT citable as quiet; will retry" \
          | tee -a "$HERE/quiet.log"
        mv -f "$HERE/h2h_quiet.json" "$HERE/h2h_window_broke_$(date +%s).json" 2>/dev/null
        sleep "$POLL"; waited=$((waited + POLL)); continue
      fi
      exit 0
    else
      echo "[$(date -Is)] load $l looked quiet but calibration read ${cal}s (>${CAL_FACTOR}x best) — not a real window"
    fi
  fi
  sleep "$POLL"
  waited=$((waited + POLL))
done

echo "[$(date -Is)] GAVE UP after ${MAX_WAIT}s: no quiet window" | tee -a "$HERE/quiet.log"
echo "no quiet window within ${MAX_WAIT}s" > "$HERE/quiet.GAVEUP"
