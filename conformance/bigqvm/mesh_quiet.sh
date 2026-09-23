#!/usr/bin/env bash
# MESH-CLIFFORD-1 G3/G4 — the citable table, taken when the box is actually quiet.
#
# THE LEAD LAUNCHES THIS, NOT THE HARNESS AGENT. It takes P-cores 8-15 and
# E-cores 16-23 for tens of minutes and needs a 14 GB memory window for d=221.
# Other campaigns hold those cores today. It is written, checked and STOOD DOWN:
#
#     cd conformance/bigqvm && setsid nohup ./mesh_quiet.sh \
#         > mesh_quiet_waiter.log 2>&1 &
#
#   -> mesh_h2h_pcore.json    the STAKED table (prereg G3 stakes P-cores)
#   -> mesh_h2h_ecore.json    the reproducibility table (no SMT sibling)
#   -> mesh_quiet.log         both sweeps' full output
#   -> mesh_quiet.DONE        written ONLY when both sweeps passed their own load gate
#   -> mesh_quiet.GAVEUP      no window inside MAX_WAIT
#
# Overrides, all environment: LOAD_MAX SWEEP_LOAD_MAX MEM_MIN_GB CAL_D
# CAL_FACTOR CAL_BEST_P CAL_BEST_E CONC_FACTOR POLL MAX_WAIT DS SHARDS REPS
# ROUNDS SEED PCORES ECORES OUT_P OUT_E LOG STIMPY.
#
# A d=45,141 rehearsal that needs no memory window, finishes in minutes and
# overwrites no citable table:
#
#     DS=45,141 MEM_MIN_GB=4 OUT_P=/tmp/rp.json OUT_E=/tmp/re.json \
#         LOG=/tmp/rehearse.log ./mesh_quiet.sh
#
# Both branches of the window check were exercised on cores 21-26 before this
# was committed: the sweeps running and the DONE marker written (LOAD_MAX=999),
# and the window breaking mid-sweep (LOAD_MAX=999 SWEEP_LOAD_MAX=8, rc=3 from
# both sweeps, no marker, retry).
#
# ---------------------------------------------------------------------------
# HOW IT DECIDES THE MACHINE IS QUIET, and why not loadavg alone. This lane
# already settled that: loadavg is a proxy for the thing we care about, which
# is "does our process get a core when it asks", so the gate is a CALIBRATION
# RUN of known cost and loadavg is only a cheap pre-filter. d=45 was tried as
# the calibration job and REJECTED by measurement — a 55 ms job gets an idle
# core even on a saturated box and read 1.06x of its record at loadavg 46. The
# calibration is d=101, which runs long enough to compete.
#
# WHAT IS NEW HERE, and why. `run_when_quiet.sh` pinned ONE core, because the
# engine was one thread. This sweep runs up to EIGHT shards, so "one core is
# free" is no longer the question — "are eight cores free" is. So there are
# two calibrations:
#
#   (1) SINGLE-CORE, against this box's own banked quiet records: 0.763 s on a
#       P-core and 1.371 s on an E-core (the min columns of h2h_quiet_pcore.json
#       and h2h_quiet_ecore.json, taken in the confirmed quiet window of
#       2026-08-31). Those are MEASURED records from the class being gated,
#       which the inherited 1.319 s was not — it was taken on a loaded box.
#
#   (2) CONCURRENT, one copy per core of the set, gated on the SLOWEST. This
#       is the one that catches "our core is free and the other seven are not".
#       Its factor (CONC_FACTOR, default 1.6x the single-core calibration) is a
#       JUDGEMENT, not a measurement: eight concurrent tableaux share L3 and
#       memory bandwidth and would be slower than one even on an idle box, and
#       nobody has yet measured by how much on this part. The FIRST quiet run
#       should record what it actually reads and this default should then be
#       replaced by that measurement. Set CONC_FACTOR=0 to skip the check
#       entirely (and say so when citing the table).
#
# AND IT CHECKS THE WINDOW HELD, but it does not re-implement the check.
# mesh_h2h.py refuses to write its --out if the 1-minute loadavg exceeded
# --max-load at EITHER end, keeps the data beside it under a .not-citable-*
# name, and exits 3. So rc=3 here means "the window broke" and the waiter goes
# back to polling; rc=0 means the file on disk passed its own gate. One gate,
# in one place, enforced by the thing that took the measurement.
# ---------------------------------------------------------------------------

