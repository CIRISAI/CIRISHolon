#!/usr/bin/env python3
"""HOUSEHOLD-0 (Germany): the instrument of `HOUSEHOLD0_DE_PREREG.md`.

  household0_de.py plants
  household0_de.py run RAW_DIR TWIN_HIST.json OUT_DIR

`plants` opens no census file. `run` runs the plants first. Single-threaded.
Only the 1 km and 10 km csv files inside each archive are opened.
"""
import csv
import hashlib
import io
import json
import math
import os
import sys
import zipfile

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "household0"))
from household0 import classify, neff, CLASS_NAMES, BOUNDS, gen_from_counts  # noqa: E402

P1_BUDGET = 0.03
MEAN_BUDGET_FRACTION = 0.05
NEFF_MIN = 30.0
SPLIT_SEED = 20261114
BAND_SEED = 20261113
INTERLEAVE_SEED = 20261115      # not in the freeze, which names no seed for the interleaved print
COLUMN_TOL = 0.03
SIZE_COLS = ["1_Person", "2_Personen", "3_Personen", "4_Personen", "5_Personen", "6_Personen_und_mehr"]
HH_TOTAL = "Insgesamt_Haushalte"
TYPE_COLS = ["FreiEFH", "EFH_DHH", "EFH_Reihenhaus", "Freist_ZFH", "ZFH_DHH", "ZFH_Reihenhaus",
             "MFH_3bis6Wohnungen", "MFH_7bis12Wohnungen", "MFH_13undmehrWohnungen", "AndererGebaeudetyp"]
US = {"mean": {"village": 2.537, "suburban": 2.507, "terraced": 2.586},
      "p1": {"village": 0.255, "suburban": 0.285, "terraced": 0.300}}


# ---------------------------------------------------------------- verdict

def qty(ct):
    """ct columns: households by parts, persons, one-person households, households with a mean."""
    t = ct.sum(0) if ct.ndim == 2 else ct
    return {"mean": float(t[1] / t[3]) if t[3] > 0 else float("nan"),
            "p1": float(t[2] / t[0]) if t[0] > 0 else float("nan")}


