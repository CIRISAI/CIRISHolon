#!/usr/bin/env python3
"""VIEW-SEARCH-1 (VIEW_SEARCH_PREREG.md): the closed coarse view as a search, scored against
its variational bound on the banked equilibrium walks.

  view_search.py plants                          # PV-1..PV-4, must all fire
  view_search.py score BASE SRC ARM [SEEDS] [--tau-fs 20,40,100,200,400]
     e.g. score conformance/water_observatory/replace0 transport_seed flexible 0,1,2

Features per frame: 2x2x2 cell occupancy (7, total dropped) + momentum (24); first-harmonic
Fourier modes along each axis (density cos/sin, current cos/sin x 3 components: 8 per axis,
24). Score: VAMP-2 (Wu & Noe 2020) - sum of squared singular values of the half-whitened
Koopman matrix; held out by seed (fit on two, evaluate on the third, rotated). Masses are
uniform (oxygens only), so momentum is the velocity sum.
"""
import sys, os, math
import numpy as np

# ------------------------------------------------------------------ data
def load_walk(base, tag):
    L = None; P = []; V = []
    for ext, store in (("walk", P), ("vwalk", V)):
        lines = open(f"{base}/{tag}.{ext}").read().splitlines()
        h = lines[0].split(); assert h[1] == ext, h
        L = float(h[3])
        for ln in lines[1:]:
            store.append(np.array(ln.split(), dtype=float).reshape(-1, 3))
    P = np.array(P); V = np.array(V)
    assert P.shape == V.shape, (P.shape, V.shape)
    return P, V, L

# ------------------------------------------------------------------ features
def cell_features(P, V, L, g, perm=None):
    """occupancy (total dropped) and momentum per cell on a g x g x g grid.
    perm: a fixed per-atom cell label (the position-blind placebo) instead of position."""
    T, n, _ = P.shape
    X = P % L
    if perm is None:
        idx = (np.floor(X / L * g).astype(int) % g)
        cell = (idx[..., 2] * g + idx[..., 1]) * g + idx[..., 0]         # (T, n)
    else:
        cell = np.broadcast_to(perm[None, :], (T, n))
    nc = g ** 3
    occ = np.zeros((T, nc)); mom = np.zeros((T, nc, 3))
    for c in range(nc):
        m = (cell == c)
        occ[:, c] = m.sum(1)
        mom[:, c, :] = (V * m[..., None]).sum(1)
    # drop one cell's occupancy (the oxygen count is fixed); the momenta are kept - the oxygen
    # momentum is not conserved on its own (the hydrogens carry the rest), measured on building
    return np.concatenate([occ[:, 1:], mom.reshape(T, -1)], axis=1)

def fourier_features(P, V, L):
    """first harmonic along each axis: [rho_cos, rho_sin, j_x cos, j_x sin, j_y cos, j_y sin, j_z cos, j_z sin] per axis"""
    T = P.shape[0]; k = 2 * math.pi / L
    cols = []
    for ax in range(3):
        c = np.cos(k * (P[..., ax] % L)); s = np.sin(k * (P[..., ax] % L))
        cols += [c.sum(1), s.sum(1)]
        for comp in range(3):
            cols += [(V[..., comp] * c).sum(1), (V[..., comp] * s).sum(1)]
    return np.stack(cols, axis=1)   # (T, 24)

FOURIER_DENSITY = [ax * 8 + q for ax in range(3) for q in (0, 1)]
FOURIER_LONG = [ax * 8 + 2 + 2 * ax + q for ax in range(3) for q in (0, 1)]
FOURIER_TRANS = [ax * 8 + 2 + 2 * comp + q for ax in range(3) for comp in range(3) if comp != ax for q in (0, 1)]

def blind_perm(n, g, seed=0x424C494E44):
    rng = np.random.default_rng(seed); return rng.integers(0, g ** 3, size=n)

# ------------------------------------------------------------------ VAMP
def covs(X, lag, mean=None, scale=None):
    """Features are STANDARDISED by the (training) SD before whitening: occupancies vary by
    ~1 and momenta by ~1e-4 au, and a ridge relative to the trace otherwise swamps every
    momentum direction (PV-3 read σ = 0.05 on those before this, on building)."""
    A, B = X[:-lag] if lag > 0 else X, X[lag:]
    if mean is None: mean = 0.5 * (A.mean(0) + B.mean(0))
    if scale is None: scale = np.maximum(X.std(0), 1e-300)
    A = (A - mean) / scale; B = (B - mean) / scale
    N = len(A)
    return (mean, scale), A.T @ A / N, A.T @ B / N, B.T @ B / N

