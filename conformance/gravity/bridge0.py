#!/usr/bin/env python3
"""GRAVITY-BRIDGE-0 instrument: quantum double of S3 on a triangulated disk.

Exact integer arithmetic throughout: states are unnormalized int64 vectors
over edge-group configurations; gauge averages are integer sums; every gate
is an exact equality. No float appears in any value path. See
GRAVITY_BRIDGE0_PREREG.md — frozen before this file was written."""
import numpy as np
import sys

# ---------------------------------------------------------------- S3, exact
# Elements as permutations of (0,1,2), indexed 0..5.
PERMS = [(0, 1, 2), (1, 2, 0), (2, 0, 1), (0, 2, 1), (2, 1, 0), (1, 0, 2)]
IDX = {p: i for i, p in enumerate(PERMS)}
E_ID = 0

def _compose(p, q):  # (p∘q)(x) = p(q(x))
    return tuple(p[q[x]] for x in range(3))

MUL = np.array([[IDX[_compose(PERMS[a], PERMS[b])] for b in range(6)] for a in range(6)],
               dtype=np.int64)
INV = np.array([IDX[tuple(np.argsort(PERMS[a]))] for a in range(6)], dtype=np.int64)
# Conjugacy classes: e | 3-cycles rho (indices 1,2) | transpositions tau (3,4,5)
CLASS = np.array([0, 1, 1, 2, 2, 2], dtype=np.int64)  # 0=e, 1=rho, 2=tau
CLASS_NAME = {0: "e", 1: "rho(±120°)", 2: "tau(reflection)"}
RHO_REP, TAU_REP = 1, 3

# ------------------------------------------------------------- the graphs
# Edges as (tail, head). Plaquette = ordered list of (edge_index, +1|-1),
# holonomy = product of u_e^{±1} along the word. Vertex action of g at v:
# u_e -> g·u_e if tail==v ; u_e -> u_e·g^{-1} if head==v.
def base_graph():
    edges = [("c", 1), ("c", 2), ("c", 3), (1, 2), (2, 3), (3, 1)]
    plaq = [
        [(0, +1), (3, +1), (1, -1)],   # p0: c->1, 1->2, (c->2)^-1
        [(1, +1), (4, +1), (2, -1)],   # p1: c->2, 2->3, (c->3)^-1
        [(2, +1), (5, +1), (0, -1)],   # p2: c->3, 3->1, (c->1)^-1
    ]
    loop = [(3, +1), (4, +1), (5, +1)]  # 1->2->3->1
    return edges, plaq, loop, 0  # p0 index

def refined_graph():
    # split 1->2 at vertex 4, add spoke c->4
    edges = [("c", 1), ("c", 2), ("c", 3), (1, 4), (4, 2), (2, 3), (3, 1), ("c", 4)]
    plaq = [
        [(0, +1), (3, +1), (7, -1)],   # p0: c->1, 1->4, (c->4)^-1
        [(7, +1), (4, +1), (1, -1)],   # p0b: c->4, 4->2, (c->2)^-1
        [(1, +1), (5, +1), (2, -1)],   # c->2, 2->3, (c->3)^-1
        [(2, +1), (6, +1), (0, -1)],   # c->3, 3->1, (c->1)^-1
    ]
    loop = [(3, +1), (4, +1), (5, +1), (6, +1)]  # 1->4->2->3->1
    return edges, plaq, loop, 0

