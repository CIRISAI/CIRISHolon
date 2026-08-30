#!/bin/bash
# The three owed ELEMENTS-1 pairs, plus the two landed species whose Hermite
# probe records are not in the cache (so the drop is re-derivable, not trusted).
#
# Sequential by design: every stage after --energies reads what the one before
# it wrote, and a pool that starts before its inputs exist does not fail, it
# quietly computes fewer points.  Markers not narration: session death must kill
# only the narration.
set -u
cd "$(dirname "$0")"
export NPROC_LIGHT=12 NPROC_HEAVY=12
# ONE BLAS THREAD PER WORKER.  Unset, scipy's bundled OpenBLAS builds a pool of
# 63 threads IN EVERY WORKER -- 756 threads for a 12-worker pool -- and they
# spin-wait.  Measured on this pool: 148% CPU per worker, i.e. half a core each
# of pure spin, six cores of this machine burned on nothing, and a load average
# of 83 that every other campaign on the box was paying for too.  The work here
# is mpmath-bound and the only BLAS call in the hot path is a sparse matvec,
# which is single-threaded regardless.  These must be set BEFORE numpy is
# imported, which is why they are here and not in Python.
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export NUMEXPR_NUM_THREADS=1 VECLIB_MAXIMUM_THREADS=1
log(){ echo "[$(date -Is)] $*"; }
step(){                      # step <marker> <args...>
  local m="$1"; shift
  if [ -f "$m.DONE" ]; then log "SKIP $m (done)"; return 0; fi
  log ">>> $m : build_curves.py $*"
  if nice -n 5 python3 -u build_curves.py "$@"; then
    touch "$m.DONE"; log "<<< $m ok"
  else
    touch "$m.FAILED"; log "<<< $m FAILED rc=$?"; return 1
  fi
}

step owed_hermite_landed --hermite F2 Ne2          || exit 1
step owed_li2_energies   --energies Li2            || exit 1
step owed_stencils       --stencil Li2 N2          || exit 1
step owed_hermite_heavy  --hermite Li2 N2          || exit 1
step owed_spin           --spin Li2 N2             || exit 1
step owed_recertify      --recertify Li2 N2        || exit 1
step owed_probe          --probe Li2 N2            || exit 1
step owed_assemble       --assemble Li2 N2         || exit 1
log "ALL OWED STAGES (Li2, N2) COMPLETE"
touch run_owed.DONE
