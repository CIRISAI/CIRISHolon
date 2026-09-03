#!/bin/bash
# A2's N = 8 EXAM (G0'', V1, plants vi/vii/viii), every run checkpointed and resumable:
# re-invoking this script resumes whatever was interrupted and skips whatever completed.
#   QCD2_WORKERS=4 Q8_THREADS=4 bash exam.sh
set -u
cd "$(dirname "$0")"
DEV=../../../engine/crates/holon-gpu/target/release/examples/qcd2_sym_device
HOST=../../../engine/target/release/examples/qcd2_dmrg
mkdir -p ckpt rows
jobs=()
# G0'' + V1: the mixed warm ladder, both x, all sectors (device)
for x in 4.0 9.0; do for b in 0 1 2; do
  jobs+=("mixed_x${x}_B${b} $DEV --n 8 --x $x --b $b --chi 64,128,256 --sweeps 60 --mix 1e-4 --variance --ckpt ckpt/mixed")
done; done
# G0'': the cold chi=256 start, both x, all sectors (device)
for x in 4.0 9.0; do for b in 0 1 2; do
  jobs+=("cold_x${x}_B${b} $DEV --n 8 --x $x --b $b --chi 256 --sweeps 60 --variance --ckpt ckpt/cold")
done; done
# plant (vi): the same warm ladder with mixing OFF on the sector that fired (device)
jobs+=("unmixed_x4.0_B1 $DEV --n 8 --x 4.0 --b 1 --chi 64,128,256 --sweeps 60 --variance --ckpt ckpt/unmixed")
# plant (vii): the labels-ignored mutant at chi=256, cold (host: the mutant has no plan)
jobs+=("mutant_x4.0_B1 $HOST --n 8 --x 4.0 --b 1 --chi 256 --sweeps 60 --mutant --variance --ckpt ckpt/mutant")
printf '%s\n' "${jobs[@]}" | xargs -P "${QCD2_WORKERS:-4}" -L 1 bash -c '
  set -- $0 "$@"; name=$1; shift
  out=rows/$name.json
  [ -s "$out" ] && { echo "$name [done]"; exit 0; }
  "$@" > "$out.tmp" 2> "rows/$name.err" && mv "$out.tmp" "$out" && echo "$name $(head -c 160 "$out")" || echo "FAILED $name (see rows/$name.err)"
'
echo EXAM_DONE
