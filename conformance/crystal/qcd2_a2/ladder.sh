#!/bin/bash
# A2.6's VOLUME LADDER: x=4 at N=16,24,40 and x=9 at N=24,40,60 (N=8 is the exam), all
# three sectors, two chi rungs per N from the cut's rank (lower cold, upper warm + mixing),
# variance per rung, every rung checkpointed per sweep. Re-invoking resumes. Launch ONLY
# after score.py prints ALL STAKED VERDICTS PASS (A2.9).
#   QCD2_WORKERS=2 Q8_THREADS=8 bash ladder.sh
set -u
cd "$(dirname "$0")"
DEV=../../../engine/crates/holon-gpu/target/release/examples/qcd2_sym_device
mkdir -p ckpt/ladder rows
chi_for() { case "$1" in 16) echo 256,512;; 24) echo 512,1024;; 40) echo 1024,2048;; 60) echo 1024,2048;; esac; }
jobs=()
for x in 4.0 9.0; do
  ns=$([ "$x" = "4.0" ] && echo "16 24 40" || echo "24 40 60")
  for n in $ns; do for b in 0 1 2; do
    jobs+=("ladder_x${x}_N${n}_B${b} $DEV --n $n --x $x --b $b --chi $(chi_for $n) --sweeps 60 --mix 1e-4 --variance --ckpt ckpt/ladder --reserve-mib 1024")
  done; done
done
printf '%s\n' "${jobs[@]}" | xargs -P "${QCD2_WORKERS:-2}" -L 1 bash -c '
  set -- $0 "$@"; name=$1; shift
  out=rows/$name.json
  [ -s "$out" ] && { echo "$name [done]"; exit 0; }
  "$@" > "$out.tmp" 2> "rows/$name.err" && mv "$out.tmp" "$out" && echo "$name $(head -c 160 "$out")" || echo "FAILED $name (see rows/$name.err)"
'
echo LADDER_DONE
