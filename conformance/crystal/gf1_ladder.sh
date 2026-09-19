#!/bin/bash
# GF1's ladder (GF1_PREREG.md §2 as re-sized by GF1_AMENDMENT_1.md): couplings x in
# {0.25, 1, 4, 16}, volumes N in {8, 12, 16, 24, 32, 48}, each point read at the smallest
# chi in {6, 8, 11} whose energy variance per site passes A3's gate (1e-3); a point that
# fails at chi = 11 is REFUSED and banked as such. Two lanes on the desktop's P-cores
# (0-3, 4-7); each chi = 11 reading needs ~8.6 GB, so never more than two at once.
# Reads S1, S2, S4; M2_nl is M2 by the amendment's C1 and no descent is run.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="$ROOT/engine/target/release/examples/gf1_read"
OUT="$ROOT/conformance/crystal/gf1_ladder"; mkdir -p "$OUT"
GATE=1e-3
rm -f "$OUT/ladder.DONE" "$OUT/ladder.KILLED"
point() {  # point CORES X N
  local cores=$1 x=$2 n=$3
  for chi in 6 8 11; do
    local f="$OUT/x${x}_N${n}_chi${chi}.json"
    [ -s "$f" ] || taskset -c "$cores" "$BIN" --x "$x" --n "$n" --chi "$chi" --sweeps 0 --nl-sweeps 0 --box 10 > "$f" 2> "$f.err" || { echo "x=$x N=$n chi=$chi: reader exited nonzero" >> "$OUT/ladder.log"; continue; }
    local v; v=$(python3 -c "import json,sys; d=json.load(open('$f')); print(d.get('variance_per_site', d.get('variance',{}).get('per_site','nan')))" 2>/dev/null)
    local pass; pass=$(python3 -c "print(1 if float('$v') <= $GATE else 0)" 2>/dev/null || echo 0)
    echo "$(date -Is) x=$x N=$n chi=$chi variance/site=$v $([ "$pass" = 1 ] && echo ADMITTED || echo refused)" >> "$OUT/ladder.log"
    [ "$pass" = 1 ] && { echo "$chi" > "$OUT/x${x}_N${n}.admitted"; return 0; }
  done
  echo "REFUSED at chi=11" > "$OUT/x${x}_N${n}.refused"; return 0
}
lane() { local cores=$1; shift; for spec in "$@"; do IFS=: read -r x n <<< "$spec"; point "$cores" "$x" "$n"; done; }
# small N first so S2 at x=0.25 and the plants' region report early; the N=48 points last
( lane 0-3 0.25:8 0.25:16 0.25:32 1:12 1:24 1:48 4:8 4:16 4:32 16:12 16:24 16:48 ) &
pa=$!
( lane 4-7 0.25:12 0.25:24 0.25:48 1:8 1:16 1:32 4:12 4:24 4:48 16:8 16:16 16:32 ) &
pb=$!
echo "gf1 ladder: $(date -Is) lanes $pa (0-3) $pb (4-7)" >> "$OUT/ladder.log"
wait $pa; wait $pb
echo 0 > "$OUT/ladder.DONE"
