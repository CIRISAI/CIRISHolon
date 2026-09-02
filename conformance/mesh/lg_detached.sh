#!/usr/bin/env bash
# Node LG's campaign, detached. Session death must only kill narration, never computation.
set -u
ROOT=/tmp/claude-1000/lg-wt
OUT=$ROOT/conformance/mesh
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
