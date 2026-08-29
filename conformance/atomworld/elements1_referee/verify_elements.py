"""
verify_elements.py -- standalone re-check of the ELEMENTS-1 referee output.

Reads the published artifacts and re-derives, from scratch where it can, the
things the freeze stakes.  Exits nonzero on any failure.

  V1  dual-route agreement per species, at every staked geometry (gate R1)
  V2  H2 against the BANKED referee: h2_core.py run live on both of its own
      routes, at every one of the bank's own grid points (gate R2's "H2 must
      reproduce the banked referee exactly as before").  The gap to
      h2_potential.json's stored strings is measured separately and attributed
  V3  the eigenvalue certificates: every reported energy carries a residual
      bound small enough that the reported digits are earned, and the Temple
      bound is recomputed here rather than trusted
  V4  gate E1 recomputed from the tables: no closed-shell well deeper than 1e-4
  V5  gate E3, the sandbox contract: schema, monotone knots, Hermite
      reproduction at the knots, envelope monotone and rounded UP
  V6  the atoms: ground spin re-derived from the recorded sector energies
  V7  the plants reproduced: the Z-mutation and the basis-mutation are re-run
      and must fire again, and the empty-sector control must still VOID
  V9  the guards are CONNECTED: the pool wrapper, the run lock and the
      merge-not-narrow rule are on the path `main()` actually takes -- a guard
      that is correct, tested and uncalled is the failure this section exists
      for, and it has happened here

Usage:  python verify_elements.py [--quick]
        --quick skips the from-scratch recomputations (V2 spot only, V7 short).
"""

import json
import os
import sys
import time

from mpmath import mp, mpf, nstr, sqrt

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
# h2_core.py is the BANKED foundation and lives one level up beside the
# freeze it belongs to.  Import it from there rather than keeping a copy
# here: two copies of a bank is how a bank stops being one.
sys.path.insert(1, os.path.dirname(HERE))

import elements_core as EC          # noqa: E402
import fci as F                     # noqa: E402
import runner as R                  # noqa: E402
import species as SP                # noqa: E402
import curve as CV                  # noqa: E402

BANK = "/home/emoore/CIRISHolon/conformance/atomworld/h2_potential.json"

FAIL = []
NOTE = []


def check(name, ok, detail=""):
    print(("  PASS  " if ok else "  FAIL  ") + name + ("   " + detail
                                                       if detail else ""),
          flush=True)
    if not ok:
        FAIL.append(name)


def load(fn):
    with open(os.path.join(HERE, fn)) as f:
        return json.load(f)


# ---------------------------------------------------------------------------
def h2_energy(Rv):
    mol = EC.molecule([(1, (0, 0, 0)), (1, (0, 0, mpf(Rv)))])
    C, _ = F.lowdin_orbitals(mol["S"])
    h, g = F.mo_integrals(mol, C)
    sp = F.DetSpace(mol["nbf"], 1, 1)
    r = F.solve_certified(F.RouteAOp(sp, h, g), tol_digits=mp.dps - 8)
    return r["energy"] + mol["E_nuc"]


def v1_dual_route(pot):
    print("\n[V1] dual-route FCI agreement (gate R1)")
    worst = 0.0
    for nm, s in pot["species"].items():
        dg = s["diagnostics"]
        d = dg["route_agreement_max_abs"]
        unex = dg.get("route_agreement_unexplained_max")
        worst = max(worst, d)
        if unex is None:                     # older artifact
            check("%-4s A vs B over %d geometries" % (nm, dg["n_grid"]),
                  d < 1e-40, "max |E_A - E_B| = %.2e" % d)
        else:
            # The routes agree when their difference is within what their own
            # convergence bounds allow. A flat threshold on the raw difference
            # convicts a route for having converged less far at one geometry,
            # which is a fact about that solve and not a disagreement about the
            # energy.
            check("%-4s A vs B over %d geometries, beyond their own bounds"
                  % (nm, dg["n_grid"]),
                  unex < 1e-40,
                  "raw max |E_A - E_B| = %.2e, of which %.2e is unexplained by "
                  "the two routes' own bounds (A %.2e, B %.2e)"
                  % (d, unex, dg.get("eigen_temple_bound_max", 0.0),
                     dg.get("route_B_bound_max", 0.0)))
        c = s["diagnostics"].get("route_C_agreement_max_abs")
        if c is not None:
            check("%-4s vs route C (Fock-space ladder operators)" % nm,
                  c < 1e-40, "max |E_A - E_C| = %.2e" % c)
    print("  worst dual-route deviation over all species: %.2e" % worst)