set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/surface_flagship"
H2H="$HERE/mesh_h2h.py"
LOG="${LOG:-$HERE/mesh_quiet.log}"

# stim lives in a venv on this box. mesh_h2h.py discovers one and re-execs
# itself if the interpreter it is given has no stim, so this only has to hand
# it a python — but an EXPLICIT STIMPY is logged and reproducible, and a
# hardcoded per-session scratchpad path is the reproducibility defect this
# lane has already been bitten by, so: override, then discovery, then a loud
# refusal with the command to build one.
STIMPY="${STIMPY:-}"
if [ -z "$STIMPY" ]; then
  for c in "$ROOT/.venv/bin/python" "$HOME/.venvs/stim/bin/python" \
           "$HOME/CIRISOntology/scratchpad/temporal-share/qenv/bin/python" \
           /tmp/claude-*/*/*/scratchpad/stimvenv/bin/python; do
    if [ -x "$c" ] && "$c" -c 'import stim' >/dev/null 2>&1; then STIMPY="$c"; break; fi
  done
fi
if [ -z "$STIMPY" ]; then
  echo "REFUSING: no python with stim found." >&2
  echo "  set STIMPY=/path/to/python, or: python3 -m venv .venv && .venv/bin/pip install stim" >&2
  exit 3
fi
export STIMPY

[ -x "$BIN" ] || { echo "REFUSING: no binary at $BIN" >&2
                   echo "  cd engine && taskset -c 21-27 cargo build --release -p holon --example surface_flagship" >&2
                   exit 3; }
[ -f "$H2H" ] || { echo "REFUSING: no harness at $H2H" >&2; exit 3; }

# ---- the sweep ----
DS=${DS:-45,141,221}
SHARDS=${SHARDS:-1,2,4,8}
REPS=${REPS:-5}
ROUNDS=${ROUNDS:-3}
SEED=${SEED:-1}
# PCORES 8-15 is FOUR PHYSICAL P-CORES, eight hardware threads (siblings 8/9,
# 10/11, 12/13, 14/15 on this i9-13900HX). ECORES 16-23 is EIGHT physical
# E-cores with no siblings at all. That asymmetry is real and it is not in the
# prereg: at S=8 the P sweep puts two shards on each physical core and the E
# sweep puts one on each. Both are run and both are reported; see
# MESH_CLIFFORD_RESULTS.md §on-building. For a no-SMT P control on eight
# PHYSICAL P-cores, one thread each, use PCORES=0,2,4,6,8,10,12,14 — which
# needs cores 0-7, so it waits for the campaigns that hold them.
PCORES=${PCORES:-8-15}
ECORES=${ECORES:-16-23}
# Overridable so a rehearsal can be run without overwriting the citable tables.
OUT_P=${OUT_P:-$HERE/mesh_h2h_pcore.json}
OUT_E=${OUT_E:-$HERE/mesh_h2h_ecore.json}

# ---- the gates ----
LOAD_MAX=${LOAD_MAX:-8}
# What the SWEEP is gated on, which may be stricter than the pre-filter. It
# defaults to the same number, so by default there is one gate and not two.
# It exists because the "the window broke mid-sweep" branch below is
# otherwise unreachable on demand, and this lane's rule is that a fallback
# nothing ever runs is an untested claim: LOAD_MAX=999 SWEEP_LOAD_MAX=8
# exercises it in seconds.
SWEEP_LOAD_MAX=${SWEEP_LOAD_MAX:-$LOAD_MAX}
# d=221's working set is 9.54 GB (column engine + row-major reference) and the
# stim arm builds its own ~2.4 GB tableau beside it. 14 GB is the floor; the
# BINARY is still the authority (exit code 2 = no room), exactly as
# run_when_memory.sh established — the shell polls only to avoid hammering.
MEM_MIN_GB=${MEM_MIN_GB:-14}
CAL_D=${CAL_D:-101}
CAL_BEST_P=${CAL_BEST_P:-0.763}   # h2h_quiet_pcore.json, d=101, min, quiet window
CAL_BEST_E=${CAL_BEST_E:-1.371}   # h2h_quiet_ecore.json, d=101, min, quiet window
CAL_FACTOR=${CAL_FACTOR:-1.15}
CONC_FACTOR=${CONC_FACTOR:-1.6}
POLL=${POLL:-300}
MAX_WAIT=${MAX_WAIT:-172800}

