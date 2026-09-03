#!/usr/bin/env python3
"""A2's verdicts from the rows: G0'' (1e-6 at chi=256, mixed; cold agrees within 1e-6),
V1 (variance strictly decreasing in chi; unmixed/mixed variance ratio >= 10 on x=4 B=1),
plants vi (unmixed misses by > 1e-5, mixed meets), vii (mutant misses by > 1e-3), viii
(the variance ratio with residuals within 2x)."""
import json, os, sys, glob
EX = {(4.0,0):-51.9229999638,(4.0,1):-47.9964825669,(4.0,2):-36.6401053164,(9.0,0):-123.0642401146,(9.0,1):-113.9136751337,(9.0,2):-87.5269948585}
here = os.path.dirname(os.path.abspath(__file__))
def load(name):
    p = os.path.join(here, "rows", name + ".json")
    return json.load(open(p)) if os.path.exists(p) else None
def top(j): return [r for r in j["rungs"] if "energy" in r][-1] if j else None
ok_all = True
def verdict(label, cond, detail):
    global ok_all
    ok_all &= bool(cond)
    print(f"  {'PASS' if cond else 'FIRE'}  {label}: {detail}")
print("G0'' — mixed warm ladder 64->128->256 within 1e-6 at chi=256, cold chi=256 within 1e-6 of it")
for x in (4.0, 9.0):
    for b in (0, 1, 2):
        m, c = load(f"mixed_x{x}_B{b}"), load(f"cold_x{x}_B{b}")
        if not (m and c): print(f"  ....  x={x} B={b}: waiting"); ok_all = False; continue
        tm, tc = top(m), top(c)
        e = EX[(x, b)]
        verdict(f"x={x} B={b} mixed", abs(tm["energy"] - e) <= 1e-6, f"miss {tm['energy']-e:+.3e} at chi {tm['chi']} ({tm['exit']}, {tm['sweeps']} sw, {tm['class']}, var {tm.get('variance')}, {tm['seconds']:.0f}s)")
        verdict(f"x={x} B={b} cold vs mixed", abs(tc["energy"] - tm["energy"]) <= 1e-6, f"cold {tc['energy']-e:+.3e}, |cold-mixed| {abs(tc['energy']-tm['energy']):.2e}, var {tc.get('variance')}")
print("V1 — the variance strictly decreasing in chi on every sector")
for x in (4.0, 9.0):
    for b in (0, 1, 2):
        m = load(f"mixed_x{x}_B{b}")
        if not m: continue
        vs = [r.get("variance") for r in m["rungs"] if "energy" in r]
        mono = all(v is not None for v in vs) and all(vs[i] > vs[i+1] for i in range(len(vs)-1))
        verdict(f"x={x} B={b}", mono, " > ".join(f"{v:.3e}" if v is not None else "?" for v in vs))
u, m, mu = load("unmixed_x4.0_B1"), load("mixed_x4.0_B1"), load("mutant_x4.0_B1")
e = EX[(4.0, 1)]
if u and m:
    tu, tm = top(u), top(m)
    print("plant (vi) — mixing is load-bearing")
    verdict("unmixed misses by > 1e-5", abs(tu["energy"] - e) > 1e-5, f"{tu['energy']-e:+.3e}")
    verdict("mixed meets 1e-6", abs(tm["energy"] - e) <= 1e-6, f"{tm['energy']-e:+.3e}")
    print("V1 / plant (viii) — the variance separates what the residual cannot")
    vu, vm = tu.get("variance"), tm.get("variance")
    ru, rm = tu["worst_residual"], tm["worst_residual"]
    if vu is not None and vm is not None and vm > 0:
        verdict("unmixed/mixed variance >= 10x", vu / vm >= 10, f"{vu:.3e} / {vm:.3e} = {vu/vm:.1f}x")
        verdict("residuals within 2x", max(ru, rm) / min(ru, rm) <= 2, f"{ru:.2e} vs {rm:.2e}")
    else:
        verdict("variance available on both", False, f"unmixed {vu} mixed {vm}")
if mu:
    print("plant (vii) — the labels are load-bearing at chi=256")
    tmu = top(mu)
    verdict("mutant misses by > 1e-3", abs(tmu["energy"] - e) > 1e-3, f"{tmu['energy']-e:+.3e} ({tmu['exit']}, {tmu['sweeps']} sw)")
print("ALL STAKED VERDICTS PASS" if ok_all else "NOT ALL PASS (or still running)")
