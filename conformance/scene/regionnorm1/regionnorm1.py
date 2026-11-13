#!/usr/bin/env python3
"""REGIONNORM-1: at what level is a dwelling's make-up shared (`REGIONNORM1_PREREG.md`).

  regionnorm1.py plants
  regionnorm1.py run RAW.csv OUT_DIR

Needs numpy. Reads FIT households alone (a named hash of the record identifier).
"""
import csv
import hashlib
import json
import os
import sys

import numpy as np

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "buildgen1"))
import buildgen1 as b1  # noqa: E402  (codes, mid-rank Spearman, total variation)

DIGEST = b1.DIGEST
SALT = "20261113"
NA = 255
TABLES = ["heating fuel by type", "wall material by era", "roof material", "type given urban type"]


def is_fit(doeid):
    return hashlib.sha256(("%s:%s" % (doeid, SALT)).encode("ascii")).digest()[0] & 1 == 0


def load(path):
    cols = b1.COLS + ["DIVISION", "state_postal", "HDD65", "CDD65"]
    with open(path, newline="") as f:
        rd = csv.reader(f)
        h = next(rd)
        ix = {c: i for i, c in enumerate(h)}
        raw = {c: [] for c in cols}
        w, n_all = [], 0
        for r in rd:
            n_all += 1
            if not is_fit(r[ix["DOEID"]]):
                continue
            for c in cols:
                raw[c].append(r[ix[c]])
            w.append(float(r[ix["NWEIGHT"]]))
    F = b1.records({c: raw[c] for c in b1.COLS})
    states = sorted(set(raw["state_postal"]))
    divs = sorted(set(raw["DIVISION"]))
    F["state"] = np.array([states.index(s) for s in raw["state_postal"]])
    F["division"] = np.array([divs.index(d) for d in raw["DIVISION"]])
    F["hdd"], F["cdd"] = np.array([float(v) for v in raw["HDD65"]]), np.array([float(v) for v in raw["CDD65"]])
    return F, np.array(w), states, divs, n_all


# ---------------------------------------------------------------- tables

def norm(v):
    s = v.sum()
    return v / s if s > 0 else v


def own_table(F, w, k, idx):
    """A group's own table k as a probability vector."""
    if k == 0:
        return norm(np.bincount(F["fuel"][idx] * 5 + F["kind"][idx], weights=w[idx], minlength=35))
    if k == 1:
        return norm(np.bincount(F["wall"][idx] * 3 + F["era"][idx], weights=w[idx], minlength=24))
    if k == 2:
        i = idx[F["roof"][idx] != NA]
        return norm(np.bincount(F["roof"][i], weights=w[i], minlength=7))
    return norm(np.bincount(F["urban"][idx] * 5 + F["kind"][idx], weights=w[idx], minlength=15))


def predict(F, w, k, pool, state_idx):
    """The pool's prediction of the state's table k. Table 3 is the pool's type given urban, at the STATE's urban shares."""
    if k != 3:
        return own_table(F, w, k, pool)
    j = np.bincount(F["urban"][pool] * 5 + F["kind"][pool], weights=w[pool], minlength=15).reshape(3, 5)
    allt = norm(j.sum(axis=0))
    us = norm(np.bincount(F["urban"][state_idx], weights=w[state_idx], minlength=3))
    out = np.zeros((3, 5))
    for u in range(3):
        out[u] = us[u] * (norm(j[u]) if j[u].sum() > 0 else allt)
    return out.ravel()


def tv(p, q):
    return 0.5 * float(np.abs(p - q).sum())


