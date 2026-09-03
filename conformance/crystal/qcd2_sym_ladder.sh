#!/usr/bin/env bash
# GF2a amendment A1 §A1.3 (G0′): the symmetric arm at N = 8, both x, all sectors, the warm
# χ-ladder 32 → 64 → 128 → 256 in one process; one JSON per point; resumable.
BIN=../../engine/target/release/examples/qcd2_dmrg
jobs=()
for x in 4.0 9.0; do for b in 0 1 2; do jobs+=("$x 8 $b"); done; done
printf '%s\n' "${jobs[@]}" | xargs -P "${QCD2_WORKERS:-6}" -L 1 bash -c '
  set -- $0 $@; x=$1; n=$2; b=$3
  out=qcd2_sym/x${x}_N${n}_B${b}.json
  [ -s "$out" ] && { echo "$out [checkpoint]"; exit 0; }
  Q8_THREADS='"${Q8_THREADS:-4}"' '"$BIN"' --n $n --x $x --b $b --chi 32,64,128,256 --sweeps 60 --sym > "$out.tmp" && mv "$out.tmp" "$out" && echo "$out $(cat "$out")" || echo "FAILED $x $n $b"
'
echo LADDER_DONE
