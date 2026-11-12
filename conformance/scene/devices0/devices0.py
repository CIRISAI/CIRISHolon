#!/usr/bin/env python3
"""DEVICES-0 (United States): the instrument of `DEVICES0_PREREG.md`.

  devices0.py plants
  devices0.py run RAW_DIR OUT_DIR

`plants` opens no survey file. Single-threaded. Standard errors by the agency's jackknife:
V = (R-1)/R * sum over replicates of (estimate_r - estimate)^2, with R = 60.
"""
import csv
import json
import os
import sys

import numpy as np

KINDS = ["smartphone", "other mobile phone", "laptop", "desktop", "tablet", "television",
         "set-top box", "streaming device", "games console", "smart speaker"]
COLS = {"smartphone": ["NUMSMPHONE"], "other mobile phone": ["CELLPHONE"], "laptop": ["NUMLAPTOP"],
        "desktop": ["DESKTOP"], "tablet": ["NUMTABLET"], "television": ["TVCOLOR"],
        "set-top box": ["CABLESAT", "COMBODVR", "SEPDVR"], "streaming device": ["INTSTREAM"],
        "games console": ["PLAYSTA"], "smart speaker": ["SMARTSPK"]}
ATTR = {"gateway (declared present with fixed broadband)": ["always on", "unknown"],
        "set-top box": ["always on", "no"], "smart speaker": ["always on", "no"],
        "desktop": ["plugged in, state unknown", "yes"],
        "television": ["plugged in, state unknown", "no"], "games console": ["plugged in, state unknown", "no"],
        "streaming device": ["plugged in, state unknown", "no"],
        "laptop": ["duty-cycled", "yes, while awake"], "tablet": ["duty-cycled", "yes, while awake"],
        "smartphone": ["duty-cycled", "yes, while awake"], "other mobile phone": ["duty-cycled", "no"]}
B_COUNT, B_NONE = 0.15, 0.05
SCHEMES = [[[1], [2], [3], [4], [5], [6], [7]], [[1], [2], [3], [4], [5, 6, 7]], [[1], [2], [3], [4, 5, 6, 7]]]
STATED_TOTAL = {2020: 123.5e6, 2024: 132.5e6}


def decide(miss, gap, budget):
    if abs(miss) + gap <= budget:
        return "certified"
    if abs(miss) - gap > budget:
        return "refused"
    return "unresolved"


def jk(stat, W):
    """stat(w) -> array. Returns (estimate, standard error) by the agency's jackknife."""
    full = np.atleast_1d(np.asarray(stat(W[:, 0]), dtype=float))
    R = W.shape[1] - 1
    reps = np.stack([np.atleast_1d(np.asarray(stat(W[:, r]), dtype=float)) for r in range(1, R + 1)])
    v = (R - 1) / R * ((reps - full) ** 2).sum(0)
    return full, np.sqrt(v)


def wmean(x, w, m=None):
    if m is not None:
        x, w = x[m], w[m]
    s = w.sum()
    return (w * x).sum() / s if s > 0 else np.nan


def size_group(size, groups):
    """index of the pooled group for each household size (7 = seven or more)."""
    out = np.full(len(size), -1)
    for i, g in enumerate(groups):
        out[np.isin(size, g)] = i
    return out


def label(g):
    return str(g[0]) if len(g) == 1 and g[0] < 7 else ("%d or more" % g[0])


def kind_table(D, W, kind, size):
    """Mean count and share with none per pooled size, with the first scheme whose cells all resolve."""
    x = D[kind]
    last = None
    for groups in SCHEMES:
        gi = size_group(size, groups)

        def stat(w):
            return np.concatenate([[wmean(x, w, gi == i) for i in range(len(groups))],
                                   [wmean((x == 0).astype(float), w, gi == i) for i in range(len(groups))]])
        est, se = jk(stat, W)
        k = len(groups)
        rse = se[:k] / np.maximum(est[:k], 1e-12)
        ok = bool((rse <= B_COUNT / 2).all())
        last = {"groups": [label(g) for g in groups], "raw_groups": groups, "mean": est[:k].tolist(), "se": se[:k].tolist(),
                "rse": rse.tolist(), "none": est[k:].tolist(), "none_se": se[k:].tolist(),
                "records": [int((gi == i).sum()) for i in range(k)], "resolved": ok}
        if ok:
            return last
    return last


def three_rates(D, W, kind, keep):
    """Weighted least squares, no intercept: count ~ a*adults + b*seniors + c*children."""
    X = np.stack([D["adults"][keep], D["seniors"][keep], D["children"][keep]], 1)
    y = D[kind][keep]

    def fit(w):
        ww = w[keep]
        A = (X * ww[:, None]).T @ X
        return np.linalg.solve(A, (X * ww[:, None]).T @ y)
    return fit, X, y


