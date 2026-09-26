#!/usr/bin/env python3
"""ORDER-1 (ORDER1_PREREG.md): the order parameter, hunted blind on walks already on disk.

Arm S (structural). Per readout, from the rigid walk's OXYGEN positions and velocities:
  H   the hydrodynamic sector: VIEW-SEARCH-1's first-harmonic density and current modes
      (view_search.fourier_features, 24 columns: rho_k cos/sin and j_k x,y,z cos/sin per axis)
  Q   the coarse tetrahedral-order field at the same k: sum_i (q_i - <q>) cos/sin(k x_i,a),
      a = x, y, z (6 columns); q_i is SLOW-1's Errington-Debenedetti q (slow1_search.structure)
  K   the kinetic-energy density at the same k, sum_i (|v_i|^2 - <v^2>) cos/sin (6; a REPORTED
      control: the walk carries no energies, and the heat mode is the named confound)
  Q0  the box-mean q (1; reported)
PRIMARY FORM: the per-k-vector chains (see `chains`); the literal 30-column dictionary is read
beside it and reported. Arm D (the dipole) is REFUSED: the walks carry no hydrogen, no orientation.

  order1_search.py po1                     # PO-1, the VIEW-SEARCH-1 regression, on the 128-water rigid walks
  order1_search.py synth                   # PO-4 on a synthetic carrier (building only; NOT the plant of record)
  order1_search.py read --t293 D1 D2 D3 --t400 D4 [--scale S] [--max-readouts N]
--scale multiplies every time of the prereg; 1.0 is the reading, anything else is NOT A READING."""
import sys, os, math, argparse
import numpy as np
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, "..", "reasoning"))
from reason_search0b import vamp
import slow1_search as S1
import view_search as VS
from scipy.optimize import curve_fit

LAGS_PS = [0.1, 0.5, 1, 2, 5, 10, 20]; O2_TAUS_PS = [1, 5]
DEN = VS.FOURIER_DENSITY
PLANT_TAU_PS, PLANT_LATENCY_PS, PLANT_NOISE, PLANT_AMP = 5.0, 1.0, 0.05, float(os.environ.get("ORDER1_PLANT_AMP", "0.7"))

# ---------------------------------------------------------------- features
def coarse(P, w, L):
    """sum_i w_i cos/sin(2 pi x_i,a / L) per axis: (T, 6) [x cos, x sin, y cos, y sin, z cos, z sin]"""
    k = 2 * math.pi / L; out = []
    for a in range(3):
        ph = k * (P[..., a] % L); out += [(w * np.cos(ph)).sum(1), (w * np.sin(ph)).sum(1)]
    return np.stack(out, 1)

def walk_features(d, max_readouts=None):
    P, V, L, dt_fs, meta = S1.load(d, max_readouts)
    q = S1.structure(P, L)[:, :, 0]
    H = VS.fourier_features(P, V, L)
    Q = coarse(P, q - q.mean(), L)
    v2 = (V ** 2).sum(2); K = coarse(P, v2 - v2.mean(), L)
    Q0 = q.mean(1, keepdims=True)
    offs = H[:, DEN].mean(0)   # per-seed centring (VIEW-SEARCH-1 Amendment 1, item 5); offsets kept for the record
    H = H - H.mean(0); Q = Q - Q.mean(0); K = K - K.mean(0); Q0 = Q0 - Q0.mean(0)
    return dict(P=P, V=V, L=L, dt_fs=dt_fs, meta=meta, q=q, H=H, Q=Q, K=K, Q0=Q0, offs=offs)

# ---------------------------------------------------------------- the per-k-vector chains (the primary form)
# Modes at different wavevectors are uncorrelated by translation invariance, and a quarter-wavelength
# translation maps (cos, sin) -> (sin, -cos). So each first-harmonic k-vector (axis a) is one chain,
# and each chain is taken twice (as is, and quarter-rotated): six chains of
#   [rho_c, rho_s, jL_c, jL_s, jT1_c, jT1_s, jT2_c, jT2_s, X_c, X_s]
# pooled like SLOW-1's molecules. A scalar field couples at linear order only to rho and jL (isotropy).
C_RHO, C_JL, C_JT, C_X = [0, 1], [2, 3], [4, 5, 6, 7], [8, 9]; C_H = C_RHO + C_JL + C_JT; C_LS = C_RHO + C_JL
def chains(H, X):
    out = []
    for a in range(3):
        b = 8 * a
        cols = [b, b + 1, b + 2 + 2 * a, b + 3 + 2 * a] + [b + 2 + 2 * c + q for c in range(3) if c != a for q in (0, 1)]
        M = np.column_stack([H[:, cols], X[:, 2 * a:2 * a + 2]])
        R = np.empty_like(M); R[:, 0::2], R[:, 1::2] = M[:, 1::2], -M[:, 0::2]
        out += [M, R]
    return out

