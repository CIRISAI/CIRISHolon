#!/usr/bin/env python3
"""PRESENCEGEN-1: fitting and scoring for `PRESENCEGEN1_PREREG.md`.

  presencegen1.py plants BIN
  presencegen1.py run RAW_DIR BIN OUT_DIR

BIN is the built example `presencegen_draw` of `holon-scene`. Single-threaded.
Slot s of a day is clock hour (4 + s) mod 24. States as in PRESENCE-0.
"""
import json
import os
import subprocess
import sys
import tempfile

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "presence0"))
from presence0 import read_csv_cols, place_of, DT_OF_DAY  # noqa: E402

T_KIND, T_INIT, T_CHAIN, T_HOME_INIT, T_HOME_CHAIN, T_LEFT, T_AWAY, T_INSIDE_INIT, T_INSIDE, T_CHANGES, T_EMPLOYED = range(1, 12)
ANY, ANY_SLOT, MIN_N, SEEDS = 4, 24, 30, 20
INSIDE_OF = {2: 0, 3: 1, 4: 2, 5: 3, 1: 4, 0: 4}
KINDS = ["home", "work, early start", "work, later start", "out"]
BUDGET = 0.03


def key(table, employed, x, slot, frm):
    return (((table * 2 + employed) * 8 + x) * 32 + slot) * 16 + frm


def ppm(wts):
    w = np.asarray(wts, dtype=float)
    if w.sum() <= 0:
        return None
    p = np.floor(1e6 * w / w.sum()).astype(np.int64)
    p[int(np.argmax(p))] += 1_000_000 - p.sum()
    return p.tolist()


# ---------------------------------------------------------------- diaries

