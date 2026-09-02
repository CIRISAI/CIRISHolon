#!/bin/bash
# The BETWEEN-INVOCATION spread of the GPU sigma rate — the only spread a dispatch registry
# entry may be built from.
#
# WHY THIS EXISTS. `fci_bench`'s in-process block times five back-to-back loops inside one
# warm process and reports an sd of about 1.0 sigma/s (1.5%). That number describes a quiet
# moment, not the machine. A D12 spot-check runs whenever dispatch asks — on whatever the box
# is doing then — so an entry calibrated on the quiet moment convicts the machine and calls it
# the registration. That is M-IDLE-CALIBRATED-TIMEOUT one layer up from the reaper it was
# registered on, and saturation3-mesh surfaced it on 2026-09-01 from the other side: their six
# separate invocations found the round-trip block BIMODAL (14.9 ms against 81-90 ms, 6.03x)
# with the slow mode's excess 132x the measured PCIe cost — a descheduled host thread, not a
# device effect.
#
# Each invocation is a separate process, a separate CUDA context, and whatever clock and
# scheduling state the machine happens to be in. That is the regime the spot-check runs in,
# so that is the regime the spread has to come from.
#
# BOTH RATES, because the registry holds the wrong quantity if it holds only the kernel one.
# Every application in the production Davidson is `SigmaOp::apply` -> htod + sigma + dtoh +
# synchronize, so what a CALLER experiences is the round trip. The kernel figure is
# device-internal and overstates it. They are recorded together because the contrast is the
# finding: on this instrument the kernel block is stable across invocations and the round-trip
# block is not.
#
# Usage: spread_runs.sh [N] [core] [label]
set -u
N=${1:-12}
CORE=${2:-0}
LABEL=${3:-spread_runs}

# RESOLVED FROM THE SCRIPT'S OWN LOCATION, once (gate 10a3's rule, and the DRY half of it).
# This file previously wrote the repo root three times as an absolute path. That passes the
# gate's grep -- it is machine-hardcoded, not session-keyed -- and fails the rule: an
# instrument that only runs from one checkout on one box is an instrument that cannot be
# re-run by the person auditing it. HERE is where this file is; everything else hangs off it.
HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$HERE/../../.." && pwd)
BIN="$ROOT/engine/crates/holon-gpu/target/release/examples/fci_bench"
OUT="$HERE/${LABEL}.txt"

# DISCRIMINATED REFUSALS: three different things can be wrong and they need three different
# messages, because "it did not run" sends the reader looking in the wrong place. A fence is
# a bug under repair, never content -- so each of these names what to DO, not just what failed.
if [ ! -d "$ROOT/.git" ]; then
  echo "REFUSED: resolved repo root '$ROOT' has no .git. This script locates itself from" >&2
  echo "         \$BASH_SOURCE and expects to live at conformance/atomworld/gpu_fci/." >&2
  echo "         If it has been moved, fix the ../../.. above rather than hardcoding a path." >&2
  exit 2
fi
if [ ! -x "$BIN" ]; then
  echo "REFUSED: $BIN is missing or not executable." >&2
  echo "         Build it first:  cd $ROOT/engine/crates/holon-gpu &&" >&2
  echo "                          cargo build --release --example fci_bench" >&2
  echo "         Not built here on purpose: a benchmark that silently rebuilds its own" >&2
  echo "         subject cannot report the binary sha256 its header claims to pin." >&2
  exit 3
fi
if ! command -v nvidia-smi >/dev/null 2>&1; then
  echo "REFUSED: no nvidia-smi, so there is no device to measure. This instrument needs" >&2
  echo "         a CUDA card; it does not fall back to a host arm, because a host number" >&2
  echo "         under a GPU label is worse than no number (D4, and D0's whole point)." >&2
  exit 4
fi

{
  echo "# between-invocation spread of the (O,O,O) GPU sigma, kernel-only, warm"
  echo "# started_utc   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "# binary_sha256 $(sha256sum "$BIN" | cut -d' ' -f1)"
  echo "# repo_HEAD     $(cd "$ROOT" && git rev-parse HEAD)"
  echo "# invocations   $N, each a separate process on cpu $CORE"
  echo "# loadavg_start $(cut -d' ' -f1 /proc/loadavg)"
} > "$OUT"

for i in $(seq 1 "$N"); do
  line=$(taskset -c "$CORE" "$BIN" --species O,O,O --core-type P --rate-only 2>/dev/null \
         | grep '^RATE ')
  k=$(echo "$line" | awk '{print $3}')
  rt=$(echo "$line" | awk '{print $5}')
  la=$(cut -d' ' -f1 /proc/loadavg)
  # A failed invocation is recorded as a failure, never skipped: dropping the runs that went
  # wrong is how a spread comes back narrower than the machine.
  echo "${k:-FAILED} ${rt:-FAILED} $la" >> "$OUT"
done

awk '
  !/^#/ && $1 != "FAILED" { n++; k[n]=$1; r[n]=$2; sk+=$1; sr+=$2 }
  $1 == "FAILED" { f++ }
  END {
    if (n < 2) { print "# too few successful invocations to state a spread"; exit }
    mk = sk/n; mr = sr/n
    for (i=1;i<=n;i++) { dk=k[i]-mk; vk+=dk*dk; dr=r[i]-mr; vr+=dr*dr }
    sdk = sqrt(vk/(n-1)); sdr = sqrt(vr/(n-1))
    lok=k[1]; hik=k[1]; lor=r[1]; hir=r[1]
    for (i=1;i<=n;i++) {
      if (k[i]<lok) lok=k[i]; if (k[i]>hik) hik=k[i]
      if (r[i]<lor) lor=r[i]; if (r[i]>hir) hir=r[i]
    }
    printf "#\n# BETWEEN-INVOCATION, n %d, failed %d\n", n, f+0
    printf "#   kernel-only  mean %.3f  sd %.3f (%.2f%%)  min %.3f  max %.3f  spread %.3fx\n",
           mk, sdk, 100*sdk/mk, lok, hik, hik/lok
    printf "#   ROUND TRIP   mean %.3f  sd %.3f (%.2f%%)  min %.3f  max %.3f  spread %.3fx\n",
           mr, sdr, 100*sdr/mr, lor, hir, hir/lor
    printf "#   the ROUND TRIP is the caller-relevant quantity and is what the registry holds\n"
  }' "$OUT" >> "$OUT"

tail -6 "$OUT"
