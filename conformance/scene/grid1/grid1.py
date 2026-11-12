#!/usr/bin/env python3
"""GRID-1: Ember's yearly electricity data read into GRID-0's table (`GRID1_PREREG.md`).

  grid1.py plants
  grid1.py run EMBER_CSV TABLE_V2_JSON OUT_DIR

Standard library only. Writes OUT_DIR/grid_table.json (version 3) and OUT_DIR/results.json.
"""
import copy
import csv
import io
import json
import os
import re
import sys

INT_COL = "Emissions intensity (gCO2e/kWh)"
SOURCE_URL = "https://files.ember-energy.org/public-downloads/generation/outputs/release_generation_yearly_global.csv"
PAGE_URL = "https://ember-energy.org/data/yearly-electricity-data/"
EDITION = "file stamped 2026-11-03; data page says last updated 2026-11-11"
# Fixed in the record before the parse, from the file's labels alone.
NAME_MAP = {"EU": "EU-27", "Latin America and Caribbean": "Latin America", "United States of America": "United States"}
GROUPING = {"World": "geographic", "Africa": "geographic", "Asia": "geographic", "Europe": "geographic", "Latin America and Caribbean": "geographic",
            "Middle East": "geographic", "North America": "geographic", "Oceania": "geographic", "MENA": "geographic",
            "EU": "political", "AU": "political", "ASEAN": "political", "G20": "economic", "G7": "economic", "OECD": "economic"}
ADDED = ("superseded_by", "map_alternative", "same_series_as", "disagrees_with")
R1_IDS = ["ember-2025-world", "ember-2025-china", "ember-2025-united-states", "ember-2025-eu-27", "ember-2025-india", "ember-2025-russia", "ember-2025-japan",
          "ember-2025-brazil", "ember-2025-asia", "ember-2025-latin-america", "ember-2024-world", "ember-2024-china", "ember-2024-united-states", "ember-2024-eu-27",
          "ember-2024-india", "ember-2024-russia", "ember-2024-japan", "ember-2024-brazil", "ember-2015-world", "ember-2023-united-states-rev", "ember-2007-united-states"]
S1A_IDS = ["gc-ember2025-norway", "gc-ember2025-sweden", "gc-ember2025-canada", "gc-ember2025-mexico", "gc-ember2025-indonesia"]
S1B_IDS = ["statista-ember-2023-poland", "statista-ember-2023-sweden", "statista-ember-2023-france"]
S2_REPRO = ["egrid2023-us-total", "fr-rte-2025-direct"]
S2_DOUBT = ["ca-nir-2023-canada-gen", "za-dffe-2024-dggef", "cn-mee-2022-national", "in-cea-fy2022-23-avg-incl-res", "de-uba-2025-co2-strommix"]


class Stop(Exception):
    pass


def num(s):
    try:
        v = float(s)
    except (TypeError, ValueError):
        return None
    return v if v == v else None


def extract(f):
    """{area: {"type", "iso3", "years": {year: {"intensity", "generation", "shares", "estimated"}}}}"""
    r = csv.DictReader(f)
    cols = r.fieldnames or []
    ic = [c for c in cols if c.lower().startswith("emissions intensity")]
    if not ic:
        raise Stop("no carbon-intensity series")
    if ic[0] != INT_COL:
        raise Stop("intensity in another unit: %s" % ic[0])
    est = next((c for c in cols if "estimat" in c.lower()), None)
    A, seen = {}, set()
    for x in r:
        k = (x["Area"], x["Year"], x["Electricity source"])
        if k in seen:
            raise Stop("repeated row: %s" % (k,))
        seen.add(k)
        a = A.setdefault(x["Area"], {"type": x["Area type"], "iso3": x.get("ISO 3 code") or None, "years": {}})
        y = a["years"].setdefault(int(x["Year"]), {"intensity": None, "generation": None, "shares": {}, "estimated": False})
        if est and x[est].strip().lower() in ("true", "1", "yes"):
            y["estimated"] = True
        if x["Electricity source"] == "Total generation":
            v = num(x[INT_COL])
            y["intensity"] = v if v else None          # blank, zero or not a number: a gap
            y["generation"] = num(x["Generation (TWh)"])
        elif x["Is aggregated source"] == "False" and x["Electricity source"] not in ("Demand", "Net imports", "Total generation"):
            s = num(x["Share of generation (%)"])
            if s is not None:
                y["shares"][x["Electricity source"].lower()] = s
    for name, a in A.items():
        if a["type"] == "Region" and name not in GROUPING:
            raise Stop("a grouping with no entry in the map: %s" % name)
        if a["type"] not in ("Region", "Country or economy"):
            raise Stop("unknown type of area: %s" % a["type"])
    return A


