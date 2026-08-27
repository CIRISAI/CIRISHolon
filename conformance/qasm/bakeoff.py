#!/usr/bin/env python3
"""The quiet-machine bake-off: holon's flat-sampler Clifford path vs stim,
same circuits, engine-only timing both sides, medians of 5. Runs anywhere;
built for CI runners (no background load). Sized by n, never by clock."""
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
    print(f"bake-off: holon flat-sampler vs stim {stim.__version__}, "
          f"medians of {REPS}, engine-only timing both sides")
    for n in SIZES:
        ops = gen_random(n, 20 * n, CLIFFORD_ALPHABET, seed=1000 + n)
        measured = list(range(n))
        path = os.path.join(HERE, f"bakeoff_{n}.qasm")
        with open(path, "w") as f:
            f.write(to_qasm(n, ops, measured))
        o = statistics.median(ours(path)["seconds"] for _ in range(REPS))
        s = statistics.median(stim_time(n, ops, measured) for _ in range(REPS))
        rows.append({"n": n, "ours_ms": o * 1e3, "stim_ms": s * 1e3, "ratio": o / s})
        print(f"n={n:5d}  ours {o*1e3:10.3f} ms   stim {s*1e3:10.3f} ms   ours/stim {o/s:6.3f}")
        os.remove(path)
    out = os.path.join(HERE, "bakeoff_results.json")
    json.dump({"stim_version": stim.__version__, "reps": REPS, "rows": rows},
              open(out, "w"), indent=1)
    print(f"wrote {out}")
    lead = sum(1 for r in rows if r["ratio"] < 1.0)
    print(f"VERDICT: ahead at {lead}/{len(rows)} sizes; worst ratio "
          f"{max(r['ratio'] for r in rows):.3f}, best {min(r['ratio'] for r in rows):.3f}")

if __name__ == "__main__":
    main()
