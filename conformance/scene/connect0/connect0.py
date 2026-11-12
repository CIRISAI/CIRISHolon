#!/usr/bin/env python3
"""CONNECT-0: home connectivity tables (`CONNECT0_PREREG.md`, `BEFORE_THE_PARSE.md`).

  connect0.py plants
  connect0.py run RAW_DIR OUT_DIR

Standard library only. Reads RAW_DIR/isoc_ci_in_h.tsv, isoc_r_iacc_h.tsv, b28002.dat.
The ITU units were not obtained; nothing here reads them.
"""
import hashlib
import io
import json
import math
import os
import sys
from decimal import Decimal
from fractions import Fraction

FIRST = 2014
GROUPS = {"EA", "EU15", "EU25", "EU27_2007", "EU27_2020", "EU28"}
ISO = {"AL": "ALB", "AT": "AUT", "BA": "BIH", "BE": "BEL", "BG": "BGR", "CH": "CHE", "CY": "CYP", "CZ": "CZE", "DE": "DEU", "DK": "DNK", "EE": "EST", "EL": "GRC",
       "ES": "ESP", "FI": "FIN", "FR": "FRA", "HR": "HRV", "HU": "HUN", "IE": "IRL", "IS": "ISL", "IT": "ITA", "LT": "LTU", "LU": "LUX", "LV": "LVA", "ME": "MNE",
       "MK": "MKD", "MT": "MLT", "NL": "NLD", "NO": "NOR", "PL": "POL", "PT": "PRT", "RO": "ROU", "RS": "SRB", "SE": "SWE", "SI": "SVN", "SK": "SVK", "TR": "TUR",
       "UK": "GBR", "XK": "XKX", "LI": "LIE"}
EU27 = ["AT", "BE", "BG", "CY", "CZ", "DE", "DK", "EE", "EL", "ES", "FI", "FR", "HR", "HU", "IE", "IT", "LT", "LU", "LV", "MT", "NL", "PL", "PT", "RO", "SE", "SI", "SK"]
FIPS = {"01": "Alabama", "02": "Alaska", "04": "Arizona", "05": "Arkansas", "06": "California", "08": "Colorado", "09": "Connecticut", "10": "Delaware",
        "11": "District of Columbia", "12": "Florida", "13": "Georgia", "15": "Hawaii", "16": "Idaho", "17": "Illinois", "18": "Indiana", "19": "Iowa", "20": "Kansas",
        "21": "Kentucky", "22": "Louisiana", "23": "Maine", "24": "Maryland", "25": "Massachusetts", "26": "Michigan", "27": "Minnesota", "28": "Mississippi",
        "29": "Missouri", "30": "Montana", "31": "Nebraska", "32": "Nevada", "33": "New Hampshire", "34": "New Jersey", "35": "New Mexico", "36": "New York",
        "37": "North Carolina", "38": "North Dakota", "39": "Ohio", "40": "Oklahoma", "41": "Oregon", "42": "Pennsylvania", "44": "Rhode Island", "45": "South Carolina",
        "46": "South Dakota", "47": "Tennessee", "48": "Texas", "49": "Utah", "50": "Vermont", "51": "Virginia", "53": "Washington", "54": "West Virginia",
        "55": "Wisconsin", "56": "Wyoming", "72": "Puerto Rico"}


class Stop(Exception):
    pass


def cell(c):
    """'94.7 b' -> (Decimal, 'b'); ':' or blank or letters alone -> (None, flags)."""
    parts = c.strip().split()
    val, flags = None, ""
    for p in parts:
        try:
            val = Decimal(p)
        except Exception:
            flags += "".join(ch for ch in p if ch.isalpha())
    return val, flags


def eurostat(f, want):
    """{sel_key: {geo: {year: (Decimal, flags)}}}; `want` = fixed labels by dimension name, others kept in sel_key."""
    lines = f.read().split("\n")
    head = lines[0].split("\t")
    dims = head[0].split("\\")[0].split(",")
    if dims[-1] != "geo" or "unit" not in dims:
        raise Stop("layout: %s" % head[0])
    years = [int(h.strip()) for h in head[1:]]
    out, seen = {}, set()
    for line in lines[1:]:
        if not line.strip():
            continue
        cols = line.split("\t")
        key = dict(zip(dims, cols[0].split(",")))
        if key["unit"] != want["unit"]:
            continue
        if tuple(cols[0].split(",")) in seen:
            raise Stop("repeated row: %s" % cols[0])
        seen.add(tuple(cols[0].split(",")))
        sel = key.get("hhtyp", "TOTAL")
        for y, c in zip(years, cols[1:]):
            v, fl = cell(c)
            if v is None:
                continue
            if v < 0 or v > 100:
                raise Stop("share outside 0 to 100: %s %d %s" % (cols[0], y, v))
            out.setdefault(sel, {}).setdefault(key["geo"], {})[y] = (v, fl)
    return out


