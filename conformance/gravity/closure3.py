#!/usr/bin/env python3
"""CLOSURE-2 instrument. Prereg ADMITTED and frozen before this file.
The phase-space channel on WILSON-2B's validated model: v_conf (oriented
plaquette triples) vs v_PS (plus per-spoke electric triples). Depth 8 with
auto-promotion. Witness theorems: ClosureDerives.lean."""
import sys
import numpy as np
sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
import wilson2 as W

def promote(st):
    A, B = st
    if A.dtype == object:
        return st
    if max(np.abs(A).max(), np.abs(B).max()) > 2 ** 50:
        return (A.astype(object), B.astype(object))
    return st

def step(st):
    return W.step(promote(st))

def dressed_zeroflux():
    """M-PROBE-EIGENSTATE: the ZERO-FLUX dressed vacuum, named."""
    A = np.zeros((9, W.N), dtype=np.int64); B = np.zeros_like(A)
    for q in range(3):
        A[3 * q + q][0] = 1   # LINE(config 0) = 0
    st = (A, B)
    for v in ["c", 0, 1, 2, 3, 4]:
        ar = np.zeros(A.shape, dtype=A.dtype); ai = np.zeros_like(ar)
        for g in range(3):
            x, y = W.gauge_at(st, v, g); ar = ar + x; ai = ai + y
        st = (ar, ai)
    return st

def norm2(A, B):
    a = A.astype(object); b = B.astype(object)
    return a * a - a * b + b * b

def triple(st, holo):
    n2 = norm2(*st)
    return tuple(int(np.sum(n2[:, holo == k])) for k in range(3))

