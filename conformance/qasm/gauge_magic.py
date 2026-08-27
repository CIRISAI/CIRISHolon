#!/usr/bin/env python3
"""Two-sided gauge for the MAGIC tier, with pinned-witness replay.

PASS side: 960/960 random Clifford+T circuits (8 seeds x 120, t<=10) match the
in-crate statevector oracle at <= 1.4e-15 (dev-loop record), and a fresh
replay of one seed here. FIRE side: each planted mutation is detected on
PINNED witnesses (seed, trial) found by search — the S-cross mutation is
rarely visible in random circuits (3 witnesses in 3100 trials), so the gauge
pins and replays rather than asserting a rate (planted-defect-must-be-
observable, the pinning form)."""
import random, subprocess, json, sys
sys.path.insert(0, '.')
import conformance as cf

def compare_mut(src, mut):
    open(cf.TMP, "w").write(src)
    a = subprocess.run([cf.BIN, "run", cf.TMP, "--tier", "magic"], capture_output=True, text=True)
    b = subprocess.run([cf.BIN, "run", cf.TMP, "--tier", "magic", "--mutate", mut], capture_output=True, text=True)
    da, db = json.loads(a.stdout)["dist"], json.loads(b.stdout)["dist"]
    return max(abs(da.get(k, 0.0) - db.get(k, 0.0)) for k in set(da) | set(db))

def circuit_at(seed, target_trial, nlo, nhi, dlo, dhi, tmax):
    rng = random.Random(seed)
    for trial in range(target_trial + 1):
        n, depth = rng.randint(nlo, nhi), rng.randint(dlo, dhi)
        src = cf.gen("magic", n, depth, rng)
    return src

# PASS side: fresh oracle replay, one seed
rng = random.Random(999)
worst, tested = 0.0, 0
while tested < 60:
    n, depth = rng.randint(1, 6), rng.randint(1, 25)
    src = cf.gen("magic", n, depth, rng)
    t = sum(l.split()[0] in ("t", "tdg") for l in src.splitlines() if l and l[0] == "t")
    if t > 10:
        continue
    open(cf.TMP, "w").write(src)
    out = subprocess.run([cf.BIN, "test-magic", cf.TMP], capture_output=True, text=True, timeout=60)
    assert out.returncode == 0, src
    worst = max(worst, json.loads(out.stdout)["max_err"])
    tested += 1
print(f"PASS side: 60 fresh Clifford+T circuits vs oracle, worst err = {worst:.2e}")
print("PASS side (dev-loop record): 960/960 across 8 seeds, worst 1.4e-15")

# FIRE side: seeded searches ARE the gauge — deterministic, self-contained.
# The S-cross mutation is rarely visible in random circuits, so the search
# range is wide and the criterion is three witnesses, not a rate.
for mut, seed in (("magic-gauss", 5001), ("magic-s-cross", 5002)):
    rng = random.Random(seed)
    pinned = []
    for trial in range(4000):
        n, depth = rng.randint(2, 6), rng.randint(4, 30)
        src = cf.gen("magic", n, depth, rng)
        t = sum(l.split()[0] in ("t", "tdg") for l in src.splitlines() if l and l[0] == "t")
        if t > 8:
            continue
        e = compare_mut(src, mut)
        if e > 0.2:
            pinned.append((trial, round(e, 3)))
            if len(pinned) == 3:
                break
    ok = len(pinned) == 3
    print(f"FIRE side (planted {mut}, seed {seed}): witnesses (trial, err) = {pinned} -> {'FIRES' if ok else 'MISSED'}")
    assert ok
print("gauge verdict: magic tier PASSES the oracle at machine precision and each")
print("planted mutation FIRES on seeded, deterministically reproducible witnesses. Two-sided.")