def diary_minutes(episodes):
    """1,440 states by minute, by PRESENCE-0's placement rule."""
    place = [place_of(w, c // 10000) for _, c, w in episodes]
    asleep = [c // 100 == 101 for _, c, _ in episodes]
    n = len(episodes)
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
            if place[k] == "U":
                before = [q for q in located if q < k]
                place[k] = "H" if not located else (place[before[-1]] if before else place[located[0]])
        i = j
    out = np.zeros(1440, dtype=np.uint8)
    t = 0
    for (dur, _, _), p, a in zip(episodes, place, asleep):
        s = 5 if p == "T" else ((0 if p == "H" else 4) if a else {"H": 1, "W": 2, "E": 3}[p])
        e = min(t + dur, 1440)
        out[t:e] = s
        t = e
    if t < 1440:
        out[t:] = out[t - 1] if t else 0
    return out


def derive_from_minutes(M, idx):
    """Per-diary quantities from a minute array M and the diary's index (for the seeded minute)."""
    r = np.random.default_rng(900_000 + idx)
    mins = r.integers(0, 60, 24)
    samp = M[np.arange(24) * 60 + mins]
    start = M[np.arange(24) * 60]
    counts = np.bincount(M, minlength=6)
    away = M > 1
    work = M == 2
    if not away.any():
        kind = 0
    elif work.sum() >= 120:
        kind = 1 if int(np.argmax(work)) < 480 else 2
    else:
        kind = 3
    pl = np.where(M <= 1, 0, np.where(M == 2, 1, np.where(M == 5, 9, 2)))
    pl = pl[pl != 9]
    changes = int((np.diff(pl) != 0).sum()) if len(pl) > 1 else 0
    return samp, start, counts, kind, min(changes, 12)


def frame_of(samp):
    away = np.flatnonzero(samp > 1)
    if len(away) == 0:
        return None
    return int(away[0]), int(away[-1]) + 1


def load(raw, years):
    ys = set(str(y) for y in years)
    keep = lambda cid: cid[:4] in ys
    R = {}
    for cid, yr, dday, wgt, lfs in read_csv_cols(os.path.join(raw, "atusresp-0324.zip"), "atusresp_0324.dat",
                                                 ["TUCASEID", "TUYEAR", "TUDIARYDAY", "TUFNWGTP", "TELFS"], keep):
        R[cid] = [int(yr), DT_OF_DAY[int(dday)], float(wgt), int(lfs) in (1, 2), -1]
    for cid, line, age in read_csv_cols(os.path.join(raw, "atusrost-0324.zip"), "atusrost_0324.dat", ["TUCASEID", "TULINENO", "TEAGE"], keep):
        if line == "1" and cid in R:
            R[cid][4] = int(age)
    ep = {}
    for cid, dur, code, where in read_csv_cols(os.path.join(raw, "atusact-0324.zip"), "atusact_0324.dat",
                                               ["TUCASEID", "TUACTDUR24", "TRCODEP", "TEWHERE"], keep):
        ep.setdefault(cid, []).append((int(dur), int(code), int(where)))
    ids = sorted(c for c in R if c in ep)
    n = len(ids)
    D = {"samp": np.zeros((n, 24), dtype=np.uint8), "start": np.zeros((n, 24), dtype=np.uint8), "mins": np.zeros((n, 6)),
         "kind": np.zeros(n, dtype=int), "changes": np.zeros(n, dtype=int)}
    for i, c in enumerate(ids):
        D["samp"][i], D["start"][i], D["mins"][i], D["kind"][i], D["changes"][i] = derive_from_minutes(diary_minutes(ep[c]), i)
    A = np.array([R[c] for c in ids], dtype=float)
    D.update(year=A[:, 0].astype(int), dt=A[:, 1].astype(int), w=A[:, 2], emp=A[:, 3].astype(int), age=A[:, 4].astype(int))
    return D


def from_hours(S, emp, dt, w=None):
    """A derived set built from hourly state sequences alone (for synthetic carriers)."""
    n = len(S)
    kind = np.zeros(n, dtype=int)
    for i in range(n):
        away = S[i] > 1
        work = S[i] == 2
        kind[i] = 0 if not away.any() else ((1 if int(np.argmax(work)) < 8 else 2) if work.sum() >= 2 else 3)
    ch = np.array([min(int((np.diff(np.where(s <= 1, 0, np.where(s == 2, 1, 2))) != 0).sum()), 12) for s in S])
    return {"samp": S.astype(np.uint8), "kind": kind, "changes": ch, "emp": np.asarray(emp), "dt": np.asarray(dt),
            "w": np.ones(n) if w is None else w}


# ---------------------------------------------------------------- fitting

def fit(D, within, kindsel):
    """Tables as {key: values}. `within`: diaries for the within-kind and chain tables;
    `kindsel`: diaries for the kind shares and the 4 a.m. state."""
    rows = {}
    S, w, kind = D["samp"].astype(int), D["w"], D["kind"]
    info = {"chain_rows": 0, "chain_rows_own_day_type": 0, "inside_rows_by_slot": 0}
    for e in (0, 1):
        me = D["emp"] == e
        for d in range(4):
            m = kindsel & me & (D["dt"] == d)
            if m.sum() == 0:
                m = kindsel & me
            rows[key(T_KIND, e, d, 0, 0)] = ppm([w[m & (kind == k)].sum() for k in range(4)])
            rows[key(T_INIT, e, d, 0, 0)] = ppm([w[m & (S[:, 0] == s)].sum() for s in range(6)])
        mw = within & me
        left = np.cumsum(S > 1, axis=1) > 0
        for slot in range(23):
            frm = S[:, slot] * 2 + left[:, slot]
            nxt = S[:, slot + 1]
            for f in range(12):
                mf = mw & (frm == f)
                if mf.sum() >= MIN_N:
                    rows[key(T_CHAIN, e, ANY, slot, f)] = ppm([w[mf & (nxt == s)].sum() for s in range(6)])
                    info["chain_rows"] += 1
                for d in range(4):
                    md = mf & (D["dt"] == d)
                    if md.sum() >= MIN_N:
                        rows[key(T_CHAIN, e, d, slot, f)] = ppm([w[md & (nxt == s)].sum() for s in range(6)])
                        info["chain_rows_own_day_type"] += 1
        for f in range(12):
            tot = np.zeros(6)
            for slot in range(23):
                mf = mw & ((S[:, slot] * 2 + left[:, slot]) == f)
                tot += np.array([w[mf & (S[:, slot + 1] == s)].sum() for s in range(6)])
            p = ppm(tot)
            if p:
                rows[key(T_CHAIN, e, ANY, ANY_SLOT, f)] = p
        # at home
        mh = mw & (S[:, 0] <= 1)
        rows[key(T_HOME_INIT, e, 0, 0, 0)] = ppm([w[mh & (S[:, 0] == s)].sum() for s in range(2)])
        for slot in range(23):
            for f in range(2):
                mf = mw & (S[:, slot] == f) & (S[:, slot + 1] <= 1)
                p = ppm([w[mf & (S[:, slot + 1] == s)].sum() for s in range(2)])
                rows[key(T_HOME_CHAIN, e, 0, slot, f)] = p if p else [500000, 500000]
        # frames
        fr = [frame_of(s) for s in S]
        L = np.array([f[0] if f else -1 for f in fr])
        Rr = np.array([f[1] if f else -1 for f in fr])
        for k in range(1, 4):
            mk = mw & (kind == k)
            if mk.sum() == 0:
                mk = within & (kind == k)
            lb = np.where(L >= 0, L // 2, 12)
            rows[key(T_LEFT, e, k, 0, 0)] = ppm([w[mk & (lb == b)].sum() for b in range(13)]) or [0] * 12 + [1000000]
            db = np.where(L >= 0, Rr - L - 1, -1)  # hours away less one, in one-hour cells (amendment of 2026-11-12)
            mfr = mk & (L >= 0)
            rows[key(T_AWAY, e, k, 12, 0)] = ppm([w[mfr & (db == b)].sum() for b in range(24)]) or [1000000] + [0] * 23
            for b in range(12):
                mb = mfr & (lb == b)
                if mb.sum() >= MIN_N:
                    rows[key(T_AWAY, e, k, b, 0)] = ppm([w[mb & (db == q)].sum() for q in range(24)])
            ins = np.vectorize(INSIDE_OF.get)(S)
            idx = np.flatnonzero(mfr)
            first = ins[idx, L[idx]]
            rows[key(T_INSIDE_INIT, e, k, 0, 0)] = ppm([w[idx][first == q].sum() for q in range(5)]) or [0, 1000000, 0, 0, 0]
            pooled = np.zeros((5, 5))
            for slot in range(23):
                inside = mfr & (L <= slot) & (slot + 1 < Rr)
                for f in range(5):
                    mf = inside & (ins[:, slot] == f)
                    v = np.array([w[mf & (ins[:, slot + 1] == q)].sum() for q in range(5)])
                    pooled[f] += v
                    if mf.sum() >= MIN_N:
                        rows[key(T_INSIDE, e, k, slot, f)] = ppm(v)
                        info["inside_rows_by_slot"] += 1
            for f in range(5):
                rows[key(T_INSIDE, e, k, ANY_SLOT, f)] = ppm(pooled[f]) or [0, 1000000, 0, 0, 0]
            rows[key(T_CHANGES, e, k, 0, 0)] = ppm([w[mk & (D["changes"] == c)].sum() for c in range(13)]) or [1000000] + [0] * 12
    return rows, info


def write_tables(rows, path):
    with open(path, "w") as f:
        for k in sorted(rows):
            if rows[k] is None:  # no diaries in the cell: the row is absent and the generator's fallback applies
                continue
            f.write("%d %s\n" % (k, " ".join(str(int(v)) for v in rows[k])))


def gen(binary, table_path, which, seeds, emp, dt):
    inp = "".join("%d %d %d\n" % (s, e, d) for s, e, d in zip(seeds, emp, dt))
    out = subprocess.run([binary, table_path, which], input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
    n = len(seeds)
    A = np.array([l.split() for l in out[:n]], dtype=np.int64)
    return A[:, :24], A[:, 24], A[:, 25], A[:, 26], out[n].split()[1]


# ---------------------------------------------------------------- statistics

def spells(S, w):
    """(lengths, weights) of unbroken runs away from home."""
    away = (S > 1).astype(np.int8)
    pad = np.zeros((len(S), 1), dtype=np.int8)
    d = np.diff(np.concatenate([pad, away, pad], 1), axis=1)
    rs, cs = np.nonzero(d == 1)
    re_, ce = np.nonzero(d == -1)
    return ce - cs, w[rs]


def stats(S, w, emp=None):
    ln, lw = spells(S, w)
    home = S <= 1
    hrs_away = (~home).sum(1)
    leavers = hrs_away > 0
    order = np.argsort(hrs_away[leavers])
    cw = np.cumsum(w[leavers][order]) / w[leavers].sum()
    q = [float(hrs_away[leavers][order][np.searchsorted(cw, x)]) for x in (0.25, 0.5, 0.75)]
    out = {"spells_2h_or_less": float(lw[ln <= 2].sum() / lw.sum()), "spells_8h_or_more": float(lw[ln >= 8].sum() / lw.sum()),
           "mean_spell_away": float((lw * ln).sum() / lw.sum()), "stays_home": float((w * ~leavers).sum() / w.sum()),
           "home_at_9_14_18": float((w * (home[:, 5] & home[:, 10] & home[:, 14])).sum() / w.sum()),
           "hours_away_quartiles": q, "awake_share": float((w[:, None] * ((S != 0) & (S != 4))).sum() / (24 * w.sum()))}
    if emp is not None:
        m = emp == 1
        out["employed_no_workplace_hour"] = float((w[m] * ((S[m] == 2).sum(1) == 0)).sum() / w[m].sum())
        out["employed_six_or_more_workplace_hours"] = float((w[m] * ((S[m] == 2).sum(1) >= 6)).sum() / w[m].sum())
    return out


def hourly(S, w, states):
    return np.array([(w * np.isin(S[:, s], states)).sum() / w.sum() for s in range(24)])


def boot_stats(S, w, rng, B=300):
    n = len(w)
    keys = ("spells_2h_or_less", "spells_8h_or_more", "home_at_9_14_18")
    reps = {k: [] for k in keys}
    qs = []
    for _ in range(B):
        ix = rng.integers(0, n, n)
        s = stats(S[ix], w[ix])
        for k in keys:
            reps[k].append(s[k])
        qs.append(s["hours_away_quartiles"])
    return {k: 2 * float(np.std(v)) for k, v in reps.items()}, (2 * np.std(np.array(qs), axis=0)).tolist()


def run_gen(binary, tp, which, D, mask, seeds_per=SEEDS):
    idx = np.flatnonzero(mask)
    seeds = (idx[:, None] * 1000 + np.arange(seeds_per)[None, :]).ravel()
    S, kind, ch, fb, dg = gen(binary, tp, which, seeds, np.repeat(D["emp"][idx], seeds_per), np.repeat(D["dt"][idx], seeds_per))
    return S, np.repeat(D["w"][idx], seeds_per), np.repeat(D["emp"][idx], seeds_per), np.repeat(D["dt"][idx], seeds_per), fb, kind, dg


# ---------------------------------------------------------------- plants

def synth_days(rng, n, late_share=0.0, chain_only=False):
    S = np.zeros((n, 24), dtype=int)
    for i in range(n):
        wake = int(rng.integers(2, 5))
        S[i, wake:19] = 1
        if chain_only:
            s, left = 1, 0
            for h in range(wake, 23):
                if s <= 1 and not left and rng.random() < 0.12:
                    s, left = 3, 1
                elif s > 1 and rng.random() < 0.25:
                    s = 1
                S[i, h] = s if (h < 19 or s > 1) else 0
            continue
        u = rng.random()
        if u < 0.25:
            continue
        if u < 0.65:
            a = 10 if rng.random() < late_share else int(rng.integers(3, 6))
            b = min(a + int(rng.integers(7, 10)), 23)
            S[i, a] = 5
            S[i, a + 1:b] = 2
            S[i, b] = 5
        else:
            a = int(rng.integers(5, 14))
            b = min(a + int(rng.integers(1, 4)), 23)
            S[i, a:b] = 3
    return S


def plants(binary):
    rng = np.random.default_rng(17)
    res = {}
    # P-HEAP: trips that end on the hour
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

        def fit_and_run(S):
            n = len(S)
            D = from_hours(S, np.arange(n) % 2, np.zeros(n, dtype=int))  # both employment values, so every row the carrier's digest loop reads exists
            allm = np.ones(n, dtype=bool)
            rows, _ = fit(D, allm, allm)
            write_tables(rows, tp)
            out = {}
            for which in ("chain", "frame"):
                G, w, _, _, _, _, _ = run_gen(binary, tp, which, D, allm, seeds_per=4)
                out[which] = stats(G, w)
            return stats(S, np.ones(n)), out
        truth, g = fit_and_run(synth_days(rng, 6000))
        res["P-MEAN"] = abs(g["chain"]["mean_spell_away"] / truth["mean_spell_away"] - 1) < 0.05
        res["_mean_spell_truth_chain_frame"] = [truth["mean_spell_away"], g["chain"]["mean_spell_away"], g["frame"]["mean_spell_away"]]
        truth, g = fit_and_run(synth_days(rng, 6000, late_share=0.2))
        res["P-LATE"] = (truth["spells_8h_or_more"] - g["chain"]["spells_8h_or_more"] > 0.02
                         and abs(g["frame"]["spells_8h_or_more"] - truth["spells_8h_or_more"]) <= 0.02)
        res["_late_8h_truth_chain_frame"] = [truth["spells_8h_or_more"], g["chain"]["spells_8h_or_more"], g["frame"]["spells_8h_or_more"]]
        truth, g = fit_and_run(synth_days(rng, 6000, chain_only=True))
        res["P-CHAIN"] = (abs(g["chain"]["spells_2h_or_less"] - truth["spells_2h_or_less"]) < 0.03
                          and abs(g["chain"]["spells_8h_or_more"] - truth["spells_8h_or_more"]) < 0.03)
        res["_chain_2h_truth_chain_frame"] = [truth["spells_2h_or_less"], g["chain"]["spells_2h_or_less"], g["frame"]["spells_2h_or_less"]]
    return res


# ---------------------------------------------------------------- the run

def run(raw, binary, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    pl = plants(binary)
    log = {"plants": pl}
    print("plants:", pl)

    def stop(why):
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
        print("BRANCH (e):", why)
        return 1
    if not all(v for k, v in pl.items() if not k.startswith("_")):
        return stop("a plant failed")
    rng = np.random.default_rng(51)
    D = load(raw, [2022, 2023, 2024])
    fitm, sco = D["year"] < 2024, D["year"] == 2024
    w, S = D["w"], D["samp"].astype(int)
    samp_sh = np.array([(w[fitm, None] * (S[fitm] == s)).sum() / (24 * w[fitm].sum()) for s in range(6)])
    min_sh = (w[fitm, None] * D["mins"][fitm]).sum(0) / (1440 * w[fitm].sum())
    start_sh = np.array([(w[fitm, None] * (D["start"][fitm] == s)).sum() / (24 * w[fitm].sum()) for s in range(6)])
    rows, info = fit(D, fitm, fitm)
    tp = os.path.join(out_dir, "tables_fit.txt")
    write_tables(rows, tp)
    log["fit"] = dict(info, rows=len(rows), diaries_fit=int(fitm.sum()), diaries_score=int(sco.sum()),
                      kind_shares_fit={KINDS[k]: float(w[fitm & (D["kind"] == k)].sum() / w[fitm].sum()) for k in range(4)})
    G = {}
    for which in ("chain", "frame"):
        Sg, wg, eg, dg_, fb, kd, digest = run_gen(binary, tp, which, D, fitm, seeds_per=5)
        G[which] = {"awake": float((wg[:, None] * ((Sg != 0) & (Sg != 4))).sum() / (24 * wg.sum())), "digest": digest}
    fit_awake = float((w[fitm, None] * ((S[fitm] != 0) & (S[fitm] != 4))).sum() / (24 * w[fitm].sum()))
    g0 = {"shares_by_minutes": min_sh.tolist(), "shares_by_seeded_minute": samp_sh.tolist(), "shares_at_hour_start": start_sh.tolist(),
          "largest_difference_seeded_minute": float(np.abs(samp_sh - min_sh).max()), "travel_at_hour_start_over_minutes": float(start_sh[5] / min_sh[5]),
          "awake_share_diaries": fit_awake, "awake_share_chain": G["chain"]["awake"], "awake_share_frame": G["frame"]["awake"],
          "digests_native": {k: v["digest"] for k, v in G.items()}}
    g0["pass"] = bool(g0["largest_difference_seeded_minute"] <= 0.01 and abs(G["chain"]["awake"] - fit_awake) <= 0.01 and abs(G["frame"]["awake"] - fit_awake) <= 0.01)
    log["G0"] = g0
    print("G0:", json.dumps(g0))
    if not g0["pass"]:
        return stop("the gate failed")

    # scoring on 2024
    ws, Ss, es = w[sco], S[sco], D["emp"][sco]
    truth = stats(Ss, ws, es)
    gaps, qgap = boot_stats(Ss, ws, rng)
    res = {"diaries_2024": truth, "gap": gaps, "quartile_gap_hours": qgap}
    gen_out = {}
    for which in ("chain", "frame"):
        Sg, wg, eg, dtg, fb, kd, _ = run_gen(binary, tp, which, D, sco)
        gen_out[which] = (Sg, wg, eg, dtg)
        res[which] = stats(Sg, wg, eg)
        res[which]["fallback_share_of_days"] = float((wg * fb).sum() / wg.sum())
        hh = []
        for d in range(4):
            m, mt = dtg == d, D["dt"][sco] == d
            for st_ in range(6):
                hh.append(np.abs(hourly(Sg[m], wg[m], [st_]) - hourly(Ss[mt], ws[mt], [st_])))
        res[which]["hourly_shares_largest_miss"] = float(np.max(hh))
        res[which]["hourly_shares_mean_miss"] = float(np.mean(hh))
    log["scores"] = res

    def within(g, k):
        return abs(res[g][k] - truth[k]) <= BUDGET + gaps[k]
    s1 = {g: {k: within(g, k) for k in ("spells_2h_or_less", "spells_8h_or_more")} for g in ("chain", "frame")}
    f_ok, c_ok = all(s1["frame"].values()), all(s1["chain"].values())
    log["S1"] = {"within_budget": s1, "outcome": "met" if (f_ok and not c_ok) else ("killed" if c_ok else "neither")}

    def s2_ok(g):
        qq = all(abs(res[g]["hours_away_quartiles"][i] - truth["hours_away_quartiles"][i]) <= 1 + qgap[i] for i in range(3))
        return within(g, "home_at_9_14_18") and qq
    f2, c2 = s2_ok("frame"), s2_ok("chain")
    better = abs(res["chain"]["home_at_9_14_18"] - truth["home_at_9_14_18"]) - abs(res["frame"]["home_at_9_14_18"] - truth["home_at_9_14_18"])
    log["S2"] = {"frame_within": bool(f2), "chain_within": bool(c2), "frame_better_than_chain_by": float(better),
                 "outcome": "met" if f2 else ("killed" if (c2 and better <= gaps["home_at_9_14_18"]) else "neither")}

    # S3: within-kind tables from Monday to Thursday, Saturday's kind shares and start
    mth, sat = D["dt"] == 0, D["dt"] == 2
    rows3, _ = fit(D, fitm & mth, fitm & sat)
    for e in (0, 1):
        for d in range(4):
            rows3[key(T_KIND, e, d, 0, 0)] = rows3[key(T_KIND, e, 2, 0, 0)] if key(T_KIND, e, 2, 0, 0) in rows3 else rows3[key(T_KIND, e, d, 0, 0)]
    tp3 = os.path.join(out_dir, "tables_s3.txt")
    write_tables(rows3, tp3)
    Sg, wg, _, _, _, _, _ = run_gen(binary, tp3, "frame", D, sco & sat)
    a = hourly(Sg, wg, [1])
    b = hourly(S[sco & sat], w[sco & sat], [1])
    n3 = int((np.abs(a - b) <= BUDGET).sum())
    own, wo, _, _ = gen_out["frame"]
    msat = gen_out["frame"][3] == 2
    a0 = hourly(own[msat], wo[msat], [1])
    log["S3"] = {"hours_within_0.03": n3, "largest_miss": float(np.abs(a - b).max()), "with_tables_pooled_over_day_types_hours_within": int((np.abs(a0 - b) <= BUDGET).sum()),
                 "outcome": "met" if n3 >= 18 else ("killed" if n3 < 12 else "neither")}

    # S4: within-kind tables from 2015-2019, kind shares from 2022-2023
    Dold = load(raw, [2015, 2016, 2017, 2018, 2019])
    n_old = len(Dold["w"])
    C = {k: np.concatenate([Dold[k], D[k]]) for k in ("samp", "kind", "changes", "emp", "dt", "w")}
    old = np.arange(len(C["w"])) < n_old
    newfit = np.concatenate([np.zeros(n_old, dtype=bool), fitm])
    rows4, _ = fit(C, old, newfit)
    tp4 = os.path.join(out_dir, "tables_s4.txt")
    write_tables(rows4, tp4)
    m4 = sco & mth
    Sg, wg, _, _, _, _, _ = run_gen(binary, tp4, "frame", D, m4)
    a = hourly(Sg, wg, [0, 1])
    b = hourly(S[m4], w[m4], [0, 1])
    n4 = int((np.abs(a - b) <= BUDGET).sum())
    rows5, _ = fit(C, old, old)
    tp5 = os.path.join(out_dir, "tables_s4_old_kinds.txt")
    write_tables(rows5, tp5)
    Sg5, wg5, _, _, _, _, _ = run_gen(binary, tp5, "frame", D, m4)
    a5 = hourly(Sg5, wg5, [0, 1])
    log["S4"] = {"hours_within_0.03": n4, "largest_miss": float(np.abs(a - b).max()),
                 "with_old_kind_shares_too_hours_within": int((np.abs(a5 - b) <= BUDGET).sum()), "with_old_kind_shares_largest_miss": float(np.abs(a5 - b).max()),
                 "kind_shares_2015_2019": {KINDS[k]: float(Dold["w"][Dold["kind"] == k].sum() / Dold["w"].sum()) for k in range(4)},
                 "kind_shares_2022_2023": log["fit"]["kind_shares_fit"], "diaries_2015_2019": n_old,
                 "outcome": "met" if n4 >= 18 else ("killed" if n4 < 12 else "neither")}

    # S5
    p = hourly(Ss[D["dt"][sco] == 0], ws[D["dt"][sco] == 0], [0, 1])
    log["S5"] = {"person_at_home_Mon_Thu_by_slot": p.tolist(), "nobody_home_two_adults_if_independent": ((1 - p) ** 2).tolist(),
                 "nobody_home_ceiling": (1 - p).tolist(), "household_joint": "unresolved"}
    # employment by age band
    ag = np.where(D["age"] < 25, 0, np.where(D["age"] < 65, 1, 2))
    emp_age = [float(w[(ag == a_) & (D["emp"] == 1)].sum() / w[ag == a_].sum()) for a_ in range(3)]
    log["employed_share_by_age_band_15_24_25_64_65_plus"] = emp_age

    o1, o2 = log["S1"]["outcome"], log["S2"]["outcome"]
    log["branch"] = "(a): the frame ships" if (o1 == "met" or o2 == "met") else ("(b): the chain ships" if o1 == "killed" else "(c): neither within budget on both; the better ships marked unresolved for durations")
    allm = np.ones(len(w), dtype=bool)
    rows_all, info_all = fit(D, allm, allm)
    for a_ in range(3):
        rows_all[key(T_EMPLOYED, 0, a_, 0, 0)] = ppm([emp_age[a_], 1 - emp_age[a_]])
    write_tables(rows_all, os.path.join(out_dir, "tables_all.txt"))
    log["all_tables"] = dict(info_all, rows=len(rows_all))
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
    print(json.dumps({k: log[k] for k in ("fit", "scores", "S1", "S2", "S3", "S4", "employed_share_by_age_band_15_24_25_64_65_plus", "branch")}, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants(sys.argv[2])
        for k, v in r.items():
            print("%-10s %s" % (k, v if k.startswith("_") else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for k, v in r.items() if not k.startswith("_")) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3], sys.argv[4]))
