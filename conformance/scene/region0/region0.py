#!/usr/bin/env python3
"""REGION-0: a US state for each cell of the twin's 1/8° grid (`REGION0_PREREG.md`).

  region0.py plants
  region0.py header ARCHIVE.zip            # file list, attribute columns, countries, shape count; no coordinate
  region0.py run ARCHIVE.zip OUT_DIR

Needs numpy. Reads the shapefile with a small reader of its own (polygons and the attribute table).
"""
import gzip
import hashlib
import io
import json
import math
import os
import struct
import sys
import zipfile

import numpy as np

ROWS, COLS, SUB = 1440, 2880, 5
RADIUS = 6371.0088
CODES = {"AL": 1, "AK": 2, "AZ": 4, "AR": 5, "CA": 6, "CO": 8, "CT": 9, "DE": 10, "DC": 11, "FL": 12, "GA": 13, "HI": 15, "ID": 16, "IL": 17, "IN": 18, "IA": 19, "KS": 20,
         "KY": 21, "LA": 22, "ME": 23, "MD": 24, "MA": 25, "MI": 26, "MN": 27, "MS": 28, "MO": 29, "MT": 30, "NE": 31, "NV": 32, "NH": 33, "NJ": 34, "NM": 35, "NY": 36,
         "NC": 37, "ND": 38, "OH": 39, "OK": 40, "OR": 41, "PA": 42, "RI": 44, "SC": 45, "SD": 46, "TN": 47, "TX": 48, "UT": 49, "VT": 50, "VA": 51, "WA": 53, "WV": 54,
         "WI": 55, "WY": 56}
DIVISION = {"New England": "CT ME MA NH RI VT", "Middle Atlantic": "NJ NY PA", "East North Central": "IL IN MI OH WI", "West North Central": "IA KS MN MO NE ND SD",
            "South Atlantic": "DE DC FL GA MD NC SC VA WV", "East South Central": "AL KY MS TN", "West South Central": "AR LA OK TX",
            "Mountain North": "CO ID MT UT WY", "Mountain South": "AZ NM NV", "Pacific": "AK CA HI OR WA"}
CITIES = None      # set from the freeze in `run`
AREAS = None


class Stop(Exception):
    pass


def read_dbf(b):
    n, hlen, rlen = struct.unpack("<IHH", b[4:12])
    fields, pos = [], 32
    while b[pos] != 0x0D:
        name = b[pos:pos + 11].split(b"\0")[0].decode("ascii", "replace")
        fields.append((name, b[pos + 16]))
        pos += 32
    recs = []
    for i in range(n):
        r = b[hlen + i * rlen: hlen + (i + 1) * rlen]
        o, d = 1, {}
        for name, ln in fields:
            d[name] = r[o:o + ln].decode("utf-8", "replace").strip()
            o += ln
        recs.append(d)
    return [f[0] for f in fields], recs


def read_shp(b):
    """A list, in record order, of shapes; a shape is a list of rings; a ring an (n, 2) array of lon, lat."""
    shapes, pos = [], 100
    while pos < len(b):
        _, clen = struct.unpack(">ii", b[pos:pos + 8])
        rec = b[pos + 8: pos + 8 + 2 * clen]
        pos += 8 + 2 * clen
        st = struct.unpack("<i", rec[:4])[0]
        if st == 0:
            shapes.append([])
            continue
        if st not in (5, 15, 25):
            raise Stop("a shape that is not a polygon: type %d" % st)
        nparts, npts = struct.unpack("<ii", rec[36:44])
        parts = list(struct.unpack("<%di" % nparts, rec[44:44 + 4 * nparts])) + [npts]
        pts = np.frombuffer(rec, dtype="<f8", count=2 * npts, offset=44 + 4 * nparts).reshape(npts, 2)
        shapes.append([pts[parts[i]:parts[i + 1]].copy() for i in range(nparts)])
    return shapes