class Country:
    def __init__(self, F, w, min_households=100):
        self.F, self.w = F, w
        self.ns = int(F["state"].max()) + 1
        self.rows = [np.flatnonzero(F["state"] == s) for s in range(self.ns)]
        self.count = np.array([len(r) for r in self.rows])
        self.div = np.array([int(np.bincount(F["division"][r]).argmax()) if len(r) else -1 for r in self.rows])
        self.zone = np.array([int(np.bincount(F["zone"][r], weights=w[r], minlength=8).argmax()) if len(r) else -1 for r in self.rows])
        hd = np.array([np.average(F["hdd"][r], weights=w[r]) if len(r) else 0.0 for r in self.rows])
        cd = np.array([np.average(F["cdd"][r], weights=w[r]) if len(r) else 0.0 for r in self.rows])
        ok = self.count > 0
        self.z = np.column_stack([(hd - hd[ok].mean()) / hd[ok].std(), (cd - cd[ok].mean()) / cd[ok].std()])
        self.scored = [s for s in range(self.ns) if self.count[s] >= min_households]
        self.own = {(s, k): own_table(F, w, k, self.rows[s]) for s in range(self.ns) for k in range(4) if self.count[s] > 0}

    def donors(self, s, kind, m=None):
        others = [t for t in range(self.ns) if t != s and self.count[t] > 0]
        if kind == "division":
            return [t for t in others if self.div[t] == self.div[s]]
        if kind == "zone":
            return [t for t in others if self.zone[t] == self.zone[s]]
        if kind == "nation":
            return others
        d = sorted(others, key=lambda t: float(((self.z[t] - self.z[s]) ** 2).sum()))
        return d[:m] if m else d

    def equal_distance(self, s, pools, rng, draws):
        """Mean distance of state s's 4 tables to each pool's prediction, pools equalised in donors and households."""
        m = min(len(p) for p in pools.values())
        if m < 2:
            return None
        h = int(min(self.count[t] for p in pools.values() for t in p))
        out = {name: np.zeros(4) for name in pools}
        for _ in range(draws):
            for name, p in pools.items():
                pick = rng.choice(p, size=m, replace=False)
                idx = np.concatenate([rng.choice(self.rows[t], size=h, replace=False) for t in pick])
                for k in range(4):
                    out[name][k] += tv(self.own[(s, k)], predict(self.F, self.w, k, idx, self.rows[s]))
        return {name: (v / draws).tolist() for name, v in out.items()}, m, h


def sign_test(wins, losses):
    """One-sided binomial p for wins against losses at one half."""
    from math import comb
    n = wins + losses
    return sum(comb(n, i) for i in range(wins, n + 1)) / 2 ** n if n else 1.0


def l1(C, rival, rng, draws):
    pairs, per_state, skipped = [], {}, []
    for s in C.scored:
        dv = C.donors(s, "division")
        rv = C.donors(s, rival, m=len(dv)) if rival == "like climate" else C.donors(s, rival)
        if len(dv) < 2 or len(rv) < 2:
            skipped.append(s)
            continue
        res, m, h = C.equal_distance(s, {"division": dv, rival: rv}, rng, draws)
        wins = [res["division"][k] < res[rival][k] for k in range(4)]
        pairs += wins
        per_state[s] = {"division": [round(x, 4) for x in res["division"]], rival: [round(x, 4) for x in res[rival]], "donor_states": m, "households_per_donor": h, "tables_won_by_division": int(sum(wins))}
    share = float(np.mean(pairs)) if pairs else 0.0
    sw = sum(1 for v in per_state.values() if v["tables_won_by_division"] > 2)
    sl = sum(1 for v in per_state.values() if v["tables_won_by_division"] < 2)
    p_div, p_riv = sign_test(sw, sl), sign_test(sl, sw)
    reading = "met" if (share >= 0.60 and p_div < 0.05) else ("kill" if (share <= 0.40 and p_riv < 0.05) else "neither")
    return {"rival": rival, "pairs": len(pairs), "share_division_nearer": round(share, 4), "states_for_division": sw, "states_for_rival": sl, "states_tied": len(per_state) - sw - sl,
            "sign_test_p_for_division": round(p_div, 5), "sign_test_p_for_rival": round(p_riv, 5), "states_not_scored": skipped, "reading": reading,
            "share_by_table": {TABLES[k]: round(float(np.mean(pairs[k::4])), 4) for k in range(4)}}, per_state