# --------------------------------------------------- vectorized machinery
class Disk:
    def __init__(self, edges, plaq, loop, p0, broken_word=False):
        self.edges, self.plaq, self.loop, self.p0 = edges, plaq, loop, p0
        self.E = len(edges)
        self.N = 6 ** self.E
        self.verts = sorted({v for e in edges for v in e}, key=str)
        cfg = np.arange(self.N, dtype=np.int64)
        self.dig = np.empty((self.E, self.N), dtype=np.int64)
        for e in range(self.E):
            self.dig[e] = cfg % 6
            cfg //= 6
        self.pow6 = np.array([6 ** e for e in range(self.E)], dtype=np.int64)
        self.broken_word = broken_word  # G8 plant (ii): drop one inverse
        self.broken_action = False      # G8 plant (i): wrong-side group action

    def word_hol(self, word):
        h = np.zeros(self.N, dtype=np.int64)  # identity
        for k, (e, s) in enumerate(word):
            g = self.dig[e] if s > 0 else INV[self.dig[e]]
            if self.broken_word and s < 0:
                g = self.dig[e]  # plant: forgot the inverse
            h = MUL[h, g]
        return h

    def flat_mask(self, p):
        return self.word_hol(self.plaq[p]) == E_ID

    def class_mask(self, p, cls):
        return CLASS[self.word_hol(self.plaq[p])] == cls

    def vertex_index_map(self, v, g):
        """Config permutation: apply g at vertex v (returns index array)."""
        idx = np.zeros(self.N, dtype=np.int64)
        for e, (a, b) in enumerate(self.edges):
            d = self.dig[e]
            if a == v:
                d = MUL[d, g] if self.broken_action else MUL[g, d]
            if b == v:
                d = MUL[d, INV[g]]
            idx += d * self.pow6[e]
        return idx

    def gauge_average(self, psi, v, skip=None):
        """Ā_v ψ = Σ_g ψ∘(action of g at v)⁻¹ — integer sum over 6 images
        (or 5, for the G8 plant)."""
        out = np.zeros_like(psi)
        for g in range(6):
            if skip is not None and g == skip:
                continue
            idx = self.vertex_index_map(v, g)
            # action permutes basis configs; push weights forward
            np.add.at(out, idx, psi)
        return out

    def edge_kick(self, psi, e, g):
        """Left-multiply edge e by g: a local holonomy intervention."""
        d = MUL[g, self.dig[e]]
        idx = (np.sum(self.dig * self.pow6[:, None], axis=0)
               - self.dig[e] * self.pow6[e] + d * self.pow6[e])
        out = np.zeros_like(psi)
        np.add.at(out, idx, psi)
        return out

    # ------------------------------------------------------ state builders
    def vacuum(self, broken_gauss_at=None):
        psi = np.zeros(self.N, dtype=np.int64)
        psi[0] = 1  # all edges identity: flat
        for v in self.verts:
            skip = 5 if (broken_gauss_at is not None and v == broken_gauss_at) else None
            psi = self.gauge_average(psi, v, skip=skip)
        return psi

    def flux(self, rep):
        """Pin p0's holonomy to the class of `rep` by seeding one edge."""
        psi = np.zeros(self.N, dtype=np.int64)
        # seed: the p0 word's first +1 boundary-side edge carries rep... we
        # seed edge = the second letter of p0's word (a boundary/split edge)
        e_seed = self.plaq[self.p0][1][0]
        psi[rep * self.pow6[e_seed]] = 1
        for v in self.verts:
            psi = self.gauge_average(psi, v)
        return psi

    # ---------------------------------------------------------- the gates
    def support_classes(self, psi, word):
        sup = psi != 0
        return set(CLASS[self.word_hol(word)[sup]].tolist()), int(np.count_nonzero(sup))

    def constraints_held(self, psi, pinned=None):
        for v in self.verts:
            if not np.array_equal(self.gauge_average(psi, v), 6 * psi):
                return False, f"Gauss fails at {v}"
        for p in range(len(self.plaq)):
            if pinned is not None and p == self.p0:
                m = self.class_mask(p, CLASS[pinned])
            else:
                m = self.flat_mask(p)
            if np.any(psi[~m] != 0):
                return False, f"plaquette {p} constraint fails"
        return True, "held"

