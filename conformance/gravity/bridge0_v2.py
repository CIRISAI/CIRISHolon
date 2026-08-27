#!/usr/bin/env python3
"""BRIDGE0-V2 instrument. Prereg: GRAVITY_BRIDGE0_V2_PREREG.md, frozen at
commit ca42b33 BEFORE this file existed. Exact integer arithmetic; states
are int64 arrays of shape (3, N): matter row ⊗ gauge configuration."""
import numpy as np
import sys

PERMS = [(0, 1, 2), (1, 2, 0), (2, 0, 1), (0, 2, 1), (2, 1, 0), (1, 0, 2)]
IDX = {p: i for i, p in enumerate(PERMS)}
E_ID = 0

def _compose(p, q):
    return tuple(p[q[x]] for x in range(3))

MUL = np.array([[IDX[_compose(PERMS[a], PERMS[b])] for b in range(6)] for a in range(6)],
               dtype=np.int64)
INV = np.array([IDX[tuple(np.argsort(PERMS[a]))] for a in range(6)], dtype=np.int64)
CLASS = np.array([0, 1, 1, 2, 2, 2], dtype=np.int64)  # 0=e, 1=rho, 2=tau
CLASSES = {0: [0], 1: [1, 2], 2: [3, 4, 5]}
# Matter rows: 0 = |0>, 1 = |rho>, 2 = |tau>; C_m per row:
ROW_CLASS = {0: 0, 1: 1, 2: 2}

def base_graph():
    edges = [("c", 1), ("c", 2), ("c", 3), (1, 2), (2, 3), (3, 1)]
    plaq = [
        [(0, +1), (3, +1), (1, -1)],
        [(1, +1), (4, +1), (2, -1)],
        [(2, +1), (5, +1), (0, -1)],
    ]
    loop = [(3, +1), (4, +1), (5, +1)]
    return edges, plaq, loop, 0, 3  # e* = edge 3 (1->2)

def refined_graph():
    edges = [("c", 1), ("c", 2), ("c", 3), (1, 4), (4, 2), (2, 3), (3, 1), ("c", 4)]
    plaq = [
        [(0, +1), (3, +1), (7, -1)],
        [(7, +1), (4, +1), (1, -1)],
        [(1, +1), (5, +1), (2, -1)],
        [(2, +1), (6, +1), (0, -1)],
    ]
    loop = [(3, +1), (4, +1), (5, +1), (6, +1)]
    return edges, plaq, loop, 0, 3  # e* = edge 3 (1->4), sole edge of p0's rim