def cpairs(ch, l, cols):
    return np.concatenate([c[:-l, cols] for c in ch]), np.concatenate([c[l:, cols] for c in ch])

def csig(ch, l, cols):
    return vamp(*cpairs(ch, l, cols))["s"]

def cresid(ch, ycols, xcols):
    """each chain with ycols replaced by ycols minus the pooled equal-time least-squares projection on xcols;
    returns (chains, R^2 of y on x)"""
    Y = np.concatenate([c[:, ycols] for c in ch]); X = np.concatenate([c[:, xcols] for c in ch])
    B = np.linalg.lstsq(X, Y, rcond=None)[0]; out = []
    for c in ch:
        d = c.copy(); d[:, ycols] = c[:, ycols] - c[:, xcols] @ B; out.append(d)
    return out, float(1 - ((Y - X @ B) ** 2).sum() / (Y ** 2).sum())

def ridge_multi(Xtr, Ytr, Xte, Yte, lam=1e-3):
    """multi-output ridge on standardised X; per target column SS_res and SS_tot of standardised Y (train stats)"""
    mx, sx = Xtr.mean(0), Xtr.std(0) + 1e-300; my, sy = Ytr.mean(0), Ytr.std(0) + 1e-300
    A = (Xtr - mx) / sx; Yt = (Ytr - my) / sy
    W = np.linalg.solve(A.T @ A + lam * len(A) * np.eye(A.shape[1]), A.T @ Yt)
    pred = ((Xte - mx) / sx) @ W; Ye = (Yte - my) / sy
    return ((Ye - pred) ** 2).sum(0), ((Ye - Ye.mean(0)) ** 2).sum(0)

def pooled(ssr, sst, idx):
    return float(1 - ssr[idx].sum() / sst[idx].sum())

def ccarried(ch, l, nfold=5, base=C_LS, extra=C_X, target=C_LS):
    """blocked CV in time (training pairs within l of the test block dropped), pooled over the chains:
    R^2 of target(t+l) from base(t) against base+extra(t); pooled over the standardised targets"""
    T = len(ch[0]); n = T - l; edges = np.linspace(0, n, nfold + 1).astype(int); nt = len(target)
    acc = {m: [np.zeros(nt), np.zeros(nt)] for m in ("H", "HX")}
    for b in range(nfold):
        s, e = edges[b], edges[b + 1]; te = np.arange(s, e); tr = np.array([t for t in range(n) if t < s - l or t >= e + l])
        for m, cols in (("H", base), ("HX", base + extra)):
            Xtr = np.concatenate([c[tr][:, cols] for c in ch]); Ytr = np.concatenate([c[tr + l][:, target] for c in ch])
            Xte = np.concatenate([c[te][:, cols] for c in ch]); Yte = np.concatenate([c[te + l][:, target] for c in ch])
            r, tt = ridge_multi(Xtr, Ytr, Xte, Yte); acc[m][0] += r; acc[m][1] += tt
    ir = [target.index(c) for c in C_RHO if c in target]; ij = [target.index(c) for c in C_JL if c in target]
    out = {}
    for m in acc:
        r, tt = acc[m]; out[m] = dict(L=pooled(r, tt, list(range(nt))), D=pooled(r, tt, ir) if ir else float("nan"), JL=pooled(r, tt, ij) if ij else float("nan"))
    out["inc"] = {k: out["HX"][k] - out["H"][k] for k in out["H"]}
    return out

def ccarried_seeds(chs, l, base=C_LS, extra=C_X, target=C_LS):
    """seed-held-out (reported): fit on the other seeds' chains, test on this seed's"""
    incs = []
    for i in range(len(chs)):
        tr = [c for j, ch in enumerate(chs) if j != i for c in ch]; te = chs[i]; res = {}
        for m, cols in (("H", base), ("HX", base + extra)):
            Xtr = np.concatenate([c[:-l, cols] for c in tr]); Ytr = np.concatenate([c[l:, target] for c in tr])
            Xte = np.concatenate([c[:-l, cols] for c in te]); Yte = np.concatenate([c[l:, target] for c in te])
            r, tt = ridge_multi(Xtr, Ytr, Xte, Yte); res[m] = pooled(r, tt, list(range(len(target))))
        incs.append(res["HX"] - res["H"])
    return incs

def with_X(ch, donor):
    """chains whose X columns are taken from another set of chains (the re-paired nulls)"""
    T = min(len(ch[0]), len(donor[0]))
    return [np.column_stack([c[:T, :8], d[:T, 8:10]]) for c, d in zip(ch, donor)]

