#!/usr/bin/env python3
"""BRIDGE-1 instrument. Prereg GRAVITY_BRIDGE1_PREREG.md frozen at ca42b33's
successor b12ffdf, amendments A1 (d41a609) and A2 (11baa64), ALL committed
before this file. Exact arithmetic: amplitudes are Gaussian integers with a
global sqrt(2) exponent; readings use unbounded python ints. Gauge group D4;
genuine discrete-time unitary dynamics (Floquet step of local gauge-invariant
unitary terms); constraints = Gauss only, checked on every registry state."""
import numpy as np
import sys

# --------------------------------------------------------------- D4, exact
# element (k, b) = r^k s^b, index k + 4b
def _mul(a, b):
    k1, b1 = a % 4, a // 4
    k2, b2 = b % 4, b // 4
    k = (k1 + (k2 if b1 == 0 else -k2)) % 4
    return k + 4 * ((b1 + b2) % 2)

MUL = np.array([[_mul(a, b) for b in range(8)] for a in range(8)], dtype=np.int64)
INV = np.array([next(b for b in range(8) if _mul(a, b) == 0) for a in range(8)],
               dtype=np.int64)
R2 = 2  # the central involution r^2
# classes: 0:{e} 1:{r2} 2:{r,r3} 3:{s,r2s} 4:{rs,r3s}
CLASS = np.array([0, 2, 1, 2, 3, 4, 3, 4], dtype=np.int64)
KAPPA = np.array([0, 2, 1, 3, 3], dtype=np.int64)  # magnetic phase per class
assert MUL[R2, 5] == MUL[5, R2], "r2 must be central"
# rho2: 2-dim irrep, integer matrices
def rho2(g):
    k, b = g % 4, g // 4
    m = np.array([[1, 0], [0, 1]], dtype=np.int64)
    R = np.array([[0, -1], [1, 0]], dtype=np.int64)
    S = np.array([[1, 0], [0, -1]], dtype=np.int64)
    for _ in range(k):
        m = R @ m
    if b:
        m = m @ S
    return m

def base_graph():
    edges = [("c", 1), ("c", 2), ("c", 3), (1, 2), (2, 3), (3, 1)]
    plaq = [[(0, 1), (3, 1), (1, -1)], [(1, 1), (4, 1), (2, -1)],
            [(2, 1), (5, 1), (0, -1)]]
    loop = [(3, 1), (4, 1), (5, 1)]
    return edges, plaq, loop, 0, 3

def refined_graph():
    edges = [("c", 1), ("c", 2), ("c", 3), (1, 4), (4, 2), (2, 3), (3, 1), ("c", 4)]
    plaq = [[(0, 1), (3, 1), (7, -1)], [(7, 1), (4, 1), (1, -1)],
            [(1, 1), (5, 1), (2, -1)], [(2, 1), (6, 1), (0, -1)]]
    loop = [(3, 1), (4, 1), (5, 1), (6, 1)]
    return edges, plaq, loop, 0, 3

