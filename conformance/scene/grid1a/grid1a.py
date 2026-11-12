#!/usr/bin/env python3
"""GRID-1A: the same Ember file read again, a row in each year from 2014 (`GRID1A_AMENDMENT.md`).

  grid1a.py plants
  grid1a.py run EMBER_CSV TABLE_V3_JSON OUT_DIR [STAMP]

Standard library only. Uses GRID-1's extractor unchanged. Writes OUT_DIR/grid_table.json
(version 4) and OUT_DIR/results.json.
"""
import copy
import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "grid1"))
import grid1  # noqa: E402

RECORDED = "08932763076fa781634371dd1b75d8156f76592710c7e984836ed56a663d5688"
FIRST = 2014
DROP = ("disagrees_with", "shares_partial", "superseded_by", "map_alternative", "same_series_as")


def reading(A, v3):
    """R2's comparison: each version-3 dataset row against the file."""
    rows = [x for x in v3["rows"] if x.get("read") == "dataset"]
    out = []
    for x in rows:
        d = A.get(x["place_as_published"], {"years": {}})["years"].get(x["year"], {})
        out.append((x, d.get("intensity")))
    return out


def r2(pairs, same):
    absent = [x["id"] for x, v in pairs if not v]
    if same:
        bad = [x["id"] for x, v in pairs if not v or v != x["value"]]
        return {"compared": len(pairs), "mode": "same file", "unequal": len(bad), "first_unequal": bad[:10], "absent_in_later_edition": absent, "met": not bad}
    moved, over10, big_f2, small_f2 = [], len(absent), [], []
    for x, v in pairs:
        if not v:
            continue
        diff = abs(v - x["value"])
        if diff > max(0.02 * v, 1.0):
            moved.append({"id": x["id"], "place": x["place"], "year": x["year"], "old": x["value"], "new": v})
        if diff > 0.10 * v:
            over10 += 1
        ratio = x["value"] / v
        if ratio < 0.5 or ratio > 2:
            (small_f2 if x.get("small_system") else big_f2).append(x["id"])
    misread = over10 > 70 or bool(big_f2) or len(small_f2) > 15
    moved.sort(key=lambda m: -abs(m["new"] - m["old"]) / m["new"])
    return {"compared": len(pairs), "mode": "another edition", "moved": len(moved), "ten_largest": moved[:10], "moved_in_later_edition": moved,
            "over_10_percent_absent_counted": over10, "factor_2_not_small": big_f2, "factor_2_small": small_f2, "absent_in_later_edition": absent,
            "misread": misread, "met": not misread}


def build(A, v3, same, digest, stamp):
    data = [x for x in v3["rows"] if x.get("read") == "dataset"]
    have = {(x["place_as_published"], x["year"]) for x in data}
    tmpl = {}
    for x in data:
        if x["latest_for_place"]:
            tmpl[x["place_as_published"]] = x
    old_ids = {x["id"] for x in v3["rows"]}
    edition = grid1.EDITION if same else "%s; digest %s" % (stamp or "no stamp; fetched 2026-11-12", digest[:8])
    new, later = [], []
    for name in sorted(tmpl):
        if name not in A:
            continue
        t, a = tmpl[name], A[name]
        ys = sorted(y for y, d in a["years"].items() if d["intensity"] and y >= FIRST)
        for y in ys:
            if same and (name, y) in have:
                continue
            d = a["years"][y]
            row = {k: v for k, v in copy.deepcopy(t).items() if k not in DROP}
            row.update({"id": "ember-data-%s-%d" % (grid1.slug(t["place"]), y) + ("" if same else "-e" + digest[:8]), "year": y, "value": d["intensity"],
                        "shares_by_source_percent": None, "year_lag": None, "latest_for_place": False, "provisional": True if d["estimated"] else None,
                        "small_system": bool(d["generation"] is not None and d["generation"] < 1.0), "edition": edition,
                        "source": "Ember, Yearly electricity data – Global (CSV); " + edition})
            if row["id"] in old_ids or any(r["id"] == row["id"] for r in new[-40:]):
                raise grid1.Stop("id collides: %s" % row["id"])
            old_ids.add(row["id"])
            if y > t["year"]:
                later.append({"place": t["place"], "year": y, "id": row["id"]})
            new.append(row)
    v4 = copy.deepcopy(v3)
    v4["rows"] = v4["rows"] + new
    return v4, new, later, [n for n in sorted(A) if n not in tmpl]