class Disk:
    def __init__(self, edges, plaq, loop, p0, e_star,
                 broken_word=False, broken_action=False):
        self.edges, self.plaq, self.loop = edges, plaq, loop
        self.p0, self.e_star = p0, e_star
        self.E = len(edges)
        self.N = 6 ** self.E
        self.verts = sorted({v for e in edges for v in e}, key=str)
        cfg = np.arange(self.N, dtype=np.int64)
        self.dig = np.empty((self.E, self.N), dtype=np.int64)
        for e in range(self.E):
            self.dig[e] = cfg % 6
            cfg //= 6
        self.pow6 = np.array([6 ** e for e in range(self.E)], dtype=np.int64)
        self.broken_word = broken_word
        self.broken_action = broken_action
        # M1 belt-and-braces precondition: e* belongs to exactly one plaquette.
        owners = [p for p, w in enumerate(plaq) if any(e == e_star for e, _ in w)]
        assert owners == [p0], f"e* must rim only p0, owners={owners}"

    def word_hol(self, word):
        h = np.zeros(self.N, dtype=np.int64)
        for e, s in word:
            g = self.dig[e] if s > 0 else INV[self.dig[e]]
            if self.broken_word and s < 0:
                g = self.dig[e]
            h = MUL[h, g]
        return h

    def vertex_index_map(self, v, g):
        idx = np.zeros(self.N, dtype=np.int64)
        for e, (a, b) in enumerate(self.edges):
            d = self.dig[e]
            if a == v:
                d = MUL[d, g] if self.broken_action else MUL[g, d]
            if b == v:
                d = MUL[d, INV[g]]
            idx += d * self.pow6[e]
        return idx

    def gauge_average_row(self, row, v):
        out = np.zeros_like(row)
        for g in range(6):
            idx = self.vertex_index_map(v, g)
            np.add.at(out, idx, row)
        return out

    def gauge_average(self, psi, v):
        return np.stack([self.gauge_average_row(psi[m], v) for m in range(3)])

    def left_mult_edge_row(self, row, e, h):
        d = MUL[h, self.dig[e]]
        idx = (np.sum(self.dig * self.pow6[:, None], axis=0)
               - self.dig[e] * self.pow6[e] + d * self.pow6[e])
        out = np.zeros_like(row)
        np.add.at(out, idx, row)
        return out

    # ------------------------------------------------------- the operators
    def K(self, row, cls):
        """K_C = sum over the class of left-multiplication at e* (prereg
        derivation: Gauss-commuting by class conjugation-invariance;
        flatness off p0 because e* rims only p0)."""
        if cls == 0:
            return row.copy()
        out = np.zeros_like(row)
        for h in CLASSES[cls]:
            out += self.left_mult_edge_row(row, self.e_star, h)
        return out

    def T(self, psi):
        """The joint coupling: row m gets K_{C_m}."""
        return np.stack([self.K(psi[m], ROW_CLASS[m]) for m in range(3)])

    def X_shift(self, psi, frm, to):
        """Independent matter operator: move row `frm` to row `to`."""
        out = psi.copy()
        out[to] = out[to] + out[frm]
        out[frm] = 0
        return out

    def G_op(self, psi, word):
        """Geometry->matter: class of hol(word) controls the cyclic matter
        shift S_c (shift by class index, mod 3)."""
        cls = CLASS[self.word_hol(word)]
        out = np.zeros_like(psi)
        for c in range(3):
            mask = cls == c
            for m in range(3):
                out[(m + c) % 3][mask] += psi[m][mask]
        return out

    # --------------------------------------------------------- state prep
    def vacuum(self, row=0):
        psi = np.zeros((3, self.N), dtype=np.int64)
        psi[row][0] = 1
        for v in self.verts:
            psi = self.gauge_average(psi, v)
        return psi

    # ----------------------------------------------------------- readings
    def loop_classes(self, psi, word=None):
        word = self.loop if word is None else word
        h = CLASS[self.word_hol(word)]
        return {m: set(h[psi[m] != 0].tolist()) for m in range(3) if np.any(psi[m])}

    def occupied_rows(self, psi):
        return {m for m in range(3) if np.any(psi[m])}

    def constraints_held(self, psi):
        """Full check per prereg G5: Gauss every vertex per occupied row;
        flatness everywhere except p0, where row m requires class C_m."""
        for v in self.verts:
            if not np.array_equal(self.gauge_average(psi, v), 6 * psi):
                return False, f"Gauss fails at {v}"
        for p in range(len(self.plaq)):
            h = CLASS[self.word_hol(self.plaq[p])]
            for m in range(3):
                if not np.any(psi[m]):
                    continue
                want = ROW_CLASS[m] if p == self.p0 else 0
                if np.any(psi[m][h != want] != 0):
                    return False, f"plaquette {p} row {m} not class {want}"
        return True, "held"

