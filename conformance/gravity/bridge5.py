#!/usr/bin/env python3
"""BRIDGE-5 instrument. Prereg GRAVITY_BRIDGE5_PREREG.md frozen at d8352b0
BEFORE this file. Closes the three overclaims an external review found:
endogenous (G_p inside T), charged (pump sourced by the spinor pair's own
invariant channel), local (spokes-only electric term breaks the parity
homogeneity that made BRIDGE-3's control inert by symmetry, not distance).
Exact integer arithmetic; matter = rho2 spinor pair with joint Gauss."""
import sys
import numpy as np

sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import MUL, INV, CLASS, KAPPA, R2, rho2, base_graph, refined_graph

# matter index: m = (channel_bit, s1, s2) is NOT used; we keep the full
# 4-dim spinor pair space m = s1*2 + s2, and the CHANNEL is a derived,
# gauge-invariant label: the singlet |00>+|11> vs its orthogonal complement.
SINGLET = np.array([1, 0, 0, 1], dtype=np.int64)  # BARE |00>+|11> (diagonal-invariant ONLY)

def dressed_singlet(M):
    """M14: a charged pair is gauge-invariant only when DRESSED BY A WILSON
    LINE. The physical state is  Sum_ij rho2(U_gamma)_ij |ij>  for a path
    gamma from vertex 1 to vertex 2 -- the gauge field carries the
    invariance, which is what 'screened' means. Returns the per-configuration
    coefficient of each matter component, so the projector is CONFIGURATION
    DEPENDENT (it must be: the dressing is the gauge field)."""
    # path 1 -> c -> 2 : U = u(c,1)^{-1} u(c,2), read off the spokes
    e1 = next(i for i, (a, b) in enumerate(M.edges) if (a, b) == ("c", 1))
    e2 = next(i for i, (a, b) in enumerate(M.edges) if (a, b) == ("c", 2))
    u = MUL[INV[M.dig[e1]], M.dig[e2]]           # group element per config
    coef = np.zeros((4, M.N), dtype=np.int64)
    for g in range(8):
        m = rho2(g)
        sel = u == g
        if not np.any(sel):
            continue
        for i in range(2):
            for j in range(2):
                coef[(i << 1) | j][sel] = m[i, j]
    return coef