def keep4(v3, v4):
    by = {x["id"]: x for x in v4["rows"]}
    return len(by) == len(v4["rows"]) and all(x["id"] in by and by[x["id"]] == x and set(by[x["id"]]) == set(x) for x in v3["rows"])


def onelatest(v3, v4):
    was = {x["place"]: x["id"] for x in v3["rows"] if x.get("read") == "dataset" and x["latest_for_place"]}
    now = {}
    for x in v4["rows"]:
        if x.get("read") == "dataset" and x["latest_for_place"]:
            now.setdefault(x["place"], []).append(x["id"])
    return set(now) == set(was) and all(v == [was[p]] for p, v in now.items())


def core(f, v3, digest, stamp=None):
    """Returns (branch, log, v4 or None). One branch."""
    log = {}
    same = digest == RECORDED
    try:
        A = grid1.extract(f)
        pairs = reading(A, v3)
        log["R2"] = r2(pairs, same)
        want = sum(1 for x in v3["rows"] if x.get("read") == "dataset")
        if log["R2"]["compared"] != want:
            return "(e): compared %d, not %d" % (log["R2"]["compared"], want), log, None
        if same and not log["R2"]["met"]:
            return "(m): the same file no longer reads as GRID-1 read it; nothing merged", log, None
        if not same and not log["R2"]["met"]:
            return "(r): another edition and R2 reads misread; nothing merged", log, None
        v4, new, later, unwritten = build(A, v3, same, digest, stamp)
    except Exception as e:            # any stop or error of the extractor or merge
        return "(e): %s: %s" % (type(e).__name__, e), log, None
    log["P-KEEP4_on_the_merge"], log["P-ONELATEST_on_the_merge"] = keep4(v3, v4), onelatest(v3, v4)
    if not (log["P-KEEP4_on_the_merge"] and log["P-ONELATEST_on_the_merge"]):
        return "(e): P-KEEP4 or P-ONELATEST failed on the merge", log, None
    data = [x for x in v4["rows"] if x.get("read") == "dataset"]
    ed = lambda x: x["edition"]
    eds = sorted({ed(x) for x in data})
    yrs = {}
    for x in data:
        if same or x["edition"] != grid1.EDITION:
            yrs.setdefault(x["place"], set()).add(x["year"])
    places = sorted({x["place"] for x in data})
    log["printed"] = {"new_rows": len(new), "editions": eds, "places": len(places), "with_2019_and_2024": sum(1 for p in places if {2019, 2024} <= yrs.get(p, set())),
                      "without_2024": sorted(p for p in places if 2024 not in yrs.get(p, set())), "later_year_in_later_edition": later,
                      "areas_in_the_file_not_written": unwritten}
    return "(a): version 4 written", log, v4


# ---------------------------------------------------------------- plants

def table_from(rows_spec):
    """A small version-3-shaped table: rows_spec = [(file name, table place, year, value, latest)]."""
    return {"rows": [{"id": "ember-data-%s-%d" % (grid1.slug(p), y), "place": p, "place_as_published": n, "place_kind": "country", "year": y, "value": v, "read": "dataset",
                      "latest_for_place": lat, "year_lag": 0 if lat else None, "kind": "K", "trust": "B", "edition": grid1.EDITION, "small_system": False,
                      "shares_by_source_percent": {"Coal": 60.0} if lat else None, "source": "s", "iso3": "AAA", "provisional": None}
                     for n, p, y, v, lat in rows_spec] + [{"id": "doc-1", "place": "Aland", "year": 2024, "value": 1.0, "read": "page", "trust": "A", "kind": "direct"}]}


def synth_file(spec, shift=0):
    rows = []
    for name, years in spec.items():
        for y, v in years.items():
            rows += grid1.area(name, "AAA", "Country or economy", y + shift, v)
    return grid1.synth(rows)


