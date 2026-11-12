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
import os, sys, itertools
for _v in ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"): os.environ.setdefault(_v, "1")
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
    """Lowest k energies, the ground vector, the sector, and the ground state's residual
    |Hv - Ev| computed here and not taken from the solver. ARPACK starts from a FIXED vector so a
    run is reproducible, and raises on non-convergence: the caller treats that as VOID."""
    H, st = build(N, x, mg, alpha, Q, **kw)
    if H.shape[0] <= 400:
        w, v = np.linalg.eigh(H.toarray()); w, v0 = w[:k], v[:, 0]
    else:
        start = np.ones(H.shape[0]) / np.sqrt(H.shape[0])
        w, v = eigsh(H, k=k, which="SA", tol=1e-12, maxiter=20000, v0=start)
        o = np.argsort(w); w, v0 = w[o], v[:, o[0]]
    res = float(np.linalg.norm(H @ v0 - w[0] * v0))
    return w, v0, st, res

def kron_spectrum(N, x, mg, bg, Q, mass_sign=1.0):
    """An independent referee: the dense W built from Kronecker products of 2 x 2 blocks,
    restricted to the charge sector by the eigenvalue of the total-charge operator."""
    I2 = np.eye(2); sz = np.diag([1.0, -1.0]); sp = np.array([[0.0, 1.0], [0.0, 0.0]])
    def site(op, n):
        m = np.array([[1.0]])
        for k in range(N): m = np.kron(m, op if k == n else I2)
        return m
    stag = [1.0 if n % 2 == 0 else -1.0 for n in range(N)]
    dim = 2 ** N; mu = 2.0 * mg * np.sqrt(x)
    q = [0.5 * (site(sz, n) + stag[n] * np.eye(dim)) for n in range(N)]
    W = np.zeros((dim, dim))
    for n in range(N - 1):
        hop = site(sp, n) @ site(sp.T, n + 1); W += x * (hop + hop.T)
    for n in range(N): W += mass_sign * 0.5 * mu * stag[n] * site(sz, n)
    L = np.zeros((dim, dim))
    for n in range(N - 1):
        L = L + q[n]; F = L + bg[n] * np.eye(dim); W += F @ F
    Qd = np.round(np.diag(sum(q))).astype(int)
    keep = np.nonzero(Qd == Q)[0]
    return np.linalg.eigvalsh(W[np.ix_(keep, keep)])

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
    e0, v0, s0, r0 = lowest(N, x, mg, alpha, 0, bg=bg); q0, L0 = profiles(N, v0, s0)
    em, vm, sm, rm = lowest(N, x, mg, alpha, -1, bg=bg); qm, Lm = profiles(N, vm, sm)
    ep, vp, sp, rp = lowest(N, x, mg, alpha, +1, bg=bg); qp, Lp = profiles(N, vp, sp)
    a = np.full(N - 1, float(alpha)) if bg is None else np.asarray(bg, float)
    n = np.arange(N)
    out["E0"] = e0[0]; out["dE_minus"] = em[0] - e0[0]; out["dE_plus"] = ep[0] - e0[0]
    out["gap_minus"] = em[1] - em[0]; out["gap_zero"] = e0[1] - e0[0]
    out["X_minus"] = float(np.dot(n, qm - q0) / -1.0); out["X_plus"] = float(np.dot(n, qp - q0))
    out["dist_minus"] = (N - 1) - out["X_minus"]; out["dist_plus"] = (N - 1) - out["X_plus"]
    out["resid"] = max(r0, rm, rp)
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
          f"Ec={r['E_centre_zero']:+.4f} gauss={r['gauss']:.1e} resid={r['resid']:.1e}", flush=True)

def gate():
    e, _, _, _ = lowest(12, 4.0, 0.0, 0.0, 0)
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

