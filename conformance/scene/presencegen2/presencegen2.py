#!/usr/bin/env python3
"""PRESENCEGEN-2: fitting and scoring for `PRESENCEGEN2_PREREG.md`.

  presencegen2.py plants BIN
  presencegen2.py run RAW_DIR BIN OUT_DIR

BIN is the built example `presencegen_draw` of `holon-scene`. Single-threaded.
The script loads 2022 and 2023 alone until the gate and the plants have passed.
Generators: chain (G1), frame (G3, one chain at home), band and countdown (the two keys of G4).
"""
import json
import os
import subprocess
import sys
import tempfile

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "presencegen1"))
from presencegen1 import (load, key, ppm, write_tables, from_hours, stats, hourly, synth_days,  # noqa: E402
                          T_KIND, T_INIT, T_CHAIN, T_HOME_INIT, T_HOME_CHAIN, T_LEFT, T_AWAY, T_INSIDE_INIT, T_INSIDE,
                          T_CHANGES, T_EMPLOYED, ANY, ANY_SLOT, MIN_N, KINDS, BUDGET)

T_H2 = {"band": 12, "countdown": 13}
T_H2_INIT = {"band": 14, "countdown": 15}
T_RETURN, T_INSIDE6_INIT, T_INSIDE6 = 16, 17, 18
POOLED, NONE, AFTER = 31, 6, 7
INS5 = np.array([4, 4, 0, 1, 2, 3])       # state -> the plain frame's inside index (home asleep read as home awake)
INS6 = np.array([5, 4, 0, 1, 2, 3])       # state -> the keyed fill's inside index
GROUPS = ["before 0", "before 1", "before 2", "before 3", "before 4", "before 5", "no outing", "after"]
SEEDS = 20


def frames(S):
    away = S > 1
    has = away.any(1)
    L = np.where(has, away.argmax(1), -1)
    R = np.where(has, 24 - away[:, ::-1].argmax(1), -1)
    return L, R


def wsum(cat, w, mask, n):
    return np.bincount(cat[mask], weights=w[mask], minlength=n)[:n]


