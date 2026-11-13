#!/usr/bin/env python3
"""BUILDGEN-1: a US dwelling from the joint (`BUILDGEN1_PREREG.md`).

  buildgen1.py plants BINARY
  buildgen1.py gate   RAW.csv BINARY OUT_DIR     # reads FIT rows alone; writes gate.json and the tables
  buildgen1.py score  RAW.csv BINARY OUT_DIR     # runs only on the gate's written pass

BINARY is `buildgen_draw`. Needs numpy. Latent correlations are fitted by bisection THROUGH the
shipped function (the binary), not a mirror of it.
"""
import csv
import hashlib
import json
import os
import subprocess
import sys

import numpy as np

DIGEST = "8d5f6d3120d2cbc2535151c7d455e7a536ca9768f773e15edd45932ec38b3592"
STATE_SEED, SEEDS, FLOOR, CELL_FLOOR = 20261111, 20, 100, 400
SPLIT = {"states": 51, "fit_records": 9948, "score_records": 8548}
ZONES = ["Cold", "Hot-Dry", "Hot-Humid", "Marine", "Mixed-Dry", "Mixed-Humid", "Subarctic", "Very-Cold"]
NEIGH = {"Subarctic": "Very-Cold", "Very-Cold": "Cold", "Cold": "Mixed-Humid", "Marine": "Mixed-Humid", "Mixed-Dry": "Hot-Dry", "Hot-Dry": "Mixed-Dry",
         "Hot-Humid": "Mixed-Humid", "Mixed-Humid": "Cold"}
URBAN = {"U": 0, "C": 1, "R": 2}
FUEL = {5: 0, 1: 1, 2: 2, 3: 3, 7: 4, 99: 5, -2: 6}
WALL = {1: 0, 2: 1, 3: 2, 4: 3, 5: 4, 6: 5, 7: 6, 99: 7}
ROOF = {1: 0, 2: 1, 3: 2, 4: 3, 5: 4, 6: 5, 99: 6, -2: 255}
NA = 255
T = dict(TYPE=1, VINTAGE=2, FUEL=3, WALL=4, ROOF=5, SPLIT=6, STOREYS=7, AREA=8, KWH=9, NONELEC=10, ZERO=11, CHOL=12, ENDUSE=13)
PQ = 0.01 + 0.98 * np.arange(33) / 32
COLS = ["TYPEHUQ", "YEARMADERANGE", "STORIES", "WALLTYPE", "ROOFTYPE", "FUELHEAT", "TOTSQFT_EN", "KWH", "BTUEL", "TOTALBTU", "TOTALBTUSPH", "BTUELCOL", "BA_climate", "UATYP10"]


def key(table, a=0, b=0, c=0, d=0):
    assert a < 8 and b < 16 and c < 8 and d < 8
    return (((table * 8 + a) * 16 + b) * 8 + c) * 8 + d


def era_of(v):
    return np.where(v <= 3, 0, np.where(v <= 5, 1, 2))


def fg_of(f):
    return np.where(f == 0, 0, np.where(f == 1, 1, np.where(f == 6, 3, 2)))


def is_house(k):
    return (k == 1) | (k == 2)


# ---------------------------------------------------------------- reading the survey

def split_states(path):
    with open(path, newline="") as f:
        rd = csv.reader(f)
        h = next(rd)
        si = h.index("STATE_FIPS")
        state = np.array([int(r[si]) for r in rd])
    codes = sorted(set(state.tolist()))
    perm = np.random.default_rng(STATE_SEED).permutation(len(codes))
    fit_states = set(codes[i] for i in perm[:26])
    fit = np.array([s in fit_states for s in state])
    return fit, {"states": len(codes), "fit_records": int(fit.sum()), "score_records": int((~fit).sum())}


def load(path, rows):
    """Reads the columns of the rows whose mask is true, and no other row's."""
    with open(path, newline="") as f:
        rd = csv.reader(f)
        h = next(rd)
        ix = {c: i for i, c in enumerate(h)}
        wi = [ix["NWEIGHT"]] + [ix["NWEIGHT%d" % i] for i in range(1, 61)]
        raw = {c: [] for c in COLS}
        W = []
        for n, r in enumerate(rd):
            if not rows[n]:
                continue
            for c in COLS:
                raw[c].append(r[ix[c]])
            W.append([float(r[i]) for i in wi])
    return records(raw), np.array(W)


def records(raw):
    num = lambda c: np.array([float(v) for v in raw[c]])
    stories = num("STORIES").astype(int)
    kind = num("TYPEHUQ").astype(int) - 1
    F = {"kind": kind, "vint": num("YEARMADERANGE").astype(int) - 1, "zone": np.array([ZONES.index(z) for z in raw["BA_climate"]]),
         "urban": np.array([URBAN[u] for u in raw["UATYP10"]]), "fuel": np.array([FUEL[int(float(v))] for v in raw["FUELHEAT"]]),
         "wall": np.array([WALL[int(float(v))] for v in raw["WALLTYPE"]]), "roof": np.array([ROOF[int(float(v))] for v in raw["ROOFTYPE"]]),
         "area": num("TOTSQFT_EN"), "kwh": num("KWH"), "total": num("TOTALBTU")}
    F["split"] = np.where(is_house(kind), (stories == 5).astype(int), NA)
    F["storeys"] = np.where(is_house(kind), np.where(stories == 5, 2, np.clip(stories, 1, 3)), 0)   # a split-level is level 2, with its flag
    F["nonelec"] = np.maximum(F["total"] - num("BTUEL"), 0.0)
    F["nonelec"][F["nonelec"] < 1.0] = 0.0
    F["heat"], F["cool"] = num("TOTALBTUSPH"), num("BTUELCOL")
    return finish(F)


def finish(F):
    F["era"], F["fg"] = era_of(F["vint"]), fg_of(F["fuel"])
    return F


# ---------------------------------------------------------------- statistics

def wquant(x, w, p):
    if len(x) == 0:                               # nothing to read: a table of zeros, never an error
        return np.zeros(np.shape(p))
    o = np.argsort(x, kind="stable")
    x, w = x[o], w[o]
    c = (np.cumsum(w) - 0.5 * w) / w.sum()
    return np.interp(p, c, x)


