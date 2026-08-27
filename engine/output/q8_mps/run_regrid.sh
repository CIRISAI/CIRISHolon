#!/usr/bin/env bash
# Detached re-adjudication of the eight-configuration grid on the REPAIRED SVD.
# Same harness that produced the recorded table -- not a different binary.
set -uo pipefail
cd /home/emoore/CIRISOntology/sim_engine
O=output/q8_mps
timeout --signal=TERM --kill-after=60s 14400 \
  cargo test --release -p q8-mps --test full_grid_gates -- --ignored --nocapture \
  > "$O/regrid.log" 2>&1
rc=$?
echo "$rc" > "$O/regrid.DONE"
if [ "$rc" -eq 124 ] || [ "$rc" -eq 137 ]; then
  echo "TIMED OUT after 4h (rc=$rc) - hang, not a slow machine" >> "$O/regrid.log"
fi
