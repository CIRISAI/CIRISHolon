#!/bin/bash
# ONE monitor for every long-running lane (2026-09-24). Detached, no agent owns it, no timer
# is re-armed by anyone. Every 10 minutes it rewrites status.txt (one line per lane) and
# appends to events.log ONLY on a state change a human would act on: a lane finished,
# crashed, stopped writing, found a witness, or the disk ran low. The lead waits on
# events.log's line count, not on a clock.
set -u
ROOT=/home/emoore/CIRISHolon
M=$ROOT/conformance/monitor; S=$M/status.txt; E=$M/events.log; touch "$E"
RANK=$ROOT/.claude/worktrees/agent-abf64e2822d6ca5a5/conformance/rank
OBS=$ROOT/conformance/water_observatory/replace0
declare -A seen
ev() { local k=$1; shift; [ -n "${seen[$k]:-}" ] && return; seen[$k]=1; echo "$(date -Is) $*" >> "$E"; }
stale() { [ -f "$1" ] && [ $(( $(date +%s) - $(stat -c %Y "$1") )) -gt "$2" ]; }
while true; do
  {
  echo "# $(date -Is)"
  # QVM-RANK-1
  if pgrep -f "qvm_rank1 tower" >/dev/null; then
    echo "RANK-1 running: $(tail -1 $RANK/shot/run3.log 2>/dev/null | cut -c1-120)"
    stale $RANK/shot/run3.log 3600 && ev rank_stale "RANK-1 STALLED: run3.log unwritten for over an hour"
  else
    echo "RANK-1 not running; last: $(tail -1 $RANK/shot/run3.log 2>/dev/null | cut -c1-120)"
    ev rank_end "RANK-1 search process EXITED (finished, stopped or crashed); last line: $(tail -1 $RANK/shot/run3.log 2>/dev/null | cut -c1-160)"
  fi
  for w in $(ls $RANK/shot $RANK 2>/dev/null | grep -i "m7-rank6-WITNESS" ); do ev "wit_$w" "RANK-1 WITNESS FILE WRITTEN: $w"; done
  # SLOW-1
  for a in T293_seed0 T293_seed1 T293_seed2 T400_seed0; do
    d=$OBS/slow1/$a; echo "SLOW-1 $a: $(grep -oE 'readout +[0-9]+' $d/scout.err 2>/dev/null | tail -1) done=$(cat $d/run.done 2>/dev/null)"
    [ -f $d/run.done ] && [ "$(cat $d/run.done)" != "0" ] && ev "slow_fail_$a" "SLOW-1 arm $a EXITED NONZERO: $(cat $d/run.done)"
    [ ! -f $d/run.done ] && stale $d/scout.err 3600 && ev "slow_stale_$a" "SLOW-1 arm $a STALLED: no output for over an hour"
  done
  [ -f $OBS/slow1/slow1_read.DONE ] && ev slow_read "SLOW-1 READ WRITTEN: $OBS/slow1/slow1_read.txt"
  # SLOW-3 (fresh seeds 3 and 4, cores 16 and 17)
  for a in T293_seed3 T293_seed4; do
    d=$OBS/slow3/$a; echo "SLOW-3 $a: $(grep -oE 'readout +[0-9]+' $d/scout.err 2>/dev/null | tail -1) done=$(cat $d/run.done 2>/dev/null)"
    [ -f $d/run.done ] && [ "$(cat $d/run.done)" != "0" ] && ev "slow3_fail_$a" "SLOW-3 arm $a EXITED NONZERO: $(cat $d/run.done)"
    [ ! -f $d/run.done ] && stale $d/scout.err 3600 && ev "slow3_stale_$a" "SLOW-3 arm $a STALLED: no output for over an hour"
  done
  [ "$(cat $OBS/slow3/T293_seed3/run.done 2>/dev/null)" = "0" ] && [ "$(cat $OBS/slow3/T293_seed4/run.done 2>/dev/null)" = "0" ] && ev slow3_done "SLOW-3 ARMS DONE: $OBS/slow3/T293_seed3 and T293_seed4 run.done 0"
  # fine seeds
  for a in L T; do
    d=$OBS/response1_fine_${a}_seed0; echo "fine_$a: $(grep -oE 'readout +[0-9]+' $d/scout.err 2>/dev/null | tail -1) done=$(cat $d/run.done 2>/dev/null)"
    [ -f $d/run.done ] && ev "fine_end_$a" "fine seed $a FINISHED with exit $(cat $d/run.done)"
    [ ! -f $d/run.done ] && stale $d/flexible.walk 7200 && ev "fine_stale_$a" "fine seed $a STALLED: its walk unwritten for two hours"
  done
  # disk
  free=$(df -BG --output=avail /home | tail -1 | tr -dc 0-9); echo "disk ${free} GiB free"
  [ "$free" -lt 20 ] && ev disk_low "DISK LOW: ${free} GiB free"
  } > "$S.tmp"; mv "$S.tmp" "$S"
  sleep 600
done
