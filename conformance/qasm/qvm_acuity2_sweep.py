#!/usr/bin/env python3
"""QVM-ACUITY-2 sweep: the dictionary of views over four unlabelled families, graded as staked.

Runs `qvm_acuity2` (engine/crates/holon/src/bin/qvm_acuity2.rs) once over the whole campaign —
80 instances, C/T/L/D x 20, generated in a SHUFFLED order from one seed and handed to the search
without their family — then times stim on family C's identical circuits (the S5 comparison
class), grades S1-S5 and the sweep-level plants (PQ-4 on the recorded prices and walls, PQ-5 on
the recorded label-revealed re-selection) exactly as `QVM_ACUITY2_PREREG.md` stakes them, and
writes `qvm_acuity2_results.json` and `qvm_acuity2_table.md`.

Two readings of the MPS closure test are graded side by side, labelled:
  * `cadence` — QVM_ACUITY2_AMENDMENT_1.md's probe (the reading the campaign is READ under);
  * `prefix`  — the probe as frozen (the first 2n gates, projected), kept and graded so the
                amendment cannot hide what the frozen text would have read.

Everything runs on cores 21-27 (taskset); stim is imported from a python that has it
(`QVM_ACUITY2_PY`, default: this interpreter).

  QVM_ACUITY2_SEED   campaign seed (default 2026, Amendment 1)
  QVM_ACUITY2_COUNT  instances per family (default 20)
  QVM_ACUITY2_RAW    reuse a raw driver output instead of running the driver
"""
import json, os, statistics, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
BIN = os.environ.get("QVM_ACUITY2_BIN", os.path.join(ROOT, "engine", "target", "release", "qvm_acuity2"))
SEED = int(os.environ.get("QVM_ACUITY2_SEED", "2026"))
COUNT = int(os.environ.get("QVM_ACUITY2_COUNT", "20"))
RAW = os.environ.get("QVM_ACUITY2_RAW")
CORES = "21-27"
TAU = 1e-12  # the f64 referee's own floor (Amendment 1, item 3)
EXPECTED = {"C": "tableau", "T": "sum", "L": "mps", "D": "dense"}
COMPARISON = {"C": "stim (TableauSimulator)", "T": "full stabilizer-rank sum (eps = 0)",
              "L": "MPS at chi = 64", "D": "dense vector"}
VIEWS = ["tableau", "sum", "mps", "dense"]


def run_driver():
    if RAW:
        return open(RAW).read().splitlines()
    cmd = ["taskset", "-c", CORES, BIN, "--seed", str(SEED), "--count", str(COUNT)]
    print("running:", " ".join(cmd), flush=True)
    t0 = time.time()
    out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    print(f"driver wall {time.time() - t0:.1f} s", flush=True)
    raw = os.path.join(HERE, "qvm_acuity2_raw.jsonl")
    with open(raw, "w") as f:
        f.write(out.stdout)
    return out.stdout.splitlines()


def stim_wall(n, circuit, obs_kind, bits_of_obs, reps=5):
    """stim's wall on the identical circuit: TableauSimulator.do plus the observable's
    probability by peek/postselect (for an amplitude that is |a|^2 - stim carries no phase).
    Construction of the stim.Circuit is not timed (bakeoff.py's rule)."""
    import stim
    names = {"h": "H", "s": "S", "sdg": "S_DAG", "x": "X", "z": "Z", "cx": "CX"}
    c = stim.Circuit()
    for g in circuit:
        c.append(names[g[0]], g[1:])
    qubits = list(range(n)) if obs_kind == "amplitude" else [0, 1, 2, 3]
    walls, prob = [], None
    for _ in range(reps):
        sim = stim.TableauSimulator()
        sim.set_num_qubits(n)
        t0 = time.perf_counter()
        sim.do(c)
        p = 1.0
        for q, b in zip(qubits, bits_of_obs):
            z = sim.peek_z(q)
            if z == 0:
                p *= 0.5
                sim.postselect_z(q, desired_value=bool(b))
            elif (z == -1) != bool(b):
                p = 0.0
                break
        walls.append(time.perf_counter() - t0)
        prob = p
    return statistics.median(walls), prob


def bits_of(r):
    s = r["observable"]
    return [c == "1" for c in s.split("=")[-1].rstrip("]")] if "marginal" in s else [c == "1" for c in s[4:-1]]


def ref_prob(r):
    v = r["referee"]
    return v["re"] ** 2 + v["im"] ** 2 if isinstance(v, dict) else v