def euro_rows(series, table, regional):
    rows = []
    for geo in sorted(series):
        if geo in GROUPS:
            kind_place = "grouping"
        elif len(geo) == 2:
            kind_place = "country"
        else:
            kind_place = "region"
        if regional and kind_place != "region":
            continue
        if not regional and kind_place == "country" and geo not in ISO:
            raise Stop("a country code not in the name map: %s" % geo)
        seg = 0
        for y in sorted(series[geo]):
            v, fl = series[geo][y]
            if "b" in fl:
                seg += 1
            if y < FIRST:
                continue
            r = {"id": "eurostat-%s-%s-%d" % (table, geo.lower(), y), "place_code": geo, "place_kind": kind_place, "year": y, "kind": "K1", "value": float(v), "unit": "% of households",
                 "definition": "households with internet access at home, all forms of internet use (Eurostat)", "denominator": "private households with at least one member aged 16 to 74",
                 "read": "dataset", "table": table, "estimated": True if "e" in fl else "not flagged in the file", "break_in_series": "b" in fl, "low_reliability": "u" in fl, "series_segment": seg}
            if kind_place == "country":
                r["iso3"] = ISO[geo]
            if kind_place == "region":
                r["nuts_level"] = len(geo) - 2
                r["country_code"] = geo[:2]
                r["nuts_edition"] = "not stated by the file"
            rows.append(r)
    return rows


def r2(series):
    tot = series.get("TOTAL", {})
    eu = tot.get("EU27_2020", {}).get(2025)
    out = {"eu27_2025": str(eu[0]) if eu else None}
    if not eu:
        out["reading"] = "misread"
        return out
    members = [tot.get(m, {}).get(2025) for m in EU27]
    present = [m[0] for m in members if m]
    out["members_present"] = len(present)
    rounds = eu[0].quantize(Decimal("0.1")) == Decimal("94.7")
    between = bool(present) and min(present) <= eu[0] <= max(present)
    out["member_min_max"] = [str(min(present)), str(max(present))] if present else None
    neigh = {}
    for g in ("EA", "EU27_2007", "EU28"):
        v = tot.get(g, {}).get(2025)
        neigh["geo=" + g] = str(v[0]) if v else None
    for h in sorted(series):
        if h != "TOTAL":
            v = series[h].get("EU27_2020", {}).get(2025)
            neigh["hhtyp=" + h] = str(v[0]) if v else None
    out["neighbours_2025"] = neigh
    out["a_neighbour_also_rounds_to_94.7"] = sorted(k for k, v in neigh.items() if v and Decimal(v).quantize(Decimal("0.1")) == Decimal("94.7"))
    v23 = tot.get("EU27_2020", {}).get(2023)
    out["eu27_2023_printed_no_verdict"] = str(v23[0]) if v23 else None
    out["reading"] = "met" if (rounds and between and len(present) == 27) else "misread"
    return out


def r2b(country, regional):
    tot, reg = country.get("TOTAL", {}), regional.get("TOTAL", {})
    comp = []
    for g in sorted(tot):
        if g in GROUPS or g not in reg:
            continue
        ys = [y for y in tot[g] if y >= FIRST and y in reg[g]]
        if ys:
            y = max(ys)
            comp.append((g, y, tot[g][y][0], reg[g][y][0]))
    ok = [c for c in comp if abs(c[2] - c[3]) <= Decimal("0.1")]
    return {"countries_compared": len(comp), "within_0.1": len(ok), "not_within": [(g, y, str(a), str(b)) for g, y, a, b in comp if abs(a - b) > Decimal("0.1")],
            "met": bool(comp) and len(ok) * 10 >= len(comp) * 9}


def acs(f):
    lines = f.read().split("\n")
    head = lines[0].strip().split("|")
    need = ["B28002_E%03d" % i for i in range(1, 14)] + ["B28002_M%03d" % i for i in range(1, 14)]
    if any(n not in head for n in need):
        raise Stop("ACS columns missing")
    ix = {n: head.index(n) for n in need}
    out = {}
    for line in lines[1:]:
        c = line.strip().split("|")
        if not c[0].startswith("0400000US"):
            continue
        fips = c[0][9:]
        if fips in out:
            raise Stop("repeated state: %s" % fips)
        if fips not in FIPS:
            raise Stop("a state code not in the name map: %s" % fips)
        out[fips] = {n: int(c[ix[n]]) for n in need}
    return out


