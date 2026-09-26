#!/bin/bash
# SLOW-3 read (SLOW3_PREREG.md): run ONLY when both fresh arms have run.done = 0. Builds the fresh seeds' feature
# caches (slow2_compare.py features, cores 18-20, one thread each), links T400 seed 0 (control), then loads the
# FROZEN models (slow3/frozen, checked against MANIFEST.sha256 before use) and reads. Trains nothing.
# T293_seed2 is never linked into this cache.
set -u
W="$(cd "$(dirname "$0")" && pwd)"; cd "$W"
PY="${SLOW3_PY:-python3}"   # the read needs torch + deeptime on this interpreter (a venv is fine; pass it in SLOW3_PY)
C="${SLOW3_CACHE:-$(cd "$(dirname "$0")" && pwd)/slow3/cache}"
OBS="$(cd "$(dirname "$0")" && pwd)/replace0"
for s in 3 4; do [ "$(cat $OBS/slow3/T293_seed$s/run.done 2>/dev/null)" = "0" ] || { echo "slow3_read: T293_seed$s has no run.done = 0; not reading"; exit 2; }; done
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1 CUBLAS_WORKSPACE_CONFIG=:4096:8
mkdir -p "$C"
taskset -c 18 "$PY" slow2_compare.py features --walk $OBS/slow3/T293_seed3 --out "$C/T293_seed3.npz" > slow3/features_seed3.log 2>&1 &
taskset -c 19 "$PY" slow2_compare.py features --walk $OBS/slow3/T293_seed4 --out "$C/T293_seed4.npz" > slow3/features_seed4.log 2>&1 &
[ -f "$C/T400_seed0.npz" ] || taskset -c 20 "$PY" slow2_compare.py features --walk $OBS/slow1/T400_seed0 --out "$C/T400_seed0.npz" > slow3/features_T400.log 2>&1 &
wait
cat $OBS/slow3/provenance_T293_seed0/provenance.txt > slow3/provenance.txt 2>/dev/null
export OMP_NUM_THREADS=3 OPENBLAS_NUM_THREADS=3 MKL_NUM_THREADS=3
taskset -c 18-20 "$PY" slow3_train.py read --cache "$C" --frozen slow3/frozen --out slow3/slow3_read.json > slow3/slow3_read.txt 2> slow3/slow3_read.err
echo "READ EXIT $?" >> slow3/slow3_read.txt; tail -12 slow3/slow3_read.txt