def group_of(which, L, R, slot):
    """The group of each diary's hour `slot` at home; -1 inside the frame."""
    g = np.full(len(L), -1)
    g[L < 0] = NONE
    before = (L >= 0) & (slot < L)
    g[before] = (L[before] // 4) if which == "band" else (np.minimum(L[before] - slot, 6) - 1)
    g[(L >= 0) & (slot >= R)] = AFTER
    return g


def fit(D, within, kindsel):
    """Tables as {key: values}. `kindsel`: the diaries for kind shares and 4 a.m. states;
    `within`: the diaries for everything else (the prereg's table of sources)."""
    rows = {}
    S, w, kind, emp, dt = D["samp"].astype(int), D["w"], D["kind"], D["emp"], D["dt"]
    L, R = frames(S)
    left = np.cumsum(S > 1, axis=1) > 0
    cell = np.where(L < 0, 13, np.where(L < 2, L, L // 2 + 1))
    home = S <= 1
    info = {"keyed_rows": {"band": 0, "countdown": 0}, "keyed_rows_under_30": {"band": 0, "countdown": 0}}

    def put(k, v, mask, need=0):
        if mask.sum() >= max(need, 1):
            p = ppm(v)
            if p:
                rows[k] = p
                return True
        return False

    for e in (0, 1):
        me = emp == e
        ks, wi = kindsel & me, within & me
        for d in range(4):
            m = ks & (dt == d)
            if m.sum() == 0:
                m = ks
            put(key(T_KIND, e, d, 0, 0), wsum(kind, w, m, 4), m)
            put(key(T_INIT, e, d, 0, 0), wsum(S[:, 0], w, m, 6), m)
        # G1
        tot = np.zeros((12, 6))
        for slot in range(23):
            frm = S[:, slot] * 2 + left[:, slot]
            nxt = S[:, slot + 1]
            for f in range(12):
                mf = wi & (frm == f)
                if not mf.any():
                    continue
                v = wsum(nxt, w, mf, 6)
                tot[f] += v
                put(key(T_CHAIN, e, ANY, slot, f), v, mf, MIN_N)
                for d in range(4):
                    md = mf & (dt == d)
                    put(key(T_CHAIN, e, d, slot, f), wsum(nxt, w, md, 6), md, MIN_N)
        for f in range(12):
            if tot[f].sum() > 0:
                rows[key(T_CHAIN, e, ANY, ANY_SLOT, f)] = ppm(tot[f])
        # at home: the row over everyone, and the keyed rows
        m0 = ks & home[:, 0]
        put(key(T_HOME_INIT, e, 0, 0, 0), wsum(S[:, 0], w, m0, 2), m0)
        for which in ("band", "countdown"):
            g0 = group_of(which, L, R, 0)
            for x in range(7):
                mx = m0 & (g0 == x)
                put(key(T_H2_INIT[which], e, x, 0, 0), wsum(S[:, 0], w, mx, 2), mx, MIN_N)
        for slot in range(23):
            both = wi & home[:, slot] & home[:, slot + 1]
            for f in range(2):
                mf = both & (S[:, slot] == f)
                if not put(key(T_HOME_CHAIN, e, 0, slot, f), wsum(S[:, slot + 1], w, mf, 2), mf):
                    rows[key(T_HOME_CHAIN, e, 0, slot, f)] = [500000, 500000]
                for which in ("band", "countdown"):
                    g = group_of(which, L, R, slot + 1)
                    # a transition into the hour of return is the return draw, not the chain
                    g[(L >= 0) & (slot + 1 == R)] = -1
                    for x in range(8):
                        mx = mf & (g == x)
                        if mx.any():
                            ok = put(key(T_H2[which], e, x, slot, f), wsum(S[:, slot + 1], w, mx, 2), mx, MIN_N)
                            info["keyed_rows" if ok else "keyed_rows_under_30"][which] += 1
        # the state on return
        mr = wi & (L >= 0) & (R < 24)
        sr = S[np.arange(len(S)), np.minimum(R, 23)]
        put(key(T_RETURN, e, 0, POOLED, 0), wsum(sr, w, mr, 2), mr)
        for b in range(12):
            mb = mr & (R // 2 == b)
            put(key(T_RETURN, e, 0, b, 0), wsum(sr, w, mb, 2), mb, MIN_N)
        # the frame and its fill, by kind
        dur = R - L - 1
        for k in range(1, 4):
            mk = wi & (kind == k)
            if mk.sum() == 0:
                mk = within & (kind == k)
            if not put(key(T_LEFT, e, k, 0, 0), wsum(cell, w, mk, 14), mk):
                rows[key(T_LEFT, e, k, 0, 0)] = [0] * 13 + [1000000]
            mfr = mk & (L >= 0)
            if not put(key(T_AWAY, e, k, POOLED, 0), wsum(np.maximum(dur, 0), w, mfr, 24), mfr):
                rows[key(T_AWAY, e, k, POOLED, 0)] = [1000000] + [0] * 23
            for c in range(13):
                mc = mfr & (cell == c)
                put(key(T_AWAY, e, k, c, 0), wsum(np.maximum(dur, 0), w, mc, 24), mc, MIN_N)
            idx = np.flatnonzero(mfr)
            for tab_i, tab, n, ins in ((T_INSIDE_INIT, T_INSIDE, 5, INS5[S]), (T_INSIDE6_INIT, T_INSIDE6, 6, INS6[S])):
                first = ins[idx, L[idx]] if len(idx) else np.zeros(0, dtype=int)
                wf = w[idx]
                if tab_i == T_INSIDE_INIT:
                    rows[key(tab_i, e, k, 0, 0)] = ppm(np.bincount(first, weights=wf, minlength=n)[:n]) or [0, 1000000, 0, 0, 0]
                else:
                    for late in (0, 1):
                        ml = (L[idx] != 0) == bool(late)
                        if ml.sum() >= MIN_N:
                            rows[key(tab_i, e, k, late, 0)] = ppm(np.bincount(first[ml], weights=wf[ml], minlength=n)[:n])
                pooled = np.zeros((n, n))
                for slot in range(23):
                    inside = mfr & (L <= slot) & (slot + 1 < R)
                    for f in range(n):
                        mf = inside & (ins[:, slot] == f)
                        if not mf.any():
                            continue
                        v = wsum(ins[:, slot + 1], w, mf, n)
                        pooled[f] += v
                        put(key(tab, e, k, slot, f), v, mf, MIN_N)
                for f in range(n):
                    rows[key(tab, e, k, ANY_SLOT, f)] = ppm(pooled[f]) or ([0, 1000000] + [0] * (n - 2))
            if not put(key(T_CHANGES, e, k, 0, 0), wsum(D["changes"], w, mk, 13), mk):
                rows[key(T_CHANGES, e, k, 0, 0)] = [1000000] + [0] * 12
    return rows, info


def gen(binary, table_path, which, seeds, emp, dt):
    inp = "".join("%d %d %d\n" % (s, e, d) for s, e, d in zip(seeds, emp, dt))
    out = subprocess.run([binary, table_path, which], input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
    n = len(seeds)
    A = np.array([l.split() for l in out[:n]], dtype=np.int64)
    return {"S": A[:, :24], "kind": A[:, 24], "fallback": A[:, 26], "groups": A[:, 27], "clipped": A[:, 28], "digest": out[n].split()[1]}


def run_gen(binary, tp, which, D, mask, seeds_per=SEEDS):
    idx = np.flatnonzero(mask)
    seeds = (idx[:, None] * 1000 + np.arange(seeds_per)[None, :]).ravel()
    G = gen(binary, tp, which, seeds, np.repeat(D["emp"][idx], seeds_per), np.repeat(D["dt"][idx], seeds_per))
    G["w"], G["emp"], G["dt"] = np.repeat(D["w"][idx], seeds_per), np.repeat(D["emp"][idx], seeds_per), np.repeat(D["dt"][idx], seeds_per)
    return G


# ---------------------------------------------------------------- statistics

def wquart(v, w):
    o = np.argsort(v, kind="stable")
    cw = np.cumsum(w[o]) / w.sum()
    return [float(v[o][min(np.searchsorted(cw, x), len(v) - 1)]) for x in (0.25, 0.5, 0.75)]


def s6(S, w):
    L, _ = frames(S)
    m1 = L >= 1
    before = S[np.flatnonzero(m1), L[m1] - 1]
    m3 = L >= 6
    asleep = ((S == 0) | (S == 4)).sum(1)
    return {"asleep_in_hour_before_leaving": float((w[m1] * (before == 0)).sum() / w[m1].sum()),
            "hours_asleep_quartiles": wquart(asleep, w),
            "home_asleep_at_7_among_leaving_10_or_later": float((w[m3] * (S[m3, 3] == 0)).sum() / w[m3].sum())}


def all_stats(S, w, emp=None):
    out = stats(S, w, emp)
    out.update(s6(S, w))
    return out


SCALARS = ("spells_2h_or_less", "spells_8h_or_more", "home_at_9_14_18", "asleep_in_hour_before_leaving", "home_asleep_at_7_among_leaving_10_or_later")
QUARTS = ("hours_away_quartiles", "hours_asleep_quartiles")


def boot(S, w, rng, B=300):
    n = len(w)
    reps = {k: [] for k in SCALARS + QUARTS}
    for _ in range(B):
        ix = rng.integers(0, n, n)
        s = all_stats(S[ix], w[ix])
        for k in reps:
            reps[k].append(s[k])
    return {k: (2 * np.std(np.array(v), axis=0)).tolist() for k, v in reps.items()}


def boot_hourly(S, w, states_list, rng, B=200):
    n = len(w)
    ind = [np.isin(S, st).astype(float) for st in states_list]
    reps = np.zeros((B, len(states_list), 24))
    for b in range(B):
        ix = rng.integers(0, n, n)
        ww = w[ix]
        for j, a in enumerate(ind):
            reps[b, j] = (ww[:, None] * a[ix]).sum(0) / ww.sum()
    return 2 * reps.std(0)


def flagged_by_group(G, which):
    L, R = frames(G["S"])
    out = {}
    for x in range(8):
        if x == NONE:
            has = L < 0
        elif x == AFTER:
            has = (L >= 0) & (R < 24)
        elif which == "band":
            has = (L >= 1) & (L // 4 == x)
        else:
            has = (L >= x + 1) if x < 5 else (L >= 6)
        tot = (G["w"] * has).sum()
        out[GROUPS[x]] = float((G["w"] * has * ((G["groups"] >> x) & 1)).sum() / tot) if tot > 0 else 0.0
    return out


# ---------------------------------------------------------------- plants

def base_days(rng, n, first_lo=2, first_hi=12, stay=0.4):
    """Leavers wake two hours before leaving; those who stay wake at 10. Evening: asleep from slot 19."""
    S = np.zeros((n, 24), dtype=int)
    info = np.zeros((n, 2), dtype=int) - 1
    for i in range(n):
        if rng.random() < stay:
            S[i, 6:19] = 1
            continue
        a = int(rng.integers(first_lo, first_hi + 1))
        d = int(rng.integers(1, 10))
        b = min(a + d, 23)
        S[i, max(a - 2, 0):19] = 1
        S[i, a:b] = 2 if d >= 6 else 3
        info[i] = (a, b)
    return S, info


def plants(binary):
    rng = np.random.default_rng(23)
    res = {}
    # P-HEAP: trips that end on the hour (PRESENCEGEN-1's, unchanged)
    M = np.ones((4000, 1440), dtype=np.uint8)
    for i in range(4000):
        for _ in range(3):
            h = int(rng.integers(3, 20))
            dur = int(rng.integers(8, 30))
            M[i, h * 60 - dur:h * 60] = 5
    true = float((M == 5).mean())
    at_start = float((M[:, np.arange(24) * 60] == 5).mean())
    seeded = float(np.mean([(M[i, np.arange(24) * 60 + np.random.default_rng(i).integers(0, 60, 24)] == 5).mean() for i in range(4000)]))
    res["P-HEAP"] = at_start < 0.8 * true and abs(seeded - true) < 0.003
    with tempfile.TemporaryDirectory() as td:
        tp = os.path.join(td, "t.txt")

        def fit_run(S, dt=None, within=None, kindsel=None, gens=("chain", "frame", "band", "countdown"), seeds_per=4, gen_mask=None):
            n = len(S)
            D = from_hours(S, np.arange(n) % 2, np.zeros(n, dtype=int) if dt is None else dt)
            allm = np.ones(n, dtype=bool)
            rows, _ = fit(D, allm if within is None else within, allm if kindsel is None else kindsel)
            write_tables(rows, tp)
            return {g: run_gen(binary, tp, g, D, allm if gen_mask is None else gen_mask, seeds_per) for g in gens}

        ones = lambda S: np.ones(len(S))
        # PRESENCEGEN-1's plants on the script's side, rerun through the new tables
        S = synth_days(rng, 6000)
        G = fit_run(S, gens=("chain",))
        a, b = stats(S, ones(S))["mean_spell_away"], stats(G["chain"]["S"], G["chain"]["w"])["mean_spell_away"]
        res["P-MEAN"] = abs(b / a - 1) < 0.05
        res["_mean_spell_truth_chain"] = [a, b]
        S = synth_days(rng, 6000, late_share=0.2)
        G = fit_run(S)
        t8 = stats(S, ones(S))["spells_8h_or_more"]
        g8 = {g: stats(G[g]["S"], G[g]["w"])["spells_8h_or_more"] for g in G}
        res["P-LATE"] = t8 - g8["chain"] > 0.02 and all(abs(g8[g] - t8) <= 0.02 for g in ("frame", "band", "countdown"))
        res["_late_8h_truth_and_generators"] = [t8, g8]
        S = synth_days(rng, 6000, chain_only=True)
        G = fit_run(S, gens=("chain",))
        ts, gs = stats(S, ones(S)), stats(G["chain"]["S"], G["chain"]["w"])
        res["P-CHAIN"] = abs(gs["spells_2h_or_less"] - ts["spells_2h_or_less"]) < 0.03 and abs(gs["spells_8h_or_more"] - ts["spells_8h_or_more"]) < 0.03

        # P-SELECT
        S, _ = base_days(rng, 12000)
        G = fit_run(S, gens=("frame", "band", "countdown"))
        awake = lambda X, w: float((w[:, None] * ((X != 0) & (X != 4))).sum() / (24 * w.sum()))
        t = awake(S, ones(S))
        d = {g: awake(G[g]["S"], G[g]["w"]) - t for g in G}
        ab = {g: s6(G[g]["S"], G[g]["w"])["asleep_in_hour_before_leaving"] for g in G}
        res["P-SELECT"] = d["frame"] > 0.01 and abs(d["band"]) <= 0.005 and abs(d["countdown"]) <= 0.005 and ab["countdown"] <= 0.02
        res["_select_awake_generated_minus_planted"] = d
        res["_select_asleep_in_hour_before_leaving_planted_0"] = ab

        # P-RETURN: the share of returns straight to sleep rises with the two-hour bin of return
        S, info = base_days(rng, 20000, first_lo=2, first_hi=14)
        for i in np.flatnonzero(info[:, 1] >= 0):
            r = info[i, 1]
            p = min(max((r // 2 - 5) * 0.15, 0.0), 0.9)
            S[i, r:] = 1
            S[i, 21:] = 0
            if rng.random() < p:
                S[i, r:] = 0
        G = fit_run(S, gens=("band", "countdown"), seeds_per=10)

        def ret(X, w):
            L, R = frames(X)
            m = (L >= 0) & (R < 24)
            st = X[np.flatnonzero(m), R[m]]
            tot = np.bincount(R[m] // 2, weights=w[m], minlength=12)
            asl = np.bincount(R[m] // 2, weights=w[m] * (st == 0), minlength=12)
            return tot / tot.sum(), np.divide(asl, tot, out=np.zeros(12), where=tot > 0)
        sh, pt = ret(S, ones(S))
        worst = {}
        for g in G:
            _, pg = ret(G[g]["S"], G[g]["w"])
            worst[g] = float(np.abs(pg - pt)[sh >= 0.05].max())
        res["P-RETURN"] = all(v <= 0.015 for v in worst.values())
        res["_return_worst_bin_miss"] = worst
        res["_return_planted_share_asleep_by_bin"] = [round(float(x), 3) for x in pt]

        # P-NAP: a tenth of days with two outings and sleep at home between
        S, info = base_days(rng, 12000, first_lo=10, first_hi=14)
        nap = rng.random(len(S)) < 0.1
        for i in np.flatnonzero(nap):
            a = int(rng.integers(2, 4))
            S[i] = 0
            S[i, a - 1] = 1
            S[i, a:5] = 3
            S[i, 5:9] = 0
            S[i, 9:13] = 3
            S[i, 13:19] = 1
        G = fit_run(S, gens=("band", "countdown"))

        def napstat(X, w):
            L, R = frames(X)
            inside = (np.arange(24)[None, :] >= L[:, None]) & (np.arange(24)[None, :] < R[:, None]) & (L[:, None] >= 0)
            h = inside & (X == 0)
            return float((w[:, None] * h).sum() / (24 * w.sum())), float((w * h.any(1)).sum() / w.sum())
        tn = napstat(S, ones(S))
        gn = {g: napstat(G[g]["S"], G[g]["w"]) for g in G}
        res["P-NAP"] = all(abs(v[0] - tn[0]) <= 0.1 * tn[0] and abs(v[1] - tn[1]) <= 0.02 for v in gn.values())
        res["_nap_hours_share_and_days_planted_then_generated"] = [tn, gn]

        # P-FIRST: a tenth of days start away and asleep at 4
        S, info = base_days(rng, 12000)
        fa = rng.random(len(S)) < 0.1
        for i in np.flatnonzero(fa):
            S[i] = 0
            S[i, 0:3] = 4
            S[i, 3] = 5
            S[i, 4:19] = 1
        G = fit_run(S, gens=("frame", "band", "countdown"))
        tf = float((S[:, 0] == 4).mean())
        gf = {g: float((G[g]["w"] * (G[g]["S"][:, 0] == 4)).sum() / G[g]["w"].sum()) for g in G}
        res["P-FIRST"] = abs(gf["band"] - tf) <= 0.01 and abs(gf["countdown"] - tf) <= 0.01 and tf - gf["frame"] > 0.02
        res["_first_elsewhere_asleep_at_4_planted_then_generated"] = [tf, gf]

        # P-S3-START: Saturdays all start awake, weekdays all start asleep
        S, _ = base_days(rng, 8000)
        dt = np.where(np.arange(len(S)) % 4 < 2, 0, 2)
        S[dt == 2, 0] = np.where(S[dt == 2, 0] <= 1, 1, S[dt == 2, 0])
        G = fit_run(S, dt=dt, within=dt == 0, kindsel=dt == 2, gens=("band", "countdown"), gen_mask=dt == 2)
        st = {}
        for g in G:
            L, _ = frames(G[g]["S"])
            m = L < 0
            st[g] = float((G[g]["w"][m] * (G[g]["S"][m, 0] == 1)).sum() / G[g]["w"][m].sum())
        res["P-S3-START"] = all(v > 0.95 for v in st.values())
        res["_s3_start_awake_share_of_generated_saturdays_with_no_outing"] = st
    return res


# ---------------------------------------------------------------- the run

THREE = ([0], [1], [4])
THREE_NAMES = ("home asleep", "home awake", "elsewhere asleep")


def run(raw, binary, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    log = {}

    def save():
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item() if hasattr(o, "item") else str(o))

    def stop(why):
        log["branch"] = "(e): " + why
        save()
        print("BRANCH (e):", why)
        return 1
    pl = plants(binary)
    log["plants"] = pl
    print("plants:", json.dumps(pl, default=float))
    if not all(v for k, v in pl.items() if not k.startswith("_")):
        return stop("a plant failed")
    rng = np.random.default_rng(52)
    D = load(raw, [2022, 2023])
    w, S = D["w"], D["samp"].astype(int)
    allm = np.ones(len(w), dtype=bool)
    tp = os.path.join(out_dir, "t.txt")

    # the choice of key and G0c: across the two fit years
    cross = {}
    for a_, b_ in ((2022, 2023), (2023, 2022)):
        ma, mb = D["year"] == a_, D["year"] == b_
        rows, _ = fit(D, ma, ma)
        write_tables(rows, tp)
        gap = boot_hourly(S[mb], w[mb], THREE, rng)
        tb = s6(S[mb], w[mb])["asleep_in_hour_before_leaving"]
        for which in ("band", "countdown"):
            G = run_gen(binary, tp, which, D, mb, seeds_per=10)
            miss = np.array([np.abs(hourly(G["S"], G["w"], st) - hourly(S[mb], w[mb], st)) for st in THREE])
            gb = s6(G["S"], G["w"])["asleep_in_hour_before_leaving"]
            cross["%s fitted %d scored %d" % (which, a_, b_)] = {
                "largest_miss_home_asleep": float(miss[0].max()), "asleep_in_hour_before_leaving": [tb, gb],
                "choice_term": float(miss[0].max() + abs(gb - tb)),
                "hours_within": {THREE_NAMES[j]: int((miss[j] <= 0.02 + gap[j]).sum()) for j in range(3)}}
    tot = {which: sum(v["choice_term"] for k, v in cross.items() if k.startswith(which)) for which in ("band", "countdown")}
    g4 = min(tot, key=tot.get)
    g0c = all(h >= 22 for k, v in cross.items() if k.startswith(g4) for h in v["hours_within"].values())
    log["key_choice"] = {"sums": tot, "chosen": g4, "across_years": cross}
    log["G0c"] = {"generator": g4, "pass": bool(g0c)}
    print("choice:", json.dumps(log["key_choice"]), "G0c", g0c)

    # G0 on all of FIT
    rows, info = fit(D, allm, allm)
    tpf = os.path.join(out_dir, "tables_fit.txt")
    write_tables(rows, tpf)
    samp_sh = np.array([(w[:, None] * (S == s)).sum() / (24 * w.sum()) for s in range(6)])
    min_sh = (w[:, None] * D["mins"]).sum(0) / (1440 * w.sum())
    awake = lambda X, ww: float((ww[:, None] * ((X != 0) & (X != 4))).sum() / (24 * ww.sum()))
    fit_awake = awake(S, w)
    g0 = {"largest_difference_seeded_minute": float(np.abs(samp_sh - min_sh).max()), "awake_share_diaries": fit_awake, "awake_share": {}, "digests_native": {},
          "reproduction_three_states_by_hour": {}}
    for which in ("chain", "frame", g4):
        G = run_gen(binary, tpf, which, D, allm, seeds_per=5)
        g0["awake_share"][which] = awake(G["S"], G["w"])
        g0["digests_native"][which] = G["digest"]
        miss = np.array([np.abs(hourly(G["S"], G["w"], st) - hourly(S, w, st)) for st in THREE])
        g0["reproduction_three_states_by_hour"][which] = {THREE_NAMES[j]: {"largest_miss": float(miss[j].max()), "hours_within_0.02": int((miss[j] <= 0.02).sum())} for j in range(3)}
    g0["pass"] = bool(g0["largest_difference_seeded_minute"] <= 0.01 and abs(g0["awake_share"][g4] - fit_awake) <= 0.01)
    g0["chain_pass"] = bool(abs(g0["awake_share"]["chain"] - fit_awake) <= 0.01)
    log["G0"] = g0
    log["fit"] = dict(info, rows=len(rows), diaries_fit=int(len(w)))
    print("G0:", json.dumps(g0))
    if not (g0["pass"] and g0c):
        return stop("the gate failed")
    save()

    # ------------------------------------------------ 2024 is loaded here, after the gate
    D = load(raw, [2022, 2023, 2024])
    w, S = D["w"], D["samp"].astype(int)
    fitm, sco = D["year"] < 2024, D["year"] == 2024
    rows, _ = fit(D, fitm, fitm)
    write_tables(rows, tp)
    assert open(tp).read() == open(tpf).read(), "the fit tables changed on reloading"
    ws, Ss, es = w[sco], S[sco], D["emp"][sco]
    truth = all_stats(Ss, ws, es)
    gaps = boot(Ss, ws, rng)
    res = {"diaries_2024": truth, "gap": gaps, "diaries_scored": int(sco.sum())}
    gens = {}
    for which in ("chain", "frame", g4):
        G = run_gen(binary, tpf, which, D, sco)
        gens[which] = G
        r = all_stats(G["S"], G["w"], G["emp"])
        r["fallback_share_of_days"] = float((G["w"] * G["fallback"]).sum() / G["w"].sum())
        r["clipped_share_of_days"] = float((G["w"] * G["clipped"]).sum() / G["w"].sum())
        hh = []
        for d in range(4):
            m, mt = G["dt"] == d, D["dt"][sco] == d
            for st_ in range(6):
                hh.append(np.abs(hourly(G["S"][m], G["w"][m], [st_]) - hourly(Ss[mt], ws[mt], [st_])))
        r["hourly_shares_largest_miss"], r["hourly_shares_mean_miss"] = float(np.max(hh)), float(np.mean(hh))
        if which == g4:
            r["flagged_by_group"] = flagged_by_group(G, g4)
        res[which] = r
    log["scores"] = res

    def within(g, k):
        return abs(res[g][k] - truth[k]) <= BUDGET + gaps[k]

    def qwithin(g, k):
        return all(abs(res[g][k][i] - truth[k][i]) <= 1 + gaps[k][i] for i in range(3))
    s1 = {g: {k: bool(within(g, k)) for k in ("spells_2h_or_less", "spells_8h_or_more")} for g in ("chain", "frame", g4)}
    f_ok, c_ok = all(s1[g4].values()), all(s1["chain"].values())
    log["S1"] = {"within_budget": s1, "outcome": "met" if (f_ok and not c_ok) else ("killed" if c_ok else "neither")}
    s2 = {g: bool(within(g, "home_at_9_14_18") and qwithin(g, "hours_away_quartiles")) for g in ("chain", "frame", g4)}
    better = abs(res["chain"]["home_at_9_14_18"] - truth["home_at_9_14_18"]) - abs(res[g4]["home_at_9_14_18"] - truth["home_at_9_14_18"])
    log["S2"] = {"within": s2, "g4_better_than_chain_by": float(better),
                 "outcome": "met" if s2[g4] else ("killed" if (s2["chain"] and better <= gaps["home_at_9_14_18"]) else "neither")}
    s6r = {g: {"asleep_in_hour_before_leaving": bool(within(g, "asleep_in_hour_before_leaving")), "hours_asleep_quartiles": bool(qwithin(g, "hours_asleep_quartiles")),
               "home_asleep_at_7_among_leaving_10_or_later": bool(within(g, "home_asleep_at_7_among_leaving_10_or_later"))} for g in ("chain", "frame", g4)}
    log["S6"] = {"within": s6r, "outcome": "killed" if all(s6r["frame"].values()) else ("met" if all(s6r[g4].values()) else "neither")}

    # S3
    mth, sat = D["dt"] == 0, D["dt"] == 2
    rows3, _ = fit(D, fitm & mth, fitm & sat)
    write_tables(rows3, os.path.join(out_dir, "tables_s3.txt"))
    b = hourly(S[sco & sat], w[sco & sat], [1])
    s3 = {}
    for which in ("frame", g4):
        G = run_gen(binary, os.path.join(out_dir, "tables_s3.txt"), which, D, sco & sat)
        a = hourly(G["S"], G["w"], [1])
        s3[which] = {"hours_within_0.03": int((np.abs(a - b) <= BUDGET).sum()), "largest_miss": float(np.abs(a - b).max())}
    msat = gens[g4]["dt"] == 2
    a0 = hourly(gens[g4]["S"][msat], gens[g4]["w"][msat], [1])
    n3 = s3[g4]["hours_within_0.03"]
    log["S3"] = dict(s3, with_tables_pooled_over_day_types_hours_within=int((np.abs(a0 - b) <= BUDGET).sum()),
                     outcome="met" if n3 >= 18 else ("killed" if n3 < 12 else "neither"))

    # S4
    Dold = load(raw, [2015, 2016, 2017, 2018, 2019])
    n_old = len(Dold["w"])
    C = {k: np.concatenate([Dold[k], D[k]]) for k in ("samp", "kind", "changes", "emp", "dt", "w")}
    old = np.arange(len(C["w"])) < n_old
    newfit = np.concatenate([np.zeros(n_old, dtype=bool), fitm])
    m4 = sco & mth
    b = hourly(S[m4], w[m4], [0, 1])
    s4 = {}
    for name, ks in (("kind_shares_2022_2023", newfit), ("kind_shares_2015_2019_too", old)):
        rows4, _ = fit(C, old, ks)
        tp4 = os.path.join(out_dir, "tables_s4_%s.txt" % name)
        write_tables(rows4, tp4)
        G = run_gen(binary, tp4, g4, D, m4)
        a = hourly(G["S"], G["w"], [0, 1])
        s4[name] = {"hours_within_0.03": int((np.abs(a - b) <= BUDGET).sum()), "largest_miss": float(np.abs(a - b).max())}
    n4 = s4["kind_shares_2022_2023"]["hours_within_0.03"]
    kshare = lambda X, m: {KINDS[k]: float(X["w"][m & (X["kind"] == k)].sum() / X["w"][m].sum()) for k in range(4)}
    log["S4"] = dict(s4, kind_shares_2015_2019=kshare(Dold, np.ones(n_old, dtype=bool)), kind_shares_2022_2023=kshare(D, fitm), diaries_2015_2019=n_old,
                     outcome="met" if n4 >= 18 else ("killed" if n4 < 12 else "neither"))

    # S5
    m0 = D["dt"][sco] == 0
    p = hourly(Ss[m0], ws[m0], [0, 1])
    log["S5"] = {"person_at_home_Mon_Thu_by_slot": p.tolist(), "nobody_home_two_adults_if_independent": ((1 - p) ** 2).tolist(),
                 "nobody_home_ceiling": (1 - p).tolist(), "household_joint": "unresolved"}
    ag = np.where(D["age"] < 25, 0, np.where(D["age"] < 65, 1, 2))
    emp_age = [float(w[(ag == a_) & (D["emp"] == 1)].sum() / w[ag == a_].sum()) for a_ in range(3)]
    log["employed_share_by_age_band_15_24_25_64_65_plus"] = emp_age

    o1, o2, o6 = log["S1"]["outcome"], log["S2"]["outcome"], log["S6"]["outcome"]
    thin = [g for g, v in res[g4]["flagged_by_group"].items() if v > 0.1]
    if o1 == "met" or (o2 == "met" and s1[g4]["spells_2h_or_less"]):
        br = "(a): the frame ships with the %s fill" % g4
    elif o1 == "killed":
        br = "(b): the chain ships"
    else:
        br = "(c): neither within budget; the better on S1 ships marked unresolved for durations"
    log["branch"] = br
    log["asleep_awake_split"] = "certified" if (o6 == "met" and not thin) else "unresolved"
    log["thin_groups"] = thin
    allm = np.ones(len(w), dtype=bool)
    rows_all, info_all = fit(D, allm, allm)
    for a_ in range(3):
        rows_all[key(T_EMPLOYED, 0, a_, 0, 0)] = ppm([emp_age[a_], 1 - emp_age[a_]])
    write_tables(rows_all, os.path.join(out_dir, "tables_all.txt"))
    log["all_tables"] = dict(info_all, rows=len(rows_all))
    save()
    print(json.dumps({k: log[k] for k in ("scores", "S1", "S2", "S6", "S3", "S4", "branch", "asleep_awake_split", "thin_groups")}, indent=1, default=float))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants(sys.argv[2])
        for k, v in r.items():
            print("%-10s %s" % (k, json.dumps(v, default=float) if k.startswith("_") else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for k, v in r.items() if not k.startswith("_")) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3], sys.argv[4]))
