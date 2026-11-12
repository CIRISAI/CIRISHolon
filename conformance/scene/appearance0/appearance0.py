#!/usr/bin/env python3
"""APPEARANCE-0: a Köppen-Geiger class for each cell of the twin's 1/8° grid (`APPEARANCE0_PREREG.md`).

  appearance0.py plants
  appearance0.py run RASTER.tif LEGEND.txt OUT_DIR

Needs numpy and Pillow. The reducer is `reduce(pixels, k)`: k by k source pixels per cell.
"""
import gzip
import hashlib
import io
import json
import os
import re
import sys

import numpy as np

ROWS, COLS, K = 1440, 2880, 15
PLACES = [("Singapore", 1.35, 103.82, ["Af"]), ("Manaus", -3.10, -60.02, ["Af", "Am"]), ("Cairo", 30.04, 31.24, ["BWh"]), ("Riyadh", 24.71, 46.68, ["BWh"]),
          ("Phoenix", 33.45, -112.07, ["BWh", "BSh"]), ("Madrid", 40.42, -3.70, ["BSk", "Csa"]), ("Rome", 41.90, 12.50, ["Csa"]), ("London", 51.51, -0.13, ["Cfb"]),
          ("Sydney", -33.87, 151.21, ["Cfa", "Cfb"]), ("Moscow", 55.75, 37.62, ["Dfb"]), ("Yakutsk", 62.03, 129.73, ["Dfc", "Dfd", "Dwc", "Dwd"]), ("Nuuk", 64.18, -51.72, ["ET"])]


class Stop(Exception):
    pass