def best_hand(r):
    """The best wall among the four views run by hand that REACHED the acuity."""
    best, who = None, None
    for v in VIEWS:
        h = r["hand"][v]
        if h["reached"] and h["wall"] is not None and (best is None or h["wall"] < best):
            best, who = h["wall"], v
    return best, who


def grade(rows, reading):
    sel_key = "selected" if reading == "cadence" else "selected_prefix"
    run_key = "run" if reading == "cadence" else "run_prefix"
    err_key = "abs_error" if reading == "cadence" else "abs_error_prefix"
    fams = {}
    for f in "CTLD":
        rs = [r for r in rows if r["family"] == f]
        right = sum(1 for r in rs if r[sel_key] == EXPECTED[f])
        wrong = [(r["id"], r["n"], r["observable_kind"], r[sel_key],
                  r["select_" + reading]["line"]) for r in rs if r[sel_key] != EXPECTED[f]]
        ratios = []
        for r in rs:
            b, _ = best_hand(r)
            w = r[run_key]["wall"]
            ratios.append(None if (b is None or w is None or not r[run_key]["reached"]) else w / b)
        within2 = sum(1 for x in ratios if x is not None and x <= 2.0)
        over4 = sum(1 for x in ratios if x is None or x > 4.0)
        viol, worst_cert_over_err, cert_list = [], 0.0, []
        for r in rs:
            cert, e = r[run_key]["certificate"], r[err_key]
            ok = cert is not None and e is not None and e <= cert + TAU and cert <= r["eps"]
            if not ok:
                viol.append({"id": r["id"], "n": r["n"], "obs": r["observable_kind"], "view": r[sel_key],
                             "certificate": cert, "abs_error": e, "eps": r["eps"]})
            if cert and e:
                worst_cert_over_err = max(worst_cert_over_err, cert / e)
            cert_list.append((cert, e))
        fams[f] = {"n": len(rs), "right": right, "S1": "MET" if right >= 18 else ("KILLED" if right < 15 else "MISSED (not killed)"),
                   "wrong": wrong, "S2_ratios": ratios, "S2_within_2x": within2, "S2_over_4x_or_failed": over4,
                   "S2": "MET" if within2 == len(rs) else ("KILLED" if over4 > 5 else "MISSED (not killed)"),
                   "S3_violations": viol, "S3": "MET" if not viol else "VIOLATED",
                   "worst_certificate_over_error": worst_cert_over_err}
    return fams


def s4(rows):
    out = {}
    for er in (1e-1, 1e-2):
        rs = [r for r in rows if r["family"] == "L" and abs(r["eps_rel"] - er) < 1e-12]
        ratios = []
        for r in rs:
            chi = r["select_cadence"]["probe_chi"]
            if r["selected"] == "mps":
                w = r["run"]["wall"]
            elif r["hand"]["mps"]["chi"] == chi:
                w = r["hand"]["mps"]["wall"]
            else:
                w = None
            w64 = r.get("mps64", {}).get("wall")
            ratios.append(None if (w is None or w64 is None) else w / w64)
        good = [x for x in ratios if x is not None]
        out[f"{er:g}"] = {"ratios": ratios, "median": statistics.median(good) if good else None,
                          "max": max(good) if good else None,
                          "chis": [r["select_cadence"]["probe_chi"] for r in rs],
                          "max_bond_at_64": [r.get("mps64", {}).get("note", "") for r in rs]}
    m1, m2 = out["0.1"]["median"], out["0.01"]["median"]
    verdict = "MET" if (m1 is not None and m1 <= 0.25 and m2 is not None and m2 <= 0.5) else \
        ("KILLED" if (m1 is None or m1 > 0.75) else "MISSED (not killed)")
    t = [r for r in rows if r["family"] == "T"]
    frac = [r["hand"]["sum"]["fraction"] for r in t if r["hand"]["sum"]["fraction"] is not None]
    return {"L": out, "S4": verdict, "T_executed_fraction": {"values": frac, "median": statistics.median(frac) if frac else None}}


def s5(rows):
    out = {}
    for f in "CTLD":
        rs = [r for r in rows if r["family"] == f]
        lines = []
        for r in rs:
            sel = r["run"]["wall"]
            if f == "C":
                comp = r.get("stim_wall")
            elif f == "T":
                comp = r.get("sum_full", {}).get("wall")
            elif f == "L":
                comp = r.get("mps64", {}).get("wall")
            else:
                comp = r["hand"]["dense"]["wall"]
            lines.append({"id": r["id"], "n": r["n"], "obs": r["observable_kind"], "selected": r["selected"],
                          "selected_wall": sel, "comparison_wall": comp,
                          "ratio": None if (sel is None or comp is None) else sel / comp})
        rat = [x["ratio"] for x in lines if x["ratio"] is not None]
        out[f] = {"comparison": COMPARISON[f], "rows": lines,
                  "median_ratio": statistics.median(rat) if rat else None,
                  "within_2x": sum(1 for x in rat if x <= 2.0), "n": len(lines)}
    return out