def inv_sqrt(C, ridge):
    C = C + ridge * np.eye(len(C))
    w, U = np.linalg.eigh(C); w = np.clip(w, 1e-300, None)
    return U @ np.diag(w ** -0.5) @ U.T

def koopman(X, lag, ridge_rel=1e-3):
    (mean, scale), C00, C0t, Ctt = covs(X, lag)
    ridge = ridge_rel * np.trace(C00) / len(C00)
    W0, Wt = inv_sqrt(C00, ridge), inv_sqrt(Ctt, ridge)
    K = W0 @ C0t @ Wt
    U, s, Vt = np.linalg.svd(K)
    return dict(mean=mean, scale=scale, W0=W0, Wt=Wt, K=K, U=U, s=s, Vt=Vt, C00=C00)

def score_k(s, k): return float((s[:k] ** 2).sum())

def heldout_score(train_X, test_X, lag, k):
    """Wu & Noe's validation score: the top-k singular subspaces are fitted on train (as linear
    functions of the raw features), then the VAMP-2 score of THOSE k-dimensional views is
    computed on test with the test data's own covariances - bounded by k, and immune to a
    variance mismatch between seeds (the first form, train whitening applied to test
    covariances, read scores of 3-5 x k on building whenever a test seed's variance in a
    low-variance train direction was larger)."""
    m = koopman(train_X, lag)
    A = m["W0"] @ m["U"][:, :k]; B = m["Wt"] @ m["Vt"][:k, :].T     # views in standardised coordinates
    Xs = (test_X - m["mean"]) / m["scale"]
    XA, XB = Xs @ A, Xs @ B
    P, Q = XA[:-lag] if lag > 0 else XA, XB[lag:]
    P = P - P.mean(0); Q = Q - Q.mean(0); N = len(P)
    Caa, Cab, Cbb = P.T @ P / N, P.T @ Q / N, Q.T @ Q / N
    ridge = 1e-6 * (np.trace(Caa) / k + np.trace(Cbb) / k) / 2
    Kt = inv_sqrt(Caa, ridge) @ Cab @ inv_sqrt(Cbb, ridge)
    return float(np.linalg.norm(Kt, "fro") ** 2), m

def r2_on_subspace(w, X, cols):
    """R^2 of the function f = w.x (x mean-centred) regressed on the feature columns `cols`."""
    Xc = (X - X.mean(0)) / np.maximum(X.std(0), 1e-300); f = Xc @ w
    Z = Xc[:, cols]; coef, *_ = np.linalg.lstsq(Z, f, rcond=None)
    resid = f - Z @ coef
    return 1.0 - float(resid @ resid) / max(float(f @ f), 1e-300)

# ------------------------------------------------------------------ the read
VIEWS = {
    "cells 2x2x2 occ+mom": lambda C, F: C,   # 7 + 24 = 31
    "cells 2x2x2 occ only": lambda C, F: C[:, :7],
    "fourier all": lambda C, F: F,
    "fourier density": lambda C, F: F[:, FOURIER_DENSITY],
    "fourier transverse current": lambda C, F: F[:, FOURIER_TRANS],
    "fourier longitudinal current": lambda C, F: F[:, FOURIER_LONG],
}

