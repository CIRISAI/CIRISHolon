#!/usr/bin/env python3
"""SCHWINGER-2 — the MPS rung: two-site DMRG for the massless Schwinger model.

Same Hamiltonian as SCHWINGER-1 (Hamer–Kogut spin form, W-units), now at
N ≥ 32 where the open chain actually holds the correlation length. The
Coulomb term Σ_n L_n² expands to Σ_{k≤l} (N−1−max) q_k q_l, whose coefficient
depends only on the LATER site — so a single accumulated-charge MPO channel
carries it (bond dimension 5). Excited state by ground-state penalty
projection. Credits: White 1992 (DMRG); Bañuls–Cichy–Jansen–Cirac for the
MPS-Schwinger tradition; the exact referee remains Schwinger 1962.

Gauge: DMRG must reproduce SCHWINGER-1's exact ED at N ≤ 16 to 1e-6, and a
planted MPO-coefficient mutation must fire."""
import os
import sys, json
import numpy as np
from math import sqrt, pi
from scipy.sparse.linalg import eigsh, LinearOperator

SP = np.array([[0.0, 1.0], [0.0, 0.0]])   # sigma+
SM = SP.T
I2 = np.eye(2)

def site_ops(n_sites, l):
    stag = 1.0 if l % 2 == 0 else -1.0
    q = 0.5 * (np.diag([1.0, -1.0]) + stag * I2)
    return q

def mpo(n, x, mutate=None, lam=None):
    """W[l]: (6, 6, 2, 2). Channels: 0 id-start, 1 after sp, 2 after sm,
    3 Coulomb accumulator, 4 total-charge accumulator (sector penalty
    lam*Q^2 — the MPS roams all charge sectors and the global minimum lives
    outside Q=0, measured: DMRG -27.99 vs Q=0 ED -26.17), 5 done."""
    if lam is None:
        lam = 20.0 * (x + 1.0)
    ws = []
    for l in range(n):
        a = float(n - 1 - l)
        a_pair = a
        if mutate == "coeff-off-by-one":
            # Planted OBSERVABLE off-by-one: the pair channel uses N-l while
            # the diagonal keeps N-1-l, so DH = Q^2 - sum q^2 -- visible in the
            # physical sector. (The first plant, a uniform shift, was EXACTLY
            # Q^2 and identically zero on the Q=0 sector: measured 0.0000
            # shift -- a mathematically null mutation, caught by its own
            # gauge.)
            a_pair = float(n - l)
        q = site_ops(n, l)
        w = np.zeros((6, 6, 2, 2))
        w[0, 0] = I2
        w[0, 1] = SP
        w[1, 5] = x * SM
        w[0, 2] = SM
        w[2, 5] = x * SP
        w[0, 3] = q
        w[3, 3] = I2
        w[3, 5] = 2.0 * a_pair * q
        w[0, 4] = q
        w[4, 4] = I2
        w[4, 5] = 2.0 * lam * q
        w[0, 5] = a * (q @ q) + lam * (q @ q)
        w[5, 5] = I2
        ws.append(w)
    return ws

def random_mps(n, chi):
    rng = np.random.default_rng(7)
    mps, d = [], 1
    for l in range(n):
        dr = min(chi, 2 ** min(l + 1, n - l - 1))
        mps.append(rng.standard_normal((d, 2, dr)) * 0.1)
        d = dr
    return mps

def right_envs(mps, ws, chi_none=None):
    n = len(mps)
    envs = [None] * (n + 1)
    envs[n] = np.ones((1, 1, 1))
    for l in range(n - 1, -1, -1):
        A, W, R = mps[l], ws[l], envs[l + 1]
        # R: (ar, wr, br). A: (al, s, ar). W: (wl, wr, s, s')
        T = np.einsum("asr,rwb->aswb", A, R)
        T = np.einsum("aswb,vwts->avtb", T, W)
        envs[l] = np.einsum("avtb,ctb->avc", T, np.conj(mps[l]).transpose(0, 2, 1).transpose(0, 2, 1) if False else mps[l].conj())
        # simpler: contract ket-bra properly below
        envs[l] = np.einsum("asr,vwts,ctb,rwb->avc", A, W, mps[l].conj(), R, optimize=True)
    return envs

