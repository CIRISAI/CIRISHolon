#!/usr/bin/env python3
"""QASM-1 conformance harness: holon-qasm vs qiskit exact probabilities.

Strata: classical {x,cx,ccx} / clifford {x,z,h,s,sdg,cx} / magic (clifford+t,tdg).
Circuits measure every qubit q[i]->c[i], so holon's key order (c[n-1..0]) equals
qiskit's probabilities_dict order (q[n-1..0]). Comparison: max abs prob error.
Modes: gauge (planted mutants must FIRE) | staked <stratum> | bench."""
import json, random, subprocess, sys, time

BIN = "/home/emoore/CIRISHolon/engine/target/release/holon-qasm"
TMP = "/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/qasm/_cc.qasm"

GATES = {
    "classical": [("x",1),("cx",2),("ccx",3)],
    "clifford":  [("x",1),("z",1),("h",1),("s",1),("sdg",1),("cx",2)],
    "magic":     [("x",1),("z",1),("h",1),("s",1),("sdg",1),("cx",2),("t",1),("tdg",1)],
}

def gen(stratum, n, depth, rng):
    lines = ["OPENQASM 2.0;", 'include "qelib1.inc";', f"qreg q[{n}];", f"creg c[{n}];"]
    for _ in range(depth):
        g, k = rng.choice([gk for gk in GATES[stratum] if gk[1] <= n])
        qs = rng.sample(range(n), k)
        lines.append(f"{g} " + ",".join(f"q[{q}]" for q in qs) + ";")
    for i in range(n):
        lines.append(f"measure q[{i}] -> c[{i}];")
    return "\n".join(lines) + "\n"

def holon(src, mutate=None, tier=None):
    open(TMP, "w").write(src)
    cmd = [BIN, "run", TMP]
    if tier: cmd += ["--tier", tier]
    if mutate: cmd += ["--mutate", mutate]
    out = subprocess.run(cmd, capture_output=True, text=True)
    if out.returncode != 0:
        raise RuntimeError(out.stderr[:200])
    return json.loads(out.stdout)

def qiskit_ref(src):
    from qiskit import qasm2
    from qiskit.quantum_info import Statevector
    qc = qasm2.loads(src)
    qc.remove_final_measurements(inplace=True)
    return Statevector.from_instruction(qc).probabilities_dict(decimals=12)

def compare(src, mutate=None):
    h = holon(src, mutate=mutate)
    q = qiskit_ref(src)
    keys = set(h["dist"]) | set(q)
    err = max(abs(h["dist"].get(k, 0.0) - float(q.get(k, 0.0))) for k in keys)
    unit = abs(sum(h["dist"].values()) - 1.0)
    return err, h["tier"], unit

INV = {"x":"x","z":"z","h":"h","s":"sdg","sdg":"s","cx":"cx","ccx":"ccx"}

def gen_echo(stratum, n, depth, rng):
    """Gates then their inverse: every measurement deterministic (all zeros),
    so PHASE-level implementation errors are observable in the distribution
    (house rule: a planted defect must be observable; random-outcome
    distributions are blind to sign errors)."""
    body = []
    for _ in range(depth):
        g, k = rng.choice([gk for gk in GATES[stratum] if gk[1] <= n])
        body.append((g, rng.sample(range(n), k)))
    lines = ["OPENQASM 2.0;", 'include "qelib1.inc";', f"qreg q[{n}];", f"creg c[{n}];"]
    for g, qs in body:
        lines.append(f"{g} " + ",".join(f"q[{q}]" for q in qs) + ";")
    for g, qs in reversed(body):
        lines.append(f"{INV[g]} " + ",".join(f"q[{q}]" for q in qs) + ";")
    for i in range(n):
        lines.append(f"measure q[{i}] -> c[{i}];")
    return "\n".join(lines) + "\n"