class Model5:
    def __init__(self, mk, broken_action=False, broken_pump=False):
        self.edges, self.plaq, self.loop, self.p0, self.e_star = mk()
        self.E = len(self.edges)
        self.N = 8 ** self.E
        self.verts = sorted({v for e in self.edges for v in e}, key=str)
        cfg = np.arange(self.N, dtype=np.int64)
        self.dig = np.empty((self.E, self.N), dtype=np.int64)
        for e in range(self.E):
            self.dig[e] = cfg % 8
            cfg //= 8
        self.pow8 = np.array([8 ** e for e in range(self.E)], dtype=np.int64)
        self.M = 4                      # rho2 (x) rho2
        self.broken_action = broken_action
        self.broken_pump = broken_pump
        self.base_idx = np.sum(self.dig * self.pow8[:, None], axis=0)
        self.hol = {p: self._word(w) for p, w in enumerate(self.plaq)}
        self.loop_hol = self._word(self.loop)
        # R3: SPOKES ONLY -- the edges incident to the centre. This breaks
        # the parity homogeneity that made BRIDGE-3's control inert.
        self.spokes = [i for i, (a, b) in enumerate(self.edges) if a == "c" or b == "c"]
        self.ub_t = np.zeros(self.N, dtype=np.int64)
        for p in range(len(self.plaq)):
            self.ub_t += KAPPA[CLASS[self.hol[p]]]
        self.ub_t %= 4

    def _word(self, word):
        h = np.zeros(self.N, dtype=np.int64)
        for e, s in word:
            g = self.dig[e] if s > 0 else INV[self.dig[e]]
            h = MUL[h, g]
        return h

    # ---------------------------------------------------------- gauge action
    def vperm(self, v, g):
        idx = np.zeros(self.N, dtype=np.int64)
        for e, (a, b) in enumerate(self.edges):
            d = self.dig[e]
            if a == v:
                d = MUL[d, g] if self.broken_action else MUL[g, d]
            if b == v:
                d = MUL[d, INV[g]]
            idx += d * self.pow8[e]
        return idx

    def joint(self, st, v, g):
        """Gauge at v; at matter sites 1,2 rotate the corresponding spinor."""
        re, im = st
        idx = self.vperm(v, g)
        nre = np.zeros_like(re); nim = np.zeros_like(im)
        if v in (1, 2):
            R = rho2(g)
            for m in range(4):
                s1, s2 = m >> 1, m & 1
                for sp in range(2):
                    if v == 1:
                        c = R[s1, sp]; mm = (sp << 1) | s2
                    else:
                        c = R[s2, sp]; mm = (s1 << 1) | sp
                    if c:
                        np.add.at(nre[m], idx, c * re[mm])
                        np.add.at(nim[m], idx, c * im[mm])
        else:
            for m in range(4):
                np.add.at(nre[m], idx, re[m])
                np.add.at(nim[m], idx, im[m])
        return (nre, nim)

    def gauss_holds(self, st):
        re, im = st
        for v in self.verts:
            sre = np.zeros_like(re); sim = np.zeros_like(im)
            for g in range(8):
                a, b = self.joint(st, v, g)
                sre += a; sim += b
            if not (np.array_equal(sre, 8 * re) and np.array_equal(sim, 8 * im)):
                return False, f"Gauss fails at {v}"
        return True, "held"

    # ------------------------------------------------------- Floquet terms
    def _left_mult(self, row, e, h):
        d = MUL[h, self.dig[e]]
        idx = self.base_idx - self.dig[e] * self.pow8[e] + d * self.pow8[e]
        out = np.zeros_like(row)
        np.add.at(out, idx, row)
        return out

    def k_charge(self, st):
        """R2: the pump is sourced by the CHARGED sector. Decompose the
        spinor pair into singlet (trivial isotypic component -- gauge
        invariant) and its complement; the flux fires only OFF the singlet."""
        re, im = st
        nre = re.copy(); nim = im.copy()
        h = 1 if self.broken_pump else R2       # plant (ii) uses non-central r
        # singlet amplitude per configuration (unnormalized projector)
        s_re = sum(SINGLET[m] * re[m] for m in range(4))
        s_im = sum(SINGLET[m] * im[m] for m in range(4))
        for m in range(4):
            # component orthogonal to the singlet, times 2 (keeps integers)
            ore = 2 * re[m] - SINGLET[m] * s_re
            oim = 2 * im[m] - SINGLET[m] * s_im
            sre_m = SINGLET[m] * s_re
            sim_m = SINGLET[m] * s_im
            # singlet part: untouched (x2). orthogonal part: pumped.
            nre[m] = sre_m + self._left_mult(ore, self.e_star, h)
            nim[m] = sim_m + self._left_mult(oim, self.e_star, h)
        return (nre, nim)

    def g_geo(self, st, p):
        """R1/R3: geometry->matter INSIDE the step. sigma_z on the invariant
        channel label (singlet +1, complement -1), conditioned on plaquette
        p's central parity. Gauge-invariant: class projector x isotypic
        sigma_z."""
        re, im = st
        par1 = CLASS[self.hol[p]] == 1
        nre = re.copy(); nim = im.copy()
        s_re = sum(SINGLET[m] * re[m] for m in range(4))
        s_im = sum(SINGLET[m] * im[m] for m in range(4))
        for m in range(4):
            sre_m = SINGLET[m] * s_re; sim_m = SINGLET[m] * s_im
            ore = 2 * re[m] - sre_m;    oim = 2 * im[m] - sim_m
            # where parity is 1: singlet +, complement -  (times 2 overall)
            nre[m] = np.where(par1, sre_m - ore, 2 * re[m])
            nim[m] = np.where(par1, sim_m - oim, 2 * im[m])
        return (nre, nim)

    def u_e(self, st):
        """Spokes-only electric term: (1 + i R_{r2}(e))/sqrt2 per SPOKE."""
        re, im = st
        for e in self.spokes:
            d = MUL[self.dig[e], R2]
            idx = self.base_idx - self.dig[e] * self.pow8[e] + d * self.pow8[e]
            nre = np.empty_like(re); nim = np.empty_like(im)
            for m in range(4):
                pr = re[m][idx]; pi = im[m][idx]
                nre[m] = re[m] - pi
                nim[m] = im[m] + pr
            re, im = nre, nim
        assert max(np.abs(re).max(), np.abs(im).max()) < 2 ** 62, "overflow: REFUSE"
        return (re, im)

    def u_b(self, st):
        re, im = st
        nre = re.copy(); nim = im.copy()
        for tv in (1, 2, 3):
            mask = self.ub_t == tv
            if not np.any(mask):
                continue
            for m in range(4):
                a, b = re[m][mask], im[m][mask]
                if tv == 1:   nre[m][mask], nim[m][mask] = -b, a
                elif tv == 2: nre[m][mask], nim[m][mask] = -a, -b
                else:         nre[m][mask], nim[m][mask] = b, -a
        return (nre, nim)

    def step(self, st):
        """R1: ONE joint unitary step. Nothing is applied by hand outside."""
        return self.u_b(self.u_e(self.g_geo(self.k_charge(st), self.p0)))

    # ------------------------------------------------------- state and reads
    def vacuum(self, matter):
        re = np.zeros((4, self.N), dtype=np.int64)
        im = np.zeros_like(re)
        for m in range(4):
            re[m][0] = matter[m]
        st = (re, im)
        for v in self.verts:
            acc_r = np.zeros_like(re); acc_i = np.zeros_like(im)
            for g in range(8):
                a, b = self.joint(st, v, g)
                acc_r += a; acc_i += b
            st = (acc_r, acc_i)
        return st

    def loop_classes(self, st):
        re, im = st
        sup = np.zeros(self.N, dtype=bool)
        for m in range(4):
            sup |= (re[m] != 0) | (im[m] != 0)
        return set(CLASS[self.loop_hol][sup].tolist())

    def channel_weight(self, st):
        """Exact singlet weight -- the matter reading."""
        re, im = st
        s_re = sum(SINGLET[m] * re[m] for m in range(4))
        s_im = sum(SINGLET[m] * im[m] for m in range(4))
        a = s_re.astype(object); b = s_im.astype(object)
        return int(np.sum(a * a + b * b))