# ---------------------------------------------------------------- plants

def synth(n, rng, groups=60):
    w = rng.uniform(0.5, 1.5, n) * 1000
    g = rng.integers(0, groups, n)
    W = np.empty((n, groups + 1))
    W[:, 0] = w
    for r in range(groups):
        W[:, r + 1] = np.where(g == r, 0.0, w * groups / (groups - 1))
    return W


def plants():
    rng = np.random.default_rng(5)
    res = {}
    res["P-EQUAL"] = decide(0.01, 0.03, 0.15) == "certified"
    res["P-SHIFT"] = decide(0.30, 0.03, 0.15) == "refused" and decide(0.16, 0.03, 0.15) == "unresolved"

    n = 20000
    W = synth(n, rng)
    x = rng.poisson(1.2, n).astype(float)
    grp = rng.random(n) < 0.1
    x[grp] = 5.0
    W2 = W.copy()
    W2[grp] *= 10
    truth = (W2[:, 0] * x).sum() / W2[:, 0].sum()
    res["P-WEIGHT"] = abs(wmean(x, W2[:, 0]) - truth) < 1e-9 and abs(x.mean() - truth) > 0.5

    y = rng.normal(3.0, 2.0, n)
    Wsrs = synth(n, rng)
    Wsrs[:, 0] = 1.0
    g = rng.integers(0, 60, n)
    for r in range(60):
        Wsrs[:, r + 1] = np.where(g == r, 0.0, 60 / 59)
    _, se = jk(lambda w: wmean(y, w), Wsrs)
    res["P-REPLICATE"] = abs(se[0] / (2.0 / np.sqrt(n)) - 1) < 0.10

    size = np.concatenate([np.repeat([1, 2, 3, 4, 5, 6], 3000), np.repeat(7, 40), np.repeat(1, n - 18040)])
    D = {"k": rng.poisson(1.0, n).astype(float)}
    t = kind_table(D, synth(n, rng), "k", size)
    res["P-POOL"] = t["groups"][-1] != "7 or more" and t["resolved"]

    ad = rng.integers(1, 4, n).astype(float)
    sen = rng.integers(0, 2, n).astype(float)
    ch = rng.integers(0, 4, n).astype(float)
    cnt = rng.binomial(ad.astype(int), 0.95) + rng.binomial(sen.astype(int), 0.70) + rng.binomial(ch.astype(int), 0.30)
    Dr = {"adults": ad, "seniors": sen, "children": ch, "p": cnt.astype(float), "q": cnt + 0.5 * (ad + sen + ch >= 5) * (ad + sen + ch)}
    Wr = synth(n, rng)
    keep = np.ones(n, dtype=bool)
    fit, X, yv = three_rates(Dr, Wr, "p", keep)
    b = fit(Wr[:, 0])
    ok = np.abs(b - np.array([0.95, 0.70, 0.30])).max() < 0.03
    res["P-RATES"] = bool(ok and s2_eval(Dr, Wr, "p", keep, ad + sen + ch)["met"] and s2_eval(Dr, Wr, "q", keep, ad + sen + ch)["refused_sizes"] >= 2)

    a = rng.random(n) < 0.2
    bnd = rng.random(n) < 0.15
    res["P-CLUSTER"] = abs(cluster(a, bnd, W[:, 0]) - 1.0) < 0.1 and cluster(a, a, W[:, 0]) > 1.5
    res["P-TOPCODE"] = int(below_top(np.array([7, 3, 2]), np.array([0, 4, 1]), 2020).sum()) == 1
    return res


def cluster(a, b, w):
    pa, pb, pab = wmean(a.astype(float), w), wmean(b.astype(float), w), wmean((a & b).astype(float), w)
    return pab / (pa * pb)


def below_top(members, children, year):
    return (members < 7) & (children < 4) if year == 2020 else (members < 7) & (children < 4)


def s2_eval(D, W, kind, keep, members):
    fit, X, y = three_rates(D, W, kind, keep)
    mk = members[keep]

    def stat(w):
        b = fit(w)
        ww = w[keep]
        out = []
        for s in range(1, 7):
            m = mk == s
            pred = wmean(X[m] @ b, ww[m])
            act = wmean(y[m], ww[m])
            out += [pred / act - 1, act]
        return np.concatenate([out, b])
    est, se = jk(stat, W)
    rows, refused, cert = [], 0, 0
    for i, s in enumerate(range(1, 7)):
        v = decide(est[2 * i], 2 * se[2 * i], B_COUNT)
        rows.append({"size": s, "relative_miss": float(est[2 * i]), "gap": float(2 * se[2 * i]), "actual": float(est[2 * i + 1]), "verdict": v})
        refused += v == "refused"
        cert += v == "certified"
    return {"rates_adult_senior_child": est[12:].tolist(), "rates_se": se[12:].tolist(), "by_size": rows,
            "met": bool(cert == 6), "refused_sizes": int(refused)}


