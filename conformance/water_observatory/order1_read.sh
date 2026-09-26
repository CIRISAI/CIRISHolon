#!/bin/bash
# ORDER-1 read (ORDER1_PREREG.md, frozen e53a803): the SLOW-1 walks, read-only, cores 16-20
cd "$(dirname "$0")"
S="$(cd "$(dirname "$0")" && pwd)/replace0/slow1"
export OPENBLAS_NUM_THREADS=5 OMP_NUM_THREADS=5
taskset -c 16-20 python3 order1_search.py read --t293 $S/T293_seed0 $S/T293_seed1 $S/T293_seed2 --t400 $S/T400_seed0
