#!/usr/bin/env python3
"""EINSTEIN-ADM-1 instrument. Prereg ADMITTED and frozen before this file.
One-plaquette D4 torus: 64 configs, exact integer amplitudes."""
import sys
import numpy as np
sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import MUL, INV, R2

N = 64
GA = np.arange(N) % 8      # g_a
GB = np.arange(N) // 8     # g_b
IDX = lambda a, b: a + 8 * b

def comm(a, b):
    return MUL[MUL[a, b], MUL[INV[a], INV[b]]]

FLAT = comm(GA, GB) == 0                       # [g_a,g_b] = 1

# simultaneous conjugation orbits -> v_ADM labels
ORBIT = np.full(N, -1, dtype=np.int64)
lab = 0
for i in range(N):
    if ORBIT[i] >= 0:
        continue
    stack = [i]
    while stack:
        j = stack.pop()
        if ORBIT[j] >= 0:
            continue
        ORBIT[j] = lab
        a, b = GA[j], GB[j]
        for x in range(8):
            k = IDX(MUL[MUL[x, a], INV[x]], MUL[MUL[x, b], INV[x]])
            if ORBIT[k] < 0:
                stack.append(k)
    lab += 1
N_ORBITS = lab

# mapping-class generators (permutations of configs)
T_MAP = IDX(GA, MUL[GA, GB])                        # T: b -> a b
S_MAP = IDX(GB, MUL[MUL[GB, GA], INV[GB]])          # S: (a,b) -> (b, b a b^-1)

def apply_perm(psi, perm):
    out = np.zeros_like(psi)
    np.add.at(out, perm, psi)
    return out

def step(psi):
    return apply_perm(apply_perm(psi, T_MAP), S_MAP)

def gauss_project(psi):
    out = np.zeros_like(psi)
    for x in range(8):
        perm = IDX(MUL[MUL[x, GA], INV[x]], MUL[MUL[x, GB], INV[x]])
        out += apply_perm(psi, perm)
    return out

def gauss_holds(psi):
    for x in range(8):
        perm = IDX(MUL[MUL[x, GA], INV[x]], MUL[MUL[x, GB], INV[x]])
        if not np.array_equal(apply_perm(psi, perm), psi):
            return False
    return True

def v_adm(psi):
    w = psi.astype(object) ** 2
    return tuple(int(np.sum(w[ORBIT == o])) for o in range(N_ORBITS))

def flat_weight(psi):
    w = psi.astype(object) ** 2
    return (int(np.sum(w[FLAT])), int(np.sum(w)))

def collisions(vs):
    hits = []
    for i in range(len(vs) - 1):
        for j in range(i + 1, len(vs) - 1):
            if vs[i] == vs[j]:
                hits.append((i, j, vs[i + 1] == vs[j + 1]))
    return hits

def run():
    rep = {}
    flat_vac = gauss_project(FLAT.astype(np.int64))
    if not np.any(flat_vac):
        print("G0=FIRE"); return None
    rep["G0"] = "PASS" if gauss_holds(flat_vac) else "FIRE"

    # E1/E2 on the flat carrier
    psi = flat_vac.copy()
    vs = [v_adm(psi)]
    fw0 = flat_weight(psi)
    fw_drift = []
    reg = [psi.copy()]
    for k in range(1, 9):
        psi = step(psi)
        reg.append(psi.copy())
        vs.append(v_adm(psi))
        fw = flat_weight(psi)
        if fw[0] * fw0[1] != fw0[0] * fw[1]:
            fw_drift.append(k)
    hits = collisions(vs)
    firing = [(i, j) for (i, j, cons) in hits if not cons]
    rep["E1"] = "PASS (closed: %d collisions, all consistent)" % len(hits) if not firing else f"FIRE {firing}"
    rep["E2"] = "PASS (flatness inherited exactly)" if not fw_drift else f"FIRE (drift at {fw_drift})"

    # E3: kicked carrier off the flat sector
    kick = gauss_project((comm(GA, GB) == R2).astype(np.int64))  # r^2-curved sector (ADM-1B)
    if not np.any(kick):
        rep["E3"] = "VOID (kicked sector empty)"
    else:
        assert int(np.sum(kick.astype(object)[~FLAT] ** 2)) > 0, "kick has no off-flat support"
        psi = kick.copy()
        vs2 = [v_adm(psi)]
        for k in range(1, 9):
            psi = step(psi)
            reg.append(psi.copy())
            vs2.append(v_adm(psi))
        hits2 = collisions(vs2)
        firing2 = [(i, j) for (i, j, cons) in hits2 if not cons]
        rep["E3"] = (f"BRANCH(a): off-shell defect > 0 at {firing2}" if firing2
                     else "BRANCH(b): off-shell not refuted at depth 8 (recorded)")

    rep["B3"] = "PASS" if all(gauss_holds(s) for s in reg) else "FIRE"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep, flat_vac

def plants(flat_vac):
    ok = True
    # (i) ADM-1B: carrier = Gauss projection of the single flat config
    # (s, 1), s a reflection -- the broken twist's image of THIS orbit is
    # provably not conjugation-closed, so the defect is visible.
    refl = next(x for x in range(8) if INV[x] == x and x not in (0, R2))
    rot4 = next(x for x in range(8) if INV[x] != x)
    delta = np.zeros(N, dtype=np.int64)
    delta[IDX(refl, 0)] = 1
    carrier = gauss_project(delta)
    assert np.any(carrier), "plant (i) carrier empty"
    perm_bad = IDX(GA, MUL[np.full(N, rot4), GB])
    bad = apply_perm(carrier, perm_bad)
    fired = not gauss_holds(bad)
    print(f"[plant i] non-covariant twist on the (s,1)-orbit -> B3 {'FIRES' if fired else 'MISSED'}")
    ok &= fired
    # (ii) flatness mutant: swap one flat config with a non-flat one
    psi = flat_vac.copy()
    f_idx = int(np.argmax(FLAT & (psi > 0)))
    n_idx = int(np.argmax(~FLAT))
    psi[n_idx], psi[f_idx] = psi[f_idx], psi[n_idx]
    fw0 = flat_weight(flat_vac); fwm = flat_weight(psi)
    fired2 = fw0[0] * fwm[1] != fwm[0] * fw0[1]
    print(f"[plant ii] flatness mutant -> E2 {'FIRES' if fired2 else 'MISSED'}")
    ok &= fired2
    return ok

if __name__ == "__main__":
    out = run()
    if out is None:
        sys.exit(1)
    rep, flat_vac = out
    ok = plants(flat_vac)
    hard = all(not str(v).startswith("FIRE") and v != "VOID (kicked sector empty)" for v in rep.values())
    print(f"\nVERDICT: {'no gate fired' if hard else 'FIRED/VOID'}; E3={rep['E3'][:44]}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (hard and ok) else 1)