def carried_full(H, X, l, nfold=5):
    """the literal 30-column form (reported): all 24 H columns at t+l from H(t) against H+X(t)"""
    T = len(H); n = T - l; edges = np.linspace(0, n, nfold + 1).astype(int); acc = {m: [np.zeros(24), np.zeros(24)] for m in ("H", "HX")}
    for b in range(nfold):
        s, e = edges[b], edges[b + 1]; te = np.arange(s, e); tr = np.array([t for t in range(n) if t < s - l or t >= e + l])
        for m, Z in (("H", H), ("HX", np.column_stack([H, X]))):
            r, tt = ridge_multi(Z[tr], H[tr + l], Z[te], H[te + l]); acc[m][0] += r; acc[m][1] += tt
    lcols = DEN + VS.FOURIER_LONG
    return {m: pooled(*acc[m], lcols) for m in acc}

# ---------------------------------------------------------------- O3 helpers
def acf(X, lags):
    """normalised autocorrelation of the columns of X (centred), summed over columns"""
    den = (X * X).mean(0).sum(); return np.array([((X[:-l] * X[l:]).mean(0).sum() if l else den) / den for l in lags])

def fits(t, c):
    """single exponential, double exponential and KWW fits to c(t) (t in ps). TWO TIMES iff the double
    fit halves the residual sum of squares, both amplitudes are >= 0.1 of c at the window's start, and
    the times differ by >= 3x (ORDER1_PREREG.md O3)."""
    out = {}
    try:
        p1, _ = curve_fit(lambda t, a, tau: a * np.exp(-t / tau), t, c, p0=[c[0], 2.0], bounds=([0, 1e-3], [2, 1e3]), maxfev=20000)
        r1 = c - p1[0] * np.exp(-t / p1[1]); out.update(a=p1[0], tau=p1[1], ss1=float((r1 ** 2).sum()), rms1=float(np.sqrt((r1 ** 2).mean())))
    except Exception:
        out.update(a=np.nan, tau=np.nan, ss1=np.nan, rms1=np.nan)
    best = None
    for g1, g2 in ((0.3, 3.0), (0.5, 5.0), (1.0, 10.0), (0.2, 2.0)):
        try:
            f2 = lambda t, a1, t1, a2, t2: a1 * np.exp(-t / t1) + a2 * np.exp(-t / t2)
            p2, _ = curve_fit(f2, t, c, p0=[c[0] * 0.6, g1, c[0] * 0.4, g2], bounds=([0, 1e-3, 0, 1e-3], [2, 1e3, 2, 1e3]), maxfev=40000)
            ss = float(((c - f2(t, *p2)) ** 2).sum())
            if best is None or ss < best[0]: best = (ss, p2)
        except Exception:
            pass
    if best:
        a1, t1, a2, t2 = best[1]
        if t1 > t2: a1, t1, a2, t2 = a2, t2, a1, t1
        out.update(a1=a1, t1=t1, a2=a2, t2=t2, ss2=best[0])
    else:
        out.update(a1=np.nan, t1=np.nan, a2=np.nan, t2=np.nan, ss2=np.nan)
    try:
        pk, _ = curve_fit(lambda t, a, tau, b: a * np.exp(-(t / tau) ** b), t, c, p0=[c[0], 2.0, 0.8], bounds=([0, 1e-3, 0.1], [2, 1e3, 2]), maxfev=20000)
        out.update(kww_tau=pk[1], kww_beta=pk[2])
    except Exception:
        out.update(kww_tau=np.nan, kww_beta=np.nan)
    two = (np.isfinite(out["ss2"]) and out["ss2"] <= 0.5 * out["ss1"] and min(out["a1"], out["a2"]) >= 0.1 * c[0] and out["t2"] >= 3 * out["t1"])
    out["two_times"] = bool(two); return out

def r2_dir(m, X, cols, i):
    """R^2 of the i-th left singular function of VAMP model m (fitted on X's columns) on the columns `cols`"""
    return VS.r2_on_subspace(m["W0"] @ m["U"][:, i], X, cols)