def midrank(x, w):
    o = np.argsort(x, kind="stable")
    xs, ws = x[o], w[o]
    cum = np.cumsum(ws)
    first = np.r_[True, xs[1:] != xs[:-1]]
    gid = np.cumsum(first) - 1
    gsum = np.bincount(gid, ws)
    gstart = np.r_[0.0, np.cumsum(gsum)[:-1]]
    r = np.empty(len(x))
    r[o] = gstart[gid] + 0.5 * gsum[gid]
    return r


def wspearman(x, y, w):
    if len(x) < 3:
        return 0.0
    rx, ry = midrank(x, w), midrank(y, w)
    mx, my = np.average(rx, weights=w), np.average(ry, weights=w)
    cov = np.average((rx - mx) * (ry - my), weights=w)
    den = np.sqrt(np.average((rx - mx) ** 2, weights=w) * np.average((ry - my) ** 2, weights=w))
    return float(cov / den) if den > 0 else 0.0


def jk_se(stat, W):
    full = np.atleast_1d(np.asarray(stat(W[:, 0]), dtype=float))
    R = W.shape[1] - 1
    reps = np.stack([np.atleast_1d(np.asarray(stat(W[:, r]), dtype=float)) for r in range(1, R + 1)])
    return full, np.sqrt((R - 1) / R * ((reps - full) ** 2).sum(0))


def joint(a, b, w, na, nb):
    m = np.zeros((na, nb))
    np.add.at(m, (a, b), w)
    return m / m.sum()


def tvd(p, q):
    return 0.5 * float(np.abs(p - q).sum())


# ---------------------------------------------------------------- fitting

def ladder(F, fit, fixed, zone=None, urban=None, era=None, use_era=True):
    """The first mask with FLOOR records: full; without urban; without era; zone with its neighbour; all zones."""
    base = fit.copy()
    for k, v in fixed.items():
        base &= F[k] == v
    steps = []
    zm = (F["zone"] == zone) if zone is not None else np.ones(len(fit), bool)
    um = (F["urban"] == urban) if urban is not None else np.ones(len(fit), bool)
    em = (F["era"] == era) if (era is not None and use_era) else np.ones(len(fit), bool)
    steps.append(base & zm & um & em)
    if urban is not None:
        steps.append(base & zm & em)
    if era is not None and use_era:
        steps.append(base & zm)
    if zone is not None:
        z2 = zm | (F["zone"] == ZONES.index(NEIGH[ZONES[zone]]))
        steps.append(base & z2)
        steps.append(base)
    for lvl, m in enumerate(steps):
        if m.sum() >= FLOOR:
            return lvl, m
    for k in list(fixed)[::-1]:                 # still thin at all zones: let go of the last fixed attribute
        base2 = fit.copy()
        for kk, v in fixed.items():
            if kk != k:
                base2 &= F[kk] == v
        if base2.sum() >= FLOOR:
            return 8, base2
    return 9, fit


def ppm(F, w, m, name, n):
    c = np.bincount(F[name][m], weights=w[m], minlength=n)[:n]
    if c.sum() <= 0:
        c = np.ones(n)
    v = np.round(c / c.sum() * 1e6).astype(int)
    return v.tolist()


def fit_tables(F, w, fit, era_chain=("fuel", "wall", "roof"), chol=None):
    """All table rows but T_CHOL (identity unless `chol` is given: {kind: lower-triangular Q16 list})."""
    rows, levels = {}, {}

    def put(k, lvl, vals, tag):
        rows[k] = [int(lvl)] + [int(v) for v in vals]
        levels.setdefault(tag, []).append(int(lvl))
    for z in range(8):
        for u in range(3):
            lvl, m = ladder(F, fit, {}, zone=z, urban=u)
            put(key(T["TYPE"], 0, z, u), lvl, ppm(F, w, m, "kind", 5), "type")
        for k in range(5):
            lvl, m = ladder(F, fit, {"kind": k}, zone=z)
            put(key(T["VINTAGE"], k, z), lvl, ppm(F, w, m, "vint", 9), "vintage")
            put(key(T["AREA"], k, z), lvl, np.round(wquant(F["area"][m], w[m], PQ)), "area")
            lvl, m = ladder(F, fit & is_house(F["kind"]), {"kind": k} if k in (1, 2) else {}, zone=z)
            sh = np.bincount(F["storeys"][m] - 1, weights=w[m], minlength=3)[:3]
            put(key(T["STOREYS"], k, z), lvl, np.round(sh / sh.sum() * 1e6), "storeys")
            for e in range(3):
                lvl, m = ladder(F, fit, {"kind": k}, zone=z, era=e, use_era="fuel" in era_chain)
                put(key(T["FUEL"], k, z, e), lvl, ppm(F, w, m, "fuel", 7), "fuel")
                lvl, m = ladder(F, fit, {"kind": k}, zone=z, era=e, use_era="wall" in era_chain)
                put(key(T["WALL"], k, z, e), lvl, ppm(F, w, m, "wall", 8), "wall")
            for g in range(4):
                lvl, m = ladder(F, fit, {"kind": k, "fg": g}, zone=z)
                put(key(T["KWH"], k, z, g), lvl, np.round(wquant(F["kwh"][m], w[m], PQ)), "electricity")
                tot, heat, cool = (float((w[m] * F[c][m]).sum()) for c in ("total", "heat", "cool"))
                hs = min(1.0, heat / tot) if tot > 0 else 0.0
                put(key(T["ENDUSE"], k, z, g), lvl, [round(hs * 1e6), round(min(1.0 - hs, cool / tot if tot > 0 else 0.0) * 1e6)], "enduse")
                zero = float(w[m][F["nonelec"][m] == 0].sum())
                put(key(T["ZERO"], k, z, g), lvl, [round(zero / w[m].sum() * 1e6), round((1 - zero / w[m].sum()) * 1e6)], "zero")
                lvl, mp = ladder(F, fit & (F["nonelec"] > 0), {"kind": k, "fg": g}, zone=z)
                put(key(T["NONELEC"], k, z, g), lvl, np.round(wquant(F["nonelec"][mp], w[mp], PQ)), "nonelectric")
        roofed = fit & (F["roof"] != NA)
        for wl in range(8):
            for e in range(3):
                lvl, m = ladder(F, roofed, {"wall": wl}, zone=z, era=e, use_era="roof" in era_chain)
                put(key(T["ROOF"], wl, z, e), lvl, ppm(F, w, m, "roof", 7), "roof")
    for k in range(5):
        for e in range(3):
            lvl, m = ladder(F, fit & is_house(F["kind"]), {"kind": k} if k in (1, 2) else {}, era=e)
            sh = np.bincount(np.where(F["split"][m] == 1, 1, 0), weights=w[m], minlength=2)
            put(key(T["SPLIT"], k, 0, e), lvl, np.round(sh / sh.sum() * 1e6), "split")
        d = 5 if k in (1, 2) else 4
        L = chol[k] if chol else [65536 if i == j else 0 for i in range(d) for j in range(i + 1)]
        rows[key(T["CHOL"], k)] = [0] + [int(v) for v in L]
    return rows, {k: {str(l): v.count(l) for l in sorted(set(v))} for k, v in levels.items()}