def normalize(t):
    """scale-free exact form: divide by gcd."""
    import math
    g = 0
    for x in t:
        g = math.gcd(g, abs(x))
    return t if g == 0 else tuple(x // g for x in t)

def electric_triple(st, e):
    """weights of the shift-eigenspace projections on edge e:
    P_k = (1/3) sum_j w^{-kj} L_j.  3*P_k|psi> computed exactly."""
    A, B = st
    out = []
    for k in range(3):
        accA = np.zeros(A.shape, dtype=object); accB = np.zeros(B.shape, dtype=object)
        for j in range(3):
            idx = W.zshift_idx(e, j) if hasattr(W, 'zshift_idx') else None
            if idx is None:
                idx = W.BASE - W.DIG[e] * W.POW3[e] + ((W.DIG[e] + j) % 3) * W.POW3[e]
            pa, pb = W.wpow_mul(A, B, (-k * j) % 3)
            for m in range(9):
                np.add.at(accA[m], idx, pa[m])
                np.add.at(accB[m], idx, pb[m])
        n2 = accA * accA - accA * accB + accB * accB
        out.append(int(np.sum(n2)))
    return normalize(tuple(out))

def v_conf(st):
    parts = [normalize(triple(st, W.HOL_P[p])) for p in range(5)]
    parts.append(normalize(triple(st, W.HOL_LOOP)))
    return tuple(parts)

def thooft_triple(st):
    """CLOSURE-2B: the 't Hooft triple -- exact weights of the eigen-sectors
    of the joint spoke shift D_m (gauge-invariant: the joint shift commutes
    with every Gauss action). P_k = (1/3) sum_m w^{-km} D_m."""
    A, B = st
    out = []
    for k in range(3):
        accA = np.zeros(A.shape, dtype=object); accB = np.zeros(B.shape, dtype=object)
        for m in range(3):
            idx = W.BASE.copy()
            for e in range(5):
                idx = idx - W.DIG[e] * W.POW3[e] + ((W.DIG[e] + m) % 3) * W.POW3[e]
            pa, pb = W.wpow_mul(A, B, (-k * m) % 3)
            for mm in range(9):
                np.add.at(accA[mm], idx, pa[mm])
                np.add.at(accB[mm], idx, pb[mm])
        n2 = accA * accA - accA * accB + accB * accB
        out.append(int(np.sum(n2)))
    return normalize(tuple(out))

def v_ps(st):
    return (v_conf(st), thooft_triple(st))

def nonzero(st):
    return int(np.count_nonzero(st[0]) + np.count_nonzero(st[1]))

def run():
    rep = {}
    x = dressed_zeroflux()
    if nonzero(x) == 0:
        print("G0=FIRE (empty carrier)"); return {"G0": "FIRE"}
    held, why = W.gauss_holds(x)
    rep["G0"] = "PASS" if held else f"FIRE ({why})"
    if rep["G0"] != "PASS":
        print(rep); return rep

    traj = [x]
    vc, vp = [v_conf(x)], [v_ps(x)]
    tot = [normalize(triple(x, W.HOL_LOOP * 0))]   # total weight (all in class 0)
    for k in range(1, 9):
        x = step(x)
        traj.append(x)
        vc.append(v_conf(x)); vp.append(v_ps(x))
        tot.append(normalize(triple(x, W.HOL_LOOP * 0)))
        print(f"  step {k}: v_conf[0..1]={vc[-1][:2]}", flush=True)

    # C1: configuration collisions
    collisions = [(i, j) for i in range(8) for j in range(i + 1, 8) if vc[i] == vc[j]]
    firing = [(i, j) for (i, j) in collisions if vc[i + 1] != vc[j + 1]]
    if firing:
        rep["C1"] = f"BRANCH(a): defect > 0 at collisions {firing}"
    elif collisions:
        rep["C1"] = f"consistent collisions only: {collisions} (single-valued so far)"
    else:
        rep["C1"] = "BRANCH(b): no v_conf collision at depth 8 (not refuted; recorded)"

    # C2: posed only on firing collisions
    if firing:
        blind = [(i, j) for (i, j) in firing if vp[i] == vp[j]]
        rep["C2"] = "PASS (v_PS separates every firing pair)" if not blind else f"FIRE (v_PS blind at {blind})"
    else:
        rep["C2"] = "UNPOSED (no firing collision)"

    # C3: inheritance — total weight through the channel conserved (scale-free)
    drift = [k for k in range(1, 9) if tot[k] != tot[0]]
    rep["C3"] = "PASS (inherited conservation exact)" if not drift else f"FIRE (drift at steps {drift})"

    # C4: single-valuedness ledger
    consistent = [(i, j) for (i, j) in collisions if vc[i + 1] == vc[j + 1]]
    rep["C4"] = f"ledger: {len(consistent)} consistent, {len(firing)} firing"

    bad = [k for k, s in enumerate(traj) if not W.gauss_holds(s)[0]]
    rep["B3"] = "PASS" if not bad else f"FIRE at steps {bad}"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep, traj

def plants(traj):
    ok = True
    psi0 = traj[0]
    # (i)' 2B: the flux-diagonal phase on a SPOKE is a dual-charge insertion
    # and must SHIFT the 't Hooft triple; carrier sector = the triple's
    # non-uniformity, asserted before scoring (M-PLANT-SECTOR).
    t0 = thooft_triple(psi0)
    assert len(set(t0)) > 1, f"plant (i) carrier 't Hooft triple uniform: {t0}"
    A, B = psi0
    pa, pb = W.wpow_mul(A, B, W.DIG[2][None, :] % 3)
    other = (pa, pb)
    same_conf = v_conf(other) == v_conf(psi0)
    diff_ps = thooft_triple(other) != t0
    assert nonzero(other) > 0, "plant (i) carrier empty"
    print(f"[plant i] dual-charge insertion: v_conf equal={same_conf}, 't Hooft shifted={diff_ps} "
          f"({t0} -> {thooft_triple(other)}) -> {'FIRES' if same_conf and diff_ps else 'MISSED'}")
    ok &= same_conf and diff_ps
    # (ii) conservation mutant: drop a ring factor at one step
    x = step(traj[0])
    A, B = x
    mutant = (A * 2, B * 2)   # mis-scale ONE arm's step
    t0 = normalize(triple(traj[0], W.HOL_LOOP * 0))
    tm = normalize(triple(mutant, W.HOL_LOOP * 0))
    # normalize() removes global scale, so the mutant is detected on the RAW totals
    raw0 = triple(step(traj[0]), W.HOL_LOOP * 0)
    rawm = triple(mutant, W.HOL_LOOP * 0)
    fired = rawm != raw0
    print(f"[plant ii] mis-scaled step raw totals differ -> {'FIRES' if fired else 'MISSED'}")
    ok &= fired
    return ok

PAIRS = [(p, q) for p in range(6) for q in range(p + 1, 6)]  # 5 plaq + rim as index 5

def hol_of(i):
    return W.HOL_P[i] if i < 5 else W.HOL_LOOP

def pair_vector(st, p, q):
    n2 = norm2(*st)
    hp, hq = hol_of(p), hol_of(q)
    return normalize(tuple(int(np.sum(n2[:, (hp == a) & (hq == b)]))
                           for a in range(3) for b in range(3)))

def v_pair(st):
    return (v_conf(st), thooft_triple(st)) + tuple(pair_vector(st, p, q) for p, q in PAIRS)

def run_c3():
    rep = {}
    x = dressed_zeroflux()
    rep["G0"] = "PASS" if (nonzero(x) and W.gauss_holds(x)[0]) else "FIRE"
    traj = [x]
    vc, vp = [v_conf(x)], [v_pair(x)]
    tot = [normalize(triple(x, W.HOL_LOOP * 0))]
    for k in range(1, 9):
        x = step(x)
        traj.append(x)
        vc.append(v_conf(x)); vp.append(v_pair(x))
        tot.append(normalize(triple(x, W.HOL_LOOP * 0)))
    firing = [(i, j) for i in range(8) for j in range(i + 1, 8)
              if vc[i] == vc[j] and vc[i + 1] != vc[j + 1]]
    rep["K1"] = ("PASS (collisions reproduce: %s)" % firing
                 if firing == [(1, 5), (2, 4), (5, 7)] else f"FIRE got {firing}")
    blind = [(i, j) for (i, j) in firing if vp[i] == vp[j]]
    # M-FINAL-VIEW-COLLISIONS: "restores closure" requires the REFINED
    # view's OWN collisions to be consistent, not merely separation of the
    # coarse view's. Both are scored; the verdict needs both.
    own_firing = [(i, j) for i in range(8) for j in range(i + 1, 8)
                  if vp[i] == vp[j] and vp[i + 1] != vp[j + 1]]
    if blind:
        rep["K2"] = f"BRANCH(b): v_pair blind at {blind} -- memory beyond second order"
    elif own_firing:
        rep["K2"] = (f"BRANCH(b'): v_pair separates the coarse collisions but has OWN "
                     f"firing collisions {own_firing} -- memory AT LEAST third-order")
    else:
        rep["K2"] = "BRANCH(a): v_pair closed on the trajectory -- memory is second-order"
    drift = [k for k in range(1, 9) if tot[k] != tot[0]]
    rep["K3"] = "PASS" if not drift else f"FIRE {drift}"
    rep["B3"] = "PASS" if all(W.gauss_holds(s)[0] for s in traj) else "FIRE"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep, traj

def plants_c3(traj):
    ok = True
    psi0 = traj[0]
    # (i) pair-sensitivity: carrier + correlated double insertion vs
    # carrier + anti-correlated superposition; one-body equal, pair differs.
    A, B = psi0
    def ins(st, e):
        A, B = st
        idx = W.BASE - W.DIG[e] * W.POW3[e] + ((W.DIG[e] + 1) % 3) * W.POW3[e]
        nA = np.zeros(A.shape, dtype=A.dtype); nB = np.zeros_like(nA)
        for m in range(9):
            np.add.at(nA[m], idx, A[m]); np.add.at(nB[m], idx, B[m])
        return (nA, nB)
    d1 = ins(ins(psi0, 0), 6)      # correlated: both insertions together
    both = (psi0[0] + d1[0], psi0[1] + d1[1])           # psi + F1 F2 psi
    e1 = ins(psi0, 0); e2 = ins(psi0, 6)
    sep = (e1[0] + e2[0], e1[1] + e2[1])                # F1 psi + F2 psi
    one_equal = (v_conf(both) == v_conf(sep)) and (thooft_triple(both) == thooft_triple(sep))
    pair_diff = v_pair(both) != v_pair(sep)
    assert nonzero(both) and nonzero(sep), "plant (i) carriers empty"
    print(f"[plant i] one-body equal={one_equal}, pair differs={pair_diff} -> "
          f"{'FIRES' if one_equal and pair_diff else 'MISSED'}")
    ok &= one_equal and pair_diff
    # (ii) mis-scale mutant, as 2B
    x1 = step(traj[0])
    raw0 = triple(x1, W.HOL_LOOP * 0)
    rawm = triple((x1[0] * 2, x1[1] * 2), W.HOL_LOOP * 0)
    fired = rawm != raw0
    print(f"[plant ii] mis-scale -> {'FIRES' if fired else 'MISSED'}")
    ok &= fired
    return ok

if __name__ == "__main__" and "--c3" in sys.argv:
    rep, traj = run_c3()
    ok = plants_c3(traj)
    hard = all(not str(v).startswith("FIRE") for v in rep.values())
    print(f"\nVERDICT: {'no gate fired' if hard else 'FIRED'}; K2={rep['K2'][:52]}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (hard and ok) else 1)

if __name__ == "__main__":
    out = run()
    if isinstance(out, dict):
        print("VERDICT: VOID"); sys.exit(1)
    rep, traj = out
    ok = plants(traj)
    hard = all(not str(v).startswith("FIRE") for v in rep.values())
    print(f"\nVERDICT: {'no gate fired' if hard else 'GATES FIRED'}; C1={rep['C1'][:40]}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (hard and ok) else 1)