def dmrg(n, x, chi, n_sweeps=14, penalty=None, w_pen=None, mutate=None, seed=7,
         init=None):
    # AMENDMENT 2026-08-27 (prereg amendment A1, pre-data): `init` warm-starts
    # from a previously converged MPS (the chi-ladder reuses the chi=40 state
    # for chi=64 -- the Arguello Cruz lesson: never start from random when a
    # converged neighbour exists). Correctness is NOT delegated to the warm
    # start: the staked driver's chi/N convergence premises (unchanged) are
    # what catch a badly-converged point, as VOID.
    ws = mpo(n, x, mutate)
    rng = np.random.default_rng(seed)
    if init is not None:
        mps = [t.copy() for t in init]
        assert len(mps) == n
    else:
        mps, d = [], 1
        for l in range(n):
            dr = min(chi, 2 ** min(l + 1, n - l - 1))
            mps.append(rng.standard_normal((d, 2, dr)) * 0.1)
            d = dr
    # right-canonicalize
    for l in range(n - 1, 0, -1):
        a, s, b = mps[l].shape
        m = mps[l].reshape(a, s * b)
        q_, r_ = np.linalg.qr(m.T)
        mps[l] = q_.T.reshape(-1, s, b)
        mps[l - 1] = np.einsum("asr,rb->asb", mps[l - 1], r_.T)
    # environments
    L = [None] * (n + 1)
    R = [None] * (n + 1)
    # Boundary CHANNEL SELECTORS, not all-ones: einsum broadcasts a size-1
    # axis across the six MPO channels, which silently sums every partial
    # channel path (measured: e0 = -103.7 vs dense truth -7.955 at N=4).
    nch = ws[0].shape[0]
    L[0] = np.zeros((1, nch, 1))
    L[0][0, 0, 0] = 1.0
    R[n] = np.zeros((1, nch, 1))
    R[n][0, nch - 1, 0] = 1.0
    for l in range(n - 1, 0, -1):
        R[l] = np.einsum("asr,vwts,ctb,rwb->avc", mps[l], ws[l], mps[l].conj(), R[l + 1], optimize=True)
    # penalty overlap environments
    if penalty is not None:
        Lp = [None] * (n + 1)
        Rp = [None] * (n + 1)
        Lp[0] = np.ones((1, 1))
        Rp[n] = np.ones((1, 1))
        for l in range(n - 1, 0, -1):
            Rp[l] = np.einsum("asr,csb,rb->ac", penalty[l], mps[l].conj(), Rp[l + 1], optimize=True)
    energy = None
    # A1: sweep-adaptive Lanczos tolerance (loose early, machine-precision
    # late; warm starts skip the loose rungs) and a GUARDED stagnation exit:
    # only after two consecutive machine-precision sweeps whose energy moved
    # <= 1e-10 relative. The exit saves sweeps; the premises do the catching.
    prev_e, stall = None, 0
    for sweep in range(n_sweeps):
        if init is None:
            tol = 1e-6 if sweep < 2 else (1e-9 if sweep < 4 else 0)
        else:
            tol = 1e-9 if sweep < 1 else 0
        for direction in (range(n - 1), range(n - 2, -1, -1)):
            for l in direction:
                Wl, Wr = ws[l], ws[l + 1]
                Le, Re = L[l], R[l + 2]
                al = Le.shape[0]
                br = Re.shape[0]
                dim = al * 2 * 2 * br
                W2 = np.einsum("vmts,mwua->vwtusa", Wl, Wr)  # (wl, wr, s_l, s_r, s'_l, s'_r)
                def matvec(v):
                    psi = v.reshape(al, 2, 2, br)
                    t = np.einsum("avc,astb->vstcb", Le, psi, optimize=True)
                    t = np.einsum("vstcb,vwtusa->wuacb", t, W2, optimize=True)
                    # indices: t (w, s'_l→u?, ...) — fully explicit contraction:
                    out = np.einsum("avc,astb,vwtusa2,rwb->cuar", Le, psi, W2, Re, optimize=True) if False else None
                    return t  # replaced below
                # do the contraction explicitly and correctly:
                def matvec2(v):
                    psi = v.reshape(al, 2, 2, br)
                    t1 = np.einsum("avc,astb->vstcb", Le, psi, optimize=True)
                    # W2: (v w t u s a) with t=s_l, s=s'_l? — rebuild names:
                    # Wl (v m t s): in-chan v, out m, bra t, ket s
                    # careful: our W[chan_in, chan_out, bra, ket]
                    t2 = np.einsum("vstcb,vmus->mutcb", t1, Wl, optimize=True)
                    t3 = np.einsum("mutcb,mwyt->wyucb", t2, Wr, optimize=True)
                    # Re layout is (ket_bond, chan, bra_bond): contract the KET
                    # bond with psi's right bond; the transposed form left the
                    # effective operator non-Hermitian (measured: bond-dependent
                    # cycling energies at N=4).
                    out = np.einsum("wyucb,bwr->cuyr", t3, Re, optimize=True)
                    return out.reshape(dim)
                mv = matvec2
                if penalty is not None:
                    ov = np.einsum("ac,astb,br->cstr", Lp[l],
                                   np.einsum("asm,mtb->astb", penalty[l], penalty[l + 1]),
                                   Rp[l + 2], optimize=True).reshape(dim)
                    nrm = np.linalg.norm(ov)
                    ovn = ov / nrm if nrm > 1e-12 else ov
                    base = mv
                    def mv_pen(v, base=base, ovn=ovn):
                        return base(v) + w_pen * ovn * (ovn @ v)
                    op = LinearOperator((dim, dim), matvec=mv_pen)
                else:
                    op = LinearOperator((dim, dim), matvec=mv)
                v0 = np.einsum("asm,mtb->astb", mps[l], mps[l + 1]).reshape(dim)
                # Rung ladder (the fci.py CO lesson, applied here after the x=16
                # N=144 point died the same death): rung (a) is the ORIGINAL request,
                # so every point computed before this edit reproduces identically.
                # Rung (b) widens the Krylov basis and asks for seed-grade tolerance
                # instead of tol=0 -- the sweep's own stagnation criterion (below)
                # still governs final quality, so nothing about convergence loosens.
                try:
                    val, vec = eigsh(op, k=1, which="SA", v0=v0, maxiter=300, tol=tol)
                except Exception:
                    dim = op.shape[0]
                    val, vec = eigsh(op, k=1, which="SA", v0=v0,
                                     maxiter=5000, tol=max(tol, 1e-12),
                                     ncv=min(dim - 1, 48))
                energy = val[0]
                psi = vec[:, 0].reshape(al * 2, 2 * br)
                u, sv, vt = np.linalg.svd(psi, full_matrices=False)
                keep = min(chi, np.sum(sv > 1e-10))
                u, sv, vt = u[:, :keep], sv[:keep], vt[:keep]
                going_right = direction == range(n - 1) or (isinstance(direction, range) and direction.step == 1)
                if going_right:
                    mps[l] = u.reshape(al, 2, keep)
                    mps[l + 1] = (np.diag(sv) @ vt).reshape(keep, 2, br)
                    L[l + 1] = np.einsum("avc,asr,vwts,ctb->rwb", L[l], mps[l], ws[l], mps[l].conj(), optimize=True).transpose(0, 1, 2) if False else np.einsum("avc,asb,vwts,ctd->bwd", L[l], mps[l], ws[l], mps[l].conj(), optimize=True)
                    if penalty is not None:
                        Lp[l + 1] = np.einsum("ac,asb,csd->bd", Lp[l], penalty[l], mps[l].conj(), optimize=True)
                else:
                    mps[l] = (u @ np.diag(sv)).reshape(al, 2, keep)
                    mps[l + 1] = vt.reshape(keep, 2, br)
                    R[l + 1] = np.einsum("asr,vwts,ctb,rwb->avc", mps[l + 1], ws[l + 1], mps[l + 1].conj(), R[l + 2], optimize=True)
                    if penalty is not None:
                        Rp[l + 1] = np.einsum("asr,csb,rb->ac", penalty[l + 1], mps[l + 1].conj(), Rp[l + 2], optimize=True)
        if tol == 0 and prev_e is not None and \
                abs(energy - prev_e) <= 1e-10 * max(1.0, abs(energy)):
            stall += 1
            if stall >= 2:
                break
        else:
            stall = 0
        prev_e = energy
    return energy, mps

