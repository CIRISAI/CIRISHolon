#!/bin/bash
# SLOW-1 (SLOW1_PREREG.md): four rigid-operator arms, 250 waters, 50 ps at 20 fs, one per E-core.
# ROOT defaults to the checkout holding this script; SLOW1_ROOT points it at the main checkout
# (its committed binary and its data tree) when launched from a worktree.
set -u
ROOT="${SLOW1_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"; BIN="$ROOT/engine/target/release/examples/replace0"; BASE="$ROOT/conformance/water_observatory/replace0"; STIFF="$BASE/transport_seed0/stiffness.json"
[ -x "$BIN" ] || { echo "slow1: no binary at $BIN"; exit 2; }
cd "$ROOT/engine" || exit 2; OUT="$BASE/slow1"; mkdir -p "$OUT"
arms=("293 0 16" "293 1 17" "293 2 18" "400 0 19")
for a in "${arms[@]}"; do
  set -- $a; T=$1; seed=$2; core=$3; d="$OUT/T${T}_seed$seed"; mkdir -p "$d"
  ( taskset -c $core "$BIN" scout "$d" --cells 5 --settle 1000 --settle-rigid 1000 --readouts 2500 --readout-fs 20 --stiffness "$STIFF" --seed $seed --workers 1 --temperature-k $T > "$d/scout.log" 2> "$d/scout.err"; echo $? > "$d/run.done" ) &
  echo "slow1: $(date -Is) arm T${T}_seed$seed on core $core"
done
# the price watcher (core 20): the scout's own core-s/ps at readout 100, and the ETA for 2,500
( taskset -c 20 bash -c '
  OUT="$1"
  for d in "$OUT"/T*_seed*; do
    until grep -q "scout readout    100 " "$d/scout.err" 2>/dev/null || [ -f "$d/run.done" ]; do sleep 60; done
    rate=$(grep "scout readout    100 " "$d/scout.err" | head -1 | sed -E "s/.* ([0-9.]+) core-s\/ps.*/\1/")
    if [ -n "$rate" ]; then
      eta=$(awk -v r="$rate" "BEGIN{printf \"%.1f h for the remaining 48 ps, %.2f d for 50 ps; 5-day ceiling %s\", r*48/3600, r*50/86400, (r*50<5*86400)?\"HELD\":\"EXCEEDED: stop at readout 1250 (25 ps)\"}")
      echo "$(date -Is) $(basename "$d"): measured $rate core-s/ps at readout 100; $eta" >> "$OUT/slow1.eta"
    else echo "$(date -Is) $(basename "$d"): ended before readout 100 (run.done $(cat "$d/run.done"))" >> "$OUT/slow1.eta"; fi
  done' _ "$OUT" ) &
echo "slow1: $(date -Is) four arms launched"; wait; echo 0 > "$OUT/slow1.DONE"; echo "slow1: $(date -Is) done"