def reduce(px, k, codes):
    """px: 2-D uint8, rows north to south, columns from -180. Returns (class, share255, classed, ties, counts3d)."""
    r, c = px.shape[0] // k, px.shape[1] // k
    if px.shape != (r * k, c * k):
        raise Stop("the raster does not nest")
    present = np.unique(px)
    bad = [int(v) for v in present if v != 0 and int(v) not in codes]
    if bad:
        raise Stop("codes not in the legend: %s" % bad)
    top = max(codes)
    counts = np.zeros((top + 1, r, c), dtype=np.uint16)
    v = px.reshape(r, k, c, k)
    for code in range(1, top + 1):
        counts[code] = (v == code).sum(axis=(1, 3), dtype=np.uint16)
    n = counts.sum(axis=0, dtype=np.uint32)
    cls = counts.argmax(axis=0).astype(np.uint8)          # the first maximum is the lower code
    best = counts.max(axis=0).astype(np.uint32)
    cls[n == 0] = 0
    srt = np.sort(counts, axis=0)
    ties = int(((srt[-1] == srt[-2]) & (n > 0)).sum())
    share = np.zeros((r, c), dtype=np.uint8)
    m = n > 0
    share[m] = ((2 * 255 * best[m] + n[m]) // (2 * n[m])).astype(np.uint8)   # round half up of 255 k / n
    return cls, share, n.astype(np.uint8 if k * k < 256 else np.uint16), ties, counts


def cell(lat, lon):
    r = min(ROWS - 1, max(0, int(np.floor((90 - lat) / 0.125))))
    c = int(np.floor(((lon + 180) % 360) / 0.125))
    return r, c


def legend(text):
    out = {}
    for m in re.finditer(r"^\s*(\d+):\s+(\S+)\s+(.*?)\s+\[(\d+) (\d+) (\d+)\]\s*$", text, re.M):
        out[int(m.group(1))] = {"symbol": m.group(2), "name": m.group(3), "source_map_colour_not_an_appearance": [int(m.group(i)) for i in (4, 5, 6)]}
    return out


def gz(b):
    o = io.BytesIO()
    with gzip.GzipFile(filename="", fileobj=o, mode="wb", mtime=0) as f:
        f.write(b)
    return o.getvalue()


def plants():
    res = {}
    codes = set(range(1, 31))
    a = np.zeros((40, 80), dtype=np.uint8)                 # k = 10: 4 by 8 cells
    a[:10, :10] = 7
    a[30:, 70:] = 9
    cls, share, n, ties, _ = reduce(a, 10, codes)
    res["P-ORIENT"] = cls.flat[0] == 7 and cls.flat[-1] == 9 and cls[0, 1] == 0
    b = np.zeros((10, 20), dtype=np.uint8)
    b[:6, :10] = 3
    b[6:, :10] = 5                                         # 60 % and 40 %
    b[:5, 10:] = 12
    b[5:, 10:] = 4                                         # a tie between 4 and 12
    cls, share, n, ties, _ = reduce(b, 10, codes)
    res["P-MAJOR"] = cls[0, 0] == 3 and share[0, 0] == 153
    res["P-TIE"] = cls[0, 1] == 4 and ties == 1
    s = np.zeros((10, 20), dtype=np.uint8)
    s[0, 10:] = 8                                          # 10 land pixels of 100
    cls, share, n, ties, _ = reduce(s, 10, codes)
    res["P-SEA"] = cls[0, 0] == 0 and share[0, 0] == 0 and n[0, 0] == 0 and cls[0, 1] == 8 and share[0, 1] == 255 and n[0, 1] == 10
    res["P-WRAP"] = cell(0.0, -180.0)[1] == 0 and cell(0.0, 179.95)[1] == COLS - 1 and cell(90.0, 0)[0] == 0 and cell(-90.0, 0)[0] == ROWS - 1 and cell(89.9, 0)[0] == 0
    try:
        reduce(np.full((10, 10), 31, dtype=np.uint8), 10, codes)
        res["P-LEGEND"] = False
    except Stop:
        z = reduce(np.zeros((10, 10), dtype=np.uint8), 10, codes)
        res["P-LEGEND"] = z[0][0, 0] == 0
    return res


def run(raster, legend_path, out):
    os.makedirs(out, exist_ok=True)
    log = {"plants": {k: bool(v) for k, v in plants().items()}}
    if not all(log["plants"].values()):
        log["branch"] = "(e): a plant failed"
        print(json.dumps(log, indent=1))
        return 1
    leg = legend(open(legend_path, encoding="utf-8").read())
    want = {1: "Af", 4: "BWh", 15: "Cfb", 29: "ET", 30: "EF"}
    if sorted(leg) != list(range(1, 31)) or any(leg[k]["symbol"] != v for k, v in want.items()):
        log["branch"] = "(e): the legend is not as required"
        print(json.dumps(log, indent=1))
        return 1
    from PIL import Image
    Image.MAX_IMAGE_PIXELS = None
    try:
        im = Image.open(raster)
        if im.size != (COLS * K, ROWS * K):
            raise Stop("raster size %s" % (im.size,))
        px = np.asarray(im, dtype=np.uint8)
        cls, share, n, ties, counts = reduce(px, K, set(leg))
        del px
    except Exception as e:
        log["branch"] = "(e): %s: %s" % (type(e).__name__, e)
        print(json.dumps(log, indent=1))
        return 1
    sym = {k: v["symbol"] for k, v in leg.items()}
    r1 = []
    for name, lat, lon, allowed in PLACES:
        r, c = cell(lat, lon)
        got = sym.get(int(cls[r, c]), "none")
        r1.append({"place": name, "cell": int(r * COLS + c), "class": got, "allowed": allowed, "met": got in allowed})
    hits = sum(x["met"] for x in r1)
    log["R1"] = {"met_of_12": hits, "places": r1, "reading": "met" if hits >= 10 else ("misread" if hits <= 6 else "between")}
    classed = cls > 0
    north = int(classed[:1200].sum())
    log["R2"] = {"classed_cells_rows_0_to_1199": north, "share": round(north / (1200 * COLS), 4), "met": 898560 <= north <= 1244160,
                 "share_rows_1200_to_1439_no_verdict": round(float(classed[1200:].mean()), 4)}
    nmid, smid = float(classed[240:480].mean()), float(classed[960:1200].mean())
    log["R3"] = {"north_30_to_60": round(nmid, 4), "south_30_to_60": round(smid, 4), "met": nmid > 2 * smid}
    best = counts.max(axis=0).astype(np.float64)
    nn = n.astype(np.float64)
    frac = np.divide(best, nn, out=np.zeros_like(best), where=nn > 0)
    total = int(classed.sum())
    s1 = float(((frac < 0.8) & classed).sum()) / total
    group = lambda code: "ABCDE"[0 if code <= 3 else 1 if code <= 7 else 2 if code <= 16 else 3 if code <= 28 else 4]
    gcounts = np.zeros((5,) + cls.shape, dtype=np.uint32)
    for code in leg:
        gcounts["ABCDE".index(group(code))] += counts[code]
    gmaj = np.zeros(cls.shape, dtype=np.uint32)
    gof = np.zeros(cls.shape, dtype=np.int8)
    for code in leg:
        gof[cls == code] = "ABCDE".index(group(code))
    for g in range(5):
        gmaj[gof == g] = gcounts[g][gof == g]
    outside = np.divide(nn - gmaj, nn, out=np.zeros_like(nn), where=nn > 0)
    by_group, s1_group = {}, {}
    for g in range(5):
        m = classed & (gof == g)
        by_group["ABCDE"[g]] = round(float(m.sum()) / total, 4)
        s1_group["ABCDE"[g]] = round(float(((frac < 0.8) & m).sum()) / max(int(m.sum()), 1), 4)
    log["S1"] = {"share_of_classed_cells_with_majority_under_0.8": round(s1, 4), "reading": "met" if s1 <= 0.15 else ("kill" if s1 > 0.35 else "neither"), "by_main_group": s1_group,
                 "share_with_0.2_or_more_outside_the_majority_group": round(float(((outside >= 0.2) & classed).sum()) / total, 4)}
    cells_by = {sym[k]: int((cls == k).sum()) for k in sorted(leg)}
    log["printed"] = {"classed_cells": total, "share_of_all_cells": round(total / cls.size, 4), "cells_by_class": cells_by, "five_largest": sorted(cells_by.items(), key=lambda kv: -kv[1])[:5],
                      "share_by_main_group": by_group, "ties": ties, "cells_with_under_23_classed_pixels": int(((n > 0) & (n < 23)).sum())}
    if log["R1"]["reading"] == "misread" or not log["R2"]["met"] or not log["R3"]["met"]:
        log["branch"] = "(r): a reading check failed; nothing written"
    else:
        files = {}
        for name, arr in (("koppen_1991_2020_cell", cls), ("koppen_1991_2020_cell_share", share), ("koppen_1991_2020_cell_classed", n)):
            raw = arr.astype(np.uint8).tobytes()
            packed = gz(raw)
            open(os.path.join(out, name + ".bin.gz"), "wb").write(packed)
            files[name + ".bin.gz"] = {"bytes_packed": len(packed), "sha256_packed": hashlib.sha256(packed).hexdigest(), "bytes_unpacked": len(raw), "sha256_unpacked": hashlib.sha256(raw).hexdigest()}
        log["files"] = files
        log["legend"] = {str(k): v for k, v in leg.items()}
        log["branch"] = "(a): the three grids are written"
    json.dump(log, open(os.path.join(out, "results.json"), "w"), indent=1)
    print(json.dumps({k: v for k, v in log.items() if k != "legend"}, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-9s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(*sys.argv[2:5]))
