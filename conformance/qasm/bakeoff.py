#!/usr/bin/env python3
"""The quiet-machine bake-off: holon's flat-sampler Clifford path vs stim,
same circuits, engine-only timing both sides, medians of 5 per circuit, and —
since BAKEOFF_PREREG.md (2026-09-19) — K independent circuits per size, so the
verdict is on the WORST circuit and the spread is printed beside the lead.
Runs anywhere; built for CI runners (no background load). Sized by n, never
by clock. A local run prints its load average and says so."""
import json, os, statistics, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from battlerig import gen_random, CLIFFORD_ALPHABET, to_qasm, to_stim  # noqa: E402

HOLON_RUN = os.environ.get(
    "HOLON_RUN",
    os.path.join(HERE, "..", "..", "engine", "target", "release", "holon-run"),
)
SIZES = [int(x) for x in os.environ.get("BAKEOFF_SIZES", "64,128,256,512,1024,2048,4096").split(",")]
REPS = int(os.environ.get("BAKEOFF_REPS", "5"))
SEEDS = int(os.environ.get("BAKEOFF_CIRCUIT_SEEDS", "5"))   # BAKEOFF_PREREG.md: K = 5

def ours(path):
    out = subprocess.run([HOLON_RUN, "clifford-sample", path],
                         capture_output=True, text=True, check=True)
    return json.loads(out.stdout)

def stim_time(n, ops, measured):
    import stim
    circ = to_stim(n, ops, measured)          # construction: NOT timed
    sim = stim.TableauSimulator()
    sim.set_num_qubits(n)
    t0 = time.perf_counter()
    sim.do(circ)
    return time.perf_counter() - t0

def main():
    import stim
    rows = []
    loadavg = os.getloadavg()
    print(f"bake-off: holon flat-sampler vs stim {stim.__version__}, "
          f"{SEEDS} circuits per size, medians of {REPS} per circuit, engine-only timing both sides; "
          f"load average {loadavg[0]:.2f} (a CI runner is ~0; a loaded host is not the citable venue)")
    for n in SIZES:
        per = []
        for k in range(SEEDS):
            seed = 1000 + n + 100 * k                 # k = 0 is the 2026-08-27 circuit
            ops = gen_random(n, 20 * n, CLIFFORD_ALPHABET, seed=seed)
            measured = list(range(n))
            path = os.path.join(HERE, f"bakeoff_{n}_{k}.qasm")
            with open(path, "w") as f:
                f.write(to_qasm(n, ops, measured))
            o = statistics.median(ours(path)["seconds"] for _ in range(REPS))
            s = statistics.median(stim_time(n, ops, measured) for _ in range(REPS))
            per.append({"seed": seed, "ours_ms": o * 1e3, "stim_ms": s * 1e3, "ratio": o / s})
            os.remove(path)
        ratios = [p["ratio"] for p in per]
        mean, worst, spread = statistics.mean(ratios), max(ratios), max(ratios) - min(ratios)
        s1 = worst < 1.0
        s2 = spread < (1.0 - mean)
        rows.append({"n": n, "circuits": per, "ratio_mean": mean, "ratio_worst": worst,
                     "ratio_spread": spread, "S1_ahead_on_worst": s1, "S2_spread_under_lead": s2})
        print(f"n={n:5d}  ours/stim mean {mean:6.3f}  worst {worst:6.3f}  spread {spread:6.3f}  "
              f"S1 {'ahead' if s1 else 'NOT ahead'}  S2 {'separates' if s2 else 'RANKS, does not separate'}")
    out = os.path.join(HERE, "bakeoff_results.json")
    json.dump({"stim_version": stim.__version__, "reps": REPS, "circuit_seeds": SEEDS,
               "loadavg": list(loadavg), "prereg": "BAKEOFF_PREREG.md", "rows": rows},
              open(out, "w"), indent=1)
    print(f"wrote {out}")
    s1 = sum(1 for r in rows if r["S1_ahead_on_worst"])
    s1m = sum(1 for r in rows if r["ratio_mean"] < 1.0)
    s2 = sum(1 for r in rows if r["S2_spread_under_lead"])
    print(f"VERDICT (BAKEOFF_PREREG.md): S1 ahead on the WORST circuit at {s1}/{len(rows)} sizes "
          f"(on the mean at {s1m}/{len(rows)}); S2 spread under the lead at {s2}/{len(rows)}; "
          f"worst ratio anywhere {max(r['ratio_worst'] for r in rows):.3f}")

if __name__ == "__main__":
    main()
