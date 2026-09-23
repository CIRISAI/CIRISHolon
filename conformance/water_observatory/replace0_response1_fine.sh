#!/bin/bash
# RESPONSE-1, the FINE-MODEL SEED owed by the prereg (§1) and Amendment 5 (RESPONSE1_AMENDMENT_6.md),
# 2026-09-23. ONE arm: longitudinal, seed 0, 50 m/s, twelve cycles of 314 readouts, on the fine
# (flexible, all-atom) model, `--kick-arm flexible`, eight workers pinned to the P-cores 0-15.
# The box, the 3,000-frame fine settle, the reference geometry and the rigid settle by criterion
# are rigid seed 0's bit for bit; the fine arm then settles by the rigid rule on the six retained
# modes and runs the kick on the engine's own integrator (Amendment 6, (1)-(5)).
#
# Binary: replace0/replace0_fine (built in the Amendment 6 worktree; the closed campaign's
# engine/target/release/examples/replace0 is NOT overwritten). Plant PR-4 on the fine arm must
# have passed on this binary (response1_fine_pr4_50.log); its PASS line is checked here and
# copied into the launch log.
#
# Readout cadence: 384 fine frames = 10.0082 fs (the fine step is the tables' hold); read with
#   walk2traj.py OUT TREE arm 1 10.0082 flexible   then   rung2 OUT flexible --response L --cycles 12 --relax 314
#
# WALL ESTIMATE (the prereg's price was the wrong regime; see Amendment 6, "Cost"):
#   arm = 3,000 fine frames (first settle) + rigid seed 0's 12,500 rigid steps (bit-identical, ~3.7 h)
#       + fine settle >= 42,630 frames (floor 1.1 ps; cap 426,300) + 3,768 x 384 = 1,446,912 production
#       frames (37.71 ps; 38,369 frames/ps)  =>  >= 1.49 M fine frames
#   measured price on 432 waters: 1.35 wall-s/frame, 4 workers on E-cores 21-24 (51,800 wall-s/ps)
#   => 14 days at 0.8 s/frame, 17 at 1.0, 23 at 1.35 on the launch's 8 workers on P-cores 0-15;
#      ~3 weeks is the planning figure. The prereg's "~150 core-hours" (= 14,300 s/ps) was the
#      128-water fine price carried to 432 waters, and "2,171" was the operator's, not the model's.
#   The run prints wall-s/frame every 500 frames of the fine settle and wall-s/ps + ETA every 25 readouts.
#
# DISK WATCHDOG as in replace0_response1_c.sh: SIGSTOP the arm while free space is under the
# floor, SIGCONT when it recovers; a paused arm loses nothing, an append into a full disk panics.
set -u
# RESPONSE1_ROOT overrides the tree the run writes into (launched from the Amendment 6 worktree
# into the MAIN checkout: RESPONSE1_ROOT=/home/emoore/CIRISHolon).
ROOT="${RESPONSE1_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
BASE="$ROOT/conformance/water_observatory/replace0"
BIN="$BASE/replace0_fine"
STIFF="$BASE/transport_seed0/stiffness.json"
FLOOR_GIB=12
cd "$ROOT/engine" || exit 2
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
echo "response1-fine: $(date -Is) pid $$; $(free_gib) GiB free; binary $(stat -c %y "$BIN" | cut -c1-19) sha $(sha256sum "$BIN" | cut -c1-16)"
grep "plant PR-4 PASS (fine arm)" "$BASE/response1_fine_pr4_50.log" || { echo "response1-fine: no fine PR-4 PASS on record; REFUSED"; exit 4; }
OUT="$BASE/response1_fine_L_seed0"
[ -e "$OUT/run.done" ] && { echo "response1-fine: $OUT/run.done exists; REFUSED to overwrite"; exit 5; }
mkdir -p "$OUT"
echo "response1-fine: $(date -Is) arm fine_L seed 0 on cores 0-15, 8 workers"
taskset -c 0-15 "$BIN" scout "$OUT" --cells 6 --settle 3000 --settle-rigid 3000 --readout-fs 10 --stiffness "$STIFF" --seed 0 --workers 8 \
    --kick L --kick-mps 50 --kick-cycles 12 --kick-relax 314 --kick-arm flexible > "$OUT/scout.log" 2> "$OUT/scout.err" &
arm=$!
echo "response1-fine: arm pid $arm"
( paused=0
  while kill -0 "$arm" 2>/dev/null; do
    if [ "$(free_gib)" -lt "$FLOOR_GIB" ] && [ "$paused" = 0 ]; then kill -STOP "$arm"; paused=1; echo "response1-fine: $(date -Is) PAUSED: $(free_gib) GiB free"
    elif [ "$(free_gib)" -ge "$FLOOR_GIB" ] && [ "$paused" = 1 ]; then kill -CONT "$arm"; paused=0; echo "response1-fine: $(date -Is) resumed: $(free_gib) GiB free"; fi
    sleep 15
  done ) &
wait "$arm"; c=$?
echo "$c" > "$OUT/run.done"
echo "response1-fine: $(date -Is) arm fine_L seed 0 exit $c"
wait
