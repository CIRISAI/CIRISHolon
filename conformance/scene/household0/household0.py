#!/usr/bin/env python3
"""HOUSEHOLD-0 (United States): the instrument of `HOUSEHOLD0_PREREG.md`.

  household0.py plants
  household0.py run RAW_DIR TWIN_HIST.json OUT_DIR

`plants` runs every plant on synthetic carriers and opens no survey file.
`run` runs the plants first, records the twin's histogram and its digest, and
only then opens a table. Single-threaded. Raw files are never written anywhere.
"""
import hashlib
import json
import os
import struct
import sys
import zipfile

import numpy as np

BOUNDS = [0.0, 250.0, 1500.0, 6000.0, 12000.0]     # people per km2, twin/globe classes.json
FLOOR = 1.0
CLASS_NAMES = ["village", "suburban", "terraced", "apartment blocks", "dense fabric"]
BUDGET = {"mean": 0.12, "p1": 0.03}
NEFF_MIN = 30.0
UNIT_FLOOR = 20000.0
TOP = 8.0
SUPPORT = np.array([0, 1, 2, 3, 4, 5, 6, TOP])       # size 0 kept for a generator that emits it
SPLIT_SEED = 20261112
STATE_SEED = 20261111
STRUCT = ["1, detached or attached", "2 to 4", "5 or more", "mobile home", "boat, RV, van, etc."]


# ---------------------------------------------------------------- core

def classify(density):
    """Class 0..4 by lower bound; -1 under the floor."""
    d = np.asarray(density, dtype=float)
    c = np.searchsorted(np.array(BOUNDS), d, side="right") - 1
    c[d < FLOOR] = -1
    return c


def neff(w):
    w = np.asarray(w, dtype=float)
    s2 = (w * w).sum()
    return float(w.sum() ** 2 / s2) if s2 > 0 else 0.0


def by_county(county, cols):
    """Sum columns over units of one county. Returns (ids, summed 2-D array)."""
    ids, inv = np.unique(county, return_inverse=True)
    out = np.zeros((len(ids), cols.shape[1]))
    np.add.at(out, inv, cols)
    return ids, out


def quantities(hh, pop, h1):
    return {"mean": float(pop / hh), "p1": float(h1 / hh)}


