#!/usr/bin/env python3
"""BRIDGE-6 instrument. Prereg 97d2958, frozen BEFORE this file, with M14's
Wilson-dressed charge verified before the freeze. G0 (the invariance gate)
runs first and gates everything else."""
import sys
import numpy as np
sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import MUL, INV, CLASS, KAPPA, R2, rho2, base_graph, refined_graph
from bridge5 import Model5, dressed_singlet

class Model6(Model5):
    """Model5's dynamics with the BARE channel replaced by the DRESSED one."""
    def __init__(self, mk, **kw):
        super().__init__(mk, **kw)
        self.W = dressed_singlet(self)          # coef[m][config], M14
        # <W|W> per configuration (rho2 is orthogonal -> constant 2)
        self.Wnorm = sum(self.W[m].astype(object) ** 2 for m in range(4))

    def _split(self, re, im):
        """Decompose into (dressed-singlet part, complement), both x2 scaled
        so the arithmetic stays exact integers."""
        pr = sum(self.W[m] * re[m] for m in range(4))
        pi = sum(self.W[m] * im[m] for m in range(4))
        sre = [self.W[m] * pr for m in range(4)]
        sim = [self.W[m] * pi for m in range(4)]
        ore = [2 * re[m] - sre[m] for m in range(4)]
        oim = [2 * im[m] - sim[m] for m in range(4)]
        return sre, sim, ore, oim

    def k_charge(self, st):
        re, im = st
        h = 1 if self.broken_pump else R2
        sre, sim, ore, oim = self._split(re, im)
        nre = np.empty_like(re); nim = np.empty_like(im)
        for m in range(4):
            nre[m] = sre[m] + self._left_mult(ore[m], self.e_star, h)
            nim[m] = sim[m] + self._left_mult(oim[m], self.e_star, h)
        return (nre, nim)

    def g_geo(self, st, p):
        re, im = st
        par1 = CLASS[self.hol[p]] == 1
        sre, sim, ore, oim = self._split(re, im)
        nre = np.empty_like(re); nim = np.empty_like(im)
        for m in range(4):
            nre[m] = np.where(par1, sre[m] - ore[m], 2 * re[m])
            nim[m] = np.where(par1, sim[m] - oim[m], 2 * im[m])
        return (nre, nim)

    def channel_weight(self, st):
        re, im = st
        pr = sum(self.W[m] * re[m] for m in range(4)).astype(object)
        pi = sum(self.W[m] * im[m] for m in range(4)).astype(object)
        return int(np.sum(pr * pr + pi * pi))

    def dressed_vacuum(self, sign=+1):
        """Gauss-project the dressed pair (sign=+1) or a complement state."""
        re = np.zeros((4, self.N), dtype=np.int64); im = np.zeros_like(re)
        for m in range(4):
            re[m] = self.W[m] if sign > 0 else (m == 0) * 1 - self.W[m] * 0
        if sign < 0:                    # a state deliberately off the channel
            re = np.zeros((4, self.N), dtype=np.int64)
            re[1][:] = 1
        st = (re, im)
        for v in self.verts:
            ar = np.zeros_like(re); ai = np.zeros_like(im)
            for g in range(8):
                a, b = self.joint(st, v, g); ar += a; ai += b
            st = (ar, ai)
        return st

def nonzero(st):
    return int(sum(np.count_nonzero(st[0][m]) + np.count_nonzero(st[1][m]) for m in range(4)))

def run(tag, mk):
    M = Model6(mk)
    rep = {}
    # ---- G0 FIRST: the projector's invariance gates everything else.
    probe = M.dressed_vacuum(+1)
    if nonzero(probe) == 0:
        print(f"[{tag}] G0=FIRE (dressed sector empty)"); return {"G0": "FIRE"}
    held, why = M.gauss_holds(probe)
    split_then_gauss = M.gauss_holds(M.k_charge(probe))[0]
    rep["G0"] = "PASS" if held and split_then_gauss else f"FIRE (held={held} after_pump={split_then_gauss}: {why})"
    if rep["G0"] != "PASS":
        print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in rep.items())); return rep

    reg = [("dressed", probe)]
    # ---- R1: endogenous — the channel reading moves under step ALONE.
    ws = [M.channel_weight(probe)]
    c = probe
    for k in range(1, 4):
        c = M.step(c); reg.append((f"T{k}", c)); ws.append(M.channel_weight(c))
    rep["R1"] = "PASS" if len(set(ws)) > 1 else f"FIRE (inert under T: {[str(w)[:12] for w in ws]})"

    # ---- R2: the charged channel decides the pump, and it reaches the loop.
    off = M.dressed_vacuum(-1)
    if nonzero(off) == 0:
        rep["R2"] = "FIRE (off-channel carrier empty — cannot pose)"
    else:
        a, b = probe, off
        seen = False
        for _ in range(3):
            a = M.step(a); b = M.step(b)
            if M.loop_classes(a) != M.loop_classes(b):
                seen = True
        rep["R2"] = "PASS" if seen else "FIRE (charged channel does not reach the loop)"

    # ---- R3: locality with the spokes-only term.
    p1 = M.step(probe)
    base = M.channel_weight(p1)
    near = M.channel_weight(M.g_geo(p1, M.p0))
    far_p = next(p for p in range(len(M.plaq)) if p != M.p0)
    far = M.channel_weight(M.g_geo(p1, far_p))
    rep["R3"] = ("PASS" if near != base and far == base
                 else f"FIRE base={str(base)[:10]} near={str(near)[:10]} far={str(far)[:10]}")

    bad = [nm for nm, s in reg for h, _ in [M.gauss_holds(s)] if not h]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"
    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep

def plants():
    ok = True
    Mok = Model6(base_graph)
    carrier = Mok.dressed_vacuum(+1)
    assert nonzero(carrier) > 0, "plant carrier is EMPTY — a plant on zero is invisible (M8)"
    Mb = Model6(base_graph, broken_action=True)
    held, why = Mok.gauss_holds(Mb.dressed_vacuum(+1))
    print(f"[plant i] wrong-side action -> B3 {'FIRES' if not held else 'MISSED'} ({why})")
    ok &= not held
    Mp = Model6(base_graph, broken_pump=True)
    c2 = Mp.dressed_vacuum(+1)
    assert nonzero(c2) > 0, "plant ii carrier is EMPTY"
    held2, why2 = Mp.gauss_holds(Mp.k_charge(c2))
    print(f"[plant ii] non-central pump -> B3 {'FIRES' if not held2 else 'MISSED'} ({why2})")
    return ok and not held2

if __name__ == "__main__":
    r1 = run("base", base_graph)
    r2 = run("refined", refined_graph)
    ok = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values())
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (gates and ok) else 1)