def v2_banked_h2(quick):
    """Gate R2's "H2 must reproduce the banked referee exactly as before".

    The comparison is against h2_core.py -- the banked referee's CODE, run live
    at the banked working precision, on both of its own routes.  It is NOT
    against the 50-digit strings stored in h2_potential.json, and it must not
    be: those strings sit about 5e-50 from what h2_core itself computes at the
    same R (the file stores R rounded to 50 digits while it evaluated E at the
    unrounded value, on top of its own output rounding).  That gap is a property
    of the banked FILE, not of any engine, and this check measures it separately
    so the two are never confused.
    """
    print("\n[V2] H2 against the banked referee (gate R2)")
    if not os.path.exists(BANK):
        check("banked h2_potential.json present", False, BANK)
        return
    import h2_core as HB
    bank = json.load(open(BANK))
    mp.dps = R.DPS
    Rs, Es = bank["exact"]["R_grid_bohr"], bank["exact"]["E_hartree"]
    idx = list(range(0, len(Rs), 1 if not quick else 23))
    w_live = w_store = w_bankself = mpf(0)
    t0 = time.time()
    for i in idx:
        Rv = mpf(Rs[i])
        mine = h2_energy(Rv)
        la, lb = HB.energy_route_a(Rv), HB.energy_route_b(Rv)
        w_live = max(w_live, abs(mine - la), abs(mine - lb))
        w_store = max(w_store, abs(mine - mpf(Es[i])))
        w_bankself = max(w_bankself, abs(la - mpf(Es[i])))
    check("H2 reproduces h2_core LIVE (both banked routes) at %d %spoints"
          % (len(idx), "" if not quick else "sampled "),
          w_live < mpf("1e-52"),
          "max |dE| = %s   (%.1fs)" % (nstr(w_live, 6), time.time() - t0))
    check("the residual gap to the STORED strings is the bank file's own, "
          "not this engine's",
          abs(w_store - w_bankself) <= w_live * 10 + mpf("1e-55"),
          "engine vs stored %s ; h2_core vs stored %s"
          % (nstr(w_store, 6), nstr(w_bankself, 6)))
    NOTE.append("h2_potential.json's stored 50-digit E strings differ from "
                "h2_core's own live values by up to %s Ha; a 50th-digit string "
                "match with that file is not achievable by any engine, "
                "including the one that wrote it." % nstr(w_bankself, 4))
    check("banked referee's own gates all true",
          all(bank["gates"].values()), str(bank["gates"]))


def v3_certificates(pot):
    print("\n[V3] eigenvalue certificates")
    for nm, s in pot["species"].items():
        d = s["diagnostics"]
        rm = d["eigen_residual_max"]
        tb = d.get("eigen_temple_bound_max", 0.0)
        tot = d.get("energy_uncertainty_total", tb)
        tgt = d.get("certificate_target") or 1e-48
        dig = d.get("certified_significant_digits_min")
        # The declared bound is the larger of Temple's (rigorous for the
        # eigensolve) and the measured route A vs route B deviation (empirical,
        # for the arithmetic upstream). Checking Temple alone would pass a
        # one-determinant species on a bound of exactly zero.
        check("%-4s declared energy bound covers all 50 reported digits" % nm,
              tot <= tgt and tot > 0,
              "declared %.3e vs target %.3e  (Temple %.2e, route A-B %.2e); "
              "worst geometry carries %s certified significant digits"
              % (tot, tgt, tb, d.get("route_agreement_max_abs", 0.0), dig))
        check("%-4s no column declares a zero uncertainty" % nm,
              tot > 0 and d.get("interpolant_window_spread_d1", 0.0) > 0,
              "energy %.2e, interpolant spread %.2e"
              % (tot, d.get("interpolant_window_spread_d1", 0.0)))