def l2(C, rng, perms):
    out, rej, corr = [], 0, []
    for s in C.scored:
        mates = [t for t in range(C.ns) if C.div[t] == C.div[s] and C.count[t] > 0]
        if len(mates) < 2:
            continue
        allr = np.concatenate([C.rows[t] for t in mates])
        n = C.count[s]
        mine = np.isin(allr, C.rows[s])
        for k in range(4):
            obs = tv(own_table(C.F, C.w, k, allr[mine]), predict(C.F, C.w, k, allr[~mine], allr[mine]))
            null = np.empty(perms)
            for i in range(perms):
                p = rng.permutation(len(allr))
                a, b = allr[p[:n]], allr[p[n:]]
                null[i] = tv(own_table(C.F, C.w, k, a), predict(C.F, C.w, k, b, a))
            pv = float((1 + (null >= obs).sum()) / (perms + 1))
            out.append({"state": s, "table": TABLES[k], "distance": round(obs, 4), "permutation_mean": round(float(null.mean()), 4), "p": round(pv, 4)})
            rej += pv < 0.05
            corr.append(obs - float(null.mean()))
    share = rej / len(out) if out else 0.0
    med = float(np.median(corr)) if corr else 0.0
    reading = "met" if share >= 0.50 else ("kill" if (share < 0.15 and med < 0.03) else "neither")
    return {"pairs": len(out), "share_rejecting_at_5pct": round(share, 4), "median_distance_less_permutation_mean": round(med, 4), "reading": reading,
            "share_rejecting_by_table": {TABLES[k]: round(float(np.mean([o["p"] < 0.05 for o in out if o["table"] == TABLES[k]])), 4) for k in range(4)}}, out


def l3(C, rng, perms):
    F, w = C.F, C.w
    det = np.flatnonzero(F["kind"] == 1)
    res = {}
    for a, b in (("area", "total"), ("vint", "storeys"), ("area", "kwh")):
        def stat(lab):
            rho, wt = [], []
            for d in np.unique(lab):
                i = det[lab == d]
                if len(i) >= 30:
                    rho.append(b1.wspearman(F[a][i], F[b][i], w[i]))
                    wt.append(w[i].sum())
            rho, wt = np.array(rho), np.array(wt)
            return float(np.average((rho - np.average(rho, weights=wt)) ** 2, weights=wt)), rho
        lab = F["division"][det]
        obs, rho = stat(lab)
        null = np.array([stat(rng.permutation(lab))[0] for _ in range(perms)])
        res["%s~%s" % (a, b)] = {"by_division": [round(float(r), 3) for r in rho], "p": round(float((1 + (null >= obs).sum()) / (perms + 1)), 4)}
    rej = sum(v["p"] < 0.05 for v in res.values())
    return {"pairs": res, "rejecting": rej, "reading": "met" if rej >= 2 else "not met"}


def clusters(C, groups=10):
    S = [s for s in range(C.ns) if C.count[s] > 0]
    D = np.array([[tv(C.own[(a, 0)], C.own[(b, 0)]) for b in S] for a in S])
    cl = [[i] for i in range(len(S))]
    while len(cl) > groups:
        best, pair = None, None
        for i in range(len(cl)):
            for j in range(i):
                d = float(np.mean([D[a, b] for a in cl[i] for b in cl[j]]))
                if best is None or d < best:
                    best, pair = d, (i, j)
        i, j = pair
        cl[j] = cl[j] + cl[i]
        del cl[i]
    dist = []
    for g in cl:
        for a in g:
            mates = [S[b] for b in g if b != a]
            if mates:
                idx = np.concatenate([C.rows[t] for t in mates])
                dist.append(tv(C.own[(S[a], 0)], own_table(C.F, C.w, 0, idx)))
    return [[S[a] for a in g] for g in cl], (float(np.median(dist)) if dist else None)