def slug(s):
    return re.sub(r"[^a-z0-9]+", "-", s.lower()).strip("-")


def latest_year(A):
    countries = [a for a in A.values() if a["type"] != "Region"]
    for y in sorted({y for a in countries for y in a["years"]}, reverse=True):
        if sum(1 for a in countries if a["years"].get(y, {}).get("intensity")) * 2 >= len(countries):
            return y
    raise Stop("no year with half the countries")


def build(A, v2):
    """New rows, and version 3 of the table."""
    fly = latest_year(A)
    kind = next((x["kind"] for x in v2["rows"] if x["id"] == "ember-2025-world"), "generation average, life-cycle (Ember)")
    by_place = {NAME_MAP.get(n, n): n for n in A}
    want = {}
    for name, a in A.items():
        ys = [y for y, d in a["years"].items() if d["intensity"]]
        if not ys:
            continue
        want[name] = {max(ys)} | {y for y in (fly - 5, fly - 10) if a["years"].get(y, {}).get("intensity")}
    for x in v2["rows"]:                                   # a row for every place and year a stake reads
        if x["id"] in R1_IDS + S1A_IDS + S1B_IDS + S2_REPRO + S2_DOUBT or x["id"].startswith("ember-"):
            n = by_place.get(x["place"])
            for y in (x["year"], x["year"] - 1) if x["id"] in S1A_IDS else (x["year"],):
                if n and A[n]["years"].get(y, {}).get("intensity"):
                    want.setdefault(n, set()).add(y)
    old_ids = {x["id"] for x in v2["rows"]}
    new, index = [], {}
    for name in sorted(want):
        a = A[name]
        place = NAME_MAP.get(name, name)
        lat = max(y for y, d in a["years"].items() if d["intensity"])
        for y in sorted(want[name]):
            d = a["years"][y]
            is_region = a["type"] == "Region"
            row = {"id": "ember-data-%s-%d" % (slug(place), y), "place": place, "place_kind": "world" if name == "World" else ("region" if is_region else "country"),
                   "parent": None, "year": y, "value": d["intensity"], "unit": "gCO2e/kWh", "kind": kind, "original_value": None,
                   "shares_by_source_percent": None, "source": "Ember, Yearly electricity data – Global (CSV); " + EDITION, "url": SOURCE_URL, "read": "dataset",
                   "licence": "CC BY 4.0", "trust": "B", "trust_reason": "read at source from the publisher's dataset; an analyst's estimate from generation and life-cycle factors, not an official inventory",
                   "note": "Data page: " + PAGE_URL, "edition": EDITION, "iso3": a["iso3"], "place_as_published": name,
                   "year_lag": fly - y if y == lat else None, "latest_for_place": y == lat, "provisional": True if d["estimated"] else None,
                   "small_system": bool(d["generation"] is not None and d["generation"] < 1.0)}
            if is_region:
                row["grouping"] = GROUPING[name]
            if y == lat and d["shares"]:
                row["shares_by_source_percent"] = dict(sorted(d["shares"].items()))
                if abs(sum(d["shares"].values()) - 100) > 1:
                    row["shares_partial"] = True
            if row["id"] in old_ids or row["id"] in index:
                raise Stop("id collides: %s" % row["id"])
            index[(place, y)] = row
            new.append(row)
    latest = {r["place"]: r for r in new if r["latest_for_place"]}
    v3 = copy.deepcopy(v2)
    for x in v3["rows"]:
        if x["trust"] == "A":
            continue
        hit = index.get((x["place"], x["year"]))
        says_ember = x["id"] in S1A_IDS + S1B_IDS
        if x["trust"] == "C" and says_ember and hit:
            x["superseded_by"] = hit["id"]
        elif x["trust"] == "C" and x["place"] in latest and x["place_kind"] != "subregion":
            x["map_alternative"] = latest[x["place"]]["id"]
        elif x["id"].startswith("ember-") and x["read"] == "page" and hit:
            x["same_series_as"] = hit["id"]
            if abs(x["value"] - hit["value"]) > 0.5:
                x["disagrees_with"] = "%s: the file reads %.2f, the page %s (%+.1f %%)" % (hit["id"], hit["value"], x["value"], 100 * (x["value"] - hit["value"]) / hit["value"])
                hit.setdefault("disagrees_with", "%s: the page reads %s" % (x["id"], x["value"]))
    v3["rows"] = v3["rows"] + new
    return v3, new, index, fly


