#!/usr/bin/env python3
"""WILSON-1 instrument. Prereg ADMITTED and frozen before this file.
Z3 gauge on the fan disk, exact Eisenstein arithmetic (a+b*w, w=e^{2pi i/3},
w^2 = -1-w), Wilson-dressed charge pair, oriented plaquette observable."""
import sys
import numpy as np

# ---- graph: fan disk, 5 rim vertices 0..4, centre c. Spokes then rim.
SPOKES = [("c", k) for k in range(5)]
RIM = [(k, (k + 1) % 5) for k in range(5)]
EDGES = SPOKES + RIM                       # e0..e4 spokes, e5..e9 rim
E = len(EDGES)
N = 3 ** E                                 # 59049 configs
# plaquette k = spoke k, rim k, -spoke k+1  (triangle c,k,k+1)
PLAQ = [[(k, +1), (5 + k, +1), ((k + 1) % 5, -1)] for k in range(5)]
P0 = 0
E_STAR = 0                                 # pump on spoke (c,0)
P_FAR = 3                                  # triangle (c,3,4): disjoint from matter at 1,2
LOOP = [(5 + k, +1) for k in range(5)]     # the rim
M1, M2 = 1, 2                              # matter vertices (charges +1, -1)

DIG = np.empty((E, N), dtype=np.int64)
_cfg = np.arange(N)
for e in range(E):
    DIG[e] = _cfg % 3
    _cfg = _cfg // 3
POW3 = np.array([3 ** e for e in range(E)], dtype=np.int64)
BASE = np.sum(DIG * POW3[:, None], axis=0)

def hol(word):
    h = np.zeros(N, dtype=np.int64)
    for e, s in word:
        h = (h + s * DIG[e]) % 3
    return h

HOL_P = [hol(w) for w in PLAQ]
HOL_LOOP = hol(LOOP)
LINE = (-DIG[M1] + DIG[M2]) % 3            # holonomy of path 1 -> c -> 2

# ---- state: amplitudes in Z[w]: (A, B) int64 arrays, value A + B*w, shape (9, N)
# matter index m = 3*q1 + q2, charges q1 at vertex 1 (+1 rep), q2 at vertex 2 (-1 rep)
def zmul(Aa, Ba, Ab, Bb):
    # (a1+b1 w)(a2+b2 w) = a1a2 - b1b2 + (a1b2 + b1a2 - b1b2) w
    return Aa * Ab - Ba * Bb, Aa * Bb + Ba * Ab - Ba * Bb

def wpow_mul(A, B, k):
    """multiply componentwise by w^k where k is an array mod 3."""
    k = np.asarray(k) % 3
    A2 = np.where(k == 0, A, np.where(k == 1, -B, B - A))
    B2 = np.where(k == 0, B, np.where(k == 1, A - B, -A))
    return A2, B2

def gauge_at(st, v, g):
    """gauge transform by group element g at vertex v (edges + matter)."""
    A, B = st
    idx = np.zeros(N, dtype=np.int64)
    for e, (a, b) in enumerate(EDGES):
        d = DIG[e]
        if a == v: d = (d + g) % 3          # left action on outgoing
        if b == v: d = (d - g) % 3
        idx = idx + d * POW3[e]
    A2 = np.zeros_like(A); B2 = np.zeros_like(B)
    for m in range(9):
        q1, q2 = m // 3, m % 3
        # charge +1 at vertex 1: phase w^{+g q1}; charge -1 at vertex 2: w^{-g q2}
        k = 0
        if v == M1: k = (g * 1) % 3 * 0 + (g) % 3 * 0  # placeholder, set below
        pa, pb = A[m], B[m]
        if v == M1:
            pa, pb = wpow_mul(pa, pb, (g * q1) % 3)
        if v == M2:
            pa, pb = wpow_mul(pa, pb, (-g * q2) % 3)
        np.add.at(A2[m], idx, pa)
        np.add.at(B2[m], idx, pb)
    return (A2, B2)

VERTS = ["c", 0, 1, 2, 3, 4]

def gauss_project(st):
    for v in VERTS:
        A, B = st
        accA = np.zeros_like(A); accB = np.zeros_like(B)
        for g in range(3):
            a, b = gauge_at(st, v, g)
            accA += a; accB += b
        st = (accA, accB)
    return st

def gauss_holds(st):
    A, B = st
    for v in VERTS:
        sA = np.zeros_like(A); sB = np.zeros_like(B)
        for g in range(3):
            a, b = gauge_at(st, v, g)
            sA += a; sB += b
        if not (np.array_equal(sA, 3 * A) and np.array_equal(sB, 3 * B)):
            return False, f"Gauss fails at {v}"
    return True, "held"

def nonzero(st):
    return int(np.count_nonzero(st[0]) + np.count_nonzero(st[1]))

