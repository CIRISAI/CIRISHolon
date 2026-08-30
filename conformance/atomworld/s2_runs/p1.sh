#!/bin/bash
# SATURATION-2 gate P1: the three arms, in the frozen order — controls first, then the
# mixed arm. One writer per output path, sequential, so there is no run-lock exposure
# within a run; the exposure that remains is relaunching this script while one is live.
set -u
B=/home/emoore/CIRISHolon/engine/target/release/examples/waterquench
D=/home/emoore/CIRISHolon/conformance/atomworld/s2_runs
for arm in hydrogen oxygen mixed; do
  nice -n 15 "$B" "$arm" > "$D/p1_$arm.log" 2>&1
  echo $? > "$D/p1_$arm.DONE"
done
echo done > "$D/p1_all.DONE"