# ---------------------------------------------------------------- reading

def load(path, year):
    with open(path, newline="") as f:
        rd = csv.reader(f)
        h = next(rd)
        ix = {c: i for i, c in enumerate(h)}
        suf = "" if year == 2020 else "_PUB"
        need = sorted({c for v in COLS.values() for c in v} | {"NHSLDMEM" + suf, "NUMCHILD" + suf, "NUMADULT1" + suf, "NUMADULT2" + suf,
                      "ATHOME", "INTERNET", "INTYPECELL", "INTYPEBROAD", "TYPEHUQ", "TVONWD1", "UATYP10" if year == 2020 else "UATYP20"})
        missing = [c for c in need + ["NWEIGHT"] + ["NWEIGHT%d" % i for i in range(1, 61)] if c not in ix]
        raw = {c: [] for c in need}
        W = []
        for r in rd:
            for c in need:
                raw[c].append(r[ix[c]])
            W.append([float(r[ix["NWEIGHT"]])] + [float(r[ix["NWEIGHT%d" % i]]) for i in range(1, 61)])
    W = np.array(W)
    num = lambda c: np.array([float(v) if v not in ("", ".") else np.nan for v in raw[c]])
    D = {}
    for k, cs in COLS.items():
        D[k] = sum(np.where(num(c) < 0, 0.0, np.nan_to_num(num(c), nan=0.0)) for c in cs)
    D["members"] = num("NHSLDMEM" + suf)
    D["children"] = num("NUMCHILD" + suf)
    D["adults"] = num("NUMADULT1" + suf)
    D["seniors"] = num("NUMADULT2" + suf)
    D["athome"] = num("ATHOME")
    D["internet"] = num("INTERNET")
    D["broadband"] = (num("INTYPEBROAD") == 1)
    D["cell"] = (num("INTYPECELL") == 1)
    D["housing"] = num("TYPEHUQ")
    D["tvhours"] = num("TVONWD1")
    D["urban"] = np.array(raw["UATYP10" if year == 2020 else "UATYP20"])
    D["size"] = np.clip(D["members"], 1, 7)
    return D, W, missing