def rasterise(shapes_by_code, lat_range=None):
    """Sample points at sub-cell centres; even-odd fill over all rings of a shape. Returns (code grid, counts dict of fine arrays)."""
    H, Wd = ROWS * SUB, COLS * SUB
    step = 0.125 / SUB
    fine = np.zeros((H, Wd), dtype=np.uint8)
    for code in sorted(shapes_by_code):
        rings = shapes_by_code[code]
        lats = np.concatenate([r[:, 1] for r in rings])
        i0 = max(0, int(math.floor((90 - lats.max()) / step)) - 1)
        i1 = min(H - 1, int(math.ceil((90 - lats.min()) / step)) + 1)
        segs = []
        for r in rings:
            lon = r[:, 0].copy()
            if lon.max() - lon.min() > 180:           # a ring that crosses longitude 180: unwrap it, fill, and wrap the columns
                lon = np.where(lon < 0, lon + 360, lon)
            segs.append((lon[:-1], r[:-1, 1], lon[1:], r[1:, 1]))
        x0, y0, x1, y1 = (np.concatenate([s[i] for s in segs]) for i in range(4))
        for i in range(i0, i1 + 1):
            y = 90 - (i + 0.5) * step
            hit = (y0 <= y) != (y1 <= y)              # the half-open rule: a sample on a vertex is counted once
            if not hit.any():
                continue
            xs = np.sort(x0[hit] + (y - y0[hit]) * (x1[hit] - x0[hit]) / (y1[hit] - y0[hit]))
            for a, b in zip(xs[0::2], xs[1::2]):
                j0 = int(math.ceil((a + 180) / step - 0.5))
                j1 = int(math.floor((b + 180) / step - 0.5))
                if b <= a or j1 < j0:
                    continue
                cols = np.arange(j0, j1 + 1) % Wd
                row = fine[i]
                free = row[cols] == 0                 # a point already taken keeps the lower code (codes are filled in rising order)
                row[cols[free]] = code
    return fine