def gate_b():
    """G0b: the mass term, a background and the charged sectors, against the Kronecker referee,
    at a point that is not the campaign's."""
    rng = np.random.default_rng(7); N, x, mg = 8, 6.0, 1.0
    bg = rng.uniform(-0.7, 0.7, N - 1); worst = 0.0; moved = 1e9
    for Q in (-1, 0, 1):
        H, _ = build(N, x, mg, 0.0, Q, bg=bg)
        a = np.linalg.eigvalsh(H.toarray()); b = kron_spectrum(N, x, mg, bg, Q)
        worst = max(worst, float(np.max(np.abs(a - b))))
        c = kron_spectrum(N, x, mg, bg, Q, mass_sign=-1.0)
        moved = min(moved, float(np.max(np.abs(a - c))))
    ok = worst < 1e-10 and moved > 1.0
    print(f"G0b N=8 x=6 m/g=1 random background, Q in -1,0,+1: sector spectra against the Kronecker "
          f"referee differ by at most {worst:.1e}; with the mass sign reversed by at least {moved:.2f}  "
          f"{'MET' if ok else 'FAILED'}", flush=True)
    return ok

def campaign():
    X_, HEAVY, LIGHT, SIZES = 3.0, 0.8, 0.15, (12, 14, 16, 18)
    print(f"# KINK-0 campaign: x={X_}, heavy m/g={HEAVY}, light m/g={LIGHT}, N in {SIZES}")
    if not (gate() and gate_b()): print("BRANCH (e): a gate failed; nothing is read"); return
    # the reflection identity, printed as the check on which sector is which
    a = lowest(12, X_, HEAVY, 0.5, -1)[0][0]; b = lowest(12, X_, HEAVY, -0.5, +1)[0][0]
    print(f"IDENTITY E0(-1; 1/2) - E0(+1; -1/2) at N=12 heavy = {a-b:.1e}")
    plants_ok = True
    e1 = lowest(8, X_, HEAVY, 0.0, 0)[0][0]; e2 = lowest(8, X_, HEAVY, 0.0, 0, mass_sign=-1.0)[0][0]
    f = abs(e1 - e2) > 1e-3; plants_ok &= f
    print(f"P-MASS-SIGN N=8: lowest energy moves by {abs(e1-e2):.4f}  {'FIRES' if f else 'FAILS'}")
    a = lowest(12, X_, HEAVY, 1.0, -1)[0][0]; b = lowest(12, X_, HEAVY, 0.0, +1)[0][0]; c = lowest(12, X_, HEAVY, 0.0, 0)[0][0]
    f = abs(a - b) < 1e-9 and abs(a - c) > 1.0; plants_ok &= f
    print(f"P-BACKGROUND N=12: E0(-1; 1) - E0(+1; 0) = {a-b:.1e}; E0(-1; 1) - E0(0; 0) = {a-c:.4f}  {'FIRES' if f else 'FAILS'}")
    N = 18; base = read(N, X_, HEAVY, 0.5)["X_minus"]; mv = []
    for d in (+0.05, -0.05):
        bg = np.full(N - 1, 0.5); bg[0:9] += d          # links 0 to 8
        mv.append(read(N, X_, HEAVY, 0.5, bg=bg)["X_minus"] - base)
    f = mv[0] <= -1.0 and mv[1] >= 0.3; plants_ok &= f
    print(f"P-PINNED N=18: background 0.55 on links 0..8 moves X by {mv[0]:+.3f}; 0.45 by {mv[1]:+.3f}  {'FIRES' if f else 'FAILS'}")
    if not plants_ok: print("BRANCH (e): a plant failed; nothing is read"); return
    R = {}; void = []; nstates = 0
    for mg in (HEAVY, LIGHT):
        for alpha in (0.0, 0.5):
            for N in SIZES:
                try:
                    R[(mg, alpha, N)] = read(N, X_, mg, alpha); nstates += 3
                    show("READ ", N, X_, mg, alpha, R[(mg, alpha, N)])
                except Exception as ex:
                    void.append((mg, alpha, N)); print(f"VOID  N={N} m/g={mg} alpha={alpha}: {type(ex).__name__}")
    print(f"states solved: {nstates}; void: {len(void)}; largest residual |Hv-Ev|: {max(r['resid'] for r in R.values()):.1e}")
    if void: print(f"BRANCH (v): VOID at {void}; the stakes that read those states are not read"); return
    g = lambda mg, a, N, k: R[(mg, a, N)][k]
    gauss = max(r["gauss"] for r in R.values()); s1 = gauss < 1e-10
    print(f"S1  total charge = Q in every state, to {gauss:.1e}  {'MET' if s1 else 'KILLED'}")
    d0 = abs(g(HEAVY, 0.0, 18, "dist_minus") - g(HEAVY, 0.0, 12, "dist_minus"))
    dh = g(HEAVY, 0.5, 18, "dist_minus") - g(HEAVY, 0.5, 12, "dist_minus")
    s2 = d0 < 0.05 and dh >= 1.5; k2 = dh < 0.5
    print(f"S2  heavy: distance from the end changes by {d0:.4f} at no background and grows by {dh:.3f} at half a unit; "
          f"dE+ - dE- at no background = {g(HEAVY,0.0,18,'dE_plus')-g(HEAVY,0.0,18,'dE_minus'):.3f} (reported)  "
          f"{'MET' if s2 else ('KILLED' if k2 else 'NOT MET')}")
    ratio = lambda mg: g(mg, 0.5, 18, "gap_minus") / g(mg, 0.5, 12, "gap_minus")
    p = lambda mg, n1, n2: np.log(g(mg, 0.5, n1, "gap_minus") / g(mg, 0.5, n2, "gap_minus")) / np.log(n2 / n1)
    rh, rl = ratio(HEAVY), ratio(LIGHT)
    ph, pl = p(HEAVY, 16, 18) - p(HEAVY, 12, 14), p(LIGHT, 16, 18) - p(LIGHT, 12, 14)
    s3 = rh <= 0.65 and (rl - rh) >= 0.15 and ph > 0 and pl < 0
    k3 = rh >= 0.80 or abs(rl - rh) < 0.05 or ph < 0
    print(f"S3  gap ratio (18 over 12): heavy {rh:.3f}, light {rl:.3f}, contrast {rl-rh:+.3f}; "
          f"change of the closing exponent: heavy {ph:+.3f}, light {pl:+.3f}  {'MET' if s3 else ('KILLED' if k3 else 'NOT MET')}")
    dE = lambda mg, N: g(mg, 0.5, N, "dE_minus")
    shrink = abs(dE(HEAVY, 18) - dE(HEAVY, 16)) < abs(dE(HEAVY, 16) - dE(HEAVY, 14))
    s4 = dE(HEAVY, 18) > 0 and shrink and dE(LIGHT, 18) < 0; k4 = dE(HEAVY, 18) < 0 or dE(LIGHT, 18) > 0
    print(f"S4  dE-(18): heavy {dE(HEAVY,18):+.4f} (differences shrinking: {shrink}), light {dE(LIGHT,18):+.4f}  "
          f"{'MET' if s4 else ('KILLED' if k4 else 'NOT MET')}")
    for mg in (HEAVY, LIGHT):
        y = np.array([dE(mg, N) for N in SIZES]); n = np.array(SIZES, float)
        for name, basis in (("M + b/N", 1.0 / n), ("M + b/N^2", 1.0 / n ** 2)):
            A = np.vstack([np.ones(4), basis]).T; c, *_ = np.linalg.lstsq(A, y, rcond=None)
            print(f"    reported, not staked: m/g={mg} {name}: M = {c[0]:+.4f}, worst residual {np.max(np.abs(A @ c - y)):.1e}")
    eh, el = g(HEAVY, 0.5, 18, "E_centre_zero"), g(LIGHT, 0.5, 18, "E_centre_zero")
    s5 = abs(eh - 0.5) <= 0.05 and (eh - el) >= 0.10; k5 = abs(eh - el) < 0.03
    print(f"S5  field at the centre, N=18: heavy {eh:.4f}, light {el:.4f}, difference {eh-el:+.4f}  "
          f"{'MET' if s5 else ('KILLED' if k5 else 'NOT MET')}")
    m_branch = rl <= 0.65 and dE(LIGHT, 18) > 0 and abs(el - 0.5) <= 0.05
    if m_branch: print("BRANCH (m): every light reading lies on the heavy side of the heavy bars")
    elif s1 and s2 and s3 and s4 and s5: print("BRANCH (a)")
    else: print("BRANCH (b): " + ", ".join(n for n, ok in (("S1", s1), ("S2", s2), ("S3", s3), ("S4", s4), ("S5", s5)) if not ok) + " not met")

if __name__ == "__main__" and sys.argv[1] == "campaign":
    campaign()
if __name__ == "__main__" and sys.argv[1] == "gateb":
    gate(); gate_b()
