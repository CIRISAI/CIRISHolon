#!/usr/bin/env python3
"""SLOW-1 (SLOW1_PREREG.md): the search on an open question - the slow variable of a sluggish liquid.

Dictionary per molecule per readout, from the rigid walk's OXYGEN positions and velocities:
  STRUCT  q (Errington-Debenedetti, four nearest O), n33 / n50 (O within 3.3 / 5.0 A), nb (O-O < 3.5 A),
          s2 (Piaggi-Parrinello local pair excess entropy) and s2w (weighted by exp(r/xi); SLOW1_AMENDMENT_1.md)
  HIST    h1 h2 h5 h10: |r(t) - r(t - D)|, D in {1, 2, 5, 10} ps (backward, known at t)
  MOM     vx vy vz (the control that must NOT be slow)
  MODE    cos/sin(2 pi x_a / L), a = x, y, z (the first-harmonic density-mode phase; reported)
The VAMP core is reason_search0b's (vamp, blocks, heldout), the one hbond_search.py imports.

  slow1_search.py plants [--out F]
  slow1_search.py read|arms --t293 D1 [D2 ...] --t400 D4 [--scale S] [--max-readouts N] [--label L]
--scale multiplies EVERY time of the prereg (lags, history, horizon, the >= 1 ps thresholds);
1.0 is the reading, anything else is a dry run and is printed as NOT A READING."""
import sys, os, math, json, argparse
import numpy as np
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, "..", "reasoning"))
from reason_search0b import vamp, blocks, heldout

BOHR_A = 0.529177210903
STRUCT = [0, 1, 2, 3, 17, 18]; HIST = [4, 5, 6, 7]; MOM = [8, 9, 10]; MODE = [11, 12, 13, 14, 15, 16]
NAMES = ["q", "n33", "n50", "nb", "h1", "h2", "h5", "h10", "vx", "vy", "vz", "cx", "sx", "cy", "sy", "cz", "sz", "s2", "s2w"]
LAGS_PS = [0.1, 0.5, 1, 2, 5, 10, 20]; HIST_PS = [1, 2, 5, 10]; HORIZON_PS = 5.0; SECTOR_LAGS_PS = [1, 2, 5]
# S3 candidates (STRUCT column groups) and S2 candidates (those plus each history column)
# S3_POS indexes positions inside STRUCT (q n33 n50 nb s2 s2w [planted]); S2_CANDS indexes dictionary columns
S3_POS = {"q": [0], "s2": [4], "s2w": [5], "density": [1, 2], "bond count": [3]}; S3_CANDS = S3_POS
S2_CANDS = {"q": [0], "s2": [17], "s2w": [18], "density": [1, 2], "bond count": [3], "h1": [4], "h2": [5], "h5": [6], "h10": [7]}
S2_RM, S2_SIG, S2_DR = 7.5, 0.15, 0.03   # SLOW1_AMENDMENT_1.md: r_m, the Gaussian width, the radial grid (A)

# ---------------------------------------------------------------- loading and the dictionary
def load(d, max_readouts=None):
    """rigid.walk / rigid.vwalk: header '# walk N L' (L in bohr), then one row per readout of
    3 n numbers (unwrapped oxygen positions, bohr) / velocities (au). A running arm's last row may
    be partial: only complete rows common to both files are used."""
    def rows(p):
        with open(p) as fh:
            head = fh.readline().split(); L = float(head[3]); out = []
            for ln in fh:
                x = ln.split()
                if not x: continue
                out.append(x)
        return L, out
    L, pr = rows(f"{d}/rigid.walk"); _, vr = rows(f"{d}/rigid.vwalk")
    n3 = len(pr[0]); k = min(len(pr), len(vr))
    while k and (len(pr[k - 1]) != n3 or len(vr[k - 1]) != n3): k -= 1
    if max_readouts: k = min(k, max_readouts + 1)
    P = np.array(pr[:k], float).reshape(k, -1, 3) * BOHR_A; V = np.array(vr[:k], float).reshape(k, -1, 3)
    meta = {}
    try: meta = json.load(open(f"{d}/scout.json"))
    except Exception: pass
    dt_fs = float(meta.get("readout_fs", 20.0))
    return P, V, L * BOHR_A, dt_fs, meta

