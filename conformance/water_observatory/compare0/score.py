#!/usr/bin/env python3
"""COMPARE-0's scoring: the served law, MB-pol and TIP4P/2005 against the exact record.

Merges `served.json` (the engine's own numbers) with `references.json` (the two reference
models) and writes:

  `scores.json`  one row per geometry: the exact interaction energy, every model's, the
                 served law's channel decomposition, the interpolant's own miss, and the
                 forces each model gives.
  `gate.json`    the freeze's `gate` phase: the summary statistics by family, the
                 INTERPOLANT's error reported apart from the HAMILTONIAN's, the cost of one
                 reading each way, and every gate this phase can read.

Two errors are kept apart throughout and never summed into one headline:

  the INTERPOLANT's error   the CT-3 table against the exact `E_CT` it interpolates.  It is
                            an exact ZERO in sample (an interpolant interpolates), so the
                            honest gauge is leave-one-out on the map and the direct miss on
                            the one node held out from the table.

  the HAMILTONIAN's error   MB-pol against the exact FCI/STO-3G interaction energy on the
                            same geometry.  Neither model is at fault for this one: it is
                            the minimal basis, and it is the number the basis lane (I-5)
                            has to close.  The programme's law cannot be nearer the truth
                            than its own Hamiltonian is.

    python3 score.py [OUT_DIR]
"""
import json
import math
import pathlib
import sys

HARTREE_TO_KCAL = None  # set from MBX's own constant if the reference ran; else stays None


def rms(v):
    return math.sqrt(sum(x * x for x in v) / len(v)) if v else None


def stats(errs):
    """RMS, max (with its node), mean signed -- the three a reader needs to tell a bias
    from a scatter."""
    if not errs:
        return None
    vals = [e for _, e in errs]
    worst = max(errs, key=lambda p: abs(p[1]))
    return {
        "n": len(vals),
        "rms": rms(vals),
        "max_abs": abs(worst[1]),
        "max_at": worst[0],
        "mean_signed": sum(vals) / len(vals),
    }


