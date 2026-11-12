#!/usr/bin/env python3
"""TIES-1 (United States, 2008): the instrument of `TIES1_PREREG.md`.

  ties1.py plants
  ties1.py run DATA.csv OUT_DIR

`plants` opens no file. Single-threaded.
"""
import csv
import json
import os
import sys

import numpy as np

BANDS = ["same house", "same block or street", "under 1 mile", "1 to 5 miles", "5 to 50 miles", "50 to 100 miles",
         "100 to 500 miles", "500 to 3,000 miles", "3,000 miles or more"]
BANDS_KM = ["same house", "same block or street", "under 1.6 km", "1.6 to 8 km", "8 to 80 km", "80 to 160 km", "160 to 800 km", "800 to 4,800 km", "4,800 km or more"]
MEANS = ["in person", "mobile voice", "landline voice", "card or letter", "email", "text", "instant message", "social-network message"]
LETTERS = "abcdefgh"
WRITTEN, VOICE = [4, 5, 6, 7], [1, 2]
DAYS = {1: 365.0, 2: 365.0, 3: 156.0, 4: 52.0, 5: 12.0, 6: 4.0, 7: 0.0}
TOP = [1, 3, 10, 30]
RELS = ["partner", "child", "parent", "sibling", "other family", "non-kin"]
BOOT, SEED = 500, 20261119


def to_int(v):
    v = v.strip()
    try:
        return int(float(v))
    except ValueError:
        return None


def relation(codes):
    """Relation from the connection codes (several may be given): first family code wins."""
    for c, r in ((1, "partner"), (4, "child"), (2, "parent"), (3, "sibling"), (5, "other family")):
        if c in codes:
            return r
    return "non-kin"


def contact(answer, held, top=1):
    """(days or acts a year, missing?) for one means of one tie."""
    if not held:
        return 0.0, False
    if answer is None or answer not in DAYS:
        return 0.0, True
    return (365.0 * top if answer == 1 else DAYS[answer]), False


def build(rows):
    """rows: list of dicts (column -> string). Returns respondents and ties."""
    R, T = [], []
    for ri, r in enumerate(rows):
        g = lambda k: to_int(r.get(k, ""))
        held = {1: g("q7a") == 1, 2: True, 3: True, 0: True, 4: g("q2a") == 1 or g("q2b") == 1, 5: g("q7a") == 1 and g("q7b") == 1,
                6: g("act14a") == 1, 7: g("act87a") == 1}
        adults = {1: 1, 2: 2, 3: 3}.get(g("q30"))
        kids = {1: 1, 2: 2, 3: 3, 4: 0}.get(g("q31"))
        resp = {"w": float(r["weight"]), "samptype": r.get("samptype", "").strip(), "usr": r.get("usr", "").strip(), "density3": r.get("density3", "").strip(),
                "q19": g("q19"), "adults": adults, "kids": kids, "partnered": g("mar") in (1, 2), "held": held,
                "n_means": sum(held[m] for m in (1, 4, 5, 6, 7)), "ties": []}
        for k in range(1, 11):
            codes = [g("q11.%dm%d" % (k, m)) for m in range(1, 6)]
            codes = [c for c in codes if c is not None and c < 98]
            band = g("q15.%d" % k)
            if not codes and band is None:
                continue                                    # an empty name slot is not a tie
            ans = [g("q13%s.%d" % (LETTERS[m], k)) for m in range(8)]
            t = {"r": ri, "pos": k, "band": band if band in range(1, 10) else None, "rel": relation(codes) if codes else None,
                 "cow": 6 in codes, "grp": 7 in codes, "nbr": 8 in codes, "onl": 11 in codes, "ans": ans}
            resp["ties"].append(len(T))
            T.append(t)
        R.append(resp)
    return R, T


def tie_contact(R, T, top=1):
    """ties x 8 contact values (zero if not held) and a missing mask."""
    C = np.zeros((len(T), 8))
    M = np.zeros((len(T), 8), dtype=bool)
    for i, t in enumerate(T):
        h = R[t["r"]]["held"]
        for m in range(8):
            C[i, m], M[i, m] = contact(t["ans"][m], h[m], top)
    return C, M


