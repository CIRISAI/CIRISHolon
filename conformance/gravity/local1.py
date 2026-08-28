#!/usr/bin/env python3
"""LOCAL-1 instrument. Prereg ADMITTED and frozen before this file.
Locality as a response function on the tailed graph, exact light cone."""
import sys
import numpy as np

# ---- tailed graph: fan disk + pendant triangle reusing rim edge (4,3)
SPOKES = [("c", k) for k in range(5)]
RIM = [(k, (k + 1) % 5) for k in range(5)]
TAIL = [(3, "d1"), ("d1", 4)]
EDGES = SPOKES + RIM + TAIL                  # 12 edges
E = len(EDGES)
N = 3 ** E                                   # 531441
PLAQ = [[(k, +1), (5 + k, +1), ((k + 1) % 5, -1)] for k in range(5)]
# pendant: (3,d1)+(d1,4)-(3,4)  i.e. edges 10, 11, and rim edge (3,4)=index 8 reversed
PLAQ.append([(10, +1), (11, +1), (8, -1)])
P_PEND = 5
P_NEAR = 4                                   # (c,4,0): contains spoke e*=0
E_STAR = 0
LOOP = [(5 + k, +1) for k in range(5)]
M1, M2 = 1, 2

DIG = np.empty((E, N), dtype=np.int64)
_c = np.arange(N)
for e in range(E):
    DIG[e] = _c % 3
    _c //= 3
POW3 = np.array([3 ** e for e in range(E)], dtype=np.int64)
BASE = np.sum(DIG * POW3[:, None], axis=0)

def hol(word):
    h = np.zeros(N, dtype=np.int64)
    for e, s in word:
        h = (h + s * DIG[e]) % 3
    return h

HOL_P = [hol(w) for w in PLAQ]
LINE = (-DIG[M1] + DIG[M2]) % 3

def zshift_idx(e, k):
    return BASE - DIG[e] * POW3[e] + ((DIG[e] + k) % 3) * POW3[e]

def wpow_mul(A, B, k):
    k = np.asarray(k) % 3
    A2 = np.where(k == 0, A, np.where(k == 1, -B, B - A))
    B2 = np.where(k == 0, B, np.where(k == 1, A - B, -A))
    return A2, B2

VERTS = ["c", 0, 1, 2, 3, 4, "d1"]

def gauge_at(st, v, g):
    A, B = st
    idx = np.zeros(N, dtype=np.int64)
    for e, (a, b) in enumerate(EDGES):
        d = DIG[e]
        if a == v: d = (d + g) % 3
        if b == v: d = (d - g) % 3
        idx = idx + d * POW3[e]
    A2 = np.zeros(A.shape, dtype=A.dtype); B2 = np.zeros(B.shape, dtype=B.dtype)
    for m in range(9):
        q1, q2 = m // 3, m % 3
        pa, pb = A[m], B[m]
        if v == M1: pa, pb = wpow_mul(pa, pb, (g * q1) % 3)
        if v == M2: pa, pb = wpow_mul(pa, pb, (-g * q2) % 3)
        np.add.at(A2[m], idx, pa)
        np.add.at(B2[m], idx, pb)
    return (A2, B2)

def gauss_project(st):
    for v in VERTS:
        A, B = st
        aA = np.zeros_like(A); aB = np.zeros_like(B)
        for g in range(3):
            x, y = gauge_at(st, v, g); aA += x; aB += y
        st = (aA, aB)
    return st

def gauss_holds(st):
    A, B = st
    for v in VERTS:
        sA = np.zeros_like(A); sB = np.zeros_like(B)
        for g in range(3):
            x, y = gauge_at(st, v, g); sA += x; sB += y
        if not (np.array_equal(sA, 3 * A) and np.array_equal(sB, 3 * B)):
            return False, f"Gauss fails at {v}"
    return True, "held"

def nonzero(st):
    return int(np.count_nonzero(st[0]) + np.count_nonzero(st[1]))

