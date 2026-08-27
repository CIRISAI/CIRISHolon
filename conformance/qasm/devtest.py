import random, subprocess, json, sys
sys.path.insert(0,'.')
from conformance import gen, TMP
BIN = "/home/emoore/CIRISHolon/engine/target/release/holon-qasm"
rng = random.Random(int(sys.argv[1]) if len(sys.argv)>1 else 999)
worst = 0.0; fails = []; tested = 0
while tested < 120:
    n = rng.randint(1, 6); depth = rng.randint(1, 25)
    src = gen("magic", n, depth, rng)
    t = sum(l.split()[0] in ("t","tdg") for l in src.splitlines() if l and l[0] in "t")
    if t > 10: continue
    open(TMP,"w").write(src)
    out = subprocess.run([BIN, "test-magic", TMP], capture_output=True, text=True, timeout=60)
    tested += 1
    if out.returncode != 0 or not out.stdout.strip():
        fails.append((tested, src.count("\n"), out.stderr.strip()[:150]))
        if len(fails) > 4: break
        continue
    worst = max(worst, json.loads(out.stdout)["max_err"])
print(f"property test: {tested} circuits (t<=10), worst err = {worst:.3e}, failures: {len(fails)}")
for f in fails[:4]: print(" FAIL", f)
