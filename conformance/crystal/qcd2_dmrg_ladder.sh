#!/usr/bin/env bash
# GF2a DMRG ladder: one engine process per (x, N, B); the χ-ladder 40 → 64 runs INSIDE the
# process (warm start), so one JSON per point carries both rungs; resumable.
BIN=../../engine/target/release/examples/qcd2_dmrg
jobs=()
for x in 4.0 9.0; do
  if [ "$x" = "4.0" ]; then NS="8 16 24 40"; else NS="8 24 40 60"; fi
  for n in $NS; do for b in 0 1 2; do jobs+=("$x $n $b"); done; done
done
printf '%s\n' "${jobs[@]}" | xargs -P "${QCD2_WORKERS:-12}" -L 1 bash -c '
  set -- $0 $@; x=$1; n=$2; b=$3
  out=qcd2_dmrg/x${x}_N${n}_B${b}.json
  [ -s "$out" ] && { echo "$out [checkpoint]"; exit 0; }
  Q8_THREADS='"${Q8_THREADS:-2}"' '"$BIN"' --n $n --x $x --b $b --chi 40,64 --sweeps 120 --rtol 1e-9 > "$out.tmp" && mv "$out.tmp" "$out" && echo "$out $(cat "$out")" || echo "FAILED $x $n $b"
'
echo LADDER_DONE