def batch(stratum, count, seed, mutate=None, echo=False, shallow=False):
    rng = random.Random(seed)
    errs, tiers, units = [], set(), []
    for _ in range(count):
        if shallow:
            n, depth = rng.randint(2, 3), rng.randint(3, 10)
        else:
            n = rng.randint(2, 8 if stratum != "classical" else 10)
            depth = rng.randint(5, 60)
        src = (gen_echo if echo else gen)(stratum, n, depth, rng)
        if stratum == "magic" and not echo:
            while sum(l.split()[0] in ("t", "tdg") for l in src.splitlines()
                      if l and l[0] == "t") > 8:
                src = gen(stratum, n, depth, rng)
        e, t, u = compare(src, mutate=mutate)
        errs.append(e); tiers.add(t); units.append(u)
    return max(errs), sum(e > 1e-9 for e in errs), tiers, max(units)

if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "gauge":
        m, fired, tiers, _ = batch("clifford", 40, 111)
        print(f"PASS side (unmutated clifford, gauge seed): max_err={m:.2e} mismatches={fired}/40 tiers={tiers}")
        assert m <= 1e-9
        me, firede, _, _ = batch("clifford", 40, 111, echo=True)
        print(f"PASS side (unmutated ECHO clifford): max_err={me:.2e} mismatches={firede}/40")
        assert me <= 1e-9
        # RECORDED LESSONS. (1) Echo circuits CANNOT gauge these mutants --
        # a consistently-wrong Clifford passes its own inverse (0/40
        # measured): an echo tests self-consistency, not correctness.
        # (2) Phase-level mutants are visible only through generator patterns
        # most random circuits never produce (1-2/40) -- so the gauge PINS
        # firing witnesses by search, per the planted-defect-must-be-
        # observable rule, and asserts detection on the pinned witnesses.
        for mut, stratum in (("s-phase","clifford"), ("cx-phase","clifford"), ("cx-swap","classical")):
            rng = random.Random(4242)
            pinned = []
            for trial in range(2000):
                n, depth = rng.randint(2, 4), rng.randint(3, 14)
                src = gen(stratum, n, depth, rng)
                e, _ = compare(src, mutate=mut)
                if e > 0.3:
                    pinned.append((trial, e))
                    if len(pinned) == 3:
                        break
            print(f"FIRE side (planted {mut}): pinned witnesses (trial, err) = {pinned}"
                  f" -> {'FIRES' if len(pinned) == 3 else 'MISSED'}")
            assert len(pinned) == 3, f"{mut}: found only {len(pinned)} witnesses in 2000 trials"
        print("gauge verdict: unmutated PASSES exactly (incl. echo); each planted")
        print("mutant has three pinned firing witnesses. Two-sided, observability enforced.")
    elif mode == "staked":
        stratum, count, seed = sys.argv[2], int(sys.argv[3]), int(sys.argv[4])
        m, fired, tiers, unit = batch(stratum, count, seed)
        print(json.dumps({"stratum": stratum, "count": count, "seed": seed,
                          "max_err": m, "mismatches": fired, "tiers": sorted(tiers),
                          "max_unitarity_defect": unit}))
    elif mode == "bench":
        rng = random.Random(7)
        rows = []
        for n in (8, 16, 32, 64, 128, 256):
            src = gen("clifford", n, 20 * n, rng)
            open(TMP, "w").write(src)
            out = subprocess.run([BIN, "run", TMP, "--sample"], capture_output=True, text=True)
            h = json.loads(out.stdout)
            rows.append((n, h["seconds"]))
            print(f"tableau  n={n:4d} depth={20*n:5d}  {h['seconds']:.4f}s")
        import math
        xs = [math.log(n) for n, _ in rows[1:]]
        ys = [math.log(max(s, 1e-6)) for _, s in rows[1:]]
        nn = len(xs)
        slope = (nn * sum(x*y for x, y in zip(xs, ys)) - sum(xs)*sum(ys)) / (nn * sum(x*x for x in xs) - sum(xs)**2)
        print(f"tableau log-log slope = {slope:.2f} (poly)")
        rows2 = []
        for n in (12, 16, 20, 22, 24):
            src = gen("magic", n, 8 * n, rng)
            h = holon(src, tier="statevector")
            rows2.append((n, h["seconds"]))
            print(f"statevec n={n:4d} depth={8*n:5d}  {h['seconds']:.4f}s")
        xs2 = [n for n, _ in rows2[1:]]; ys2 = [math.log2(max(s, 1e-6)) for _, s in rows2[1:]]
        n2 = len(xs2)
        slope2 = (n2 * sum(x*y for x, y in zip(xs2, ys2)) - sum(xs2)*sum(ys2)) / (n2 * sum(x*x for x in xs2) - sum(xs2)**2)
        print(f"statevector log2(seconds) slope = {slope2:.3f} per qubit (exponential)")

