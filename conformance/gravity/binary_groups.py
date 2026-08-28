#!/usr/bin/env python3
"""RECON for PT-2O / PT-2I: exact binary-octahedral (order 48) and
binary-icosahedral (order 120) group tables, conjugacy classes with their
SU(2) deficit angles, commutator SET and commutator SUBGROUP, and the
predicted realizable mass spectrum on the once-punctured one-vertex torus.

EXACT ARITHMETIC ONLY for every structural decision.  Quaternions are stored
SCALED BY 2, with each coordinate an element of a real quadratic ring held as
an integer pair:

    2O :  Z[sqrt2]  -- pair (p,q) means p + q*sqrt2
    2I :  Z[phi]    -- pair (p,q) means p + q*phi,  phi^2 = phi + 1

Floats appear ONLY to LABEL angles and to cross-check the exact angle
dictionary; no membership, class, subgroup or spectrum decision reads a float.

Everything is verified BY EXHAUSTION: closure, associativity, identity,
inverses, order, classes, commutators.
"""
import json
import math
import sys
from itertools import permutations, product

import numpy as np

# --------------------------------------------------------------------------
# ring arithmetic:  elements are integer pairs (p, q)
# --------------------------------------------------------------------------


def mul_sqrt2(a, b):
    """(p1+q1*r2)(p2+q2*r2) = (p1p2 + 2q1q2) + (p1q2+q1p2)*r2"""
    return (a[0] * b[0] + 2 * a[1] * b[1], a[0] * b[1] + a[1] * b[0])


def mul_phi(a, b):
    """(p1+q1*phi)(p2+q2*phi), phi^2 = phi+1
       = (p1p2 + q1q2) + (p1q2 + q1p2 + q1q2)*phi"""
    return (a[0] * b[0] + a[1] * b[1],
            a[0] * b[1] + a[1] * b[0] + a[1] * b[1])


def add(a, b):
    return (a[0] + b[0], a[1] + b[1])


def sub(a, b):
    return (a[0] - b[0], a[1] - b[1])


def neg(a):
    return (-a[0], -a[1])


