#!/bin/bash
# RESPONSE-1 (RESPONSE1_PREREG.md): the fluid element as a RESPONSE on the 432-water rigid
# operator. Eight arms — longitudinal and transverse kicks at 50 m/s on three seeds, and one
# seed of each at 200 m/s as the nonlinearity control — each twelve cycles of kick, 3.14 ps of
# NVE (314 readouts of 10 fs), rescale, sign flip. Three eight-worker groups; the third waits
# for whatever holds its cores (a file the caller names) before it starts. Every arm settles its
# own box: the settle is deterministic per seed and bit-identical across worker counts, so
# arms L and T of a seed start from the SAME state without a checkpoint. Plant PR-4 must have
# passed on this binary before this launcher runs (`replace0 plant-kick`); its pass line is
# copied into the group logs so the record carries it.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/replace0"
BASE="$ROOT/conformance/water_observatory/replace0"
STIFF="$BASE/transport_seed0/stiffness.json"
FLOOR_GIB=8
WAIT_FOR="${1:-}"   # optional: a path; group C starts only once it exists
cd "$ROOT/engine" || exit 2
rm -f "$BASE/response1.KILLED" "$BASE/response1.DONE" "$BASE/response1.LOWDISK"
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
have=$(free_gib); echo "response1: $(date -Is) pid $$; $have GiB free; binary $(stat -c %y "$BIN" | cut -c1-19)"
[ "$have" -lt "$FLOOR_GIB" ] && { echo "REFUSED: $have GiB free" > "$BASE/response1.LOWDISK"; exit 3; }
# plant PR-4 on this binary, both amplitudes, before any arm
for mps in 50 200; do
  taskset -c 8-11 "$BIN" plant-kick "$BASE" --cells 6 --seed 0 --kick-mps "$mps" > "$BASE/response1_pr4_$mps.log" 2>&1 \
    || { echo "plant PR-4 at $mps m/s did not pass; see response1_pr4_$mps.log" > "$BASE/response1.KILLED"; exit 4; }
  grep "plant PR-4 PASS" "$BASE/response1_pr4_$mps.log" || { echo "no PASS line at $mps" > "$BASE/response1.KILLED"; exit 4; }
done
arm() {  # arm CORES KIND SEED MPS TAG
  local cores=$1 kind=$2 seed=$3 mps=$4 tag=$5
  local out="$BASE/response1_${tag}_seed$seed"; mkdir -p "$out"
  echo "response1: $(date -Is) arm $tag seed $seed on cores $cores"
  taskset -c "$cores" "$BIN" scout "$out" --cells 6 --settle 3000 --settle-rigid 3000 --readout-fs 10 --stiffness "$STIFF" --seed "$seed" --workers 8 \
      --kick "$kind" --kick-mps "$mps" --kick-cycles 12 --kick-relax 314 > "$out/scout.log" 2> "$out/scout.err"
  local c=$?; echo "$c" > "$out/run.done"; echo "response1: $(date -Is) arm $tag seed $seed exit $c"; return $c
}
group() {  # group CORES [arms...] each "KIND:SEED:MPS:TAG"
  local cores=$1; shift
  for spec in "$@"; do IFS=: read -r kind seed mps tag <<< "$spec"; arm "$cores" "$kind" "$seed" "$mps" "$tag" || return 1; done
}
( group 8-15  L:0:50:L   T:1:50:T   L:0:200:L200 ) > "$BASE/response1_groupA.log" 2>&1 &
pa=$!
( group 24-31 T:0:50:T   L:2:50:L   T:0:200:T200 ) > "$BASE/response1_groupB.log" 2>&1 &
pb=$!
( if [ -n "$WAIT_FOR" ]; then until [ -e "$WAIT_FOR" ]; do sleep 30; done; echo "response1: $(date -Is) $WAIT_FOR present; group C starts"; fi
  group 16-23 L:1:50:L   T:2:50:T ) > "$BASE/response1_groupC.log" 2>&1 &
pc=$!
echo "response1: groups A $pa (8-15), B $pb (24-31), C $pc (16-23${WAIT_FOR:+, after $WAIT_FOR})"
( while kill -0 $pa 2>/dev/null || kill -0 $pb 2>/dev/null || kill -0 $pc 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ]; then echo "$(date -Is): under the floor; arms stopped" > "$BASE/response1.LOWDISK"; pkill -P $pa; pkill -P $pb; pkill -P $pc; kill $pa $pb $pc 2>/dev/null; break; fi; sleep 120
  done ) &
watch=$!; fail=0
for p in $pa $pb $pc; do wait "$p" || fail=1; done
kill "$watch" 2>/dev/null
cat "$BASE"/response1_group?.log
[ "$fail" -ne 0 ] && { echo "an arm exited nonzero; see response1_*_seed*/scout.err" > "$BASE/response1.KILLED"; exit 1; }
echo 0 > "$BASE/response1.DONE"