def v4_gate_e1(pot):
    print("\n[V4] gate E1 -- the emergent negatives, recomputed from the tables")
    for nm in SP.E2_UNBOUND:
        s = pot["species"][nm]
        E = [mpf(x) for x in s["exact"]["E_hartree"]]
        Ea = mpf(s["exact"]["E_asymptote"])
        depth = max(Ea - e for e in E)
        rec = mpf(s["exact"]["well_depth_hartree"])
        check("%-4s well depth recomputed matches the record" % nm,
              abs(depth - rec) < mpf("1e-45"),
              "recomputed %s" % nstr(depth, 8))
        check("%-4s has no well deeper than 1e-4 Ha" % nm,
              depth < mpf("1e-4"), "deepest excursion below dissociation = %s "
                                   "Ha" % nstr(depth, 8))
        # The derivative-free witness: compare the energies themselves.
        dec = all(E[i] > E[i + 1] for i in range(len(E) - 1))
        check("%-4s energies strictly decrease along the whole staked grid" % nm,
              dec and dec == s["strictly_decreasing_on_grid"],
              "%d of %d steps go downhill (record says %s)"
              % (sum(1 for i in range(len(E) - 1) if E[i] > E[i + 1]),
                 len(E) - 1, s["strictly_decreasing_on_grid"]))
        check("%-4s never reaches below its dissociation limit" % nm,
              not s["curve_reaches_below_dissociation"],
              "min(E) - E_asymptote = %s Ha" % nstr(min(E) - Ea, 8))


def v5_gate_e3(pot):
    print("\n[V5] gate E3 -- the sandbox contract")
    need_top = {"R_grid_bohr", "E_hartree", "F_hartree_per_bohr",
                "E2_hartree_per_bohr2", "hermite", "max_curvature_up_to_E",
                "exact", "provenance", "diagnostics", "units"}
    need_prov = {"Z1", "Z2", "mass1_u", "mass2_u", "ground_Sz",
                 "n_determinants", "basis", "method",
                 "working_precision_dps"}
    for nm, s in pot["species"].items():
        check("%-4s emits the full schema" % nm, need_top <= set(s),
              "missing %s" % (need_top - set(s)) if not need_top <= set(s)
              else "")
        check("%-4s carries per-pair provenance" % nm,
              need_prov <= set(s["provenance"]), "")
        g = s["R_grid_bohr"]
        check("%-4s grid strictly increasing" % nm,
              all(g[i] < g[i + 1] for i in range(len(g) - 1)), "")
        h = s["hermite"]
        ok = (h["knots_bohr"] == g and h["values_hartree"] == s["E_hartree"]
              and len(h["derivatives_hartree_per_bohr"]) == len(g))
        check("%-4s Hermite table reproduces E and F at its own knots" % nm, ok,
              "")
        dF = max(abs(h["derivatives_hartree_per_bohr"][i]
                     + s["F_hartree_per_bohr"][i]) for i in range(len(g)))
        check("%-4s Hermite knot slopes are -F" % nm, dF < 1e-9,
              "max |d + F| = %.2e" % dF)
        rungs = s["max_curvature_up_to_E"]["rungs"]
        if rungs:
            et = [r["E_total_hartree"] for r in rungs]
            check("%-4s envelope rungs increase in energy" % nm,
                  all(et[i] < et[i + 1] for i in range(len(et) - 1)), "")
            cm = [r["max_abs_E2_hartree_per_bohr2"] for r in rungs]
            check("%-4s envelope stiffness is non-decreasing (round UP is "
                  "safe)" % nm,
                  all(cm[i] <= cm[i + 1] * (1 + 1e-12)
                      for i in range(len(cm) - 1)),
                  "%d rungs, stiffness %.4g -> %.4g" % (len(cm), cm[0], cm[-1]))
        bound = s["bound"]
        if not bound:
            # judged on the energies, not on a derivative column
            check("%-4s (unbound) emits a repulsive-only table" % nm,
                  s["R_e"] is None and s["strictly_decreasing_on_grid"],
                  "no R_e, and E falls at every step of the grid")


