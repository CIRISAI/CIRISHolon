#!/bin/bash
# Reads each RESPONSE-1 arm as it lands: waits for run.done, converts the banked walk and
# velocities to a trajectory the frozen rung-2 reader takes, runs `rung2 --response` under
# Amendment 1's read, and leaves the output beside the arm as response1.read.txt. Per-arm
# reads are per-seed readings; the pooled read over seeds is run once every 50 m/s arm of
# an axis is in (response1_pooled_{L,T}.txt). Never touches the arms' own files.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BASE="$ROOT/conformance/water_observatory/replace0"
RUNG2="$ROOT/engine/target/release/examples/rung2"
W2T="$BASE/rung2_occ/walk2traj.py"
S="$BASE/response1_traj"; mkdir -p "$S"
read_arm() {  # read_arm TAG SEED AXIS
  local tag=$1 seed=$2 axis=$3 dir="$BASE/response1_${tag}_seed$seed"
  until [ -f "$dir/run.done" ]; do sleep 120; done
  local out="$S/${tag}_seed$seed"; mkdir -p "$out"
  # walk2traj expects {B}/{SRC}{k}/{arm}.walk; point it at one seed by a symlink tree
  # (walk2traj stamps SEEDS[k] from the directory index, so the tree names the seed's index)
  local tree="$S/tree_${tag}_seed$seed"; rm -rf "$tree"; mkdir -p "$tree/arm$seed"
  ln -sf "$dir/rigid.walk" "$tree/arm$seed/rigid.walk"; ln -sf "$dir/rigid.vwalk" "$tree/arm$seed/rigid.vwalk"
  python3 "$W2T" "$out" "$tree" arm $((seed + 1)) 10.0 rigid > "$out/convert.log" 2>&1
  taskset -c 0-3 "$RUNG2" "$out" rigid --response "$axis" --cycles 12 --relax 314 > "$dir/response1.read.txt" 2>&1
  echo "$(date -Is) read $tag seed $seed -> $dir/response1.read.txt (exit $(cat "$dir/run.done"))" >> "$S/reads.log"
}
pooled() {  # pooled TAG AXIS SEEDS...
  local tag=$1 axis=$2; shift 2
  local pool="$S/pooled_$tag/rigid"; rm -rf "$pool"; mkdir -p "$pool"
  for k in "$@"; do ln -sf "$S/${tag}_seed$k/rigid/seed$k.traj" "$pool/seed$k.traj"; done
  taskset -c 0-3 "$RUNG2" "$S/pooled_$tag" rigid --response "$axis" --cycles 12 --relax 314 > "$BASE/response1_pooled_$tag.txt" 2>&1
  echo "$(date -Is) pooled $tag over seeds $* -> response1_pooled_$tag.txt" >> "$S/reads.log"
}
( read_arm L 0 L; read_arm L 1 L; read_arm L 2 L; pooled L L 0 1 2; read_arm L200 0 L ) &
( read_arm T 0 T; read_arm T 1 T; read_arm T 2 T; pooled T T 0 1 2; read_arm T200 0 T ) &
wait
echo 0 > "$S/reads.DONE"
