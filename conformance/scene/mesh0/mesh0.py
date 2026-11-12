#!/usr/bin/env python3
"""MESH-0: reads the carrier's lines (`engine/crates/holon-scene/examples/mesh0.rs`) and the
published PRESENCE-0 table, prints the arithmetic first, then the gate, the stakes and the
printed readings of `MESH0_PREREG.md`.   mesh0.py CARRIER.jsonl PRESENCE_TABLE.json OUT.json
Standard library only. Nothing is generated here."""
import json
import statistics
import sys

BASE = "Mon-Thu|afresh|off0|independent"
T0 = 237000


def main():
    lines = [json.loads(l) for l in open(sys.argv[1])]
    pres = json.load(open(sys.argv[2]))
    pops = {l["label"]: l for l in lines if l["kind"] == "population"}
    runs = {(l["label"], l["t_ppm"], l["host"]): l for l in lines if l["kind"] == "run"}
    seeds = [l for l in lines if l["kind"] == "seed"]
    out = {"verdict_of_every_count": next(l for l in lines if l["kind"] == "verdict"), "digest_native": next(l for l in lines if l["kind"] == "digest")["value"]}
    share = lambda r, k: r[k] / r["messages"]

    def within(r, which, t):
        return (r["at_once"] + sum(r[which][1:t + 1])) / r["messages"]
    # C1 from PRESENCE-0's published hourly shares alone (all persons, Monday to Thursday)
    tab = pres["all_persons"]["table"][0]
    a = [1 - (h[0] + h[4]) for h in tab]
    c1 = sum(x * x for x in a) / sum(a)
    p0 = pops[BASE]
    out["arithmetic"] = {"C1_taken_at_once_from_PRESENCE0_hourly_shares": c1, "C1_from_the_generated_population_own_hours": p0["closed_form_ppm"] / 1e6,
                         "C2_desktop_share": p0["desktop"] / p0["households"], "C2_fixed_line_share": p0["fixed_line"] / p0["households"],
                         "C2_desktop_or_fixed_line": p0["desktop_or_line"] / p0["households"],
                         "C3_left_under_NONE_is_one_minus_at_once": 1 - share(runs[(BASE, T0, "NONE")], "at_once")}
    awake = p0["awake_hours"] / p0["person_hours"]
    nothing = p0["nothing_to_host_on"] / p0["households"]
    sums = all(r["at_once"] + sum(r["peer_wait"]) == r["messages"] and r["at_once"] + sum(r["host_wait"]) == r["messages"] for r in runs.values())
    out["G0"] = {"outcomes_sum": sums, "awake_share_generated": awake, "nothing_to_host_on_generated": nothing,
                 "pass": bool(sums and abs(awake - 0.623) <= 0.02 and abs(nothing - 0.088) <= 0.01)}
    if not out["G0"]["pass"]:
        out["branch"] = "(e): the gate failed"
        json.dump(out, open(sys.argv[3], "w"), indent=1)
        print(json.dumps(out, indent=1))
        return 1
    r0 = runs[(BASE, T0, "NONE")]
    d1 = share(r0, "at_once") - c1
    out["S1"] = {"taken_at_once_measured": share(r0, "at_once"), "minus_C1": d1,
                 "by_sender_kind_then_addressee_kind_not_employed_employed": [[r0["at_once_by_kind"][s][k] / max(r0["messages_by_kind"][s][k], 1) for k in (0, 1)] for s in (0, 1)],
                 "minus_the_population_own_closed_form": share(r0, "at_once") - p0["closed_form_ppm"] / 1e6,
                 "share_of_senders_in_the_household_realised": share(r0, "from_household"),
                 "outcome": "met" if abs(d1) <= 0.02 else ("killed" if abs(d1) > 0.05 else "neither")}
    four = {}
    for join in ("afresh", "same"):
        for off in ("off0", "off150000"):
            r = runs[("Mon-Thu|%s|%s|independent" % (join, off), T0, "ALL")]
            four["%s, %s" % (join, "phone on when awake" if off == "off0" else "0.15 of awake hours off")] = {
                "sender_holds_within_hours": {str(t): within(r, "peer_wait", t) for t in (0, 1, 4, 8, 12, 24)},
                "host_at_home_within_hours": {str(t): within(r, "host_wait", t) for t in (0, 1, 4, 8, 12, 24)},
                "host_minus_sender_within_4": within(r, "host_wait", 4) - within(r, "peer_wait", 4)}
    p12 = [v["sender_holds_within_hours"]["12"] for v in four.values()]
    out["S2"] = {"by_join_and_phone_rule": four, "lowest_within_12": min(p12),
                 "outcome": "met" if min(p12) >= 0.95 else ("killed" if min(p12) < 0.90 else "neither")}
    g4 = [v["host_minus_sender_within_4"] for v in four.values()]
    out["S3"] = {"host_minus_sender_within_4_by_join_and_phone_rule": g4,
                 "outcome": "met" if min(g4) >= 0.03 else ("killed" if max(g4) <= 0.01 else "neither")}
    # printed, no verdict
    left = {}
    for label in pops:
        for t in (237000, 290000, 520000):
            for host in ("NONE", "ANY-COMPUTER", "DECLARED", "TWIN", "TWIN-IN-DECLARED", "HOSTLESS-OLD"):
                left["%s | t %.3f | %s" % (label, t / 1e6, host)] = share(runs[(label, t, host)], "left")
    out["left_to_someone_else"] = left
    out["twin_hosts_on_households_with_nothing_declared"] = p0["twin_hosts_on_undeclared"] / p0["twin_hosts"]
    ratios = [(s["left"] / s["messages"]) / ((1 - s["at_once"] / s["messages"]) * s["addressees_in_hostless"] / s["addressees"]) for s in seeds]
    ho = runs[(BASE, T0, "HOSTLESS-OLD")]
    out["product_reading"] = {"ratio_mean_over_20_seeds": statistics.mean(ratios), "ratio_sd": statistics.pstdev(ratios), "ratios": ratios,
                              "HOSTLESS_OLD_left_over_ANY_COMPUTER_left": ho["left"] / runs[(BASE, T0, "ANY-COMPUTER")]["left"]}

    def quart(h):
        tot, acc, q = sum(h), 0, []
        for want in (0.25, 0.5, 0.75):
            acc = 0
            for i, v in enumerate(h):
                acc += v
                if acc >= want * tot:
                    q.append(i)
                    break
        return q
    waits = {}
    for join in ("afresh", "same"):
        r = runs[("Mon-Thu|%s|off0|independent" % join, T0, "ALL")]
        held = sum(r["host_wait"])
        waits[join] = {"unbroken_off_runs_quartiles_hours": quart(pops["Mon-Thu|%s|off0|independent" % join]["off_runs"]),
                       "wait_quartiles_hours": quart(r["host_wait"]),
                       "share_waiting_over_hours": {str(t): sum(r["host_wait"][t + 1:]) / held for t in (1, 4, 8, 12)},
                       "distribution": [v / held for v in r["host_wait"]]}
    out["waits_with_a_host"] = waits
    out["seat_queue_peak_over_mean"] = {h: max(runs[(BASE, T0, h)]["held_by_hour"]) / (sum(runs[(BASE, T0, h)]["held_by_hour"]) / 24) for h in ("NONE", "ANY-COMPUTER")}
    out["seat_queue_by_slot_NONE_share_of_peak"] = [v / max(r0["held_by_hour"]) for v in r0["held_by_hour"]]
    L = lambda label, t, host: share(runs[(label, t, host)], "left")
    out["swings_in_left_to_someone_else"] = {
        "host setting (NONE to ANY-COMPUTER)": L(BASE, T0, "NONE") - L(BASE, T0, "ANY-COMPUTER"),
        "host setting (ANY-COMPUTER to DECLARED)": L(BASE, T0, "DECLARED") - L(BASE, T0, "ANY-COMPUTER"),
        "phone rule (0.15 off), under TWIN": L("Mon-Thu|afresh|off150000|independent", T0, "TWIN") - L(BASE, T0, "TWIN"),
        "tie share 0.237 to 0.52, under TWIN": L(BASE, 520000, "TWIN") - L(BASE, T0, "TWIN"),
        "day type (Saturday), under TWIN": L("Saturday|afresh|off0|independent", T0, "TWIN") - L(BASE, T0, "TWIN"),
        "household shares one day, under TWIN": L("Mon-Thu|afresh|off0|shared-day", T0, "TWIN") - L(BASE, T0, "TWIN")}
    out["counted_apart"] = {"addressees": p0["addressees"], "households": p0["households"], "children_phones": p0["children_phones"], "adults_without_phone": p0["adults_without_phone"]}
    out["branch"] = "(a1): the closed form suffices for the share taken at once" if out["S1"]["outcome"] == "met" else "(a2): S1 read %s" % out["S1"]["outcome"]
    json.dump(out, open(sys.argv[3], "w"), indent=1)
    small = {k: v for k, v in out.items() if k not in ("left_to_someone_else", "waits_with_a_host", "seat_queue_by_slot_NONE_share_of_peak", "verdict_of_every_count")}
    small["product_reading"] = {k: v for k, v in out["product_reading"].items() if k != "ratios"}
    print(json.dumps(small, indent=1))
    print(json.dumps({k: {a: b for a, b in v.items() if a != "distribution"} for k, v in waits.items()}))
    print(json.dumps({k: round(v, 4) for k, v in left.items() if k.startswith(BASE) and "t 0.237" in k}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
