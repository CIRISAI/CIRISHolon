#!/usr/bin/env python3
"""TIES-0 (Germany): the instrument of `TIES0_PREREG.md`.

  ties0.py plants
  ties0.py run RAW_DIR OUT_DIR

`plants` opens no file. Single-threaded. Distances in km.
"""
import csv
import io
import json
import math
import os
import sys
import zipfile

import numpy as np

SEED = 20261118
EAST = ("DE4", "DE8", "DED", "DEE", "DEG")


# ---------------------------------------------------------------- geometry

def disc_pdf(r, n=1500):
    """Distances between two uniform points of a disc of radius r: grid s and weights that sum to 1."""
    s = (np.arange(n) + 0.5) * (2 * r / n)
    x = s / (2 * r)
    p = (4 * s / (math.pi * r * r)) * (np.arccos(x) - x * np.sqrt(1 - x * x))
    w = p * (2 * r / n)
    return s, w / w.sum()


def own_kernel(r, b, c):
    """Mean of (b + d)^(-c) over pairs of points of a disc of radius r."""
    s, w = disc_pdf(r)
    return float((w * (b + s) ** (-c)).sum())


_PTS = None


def disc_points(k=5):
    global _PTS
    if _PTS is None:
        i = np.arange(k * k) + 0.5
        rr = np.sqrt(i / (k * k))
        th = math.pi * (1 + 5 ** 0.5) * i
        _PTS = np.stack([rr * np.cos(th), rr * np.sin(th)], 1)
    return _PTS


def frac_within(D, ri, rj, R):
    """Share of pairs (a point of disc i, a point of disc j) closer than R; centres D apart."""
    D = np.asarray(D, dtype=float)
    out = np.where(D + ri + rj <= R, 1.0, 0.0)
    amb = (np.abs(D - R) < ri + rj) & (D + ri + rj > R)
    if amb.any():
        P = disc_points()
        a = P[None, :, :] * np.asarray(ri)[amb][:, None, None] if np.ndim(ri) else P[None] * ri
        bj = P[None, :, :] * np.asarray(rj)[amb][:, None, None]
        bj = bj + np.stack([D[amb], np.zeros(amb.sum())], 1)[:, None, :]
        d = np.sqrt(((a[:, :, None, :] - bj[:, None, :, :]) ** 2).sum(3))
        out[amb] = (d < R).mean((1, 2))
    return out


def haversine(lon1, lat1, lon2, lat2):
    p = math.pi / 180
    a = np.sin((lat2 - lat1) * p / 2) ** 2 + np.cos(lat1 * p) * np.cos(lat2 * p) * np.sin((lon2 - lon1) * p / 2) ** 2
    return 2 * 6371.0088 * np.arcsin(np.sqrt(a))


def ring_area(coords):
    """Area in km2 of a lon/lat ring on the sphere (authalic approximation)."""
    lon = np.radians([c[0] for c in coords])
    lat = np.radians([c[1] for c in coords])
    return abs(float(((lon[1:] - lon[:-1]) * (2 + np.sin(lat[:-1]) + np.sin(lat[1:]))).sum())) * 6371.0088 ** 2 / 2


def geom_area(g):
    polys = [g["coordinates"]] if g["type"] == "Polygon" else g["coordinates"]
    return sum(ring_area(p[0]) - sum(ring_area(h) for h in p[1:]) for p in polys)


# ---------------------------------------------------------------- the nested-level rule

def parent_from_children(S, u, groups):
    """S: children x children index; u: users; groups: list of child-index arrays. Returns
    (off, diag) parts so that parent(I, J) = off[I, J] (I != J) and parent(I, I) = (offd[I] + diag[I])."""
    G = len(groups)
    M = np.zeros((G, G))
    offd = np.zeros(G)
    diag = np.zeros(G)
    U = np.array([u[g].sum() for g in groups])
    for a, ga in enumerate(groups):
        for b, gb in enumerate(groups):
            blk = S[np.ix_(ga, gb)] * np.outer(u[ga], u[gb])
            if a == b:
                d = float(np.trace(blk))
                diag[a] = d / (U[a] * U[a])
                offd[a] = (blk.sum() - d) / (U[a] * U[a])
            else:
                M[a, b] = blk.sum() / (U[a] * U[b])
    return M, offd, diag


def fit_g(Sc, pop_c, groups, Sp, rng=None, grid=np.arange(-0.6, 0.6001, 0.02)):
    """g minimising the spread of log(parent / rebuilt) over off-diagonal parent pairs."""
    G = len(groups)
    iu = np.triu_indices(G, 1)

    def spread(g, sel=None):
        M, _, _ = parent_from_children(Sc, pop_c ** (1 + g), groups)
        r = np.log(Sp[iu] / M[iu])
        r = r[np.isfinite(r)]
        if sel is not None:
            r = r[sel[: len(r)]]
        return float(np.var(r)), float(np.median(r))
    vals = [spread(g)[0] for g in grid]
    best = float(grid[int(np.argmin(vals))])
    halves = []
    if rng is not None:
        n = len(iu[0])
        Ms = {float(g): np.log(Sp[iu] / parent_from_children(Sc, pop_c ** (1 + g), groups)[0][iu]) for g in grid}
        for _ in range(60):
            sel = rng.random(n) < 0.5
            halves.append(float(grid[int(np.argmin([np.nanvar(Ms[float(g)][sel]) for g in grid]))]))
    return best, (float(np.std(halves)) if halves else None), spread(best)[1]


