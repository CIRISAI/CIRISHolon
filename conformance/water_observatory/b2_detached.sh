#!/usr/bin/env bash
# B2's long runs. setsid + done-markers, so session death kills narration and not compute.
# Pinned to an E-core: M-PLACEMENT-LOTTERY's remedy is quiet-and-pinned, and an E-core has
# no SMT sibling whose load nobody controls.
set -u
# WT defaults to this repo's own root; the lane's worktree was a copy of this
# tree and its path dies with the session (gate 10a3). Override with B2_WT.
WT=${B2_WT:-$(git rev-parse --show-toplevel)}
OUT=$WT/conformance/water_observatory
BIN=$WT/engine/target/release/examples/b2_longrange
CORE=${CORE:-24}

run() {   # run <marker> <logfile> <command...>
  local marker="$OUT/$1.DONE" log="$OUT/$2"; shift 2
  rm -f "$marker"
  ( cd "$WT" && taskset -c "$CORE" "$@" >"$log" 2>&1; echo "exit=$?" >"$marker" ) &
}

run b2_engine_full b2_engine_full.log "$BIN" --arm=engine --curves=full --steps=20000
run b2_frames      b2_frames.log      "$BIN" --arm=frames --stride=400
wait