load1() { awk '{print $1}' /proc/loadavg; }
avail_gb() { awk '/MemAvailable/ {printf "%.2f", $2/1048576}' /proc/meminfo; }

# Expand "8-15" or "8,10,12" into a list of cpu numbers.
expand_cores() {
  echo "$1" | tr ',' '\n' | while read -r p; do
    case "$p" in
      *-*) seq "${p%%-*}" "${p##*-}" ;;
      "")  ;;
      *)   echo "$p" ;;
    esac
  done
}

# One calibration run of known cost, pinned to ONE cpu — the same shape the
# banked records were taken in, so the comparison is like for like.
calibrate_one() {
  local cpu=$1 t
  t=$(taskset -c "$cpu" "$BIN" --d "$CAL_D" --mode bench --rounds 3 --json /dev/null 2>/dev/null \
      | python3 -c "import json,sys; print(json.load(sys.stdin)['results'][0]['metadata']['timing_seconds']['wall'])" 2>/dev/null)
  [ -z "$t" ] && t=999
  echo "$t"
}

# One copy per cpu of the set, all at once; the answer is the SLOWEST, because
# a sweep that will use eight cores is only as quiet as its worst one.
calibrate_concurrent() {
  local set=$1 cpu tmp worst=0 t
  tmp=$(mktemp -d)
  for cpu in $(expand_cores "$set"); do
    ( calibrate_one "$cpu" > "$tmp/$cpu" ) &
  done
  wait
  for cpu in $(expand_cores "$set"); do
    t=$(cat "$tmp/$cpu" 2>/dev/null); [ -z "$t" ] && t=999
    worst=$(awk -v a="$worst" -v b="$t" 'BEGIN{print (b>a)?b:a}')
  done
  rm -rf "$tmp"
  echo "$worst"
}

# Is this core class giving us its cores? Prints a verdict line either way.
class_is_quiet() {
  local set=$1 best=$2 label=$3 first cal conc lim climit
  first=$(expand_cores "$set" | head -1)
  cal=$(calibrate_one "$first")
  lim=$(awk -v b="$best" -v f="$CAL_FACTOR" 'BEGIN{printf "%.4f", b*f}')
  if ! awk "BEGIN{exit !($cal < $lim)}"; then
    echo "[$(date -Is)] $label: single-core calibration ${cal}s on cpu$first (limit ${lim}s = ${CAL_FACTOR}x ${best}s) — NOT a window"
    return 1
  fi
  if awk "BEGIN{exit !($CONC_FACTOR > 0)}"; then
    conc=$(calibrate_concurrent "$set")
    climit=$(awk -v c="$cal" -v f="$CONC_FACTOR" 'BEGIN{printf "%.4f", c*f}')
    if ! awk "BEGIN{exit !($conc < $climit)}"; then
      echo "[$(date -Is)] $label: single-core ${cal}s PASSED but the slowest of $(expand_cores "$set" | wc -w) concurrent copies read ${conc}s (limit ${climit}s = ${CONC_FACTOR}x) — our core is free and the others are not"
      return 1
    fi
    echo "[$(date -Is)] $label QUIET: single-core ${cal}s (< ${lim}s), slowest concurrent ${conc}s (< ${climit}s)"
    echo "  note: the concurrent limit is a judgement, not a measurement. Observed here: ${conc}s against a single-core ${cal}s, i.e. $(awk -v a="$conc" -v b="$cal" 'BEGIN{printf "%.2f", a/b}')x. Bank it and replace the default."
  else
    echo "[$(date -Is)] $label QUIET: single-core ${cal}s (< ${lim}s); concurrent check SKIPPED (CONC_FACTOR=0) — say so when citing"
  fi
  return 0
}