def read(base, src, arm, seeds, taus_fs, dt_fs=20.0):
    data = []
    for k in seeds:
        P, V, L = load_walk(f"{base}/{src}{k}", arm)
        C = cell_features(P, V, L, 2); F = fourier_features(P, V, L)
        B = cell_features(P, V, L, 2, perm=blind_perm(P.shape[1], 2))
        # PER-SEED CENTRING (found on building): each seed's Fourier density modes carry a
        # quasi-static offset of 1-2 counts over its 5-6 ps (a frozen long-wavelength density
        # pattern - this model does not diffuse on that time; LIQUID-1 refused D) and the pooled
        # Koopman read those offsets as 20 ps modes with sigma 0.995, inflating every held-out
        # score past its dimension. Centring by seed removes the static part and reads the
        # dynamics within a seed; the offsets are printed beside the read as what was removed.
        off = np.concatenate([C, F], 1).mean(0)
        C = C - C.mean(0); F = F - F.mean(0); B = B - B.mean(0)
        data.append(dict(P=P, V=V, L=L, C=C, F=F, B=B, D=np.concatenate([C, F], 1), off=off))
    n = data[0]["P"].shape[1]; L = data[0]["L"]
    rho = n * 18.01528 * 1.66053906660e-27 / (L * 5.29177210903e-11) ** 3
    kk = 2 * math.pi / (L * 5.29177210903e-11)
    print(f"# VIEW-SEARCH-1 on {src}* / {arm}: {len(seeds)} seeds, {n} oxygens, L = {L:.3f} bohr, {data[0]['P'].shape[0]} frames at {dt_fs} fs; dictionary dim {data[0]['D'].shape[1]}")
    nC0 = data[0]["C"].shape[1]
    print("# removed by per-seed centring - the density modes' static offsets (counts) per seed, against the mode's within-seed sd:")
    for k, d in zip(seeds, data):
        offs = d["off"][nC0:][FOURIER_DENSITY]; sds = d["F"][:, FOURIER_DENSITY].std(0)
        print(f"#   seed {k}: offsets {np.array2string(offs, precision=2)}  sd {np.array2string(sds, precision=2)}")
    # direct autocorrelation reads per seed, the fast side's own check on S2 and on R2's branch
    kk_m = kk; dt = dt_fs
    print("# direct mode autocorrelations per seed (mean over the three axes' modes):")
    for k, d in zip(seeds, data):
        F = d["F"]
        def acf(cols, lags):
            out = []
            for l in lags:
                num = den = 0.0
                for c in cols:
                    x = F[:, c]; num += (x[:-l] * x[l:]).mean() if l > 0 else (x * x).mean(); den += (x * x).mean()
                out.append(num / den)
            return np.array(out)
        lags = [1, 2, 3, 4, 5, 7, 10, 15, 20, 25]
        aT = acf(FOURIER_TRANS, lags); aD = acf(FOURIER_DENSITY, lags)
        # transverse: exponential fit over lags while acf > 0.2 -> Gamma_s = nu k^2 -> eta = rho Gamma / k^2
        pts = [(l * dt, a) for l, a in zip(lags, aT) if a > 0.2]
        if len(pts) >= 3:
            t = np.array([p[0] for p in pts]); y = np.log([p[1] for p in pts]); g = -np.polyfit(t, y, 1)[0]
            eta = rho * (g * 1e15) / kk_m ** 2
        else: g, eta = float('nan'), float('nan')
        # density: first zero crossing -> a quarter period if oscillatory
        zc = next((i for i in range(1, len(lags)) if aD[i] < 0), None)
        period = 4 * lags[zc] * dt if zc is not None else float('nan')
        cs = 2 * math.pi / (period * 1e-15) / kk_m if zc is not None else float('nan')
        print(f"#   seed {k}: transverse acf {np.array2string(aT[:6], precision=2)} at {[l*dt for l in lags[:6]]} fs -> Γ_s {g*1e3:.2f} /ps, η = ρΓ_s/k² = {eta:.2e} Pa s | density acf {np.array2string(aD, precision=2)} -> first zero at {lags[zc]*dt if zc is not None else float('nan'):.0f} fs, period ≈ {period:.0f} fs, c_s ≈ ω/k = {cs:.0f} m/s")
    out = {}
    for tau_fs in taus_fs:
        lag = int(round(tau_fs / dt_fs))
        print(f"\n== lag {tau_fs:.0f} fs ({lag} frames) ==")
        # the dictionary: held-out bound at each k, timescales, singular functions
        folds = []
        for t in range(len(data)):
            train = np.concatenate([d["D"] for i, d in enumerate(data) if i != t])
            sc, m = heldout_score(train, data[t]["D"], lag, 6)
            folds.append(m)
        m_all = koopman(np.concatenate([d["D"] for d in data]), lag)
        s = m_all["s"]
        times = -tau_fs / np.log(np.clip(s, 1e-12, 1 - 1e-12))
        Xall = np.concatenate([d["D"] for d in data])
        print(f"   dictionary top σ (in-sample, all seeds): {np.array2string(s[:8], precision=3)}")
        print(f"   implied timescales, fs:                {np.array2string(times[:8], precision=0)}")
        nF = data[0]["F"].shape[1]; nC = data[0]["C"].shape[1]
        fcols = list(range(nC, nC + nF))
        weights = []
        for i in range(6):
            w = m_all["W0"] @ m_all["U"][:, i]          # function f_i = w . (x - mean)
            wF = r2_on_subspace(w, Xall, fcols)
            wT = r2_on_subspace(w, Xall, [nC + j for j in FOURIER_TRANS])
            wD = r2_on_subspace(w, Xall, [nC + j for j in FOURIER_DENSITY])
            wLo = r2_on_subspace(w, Xall, [nC + j for j in FOURIER_LONG])
            weights.append((wF, wT, wD, wLo))
            print(f"   σ_{i+1} = {s[i]:.3f}  t = {times[i]:6.0f} fs | R² on Fourier {wF:.2f} (transverse {wT:.2f}, density {wD:.2f}, longitudinal {wLo:.2f})")
        top6F = np.mean([w[0] for w in weights])
        # S2: the leading mode's eta, and the transverse subspace's own leading timescale
        eta_lead = rho / (kk ** 2 * times[0] * 1e-15)
        etas_T = []
        for d in data:
            mT = koopman(d["F"][:, FOURIER_TRANS], lag); tT = -tau_fs / math.log(min(max(mT["s"][0], 1e-12), 1 - 1e-12))
            etas_T.append(rho / (kk ** 2 * tT * 1e-15))
        print(f"   S1: mean R² of top-6 on Fourier = {top6F:.2f} (stake ≥ 0.7; kill < 0.5)")
        print(f"   S2: leading mode transverse weight {weights[0][1]:.2f}; η from t₁ = {eta_lead:.2e} Pa s; transverse subspace's own t₁ per seed -> η = {', '.join(f'{e:.2e}' for e in etas_T)} Pa s (band [2e-4, 3e-3])")
        # views: held-out score vs the held-out bound at the same k
        print(f"   {'view':34} {'k':>3} {'held-out':>9} {'bound_k':>9} {'fraction':>9} {'in-sample':>10} {'D':>6}")
        res = {}
        for name, fn in list(VIEWS.items()) + [("blind cells 2x2x2 (placebo)", None)]:
            ho, bd, ins = [], [], []
            for t in range(len(data)):
                view = lambda d: d["B"] if fn is None else fn(d["C"], d["F"])
                k = view(data[t]).shape[1]
                train = np.concatenate([view(d) for i, d in enumerate(data) if i != t])
                sc, m = heldout_score(train, view(data[t]), lag, k)
                trainD = np.concatenate([d["D"] for i, d in enumerate(data) if i != t])
                bsc, _ = heldout_score(trainD, data[t]["D"], lag, k)
                ho.append(sc); bd.append(bsc); ins.append(score_k(m["s"], k))
            ho, bd, ins = np.mean(ho), np.mean(bd), np.mean(ins)
            frac = ho / bd if bd > 0 else float("nan")
            Dv = math.sqrt(max(1 - ho / k, 0))
            res[name] = (k, ho, bd, frac, ins, Dv)
            print(f"   {name:34} {k:>3} {ho:9.3f} {bd:9.3f} {frac:9.3f} {ins:10.3f} {Dv:6.3f}")
        if tau_fs == 100:
            fa = res["fourier all"]; ca = res["cells 2x2x2 occ+mom"]; bl = res["blind cells 2x2x2 (placebo)"]
            # S3: the cell view at k=24 (its top 24) against the bound at 24
            print(f"   S3: Fourier-all fraction {fa[3]:.2f} (stake ≥ 0.8); cells fraction at its k {ca[3]:.2f} — {'cells BELOW Fourier' if ca[3] < fa[3] else 'cells MATCH/BEAT Fourier: KILL'}")
            print(f"   S4: placebo fraction {bl[3]:.3f} (stake < 0.1): {'holds' if bl[3] < 0.1 else 'FIRES — see the note on single-particle velocity memory'}")
        out[tau_fs] = dict(s=s, times=times, weights=weights, views=res, etas_T=etas_T)
    return out

