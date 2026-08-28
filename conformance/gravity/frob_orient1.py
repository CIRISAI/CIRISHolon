#!/usr/bin/env python3
"""FROB-ORIENT-1 instrument. Prereg ADMITTED and frozen before this file.
F21 = Z7:Z3 built from its presentation, verified by exhaustion (N0)."""
import sys
import numpy as np
from itertools import product

# ---- build F21: elements (i, j) = a^i b^j, i mod 7, j mod 3; b a b^-1 = a^2
# b^j a b^-j = a^(2^j)  =>  (i1,j1)*(i2,j2) = (i1 + i2*2^j1 mod 7, j1+j2 mod 3)
ELEMS = [(i, j) for j in range(3) for i in range(7)]
IDXOF = {e: k for k, e in enumerate(ELEMS)}
NG = 21
POW2 = [1, 2, 4]
MUL = np.zeros((NG, NG), dtype=np.int64)
for (i1, j1) in ELEMS:
    for (i2, j2) in ELEMS:
        MUL[IDXOF[(i1, j1)], IDXOF[(i2, j2)]] = IDXOF[((i1 + i2 * POW2[j1]) % 7, (j1 + j2) % 3)]
INV = np.zeros(NG, dtype=np.int64)
for g in range(NG):
    for h in range(NG):
        if MUL[g, h] == IDXOF[(0, 0)]:
            INV[g] = h
IDE = IDXOF[(0, 0)]

def n0_checks():
    # associativity by exhaustion
    for x, y, z in product(range(NG), repeat=3):
        if MUL[MUL[x, y], z] != MUL[x, MUL[y, z]]:
            return False, "associativity fails"
    # class table
    cls = np.full(NG, -1, dtype=np.int64)
    lab = 0
    for g in range(NG):
        if cls[g] >= 0:
            continue
        orbit = {MUL[MUL[x, g], INV[x]] for x in range(NG)}
        for h in orbit:
            cls[h] = lab
        lab += 1
    # non-ambivalence: some class != its inverse class
    nonamb = any(cls[g] != cls[INV[g]] for g in range(NG))
    return (nonamb, cls) if nonamb else (False, "ambivalent?!")

OK, CLS = n0_checks()
N_CLS = int(CLS.max()) + 1

# ---- one triangular plaquette: edges e0,e1,e2, holonomy g0 g1 g2
N = NG ** 3
D0 = np.arange(N) % NG
D1 = (np.arange(N) // NG) % NG
D2 = np.arange(N) // (NG * NG)
HOL = MUL[MUL[D0, D1], D2]
IDX3 = lambda a, b, c: a + NG * b + NG * NG * c
VERTS = [0, 1, 2]  # v0: e2 in, e0 out ; v1: e0 in, e1 out ; v2: e1 in, e2 out

def gauge_perm(v, x):
    a, b, c = D0, D1, D2
    if v == 0:
        a2, c2 = MUL[x, a], MUL[c, INV[x]]
        return IDX3(a2, b, c2)
    if v == 1:
        b2, a2 = MUL[x, b], MUL[a, INV[x]]
        return IDX3(a2, b2, c)
    c2, b2 = MUL[x, c], MUL[b, INV[x]]
    return IDX3(a, b2, c2)

def apply_perm(psi, perm):
    out = np.zeros_like(psi)
    np.add.at(out, perm, psi)
    return out

def gauss_project(psi):
    for v in VERTS:
        acc = np.zeros_like(psi)
        for x in range(NG):
            acc += apply_perm(psi, gauge_perm(v, x))
        psi = acc
    return psi

def gauss_holds(psi):
    return all(np.array_equal(apply_perm(psi, gauge_perm(v, x)), psi)
               for v in VERTS for x in range(NG))

# oriented reading: exact class-weight vector of the holonomy
def class_weights(psi):
    w = psi.astype(object) ** 2
    return tuple(int(np.sum(w[CLS[HOL] == k])) for k in range(N_CLS))

def ambivalent_weights(psi):
    """the D4-forced reading: identify each class with its inverse class."""
    w = psi.astype(object) ** 2
    pair_label = np.minimum(CLS, CLS[INV])
    labels = sorted(set(pair_label.tolist()))
    return tuple(int(np.sum(w[pair_label[HOL] == L])) for L in labels)

def flux_state(cls_id):
    return gauss_project((CLS[HOL] == cls_id).astype(np.int64))

def norm_t(t):
    import math
    g = 0
    for x in t:
        g = math.gcd(g, abs(x))
    return t if g == 0 else tuple(x // g for x in t)

def run():
    rep = {}
    a_cls = int(CLS[IDXOF[(1, 0)]])          # class of a  (C)
    ainv_cls = int(CLS[INV[IDXOF[(1, 0)]]])  # class of a^-1 (C^-1)
    rep["N0"] = ("PASS (F21 associative, non-ambivalent: class(a)=%d != class(a^-1)=%d)"
                 % (a_cls, ainv_cls)) if OK and a_cls != ainv_cls else "FIRE"
    sC, sCi = flux_state(a_cls), flux_state(ainv_cls)
    rep["O3"] = "PASS" if gauss_holds(sC) and gauss_holds(sCi) else "FIRE"
    oC, oCi = norm_t(class_weights(sC)), norm_t(class_weights(sCi))
    rep["O1"] = "PASS (oriented vectors differ)" if oC != oCi else f"FIRE (identical: {oC})"
    aC, aCi = norm_t(ambivalent_weights(sC)), norm_t(ambivalent_weights(sCi))
    rep["O2"] = "PASS (ambivalent projection blind)" if aC == aCi else f"FIRE ({aC} vs {aCi})"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep, sC, a_cls, ainv_cls

def plants(sC, a_cls, ainv_cls):
    ok = True
    sym = gauss_project(((CLS[HOL] == a_cls) | (CLS[HOL] == ainv_cls)).astype(np.int64))
    assert np.any(sym), "plant (i) carrier empty"
    # symmetrized state: oriented weights on C and C^-1 must be EQUAL
    w = norm_t(class_weights(sym))
    equal_or = w[a_cls] == w[ainv_cls]
    print(f"[plant i] symmetrized carrier: oriented C-weight == C^-1-weight: {equal_or} -> "
          f"{'FIRES' if equal_or else 'MISSED'}")
    ok &= equal_or
    # (ii) wrong-side action: left-only transform must break Gauss
    bad = apply_perm(sC, IDX3(MUL[3, D0], D1, D2))
    fired = not gauss_holds(bad)
    print(f"[plant ii] wrong-side action -> Gauss {'FIRES' if fired else 'MISSED'}")
    ok &= fired
    return ok

if __name__ == "__main__":
    rep, sC, a_cls, ainv_cls = run()
    ok = plants(sC, a_cls, ainv_cls)
    hard = all(str(v).startswith("PASS") for v in rep.values())
    print(f"\nVERDICT: {'ALL PASS' if hard else 'FIRED'}; plants {'both FIRE' if ok else 'VOID'}")
    sys.exit(0 if (hard and ok) else 1)
