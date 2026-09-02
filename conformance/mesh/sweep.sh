#!/usr/bin/env bash
# Resolve from this script's own location; the lane worktree died with its
# session (gate 10a3). LG_ROOT overrides.
cd "${LG_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}/conformance/mesh" || exit 4
rm -f invariant_sweep.DONE
python3 ref_invariants_sweep.py > invariant_sweep.log 2>&1
echo "exit=$?" >> invariant_sweep.log
touch invariant_sweep.DONE
