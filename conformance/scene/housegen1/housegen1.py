#!/usr/bin/env python3
"""HOUSEGEN-1: the scoring of `HOUSEGEN1_PREREG.md`.

  housegen1.py plants BIN
  housegen1.py run RAW_DIR BIN TWIN_HIST.json OUT_DIR

BIN is the built example `housegen_draw` of the crate `holon-scene`: every household this
script scores was drawn by the shipped function. `plants` opens no census file.
Single-threaded.
"""
import hashlib
import json
import os
import subprocess
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "household0_de"))
sys.path.insert(0, os.path.join(HERE, "..", "household0"))
from household0_de import (read_grid, to_float, join, key, num, split_gap, qty, decide,  # noqa: E402
                           HH_TOTAL, SIZE_COLS, BAND_SEED, NEFF_MIN, P1_BUDGET, MEAN_BUDGET_FRACTION)
from household0 import classify, neff, CLASS_NAMES, gen_from_counts  # noqa: E402

SIZES = [1, 2, 3, 4, 5, 7]
UNIT_SIZES = [10, 20, 40, 80, 160, 320]
TILE_FLOOR = 80


def gen(binary, shares_ppm, sizes, seeds, people, plant=False):
    """Run the shipped draw. Returns (households per unit, ones per unit, pooled histogram)."""
    inp = "".join("%d %d\n" % (s, p) for s, p in zip(seeds, people))
    cmd = [binary, ",".join(str(int(x)) for x in shares_ppm), ",".join(str(x) for x in sizes)]
    if plant:
        cmd.append("truncating")
    out = subprocess.run(cmd, input=inp, capture_output=True, text=True, check=True).stdout.split("\n")
    n = len(seeds)
    a = np.array([l.split() for l in out[:n]], dtype=np.int64).reshape(n, 2) if n else np.zeros((0, 2), dtype=np.int64)
    hist = np.array(out[n].split()[1:], dtype=np.int64)
    return a[:, 0], a[:, 1], hist


def row_mean(shares_ppm, top):
    s = np.asarray(shares_ppm, dtype=float) / np.sum(shares_ppm)
    return float((s * np.array([1, 2, 3, 4, 5, top])).sum())


def between(all_ppm, a_ppm, b_ppm):
    lo, hi = np.minimum(a_ppm, b_ppm) - 1, np.maximum(a_ppm, b_ppm) + 1
    return bool(((np.asarray(all_ppm) >= lo) & (np.asarray(all_ppm) <= hi)).all())


def draw_against_row(binary, shares, people, n=20000, plant=False, seed0=0):
    hh, ones, hist = gen(binary, shares, SIZES, range(seed0, seed0 + n), [people] * n, plant)
    m, p1 = people * n / hh.sum(), ones.sum() / hh.sum()
    rm, rp = row_mean(shares, 7), shares[0] / np.sum(shares)
    return {"people": people, "mean": float(m), "row_mean": rm, "mean_miss_fraction": float(m / rm - 1),
            "p1": float(p1), "row_p1": float(rp), "p1_miss": float(p1 - rp),
            "inside": bool(abs(m / rm - 1) <= MEAN_BUDGET_FRACTION and abs(p1 - rp) <= P1_BUDGET), "hist": hist.tolist()}


def plants(binary):
    res = {}
    made_up = [240000, 330000, 160000, 150000, 80000, 40000]
    res["P-BIAS"] = not draw_against_row(binary, made_up, 8, plant=True)["inside"]
    # P-SHIFTED: a row with the one-person share raised 0.10, scored against the unshifted truth
    truth = np.array([360000, 340000, 140000, 110000, 35000, 15000])
    shifted = truth.copy()
    shifted[0] += 100000
    shifted[1] -= 100000
    hh, ones, _ = gen(binary, shifted, SIZES, range(5000), [300] * 5000)
    v, _ = decide(ones.sum() / hh.sum() - 0.36, 0.008, P1_BUDGET)
    res["P-SHIFTED"] = v == "refused"
    hh, ones, _ = gen(binary, truth, SIZES, range(5000), [300] * 5000)
    res["P-SHIFTED"] &= decide(ones.sum() / hh.sum() - 0.36, 0.008, P1_BUDGET)[0] == "certified"
    res["P-BETWEEN"] = between([5, 5], [4, 4], [6, 6]) and not between([9, 5], [4, 4], [6, 6])
    res["P-ZERO"] = gen(binary, truth, SIZES, [1, 2], [0, 0])[0].tolist() == [0, 0]
    return res


