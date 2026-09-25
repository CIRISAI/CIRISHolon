#!/bin/bash
# SLOW-3 binary provenance: the rebuilt replace0 (slow3.sh's header) re-runs SLOW-1's T293 seed 0 for 20
# readouts on core 18, with slow1.sh's flags, and its rigid.walk / rigid.vwalk rows are compared with SLOW-1's
# own, byte for byte. IDENTICAL means the fresh arms run the operator SLOW-1 ran.
set -u
ROOT=/home/emoore/CIRISHolon; BASE="$ROOT/conformance/water_observatory/replace0"; BIN="$BASE/slow3/replace0_slow3"
STIFF="$BASE/transport_seed0/stiffness.json"; d="$BASE/slow3/provenance_T293_seed0"; ref="$BASE/slow1/T293_seed0"; mkdir -p "$d"
cd "$ROOT/engine" || exit 2
taskset -c 18 "$BIN" scout "$d" --cells 5 --settle 1000 --settle-rigid 1000 --readouts 20 --readout-fs 20 --stiffness "$STIFF" --seed 0 --workers 1 --temperature-k 293 > "$d/scout.log" 2> "$d/scout.err"; echo $? > "$d/run.done"
for f in rigid.walk rigid.vwalk; do
  n=$(wc -l < "$d/$f")
  if cmp -s <(head -n "$n" "$d/$f") <(head -n "$n" "$ref/$f"); then echo "$f: first $n lines IDENTICAL to SLOW-1 seed 0"; else echo "$f: first $n lines DIFFER from SLOW-1 seed 0"; fi
done > "$d/provenance.txt"