def half(a, where):
    """exact division by 2 in the ring; refuses if not exact.
    {1,sqrt2} and {1,phi} are Z-bases, so 2*z has BOTH coefficients even."""
    if a[0] % 2 or a[1] % 2:
        raise ArithmeticError(f"inexact halving {a} at {where}")
    return (a[0] // 2, a[1] // 2)


VAL_SQRT2 = lambda a: a[0] + a[1] * math.sqrt(2.0)
VAL_PHI = lambda a: a[0] + a[1] * (1.0 + math.sqrt(5.0)) / 2.0

# --------------------------------------------------------------------------
# quaternions over the ring:  8-int flat tuple (w,x,y,z) each a ring pair
# --------------------------------------------------------------------------


def qmul(P, Q, rmul):
    """P, Q are SCALED-BY-2 unit quaternions.  P*Q = 4*p*q = 2*(scaled p*q),
    so the scaled product is (P*Q)/2, halved exactly in the ring."""
    p = [(P[0], P[1]), (P[2], P[3]), (P[4], P[5]), (P[6], P[7])]
    q = [(Q[0], Q[1]), (Q[2], Q[3]), (Q[4], Q[5]), (Q[6], Q[7])]
    w1, x1, y1, z1 = p
    w2, x2, y2, z2 = q
    m = rmul
    w = sub(sub(sub(m(w1, w2), m(x1, x2)), m(y1, y2)), m(z1, z2))
    x = add(add(add(m(w1, x2), m(x1, w2)), m(y1, z2)), neg(m(z1, y2)))
    y = add(add(add(m(w1, y2), neg(m(x1, z2))), m(y1, w2)), m(z1, x2))
    z = add(add(add(m(w1, z2), m(x1, y2)), neg(m(y1, x2))), m(z1, w2))
    out = []
    for c in (w, x, y, z):
        h = half(c, "qmul")
        out.extend(h)
    return tuple(out)


def Z(n):
    """rational integer n as a ring pair"""
    return (n, 0)


# --------------------------------------------------------------------------
# element lists (SCALED BY 2)
# --------------------------------------------------------------------------


def hurwitz_scaled():
    """the 24 elements of 2T, scaled by 2: 2*(+-1,+-i,+-j,+-k) and
    2*(+-1+-i+-j+-k)/2 = (+-1,+-1,+-1,+-1)."""
    out = []
    for pos in range(4):
        for s in (2, -2):
            c = [Z(0)] * 4
            c[pos] = Z(s)
            out.append(tuple(v for pair in c for v in pair))
    for signs in product((1, -1), repeat=4):
        c = [Z(s) for s in signs]
        out.append(tuple(v for pair in c for v in pair))
    assert len(out) == 24 and len(set(out)) == 24
    return out


def build_2O():
    """2T  U  { 2 * (+-e_a +- e_b)/sqrt2 } = 2T U { +-sqrt2 e_a +- sqrt2 e_b }"""
    out = list(hurwitz_scaled())
    R2 = (0, 1)          # sqrt2
    for a in range(4):
        for b in range(a + 1, 4):
            for sa in (1, -1):
                for sb in (1, -1):
                    c = [Z(0)] * 4
                    c[a] = (0, sa)
                    c[b] = (0, sb)
                    out.append(tuple(v for pair in c for v in pair))
    assert len(out) == 48 and len(set(out)) == 48, len(set(out))
    return out, mul_sqrt2, VAL_SQRT2, "Z[sqrt2]"


def build_2I():
    """2T  U  { all EVEN permutations of (+-phi, +-1, +-1/phi, 0) },
    scaled by 2 so entries are phi, 1, phi-1, 0 in Z[phi]  (1/phi = phi-1)."""
    out = list(hurwitz_scaled())
    base = [(0, 1), (1, 0), (-1, 1), (0, 0)]      # phi, 1, phi-1, 0
    even = [p for p in permutations(range(4))
            if sum(1 for i in range(4) for j in range(i + 1, 4)
                   if p[i] > p[j]) % 2 == 0]
    assert len(even) == 12
    for p in even:
        for signs in product((1, -1), repeat=3):
            c = [None] * 4
            vals = [(signs[0] * base[0][0], signs[0] * base[0][1]),
                    (signs[1] * base[1][0], signs[1] * base[1][1]),
                    (signs[2] * base[2][0], signs[2] * base[2][1]),
                    base[3]]
            for slot in range(4):
                c[p[slot]] = vals[slot]
            out.append(tuple(v for pair in c for v in pair))
    assert len(out) == 120, len(out)
    assert len(set(out)) == 120, len(set(out))
    return out, mul_phi, VAL_PHI, "Z[phi]"


# --------------------------------------------------------------------------
# group construction + exhaustive axiom checks
# --------------------------------------------------------------------------


class Grp:
    def __init__(self, name, elems, rmul, val, ring):
        self.name = name
        self.elems = elems
        self.rmul = rmul
        self.val = val
        self.ring = ring
        self.n = len(elems)
        self.idx = {e: k for k, e in enumerate(elems)}
        self.MUL = np.zeros((self.n, self.n), dtype=np.int64)
        for a in range(self.n):
            for b in range(self.n):
                prod = qmul(elems[a], elems[b], rmul)
                if prod not in self.idx:                     # CLOSURE, exhaustive
                    raise AssertionError(f"{name}: not closed at {a},{b} -> {prod}")
                self.MUL[a, b] = self.idx[prod]
        one = tuple(v for pair in (Z(2), Z(0), Z(0), Z(0)) for v in pair)
        self.e = self.idx[one]
        self.checks = {}
        self._axioms()
        self._classes()
        self._orders()

    def _axioms(self):
        n, M = self.n, self.MUL
        # identity, exhaustive
        assert all(M[self.e, g] == g and M[g, self.e] == g for g in range(n))
        # inverses, exhaustive
        self.INV = np.zeros(n, dtype=np.int64)
        for g in range(n):
            hs = [h for h in range(n) if M[g, h] == self.e]
            assert len(hs) == 1, f"inverse not unique for {g}"
            assert M[hs[0], g] == self.e
            self.INV[g] = hs[0]
        # associativity, FULL exhaustion over n^3
        A = M[M[:, :, None], np.arange(n)[None, None, :]]
        B = M[np.arange(n)[:, None, None], M[:, :]]
        assert np.array_equal(A, B), "associativity fails"
        # latin square (cancellation), exhaustive
        for g in range(n):
            assert len(set(M[g, :].tolist())) == n
            assert len(set(M[:, g].tolist())) == n
        self.checks["closure"] = f"exhaustive {n}^2 = {n*n} products land in the set"
        self.checks["associativity"] = f"exhaustive {n}^3 = {n**3} triples"
        self.checks["identity_inverses"] = f"exhaustive, unique two-sided inverse for all {n}"
        self.checks["order"] = n

    def _classes(self):
        n, M, I = self.n, self.MUL, self.INV
        self.CLS = np.full(n, -1, dtype=np.int64)
        self.reps = []
        lab = 0
        for g in range(n):
            if self.CLS[g] >= 0:
                continue
            orb = {int(M[M[x, g], I[x]]) for x in range(n)}
            for h in orb:
                self.CLS[h] = lab
            self.reps.append(g)
            lab += 1
        self.ncls = lab
        assert (self.CLS >= 0).all()

    def _orders(self):
        self.ORD = np.zeros(self.n, dtype=np.int64)
        for g in range(self.n):
            k, x = 1, g
            while x != self.e:
                x = int(self.MUL[x, g])
                k += 1
            self.ORD[g] = k

    # ---- structural computations -----------------------------------------
    def commutator_set(self):
        """SET of single commutators [a,b] = a b a^-1 b^-1, exhaustive n^2,
        with the Frobenius count |{(a,b): [a,b]=g}| for every g."""
        M, I, n = self.MUL, self.INV, self.n
        cnt = np.zeros(n, dtype=np.int64)
        for a in range(n):
            for b in range(n):
                c = int(M[M[M[a, b], I[a]], I[b]])
                cnt[c] += 1
        return set(int(g) for g in np.nonzero(cnt)[0]), cnt

    def generated(self, seed):
        """subgroup generated by seed, by exhaustive closure."""
        S = set(int(s) for s in seed) | {int(self.e)}
        changed = True
        while changed:
            changed = False
            for a in list(S):
                for b in list(S):
                    p = int(self.MUL[a, b])
                    if p not in S:
                        S.add(p)
                        changed = True
        for g in S:                                  # inverse-closed, exhaustive
            assert int(self.INV[g]) in S
        return S

    def scalar(self, g):
        """exact SCALED-BY-2 scalar part as a ring pair; true w = this / 2."""
        e = self.elems[g]
        return (e[0], e[1])


# --------------------------------------------------------------------------
# exact deficit-angle dictionary, keyed by the SCALED scalar part (2w)
# --------------------------------------------------------------------------

ANGLE_SQRT2 = {
    (2, 0):  ("0", 0.0),
    (-2, 0): ("2*pi", 2 * math.pi),
    (0, 0):  ("pi", math.pi),
    (1, 0):  ("2*pi/3", 2 * math.pi / 3),
    (-1, 0): ("4*pi/3", 4 * math.pi / 3),
    (0, 1):  ("pi/2", math.pi / 2),
    (0, -1): ("3*pi/2", 3 * math.pi / 2),
}
ANGLE_PHI = {
    (2, 0):  ("0", 0.0),
    (-2, 0): ("2*pi", 2 * math.pi),
    (0, 0):  ("pi", math.pi),
    (1, 0):  ("2*pi/3", 2 * math.pi / 3),
    (-1, 0): ("4*pi/3", 4 * math.pi / 3),
    (0, 1):  ("2*pi/5", 2 * math.pi / 5),        # 2w = phi   -> w = cos(pi/5)
    (0, -1): ("8*pi/5", 8 * math.pi / 5),
    (-1, 1): ("4*pi/5", 4 * math.pi / 5),        # 2w = phi-1 -> w = cos(2pi/5)
    (1, -1): ("6*pi/5", 6 * math.pi / 5),
}

W_LABEL_SQRT2 = {(2, 0): "1", (-2, 0): "-1", (0, 0): "0", (1, 0): "1/2",
                 (-1, 0): "-1/2", (0, 1): "sqrt2/2", (0, -1): "-sqrt2/2"}
W_LABEL_PHI = {(2, 0): "1", (-2, 0): "-1", (0, 0): "0", (1, 0): "1/2",
               (-1, 0): "-1/2", (0, 1): "phi/2", (0, -1): "-phi/2",
               (-1, 1): "(phi-1)/2", (1, -1): "-(phi-1)/2"}


def analyse(G, table, wlab, tag):
    n = G.n
    comm_set, comm_cnt = G.commutator_set()
    derived = G.generated(comm_set)

    # scalar part is a class function -- VERIFY exhaustively, do not assume
    for c in range(G.ncls):
        members = [g for g in range(n) if G.CLS[g] == c]
        s = {G.scalar(g) for g in members}
        assert len(s) == 1, f"scalar part not constant on class {c}: {s}"

    rows = []
    for c in range(G.ncls):
        members = [g for g in range(n) if G.CLS[g] == c]
        rep = min(members)
        sc = G.scalar(rep)
        assert sc in table, f"unexpected scalar part {sc}"
        ang_exact, ang_val = table[sc]
        # float cross-check of the exact angle label (LABEL ONLY)
        w = G.val(sc) / 2.0
        w = max(-1.0, min(1.0, w))
        assert abs(2 * math.acos(w) - ang_val) < 1e-12, "angle label mismatch"
        in_derived = all(g in derived for g in members)
        any_derived = any(g in derived for g in members)
        assert in_derived == any_derived, "derived subgroup not a union of classes"
        meets_comm = any(g in comm_set for g in members)
        all_comm = all(g in comm_set for g in members)
        assert meets_comm == all_comm, "commutator SET not a union of classes"
        rows.append({
            "class_id": c,
            "rep_index": rep,
            "rep_quaternion_scaled_by_2": list(G.elems[rep]),
            "rep_readable": readable(G.elems[rep], tag),
            "class_size": len(members),
            "element_order": int(G.ORD[rep]),
            "scalar_2w_ring_pair": list(sc),
            "w_exact": wlab[sc],
            "deficit_angle_exact": ang_exact,
            "deficit_angle_float": ang_val,
            "in_commutator_subgroup": bool(in_derived),
            "is_commutator": bool(meets_comm),
            "frobenius_commutator_count": int(comm_cnt[rep]),
            "ambivalent": bool(G.CLS[G.INV[rep]] == c),
        })
    rows.sort(key=lambda r: (r["deficit_angle_float"], -r["class_size"]))
    return rows, comm_set, derived, comm_cnt


def readable(e, tag):
    names = ["", "i", "j", "k"]
    if tag == "sqrt2":
        sym, lab = "sqrt2", ["1", "sqrt2"]
    else:
        sym, lab = "phi", ["1", "phi"]
    parts = []
    for slot in range(4):
        p, q = e[2 * slot], e[2 * slot + 1]
        if p == 0 and q == 0:
            continue
        t = []
        if p:
            t.append(f"{p}")
        if q:
            t.append(f"{q:+d}*{sym}" if p else f"{q}*{sym}")
        s = "".join(t)
        parts.append(f"({s}){names[slot]}" if names[slot] else f"({s})")
    return "[ " + " + ".join(parts) + " ] / 2"


# --------------------------------------------------------------------------
# base punctured torus: realizable spectrum, by exhaustion over |G|^2
# --------------------------------------------------------------------------


def base_torus(G):
    n, M, I = G.n, G.MUL, G.INV
    N = n * n
    GA = np.arange(N) % n
    GB = np.arange(N) // n
    IDX = lambda a, b: a + n * b
    PUNCT = M[M[GA, GB], M[I[GA], I[GB]]]
    T_MAP = IDX(GA, M[GA, GB])
    S_MAP = IDX(GB, M[M[GB, GA], I[GB]])
    bij = (len(set(T_MAP.tolist())) == N and len(set(S_MAP.tolist())) == N)
    occupied = sorted({int(G.CLS[p]) for p in PUNCT})
    # Gauss projector on a class indicator: sum over simultaneous conjugation
    realizable = []
    for c in range(G.ncls):
        tgt = (G.CLS[PUNCT] == c).astype(np.int64)
        acc = np.zeros(N, dtype=np.int64)
        for x in range(n):
            perm = IDX(M[M[x, GA], I[x]], M[M[x, GB], I[x]])
            out = np.zeros(N, dtype=np.int64)
            np.add.at(out, perm, tgt)
            acc += out
        if np.count_nonzero(acc):
            realizable.append(c)
    return dict(N=N, bijective=bool(bij), occupied=occupied,
                realizable=sorted(realizable))


# --------------------------------------------------------------------------


def run(name, builder, table, wlab, tag):
    elems, rmul, val, ring = builder()
    G = Grp(name, elems, rmul, val, ring)
    rows, comm_set, derived, cnt = analyse(G, table, wlab, tag)
    bt = base_torus(G)

    hur = set(G.idx[e] for e in hurwitz_scaled() if e in G.idx)
    derived_is_2T = (derived == hur) and len(hur) == 24
    derived_is_all = (len(derived) == G.n)

    print(f"\n=== {name}  (order {G.n}, ring {ring}) ===")
    print(f"axioms: {G.checks}")
    print(f"conjugacy classes: {G.ncls}")
    print(f"commutator SET size      : {len(comm_set)}")
    print(f"commutator SUBGROUP size : {len(derived)}"
          f"   [== 2T? {derived_is_2T}]  [== G (perfect)? {derived_is_all}]")
    print(f"commutator set == commutator subgroup ? "
          f"{set(comm_set) == set(derived)}")
    print(f"base torus |G|^2 = {bt['N']}, Dehn steps bijective = {bt['bijective']}")
    print(f"classes occupied by some commutator : {bt['occupied']}")
    print(f"classes realizable after Gauss      : {bt['realizable']}")
    print(f"classes contained in [G,G]          : "
          f"{sorted({r['class_id'] for r in rows if r['in_commutator_subgroup']})}")
    hdr = (f"{'cls':>3} {'size':>4} {'ord':>4} {'w':>11} {'deficit':>9} "
           f"{'in[G,G]':>8} {'is_comm':>8} {'#[x,y]=g':>9} {'amb':>4}  rep")
    print(hdr)
    print("-" * len(hdr))
    for r in rows:
        print(f"{r['class_id']:>3} {r['class_size']:>4} {r['element_order']:>4} "
              f"{r['w_exact']:>11} {r['deficit_angle_exact']:>9} "
              f"{str(r['in_commutator_subgroup']):>8} {str(r['is_commutator']):>8} "
              f"{r['frobenius_commutator_count']:>9} {str(r['ambivalent']):>4}  "
              f"{r['rep_readable']}")

    spectrum = [r for r in rows if r["is_commutator"]]
    print(f"\nPREDICTED MASS SPECTRUM ({len(spectrum)} rungs), deficits: "
          f"{[r['deficit_angle_exact'] for r in spectrum]}")
    dis = sorted({r["deficit_angle_exact"] for r in spectrum},
                 key=lambda s: [r["deficit_angle_float"] for r in spectrum
                                if r["deficit_angle_exact"] == s][0])
    print(f"DISTINCT deficits: {len(dis)} -> {dis}")
    assert sorted(r["class_id"] for r in spectrum) == bt["realizable"], \
        "commutator-set prediction disagrees with the Gauss-projected sector test"

    doc = {
        "group": name,
        "order": G.n,
        "ring": ring,
        "n_conjugacy_classes": G.ncls,
        "axiom_checks": G.checks,
        "commutator_set_size": len(comm_set),
        "commutator_subgroup_size": len(derived),
        "commutator_set_equals_subgroup": bool(set(comm_set) == set(derived)),
        "commutator_subgroup_is_2T": bool(derived_is_2T),
        "group_is_perfect": bool(derived_is_all),
        "base_torus_configs": bt["N"],
        "base_dehn_steps_bijective": bt["bijective"],
        "realizable_class_ids": bt["realizable"],
        "n_rungs": len(spectrum),
        "n_distinct_deficits": len(dis),
        "distinct_deficits": dis,
        "classes": rows,
    }
    return G, doc


if __name__ == "__main__":
    out = {}
    _, out["2O"] = run("2O (binary octahedral)", build_2O,
                       ANGLE_SQRT2, W_LABEL_SQRT2, "sqrt2")
    _, out["2I"] = run("2I (binary icosahedral)", build_2I,
                       ANGLE_PHI, W_LABEL_PHI, "phi")

    # control: reproduce the PT-2T result with the same code path
    def build_2T():
        return hurwitz_scaled(), mul_sqrt2, VAL_SQRT2, "Z (subring of Z[sqrt2])"
    _, out["2T"] = run("2T (binary tetrahedral, CONTROL)", build_2T,
                       ANGLE_SQRT2, W_LABEL_SQRT2, "sqrt2")

    path = ("/tmp/claude-1000/-home-emoore-CIRISOntology/"
            "4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/rung5/"
            "rung5_spectra.json")
    with open(path, "w") as f:
        json.dump(out, f, indent=2)
    print(f"\nwrote {path}")
