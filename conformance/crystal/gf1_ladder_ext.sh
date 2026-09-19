#!/bin/bash
# GF1 branch (c): S2 failed at x = 0.25 (M2/N = 0.207 against 0.05), so the ladder is extended
# to x = 0.0625 BEFORE any verdict, as the prereg says; x = 0.015625 is read beside it as a
# LABELLED EXTRA (not a gate) so the trend c(x) toward the stabilizer fixed point is visible.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/gf1_read"
OUT="$ROOT/conformance/crystal/gf1_ladder"; mkdir -p "$OUT"
GATE=1e-3
point() { local cores=$1 x=$2 n=$3
  for chi in 6 8 11; do local f="$OUT/x${x}_N${n}_chi${chi}.json"
    [ -s "$f" ] || taskset -c "$cores" "$BIN" --x "$x" --n "$n" --chi "$chi" --sweeps 0 --nl-sweeps 0 --box 10 > "$f" 2> "$f.err" || { echo "x=$x N=$n chi=$chi: reader exited nonzero" >> "$OUT/ladder.log"; continue; }
    local v; v=$(python3 -c "import json; d=json.load(open('$f')); print(d['variance_per_site'])" 2>/dev/null)
    local pass; pass=$(python3 -c "print(1 if float('$v') <= $GATE else 0)" 2>/dev/null || echo 0)
    echo "$(date -Is) x=$x N=$n chi=$chi variance/site=$v $([ "$pass" = 1 ] && echo ADMITTED || echo refused) [extension]" >> "$OUT/ladder.log"
    [ "$pass" = 1 ] && { echo "$chi" > "$OUT/x${x}_N${n}.admitted"; return 0; }
  done; echo "REFUSED at chi=11" > "$OUT/x${x}_N${n}.refused"; }
for x in 0.0625 0.015625; do for n in 8 12 16 24 32 48; do point 0-3 $x $n; done; done
echo 0 > "$OUT/ladder_ext.DONE"
