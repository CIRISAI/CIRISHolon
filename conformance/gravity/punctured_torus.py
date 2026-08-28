#!/usr/bin/env python3
"""PUNCTURED-TORUS-1 instrument. Prereg ADMITTED, frozen before this file."""
import sys
import numpy as np
sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import MUL, INV, R2, CLASS
from einstein_adm1 import (N, GA, GB, IDX, comm, ORBIT, N_ORBITS,
                           apply_perm, T_MAP, S_MAP, gauss_project, gauss_holds, v_adm)

def step(psi):
    return apply_perm(apply_perm(psi, T_MAP), S_MAP)

def mass_sector(m_class_rep):
    """projector onto [g_a,g_b] in class(m): seed uniform there, Gauss-project."""
    target = CLASS[comm(GA, GB)] == CLASS[m_class_rep]
    return gauss_project(target.astype(np.int64)), target

def sector_weight(psi, target):
    w = psi.astype(object) ** 2
    return (int(np.sum(w[target])), int(np.sum(w)))

def collisions(vs):
    hits = []
    for i in range(len(vs) - 1):
        for j in range(i + 1, len(vs) - 1):
            if vs[i] == vs[j]:
                hits.append((i, j, vs[i + 1] == vs[j + 1]))
    return hits

# ---- PT-1B refined instance: BOTH edges split (a1,a2,b1,b2), 4096 configs.
# Bijective lifts with explicit inverses (M-NONBIJECTIVE-STEP):
#   T_ref: (a1,a2,b1,b2) -> (a1,a2, a1a2 b1, b2)
#     inverse: (a1,a2, (a1a2)^-1 b1', b2)
#   S_ref: (a1,a2,b1,b2) -> (b1,b2, c a1 c^-1, c a2 c^-1), c = b1 b2
#     inverse: from (p1,p2,q1,q2): b=(p1,p2), c=p1p2, a=(c^-1 q1 c, c^-1 q2 c)
NR = 4096
RA1 = np.arange(NR) % 8
RA2 = (np.arange(NR) // 8) % 8
RB1 = (np.arange(NR) // 64) % 8
RB2 = np.arange(NR) // 512
RIDX = lambda a1, a2, b1, b2: a1 + 8 * a2 + 64 * b1 + 512 * b2
R_GA = MUL[RA1, RA2]
R_GB = MUL[RB1, RB2]

def r_comm():
    return MUL[MUL[R_GA, R_GB], MUL[INV[R_GA], INV[R_GB]]]

T_REF = RIDX(RA1, RA2, MUL[R_GA, RB1], RB2)
_c = R_GB
S_REF = RIDX(RB1, RB2, MUL[MUL[_c, RA1], INV[_c]], MUL[MUL[_c, RA2], INV[_c]])

def r_step(psi):
    return apply_perm(apply_perm(psi, T_REF), S_REF)

def r_gauss_project(psi):
    # base vertex v0: left on a1, right on a2? Chart: a = a1 a2 with midpoint
    # va between them; b = b1 b2 with midpoint vb. v0 conjugates a and b at
    # their ends: a1 -> x a1, a2 -> a2 x^-1, b1 -> x b1, b2 -> b2 x^-1.
    out = psi.copy()
    for perm_fn in (
        lambda x: RIDX(MUL[x, RA1], MUL[RA2, INV[x]], MUL[x, RB1], MUL[RB2, INV[x]]),
        lambda x: RIDX(MUL[RA1, INV[x]], MUL[x, RA2], RB1, RB2),
        lambda x: RIDX(RA1, RA2, MUL[RB1, INV[x]], MUL[x, RB2]),
    ):
        acc = np.zeros_like(out)
        for x in range(8):
            acc += apply_perm(out, perm_fn(x))
        out = acc
    return out

def r_gauss_holds(psi):
    for perm_fn in (
        lambda x: RIDX(MUL[x, RA1], MUL[RA2, INV[x]], MUL[x, RB1], MUL[RB2, INV[x]]),
        lambda x: RIDX(MUL[RA1, INV[x]], MUL[x, RA2], RB1, RB2),
        lambda x: RIDX(RA1, RA2, MUL[RB1, INV[x]], MUL[x, RB2]),
    ):
        for x in range(8):
            if not np.array_equal(apply_perm(psi, perm_fn(x)), psi):
                return False
    return True

def r_bijective():
    for M in (T_REF, S_REF):
        if len(set(M.tolist())) != NR:
            return False
    return True

def run():
    rep = {}
    # D1: mass spectrum
    realizable, empty_bad = [], []
    for rep_el in range(8):
        sec, tgt = mass_sector(rep_el)
        nz = int(np.count_nonzero(sec))
        cls = int(CLASS[rep_el])
        if nz:
            realizable.append(cls)
        else:
            empty_bad.append(cls)
    realizable = sorted(set(realizable)); empty_bad = sorted(set(empty_bad))
    want = sorted({int(CLASS[0]), int(CLASS[R2])})
    rep["D1"] = (f"PASS (mass spectrum = classes {realizable}, others empty)"
                 if realizable == want else f"FIRE realizable={realizable} want={want}")
    rep["G0"] = "PASS" if gauss_holds(mass_sector(0)[0]) else "FIRE"

    # D2: mass conservation along the dynamics, both realizable sectors
    drift = []
    reg = []
    for m in (0, R2):
        psi, tgt = mass_sector(m)
        w0 = sector_weight(psi, tgt)
        for k in range(1, 9):
            psi = step(psi)
            reg.append(psi.copy())
            w = sector_weight(psi, tgt)
            if w[0] * w0[1] != w0[0] * w[1]:
                drift.append((int(CLASS[m]), k))
    rep["D2"] = "PASS (mass conserved exactly)" if not drift else f"FIRE {drift}"

    # D3 (PT-1B): bijectivity checked FIRST; refined spectrum, conservation,
    # and refined B3 on every trajectory state.
    if not r_bijective():
        rep["D3"] = "VOID (refined step not bijective -- refuse to call it dynamics)"
    else:
        r_real = []
        for rep_el in range(8):
            tgt = CLASS[r_comm()] == CLASS[rep_el]
            sec = r_gauss_project(tgt.astype(np.int64))
            if np.count_nonzero(sec):
                r_real.append(int(CLASS[rep_el]))
        r_real = sorted(set(r_real))
        r_drift = []
        r_b3_ok = True
        for m in (0, R2):
            tgt = CLASS[r_comm()] == CLASS[m]
            psi = r_gauss_project(tgt.astype(np.int64))
            w0 = sector_weight(psi, tgt)
            for k in range(1, 5):
                psi = r_step(psi)
                if not r_gauss_holds(psi):
                    r_b3_ok = False
                w = sector_weight(psi, tgt)
                if w[0] * w0[1] != w0[0] * w[1]:
                    r_drift.append((int(CLASS[m]), k))
        rep["D3"] = ("PASS (bijective lift; refined spectrum and conservation identical; refined B3 held)"
                     if r_real == want and not r_drift and r_b3_ok
                     else f"FIRE spectrum={r_real} drift={r_drift} refined_B3={r_b3_ok}")
    # D4g: off-shell channel defect on the r^2 sector
    psi, _ = mass_sector(R2)
    vs = [v_adm(psi)]
    for k in range(1, 9):
        psi = step(psi)
        vs.append(v_adm(psi))
    hits = collisions(vs)
    firing = [(i, j) for (i, j, c) in hits if not c]
    rep["D4g"] = (f"BRANCH(a): off-shell defect > 0 at {firing}" if firing
                  else f"BRANCH(b): not refuted at depth 8 ({len(hits)} consistent collisions)")

    rep["B3"] = "PASS" if all(gauss_holds(s) for s in reg) else "FIRE"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep

def plants():
    ok = True
    # (i) forbidden-mass control with visibility twin
    sec_bad, _ = mass_sector(1)          # class(r): must be EXACTLY empty
    sec_good, _ = mass_sector(R2)        # twin: must be nonzero
    twin_ok = int(np.count_nonzero(sec_good)) > 0
    fired = int(np.count_nonzero(sec_bad)) == 0 and twin_ok
    print(f"[plant i] class(r) sector empty={int(np.count_nonzero(sec_bad))==0}, "
          f"class(r2) twin nonzero={twin_ok} -> {'FIRES' if fired else 'MISSED'}")
    ok &= fired
    # (ii) broken twist on the (s,1)-orbit (ADM-1B)
    refl = next(x for x in range(8) if INV[x] == x and x not in (0, R2))
    rot4 = next(x for x in range(8) if INV[x] != x)
    delta = np.zeros(N, dtype=np.int64); delta[IDX(refl, 0)] = 1
    carrier = gauss_project(delta)
    assert np.any(carrier)
    bad = apply_perm(carrier, IDX(GA, MUL[np.full(N, rot4), GB]))
    fired2 = not gauss_holds(bad)
    print(f"[plant ii] broken twist -> B3 {'FIRES' if fired2 else 'MISSED'}")
    ok &= fired2
    return ok

if __name__ == "__main__":
    rep = run()
    ok = plants()
    hard = all(not str(v).startswith("FIRE") for v in rep.values())
    print(f"\nVERDICT: {'no gate fired' if hard else 'FIRED'}; D4g={rep['D4g'][:40]}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (hard and ok) else 1)