def run(tag, mk):
    edges, plaq, loop, p0 = mk()
    D = Disk(edges, plaq, loop, p0)
    V = len(D.verts)
    report = {}

    # G1 vacuum flatness + G7 oracle + G5
    vac = D.vacuum()
    cls, sup = D.support_classes(vac, D.loop)
    report["G1"] = ("PASS" if cls == {0} else f"FIRE {cls}")
    oracle_vac = 6 ** (V - 1)
    report["G7_vac"] = ("PASS" if sup == oracle_vac else f"FIRE measured {sup} vs {oracle_vac}")
    held, why = D.constraints_held(vac)
    g5 = ["vac:" + ("ok" if held else why)]

    # G2 deficit, both classes + G7 flux oracles + G3 + G5
    for name, rep, csize in (("tau", TAU_REP, 3), ("rho", RHO_REP, 2)):
        st = D.flux(rep)
        cls, sup = D.support_classes(st, D.loop)
        want = {int(CLASS[rep])}
        report[f"G2_{name}"] = ("PASS" if cls == want else f"FIRE {cls} want {want}")
        oracle = csize * 6 ** (V - 1)
        report[f"G7_{name}"] = ("PASS" if sup == oracle else f"FIRE measured {sup} vs {oracle}")
        # G3: element-level inversion on every support config
        rev = [(e, -s) for (e, s) in reversed(D.loop)]
        hf, hr = D.word_hol(D.loop), D.word_hol(rev)
        supm = st != 0
        report[f"G3_{name}"] = ("PASS" if np.array_equal(hr[supm], INV[hf[supm]])
                                else "FIRE")
        held, why = D.constraints_held(st, pinned=rep)
        g5.append(f"{name}:" + ("ok" if held else why))
    report["G5"] = "PASS" if all(x.endswith("ok") for x in g5) else f"FIRE {g5}"

    # G4 reciprocity + locality control (base graph semantics)
    st_tau = D.flux(TAU_REP)
    st_rho = D.flux(RHO_REP)
    c_tau, _ = D.support_classes(st_tau, D.loop)
    c_rho, _ = D.support_classes(st_rho, D.loop)
    g4i = c_tau == {2} and c_rho == {1}
    # (ii) geometry kick on an edge of p0's boundary -> matter reading at p0 moves
    e_p0 = D.plaq[D.p0][1][0]
    # D3 group theory: rotation·reflection = reflection, so a rho kick cannot
    # move a tau flux's class — kick with a TRANSPOSITION (tau·tau ∈ {e, rho}).
    kicked = D.edge_kick(st_tau, e_p0, TAU_REP)
    before = set(CLASS[D.word_hol(D.plaq[D.p0])[st_tau != 0]].tolist())
    after = set(CLASS[D.word_hol(D.plaq[D.p0])[kicked != 0]].tolist())
    g4ii = after != before
    # (iii) control: kick an edge bounding only OUTER plaquettes -> p0 reading fixed
    outer_edges = {e for p in range(len(D.plaq)) if p != D.p0 for (e, _) in D.plaq[p]}
    p0_edges = {e for (e, _) in D.plaq[D.p0]}
    ctrl_e = sorted(outer_edges - p0_edges)[0]
    ctrl = D.edge_kick(st_tau, ctrl_e, RHO_REP)
    ctrl_after = set(CLASS[D.word_hol(D.plaq[D.p0])[ctrl != 0]].tolist())
    g4iii = ctrl_after == before
    report["G4"] = ("PASS" if (g4i and g4ii and g4iii)
                    else f"FIRE i={g4i} ii={g4ii} iii={g4iii}")

    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in report.items()))
    return report

def plants():
    edges, plaq, loop, p0 = base_graph()
    # Plant-history, kept per house rules (planted defects must be
    # OBSERVABLE, and two candidates were provably not): (a) skipping one
    # group element in a vertex average launders into uniform orbit weights
    # (each config's diagonal preimages hit the skipped element exactly
    # once); (b) omitting ONE vertex's Gauss enforcement is redundant on a
    # disk (the flat sector's global gauge kernel implies it from the rest);
    # (c) the tau-flux broken-word reading is PARITY-PROTECTED (odd
    # reflection count forces a reflection). The plants below are proved
    # observable and must FIRE.
    # (i) WRONG-SIDE group action in the gauge average (a realistic
    # implementation bug), checked by the CORRECT instrument.
    Dbad = Disk(edges, plaq, loop, p0)
    Dbad.broken_action = True
    Dok = Disk(edges, plaq, loop, p0)
    vac_bad = Dbad.vacuum()
    held, why = Dok.constraints_held(vac_bad)
    p1 = (not held)
    print(f"[plant i] wrong-side action -> G5 {'FIRES' if p1 else 'MISSED'} ({why})")
    # (ii) broken holonomy word on the RHO flux (parity-even sector, where
    # the identity provably leaks into the broken reading).
    Db = Disk(edges, plaq, loop, p0, broken_word=True)
    st = Db.flux(RHO_REP)
    cls = set(CLASS[Db.word_hol(Db.plaq[Db.p0])[st != 0]].tolist())
    p2 = cls != {1}
    print(f"[plant ii] broken word (rho flux) -> reading {'FIRES' if p2 else 'MISSED'} "
          f"(classes {cls})")
    return p1 and p2

if __name__ == "__main__":
    r1 = run("base", base_graph)
    r2 = run("refined", refined_graph)
    g6 = all(r1[k] == r2[k] == "PASS" for k in ("G1", "G2_tau", "G2_rho"))
    print(f"[G6 refinement] {'PASS' if g6 else 'FIRE'}")
    ok_plants = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values()) and g6
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE (harness can fail)' if ok_plants else 'VOID - a plant missed'}")
    sys.exit(0 if (gates and ok_plants) else 1)