def plants():
    res = {}
    yrs = {y: 100.0 + y - 2014 for y in range(2010, 2025)}
    yrs[2017] = ""
    spec = {"Aland": yrs, "Bland": {y: 50.0 for y in range(2014, 2025)}}
    v3 = table_from([("Aland", "Aland", 2014, 100.0, False), ("Aland", "Aland", 2019, 105.0, False), ("Aland", "Aland", 2024, 110.0, True), ("Bland", "Bland", 2024, 50.0, True)])
    b, log, v4 = core(synth_file(spec), v3, RECORDED)
    new = [x for x in (v4 or {"rows": []})["rows"] if x["id"] not in {r["id"] for r in v3["rows"]}]
    ay = sorted(x["year"] for x in new if x["place"] == "Aland")
    res["P-YEARS"] = b.startswith("(a)") and ay == [2015, 2016, 2018, 2020, 2021, 2022, 2023]
    res["P-NODUP"] = b.startswith("(a)") and len({x["id"] for x in v4["rows"]}) == len(v4["rows"]) and not any(x["year"] in (2014, 2019, 2024) for x in new if x["place"] == "Aland")
    res["P-KEEP4"] = b.startswith("(a)") and keep4(v3, v4) and not keep4(v3, {"rows": [dict(x, extra=1) if x["id"] == "doc-1" else x for x in v4["rows"]]})
    res["P-ONELATEST"] = b.startswith("(a)") and onelatest(v3, v4) and not any(x["latest_for_place"] or x["year_lag"] is not None or x["shares_by_source_percent"] for x in new)
    spec2 = copy.deepcopy(spec)
    spec2["Aland"][2025] = 111.0
    b2, log2, w4 = core(synth_file(spec2), v3, "ab" * 32, "stamp X")
    n2 = [x for x in (w4 or {"rows": []})["rows"] if x["id"].endswith("-eabababab")]
    res["P-EDITION"] = (b2.startswith("(a)") and sorted(x["year"] for x in n2 if x["place"] == "Aland") == [y for y in range(2014, 2026) if y != 2017]
                        and all("abababab" in x["edition"] for x in n2) and onelatest(v3, w4) and keep4(v3, w4)
                        and log2["printed"]["later_year_in_later_edition"] == [{"place": "Aland", "year": 2025, "id": "ember-data-aland-2025-eabababab"}])
    spec3 = {"Aland": {y: v for y, v in yrs.items() if y != 2019}}          # Bland gone, Aland 2019 gone
    b3, log3, _ = core(synth_file(spec3), v3, RECORDED)
    res["P-ABSENT"] = b3.startswith("(m)") and len(log3["R2"]["absent_in_later_edition"]) == 2
    b4, log4, _ = core(synth_file(spec), v3, "cd" * 32)
    b5, log5, _ = core(synth_file({"Aland": {y: (300.0 if y % 2 else 20.0) for y in range(2010, 2026)}, "Bland": {y: 50.0 for y in range(2014, 2025)}}, shift=0),
                       table_from([("Aland", "Aland", 2014, 300.0, False), ("Aland", "Aland", 2019, 20.0, False), ("Aland", "Aland", 2024, 300.0, True), ("Bland", "Bland", 2024, 50.0, True)]), "cd" * 32)
    res["P-SAME"] = b4.startswith("(a)") and log4["R2"]["moved"] == 0 and b5.startswith("(r)")   # the second is the first table read one year off
    res["P-GRID1"] = all(grid1.plants().values())
    return res


def run(csv_path, v3_path, out_dir, stamp=None):
    os.makedirs(out_dir, exist_ok=True)
    log = {"plants": plants()}
    raw = open(csv_path, "rb").read()
    digest = hashlib.sha256(raw).hexdigest()
    log["file"] = {"bytes": len(raw), "sha256": digest, "same_as_recorded": digest == RECORDED, "stamp": stamp}
    if not all(log["plants"].values()):
        log["branch"] = "(e): a plant failed"
    else:
        v3 = json.load(open(v3_path, encoding="utf-8"))
        log["branch"], more, v4 = core(open(csv_path, newline="", encoding="utf-8-sig"), v3, digest, stamp)
        log.update(more)
        if v4 is not None:
            json.dump(v4, open(os.path.join(out_dir, "grid_table.json"), "w", encoding="utf-8"), indent=1, ensure_ascii=False)
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
    short = dict(log)
    if "R2" in short:
        short["R2"] = {k: v for k, v in short["R2"].items() if k != "moved_in_later_edition"}
    print(json.dumps(short, indent=1)[:6000])
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-12s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(*sys.argv[2:6]))