def v6_atoms(atoms):
    print("\n[V6] atoms -- ground spin re-derived from the recorded sectors")
    mp.dps = R.DPS
    for sym, a in atoms["atoms"].items():
        en = {int(k): mpf(v["E_A"]) for k, v in a["sectors"].items()}
        two_s, emin, hits = SP.ground_spin_from_sectors(en, mpf("1e-40"))
        check("%-3s ground 2Sz = %d re-derived" % (sym, a["ground_two_Sz"]),
              two_s == a["ground_two_Sz"] and hits == a["degenerate_sectors"],
              "sectors attaining the minimum: %s" % hits)
        check("%-3s recorded ground energy is the sector minimum" % sym,
              R.s(emin) == a["E"], "")
        # the second, independent derivation of the same multiplicity
        s2 = a.get("S2_expectation")
        check("%-3s <S^2> confirms 2S = %s independently of the degeneracy "
              "pattern" % (sym, a["ground_two_Sz"]),
              a.get("spin_derivations_agree") is True and s2 is not None,
              "<S^2> = %s, off S(S+1) by %s" % (s2,
                                                a.get("S2_deviation_from_exact")))


def v8_spin(pot):
    """Consume the spin block, rather than merely shipping it.

    The engine lane found a worst_residual its own emitter had recorded and
    printed for the whole campaign with nothing obliged to read it: a curve
    whose solver hit its iteration cap would have shipped looking healthy with
    the evidence in a field no consumer had to touch.  An audit here found
    eighteen emitted keys with no reader, most of them in this block -- including
    the deviation from S(S+1), which is the number that says whether a spin
    reading is a reading at all.  A diagnostic nobody must read is not a check.
    """
    print("\n[V8] the spin block is consumed, not just carried")
    for nm, s in pot["species"].items():
        sp = s.get("spin")
        check("%-4s carries a spin audit" % nm, sp is not None, "")
        if sp is None:
            continue
        n = len(s["R_grid_bohr"])
        byg = sp.get("two_S_by_geometry") or []
        check("%-4s spin audit covers every staked geometry" % nm,
              sp.get("n_geometries") == n and len(byg) == n
              and len(sp.get("ground_level_sizes") or []) == n,
              "%s of %d geometries, %d per-geometry readings"
              % (sp.get("n_geometries"), n, len(byg)))
        # the reading has to BE a reading: <S^2> must equal S(S+1)
        dev = mpf(sp.get("max_abs_deviation_from_exact_S_S_plus_1") or "1")
        check("%-4s every <S^2> is an exact S(S+1)" % nm, dev < mpf("1e-30"),
              "worst deviation %s" % nstr(dev, 6))
        # and the parity must match the electron count, for every reading
        nelec = s["provenance"]["Z1"] + s["provenance"]["Z2"]
        check("%-4s every multiplicity has the right parity for %d electrons"
              % (nm, nelec),
              all((t % 2) == (nelec % 2) for t in byg),
              "2S values seen: %s" % sorted(set(byg)))
        # the summary fields must follow from the per-geometry column
        crossings = [s["R_grid_bohr"][i] for i in range(1, len(byg))
                     if byg[i] != byg[i - 1]]
        declared = sp.get("multiplicity_crossings_bohr") or []
        check("%-4s declared crossings follow from the per-geometry column" % nm,
              len(declared) <= len(crossings),
              "declared %d, sign changes in the column %d"
              % (len(declared), len(crossings)))
        check("%-4s multiplicity_changes flag matches the column" % nm,
              bool(sp.get("multiplicity_changes_along_the_curve"))
              == (len(set(byg)) > 1),
              "flag %s, distinct 2S in column %s"
              % (sp.get("multiplicity_changes_along_the_curve"),
                 sorted(set(byg))))
        # EVERY SUMMARY FIELD RE-DERIVED FROM THE PER-GEOMETRY COLUMNS.
        # An audit that stopped counting the writer as a reader found fifteen
        # emitted keys that nothing downstream touched, most of them here: a
        # summary nobody recomputes is a claim, and this file is full of
        # columns that make each one checkable in one line.
        rbg = sp.get("resolved_by_geometry") or []
        check("%-4s resolution column covers every geometry" % nm,
              len(rbg) == n, "%d of %d" % (len(rbg), n))
        check("%-4s the distinct-2S summary follows from the column" % nm,
              sorted(sp.get("two_S") or []) == sorted(set(byg)),
              "declared %s, column has %s"
              % (sp.get("two_S"), sorted(set(byg))))
        exact = [mpf(t) / 2 * (mpf(t) / 2 + 1) for t in byg] or [mpf(0)]
        for fld, want in (("S2_max", max(exact)), ("S2_min", min(exact))):
            got = mpf(sp.get(fld) or "-1")
            check("%-4s %s is the exact S(S+1) of the column" % (nm, fld),
                  abs(got - want) < mpf("1e-10"),
                  "declared %s, column implies %s"
                  % (nstr(got, 8), nstr(want, 8)))
        check("%-4s 'n of n' resolved matches the column" % nm,
              sp.get("spin_resolved_at_n_of_n")
              == "%d of %d" % (sum(1 for r in rbg if r), len(rbg)),
              "declared %s, column has %d of %d"
              % (sp.get("spin_resolved_at_n_of_n"),
                 sum(1 for r in rbg if r), len(rbg)))
        lastres = max([i for i in range(len(rbg)) if rbg[i]], default=-1)
        # THE EXACT DECIMAL STRINGS, not the f64 convenience column.  The spin
        # summaries are geometry STRINGS, and comparing them to `R_grid_bohr`
        # -- which is float in this file -- failed while printing two lines of
        # identical-looking digits, because repr of the float is the same text.
        gs = s["R_grid_exact_decimal"]
        check("%-4s 'resolved out to' is the last resolved geometry" % nm,
              sp.get("spin_resolved_out_to_bohr")
              == (gs[lastres] if lastres >= 0 else None),
              "declared %s, column's last resolved %s"
              % (sp.get("spin_resolved_out_to_bohr"),
                 gs[lastres] if lastres >= 0 else None))
        want_deg = (gs[lastres + 1] if 0 <= lastres < len(gs) - 1 else None)
        check("%-4s 'degenerate from' is the geometry after it" % nm,
              sp.get("degenerate_from_bohr") == want_deg,
              "declared %s, expected %s"
              % (sp.get("degenerate_from_bohr"), want_deg))
        want_off = [gs[i] for i in range(len(byg)) if byg[i] != byg[0]]
        check("%-4s the differing-S list is exactly the column's" % nm,
              list(sp.get("geometries_differing_in_S") or []) == want_off,
              "declared %d, column gives %d"
              % (len(sp.get("geometries_differing_in_S") or []),
                 len(want_off)))
        # and the derivative block's unresolved list must be the same set
        unres = [gs[i] for i in range(len(rbg)) if not rbg[i]]
        dpu = (s.get("derivative_provenance") or {}).get(
            "geometries_where_multiplicity_is_unresolved")
        if dpu is not None:
            check("%-4s the derivative block's unresolved list matches" % nm,
                  list(dpu) == unres,
                  "declared %d, column gives %d" % (len(dpu), len(unres)))
        # the claim's boundary must have been probed
        probe = sp.get("beyond_grid_probe")
        check("%-4s probed past the staked grid" % nm,
              bool(probe) and len(probe) >= 3,
              "%d probes beyond %s; %s"
              % (len(probe or []), s["R_grid_bohr"][-1],
                 ("degeneracy by %s" % next((p["R"] for p in probe
                                             if not p["resolved"]), None))
                 if probe and any(not p["resolved"] for p in probe)
                 else "still resolved at the far probe"))
        for pr in (probe or []):
            # An UNRESOLVED probe's 2S is the solver's arbitrary mixture and
            # means nothing -- that is what unresolved says.  Only a resolved
            # reading is required to be a possible multiplicity; demanding it
            # of the others would fail on exactly the geometries the column
            # already declares meaningless.
            if pr.get("resolved"):
                check("%-4s probe at %s reads a possible multiplicity"
                      % (nm, pr["R"]),
                      (pr["two_S"] % 2) == (nelec % 2),
                      "2S = %s for %d electrons" % (pr.get("two_S"), nelec))
            check("%-4s probe at %s reports a level size" % (nm, pr["R"]),
                  isinstance(pr.get("level_size"), int)
                  and pr["level_size"] >= 1,
                  "level_size = %s" % pr.get("level_size"))


