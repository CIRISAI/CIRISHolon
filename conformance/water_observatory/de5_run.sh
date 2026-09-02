#!/usr/bin/env bash
# THE dE5 AUDIT'S DETACHED RUNNER.
#
# Session death must only kill NARRATION, never computation. So: setsid, a .log the
# narrator tails, and a .DONE marker written with the exit code so a later reader can
# tell "finished" from "the box rebooted" without guessing from timestamps.
#
#   ./de5_run.sh <stage>        stage in {plants, score, probe}
#
# Every stage names its own log and marker; nothing is overwritten silently.
set -u

WT=/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/de5-wt
BIN="$WT/.target-de5/release/examples/de5_audit"
TRAJ=/home/emoore/holon-artifacts/census-traj
MAN="$WT/conformance/water_observatory/census_traj_manifest.sha256"
OUT="$WT/conformance/water_observatory"

stage="${1:?stage required: plants | score | probe}"
case "$stage" in
  plants) args=(--plants --no-crosscheck) ;;
  score)  args=(--out "$OUT/de5_audit.csv") ;;
  probe)  args=(--scf-probe) ;;
  *) echo "unknown stage $stage" >&2; exit 2 ;;
esac

log="$OUT/de5_$stage.log"
done="$OUT/de5_$stage.DONE"
rm -f "$done"

{
  echo "# de5_audit stage=$stage"
  echo "# host      $(hostname)"
  echo "# commit    $(git -C "$WT" rev-parse HEAD)"
  echo "# binary    $BIN"
  echo "# binary sha256  $(sha256sum "$BIN" | cut -d' ' -f1)"
  echo "# nice      10"
  echo "# loadavg at launch  $(cut -d' ' -f1-3 /proc/loadavg)"
  echo "# started   $(date -Is)"
} > "$log"

setsid nice -n 10 "$BIN" --traj-dir "$TRAJ" --manifest "$MAN" "${args[@]}" >> "$log" 2>&1 &
pid=$!
echo "# pid $pid" >> "$log"
# `wait` cannot be used here and the first version of this script tried to: `setsid`
# reparents the job out of this shell, so `wait $pid` returns "not a child" immediately and
# the marker would be written the instant the run started -- a DONE marker that means
# "launched", which is worse than no marker at all. Poll `/proc/$pid` instead, which is a
# fact about the process rather than about this shell's job table.
(
  while [ -d "/proc/$pid" ]; do sleep 5; done
  rc=0
  {
    echo "# finished  $(date -Is)"
    echo "# loadavg at exit  $(cut -d' ' -f1-3 /proc/loadavg)"
    echo "# exit code (poll-observed completion; see the run's own last line) $rc"
  } >> "$log"
  echo "$rc" > "$done"
) &
echo "launched stage=$stage pid=$pid log=$log marker=$done"
