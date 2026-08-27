#!/usr/bin/env python3
"""BRIDGE-3 instrument. Prereg 8b12296, frozen before this file. One rung:
the geometry-conditioned occupation action G_p, staked ratios exact."""
import sys
import numpy as np
from fractions import Fraction

sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import Model, base_graph, refined_graph, CLASS, KAPPA

def G_p(M, st, p):
    re, im, k2 = st
    par1 = CLASS[M.hol_tab[p]] == 1
    nre = re.copy(); nim = im.copy()
    for m in range(M.M):
        if M.occ_of(m) == 1:
            nre[m][par1] = -nre[m][par1]
            nim[m][par1] = -nim[m][par1]
    return (nre, nim, k2)

def coherence(M, st):
    re, im, _ = st
    half = M.M // 2
    tot = 0
    for m in range(half):
        a0 = re[m].astype(object); b0 = im[m].astype(object)
        a1 = re[m + half].astype(object); b1 = im[m + half].astype(object)
        tot += int(np.sum(a0 * a1 + b0 * b1))
    return tot

# occupation-labeled combinatorial route (independent, 2x8-dim)
def comb_c_ratio(mk, k_steps, act=True):
    """Returns (C_before, C_after) in the parity algebra, exact."""
    edges, plaq, loop, p0, e_star = mk()
    members = [[p for p, w in enumerate(plaq) if any(we == ei for we, _ in w)]
               for ei in range(len(edges))]
    KAP = int(KAPPA[1])
    def phase_mul(z, t):
        a, b = z; t %= 4
        return [(a, b), (-b, a), (-a, -b), (b, -a)][t]
    amp = {(0, 0): (Fraction(1), Fraction(0)), (1, 0): (Fraction(1), Fraction(0))}
    for _ in range(k_steps):
        amp = {(o, s ^ ((1 << p0) if o else 0)): z for (o, s), z in amp.items()}
        amp = {(o, s): phase_mul(z, KAP if (s >> p0) & 1 else 0)
               for (o, s), z in amp.items()}
        for ei in range(len(edges)):
            flip = 0
            for p in members[ei]:
                flip ^= 1 << p
            nxt = {}
            for (o, s), (a, b) in amp.items():
                ra, rb = nxt.get((o, s), (Fraction(0), Fraction(0)))
                nxt[(o, s)] = (ra + a, rb + b)
                ra, rb = nxt.get((o, s ^ flip), (Fraction(0), Fraction(0)))
                nxt[(o, s ^ flip)] = (ra - b, rb + a)
            amp = nxt
        amp = {(o, s): phase_mul(z, KAP * bin(s).count("1"))
               for (o, s), z in amp.items()}
    def C_of(a):
        tot = Fraction(0)
        for s in {s for (_, s) in a}:
            a0, b0 = a.get((0, s), (Fraction(0), Fraction(0)))
            a1, b1 = a.get((1, s), (Fraction(0), Fraction(0)))
            tot += a0 * a1 + b0 * b1
        return tot
    before = C_of(amp)
    after_amp = {(o, s): ((-z[0], -z[1]) if (o == 1 and (s >> p0) & 1) else z)
                 for (o, s), z in amp.items()}
    return before, C_of(after_amp)

def run(tag, mk, with_spinors):
    M = Model(mk, with_spinors)
    rep = {}
    registry = []
    st = M.vacuum_occ_superposed()
    registry.append(("sup0", st))
    want = {1: "zero", 2: "flip", 3: "zero", 4: "inert"}
    ok = True
    for k in range(1, 5):
        st = M.step(st)
        registry.append((f"T{k}", st))
        C0 = coherence(M, st)
        stG = G_p(M, st, M.p0)
        registry.append((f"G(T{k})", stG))
        C1 = coherence(M, stG)
        cb, ca = comb_c_ratio(mk, k)
        # two-route on the (before, after) pair up to a common positive scale:
        route_ok = (C0 == 0 and cb == 0) or (cb != 0 and Fraction(C1) * cb == Fraction(C0) * ca)
        w = want[k]
        gate_ok = ((w == "zero" and C0 == 0)
                   or (w == "flip" and C0 != 0 and C1 == -C0)
                   or (w == "inert" and C0 != 0 and C1 == C0))
        if not (gate_ok and route_ok):
            ok = False
            rep[f"B2_k{k}"] = f"FIRE C0={C0} C1={C1} comb=({cb},{ca})"
        print(f"    k={k}: C={C0} -> {C1}  [{w}]  route_ok={route_ok}")
    rep["B2''"] = "PASS" if ok else "FIRE"
    bad = [nm for nm, s in registry for held, _ in [M.gauss_holds(s)] if not held]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"
    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())))
    return rep

def plants():
    from bridge1 import Model as Mo
    Mb = Mo(base_graph, with_spinors=False, broken_action=True)
    Mk = Mo(base_graph, with_spinors=False)
    held, why = Mk.gauss_holds(Mb.vacuum(0))
    p1 = not held
    print(f"[plant i] wrong-side action -> {'FIRES' if p1 else 'MISSED'}")
    Mp = Mo(base_graph, with_spinors=False, broken_pump=True)
    held, why = Mp.gauss_holds(Mp.u_pump(Mp.vacuum(1)))
    p2 = not held
    print(f"[plant ii] non-central pump -> {'FIRES' if p2 else 'MISSED'}")
    return p1 and p2

if __name__ == "__main__":
    r1 = run("base", base_graph, with_spinors=True)
    r2 = run("refined", refined_graph, with_spinors=False)
    ok = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values())
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE' if ok else 'VOID'}")
    sys.exit(0 if (gates and ok) else 1)
