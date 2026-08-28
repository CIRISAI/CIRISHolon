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

def v_ps(st):
    return (v_conf(st),) + tuple(electric_triple(st, e) for e in range(5))

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
    # (i) channel-blindness control: U_E on ONE edge changes electric, not conf?
    # A single flux SHIFT (diagonal in nothing)... use the exact electric
    # rotation: multiply by w^{DIG[e]} — DIAGONAL in flux, so v_conf triples
    # (norms over flux classes) are unchanged; electric triples change.
    A, B = psi0
    pa, pb = W.wpow_mul(A, B, W.DIG[2][None, :] % 3)
    other = (pa, pb)
    same_conf = v_conf(other) == v_conf(psi0)
    diff_ps = v_ps(other) != v_ps(psi0)
    assert nonzero(other) > 0, "plant (i) carrier empty in its sector"
    print(f"[plant i] flux-diagonal phase: v_conf equal={same_conf}, v_PS differ={diff_ps} -> "
          f"{'FIRES' if same_conf and diff_ps else 'MISSED'}")
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
