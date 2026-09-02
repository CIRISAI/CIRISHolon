#!/usr/bin/env bash
# GF2a DMRG ladder: one engine process per (x, N, B, chi); JSON per point; resumable.
BIN=../../engine/target/release/examples/qcd2_dmrg
jobs=()
for x in 4.0 9.0; do
  if [ "$x" = "4.0" ]; then NS="8 16 24 40"; else NS="8 24 40 60"; fi
  for n in $NS; do for b in 0 1 2; do for chi in 40 64; do
    jobs+=("$x $n $b $chi")
  done; done; done
done
printf '%s\n' "${jobs[@]}" | xargs -P "${QCD2_WORKERS:-16}" -L 1 bash -c '
  set -- $0 $@; x=$1; n=$2; b=$3; chi=$4
  out=qcd2_dmrg/x${x}_N${n}_B${b}_chi${chi}.json
  [ -s "$out" ] && { echo "$out [checkpoint]"; exit 0; }
  OMP_NUM_THREADS=1 '"$BIN"' --n $n --x $x --b $b --chi $chi --sweeps 120 --tol 1e-9 > "$out.tmp" && mv "$out.tmp" "$out" && echo "$out $(cat "$out")" || echo "FAILED $x $n $b $chi"
'
echo LADDER_DONE