def dressed_state():
    """LOCAL-1C: the ZERO-FLUX dressed vacuum (Gauss projection of the
    all-zero configuration) -- the BF vacuum, on which the flux shift acts
    nontrivially. The uniform electric vacuum is shift-invariant and made
    every response vacuously zero (M-PROBE-EIGENSTATE)."""
    A = np.zeros((9, N), dtype=np.int64); B = np.zeros_like(A)
    for q in range(3):
        A[3 * q + q][0] = 1   # LINE(config 0) = 0, so the dressing phase is 1
    return gauss_project((A, B))

def channel_split(st):
    A, B = st
    pA = np.zeros(N, dtype=A.dtype); pB = np.zeros(N, dtype=A.dtype)
    for q in range(3):
        a, b = wpow_mul(A[3 * q + q], B[3 * q + q], (-q * LINE) % 3)
        pA = pA + a; pB = pB + b
    onA = np.zeros_like(A); onB = np.zeros_like(B)
    for q in range(3):
        a, b = wpow_mul(pA, pB, (q * LINE) % 3)
        onA[3 * q + q] = a; onB[3 * q + q] = b
    return (onA, onB), (3 * A - onA, 3 * B - onB)

def k_charge(st):
    on, off = channel_split(st)
    idx = zshift_idx(E_STAR, 1)
    nA = np.zeros_like(off[0]); nB = np.zeros_like(off[1])
    for m in range(9):
        np.add.at(nA[m], idx, off[0][m])
        np.add.at(nB[m], idx, off[1][m])
    return (on[0] + nA, on[1] + nB)

def u_b(st):
    A, B = st
    for p in range(len(PLAQ)):
        A, B = wpow_mul(A, B, HOL_P[p][None, :])
    return (A, B)

# LOCAL-1D: Eisenstein multiply by a fixed (a+bw): (x+yw)(a+bw) =
# xa - yb + (xb + ya - yb) w
def emul(X, Y, a, b):
    return X * a - Y * b, X * b + Y * a - Y * b

# 1D weak coupler: c0=5+4w (diag 21/27), c1=2+w, c2=-1-2w (hop 3/27 each);
# eigenvalue norms all 27 -- unitary at scale 3*sqrt(3), found by exhaustive
# ring search (M-RING-MIXING: scale sqrt(3) forces maximal mixing).
C_DIAG = (5, 4); C_HOP1 = (2, 1); C_HOP2 = (-1, -2)

def promote(st):
    """LOCAL-1D auto-promotion: scale the carrier to the circuit instead of
    refusing -- int64 -> arbitrary-precision objects when headroom runs out
    (the Python analogue of the engine's residue-carrier discipline)."""
    A, B = st
    if A.dtype == object:
        return st
    if max(np.abs(A).max(), np.abs(B).max()) > 2 ** 50:
        return (A.astype(object), B.astype(object))
    return st

def u_e(st, fourier_on_edge=None):
    """1D weak-coupling covariant term on EVERY edge; plant (ii) swaps one
    edge to WILSON-1's convicted Fourier kernel."""
    A, B = st
    for e in range(E):
        A, B = promote((A, B))
        if e == fourier_on_edge:
            A2 = np.zeros_like(A); B2 = np.zeros_like(B)
            for k in range(3):
                idx = BASE - DIG[e] * POW3[e] + k * POW3[e]
                pa, pb = wpow_mul(A, B, (DIG[e] * k)[None, :] % 3)
                for m in range(9):
                    np.add.at(A2[m], idx, pa[m])
                    np.add.at(B2[m], idx, pb[m])
            A, B = A2, B2
            continue
        dA, dB = emul(A, B, *C_DIAG)
        for k, coef in ((1, C_HOP1), (2, C_HOP2)):
            idx = zshift_idx(e, k)
            pa, pb = emul(A, B, *coef)
            for m in range(9):
                np.add.at(dA[m], idx, pa[m])
                np.add.at(dB[m], idx, pb[m])
        A, B = dA, dB
    return (A, B)

def step(st, fourier_on_edge=None):
    return u_b(u_e(k_charge(st), fourier_on_edge))

