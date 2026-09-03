#!/usr/bin/env python3
"""A3's G0''' verdicts: each sector at its rank-derived ceiling, mixed within 1e-6 of the
exact arm and the cold start within 1e-6 of the mixed top rung; V1 on the longer ladders."""
import json, os
EX = {(4.0,0):-51.9229999638,(4.0,1):-47.9964825669,(4.0,2):-36.6401053164,(9.0,0):-123.0642401146,(9.0,1):-113.9136751337,(9.0,2):-87.5269948585}
here = os.path.dirname(os.path.abspath(__file__))
def load(n):
    p = os.path.join(here, "rows", n + ".json"); return json.load(open(p)) if os.path.exists(p) else None
def top(j): return [r for r in j["rungs"] if "energy" in r][-1]
ok = True
for x, b in [(4.0,0),(9.0,0),(9.0,1),(4.0,1)]:
    m, c = load(f"a3mixed_x{x}_B{b}"), load(f"a3cold_x{x}_B{b}")
    e = EX[(x,b)]
    if not (m and c): print(f"  ....  x={x} B={b}: waiting"); ok = False; continue
    tm, tc = top(m), top(c)
    pm = abs(tm["energy"]-e) <= 1e-6; pc = abs(tc["energy"]-tm["energy"]) <= 1e-6
    ok &= pm and pc
    rungs = " ".join(f"{r['chi']}:{r['energy']-e:+.2e}/{r.get('variance', float('nan')):.1e}/{r['exit'][:4]}" for r in m["rungs"] if "energy" in r)
    print(f"  {'PASS' if pm else 'FIRE'}  x={x} B={b} mixed at chi {tm['chi']}: miss {tm['energy']-e:+.3e} ({tm['exit']}, {tm['sweeps']} sw, {tm['seconds']:.0f}s) | rungs miss/var/exit: {rungs}")
    print(f"  {'PASS' if pc else 'FIRE'}  x={x} B={b} cold at chi {tc['chi']}: miss {tc['energy']-e:+.3e}, |cold-mixed| {abs(tc['energy']-tm['energy']):.2e}, var {tc.get('variance')} ({tc['seconds']:.0f}s)")
    vs = [r.get("variance") for r in m["rungs"] if "energy" in r]
    mono = all(v is not None for v in vs) and all(vs[i] > vs[i+1] for i in range(len(vs)-1))
    print(f"  {'PASS' if mono else 'FIRE'}  V1 x={x} B={b}: " + " > ".join(f"{v:.2e}" for v in vs if v is not None))
    ok &= mono
print("G0''' ALL PASS" if ok else "NOT ALL PASS (or waiting)")
