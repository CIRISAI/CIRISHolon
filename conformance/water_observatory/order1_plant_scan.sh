#!/bin/bash
# ORDER-1 POST-FREEZE sensitivity scan of PO-4's amplitude on the carrier of record (NOT a plant of record)
cd "$(dirname "$0")"
export OPENBLAS_NUM_THREADS=5 OMP_NUM_THREADS=5
for A in 0.5 0.7 0.85 1.0 1.5; do
  ORDER1_PLANT_AMP=$A taskset -c 16-20 python3 order1_plant_scan.py "$(cd "$(dirname "$0")" && pwd)"/replace0/slow1/T293_seed0
done