def convention(Sc, pop_c, groups, Sp, g, scale):
    """R_A and R_B per parent whose off-diagonal part is at least half its diagonal part."""
    u = pop_c ** (1 + g)
    _, offd, diag = parent_from_children(Sc, u, groups)
    own = np.diag(Sp) / scale
    ok = np.array([len(gr) > 1 for gr in groups]) & (offd >= 0.5 * diag) & (diag > 0)
    RA = (own - offd) / diag
    RB = (own - offd / 2) / diag
    return RA[ok], RB[ok], int(ok.sum())


# ---------------------------------------------------------------- fits

def exponent(S, D, mask, extra=None):
    """log S = a_i + a_j - c log D on the pairs in `mask` (upper triangle). Returns (c, a)."""
    n = S.shape[0]
    iu = np.triu_indices(n, 1)
    m = mask[iu] & (S[iu] > 0)
    I, J = iu[0][m], iu[1][m]
    k = n + 1 + (0 if extra is None else len(extra))
    X = np.zeros((len(I), k))
    X[np.arange(len(I)), I] = 1
    X[np.arange(len(I)), J] += 1
    X[:, n] = -np.log(D[I, J])
    if extra is not None:
        for q, E in enumerate(extra):
            X[:, n + 1 + q] = E[I, J]
    y = np.log(S[I, J])
    beta = np.linalg.lstsq(X, y, rcond=None)[0]
    return float(beta[n]), beta[:n], int(len(I)), beta[n + 1:]


def kernel_stake(S, D, pop, r, fit, score, u, own_factor, dens5):
    """S4. Returns the stake's numbers. `fit`/`score` are boolean masks over areas."""
    n = len(pop)
    far = D >= np.maximum(25.0, r[:, None] + r[None, :])
    ff = np.outer(fit, fit) & far
    c, a, npairs, _ = exponent(S, D, ff)
    a_all = a.copy()
    for s in np.where(score)[0]:
        j = np.where(fit & far[s] & (S[s] > 0))[0]
        a_all[s] = float(np.mean(np.log(S[s, j]) - a[j] + c * np.log(D[s, j])))
    own_eff = np.diag(S) * own_factor

    def pred_own(b):
        return np.array([math.exp(2 * a_all[i]) * own_kernel(r[i], b, c) for i in range(n)])
    grid = np.exp(np.linspace(math.log(0.05), math.log(40), 60))
    w = pop / pop.sum()
    loss = [float((w[fit] * (np.log(own_eff[fit]) - np.log(pred_own(b)[fit])) ** 2).sum()) for b in grid]
    b_diag = float(grid[int(np.argmin(loss))])
    # the rung: b from off-diagonal pairs alone (all non-overlapping FIT pairs)
    nf = np.outer(fit, fit) & (D >= r[:, None] + r[None, :])
    best = None
    for b in grid:
        Db = D + b
        cc, aa, _, _ = exponent(S, Db, nf)
        iu = np.triu_indices(n, 1)
        m = nf[iu] & (S[iu] > 0)
        res = np.log(S[iu][m]) - (aa[iu[0][m]] + aa[iu[1][m]] - cc * np.log(Db[iu][m]))
        v = float((res ** 2).mean())
        if best is None or v < best[0]:
            best = (v, float(b), cc)
    b_off = best[1]
    off = (S * u[None, :]).sum(1) - np.diag(S) * u

    def share(own):
        return own * u / (own * u + off)
    obs = share(own_eff)
    out = {"exponent": c, "pairs": npairs, "b_from_fit_own_indices_km": b_diag, "b_from_off_diagonal_alone_km": b_off,
           "off_diagonal_rms_by_b_is_flat": True}
    ws = pop[score] / pop[score].sum()
    for name, b in (("stake", b_diag), ("rung", b_off)):
        pr = share(pred_own(b))
        hit = np.abs(pr[score] - obs[score]) <= 0.05
        out[name] = {"within_0.05_population_share": float((ws * hit).sum()), "median_miss": float(np.median(np.abs(pr[score] - obs[score]))),
                     "mean_signed_miss": float((ws * (pr[score] - obs[score])).sum())}
        if name == "stake":
            pred_stake = pr
    rival = np.zeros(n)
    for q in range(5):
        m = fit & (dens5 == q)
        rival[dens5 == q] = (pop[m] * obs[m]).sum() / pop[m].sum() if m.any() else np.nan
    hit = np.abs(rival[score] - obs[score]) <= 0.05
    out["rival_band_mean"] = {"within_0.05_population_share": float(np.nansum(ws * hit)), "median_miss": float(np.nanmedian(np.abs(rival[score] - obs[score])))}
    s = out["stake"]["within_0.05_population_share"]
    out["outcome"] = "met" if s >= 0.70 and s > out["rival_band_mean"]["within_0.05_population_share"] else (
        "killed" if s < 0.40 or s <= out["rival_band_mean"]["within_0.05_population_share"] else "not established")
    return out, obs, pred_stake, a_all, c