def write_tables(rows, path):
    with open(path, "w") as f:
        for k in sorted(rows):
            f.write("%d %s\n" % (k, " ".join(str(v) for v in rows[k])))


def draw(binary, table_path, seeds, zone, urban):
    inp = "\n".join("%d %d %d" % t for t in zip(seeds.tolist(), zone.tolist(), urban.tolist())) + "\n"
    out = subprocess.run([binary, table_path], input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
    n = len(seeds)
    A = np.array([l.split() for l in out[:n]], dtype=np.int64)
    G = {"kind": A[:, 0], "vint": A[:, 1], "fuel": A[:, 2], "wall": A[:, 3], "roof": A[:, 4], "storeys": A[:, 5], "split": A[:, 6], "area": A[:, 7].astype(float),
         "kwh": A[:, 8].astype(float), "nonelec": A[:, 9].astype(float), "total": A[:, 10].astype(float), "heat": A[:, 11].astype(float), "cool": A[:, 12].astype(float),
         "zone": zone.copy(), "urban": urban.copy()}
    return finish(G), out[n].split()[1]


def chol_q16(C):
    vals, vecs = np.linalg.eigh(C)
    repaired = bool(vals.min() < 0.01)
    if repaired:
        C = (vecs * np.clip(vals, 0.01, None)) @ vecs.T
        d = np.sqrt(np.diag(C))
        C = C / np.outer(d, d)
    L = np.linalg.cholesky(C)
    return [int(round(L[i, j] * 65536)) for i in range(len(C)) for j in range(i + 1)], repaired


VARS_H, VARS_O = ["vint", "storeys", "area", "kwh", "nonelec"], ["vint", "area", "kwh", "nonelec"]


def pair_rho(R, a, b, w):
    """Spearman of a pair; pairs with non-electric energy are read where it is positive."""
    m = np.ones(len(w), bool)
    if "nonelec" in (a, b):
        m = R["nonelec"] > 0
    return wspearman(R[a][m], R[b][m], w[m])


def fit_copula(F, w, fit, base_rows, binary, tmp, rng, n_draw=60000, sweeps=2, iters=9):
    """For each type, latent correlations so that the GENERATED Spearman equals FIT's; through the binary."""
    chol, info = {}, {}
    for k in range(5):
        names = VARS_H if k in (1, 2) else VARS_O
        d = len(names)
        mk = fit & (F["kind"] == k)
        if mk.sum() < FLOOR:                      # a type with too few records keeps the identity matrix, and says so
            chol[k] = [65536 if i == j else 0 for i in range(d) for j in range(i + 1)]
            info[k] = {"records": int(mk.sum()), "repaired": False, "latent": {}, "fit_spearman": {}, "generated_spearman": {}, "worst_miss": 0.0, "identity_for_want_of_records": True}
            continue
        Fk = {c: F[c][mk] for c in names}
        target = {(i, j): pair_rho(Fk, names[i], names[j], w[mk]) for i in range(d) for j in range(i)}
        idx = rng.choice(np.flatnonzero(mk), size=n_draw, p=w[mk] / w[mk].sum())
        zone, urban = F["zone"][idx], F["urban"][idx]
        seeds = rng.integers(1, 2 ** 62, size=n_draw)
        forced = dict(base_rows)
        for z in range(8):
            for u in range(3):
                forced[key(T["TYPE"], 0, z, u)] = [0] + [1 if t == k else 0 for t in range(5)]
        C = np.eye(d)
        for (i, j), t in target.items():
            C[i, j] = C[j, i] = np.clip(t, -0.9, 0.9)       # a start: the target itself

        def gen_rho(Cm, pair):
            forced[key(T["CHOL"], k)] = [0] + chol_q16(Cm)[0]
            p = os.path.join(tmp, "fitcop.txt")
            write_tables(forced, p)
            G, _ = draw(binary, p, seeds, zone, urban)
            return pair_rho(G, names[pair[0]], names[pair[1]], np.ones(n_draw))
        for _ in range(sweeps):
            for pair, t in target.items():
                lo, hi = -0.97, 0.97
                for _ in range(iters):
                    mid = 0.5 * (lo + hi)
                    C[pair[0], pair[1]] = C[pair[1], pair[0]] = mid
                    if gen_rho(C, pair) < t:
                        lo = mid
                    else:
                        hi = mid
                C[pair[0], pair[1]] = C[pair[1], pair[0]] = 0.5 * (lo + hi)
        L, repaired = chol_q16(C)
        chol[k] = L
        forced[key(T["CHOL"], k)] = [0] + L
        p = os.path.join(tmp, "fitcop.txt")
        write_tables(forced, p)
        G, _ = draw(binary, p, seeds, zone, urban)
        got = {(i, j): pair_rho(G, names[i], names[j], np.ones(n_draw)) for (i, j) in target}
        info[k] = {"records": int(mk.sum()), "repaired": repaired, "latent": {"%s~%s" % (names[i], names[j]): round(float(C[i, j]), 3) for (i, j) in target},
                   "fit_spearman": {"%s~%s" % (names[i], names[j]): round(t, 3) for (i, j), t in target.items()},
                   "generated_spearman": {"%s~%s" % (names[i], names[j]): round(g, 3) for (i, j), g in got.items()},
                   "worst_miss": round(max(abs(got[p_] - target[p_]) for p_ in target), 4)}
    return chol, info


def build(F, w, fit, binary, tmp, rng, era_chain=("fuel", "wall", "roof"), identity=False, chol=None):
    rows, levels = fit_tables(F, w, fit, era_chain=era_chain)
    info = None
    if not identity:
        if chol is None:
            chol, info = fit_copula(F, w, fit, rows, binary, tmp, rng)
        for k in range(5):
            rows[key(T["CHOL"], k)] = [0] + chol[k]
    return rows, levels, chol, info


def generate_for(F, mask, binary, table_path, rng_seed, seeds_per=SEEDS):
    idx = np.repeat(np.flatnonzero(mask), seeds_per)
    seeds = np.random.default_rng(rng_seed).integers(1, 2 ** 62, size=len(idx))
    G, dg = draw(binary, table_path, seeds, F["zone"][idx], F["urban"][idx])
    return G, idx, dg


# ---------------------------------------------------------------- the stakes

D1_VARS = ["vint", "storeys", "area", "kwh", "total"]
PAIRS = [(a, b) for i, a in enumerate(D1_VARS) for b in D1_VARS[:i]]


def d1_table(S, W, G, gw, mask_s, mask_g, pairs):
    out = []
    for a, b in pairs:
        ms = mask_s & (is_house(S["kind"]) if "storeys" in (a, b) else True)
        mg = mask_g & (is_house(G["kind"]) if "storeys" in (a, b) else True)
        ref, se = jk_se(lambda ww: wspearman(S[a][ms], S[b][ms], ww[ms]), W)
        gen = wspearman(G[a][mg], G[b][mg], gw[mg])
        out.append({"pair": "%s~%s" % (b, a), "survey": round(float(ref[0]), 4), "gap": round(float(2 * se[0]), 4), "generated": round(gen, 4), "records": int(ms.sum()),
                    "within": bool(abs(gen - ref[0]) <= 2 * se[0] + 0.05)})
    return out


def verdict(gen, ref, se, budget=0.15):
    miss, gap = abs(gen - ref) / ref, 2 * se / ref
    return "certified" if miss + gap <= budget else ("refused" if miss - gap > budget else "unresolved")


def d2(S, W, G, gw, cuts):
    cells = []
    det_s, det_g = S["kind"] == 1, G["kind"] == 1
    ter = lambda R: np.digitize(R["area"], cuts)
    groups = [("area tercile", ter, 3), ("era", lambda R: R["era"], 3), ("fuel group", lambda R: R["fg"], 3)]
    for name, f, n in groups:
        for c in range(n):
            ms, mg = det_s & (f(S) == c), det_g & (f(G) == c)
            for meas in ("total", "kwh"):
                if ms.sum() < CELL_FLOOR:
                    cells.append({"by": name, "cell": c, "measure": meas, "records": int(ms.sum()), "verdict": "not scored"})
                    continue
                ref, se = jk_se(lambda ww: np.average(S[meas][ms], weights=ww[ms]), W)
                gen = float(np.average(G[meas][mg], weights=gw[mg])) if mg.sum() else 0.0
                cells.append({"by": name, "cell": c, "measure": meas, "records": int(ms.sum()), "survey": round(float(ref[0]), 1), "gap_pct": round(float(200 * se[0] / ref[0]), 2),
                              "generated": round(gen, 1), "miss_pct": round(float(100 * abs(gen - ref[0]) / ref[0]), 2), "verdict": verdict(gen, ref[0], se[0])})
    inten = lambda R, ww, m: (ww[m] * R["total"][m]).sum() / (ww[m] * R["area"][m]).sum()
    ratio = lambda R, ww, dm: inten(R, ww, dm & (R["era"] == 2)) / inten(R, ww, dm & (R["era"] == 0))
    ref, se = jk_se(lambda ww: ratio(S, ww, det_s), W)
    gen = float(ratio(G, gw, det_g))
    scored = [c for c in cells if c["verdict"] != "not scored"]
    refused, cert = sum(c["verdict"] == "refused" for c in scored), sum(c["verdict"] == "certified" for c in scored)
    dd = {"survey": round(float(ref[0]), 4), "gap": round(float(2 * se[0]), 4), "generated": round(gen, 4), "difference": round(gen - float(ref[0]), 4)}
    in5, in10 = abs(gen - ref[0]) <= 2 * se[0] + 0.05, abs(gen - ref[0]) <= 2 * se[0] + 0.10
    met = refused == 0 and scored and cert >= 0.8 * len(scored) and in5
    kill = refused >= 3 or not in10
    return {"cells": cells, "scored": len(scored), "certified": cert, "refused": refused, "energy_per_area_newest_over_oldest": dd,
            "reading": "met" if met else ("kill" if kill else "neither")}


D3_TABLES = [("wall by roof", "wall", "roof", 8, 7, lambda R: R["kind"] != 4), ("fuel by zone", "fuel", "zone", 7, 8, None), ("fuel by era", "fuel", "era", 7, 3, None),
             ("type by urban type", "kind", "urban", 5, 3, None)]


def d3(S, W, G, gw, FITREC, fw, rng, tables=D3_TABLES):
    out = []
    for name, a, b, na, nb, sel in tables:
        ms = sel(S) if sel else np.ones(len(W), bool)
        mg = sel(G) if sel else np.ones(len(gw), bool)
        q = joint(G[a][mg], G[b][mg], gw[mg], na, nb)
        ref, se = jk_se(lambda ww: tvd(joint(S[a][ms], S[b][ms], ww[ms], na, nb), q), W)
        mf = sel(FITREC) if sel else np.ones(len(fw), bool)
        pf = joint(FITREC[a][mf], FITREC[b][mf], fw[mf], na, nb)
        idxf = np.flatnonzero(mf)
        noise = []
        for _ in range(40):
            bs = rng.choice(idxf, size=int(ms.sum()))
            noise.append(tvd(joint(FITREC[a][bs], FITREC[b][bs], fw[bs], na, nb), pf))
        out.append({"table": name, "distance": round(float(ref[0]), 4), "gap": round(float(2 * se[0]), 4), "perfect_generator_at_this_size": round(float(np.mean(noise)), 4),
                    "within_0.05": bool(ref[0] <= 0.05 + 2 * se[0]), "over_0.10": bool(ref[0] > 0.10 + 2 * se[0])})
    over = sum(t["over_0.10"] for t in out)
    return {"tables": out, "reading": "met" if all(t["within_0.05"] for t in out) else ("kill" if over >= 2 else "neither")}


def s5(S, W, G, gw):
    def share(R, ww, m):
        a, t = R["area"][m], R["total"][m]
        return float(ww[m][(a >= wquant(a, ww[m], 0.95)) & (t >= wquant(t, ww[m], 0.95))].sum() / ww[m].sum())
    ms, mg = S["kind"] == 1, G["kind"] == 1
    ref, se = jk_se(lambda ww: share(S, ww, ms), W)
    gen = share(G, gw, mg)
    ratio = float(ref[0]) / gen if gen > 0 else float("inf")
    outside = abs(gen - ref[0]) > 2 * se[0]
    return {"survey": round(float(ref[0]), 4), "gap": round(float(2 * se[0]), 4), "generated": round(gen, 4), "survey_over_generated": round(ratio, 3),
            "reading": "kill: tail understated" if (ratio > 1.5 and outside) else ("tail overstated" if (ratio < 1 / 1.5 and outside) else "met")}


def d1_all(S, W, G, gw):
    alls, allg = np.ones(len(W), bool), np.ones(len(gw), bool)
    a = d1_table(S, W, G, gw, alls, allg, PAIRS)
    det = d1_table(S, W, G, gw, S["kind"] == 1, G["kind"] == 1, PAIRS)
    apt = d1_table(S, W, G, gw, S["kind"] == 4, G["kind"] == 4, [p for p in PAIRS if "storeys" not in p])
    na, nb = sum(r["within"] for r in a), sum(r["within"] for r in det) + sum(r["within"] for r in apt)
    reading = "met" if (na >= 9 and nb >= 14) else ("kill" if (na <= 6 or nb <= 10) else "neither")
    return {"D1a_pooled": a, "D1b_detached": det, "D1b_apartments_5_or_more": apt, "D1a_within": na, "D1b_within": nb, "reading": reading}


# ---------------------------------------------------------------- synthetic surveys, for the plants

def synth(rng, n=24000, rho_area_kwh=0.0, rho_vint_storeys=0.0, wall_by_era=False, fuel_effect=False, tail=False, only_detached=False, all_electric=False):
    kind = np.full(n, 1) if only_detached else rng.choice(5, size=n, p=[0.06, 0.62, 0.06, 0.08, 0.18])
    zone = rng.choice(8, size=n, p=[0.3, 0.1, 0.2, 0.06, 0.04, 0.25, 0.01, 0.04])
    urban = rng.choice(3, size=n, p=[0.7, 0.1, 0.2])
    C = np.eye(4)
    C[0, 1] = C[1, 0] = rho_vint_storeys
    C[2, 3] = C[3, 2] = rho_area_kwh
    z = rng.multivariate_normal(np.zeros(4), C, size=n)
    from statistics import NormalDist
    cdf = np.vectorize(NormalDist().cdf)
    u = cdf(z)
    if tail:                                     # the top of area and of electricity coincide more than a normal copula allows
        top = rng.random(n) < 0.06
        u[top, 2] = 0.94 + 0.06 * rng.random(top.sum())
        u[top, 3] = np.clip(u[top, 2] + 0.01 * (rng.random(top.sum()) - 0.5), 0, 0.99999)
    vint = np.minimum((u[:, 0] * 9).astype(int), 8)
    storeys = np.where(is_house(kind), 1 + np.minimum((u[:, 1] * 3).astype(int), 2), 0)
    area = 600 + 3000 * u[:, 2] ** 1.5
    fuel = np.zeros(n, dtype=int) if all_electric else rng.choice(7, size=n, p=[0.4, 0.45, 0.05, 0.04, 0.02, 0.02, 0.02])
    kwh = 3000 + 14000 * u[:, 3] ** 1.3
    nonelec = np.where(fuel == 0, 0.0, 20000 + 60000 * rng.random(n))
    if fuel_effect:
        kwh = np.where(fuel == 0, 2 * kwh, kwh)
    e = era_of(vint)
    if wall_by_era:
        wall = np.where(e == 0, rng.choice(8, size=n, p=[0.6, 0.2, 0.05, 0.05, 0.03, 0.03, 0.02, 0.02]),
                        np.where(e == 1, rng.choice(8, size=n, p=[0.2, 0.2, 0.4, 0.1, 0.03, 0.03, 0.02, 0.02]), rng.choice(8, size=n, p=[0.05, 0.05, 0.7, 0.1, 0.03, 0.03, 0.02, 0.02])))
    else:
        wall = rng.choice(8, size=n, p=[0.25, 0.15, 0.35, 0.1, 0.05, 0.04, 0.03, 0.03])
    roof = np.where(kind == 4, NA, rng.choice(7, size=n, p=[0.05, 0.03, 0.1, 0.02, 0.7, 0.05, 0.05]))
    F = {"kind": kind, "vint": vint, "zone": zone, "urban": urban, "fuel": fuel, "wall": wall, "roof": roof, "area": area, "kwh": kwh, "nonelec": nonelec,
         "total": 3.412 * kwh + nonelec, "split": np.where(is_house(kind), (rng.random(n) < 0.05).astype(int), NA), "storeys": storeys}
    F["heat"], F["cool"] = 0.4 * F["total"], np.minimum(0.1 * F["total"], 3.412 * kwh)
    return finish(F), np.ones(n)


def plants(binary, tmp="/tmp"):
    res = {}
    tmp = os.path.join(tmp, "buildgen1_plants_%d" % os.getpid())
    os.makedirs(tmp, exist_ok=True)
    rng = np.random.default_rng(7)
    allm = lambda F: np.ones(len(F["kind"]), bool)

    def fitgen(F, w, seeds_per=3, **kw):
        rows, levels, chol, info = build(F, w, allm(F), binary, tmp, np.random.default_rng(11), **kw)
        p = os.path.join(tmp, "t.txt")
        write_tables(rows, p)
        G, idx, dg = generate_for(F, allm(F), binary, p, 5, seeds_per=seeds_per)
        return G, np.ones(len(idx)), info, levels, dg, p
    # P-INDEP
    F, w = synth(rng, n=120000)                  # large, so that the survey's own sampling noise is under the bar
    G, gw, info, levels, dg, p = fitgen(F, w)
    lat = [abs(v) for v in info[1]["latent"].values()]          # detached houses, about 74,000 records
    Gr, grw, _, _, _, _ = fitgen(F, w, identity=True)
    d = G["kind"] == 1
    dr = Gr["kind"] == 1
    res["_P-INDEP_largest_latent"] = round(max(lat), 4)
    res["P-INDEP"] = bool(max(lat) <= 0.03 and abs(wspearman(G["area"][d], G["kwh"][d], gw[d]) - wspearman(Gr["area"][dr], Gr["kwh"][dr], grw[dr])) <= 0.03)
    # P-NA and P-LAW on 100,000 calls
    seeds = np.random.default_rng(3).integers(1, 2 ** 62, size=100000)
    H, dg1 = draw(binary, p, seeds, seeds % 8, seeds % 3)
    _, dg2 = draw(binary, p, seeds, seeds % 8, seeds % 3)
    res["P-NA"] = bool(np.all((H["storeys"] != 0) == is_house(H["kind"])) and np.all((H["roof"] == NA) == (H["kind"] == 4)) and np.all((H["split"] != NA) == is_house(H["kind"])))
    res["P-LAW"] = bool(np.all(H["total"] * 1000 + 500 >= H["kwh"] * 3412) and np.all(H["heat"] + H["cool"] <= H["total"]) and np.all(H["cool"] * 1000 <= H["kwh"] * 3412 + 500))
    res["P-DET_native_twice"] = dg1 == dg2
    # P-MARG: population marginals within 0.005 plus 3 standard errors of the simulation
    worst = 0.0
    n = len(gw)
    for name, k in (("kind", 5), ("vint", 9), ("fuel", 7), ("wall", 8)):
        a, b = np.bincount(F[name], minlength=k) / len(w), np.bincount(G[name], minlength=k) / n
        worst = max(worst, float(np.max(np.abs(a - b) - 3 * np.sqrt(a * (1 - a) / n))))
    for name in ("area", "kwh"):
        cuts = np.quantile(F[name], [0.2, 0.4, 0.6, 0.8])
        a, b = np.bincount(np.digitize(F[name], cuts), minlength=5) / len(w), np.bincount(np.digitize(G[name], cuts), minlength=5) / n
        worst = max(worst, float(np.max(np.abs(a - b) - 3 * np.sqrt(a * (1 - a) / n))))
    res["P-MARG"] = bool(worst <= 0.005)
    res["_P-MARG_worst_excess"] = round(worst, 4)
    # P-DEP
    F, w = synth(rng, n=120000, rho_area_kwh=0.6, rho_vint_storeys=0.5, wall_by_era=True)
    G, gw, info, _, _, _ = fitgen(F, w)
    Gr, grw, _, _, _, _ = fitgen(F, w, identity=True, era_chain=())
    d, dg_, dr = F["kind"] == 1, G["kind"] == 1, Gr["kind"] == 1
    t1, g1, r1 = (wspearman(R["area"][m], R["kwh"][m], ww[m]) for R, m, ww in ((F, d, w), (G, dg_, gw), (Gr, dr, grw)))
    t2, g2, r2 = (wspearman(R["vint"][m], R["storeys"][m], ww[m]) for R, m, ww in ((F, d, w), (G, dg_, gw), (Gr, dr, grw)))
    je = lambda R, ww: joint(R["wall"], R["era"], ww, 8, 3)
    t3, r3 = tvd(je(F, w), je(G, gw)), tvd(je(F, w), je(Gr, grw))
    res["P-DEP"] = bool(abs(g1 - t1) <= 0.03 and abs(g2 - t2) <= 0.03 and t3 <= 0.03 and abs(r1 - t1) > 0.2 and abs(r2 - t2) > 0.2 and r3 > 0.1)
    res["_P-DEP"] = [round(v, 3) for v in (t1, g1, r1, t2, g2, r2, t3, r3)]
    # P-FUEL
    F, w = synth(rng, fuel_effect=True)
    G, gw, _, _, _, _ = fitgen(F, w)
    ok = True
    for g in range(3):
        a, b = F["kwh"][(F["fg"] == g) & (F["kind"] == 1)].mean(), G["kwh"][(G["fg"] == g) & (G["kind"] == 1)].mean()
        ok &= abs(b - a) / a <= 0.05
    res["P-FUEL"] = bool(ok)
    # P-TAIL at 5,700 detached houses
    F, w = synth(rng, n=5700, rho_area_kwh=0.5, tail=True, only_detached=True, all_electric=True)   # all-electric, so that total energy follows electricity
    G, gw, _, _, _, _ = fitgen(F, w, seeds_per=20)
    W1 = np.column_stack([w] + [w * (rng.random(len(w)) > 1 / 60) * 60 / 59 for _ in range(60)])
    res["P-TAIL"] = s5(F, W1, G, gw)["reading"].startswith("kill")
    res["_P-TAIL"] = {k: v for k, v in s5(F, W1, G, gw).items()}
    # P-STRAT: a thin cell drops in the stated order and returns its level
    F, w = synth(rng, n=6000)
    rows, levels = fit_tables(F, w, allm(F))
    thin = rows[key(T["FUEL"], 0, 6, 2)][0]                 # mobile homes in the rarest zone, one era
    full = rows[key(T["TYPE"], 0, 0, 0)][0]
    res["P-STRAT"] = bool(thin >= 2 and full == 0)
    return res


# ---------------------------------------------------------------- gate and score

def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


def gate(raw, binary, out):
    os.makedirs(out, exist_ok=True)
    log = {"passed": False}

    def stop(why):
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out, "gate.json"), "w"), indent=1)
        print(json.dumps(log, indent=1))
        return 1
    if sha(raw) != DIGEST:
        return stop("the file is not the one recorded")
    fit, split = split_states(raw)
    log["split"] = split
    if split != SPLIT:
        return stop("the split's counts differ from DEVICEGEN-1's")
    log["plants"] = plants(binary, out)
    if not all(v for k, v in log["plants"].items() if not k.startswith("_")):
        return stop("a plant failed")
    F, W = load(raw, fit)                                    # FIT rows alone
    w, allm = W[:, 0], np.ones(len(W), bool)
    rng = np.random.default_rng(31)
    rows, levels, chol, info = build(F, w, allm, binary, out, rng)
    write_tables(rows, os.path.join(out, "tables_fit.txt"))
    rival, _, _, _ = build(F, w, allm, binary, out, rng, era_chain=(), identity=True)
    write_tables(rival, os.path.join(out, "tables_rival.txt"))
    for name, chain in (("wall", ("fuel", "roof")), ("fuel", ("wall", "roof"))):
        r2, _, _, _ = build(F, w, allm, binary, out, rng, era_chain=chain, chol=chol)
        write_tables(r2, os.path.join(out, "tables_no_era_%s.txt" % name))
    log["levels"], log["copula"] = levels, {str(k): v for k, v in info.items()}
    json.dump({str(k): v for k, v in chol.items()}, open(os.path.join(out, "chol.json"), "w"))
    G, idx, dg = generate_for(F, allm, binary, os.path.join(out, "tables_fit.txt"), 41)
    gw = w[idx]
    worst, n = 0.0, float(len(idx))
    detail = {}
    for name, k in (("kind", 5), ("vint", 9), ("fuel", 7), ("wall", 8)):
        a, b = np.bincount(F[name], weights=w, minlength=k) / w.sum(), np.bincount(G[name], weights=gw, minlength=k) / gw.sum()
        ex = float(np.max(np.abs(a - b) - 3 * np.sqrt(a * (1 - a) / (n / SEEDS))))
        detail[name] = round(float(np.max(np.abs(a - b))), 4)
        worst = max(worst, ex)
    for name in ("area", "kwh"):
        cuts = wquant(F[name], w, np.array([0.2, 0.4, 0.6, 0.8]))
        a = np.bincount(np.digitize(F[name], cuts), weights=w, minlength=5) / w.sum()
        b = np.bincount(np.digitize(G[name], cuts), weights=gw, minlength=5) / gw.sum()
        ex = float(np.max(np.abs(a - b) - 3 * np.sqrt(a * (1 - a) / (n / SEEDS))))
        detail[name] = round(float(np.max(np.abs(a - b))), 4)
        worst = max(worst, ex)
    log["P-MARG_on_FIT"] = {"largest_difference_by_attribute": detail, "worst_excess_over_3_se": round(worst, 4), "met": bool(worst <= 0.005)}
    law = bool(np.all(G["total"] * 1000 + 500 >= G["kwh"] * 3412) and np.all(G["heat"] + G["cool"] <= G["total"]) and np.all((G["storeys"] != 0) == is_house(G["kind"])) and np.all((G["roof"] == NA) == (G["kind"] == 4)))
    log["section_law_on_FIT"] = law
    miss = max(v["worst_miss"] for v in info.values())
    log["copula_fit_worst_miss"] = miss
    log["terciles_detached_area"] = [float(x) for x in wquant(F["area"][F["kind"] == 1], w[F["kind"] == 1], np.array([1 / 3, 2 / 3]))]
    if not log["P-MARG_on_FIT"]["met"]:
        return stop("P-MARG fails on the real tables")
    if not law:
        return stop("the section law fails")
    if miss > 0.02:
        return stop("generated Spearman on FIT is not within 0.02 of FIT's for each pair (worst %.4f)" % miss)
    log["passed"] = True
    json.dump(log, open(os.path.join(out, "gate.json"), "w"), indent=1)
    print(json.dumps({k: v for k, v in log.items() if k != "copula"}, indent=1))
    return 0


