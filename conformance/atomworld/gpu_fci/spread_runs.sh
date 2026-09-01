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
# Usage: spread_runs.sh [N] [core]
set -u
N=${1:-12}
CORE=${2:-0}
BIN=/home/emoore/CIRISHolon/engine/crates/holon-gpu/target/release/examples/fci_bench
OUT=/home/emoore/CIRISHolon/conformance/atomworld/gpu_fci/spread_runs.txt

{
  echo "# between-invocation spread of the (O,O,O) GPU sigma, kernel-only, warm"
  echo "# started_utc   $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "# binary_sha256 $(sha256sum "$BIN" | cut -d' ' -f1)"
  echo "# repo_HEAD     $(cd /home/emoore/CIRISHolon && git rev-parse HEAD)"
  echo "# invocations   $N, each a separate process on cpu $CORE"
  echo "# loadavg_start $(cut -d' ' -f1 /proc/loadavg)"
} > "$OUT"

for i in $(seq 1 "$N"); do
  r=$(taskset -c "$CORE" "$BIN" --species O,O,O --core-type P --rate-only 2>/dev/null \
      | grep '^RATE ' | awk '{print $2}')
  la=$(cut -d' ' -f1 /proc/loadavg)
  # A failed invocation is recorded as a failure, never skipped: dropping the runs that went
  # wrong is how a spread comes back narrower than the machine.
  echo "${r:-FAILED} $la" >> "$OUT"
done

awk '
  !/^#/ && $1 != "FAILED" { n++; x[n]=$1; s+=$1 }
  $1 == "FAILED" { f++ }
  END {
    if (n < 2) { print "# too few successful invocations to state a spread"; exit }
    m = s/n
    for (i=1;i<=n;i++) { d=x[i]-m; v+=d*d }
    sd = sqrt(v/(n-1))
    lo=x[1]; hi=x[1]
    for (i=1;i<=n;i++) { if (x[i]<lo) lo=x[i]; if (x[i]>hi) hi=x[i] }
    printf "#\n# BETWEEN-INVOCATION: n %d, failed %d, mean %.3f, sd %.3f (%.2f%%), min %.3f, max %.3f, spread %.3fx\n",
           n, f+0, m, sd, 100*sd/m, lo, hi, hi/lo
  }' "$OUT" >> "$OUT"

tail -3 "$OUT"