def boot_ratio(num, den, strata, w, rng, B=BOOT):
    """num, den: respondents x k per-respondent weighted sums. Ratio of column sums with a
    bootstrap over respondents within strata. Returns (estimate, standard error)."""
    num, den = np.atleast_2d(num.T).T, np.atleast_2d(den.T).T
    est = num.sum(0) / den.sum(0)
    reps = np.empty((B, num.shape[1]))
    idx_by = [np.where(strata == s)[0] for s in np.unique(strata)]
    for b in range(B):
        take = np.concatenate([rng.choice(ix, len(ix)) for ix in idx_by])
        reps[b] = num[take].sum(0) / den[take].sum(0)
    return est, np.nanstd(reps, axis=0)


def per_resp(R, T, values, mask=None):
    """Sum over a respondent's ties of w * values (ties x k), optionally only ties in mask."""
    out = np.zeros((len(R), values.shape[1] if values.ndim > 1 else 1))
    v = values if values.ndim > 1 else values[:, None]
    for i, t in enumerate(T):
        if mask is None or mask[i]:
            out[t["r"]] += R[t["r"]]["w"] * v[i]
    return out


def coverage(R, T):
    """S0, and for each respondent the number of unnamed partners, other adults and children."""
    wp = np_ = wa = na = wc = nc = 0.0
    extra = np.zeros((len(R), 3))
    for ri, r in enumerate(R):
        home = [T[i] for i in r["ties"] if T[i]["band"] == 1]
        partner = sum(t["rel"] == "partner" for t in home)
        child = sum(t["rel"] == "child" for t in home)
        other = len(home) - partner - child
        if r["partnered"]:
            wp += r["w"]
            np_ += r["w"] * min(partner, 1)
        add_p = 1 if (r["partnered"] and partner == 0) else 0
        if r["adults"] is not None:
            wa += r["w"] * (r["adults"] - 1)
            na += r["w"] * min(partner + other, r["adults"] - 1)
            extra[ri, 1] = max(0, (r["adults"] - 1) - (partner + other) - add_p)
        if r["kids"] is not None:
            wc += r["w"] * r["kids"]
            nc += r["w"] * min(child, r["kids"])
            extra[ri, 2] = max(0, r["kids"] - child)
        extra[ri, 0] = add_p
    return {"partnered_who_named_a_same_house_partner": np_ / wp if wp else None,
            "named_same_house_adults_over_other_adults_at_home": na / wa if wa else None,
            "named_same_house_children_over_children_at_home": nc / wc if wc else None}, extra


# ---------------------------------------------------------------- plants

def row(**kw):
    base = {"weight": "1", "samptype": "1", "usr": "U", "density3": "1", "q19": "1", "q30": "1", "q31": "4", "mar": "6",
            "q2a": "1", "q2b": "1", "q7a": "1", "q7b": "1", "act14a": "1", "act87a": "1"}
    base.update({k: str(v) for k, v in kw.items()})
    return base


def tie(d, k, codes, band, ans):
    for m, c in enumerate(codes):
        d["q11.%dm%d" % (k, m + 1)] = str(c)
    d["q15.%d" % k] = str(band)
    for m, a in enumerate(ans):
        d["q13%s.%d" % (LETTERS[m], k)] = str(a)
    return d