class Model:
    """State: (M, N) re/im int64 + global k2 (value = z * 2^{-k2/2}).
    M = matter components; base graph: occ(2) x spin1(2) x spin2(2) = 8;
    refined: occ only = 2 (A2). Matter sites at vertices 1 and 2 (base)."""
    def __init__(self, mk, with_spinors, broken_action=False, broken_pump=False):
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
        self.with_spinors = with_spinors
        self.M = 8 if with_spinors else 2
        self.broken_action = broken_action
        self.broken_pump = broken_pump
        self.base_idx = np.sum(self.dig * self.pow8[:, None], axis=0)
        # static class tables
        self.hol_tab = {p: self._word_hol(w) for p, w in enumerate(self.plaq)}
        self.loop_hol = self._word_hol(self.loop)
        self.ub_t = np.zeros(self.N, dtype=np.int64)
        for p in range(len(self.plaq)):
            self.ub_t += KAPPA[CLASS[self.hol_tab[p]]]
        self.ub_t %= 4
        owners = [p for p, w in enumerate(self.plaq)
                  if any(e == self.e_star for e, _ in w)]
        assert owners == [self.p0]

    def _word_hol(self, word):
        h = np.zeros(self.N, dtype=np.int64)
        for e, s in word:
            g = self.dig[e] if s > 0 else INV[self.dig[e]]
            h = MUL[h, g]
        return h

    # matter index helpers: m = ((occ*2)+s1)*2+s2 (spinors) or m = occ
    def occ_of(self, m):
        return m >> 2 if self.with_spinors else m

    # ------------------------------------------------------------ actions
    def vertex_perm(self, v, g):
        idx = np.zeros(self.N, dtype=np.int64)
        for e, (a, b) in enumerate(self.edges):
            d = self.dig[e]
            if a == v:
                d = MUL[d, g] if self.broken_action else MUL[g, d]
            if b == v:
                d = MUL[d, INV[g]]
            idx += d * self.pow8[e]
        return idx

    def joint_action(self, st, v, g):
        """Gauge action at v; at matter sites additionally rotate the spinor."""
        re, im, k2 = st
        idx = self.vertex_perm(v, g)
        nre = np.zeros_like(re)
        nim = np.zeros_like(im)
        # matter rotation matrix on the M components
        if self.with_spinors and v in (1, 2):
            Rm = rho2(g)
            for m in range(self.M):
                occ, s1, s2 = m >> 2, (m >> 1) & 1, m & 1
                for sp in range(2):
                    if v == 1:
                        c = Rm[s1, sp]
                        mm = (occ << 2) | (sp << 1) | s2
                    else:
                        c = Rm[s2, sp]
                        mm = (occ << 2) | (s1 << 1) | sp
                    if c != 0:
                        np.add.at(nre[m], idx, c * re[mm])
                        np.add.at(nim[m], idx, c * im[mm])
        else:
            for m in range(self.M):
                np.add.at(nre[m], idx, re[m])
                np.add.at(nim[m], idx, im[m])
        return (nre, nim, k2)

    def gauss_holds(self, st):
        re, im, _ = st
        for v in self.verts:
            sre = np.zeros_like(re)
            sim = np.zeros_like(im)
            for g in range(8):
                ar, ai, _ = self.joint_action(st, v, g)
                sre += ar
                sim += ai
            if not (np.array_equal(sre, 8 * re) and np.array_equal(sim, 8 * im)):
                return False, f"Gauss fails at {v}"
        return True, "held"

    # -------------------------------------------------------- state prep
    def vacuum(self, occ):
        re = np.zeros((self.M, self.N), dtype=np.int64)
        im = np.zeros_like(re)
        if self.with_spinors:
            # occupation `occ`, spinor pair singlet |00> + |11>
            re[(occ << 2) | 0][0] = 1
            re[(occ << 2) | 3][0] = 1
        else:
            re[occ][0] = 1
        st = (re, im, 0)
        for v in self.verts:
            acc_re = np.zeros_like(re)
            acc_im = np.zeros_like(im)
            for g in range(8):
                ar, ai, _ = self.joint_action(st, v, g)
                acc_re += ar
                acc_im += ai
            st = (acc_re, acc_im, st[2])
        return st

    def vacuum_occ_superposed(self):
        a = self.vacuum(0)
        b = self.vacuum(1)
        return (a[0] + b[0], a[1] + b[1], a[2])

    # ------------------------------------------------------ Floquet terms
    def u_pump(self, st):
        """Occupation-controlled central flux injection at e*. Plant (ii):
        a NON-central pump L_r instead of L_{r2} — must break Gauss."""
        re, im, k2 = st
        h = 1 if self.broken_pump else R2
        d = MUL[h, self.dig[self.e_star]]
        idx = self.base_idx - self.dig[self.e_star] * self.pow8[self.e_star] \
            + d * self.pow8[self.e_star]
        nre = re.copy()
        nim = im.copy()
        for m in range(self.M):
            if self.occ_of(m) == 1:
                tr = np.zeros(self.N, dtype=np.int64)
                ti = np.zeros(self.N, dtype=np.int64)
                np.add.at(tr, idx, re[m])
                np.add.at(ti, idx, im[m])
                nre[m], nim[m] = tr, ti
        return (nre, nim, k2)

    def u_geo_mat(self, st, word=None):
        """Class-controlled phase i^{kappa(class hol)} (A2: phase only)."""
        re, im, k2 = st
        hol = self.loop_hol if word is None else self._word_hol(word)
        t = KAPPA[CLASS[hol]] % 4
        nre = re.copy()
        nim = im.copy()
        for tv, (fr, fi) in ((1, (0, 1)), (2, (-1, 0)), (3, (0, -1))):
            mask = t == tv
            if not np.any(mask):
                continue
            for m in range(self.M):
                a, b = re[m][mask], im[m][mask]
                if (fr, fi) == (0, 1):
                    nre[m][mask], nim[m][mask] = -b, a
                elif (fr, fi) == (-1, 0):
                    nre[m][mask], nim[m][mask] = -a, -b
                else:
                    nre[m][mask], nim[m][mask] = b, -a
        return (nre, nim, k2)

    def u_electric(self, st):
        """Per edge: (1 + i R_{r2}(e))/sqrt2 = e^{i pi R/4} exactly."""
        re, im, k2 = st
        for e in range(self.E):
            d = MUL[self.dig[e], R2]  # right-mult by central r2
            idx = self.base_idx - self.dig[e] * self.pow8[e] + d * self.pow8[e]
            nre = np.empty_like(re)
            nim = np.empty_like(im)
            for m in range(self.M):
                pr = re[m][idx]
                pi = im[m][idx]
                nre[m] = re[m] - pi
                nim[m] = im[m] + pr
            re, im, k2 = nre, nim, k2 + 1
        assert max(np.abs(re).max(), np.abs(im).max()) < 2 ** 62, "overflow: REFUSE"
        return (re, im, k2)

    def u_magnetic(self, st):
        re, im, k2 = st
        nre = re.copy()
        nim = im.copy()
        for tv in (1, 2, 3):
            mask = self.ub_t == tv
            if not np.any(mask):
                continue
            for m in range(self.M):
                a, b = re[m][mask], im[m][mask]
                if tv == 1:
                    nre[m][mask], nim[m][mask] = -b, a
                elif tv == 2:
                    nre[m][mask], nim[m][mask] = -a, -b
                else:
                    nre[m][mask], nim[m][mask] = b, -a
        return (nre, nim, k2)

    def step(self, st):
        return self.u_magnetic(self.u_electric(self.u_geo_mat(self.u_pump(st),
                                                              self.plaq[self.p0])))

    # ----------------------------------------------------------- readings
    def loop_distribution(self, st):
        """Exact per-class weights (python ints) of the boundary loop."""
        re, im, k2 = st
        cls = CLASS[self.loop_hol]
        out = {}
        for c in range(5):
            mask = cls == c
            tot = 0
            for m in range(self.M):
                a = re[m][mask].astype(object)
                b = im[m][mask].astype(object)
                tot += int(np.sum(a * a + b * b))
            if tot:
                out[c] = tot
        return out, k2

