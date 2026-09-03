#!/bin/bash
# A3's G0''' rows: the three firing sectors (and B=1 at x=4 for completeness) to their
# rank-derived ceilings, mixed ladders plus the cold start at the ceiling. Resumable.
#   QCD2_WORKERS=2 Q8_THREADS=8 bash exam3.sh
set -u
cd "$(dirname "$0")"
DEV=../../../engine/crates/holon-gpu/target/release/examples/qcd2_sym_device
mkdir -p ckpt/a3mixed ckpt/a3cold rows
jobs=()
jobs+=("a3mixed_x4.0_B0 $DEV --n 8 --x 4.0 --b 0 --chi 64,128,256,512,1024 --sweeps 60 --mix 1e-4 --variance --ckpt ckpt/a3mixed --reserve-mib 1024")
jobs+=("a3mixed_x9.0_B0 $DEV --n 8 --x 9.0 --b 0 --chi 64,128,256,512,1024 --sweeps 60 --mix 1e-4 --variance --ckpt ckpt/a3mixed --reserve-mib 1024")
jobs+=("a3mixed_x9.0_B1 $DEV --n 8 --x 9.0 --b 1 --chi 64,128,256,512 --sweeps 60 --mix 1e-4 --variance --ckpt ckpt/a3mixed --reserve-mib 1024")
jobs+=("a3mixed_x4.0_B1 $DEV --n 8 --x 4.0 --b 1 --chi 64,128,256,512 --sweeps 60 --mix 1e-4 --variance --ckpt ckpt/a3mixed --reserve-mib 1024")
jobs+=("a3cold_x4.0_B0 $DEV --n 8 --x 4.0 --b 0 --chi 1024 --sweeps 60 --variance --ckpt ckpt/a3cold --reserve-mib 1024")
jobs+=("a3cold_x9.0_B0 $DEV --n 8 --x 9.0 --b 0 --chi 1024 --sweeps 60 --variance --ckpt ckpt/a3cold --reserve-mib 1024")
jobs+=("a3cold_x9.0_B1 $DEV --n 8 --x 9.0 --b 1 --chi 512 --sweeps 60 --variance --ckpt ckpt/a3cold --reserve-mib 1024")
jobs+=("a3cold_x4.0_B1 $DEV --n 8 --x 4.0 --b 1 --chi 512 --sweeps 60 --variance --ckpt ckpt/a3cold --reserve-mib 1024")
printf '%s\n' "${jobs[@]}" | xargs -P "${QCD2_WORKERS:-2}" -L 1 bash -c '
  set -- $0 "$@"; name=$1; shift
  out=rows/$name.json
  [ -s "$out" ] && { echo "$name [done]"; exit 0; }
  "$@" > "$out.tmp" 2> "rows/$name.err" && mv "$out.tmp" "$out" && echo "$name $(head -c 160 "$out")" || echo "FAILED $name (see rows/$name.err)"
'
echo EXAM3_DONE
