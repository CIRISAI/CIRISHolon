#!/bin/bash
# MIXTURES-1's staked pairs, cheapest first, queued behind gate R1.
#
# CHEAPEST FIRST IS NOT IMPATIENCE.  Five of the eight pairs are minutes of
# compute; two are days; one is under measurement.  Running them in cost order
# means the campaign has real referee tables -- and therefore something the
# engine lane can be graded against -- long before the expensive tail lands, and
# it means any defect in the shared pipeline shows up on a pair that costs an
# hour to redo rather than on one that costs a week.
#
# PARALLELISM IS DELIBERATELY LOW.  ELEMENTS-1's three owed pairs are priority
# one on this machine and hold a 12-worker pool; four other campaigns share the
# same 32 cores.  Four workers here is a lane, not a land grab.
set -u
cd "$(dirname "$0")"
. ./env.sh
export NPROC_LIGHT=4 NPROC_HEAVY=3
log(){ echo "[$(date -Is)] $*"; }
step(){
  local m="$1"; shift
  if [ -f "markers/$m.DONE" ]; then log "SKIP $m"; return 0; fi
  log ">>> $m : curves2.py $*"
  if nice -n 10 python3 -u curves2.py "$@"; then
    touch "markers/$m.DONE"; log "<<< $m ok"
  else
    touch "markers/$m.FAILED"; log "<<< $m FAILED"; return 1
  fi
}
mkdir -p markers

log "waiting for gate R1 (mixtures_atoms.json)"
while [ ! -f mixtures_atoms.json ]; do sleep 60; done
log "R1 present; starting pairs"

# The five cheap pairs, together: their grids are large but every point is
# dominated by the AO integrals, so one pool over all of them keeps the workers
# fed better than five pools in sequence.
CHEAP="HCl NeAr ClF Ar2 Cl2"
step cheap_energies  --energies  $CHEAP || exit 1
step cheap_stencil   --stencil   $CHEAP || exit 1
step cheap_hermite   --hermite   $CHEAP || exit 1
step cheap_spin      --spin      $CHEAP || exit 1
step cheap_recertify --recertify $CHEAP || exit 1
step cheap_probe     --probe     $CHEAP || exit 1
step cheap_assemble  --assemble  $CHEAP || exit 1
log "CHEAP PAIRS COMPLETE"
touch markers/cheap.DONE

# The two heavy pairs that are known feasible.  SiO is NOT here: its cost is
# under measurement and the answer decides whether it is a grid or a campaign.
for sp in S2 NaH; do
  step ${sp}_energies  --energies  $sp || exit 1
  step ${sp}_stencil   --stencil   $sp || exit 1
  step ${sp}_hermite   --hermite   $sp || exit 1
  step ${sp}_spin      --spin      $sp || exit 1
  step ${sp}_recertify --recertify $sp || exit 1
  step ${sp}_probe     --probe     $sp || exit 1
  step ${sp}_assemble  --assemble  $sp || exit 1
  log "$sp COMPLETE"
  touch markers/$sp.DONE
done
log "SEVEN OF EIGHT PAIRS COMPLETE (SiO pending its feasibility call)"
touch markers/run_pairs.DONE