def analyse(F, w, rng, draws=200, perms=999, min_households=100, with_extras=True):
    C = Country(F, w, min_households)
    out = {}
    out["L1"], ps1 = l1(C, "zone", rng, draws)
    out["L1b"], ps1b = l1(C, "like climate", rng, draws)
    out["L2"], rows2 = l2(C, rng, perms)
    if with_extras:
        out["L3"] = l3(C, rng, perms)
        # the four pools on one footing, printed
        won, med = {"division": 0, "zone": 0, "like climate": 0, "nation": 0}, {k: [] for k in ("division", "zone", "like climate", "nation")}
        per_state = {}
        for s in C.scored:
            dv = C.donors(s, "division")
            pools = {"division": dv, "zone": C.donors(s, "zone"), "like climate": C.donors(s, "like climate", m=max(len(dv), 2)), "nation": C.donors(s, "nation")}
            if min(len(p) for p in pools.values()) < 2:
                continue
            res, m, h = C.equal_distance(s, pools, rng, draws)
            per_state[s] = {k: [round(x, 4) for x in v] for k, v in res.items()}
            for k in range(4):
                best = min(res, key=lambda name: res[name][k])
                won[best] += 1
                for name in res:
                    med[name].append(res[name][k])
        tot = sum(won.values())
        out["four_pools_on_one_footing"] = {"share_of_pairs_won": {k: round(v / tot, 4) for k, v in won.items()} if tot else {}, "median_distance": {k: round(float(np.median(v)), 4) for k, v in med.items() if v}, "states": len(per_state)}
        groups, gmed = clusters(C)
        out["data_driven_groups_upper_reference"] = {"groups": groups, "median_distance_of_a_state_to_its_group_fuel_by_type": None if gmed is None else round(gmed, 4)}
        out["_per_state"] = {"L1": ps1, "L1b": ps1b, "four": per_state, "L2": rows2}
        out["_states"] = {"count": C.count.tolist(), "division": C.div.tolist(), "zone": C.zone.tolist(), "scored": C.scored}
    return out


# ---------------------------------------------------------------- plants

def synth(rng, mode):
    """40 states in 8 divisions of unequal size and 4 zones that cut across them."""
    div_sizes = [2, 3, 3, 4, 5, 6, 8, 9]
    div_of = np.repeat(np.arange(8), div_sizes)
    zone_of = rng.permutation(np.repeat(np.arange(4), [4, 8, 12, 16]))
    base = {"fuel": rng.dirichlet(np.ones(7) * 3), "kind": rng.dirichlet(np.ones(5) * 3), "wall": rng.dirichlet(np.ones(8) * 3), "era": rng.dirichlet(np.ones(3) * 5),
            "roof": rng.dirichlet(np.ones(7) * 3), "urban": rng.dirichlet(np.ones(3) * 5)}

    def shifted(p, eps):
        return (1 - eps) * p + eps * rng.dirichlet(np.ones(len(p)))
    by_div = [{k: shifted(v, 0.6) for k, v in base.items()} for _ in range(8)]
    by_zone = [{k: shifted(v, 0.6) for k, v in base.items()} for _ in range(4)]
    F = {k: [] for k in ("kind", "fuel", "wall", "era", "roof", "urban", "state", "division", "zone", "hdd", "cdd", "area", "total", "kwh", "vint", "storeys")}
    for s in range(40):
        n = int(rng.integers(150, 500)) if mode != "state" else 300
        src = base if mode in ("null", "state") else (by_div[div_of[s]] if mode == "div" else by_zone[zone_of[s]])
        if mode == "state":
            src = {k: shifted(v, 0.2) for k, v in by_div[div_of[s]].items()}      # about 0.10 from its division
        for k in ("kind", "fuel", "wall", "era", "roof", "urban"):
            F[k].append(rng.choice(len(src[k]), size=n, p=src[k]))
        F["state"].append(np.full(n, s))
        F["division"].append(np.full(n, div_of[s]))
        F["zone"].append(np.full(n, zone_of[s]))
        F["hdd"].append(np.full(n, 2000.0 * zone_of[s]) + rng.normal(0, 100, n))
        F["cdd"].append(np.full(n, 3000.0 - 700.0 * zone_of[s]) + rng.normal(0, 100, n))
        for k in ("area", "total", "kwh"):
            F[k].append(rng.random(n))
        F["vint"].append(rng.integers(0, 9, n))
        F["storeys"].append(rng.integers(1, 4, n))
    F = {k: np.concatenate(v) for k, v in F.items()}
    return F, np.ones(len(F["kind"]))