def run(raw, binary, twin_path, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    log = {"plants": plants(binary)}
    print("plants:", log["plants"])

    def stop(why):
        log["branch"] = "(e): " + why
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
        print("BRANCH (e):", why)
        return 1
    if not all(log["plants"].values()):
        return stop("a plant failed")
    twin = gen_from_counts(json.loads(open(twin_path, "rb").read())["final"]["counts_by_size"])
    twin = {"mean": twin["mean"], "p1": twin["p1"]}
    log["binary_sha256"] = hashlib.sha256(open(binary, "rb").read()).hexdigest()

    Z = lambda n: os.path.join(raw, n)
    _, x, y, v = read_grid(Z("Zensus2022_Groesse_des_privaten_Haushalts_in_Gitterzellen.zip"), "1km", [HH_TOTAL] + SIZE_COLS)
    H = to_float(v)
    order = np.argsort(key(x, y))
    x, y, H = x[order], y[order], H[order]
    K = key(x, y)
    zm = Z("Durchschnittliche_Haushaltsgroesse_in_Gitterzellen.zip")
    hm, _, _, _ = read_grid(zm, "1km")
    mean_col = [h for h in hm if h.lower().startswith("durchschn")][0]
    _, xm, ym, vm = read_grid(zm, "1km", [mean_col])
    zp = Z("Zensus2022_Bevoelkerungszahl.zip")
    hp, _, _, _ = read_grid(zp, "1km")
    pop_col = [h for h in hp if h.lower().startswith("einwohner")][0]
    _, xp, yp, vp = read_grid(zp, "1km", [pop_col])
    mean_al, has_mean = join(K, key(xm, ym), np.array([num(r[0]) for r in vm])[:, None])
    mean_al = mean_al[:, 0]
    pop_al, has_pop = join(K, key(xp, yp), to_float(vp))
    pop_al = np.nan_to_num(pop_al[:, 0], nan=0.0)
    has_mean &= mean_al > 0
    parts, total = H[:, 1:].sum(1), H[:, 0]
    persons = np.where(has_mean, mean_al * total, 0.0)
    wmean = np.where(has_mean, total, 0.0)
    cls = np.where(has_pop, classify(pop_al), -2)
    sw_x, sw_y = x - 500, y - 500
    block = (sw_x // 50000) * 1000 + (sw_y // 50000)
    bx = np.minimum(((sw_x - sw_x.min()) * 3) // (sw_x.max() - sw_x.min() + 1), 2)
    by = np.minimum(((sw_y - sw_y.min()) * 2) // (sw_y.max() - sw_y.min() + 1), 1)
    band = by * 3 + bx
    perm = np.random.default_rng(BAND_SEED).permutation(6)
    fit = np.isin(band, perm[:3]) & (cls >= 0)
    sco = np.isin(band, perm[3:]) & (cls >= 0)
    log["bands"] = {"FIT": sorted(perm[:3].tolist()), "SCORE": sorted(perm[3:].tolist())}
    cols = np.stack([parts, persons, H[:, 1], wmean], 1)
    index = np.arange(len(K))

    def ct_of(mask):
        ids, inv = np.unique(block[mask], return_inverse=True)
        out = np.zeros((len(ids), 4))
        np.add.at(out, inv, cols[mask])
        return out

    def ppm(mask):
        h = H[mask][:, 1:].sum(0)
        p = np.floor(1e6 * h / h.sum()).astype(np.int64)
        p[np.argmax(p)] += 1_000_000 - p.sum()
        return p

    rows, gate = {}, True
    for c in range(5):
        m = cls == c
        rows[CLASS_NAMES[c]] = {"FIT": ppm(m & fit).tolist(), "SCORE": ppm(m & sco).tolist(), "ALL": ppm(m).tolist(),
                                "households_ALL": float(parts[m].sum()),
                                "persons_per_household_ALL": float(persons[m].sum() / wmean[m].sum())}
        r = rows[CLASS_NAMES[c]]
        r["between"] = between(r["ALL"], r["FIT"], r["SCORE"])
        gate &= r["between"]
    log["rows"] = rows
    log["G0"] = {"shipped_rows_between_halves": bool(gate)}
    print("G0:", log["G0"])
    if not gate:
        return stop("a shipped share lies outside its two halves'")

    def judge(g, ref, gap):
        out = {}
        for q in ("mean", "p1"):
            bud = MEAN_BUDGET_FRACTION * ref["mean"] if q == "mean" else P1_BUDGET
            vd, why = decide(g[q] - ref[q], gap[q], bud)
            out[q] = {"generator": g[q], "reference": ref[q], "difference": g[q] - ref[q], "budget": bud, "gap_95": gap[q], "verdict": vd}
        return out

    worst = {"certified": 0, "unresolved": 1, "refused": 2}
    S1, S2, S3, S4, S5, S6 = {}, {}, {}, {}, {}, {}
    resolved = []
    for c in range(5):
        name = CLASS_NAMES[c]
        ms = sco & (cls == c)
        if ms.sum() == 0:
            continue
        ct = ct_of(ms)
        ne = neff(ct[:, 0])
        ref = qty(ct)
        fitrow = np.array(rows[name]["FIT"])
        e = {"n_eff_score": ne, "tiles": int(ms.sum()), "reference": ref}
        if ne >= NEFF_MIN:
            resolved.append(name)
            gap = split_gap(ct)
        else:
            gap = {"mean": float("nan"), "p1": float("nan")}
        e["gap_95"] = gap
        people = np.rint(pop_al[ms]).astype(np.int64)
        part_i, part_ii = {}, {}
        for top in (6, 7, 8):
            sizes = [1, 2, 3, 4, 5, top]
            g_i = {"mean": row_mean(fitrow, top), "p1": float(fitrow[0] / fitrow.sum())}
            hh, ones, hist = gen(binary, fitrow, sizes, index[ms], people)
            g_ii = {"mean": float(people.sum() / hh.sum()), "p1": float(ones.sum() / hh.sum())}
            if ne >= NEFF_MIN:
                part_i[str(top)] = judge(g_i, ref, gap)
                part_ii[str(top)] = judge(g_ii, ref, gap)
            else:
                part_i[str(top)] = {"generator": g_i, "verdict": "unresolved: effective blocks under 30"}
                part_ii[str(top)] = {"generator": g_ii, "verdict": "unresolved: effective blocks under 30"}
            if top == 7:
                hh7, ones7 = hh, ones
        e["part_i_row_against_heldout"], e["part_ii_generator_against_heldout"] = part_i, part_ii
        if ne >= NEFF_MIN:
            fin = {}
            for q in ("mean", "p1"):
                tops = ("6", "7", "8") if q == "mean" else ("7",)
                vs = [part[t][q]["verdict"] for part in (part_i, part_ii) for t in tops]
                fin[q] = max(vs, key=lambda z: worst[z])
            e["verdict"] = fin
            S4[name] = judge(twin, ref, gap)
        S1[name] = e

        # S5 and S6: tiles of at least 80 households
        big = total[ms] >= TILE_FLOOR
        n_c, p_c = total[ms][big], (H[ms][:, 1] / np.maximum(parts[ms], 1))[big]
        gh, go = hh7[big].astype(float), ones7[big].astype(float)
        gp = go / np.maximum(gh, 1)
        tol = P1_BUDGET + 2 * np.sqrt(np.clip(p_c * (1 - p_c), 0, None) / n_c)
        ok_n = np.abs(gh - n_c) <= 0.05 * n_c
        ok_p = np.abs(gp - p_c) <= tol
        lh = np.rint(people[big] / 2.5)
        w = n_c
        S5[name] = {"tiles": int(big.sum()), "households": float(w.sum()),
                    "count_within_5pct": float((w * ok_n).sum() / w.sum()) if big.sum() else None,
                    "p1_within": float((w * ok_p).sum() / w.sum()) if big.sum() else None,
                    "both": float((w * (ok_n & ok_p)).sum() / w.sum()) if big.sum() else None,
                    "legacy_count_within_5pct": float((w * (np.abs(lh - n_c) <= 0.05 * n_c)).sum() / w.sum()) if big.sum() else None,
                    "legacy_p1_within": float((w * (np.abs(twin["p1"] - p_c) <= tol)).sum() / w.sum()) if big.sum() else None}
        if big.sum() > 10:
            sd_c, sd_g = float(np.std(p_c)), float(np.std(gp))
            S6[name] = {"tiles": int(big.sum()), "sd_census": sd_c, "sd_generator": sd_g, "ratio": sd_c / sd_g if sd_g else None,
                        "sd_household_count_ratio": float(np.std(n_c / np.maximum(people[big], 1)) / max(np.std(gh / np.maximum(people[big], 1)), 1e-12))}

        # S2 and S3: the draw against its own row
        S2[name] = [{k: v for k, v in draw_against_row(binary, fitrow, p).items() if k != "hist"} for p in UNIT_SIZES]
        d = draw_against_row(binary, fitrow, 320)
        hist = np.array(d["hist"], dtype=float)
        gsh = np.array([hist[1], hist[2], hist[3], hist[4], hist[5], hist[7]]) / hist.sum()
        other = float(1 - gsh.sum())
        err = np.abs(gsh - fitrow / fitrow.sum())
        S3[name] = {"largest_share_error": float(err.max()), "share_at_other_sizes": other, "met": bool(err.max() <= 0.01)}

    floor = None
    for i, p in enumerate(UNIT_SIZES):
        if all(all(r["inside"] for r in S2[n][i:]) for n in S2):
            floor = p
            break
    s2_out = {"unit_floor": floor,
              "expected_met": bool(all(r["inside"] for n in S2 for r in S2[n] if r["people"] >= 40)),
              "killed": bool(any(not r["inside"] for n in S2 for r in S2[n] if r["people"] == 160))}
    s1_out = {"resolved": resolved}
    verds = [S1[n]["verdict"][q] for n in resolved for q in ("mean", "p1")]
    s1_out["outcome"] = "met" if verds and all(v == "certified" for v in verds) else ("killed" if any(v == "refused" for v in verds) else "neither")
    r6 = [S6[n]["ratio"] for n in resolved if n in S6]
    s6_out = "met" if r6 and all(r > 1.5 for r in r6) else ("killed" if r6 and all(r < 1.2 for r in r6) else "neither")
    log.update(S1=S1, S1_summary=s1_out, S2=S2, S2_summary=s2_out, S3=S3, S4=S4, S5=S5, S6=S6, S6_outcome=s6_out)
    log["branch"] = "(a)" if s1_out["outcome"] == "met" else "(b)" if s1_out["outcome"] == "killed" else "neither: see S1"
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
    print(json.dumps({k: log[k] for k in ("bands", "rows", "S1_summary", "S2_summary", "S3", "S4", "S5", "S6", "S6_outcome", "branch")}, indent=1))
    print("S1", json.dumps(S1, indent=1))
    print("S2", json.dumps(S2, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants(sys.argv[2])
        for k, v in r.items():
            print("%-10s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(*sys.argv[2:6]))