def pq4(rows):
    """A wrong price planted: the most frequent runner-up's predicted price halved. Flip rule
    re-derived from the recorded prices; the walls of the flipped selections read by S2."""
    counts = {}
    for r in rows:
        rk = r["select_cadence"]["ranking"]
        if len(rk) > 1:
            counts[rk[1]] = counts.get(rk[1], 0) + 1
    planted = max(counts, key=counts.get)
    flips = []
    for r in rows:
        sc = r["select_cadence"]
        rk, pr = sc["ranking"], sc["prices"]
        if len(rk) > 1 and rk[1] == planted and 0.5 * pr[planted]["price"] < pr[rk[0]]["price"]:
            b, _ = best_hand(r)
            w = r["hand"][planted]["wall"] if r["hand"][planted]["reached"] else None
            flips.append({"id": r["id"], "family": r["family"], "n": r["n"], "from": rk[0], "to": planted,
                          "price_ratio": pr[planted]["price"] / pr[rk[0]]["price"],
                          "wall_ratio_to_best": None if (w is None or b is None) else w / b})
    caught = sum(1 for x in flips if x["wall_ratio_to_best"] is None or x["wall_ratio_to_best"] > 2.0)
    return {"planted_on": planted, "runner_up_counts": counts, "flips": flips, "S2_catches": caught}