def plants():
    res = {}
    rng = np.random.default_rng(17)
    kw = dict(draws=40, perms=199, with_extras=False)
    a = analyse(*synth(rng, "null"), rng, **kw)
    res["P-NULL"] = bool(0.45 <= a["L1"]["share_division_nearer"] <= 0.55 and a["L2"]["share_rejecting_at_5pct"] <= 0.10)
    res["_P-NULL"] = [a["L1"]["share_division_nearer"], a["L1"]["reading"], a["L2"]["share_rejecting_at_5pct"]]
    a = analyse(*synth(rng, "div"), rng, **kw)
    res["P-DIV"] = a["L1"]["reading"] == "met"
    res["_P-DIV"] = [a["L1"]["share_division_nearer"], a["L1"]["sign_test_p_for_division"]]
    a = analyse(*synth(rng, "zone"), rng, **kw)
    res["P-ZONE"] = a["L1"]["reading"] == "kill"
    res["_P-ZONE"] = [a["L1"]["share_division_nearer"], a["L1"]["sign_test_p_for_rival"]]
    a = analyse(*synth(rng, "state"), rng, **kw)
    res["P-STATE"] = a["L2"]["reading"] == "met"
    res["_P-STATE"] = [a["L2"]["share_rejecting_at_5pct"], a["L2"]["median_distance_less_permutation_mean"]]
    ids = list(range(100001, 118497))
    f = np.array([is_fit(i) for i in ids])
    res["P-HASH"] = bool(0.45 <= f.mean() <= 0.55 and np.array_equal(f, np.array([is_fit(i) for i in ids])) and 0.4 < float(np.mean(f[::2])) < 0.6 and not np.array_equal(f, np.arange(len(ids)) % 2 == 0))
    res["_P-HASH_share"] = round(float(f.mean()), 4)
    return res


def run(raw, out):
    os.makedirs(out, exist_ok=True)
    log = {"plants": plants()}
    if b1.sha(raw) != DIGEST:
        log["branch"] = "(e): the file is not the one recorded"
    elif not all(v for k, v in log["plants"].items() if not k.startswith("_")):
        log["branch"] = "(e): a plant failed"
    else:
        F, w, states, divs, n_all = load(raw)
        log["households"] = {"in_the_file": n_all, "fit": int(len(w)), "fit_share": round(len(w) / n_all, 4)}
        rng = np.random.default_rng(71)
        res = analyse(F, w, rng)
        ps, st = res.pop("_per_state"), res.pop("_states")
        log.update(res)
        log["states"] = [{"state": states[s], "fit_households": st["count"][s], "division": divs[st["division"][s]], "zone": b1.ZONES[st["zone"][s]], "scored": s in st["scored"],
                          "distance_division_zone": ps["L1"].get(s), "distance_division_like_climate": ps["L1b"].get(s), "four_pools": ps["four"].get(s)} for s in range(len(states))]
        for k in ("L1", "L1b"):
            log[k]["states_not_scored"] = [states[s] for s in log[k]["states_not_scored"]]
        log["data_driven_groups_upper_reference"]["groups"] = [[states[s] for s in g] for g in log["data_driven_groups_upper_reference"]["groups"]]
        log["L2_pairs"] = [dict(o, state=states[o["state"]]) for o in ps["L2"]]
        log["branch"] = "(a): written"
    json.dump(log, open(os.path.join(out, "results.json"), "w"), indent=1)
    print(json.dumps({k: v for k, v in log.items() if k not in ("states", "L2_pairs")}, indent=1)[:7000])
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-14s %s" % (k, v if k.startswith("_") else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for k, v in r.items() if not k.startswith("_")) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3]))