def split_gap(ct, seed=SPLIT_SEED, n=1000):
    """ct: counties x (hh, pop, h1). 95th percentile of the gap between two
    random halves of the counties, in each quantity."""
    rng = np.random.default_rng(seed)
    k = ct.shape[0]
    gm, gp = [], []
    for _ in range(n):
        a = np.zeros(k, dtype=bool)
        a[rng.permutation(k)[: k // 2]] = True
        A, B = ct[a].sum(0), ct[~a].sum(0)
        if A[0] <= 0 or B[0] <= 0:
            continue
        gm.append(abs(A[1] / A[0] - B[1] / B[0]))
        gp.append(abs(A[2] / A[0] - B[2] / B[0]))
    return {"mean": float(np.percentile(gm, 95)), "p1": float(np.percentile(gp, 95))}


def verdict(gen, ct, extra_neff=None):
    """Per consumer: floor (count, then error), then match. `gen` holds mean and
    p1; `ct` is the reference's counties x (hh, pop, h1)."""
    tot = ct.sum(0)
    ne = neff(ct[:, 0])
    out = {"n_eff": ne, "counties": int((ct[:, 0] > 0).sum()), "households": float(tot[0])}
    if ne < NEFF_MIN or (extra_neff is not None and extra_neff < NEFF_MIN):
        for q in BUDGET:
            out[q] = {"verdict": "unresolved", "reason": "effective counties under 30"}
        return out
    ref = quantities(*tot)
    gap = split_gap(ct)
    for q in BUDGET:
        r = {"generator": gen[q], "reference": ref[q], "difference": gen[q] - ref[q],
             "budget": BUDGET[q], "split_half_gap_95": gap[q]}
        if gap[q] > BUDGET[q] / 2:
            r.update(verdict="unresolved", reason="reference's own split-half gap exceeds half the budget")
        elif abs(gen[q] - ref[q]) <= BUDGET[q]:
            r.update(verdict="certified", reason="within budget")
        else:
            r.update(verdict="refused", reason="outside budget",
                     witness="one-person share" if q == "p1" else "persons per household")
        out[q] = r
    return out


def w1(pa, pb):
    """Exact W1 on SUPPORT between two share vectors of length 8 (sizes 0..6, 7+)."""
    ca, cb = np.cumsum(pa)[:-1], np.cumsum(pb)[:-1]
    return float((np.abs(ca - cb) * np.diff(SUPPORT)).sum())


def shares8(h7):
    """Reference shares (sizes 1..6, 7+) as a vector on SUPPORT."""
    s = np.concatenate([[0.0], np.asarray(h7, dtype=float)])
    return s / s.sum()


def residual_top(h7, pop, bump=0.0):
    k = np.arange(1, 7)
    return float((pop * (1 + bump) - (k * h7[:6]).sum()) / h7[6]) if h7[6] > 0 else None


# ---------------------------------------------------------------- the port of the twin's rule

def port(n_leaves=16667, hh=60, people=150, seed=7, apportion=True):
    """clamp(round(2.5 + 1.1 z), 1, 6) per household; the leaf's people shared by
    largest remainder with the drawn sizes as weights. Counts by size 0..."""
    rng = np.random.default_rng(seed)
    w = np.clip(np.floor(2.5 + 1.1 * rng.standard_normal((n_leaves, hh)) + 0.5), 1, 6)
    if not apportion:
        s = w.astype(int)
    else:
        q = people * w / w.sum(1, keepdims=True)
        s = np.floor(q).astype(int)
        left = people - s.sum(1)
        order = np.argsort(-(q - s), axis=1, kind="stable")
        rank = np.empty_like(order)
        np.put_along_axis(rank, order, np.arange(hh)[None, :].repeat(n_leaves, 0), axis=1)
        s += (rank < left[:, None]).astype(int)
    return np.bincount(s.ravel())


def gen_from_counts(counts):
    c = np.asarray(counts, dtype=float)
    n = c.sum()
    persons = (np.arange(len(c)) * c).sum()
    s = np.zeros(8)
    s[:7] = c[:7]
    s[7] = c[7:].sum()
    return {"mean": float(persons / n), "p1": float(c[1] / n), "shares": (s / n)}


# ---------------------------------------------------------------- plants

def synth_counties(rng, k, dist, hh_each=20000, jitter=0.0):
    """k counties, each with households multinomial from `dist` over sizes 1..7+
    (7+ at a true mean given by dist's tag). Returns ct (hh, pop, h1)."""
    sizes = np.array([1, 2, 3, 4, 5, 6, 9.0])
    ct = np.zeros((k, 3))
    for i in range(k):
        p = np.array(dist, dtype=float)
        if jitter:
            p = p * np.exp(jitter * rng.standard_normal(7))
            p /= p.sum()
        h = rng.multinomial(hh_each, p)
        ct[i] = [h.sum(), (h * sizes).sum(), h[0]]
    return ct


def plants():
    rng = np.random.default_rng(1)
    base = [0.28, 0.34, 0.15, 0.13, 0.06, 0.025, 0.015]
    res = {}

    a, b = synth_counties(rng, 200, base), synth_counties(rng, 200, base)
    g = quantities(*a.sum(0))
    v = verdict(g, b, neff(a[:, 0]))
    res["P-EQUAL"] = v["mean"]["verdict"] == "certified" and v["p1"]["verdict"] == "certified"

    sh = list(base)
    sh[0] += 0.10
    sh[1] -= 0.10
    b2 = synth_counties(rng, 200, sh)
    v = verdict(g, b2, neff(a[:, 0]))
    res["P-SHIFT"] = v["p1"]["verdict"] == "refused" and v["p1"].get("witness") == "one-person share"

    d1 = [0, .5, .5, 0, 0, 0, 0]
    d2 = [.5, 0, 0, .5, 0, 0, 0]
    c1, c2 = synth_counties(rng, 200, d1), synth_counties(rng, 200, d2)
    v = verdict(quantities(*c1.sum(0)), c2, neff(c1[:, 0]))
    wm = w1(shares8(d1), shares8(d2))
    res["P-MEAN"] = (v["mean"]["verdict"] == "certified" and v["p1"]["verdict"] == "refused"
                     and abs(wm - 1.0) < 1e-12)

    # 5 % in the top band at a true mean of 9, overall mean 2.75
    h = np.array([0.0, 0, 0, 0, 0, 0, 0.05])
    # the other 95 % split between sizes 2 and 3 to bring the overall mean to 2.75
    # 2x + 3y = 2.75 - 0.45 = 2.30, x + y = 0.95  ->  y = 0.40, x = 0.55
    h[1], h[2] = 0.55, 0.40
    h *= 1e6
    pop = (np.array([1, 2, 3, 4, 5, 6, 9.0]) * h).sum()
    r0, r1 = residual_top(h, pop), residual_top(h, pop, 0.01)
    res["P-TOP"] = (abs(r0 - 9.0) < 0.005 and abs(r1 - 9.55) < 0.005
                    and w1(shares8(h), shares8(h)) == 0.0)

    thin = synth_counties(rng, 12, base)
    v = verdict(g, thin)
    res["P-THIN"] = v["mean"]["verdict"] == "unresolved" and v["p1"]["verdict"] == "unresolved" \
        and "difference" not in v["mean"]

    dom = synth_counties(rng, 100, base, hh_each=2000)
    dom[0] *= 99 * 99.0
    v = verdict(g, dom)
    res["P-NEFF"] = v["p1"]["verdict"] == "unresolved" and v["n_eff"] < 2

    # counties whose one-person share varies at a fixed mean of 2.5
    k = 80
    aa = np.clip(0.25 + 0.06 * rng.standard_normal(k), 0.02, 0.48)
    noisy = np.zeros((k, 3))
    for i in range(k):
        hh_ = 20000.0
        noisy[i] = [hh_, 2.5 * hh_, aa[i] * hh_]
    gp = split_gap(noisy)
    v = verdict({"mean": 2.5, "p1": 0.25}, noisy)
    res["P-NOISY"] = (gp["p1"] > 0.015 and v["p1"]["verdict"] == "unresolved"
                      and v["mean"]["verdict"] == "certified")

    # reader plants: a tiny synthetic unit table through the same exclusion code
    U = make_units(
        geoid=["T1", "T2", "T3", "T4"], county=[1, 1, 1, 1],
        h=[[10, 20, 10, 5, 2, 1, 1]] * 3 + [[10, 20, -666666666, 5, 2, 1, 1]],
        pop=[120, 120, 120, 120], land=[1e6, 1e6, 0.0, 1e6], gq=[0, 1080, 0, 0])
    ex = exclusions(U)
    res["P-GQ"] = bool(ex["group_quarters"][1]) and int(ex["group_quarters"].sum()) == 1
    res["P-ZEROLAND"] = bool(ex["zero_land"][2]) and int(ex["zero_land"].sum()) == 1
    res["P-SENTINEL"] = bool(ex["coded"][3]) and int(ex["coded"].sum()) == 1
    kept = ~(ex["group_quarters"] | ex["zero_land"] | ex["coded"] | ex["zero_households"])
    res["P-SENTINEL"] &= int(kept.sum()) == 1 and float(U["h"][kept].sum()) == 49.0

    full, raw = port(n_leaves=4000), port(n_leaves=4000, apportion=False)
    sf, sr = gen_from_counts(full)["shares"], gen_from_counts(raw)["shares"]
    res["P-PORT"] = float(np.abs(sf - sr).max()) > 0.002
    return res


# ---------------------------------------------------------------- the reader

def make_units(geoid, county, h, pop, land, gq, state=None, **more):
    n = len(geoid)
    U = {"geoid": np.array(geoid), "county": np.array(county), "h": np.array(h, dtype=float),
         "pop": np.array(pop, dtype=float), "land": np.array(land, dtype=float),
         "gq": np.array(gq, dtype=float),
         "state": np.array(state if state is not None else [0] * n)}
    U.update(more)
    return U


def exclusions(U):
    coded = (U["h"] < 0).any(1) | (U["pop"] < 0) | np.isnan(U["h"]).any(1) | np.isnan(U["pop"])
    for k in ("ho", "hr", "popo", "popr"):
        if k in U:
            coded |= (U[k] < 0).reshape(len(coded), -1).any(1)
    hh = np.where(coded, 0, U["h"].sum(1))
    zero_land = ~(U["land"] > 0)
    zero_hh = ~coded & (hh <= 0)
    tot = U["pop"] + U["gq"]
    gqf = ~coded & (tot > 0) & (U["gq"] > 0.2 * tot)
    return {"coded": coded, "zero_land": zero_land, "zero_households": zero_hh, "group_quarters": gqf}


def read_dat(path, level_prefixes):
    """Pipe-delimited table file. Returns (header, {geoid: [estimates...]}) for
    rows of the wanted summary levels in the 50 states and DC."""
    rows = {}
    with open(path, encoding="latin-1") as f:
        header = f.readline().rstrip("\r\n").split("|")
        est = [i for i, c in enumerate(header) if "_E" in c]
        for line in f:
            if line[:9] not in level_prefixes:
                continue
            p = line.rstrip("\r\n").split("|")
            st = p[0][9:11]
            if not st.isdigit() or int(st) > 56:
                continue
            rows[p[0][9:]] = [float(p[i]) if p[i] not in ("", ".") else float("nan") for i in est]
    return [header[i] for i in est], rows


def read_dbf_fields(zpath, want):
    """Minimal dBase reader for the boundary file's attribute table."""
    with zipfile.ZipFile(zpath) as z:
        name = [n for n in z.namelist() if n.lower().endswith(".dbf")][0]
        b = z.read(name)
    nrec, hlen, rlen = struct.unpack("<IHH", b[4:12])
    fields, off, pos = [], 32, 1
    while b[off] != 0x0D:
        nm = b[off:off + 11].split(b"\0")[0].decode()
        ln = b[off + 16]
        fields.append((nm, pos, ln))
        pos += ln
        off += 32
    idx = {nm: (p, ln) for nm, p, ln in fields}
    out = {}
    for r in range(nrec):
        rec = b[hlen + r * rlen: hlen + (r + 1) * rlen]
        vals = [rec[idx[w][0]: idx[w][0] + idx[w][1]].decode("latin-1").strip() for w in want]
        out[vals[0]] = vals[1:]
    return out


def load(raw):
    P = ("1400000US", "1500000US")
    f = lambda t: os.path.join(raw, "acsdt5y2024-%s.dat" % t)
    hd9, t9 = read_dat(f("b25009"), P)
    hd8, t8 = read_dat(f("b25008"), P)
    hd32, t32 = read_dat(f("b25032"), P)
    hd33, t33 = read_dat(f("b25033"), P)
    hdg, tg = read_dat(f("b26001"), P)
    heads = {"B25009": hd9, "B25008": hd8, "B25032": hd32, "B25033": hd33, "B26001": hdg}
    land = {}
    with zipfile.ZipFile(os.path.join(raw, "2024_Gaz_tracts_national.zip")) as z:
        with z.open(z.namelist()[0]) as g:
            cols = g.readline().decode().strip().split("\t")
            cols = [c.strip() for c in cols]
            ig, ia = cols.index("GEOID"), cols.index("ALAND")
            for line in g:
                p = line.decode().split("\t")
                land[p[ig].strip()] = float(p[ia])
    for gid, (al,) in read_dbf_fields(os.path.join(raw, "cb_2024_us_bg_500k.zip"), ["GEOID", "ALAND"]).items():
        land[gid] = float(al)
    keys = {}
    for name, ln in (("tract", 11), ("block group", 12)):
        ids = sorted(g for g in t9 if len(g) == ln and g in t8)
        n = len(ids)
        a9 = np.array([t9[g] for g in ids])
        a8 = np.array([t8[g] for g in ids])
        nan5 = [float("nan")] * 5
        s_hh = np.array([struct5(t32[g], 32) if g in t32 else nan5 for g in ids])
        s_pop = np.array([struct5(t33[g], 33) if g in t33 else nan5 for g in ids])
        gq = np.array([tg[g[:11]][0] if g[:11] in tg else 0.0 for g in ids])
        gq = np.nan_to_num(gq, nan=0.0)
        if name == "block group":       # a block group carries its tract's flag
            tpop = {g: t8[g][0] for g in t8 if len(g) == 11}
            tot = np.array([tpop.get(g[:11], 0.0) for g in ids])
            gq_for_flag, pop_for_flag = gq, tot
        else:
            gq_for_flag, pop_for_flag = gq, a8[:, 0]
        U = make_units(
            geoid=ids, county=[int(g[:5]) for g in ids], state=[int(g[:2]) for g in ids],
            h=a9[:, 2:9] + a9[:, 10:17], pop=a8[:, 0],
            land=[land.get(g, 0.0) for g in ids], gq=gq,
            ho=a9[:, 2:9], hr=a9[:, 10:17], popo=a8[:, 1], popr=a8[:, 2],
            hh_total=a9[:, 0], s_hh=s_hh, s_pop=s_pop,
            no_area=np.array([g not in land for g in ids]))
        ex = exclusions(U)
        totf = pop_for_flag + gq_for_flag
        ex["group_quarters"] = ~ex["coded"] & (totf > 0) & (gq_for_flag > 0.2 * totf)
        U["ex"] = ex
        U["keep"] = ~(ex["coded"] | ex["zero_land"] | ex["zero_households"] | ex["group_quarters"])
        dens = np.where(U["keep"], U["pop"] / np.where(U["land"] > 0, U["land"], 1) * 1e6, 0.0)
        U["cls"] = np.where(U["keep"], classify(dens), -2)
        U["under_floor"] = U["keep"] & (U["cls"] == -1)
        keys[name] = U
    return heads, keys, {"B25032_rows": len(t32), "B25033_rows": len(t33)}


def struct5(row, table):
    """Fold a table's estimates into the five bands of B25033, owner + renter."""
    r = np.array(row)
    if table == 33:      # 001 total, 002 owner, 003..007, 008 renter, 009..013
        return list(r[2:7] + r[8:13])
    # B25032: 001 total, 002 owner, 003..012, 013 renter, 014..023
    def five(x):
        return np.array([x[0] + x[1], x[2] + x[3], x[4] + x[5] + x[6] + x[7], x[8], x[9]])
    return list(five(r[2:12]) + five(r[13:23]))


# ---------------------------------------------------------------- the run

def ct_of(U, mask):
    ids, s = by_county(U["county"][mask],
                       np.stack([U["h"][mask].sum(1), U["pop"][mask], U["h"][mask][:, 0]], 1))
    return ids, s


def reading(U, mask):
    h = U["h"][mask].sum(0)
    pop = U["pop"][mask].sum()
    out = {"units": int(mask.sum()), "households": float(h.sum()),
           "shares_ppm": [int(round(x)) for x in 1e6 * h / h.sum()],
           "persons_per_household": float(pop / h.sum()), "one_person_share": float(h[0] / h.sum()),
           "top_band_residual_mean": residual_top(h, pop),
           "top_band_residual_mean_pop_plus_1pct": residual_top(h, pop, 0.01),
           "top_band_residual_mean_pop_minus_1pct": residual_top(h, pop, -0.01)}
    ten = {}
    for nm, hk, pk in (("owner", "ho", "popo"), ("renter", "hr", "popr")):
        hh = U[hk][mask].sum(0)
        ten[nm] = {"households": float(hh.sum()), "persons_per_household": float(U[pk][mask].sum() / hh.sum()),
                   "one_person_share": float(hh[0] / hh.sum())}
    out["by_tenure"] = ten
    ok = mask & ~np.isnan(U["s_hh"]).any(1) & ~np.isnan(U["s_pop"]).any(1) \
        & (U["s_hh"] >= 0).all(1) & (U["s_pop"] >= 0).all(1)
    sh, sp = U["s_hh"][ok].sum(0), U["s_pop"][ok].sum(0)
    out["by_structure"] = {STRUCT[i]: {"households": float(sh[i]),
                                       "persons_per_household": float(sp[i] / sh[i]) if sh[i] > 0 else None}
                           for i in range(5)}
    out["by_structure_units"] = int(ok.sum())
    return out


def run(raw, twin_path, out_dir):
    log = {}
    pl = plants()
    log["plants"] = pl
    print("plants:", pl)
    if not all(pl.values()):
        print("BRANCH (e): a plant failed. Nothing is read.")
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
        return 1

    tb = open(twin_path, "rb").read()
    twin = json.loads(tb)
    log["twin_file_sha256"] = hashlib.sha256(tb).hexdigest()
    # the twin's file: "final" is what its code emits (scored); "drawn" is the
    # sizes before its apportionment step (reported only)
    gens = {k: gen_from_counts(twin[k]["counts_by_size"]) for k in ("final", "drawn")}
    log["twin_meta"] = {k: twin[k] for k in ("what", "leaf_shape", "seed_scheme", "conditioning",
                                             "library_commit", "command")}
    main = "final"
    G = gens[main]
    pc = gen_from_counts(port())
    log["generator"] = {k: {"mean": g["mean"], "p1": g["p1"], "shares": list(map(float, g["shares"]))}
                        for k, g in gens.items()}
    log["generator_scored"] = main
    log["port"] = {"mean": pc["mean"], "p1": pc["p1"], "shares": list(map(float, pc["shares"])),
                   "max_share_difference_from_twin": float(np.abs(pc["shares"] - G["shares"]).max())}
    print("twin file sha256", log["twin_file_sha256"], "scored:", main, "mean %.4f p1 %.4f" % (G["mean"], G["p1"]))
    print("port agrees to", log["port"]["max_share_difference_from_twin"])

    # the draw is over the state codes the release carries, sorted
    rng = np.random.default_rng(STATE_SEED)

    digests = {}
    for fn in sorted(os.listdir(raw)):
        hsh = hashlib.sha256()
        with open(os.path.join(raw, fn), "rb") as f:
            for chunk in iter(lambda: f.read(1 << 22), b""):
                hsh.update(chunk)
        digests[fn] = hsh.hexdigest()
    log["file_sha256"] = digests

    heads, keys, misc = load(raw)
    log["headers"] = {k: len(v) for k, v in heads.items()}
    T, B = keys["tract"], keys["block group"]
    codes = sorted(set(T["state"].tolist()))
    perm = rng.permutation(len(codes))
    fit_states = sorted(codes[i] for i in perm[:26])
    score_states = sorted(codes[i] for i in perm[26:])
    log["states"] = {"codes": codes, "FIT": fit_states, "SCORE": score_states}
    print("states read:", len(codes), "FIT", fit_states, "SCORE", score_states)

    # G0
    exp = {"B25009": 17, "B25008": 3, "B25032": 23, "B25033": 13, "B26001": 1}
    g0 = {"headers": all(len(heads[k]) == exp[k] for k in exp)}
    # labels from the documentation file, at the lines the reader's slices assume
    want = {"B25009_003": "1-person household", "B25009_009": "7-or-more person household",
            "B25009_010": "Renter occupied:", "B25009_011": "1-person household",
            "B25009_017": "7-or-more person household", "B25008_002": "Owner occupied",
            "B25033_003": "1, detached or attached", "B25033_008": "Renter occupied:",
            "B25032_003": "1, detached", "B25032_013": "Renter-occupied housing units:", "B25032_014": "1, detached"}
    got = {}
    with open(os.path.join(raw, "ACS20245YR_Table_Shells.txt"), encoding="latin-1") as sf:
        for line in sf:
            p = line.split("|")
            if len(p) > 4 and p[3] in want:
                got[p[3]] = p[4].strip()
    g0["labels"] = got == want
    g0["labels_read"] = got
    # the owner and renter cells sum to the table's own total, unit by unit
    okT0 = ~T["ex"]["coded"]
    g0["cells_equal_total_units"] = int((np.abs(T["h"][okT0].sum(1) - T["hh_total"][okT0]) > 0.5).sum())
    okT = ~T["ex"]["coded"]
    nat_hh9 = float(T["hh_total"][okT].sum())
    nat_sum = float(T["h"][okT].sum())
    s32 = T["s_hh"][okT]
    nat_hh32 = float(np.nansum(s32))
    okB = ~B["ex"]["coded"]
    bg_hh = float(B["h"][okB].sum())
    nat_ratio = float(T["pop"][okT].sum() / nat_sum)
    g0.update(households_B25009_total=nat_hh9, households_B25009_cells=nat_sum,
              households_B25032=nat_hh32, block_group_households=bg_hh,
              persons_per_household=nat_ratio, counties=len(set(T["county"].tolist())))
    g0["pass"] = bool(g0["headers"] and g0["labels"] and g0["cells_equal_total_units"] == 0 and abs(nat_hh9 - nat_hh32) <= 1 and abs(nat_sum - nat_hh9) <= 1
                      and abs(bg_hh / nat_sum - 1) <= 1e-3 and 2.3 <= nat_ratio <= 2.8)
    log["G0"] = g0
    print("G0:", g0)
    if not g0["pass"]:
        print("BRANCH (e): G0 failed. Nothing is read.")
        json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
        return 1

    table = {"keys": {}}
    S1 = {}
    resolved = {}
    for kn, U in keys.items():
        ex = U["ex"]
        excl = {k: int(v.sum()) for k, v in ex.items()}
        excl["no_area_on_file"] = int(U["no_area"].sum())
        excl["under_floor"] = int(U["under_floor"].sum())
        excl["units_total"] = int(len(U["geoid"]))
        excl["households_kept_share"] = float(U["h"][U["cls"] >= 0].sum() / U["h"][~ex["coded"]].sum())
        rows, s1 = {}, {}
        allm = U["cls"] >= 0
        rows["nation"] = reading(U, allm)
        _, ct = ct_of(U, allm)
        s1["nation"] = verdict(G, ct)
        s1["nation"]["w1_top_at_8"] = w1(G["shares"], shares8(U["h"][allm].sum(0)))
        res = []
        for c in range(5):
            m = U["cls"] == c
            if m.sum() == 0:
                continue
            rows[CLASS_NAMES[c]] = reading(U, m)
            _, ct = ct_of(U, m)
            v = verdict(G, ct)
            v["w1_top_at_8"] = w1(G["shares"], shares8(U["h"][m].sum(0)))
            gp = split_gap(ct) if v["n_eff"] >= NEFF_MIN else None
            rows[CLASS_NAMES[c]]["n_eff_counties"] = v["n_eff"]
            rows[CLASS_NAMES[c]]["split_half_gap_95"] = gp
            s1[CLASS_NAMES[c]] = v
            if v["n_eff"] >= NEFF_MIN:
                res.append(c)
        resolved[kn] = res
        table["keys"][kn] = {"exclusions": excl, "rows": rows}
        S1[kn] = s1
    log["S1"] = S1
    log["resolved"] = {k: [CLASS_NAMES[c] for c in v] for k, v in resolved.items()}
    comp_cert = [c for c in resolved["tract"] if S1["tract"][CLASS_NAMES[c]]["p1"]["verdict"] == "certified"]
    log["S1_summary"] = {
        "mean_nation": S1["tract"]["nation"]["mean"]["verdict"],
        "composition_by_class": {CLASS_NAMES[c]: S1["tract"][CLASS_NAMES[c]]["p1"]["verdict"] for c in resolved["tract"]},
        "expectation": "killed" if len(comp_cert) >= 2 else "stands"}

    # group quarters, national, from the tract table
    log["group_quarters_share_of_people"] = float(T["gq"].sum() / (T["gq"].sum() + T["pop"][okT].sum()))

    # S2
    fit = np.isin(T["state"], fit_states) & (T["cls"] >= 0)
    sco = np.isin(T["state"], score_states) & (T["cls"] >= 0)
    nat_rule = quantities(T["h"][fit].sum(), T["pop"][fit].sum(), T["h"][fit][:, 0].sum())
    s2 = {"FIT-NATION-1": nat_rule, "FIT-CLASS-1": {}, "classes_used": [], "classes_left_out": []}
    wsum = win_c = win_n = 0.0
    nunits = 0
    per_class = {}
    for c in range(5):
        mf, ms = fit & (T["cls"] == c), sco & (T["cls"] == c)
        if mf.sum() == 0 or ms.sum() == 0:
            continue
        _, cf = ct_of(T, mf)
        _, cs = ct_of(T, ms)
        rule = quantities(*cf.sum(0))
        s2["FIT-CLASS-1"][CLASS_NAMES[c]] = dict(rule, n_eff_fit=neff(cf[:, 0]), n_eff_score=neff(cs[:, 0]))
        if neff(cf[:, 0]) < NEFF_MIN or neff(cs[:, 0]) < NEFF_MIN:
            s2["classes_left_out"].append(CLASS_NAMES[c])
            continue
        s2["classes_used"].append(CLASS_NAMES[c])
        big = cs[cs[:, 0] >= UNIT_FLOOR]
        um, up = big[:, 1] / big[:, 0], big[:, 2] / big[:, 0]
        inc = (np.abs(um - rule["mean"]) <= BUDGET["mean"]) & (np.abs(up - rule["p1"]) <= BUDGET["p1"])
        inn = (np.abs(um - nat_rule["mean"]) <= BUDGET["mean"]) & (np.abs(up - nat_rule["p1"]) <= BUDGET["p1"])
        w = big[:, 0]
        per_class[CLASS_NAMES[c]] = {"units": int(len(big)), "households": float(w.sum()),
                                     "within_class_rule": float((w * inc).sum() / w.sum()) if len(big) else None,
                                     "within_national_rule": float((w * inn).sum() / w.sum()) if len(big) else None,
                                     "within_mean_only_class": float((w * (np.abs(um - rule["mean"]) <= BUDGET["mean"])).sum() / w.sum()) if len(big) else None,
                                     "within_p1_only_class": float((w * (np.abs(up - rule["p1"]) <= BUDGET["p1"])).sum() / w.sum()) if len(big) else None,
                                     "within_mean_only_nation": float((w * (np.abs(um - nat_rule["mean"]) <= BUDGET["mean"])).sum() / w.sum()) if len(big) else None,
                                     "within_p1_only_nation": float((w * (np.abs(up - nat_rule["p1"]) <= BUDGET["p1"])).sum() / w.sum()) if len(big) else None}
        wsum += w.sum()
        win_c += (w * inc).sum()
        win_n += (w * inn).sum()
        nunits += len(big)
    s2["per_class"] = per_class
    s2["units"] = nunits
    s2["coverage_of_score_households"] = float(wsum / T["h"][sco].sum())
    s2["share_within_class_rule"] = float(win_c / wsum)
    s2["share_within_national_rule"] = float(win_n / wsum)
    d = s2["share_within_class_rule"] - s2["share_within_national_rule"]
    s2["difference"] = float(d)
    s2["outcome"] = "met" if d >= 0.05 else ("killed" if d <= 0 else "neither")
    log["S2"] = s2

    # S2b and S3
    both = [c for c in resolved["tract"] if c in resolved["block group"]]
    s2b = {}
    for c in both:
        a, b = table["keys"]["tract"]["rows"][CLASS_NAMES[c]], table["keys"]["block group"]["rows"][CLASS_NAMES[c]]
        dm = a["persons_per_household"] - b["persons_per_household"]
        dp = a["one_person_share"] - b["one_person_share"]
        s2b[CLASS_NAMES[c]] = {"mean_tract_minus_block_group": dm, "p1_tract_minus_block_group": dp,
                               "transfers": bool(abs(dm) < BUDGET["mean"] and abs(dp) < BUDGET["p1"])}
    log["S2b"] = s2b
    r = table["keys"]["tract"]["rows"]
    ms_ = [r[CLASS_NAMES[c]]["persons_per_household"] for c in resolved["tract"]]
    ps_ = [r[CLASS_NAMES[c]]["one_person_share"] for c in resolved["tract"]]
    dm, dp = max(ms_) - min(ms_), max(ps_) - min(ps_)
    log["S3"] = {"largest_mean_difference": dm, "largest_p1_difference": dp,
                 "outcome": "met" if (dm >= 0.12 or dp >= 0.03) else ("killed" if (dm < 0.06 and dp < 0.015) else "neither")}

    log["branch"] = "(a)" if log["S1_summary"]["expectation"] == "stands" else "(b)"
    table["budgets"] = BUDGET
    table["class_lower_bounds_per_km2"] = BOUNDS
    table["generator"] = log["generator"]
    table["verdicts"] = S1
    table["S2"] = s2
    table["S2b"] = s2b
    table["S3"] = log["S3"]
    os.makedirs(out_dir, exist_ok=True)
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
    json.dump(table, open(os.path.join(out_dir, "household_table.json"), "w"), indent=1)
    print(json.dumps({k: log[k] for k in ("S1_summary", "S3", "S2b", "branch", "resolved")}, indent=1))
    print("S2:", json.dumps({k: s2[k] for k in s2 if k != "per_class"}, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-11s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3], sys.argv[4]))
