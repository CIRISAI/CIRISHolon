#!/usr/bin/env bash
# Node LG's campaign, detached. Session death must only kill narration, never computation.
set -u
# ROOT resolves from this script's own location (conformance/mesh -> two up);
# the lane's worktree path died with its session (gate 10a3). LG_ROOT overrides
# for a re-run in a different checkout. Refusals are discriminated: exit 4 =
# not a checkout shape, exit 3 = campaign binary not built.
ROOT=${LG_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}
OUT=$ROOT/conformance/mesh
if [ ! -d "$ROOT/engine" ] || [ ! -d "$ROOT/conformance/mesh" ]; then
  echo "refuse: ROOT=$ROOT is not a CIRISHolon checkout (engine/ and conformance/mesh/ required); set LG_ROOT" >&2
  exit 4
fi
if [ ! -x "$ROOT/engine/target/release/lg_run" ]; then
  echo "refuse: campaign binary not built at $ROOT/engine/target/release/lg_run; run: cargo build --release -p holon-lattice --bin lg_run" >&2
  exit 3
fi
if [ "${LG_DRY:-0}" = "1" ]; then
  echo "ROOT=$ROOT"; echo "OUT=$OUT"; echo "BIN=$ROOT/engine/target/release/lg_run"; exit 0
fi
cd $ROOT/engine || exit 1
rm -f $OUT/lg_full.DONE $OUT/lg_tests.DONE
{
  echo "instrument commit: $(git -C $ROOT rev-parse HEAD)"
  echo "instrument tree state: $(git -C $ROOT status --porcelain -- engine/crates/holon-lattice | wc -l) uncommitted paths under holon-lattice"
  echo "rustc: $(rustc --version)"
  echo "started: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
} > $OUT/lg_full.log
./target/release/lg_run >> $OUT/lg_full.log 2>&1
echo "exit=$? finished: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> $OUT/lg_full.log
touch $OUT/lg_full.DONE
cargo test -p holon-lattice --release -- --nocapture > $OUT/lg_tests.log 2>&1
echo "exit=$?" >> $OUT/lg_tests.log
touch $OUT/lg_tests.DONE
