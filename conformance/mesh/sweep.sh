#!/usr/bin/env bash
cd /tmp/claude-1000/lg-wt/conformance/mesh
rm -f invariant_sweep.DONE
python3 ref_invariants_sweep.py > invariant_sweep.log 2>&1
echo "exit=$?" >> invariant_sweep.log
touch invariant_sweep.DONE
