#!/bin/bash
# REPLACE-0 TRANSPORT re-read at the declared longer ladder (2026-09-13): the same three branch
# points and the SAME flexible arms (reused from their bundles, bit for bit), the rigid arm
# re-run, and the MSD read to readouts/2 instead of readouts/4. The question is whether the
# departure that was monotone to the last lag SATURATES or keeps growing; nothing else changes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
cd "$ROOT/engine" || exit 2
rm -f "$BASE/reread.DONE" "$BASE/reread.KILLED"
have=$(df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9')
echo "reread: $(date -Is) pid $$; $have GiB free"
[ "$have" -lt 8 ] && { echo "REFUSED: $have GiB free" > "$BASE/reread.KILLED"; exit 3; }
pids=()
for k in 0 1 2; do
  OUT="$BASE/transport_seed$k"
  taskset -c $((24 + k)) "$BIN" run "$OUT" --seed "$k" --settle 102100 --frames 115105 --readouts 150 --reuse \
      > "$OUT/reread.log" 2> "$OUT/reread.err" &
  pids+=($!)
done
fail=0
for k in 0 1 2; do wait "${pids[$k]}"; c=$?; echo "reread: seed $k exit $c"; [ "$c" -ne 0 ] && fail=1; done
[ "$fail" -ne 0 ] && { echo "a re-read exited nonzero" > "$BASE/reread.KILLED"; exit 1; }
echo 0 > "$BASE/reread.DONE"