# ------------------------------------------------------------------ plants
def plants():
    rng = np.random.default_rng(1)
    # PV-1
    # N = 30,000: at the prereg's 3,000 the sampling error of σ is ~0.02 and of t₁ ~10 %,
    # over the plant's own tolerance (found on building; the prereg's N was under-sized)
    A = np.diag([0.9, 0.7, 0.5, 0.0, 0.0, 0.0]); N = 30000
    X = np.zeros((N, 6)); x = np.zeros(6)
    for t in range(N):
        x = A @ x + rng.normal(size=6) * np.array([math.sqrt(1 - 0.81), math.sqrt(1 - 0.49), math.sqrt(0.75), 1, 1, 1])
        X[t] = x
    R = rng.normal(size=(6, 6)); Xm = X @ R          # a random linear mixing: the dictionary is not the eigenbasis
    m = koopman(Xm, 1); s = m["s"][:3]
    t1 = -1 / math.log(s[0])
    ok1 = np.allclose(s, [0.9, 0.7, 0.5], rtol=0.03) and abs(t1 / (-1 / math.log(0.9)) - 1) < 0.05
    print(f"PV-1: top σ {np.array2string(s, precision=3)} vs [0.9 0.7 0.5]; t₁ {t1:.2f} vs {-1/math.log(0.9):.2f} -> {'PASS' if ok1 else 'FAIL'}")
    # PV-2, PV-3, PV-4 on real walks - the read's own fold (fit on two seeds, evaluate on the
    # third): a half/half split of ONE seed was not stationary on building (the flexible arm
    # drifts within a seed) and read a held-out score above its dimension
    base = os.path.join(os.path.dirname(__file__), "replace0")
    Ds = []
    for k in range(3):
        P, V, L = load_walk(f"{base}/transport_seed{k}", "flexible")
        Ds.append(np.concatenate([cell_features(P, V, L, 2), fourier_features(P, V, L)], 1))
    D = Ds[0]
    lag = 5
    sc_true, m_true = heldout_score(np.concatenate(Ds[:2]), Ds[2], lag, 6)
    Dsh = [d[rng.permutation(len(d))] for d in Ds]
    sc_sh, _ = heldout_score(np.concatenate(Dsh[:2]), Dsh[2], lag, 6)
    bound_in = score_k(m_true["s"], 6)
    ok2 = sc_sh < 0.05 * bound_in
    print(f"PV-2: shuffled held-out score {sc_sh:.3f} vs in-sample bound {bound_in:.3f} ({sc_sh/bound_in:.3f}; stake < 0.05; unshuffled held-out {sc_true:.3f}) -> {'PASS' if ok2 else 'FAIL'}")
    # the ridge (1e-6 of the mean variance) bounds |σ − 1| at lag 0 by its own size against the
    # smallest covariance eigenvalue; 1e-4 is that bound, the prereg's 1e-9 assumed no ridge
    m0 = koopman(D, 0); ok3 = np.all(np.abs(m0["s"] - 1) < 1e-4)
    print(f"PV-3: lag 0, max |σ − 1| = {np.abs(m0['s'] - 1).max():.1e} (stake 1e-4, the ridge's size) -> {'PASS' if ok3 else 'FAIL'}")
    # PV-4: density modes as an exact linear image of a 4x4x4 occupancy view
    g = 4; nc = g ** 3
    X = P % L; idx = (np.floor(X / L * g).astype(int) % g)
    cell = (idx[..., 2] * g + idx[..., 1]) * g + idx[..., 0]
    occ = np.zeros((len(P), nc))
    for c in range(nc): occ[:, c] = (cell == c).sum(1)
    centres = (np.arange(g) + 0.5) * L / g; k = 2 * math.pi / L
    W = np.zeros((nc, 6))
    for c in range(nc):
        ix, iy, iz = c % g, (c // g) % g, c // (g * g)
        for ax, ic in enumerate((ix, iy, iz)):
            W[c, 2 * ax] = math.cos(k * centres[ic]); W[c, 2 * ax + 1] = math.sin(k * centres[ic])
    image = occ @ W
    s_src = koopman(occ, lag)["s"]; s_img = koopman(image, lag)["s"]
    ok4 = score_k(s_img, 6) <= score_k(s_src, 6) + 1e-9
    print(f"PV-4: image (6 density modes from 4x4x4 cells) score {score_k(s_img, 6):.3f} ≤ source (64 cells) score {score_k(s_src, 6):.3f} -> {'PASS' if ok4 else 'FAIL'}")
    return ok1 and ok2 and ok3 and ok4

if __name__ == "__main__":
    if sys.argv[1] == "plants":
        sys.exit(0 if plants() else 1)
    base, src, arm = sys.argv[2:5]
    seeds = [int(x) for x in (sys.argv[5] if len(sys.argv) > 5 and not sys.argv[5].startswith("--") else "0,1,2").split(",")]
    taus = [20, 40, 100, 200, 400]
    if "--tau-fs" in sys.argv: taus = [float(x) for x in sys.argv[sys.argv.index("--tau-fs") + 1].split(",")]
    read(base, src, arm, seeds, taus)