# Amendment 2: every sweep runs TWO arms - the cut as frozen (banded numbering, branch (c))
# and the cut with the re-aim (--transpose-parallel) - into two JSONs, both banked.
sweep() {
  local set=$1 out=$2 label=$3 rc
  sweep_arm "$set" "$out" "$label frozen" "--layout banded" || return $?
  sweep_arm "$set" "${out%.json}_reaim.json" "$label re-aim" "--layout banded --transpose-parallel"
}
sweep_arm() {
  local set=$1 out=$2 label=$3 eargs=$4 rc
  # NOT inside a { ... } >> LOG block: `$?` after a group is the group's exit
  # status, so the harness's own rc — the thing the whole window gate turns on
  # — would be silently replaced by the last echo's. The redirect goes on the
  # command that matters and nowhere else.
  echo "=== $label sweep $(date -Is): cores $set, d=$DS, S=$SHARDS, reps=$REPS ===" >> "$LOG"
  nice -n 5 "$STIMPY" "$H2H" "$BIN" \
      --d "$DS" --shards "$SHARDS" --reps "$REPS" --rounds "$ROUNDS" --seed "$SEED" \
      --cores "$set" --max-load "$SWEEP_LOAD_MAX" --tmpdir /tmp --out "$out" \
      --engine-args "$eargs" \
      --note "MESH-CLIFFORD-1 G3/G4 quiet-window waiter - $label cores $set ($eargs)" >> "$LOG" 2>&1
  rc=$?
  echo "--- $label rc=$rc ---" >> "$LOG"
  return $rc
}

echo "[$(date -Is)] MESH-CLIFFORD-1 G3/G4 quiet-window waiter starting"
echo "  load $(load1) (gate < $LOAD_MAX), MemAvailable $(avail_gb) GB (gate >= $MEM_MIN_GB)"
echo "  P sweep: cores $PCORES -> $OUT_P   (the prereg's staked placement)"
echo "  E sweep: cores $ECORES -> $OUT_E   (no SMT sibling: the reproducible one)"
echo "  d=$DS  S=$SHARDS  reps=$REPS  binary $BIN"

waited=0
while [ "$waited" -lt "$MAX_WAIT" ]; do
  l=$(load1)
  m=$(avail_gb)
  if awk "BEGIN{exit !($l < $LOAD_MAX)}" && awk "BEGIN{exit !($m >= $MEM_MIN_GB)}"; then
    # The cheap pre-filter passed. Now ask the machine, not /proc.
    if class_is_quiet "$PCORES" "$CAL_BEST_P" "P-class" && \
       class_is_quiet "$ECORES" "$CAL_BEST_E" "E-class"; then
      echo "[$(date -Is)] WINDOW OPEN (load $l, MemAvailable ${m} GB) — running both sweeps"

      # P FIRST: the prereg stakes G3 on P-cores, so the staked table gets the
      # freshest part of the window. E follows as the reproducibility arm.
      sweep "$PCORES" "$OUT_P" "P-core"; rcp=$?
      echo "[$(date -Is)] P-core sweep rc=$rcp $([ "$rcp" -eq 3 ] && echo '(load gate refused: the window broke)')"
      sweep "$ECORES" "$OUT_E" "E-core"; rce=$?
      echo "[$(date -Is)] E-core sweep rc=$rce $([ "$rce" -eq 3 ] && echo '(load gate refused: the window broke)')"

      if [ "$rcp" -eq 0 ] && [ "$rce" -eq 0 ]; then
        echo "[$(date -Is)] BOTH SWEEPS HELD THEIR WINDOW — the tables are citable" | tee -a "$LOG"
        {
          date -Is
          echo "pcore $OUT_P"
          echo "ecore $OUT_E"
        } > "$HERE/mesh_quiet.DONE"
        exit 0
      fi
      echo "[$(date -Is)] a sweep did not hold its window (P rc=$rcp, E rc=$rce); the data it kept is beside the output under .not-citable-*, and this will retry" | tee -a "$LOG"
    fi
  else
    : # quiet about the common case; the pre-filter runs every POLL seconds
  fi
  sleep "$POLL"
  waited=$((waited + POLL))
done

echo "[$(date -Is)] GAVE UP after ${MAX_WAIT}s: no window (load and memory, then calibration)" | tee -a "$LOG"
echo "no quiet window within ${MAX_WAIT}s" > "$HERE/mesh_quiet.GAVEUP"
exit 1