def keep_check(v2, v3):
    by = {x["id"]: x for x in v3["rows"]}
    for x in v2["rows"]:
        y = by.get(x["id"])
        if y is None or any(y.get(k) != v for k, v in x.items()) or any(k not in x and k not in ADDED for k in y):
            return False
    return True


def stakes(v2, index):
    rows = {x["id"]: x for x in v2["rows"]}
    out = {}

    def cmp(i, year=None):
        x = rows[i]
        h = index.get((x["place"], year if year is not None else x["year"]))
        if not h:
            return {"id": i, "place": x["place"], "table": x["value"], "file": None, "absent": True}
        return {"id": i, "place": x["place"], "year": h["year"], "table": x["value"], "file": round(h["value"], 2),
                "difference_percent": round(100 * (x["value"] - h["value"]) / h["value"], 2), "difference_g": round(x["value"] - h["value"], 2)}
    within = lambda c, pct, g: (not c.get("absent")) and abs(c["difference_g"]) <= max(pct / 100 * c["file"], g)
    r1 = [cmp(i) for i in R1_IDS]
    off = [c for c in r1 if not within(c, 2, 1)]
    big = [c for c in off if c.get("absent") or abs(c["difference_percent"]) > 10]
    two = [c for c in off if not c.get("absent") and not (0.5 <= c["table"] / c["file"] <= 2)]
    reading = "misread" if (len(big) >= 5 or two) else ("later edition" if len(off) > 3 else "met")
    out["R1"] = {"rows": r1, "outside_2_percent_or_1_g": [c["id"] for c in off], "reading": reading,
                 "earlier_method_rows_no_verdict": [cmp(i) for i in rows if i.startswith("ember-") and i.endswith("-ed2024")]}
    a25 = [cmp(i) for i in S1A_IDS]
    a24 = [cmp(i, rows[i]["year"] - 1) for i in S1A_IDS]
    n25, n24 = sum(within(c, 5, 2) for c in a25), sum(within(c, 5, 2) for c in a24)
    out["S1a"] = {"against_2025": a25, "against_2024_printed": a24, "within_2025": n25, "within_2024": n24,
                  "outcome": "met" if n25 >= 4 else ("killed" if (n25 <= 2 and n24 <= 2) else "neither")}
    out["S1b_no_verdict"] = [cmp(i) for i in S1B_IDS]

    def lower(i):
        c = cmp(i)
        c["file_minus_official_g"] = None if c.get("absent") else round(c["file"] - c["table"], 2)
        c["file_is_lower"] = bool(c.get("absent") or (c["table"] - c["file"] > max(0.02 * c["file"], 3)))
        return c
    doubt = [lower(i) for i in S2_DOUBT]
    nl = sum(c["file_is_lower"] for c in doubt)
    out["S2"] = {"reproduction": [lower(i) for i in S2_REPRO], "in_doubt": doubt, "file_lower_in": nl, "outcome": "killed" if nl >= 2 else "met"}
    return out


# ---------------------------------------------------------------- plants

HEAD = ["Area", "ISO 3 code", "Year", "Area type", "Electricity source", "Is aggregated source", "Generation (TWh)", "Share of generation (%)", INT_COL]


def synth(rows, head=HEAD, extra=None):
    b = io.StringIO()
    w = csv.writer(b)
    w.writerow(head + ([extra] if extra else []))
    for r in rows:
        w.writerow(r)
    b.seek(0)
    return b


def area(name, iso, typ, year, inten, gen=100.0, shares=(("Coal", 60.0), ("Wind", 40.0)), est=None):
    rows = [[name, iso, year, typ, "Total generation", "True", gen, 100.0, inten] + ([est] if est is not None else [])]
    rows += [[name, iso, year, typ, s, "False", gen * p / 100, p, ""] + ([est] if est is not None else []) for s, p in shares]
    rows += [[name, iso, year, typ, "Fossil", "True", gen * 0.6, 60.0, ""] + ([est] if est is not None else [])]
    return rows


