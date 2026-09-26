#!/bin/bash
# SLOW-3 seed 4 RELAUNCHED 2026-09-26: the first run died at readout 425 on 2026-09-25 18:54 CDT when
# the disk filled (another session's builds; `walk row: No space left on device`, kept in
# T293_seed4_crashed_diskfull/). Same binary, flags and core as slow3.sh, plus the fine launcher's
# disk watchdog: SIGSTOP under 12 GiB free, SIGCONT when it recovers - a paused arm loses nothing.
set -u
ROOT=/home/emoore/CIRISHolon; BASE="$ROOT/conformance/water_observatory/replace0"
BIN="$BASE/slow3/replace0_slow3"; STIFF="$BASE/transport_seed0/stiffness.json"
d="$BASE/slow3/T293_seed4"; mkdir -p "$d"; FLOOR_GIB=12
free_gib() { df -BG --output=avail "$BASE" | tail -1 | tr -dc '0-9'; }
cd "$ROOT/engine"
( taskset -c 17 "$BIN" scout "$d" --cells 5 --settle 1000 --settle-rigid 1000 --readouts 2500 --readout-fs 20 --stiffness "$STIFF" --seed 4 --workers 1 --temperature-k 293 > "$d/scout.log" 2> "$d/scout.err"; echo $? > "$d/run.done" ) &
echo "slow3-seed4: $(date -Is) relaunched on core 17, $(free_gib) GiB free"
sleep 5; pid=$(pgrep -f "replace0_slow3 scout $d")
paused=0
while [ -n "$pid" ] && kill -0 $pid 2>/dev/null; do
  if [ "$(free_gib)" -lt "$FLOOR_GIB" ] && [ "$paused" = 0 ]; then kill -STOP $pid; paused=1; echo "slow3-seed4: $(date -Is) PAUSED: $(free_gib) GiB free"
  elif [ "$(free_gib)" -ge "$FLOOR_GIB" ] && [ "$paused" = 1 ]; then kill -CONT $pid; paused=0; echo "slow3-seed4: $(date -Is) resumed: $(free_gib) GiB free"; fi
  sleep 15
done
wait; echo "slow3-seed4: $(date -Is) exit $(cat $d/run.done)"