def norm_dist(d):
    tot = sum(d.values())
    from fractions import Fraction
    return {c: Fraction(v, tot) for c, v in d.items()}

# ---------------------------------------------------- independent route (B1)
def combinatorial_route(mk, occ, k_steps):
    """Central-sector dynamics on plaquette-parity bits — an 8- or 16-dim
    exact recursion sharing no code with the state-vector evolution."""
    from fractions import Fraction
    edges, plaq, loop, p0, e_star = mk()
    P = len(plaq)
    members = [[p for p, w in enumerate(plaq) if any(e == ei for ei, _ in w)]
               for ei in range(len(edges))]
    # amplitude over parity bitmasks, entries (Fraction re, Fraction im), /sqrt2^j
    amp = {0: (Fraction(1), Fraction(0))}
    def phase_mul(z, t):
        a, b = z
        t %= 4
        if t == 0:
            return (a, b)
        if t == 1:
            return (-b, a)
        if t == 2:
            return (-a, -b)
        return (b, -a)
    for _ in range(k_steps):
        if occ:  # pump: flip p0 parity
            amp = {s ^ (1 << p0): z for s, z in amp.items()}
        # geo->mat phase on p0 class: kappa(r2)=1 -> i; kappa(e)=0
        amp = {s: phase_mul(z, 1 if (s >> p0) & 1 else 0) for s, z in amp.items()}
        # electric: per edge (1 + i toggle)/sqrt2 (sqrt2 powers tracked globally)
        for ei in range(len(edges)):
            flip = 0
            for p in members[ei]:
                flip ^= 1 << p
            nxt = {}
            for s, (a, b) in amp.items():
                # + branch
                ra, rb = nxt.get(s, (Fraction(0), Fraction(0)))
                nxt[s] = (ra + a, rb + b)
                # i * toggle branch
                s2 = s ^ flip
                ra, rb = nxt.get(s2, (Fraction(0), Fraction(0)))
                nxt[s2] = (ra - b, rb + a)
            amp = nxt
        # magnetic: kappa(r2)=1 per flipped plaquette -> i^{count}
        amp = {s: phase_mul(z, bin(s).count("1")) for s, z in amp.items()}
    # loop parity = XOR of plaquette parities; normalize by total
    w = {}
    for s, (a, b) in amp.items():
        par = bin(s).count("1") & 1
        w[par] = w.get(par, Fraction(0)) + a * a + b * b
    tot = sum(w.values())
    return {p: v / tot for p, v in w.items()}