def reduce(fine):
    v = fine.reshape(ROWS, SUB, COLS, SUB).transpose(0, 2, 1, 3).reshape(ROWS, COLS, SUB * SUB)
    classed = (v > 0).sum(axis=2).astype(np.uint8)
    cls = np.zeros((ROWS, COLS), dtype=np.uint8)
    best = np.zeros((ROWS, COLS), dtype=np.uint8)
    ys, xs = np.nonzero(classed)
    for y, x in zip(ys.tolist(), xs.tolist()):
        vals, cnt = np.unique(v[y, x][v[y, x] > 0], return_counts=True)
        k = int(np.argmax(cnt))                        # the first maximum is the lower code
        cls[y, x], best[y, x] = vals[k], cnt[k]
    share = np.zeros((ROWS, COLS), dtype=np.uint8)
    m = classed > 0
    share[m] = ((2 * 255 * best[m].astype(np.uint32) + classed[m]) // (2 * classed[m].astype(np.uint32))).astype(np.uint8)
    return cls, share, classed


def point_area_km2():
    """The area on the sphere of one sample point's sub-cell, by fine row."""
    step = math.radians(0.125 / SUB)
    i = np.arange(ROWS * SUB)
    top, bot = math.pi / 2 - i * step, math.pi / 2 - (i + 1) * step
    return RADIUS ** 2 * step * (np.sin(top) - np.sin(bot))


def cell(lat, lon):
    return min(ROWS - 1, max(0, int(math.floor((90 - lat) / 0.125)))), int(math.floor(((lon + 180) % 360) / 0.125))


def gz(b):
    o = io.BytesIO()
    with gzip.GzipFile(filename="", fileobj=o, mode="wb", mtime=0) as f:
        f.write(b)
    return o.getvalue()


def box(lon0, lat0, lon1, lat1):
    return np.array([[lon0, lat0], [lon0, lat1], [lon1, lat1], [lon1, lat0], [lon0, lat0]], dtype=float)


def plants():
    res = {}
    fine = rasterise({7: [box(10.0, 40.0, 11.0, 41.0)]})
    cls, share, classed = reduce(fine)
    r0, c0 = cell(40.999, 10.001)
    inside = cls[r0:r0 + 8, c0:c0 + 8]
    res["P-SQUARE"] = bool((inside == 7).all() and (cls == 7).sum() == 64 and classed[r0, c0] == 25 and share[r0, c0] == 255)
    fine = rasterise({7: [box(10.0, 40.0, 11.0, 41.0), box(10.25, 40.25, 10.75, 40.75)[::-1]]})
    cls, _, _ = reduce(fine)
    rh, ch = cell(40.5, 10.5)
    res["P-HOLE"] = bool(cls[rh, ch] == 0 and (cls == 7).sum() == 64 - 16)
    # two squares meeting inside a cell: 3 of 5 sample columns to code 9, 2 to code 4; then an exact tie is impossible with 25 points, so a 10-to-10 tie is planted with a gap
    fine = rasterise({9: [box(20.0, 10.0, 20.075, 10.125)], 4: [box(20.075, 10.0, 20.125, 10.125)]})
    cls, share, classed = reduce(fine)
    rb, cb = cell(10.06, 20.06)
    ok = cls[rb, cb] == 9 and classed[rb, cb] == 25 and share[rb, cb] == 153
    fine = rasterise({9: [box(20.0, 10.0, 20.05, 10.125)], 4: [box(20.075, 10.0, 20.125, 10.125)]})
    cls, share, classed = reduce(fine)
    res["P-BORDER"] = bool(ok and cls[rb, cb] == 4 and classed[rb, cb] == 20 and share[rb, cb] == 128)
    fine = rasterise({3: [box(-179.9, 89.0, -179.0, 89.9)]})
    cls, _, _ = reduce(fine)
    ys, xs = np.nonzero(cls)
    res["P-ORIENT"] = bool(len(ys) and ys.max() < 10 and xs.max() < 10)
    fine = rasterise({5: [box(30.0, 0.0, 30.5, 0.5), box(40.0, 0.0, 40.5, 0.5)]})
    cls, _, _ = reduce(fine)
    res["P-MULTI"] = bool(cls[cell(0.25, 30.25)] == 5 and cls[cell(0.25, 40.25)] == 5 and cls[cell(0.25, 35.0)] == 0)
    # a ring across longitude 180 fills its two ends and nothing between
    ring = np.array([[179.5, 50.0], [179.5, 51.0], [-179.5, 51.0], [-179.5, 50.0], [179.5, 50.0]])
    fine = rasterise({2: [ring]})
    cls, _, _ = reduce(fine)
    res["P-ANTIMERIDIAN"] = bool(cls[cell(50.5, 179.8)] == 2 and cls[cell(50.5, -179.8)] == 2 and cls[cell(50.5, 0.0)] == 0 and (cls == 2).sum() == 64)
    return res


def open_archive(path):
    z = zipfile.ZipFile(path)
    shp = [n for n in z.namelist() if n.endswith(".shp")]
    dbf = [n for n in z.namelist() if n.endswith(".dbf")]
    if len(shp) != 1 or len(dbf) != 1:
        raise Stop("the archive does not hold exactly one shapefile")
    return z, shp[0], dbf[0]


def header(path):
    z, shp, dbf = open_archive(path)
    cols, recs = read_dbf(z.read(dbf))
    out = {"files": z.namelist(), "columns": cols, "shapes": len(recs)}
    for c in ("iso_a2", "adm0_a3", "admin"):
        if c in cols:
            from collections import Counter
            out["distinct_" + c] = dict(Counter(r[c] for r in recs).most_common(12))
    print(json.dumps(out, indent=1))


def run(path, out, cities, areas):
    os.makedirs(out, exist_ok=True)
    log = {"plants": plants(), "file": {"bytes": os.path.getsize(path), "sha256": hashlib.sha256(open(path, "rb").read()).hexdigest()}}
    try:
        if not all(log["plants"].values()):
            raise Stop("a plant failed")
        z, shp, dbf = open_archive(path)
        cols, recs = read_dbf(z.read(dbf))
        for c in ("iso_a2", "postal", "name"):
            if c not in cols:
                raise Stop("column %s is absent" % c)
        shapes = read_shp(z.read(shp))
        if len(shapes) != len(recs):
            raise Stop("shapes and attribute rows differ in number")
        by, names = {}, {}
        for r, s in zip(recs, shapes):
            if r["iso_a2"] != "US":
                continue
            if r["postal"] not in CODES:
                raise Stop("a US shape whose postal code is not a state: %r" % r["postal"])
            code = CODES[r["postal"]]
            if code in by:
                raise Stop("a state twice: %s" % r["postal"])
            by[code], names[code] = s, r["name"]
        fine = rasterise(by)
        cls, share, classed = reduce(fine)
    except Stop as e:
        log["branch"] = "(e): %s" % e
        print(json.dumps(log, indent=1))
        return 1
    inv = {v: k for k, v in CODES.items()}
    r1 = []
    for name, lat, lon, st in cities:
        r, c = cell(lat, lon)
        got = inv.get(int(cls[r, c]), "none")
        r1.append({"place": name, "cell": r * COLS + c, "state": got, "expected": st, "met": got == st})
    hits = sum(x["met"] for x in r1)
    log["R1"] = {"met_of_%d" % len(cities): hits, "misses": [x for x in r1 if not x["met"]], "reading": "met" if hits >= len(cities) - 1 else ("misread" if hits <= 8 else "between")}
    pa = point_area_km2()
    r2 = []
    for st, ref in areas.items():
        a = float(((fine == CODES[st]).sum(axis=1) * pa).sum())
        r2.append({"state": st, "sampled_km2": round(a), "reference_km2": ref, "difference_pct": round(100 * (a - ref) / ref, 2), "met": abs(a - ref) <= 0.05 * ref})
    ok2 = sum(x["met"] for x in r2)
    log["R2"] = {"met_of_10": ok2, "states": r2, "reading": "met" if ok2 >= 9 else ("misread" if ok2 <= 5 else "between")}
    cells_by = {inv[c]: int((cls == c).sum()) for c in sorted(by)}
    missing = [s for s, n in cells_by.items() if n == 0 and s != "DC"]
    log["R3"] = {"states_with_no_cell": missing, "met": not missing and len([c for c in by if inv[c] != "DC"]) == 50, "district_of_columbia_cells": cells_by.get("DC")}
    coded = cls > 0
    best_frac = np.zeros(cls.shape)
    m = classed > 0
    best_frac[m] = share[m] / 255.0
    log["printed"] = {"us_shapes": len(by), "coded_cells": int(coded.sum()), "cells_by_state": cells_by, "share_of_coded_cells_with_winner_under_0.8": round(float(((best_frac < 0.8) & coded).sum() / coded.sum()), 4),
                      "cells_with_under_5_points_in_any_state": int(((classed > 0) & (classed < 5)).sum()), "cells_fully_inside_one_state": int(((classed == 25) & (share == 255)).sum())}
    if log["R1"]["reading"] == "misread" or log["R2"]["reading"] == "misread" or not log["R3"]["met"]:
        log["branch"] = "(r): a reading check failed; nothing written"
    else:
        files = {}
        for name, arr in (("us_state_cell", cls), ("us_state_cell_share", share), ("us_state_cell_classed", classed)):
            raw = arr.astype(np.uint8).tobytes()
            packed = gz(raw)
            open(os.path.join(out, name + ".bin.gz"), "wb").write(packed)
            files[name + ".bin.gz"] = {"bytes_packed": len(packed), "sha256_packed": hashlib.sha256(packed).hexdigest(), "bytes_unpacked": len(raw), "sha256_unpacked": hashlib.sha256(raw).hexdigest()}
        log["files"] = files
        div = {s: d for d, ss in DIVISION.items() for s in ss.split()}
        log["code_table"] = [{"code": c, "postal": inv[c], "name": names[c], "census_division_of_the_survey": div[inv[c]]} for c in sorted(by)]
        log["branch"] = "(a): the three grids are written"
    json.dump(log, open(os.path.join(out, "results.json"), "w"), indent=1)
    print(json.dumps({k: v for k, v in log.items() if k not in ("code_table",)}, indent=1)[:5000])
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-15s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    if sys.argv[1] == "header":
        header(sys.argv[2])
        sys.exit(0)
    frozen = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "frozen_checks.json")))
    sys.exit(run(sys.argv[2], sys.argv[3], [tuple(c) for c in frozen["cities"]], frozen["areas_km2"]))