def perturb(st, e):
    """L1 flux shift on edge e — unitary, exact."""
    A, B = st
    idx = zshift_idx(e, 1)
    nA = np.zeros_like(A); nB = np.zeros_like(B)
    for m in range(9):
        np.add.at(nA[m], idx, A[m])
        np.add.at(nB[m], idx, B[m])
    return (nA, nB)

def norm2(A, B):
    a = A.astype(object); b = B.astype(object)
    return a * a - a * b + b * b

def triple(st, holo):
    n2 = norm2(*st)
    return tuple(int(np.sum(n2[:, holo == k])) for k in range(3))

def response(base_arm, pert_arm, p):
    return tuple(x - y for x, y in zip(triple(pert_arm, HOL_P[p]), triple(base_arm, HOL_P[p])))

def run():
    rep = {}
    psi0 = dressed_state()
    if nonzero(psi0) == 0:
        print("G0=FIRE (empty)"); return None, {"G0": "FIRE"}
    held, why = gauss_holds(psi0)
    rep["G0"] = "PASS" if held else f"FIRE ({why})"
    if rep["G0"] != "PASS":
        print(rep); return None, rep

    a = psi0                       # unperturbed arm
    b = perturb(psi0, E_STAR)      # perturbed arm
    reg = [("a0", a), ("b0", b)]
    l2 = False; l3 = None
    # LOCAL-1E: run the DIRECT-HIT control arm alongside, and stake the cone
    # only at steps where that control is live (nonzero pendant response).
    c = perturb(psi0, 10)
    dist_pend = {}
    live = set()
    ctrl_a = psi0
    for k in range(1, 5):
        a = step(a); b = step(b); c = step(c); ctrl_a = ctrl_a  # ctrl shares arm a
        reg += [(f"a{k}", a), (f"b{k}", b)]
        r_pend = response(a, b, P_PEND)
        r_near = response(a, b, P_NEAR)
        r_ctrl = response(a, c, P_PEND)
        dist_pend[k] = r_pend
        if any(r_ctrl):
            live.add(k)
        print(f"  step {k}: R(near)={r_near}  R(pendant)={r_pend}  ctrl_live={any(r_ctrl)}", flush=True)
        if k == 1 and any(r_near):
            l2 = True
        if k >= 3 and any(r_pend) and l3 is None:
            l3 = k
    l1_ok = all(not any(dist_pend[k]) for k in live) and bool(live)
    rep["L1"] = ("PASS (cone holds at live steps %s)" % sorted(live)) if l1_ok else (
        "FIRE (influence at a live step)" if live else "VOID (no live step: observable blind)")
    rep["L2"] = "PASS" if l2 else "VOID (response function never responds)"
    rep["L3"] = f"ARRIVES at step {l3}" if l3 else "NO-ARRIVAL (recorded, not a fire)"
    bad = [nm for nm, s in reg for h, _ in [gauss_holds(s)] if not h]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return psi0, rep

def plants(psi0):
    ok = True
    # (i)' 1E: direct hit must be live at SOME step <= 4; live set reported.
    a, b = psi0, perturb(psi0, 10)
    lives = []
    for k in range(1, 5):
        a = step(a); b = step(b)
        if any(response(a, b, P_PEND)):
            lives.append(k)
    print(f"[plant i] direct-hit live steps: {lives} -> {'FIRES' if lives else 'MISSED'}")
    ok &= bool(lives)
    # (ii) the convicted kernel on one edge must fire B3 in one step
    bad = step(psi0, fourier_on_edge=3)
    held, why = gauss_holds(bad)
    print(f"[plant ii] Fourier kernel on edge 3 -> B3 {'FIRES' if not held else 'MISSED'} ({why})")
    ok &= not held
    return ok

if __name__ == "__main__":
    psi0, rep = run()
    if psi0 is None:
        print("\nVERDICT: VOID"); sys.exit(1)
    ok = plants(psi0)
    hard = all(rep[k].startswith("PASS") for k in ("G0", "L1", "L2", "B3"))
    print(f"\nVERDICT: gates {'ALL PASS' if hard else 'FIRED/VOID'}; L3={rep['L3']}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (hard and ok) else 1)
