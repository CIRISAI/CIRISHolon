#!/usr/bin/env python3
"""PT-2T instrument. Prereg ADMITTED and frozen before this file.
2T built from unit quaternions in exact (2x-integer) coordinates."""
import sys
import numpy as np
from itertools import product

# ---- build 2T: unit quaternions {±1,±i,±j,±k, (±1±i±j±k)/2}, stored as
# 2*(w,x,y,z) so all coordinates are exact integers.
def qmul(p, q):
    w1, x1, y1, z1 = p; w2, x2, y2, z2 = q
    return ((w1*w2 - x1*x2 - y1*y2 - z1*z2) // 2,
            (w1*x2 + x1*w2 + y1*z2 - z1*y2) // 2,
            (w1*y2 - x1*z2 + y1*w2 + z1*x2) // 2,
            (w1*z2 + x1*y2 - y1*x2 + z1*w2) // 2)

units = [(2,0,0,0),(-2,0,0,0),(0,2,0,0),(0,-2,0,0),(0,0,2,0),(0,0,-2,0),(0,0,0,2),(0,0,0,-2)]
halves = [(a,b,c,d) for a in (1,-1) for b in (1,-1) for c in (1,-1) for d in (1,-1)]
ELEMS = units + halves
assert len(ELEMS) == 24
IDXOF = {e: k for k, e in enumerate(ELEMS)}
NG = 24
MUL = np.zeros((NG, NG), dtype=np.int64)
for a in range(NG):
    for b in range(NG):
        MUL[a, b] = IDXOF[qmul(ELEMS[a], ELEMS[b])]
IDE = IDXOF[(2,0,0,0)]
INV = np.zeros(NG, dtype=np.int64)
for g in range(NG):
    INV[g] = next(h for h in range(NG) if MUL[g, h] == IDE)

# N0-style checks: associativity + order, by exhaustion
for x, y, z in product(range(0, NG, 5), range(NG), range(0, NG, 3)):
    assert MUL[MUL[x, y], z] == MUL[x, MUL[y, z]]
# full associativity (24^3 = 13824, cheap)
A3 = MUL[MUL[:, :, None], np.arange(NG)[None, None, :]]
B3 = MUL[np.arange(NG)[:, None, None], MUL[:, :]]
assert np.array_equal(A3, B3), "associativity fails"

CLS = np.full(NG, -1, dtype=np.int64)
lab = 0
for g in range(NG):
    if CLS[g] >= 0: continue
    for h in {int(MUL[MUL[x, g], INV[x]]) for x in range(NG)}:
        CLS[h] = lab
    lab += 1
N_CLS = lab
# commutator subgroup = Q8 = the 8 unit quaternions
COMM_SET = set()
for a in range(NG):
    for b in range(NG):
        COMM_SET.add(int(MUL[MUL[a, b], MUL[INV[a], INV[b]]]))
Q8_CLASSES = sorted({int(CLS[g]) for g in COMM_SET})

# ---- base torus: (g_a, g_b), 576 configs
N = NG * NG
GA = np.arange(N) % NG
GB = np.arange(N) // NG
IDX = lambda a, b: a + NG * b
def comm(a, b):
    return MUL[MUL[a, b], MUL[INV[a], INV[b]]]
PUNCT = comm(GA, GB)
T_MAP = IDX(GA, MUL[GA, GB])
S_MAP = IDX(GB, MUL[MUL[GB, GA], INV[GB]])
assert len(set(T_MAP.tolist())) == N and len(set(S_MAP.tolist())) == N, "base step not bijective"

def apply_perm(psi, perm):
    out = np.zeros_like(psi)
    np.add.at(out, perm, psi)
    return out

def step(psi):
    return apply_perm(apply_perm(psi, T_MAP), S_MAP)

def gauss_project(psi):
    out = np.zeros_like(psi)
    for x in range(NG):
        out += apply_perm(psi, IDX(MUL[MUL[x, GA], INV[x]], MUL[MUL[x, GB], INV[x]]))
    return out

def gauss_holds(psi):
    return all(np.array_equal(apply_perm(psi, IDX(MUL[MUL[x, GA], INV[x]], MUL[MUL[x, GB], INV[x]])), psi)
               for x in range(NG))

def sector_weight(psi, tgt):
    w = psi.astype(object) ** 2
    return (int(np.sum(w[tgt])), int(np.sum(w)))

# ---- refined: both edges split (a1,a2,b1,b2), 331776 configs
NR = NG ** 4
RA1 = np.arange(NR) % NG
RA2 = (np.arange(NR) // NG) % NG
RB1 = (np.arange(NR) // NG**2) % NG
RB2 = np.arange(NR) // NG**3
RIDX = lambda a1,a2,b1,b2: a1 + NG*a2 + NG*NG*b1 + NG**3*b2
R_GA = MUL[RA1, RA2]; R_GB = MUL[RB1, RB2]
R_PUNCT = comm(R_GA, R_GB)
T_REF = RIDX(RA1, RA2, MUL[R_GA, RB1], RB2)
_c = R_GB
S_REF = RIDX(RB1, RB2, MUL[MUL[_c, RA1], INV[_c]], MUL[MUL[_c, RA2], INV[_c]])

def r_bijective():
    return all(len(set(M.tolist())) == NR for M in (T_REF, S_REF))

def r_step(psi):
    return apply_perm(apply_perm(psi, T_REF), S_REF)

R_PERMS = [
    lambda x: RIDX(MUL[x, RA1], MUL[RA2, INV[x]], MUL[x, RB1], MUL[RB2, INV[x]]),
    lambda x: RIDX(MUL[RA1, INV[x]], MUL[x, RA2], RB1, RB2),
    lambda x: RIDX(RA1, RA2, MUL[RB1, INV[x]], MUL[x, RB2]),
]

def r_gauss_project(psi):
    for pf in R_PERMS:
        acc = np.zeros_like(psi)
        for x in range(NG):
            acc += apply_perm(psi, pf(x))
        psi = acc
    return psi

def r_gauss_holds(psi):
    return all(np.array_equal(apply_perm(psi, pf(x)), psi) for pf in R_PERMS for x in range(NG))

def run():
    rep = {}
    realizable = []
    for c in range(N_CLS):
        tgt = CLS[PUNCT] == c
        if np.count_nonzero(gauss_project(tgt.astype(np.int64))):
            realizable.append(c)
    rep["L1"] = (f"PASS (mass spectrum = Q8 classes {realizable}, {N_CLS - len(realizable)} others empty)"
                 if sorted(realizable) == Q8_CLASSES else
                 f"FIRE realizable={sorted(realizable)} want={Q8_CLASSES}")
    rep["G0"] = "PASS" if gauss_holds(gauss_project((CLS[PUNCT] == CLS[IDE]).astype(np.int64))) else "FIRE"
    drift = []
    reg = []
    for c in Q8_CLASSES:
        tgt = CLS[PUNCT] == c
        psi = gauss_project(tgt.astype(np.int64))
        w0 = sector_weight(psi, tgt)
        for k in range(1, 7):
            psi = step(psi)
            reg.append(psi)
            w = sector_weight(psi, tgt)
            if w[0] * w0[1] != w0[0] * w[1]:
                drift.append((c, k))
    rep["L2"] = "PASS (mass conserved on all five rungs)" if not drift else f"FIRE {drift}"
    if not r_bijective():
        rep["L3"] = "FIRE (refined lift not bijective)"
    else:
        r_real = []
        for c in range(N_CLS):
            tgt = CLS[R_PUNCT] == c
            if np.count_nonzero(r_gauss_project(tgt.astype(np.int64))):
                r_real.append(c)
        r_drift = []
        r_b3 = True
        for c in (Q8_CLASSES[0], Q8_CLASSES[-1]):
            tgt = CLS[R_PUNCT] == c
            psi = r_gauss_project(tgt.astype(np.int64))
            w0 = sector_weight(psi, tgt)
            for k in range(1, 4):
                psi = r_step(psi)
                if not r_gauss_holds(psi):
                    r_b3 = False
                w = sector_weight(psi, tgt)
                if w[0] * w0[1] != w0[0] * w[1]:
                    r_drift.append((c, k))
        rep["L3"] = ("PASS (refined: bijective, spectrum identical, conserved, Gauss held)"
                     if sorted(r_real) == Q8_CLASSES and not r_drift and r_b3
                     else f"FIRE spectrum={sorted(r_real)} drift={r_drift} B3={r_b3}")
    rep["B3"] = "PASS" if all(gauss_holds(s) for s in reg[:6]) else "FIRE"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep

def plants():
    ok = True
    order3 = next(c for c in range(N_CLS) if c not in Q8_CLASSES and c != CLS[IDE])
    bad = gauss_project((CLS[PUNCT] == order3).astype(np.int64))
    minus1 = IDXOF[(-2, 0, 0, 0)]
    twin = gauss_project((CLS[PUNCT] == CLS[minus1]).astype(np.int64))
    fired = int(np.count_nonzero(bad)) == 0 and int(np.count_nonzero(twin)) > 0
    print(f"[plant i] non-Q8 class empty={int(np.count_nonzero(bad))==0}, "
          f"{{-1}} twin nonzero={int(np.count_nonzero(twin))>0} -> {'FIRES' if fired else 'MISSED'}")
    ok &= fired
    delta = np.zeros(N, dtype=np.int64)
    s_el = IDXOF[(0, 2, 0, 0)]   # i
    delta[IDX(s_el, IDE)] = 1
    carrier = gauss_project(delta)
    assert np.any(carrier)
    rot = IDXOF[(1, 1, 1, 1)]
    bad2 = apply_perm(carrier, IDX(GA, MUL[np.full(N, rot), GB]))
    fired2 = not gauss_holds(bad2)
    print(f"[plant ii] broken twist -> Gauss {'FIRES' if fired2 else 'MISSED'}")
    ok &= fired2
    return ok

if __name__ == "__main__":
    rep = run()
    ok = plants()
    hard = all(str(v).startswith("PASS") for v in rep.values())
    print(f"\nVERDICT: {'ALL PASS' if hard else 'FIRED'}; plants {'both FIRE' if ok else 'VOID'}")
    sys.exit(0 if (hard and ok) else 1)
