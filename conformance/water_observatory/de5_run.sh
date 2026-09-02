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

# WT RESOLUTION. The campaign originally ran in a session worktree (a copy of this
# tree), and a session-keyed path dies with the session -- gate 10a3. The default is
# therefore this repo's own root, resolved from THIS SCRIPT'S OWN LOCATION: the file
# lives in conformance/water_observatory/, so the root is two directories up. That
# is the only resolver that works from any cwd AND from outside a checkout.
#
# It used to be `git rev-parse --show-toplevel`. b2-ewald found the hole and it is
# worth naming rather than just patching: from outside any checkout rev-parse prints
# NOTHING and exits nonzero, so `WT=$(...)` assigns the EMPTY STRING -- which `set -u`
# accepts, because the variable IS set. Measured here from /tmp before changing
# anything: this script refused with exit 3 and wrote zero files, so the BIN hardening
# already stopped it dying invisibly. But it refused for the WRONG REASON, naming
# `/engine/target/...` and telling the reader to `cd /engine`. A refusal that
# misidentifies the fault sends someone hunting a missing binary when the real fault
# is that the script does not know where the repo is. So the root is now resolved from
# a fact that cannot go empty, and VALIDATED where it is resolved.
self=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
WT=${DE5_WT:-$(cd -- "$self/../.." && pwd -P)}

# Validate the root POSITIVELY, here, so a wrong or empty WT fails saying so rather
# than downstream wearing another error's clothes. Distinct exit code from the BIN
# refusal, because they are different faults with different remedies.
if [ -z "$WT" ] || [ ! -d "$WT/conformance/water_observatory" ] || [ ! -d "$WT/engine" ]; then
  {
    echo "de5_run.sh REFUSES: WT does not look like a CIRISHolon checkout."
    echo "  WT resolved to:   [${WT:-<empty>}]"
    echo "  script location:  $self"
    echo "  expected under WT: conformance/water_observatory/ and engine/"
    echo "Name it explicitly:  DE5_WT=/path/to/CIRISHolon $0 <stage>"
  } >&2
  exit 4
fi
# THE PARKED TRAJECTORIES live OUTSIDE the repo -- the closure census banked them
# there and DE5_PREREG.md section 2.5 pins their sha256, which is what actually
# identifies them. Not session-keyed, but machine-keyed, so it takes an override
# for the same reason WT does. The pin is the identity; the path is just where to
# look.
TRAJ=${DE5_TRAJ:-/home/emoore/holon-artifacts/census-traj}
MAN="$WT/conformance/water_observatory/census_traj_manifest.sha256"
OUT="$WT/conformance/water_observatory"

# THE BINARY, and the owner's answer to "archival or re-runnable?": RE-RUNNABLE.
# DE5_PREREG.md section 7 designs a re-run into the freeze as a pre-committed
# follow-up ("the same instrument is run on the hydrogen arm's eight seeds ...
# with no threshold changed"), and DE5_RESUME.md names this script as the relaunch
# command. So it has to resolve in a checkout that is not this lane's.
#
# `.target-de5` was this lane's ASSIGNED CARGO_TARGET_DIR. That is gate 10a3's
# defect one step down from the session-keyed WT the gate already fixed: not
# session-keyed but LANE-keyed, and a fresh checkout builds to engine/target
# instead. So search, in the order a reader would, and REFUSE BY NAME rather than
# launch a path that is not there.
#
# The refusal is not decoration. `setsid` on a missing binary exits immediately,
# the /proc poll below sees no process, and the DONE marker gets written with
# rc=0 -- a marker meaning "finished" over a run that never started. This lane
# has already produced one false DONE marker (see the poll comment below); it is
# not going to produce a second one this way.
BIN=${DE5_BIN:-}
if [ -z "$BIN" ]; then
  for cand in \
      ${CARGO_TARGET_DIR:+"$CARGO_TARGET_DIR/release/examples/de5_audit"} \
      "$WT/engine/target/release/examples/de5_audit" \
      "$WT/.target-de5/release/examples/de5_audit"; do
    if [ -x "$cand" ]; then BIN="$cand"; break; fi
  done
fi
if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
  {
    echo "de5_run.sh REFUSES: no executable de5_audit found. Looked at, in order:"
    echo "  \${CARGO_TARGET_DIR}/release/examples/de5_audit   (CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-unset})"
    echo "  $WT/engine/target/release/examples/de5_audit"
    echo "  $WT/.target-de5/release/examples/de5_audit"
    echo "Build it:"
    echo "  cd $WT/engine && cargo build --release --example de5_audit -p holon-chem"
    echo "or name it directly:  DE5_BIN=/path/to/de5_audit $0 <stage>"
  } >&2
  exit 3
fi

stage="${1:?stage required: plants | score | probe}"
case "$stage" in
  plants) args=(--plants --no-crosscheck) ;;
  score)  args=(--out "$OUT/de5_audit.csv") ;;
  probe)  args=(--scf-probe) ;;
  *) echo "unknown stage $stage" >&2; exit 2 ;;
esac

log="$OUT/de5_$stage.log"
done="$OUT/de5_$stage.DONE"

# DRY MODE. b2-ewald's point, and it is right: testing a launcher by launching it is
# not a test. Everything above is resolution; this prints what was resolved and exits
# without touching a marker, a log, or a process, so the resolver can be exercised on
# a tree that has running compute in it.
if [ -n "${DE5_DRY:-}" ]; then
  echo "DE5_DRY: resolution only. Nothing launched, nothing written."
  echo "  self      $self"
  echo "  WT        $WT"
  echo "  BIN       $BIN"
  echo "  TRAJ      $TRAJ"
  echo "  MAN       $MAN"
  echo "  stage     $stage"
  echo "  args      ${args[*]}"
  echo "  log       $log"
  echo "  marker    $done"
  exit 0
fi

rm -f "$done"

{
  echo "# de5_audit stage=$stage"
  echo "# host      $(hostname)"
  # If WT is not a checkout this line would otherwise go SILENTLY BLANK, and the
  # standing constraint is that run-location provenance lives in the logs. An
  # absent commit is a fact the log has to state, not one it may omit.
  echo "# commit    $(git -C "$WT" rev-parse HEAD 2>/dev/null || echo 'UNKNOWN — WT is not a git checkout')"
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