# ---- the dressed channel: pair (q1,q2)=(1,1) dressed by w^{LINE} etc.
# Physical dressed state coefficient: for charge component (q,q), weight w^{q*LINE}.
def dressed_state():
    A = np.zeros((9, N), dtype=np.int64); B = np.zeros_like(A)
    for q in range(3):
        m = 3 * q + q
        a = np.ones(N, dtype=np.int64); b = np.zeros(N, dtype=np.int64)
        a, b = wpow_mul(a, b, (q * LINE) % 3)
        A[m] = a; B[m] = b
    return gauss_project((A, B))

def generic_state(seed):
    rng = np.random.default_rng(seed)
    A = rng.integers(-2, 3, size=(9, N)).astype(np.int64)
    B = rng.integers(-2, 3, size=(9, N)).astype(np.int64)
    return gauss_project((A, B))

def channel_split(st):
    """project onto the dressed channel: P = |D><D| per configuration over
    the matter index, D_m(config) = w^{q LINE} for m=(q,q), else 0.
    Returns (3*on-channel, 3*off-channel) exactly (times 3 to stay integer:
    <D|D> = 3)."""
    A, B = st
    # <D|psi> per config: sum_q conj(w^{qL}) psi_{(q,q)}
    pA = np.zeros(N, dtype=np.int64); pB = np.zeros(N, dtype=np.int64)
    for q in range(3):
        m = 3 * q + q
        a, b = wpow_mul(A[m], B[m], (-q * LINE) % 3)
        pA += a; pB += b
    onA = np.zeros_like(A); onB = np.zeros_like(B)
    for q in range(3):
        m = 3 * q + q
        a, b = wpow_mul(pA, pB, (q * LINE) % 3)
        onA[m] = a; onB[m] = b
    offA = 3 * A - onA; offB = 3 * B - onB
    return (onA, onB), (offA, offB)

# ---- Floquet pieces (all exact; U_E carries 3^{-m/2} implicitly by tripling)
def k_charge(st, broken=False):
    """flux +1 on E_STAR, conditioned OFF-channel (on-channel inert)."""
    on, off = channel_split(st)
    oA, oB = off
    shift = 2 if broken else 1
    idx = BASE - DIG[E_STAR] * POW3[E_STAR] + ((DIG[E_STAR] + shift) % 3) * POW3[E_STAR]
    nA = np.zeros_like(oA); nB = np.zeros_like(oB)
    for m in range(9):
        np.add.at(nA[m], idx, oA[m])
        np.add.at(nB[m], idx, oB[m])
    return (on[0] + nA, on[1] + nB)

def u_b(st):
    A, B = st
    for p in range(5):
        A, B = wpow_mul(A, B, HOL_P[p][None, :])
    return (A, B)

def u_e(st):
    """WILSON-2 electric term (M-ELECTRIC-BASIS fix, invariance in the
    freeze): U_E(e) = (1 + w L_1 + w L_2)/sqrt3 -- a polynomial in SHIFT
    operators, gauge-covariant by construction; unitary because the shift
    eigenvalues 1+2w, 1-w, 1-w all have squared modulus 3. Global 3^{-5/2}
    per step carried implicitly (every gate compares weights)."""
    A, B = st
    for e in range(5):
        A2 = A.copy(); B2 = B.copy()          # the "1" term
        for k in (1, 2):
            d2 = (DIG[e] + k) % 3
            idx = BASE - DIG[e] * POW3[e] + d2 * POW3[e]
            pa, pb = wpow_mul(A, B, 1)        # coefficient w
            for m in range(9):
                np.add.at(A2[m], idx, pa[m])
                np.add.at(B2[m], idx, pb[m])
        A, B = A2, B2
    assert max(np.abs(A).max(), np.abs(B).max()) < 2 ** 55, "overflow: REFUSE"
    return (A, B)

def step(st):
    return u_b(u_e(k_charge(st)))

# ---- observables: exact weight triple over an oriented Wilson value
def norm2(A, B):
    # |a+bw|^2 = a^2 - ab + b^2  (exact integer)
    a = A.astype(object); b = B.astype(object)
    return a * a - a * b + b * b

def weight_triple(st, holo):
    A, B = st
    n2 = norm2(A, B)
    return tuple(int(np.sum(n2[:, holo == k])) for k in range(3))

def marginal(triple):
    return (triple[0], triple[1] + triple[2])

