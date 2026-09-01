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
# stim lives in a venv. This was originally HARDCODED to a per-session
# scratchpad keyed by session id — it resolved from inside the session that
# wrote it and nowhere else, which is a reproducibility defect invisible from
# inside (credit: saturation3-mesh, who found the same shape in their own
# instrument citations). Now: explicit override, then discovery, then a LOUD
# refusal with the command to build one — never a silent wrong path.
STIMPY="${STIMPY:-}"
if [ -z "$STIMPY" ]; then
  for c in "$ROOT/.venv/bin/python" "$HOME/.venvs/stim/bin/python" \
           /tmp/claude-*/*/*/scratchpad/stimvenv/bin/python; do
    if [ -x "$c" ] && "$c" -c 'import stim' >/dev/null 2>&1; then
      STIMPY="$c"; break
    fi
  done
fi
if [ -z "$STIMPY" ]; then
  echo "REFUSING: no python with stim found." >&2
  echo "  set STIMPY=/path/to/python, or: python3 -m venv .venv && .venv/bin/pip install stim" >&2
  exit 3
fi
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
# PIN BOTH ARMS. This box is an i9-13900HX: P-cores 0-15, E-cores 16-31.
# Measured 2026-08-30, both arms on the SAME core: the d=101 verdict FLIPS
# between placements (0.822 unpinned, 1.201 on a P-core, 0.989 on an E-core).
# A quiet window fixes contention and does nothing about heterogeneity, so
# quiet was necessary and never sufficient.
#
# CORRECTED 2026-09-01: the E-core run is the PRIMARY, not the P-core one, and
# the reason is counterintuitive enough to state. A P-core's throughput depends
# on its SMT SIBLING's load, which nobody controls: within-P spread measured
# 1.41x against E's 1.03x, and the FASTEST P-core in one sample was the SLOWEST
# minutes later. E-cores on this part have no sibling and repeat across cores
# and sessions. A citable table wants REPEATABILITY more than the best clock,
# so the reproducible condition is the slower one. Both are still run and both
# reported as a RANGE -- which placement is "adversarial" is itself unstable,
# so no single number is quoted as the conservative one.
PIN_E=${PIN_E:-20}
PIN_P=${PIN_P:-0}

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
        # Both core types, because the ratio is placement-dependent and
        # reporting one of them alone is how the d=101 row got banked wrong.
        nice -n 5 "$STIMPY" "$H2H" --d "$DS" --rounds 3 --reps "$REPS" \
             --pin "$PIN_E" --tmpdir /tmp --out "$HERE/h2h_quiet_ecore.json"
        echo "--- E-core (PRIMARY, reproducible: no SMT sibling) rc=$? ---"
        nice -n 5 "$STIMPY" "$H2H" --d "$DS" --rounds 3 --reps "$REPS" \
             --pin "$PIN_P" --tmpdir /tmp --out "$HERE/h2h_quiet_pcore.json"
        echo "--- P-core (secondary, sibling-dependent) rc=$? ---"
        cp -f "$HERE/h2h_quiet_ecore.json" "$HERE/h2h_quiet.json"
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
