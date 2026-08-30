#!/bin/bash
# CO's tail, queued behind the Li2/N2 chain.
#
# CO is a SEPARATE script rather than more steps in run_owed.sh because bash
# reads a script incrementally from a byte offset as it executes it: editing the
# file a running shell is in the middle of makes it resume at the wrong place.
#
# CO's R = 9.000000000000 is the geometry that died on ArpackNoConvergence with
# no fallback, and it is the NATURAL demonstration of the f64 seed ladder --
# test_seed_ladder.py exercises the plumbing by forcing each rung to fail, but a
# rung that has only ever been fired by a monkeypatch is not yet known to fire
# on the case it was written for.  Whatever rung it lands on is recorded in that
# geometry's own `seed` field, so the artifact says which.
set -u
cd "$(dirname "$0")"
export NPROC_LIGHT=12 NPROC_HEAVY=12
export OMP_NUM_THREADS=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
export NUMEXPR_NUM_THREADS=1 VECLIB_MAXIMUM_THREADS=1
log(){ echo "[$(date -Is)] $*"; }
step(){
  local m="$1"; shift
  if [ -f "$m.DONE" ]; then log "SKIP $m (done)"; return 0; fi
  log ">>> $m : build_curves.py $*"
  if nice -n 5 python3 -u build_curves.py "$@"; then
    touch "$m.DONE"; log "<<< $m ok"
  else
    touch "$m.FAILED"; log "<<< $m FAILED"; return 1
  fi
}

log "waiting for the Li2/N2 chain"
while [ ! -f run_owed.DONE ]; do
  if ls owed_*.FAILED >/dev/null 2>&1; then
    log "the Li2/N2 chain FAILED; not starting CO"; exit 1
  fi
  sleep 60
done
log "chain done; starting CO"
step co_energies  --energies CO   || exit 1
step co_stencil   --stencil CO    || exit 1
step co_hermite   --hermite CO    || exit 1
step co_spin      --spin CO       || exit 1
step co_recertify --recertify CO  || exit 1
step co_probe     --probe CO      || exit 1
step co_assemble  --assemble CO   || exit 1
log "CO COMPLETE"
touch run_co.DONE