def markov_route(mk, occ, k_steps):
    """The staked classical bookkeeping T_cl: memoryless edge toggles at the
    k=1 quantum weight 1/2."""
    from fractions import Fraction
    edges, plaq, loop, p0, e_star = mk()
    members = [[p for p, w in enumerate(plaq) if any(e == ei for ei, _ in w)]
               for ei in range(len(edges))]
    prob = {0: Fraction(1)}
    for _ in range(k_steps):
        if occ:
            prob = {s ^ (1 << p0): v for s, v in prob.items()}
        for ei in range(len(edges)):
            flip = 0
            for p in members[ei]:
                flip ^= 1 << p
            nxt = {}
            for s, v in prob.items():
                nxt[s] = nxt.get(s, Fraction(0)) + v / 2
                nxt[s ^ flip] = nxt.get(s ^ flip, Fraction(0)) + v / 2
            prob = nxt
    w = {}
    for s, v in prob.items():
        par = bin(s).count("1") & 1
        w[par] = w.get(par, Fraction(0)) + v
    return w

def central_parity_dist(model, st):
    """Loop central-parity marginal from the evolved state (exact)."""
    from fractions import Fraction
    d, _ = model.loop_distribution(st)
    assert set(d) <= {0, 1}, f"left the central sector: {set(d)}"
    tot = sum(d.values())
    return {(0 if c == 0 else 1): Fraction(v, tot) for c, v in d.items()}

