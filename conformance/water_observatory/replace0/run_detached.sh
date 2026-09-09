#!/bin/bash
# REPLACE-0's first counted run (2026-09-09): 20,000 settling frames under the stochastic
# thermostat, then both arms NVE for 20,000 frames from one branch point, the rigid arm twice
# (plain, and with the refinement demonstration). One core. The exit code is written first.
set -u
cd /home/emoore/CIRISHolon/engine || exit 2
OUT=/home/emoore/CIRISHolon/conformance/water_observatory/replace0
rm -f "$OUT/run.KILLED" "$OUT/run.EXIT"
echo "replace0: $(date -Is) pid $$"
taskset -c 31 ./target/release/examples/replace0 run "$OUT" --settle 20000 --frames 20000 --readouts 40 --refine > "$OUT/run.log" 2> "$OUT/run.err"; code=$?
echo "replace0: exit $code at $(date -Is)"
echo "$code" > "$OUT/run.EXIT"
[ "$code" -ne 0 ] && echo "replace0 run: exit $code; see run.err" > "$OUT/run.KILLED"
exit $code
