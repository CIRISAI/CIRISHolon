#!/usr/bin/env python3
"""CONNECT-0A: the Eurostat tables under the amended reading check R2' (`CONNECT0A_AMENDMENT.md`).

  connect0a.py RAW_DIR OUT_DIR

Uses CONNECT-0's extractor and row rules unchanged. R2' (CONNECT-0A) read not met. R2'' (CONNECT-0B) is on named-country statements stated before comparison.
"""
import collections
import hashlib
import json
import math
import os
import sys
from decimal import Decimal

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import connect0 as c  # noqa: E402

DIGESTS = {"isoc_ci_in_h.tsv": "4fed29fe3485ac7e5493e99d4d1b7ac7ae666d08582947b38b4c338186303538",
           "isoc_r_iacc_h.tsv": "2f507eb428a1a6063355be5b77461561076a5eb0cbcf33ac123c98058f261b3e"}


def main(raw, out):
    os.makedirs(out, exist_ok=True)
    log = {"plants": c.plants(), "amended_after_the_value_was_seen": True}
    for n, d in DIGESTS.items():
        if hashlib.sha256(open(os.path.join(raw, n), "rb").read()).hexdigest() != d:
            log["branch"] = "(e): %s is not the file recorded" % n
            print(json.dumps(log, indent=1))
            return 1
    country = c.eurostat(open(os.path.join(raw, "isoc_ci_in_h.tsv"), encoding="utf-8"), {"unit": "PC_HH"})
    r2 = c.r2(country)
    eu = Decimal(r2["eu27_2025"])
    near = sorted(k for k, v in r2["neighbours_2025"].items() if v and abs(Decimal(v) - Decimal("94.7")) <= Decimal("0.1"))
    lo, hi = (Decimal(x) for x in r2["member_min_max"])
    r2["R2_prime_of_0A"] = {"neighbours_within_0.1": near, "met": abs(eu - Decimal("94.7")) <= Decimal("0.1") and not near, "note": "CONNECT-0A's check; read not met; kept for the record"}
    tot = country["TOTAL"]
    v25 = {m: tot[m][2025][0] for m in c.EU27 if 2025 in tot.get(m, {})}
    order = sorted(v25, key=lambda m: v25[m])
    rnd = lambda x: int(x.quantize(Decimal("1"), rounding="ROUND_HALF_UP"))
    inc = {m: tot[m][2025][0] - tot[m][2015][0] for m in c.EU27 if 2025 in tot.get(m, {}) and 2015 in tot.get(m, {})}
    top = max(inc, key=lambda m: inc[m])
    items = {"1_eu_within_0.1": abs(eu - Decimal("94.7")) <= Decimal("0.1"),
             "2_highest_two_NL_LU_at_99": set(order[-2:]) == {"NL", "LU"} and all(rnd(v25[m]) == 99 for m in ("NL", "LU")),
             "3_lowest_two_HR_EL_at_88_89": set(order[:2]) == {"HR", "EL"} and rnd(v25["HR"]) == 88 and rnd(v25["EL"]) == 89,
             "4_BG_largest_increase_at_34": top == "BG" and rnd(inc["BG"]) == 34}
    r2["R2_double_prime"] = {"items": items, "seen": {"highest_two": [(m, str(v25[m])) for m in order[-2:]], "lowest_two": [(m, str(v25[m])) for m in order[:2]],
                                                       "largest_increase": [top, str(inc[top])], "members_with_2015_and_2025": len(inc)},
                             "met": items["1_eu_within_0.1"] and sum(items[k] for k in list(items)[1:]) >= 2}
    r2["reading_as_frozen"] = r2.pop("reading")
    log["R2"] = r2
    if not r2["R2_double_prime"]["met"]:
        log["branch"] = "(r): R2'' not met; no further bar"
        json.dump(log, open(os.path.join(out, "results_0b.json"), "w"), indent=1)
        print(json.dumps(log, indent=1))
        return 1
    rows = c.euro_rows(country["TOTAL"], "isoc_ci_in_h", False)
    json.dump({"what": "CONNECT-0: households with internet access at home (K1), Eurostat, countries and groupings, each year from 2014", "rows": rows},
              open(os.path.join(out, "eurostat_household_internet.json"), "w", encoding="utf-8"), indent=1)
    lat = sorted(r["value"] for r in rows if r["place_kind"] == "country" and r["year"] == 2025)
    by = {r["place_code"]: r["value"] for r in rows if r["place_kind"] == "country" and r["year"] == 2025}
    log["printed_countries"] = {"rows": len(rows), "countries": len({r["place_code"] for r in rows if r["place_kind"] == "country"}), "years": [min(r["year"] for r in rows), max(r["year"] for r in rows)],
                                "flagged_estimated": sum(1 for r in rows if r["estimated"] is True), "breaks": sum(1 for r in rows if r["break_in_series"]), "low_reliability": sum(1 for r in rows if r["low_reliability"]),
                                "countries_with_2025": len(lat), "p10_p90_2025": [lat[math.ceil(0.1 * len(lat)) - 1], lat[math.ceil(0.9 * len(lat)) - 1]],
                                "lowest_2025": min(by.items(), key=lambda kv: kv[1]), "highest_2025": max(by.items(), key=lambda kv: kv[1])}
    log["units"] = {"eurostat_countries": "(a): written under R2'' (CONNECT-0B)"}
    regional = c.eurostat(open(os.path.join(raw, "isoc_r_iacc_h.tsv"), encoding="utf-8"), {"unit": "PC_HH"})
    log["R2b"] = c.r2b(country, regional)
    if not log["R2b"]["met"]:
        log["units"]["eurostat_regions"] = "(r): R2b not met; not written"
    else:
        rr = c.euro_rows(regional["TOTAL"], "isoc_r_iacc_h", True)
        json.dump({"what": "CONNECT-0: households with access to the internet at home (K1), Eurostat, NUTS regions, each year from 2014", "rows": rr},
                  open(os.path.join(out, "eurostat_household_internet_regions.json"), "w", encoding="utf-8"), indent=1)
        last = {}
        for r in rr:
            if r["place_code"] not in last or r["year"] > last[r["place_code"]]["year"]:
                last[r["place_code"]] = r
        spread = {}
        for r in last.values():
            if r["nuts_level"] == 2:
                spread.setdefault(r["country_code"], []).append(r["value"])
        wide = sorted(((max(v) - min(v), k, len(v)) for k, v in spread.items() if len(v) > 1), reverse=True)
        log["printed_regions"] = {"rows": len(rr), "regions": len(last), "by_nuts_level": dict(collections.Counter(r["nuts_level"] for r in last.values())), "countries": len({r["country_code"] for r in rr}),
                                  "latest_year": max(r["year"] for r in rr), "regions_with_latest_year": sum(1 for r in last.values() if r["year"] == max(x["year"] for x in rr)),
                                  "low_reliability": sum(1 for r in rr if r["low_reliability"]),
                                  "widest_within_country_spread_nuts2_latest": [(k, round(d, 2), n) for d, k, n in wide[:5]]}
        log["units"]["eurostat_regions"] = "(a): written"
    json.dump(log, open(os.path.join(out, "results_0b.json"), "w"), indent=1)
    print(json.dumps(log, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
