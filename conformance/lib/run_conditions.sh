#!/usr/bin/env bash
# A RECORD WITHOUT ITS CONDITIONS IS NOT A RECORD.
#
# Fleet standard, adopted 2026-09-01 by the lead's ruling, from the joint bigqvm/mesher
# law that closed M-IDLE-CALIBRATED-TIMEOUT. Every detached run stamps its conditions at
# BOTH ENDS of the run, so that any number which later serves as a BASELINE carries the
# regime it was measured in, and a future ratio against a stored record can check that
# record's regime before believing itself.
#
# WHY BOTH ENDS, and why this is not a nicety: an entire finding was manufactured on this
# box out of baselines taken under load. Three ratios (2.37x, 1.73x, 2.54x) were each
# wrong by a DIFFERENT factor, and their ratio (2.37/1.73 = 1.37) was reported as a
# property of two probes. It was a property of two wrong baselines. Nobody could tell,
# because no record carried its conditions.
#
# WHY THE CLOCK AND NOT ONLY THE LOAD (mesher's): a run whose clock moved 54% -> 52%
# inside a single 30-second window shows NOTHING in any loadavg number. Load and clock
# are different variables and a run stamped with only one is stamped with the wrong one.
#
# WHY THE CORE CLASS (M-PLACEMENT-LOTTERY, and M-DEVICE-CLASS one level down): a citable
# ratio is a function of core class, so an undeclared placement is an undeclared variable.
# P and E cores are not the same machine. P-cores are discriminated from E-cores by
# whether they HAVE an SMT sibling, derived here rather than hardcoded, because a
# hardcoded 0-15 is a fact about one box.
#
# USAGE
#   source conformance/lib/run_conditions.sh
#   run_conditions "at launch"     # in the header, before the work
#   run_conditions "at exit"       # in the detached wrapper, after the binary returns
#
# Every line it prints is [MEASURED]. It infers nothing.

run_conditions() {
    local when="${1:-unspecified}"
    local la p_list e_list mine cur max frac cpu

    la=$(cut -d' ' -f1-3 /proc/loadavg)

    # Derive P vs E from SMT siblings rather than assuming this box's numbering.
    p_list=""; e_list=""
    for cpu in /sys/devices/system/cpu/cpu[0-9]*; do
        local n sib
        n=${cpu##*/cpu}
        sib=$(cat "$cpu/topology/thread_siblings_list" 2>/dev/null) || continue
        if [[ "$sib" == *[,-]* ]]; then p_list="$p_list $n"; else e_list="$e_list $n"; fi
    done

    # Trim: the accumulators above start with a leading space.
    p_list="${p_list# }"; e_list="${e_list# }"
    local p_first="${p_list%% *}" e_first="${e_list%% *}"

    # Clock as a FRACTION OF ADVERTISED, sampled on one core of each class.
    _clock_frac() {
        local n=$1 cur max
        [ -n "$n" ] || { echo "n/a (no core of this class)"; return; }
        cur=$(cat "/sys/devices/system/cpu/cpu$n/cpufreq/scaling_cur_freq" 2>/dev/null) || { echo "n/a"; return; }
        max=$(cat "/sys/devices/system/cpu/cpu$n/cpufreq/cpuinfo_max_freq" 2>/dev/null) || { echo "n/a"; return; }
        [ "${max:-0}" -gt 0 ] 2>/dev/null || { echo "n/a"; return; }
        awk -v c="$cur" -v m="$max" -v n="$n" \
            'BEGIN{printf "%3.0f%% of advertised  (cpu%s at %.2f of %.2f GHz)", 100*c/m, n, c/1e6, m/1e6}'
    }

    # This process's actual affinity, not the machine's capacity.
    mine=$(taskset -pc $$ 2>/dev/null | sed 's/.*: //') || mine="unknown"

    echo "--- run conditions, $when [ALL MEASURED] ---"
    echo "  loadavg           $la"
    echo "  cores             $(echo $p_list | wc -w) SMT (P-class), $(echo $e_list | wc -w) non-SMT (E-class)"
    echo "  affinity          $mine"
    echo "                    ^ taskset RESTRICTS, it does not RESERVE: this names the cores"
    echo "                      allowed, never the cores free. Sibling load is a MEASURED"
    echo "                      COVARIATE, not a controlled one, without isolcpus or a cpuset."
    echo "  clock P-class     $(_clock_frac "$p_first")"
    echo "  clock E-class     $(_clock_frac "$e_first")"
    echo "--- end conditions, $when ---"
}