def v7_plants(quick):
    """Re-run the plants.  In quick mode only the cheap cases are re-run; the
    expensive N2 cases are compared against the recorded run instead."""
    print("\n[V7] plants reproduced")
    import plants as PL
    if not os.path.exists(os.path.join(HERE, "elements_plants.json")):
        print("  SKIP  the recorded plant run is not present yet; re-running "
              "the cheap cases only for a live check")
        b = PL.plant_basis(verbose=False, cases="cheap")
        live = [r for r in b if r["carrier_nonzero"]]
        dead = [r for r in b if not r["carrier_nonzero"]]
        check("basis-mutation fires on every live sector",
              bool(live) and all(r["fired"] for r in live),
              "shifts %s Ha" % [r["shift_hartree"] for r in live])
        check("basis-mutation VOIDs on the empty-sector control",
              bool(dead) and all("VOID" in r["verdict"] for r in dead), "")
        return
    rec = load("elements_plants.json")
    fp = EC.basis_fingerprint()
    check("the recorded plant run used the basis now in force",
          rec.get("basis_fingerprint") in (None, fp),
          "recorded %s, current %s" % (rec.get("basis_fingerprint"), fp))
    b = PL.plant_basis(verbose=False, cases=("cheap" if quick else "all"))
    live = [r for r in b if r["carrier_nonzero"]]
    dead = [r for r in b if not r["carrier_nonzero"]]
    check("basis-mutation fires on every live sector",
          bool(live) and all(r["fired"] for r in live),
          "shifts %s Ha" % [r["shift_hartree"] for r in live])
    check("basis-mutation VOIDs on the empty-sector control",
          bool(dead) and all("VOID" in r["verdict"] for r in dead), "")
    # Match by CASE NAME, never by position: quick mode omits the expensive
    # N2 cases, so a positional zip silently compares the empty-sector control
    # against N2's record and reports a difference that is only the misalignment.
    by_case = {r["case"]: r for r in rec["plant_ii_basis_mutation"]}
    for now in b:
        was = by_case.get(now["case"])
        check("basis-mutation %s reproduces" % now["case"][:34],
              was is not None and now["shift_hartree"] == was["shift_hartree"],
              "%s vs %s" % (now["shift_hartree"],
                            was["shift_hartree"] if was else "NO RECORD"))
    if not quick:
        z = PL.plant_z(verbose=False)
        check("Z-mutation fires on every case",
              all(r["fired"] for r in z),
              "shifts %s Ha" % [r["shift_hartree"] for r in z])
        by_case_z = {r["case"]: r for r in rec["plant_i_Z_mutation"]}
        for now in z:
            was = by_case_z.get(now["case"])
            check("Z-mutation %s reproduces" % now["case"],
                  was is not None
                  and now["shift_hartree"] == was["shift_hartree"],
                  "%s vs %s" % (now["shift_hartree"],
                                was["shift_hartree"] if was else "NO RECORD"))