def run(raw, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    log = {"plants": plants()}
    print("plants:", log["plants"])

    def stop(why):
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
        print("BRANCH (e):", why)
        return 1
    if not all(log["plants"].values()):
        return stop("a plant failed")
    Y = {}
    g0 = {}
    for year, fn in ((2020, "recs2020_public_v7.csv"), (2024, "recs2024_public_v1.csv")):
        D, W, missing = load(os.path.join(raw, fn), year)
        Y[year] = (D, W)
        tot = float(W[:, 0].sum())
        below = below_top(D["members"], D["children"], year)
        consistent = float((D["members"][below] == (D["children"] + D["adults"] + D["seniors"])[below]).mean())
        g0[year] = {"records": int(len(W)), "missing_columns": missing, "weighted_households": tot,
                    "total_over_stated": tot / STATED_TOTAL[year], "persons_per_household_top_coded": float(wmean(D["members"], W[:, 0])),
                    "members_equal_parts_below_top_code": consistent,
                    "pass": bool(not missing and abs(tot / STATED_TOTAL[year] - 1) <= 0.01
                                 and 2.3 <= wmean(D["members"], W[:, 0]) <= 2.8 and consistent >= 0.99)}
    log["G0"] = g0
    print("G0:", json.dumps(g0))
    if not all(v["pass"] for v in g0.values()):
        return stop("G0 failed")

    D, W = Y[2020]
    w0 = W[:, 0]
    table = {k: kind_table(D, W, k, D["size"]) for k in KINDS}
    log["table_2020"] = table

    # S1
    below = below_top(D["members"], D["children"], 2020)
    s1 = {}
    for name, ks, twin in (("smartphones per person", ["smartphone"], 1.0), ("laptops per person", ["laptop"], 0.5),
                           ("phones, laptops, desktops and tablets per person", ["smartphone", "other mobile phone", "laptop", "desktop", "tablet"], 2.25)):
        x = sum(D[k] for k in ks)
        est, se = jk(lambda w: (w[below] * x[below]).sum() / (w[below] * D["members"][below]).sum(), W)
        s1[name] = {"reference": float(est[0]), "se": float(se[0]), "twin": twin,
                    "verdict": decide(twin - est[0], 2 * se[0], B_COUNT * est[0])}
    for name, k in (("smartphones per adult 18 and over", "smartphone"),):
        ad = D["adults"] + D["seniors"]
        est, se = jk(lambda w: (w[below] * D[k][below]).sum() / (w[below] * ad[below]).sum(), W)
        s1[name] = {"reference": float(est[0]), "se": float(se[0])}
    log["S1"] = s1

    # S2
    s2 = {k: s2_eval(D, W, k, below, D["members"]) for k in ("smartphone", "laptop", "tablet")}
    log["S2"] = s2
    log["S2_outcome"] = "met" if all(v["met"] for v in s2.values()) else ("killed" if s2["smartphone"]["refused_sizes"] >= 2 else "neither")

    # S3 and S4
    D4, W4 = Y[2024]
    s3, s4 = {}, {}
    for k in KINDS:
        groups = table[k]["raw_groups"]
        gi0, gi4 = size_group(D["size"], groups), size_group(D4["size"], groups)
        e0, se0 = jk(lambda w: np.array([wmean(D[k], w, gi0 == i) for i in range(len(groups))]), W)
        e4, se4 = jk(lambda w: np.array([wmean(D4[k], w, gi4 == i) for i in range(len(groups))]), W4)
        rows = []
        for i, g in enumerate(groups):
            gap = 2 * float(np.hypot(se0[i], se4[i]))
            rows.append({"size": label(g), "y2020": float(e0[i]), "y2024": float(e4[i]), "difference": float(e0[i] - e4[i]),
                         "gap": gap, "budget": float(B_COUNT * e4[i]), "verdict": decide(e0[i] - e4[i], gap, B_COUNT * e4[i])})
        s3[k] = rows
        a, sa = jk(lambda w: wmean(D[k], w), W)
        b, sb = jk(lambda w: wmean(D4[k], w), W4)
        s4[k] = {"y2020": float(a[0]), "y2024": float(b[0]), "relative_change": float(b[0] / a[0] - 1),
                 "se_of_change": float(np.hypot(sa[0], sb[0]) / a[0])}
    log["S3"], log["S4"] = s3, s4
    three = ("smartphone", "laptop", "television")
    allc = [all(r["verdict"] == "certified" for r in s3[k]) for k in three]
    anyr = [any(r["verdict"] == "refused" for r in s3[k]) for k in three]
    log["S3_outcome"] = "met" if all(allc) else ("killed" if sum(anyr) >= 2 else "neither")

    # S5
    sgroups = [[1], [2], [3], [4, 5, 6, 7]]
    gi = size_group(D["size"], sgroups)
    mix = np.array([w0[gi == i].sum() for i in range(4)]) / w0.sum()

    def contrast(maskA, maskB):
        out = {}
        for k in KINDS:
            def stat(w):
                a = sum(mix[i] * wmean(D[k], w, maskA & (gi == i)) for i in range(4))
                b = sum(mix[i] * wmean(D[k], w, maskB & (gi == i)) for i in range(4))
                return np.array([(b - a) / ((a + b) / 2), a, b])
            e, se = jk(stat, W)
            out[k] = {"A": float(e[1]), "B": float(e[2]), "relative_difference": float(e[0]), "gap": float(2 * se[0])}
        return out
    housing = contrast(D["housing"] == 2, D["housing"] == 5)
    urban = contrast(D["urban"] == "U", D["urban"] == "R")
    small = sum(abs(v["relative_difference"]) - v["gap"] <= 0.10 for v in housing.values())
    big = sum(abs(v["relative_difference"]) - v["gap"] > 0.20 for v in housing.values())
    log["S5"] = {"detached_A_against_buildings_of_five_or_more_B": housing, "urban_area_A_against_rural_B": urban,
                 "kinds_under_10pct": int(small), "kinds_over_20pct": int(big),
                 "outcome": "met" if small >= 7 else ("killed" if big >= 4 else "neither"),
                 "records": {"detached": int((D["housing"] == 2).sum()), "five_or_more_units": int((D["housing"] == 5).sum()),
                             "rural": int((D["urban"] == "R").sum())}}

    # S6, S7, S8
    g5 = [[1], [2], [3], [4], [5, 6, 7]]
    gi5 = size_group(D["size"], g5)
    nocomp = (D["laptop"] + D["desktop"] + D["tablet"]) == 0
    nobb = ~D["broadband"]
    cellonly = D["cell"] & ~D["broadband"]
    s6 = {}
    for i, g in enumerate(g5 + [[1, 2, 3, 4, 5, 6, 7]]):
        m = np.isin(D["size"], g)
        hasTV = m & (D["television"] > 0) & (D["tvhours"] >= 0)

        def stat(w):
            return np.array([wmean(D["broadband"].astype(float), w, m), wmean(cellonly.astype(float), w, m),
                             wmean((D["internet"] == 0).astype(float), w, m), wmean((D["desktop"] > 0).astype(float), w, m),
                             wmean(nocomp.astype(float), w, m), wmean(D["athome"], w, m), wmean((D["athome"] == 5).astype(float), w, m),
                             wmean((D["athome"] == 0).astype(float), w, m), wmean(D["tvhours"], w, hasTV)])
        e, se = jk(stat, W)
        names = ["fixed_broadband", "cellular_data_only", "no_internet", "has_desktop", "no_laptop_desktop_or_tablet",
                 "weekdays_someone_home_mean", "someone_home_all_five_weekdays", "nobody_home_any_weekday", "main_tv_hours_weekday"]
        s6["all" if i == 5 else label(g)] = {n: [float(e[j]), float(se[j])] for j, n in enumerate(names)}
    log["S6"] = s6
    e, se = jk(lambda w: cluster(nocomp, nobb, w), W)
    e2, se2 = jk(lambda w: np.array([wmean((nocomp & nobb).astype(float), w), wmean(nocomp.astype(float), w), wmean(nobb.astype(float), w)]), W)
    log["S7"] = {"ratio": float(e[0]), "se": float(se[0]), "share_neither": float(e2[0]), "share_no_computer": float(e2[1]),
                 "share_no_fixed_broadband": float(e2[2]),
                 "outcome": "met" if e[0] - 2 * se[0] >= 1.5 else ("killed" if e[0] + 2 * se[0] < 1.2 else "neither")}
    s8 = {}
    for k in KINDS:
        s8[k] = {}
        for i, g in enumerate(g5):
            m = gi5 == i
            mu = wmean(D[k], w0, m)
            s8[k][label(g)] = float(wmean((D[k] - mu) ** 2, w0, m) / mu) if mu > 0 else None
    log["S8_variance_over_mean"] = s8
    log["attributes"] = ATTR
    log["branch"] = "(a)"
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
    pub = {"kinds": {k: {x: table[k][x] for x in ("groups", "mean", "se", "none", "none_se", "records", "resolved")} for k in KINDS},
           "declared_attributes": ATTR, "budgets": {"count": B_COUNT, "share_with_none": B_NONE},
           "three_rates": {k: {"per_adult_18_64": v["rates_adult_senior_child"][0], "per_adult_65_plus": v["rates_adult_senior_child"][1],
                               "per_child": v["rates_adult_senior_child"][2], "se": v["rates_se"]} for k, v in s2.items()},
           "per_person": s1, "already_there": s6, "have_nots_cluster": log["S7"], "variance_over_mean": s8,
           "year_2024_against_2020": s4, "housing_contrast": log["S5"]}
    json.dump(pub, open(os.path.join(out_dir, "device_table.json"), "w"), indent=1)
    print(json.dumps({k: log[k] for k in ("S1", "S2_outcome", "S3_outcome", "S4", "S7")}, indent=1))
    for k in KINDS:
        t = table[k]
        print("%-18s %s mean %s none %s rse%% %s" % (k, t["groups"], [round(x, 2) for x in t["mean"]], [round(x, 2) for x in t["none"]], [round(100 * x, 1) for x in t["rse"]]))
    for k, v in s2.items():
        print("S2", k, [round(x, 3) for x in v["rates_adult_senior_child"]], [(r["size"], round(r["relative_miss"], 3), round(r["gap"], 3), r["verdict"]) for r in v["by_size"]])
    for k in KINDS:
        print("S3", k, [(r["size"], round(r["y2020"], 2), round(r["y2024"], 2), r["verdict"]) for r in s3[k]])
    print("S5", log["S5"]["outcome"], log["S5"]["kinds_under_10pct"], log["S5"]["kinds_over_20pct"], log["S5"]["records"])
    for k in KINDS:
        print("   %-18s housing %+.2f ±%.2f   urban/rural %+.2f ±%.2f" % (k, housing[k]["relative_difference"], housing[k]["gap"], urban[k]["relative_difference"], urban[k]["gap"]))
    for s, v in s6.items():
        print("S6", s, {n: round(x[0], 3) for n, x in v.items()})
    print("S8", {k: {s: round(x, 2) for s, x in v.items() if x} for k, v in s8.items()})
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-12s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3]))