def structure(P, L):
    """per frame per molecule: q, n33, n50, nb from wrapped positions under the minimum image."""
    T, n, _ = P.shape; out = np.zeros((T, n, 4))
    for t in range(T):
        X = P[t] % L; d = X[None, :, :] - X[:, None, :]; d -= L * np.round(d / L)
        r = np.sqrt((d ** 2).sum(2)); np.fill_diagonal(r, np.inf)
        nn = np.argpartition(r, 4, axis=1)[:, :4]
        u = np.take_along_axis(d, nn[:, :, None], axis=1); u /= np.linalg.norm(u, axis=2)[:, :, None]
        c = np.einsum("ijk,ilk->ijl", u, u); iu = np.triu_indices(4, 1)
        out[t, :, 0] = 1 - 3.0 / 8.0 * ((c[:, iu[0], iu[1]] + 1.0 / 3.0) ** 2).sum(1)
        out[t, :, 1] = (r < 3.3).sum(1); out[t, :, 2] = (r < 5.0).sum(1); out[t, :, 3] = (r < 3.5).sum(1)
    return out

def _g_frames(P, L, frames):
    """per-molecule smoothed g_i(r) on the grid for the given frames: (len(frames), n, nr), and the grid."""
    T, n, _ = P.shape; rho = n / L ** 3; edges = np.arange(0, S2_RM + 4 * S2_SIG + S2_DR, S2_DR); nb_ = len(edges) - 1
    rc = 0.5 * (edges[1:] + edges[:-1]); kx = np.arange(-int(4 * S2_SIG / S2_DR), int(4 * S2_SIG / S2_DR) + 1) * S2_DR
    ker = np.exp(-kx ** 2 / (2 * S2_SIG ** 2)); ker /= ker.sum() * S2_DR   # unit area per neighbour, density per A
    out = np.zeros((len(frames), n, nb_))
    for k, t in enumerate(frames):
        X = P[t] % L; d = X[None, :, :] - X[:, None, :]; d -= L * np.round(d / L); r = np.sqrt((d ** 2).sum(2)); np.fill_diagonal(r, np.inf)
        ii, jj = np.nonzero(r < edges[-1]); b = (r[ii, jj] / S2_DR).astype(int)
        h = np.bincount(ii * nb_ + b, minlength=n * nb_).reshape(n, nb_).astype(float)
        hs = np.zeros_like(h); m = len(kx) // 2
        for q_, kv in enumerate(ker):
            sh = q_ - m
            if sh >= 0: hs[:, sh:] += kv * h[:, :nb_ - sh]
            else: hs[:, :sh] += kv * h[:, -sh:]
        out[k] = hs / (4 * math.pi * rho * rc ** 2)
    return out, rc, rho