def v9_guards_are_connected():
    """The guards this pipeline depends on must be REACHED by the pipeline.

    Correct, tested, and never called are three different states, and this
    campaign has now shipped the third: `_install_safe()` populated the pool's
    safety map, was covered by a test, and was called by nothing in `main()`,
    so every pool ran unprotected -- including the two that died of exactly the
    failure it was written for.  A verifier that reads only the numbers cannot
    see that, because the numbers are right until the day the job hangs.
    """
    print("\n[V9] the guards are wired into the path that runs")
    src = open(os.path.join(HERE, "build_curves.py")).read()
    body = src.split("def pmap(", 1)[1].split("\ndef ", 1)[0]
    check("every pooled call is wrapped, with no registry to forget",
          "p.map(_Safe(" in body, "pmap wraps its callable itself")
    main_src = src.split("\ndef main():", 1)[1]
    check("main() fires the pool guard in-process before any stage runs",
          "selftest_pool_guard()" in main_src)
    check("main() takes a run lock before any stage runs",
          "acquire_run_locks(" in main_src)
    stages = main_src.split("selftest_pool_guard()", 1)[1]
    check("the guard runs BEFORE the stages, not after them",
          "stage1(" in stages and "stage_spin(" in stages,
          "stages follow the self-test")
    check("the partial assembly merges rather than narrowing",
          "merge_partial(" in main_src and "def merge_partial(" in src,
          "a one-species run cannot delete the others")
    import build_curves as BC
    probe = BC.pmap.__globals__.get("_Safe")
    check("the wrapper is importable and picklable by reference",
          probe is not None and probe.__module__ == "build_curves",
          "class %s" % (probe.__name__ if probe else "MISSING"))
    # The CLASS of defect, not just the instance: any function of this lane's
    # own that only its test calls has the `_install_safe` shape, and a new one
    # fails here rather than waiting for a job to hang.  `h2_core.py` is the
    # banked foundation and is not ours to prune, so its two are named.
    import subprocess
    r = subprocess.run([sys.executable, os.path.join(HERE,
                        "_dead_guard_audit.py")], capture_output=True,
                       text=True)
    tail = r.stdout.split("CALLED ONLY BY TESTS", 1)
    tests_only = [l.strip() for l in (tail[1] if len(tail) > 1 else "")
                  .splitlines() if l.startswith("   ")]
    check("no function is reachable only from its own test", not tests_only,
          "; ".join(tests_only) if tests_only else "none")
    unref = [l.strip() for l in r.stdout.split("CALLED BY NOTHING AT ALL", 1)[1]
             .split("CALLED ONLY BY TESTS")[0].splitlines()
             if l.startswith("   ")]
    stray = [u for u in unref if not u.startswith("h2_core.py")]
    check("no unreferenced function outside the banked h2_core", not stray,
          "; ".join(stray) if stray else
          "%d in h2_core (the bank, left as banked)" % len(unref))

    # The FIELD side of the same question, pinned.  Six emitted keys have no
    # reader and no write-time guard; every one of them is prose written for a
    # person, and they are listed by name so that a new INERT DATA field fails
    # here instead of shipping quietly.  "There because I decided" and "there
    # because nobody looked" are different states and only this tells them
    # apart.
    # The allowlist lives in prose_fields.txt, NOT here: the audit searches
    # this file for key names, so naming the exempt fields in it would launder
    # them out of the inert bucket by the act of exempting them.
    PROSE = set()
    with open(os.path.join(HERE, "prose_fields.txt")) as f:
        for ln in f:
            ln = ln.strip()
            if ln and not ln.startswith("#"):
                PROSE.add(ln)
    r2 = subprocess.run([sys.executable, os.path.join(HERE,
                        "_inert_audit.py")], capture_output=True, text=True)
    tail2 = r2.stdout.split("NEITHER read nor guarded", 1)
    inert = set(l.strip() for l in (tail2[1] if len(tail2) > 1 else "")
                .splitlines() if l.startswith("     "))
    check("no emitted field is inert except the named prose",
          inert <= PROSE,
          "unexpected: %s" % sorted(inert - PROSE) if inert - PROSE
          else "%d prose fields, all named" % len(inert))