def main():
    lines = run_driver()
    calib = None
    rows = []
    for l in lines:
        if l.startswith('{"calibration"'):
            calib = json.loads(l)["calibration"]
        elif l.startswith('{"id"'):
            rows.append(json.loads(l))
    rows.sort(key=lambda r: r["id"])
    print(f"{len(rows)} instances; calibration {calib}", flush=True)

    # S5: stim on family C, same circuits, same cores
    try:
        import stim
        stim_version = stim.__version__
        for r in rows:
            if r["family"] != "C":
                continue
            w, p = stim_wall(r["n"], r["circuit"], r["observable_kind"], bits_of(r))
            r["stim_wall"], r["stim_prob"] = w, p
            r["stim_prob_matches_referee"] = abs(p - ref_prob(r)) < 1e-12
    except ImportError:
        stim_version = None
        print("stim is not importable here: S5's family-C column is missing", flush=True)

    readings = {rd: grade(rows, rd) for rd in ("cadence", "prefix")}
    S4 = s4(rows)
    S5 = s5(rows)
    P4 = pq4(rows)
    pq5_same = sum(1 for r in rows if r["pq5_label_revealed_same"])
    loadavg = os.getloadavg()

    for r in rows:
        r.pop("circuit", None)
    res = {"prereg": "QVM_ACUITY2_PREREG.md", "amendment": "QVM_ACUITY2_AMENDMENT_1.md", "seed": SEED,
           "count_per_family": COUNT, "cores": CORES, "loadavg_end": list(loadavg), "stim_version": stim_version,
           "calibration": calib, "readings": readings, "S4": S4, "S5": S5, "PQ4": P4,
           "PQ5": {"same": pq5_same, "of": len(rows)}, "tau_referee_floor": TAU, "rows": rows}
    with open(os.path.join(HERE, "qvm_acuity2_results.json"), "w") as f:
        json.dump(res, f, indent=1)

    # ---------------- the table ----------------
    md = []
    md.append("# QVM-ACUITY-2 — sweep table\n")
    md.append(f"*Generated by `qvm_acuity2_sweep.py`; seed {SEED}, {COUNT} per family, cores {CORES}, "
              f"load average at the end {loadavg[0]:.1f}; stim {stim_version}. Constants (calibrated once): "
              + ", ".join(f"`{k}` = {v:.3e}" for k, v in (calib or {}).items()) + ".*\n")
    for rd in ("cadence", "prefix"):
        g = readings[rd]
        md.append(f"\n## Reading: `{rd}` probe" + (" (Amendment 1 — the reading of record)" if rd == "cadence" else " (as frozen — graded beside, not read)") + "\n")
        md.append("| family | S1 right/20 | S1 | S2 within 2× | S2 >4× or failed | S2 | S3 violations | worst cert/err |")
        md.append("|---|---|---|---|---|---|---|---|")
        for f in "CTLD":
            x = g[f]
            md.append(f"| {f} | {x['right']}/{x['n']} | {x['S1']} | {x['S2_within_2x']} | {x['S2_over_4x_or_failed']} | {x['S2']} | "
                      f"{len(x['S3_violations'])} | {x['worst_certificate_over_error']:.3g} |")
    md.append("\n## Per instance (cadence reading)\n")
    md.append("| id | fam | n | obs | ε_rel | selected | price (s) | wall (s) | best by hand | ratio | tableau | sum | mps (χ) | dense | cert | \\|err\\| |")
    md.append("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")

    def fw(x):
        return "—" if x is None else f"{x:.2e}"
    for r in rows:
        b, who = best_hand(r)
        w = r["run"]["wall"]
        h = r["hand"]
        md.append(f"| {r['id']} | {r['family']} | {r['n']} | {r['observable_kind'][:3]} | {r['eps_rel']:g} | {r['selected']} | "
                  f"{fw(r['select_cadence']['prices'][r['selected']]['price'])} | {fw(w)} | {who} {fw(b)} | "
                  f"{'—' if (w is None or b is None) else f'{w / b:.2f}'} | {fw(h['tableau']['wall'])} | {fw(h['sum']['wall'])} | "
                  f"{fw(h['mps']['wall'])} ({h['mps']['chi']}) | {fw(h['dense']['wall'])} | {fw(r['run']['certificate'])} | {fw(r['abs_error'])} |")
    md.append("\n## S4 (family L: MPS at the acuity's χ against χ = 64)\n")
    for k, v in S4["L"].items():
        md.append(f"- ε_rel = {k}: ratios {[None if x is None else round(x, 3) for x in v['ratios']]}, median "
                  f"{fw(v['median'])}, χ chosen {v['chis']}")
    md.append(f"- verdict: **{S4['S4']}**; family T executed fraction (by-hand sum at ε) median {fw(S4['T_executed_fraction']['median'])}")
    md.append("\n## S5 (the comparison class)\n")
    md.append("| family | comparison | median selected/comparison | within 2× |")
    md.append("|---|---|---|---|")
    for f in "CTLD":
        x = S5[f]
        md.append(f"| {f} | {x['comparison']} | {fw(x['median_ratio'])} | {x['within_2x']}/{x['n']} |")
    md.append("\n## Plants read on the sweep\n")
    md.append(f"- PQ-4: planted on `{P4['planted_on']}` (the most frequent runner-up, {P4['runner_up_counts']}); "
              f"{len(P4['flips'])} flips; S2 (wall > 2× the best by hand) catches {P4['S2_catches']} of them.")
    for x in P4["flips"]:
        md.append(f"  - id {x['id']} ({x['family']}, n={x['n']}): {x['from']} → {x['to']}, price ratio {x['price_ratio']:.2f}, "
                  f"wall/best {fw(x['wall_ratio_to_best'])}")
    md.append(f"- PQ-5: the label-revealed selection equals the blind one on {pq5_same}/{len(rows)} instances.")
    with open(os.path.join(HERE, "qvm_acuity2_table.md"), "w") as f:
        f.write("\n".join(md) + "\n")

    # ---------------- the verdict ----------------
    print("\nVERDICT (QVM_ACUITY2_PREREG.md, cadence reading of record; prefix reading beside)")
    for rd in ("cadence", "prefix"):
        g = readings[rd]
        print(f"  [{rd}] S1 " + "  ".join(f"{f}:{g[f]['right']}/20 {g[f]['S1']}" for f in "CTLD"))
        print(f"  [{rd}] S2 " + "  ".join(f"{f}:{g[f]['S2_within_2x']}w2 {g[f]['S2_over_4x_or_failed']}o4 {g[f]['S2']}" for f in "CTLD"))
        print(f"  [{rd}] S3 " + "  ".join(f"{f}:{len(g[f]['S3_violations'])}viol worst {g[f]['worst_certificate_over_error']:.3g}" for f in "CTLD"))
    print(f"  S4 {S4['S4']}: L medians eps0.1 {S4['L']['0.1']['median']} eps0.01 {S4['L']['0.01']['median']}")
    print("  S5 " + "  ".join(f"{f}:{fw(S5[f]['median_ratio'])} ({S5[f]['within_2x']}/{S5[f]['n']} within 2x)" for f in "CTLD"))
    print(f"  PQ-4 planted on {P4['planted_on']}: {len(P4['flips'])} flips, S2 catches {P4['S2_catches']}; PQ-5 {pq5_same}/{len(rows)}")
    print("wrote qvm_acuity2_results.json, qvm_acuity2_table.md")


if __name__ == "__main__":
    main()
