#!/bin/bash
# Emit, verify, and STAGE a pair for the repo drop.  It does not commit: the
# commit message has to say what actually changed, and only a human-written one
# can.  Run it, read what it says, then commit by pathspec.
#
#   ./land_pair.sh Li2 [N2 CO ...]
#
# The order is not negotiable and each step gates the next:
#   emit  -> the emitter's nine refusals get their chance
#   verify (--quick, MUST exit 0) -> V1..V11 on the whole drop, not the new pair
#   diff  -> what moved, per file, INCLUDING files this pair should not touch
#   copy  -> only then, and only the files named
set -u
cd "$(dirname "$0")"
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
OUT=engine_handoff/elements1
REPO=/home/emoore/CIRISHolon/engine/crates/holon-chem/tests/data/elements1
[ $# -ge 1 ] || { echo "usage: $0 <PAIR> [PAIR...]"; exit 2; }

echo "=== emit ==="
python3 -u emit_engine.py "$OUT" || { echo "EMIT FAILED"; exit 1; }
echo
echo "=== verify (must exit 0) ==="
if ! python3 -u verify_elements.py --quick > /tmp/verify_$$.log 2>&1; then
  echo "VERIFY FAILED:"; grep -E "FAIL|Error" /tmp/verify_$$.log | head -20
  exit 1
fi
echo "  $(grep -c '^  PASS' /tmp/verify_$$.log) checks passed, 0 FAIL"
rm -f /tmp/verify_$$.log
echo
echo "=== what moves ==="
moved=""
for f in "$@" ; do
  s="$OUT/$f.json"
  [ -f "$s" ] || { echo "  $f.json was NOT emitted -- is it assembled?"; exit 1; }
  if [ -f "$REPO/$f.json" ]; then
    cmp -s "$s" "$REPO/$f.json" && { echo "  $f.json unchanged"; continue; }
    echo "  $f.json CHANGED ($(wc -c < "$REPO/$f.json") -> $(wc -c < "$s") bytes)"
  else
    echo "  $f.json NEW ($(wc -c < "$s") bytes)"
  fi
  moved="$moved $f.json"
done
for f in atoms.json manifest.json; do
  if ! cmp -s "$OUT/$f" "$REPO/$f"; then
    echo "  $f CHANGED"; moved="$moved $f"
  fi
done
# Anything ELSE moving is a warning, not a landing: a pair's landing should not
# rewrite a sibling's file, and the six landed species are byte-stable.
for s in "$OUT"/*.json; do
  b=$(basename "$s")
  case " $moved " in *" $b "*) continue ;; esac
  case "$b" in atoms.json|manifest.json) continue ;; esac
  if [ -f "$REPO/$b" ] && ! cmp -s "$s" "$REPO/$b"; then
    echo "  !! $b ALSO CHANGED and was not asked for -- investigate before"
    echo "     landing; a pair should not rewrite a sibling"
    exit 1
  fi
done
[ -n "$moved" ] || { echo "  nothing to land"; exit 0; }
echo
echo "=== copy ==="
for b in $moved; do cp "$OUT/$b" "$REPO/$b"; echo "  -> $REPO/$b"; done
echo
echo "now, from /home/emoore/CIRISHolon:"
echo "  git add -- engine/crates/holon-chem/tests/data/elements1/"
echo "  git commit -F <msgfile> -- engine/crates/holon-chem/tests/data/elements1/"
echo "and tell the lead the new pin.  (This fires the engine's R2 digest gate"
echo "by design; the lead re-pins engine-side afterward.)"