def main():
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
    served = json.loads((out / "served.json").read_text())
    refs = json.loads((out / "references.json").read_text())
    rows = served["rows"]
    by_name = {r["node"]: r for r in refs["rows"]}
    mbpol_on = refs["mbpol"]["available"]

    # the exact solves' own price, out of the records the map is made of
    exact_core_seconds = []
    for r in rows:
        tot = 0.0
        for key in ("exact_source", "sector_source"):
            p = pathlib.Path(r[key])
            if not p.is_file():
                continue
            d = json.loads(p.read_text())
            for block in ("exact", "sector"):
                if isinstance(d.get(block), dict):
                    tot += float(d[block].get("cpu_seconds", 0.0))
            if "cpu_seconds" in d and not isinstance(d.get("exact"), dict):
                tot += float(d["cpu_seconds"])
        if tot > 0.0:
            exact_core_seconds.append(tot)

    scored = []
    for r in rows:
        ref = by_name[r["node"]]
        row = {
            "node": r["node"],
            "family": r["family"],
            "held_out": r["held_out"],
            "r_oo_bohr": r["r_oo_bohr"],
            "min_cross_ho_bohr": r["min_cross_ho_bohr"],
            "de_exact": r["de_exact"],
            "e_ct_exact": r["e_ct_exact"],
            "served_total": r["served_total"],
            "served_error": r["served_total"] - r["de_exact"],
            "tip4p2005_remap": ref["tip4p2005_remap"],
            "tip4p2005_remap_error": ref["tip4p2005_remap"] - r["de_exact"],
            "tip4p2005_as_is": ref["tip4p2005_as_is"],
            "tip4p2005_as_is_error": ref["tip4p2005_as_is"] - r["de_exact"],
            "tip4p2005_placement_spread": ref["tip4p2005_remap"] - ref["tip4p2005_as_is"],
            "tip4p2005_hydrogen_moved_bohr": ref["tip4p2005_remap_hydrogen_moved_bohr"],
            "channels": {
                "field_charges": r["served_field"],
                "wall_oo": r["wall_oo"],
                "wall_oh": r["wall_oh"],
                "wall_hh": r["wall_hh"],
                "contact_ho": r["contact_ho"],
                "contact_hh": r["contact_hh"],
                "ct_table": r["ct_table"],
            },
            "ct_family": r["ct_family"],
            "ct_table_error": r["ct_table"] - r["e_ct_exact"],
            "ct_family_error": r["ct_family"] - r["e_ct_exact"],
            "interpolant_miss": r["interpolant_miss"],
            "interpolant_rule": r["interpolant_rule"],
            "served_force": r["served_force"],
            "tip4p2005_remap_force_on_acceptor": ref["tip4p2005_remap_force_on_acceptor"],
        }
        if mbpol_on:
            row["mbpol"] = ref["mbpol"]
            row["mbpol_error_vs_exact"] = ref["mbpol"] - r["de_exact"]
            row["served_error_vs_mbpol"] = r["served_total"] - ref["mbpol"]
            row["mbpol_parts"] = ref["mbpol_parts"]
            row["mbpol_force"] = ref["mbpol_force"]
            fs = r["served_force"]
            fm = ref["mbpol_force"]
            row["force_served_minus_mbpol_rms"] = rms(
                [fs[i][c] - fm[i][c] for i in range(6) for c in range(3)])
            row["force_mbpol_rms"] = rms([fm[i][c] for i in range(6) for c in range(3)])
            row["force_served_rms"] = rms([fs[i][c] for i in range(6) for c in range(3)])
            net_s = [sum(fs[i][c] for i in range(3, 6)) for c in range(3)]
            net_m = [sum(fm[i][c] for i in range(3, 6)) for c in range(3)]
            net_t = ref["tip4p2005_remap_force_on_acceptor"]
            row["net_force_on_acceptor"] = {
                "served": net_s, "mbpol": net_m, "tip4p2005_remap": net_t,
                "served_minus_mbpol_norm": math.sqrt(sum((net_s[c] - net_m[c]) ** 2 for c in range(3))),
                "tip4p2005_minus_mbpol_norm": math.sqrt(sum((net_t[c] - net_m[c]) ** 2 for c in range(3))),
                "mbpol_norm": math.sqrt(sum(x * x for x in net_m)),
            }
        scored.append(row)

    (out / "scores.json").write_text(json.dumps(
        {"phase": "scores",
         "served_source": str(out / "served.json"),
         "references_source": str(out / "references.json"),
         "geometries": len(scored),
         "rows": scored}, indent=1) + "\n")

    # ------------------------------------------------------------------ the summary
    families = sorted({r["family"] for r in scored})
    models = ["served_error", "tip4p2005_remap_error", "tip4p2005_as_is_error"]
    if mbpol_on:
        models += ["mbpol_error_vs_exact", "served_error_vs_mbpol"]

    def pick(rowsel, key):
        return [(r["node"], r[key]) for r in rowsel if key in r]

    per_family = {}
    for f in families + ["ALL", "HELD_OUT"]:
        if f == "ALL":
            sel = scored
        elif f == "HELD_OUT":
            sel = [r for r in scored if r["held_out"]]
        else:
            sel = [r for r in scored if r["family"] == f]
        per_family[f] = {m: stats(pick(sel, m)) for m in models}
        per_family[f]["depth"] = {
            "mean_de_exact": sum(r["de_exact"] for r in sel) / len(sel),
            "deepest": min(r["de_exact"] for r in sel),
        }
        per_family[f]["interpolant"] = stats(pick(sel, "ct_table_error"))
        per_family[f]["interpolant_leave_one_out"] = {
            "n": len(sel),
            "rms": rms([r["interpolant_miss"] for r in sel]),
            "max_abs": max(r["interpolant_miss"] for r in sel),
            "max_at": max(sel, key=lambda r: r["interpolant_miss"])["node"],
            "median": sorted(r["interpolant_miss"] for r in sel)[len(sel) // 2],
        }
        per_family[f]["ct_family_error"] = stats(pick(sel, "ct_family_error"))

    gate = {
        "phase": "gate",
        "campaign": "COMPARE-0",
        "freeze": "../COMPARE0_PREREG.md (DRAFT)",
        "no_new_solves": True,
        "geometries": len(scored),
        "map_nodes": sum(1 for r in scored if not r["held_out"]),
        "held_out": sum(1 for r in scored if r["held_out"]),
        "engine_gates": {k: served[k] for k in ("g_t0", "g_c1", "g_x0", "g_f0")},
        "law": served["law_source"],
        "references": {
            "mbpol": {k: refs["mbpol"][k] for k in
                      ("available", "reason_unavailable", "implementation", "mbx_home", "mbx_commit")},
            "tip4p2005": {"source": refs["tip4p2005"]["source"],
                          "parameters": refs["tip4p2005"]["parameters"],
                          "primary_placement_rule": refs["tip4p2005"]["primary_rule"]},
            "constants": refs["constants"],
        },
        "units": "hartree for every energy; hartree per bohr for every force",
        "by_family": per_family,
        "the_two_errors_kept_apart": {
            "interpolant_rule": "the CT-3 table against the exact E_CT. In sample it is an "
                                "exact zero, so the gauge is leave-one-out on the map and the "
                                "direct miss on the node held out from the table.",
            "hamiltonian_rule": "MB-pol against the exact FCI/STO-3G interaction energy on the "
                                "same geometry. This is the minimal basis's own error and no "
                                "model on this record is charged with it; it is the number the "
                                "basis lane I-5 must close.",
        },
        "cost_of_one_reading": {
            "exact_solve_core_seconds_per_node": {
                "n": len(exact_core_seconds),
                "mean": sum(exact_core_seconds) / len(exact_core_seconds) if exact_core_seconds else None,
                "min": min(exact_core_seconds) if exact_core_seconds else None,
                "max": max(exact_core_seconds) if exact_core_seconds else None,
                "rule": "the records' own exact.cpu_seconds + sector.cpu_seconds; the price of "
                        "ONE point of the truth this campaign scores against",
            },
            "served_law_seconds_all_geometries": served.get("seconds"),
            "served_law_cpu_seconds": served.get("cpu_seconds"),
            "references_seconds_all_geometries": refs.get("seconds"),
            "note": "the served law's wall time here is dominated by the engine's scene "
                    "construction (one Sim per evaluation, twice per geometry for the far "
                    "reference, and 36 more per finite-difference node), not by the law's "
                    "arithmetic; it is reported as measured and is not a per-evaluation cost "
                    "model for the liquid",
        },
        "forces": {
            "exact_forces_present": False,
            "why": "no record in ct1/, ct2/ or ct3/ carries a force: the exact solves stored "
                   "energies only. Nothing here is scored against a true force.",
            "owed": "exact (FCI/STO-3G) forces on at least the held-out node and one node per "
                    "family, so the served law's force can be scored against truth rather than "
                    "against another model",
            "what_is_reported": "the served law's analytic interaction force against MB-pol's, "
                                "per atom and as the net force on the acceptor; and TIP4P/2005's "
                                "net force on the acceptor, which is the only force a rigid model "
                                "on a remapped geometry can be compared on",
        },
    }
    # ------------------------------------------------------- the fences, before any headline
    held = [r for r in scored if r["held_out"]]
    in_sample = [r for r in scored if not r["held_out"]]
    gate["fences"] = {
        "in_sample": {
            "rule": "SIXTY-FOUR of the sixty-five geometries are the served law's OWN fit set: "
                    "the table is EXACT at every one of them by construction and C1's two contact "
                    "amplitudes were fitted on their residual. TIP4P/2005 saw none of them. So the "
                    "all-65 RMS comparison is IN-SAMPLE against OUT-OF-SAMPLE and is not an "
                    "accuracy claim; the one honest out-of-sample number on this record is the "
                    "single held-out node.",
            "served_rms_in_sample": rms([r["served_error"] for r in in_sample]),
            "served_error_held_out": held[0]["served_error"] if held else None,
            "tip4p2005_remap_error_held_out": held[0]["tip4p2005_remap_error"] if held else None,
            "held_out_node": held[0]["node"] if held else None,
        },
        "the_truth_is_sto_3g": "every error here is against a full CI solve in the MINIMAL basis. "
                               "Both reference models are built for real water, so neither is "
                               "being scored on what it was made for. The reading that survives "
                               "this fence is the Hamiltonian gap, and it is reported apart.",
        "sign_census": {
            "rule": "how many of the sixty-five the minimal-basis Hamiltonian even binds",
            "bound": sum(1 for r in scored if r["de_exact"] < 0),
            "unbound": sum(1 for r in scored if r["de_exact"] >= 0),
            "deepest": min(scored, key=lambda r: r["de_exact"])["node"],
            "deepest_hartree": min(r["de_exact"] for r in scored),
            "most_repulsive": max(scored, key=lambda r: r["de_exact"])["node"],
            "most_repulsive_hartree": max(r["de_exact"] for r in scored),
        },
    }

    # -------------------------------------------------------- G-P0, the placement rule priced
    spread_fires = [r["node"] for r in scored
                    if abs(r["tip4p2005_placement_spread"]) > 0.25 * abs(r["tip4p2005_remap_error"])]
    gate["g_p0"] = {
        "rule": "the maximum hydrogen movement the remap rule imposes, per geometry, and the "
                "interaction energy under BOTH rules. Where the spread between the rules exceeds "
                "0.25 of TIP4P/2005's own error against the exact, the classical reference's "
                "number is reported WITH that spread as its bar and no sharper claim is made.",
        "max_hydrogen_moved_bohr": max(r["tip4p2005_hydrogen_moved_bohr"] for r in scored),
        "median_abs_spread_hartree": sorted(abs(r["tip4p2005_placement_spread"]) for r in scored)[len(scored) // 2],
        "max_abs_spread_hartree": max(abs(r["tip4p2005_placement_spread"]) for r in scored),
        "geometries_where_the_spread_exceeds_a_quarter_of_the_error": len(spread_fires),
        "of": len(scored),
        "verdict": ("the classical reference's number carries the placement spread as its bar on "
                    f"{len(spread_fires)} of {len(scored)} geometries"),
    }
    gate["g_p1"] = refs["tip4p2005"].get("force_is_its_own_derivative")
    gate["plants"] = refs.get("plants")

    # ---------------------------------------------------- the branches, in the freeze's order
    served_rms = rms([r["served_error"] for r in scored])
    loo_rms = rms([r["interpolant_miss"] for r in scored])
    fam_sq = {}
    for r in scored:
        fam_sq[r["family"]] = fam_sq.get(r["family"], 0.0) + r["served_error"] ** 2
    tot_sq = sum(fam_sq.values())
    fam_n = {f: sum(1 for r in scored if r["family"] == f) for f in fam_sq}
    worst_node = max(scored, key=lambda r: abs(r["served_error"]))
    concentrated = [f for f in fam_sq
                    if fam_sq[f] / tot_sq > 0.5 and fam_n[f] / len(scored) < 0.5]
    branches = {
        "rule": "the freeze's §3 order: the FIRST that fires is the choice",
        "served_rms_all": served_rms,
        "leave_one_out_rms_all": loo_rms,
        "family_share_of_squared_error": {f: fam_sq[f] / tot_sq for f in sorted(fam_sq)},
        "family_share_of_geometries": {f: fam_n[f] / len(scored) for f in sorted(fam_n)},
        "worst_served_error_node": worst_node["node"],
        "worst_served_error": worst_node["served_error"],
    }
    if mbpol_on:
        ham_rms = rms([r["mbpol_error_vs_exact"] for r in scored])
        branches["hamiltonian_gap_rms"] = ham_rms
        branches["1_larger_basis_fires"] = ham_rms > served_rms
    else:
        branches["hamiltonian_gap_rms"] = None
        branches["1_larger_basis_fires"] = None
        branches["1_note"] = "VOID: MB-pol did not run, so the Hamiltonian gap has no number and " \
                             "branch 1 cannot be tested. No later branch may be declared the " \
                             "choice while the first is VOID."
    branches["2_more_table_coverage_fires"] = loo_rms >= 0.5 * served_rms
    branches["3_better_angular_representation_fires"] = len(concentrated) == 0
    branches["3_concentrated_families"] = concentrated
    branches["4_many_body_polarisation"] = "a REFERRAL only; a dimer set cannot see a three-body term"
    fired = None
    if branches["1_larger_basis_fires"]:
        fired = "1 — a larger basis (the I-5 lane)"
    elif branches["1_larger_basis_fires"] is None:
        fired = None
    elif branches["2_more_table_coverage_fires"]:
        fired = "2 — more table coverage"
    elif branches["3_better_angular_representation_fires"]:
        fired = "3 — a better angular representation"
    else:
        fired = "4 — many-body polarisation (a referral)"
    branches["fired"] = fired
    gate["branches"] = branches

    if mbpol_on:
        allrows = scored
        gate["forces"]["served_vs_mbpol_per_atom_rms"] = {
            "rms_over_geometries": rms([r["force_served_minus_mbpol_rms"] for r in allrows]),
            "max": max(r["force_served_minus_mbpol_rms"] for r in allrows),
            "max_at": max(allrows, key=lambda r: r["force_served_minus_mbpol_rms"])["node"],
            "mbpol_force_rms_over_geometries": rms([r["force_mbpol_rms"] for r in allrows]),
        }
        gate["forces"]["net_force_on_acceptor"] = {
            "served_minus_mbpol_rms": rms([r["net_force_on_acceptor"]["served_minus_mbpol_norm"] for r in allrows]),
            "tip4p2005_minus_mbpol_rms": rms([r["net_force_on_acceptor"]["tip4p2005_minus_mbpol_norm"] for r in allrows]),
            "mbpol_rms": rms([r["net_force_on_acceptor"]["mbpol_norm"] for r in allrows]),
        }

    (out / "gate.json").write_text(json.dumps(gate, indent=1) + "\n")

    # ------------------------------------------------------------------ the console read
    def line(name, s):
        if s is None:
            return f"  {name:<26} --"
        return (f"  {name:<26} rms {s['rms']:.3e}  max {s['max_abs']:.3e} ({s['max_at']})"
                f"  mean {s['mean_signed']:.3e}")

    print(f"COMPARE-0 gate: {len(scored)} geometries, MB-pol "
          f"{'ON' if mbpol_on else 'OFF: ' + str(refs['mbpol']['reason_unavailable'])}")
    for f in families + ["ALL", "HELD_OUT"]:
        d = per_family[f]
        n = d["served_error"]["n"] if d["served_error"] else 0
        print(f"{f} (n = {n}, mean depth {d['depth']['mean_de_exact']:.3e} Ha)")
        for m in models:
            print(line(m, d[m]))
        print(line("interpolant (LOO/held-out)", {
            "rms": d["interpolant_leave_one_out"]["rms"],
            "max_abs": d["interpolant_leave_one_out"]["max_abs"],
            "max_at": d["interpolant_leave_one_out"]["max_at"],
            "mean_signed": d["interpolant_leave_one_out"]["median"],
        }))


if __name__ == "__main__":
    main()
