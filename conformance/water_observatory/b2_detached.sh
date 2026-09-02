#!/usr/bin/env bash
# B2's long runs. setsid + done-markers, so session death kills narration and not compute.
# Pinned to an E-core: M-PLACEMENT-LOTTERY's remedy is quiet-and-pinned, and an E-core has
# no SMT sibling whose load nobody controls.
set -u
# WT defaults to this repo's own root; the lane's worktree was a copy of this tree and its
# path died with the session (gate 10a3). Override with B2_WT.
#
# Resolved from THIS SCRIPT'S OWN LOCATION rather than from `git rev-parse`, which resolves
# against the CALLER'S cwd: run from outside a checkout it prints nothing to stdout, WT comes
# back EMPTY, `cd ""` silently succeeds, the binary is not found, and the marker write into
# `/` fails -- so the run leaves no log and no marker at all. A detached run whose failure is
# invisible is the one failure this script exists to prevent. The script lives in the repo,
# so its own path gives the root from any cwd and is no more session-keyed than the file is.
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
WT=${B2_WT:-$(cd "$here/../.." && pwd)}
OUT=$WT/conformance/water_observatory
BIN=$WT/engine/target/release/examples/b2_longrange
CORE=${CORE:-24}

# LOUDLY, and before anything is launched. Both of these are the operator's to fix and
# neither is worth discovering from a marker that never appears.
[ -d "$OUT" ] || { echo "b2_detached: $WT is not a CIRISHolon checkout (no $OUT)" >&2; exit 2; }
[ -x "$BIN" ] || { echo "b2_detached: no binary at $BIN -- build it first:" >&2
                   echo "  cargo build --release --manifest-path $WT/engine/Cargo.toml -p holon-render --example b2_longrange" >&2
                   exit 2; }

# The two logs below are COMMITTED receipts cited by B2_RESULTS.md, and this script's whole
# job is to overwrite them with a fresh run. That is right when the re-run IS the new record
# and wrong every other time, so it is asked rather than assumed: a TRACKED log is refused
# unless B2_FORCE=1.
#
# This guard replaced a comment saying the same thing, because the comment did not work. Its
# author clobbered both receipts inside a minute of writing it, while testing the path
# resolution from a directory where the script was not supposed to launch anything at all.
guard() {  # guard <logfile>
  local log="$OUT/$1"
  git -C "$WT" ls-files --error-unmatch "$log" >/dev/null 2>&1 || return 0
  [ "${B2_FORCE:-0}" = 1 ] && return 0
  echo "b2_detached: $1 is a COMMITTED receipt cited by B2_RESULTS.md." >&2
  echo "  Re-run over it with B2_FORCE=1, or B2_DRY=1 to check paths without launching." >&2
  exit 3
}

# B2_DRY=1 prints what WOULD run and exits. Every path this script derives is then checkable
# without starting twenty-five minutes of compute, which is the only reason the resolution
# above can be tested at all.
if [ "${B2_DRY:-0}" = 1 ]; then
  printf 'WT   = %s\nOUT  = %s\nBIN  = %s\nCORE = %s\n' "$WT" "$OUT" "$BIN" "$CORE"
  printf 'would run: --arm=engine --curves=full --steps=20000 -> %s\n' "$OUT/b2_engine_full.log"
  printf 'would run: --arm=frames --stride=400              -> %s\n' "$OUT/b2_frames.log"
  exit 0
fi

guard b2_engine_full.log
guard b2_frames.log

run() {   # run <marker> <logfile> <command...>
  local marker="$OUT/$1.DONE" log="$OUT/$2"; shift 2
  rm -f "$marker"
  ( cd "$WT" && taskset -c "$CORE" "$@" >"$log" 2>&1; echo "exit=$?" >"$marker" ) &
}

run b2_engine_full b2_engine_full.log "$BIN" --arm=engine --curves=full --steps=20000
run b2_frames      b2_frames.log      "$BIN" --arm=frames --stride=400
wait