def v10_grid_provenance(dropdir=None):
    """Read the grid rule out of the EMITTED FILE and rebuild the grid from it.

    The point of shipping a rule is that a reader can regenerate the knots
    without trusting the lane that chose them, so this section does exactly
    that: it takes rmin, rmax, nbase, nsplit and the window from the file's own
    `grid_provenance`, rebuilds the staked grid, applies the declared subset
    rule, and compares the result to the file's `R_grid_bohr`.  A rule that
    does not reproduce the grid it describes is a chosen grid with a story.
    """
    print("\n[V10] the emitted grid rule reproduces the emitted grid")
    # FIND THE DROP, AND FAIL IF THERE ISN'T ONE.  Returning quietly when the
    # directory is missing made this section check nothing at all in the
    # committed copy of the referee -- eighteen checks in the scratchpad, zero
    # in the repo, same green line at the bottom.  A section that cannot fail
    # where a reader will run it is the campaign's own defect in the verifier.
    cands = [dropdir] if dropdir else [
        os.path.join(HERE, "engine_handoff", "elements1"),
        os.path.abspath(os.path.join(HERE, "..", "..", "..", "engine",
                                     "crates", "holon-chem", "tests", "data",
                                     "elements1")),
    ]
    d, files = None, []
    for c in cands:
        if c and os.path.isdir(c):
            f = sorted(x for x in os.listdir(c)
                       if x.endswith(".json") and x[:-5] in SP.DIATOMICS)
            if f:
                d, files = c, f
                break
    check("a drop of pair files is present to check",
          bool(files), "looked in: %s" % "; ".join(str(c) for c in cands))
    if not files:
        return
    print("  drop: %s" % d)
    for f in files:
        name = f[:-5]
        j = json.load(open(os.path.join(d, f)))
        gp = j.get("grid_provenance")
        if gp is None:
            check("%s declares its grid rule" % name, False,
                  "no grid_provenance block")
            continue
        sp_ = gp["staked_parameters"]
        full = CV.build_grid(sp_["rmin_bohr"], sp_["rmax_bohr"],
                             sp_["nbase"], sp_["well_window_bohr"],
                             sp_["nsplit"])
        check("%s: full grid rebuilt from the file's own parameters" % name,
              len(full) == gp["full_grid_knots"],
              "%d rebuilt vs %d declared" % (len(full), gp["full_grid_knots"]))
        want = [str(x) for x in j["R_grid_bohr"]]
        if gp.get("subset_parameters"):
            got = [str(x) for x in SP.sparse_subset(
                full, sp_["well_window_bohr"], gp["subset_parameters"])]
            label = "%s: the declared SUBSET rule reproduces the emitted knots"
        else:
            got = [str(x) for x in full]
            label = "%s: every knot of the staked grid is emitted, as declared"
        check(label % name, got == want,
              "%d knots, first mismatch %s" % (
                  len(want),
                  next((i for i in range(min(len(got), len(want)))
                        if got[i] != want[i]), "none")))
        check("%s: the emitted knot count matches its own declaration" % name,
              len(want) == gp["emitted_knots"])


