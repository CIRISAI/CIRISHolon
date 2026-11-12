#!/usr/bin/env python3
"""KINK-0 instrument: the massive Schwinger model with a background field, by exact
diagonalisation in a sector of total charge.

Spin form, W units (as engine/crates/q8-mps/src/schwinger.rs and the Python DMRG driver):

    W = x sum_n (s+_n s-_{n+1} + h.c.) + (mu/2) sum_n (-1)^n sz_n + sum_{n<N-1} (L_n + a_n)^2
    L_n = sum_{k<=n} q_k,   q_k = (sz_k + (-1)^k)/2,   mu = 2 (m/g) sqrt(x)

`a_n` is the background on link n (a constant alpha unless a plant says otherwise). N even, so
the total charge is Q = (n_up - n_down)/2 and a sector is a fixed number of up spins.

usage: kink0.py gate | pilot | campaign   (prints; the caller redirects to a log)
"""
import sys, itertools
import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.linalg import eigsh

def sector(N, Q):
    nup = N // 2 + Q
    states = []
    for ups in itertools.combinations(range(N), nup):
        m = 0
        for i in ups: m |= 1 << i
        states.append(m)
    return states

def build(N, x, mg, alpha, Q, mass_sign=1.0, bg=None):
    mu = 2.0 * mg * np.sqrt(x)
    a = np.full(N - 1, float(alpha)) if bg is None else np.asarray(bg, float)
    st = sector(N, Q); idx = {s: i for i, s in enumerate(st)}
    stag = np.array([1.0 if n % 2 == 0 else -1.0 for n in range(N)])
    rows, cols, vals = [], [], []
    diag = np.zeros(len(st))
    for i, s in enumerate(st):
        sz = np.array([1.0 if (s >> n) & 1 else -1.0 for n in range(N)])
        q = 0.5 * (sz + stag)
        L = np.cumsum(q)[:-1]
        diag[i] = mass_sign * 0.5 * mu * float(np.dot(stag, sz)) + float(np.sum((L + a) ** 2))
        for n in range(N - 1):
            b0, b1 = (s >> n) & 1, (s >> (n + 1)) & 1
            if b0 != b1:
                t = s ^ ((1 << n) | (1 << (n + 1)))
                rows.append(idx[t]); cols.append(i); vals.append(x)
    H = coo_matrix((vals, (rows, cols)), shape=(len(st), len(st))).tocsr()
    H.setdiag(diag)
    return H, st

def lowest(N, x, mg, alpha, Q, k=2, **kw):
    """Lowest k energies and the ground vector. ARPACK raises on non-convergence: the caller
    treats that as VOID (the exit is the discriminator, not a residual)."""
    H, st = build(N, x, mg, alpha, Q, **kw)
    if H.shape[0] <= 400:
        w, v = np.linalg.eigh(H.toarray()); return w[:k], v[:, 0], st
    w, v = eigsh(H, k=k, which="SA", tol=1e-12, maxiter=20000)
    o = np.argsort(w); return w[o], v[:, o[0]], st

def profiles(N, vec, st):
    p = vec ** 2
    stag = np.array([1.0 if n % 2 == 0 else -1.0 for n in range(N)])
    q = np.zeros(N)
    for w, s in zip(p, st):
        sz = np.array([1.0 if (s >> n) & 1 else -1.0 for n in range(N)])
        q += w * 0.5 * (sz + stag)
    return q, np.cumsum(q)[:-1]

def read(N, x, mg, alpha, bg=None):
    """Everything the stakes read at one (N, x, m/g, alpha)."""
    out = {}
    e0, v0, s0 = lowest(N, x, mg, alpha, 0, bg=bg); q0, L0 = profiles(N, v0, s0)
    em, vm, sm = lowest(N, x, mg, alpha, -1, bg=bg); qm, Lm = profiles(N, vm, sm)
    ep, vp, sp = lowest(N, x, mg, alpha, +1, bg=bg); qp, Lp = profiles(N, vp, sp)
    a = np.full(N - 1, float(alpha)) if bg is None else np.asarray(bg, float)
    n = np.arange(N)
    out["E0"] = e0[0]; out["dE_minus"] = em[0] - e0[0]; out["dE_plus"] = ep[0] - e0[0]
    out["gap_minus"] = em[1] - em[0]; out["gap_zero"] = e0[1] - e0[0]
    out["X_minus"] = float(np.dot(n, qm - q0) / -1.0); out["X_plus"] = float(np.dot(n, qp - q0))
    out["dist_minus"] = (N - 1) - out["X_minus"]; out["dist_plus"] = (N - 1) - out["X_plus"]
    out["gauss"] = max(abs(Lm[-1] + qm[-1] + 1.0), abs(np.sum(q0)), abs(np.sum(qp) - 1.0))
    c = N // 2 - 1
    out["E_centre_zero"] = float(0.5 * ((L0 + a)[c] + (L0 + a)[c + 1]))
    step = -(Lm - L0)                       # the step of the field carried by the -1 charge
    pair = 0.5 * (step[:-1] + step[1:])     # two-link mean
    out["step_monotone"] = bool(np.all(np.diff(pair) >= -1e-9))
    return out

def show(tag, N, x, mg, alpha, r):
    print(f"{tag} N={N:2d} x={x} m/g={mg} alpha={alpha}: dE-={r['dE_minus']:+.6f} dE+={r['dE_plus']:+.6f} "
          f"gap-={r['gap_minus']:.6f} dist-={r['dist_minus']:.4f} dist+={r['dist_plus']:.4f} "
          f"Ec={r['E_centre_zero']:+.4f} monotone={r['step_monotone']} gauss={r['gauss']:.1e}", flush=True)

def gate():
    e, _, _ = lowest(12, 4.0, 0.0, 0.0, 0)
    ref = -26.1743157050
    print(f"G0  N=12 x=4 massless Q=0: {e[0]:.10f} against the engine's typed referee {ref:.10f}: "
          f"|diff| = {abs(e[0]-ref):.1e}  {'MET' if abs(e[0]-ref) < 1e-6 else 'FAILED'}", flush=True)
    return abs(e[0] - ref) < 1e-6

if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "gate":
        gate()
    elif mode == "pilot":
        # DECLARED PILOT. A parameter point that is NOT the campaign's, used only to source bars.
        print("# PILOT at x=5, m/g in {1.2, 0.1}: not the campaign's point; sources bars only")
        if not gate(): sys.exit(1)
        for mg in (1.2, 0.1):
            for alpha in (0.0, 0.5):
                for N in (12, 14, 16, 18):
                    show("PILOT", N, 5.0, mg, alpha, read(N, 5.0, mg, alpha))
        N = 18; half = N // 2
        for d in (+0.05, -0.05):
            bg = np.full(N - 1, 0.5); bg[:half] += d
            r = read(N, 5.0, 1.2, 0.5, bg=bg)
            print(f"PILOT pinned left-half background {0.5+d:.2f}: X-={r['X_minus']:.4f}", flush=True)
        r = read(N, 5.0, 1.2, 0.5); print(f"PILOT unpinned: X-={r['X_minus']:.4f}", flush=True)