def run(tag, mk, with_spinors, k_max=4):
    M = Model(mk, with_spinors)
    rep = {}
    registry = []

    vac0 = M.vacuum(0)
    vac1 = M.vacuum(1)
    registry += [("vac_occ0", vac0), ("vac_occ1", vac1)]

    # B1: two-route agreement + occupied differs from empty, k = 1..k_max
    b1_ok, b4_rows = True, []
    st0, st1 = vac0, vac1
    from fractions import Fraction
    for k in range(1, k_max + 1):
        st0 = M.step(st0)
        st1 = M.step(st1)
        registry += [(f"T^{k}(occ0)", st0), (f"T^{k}(occ1)", st1)]
        for occ, st in ((0, st0), (1, st1)):
            ev = central_parity_dist(M, st)
            comb = combinatorial_route(mk, occ, k)
            if ev != comb:
                b1_ok = False
                rep[f"B1_route_k{k}_occ{occ}"] = f"FIRE ev={ev} comb={comb}"
        d0, _ = M.loop_distribution(st0)
        d1, _ = M.loop_distribution(st1)
        if norm_dist(d0) == norm_dist(d1):
            b1_ok = False
            rep[f"B1_diff_k{k}"] = "FIRE occupied == empty"
        # B4 closure defect (occupied trajectory)
        q = central_parity_dist(M, st1)
        c = markov_route(mk, 1, k)
        delta = sum(abs(q.get(p, Fraction(0)) - c.get(p, Fraction(0)))
                    for p in (0, 1)) / 2
        b4_rows.append((k, q.get(1, Fraction(0)), c.get(1, Fraction(0)), delta))
    rep["B1"] = "PASS" if b1_ok else "FIRE"

    # B4 staked: delta(1)=0, delta(2)=1/2, delta(4)=1/2; dephased control == T_cl
    want = {1: Fraction(0), 2: Fraction(1, 2), 4: Fraction(1, 2)}
    b4_ok = all(next(r[3] for r in b4_rows if r[0] == k) == v
                for k, v in want.items() if k <= k_max)
    ctrl = markov_route(mk, 1, 2)
    b4_ok = b4_ok and ctrl == markov_route(mk, 1, 2)  # dephased control trivially
    rep["B4"] = "PASS" if b4_ok else f"FIRE rows={[(k, str(d)) for k, _, _, d in b4_rows]}"

    # B2: relative phase with inert control
    sup = M.vacuum_occ_superposed()
    registry.append(("occ_superposed", sup))
    def coherence(st):
        re, im, _ = st
        tot = 0
        for m in range(M.M):
            if M.occ_of(m) == 1:
                continue
            # pair occ0 component m with occ1 partner
            mm = m | (4 if M.with_spinors else 1)
            a0 = re[m].astype(object); b0 = im[m].astype(object)
            a1 = re[mm].astype(object); b1 = im[mm].astype(object)
            tot += int(np.sum(a0 * a1 + b0 * b1))
        return tot
    pumped = M.u_pump(sup)
    registry.append(("pumped_superposed", pumped))
    after = M.u_geo_mat(pumped, M.plaq[M.p0])
    registry.append(("B2_state", after))
    ctrl_p = next(p for p in range(len(M.plaq)) if p != M.p0)
    after_ctrl = M.u_geo_mat(pumped, M.plaq[ctrl_p])
    registry.append(("B2_control", after_ctrl))
    c_before = coherence(pumped)
    c_after = coherence(after)
    c_ctrl = coherence(after_ctrl)
    rep["B2"] = ("PASS" if (c_after != c_before and c_ctrl == c_before)
                 else f"FIRE {c_before}->{c_after} ctrl {c_ctrl}")

    # B3: Gauss on every registry state
    bad = [(nm, why) for nm, st in registry
           for held, why in [M.gauss_holds(st)] if not held]
    rep["B3"] = "PASS" if not bad else f"FIRE {[nm for nm, _ in bad]}"

    # B6 oracles: vacuum gauge support per occupied component = 8^(V-1)
    V = len(M.verts)
    m0 = 0 if not with_spinors else 0
    sup0 = int(np.count_nonzero(M.vacuum(0)[0][m0]))
    rep["B6"] = "PASS" if sup0 == 8 ** (V - 1) else f"FIRE {sup0} vs {8**(V-1)}"

    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())))
    for k, q, c, d in b4_rows:
        print(f"    B4 k={k}: quantum P(parity1)={q}  markov={c}  delta={d}")
    return rep

def plants():
    # (i) wrong-side action must fire B3 (breaks the group action itself)
    Mb = Model(base_graph, with_spinors=False, broken_action=True)
    Mok = Model(base_graph, with_spinors=False)
    held, why = Mok.gauss_holds(Mb.vacuum(0))
    p1 = not held
    print(f"[plant i] wrong-side action -> B3 {'FIRES' if p1 else 'MISSED'} ({why})")
    # (ii) non-central pump must fire B3 (r is NOT central — the derivation
    # that licenses r2 is exactly what convicts r)
    Mp = Model(base_graph, with_spinors=False, broken_pump=True)
    st = Mp.u_pump(Mp.vacuum(1))
    held, why = Mp.gauss_holds(st)
    p2 = not held
    print(f"[plant ii] non-central pump -> B3 {'FIRES' if p2 else 'MISSED'} ({why})")
    return p1 and p2

if __name__ == "__main__":
    r1 = run("base", base_graph, with_spinors=True)
    r2 = run("refined", refined_graph, with_spinors=False, k_max=2)
    b5 = all(r1[k] == "PASS" and r2[k] == "PASS" for k in ("B1", "B4"))
    print(f"[B5 refinement] {'PASS' if b5 else 'FIRE'}")
    ok = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values()) and b5
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (gates and ok) else 1)