def main():
    quick = "--quick" in sys.argv
    mp.dps = R.DPS
    print("=" * 74)
    print("verify_elements.py -- ELEMENTS-1 standalone re-check"
          + ("  [quick]" if quick else ""))
    print("=" * 74)
    if os.path.exists(os.path.join(HERE, "elements_potential.json")):
        pot = load("elements_potential.json")
    else:
        pot = load("elements_potential_partial.json")
        print("\nNOTE: verifying elements_potential_partial.json -- the full "
              "nine-species artifact is not present.")
        print("      species present: %s" % ", ".join(sorted(pot["species"])))
        for nm, why in (pot.get("incomplete_species") or {}).items():
            print("      INCOMPLETE: %s" % why)
    atoms = load("elements_atoms.json")
    missing = [nm for nm in SP.E2_UNBOUND if nm not in pot["species"]]
    if missing:
        print("      (gate E1 cannot be scored: %s absent)" % missing)
    v1_dual_route(pot)
    v2_banked_h2(quick)
    v3_certificates(pot)
    if not missing:
        v4_gate_e1(pot)
    v5_gate_e3(pot)
    v6_atoms(atoms)
    v8_spin(pot)
    v7_plants(quick)
    v9_guards_are_connected()
    v10_grid_provenance()
    if NOTE:
        print("\nNOTES")
        for n in NOTE:
            print("  - " + n)
    print("\n" + "=" * 74)
    if FAIL:
        print("FAILED (%d): %s" % (len(FAIL), "; ".join(FAIL)))
        return 1
    print("ALL CHECKS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