def xi_of(P, L):
    """the decay length of the envelope of |g(r) - 1| on this walk's pooled g: least squares of
    ln|g - 1| at its local maxima beyond the first peak, out to r_m (SLOW1_AMENDMENT_1.md)."""
    T = P.shape[0]; fr = list(range(0, T, max(1, T // 100)))
    g, rc, _ = _g_frames(P, L, fr); g = g.mean((0, 1)); a = np.abs(g - 1); keep = rc <= S2_RM
    i1 = int(np.argmax(g)); pk = [i for i in range(i1 + 1, len(a) - 1) if keep[i] and a[i] >= a[i - 1] and a[i] >= a[i + 1] and a[i] > 1e-3]
    if len(pk) < 2: return float("inf"), pk
    sl = np.polyfit(rc[pk], np.log(a[pk]), 1)[0]; return (-1.0 / sl if sl < 0 else float("inf")), [round(float(rc[i]), 2) for i in pk]

def pair_entropy(P, L, xi, chunk=50):
    """(T, n, 2): s2 and s2w per molecule per readout, units of k_B."""
    T, n, _ = P.shape; out = np.zeros((T, n, 2))
    for c0 in range(0, T, chunk):
        fr = list(range(c0, min(T, c0 + chunk))); g, rc, rho = _g_frames(P, L, fr); keep = rc <= S2_RM
        gg = g[:, :, keep]; r = rc[keep]; w = np.exp(r / xi) if np.isfinite(xi) else np.ones_like(r)
        with np.errstate(divide="ignore", invalid="ignore"): gl = np.where(gg > 0, gg * np.log(gg), 0.0)
        integ = (gl - gg + 1) * r ** 2
        out[c0:c0 + len(fr), :, 0] = -2 * math.pi * rho * integ.sum(2) * S2_DR
        out[c0:c0 + len(fr), :, 1] = -2 * math.pi * rho * (integ * w).sum(2) * S2_DR
    return out

def dictionary(P, V, L, hist_rd):
    """(n molecules) list of (T, 19) arrays; HIST is NaN where t < its lag. Returns (feats, xi, xi_peaks)."""
    T, n, _ = P.shape; S = structure(P, L); F = np.full((T, n, 19), np.nan)
    xi, pk = xi_of(P, L); F[:, :, 17:19] = pair_entropy(P, L, xi)
    F[:, :, 0:4] = S
    for j, h in enumerate(hist_rd):
        if h < T: F[h:, :, 4 + j] = np.linalg.norm(P[h:] - P[:-h], axis=2)
    F[:, :, 8:11] = V
    X = (P % L) * 2 * math.pi / L
    for a in range(3): F[:, :, 11 + 2 * a] = np.cos(X[:, :, a]); F[:, :, 12 + 2 * a] = np.sin(X[:, :, a])
    return [F[:, i, :] for i in range(n)], xi, pk

# ---------------------------------------------------------------- the search
def pairs(feats, lag, cols, t0=0):
    A = np.concatenate([f[t0:-lag, cols] for f in feats]); B = np.concatenate([f[t0 + lag:, cols] for f in feats]); return A, B

def shuffled(feats, seed=11):
    rng = np.random.default_rng(seed); return [f[rng.permutation(len(f))] for f in feats]

def r2_fit(X, y):
    X1 = np.column_stack([X, np.ones(len(X))]); b = np.linalg.lstsq(X1, y, rcond=None)[0]
    res = y - X1 @ b; return float(1 - (res ** 2).sum() / ((y - y.mean()) ** 2).sum())

def ridge_r2(Xtr, ytr, Xte, yte):
    m_, s_ = Xtr.mean(0), Xtr.std(0) + 1e-12; A = (Xtr - m_) / s_
    W = np.linalg.solve(A.T @ A + 1e-3 * len(A) * np.eye(A.shape[1]), A.T @ (ytr - ytr.mean()))
    pred = ((Xte - m_) / s_) @ W + ytr.mean(); return float(1 - ((yte - pred) ** 2).sum() / ((yte - yte.mean()) ** 2).sum())

def alpha2(P, max_lag):
    out = []
    for l in range(1, max_lag + 1):
        d2 = ((P[l:] - P[:-l]) ** 2).sum(2); out.append((l, 3 * (d2 ** 2).mean() / (5 * d2.mean() ** 2) - 1, d2.mean()))
    return out

def read_arm(d, scale, max_readouts=None, label="", synth=None, verbose=True):
    """one walk: the search at every lag, the sector, S2 inputs, PS-1/2/4, alpha_2. Returns a dict."""
    say = print if verbose else (lambda *a, **k: None)
    if synth is None:
        P, V, L, dt_fs, meta = load(d, max_readouts)
    else:
        P, V, L, dt_fs, meta = synth["P"], synth["V"], synth["L"], synth["dt_fs"], {}
    T, n, _ = P.shape; rd = lambda ps: max(1, int(round(ps * scale * 1000 / dt_fs)))
    hist_rd = [rd(h) for h in HIST_PS]; H = rd(HORIZON_PS); lags = [(ps, rd(ps)) for ps in LAGS_PS]
    feats, xi, xi_pk = dictionary(P, V, L, hist_rd)
    if synth is not None:   # the planted column is appended (dictionary column 19): STRUCT gains a seventh column
        feats = [np.column_stack([f, synth["planted"][:, i]]) for i, f in enumerate(feats)]
    Tk = meta.get("production_temperature_mean_k")
    say(f"## {label or d}: {n} molecules x {T} readouts at {dt_fs:g} fs = {(T - 1) * dt_fs / 1000:.2f} ps; box {L:.3f} A"
        + (f"; rigid-mode T {Tk:.1f} K" if Tk else "") + f"; time scale x{scale:g}")
    msd = [(l, ((P[l:] - P[:-l]) ** 2).sum(2).mean()) for l in range(rd(0.5), min(rd(2.5), T - 1) + 1)]
    if len(msd) >= 2:
        tt = np.array([l * dt_fs * 1e-15 for l, _ in msd]); mm = np.array([m for _, m in msd]) * 1e-20
        D = np.polyfit(tt, mm, 1)[0] / 6; say(f"   D from the O MSD over {0.5*scale:g}-{2.5*scale:g} ps: {D:.2e} m^2/s")
    mean_cols = np.nanmean(np.concatenate(feats), 0)
    say("   dictionary means: " + ", ".join(f"{NAMES[j]} {mean_cols[j]:.3f}" for j in list(range(8)) + [17, 18]) + (f", planted {mean_cols[19]:.3f}" if synth is not None else "") + f"; xi = {xi:.3f} A from |g-1| maxima at {xi_pk} A")
    st = STRUCT + ([19] if synth is not None else [])
    res = dict(label=label or d, lags={}, n=n, T=T)
    # lag-0 check
    A = np.concatenate([f[:, st] for f in feats]); e0 = np.abs(vamp(A, A)["s"] - 1).max(); res["lag0"] = e0
    say(f"   lag 0: max |sigma - 1| on STRUCT = {e0:.1e} (reported, < 1e-2)")
    folds = [[i for i in range(n) if i % 4 == j] for j in range(4)]
    tmax = max(hist_rd) if max(hist_rd) < T // 2 else 0
    sh = shuffled(feats)
    for ps, l in lags:
        if l >= T // 2:
            say(f"   lag {ps*scale:6.3g} ps ({l} rd): SKIPPED - longer than half the walk"); continue
        A, B = pairs(feats, l, st); m = vamp(A, B); s = m["s"]
        sm = vamp(*pairs(feats, l, MOM))["s"][0]
        c_sm = blocks(*pairs(feats, l, st + MOM), list(range(len(st))), list(range(len(st), len(st) + 3)))[0]
        c_smode = blocks(*pairs(feats, l, st + MODE), list(range(len(st))), list(range(len(st), len(st) + 6)))[0]
        hcols = [HIST[j] for j, h in enumerate(hist_rd) if h + l < T - 1 and h < T // 2]
        c_sh = blocks(*pairs(feats, l, st + hcols, t0=max([hist_rd[j] for j in range(4) if HIST[j] in hcols] or [0])), list(range(len(st))), list(range(len(st), len(st) + len(hcols))))[0] if hcols else float("nan")
        ho = []
        for te in folds:
            tr = [i for i in range(n) if i not in te]
            trA, trB = pairs([feats[i] for i in tr], l, st); teA, teB = pairs([feats[i] for i in te], l, st); ho.append(heldout(trA, trB, teA, teB, 1))
        s_sh = vamp(*pairs(sh, l, st))["s"][0]
        res["lags"][ps] = dict(l=l, s=s, s1=s[0], mom=sm, cross_mom=c_sm, cross_mode=c_smode, cross_hist=c_sh, ho1=float(np.mean(ho)), shuf=s_sh)
        say(f"   lag {ps*scale:6.3g} ps ({l:4d} rd): STRUCT sigma {np.array2string(s, precision=3)} | MOM own sigma1 {sm:.3f} | cross STRUCT-MOM {c_sm:.3f}, -MODE {c_smode:.3f}, -HIST {c_sh:.3f} | held-out f {np.mean(ho):.3f} | time-shuffled sigma1 {s_sh:.3f}")
    # tau*: the largest STRUCT sigma1 over the sector lags that fit
    cand = [ps for ps in SECTOR_LAGS_PS if ps in res["lags"]]
    if not cand: say("   no sector lag fits this walk"); return res
    tstar = max(cand, key=lambda ps: res["lags"][ps]["s1"]); l = res["lags"][tstar]["l"]; res["tstar"] = tstar
    Aall, Ball = pairs(feats, l, st); mm = vamp(Aall, Ball); w = mm["W0"] @ mm["U"][:, 0]
    fcol = [((f[:, st] - mm["mean"]) / mm["scale"]) @ w for f in feats]
    fall = np.concatenate(fcol); Sall = np.concatenate([f[:, st] for f in feats])
    corr = [float(np.corrcoef(fall, Sall[:, j])[0, 1]) for j in range(len(st))]
    load_ = {c: r2_fit(Sall[:, cols], fall) for c, cols in S3_CANDS.items()}
    if synth is not None: load_["planted"] = r2_fit(Sall[:, [6]], fall)
    wn = w / np.abs(w).sum()
    stnames = [NAMES[j] for j in STRUCT] + (["planted"] if synth is not None else [])
    res.update(loading=load_, weights=dict(zip(stnames, wn)), corr=dict(zip(stnames, corr)))
    say(f"   SECTOR at tau* = {tstar*scale:g} ps (sigma1 {res['lags'][tstar]['s1']:.3f}): weights (L1-normalised, standardised cols) " + ", ".join(f"{k} {v:+.2f}" for k, v in res["weights"].items()))
    say("      corr(f, col): " + ", ".join(f"{k} {v:+.2f}" for k, v in res["corr"].items()) + " | S3 loading (R^2 of f on the candidate): " + ", ".join(f"{k} {v:.3f}" for k, v in load_.items()))
    # S2: y = |r(t+H) - r(t)|, rows t in [t0, T-H); f fitted on the TRAIN molecules' own VAMP at tau*
    t0 = max([h for h in hist_rd if h < T - H - 10] or [0]); avail = {c: cols for c, cols in S2_CANDS.items() if all(not (4 <= j <= 7) or hist_rd[j - 4] <= t0 for j in cols)}
    if T - H - t0 < 10: say(f"   S2: the horizon {HORIZON_PS*scale:g} ps does not fit"); return res
    y = [np.linalg.norm(P[t0 + H:, i] - P[t0:T - H, i], axis=1) for i in range(n)]
    Z = [np.column_stack([f[t0:T - H, 8:11], (f[t0:T - H, 8:11] ** 2).sum(1)]) for f in feats]
    refound = [c for c, v in load_.items() if c in S3_CANDS and v >= 0.8]
    def run(perm):
        """perm maps target molecule i to its STRUCT donor (identity = the reading; a derangement = PS-2)"""
        r = {"Z": [], "Zf": []}; r.update({c: [] for c in avail}); r.update({c + "+f": [] for c in avail})
        for te in folds:
            tr = [i for i in range(n) if i not in te]
            A, B = pairs([feats[i] for i in tr], l, st); m_ = vamp(A, B); w_ = m_["W0"] @ m_["U"][:, 0]
            fx = lambda j: (((feats[j][t0:T - H][:, st] - m_["mean"]) / m_["scale"]) @ w_)[:, None]
            def X(idx, cols, withf):
                parts = [np.concatenate([Z[i] for i in idx])]
                if cols: parts.append(np.concatenate([feats[perm[i]][t0:T - H][:, cols] for i in idx]))
                if withf: parts.append(np.concatenate([fx(perm[i]) for i in idx]))
                return np.column_stack(parts)
            ytr = np.concatenate([y[i] for i in tr]); yte = np.concatenate([y[i] for i in te])
            r["Z"].append(ridge_r2(X(tr, [], False), ytr, X(te, [], False), yte)); r["Zf"].append(ridge_r2(X(tr, [], True), ytr, X(te, [], True), yte))
            for c, cols in avail.items():
                r[c].append(ridge_r2(X(tr, cols, False), ytr, X(te, cols, False), yte)); r[c + "+f"].append(ridge_r2(X(tr, cols, True), ytr, X(te, cols, True), yte))
        return {k: float(np.mean(v)) for k, v in r.items()}
    R = run(list(range(n)))
    inc = {c: R[c + "+f"] - R[c] for c in avail}; carry = R["Zf"] - R["Z"]
    mins = {c: v for c, v in inc.items() if c not in refound}; stat = min(mins.values()) if mins else float("nan")
    res.update(R=R, inc=inc, carry=carry, S2=stat, refound=refound, s2_argmin=min(mins, key=mins.get) if mins else None)
    say(f"   S2 target |r(t+{HORIZON_PS*scale:g} ps) - r(t)| over {T-H-t0} origins x {n}: R^2 velocity alone {R['Z']:.4f}; velocity+f {R['Zf']:.4f} (carry {carry:+.4f})")
    say("      per candidate c: R^2(Z+c) -> R^2(Z+c+f), increment: " + "; ".join(f"{c} {R[c]:.4f}->{R[c+'+f']:.4f} {inc[c]:+.4f}" for c in avail))
    say(f"      S2 statistic min_c increment = {stat:+.4f} (at {res['s2_argmin']}){'; excluded as re-found: ' + ', '.join(refound) if refound else ''}; missing candidates (history longer than the walk allows): {', '.join(c for c in S2_CANDS if c not in avail) or 'none'}")
    # PS-2: re-paired null, a fixed derangement (shift by n//2)
    Rn = run([(i + n // 2) % n for i in range(n)]); res["PS2"] = Rn["Zf"] - Rn["Z"]
    say(f"   PS-2 re-paired null (STRUCT from molecule i+n/2): increment beyond velocity {res['PS2']:+.4f} (bar <= 0.01) -> {'PASS' if res['PS2'] <= 0.01 else 'FAIL'}")
    # PS-1 and PS-4
    one = [ps for ps in res["lags"] if ps >= 1]
    if one:
        res["PS1"] = res["lags"][min(one)]["mom"]; res["PS4"] = max(res["lags"][ps]["shuf"] for ps in one)
        say(f"   PS-1 MOM own sigma1 at {min(one)*scale:g} ps = {res['PS1']:.3f} (bar < 0.2) -> {'PASS' if res['PS1'] < 0.2 else 'FAIL'}")
        say(f"   PS-4 time-shuffled STRUCT sigma1, max over lags >= {scale:g} ps = {res['PS4']:.3f} (bar < 0.2) -> {'PASS' if res['PS4'] < 0.2 else 'FAIL'}")
    # alpha_2
    a2 = alpha2(P, min(T // 2, rd(20)))
    pk = max(a2, key=lambda x: x[1]); res["alpha2_peak_ps"] = pk[0] * dt_fs / 1000; res["alpha2_peak"] = pk[1]
    edge = pk[0] == a2[-1][0]; res["alpha2_edge"] = edge
    ratio = res["alpha2_peak_ps"] / (tstar * scale)
    say(f"   alpha_2(t) max {pk[1]:.3f} at {pk[0]*dt_fs/1000:.3f} ps (searched to {len(a2)*dt_fs/1000:.2f} ps)" + (" - AT THE EDGE of the range, no peak inside it: the check cannot be read" if edge else f"; tau* = {tstar*scale:g} ps; ratio {ratio:.2f} -> {'consistent' if 0.5 <= ratio <= 2 else 'inconsistent'} (within 2x)"))
    return res

def verdict(r293, r400, scale):
    print(f"\n# VERDICT TABLE{'  -- NOT A READING (time scale x%g, a dry run of every code path)' % scale if scale != 1 else ''}")
    one = lambda r: [ps for ps in r["lags"] if ps >= 1]
    def s1(r):
        ok = [ps for ps in one(r) if r["lags"][ps]["s1"] >= 0.5 and r["lags"][ps]["cross_mom"] < 0.2]
        kill = all(r["lags"][ps]["s1"] < 0.2 for ps in one(r)); return ok, kill
    s1s = [s1(r) for r in r293]
    s1v = "MET" if all(o for o, _ in s1s) else ("KILL" if all(k for _, k in s1s) else "between")
    print(f"S1 closure: per 293 K seed, lags >= 1 ps with sigma1 >= 0.5 and cross-MOM < 0.2: {[ [p*scale for p in o] for o, _ in s1s]} -> {s1v}{' (NOT READ: PS-4 fails on ' + ', '.join(r['label'] for r in r293 if not r.get('PS4', 1) < 0.2) + ')' if any(not r.get('PS4', 1) < 0.2 for r in r293) else ''}")
    st = [r.get("S2", float("nan")) for r in r293]; s400 = r400.get("S2", float("nan")) if r400 else float("nan"); mean293 = float(np.mean(st))
    s2v = "MET" if all(x >= 0.05 for x in st) and s400 < mean293 else ("KILL" if mean293 < 0.02 or not (s400 < mean293) else "between")
    print(f"S2 carried: min_c increment at 293 K {[round(x, 4) for x in st]} (mean {mean293:+.4f}); 400 K {s400:+.4f}; carry beyond velocity alone 293 {[round(r.get('carry', float('nan')), 4) for r in r293]}, 400 {r400.get('carry', float('nan')) if r400 else float('nan'):+.4f} -> {s2v}")
    for r in r293 + ([r400] if r400 else []):
        L = r.get("loading", {}); mx = max(L.items(), key=lambda x: x[1]) if L else ("-", float("nan"))
        s3 = "RE-FINDING" if mx[1] >= 0.8 else ("DIRECTION" if mx[1] <= 0.6 else "partial")
        print(f"S3 {r['label']}: loading {', '.join(f'{k} {v:.3f}' for k, v in L.items())} -> {s3}; weights {', '.join(f'{k} {v:+.2f}' for k, v in r.get('weights', {}).items())}; alpha2 peak {r.get('alpha2_peak_ps', float('nan')):.3f} ps vs tau* {r.get('tstar', float('nan'))*scale:g} ps")
    arms = r293 + ([r400] if r400 else [])
    ps_fail = [f"{k} on {r['label']}" for r in arms for k, bad in (("PS-1", not r.get("PS1", 1) < 0.2), ("PS-2", not r.get("PS2", 1) <= 0.01), ("PS-4", not r.get("PS4", 1) < 0.2)) if bad]
    s3s = []
    for r in r293:
        L = {k: v for k, v in r.get("loading", {}).items() if k in S3_CANDS}; mx = max(L.values()) if L else float("nan")
        s3s.append("RE-FINDING" if mx >= 0.8 else ("DIRECTION" if mx <= 0.6 else "partial"))
    if ps_fail: br = "(e) a plant fails - nothing is read: " + "; ".join(ps_fail)
    elif s1v == "KILL": br = "(c) S1 killed"
    elif s2v == "KILL": br = "(d) S2 killed - closed but not carried"
    elif s1v == "MET" and s2v == "MET" and all(x == "DIRECTION" for x in s3s): br = "(a) DIRECTION"
    elif s1v == "MET" and s2v == "MET" and all(x == "RE-FINDING" for x in s3s): br = "(b) RE-FINDING"
    else: br = f"none of (a)-(e) cleanly: S1 {s1v}, S2 {s2v}, S3 per 293 K seed {s3s}"
    print(f"BRANCH: {br}" + ("  [NOT A READING]" if scale != 1 else ""))
    for r in r293 + ([r400] if r400 else []):
        print(f"plants {r['label']}: PS-1 {r.get('PS1', float('nan')):.3f} ({'PASS' if r.get('PS1', 1) < 0.2 else 'FAIL'}), PS-2 {r.get('PS2', float('nan')):+.4f} ({'PASS' if r.get('PS2', 1) <= 0.01 else 'FAIL'}), PS-4 {r.get('PS4', float('nan')):.3f} ({'PASS' if r.get('PS4', 1) < 0.2 else 'FAIL'}), lag-0 {r.get('lag0', float('nan')):.1e}")

# ---------------------------------------------------------------- PS-3: the planted slow variable
def synthetic(n=128, T=2501, dt_fs=20.0, tau_ps=5.0, D_A2ps=2.0, m_sd=0.5, noise_frac=0.25, seed=5):
    """n particles in the 128-water box (15.66 A), a random walk whose per-readout step is scaled
    by exp(m_i(t)); m_i an Ornstein-Uhlenbeck log-mobility with tau_ps memory and sd m_sd. The
    planted STRUCT column is m_i(t) + noise of noise_frac of its variance. Velocities are the
    step over the readout plus white noise (memoryless). D_A2ps sets the base diffusion so the
    synthetic's own neighbourhood counts turn over within about a picosecond (the plant must be
    the slow direction of ITS carrier; that is the sector it acts on)."""
    rng = np.random.default_rng(seed); L = 29.593631307523392 * BOHR_A; dt = dt_fs / 1000
    a = math.exp(-dt / tau_ps); m = rng.normal(size=n) * m_sd; X = rng.random((n, 3)) * L
    P = np.zeros((T, n, 3)); V = np.zeros((T, n, 3)); M = np.zeros((T, n)); step0 = math.sqrt(2 * D_A2ps * dt)
    for t in range(T):
        dx = rng.normal(size=(n, 3)) * step0 * np.exp(m)[:, None]
        X = X + dx; P[t] = X; V[t] = dx / dt + rng.normal(size=(n, 3)) * step0 / dt; M[t] = m
        m = a * m + math.sqrt(1 - a * a) * m_sd * rng.normal(size=n)
    planted = M + rng.normal(size=M.shape) * m_sd * math.sqrt(noise_frac)
    return dict(P=P, V=V, L=L, dt_fs=dt_fs, planted=planted)

def plants():
    print("# PS-3: a synthetic walk with a planted slow variable (5 ps log-mobility, noise 1/4 of its variance) - full time scale")
    sy = synthetic(); r = read_arm(None, 1.0, label="PS-3 synthetic, 128 x 2501 at 20 fs", synth=sy)
    lp = r.get("loading", {}).get("planted", float("nan"))
    print(f"PS-3: f at tau* {r.get('tstar')} ps loads {lp:.3f} on the planted column (bar >= 0.9) -> {'PASS' if lp >= 0.9 else 'FAIL'}; carry beyond velocity {r.get('carry', float('nan')):+.4f}; S2 statistic {r.get('S2', float('nan')):+.4f}")
    print(f"PS-4 on the synthetic: time-shuffled sigma1 max over lags >= 1 ps {r.get('PS4', float('nan')):.3f} (bar < 0.2) -> {'PASS' if r.get('PS4', 1) < 0.2 else 'FAIL'}")
    return r

if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("mode", choices=["plants", "read", "arms"])
    ap.add_argument("--t293", nargs="*", default=[]); ap.add_argument("--t400", nargs="*", default=[])
    ap.add_argument("--scale", type=float, default=1.0); ap.add_argument("--max-readouts", type=int, default=None)
    a = ap.parse_args()
    if a.mode == "plants": plants(); sys.exit(0)
    if a.scale != 1: print(f"# NOT A READING: every time of the prereg multiplied by {a.scale:g} to exercise the code paths on a short walk\n")
    r293 = [read_arm(d, a.scale, a.max_readouts, label=os.path.basename(d.rstrip('/')) + " (293 K arm)") for d in a.t293]
    r400 = [read_arm(d, a.scale, a.max_readouts, label=os.path.basename(d.rstrip('/')) + " (400 K arm)") for d in a.t400]
    verdict(r293, r400[0] if r400 else None, a.scale)