def r3(states):
    bad = []
    for s, d in states.items():
        e = lambda i: d["B28002_E%03d" % i]
        if not (e(2) + e(12) + e(13) == e(1) and e(7) <= e(4) <= e(2) and e(1) > 0):
            bad.append(s)
    return {"areas": len(states), "failing": bad, "met": len(states) == 52 and not bad}


def acs_rows(states, year):
    rows = []
    for s in sorted(states):
        d = states[s]
        e = lambda i: d["B28002_E%03d" % i]
        m = lambda i: (d["B28002_M%03d" % i] if d["B28002_M%03d" % i] >= 0 else None)
        sh = lambda n: Fraction(100 * n, e(1))
        r = {"id": "acs-b28002-%d-%s" % (year, s), "place": FIPS[s], "state_code": s, "place_kind": "state", "year": year, "read": "dataset", "households": e(1), "households_moe": m(1),
             "shares_percent": {"K1_any_access (lines 002+012)": round(float(sh(e(2) + e(12))), 2), "any_subscription (002)": round(float(sh(e(2))), 2),
                                "K2_cable_fibre_dsl (007)": round(float(sh(e(7))), 2), "cellular_only (006)": round(float(sh(e(6))), 2), "satellite (009)": round(float(sh(e(9))), 2),
                                "access_without_subscription (012)": round(float(sh(e(12))), 2), "no_access (013)": round(float(sh(e(13))), 2)},
             "counts": {"%03d" % i: e(i) for i in range(1, 14)}, "counts_moe": {"%03d" % i: m(i) for i in range(1, 14)},
             "moe_note": "margins are the file's, for counts (90 % level); no margin is derived for a share",
             "definition": "American Community Survey 1-year, table B28002, Presence and Types of Internet Subscriptions in Household; universe: households"}
        r["_gap"] = sh(e(2) + e(12)) - sh(e(7))
        r["_cell"] = sh(e(6))
        rows.append(r)
    return rows