def plants():
    v2 = {"rows": [{"id": "ember-2025-world", "place": "World", "place_kind": "world", "year": 2025, "value": 458.0, "kind": "K", "trust": "B", "read": "page"},
                   {"id": "gc-ember2025-norway", "place": "Norway", "place_kind": "country", "year": 2025, "value": 28.0, "kind": "said", "trust": "C", "read": "summary"},
                   {"id": "lcp-norway", "place": "Norway", "place_kind": "country", "year": 2026, "value": 32.0, "kind": "consumption", "trust": "C", "read": "summary"},
                   {"id": "official-a", "place": "Norway", "place_kind": "country", "year": 2025, "value": 20.0, "kind": "direct", "trust": "A", "read": "page"}]}
    base = []
    for y in (2015, 2020, 2025):
        base += area("World", "", "Region", y, 450.0 + (2025 - y)) + area("Norway", "NOR", "Country or economy", y, 28.4 + (2025 - y), gen=150.0)
        base += area("United States of America", "USA", "Country or economy", y, 380.0) + area("EU", "", "Region", y, 210.0)
    base += area("Tinyland", "TNY", "Country or economy", 2024, 500.0, gen=0.4, shares=(("Coal", 60.0), ("Wind", 30.0)))
    base += area("Tinyland", "TNY", "Country or economy", 2025, "", gen=0.4)
    res = {}
    A = extract(synth(base))
    v3, new, index, fly = build(A, v2)
    by = {r["id"]: r for r in new}
    res["P-UNIT"] = fly == 2025 and by["ember-data-norway-2025"]["value"] == 28.4 and by["ember-data-norway-2015"]["value"] == 38.4 and \
        by["ember-data-norway-2025"]["shares_by_source_percent"] == {"coal": 60.0, "wind": 40.0}
    res["P-AGG"] = by["ember-data-world-2025"]["place_kind"] == "world" and by["ember-data-eu-27-2025"]["place_kind"] == "region" and \
        all(r["place_kind"] == "country" for r in new if r["place"] in ("Norway", "Tinyland", "United States"))
    res["P-GAP"] = "ember-data-tinyland-2025" not in by and by["ember-data-tinyland-2024"]["year_lag"] == 1 and by["ember-data-tinyland-2024"]["small_system"] is True
    for bad in ("0", "n/a"):
        A2 = extract(synth(base + area("Gapland", "GAP", "Country or economy", 2024, 300.0) + area("Gapland", "GAP", "Country or economy", 2025, bad)))
        _, n2, _, _ = build(A2, v2)
        res["P-GAP"] = res["P-GAP"] and any(r["id"] == "ember-data-gapland-2024" and r["year_lag"] == 1 for r in n2) and not any(r["id"] == "ember-data-gapland-2025" for r in n2)
    A3 = extract(synth([r for a in (area("Estland", "EST", "Country or economy", 2025, 100.0, est="True"), area("World", "", "Region", 2025, 450.0, est="False")) for r in a], extra="Is estimated"))
    _, n3, _, _ = build(A3, v2)
    res["P-EST"] = any(r["id"] == "ember-data-estland-2025" and r["provisional"] is True for r in n3) and any(r["id"] == "ember-data-world-2025" and r["provisional"] is None for r in n3)
    res["P-SHARE"] = by["ember-data-tinyland-2024"].get("shares_partial") is True and "shares_partial" not in by["ember-data-norway-2025"] and \
        "fossil" not in by["ember-data-norway-2025"]["shares_by_source_percent"]
    ok = by["ember-data-united-states-2025"]["place"] == "United States" and by["ember-data-eu-27-2025"]["place_as_published"] == "EU"
    try:
        extract(synth(base + area("Atlantis Union", "", "Region", 2025, 1.0)))
        ok = False
    except Stop:
        pass
    res["P-NAME"] = ok
    try:
        extract(synth(base + area("Norway", "NOR", "Country or economy", 2025, 30.0)))
        res["P-DUP"] = False
    except Stop:
        res["P-DUP"] = True
    ids = [r["id"] for r in v3["rows"]]
    ok = len(ids) == len(set(ids))
    try:
        build(A, {"rows": v2["rows"] + [dict(v2["rows"][0], id="ember-data-norway-2025")]})
        ok = False
    except Stop:
        pass
    res["P-ID"] = ok
    try:
        extract(synth(base, head=HEAD[:-1] + ["Emissions intensity (kgCO2e/kWh)"]))
        res["P-UNITCHK"] = False
    except Stop:
        res["P-UNITCHK"] = True
    old = {x["id"]: x for x in v3["rows"]}
    res["P-KEEP"] = keep_check(v2, v3) and old["gc-ember2025-norway"].get("superseded_by") == "ember-data-norway-2025" and \
        old["lcp-norway"].get("map_alternative") == "ember-data-norway-2025" and old["ember-2025-world"].get("same_series_as") == "ember-data-world-2025" and \
        "disagrees_with" in old["ember-2025-world"] and not any(k in old["official-a"] for k in ADDED) and \
        not keep_check(v2, {"rows": [dict(x, value=1.0) if x["id"] == "lcp-norway" else x for x in v3["rows"]]})
    res["P-LONG"] = True   # every synthetic file above is in the layout the header shows (the columns this script reads)
    return res