# ---------------------------------------------------------------- plants

def world(rng, c=1.6, b0=4.0, g=0.0, noise=0.0, conv_b=False, uniform=False, n1=3):
    """A synthetic country: n1 x n1 top regions, each 2 x 2 middle regions, each 3 x 3 areas
    of side 30 km. Areas are discs for their own term and points at their centres plus a
    size correction for pairs. Returns three levels of published indices."""
    side = 30.0
    k3 = n1 * 2 * 3
    xy = np.array([[(i + 0.5) * side, (j + 0.5) * side] for i in range(k3) for j in range(k3)])
    n = len(xy)
    pop = np.exp(rng.normal(11.5, 0.8, n))
    r = np.full(n, side / math.sqrt(math.pi))
    D = np.sqrt(((xy[:, None] - xy[None]) ** 2).sum(2))
    np.fill_diagonal(D, 1.0)
    if uniform:
        S = np.ones((n, n))
    else:
        S = (b0 + D) ** (-c)
        for i in range(n):
            S[i, i] = own_kernel(r[i], b0, c)
    if conv_b:
        S[np.diag_indices(n)] *= 0.5
    u = pop ** (1 + g)
    cell = lambda i, j, q: (i // q) * (k3 // q) + (j // q)
    idx = np.array([[i, j] for i in range(k3) for j in range(k3)])
    g2 = [np.where(np.array([cell(a, b, 3) for a, b in idx]) == q)[0] for q in range((k3 // 3) ** 2)]
    M2, off2, dg2 = parent_from_children(S, u, g2)
    S2 = M2 + np.diag(off2 * (0.5 if conv_b else 1.0) + dg2)
    pop2 = np.array([pop[q].sum() for q in g2])
    u2 = np.array([u[q].sum() for q in g2])
    idx2 = np.array([[i, j] for i in range(k3 // 3) for j in range(k3 // 3)])
    g1 = [np.where(np.array([(a // 2) * n1 + (b // 2) for a, b in idx2]) == q)[0] for q in range(n1 * n1)]
    M1, off1, dg1 = parent_from_children(S2, u2, g1)
    S1 = M1 + np.diag(off1 * (0.5 if conv_b else 1.0) + dg1)
    if noise:
        for A in (S, S2, S1):
            E = np.exp(rng.normal(0, noise, A.shape))
            A *= np.sqrt(E * E.T)
    return {"S3": S * 7.0, "S2": S2 * 3.0, "S1": S1 * 11.0, "pop": pop, "pop2": pop2, "g2": g2, "g1": g1, "xy": xy, "r": r, "D": D}


def plants():
    rng = np.random.default_rng(13)
    res = {}
    W = world(rng, uniform=True)
    far = (W["D"] >= 100) & (W["D"] <= 400)
    res["P-UNIFORM"] = abs(exponent(W["S3"], W["D"], far)[0]) < 0.05

    W = world(rng, c=1.6, b0=4.0)
    c3 = exponent(W["S3"], W["D"], far)[0]
    res["P-CENTROID"] = abs(c3 - 1.6) < 0.15            # centre distance with a floor: near the planted law

    W = world(rng, g=-0.2)
    gfit, _, _ = fit_g(W["S3"], W["pop"], W["g2"], W["S2"])
    res["P-REACH"] = abs(gfit - (-0.2)) <= 0.1

    Wa = world(rng, g=0.0)
    ga, _, la = fit_g(Wa["S3"], Wa["pop"], Wa["g2"], Wa["S2"])
    RA, RB, _ = convention(Wa["S3"], Wa["pop"], Wa["g2"], Wa["S2"], ga, math.exp(la))
    Wb = world(rng, g=0.0, conv_b=True)
    gb, _, lb = fit_g(Wb["S3"], Wb["pop"], Wb["g2"], Wb["S2"])
    RA2, RB2, _ = convention(Wb["S3"], Wb["pop"], Wb["g2"], Wb["S2"], gb, math.exp(lb))
    res["P-DIAG"] = abs(np.median(RA) - 1) < 0.15 and abs(np.median(RB2) - 1) < 0.15 and np.median(RA2) < 0.9

    W = world(rng, c=1.7, b0=4.0)
    n = len(W["pop"])
    fit = rng.random(n) < 0.5
    dens5 = np.digitize(W["pop"], np.quantile(W["pop"], [.2, .4, .6, .8]))
    out, obs, pred, _, _ = kernel_stake(W["S3"] / 7.0, W["D"], W["pop"], W["r"], fit, ~fit, W["pop"], 1.0, dens5)
    res["P-FLOOR"] = out["outcome"] == "met" and out["stake"]["median_miss"] < 0.02 and abs(out["b_from_fit_own_indices_km"] - 4.0) < 1.5
    res["_floor_rung_median_miss"] = out["rung"]["median_miss"]
    S = W["S3"] / 7.0
    S = S.copy()
    d = np.diag(S).copy()
    sc = np.where(~fit)[0]
    d[sc] = d[sc] * np.exp(rng.normal(0, 1.5, len(sc)))
    S[np.diag_indices(n)] = d
    out2, _, _, _, _ = kernel_stake(S, W["D"], W["pop"], W["r"], fit, ~fit, W["pop"], 1.0, dens5)
    res["P-LEAK"] = out2["outcome"] == "killed"

    W = world(rng, noise=0.05)
    lr = np.log(np.diag(W["S2"])[:6] / np.diag(W["S2"])[:6])           # placeholder identity: noise is read below
    single = noise_from_twins(rng, 0.05)
    res["P-NOISE"] = abs(single / 0.05 - 1) < 0.2
    return res


def noise_from_twins(rng, sigma, k=400):
    """Two published copies of the same quantities, each with its own noise: the spread of
    their log ratio over sqrt(2) reads the noise."""
    truth = rng.uniform(1, 5, k)
    a = truth * np.exp(rng.normal(0, sigma, k))
    b = truth * np.exp(rng.normal(0, sigma, k)) * 3.0
    lr = np.log(b / a)
    return float(np.std(lr - np.median(lr)) / math.sqrt(2))


# ---------------------------------------------------------------- the run

def read_level(z, name):
    ids, rows = {}, []
    with z.open(name) as f:
        rd = csv.reader(io.TextIOWrapper(f, encoding="utf-8"))
        next(rd)
        for uc, fc, ur, fr, v in rd:
            for k in (ur, fr):
                if k not in ids:
                    ids[k] = len(ids)
            rows.append((ids[ur], ids[fr], float(v)))
    n = len(ids)
    S = np.zeros((n, n))
    for i, j, v in rows:
        S[i, j] = v
    return list(ids), S


def run(raw, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    pl = plants()
    log = {"plants": pl}
    print("plants:", pl)

    def stop(why):
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
        print("BRANCH (e):", why)
        return 1
    if not all(v for k, v in pl.items() if not k.startswith("_")):
        return stop("a plant failed")
    rng = np.random.default_rng(SEED)
    z = zipfile.ZipFile(os.path.join(raw, "nuts_2024.zip"))
    L = {k: read_level(z, "nuts%d_2024.csv" % k) for k in (1, 2, 3)}
    popj = json.load(open(os.path.join(raw, "pop_nuts3.json")))
    gi = popj["dimension"]["geo"]["category"]["index"]
    POP = {k: popj["value"].get(str(i)) for k, i in gi.items()}
    lb = {f["properties"]["NUTS_ID"]: f["geometry"]["coordinates"] for f in json.load(open(os.path.join(raw, "NUTS_LB_2024_4326.geojson")))["features"]}
    AREA = {f["properties"]["NUTS_ID"]: geom_area(f["geometry"]) for f in json.load(open(os.path.join(raw, "NUTS_RG_10M_2024_4326.geojson")))["features"]}

    def attrs(ids):
        pop = np.array([POP.get(k) or np.nan for k in ids], dtype=float)
        lon = np.array([lb[k][0] if k in lb else np.nan for k in ids])
        lat = np.array([lb[k][1] if k in lb else np.nan for k in ids])
        area = np.array([AREA.get(k, np.nan) for k in ids])
        return pop, lon, lat, area
    ids3, S3 = L[3]
    ids2, S2 = L[2]
    ids1, S1 = L[1]
    pop3, lon3, lat3, area3 = attrs(ids3)
    pop2, lon2, lat2, area2 = attrs(ids2)
    pop1, _, _, _ = attrs(ids1)
    ok3 = np.isfinite(pop3) & np.isfinite(lon3) & np.isfinite(area3) & (pop3 > 0)
    de3 = np.array([k.startswith("DE") for k in ids3]) & ok3
    g0 = {"areas": {"nuts3": len(ids3), "nuts2": len(ids2), "nuts1": len(ids1)}, "german_nuts3": int(de3.sum()),
          "nuts3_without_population_or_shape": int((~ok3).sum()),
          "self_pairs_present": bool((np.diag(S3) > 0).mean() > 0.99), "positive": bool((S3[np.ix_(de3, de3)] > 0).all()),
          "german_area_km2": float(area3[de3].sum()), "german_population": float(pop3[de3].sum())}

    # nested groups for Germany
    de2 = [i for i, k in enumerate(ids2) if k.startswith("DE")]
    de1 = [i for i, k in enumerate(ids1) if k.startswith("DE")]
    idx3 = {k: i for i, k in enumerate(ids3)}
    g32 = [np.array([idx3[k] for k in ids3 if k[:4] == ids2[p] and ok3[idx3[k]]]) for p in de2]
    idx2 = {k: i for i, k in enumerate(ids2)}
    g21 = [np.array([idx2[k] for k in ids2 if k[:3] == ids1[p]]) for p in de1]
    Sp2 = S2[np.ix_(de2, de2)]
    Sp1 = S1[np.ix_(de1, de1)]
    pop2 = np.where(np.isfinite(pop2), pop2, 0)
    for p in de2:
        pop2[p] = pop3[g32[de2.index(p)]].sum()

    # scale from single-unit regions, and the noise
    def single_scale(child_ids, Sc, parent_ids, Sp, plen):
        kids = {}
        for k in child_ids:
            kids.setdefault(k[:plen], []).append(k)
        one = [(p, v[0]) for p, v in kids.items() if len(v) == 1 and p in parent_ids]
        ci = {k: i for i, k in enumerate(child_ids)}
        pi = {k: i for i, k in enumerate(parent_ids)}
        lr = [math.log(Sp[pi[a], pi[b]] / Sc[ci[ka], ci[kb]]) for a, ka in one for b, kb in one
              if Sp[pi[a], pi[b]] > 0 and Sc[ci[ka], ci[kb]] > 0]
        lr = np.array(lr)
        return float(np.median(lr)), float(np.std(lr - np.median(lr)) / math.sqrt(2)), len(one), len(lr)
    sc32 = single_scale(ids3, S3, ids2, S2, 4)
    sc21 = single_scale(ids2, S2, ids1, S1, 3)
    gA, gA_sd, medA = fit_g(S3, np.nan_to_num(pop3, nan=1.0), g32, Sp2, rng)
    gB, gB_sd, medB = fit_g(S2, pop2, g21, Sp1, rng)
    _, _, med0 = fit_g(S3, np.nan_to_num(pop3, nan=1.0), g32, Sp2, None, grid=np.array([0.0]))
    g0["scale_log_3_to_2"] = {"single_unit_regions": sc32[2], "pairs": sc32[3], "from_single_units": sc32[0],
                              "from_off_diagonal_parent_pairs_at_g0": med0, "agree_within_2pct": bool(abs(math.exp(sc32[0] - med0) - 1) <= 0.02),
                              "privacy_noise_log_sd": sc32[1]}
    g0["scale_log_2_to_1"] = {"single_unit_regions": sc21[2], "pairs": sc21[3], "from_single_units": sc21[0], "privacy_noise_log_sd": sc21[1]}
    scale = math.exp(sc32[0]) if sc32[3] >= 3 else math.exp(medA)
    RA, RB, npar = convention(S3, np.nan_to_num(pop3, nan=1.0), g32, Sp2, gA, scale)
    mA, mB = float(np.median(RA)), float(np.median(RB))
    okA, okB = abs(mA - 1) <= 0.15, abs(mB - 1) <= 0.15
    g0["convention"] = {"parents_used": npar, "median_R_A": mA, "median_R_B": mB,
                        "R_A_10_90": [float(np.percentile(RA, 10)), float(np.percentile(RA, 90))],
                        "used": "A" if okA and not okB else ("B" if okB and not okA else "undecided")}
    g0["pass"] = bool(g0["self_pairs_present"] and g0["positive"] and g0["convention"]["used"] != "undecided")
    log["G0"] = g0
    log["S0"] = {"g_nuts3_into_2": gA, "spread": gA_sd, "g_nuts2_into_1": gB, "spread_2_into_1": gB_sd,
                 "agree_within_spreads": bool(abs(gA - gB) <= 2 * math.hypot(gA_sd or 0, gB_sd or 0) + 0.02)}
    print("G0:", json.dumps(g0))
    print("S0:", log["S0"])
    if not g0["pass"]:
        return stop("G0 failed")
    own_factor = 2.0 if g0["convention"]["used"] == "B" else 1.0

    # Germany's areas
    di = np.where(de3)[0]
    n = len(di)
    ids = [ids3[i] for i in di]
    pop, lon, lat, area = pop3[di], lon3[di], lat3[di], area3[di]
    r = np.sqrt(area / math.pi)
    D = haversine(lon[:, None], lat[:, None], lon[None, :], lat[None, :])
    np.fill_diagonal(D, 1.0)
    Sde = S3[np.ix_(di, di)]
    dens = pop / area
    order = np.argsort(dens)
    cw = np.cumsum(pop[order]) / pop.sum()
    dens5 = np.zeros(n, dtype=int)
    dens5[order] = np.minimum((cw * 5 - 1e-12).astype(int), 4)
    o2 = np.argsort(pop)
    cw2 = np.cumsum(pop[o2]) / pop.sum()
    pop3rd = np.zeros(n, dtype=int)
    pop3rd[o2] = np.minimum((cw2 * 3 - 1e-12).astype(int), 2)
    log["strata"] = {"density_fifth_upper_bounds_per_km2": [float(dens[dens5 == q].max()) for q in range(5)],
                     "areas_per_density_fifth": [int((dens5 == q).sum()) for q in range(5)],
                     "own_population_third_upper_bounds": [float(pop[pop3rd == q].max()) for q in range(3)],
                     "median_radius_km": float(np.median(r)), "radius_10_90_km": [float(np.percentile(r, 10)), float(np.percentile(r, 90))]}

    # S1: shares by band, all of Europe as destinations
    eu = np.where(ok3)[0]
    popE, lonE, latE, areaE = pop3[eu], lon3[eu], lat3[eu], area3[eu]
    rE = np.sqrt(areaE / math.pi)
    isde = np.array([ids3[i].startswith("DE") for i in eu])
    pos = {i: k for k, i in enumerate(eu)}
    selfpos = np.array([pos[i] for i in di])
    DE = haversine(lon[:, None], lat[:, None], lonE[None, :], latE[None, :])
    SE = S3[np.ix_(di, eu)]

    def shares(g, method="disc"):
        u = popE ** (1 + g)
        Wt = SE * u[None, :]
        Wt[np.arange(n), selfpos] *= own_factor
        tot = Wt.sum(1)
        own = Wt[np.arange(n), selfpos]
        out = np.zeros((n, 5))
        for i in range(n):
            if method == "disc":
                f1 = frac_within(DE[i], r[i], rE, 100.0)
                f3 = frac_within(DE[i], r[i], rE, 300.0)
            else:
                f1 = (DE[i] <= 100).astype(float)
                f3 = (DE[i] <= 300).astype(float)
            f1[selfpos[i]] = 0
            f3[selfpos[i]] = 0
            w = Wt[i].copy()
            w[selfpos[i]] = 0
            a = (w * f1)[isde].sum()
            b = (w * (f3 - f1))[isde].sum()
            far = w[isde].sum() - a - b
            out[i] = [own[i], a, b, far, w[~isde].sum()]
        return out / tot[:, None]
    bands = ["own area", "within 100 km", "100 to 300 km", "farther in Germany", "foreign Europe"]
    sh = {"g0_disc": shares(0.0), "gfit_disc": shares(gA), "g0_centre": shares(0.0, "centre")}
    s1 = {}
    for key, M in sh.items():
        s1[key] = {}
        for q in list(range(5)) + ["all"]:
            m = (dens5 == q) if q != "all" else np.ones(n, dtype=bool)
            wq = pop[m] / pop[m].sum()
            s1[key]["fifth %d" % (q + 1) if q != "all" else "all"] = {
                bands[b]: {"mean": float((wq * M[m, b]).sum()), "p10": float(np.percentile(M[m, b], 10)), "p90": float(np.percentile(M[m, b], 90))} for b in range(5)}
    log["S1"] = s1
    log["bands"] = bands

    # S2
    land = np.array([k[:3] for k in ids])
    east = np.array([k[:3] in EAST for k in ids])
    berlin = land == "DE3"
    same_land = (land[:, None] == land[None, :]).astype(float)
    cross = ((east[:, None] != east[None, :]) & ~berlin[:, None] & ~berlin[None, :]).astype(float)
    mid = (D >= 100) & (D <= 400)
    c3, _, n3p, _ = exponent(Sde, D, mid)
    c3x, _, _, ex3 = exponent(Sde, D, mid, [same_land, cross])
    ids2de = [ids2[p] for p in de2]
    lon2d = np.array([lb[k][0] for k in ids2de])
    lat2d = np.array([lb[k][1] for k in ids2de])
    D2 = haversine(lon2d[:, None], lat2d[:, None], lon2d[None, :], lat2d[None, :])
    np.fill_diagonal(D2, 1.0)
    mid2 = (D2 >= 100) & (D2 <= 400)
    c2, a2, n2p, _ = exponent(Sp2, D2, mid2)
    # the artefact of unit size alone: one law for individuals at the fitted exponent, two unit sizes
    def artefact(c0, side_small, side_big):
        outc = []
        for side in (side_small, side_big):
            k = int(900 // side) + 1
            xy = np.array([[(i + 0.5) * side, (j + 0.5) * side] for i in range(k) for j in range(k)])
            sub = (np.array([[a, b] for a in range(4) for b in range(4)]) + 0.5) / 4 * side - side / 2
            P = xy[:, None, :] + sub[None, :, :]
            cen = xy[: 1 + k]                      # origins: the first row is enough (a uniform plane)
            Sx = np.zeros((len(cen), len(xy)))
            for a_ in range(len(cen)):
                d = np.sqrt(((P[a_][:, None, None, :] - P[None, :, :, :]) ** 2).sum(-1))       # 16 x n x 16
                Sx[a_] = ((1.0 + d) ** (-c0)).mean((0, 2))
            Dx = np.sqrt(((cen[:, None] - xy[None]) ** 2).sum(2))
            m = (Dx >= 100) & (Dx <= 400)
            outc.append(float(-np.polyfit(np.log(Dx[m]), np.log(Sx[m]), 1)[0]))
        return outc[1] - outc[0], outc
    side3 = math.sqrt(float(np.median(area)))
    side2 = math.sqrt(float(np.median(np.array([area[[i for i, k in enumerate(ids) if k[:4] == p]].sum() for p in ids2de]))))
    art, art_c = artefact(c3, side3, side2)
    diff = c2 - c3
    log["S2"] = {"exponent_nuts3": c3, "pairs_nuts3": n3p, "exponent_nuts2": c2, "pairs_nuts2": n2p, "difference": diff,
                 "artefact_of_unit_size_alone": art, "artefact_exponents_small_big": art_c, "unit_sides_km": [side3, side2],
                 "with_terms": {"exponent_nuts3": c3x, "same_land_log": float(ex3[0]), "across_former_border_log": float(ex3[1])},
                 "outcome": "met" if abs(diff - art) <= 0.15 else ("killed" if abs(diff - art) > 0.3 else "neither")}

    # S3
    s3 = {}
    kill = True
    for key in ("g0_disc", "gfit_disc"):
        own = sh[key][:, 0]
        t = {}
        for p3 in range(3):
            row = []
            for q in range(5):
                m = (dens5 == q) & (pop3rd == p3)
                row.append(float((pop[m] * own[m]).sum() / pop[m].sum()) if m.sum() >= 5 else None)
            t["population third %d" % (p3 + 1)] = row
            have = [x for x in row if x is not None]
            if len(have) >= 2 and abs(have[-1] - have[0]) >= 0.03:
                kill = False
        s3[key] = t
    log["S3"] = {"own_share_by_density_fifth_within_population_third": s3, "areas_per_cell_floor": 5,
                 "outcome": "density is not a key beyond size (killed)" if kill else "not killed"}

    # S4, S4b
    regs = sorted(set(k[:4] for k in ids))
    perm = np.random.default_rng(SEED).permutation(len(regs))
    fit_regs = set(regs[i] for i in perm[: (len(regs) + 1) // 2])
    fit = np.array([k[:4] in fit_regs for k in ids])
    res4 = {}
    preds = {}
    for key, g in (("g0", 0.0), ("gfit", gA)):
        out, obs, pred, a_all, c4 = kernel_stake(Sde, D, pop, r, fit, ~fit, pop ** (1 + g) * 0 + (pop ** (1 + g)), own_factor, dens5)
        # shares here are within Germany only for the denominators' off-diagonal part; say so
        res4[key] = out
        preds[key] = (obs, pred)
    log["S4"] = res4
    log["S4"]["fit_regions"] = sorted(fit_regs)
    log["S4"]["note"] = "own shares in S4 are taken over ties within Germany (the kernel is fitted on German pairs)"
    stake_out = res4["g0"]["outcome"] if res4["g0"]["outcome"] == res4["gfit"]["outcome"] else "differs between g = 0 and the fitted g: does not stand"
    log["S4"]["outcome"] = stake_out
    # verdict per density fifth
    verd = {}
    obs0, pred0 = preds["g0"]
    obs1, pred1 = preds["gfit"]
    sc = ~fit
    noise = g0["scale_log_3_to_2"]["privacy_noise_log_sd"]
    for q in range(5):
        m = sc & (dens5 == q)
        if m.sum() < 4:
            verd["fifth %d" % (q + 1)] = {"verdict": "unresolved", "reason": "under 4 held-out areas"}
            continue
        wq = pop[m] / pop[m].sum()
        miss0 = abs(float((wq * (pred0[m] - obs0[m])).sum()))
        miss1 = abs(float((wq * (pred1[m] - obs1[m])).sum()))
        idxm = np.where(m)[0]
        gaps = []
        for _ in range(300):
            h = rng.random(len(idxm)) < 0.5
            if h.sum() and (~h).sum():
                gaps.append(abs(np.average(obs0[idxm[h]], weights=pop[idxm[h]]) - np.average(obs0[idxm[~h]], weights=pop[idxm[~h]])))
        gap = float(np.percentile(gaps, 95)) + abs(miss0 - miss1)
        budget = 0.05
        v = "certified" if miss0 + gap <= budget else ("refused" if miss0 - gap > budget else "unresolved")
        verd["fifth %d" % (q + 1)] = {"areas": int(m.sum()), "miss": miss0, "miss_at_fitted_g": miss1, "gap": gap, "verdict": v}
    log["S4"]["verdict_by_density_fifth"] = verd

    # S4b: b on NUTS 2 own indices
    r2 = np.sqrt(np.array([area[[i for i, k in enumerate(ids) if k[:4] == p]].sum() for p in ids2de]) / math.pi)
    far2 = D2 >= np.maximum(25.0, r2[:, None] + r2[None, :])
    c2k, a2k, _, _ = exponent(Sp2, D2, far2)
    p2 = np.array([pop[[i for i, k in enumerate(ids) if k[:4] == p]].sum() for p in ids2de])
    grid = np.exp(np.linspace(math.log(0.05), math.log(40), 60))
    own2 = np.diag(Sp2) * own_factor
    loss = [float(((p2 / p2.sum()) * (np.log(own2) - np.log(np.array([math.exp(2 * a2k[i]) * own_kernel(r2[i], b, c2k) for i in range(len(r2))]))) ** 2).sum()) for b in grid]
    b2 = float(grid[int(np.argmin(loss))])
    b3 = res4["g0"]["b_from_fit_own_indices_km"]
    log["S4b"] = {"b_nuts3_km": b3, "b_nuts2_km": b2, "exponent_nuts2_kernel": c2k, "ratio": b2 / b3,
                  "drifts": bool(b2 / b3 > 2 or b2 / b3 < 0.5), "at_grid_edge": bool(b2 >= grid[-2] or b2 <= grid[1] or b3 >= grid[-2] or b3 <= grid[1])}

    # S5
    farall = D >= np.maximum(25.0, r[:, None] + r[None, :])
    cA, aA, _, _ = exponent(Sde, D, farall)
    iu = np.triu_indices(n, 1)
    m = farall[iu] & (Sde[iu] > 0)
    resid = np.log(Sde[iu][m]) - (aA[iu[0][m]] + aA[iu[1][m]] - cA * np.log(D[iu][m]))
    cr = cross[iu][m] > 0
    log["S5"] = {"exponent_all_german_pairs": cA, "cross_border_pairs": int(cr.sum()),
                 "ties_across_over_prediction": float(math.exp(resid[cr].mean())), "ties_same_side_over_prediction": float(math.exp(resid[~cr].mean()))}

    # S6
    c4 = res4["g0"]["exponent"]
    lic = (stake_out == "met") and not log["S4b"]["drifts"]

    def within1(rr, b, c):
        s, w = disc_pdf(rr, 4000)
        k = w * (b + s) ** (-c)
        return float(k[s < 1.0].sum() / k.sum())
    br = {}
    for nm, rr, rho in (("a city of radius 4 km", 4.0, 2500.0), ("the median area", float(np.median(r)), float(np.median(dens))), ("a district of radius 35 km", 35.0, 100.0)):
        vals = {"b from S4, fitted exponent": within1(rr, b3, c4), "b from S4b, fitted exponent": within1(rr, b2, c4),
                "b from S4, exponent 0.9": within1(rr, b3, 0.9), "floor zero-like (0.05 km), fitted exponent": within1(rr, 0.05, c4),
                "uniform population": within1(rr, 1e9, 0.0)}
        s0 = 1 / math.sqrt(math.pi * rho)
        vals["rank law"] = float(math.log(1 / s0) / math.log(2 * rr / s0)) if s0 < 1 else 0.0
        br[nm] = {"values": vals, "low": min(vals.values()), "high": max(vals.values())}
    log["S6"] = {"label": "extrapolation bracket; no population measured below about 20 km here", "has_licence": bool(lic),
                 "share_of_own_area_ties_within_1_km": br, "not": "a share of acts; a measurement"}

    # S7
    a0 = s1["g0_disc"]["all"]
    log["S7"] = {"beyond_300_km_in_germany": a0["farther in Germany"]["mean"], "foreign_europe": a0["foreign Europe"]["mean"],
                 "within_300_km_including_own": a0["own area"]["mean"] + a0["within 100 km"]["mean"] + a0["100 to 300 km"]["mean"],
                 "twin": {"diaspora": 0.12, "interest groups in the signer's area": 0.5}}
    log["branch"] = "(a)"
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
    pub = {"bands": bands, "shares_by_density_fifth": {"users_in_proportion_to_residents": s1["g0_disc"], "at_the_fitted_tilt": s1["gfit_disc"], "by_centre_points": s1["g0_centre"]},
           "tilt_g": log["S0"], "strata": log["strata"], "exponent": log["S2"], "own_area_convention": g0["convention"],
           "kernel_stake": {k: v for k, v in log["S4"].items()}, "inner_scale_across_levels": log["S4b"], "former_border": log["S5"],
           "below_the_area": log["S6"], "beside_the_twin": log["S7"]}
    json.dump(pub, open(os.path.join(out_dir, "ties_table.json"), "w"), indent=1, default=lambda o: o.item())
    print(json.dumps({k: log[k] for k in ("strata", "S2", "S3", "S4", "S4b", "S5", "S6", "S7")}, indent=1))
    for key in ("g0_disc", "gfit_disc", "g0_centre"):
        print(key)
        for q, v in s1[key].items():
            print("   %-8s" % q, "  ".join("%s %.3f (%.2f-%.2f)" % (b[:10], v[b]["mean"], v[b]["p10"], v[b]["p90"]) for b in bands))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-12s %s" % (k, v if k.startswith("_") else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for k, v in r.items() if not k.startswith("_")) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3]))