def split_gap(ct, seed=SPLIT_SEED, n=1000):
    rng = np.random.default_rng(seed)
    k = ct.shape[0]
    gm, gp = [], []
    for _ in range(n):
        a = np.zeros(k, dtype=bool)
        a[rng.permutation(k)[: k // 2]] = True
        A, B = ct[a].sum(0), ct[~a].sum(0)
        if min(A[0], B[0], A[3], B[3]) <= 0:
            continue
        gm.append(abs(A[1] / A[3] - B[1] / B[3]))
        gp.append(abs(A[2] / A[0] - B[2] / B[0]))
    return {"mean": float(np.percentile(gm, 95)), "p1": float(np.percentile(gp, 95))}


def budgets(ref):
    return {"mean": MEAN_BUDGET_FRACTION * ref["mean"], "p1": P1_BUDGET}


def decide(miss, gap, budget):
    if abs(miss) + gap <= budget:
        return "certified", "within budget with the gap"
    if abs(miss) - gap > budget:
        return "refused", "outside budget by more than the gap"
    return "unresolved", "places differ by more than the budget resolves"


def verdict(gen, ct, fixed_budget=None):
    ne = neff(ct[:, 0])
    out = {"n_eff": ne, "blocks": int((ct[:, 0] > 0).sum()), "households": float(ct[:, 0].sum())}
    if ne < NEFF_MIN:
        for q in ("mean", "p1"):
            out[q] = {"verdict": "unresolved", "reason": "effective blocks under 30"}
        return out
    ref = qty(ct)
    gap = split_gap(ct)
    bud = fixed_budget or budgets(ref)
    for q in ("mean", "p1"):
        v, why = decide(gen[q] - ref[q], gap[q], bud[q])
        out[q] = {"generator": gen[q], "reference": ref[q], "difference": gen[q] - ref[q],
                  "budget": bud[q], "gap_95": gap[q], "verdict": v, "reason": why}
        if v == "refused":
            out[q]["witness"] = "one-person share" if q == "p1" else "persons per household"
    return out


# ---------------------------------------------------------------- the family (a port)

def family(p, s, n_leaves=3334, hh=60, seed=7):
    rng = np.random.default_rng(seed)
    people = int(round(hh * p))
    w = np.clip(np.floor(p + s * rng.standard_normal((n_leaves, hh)) + 0.5), 1, 6)
    q = people * w / w.sum(1, keepdims=True)
    sz = np.floor(q).astype(int)
    left = people - sz.sum(1)
    order = np.argsort(-(q - sz), axis=1, kind="stable")
    rank = np.empty_like(order)
    np.put_along_axis(rank, order, np.arange(hh)[None, :].repeat(n_leaves, 0), axis=1)
    sz += (rank < left[:, None]).astype(int)
    c = np.bincount(sz.ravel(), minlength=8).astype(float)
    return c / c.sum()


def fit_family(p, target_p1):
    best = None
    for s in np.arange(0.30, 4.0001, 0.05):
        sh = family(p, float(s))
        d = abs(sh[1] - target_p1)
        if best is None or d < best[0]:
            best = (d, float(s), sh)
    return {"p": p, "s": best[1], "shares_by_size": best[2].tolist(), "p1_miss": float(best[2][1] - target_p1),
            "zero_person_share": float(best[2][0])}


def bands5(sh):
    """shares at sizes 2, 3, 4 and 5-or-more from a by-size vector starting at 0."""
    return np.array([sh[2], sh[3], sh[4], sh[5:].sum()])


# ---------------------------------------------------------------- disclosure: the column check

def ckm_perturb(x, rng, rule):
    """Synthetic cell-key noise for the plants. rule 'kept': a 1 goes to 3 with
    probability 1/3 and a 2 with 2/3, else 0. rule 'down': each 1 goes to 0."""
    x = np.asarray(x).astype(int)
    out = x + rng.integers(-2, 3, size=x.shape) * (rng.random(x.shape) < 0.35)
    out = np.where(out < 3, 3, out)
    out = np.where(x == 0, 0, out)
    one, two = x == 1, x == 2
    u = rng.random(x.shape)
    if rule == "kept":
        out = np.where(one, np.where(u < 1 / 3, 3, 0), out)
    else:
        out = np.where(one, 0, out)
    out = np.where(two, np.where(u < 2 / 3, 3, 0), out)
    return out


def column_check(fine, fine_group, coarse, min_cells=20):
    """fine: cells x columns at 1 km; fine_group: index of each cell's 10 km cell
    (0..G-1); coarse: G x columns at 10 km. Returns the per-column ratio."""
    if coarse.shape[0] < min_cells:
        return None
    f = np.zeros_like(coarse, dtype=float)
    np.add.at(f, fine_group, fine)
    num, den = f.sum(0), coarse.sum(0)
    return [float(num[i] / den[i]) if den[i] > 0 else None for i in range(coarse.shape[1])]


def band_rule(r):
    """Which size bands may be published, from the ratios of columns
    [1, 2, 3, 4, 5, 6+, 5-or-more, 4-or-more]."""
    bad = lambda v: v is None or abs(v - 1) > COLUMN_TOL
    if r is None:
        return "unchecked", ["1", "2", "3", "4", "5", "6+"]
    if not bad(r[4]) and not bad(r[5]):
        return "checked", ["1", "2", "3", "4", "5", "6+"]
    if not bad(r[6]) and not bad(r[3]):
        return "tail pooled", ["1", "2", "3", "4", "5+"]
    return "tail pooled twice", ["1", "2", "3", "4+"]


# ---------------------------------------------------------------- reading

def num(s):
    s = s.strip()
    if s in ("–", "-", "", "."):
        return 0.0
    return float(s.replace(".", "").replace(",", ".")) if "," in s else float(s)


def read_grid(zpath, res, cols=None):
    """One csv of an archive at resolution '1km' or '10km'. Returns (header, ids, x, y, values)."""
    with zipfile.ZipFile(zpath) as z:
        names = [n for n in z.namelist() if n.lower().endswith(".csv") and ("_%s" % res).lower() in n.lower().replace("-", "_")]
        names = [n for n in names if ("100m" not in n) and (res != "1km" or "10km" not in n)]
        if len(names) != 1:
            raise SystemExit("ambiguous csv for %s in %s: %s" % (res, zpath, names))
        raw = z.read(names[0])
    try:
        text = raw.decode("utf-8-sig")
    except UnicodeDecodeError:
        text = raw.decode("latin-1")
    rd = csv.reader(io.StringIO(text), delimiter=";")
    header = [h.strip() for h in next(rd)]
    ix = [i for i, h in enumerate(header) if h.lower().startswith("x_mp")][0]
    iy = [i for i, h in enumerate(header) if h.lower().startswith("y_mp")][0]
    want = cols if cols is not None else [h for i, h in enumerate(header) if i > max(ix, iy)]
    wi = [header.index(c) for c in want]
    ids, xs, ys, vals = [], [], [], []
    for row in rd:
        if not row or len(row) < len(header):
            continue
        xs.append(int(float(row[ix])))
        ys.append(int(float(row[iy])))
        vals.append([row[i] for i in wi])
    return header, np.array(xs), np.array(ys), vals


def to_float(vals):
    return np.array([[num(v) for v in r] for r in vals], dtype=float)


def key(x, y):
    return x.astype(np.int64) * 10_000_000 + y.astype(np.int64)


def join(kref, k, v, fill=np.nan):
    """Values of (k, v) aligned to kref; `fill` where missing."""
    order = np.argsort(k)
    pos = np.searchsorted(k[order], kref)
    pos = np.clip(pos, 0, len(k) - 1)
    hit = k[order][pos] == kref
    out = np.full((len(kref),) + v.shape[1:], fill, dtype=float)
    out[hit] = v[order][pos[hit]]
    return out, hit


# ---------------------------------------------------------------- plants

def plants():
    from household0 import synth_counties, shares8, w1
    rng = np.random.default_rng(3)
    res = {}
    base = [0.41, 0.33, 0.12, 0.09, 0.035, 0.01, 0.005]

    def ct4(k, dist, **kw):
        c = synth_counties(rng, k, dist, **kw)
        return np.stack([c[:, 0], c[:, 1], c[:, 2], c[:, 0]], 1)
    a, b = ct4(200, base), ct4(200, base)
    g = qty(a)
    v = verdict(g, b)
    res["P-EQUAL"] = v["mean"]["verdict"] == "certified" and v["p1"]["verdict"] == "certified"
    sh = list(base)
    sh[0] += 0.10
    sh[1] -= 0.10
    v = verdict(g, ct4(200, sh))
    res["P-SHIFT"] = v["p1"]["verdict"] == "refused" and v["p1"].get("witness") == "one-person share"
    d1, d2 = [0, .5, .5, 0, 0, 0, 0], [.5, 0, 0, .5, 0, 0, 0]
    v = verdict(qty(ct4(200, d1)), ct4(200, d2))
    res["P-MEAN"] = v["mean"]["verdict"] == "certified" and v["p1"]["verdict"] == "refused" \
        and abs(w1(shares8(d1), shares8(d2)) - 1.0) < 1e-12
    v = verdict(g, ct4(12, base))
    res["P-THIN"] = v["p1"]["verdict"] == "unresolved" and "difference" not in v["p1"]
    dom = ct4(100, base, hh_each=2000)
    dom[0] *= 99 * 99.0
    res["P-NEFF"] = verdict(g, dom)["n_eff"] < 2
    res["P-FLOOR-SPLIT"] = (decide(0.117, 0.023, 0.03)[0] == "refused" and decide(0.020, 0.023, 0.03)[0] == "unresolved"
                            and decide(0.004, 0.010, 0.03)[0] == "certified" and decide(0.119, 0.05, 0.12)[0] == "unresolved")

    # cell-key plants: 4,000 cells of 30 households in 40 ten-kilometre cells
    truth = np.array([0.36, 0.33, 0.15, 0.11, 0.035, 0.015])
    cells = rng.multinomial(30, truth, size=4000)
    grp = np.repeat(np.arange(40), 100)
    coarse_true = np.zeros((40, 6))
    np.add.at(coarse_true, grp, cells)
    out = {}
    for rule in ("kept", "down"):
        fine = ckm_perturb(cells, rng, rule)
        coarse = ckm_perturb(coarse_true, rng, "kept")
        f8 = np.concatenate([fine, fine[:, 4:6].sum(1, keepdims=True), fine[:, 3:6].sum(1, keepdims=True)], 1)
        c8 = np.concatenate([coarse, coarse[:, 4:6].sum(1, keepdims=True), coarse[:, 3:6].sum(1, keepdims=True)], 1)
        r = column_check(f8, grp, c8)
        out[rule] = (r, fine[:, 5].sum() / fine.sum())
    r, s6 = out["kept"]
    res["P-CKM"] = abs(s6 / truth[5] - 1) < 0.05 and all(abs(x - 1) <= COLUMN_TOL for x in r[:6])
    r, _ = out["down"]
    res["P-CKM-BIAS"] = abs(r[4] - 1) > COLUMN_TOL and abs(r[5] - 1) > COLUMN_TOL

    res["P-DASH"] = [num(x) for x in ("–", "–", "–", "17")] == [0.0, 0.0, 0.0, 17.0]
    res["P-COMMA"] = num("2,13") == 2.13
    # a total column that holds population: parts sum to about half of it
    parts, total = 1000.0, 2030.0
    res["P-MISLABEL"] = not (abs(parts / total - 1) <= 0.01)
    tgt = family(2.2, 1.40, seed=11)
    f = fit_family(2.2, float(tgt[1]))
    res["P-FAMILY"] = abs(f["s"] - 1.40) <= 0.05 and float(np.abs(np.array(f["shares_by_size"]) - tgt).max()) <= 0.01
    res["P-TOP13"] = top13_alarm(1200.0, 100.0) and not top13_alarm(2100.0, 100.0)
    return res


def top13_alarm(dwellings, buildings):
    return buildings > 0 and dwellings / buildings < 13


# ---------------------------------------------------------------- the run

def run(raw, twin_path, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    log = {}
    pl = plants()
    log["plants"] = pl
    print("plants:", pl)

    def stop(why):
        print("BRANCH (e):", why, "Nothing is read.")
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
        return 1
    if not all(pl.values()):
        return stop("a plant failed.")
    tb = open(twin_path, "rb").read()
    log["twin_file_sha256"] = hashlib.sha256(tb).hexdigest()
    G = gen_from_counts(json.loads(tb)["final"]["counts_by_size"])
    twin = {"mean": G["mean"], "p1": G["p1"]}

    Z = lambda n: os.path.join(raw, n)
    zh = Z("Zensus2022_Groesse_des_privaten_Haushalts_in_Gitterzellen.zip")
    hd, x, y, v = read_grid(zh, "1km", [HH_TOTAL] + SIZE_COLS)
    H = to_float(v)
    K = key(x, y)
    hd10, x10, y10, v10 = read_grid(zh, "10km", [HH_TOTAL] + SIZE_COLS)
    H10 = to_float(v10)
    hm, xm, ym, vm = read_grid(Z("Durchschnittliche_Haushaltsgroesse_in_Gitterzellen.zip"), "1km")
    mean_col = [h for h in hm if h.lower().startswith("durchschn")][0]
    hm2, xm, ym, vm = read_grid(Z("Durchschnittliche_Haushaltsgroesse_in_Gitterzellen.zip"), "1km",
                                [mean_col] + [h for h in hm if "zeichen" in h.lower()])
    M = np.array([num(r[0]) for r in vm])
    bracket = np.array([("KLAMMER" in r[1].upper() or "(" in r[1]) if len(r) > 1 else False for r in vm])
    hp, xp, yp, vp = read_grid(Z("Zensus2022_Bevoelkerungszahl.zip"), "1km")
    pop_col = [h for h in hp if h.lower().startswith("einwohner")][0]
    hp2, xp, yp, vp = read_grid(Z("Zensus2022_Bevoelkerungszahl.zip"), "1km", [pop_col])
    Pp = to_float(vp)
    log["headers"] = {"household_size_1km": hd, "household_size_10km": hd10, "mean_size_1km": hm, "population_1km": hp}

    mean_al, has_mean = join(K, key(xm, ym), M[:, None])
    mean_al = mean_al[:, 0]
    br_al, _ = join(K, key(xm, ym), bracket[:, None].astype(float), fill=0.0)
    pop_al, has_pop = join(K, key(xp, yp), Pp)
    pop_al = pop_al[:, 0]
    has_mean &= mean_al > 0
    parts = H[:, 1:].sum(1)
    total = H[:, 0]
    persons = np.where(has_mean, mean_al * total, 0.0)
    wmean = np.where(has_mean, total, 0.0)

    # G0
    k5 = np.arange(1, 6)
    res6 = float((persons.sum() - (H[has_mean][:, 1:6] * k5).sum()) / H[has_mean][:, 6].sum())
    g0 = {"parts_over_total_1km": float(parts.sum() / total.sum()),
          "parts_over_total_10km": float(H10[:, 1:].sum() / H10[:, 0].sum()),
          "total_1km_over_10km": float(total.sum() / H10[:, 0].sum()),
          "persons_per_household": float(persons.sum() / wmean.sum()),
          "residual_mean_6plus": res6, "cells_1km": int(len(K)), "cells_10km": int(len(x10)),
          "households": float(total.sum())}
    g0["pass"] = bool(abs(g0["parts_over_total_1km"] - 1) <= 0.01 and abs(g0["parts_over_total_10km"] - 1) <= 0.01
                      and abs(g0["total_1km_over_10km"] - 1) <= 0.005 and 1.8 <= g0["persons_per_household"] <= 2.3
                      and 5.5 <= res6 <= 9)
    log["G0"] = g0
    print("G0:", g0)
    if not g0["pass"]:
        return stop("G0 failed.")

    cls = np.where(has_pop, classify(np.nan_to_num(pop_al, nan=0.0)), -2)
    log["cells"] = {"no_population_row": int((~has_pop).sum()),
                    "households_in_cells_without_population_row": float(total[~has_pop].sum()),
                    "no_mean": int((~has_mean).sum()), "households_without_mean_share": float(total[~has_mean].sum() / total.sum()),
                    "bracketed_mean": int(br_al[:, 0].sum()),
                    "communal_share_of_people": float(1 - persons[has_pop & has_mean].sum() / pop_al[has_pop & has_mean].sum()),
                    "population_over_households": float(np.nansum(pop_al[has_pop]) / total[has_pop].sum())}

    sw_x, sw_y = x - 500, y - 500
    block = (sw_x // 50000) * 1000 + (sw_y // 50000)
    bx = np.minimum(((sw_x - sw_x.min()) * 3) // (sw_x.max() - sw_x.min() + 1), 2)
    by = np.minimum(((sw_y - sw_y.min()) * 2) // (sw_y.max() - sw_y.min() + 1), 1)
    band = by * 3 + bx
    perm = np.random.default_rng(BAND_SEED).permutation(6)
    fit_b, score_b = sorted(perm[:3].tolist()), sorted(perm[3:].tolist())
    log["bands"] = {"FIT": fit_b, "SCORE": score_b,
                    "households_share_FIT": float(total[np.isin(band, fit_b)].sum() / total.sum())}
    sq = (sw_y // 100000) * 1000 + (sw_x // 100000)
    usq = np.unique(sq)
    p2 = np.random.default_rng(INTERLEAVE_SEED).permutation(len(usq))
    fit_sq = set(usq[p2[: (len(usq) + 1) // 2]].tolist())
    inter_fit = np.array([s in fit_sq for s in sq])

    cols = np.stack([parts, persons, H[:, 1], wmean], 1)

    def ct_of(mask):
        ids, inv = np.unique(block[mask], return_inverse=True)
        out = np.zeros((len(ids), 4))
        np.add.at(out, inv, cols[mask])
        return out

    # the column check
    g10 = (sw_x // 10000) * 100000 + (sw_y // 10000)
    k10 = ((x10 - 5000) // 10000) * 100000 + ((y10 - 5000) // 10000)
    u10, inv10 = np.unique(g10, return_inverse=True)
    byc = np.zeros((len(u10), 5))
    okc = cls >= 0
    np.add.at(byc, (inv10[okc], cls[okc]), total[okc])
    grp_cls = np.where(byc.sum(1) > 0, byc.argmax(1), -1)
    f8 = np.concatenate([H[:, 1:], H[:, 5:7].sum(1, keepdims=True), H[:, 4:7].sum(1, keepdims=True)], 1)
    c8all = np.concatenate([H10[:, 1:], H10[:, 5:7].sum(1, keepdims=True), H10[:, 4:7].sum(1, keepdims=True)], 1)
    c8, hit10 = join(u10, k10, c8all, fill=0.0)
    check = {}
    for c in range(5):
        sel = (grp_cls == c) & hit10
        idx = np.full(len(u10), -1)
        idx[sel] = np.arange(sel.sum())
        m = idx[inv10] >= 0
        r = column_check(f8[m], idx[inv10][m], c8[sel])
        state, bandsok = band_rule(r)
        check[CLASS_NAMES[c]] = {"ten_km_cells": int(sel.sum()), "ratios_1_2_3_4_5_6plus_5plus_4plus": r,
                                 "state": state, "bands_published": bandsok}
    log["column_check"] = check

    def reading(mask, name=None):
        h = H[mask][:, 1:].sum(0)
        tot = h.sum()
        mm = mask & has_mean
        bands = check[name]["bands_published"] if name in check else ["1", "2", "3", "4", "5", "6+"]
        if len(bands) == 6:
            sh = h
        elif len(bands) == 5:
            sh = np.concatenate([h[:4], [h[4:].sum()]])
        else:
            sh = np.concatenate([h[:3], [h[3:].sum()]])
        out = {"cells": int(mask.sum()), "households": float(tot), "bands": bands,
               "shares_ppm": [int(round(v)) for v in 1e6 * sh / tot],
               "one_person_share": float(h[0] / tot),
               "persons_per_household": float(persons[mm].sum() / wmean[mm].sum()),
               "population_over_households": float(np.nansum(pop_al[mask & has_pop]) / total[mask & has_pop].sum()),
               "residual_mean_6plus": float((persons[mm].sum() - (H[mm][:, 1:6] * k5).sum()) / max(H[mm][:, 6].sum(), 1)),
               "households_without_mean_share": float(total[mask & ~has_mean].sum() / max(total[mask].sum(), 1)),
               "parts_over_total": float(parts[mask].sum() / total[mask].sum())}
        out["mean_method"] = "limited" if out["households_without_mean_share"] > 0.01 else "ok"
        return out

    rows, S1, resolved = {}, {}, []
    allm = cls >= 0
    rows["nation"] = reading(allm)
    S1["nation"] = verdict(twin, ct_of(allm))
    for c in range(5):
        m = cls == c
        if m.sum() == 0:
            continue
        rows[CLASS_NAMES[c]] = reading(m, CLASS_NAMES[c])
        ct = ct_of(m)
        v = verdict(twin, ct)
        rows[CLASS_NAMES[c]]["n_eff_blocks"] = v["n_eff"]
        if v["n_eff"] >= NEFF_MIN:
            resolved.append(c)
            rows[CLASS_NAMES[c]]["gap_95"] = split_gap(ct)
        S1[CLASS_NAMES[c]] = v
    log["S1"] = S1
    log["resolved"] = [CLASS_NAMES[c] for c in resolved]
    nat = S1["nation"]
    log["S1_summary"] = {"nation_mean": nat["mean"]["verdict"], "nation_p1": nat["p1"]["verdict"],
                         "expectation": "stands" if (nat["mean"]["verdict"] != "certified" and nat["p1"]["verdict"] != "certified") else "killed",
                         "refused_on_both": bool(nat["mean"]["verdict"] == "refused" and nat["p1"]["verdict"] == "refused")}

    def hold(fit_mask, score_mask, label):
        fit, sco = fit_mask & allm, score_mask & allm
        natr = qty(ct_of(fit))
        out = {"FIT-NATION-1": natr, "FIT-CLASS-1": {}, "floors": {}}
        fam = fit_family(natr["mean"], natr["p1"])
        fb = bands5(np.array(fam["shares_by_size"]))
        s1b = {"fit": {k: fam[k] for k in ("p", "s", "p1_miss", "zero_person_share")},
               "family_shares_2_3_4_5plus": fb.tolist(), "per_class": {}}
        used = []
        for c in range(5):
            mf, ms = fit & (cls == c), sco & (cls == c)
            if mf.sum() == 0 or ms.sum() == 0:
                continue
            cf, cs = ct_of(mf), ct_of(ms)
            out["FIT-CLASS-1"][CLASS_NAMES[c]] = dict(qty(cf), n_eff_fit=neff(cf[:, 0]), n_eff_score=neff(cs[:, 0]))
            if neff(cs[:, 0]) >= NEFF_MIN:
                h = H[ms][:, 1:].sum(0)
                ref = np.array([h[1], h[2], h[3], h[4:].sum()]) / h.sum()
                s1b["per_class"][CLASS_NAMES[c]] = {"reference_shares_2_3_4_5plus": ref.tolist(),
                                                    "largest_error": float(np.abs(fb - ref).max())}
            if neff(cf[:, 0]) >= NEFF_MIN and neff(cs[:, 0]) >= NEFF_MIN:
                used.append(c)
        errs = [v["largest_error"] for v in s1b["per_class"].values()]
        if errs:
            n_ok = sum(e <= 0.03 for e in errs)
            s1b["outcome"] = "met" if n_ok * 2 >= len(errs) else ("killed" if all(e > 0.06 for e in errs) else "neither")
        out["S1b"] = s1b
        out["classes_used"] = [CLASS_NAMES[c] for c in used]
        for floor in (1000.0, 5000.0, 20000.0):
            ws = wc = wn = 0.0
            n = 0
            pc = {}
            for c in used:
                cs = ct_of(sco & (cls == c))
                rule = out["FIT-CLASS-1"][CLASS_NAMES[c]]
                big = cs[(cs[:, 0] >= floor) & (cs[:, 3] > 0)]
                if len(big) == 0:
                    continue
                um, up = big[:, 1] / big[:, 3], big[:, 2] / big[:, 0]
                bm = MEAN_BUDGET_FRACTION * um
                inc = (np.abs(um - rule["mean"]) <= bm) & (np.abs(up - rule["p1"]) <= P1_BUDGET)
                inn = (np.abs(um - natr["mean"]) <= bm) & (np.abs(up - natr["p1"]) <= P1_BUDGET)
                w = big[:, 0]
                pc[CLASS_NAMES[c]] = {"units": int(len(big)), "within_class_rule": float((w * inc).sum() / w.sum()),
                                      "within_national_rule": float((w * inn).sum() / w.sum())}
                ws += w.sum()
                wc += (w * inc).sum()
                wn += (w * inn).sum()
                n += len(big)
            d = (wc - wn) / ws if ws else float("nan")
            out["floors"][str(int(floor))] = {
                "units": n, "coverage_of_score_households": float(ws / max(cols[sco][:, 0].sum(), 1)),
                "share_within_class_rule": float(wc / ws) if ws else None,
                "share_within_national_rule": float(wn / ws) if ws else None, "difference": float(d),
                "outcome": "met" if d >= 0.05 else ("killed" if d <= 0 else "neither"), "per_class": pc}
        return out

    log["S2_bands"] = hold(np.isin(band, fit_b), np.isin(band, score_b), "bands")
    log["S2_interleaved"] = hold(inter_fit, ~inter_fit, "interleaved")

    natq = qty(ct_of(allm))
    ms_ = [rows[CLASS_NAMES[c]]["persons_per_household"] for c in resolved]
    ps_ = [rows[CLASS_NAMES[c]]["one_person_share"] for c in resolved]
    dm, dp = (max(ms_) - min(ms_), max(ps_) - min(ps_)) if resolved else (0.0, 0.0)
    bm = MEAN_BUDGET_FRACTION * natq["mean"]
    log["S3"] = {"largest_mean_difference": dm, "largest_p1_difference": dp, "mean_bar": bm,
                 "outcome": "met" if (dm >= bm or dp >= 0.03) else ("killed" if (dm < bm / 2 and dp < 0.015) else "neither")}

    # S5, a reading
    s5 = {}
    common = [n for n in ("village", "suburban", "terraced") if n in log["resolved"]]
    if "suburban" in common:
        for q, kq in (("mean", "persons_per_household"), ("p1", "one_person_share")):
            de0, us0 = rows["suburban"][kq], US[q]["suburban"]
            s5[q] = {}
            for n in common:
                de, us = rows[n][kq], US[q][n]
                lo = (lambda p: math.log(p / (1 - p))) if q == "p1" else None
                e = {"germany": de, "us": us, "us_moved_by_difference": de0 + (us - us0), "us_moved_by_ratio": de0 * us / us0}
                if lo:
                    z = lo(de0) + lo(us) - lo(us0)
                    e["us_moved_by_log_odds"] = 1 / (1 + math.exp(-z))
                e["sign_vs_suburban"] = {"germany": int(np.sign(de - de0)), "us": int(np.sign(us - us0))}
                s5[q][n] = e
    log["S5"] = s5

    # S6, buildings
    s6 = {}
    try:
        zw, zg = Z("Wohnungen_nach_Gebaeudetyp_Groesse.zip"), Z("Gebaeude_mit_Wohnraum_nach_Gebaeudetyp_Groesse.zip")
        hw, xw, yw, vw = read_grid(zw, "1km")
        hg, xg, yg, vg = read_grid(zg, "1km")
        log["headers"]["dwellings_by_type_1km"], log["headers"]["buildings_by_type_1km"] = hw, hg
        tw = [h for h in hw if h.lower().startswith("insgesamt")][0]
        tg = [h for h in hg if h.lower().startswith("insgesamt")][0]
        _, xw, yw, vw = read_grid(zw, "1km", [tw] + TYPE_COLS)
        _, xg, yg, vg = read_grid(zg, "1km", [tg] + TYPE_COLS)
        W, _ = join(K, key(xw, yw), to_float(vw), fill=0.0)
        Bd, hasb = join(K, key(xg, yg), to_float(vg), fill=0.0)
        _, xw10, yw10, vw10 = read_grid(zw, "10km", [tw] + TYPE_COLS)
        _, xg10, yg10, vg10 = read_grid(zg, "10km", [tg] + TYPE_COLS)
        W10, B10 = to_float(vw10), to_float(vg10)
        s6["national_1km_over_10km"] = {"dwellings": (W.sum(0) / np.maximum(W10.sum(0), 1)).tolist(),
                                        "buildings": (Bd.sum(0) / np.maximum(B10.sum(0), 1)).tolist(),
                                        "columns": ["total"] + TYPE_COLS,
                                        "note": "1 km sums are over cells that have a household row; the 10 km sums are over all 10 km cells"}
        for name, m in [("nation", allm)] + [(CLASS_NAMES[c], cls == c) for c in range(5) if (cls == c).sum()]:
            w, b = W[m].sum(0), Bd[m].sum(0)
            top_w, top_b = w[9], b[9]
            s6[name] = {"buildings": float(b[0]), "dwellings": float(w[0]),
                        "building_shares_ppm": [int(round(v)) for v in 1e6 * b[1:] / max(b[1:].sum(), 1)],
                        "dwelling_shares_ppm": [int(round(v)) for v in 1e6 * w[1:] / max(w[1:].sum(), 1)],
                        "dwellings_per_building": float(w[0] / b[0]) if b[0] else None,
                        "dwellings_per_building_13plus": float(top_w / top_b) if top_b else None,
                        "alarm_13plus_under_13": bool(top13_alarm(top_w, top_b)),
                        "households_per_building": float(total[m].sum() / b[0]) if b[0] else None}
        s6["categories"] = TYPE_COLS
    except Exception as e:           # the building reading is S6 only and gates nothing
        s6["error"] = repr(e)
    log["S6"] = s6

    log["branch"] = "(a)"
    table = {"class_lower_bounds_per_km2": BOUNDS, "rows": rows, "column_check": check, "verdicts": S1,
             "budgets": {"persons_per_household": "5 % of the reference value", "one_person_share": P1_BUDGET},
             "generator": twin, "S1b": log["S2_bands"]["S1b"], "S2": log["S2_bands"]["floors"], "S3": log["S3"],
             "S5": s5, "buildings": s6}
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
    json.dump(table, open(os.path.join(out_dir, "household_table.json"), "w"), indent=1)
    print(json.dumps({k: log[k] for k in ("cells", "bands", "column_check", "S1_summary", "resolved", "S3", "S5")}, indent=1))
    for lab in ("S2_bands", "S2_interleaved"):
        print(lab, json.dumps(log[lab], indent=1))
    print("rows", json.dumps(rows, indent=1))
    print("S1", json.dumps(S1, indent=1))
    print("S6", json.dumps(s6, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-14s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3], sys.argv[4]))