def run(csv_path, v2_path, out_dir):
    os.makedirs(out_dir, exist_ok=True)
    log = {"plants": plants()}
    if not all(log["plants"].values()):
        log["branch"] = "(e): a plant failed"
    else:
        v2 = json.load(open(v2_path, encoding="utf-8"))
        try:
            A = extract(open(csv_path, newline="", encoding="utf-8-sig"))
            v3, new, index, fly = build(A, v2)
        except Stop as e:
            log["branch"] = "(e): " + str(e)
            json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
            print(log["branch"])
            return 1
        log.update(stakes(v2, index))
        log["P-KEEP_on_the_real_merge"] = keep_check(v2, v3)
        lat = [r for r in new if r["latest_for_place"] and r["place_kind"] == "country"]
        zero = sorted(r["value"] for r in lat if r["year_lag"] == 0)
        pc = lambda q: zero[min(int(q * len(zero)), len(zero) - 1)]
        onlyc = ["Australia", "Indonesia", "Italy", "Kenya", "Mexico", "Netherlands", "Nigeria", "Norway", "Poland", "South Korea", "Spain", "Sweden"]
        have = {r["place"] for r in lat}
        log["S3"] = {"file_latest_year": fly, "new_rows": len(new), "countries_with_a_row": len(lat), "at_lag_zero": len(zero),
                     "lagging": sorted((r["place"], r["year"]) for r in lat if r["year_lag"]), "provisional": sum(1 for r in lat if r["provisional"]),
                     "small_systems": sum(1 for r in lat if r["small_system"]), "p10_p90_at_lag_zero": [pc(0.1), pc(0.9)], "ratio_p90_to_p10": pc(0.9) / pc(0.1),
                     "lowest_and_highest_at_lag_zero_not_small": [(r["place"], round(r["value"], 1)) for r in sorted((r for r in lat if r["year_lag"] == 0 and not r["small_system"]), key=lambda r: r["value"])[:3]] +
                     [(r["place"], round(r["value"], 1)) for r in sorted((r for r in lat if r["year_lag"] == 0 and not r["small_system"]), key=lambda r: r["value"])[-3:]],
                     "of_the_12_with_only_C_rows_now_at_B": [p for p in onlyc if p in have], "shares_partial": sum(1 for r in new if r.get("shares_partial")),
                     "fields_added_to_version_2_rows": {k: sum(1 for x in v3["rows"] if k in x and x.get("read") != "dataset") for k in ADDED}}
        if log["R1"]["reading"] == "misread" or not log["P-KEEP_on_the_real_merge"]:
            log["branch"] = "(r): R1 reads misread; nothing merged" if log["R1"]["reading"] == "misread" else "(e): P-KEEP failed on the real merge"
        else:
            log["branch"] = "(a): merged as version 3"
            json.dump(v3, open(os.path.join(out_dir, "grid_table.json"), "w", encoding="utf-8"), indent=1, ensure_ascii=False)
    json.dump(log, open(os.path.join(out_dir, "results.json"), "w"), indent=1)
    print(json.dumps(log, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-10s %s" % (k, "PASS" if v else "FAIL"))
        sys.exit(0 if all(r.values()) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3], sys.argv[4]))