def s2(rows):
    gaps = sorted(r["_gap"] for r in rows)
    over = sum(1 for g in gaps if g > 5)
    n = len(gaps)
    p10, p90 = gaps[math.ceil(0.1 * n) - 1], gaps[math.ceil(0.9 * n) - 1]
    spread = p90 - p10
    cells = sorted(r["_cell"] for r in rows)
    lo = min(rows, key=lambda r: r["_gap"])
    hi = max(rows, key=lambda r: r["_gap"])
    return {"S2": {"states_with_gap_over_5": over, "of": n, "reading": "met (a reproduction)" if over >= 40 else ("kill" if over <= 12 else "neither")},
            "S2b": {"p10": round(float(p10), 2), "p90": round(float(p90), 2), "p90_less_p10": round(float(spread), 2),
                    "reading": "met" if spread <= 5 else ("kill" if spread > 8 else "neither"),
                    "smallest_gap": [lo["place"], round(float(lo["_gap"]), 2)], "largest_gap": [hi["place"], round(float(hi["_gap"]), 2)], "median_gap": round(float(gaps[n // 2]), 2)},
            "cellular_only_share": {"lowest": round(float(cells[0]), 2), "median": round(float(cells[n // 2]), 2), "highest": round(float(cells[-1]), 2)}}


# ---------------------------------------------------------------- plants

def tsv(dims, years, rows):
    return io.StringIO("\n".join(["%s\\TIME_PERIOD\t" % ",".join(dims) + "\t".join("%d " % y for y in years)] + ["%s\t%s" % (k, "\t".join(v)) for k, v in rows]) + "\n")


def plants():
    res = {}
    yrs = [2013, 2014, 2015, 2016]
    base = [("A,PC_HH,TOTAL,DE", ["80 ", "81 ", "82 b", "83 e"]), ("A,PC_HH,TOTAL,EU27_2020", ["70 ", "71 ", ": ", "73 u"]), ("A,PC_HH,TOTAL,EA", ["60 ", "61 ", "62 ", "63 "]),
            ("A,PC_HH,A1,DE", ["50 ", "51 ", "52 ", "53 "]), ("A,PC_IND,TOTAL,DE", ["10 ", "11 ", "12 ", "13 "])]
    dims = ["freq", "unit", "hhtyp", "geo"]
    s = eurostat(tsv(dims, yrs, base), {"unit": "PC_HH"})
    rows = euro_rows(s["TOTAL"], "t", False)
    de = {r["year"]: r for r in rows if r["place_code"] == "DE"}
    res["P-FLAG"] = de[2015]["break_in_series"] and de[2015]["series_segment"] == 1 and de[2014]["series_segment"] == 0 and de[2016]["estimated"] is True and 2013 not in de
    res["P-GROUP"] = all(r["place_kind"] == "grouping" for r in rows if r["place_code"] in ("EU27_2020", "EA")) and not any("iso3" in r for r in rows if r["place_code"] in GROUPS)
    try:
        eurostat(tsv(dims, yrs, base + [base[0]]), {"unit": "PC_HH"})
        res["P-DUP"] = False
    except Stop:
        res["P-DUP"] = True
    res["P-BLANK"] = not any(r["place_code"] == "EU27_2020" and r["year"] == 2015 for r in rows) and cell("u ")[0] is None and cell(" ")[0] is None
    try:
        eurostat(tsv(dims, yrs, [("A,PC_HH,TOTAL,DE", ["80 ", "181 ", "82 ", "83 "])]), {"unit": "PC_HH"})
        res["P-SHARE"] = False
    except Stop:
        res["P-SHARE"] = True
    res["P-SEL"] = (s["TOTAL"]["DE"][2014][0] == 81 and s["A1"]["DE"][2014][0] == 51 and s["TOTAL"]["EA"][2014][0] == 61 and all(v[0] != 11 for v in s["TOTAL"]["DE"].values()))
    head = "GEO_ID|" + "|".join("B28002_E%03d|B28002_M%03d" % (i, i) for i in range(1, 14))
    def line(geo, e):
        return geo + "|" + "|".join("%d|%d" % (v, -555555555 if i == 0 else 9) for i, v in enumerate(e))
    good = [1000, 900, 5, 890, 800, 120, 700, 60, 50, 3, 5, 40, 60]
    bad = list(good)
    bad[12] = 70
    st = acs(io.StringIO("\n".join([head, line("0400000US01", good), line("0500000US01001", good), line("0400000US72", bad)]) + "\n"))
    rr = acs_rows({"01": st["01"]}, 2024)[0]
    res["P-ACS"] = (set(st) == {"01", "72"} and r3({"01": st["01"]})["failing"] == [] and r3(st)["failing"] == ["72"] and rr["shares_percent"]["K1_any_access (lines 002+012)"] == 94.0
                    and rr["shares_percent"]["K2_cable_fibre_dsl (007)"] == 70.0 and rr["_gap"] == 24 and rr["households_moe"] is None and rr["counts_moe"]["002"] == 9)
    res["P-KIND"] = None    # acts on ITU files alone; ITU was not obtained
    res["P-K4"] = None
    return res


def digest(p):
    b = open(p, "rb").read()
    return {"bytes": len(b), "sha256": hashlib.sha256(b).hexdigest()}


def run(raw, out):
    os.makedirs(out, exist_ok=True)
    log = {"plants": plants(), "units": {"itu_K1": "not obtained", "itu_K3": "not obtained", "itu_K4": "not obtained"},
           "not_run": {"S1": "ITU K1 not written", "S3": "ITU K1 not written", "P-KIND": "no ITU file", "P-K4": "no ITU file"}}
    ok = all(v for v in log["plants"].values() if v is not None)
    files = {n: digest(os.path.join(raw, n)) for n in ("isoc_ci_in_h.tsv", "isoc_r_iacc_h.tsv", "b28002.dat")}
    log["files"] = files
    country = None
    # unit: Eurostat countries
    if not ok:
        log["units"]["eurostat_countries"] = "(e): a plant failed"
    else:
        try:
            country = eurostat(open(os.path.join(raw, "isoc_ci_in_h.tsv"), encoding="utf-8"), {"unit": "PC_HH"})
            log["R2"] = r2(country)
            if log["R2"]["reading"] != "met":
                log["units"]["eurostat_countries"] = "(r): R2 reads misread"
                country = None
            else:
                rows = euro_rows(country["TOTAL"], "isoc_ci_in_h", False)
                json.dump({"what": "CONNECT-0: households with internet access at home (K1), Eurostat, countries and groupings, each year from 2014", "rows": rows},
                          open(os.path.join(out, "eurostat_household_internet.json"), "w", encoding="utf-8"), indent=1)
                log["units"]["eurostat_countries"] = "(a): written"
                lat = sorted(r["value"] for r in rows if r["place_kind"] == "country" and r["year"] == 2025)
                log["printed_eurostat"] = {"rows": len(rows), "countries": len({r["place_code"] for r in rows if r["place_kind"] == "country"}), "years": [min(r["year"] for r in rows), max(r["year"] for r in rows)],
                                           "flagged_estimated": sum(1 for r in rows if r["estimated"] is True), "breaks": sum(1 for r in rows if r["break_in_series"]),
                                           "low_reliability": sum(1 for r in rows if r["low_reliability"]), "countries_with_2025": len(lat),
                                           "p10_p90_2025": [lat[math.ceil(0.1 * len(lat)) - 1], lat[math.ceil(0.9 * len(lat)) - 1]] if lat else None}
        except Exception as e:
            log["units"]["eurostat_countries"] = "(e): %s: %s" % (type(e).__name__, e)
            country = None
    # unit: Eurostat regions
    if country is None:
        log["units"]["eurostat_regions"] = "not written: the country table is not written" if ok else "(e): a plant failed"
    else:
        try:
            regional = eurostat(open(os.path.join(raw, "isoc_r_iacc_h.tsv"), encoding="utf-8"), {"unit": "PC_HH"})
            log["R2b"] = r2b(country, regional)
            if not log["R2b"]["met"]:
                log["units"]["eurostat_regions"] = "(r): R2b not met"
            else:
                rows = euro_rows(regional["TOTAL"], "isoc_r_iacc_h", True)
                json.dump({"what": "CONNECT-0: households with access to the internet at home (K1), Eurostat, NUTS regions, each year from 2014", "rows": rows},
                          open(os.path.join(out, "eurostat_household_internet_regions.json"), "w", encoding="utf-8"), indent=1)
                log["units"]["eurostat_regions"] = "(a): written"
                import collections
                log["printed_regions"] = {"rows": len(rows), "regions": len({r["place_code"] for r in rows}), "by_nuts_level": dict(collections.Counter(r["nuts_level"] for r in {r["place_code"]: r for r in rows}.values())),
                                          "countries": len({r["country_code"] for r in rows}), "latest_year": max(r["year"] for r in rows), "low_reliability": sum(1 for r in rows if r["low_reliability"])}
        except Exception as e:
            log["units"]["eurostat_regions"] = "(e): %s: %s" % (type(e).__name__, e)
    # unit: ACS
    if not ok:
        log["units"]["acs_2024"] = "(e): a plant failed"
    else:
        try:
            states = acs(open(os.path.join(raw, "b28002.dat"), encoding="latin-1"))
            log["R3"] = r3(states)
            if not log["R3"]["met"]:
                log["units"]["acs_2024"] = "(r): R3 not met"
            else:
                rows = acs_rows(states, 2024)
                log.update(s2(rows))
                nat = {i: sum(states[s]["B28002_E%03d" % i] for s in states if s != "72") for i in (1, 2, 7, 6, 12, 13)}
                log["printed_acs"] = {"fifty_states_and_dc_summed": {"K1": round(100 * (nat[2] + nat[12]) / nat[1], 2), "any_subscription": round(100 * nat[2] / nat[1], 2), "K2": round(100 * nat[7] / nat[1], 2),
                                                                      "cellular_only": round(100 * nat[6] / nat[1], 2), "no_access": round(100 * nat[13] / nat[1], 2)}}
                for r in rows:
                    r["gap_K1_less_K2"] = round(float(r.pop("_gap")), 2)
                    r.pop("_cell")
                json.dump({"what": "CONNECT-0: presence and types of internet subscription in households by US state, ACS 2024 1-year, table B28002", "rows": rows},
                          open(os.path.join(out, "acs_b28002_states.json"), "w", encoding="utf-8"), indent=1)
                log["units"]["acs_2024"] = "(a): written"
        except Exception as e:
            log["units"]["acs_2024"] = "(e): %s: %s" % (type(e).__name__, e)
    json.dump(log, open(os.path.join(out, "results.json"), "w"), indent=1)
    print(json.dumps(log, indent=1))
    return 0


if __name__ == "__main__":
    if sys.argv[1] == "plants":
        r = plants()
        for k, v in r.items():
            print("%-9s %s" % (k, "not run (no ITU file)" if v is None else ("PASS" if v else "FAIL")))
        sys.exit(0 if all(v for v in r.values() if v is not None) else 1)
    sys.exit(run(sys.argv[2], sys.argv[3]))