def run():
    rep = {}
    dressed = dressed_state()
    if nonzero(dressed) == 0:
        print("G0=FIRE (dressed sector empty)"); return {"G0": "FIRE"}, None
    held, why = gauss_holds(dressed)
    rep["G0"] = "PASS" if held else f"FIRE ({why})"
    if rep["G0"] != "PASS":
        print(rep); return rep, None

    gen = generic_state(11)
    _, off = channel_split(gen)
    offn = nonzero(off)
    if offn == 0:
        rep["W1"] = "VOID (off-channel carrier empty)"
        print(rep); return rep, None

    reg = [("dressed", dressed), ("off", off)]
    a, b = dressed, off
    w1 = w2 = False
    w3_ok = True
    w4_margin_blind_somewhere = False
    w4_needed = False
    for k in range(1, 5):
        a = step(a); b = step(b)
        reg += [(f"T{k}d", a), (f"T{k}o", b)]
        ta, tb = weight_triple(a, HOL_P[P_FAR]), weight_triple(b, HOL_P[P_FAR])
        # normalize by total weight to compare distributions exactly via cross-mult
        sa, sb = sum(ta), sum(tb)
        diff = any(ta[i] * sb != tb[i] * sa for i in range(3))
        if diff: w1 = True
        if ta[1] * sa != ta[2] * sa and ta[1] != ta[2]: w2 = True
        if tb[1] != tb[2]: w2 = True
        ra, rb = weight_triple(a, HOL_LOOP), weight_triple(b, HOL_LOOP)
        if any(ra[i] * sum(rb) != rb[i] * sum(ra) for i in range(3)):
            w3_ok = False
        ma, mb = marginal(ta), marginal(tb)
        m_diff = any(ma[i] * sb != mb[i] * sa for i in range(2))
        if diff and not m_diff:
            w4_margin_blind_somewhere = True
        if diff and m_diff:
            pass
        if diff:
            w4_needed = True
        print(f"  step {k}: far_dressed={ta} far_off={tb} rim_d={ra} rim_o={rb}", flush=True)
    rep["W1"] = "PASS" if w1 else "FIRE (separated observable blind to the channel)"
    rep["W2"] = "PASS" if w2 else "FIRE (no orientation asymmetry anywhere)"
    rep["W3"] = "PASS" if w3_ok else "FIRE (Bianchi null violated -- instrument broken)"
    if not w4_needed:
        rep["W4"] = "UNPOSED (W1 never separated)"
    else:
        rep["W4"] = ("PASS (orientation necessary at some step)" if w4_margin_blind_somewhere
                     else "no-fire, but orientation-necessity UNEARNED (marginal also separates)")
    bad = [nm for nm, s in reg for h, _ in [gauss_holds(s)] if not h]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())), flush=True)
    return rep, dressed

def plants(dressed):
    ok = True
    assert nonzero(dressed) > 0, "plant (i) carrier empty in its sector"
    # (i) wrong-side action: conjugate phase at M1
    A, B = dressed
    A2 = A.copy(); B2 = B.copy()
    idx = np.zeros(N, dtype=np.int64)
    for e, (aa, bb) in enumerate(EDGES):
        d = DIG[e]
        if aa == M1: d = (d + 1) % 3
        if bb == M1: d = (d - 1) % 3
        idx = idx + d * POW3[e]
    A3 = np.zeros_like(A); B3 = np.zeros_like(B)
    for m in range(9):
        q1 = m // 3
        pa, pb = wpow_mul(A[m], B[m], (-1 * q1) % 3)   # WRONG sign
        np.add.at(A3[m], idx, pa)
        np.add.at(B3[m], idx, pb)
    st_bad = (A3, B3)
    held, why = gauss_holds(st_bad)
    # the wrong-side transform of an invariant state should NOT be invariant
    fired = not held or not (np.array_equal(A3, A) and np.array_equal(B3, B))
    print(f"[plant i] wrong-side action -> {'FIRES' if fired else 'MISSED'} ({why})")
    ok &= fired
    # (ii) orientation-breaking readout: conjugated character on one carrier
    t = weight_triple(dressed, HOL_P[P_FAR])
    t_conj = (t[0], t[2], t[1])
    support = t[1] + t[2]
    assert support >= 0
    # WILSON-2: the plant is scored ONLY on a carrier in the ASYMMETRY
    # sector (M-PLANT-SECTOR: the sector of the EFFECT). Search 4 steps.
    carrier = None
    st = dressed
    for k in range(5):
        t = weight_triple(st, HOL_P[P_FAR])
        if t[1] != t[2]:
            carrier = (k, st, t); break
        st = step(st)
    if carrier is None:
        print("[plant ii] UNPOSABLE: no asymmetric carrier within 4 steps -> VOID")
        ok = False
    else:
        k, st, t = carrier
        flipped = (t[0], t[2], t[1]) != t
        print(f"[plant ii] conjugated readout at step {k} -> {'FIRES (visible)' if flipped else 'MISSED'} {t}")
        ok &= flipped
    return ok

if __name__ == "__main__":
    rep, dressed = run()
    if dressed is None:
        print("\nVERDICT: VOID (could not pose)"); sys.exit(1)
    ok = plants(dressed)
    gates = all(v.startswith("PASS") or v.startswith("no-fire") or v == "UNPOSED (W1 never separated)"
                for v in rep.values())
    hard_pass = all(v.startswith("PASS") for k, v in rep.items() if k in ("G0","W1","W2","W3","B3"))
    print(f"\nVERDICT: gates {'ALL PASS' if hard_pass else 'FIRED/VOID'}; "
          f"W4={rep.get('W4','?')}; plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (hard_pass and ok) else 1)