def plants():
    res = {}
    d = row()
    tie(d, 1, [1, 6], 1, [1] * 8)
    tie(d, 2, [9, 8], 3, [7] * 8)
    R, T = build([d])
    res["P-KEYS"] = (T[0]["band"] == 1 and T[0]["rel"] == "partner" and T[0]["cow"]
                     and T[1]["band"] == 3 and T[1]["rel"] == "non-kin" and T[1]["nbr"])
    res["P-SCALE"] = ([contact(a, True)[0] for a in range(1, 8)] == [365, 365, 156, 52, 12, 4, 0]
                      and [contact(1, True, t)[0] for t in TOP] == [365, 1095, 3650, 10950])
    d = row(q7a=2, q7b=2)
    tie(d, 1, [9], 4, [3] * 8)
    R, T = build([d])
    C, M = tie_contact(R, T)
    res["P-NOTHELD"] = C[0, 1] == 0 and C[0, 5] == 0 and not M[0, 1] and C[0, 2] == 156 and R[0]["n_means"] == 3
    d = row()
    tie(d, 1, [9], 4, [3, 8, 3, 3, 3, 3, 3, 3])
    R, T = build([d])
    C, M = tie_contact(R, T)
    res["P-DK"] = bool(M[0, 1]) and not M[0, 2]
    d = row()
    for k in (1, 2, 3):
        tie(d, k, [9], 5, [4] * 8)
    R, T = build([d])
    res["P-EMPTY"] = len(T) == 3
    d = row(q30=2, q31=2, mar=1)
    tie(d, 1, [9], 5, [4] * 8)
    R, T = build([d])
    cov, extra = coverage(R, T)
    res["P-COVER"] = (cov["partnered_who_named_a_same_house_partner"] == 0 and cov["named_same_house_children_over_children_at_home"] == 0
                      and extra[0].tolist() == [1, 0, 2] and extra[0].sum() == 3)
    rows = []
    for i in range(200):
        d = row()
        tie(d, 1, [1], 1, [1] * 8)
        tie(d, 2, [9], 5, [3] * 8)
        tie(d, 3, [9], 6, [3] * 8)
        rows.append(d)
    R, T = build(rows)
    sh = []
    for t_ in TOP:
        C, _ = tie_contact(R, T, t_)
        el = C[:, WRITTEN + VOICE].sum(1)
        home = np.array([t["band"] == 1 for t in T])
        sh.append(el[home].sum() / el.sum())
    res["P-TOP"] = all(sh[i] < sh[i + 1] for i in range(3)) and sh[0] < 0.6 < sh[3]
    rng = np.random.default_rng(2)
    rows = []
    for i in range(300):
        d = row()
        b = int(rng.integers(2, 9))
        for k in range(1, 6):
            tie(d, k, [9], b, [4] * 8)
        rows.append(d)
    R, T = build(rows)
    far = np.array([t["band"] >= 6 for t in T], dtype=float)
    one = np.ones(len(T))
    strata = np.zeros(len(R))
    w = np.ones(len(R))
    _, se_r = boot_ratio(per_resp(R, T, far), per_resp(R, T, one), strata, w, np.random.default_rng(3), B=200)
    p = far.mean()
    se_t = np.sqrt(p * (1 - p) / len(T))
    res["P-CLUSTER"] = bool(se_r[0] > 1.3 * se_t)
    rows = []
    for i in range(100):
        d = row(weight=10 if i < 10 else 1)
        tie(d, 1, [9], 8 if i < 10 else 3, [4] * 8)
        rows.append(d)
    R, T = build(rows)
    far = np.array([t["band"] >= 6 for t in T], dtype=float)
    est = per_resp(R, T, far).sum() / per_resp(R, T, np.ones(len(T))).sum()
    res["P-WEIGHT"] = abs(est - 100 / 190) < 1e-9
    return res


# ---------------------------------------------------------------- the run