def gap(n, x, chi, mutate=None, states=None, return_states=False):
    i0, i1 = states if states is not None else (None, None)
    e0, gs = dmrg(n, x, chi, mutate=mutate, init=i0)
    scale = abs(e0) if abs(e0) > 1 else 1.0
    e1, es = dmrg(n, x, chi, penalty=gs, w_pen=20.0 * scale, mutate=mutate, seed=11,
                  init=i1)
    m = (e1 - e0) / (2.0 * sqrt(x))
    if return_states:
        return m, e0, e1, (gs, es)
    return m, e0, e1

if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "gauge":
        sys.path.insert(0, ".")
        from schwinger import gap_and_condensate
        for (n, x) in ((12, 4.0), (16, 9.0)):
            m_ed, _ = gap_and_condensate(n, x)
            m_dm, e0, e1 = gap(n, x, chi=48)
            print(f"plant ED-vs-DMRG (N={n}, x={x}): ED M/g={m_ed:.6f}  DMRG M/g={m_dm:.6f}  "
                  f"|diff|={abs(m_ed-m_dm):.2e} -> {'PASS' if abs(m_ed-m_dm) < 1e-4 else 'PIPELINE DEFECT'}")
            assert abs(m_ed - m_dm) < 1e-4
        m_true, _, _ = gap(12, 4.0, chi=48)
        m_mut, _, _ = gap(12, 4.0, chi=48, mutate="coeff-off-by-one")
        print(f"FIRE side (planted coeff-off-by-one): {m_true:.4f} -> {m_mut:.4f}, "
              f"|shift|={abs(m_mut-m_true):.4f} -> {'FIRES' if abs(m_mut-m_true) > 0.02 else 'MISSED'}")
        assert abs(m_mut - m_true) > 0.02
        print("gauge verdict: DMRG reproduces exact ED and the planted MPO mutation FIRES. Two-sided.")
    elif mode == "staked3":
        # SCHWINGER-3 (prereg 343776e, ADMITTED): per-column N discharging
        # M-VOLUME-SCALE -- N = ceil(k*sqrt(x)) even, k = 20, 28, 36.
        XS = [4.0, 9.0, 16.0]
        NS_BY_X = {4.0: [40, 56, 72], 9.0: [60, 84, 108], 16.0: [80, 112, 144]}
        CHIS = [40, 64]
        out = {"points": [], "voids": []}
        m_at_x = []
        for x in XS:
            NS = NS_BY_X[x]
            ms = {}
            for n_sites in NS:
                mchis = []
                st = None
                for chi in CHIS:
                    ck = f"ckpt3_x{x}_N{n_sites}_chi{chi}.npz"
                    if os.path.exists(ck):
                        z = np.load(ck, allow_pickle=True)
                        m = float(z["m"])
                        st = (list(z["gs"]), list(z["es"]))
                        print(f"x={x:5.1f} N={n_sites:3d} chi={chi:3d}  "
                              f"M/g={m:.6f}  [checkpoint]", flush=True)
                    else:
                        m, e0, e1, st = gap(n_sites, x, chi, states=st,
                                            return_states=True)
                        np.savez(ck, m=m,
                                 gs=np.array(st[0], dtype=object),
                                 es=np.array(st[1], dtype=object))
                        print(f"x={x:5.1f} N={n_sites:3d} chi={chi:3d}  M/g={m:.6f}",
                              flush=True)
                    mchis.append(m)
                if abs(mchis[1] - mchis[0]) > 1e-3:
                    out["voids"].append({"x": x, "N": n_sites, "reason": "chi-unconverged"})
                ms[n_sites] = mchis[1]
            d = abs(ms[NS[2]] - ms[NS[1]])
            if d < 0.01:
                import numpy as _np
                A = _np.array([[1.0, 1.0 / n] for n in NS])
                y = _np.array([ms[n] for n in NS])
                c, *_ = _np.linalg.lstsq(A, y, rcond=None)
                m_at_x.append((x, float(c[0])))
                out["points"].append({"x": x, "M_inf": float(c[0]), "delta": d})
            else:
                out["voids"].append({"x": x, "reason": "N-unconverged", "delta": d})
        if len(m_at_x) >= 3:
            import numpy as _np, math as _math
            A = _np.array([[1.0, 1.0 / _math.sqrt(x)] for x, _ in m_at_x])
            y = _np.array([m for _, m in m_at_x])
            c, *_ = _np.linalg.lstsq(A, y, rcond=None)
            target = 1.0 / _math.sqrt(_math.pi)
            inside = abs(float(c[0]) - target) <= 0.05
            out["verdict"] = (f"S1 BRANCH({'a' if inside else 'b'}): M_V/g = {float(c[0]):.6f} "
                              f"(target {target:.6f} +- 0.05)")
        else:
            out["verdict"] = f"VOID (fewer than 3 posable x: {len(m_at_x)})"
        import json as _json
        print(_json.dumps(out, indent=1))
        open("schwinger3_result.log", "w").write(_json.dumps(out, indent=1))
        sys.exit(0)
    elif mode == "staked":
        XS = [4.0, 9.0, 16.0]
        NS = [32, 48, 64]
        CHIS = [40, 64]
        out = {"points": [], "voids": []}
        m_at_x = []
        for x in XS:
            ms = {}
            for n_sites in NS:
                mchis = []
                st = None
                for chi in CHIS:
                    # A2 (instrument robustness only): checkpoint each
                    # completed grid point's value AND states so a killed
                    # session costs narration, never computation.
                    ck = f"ckpt_x{x}_N{n_sites}_chi{chi}.npz"
                    if os.path.exists(ck):
                        z = np.load(ck, allow_pickle=True)
                        m = float(z["m"])
                        st = (list(z["gs"]), list(z["es"]))
                        print(f"x={x:5.1f} N={n_sites:3d} chi={chi:3d}  "
                              f"M/g={m:.6f}  [checkpoint]", flush=True)
                    else:
                        m, e0, e1, st = gap(n_sites, x, chi, states=st,
                                            return_states=True)
                        np.savez(ck, m=m,
                                 gs=np.array(st[0], dtype=object),
                                 es=np.array(st[1], dtype=object))
                    mchis.append(m)
                    print(f"x={x:5.1f} N={n_sites:3d} chi={chi:3d}  M/g={m:.6f}", flush=True)
                if abs(mchis[-1] - mchis[0]) > 1e-3:
                    out["voids"].append({"x": x, "N": n_sites, "reason": "chi-unconverged",
                                         "delta": abs(mchis[-1] - mchis[0])})
                ms[n_sites] = mchis[-1]
            if abs(ms[NS[-1]] - ms[NS[-2]]) >= 0.01:
                out["voids"].append({"x": x, "reason": "N-unconverged",
                                     "delta": abs(ms[NS[-1]] - ms[NS[-2]])})
                continue
            A = np.array([[1.0, 1.0 / n_] for n_ in NS[-3:]])
            y = np.array([ms[n_] for n_ in NS[-3:]])
            coef, *_ = np.linalg.lstsq(A, y, rcond=None)
            m_at_x.append((x, coef[0]))
            out["points"].append({"x": x, "M_over_g": coef[0]})
        if len(m_at_x) < 3:
            out["verdict"] = "VOID (fewer than 3 posable x)"
        else:
            A = np.array([[1.0, 1.0 / sqrt(x_)] for x_, _ in m_at_x])
            y = np.array([m_ for _, m_ in m_at_x])
            coef, *_ = np.linalg.lstsq(A, y, rcond=None)
            target = 1.0 / sqrt(pi)
            out["M_extrapolated"] = coef[0]
            out["target"] = target
            out["abs_err"] = abs(coef[0] - target)
            out["verdict"] = "PASS" if abs(coef[0] - target) <= 0.05 else "MISS"
        print(json.dumps(out, indent=2))