def run(tag, mk):
    edges, plaq, loop, p0, e_star = mk()
    D = Disk(edges, plaq, loop, p0, e_star)
    V = len(D.verts)
    registry = []  # (name, state) — G5 iterates EVERYTHING here (M1)
    rep = {}

    vac = D.vacuum(row=0)
    registry.append(("vacuum", vac))
    rep["G1"] = "PASS" if D.loop_classes(vac) == {0: {0}} else f"FIRE {D.loop_classes(vac)}"
    sup_vac = int(np.count_nonzero(vac[0]))
    rep["G7_vac"] = "PASS" if sup_vac == 6 ** (V - 1) else f"FIRE {sup_vac}"

    # G2: curvature from the coupling
    psis = {}
    for m, name, csize in ((1, "rho", 2), (2, "tau", 3)):
        start = D.vacuum(row=m)
        registry.append((f"vac_row_{name}", start))
        psi = D.T(start)
        registry.append((f"T_{name}", psi))
        psis[m] = psi
        cl = D.loop_classes(psi)
        rep[f"G2_{name}"] = "PASS" if cl == {m: {m}} else f"FIRE {cl}"
        sup = int(np.count_nonzero(psi[m]))
        want = csize * 6 ** (V - 1)
        rep[f"G7_{name}"] = "PASS" if sup == want else f"FIRE {sup} vs {want}"
        rev = [(e, -s) for (e, s) in reversed(D.loop)]
        hf, hr = D.word_hol(D.loop), D.word_hol(rev)
        supm = psi[m] != 0
        rep[f"G3_{name}"] = "PASS" if np.array_equal(hr[supm], INV[hf[supm]]) else "FIRE"

    # G4 reciprocity, all operators physical
    # (i) matter->geometry: X then T moves the distant reading
    x = D.X_shift(vac, 0, 1)
    registry.append(("X_rho(vacuum)", x))
    tx = D.T(x)
    registry.append(("T(X_rho(vacuum))", tx))
    no_x = D.T(vac)
    registry.append(("T(vacuum)", no_x))
    g4i = D.loop_classes(tx) == {1: {1}} and D.loop_classes(no_x) == {0: {0}}
    # (ii) geometry->matter: G[loop around p0] shifts psi_rho's matter row
    g_on = D.G_op(psis[1], D.loop)
    registry.append(("G[loop](T_rho)", g_on))
    g4ii = D.occupied_rows(g_on) == {(1 + 1) % 3} and D.occupied_rows(psis[1]) == {1}
    # (iii) control: G[boundary of a flat outer plaquette] is inert
    outer = next(p for p in range(len(D.plaq)) if p != D.p0)
    g_ctrl = D.G_op(psis[1], D.plaq[outer])
    registry.append(("G[ctrl](T_rho)", g_ctrl))
    g4iii = np.array_equal(g_ctrl, psis[1])
    rep["G4"] = "PASS" if (g4i and g4ii and g4iii) else f"FIRE i={g4i} ii={g4ii} iii={g4iii}"

    # G5: FULL SCOPE over the registry
    bad = [(nm, why) for nm, st in registry
           for held, why in [D.constraints_held(st)] if not held]
    rep["G5"] = "PASS" if not bad else f"FIRE {bad}"

    # G7b: matter operators commute with every gauge constraint (M3):
    # X and S act on rows only; verify on a probe state exactly.
    probe = psis[1]
    lhs = D.gauge_average(D.X_shift(probe, 1, 2), D.verts[0])
    rhs = D.X_shift(D.gauge_average(probe, D.verts[0]), 1, 2)
    rep["G7b"] = "PASS" if np.array_equal(lhs, rhs) else "FIRE"

    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())))
    return rep

def plants():
    edges, plaq, loop, p0, e_star = base_graph()
    # (i) wrong-side action must FIRE G5 (observability: breaks the group
    # action itself, so orbit weights go non-uniform — v1 record).
    Dbad = Disk(edges, plaq, loop, p0, e_star, broken_action=True)
    Dok = Disk(edges, plaq, loop, p0, e_star)
    held, why = Dok.constraints_held(Dbad.vacuum())
    p1 = not held
    print(f"[plant i] wrong-side action -> G5 {'FIRES' if p1 else 'MISSED'} ({why})")
    # (ii) dropped inverse read on the rho sector must FIRE the G2 reading
    # (observability: parity-even sector, the identity leaks — v1 derivation).
    Db = Disk(edges, plaq, loop, p0, e_star, broken_word=True)
    psi = Db.T(Db.vacuum(row=1))
    cl = Db.loop_classes(psi)
    p2 = cl != {1: {1}}
    print(f"[plant ii] broken word on rho -> {'FIRES' if p2 else 'MISSED'} ({cl})")
    return p1 and p2

if __name__ == "__main__":
    r1 = run("base", base_graph)
    r2 = run("refined", refined_graph)
    g6 = all(r1[k] == "PASS" and r2[k] == "PASS"
             for k in ("G1", "G2_rho", "G2_tau", "G4"))
    print(f"[G6 refinement] {'PASS' if g6 else 'FIRE'}")
    ok = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values()) and g6
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (gates and ok) else 1)