def run(path, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    pl = plants()
    log = {"plants": pl}
    print("plants:", pl)

    def stop(why):
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
        print("BRANCH (e):", why)
        return 1
    if not all(pl.values()):
        return stop("a plant failed")
    with open(path, newline="", encoding="latin-1") as f:
        rows = list(csv.DictReader(f))
    R, T = build(rows)
    nR = len(R)
    w = np.array([r["w"] for r in R])
    # G0
    desc = {}
    for c in ("form", "form2", "samptype", "usr", "rbcounty", "density3"):
        vals = [r.get(c, "").strip() for r in rows]
        u = sorted(set(vals))
        desc[c] = {"distinct": len(u), "values": u[:12], "counts": {x: vals.count(x) for x in u[:6]} if len(u) <= 6 else None}
    repro = 0
    for ri, r in enumerate(rows):
        mine = [sum(1 for i in R[ri]["ties"] if T[i]["band"] == b) for b in range(1, 10)]
        theirs = [to_int(r.get("q15cnt%d" % b, "")) or 0 for b in range(1, 10)]
        repro += mine == theirs
    has_band = np.array([t["band"] is not None for t in T])
    has_rel = np.array([t["rel"] is not None for t in T])
    ntie = np.array([len(r["ties"]) for r in R])
    q8cap = sum(1 for r in rows if (r.get("q8.5", "").strip() not in ("", "97", "98", "99")))
    g0 = {"respondents": nR, "ties": len(T), "ties_per_respondent_weighted": float((w * ntie).sum() / w.sum()),
          "respondents_with_no_ties": float((w * (ntie == 0)).sum() / w.sum()), "respondents_with_ten_ties": int((ntie == 10).sum()),
          "respondents_with_a_fifth_name_in_the_first_question": q8cap,
          "ties_with_distance": float(has_band.mean()), "ties_with_relation": float(has_rel.mean()),
          "publisher_counts_reproduced": repro, "weights_positive": bool((w > 0).all()), "columns_described": desc,
          "answered_distance_by_name_position": [float(np.mean([T[i]["band"] is not None for i in range(len(T)) if T[i]["pos"] == k] or [np.nan])) for k in range(1, 11)]}
    g0["pass"] = bool(repro == nR and g0["weights_positive"] and 1.5 <= g0["ties_per_respondent_weighted"] <= 6
                      and g0["ties_with_distance"] >= 0.90 and g0["ties_with_relation"] >= 0.90)
    log["G0"] = g0
    print("G0:", json.dumps(g0))
    if not g0["pass"]:
        return stop("G0 failed")

    rng = np.random.default_rng(SEED)
    strata = np.array([r["samptype"] for r in R])
    band = np.array([t["band"] or 0 for t in T])
    rel = np.array([t["rel"] or "" for t in T])
    ok = (band > 0) & (rel != "")
    kin = np.isin(rel, ["partner", "child", "parent", "sibling", "other family"])
    home = band == 1
    out_house = ok & ~home
    one = np.ones(len(T))
    rw = np.array([R[t["r"]]["w"] for t in T])

    def share(mask, base):
        e, s = boot_ratio(per_resp(R, T, mask.astype(float)), per_resp(R, T, base.astype(float)), strata, w, rng)
        return float(e[0]), float(s[0])

    # S0
    cov, extra = coverage(R, T)
    log["S0"] = cov
    log["S0"]["note"] = "households of three or more adults or children are counted as three, so coverage is overstated; a grown child at home counts as a child"
    log["S0"]["unnamed_coresidents_per_respondent_weighted"] = {"partners": float((w * extra[:, 0]).sum() / w.sum()),
                                                                 "other_adults": float((w * extra[:, 1]).sum() / w.sum()), "children": float((w * extra[:, 2]).sum() / w.sum())}

    # T1
    t1 = {"all_ties_by_band": {}, "by_relation": {}, "flags": {}}
    for b in range(1, 10):
        t1["all_ties_by_band"][BANDS[b - 1]] = list(share(ok & (band == b), ok))
    for r_ in RELS:
        m = ok & (rel == r_)
        t1["by_relation"][r_] = {"share_of_ties": share(m, ok)[0], "ties": int(m.sum()),
                                 "by_band": [float((rw * (m & (band == b))).sum() / max((rw * m).sum(), 1e-12)) for b in range(1, 10)]}
    for nm, key in (("co-worker", "cow"), ("neighbour", "nbr"), ("group member", "grp"), ("online", "onl")):
        fl = np.array([t[key] for t in T]) & ok
        t1["flags"][nm] = {"share_of_ties": share(fl, ok)[0], "ties": int(fl.sum()),
                           "by_band": [float((rw * (fl & (band == b))).sum() / max((rw * fl).sum(), 1e-12)) for b in range(1, 10)]}
    t1["per_respondent_mean_ties"] = g0["ties_per_respondent_weighted"]
    log["T1"] = t1

    # contact
    held_all = np.array([all(R[t["r"]]["held"][m] for m in (1, 4, 5)) for t in T])
    C = {t_: tie_contact(R, T, t_)[0] for t_ in TOP}
    miss = tie_contact(R, T)[1]
    log["contact_missing_share_by_means"] = {MEANS[m]: float(miss[ok, m].mean()) for m in range(8)}
    t2 = {}
    for r_ in RELS + ["all"]:
        m_ = ok & ((rel == r_) if r_ != "all" else True)
        t2[r_] = {}
        for b in range(1, 10):
            mm = m_ & (band == b)
            if mm.sum() < 20:
                t2[r_][BANDS[b - 1]] = None
                continue
            t2[r_][BANDS[b - 1]] = {"ties": int(mm.sum()), "days_by_means": [float((rw[mm] * C[1][mm, k]).sum() / rw[mm].sum()) for k in range(8)]}
    log["T2"] = t2

    def bridge(cols, top, mask):
        v = C[top][:, cols].sum(1)
        tot = (rw * v * mask).sum()
        tt = (rw * mask).sum()
        return [float((rw * v * (mask & (band == b))).sum() / tot) for b in range(1, 10)], [float((rw * (mask & (band == b))).sum() / tt) for b in range(1, 10)]
    t3 = {}
    for pop, pm in (("each respondent, a means not held is zero", ok), ("respondents who hold mobile, text and email", ok & held_all)):
        t3[pop] = {"ties": bridge(WRITTEN, 1, pm)[1]}
        for nm, cols in (("written", WRITTEN), ("voice", VOICE), ("electronic", WRITTEN + VOICE), ("in person", [0])):
            t3[pop][nm] = {("days" if t_ == 1 else "acts, top answer %d a day" % t_): bridge(cols, t_, pm)[0] for t_ in TOP}
    log["T3"] = t3

    # S1
    near = out_house & (band <= 4)
    s1 = {}
    met = True
    for t_ in TOP:
        v = C[t_][:, WRITTEN].sum(1)
        num = np.concatenate([per_resp(R, T, v * near), per_resp(R, T, near.astype(float))], 1)
        den = np.concatenate([per_resp(R, T, v * out_house), per_resp(R, T, out_house.astype(float))], 1)
        est = num.sum(0) / den.sum(0)
        reps = []
        idx_by = [np.where(strata == s)[0] for s in np.unique(strata)]
        for b in range(BOOT):
            take = np.concatenate([rng.choice(ix, len(ix)) for ix in idx_by])
            e = num[take].sum(0) / den[take].sum(0)
            reps.append(e[0] - e[1])
        d, gap = float(est[0] - est[1]), 2 * float(np.std(reps))
        s1["days" if t_ == 1 else "top answer %d a day" % t_] = {"share_of_written_contact_within_5_miles": float(est[0]), "share_of_ties_within_5_miles": float(est[1]), "difference": d, "gap": gap}
        met &= (d - gap) >= 0.05
    d0 = s1["days"]
    s1["outcome"] = "met" if met else ("killed" if d0["difference"] + d0["gap"] < 0 else "neither")
    log["S1"] = s1

    # S2 with floor and ceiling
    relgrp = np.where(rel == "partner", 0, np.where(rel == "child", 2, 1))
    s2 = {}
    for t_ in TOP:
        el = C[t_][:, WRITTEN + VOICE].sum(1)
        mean_home = [float((rw * el * (ok & home & (relgrp == g))).sum() / max((rw * (ok & home & (relgrp == g))).sum(), 1e-12)) for g in range(3)]
        num = per_resp(R, T, el * (ok & home))
        den = per_resp(R, T, el * ok)
        add = (w[:, None] * extra * np.array(mean_home)[None, :]).sum(1, keepdims=True)
        fl, fl_se = boot_ratio(num, den, strata, w, rng)
        ce, ce_se = boot_ratio(num + add, den + add, strata, w, rng)
        s2["days" if t_ == 1 else "top answer %d a day" % t_] = {"floor_as_named": float(fl[0]), "floor_gap": 2 * float(fl_se[0]), "ceiling_with_unnamed_coresidents": float(ce[0]), "ceiling_gap": 2 * float(ce_se[0])}
    a, z = s2["days"], s2["top answer 30 a day"]
    s2["outcome"] = "met" if a["floor_as_named"] - a["floor_gap"] >= 0.15 else ("killed" if z["ceiling_with_unnamed_coresidents"] + z["ceiling_gap"] < 0.15 else "neither")
    s2["share_of_ties_same_house"] = share(ok & home, ok)[0]
    log["S2"] = s2

    # S3
    mid = ok & held_all & (band == 5)
    s3 = {}
    for nm, cols in (("electronic", WRITTEN + VOICE), ("written", WRITTEN), ("voice", VOICE)):
        v = C[1][:, cols].sum(1)
        num = np.concatenate([per_resp(R, T, v * (mid & kin)), per_resp(R, T, v * (mid & ~kin))], 1)
        den = np.concatenate([per_resp(R, T, (mid & kin).astype(float)), per_resp(R, T, (mid & ~kin).astype(float))], 1)
        est = num.sum(0) / den.sum(0)
        reps = []
        idx_by = [np.where(strata == s)[0] for s in np.unique(strata)]
        for b in range(BOOT):
            take = np.concatenate([rng.choice(ix, len(ix)) for ix in idx_by])
            e = num[take].sum(0) / den[take].sum(0)
            reps.append((e[0] - e[1]) / ((e[0] + e[1]) / 2))
        rd = float((est[0] - est[1]) / ((est[0] + est[1]) / 2))
        s3[nm] = {"kin_days": float(est[0]), "non_kin_days": float(est[1]), "relative_difference": rd, "gap": 2 * float(np.nanstd(reps))}
    e = s3["electronic"]
    s3["ties"] = {"kin": int((mid & kin).sum()), "non_kin": int((mid & ~kin).sum())}
    s3["outcome"] = "met" if abs(e["relative_difference"]) - e["gap"] >= 0.20 else ("killed" if abs(e["relative_difference"]) + e["gap"] < 0.10 else "neither")
    log["S3"] = s3

    # S4
    far50 = out_house & (band >= 6)
    s4 = {"by_community_type": {}, "by_density3": {}}
    usr = np.array([R[t["r"]]["usr"] for t in T])
    d3 = np.array([R[t["r"]]["density3"] for t in T])
    for u in sorted(set(usr)):
        if u:
            s4["by_community_type"][u] = list(share(far50 & (usr == u), out_house & (usr == u)))
    for u in sorted(set(d3)):
        if u:
            s4["by_density3"][u] = list(share(far50 & (d3 == u), out_house & (d3 == u)))
    log["S4"] = s4

    # S6, S8
    w50 = out_house & (band <= 5)
    s6e, s6s = share(out_house & (band <= 3), w50)
    blk, _ = share(out_house & (band == 2), w50)
    log["S6"] = {"share_of_ties_within_50_miles_that_are_on_the_block_or_under_a_mile": s6e, "se": s6s, "on_the_same_block_alone": blk,
                 "a_1_over_distance_law_on_uniform_population": 1 / 50, "ties_0_bracket_for_within_1_km_of_own_district_ties": [0.004, 0.48]}
    s8e, s8s = share(w50, out_house)
    log["S8"] = {"close_ties_outside_the_house_within_50_miles": s8e, "se": s8s, "all_friendship_links_within_50_miles_published_us": 0.554}

    # S7
    nm_ = np.array([R[t["r"]]["n_means"] for t in T])
    s7 = {}
    for lab, m_ in (("four or more means", nm_ >= 4), ("two or fewer means", nm_ <= 2)):
        v = C[1][:, WRITTEN + VOICE].sum(1)
        a_ = (rw * v * (near & m_)).sum() / max((rw * v * (out_house & m_)).sum(), 1e-12)
        b_ = (rw * (near & m_)).sum() / max((rw * (out_house & m_)).sum(), 1e-12)
        s7[lab] = {"share_of_electronic_contact_within_5_miles": float(a_), "share_of_ties_within_5_miles": float(b_), "difference": float(a_ - b_), "ties": int((out_house & m_).sum())}
    log["S7"] = s7

    # T4
    t4 = {}
    for t_ in TOP:
        el = C[t_][:, WRITTEN + VOICE].sum(1)
        hk = ok & (home | kin)
        mean_home = [float((rw * el * (ok & home & (relgrp == g))).sum() / max((rw * (ok & home & (relgrp == g))).sum(), 1e-12)) for g in range(3)]
        add = float((w[:, None] * extra * np.array(mean_home)[None, :]).sum())
        tot = float((rw * el * ok).sum())
        t4["days" if t_ == 1 else "top answer %d a day" % t_] = {"household_and_kin_as_named": float((rw * el * hk).sum() / tot),
                                                                "household_and_kin_with_unnamed_coresidents": float(((rw * el * hk).sum() + add) / (tot + add)),
                                                                "kin_elsewhere_within_100_miles": float((rw * el * (ok & kin & ~home & (band <= 6))).sum() / max((rw * el * (ok & kin & ~home)).sum(), 1e-12)),
                                                                "kin_elsewhere_within_500_miles": float((rw * el * (ok & kin & ~home & (band <= 7))).sum() / max((rw * el * (ok & kin & ~home)).sum(), 1e-12))}
    t4["kin_ties_elsewhere_within_100_miles"] = share(ok & kin & ~home & (band <= 6), ok & kin & ~home)[0]
    t4["kin_ties_elsewhere_within_500_miles"] = share(ok & kin & ~home & (band <= 7), ok & kin & ~home)[0]
    log["T4"] = t4
    hs = {}
    q19 = np.array([R[t["r"]]["q19"] or 0 for t in T])
    for lab, v_ in (("detached house", 1), ("apartment", 3)):
        hs[lab] = list(share(out_house & (band <= 3) & (q19 == v_), out_house & (q19 == v_)))
    log["S5_building"] = hs
    log["branch"] = "(a)"
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1, default=lambda o: o.item())
    pub = {"year": 2008, "country": "United States", "bands_miles": BANDS, "bands_km": BANDS_KM, "means": MEANS, "relations": RELS,
           "who_is_missing": log["S0"],
           "T1_ties": dict(t1, tag="may last: where closest ties live, by relation"),
           "T2_days_with_contact_a_year": dict(t2, tag="a 2008 price: do not carry the levels or the mix of means forward"),
           "T3_bridge": dict(t3, tag="a 2008 price; shares by band in the order of bands_miles"),
           "T4_two_bounds": dict(t4, tag="not a score against any twin figure"),
           "household_share_of_electronic_contact": s2, "written_contact_nearer_than_ties": s1, "kin_against_non_kin_at_5_to_50_miles": s3,
           "ties_beyond_50_miles_by_place": s4, "below_a_district": log["S6"], "close_against_all": log["S8"], "by_adoption": s7, "by_building": hs,
           "not_in_this_instrument": ["school ties", "public or broadcast follows", "a person's own devices", "weak ties", "anyone under 18 as a respondent"]}
    json.dump(pub, open(os.path.join(out_dir, "ties_by_kind_table.json"), "w"), indent=1, default=lambda o: o.item())
    print(json.dumps({k: log[k] for k in ("S0", "S1", "S2", "S3", "S4", "S6", "S7", "S8", "T4", "S5_building", "contact_missing_share_by_means")}, indent=1))
    print("T1 bands", {k: round(v[0], 3) for k, v in t1["all_ties_by_band"].items()})
    for r_ in RELS:
        print("   %-13s %.3f of ties (%d)" % (r_, t1["by_relation"][r_]["share_of_ties"], t1["by_relation"][r_]["ties"]), [round(x, 2) for x in t1["by_relation"][r_]["by_band"]])
    for nm, v in t1["flags"].items():
        print("   flag %-12s %.3f (%d)" % (nm, v["share_of_ties"], v["ties"]), [round(x, 2) for x in v["by_band"]])
    for b in BANDS:
        x = t2["all"][b]
        if x:
            print("T2 %-22s n=%5d" % (b, x["ties"]), [round(d) for d in x["days_by_means"]])
    for pop in t3:
        print("T3", pop)
        print("    ties      ", [round(x, 3) for x in t3[pop]["ties"]])
        for nm in ("written", "voice", "electronic", "in person"):
            print("    %-10s" % nm, [round(x, 3) for x in t3[pop][nm]["days"]])
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-11s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3]))