def gen_fixed(n, t, cdepth, rng):
    """Random Clifford circuit with EXACTLY t T-gates at random positions."""
    body = []
    for _ in range(cdepth):
        g, k = rng.choice([gk for gk in GATES["clifford"] if gk[1] <= n])
        body.append(f"{g} " + ",".join(f"q[{q}]" for q in rng.sample(range(n), k)) + ";")
    for pos in sorted(rng.sample(range(len(body) + 1), min(t, len(body) + 1)), reverse=True):
        body.insert(pos, f"t q[{rng.randrange(n)}];")
    lines = ["OPENQASM 2.0;", 'include "qelib1.inc";', f"qreg q[{n}];", f"creg c[{n}];"] + body
    for i in range(n):
        lines.append(f"measure q[{i}] -> c[{i}];")
    return "\n".join(lines) + "\n"

def m2_m3():
    import math
    rng = random.Random(20260830)
    print("== M2: t-scaling at fixed n=10 (amplitude of |0^n>, 3 reps median) ==")
    rows = []
    for t in range(2, 13):
        times = []
        for _ in range(3):
            src = gen_fixed(10, t, 40, rng)
            open(TMP, "w").write(src)
            out = subprocess.run([BIN, "amp", TMP], capture_output=True, text=True)
            times.append(json.loads(out.stdout)["seconds"])
        times.sort()
        rows.append((t, times[1]))
        print(f"  t={t:2d}  {times[1]:.5f}s")
    xs = [t for t, _ in rows[3:]]
    ys = [math.log2(max(s, 1e-7)) for _, s in rows[3:]]
    k = len(xs)
    slope = (k * sum(x * y for x, y in zip(xs, ys)) - sum(xs) * sum(ys)) / (k * sum(x * x for x in xs) - sum(xs) ** 2)
    print(f"M2 slope: {slope:.3f} log2-seconds per T-gate")
    print("== M3: n-scaling at fixed t=6 ==")
    rows2 = []
    for n in (8, 12, 16, 20, 24, 28, 32):
        times = []
        for _ in range(3):
            src = gen_fixed(n, 6, 4 * n, rng)
            open(TMP, "w").write(src)
            out = subprocess.run([BIN, "amp", TMP], capture_output=True, text=True)
            times.append(json.loads(out.stdout)["seconds"])
        times.sort()
        rows2.append((n, times[1]))
        print(f"  n={n:2d}  {times[1]:.5f}s")
    xs2 = [math.log(n) for n, _ in rows2[1:]]
    ys2 = [math.log(max(s, 1e-7)) for _, s in rows2[1:]]
    k2 = len(xs2)
    slope2 = (k2 * sum(x * y for x, y in zip(xs2, ys2)) - sum(xs2) * sum(ys2)) / (k2 * sum(x * x for x in xs2) - sum(xs2) ** 2)
    print(f"M3 log-log slope: {slope2:.3f} (poly in n at fixed t; n=28,32 are past the statevector cap)")

if len(sys.argv) > 1 and sys.argv[1] == "m2m3":
    m2_m3()
