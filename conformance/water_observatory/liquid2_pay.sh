#!/bin/bash
#
# LIQUID-2, THE COMPUTE PAID (2026-09-09, the fourth review's WP1): the declared 3 x 60-block
# pilot set at 1x, then the settling FROZEN from it, then the gate at the structure arm under
# the physical settling cap. One chain, because the gate's design reads the pilots and a gate
# run before them would be re-run after them at the settling's full price.
#
# The tail checks the exit code FIRST (detached_run.sh's rule): a pilot that dies leaves a
# KILLED marker and nothing downstream runs on its corpse.
#
# Launch:  setsid nohup conformance/water_observatory/liquid2_pay.sh > liquid2/pay.log 2>&1 &
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/liquid2"
OUT="$ROOT/conformance/water_observatory/liquid2"
cd "$ROOT/engine" || exit 2
echo "pay: $(date -Is) pid $$ bin $BIN out $OUT"
rm -f "$OUT/pay.KILLED" "$OUT/pay.DONE"

# the three pilots on their own cores, in parallel
pids=()
for i in 0 1 2; do
  core=$((24 + i))
  taskset -c "$core" "$BIN" pilot "$OUT" --pilot "$i" --dir pilot_1x > "$OUT/pilot_1x/pilot$i.log" 2>&1 &
  pids+=($!)
  echo "pay: pilot $i on core $core pid ${pids[$i]}"
done
fail=0
for i in 0 1 2; do
  wait "${pids[$i]}"; code=$?
  echo "pay: pilot $i exit $code at $(date -Is)"
  [ "$code" -ne 0 ] && fail=1
done
if [ "$fail" -ne 0 ]; then
  echo "pilot set: a pilot exited nonzero; see pilot_1x/pilot*.log" > "$OUT/pay.KILLED"
  exit 1
fi

# the settling frozen from the set, at this step
"$BIN" pilot "$OUT" --freeze --dir pilot_1x > "$OUT/pilot_1x/freeze.log" 2>&1; code=$?
echo "pay: freeze exit $code at $(date -Is)"
if [ "$code" -ne 0 ]; then
  echo "freeze: exit $code; see pilot_1x/freeze.log" > "$OUT/pay.KILLED"
  exit 1
fi

# the gate at the structure arm, under the physical cap
taskset -c 27 "$BIN" gate "$OUT" --arm structure > "$OUT/gate.log" 2>&1; code=$?
echo "pay: gate exit $code at $(date -Is)"
if [ "$code" -ne 0 ]; then
  echo "gate: exit $code; see gate.log" > "$OUT/pay.KILLED"
  exit 1
fi
echo "0 $(date -Is)" > "$OUT/pay.DONE"
echo "pay: DONE"