def score(raw, binary, out):
    g = json.load(open(os.path.join(out, "gate.json")))
    if not g.get("passed"):
        print("the gate did not pass; SCORE is not opened")
        return 1
    fit, _ = split_states(raw)
    S, W = load(raw, ~fit)
    FITREC, FW = load(raw, fit)
    w, allm = W[:, 0], np.ones(len(W), bool)
    rng = np.random.default_rng(53)
    log = {"gate": {k: g[k] for k in ("split", "P-MARG_on_FIT", "copula_fit_worst_miss", "levels")}, "score_records": int(len(w))}

    def gen(tables, seed=61):
        G, idx, dg = generate_for(S, allm, binary, os.path.join(out, tables), seed)
        return G, w[idx]
    G, gw = gen("tables_fit.txt")
    R, rw = gen("tables_rival.txt")
    cuts = np.array(g["terciles_detached_area"])
    log["D1"] = d1_all(S, W, G, gw)
    log["D2"] = d2(S, W, G, gw, cuts)
    log["D3"] = d3(S, W, G, gw, FITREC, FW[:, 0], rng)
    six = [p for p in PAIRS if "storeys" not in p]
    rv = d1_table(S, W, R, rw, S["kind"] == 1, R["kind"] == 1, six)
    outside = sum(not r["within"] for r in rv)
    d1met = log["D1"]["reading"] == "met"
    log["S4"] = {"rival_within_detached": rv, "rival_outside_of_6": outside,
                 "reading": "kill" if outside <= 1 else ("met" if (outside >= 4 and d1met) else "neither")}
    log["S5"] = s5(S, W, G, gw)
    era_tabs = {"wall": ("wall by era", "wall", "era", 8, 3, None), "fuel": ("fuel by era", "fuel", "era", 7, 3, None)}
    log["S6"] = {}
    for name, tab in era_tabs.items():
        gd = d3(S, W, G, gw, FITREC, FW[:, 0], rng, tables=[tab])["tables"][0]
        R2, r2w = gen("tables_no_era_%s.txt" % name)
        rd = d3(S, W, R2, r2w, FITREC, FW[:, 0], rng, tables=[tab])["tables"][0]
        bar = 0.05 + gd["gap"]
        reading = "era stays" if (rd["distance"] > 0.10 and gd["distance"] <= bar) else ("kill: era dropped from this table" if rd["distance"] <= bar else "era stays, not shown")
        log["S6"][name] = {"generator": gd["distance"], "without_era": rd["distance"], "bar": round(bar, 4), "reading": reading}
    # printed, no verdict
    pr = {}
    names = ["mobile home", "detached", "attached", "apartment 2-4", "apartment 5+"]
    pr["share_by_type_survey_generated"] = {names[k]: [round(float(w[S["kind"] == k].sum() / w.sum()), 4), round(float(gw[G["kind"] == k].sum() / gw.sum()), 4)] for k in range(5)}
    inten = lambda R_, ww, m: float((ww[m] * R_["total"][m]).sum() / (ww[m] * R_["area"][m]).sum())
    pr["kbtu_per_sqft_by_vintage_detached_survey_generated"] = {str(v): [round(inten(S, w, (S["kind"] == 1) & (S["vint"] == v)), 2), round(inten(G, gw, (G["kind"] == 1) & (G["vint"] == v)), 2)] for v in range(9)}
    mean = lambda R_, ww, m, c: round(float(np.average(R_[c][m], weights=ww[m])), 0) if m.sum() else None
    pr["mean_kwh_and_total_kbtu_by_type_survey_generated"] = {names[k]: [mean(S, w, S["kind"] == k, "kwh"), mean(G, gw, G["kind"] == k, "kwh"), mean(S, w, S["kind"] == k, "total"), mean(G, gw, G["kind"] == k, "total")] for k in range(5)}
    pr["mean_kwh_and_total_by_fuel_group_detached_survey_generated"] = {["electricity", "gas", "other", "none"][f]: [mean(S, w, (S["kind"] == 1) & (S["fg"] == f), "kwh"), mean(G, gw, (G["kind"] == 1) & (G["fg"] == f), "kwh"),
                                                                        mean(S, w, (S["kind"] == 1) & (S["fg"] == f), "total"), mean(G, gw, (G["kind"] == 1) & (G["fg"] == f), "total")] for f in range(4)}
    pr["electric_heating_share_by_zone_survey_generated"] = {ZONES[z]: [round(float(w[(S["zone"] == z) & (S["fg"] == 0)].sum() / max(w[S["zone"] == z].sum(), 1e-9)), 3),
                                                                        round(float(gw[(G["zone"] == z) & (G["fg"] == 0)].sum() / max(gw[G["zone"] == z].sum(), 1e-9)), 3)] for z in range(8) if (S["zone"] == z).sum() > 0}
    pr["electric_heating_share_by_era_survey_generated"] = {str(e): [round(float(w[(S["era"] == e) & (S["fg"] == 0)].sum() / w[S["era"] == e].sum()), 3), round(float(gw[(G["era"] == e) & (G["fg"] == 0)].sum() / gw[G["era"] == e].sum()), 3)] for e in range(3)}
    for label, z in (("Cold", 0), ("Hot-Humid", 2)):
        ms, mg = (S["kind"] == 1) & (S["zone"] == z), (G["kind"] == 1) & (G["zone"] == z)
        pr["detached_%s_area_total_spearman_survey_generated" % label] = [round(wspearman(S["area"][ms], S["total"][ms], w[ms]), 3), round(wspearman(G["area"][mg], G["total"][mg], gw[mg]), 3)]
    pr["small_types_D1_no_verdict"] = {names[k]: d1_table(S, W, G, gw, S["kind"] == k, G["kind"] == k, [p for p in PAIRS if "storeys" not in p or k == 2]) for k in (0, 2, 3)}
    pr["all_electric_share_by_zone_survey_generated"] = {ZONES[z]: [round(float(w[(S["zone"] == z) & (S["nonelec"] == 0)].sum() / max(w[S["zone"] == z].sum(), 1e-9)), 3),
                                                                    round(float(gw[(G["zone"] == z) & (G["nonelec"] == 0)].sum() / max(gw[G["zone"] == z].sum(), 1e-9)), 3)] for z in range(8) if (S["zone"] == z).sum() > 0}
    log["printed"] = pr
    r = {k: log[k]["reading"] for k in ("D1", "D2", "D3", "S4")}
    if "kill" in (r["D1"], r["D2"], r["D3"]):
        log["branch"] = "(k): the generator is killed on " + ", ".join(k for k in ("D1", "D2", "D3") if r[k] == "kill") + "; nothing ships"
    elif r["S4"] == "kill":
        log["branch"] = "(r): the rival ships; joint: not needed"
    elif "neither" in r.values():
        log["branch"] = "(c): the generator ships with marks: " + ", ".join(k for k, v in r.items() if v == "neither")
    else:
        log["branch"] = "(a): the generator ships"
    json.dump(log, open(os.path.join(out, "results.json"), "w"), indent=1)
    print(json.dumps({k: (v if k not in ("printed", "gate") else "...") for k, v in log.items()}, indent=1)[:9000])
    return 0


if __name__ == "__main__":
    cmd = sys.argv[1]
    if cmd == "plants":
        r = plants(sys.argv[2])
        for k, v in r.items():
            print("%-20s %s" % (k, v if k.startswith("_") else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for k, v in r.items() if not k.startswith("_")) else 1)
    sys.exit({"gate": gate, "score": score}[cmd](*sys.argv[2:5]))