def run(tag, mk):
    M = Model5(mk)
    rep = {}
    reg = []
    singlet = SINGLET.copy()
    triplet = np.array([1, 0, 0, -1], dtype=np.int64)   # orthogonal to singlet

    s_s = M.vacuum(singlet); s_t = M.vacuum(triplet)
    reg += [("vac_singlet", s_s), ("vac_triplet", s_t)]

    # R2: the CHARGED sector must decide whether the pump fires, and the
    # difference must reach the DISTANT loop.
    a, b = s_s, s_t
    r2_ok = False
    for k in range(1, 4):
        a = M.step(a); b = M.step(b)
        reg += [(f"T{k}_singlet", a), (f"T{k}_triplet", b)]
        ca, cb = M.loop_classes(a), M.loop_classes(b)
        if ca != cb:
            r2_ok = True
    rep["R2"] = "PASS" if r2_ok else "FIRE (charged sector does not reach the loop)"

    # R1: endogenous -- the matter reading must MOVE under step alone.
    w0 = M.channel_weight(s_t)
    ws = [w0]
    c = s_t
    for k in range(1, 4):
        c = M.step(c)
        ws.append(M.channel_weight(c))
    rep["R1"] = "PASS" if len(set(ws)) > 1 else f"FIRE (channel inert under T: {ws})"

    # R3: locality -- p0-conditioned response vs a DISTANT plaquette's.
    probe = M.step(M.vacuum(triplet))
    near = M.channel_weight(M.g_geo(probe, M.p0))
    far_p = next(p for p in range(len(M.plaq)) if p != M.p0)
    far = M.channel_weight(M.g_geo(probe, far_p))
    base = M.channel_weight(probe)
    rep["R3"] = ("PASS" if near != base and far == base
                 else f"FIRE base={base} near={near} far={far}")

    bad = [nm for nm, s in reg for held, _ in [M.gauss_holds(s)] if not held]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"
    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep

def plants():
    Mb = Model5(base_graph, broken_action=True)
    Mok = Model5(base_graph)
    held, why = Mok.gauss_holds(Mb.vacuum(SINGLET))
    p1 = not held
    print(f"[plant i] wrong-side action -> B3 {'FIRES' if p1 else 'MISSED'} ({why})")
    Mp = Model5(base_graph, broken_pump=True)
    held, why = Mp.gauss_holds(Mp.k_charge(Mp.vacuum(np.array([1,0,0,-1],dtype=np.int64))))
    p2 = not held
    print(f"[plant ii] non-central pump -> B3 {'FIRES' if p2 else 'MISSED'} ({why})")
    return p1 and p2

if __name__ == "__main__":
    r1 = run("base", base_graph)
    r2 = run("refined", refined_graph)
    ok = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values())
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (gates and ok) else 1)
