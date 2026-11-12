#!/usr/bin/env python3
"""PRESENCE-0 (United States): the instrument of `PRESENCE0_PREREG.md`.

  presence0.py plants
  presence0.py run RAW_DIR OUT_DIR

`plants` opens no survey file. Single-threaded.
States: 0 home asleep, 1 home awake, 2 workplace or school, 3 elsewhere awake,
4 elsewhere asleep, 5 travelling. Slot s of a diary is clock hour (4 + s) mod 24.
"""
import io
import json
import os
import sys
import zipfile

import numpy as np

STATES = ["home asleep", "home awake", "workplace or school", "elsewhere awake", "elsewhere asleep", "travelling"]
DAYTYPES = ["Monday to Thursday", "Friday", "Saturday", "Sunday"]
DT_OF_DAY = {1: 3, 2: 0, 3: 0, 4: 0, 5: 0, 6: 1, 7: 2}          # the survey's 1 = Sunday ... 7 = Saturday
CELL_BUDGET, AWAKE_BUDGET_FRACTION = 0.03, 0.05
BOOT, SEED = 500, 20261117


def decide(miss, gap, budget):
    if abs(miss) + gap <= budget:
        return "certified"
    if abs(miss) - gap > budget:
        return "refused"
    return "unresolved"


# ---------------------------------------------------------------- one diary

def place_of(where, tier1):
    if tier1 == 18 or 12 <= where <= 21 or where == 99:
        return "T"
    if where == 1:
        return "H"
    if where in (2, 8):
        return "W"
    if where < 0:
        return "U"
    return "E"


