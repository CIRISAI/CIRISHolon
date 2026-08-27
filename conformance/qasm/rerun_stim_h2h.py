# Rerun of the packed-vs-stim head-to-head with the TRANSPOSED column engine.
# Same methodology as the banked third entry (BENCHMARKS.md): rig-generated
# CLIFFORD circuits n in {64,256,1024} depth 20n, stim timed per-call on the
# engine only (construction excluded), ours timed by holon-run's own in-binary
# clock (parse excluded), medians of 3.
import json, subprocess, sys, time
sys.path.insert(0, ".")
from battlerig import gen_random, CLIFFORD_ALPHABET, to_qasm, to_stim

HOLON_RUN = "/home/emoore/CIRISHolon/engine/target/release/holon-run"

def ours(qasm_path):
    out = subprocess.run([HOLON_RUN, "clifford-sample", qasm_path],
                         capture_output=True, text=True, check=True)
    return json.loads(out.stdout)["seconds"]

def stim_time(n, ops, measured):
    import stim
    circ = to_stim(n, ops, measured)
    sim = stim.TableauSimulator()
    sim.set_num_qubits(n)
    t0 = time.perf_counter()
    sim.do(circ)
    return time.perf_counter() - t0

results = []
for n in (64, 256, 1024):
    ops = gen_random(n, 20 * n, CLIFFORD_ALPHABET, seed=1000 + n)
    measured = list(range(n))
    path = f"h2h_cliff_{n}.qasm"
    open(path, "w").write(to_qasm(n, ops, measured))
    o = sorted(ours(path) for _ in range(3))[1]
    s = sorted(stim_time(n, ops, measured) for _ in range(3))[1]
    results.append((n, o * 1e3, s * 1e3, o / s))
    print(f"n={n:5d}  ours(col) {o*1e3:9.3f} ms   stim {s*1e3:9.3f} ms   ratio {o/s:6.2f}x")
json.dump(results, open("h2h_col_results.json", "w"))
