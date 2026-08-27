#!/usr/bin/env bash
# Q10 §9 pre-freeze probes 1 (fence must VARY) and 2 (anchors must be VIOLABLE).
# Captures REFUSED lines too -- a refusal is a reading, not a gap.
set -uo pipefail
cd /home/emoore/CIRISOntology/sim_engine
O=output/q10
: > "$O/probe12.log"
timeout --signal=TERM --kill-after=60s 10800 bash -c '
for n in 8 10 12; do
  for u in 0 1 4 16; do
    for chi in 4 8 16 32 64 128 256; do
      timeout 900 cargo run -q --release -p q8-mps --example q10_probe -- $n $u $chi 2>&1 \
        | grep -E "^N=" || echo "N=$n U=$u chi=$chi TIMEOUT_OR_ERROR"
    done
  done
done' >> "$O/probe12.log" 2>&1
echo "$?" > "$O/probe12.DONE"