def diary_states(episodes):
    """episodes: list of (duration24 in minutes, six-digit activity code, where code).
    Returns (24 x 6 minutes, sleep minutes placed by default, total minutes)."""
    kinds = []
    for dur, code, where in episodes:
        kinds.append([dur, code // 10000 == 101 // 1 and False, place_of(where, code // 10000)])
    for i, (dur, code, where) in enumerate(episodes):
        kinds[i][1] = (code // 100 == 101)          # 0101xx: sleeping and sleeplessness
    # segments between travel episodes
    n = len(kinds)
    place = [k[2] for k in kinds]
    defaulted = 0
    i = 0
    while i < n:
        if place[i] == "T":
            i += 1
            continue
        j = i
        while j < n and place[j] != "T":
            j += 1
        located = [k for k in range(i, j) if place[k] != "U"]
        for k in range(i, j):
            if place[k] != "U":
                continue
            if not located:
                place[k] = "H"
                if kinds[k][1]:
                    defaulted += kinds[k][0]
            else:
                before = [q for q in located if q < k]
                place[k] = place[before[-1]] if before else place[located[0]]
        i = j
    out = np.zeros((24, 6))
    t = 0
    for (dur, asleep, _), p in zip(kinds, place):
        if p == "T":
            s = 5
        elif asleep:
            s = 0 if p == "H" else 4
        else:
            s = {"H": 1, "W": 2, "E": 3}[p]
        end = min(t + dur, 1440)
        while t < end:
            slot = t // 60
            step = min(end, (slot + 1) * 60) - t
            out[slot, s] += step
            t += step
        t = end
    return out, defaulted, sum(k[0] for k in kinds)


# ---------------------------------------------------------------- aggregation

def cell_days(dt, clock):
    """Diary days whose slot for this clock hour carries this day type."""
    days = []
    for d in range(1, 8):
        name = d if clock >= 4 else (d % 7) + 1
        if DT_OF_DAY[name] == dt:
            days.append(d)
    return days


def table(X, w, day, mask=None):
    """X: n x 24 x 6 minutes. Returns 4 x 24 x 6 shares (day type, clock hour, state)."""
    if mask is None:
        mask = np.ones(len(w), dtype=bool)
    T = np.zeros((4, 24, 6))
    for clock in range(24):
        slot = (clock - 4) % 24
        for dt in range(4):
            m = mask & np.isin(day, cell_days(dt, clock))
            sw = w[m].sum()
            T[dt, clock] = (w[m, None] * X[m, slot]).sum(0) / (60 * sw) if sw > 0 else np.nan
    return T


def week_table(X, w, day):
    """7 x 24 x 6 by day NAME (index 0 = Sunday) and clock hour."""
    T = np.zeros((7, 24, 6))
    for clock in range(24):
        slot = (clock - 4) % 24
        for name in range(1, 8):
            d = name if clock >= 4 else ((name - 2) % 7) + 1
            m = day == d
            T[name - 1, clock] = (w[m, None] * X[m, slot]).sum(0) / (60 * w[m].sum())
    return T


def home9to17(X):
    """Per respondent: share of 9:00 to 17:00 at home (asleep or awake)."""
    slots = [(c - 4) % 24 for c in range(9, 17)]
    return X[:, slots][:, :, [0, 1]].sum((1, 2)) / (60 * 8)


def standardised(y, w, group, cls, ncls):
    """Mean of y in each group after standardising to the all-rows mix over cls."""
    pi = np.array([w[cls == c].sum() for c in range(ncls)])
    pi = pi / pi.sum()
    out = []
    for g in np.unique(group):
        v = 0.0
        tot = 0.0
        for c in range(ncls):
            m = (group == g) & (cls == c)
            if w[m].sum() > 0:
                v += pi[c] * (w[m] * y[m]).sum() / w[m].sum()
                tot += pi[c]
        out.append(v / tot)
    return np.array(out)


def boot(stat, n, w, rng, B=BOOT):
    full = np.asarray(stat(w), dtype=float)
    reps = np.empty((B,) + full.shape)
    p = np.full(n, 1.0 / n)
    for b in range(B):
        reps[b] = stat(w * rng.multinomial(n, p))
    return full, np.nanstd(reps, axis=0)


def predict_strata(X, w, day, cls, ncls, size, fit, score):
    """S6a: at-home rows by class fitted on `fit`, reweighted to each size stratum of `score`."""
    home = X[:, :, 0] + X[:, :, 1]
    H = np.zeros((ncls, 4, 24))
    for c in range(ncls):
        H[c] = table(X, w, day, fit & (cls == c))[:, :, [0, 1]].sum(2)
    P = np.zeros((3, 4, 24))
    R = np.zeros((3, 4, 24))
    for g in range(3):
        mg = score & (size == g)
        R[g] = table(X, w, day, mg)[:, :, [0, 1]].sum(2)
        for clock in range(24):
            for dt in range(4):
                m = mg & np.isin(day, cell_days(dt, clock))
                mix = np.array([w[m & (cls == c)].sum() for c in range(ncls)])
                if mix.sum() > 0:
                    hh = np.where(np.isnan(H[:, dt, clock]), 0, H[:, dt, clock])
                    ok = ~np.isnan(H[:, dt, clock])
                    P[g, dt, clock] = (mix[ok] * hh[ok]).sum() / mix[ok].sum() if mix[ok].sum() > 0 else np.nan
                else:
                    P[g, dt, clock] = np.nan
    del home
    return P - R


def score_cells(diff, se, budget=CELL_BUDGET):
    gap = 2 * se
    can = gap <= budget / 2
    v = np.array([decide(d, g, budget) for d, g in zip(diff.ravel(), gap.ravel())]).reshape(diff.shape)
    n = int(can.sum())
    return {"cells": int(diff.size), "cells_that_can_carry_a_verdict": n,
            "certified_share": float((v[can] == "certified").mean()) if n else None,
            "refused_share": float((v[can] == "refused").mean()) if n else None,
            "refused_anywhere": int((v == "refused").sum()), "largest_abs_difference": float(np.nanmax(np.abs(diff)))}


def outcome(s):
    if not s["cells_that_can_carry_a_verdict"]:
        return "untestable: no cell can carry a verdict"
    if s["certified_share"] >= 0.70:
        return "met"
    if s["refused_share"] >= 0.10:
        return "killed"
    return "neither"


# ---------------------------------------------------------------- plants

def synth(n, rng, base, day=None):
    """n synthetic diaries drawn from a 4 x 24 x 6 table (one state per slot per person)."""
    day = rng.integers(1, 8, n) if day is None else day
    X = np.zeros((n, 24, 6))
    for slot in range(24):
        clock = (slot + 4) % 24
        for d in range(1, 8):
            name = d if clock >= 4 else (d % 7) + 1
            m = np.where(day == d)[0]
            p = base[DT_OF_DAY[name], clock]
            s = rng.choice(6, size=len(m), p=p / p.sum())
            X[m, slot, s] = 60
    return X, day


def plants():
    rng = np.random.default_rng(9)
    res = {}
    S, G, W_, H_, C = 10101, 10201, 50101, 120303, 180501      # sleep, grooming, work, tv, travel
    # P-MIDNIGHT and P-LASTEPISODE: sleep 4-7, home awake, sleep from 23:00 "until 07:30" cut at 4 a.m.
    x, dflt, tot = diary_states([(180, S, -1), (960, H_, 1), (300, S, -1)])
    res["P-LASTEPISODE"] = tot == 1440 and abs(x.sum() - 1440) < 1e-9
    res["P-MIDNIGHT"] = x[19, 0] == 60 and x[23, 0] == 60 and cell_days(2, 1) == [6] and cell_days(2, 5) == [7]
    x, dflt, _ = diary_states([(180, S, -1), (30, G, -1), (30, C, 12), (480, W_, 2), (30, C, 12), (690, H_, 1)])
    res["P-GROOM-DRIVE"] = x[0, 0] == 60 and x[:, 4].sum() == 0 and dflt == 180
    x, dflt, _ = diary_states([(240, H_, 11), (480, S, -1), (120, H_, 11), (600, H_, 11)])
    res["P-AWAY"] = x[:, 4].sum() == 480 and x[:, 0].sum() == 0 and dflt == 0
    x, _, _ = diary_states([(15, C, 12), (45, H_, 1), (15, C, 12), (45, H_, 1), (15, C, 12), (45, H_, 1), (1260, H_, 1)])
    res["P-SHORT"] = bool((x[:3, 5] == 15).all())

    base = np.zeros((4, 24, 6))
    base[:, :, 1] = 0.5
    base[:, :, 0] = 0.3
    base[:, :, 2] = 0.2
    n = 14000
    day = np.concatenate([rng.integers(2, 7, n // 2), rng.choice([1, 7], n // 2)])
    Xs, day = synth(n, rng, base, day)
    w = np.where(np.isin(day, [1, 7]), 0.4, 1.0)           # weekend diaries: 5 times the per-day rate, 0.4 the weight
    per_day = np.array([w[day == d].sum() for d in range(1, 8)])
    raw_day = np.array([(day == d).sum() for d in range(1, 8)])
    res["P-WEIGHT"] = per_day.max() / per_day.min() < 1.15 and raw_day.max() / raw_day.min() > 2

    # P-STANDARD: two groups that differ only in their mix of two classes
    m = 20000
    cls = np.concatenate([rng.random(m // 2) < 0.8, rng.random(m // 2) < 0.2]).astype(int)
    grp = np.repeat([0, 1], m // 2)
    y = np.where(cls == 1, 0.7, 0.3) + rng.normal(0, 0.05, m)
    ww = np.ones(m)
    raw = abs(y[grp == 0].mean() - y[grp == 1].mean())
    st = standardised(y, ww, grp, cls, 2)
    res["P-STANDARD"] = raw > 0.2 and abs(st[0] - st[1]) < 0.01

    # P-NULL-YEAR and P-SHIFT: a held-out half drawn from the same table, then shifted
    base2 = np.zeros((4, 24, 6))
    base2[:, :, 1] = 0.35
    base2[:, :, 0] = 0.25
    base2[:, :, 3] = 0.40
    Xa, da = synth(16000, rng, base2)
    Xb, db = synth(8000, rng, base2)

    def diff_and_se(Xb_):
        X_ = np.concatenate([Xa, Xb_])
        d_ = np.concatenate([da, db])
        fit = np.arange(len(d_)) < len(da)
        stat = lambda w_: (table(X_, w_, d_, fit)[:, :, [0, 1]].sum(2) - table(X_, w_, d_, ~fit)[:, :, [0, 1]].sum(2))
        return boot(stat, len(d_), np.ones(len(d_)), np.random.default_rng(3), B=120)
    d0, s0 = diff_and_se(Xb)
    null = score_cells(d0, s0)
    res["P-NULL-YEAR"] = null["cells_that_can_carry_a_verdict"] >= 0 and null["refused_anywhere"] <= 1
    Xc = Xb.copy()
    for clock in range(9, 17):                     # move elsewhere-awake to at-home for a quarter of persons
        slot = clock - 4
        mv = rng.random(len(Xc)) < 0.25
        Xc[mv, slot, :] = 0
        Xc[mv, slot, 1] = 60
    d1, s1 = diff_and_se(Xc)
    v = np.array([decide(a, 2 * b, CELL_BUDGET) for a, b in zip(d1[0, 9:17], s1[0, 9:17])])
    res["P-SHIFT"] = bool((v == "refused").all())
    res["_null_year_certified_share"] = null["certified_share"]
    return res


# ---------------------------------------------------------------- reading

def read_csv_cols(zpath, member, cols, keep=None):
    with zipfile.ZipFile(zpath) as z:
        with z.open(member) as f:
            t = io.TextIOWrapper(f, encoding="latin-1")
            h = t.readline().strip().split(",")
            ix = [h.index(c) for c in cols]
            for line in t:
                p = line.rstrip("\n").split(",")
                if keep is None or keep(p[0]):
                    yield [p[i] for i in ix]


def load(raw, years):
    ys = set(str(y) for y in years)
    keep = lambda cid: cid[:4] in ys
    R = {}
    for cid, line, yr, dday, wgt, lfs, nhh, hol in read_csv_cols(os.path.join(raw, "atusresp-0324.zip"), "atusresp_0324.dat",
                                                                 ["TUCASEID", "TULINENO", "TUYEAR", "TUDIARYDAY", "TUFNWGTP", "TELFS", "TRNUMHOU", "TRHOLIDAY"], keep):
        R[cid] = {"year": int(yr), "day": int(dday), "w": float(wgt), "employed": int(lfs) in (1, 2), "nhh": int(nhh), "holiday": int(hol)}
    for cid, line, age in read_csv_cols(os.path.join(raw, "atusrost-0324.zip"), "atusrost_0324.dat", ["TUCASEID", "TULINENO", "TEAGE"], keep):
        if line == "1" and cid in R:
            R[cid]["age"] = int(age)
    for cid, line, met in read_csv_cols(os.path.join(raw, "atuscps-0324.zip"), "atuscps_0324.dat", ["TUCASEID", "TULINENO", "GTMETSTA"], keep):
        if line == "1" and cid in R:
            R[cid]["metro"] = int(met)
    ep = {}
    for cid, dur, code, where in read_csv_cols(os.path.join(raw, "atusact-0324.zip"), "atusact_0324.dat",
                                               ["TUCASEID", "TUACTDUR24", "TRCODEP", "TEWHERE"], keep):
        ep.setdefault(cid, []).append((int(dur), int(code), int(where)))
    ids = sorted(c for c in R if c in ep and "age" in R[c])
    X = np.zeros((len(ids), 24, 6))
    dflt = np.zeros(len(ids))
    tot = np.zeros(len(ids))
    for i, c in enumerate(ids):
        X[i], dflt[i], tot[i] = diary_states(ep[c])
    A = {k: np.array([R[c].get(k, -1) for c in ids]) for k in ("year", "day", "w", "employed", "nhh", "age", "metro", "holiday")}
    A["ids"] = ids
    return X, A, dflt, tot


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
    X, A, dflt, tot = load(raw, [2022, 2023, 2024])
    w, day = A["w"], A["day"]
    n = len(w)
    sleep = (X[:, :, 0] + X[:, :, 4]).sum(1)
    per_day = np.array([w[day == d].sum() for d in range(1, 8)])
    T = table(X, w, day)
    g0 = {"diaries": n, "by_year": {int(y): int((A["year"] == y).sum()) for y in np.unique(A["year"])},
          "diaries_summing_to_1440": float((tot == 1440).mean()), "mean_sleep_hours": float((w * sleep).sum() / w.sum() / 60),
          "weighted_days_max_over_min": float(per_day.max() / per_day.min()),
          "shares_sum_to_one": float(np.abs(T.sum(2) - 1).max()),
          "sleep_minutes_defaulted_to_home_share": float((w * dflt).sum() / (w * sleep).sum())}
    g0["pass"] = bool(g0["diaries_summing_to_1440"] == 1.0 and 8 <= g0["mean_sleep_hours"] <= 9.5
                      and g0["weighted_days_max_over_min"] <= 1.05 / 0.95 and g0["shares_sum_to_one"] < 1e-9)
    log["G0"] = g0
    print("G0:", g0)
    if not g0["pass"]:
        return stop("G0 failed")

    emp = A["employed"].astype(bool)
    age = np.where(A["age"] < 25, 0, np.where(A["age"] < 65, 1, 2))
    size = np.where(A["nhh"] == 1, 0, np.where(A["nhh"] == 2, 1, 2))
    cls6 = age * 2 + emp.astype(int)

    # the all-persons table with two errors
    Tfull, Tse = boot(lambda ww: table(X, ww, day), n, w, rng)
    rep_ratio = None
    try:
        idx = {c: i for i, c in enumerate(A["ids"])}
        RW = np.zeros((n, 160))
        got = 0
        with zipfile.ZipFile(os.path.join(raw, "atuswgts-0324.zip")) as z:
            with z.open("atuswgts_0324.dat") as f:
                t = io.TextIOWrapper(f, encoding="latin-1")
                t.readline()
                for line in t:
                    cid = line[:14]
                    if cid[:4] in ("2022", "2023", "2024") and cid in idx:
                        RW[idx[cid]] = np.array(line.rstrip("\n").split(",")[1:161], dtype=float)
                        got += 1
        reps = np.stack([table(X, RW[:, r], day) for r in range(160)])
        rse = np.sqrt(4.0 / 160 * ((reps - Tfull) ** 2).sum(0))
        ok = (Tse > 1e-4) & (Tfull > 0.02)
        rep_ratio = float(np.median(rse[ok] / Tse[ok]))
        log["replicate_weights"] = {"respondents_matched": got, "median_ratio_replicate_over_bootstrap": rep_ratio,
                                    "ratio_10th_90th": [float(np.percentile(rse[ok] / Tse[ok], 10)), float(np.percentile(rse[ok] / Tse[ok], 90))]}
    except Exception as e:
        log["replicate_weights"] = {"error": repr(e)}
    k = max(rep_ratio, 1.0) if rep_ratio else 1.0
    log["error_inflation_applied"] = k

    # R1
    def r1(ww):
        Wk = week_table(X, ww, day)
        awake = 1 - Wk[:, :, 0] - Wk[:, :, 4]
        mean_awake = awake.mean()
        Te = table(X, ww, day, emp)
        peak = Te[0, :, 2].max()
        tot_awake = (Wk[:, :, [1, 2, 3, 5]].sum(2)).mean()
        return np.array([mean_awake, peak, (Wk[:, :, 1] + Wk[:, :, 2]).mean() / tot_awake, Wk[:, :, 1].mean() / tot_awake, awake.min(), awake.max()])
    e, se = boot(r1, n, w, rng)
    se = se * k
    Te = table(X, w, day, emp)
    half = [int(c) for c in range(24) if Te[0, c, 2] >= Te[0, :, 2].max() / 2]
    log["R1"] = {
        "weekly_mean_awake_share": {"reference": float(e[0]), "se": float(se[0]), "twin": 0.85,
                                    "verdict": decide(0.85 - e[0], 2 * se[0], AWAKE_BUDGET_FRACTION * e[0]),
                                    "awake_hours_a_day": float(24 * e[0]), "twin_hours": 20.4,
                                    "lowest_and_highest_hour_of_week": [float(e[4]), float(e[5])]},
        "employed_at_workplace_Mon_Thu": {"peak_share": float(e[1]), "se": float(se[1]), "hours_above_half_peak": half,
                                          "twin": "everyone, 8 to 18", "by_clock_hour": Te[0, :, 2].tolist()},
        "wifi_bracket_of_awake_time": {"upper_home_or_workplace_or_school": float(e[2]), "lower_home_only": float(e[3]),
                                       "se": [float(se[2]), float(se[3])], "twin": 0.65}}

    # S3
    d3 = Tfull[0, :, 1] - Tfull[2, :, 1]
    n3 = int((np.abs(d3) > 0.05).sum())
    log["S3"] = {"home_awake_MonThu_minus_Saturday_by_clock_hour": d3.tolist(), "hours_over_0.05": n3,
                 "outcome": "met" if n3 >= 12 else ("killed" if n3 < 4 else "neither")}

    # S4
    mt = np.isin(day, [2, 3, 4, 5])
    y = home9to17(X)

    def s4(ww):
        st = standardised(y[mt], ww[mt], size[mt], cls6[mt], 6)
        raw_ = np.array([(ww[mt & (size == g)] * y[mt & (size == g)]).sum() / ww[mt & (size == g)].sum() for g in range(3)])
        return np.array([st[0] - st[2], raw_[0] - raw_[2], st[0], st[1], st[2], raw_[0], raw_[1], raw_[2]])
    e, se = boot(s4, n, w, rng)
    se = se * k
    log["S4"] = {"standardised_difference_one_person_minus_three_or_more": float(e[0]), "gap": float(2 * se[0]),
                 "raw_difference": float(e[1]), "standardised_by_size": e[2:5].tolist(), "raw_by_size": e[5:8].tolist(),
                 "outcome": "met" if abs(e[0]) >= 0.03 else ("killed" if abs(e[0]) < 0.01 else "neither")}

    # S6a and S6b
    fit, score = A["year"] < 2024, A["year"] == 2024
    d6, s6 = boot(lambda ww: predict_strata(X, ww, day, cls6, 6, size, fit, score), n, w, rng, B=200)
    a = score_cells(d6, s6 * k)
    a["outcome"] = outcome(a)
    a["null_year_certified_share_in_the_plant"] = pl["_null_year_certified_share"]
    log["S6a"] = a
    metro, non = A["metro"] == 1, A["metro"] == 2
    d7, s7 = boot(lambda ww: table(X, ww, day, metro)[:, :, [0, 1]].sum(2) - table(X, ww, day, non)[:, :, [0, 1]].sum(2), n, w, rng, B=200)
    b = score_cells(d7, s7 * k)
    b["outcome"] = outcome(b)
    b["records"] = {"metropolitan": int(metro.sum()), "non_metropolitan": int(non.sum()), "not_identified": int((~metro & ~non).sum())}
    b["mean_difference_metro_minus_non"] = float(np.nanmean(d7))
    b["difference_9_to_17_Mon_Thu"] = float(np.nanmean(d7[0, 9:17]))
    log["S6b"] = b

    # strata and the household bracket
    strata = {}
    for name, m in (("employed", emp), ("not employed", ~emp), ("age 15-24", age == 0), ("age 25-64", age == 1), ("age 65 and over", age == 2),
                    ("household of 1", size == 0), ("household of 2", size == 1), ("household of 3 or more", size == 2)):
        strata[name] = {"diaries": int(m.sum()), "table": np.round(table(X, w, day, m), 4).tolist()}
    br = {}
    for g, nm, cnt in ((0, "household of 1", 1), (1, "household of 2", 2), (2, "household of 3 or more", 3)):
        p = np.array(strata[nm]["table"])[:, :, [0, 1]].sum(2)
        br[nm] = {"person_at_home": np.round(p, 4).tolist(), "anyone_home_lower": np.round(p, 4).tolist(),
                  "anyone_home_if_independent": np.round(1 - (1 - p) ** cnt, 4).tolist(),
                  "anyone_home_upper": np.round(np.minimum(1, cnt * p), 4).tolist(),
                  "note": "measured" if cnt == 1 else "a bracket, not a measurement; members counted as %d; nobody under 15 has a diary" % cnt}
    log["household_bracket"] = {k_: {"note": v["note"]} for k_, v in br.items()}

    # S5
    X0, A0, _, _ = load(raw, [2015, 2016, 2017, 2018, 2019])
    w0, day0 = A0["w"], A0["day"]
    mt0 = np.isin(day0, [2, 3, 4, 5])
    y0 = home9to17(X0)
    Wk0, Wk1 = week_table(X0, w0, day0), week_table(X, w, day)
    log["S5"] = {"diaries_2015_2019": len(w0),
                 "at_home_9_to_17_Mon_Thu": {"y2015_2019": float((w0[mt0] * y0[mt0]).sum() / w0[mt0].sum()), "y2022_2024": float((w[mt] * y[mt]).sum() / w[mt].sum())},
                 "weekly_mean_awake": {"y2015_2019": float((1 - Wk0[:, :, 0] - Wk0[:, :, 4]).mean()), "y2022_2024": float((1 - Wk1[:, :, 0] - Wk1[:, :, 4]).mean())}}
    emp0 = A0["employed"].astype(bool)
    log["S5"]["employed_at_home_9_to_17_Mon_Thu"] = {"y2015_2019": float((w0[mt0 & emp0] * y0[mt0 & emp0]).sum() / w0[mt0 & emp0].sum()),
                                                     "y2022_2024": float((w[mt & emp] * y[mt & emp]).sum() / w[mt & emp].sum())}
    log["branch"] = "(a)"
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
    pub = {"states": STATES, "day_types": DAYTYPES, "clock_hours": list(range(24)),
           "all_persons": {"table": np.round(Tfull, 4).tolist(), "standard_error": np.round(Tse * k, 4).tolist(), "diaries": n},
           "by_day_of_week": {"days": ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"],
                              "table": np.round(week_table(X, w, day), 4).tolist(), "note": "printed without verdicts"},
           "strata": strata, "household": br, "three_numbers": log["R1"], "years": "2022-2024"}
    json.dump(pub, open(os.path.join(out_dir, "presence_table.json"), "w"), indent=1, default=lambda o: o.item())
    print(json.dumps({k_: log[k_] for k_ in ("replicate_weights", "S3", "S4", "S6a", "S6b", "S5")}, indent=1))
    r = log["R1"]
    print("R1 awake", r["weekly_mean_awake_share"])
    print("R1 work peak", round(r["employed_at_workplace_Mon_Thu"]["peak_share"], 3), r["employed_at_workplace_Mon_Thu"]["hours_above_half_peak"])
    print("R1 wifi", r["wifi_bracket_of_awake_time"])
    for dt in range(4):
        print(DAYTYPES[dt])
        for s in range(6):
            print("   %-20s" % STATES[s], " ".join("%.2f" % v for v in Tfull[dt, :, s]))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-14s %s" % (k, v if k.startswith("_") else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for k, v in r.items() if not k.startswith("_")) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3]))