# ---------------------------------------------------------------- the read of one walk
def read_walk(W, scale, label, say=print, planted=None):
    H, Q, dt_fs = W["H"], W["Q"], W["dt_fs"]
    if planted is not None: H, Q = planted["H"], planted["Q"]
    real = planted is None and "K" in W
    T = len(H); rd = lambda ps: max(1, int(round(ps * scale * 1000 / dt_fs)))
    ch = chains(H, Q); res = dict(label=label, lags={}, T=T)
    say(f"## {label}: {T} readouts at {dt_fs:g} fs = {(T-1)*dt_fs/1000:.2f} ps" + (f"; rigid-mode T {W['meta'].get('production_temperature_mean_k', float('nan')):.1f} K" if W.get("meta") else ""))
    chr_, r2qh = cresid(ch, C_X, C_H)
    if real:
        K, Q0 = W["K"], W["Q0"]; chK = chains(H, K)
        chQK = [np.column_stack([c, k[:, 8:10]]) for c, k in zip(ch, chK)]          # [H(8), Q(2), K(2)]
        chrK, r2qhk = cresid(chQK, C_X, C_H + [10, 11])
        cq = [float(np.corrcoef(H[:, DEN[j]], Q[:, j])[0, 1]) for j in range(6)]; res["corr_rhoQ"] = float(np.mean(cq))
        res["R2_Q_on_H"] = r2qh
        say(f"   <q> = {W['q'].mean():.4f}, sd per molecule {W['q'].std():.4f}; density-mode static offsets removed (counts): {np.array2string(W['offs'], precision=2)}")
        say(f"   equal-time corr(rho_k, Q_k) per column {np.array2string(np.array(cq), precision=3)} (mean {np.mean(cq):+.3f}; the two-state sign is NEGATIVE)")
        say(f"   equal-time R^2 of Q_k on its own k-vector's hydrodynamic sector: {r2qh:.4f}; on it plus K_k: {r2qhk:.4f}")
    rng = np.random.default_rng(11); perm = rng.permutation(T); chsh = [c[perm] for c in ch]
    for ps in LAGS_PS:
        l = rd(ps)
        if l >= T // 2: say(f"   lag {ps*scale:g} ps: SKIPPED (longer than half the walk)"); continue
        s = csig(ch, l, C_X); sr = csig(chr_, l, C_X)[0]; ssh = csig(chsh, l, C_X)[0]
        row = dict(l=l, s1=s[0], s=s, res=sr, ratio=sr / s[0], shuf=ssh)
        extra = ""
        if real:
            row.update(resK=csig(chrK, l, C_X)[0], H1=csig(ch, l, C_LS)[0], K1=csig(chK, l, C_X)[0], Q01=vamp(Q0[:-l], Q0[l:])["s"][0])
            row["full_s1"] = vamp(Q[:-l], Q[l:])["s"][0]
            Qfr = Q - H @ np.linalg.lstsq(H, Q, rcond=None)[0]; row["full_ratio"] = vamp(Qfr[:-l], Qfr[l:])["s"][0] / row["full_s1"]
            A, B = cpairs(ch, l, C_H + C_X); m = vamp(A, B); X = np.concatenate([c[:, C_H + C_X] for c in ch]); qd = None
            for i in range(len(m["s"])):
                r2q = r2_dir(m, X, [8, 9], i)
                if r2q >= 0.5:
                    qd = dict(i=i + 1, s=m["s"][i], r2q=r2q, r2d=r2_dir(m, X, C_RHO, i), r2l=r2_dir(m, X, C_JL, i)); break
            row["joint_top"] = m["s"][:4]; row["qdir"] = qd
            extra = (f" | minus H+K {row['resK']:.3f} | own sigma1: (rho,jL) {row['H1']:.3f}, K {row['K1']:.3f}, Q0 {row['Q01']:.3f} | literal 30-col: Q sigma1 {row['full_s1']:.3f}, ratio {row['full_ratio']:.2f}"
                     f" | joint (H,Q) top {np.array2string(row['joint_top'], precision=3)}; Q-dominant direction " + (f"#{qd['i']} sigma {qd['s']:.3f}, R^2 on Q {qd['r2q']:.2f}, on rho {qd['r2d']:.2f}, on jL {qd['r2l']:.2f}" if qd else "none"))
        res["lags"][ps] = row
        say(f"   lag {ps*scale:6.3g} ps ({l:4d} rd): Q sigma {np.array2string(s, precision=3)} | minus H: sigma1 {sr:.3f} (ratio {sr/s[0]:.2f}) | time-shuffled {ssh:.3f}" + extra)
    # O2
    res["O2"] = {}
    for ps in O2_TAUS_PS:
        l = rd(ps)
        if l >= T // 3: continue
        c = ccarried(ch, l); res["O2"][ps] = c
        c["null_shift"] = ccarried(with_X(ch, [np.roll(x, T // 2, axis=0) for x in ch]), l)["inc"]["L"]
        line = (f"   O2 tau {ps*scale:g} ps: R^2 of (rho, jL)(t+tau): H {c['H']['L']:.4f} -> H+Q {c['HX']['L']:.4f}, increment {c['inc']['L']:+.4f} (rho {c['inc']['D']:+.4f}, jL {c['inc']['JL']:+.4f}; "
                f"H alone rho {c['H']['D']:.3f}, jL {c['H']['JL']:.3f}) | null, Q shifted T/2: {c['null_shift']:+.4f}")
        if real:
            ck = ccarried(chQK, l, base=C_LS + [10, 11], extra=C_X); c["beyondK"] = ck["inc"]["L"]
            kk = ccarried(chQK, l, base=C_LS, extra=[10, 11]); c["K_inc"] = kk["inc"]["L"]
            cf = carried_full(H, Q, l); c["full_inc"] = cf["HX"] - cf["H"]
            line += f" | beyond (rho,jL)+K {ck['inc']['L']:+.4f}; K's own increment {kk['inc']['L']:+.4f} | literal 30-col increment {c['full_inc']:+.4f}"
        say(line)
    # O3
    lagsr = sorted(set([rd(x) for x in np.arange(0.1, 20.01, 0.1)] + [0])); lagsr = [l for l in lagsr if l < T // 2]
    tps = np.array(lagsr) * dt_fs / 1000; w = (tps >= 0.5 * scale - 1e-9) & (tps <= 20 * scale + 1e-9)
    Qall = np.concatenate([c[:, C_X] for c in ch[::2]], 1)
    cQ = acf(Qall, lagsr); fq = fits(tps[w], cQ[w]); res["fitQ"] = fq; res["acfQ"] = (tps, cQ)
    at = lambda arr, x: arr[int(np.argmin(abs(tps - x * scale)))]
    say(f"   O3 ACF(Q_k) at 0.1/0.5/1/2/5/10/20 ps: " + ", ".join(f"{at(cQ, x):.3f}" for x in (0.1, 0.5, 1, 2, 5, 10, 20)))
    say(f"      fit on [0.5, 20] ps: single tau {fq['tau']:.3f} ps (amp {fq['a']:.3f}, rms {fq['rms1']:.4f}); double {fq['a1']:.3f} e^(-t/{fq['t1']:.3f}) + {fq['a2']:.3f} e^(-t/{fq['t2']:.3f}) (SS {fq['ss2']:.2e} vs {fq['ss1']:.2e}); KWW tau {fq['kww_tau']:.3f} beta {fq['kww_beta']:.2f} -> {'TWO TIMES' if fq['two_times'] else 'one time'}")
    if real:
        qc = W["q"] - W["q"].mean(0); cq_ = acf(qc, lagsr); fs = fits(tps[w], cq_[w]); res["fitq"] = fs
        say(f"      per-molecule q ACF (self) at 0.1/0.5/1/2/5/10/20 ps: " + ", ".join(f"{at(cq_, x):.3f}" for x in (0.1, 0.5, 1, 2, 5, 10, 20)))
        say(f"      self fit: single tau {fs['tau']:.3f} ps (rms {fs['rms1']:.4f}); double {fs['a1']:.3f}/{fs['t1']:.3f} + {fs['a2']:.3f}/{fs['t2']:.3f} -> {'TWO TIMES' if fs['two_times'] else 'one time'}; KWW tau {fs['kww_tau']:.3f} beta {fs['kww_beta']:.2f}")
        say(f"      tau(Q_k) / tau(q self) = {fq['tau']/fs['tau']:.2f}")
        P = W["P"]; kk = 2.2; ta = float("nan")
        for l in lagsr[1:]:
            dr = P[l:] - P[:-l]; f = np.mean([np.cos(kk * dr[..., a]).mean() for a in range(3)])
            if f < math.exp(-1): ta = l * dt_fs / 1000; break
        res["tau_alpha"] = ta; say(f"      alpha relaxation: F_s(k = 2.2 A^-1, t) falls below 1/e at {ta:.2f} ps")
    return res

# ---------------------------------------------------------------- PO-4: the planted slow field
def plant(W, seed=5, scale=1.0):
    """on the carrier's H: six independent OU fields Z (PLANT_TAU_PS memory, unit variance), one per
    (axis, quadrature), i.e. a complex OU per k-vector; rho'(t) = rho(t) + sd(rho) Z(t - latency) on
    the density column Z is paired with; the planted Q = sd(Q) (Z + noise of PLANT_NOISE of its variance)"""
    H, Q, dt_fs = W["H"].copy(), W["Q"], W["dt_fs"]; T = len(H); rng = np.random.default_rng(seed)
    a = math.exp(-dt_fs / (PLANT_TAU_PS * scale * 1000)); lat = int(round(PLANT_LATENCY_PS * scale * 1000 / dt_fs))
    Z = np.zeros((T + lat, 6)); z = rng.normal(size=6)
    for t in range(T + lat): Z[t] = z; z = a * z + math.sqrt(1 - a * a) * rng.normal(size=6)
    Zt = Z[lat:]; Zd = Z[:T]
    for j in range(6): H[:, DEN[j]] += PLANT_AMP * H[:, DEN[j]].std() * Zd[:, j]
    Qp = (Zt + rng.normal(size=Zt.shape) * math.sqrt(PLANT_NOISE)) * Q.std()
    H -= H.mean(0); Qp -= Qp.mean(0)
    return dict(H=H, Q=Qp, Z=Zt - Zt.mean(0))

def run_plant(W, scale, label, say=print):
    pl = plant(W, scale=scale)
    say(f"# PO-4 on {label}: a complex OU field per k-vector ({PLANT_TAU_PS*scale:g} ps), driving rho_k with {PLANT_LATENCY_PS*scale:g} ps latency at one sd; planted Q with noise {PLANT_NOISE:g} of its variance")
    r = read_walk({k: v for k, v in W.items() if k != "K"}, scale, "PO-4 " + label, say=say, planted=pl)
    one = [ps for ps in r["lags"] if ps >= 1]
    best = max(one, key=lambda ps: r["lags"][ps]["s1"]); l = r["lags"][best]["l"]
    ch = chains(pl["H"], pl["Q"]); chz = chains(pl["H"], pl["Z"])
    A, B = cpairs(ch, l, C_X); m = vamp(A, B); w = m["W0"] @ m["U"][:, 0]
    f = ((np.concatenate([c[:, C_X] for c in ch]) - m["mean"]) / m["scale"]) @ w
    load_ = S1.r2_fit(np.concatenate([c[:, C_X] for c in chz]), f)
    s1ok = any(r["lags"][ps]["s1"] >= 0.3 and r["lags"][ps]["ratio"] >= 0.7 for ps in one)
    incs = {ps: r["O2"][ps]["inc"]["L"] for ps in r["O2"]}; nulls = {ps: r["O2"][ps]["null_shift"] for ps in r["O2"]}
    ok = s1ok and load_ >= 0.9 and max(incs.values()) >= 0.05 and max(nulls.values()) <= 0.01
    say(f"PO-4: O1 legs (sigma1 >= 0.3 and ratio >= 0.7 at a lag >= 1 ps) {'met' if s1ok else 'NOT met'}; loading of Q's top closed direction at {best*scale:g} ps on the planted Z: R^2 {load_:.3f} (bar >= 0.9); O2 increment {', '.join(f'{p*scale:g} ps {v:+.4f}' for p, v in incs.items())} (bar >= 0.05 at one tau); null {', '.join(f'{v:+.4f}' for v in nulls.values())} (bar <= 0.01) -> {'PASS' if ok else 'FAIL'}")
    return ok, dict(load=load_, incs=incs, nulls=nulls, s1ok=s1ok)

# ---------------------------------------------------------------- PO-1: the VIEW-SEARCH-1 regression
PO1_BANKED = {"fourier all": (13.238, 12.163), "fourier density": (4.945, 5.648), "cells 2x2x2 occ+mom": (11.767, 15.119)}
def po1(base="/home/emoore/CIRISHolon/conformance/water_observatory/replace0"):
    data = []
    for k in range(3):
        P, V, L, dt_fs, _ = S1.load(f"{base}/transport_seed{k}")
        C = VS.cell_features(P, V, L, 2); F = VS.fourier_features(P, V, L)
        C = C - C.mean(0); F = F - F.mean(0); data.append(dict(C=C, F=F, D=np.concatenate([C, F], 1)))
    lag = 5; ok = True
    print("# PO-1: the hydrodynamic sector as ORDER-1 builds it (slow1_search.load, Angstrom) on the 128-water rigid transport walks, VIEW-SEARCH-1's held-out read at 100 fs")
    for name, fn in (("fourier all", lambda d: d["F"]), ("fourier density", lambda d: d["F"][:, DEN]), ("cells 2x2x2 occ+mom", lambda d: d["C"])):
        ho, bd = [], []
        for t in range(3):
            k = fn(data[t]).shape[1]
            sc, _ = VS.heldout_score(np.concatenate([fn(d) for i, d in enumerate(data) if i != t]), fn(data[t]), lag, k)
            bsc, _ = VS.heldout_score(np.concatenate([d["D"] for i, d in enumerate(data) if i != t]), data[t]["D"], lag, k)
            ho.append(sc); bd.append(bsc)
        h, b = np.mean(ho), np.mean(bd); eh, eb = PO1_BANKED[name]; good = abs(h - eh) <= 1.5e-3 and abs(b - eb) <= 1.5e-3; ok &= good
        print(f"PO-1 {name:22}: held-out {h:.3f} (banked {eh:.3f}), bound {b:.3f} (banked {eb:.3f}) -> {'PASS' if good else 'FAIL'}")
    print(f"PO-1 -> {'PASS' if ok else 'FAIL'}")
    return ok

# ---------------------------------------------------------------- a synthetic carrier (building only)
def synth_walk(T=2501, dt_fs=20.0, seed=3):
    """a stand-in hydrodynamic sector for building: density = a damped 2 ps oscillator plus a 100 ps
    OU offset, currents 0.3 ps OU; Q an unrelated 1 ps OU field. NOT the carrier of record."""
    rng = np.random.default_rng(seed); dt = dt_fs / 1000; H = np.zeros((T, 24)); Q = np.zeros((T, 6))
    w0, g = 2 * math.pi / 2.0, 1.0; x = rng.normal(size=6); v = rng.normal(size=6) * w0; slow = rng.normal(size=6); cur = rng.normal(size=18); qq = rng.normal(size=6)
    oth = [c for c in range(24) if c not in DEN]
    for t in range(T):
        H[t, DEN] = x + 0.7 * slow; H[t, oth] = cur; Q[t] = qq
        v += (-w0 ** 2 * x - 2 * g * v) * dt + math.sqrt(4 * g * w0 ** 2 * dt) * rng.normal(size=6); x += v * dt
        slow = math.exp(-dt / 100) * slow + math.sqrt(1 - math.exp(-2 * dt / 100)) * rng.normal(size=6)
        cur = math.exp(-dt / 0.3) * cur + math.sqrt(1 - math.exp(-2 * dt / 0.3)) * rng.normal(size=18)
        qq = math.exp(-dt / 1.0) * qq + math.sqrt(1 - math.exp(-2 * dt / 1.0)) * rng.normal(size=6)
    H -= H.mean(0); Q -= Q.mean(0)
    return dict(H=H, Q=Q, dt_fs=dt_fs, meta={})

# ---------------------------------------------------------------- the verdict
def verdict(r293, r400, plants_ok, scale):
    print(f"\n# VERDICT TABLE{'  -- NOT A READING (time scale x%g)' % scale if scale != 1 else ''}")
    one = lambda r: [ps for ps in r["lags"] if ps >= 1]
    per = []
    for r in r293:
        met = [ps for ps in one(r) if r["lags"][ps]["s1"] >= 0.3 and r["lags"][ps]["ratio"] >= 0.7]
        kill = all(r["lags"][ps]["s1"] < 0.1 for ps in one(r)) or all(r["lags"][ps]["ratio"] < 0.3 for ps in one(r))
        per.append((met, kill))
    o1 = "MET" if all(m for m, _ in per) else ("KILL" if all(k for _, k in per) else "between")
    print(f"O1 closed: per 293 K seed, lags >= 1 ps with own sigma1 >= 0.3 and residual ratio >= 0.7: {[[p*scale for p in m] for m, _ in per]} -> {o1}")
    for r in r293 + ([r400] if r400 else []):
        print(f"   {r['label']}: sigma1 / ratio at " + "; ".join(f"{ps*scale:g} ps {r['lags'][ps]['s1']:.3f}/{r['lags'][ps]['ratio']:.2f}" for ps in r["lags"]))
    taus = sorted(set().union(*[r["O2"].keys() for r in r293]))
    inc = {ps: [r["O2"][ps]["inc"]["L"] for r in r293 if ps in r["O2"]] for ps in taus}
    i400 = {ps: (r400["O2"][ps]["inc"]["L"] if r400 and ps in r400["O2"] else float("nan")) for ps in taus}
    met_t = [ps for ps in taus if all(x >= 0.05 for x in inc[ps]) and i400[ps] < np.mean(inc[ps])]
    kill = all(np.mean(inc[ps]) < 0.02 for ps in taus)
    o2 = "MET" if met_t else ("KILL" if kill else "between")
    print("O2 carried: (rho, jL) increment at 293 K " + "; ".join(f"{ps*scale:g} ps {[round(x, 4) for x in inc[ps]]} (mean {np.mean(inc[ps]):+.4f})" for ps in taus) + " | 400 K " + ", ".join(f"{ps*scale:g} ps {i400[ps]:+.4f}" for ps in taus) + f" -> {o2}")
    o3s = []
    for r in r293:
        fq = r["fitQ"]; best = max(one(r), key=lambda ps: r["lags"][ps]["s1"]); qd = r["lags"][best].get("qdir")
        mix = bool(qd and qd["r2d"] >= 0.2); two = fq["two_times"]
        o3s.append("DIRECTION" if (mix or two) else "RE-FINDING")
        print(f"O3 {r['label']}: tau(Q) {fq['tau']:.3f} ps vs q self {r.get('fitq', {}).get('tau', float('nan')):.3f} ps, alpha {r.get('tau_alpha', float('nan')):.2f} ps; {'two times' if two else 'one time'}; Q-dominant direction at {best*scale:g} ps R^2 on rho {qd['r2d'] if qd else float('nan'):.3f} ({'mixture' if mix else 'not a mixture'}) -> {o3s[-1]}")
    if not plants_ok: br = "(e) a plant fails - nothing is read"
    elif o1 == "KILL": br = "(c) O1 killed - no structural mode beyond hydrodynamics at these k"
    elif o2 == "KILL": br = "(d) O2 killed - closed, not carried"
    elif o1 == "MET" and o2 == "MET" and all(x == "DIRECTION" for x in o3s): br = "(a) DIRECTION"
    elif o1 == "MET" and o2 == "MET" and all(x == "RE-FINDING" for x in o3s): br = "(b) RE-FINDING"
    else: br = f"none of (a)-(e) cleanly: O1 {o1}, O2 {o2}, O3 per 293 K seed {o3s}"
    print(f"BRANCH: {br}" + ("  [NOT A READING]" if scale != 1 else ""))
    return dict(o1=o1, o2=o2, o3=o3s, branch=br)

if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("mode", choices=["po1", "synth", "read"])
    ap.add_argument("--t293", nargs="*", default=[]); ap.add_argument("--t400", nargs="*", default=[])
    ap.add_argument("--scale", type=float, default=1.0); ap.add_argument("--max-readouts", type=int, default=None)
    a = ap.parse_args()
    if a.mode == "po1": sys.exit(0 if po1() else 1)
    if a.mode == "synth":
        ok, _ = run_plant(synth_walk(), 1.0, "the synthetic carrier (building only, NOT the plant of record)"); sys.exit(0)
    if a.scale != 1: print(f"# NOT A READING: every time of the prereg multiplied by {a.scale:g}\n")
    print("# Arm D (the collective dipole): REFUSED by name - the rigid walks carry oxygen positions and velocities only (3 x 250 columns per readout); no hydrogen, no orientation.")
    ok1 = po1(); print()
    Ws = {d: walk_features(d, a.max_readouts) for d in a.t293 + a.t400}
    name = lambda d, T: os.path.basename(d.rstrip("/")) + f" ({T} K arm)"
    ok4, _ = run_plant(Ws[a.t293[0]], a.scale, name(a.t293[0], 293)); print()   # PO-4 on the carrier of record
    r293 = [read_walk(Ws[d], a.scale, name(d, 293)) for d in a.t293]; print()
    r400 = [read_walk(Ws[d], a.scale, name(d, 400)) for d in a.t400]; r4 = r400[0] if r400 else None
    print("\n# PO-2: re-paired nulls on the (rho, jL) increment")
    ok2 = True; ds = a.t293
    chs = [chains(Ws[d]["H"], Ws[d]["Q"]) for d in ds]
    for i, d in enumerate(ds):
        for ps in r293[i]["O2"]:
            l = max(1, int(round(ps * a.scale * 1000 / Ws[d]["dt_fs"])))
            sw = ccarried(with_X(chs[i], chs[(i + 1) % len(ds)]), l)["inc"]["L"] if len(ds) > 1 else float("nan")
            sh = r293[i]["O2"][ps]["null_shift"]; r293[i]["O2"][ps]["null_swap"] = sw
            good = sh <= 0.01 and (not np.isfinite(sw) or sw <= 0.01); ok2 &= good
            print(f"PO-2 {r293[i]['label']} tau {ps*a.scale:g} ps: time-shifted {sh:+.4f}, seed-swapped {sw:+.4f} (bar <= 0.01) -> {'PASS' if good else 'FAIL'}")
    Tm = min(len(c[0]) for c in chs)
    for ps in O2_TAUS_PS:
        l = max(1, int(round(ps * a.scale * 1000 / Ws[ds[0]]["dt_fs"])))
        if l < Tm // 3 and len(ds) > 1: print(f"   seed-held-out (rho, jL) increment (reported) tau {ps*a.scale:g} ps: {[round(x, 4) for x in ccarried_seeds([[c[:Tm] for c in ch] for ch in chs], l)]}")
    ok3 = True
    for r in r293 + r400:
        m = max(r["lags"][ps]["shuf"] for ps in r["lags"] if ps >= 1); good = m < 0.1; ok3 &= good
        print(f"PO-3 {r['label']}: time-shuffled Q sigma1, max over lags >= 1 ps {m:.3f} (bar < 0.1) -> {'PASS' if good else 'FAIL'}")
    ok5 = True
    if r4:
        l1 = min(ps for ps in r4["lags"] if ps >= 1)
        s4 = r4["lags"][l1]["s1"]; s2 = min(r["lags"][l1]["s1"] for r in r293); ok5 = s4 < s2
        print(f"PO-5: Q's own sigma1 at {l1*a.scale:g} ps, 400 K {s4:.3f} < min over 293 K {s2:.3f} -> {'PASS' if ok5 else 'FAIL'}")
    print(f"plants: PO-1 {'PASS' if ok1 else 'FAIL'}, PO-2 {'PASS' if ok2 else 'FAIL'}, PO-3 {'PASS' if ok3 else 'FAIL'}, PO-4 {'PASS' if ok4 else 'FAIL'}, PO-5 {'PASS' if ok5 else 'FAIL'}")
    verdict(r293, r4, ok1 and ok2 and ok3 and ok4 and ok5, a.scale)
