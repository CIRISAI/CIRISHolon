#!/usr/bin/env python3
"""
verify_atom_core.py -- standalone re-check of the H2/STO-3G/FCI potential core.

Reads h2_potential.json and re-derives everything it checks from h2_core.py.
It never trusts a number in the JSON: every comparison is against a value
computed fresh in this process.

EXIT CONTRACT (house law: a missing check must refuse)
  0  every REQUIRED check ran and passed
  1  a required check ran and FAILED
  2  a required check did not run, or the contract is malformed / unreadable

Usage:
  python3 verify_atom_core.py            # sampled re-check (default)
  python3 verify_atom_core.py --full     # re-check every grid point
  python3 verify_atom_core.py --json PATH
"""

import argparse
import json
import os
import random
import sys
from multiprocessing import Pool

from mpmath import mp, mpf, nstr, diff

DPS = 60
mp.dps = DPS

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import h2_core as H  # noqa: E402

NPROC = 16

# Every check that MUST run.  If any name here is missing from the results at
# the end, the verifier refuses (exit 2) rather than reporting success.
REQUIRED = [
    "V1_contract_wellformed",
    "V2_route_agreement",
    "V3_energies_match_model",
    "V4_forces_match_model",
    "V5_asymptote_in_model",
    "V6_exactly_one_minimum",
    "V7_asymptote_from_below",
    "V8_diverges_at_zero",
    "V9_R_e_is_stationary_minimum",
    "V10_D_e_consistent",
    "V11_hermite_exact_at_knots",
    "V12_hermite_bound_holds",
    "V13_exact_strings_agree_with_f64",
    "V14_hermite_slopes_match_model",
    "V15_E2_matches_model",
    "V16_envelope_wellformed",
    "V17_envelope_turning_points_correct",
    "V18_envelope_curvature_not_understated",
    "V19_envelope_dt_consistent",
]

# Tolerances
TOL_ROUTE = mpf("1e-52")          # route agreement, working precision 60 dps
TOL_F64_REL = 4e-16               # storing an mpf as a double
HERMITE_SLACK = mpf("1.02")       # verify samples random t between the build's
TOL_KNOT_SLOPE_REL = 2e-15        # (h*d)/h round trip at a knot, a few ulp


def s(x, n=8):
    return nstr(mpf(x), n)


# ---- parallel workers ------------------------------------------------------
def _w_both(rs):
    mp.dps = DPS
    a, b = H.energy_both(mpf(rs))
    return nstr(a, DPS), nstr(b, DPS)


def _w_E(rs):
    mp.dps = DPS
    return nstr(H.energy_route_a(mpf(rs)), DPS)


def _w_F(rs):
    mp.dps = DPS
    return nstr(-diff(H.energy_route_a, mpf(rs)), DPS)


def _w_E2(rs):
    mp.dps = DPS
    return nstr(diff(H.energy_route_a, mpf(rs), 2), DPS)


def _w_EF(rs):
    mp.dps = DPS
    R = mpf(rs)
    return nstr(H.energy_route_a(R), DPS), nstr(-diff(H.energy_route_a, R), DPS)


def pmap(fn, args):
    if not args:
        return []
    with Pool(min(NPROC, len(args))) as pool:
        return pool.map(fn, args)


# ---- f64 cubic Hermite, exactly as a renderer would evaluate it -------------
def hermite_f64(knots, vals, ders, x):
    lo, hi = 0, len(knots) - 1
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if knots[mid] <= x:
            lo = mid
        else:
            hi = mid
    x0, x1 = knots[lo], knots[lo + 1]
    y0, y1 = vals[lo], vals[lo + 1]
    d0, d1 = ders[lo], ders[lo + 1]
    h = x1 - x0
    t = (x - x0) / h
    t2, t3 = t * t, t * t * t
    val = ((2 * t3 - 3 * t2 + 1) * y0 + (t3 - 2 * t2 + t) * h * d0
           + (-2 * t3 + 3 * t2) * y1 + (t3 - t2) * h * d1)
    der = ((6 * t2 - 6 * t) * y0 + (3 * t2 - 4 * t + 1) * h * d0
           + (-6 * t2 + 6 * t) * y1 + (3 * t2 - 2 * t) * h * d1) / h
    return val, der


# ---------------------------------------------------------------------------
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", default=os.path.join(HERE, "h2_potential.json"))
    ap.add_argument("--full", action="store_true",
                    help="re-check every grid point instead of a sample")
    ap.add_argument("--sample", type=int, default=28)
    ap.add_argument("--seed", type=int, default=20260825)
    args = ap.parse_args()

    results = {}
    lines = []

    def emit(x=""):
        print(x, flush=True)
        lines.append(x)

    def record(name, ok, detail):
        results[name] = bool(ok)
        emit(f"  [{'PASS' if ok else 'FAIL'}] {name}: {detail}")

    emit("=" * 78)
    emit("verify_atom_core.py -- independent re-check of h2_potential.json")
    emit("=" * 78)

    # ---- read contract -----------------------------------------------------
    try:
        with open(args.json) as f:
            doc = json.load(f)
    except Exception as exc:  # noqa: BLE001
        emit(f"REFUSE: cannot read contract {args.json}: {exc}")
        return 2
    emit(f"contract: {args.json}")
    emit(f"model   : {doc.get('model')}")
    emit(f"mode    : {'FULL' if args.full else f'sampled (n={args.sample})'}")
    emit()

    # ---- V1: well-formedness ----------------------------------------------
    try:
        need_top = ["model", "precision_digits", "R_grid_bohr", "E_hartree",
                    "F_hartree_per_bohr", "E2_hartree_per_bohr2", "R_e",
                    "D_e", "E_asymptote", "E2_at_R_e", "max_curvature_up_to_E",
                    "hermite", "exact"]
        missing = [k for k in need_top if k not in doc]
        herm = doc.get("hermite", {})
        need_h = ["knots_bohr", "values_hartree", "derivatives_hartree_per_bohr",
                  "max_abs_error_E_hartree", "max_abs_error_F_hartree_per_bohr"]
        missing += ["hermite." + k for k in need_h if k not in herm]
        R = doc["R_grid_bohr"]
        E = doc["E_hartree"]
        F = doc["F_hartree_per_bohr"]
        E2j = doc["E2_hartree_per_bohr2"]
        same_len = len(R) == len(E) == len(F) == len(E2j) \
            == len(herm["knots_bohr"]) \
            == len(herm["values_hartree"]) \
            == len(herm["derivatives_hartree_per_bohr"])
        increasing = all(R[i] < R[i + 1] for i in range(len(R) - 1))
        knots_match = all(a == b for a, b in zip(R, herm["knots_bohr"]))
        vals_match = all(a == b for a, b in zip(E, herm["values_hartree"]))
        span = (abs(R[0] - 0.3) < 1e-12 and abs(R[-1] - 10.0) < 1e-12)
        enough = len(R) >= 200
        ok = (not missing) and same_len and increasing and knots_match \
            and vals_match and span and enough
        record("V1_contract_wellformed", ok,
               f"{len(R)} points (>=200: {enough}), R in [{R[0]}, {R[-1]}] "
               f"(span ok: {span}), strictly increasing: {increasing}, "
               f"arrays aligned: {same_len and knots_match and vals_match}, "
               f"missing keys: {missing or 'none'}")
        if not ok:
            emit("REFUSE: contract malformed; downstream checks are meaningless.")
            return 2
    except Exception as exc:  # noqa: BLE001
        emit(f"REFUSE: contract malformed: {exc}")
        return 2

    n = len(R)
    rng = random.Random(args.seed)
    idxs = list(range(n)) if args.full else sorted(
        set([0, 1, n // 2, n - 2, n - 1]
            + rng.sample(range(n), min(args.sample, n))))
    Rs = [doc["exact"]["R_grid_bohr"][i] for i in idxs]

    emit(f"re-evaluating the model at {len(idxs)} grid points ...")
    both = pmap(_w_both, Rs)
    emit()

    # ---- V2: route agreement ----------------------------------------------
    worst = mpf(0)
    where = Rs[0]
    for rs, (a, b) in zip(Rs, both):
        d = abs(mpf(a) - mpf(b))
        if d > worst:
            worst, where = d, rs
    record("V2_route_agreement", worst < TOL_ROUTE,
           f"max |E_2x2CI - E_FockED| = {s(worst)} at R = {s(mpf(where), 12)} "
           f"over {len(Rs)} points (tol {s(TOL_ROUTE)})")

    # ---- V3: stored energies are the model's ------------------------------
    worst = 0.0
    where = Rs[0]
    for i, (rs, (a, _)) in zip(idxs, zip(Rs, both)):
        d = abs(float(mpf(a)) - E[i]) / max(abs(E[i]), 1.0)
        if d > worst:
            worst, where = d, rs
    record("V3_energies_match_model", worst <= TOL_F64_REL,
           f"max relative |E_json - E_fresh| = {worst:.3e} at R = "
           f"{s(mpf(where),12)} (tol {TOL_F64_REL:.1e})")

    # ---- V4: stored forces are -dE/dR of the model ------------------------
    fidx = idxs if args.full else idxs[::2]
    fRs = [doc["exact"]["R_grid_bohr"][i] for i in fidx]
    fres = pmap(_w_F, fRs)
    worst = 0.0
    where = fRs[0]
    for i, rs, fv in zip(fidx, fRs, fres):
        d = abs(float(mpf(fv)) - F[i]) / max(abs(F[i]), 1.0)
        if d > worst:
            worst, where = d, rs
    record("V4_forces_match_model", worst <= TOL_F64_REL,
           f"max relative |F_json - F_fresh| = {worst:.3e} at R = "
           f"{s(mpf(where),12)} over {len(fRs)} points (tol {TOL_F64_REL:.1e})")

    # ---- V5: asymptote is 2 x the in-model H atom -------------------------
    e_h = H.h_atom_energy()
    e_asym = 2 * e_h
    d = abs(float(e_asym) - doc["E_asymptote"]) / abs(doc["E_asymptote"])
    record("V5_asymptote_in_model", d <= TOL_F64_REL,
           f"2*E_H recomputed = {nstr(e_asym, 20)}; json = {doc['E_asymptote']}; "
           f"relative diff {d:.3e}")

    # ---- V6: exactly one minimum ------------------------------------------
    dEs = [-f for f in F]
    nmin = sum(1 for i in range(n - 1) if dEs[i] < 0 <= dEs[i + 1])
    nmax = sum(1 for i in range(n - 1) if dEs[i] > 0 >= dEs[i + 1])
    record("V6_exactly_one_minimum", nmin == 1 and nmax == 0,
           f"sign changes of dE/dR on the emitted grid: {nmin} minima, "
           f"{nmax} maxima")

    # ---- V7: approach from below ------------------------------------------
    Re = doc["R_e"]
    tail = [(r, e) for r, e in zip(R, E) if r >= Re]
    below = all(e < doc["E_asymptote"] for _, e in tail)
    mono = all(tail[i][1] < tail[i + 1][1] for i in range(len(tail) - 1))
    gap = doc["E_asymptote"] - E[-1]
    record("V7_asymptote_from_below", below and mono and gap > 0,
           f"{len(tail)} tail points all below asymptote: {below}; strictly "
           f"increasing: {mono}; E_asym - E(10) = {gap:.6e} > 0")

    # ---- V8: divergence as R -> 0 -----------------------------------------
    small = ["0.1", "0.03", "0.01", "0.003", "0.001"]
    sv = [mpf(x) for x in pmap(_w_E, small)]
    prods = [mpf(x) * v for x, v in zip(small, sv)]
    up = all(sv[i] > sv[i - 1] for i in range(1, len(sv)))
    one = abs(prods[-1] - 1) < mpf("1e-2")
    pos = all(p > 0 for p in prods)
    record("V8_diverges_at_zero", up and one and pos,
           f"E rises monotonically as R falls: {up}; R*E -> 1 "
           f"(R*E = {s(prods[-1],10)} at R=0.001): {one}; all R*E > 0: {pos}")

    # ---- V9: R_e is a stationary minimum of the exact curve ---------------
    Re_m = mpf(doc["exact"]["R_e"])
    d1 = diff(H.energy_route_a, Re_m)
    d2 = diff(H.energy_route_a, Re_m, 2)
    eps = mpf("1e-6")
    lo_e = H.energy_route_a(Re_m - eps)
    hi_e = H.energy_route_a(Re_m + eps)
    mid_e = H.energy_route_a(Re_m)
    bracketed = (lo_e > mid_e) and (hi_e > mid_e)
    stationary = abs(d1) < mpf("1e-30")
    convex = d2 > 0
    in_grid_bracket = R[0] < doc["R_e"] < R[-1]
    record("V9_R_e_is_stationary_minimum",
           stationary and convex and bracketed and in_grid_bracket,
           f"dE/dR(R_e) = {s(d1)} (< 1e-30: {stationary}); "
           f"d2E/dR2(R_e) = {s(d2)} > 0: {convex}; "
           f"E(R_e +/- 1e-6) both higher: {bracketed}")

    # ---- V10: D_e ----------------------------------------------------------
    De_fresh = e_asym - H.energy_route_a(Re_m)
    d = abs(float(De_fresh) - doc["D_e"]) / abs(doc["D_e"])
    record("V10_D_e_consistent", d <= TOL_F64_REL and De_fresh > 0,
           f"D_e = E_asym - E(R_e) recomputed = {nstr(De_fresh,20)}; "
           f"json = {doc['D_e']}; relative diff {d:.3e}; positive: "
           f"{De_fresh > 0}")

    # ---- V11: Hermite reproduction at knots --------------------------------
    # The VALUE at a knot must be bit-exact (the basis functions collapse to
    # 1*y0).  The SLOPE is reconstructed as (h*d0)/h, one f64 multiply-divide
    # round trip, so it is exact only to a few ulp -- that is the honest claim.
    kn = herm["knots_bohr"]
    vals = herm["values_hartree"]
    ders = herm["derivatives_hartree_per_bohr"]
    worst_v = 0.0
    worst_d = 0.0
    for i in range(n):
        j = min(i, n - 2)
        v, dv = hermite_f64(kn, vals, ders, kn[i])
        if i == j:
            worst_v = max(worst_v, abs(v - vals[i]))
            worst_d = max(worst_d,
                          abs(dv - ders[i]) / max(abs(ders[i]), 1.0))
    v, dv = hermite_f64(kn, vals, ders, kn[-1])
    worst_v = max(worst_v, abs(v - vals[-1]))
    worst_d = max(worst_d, abs(dv - ders[-1]) / max(abs(ders[-1]), 1.0))
    ok = (worst_v == 0.0) and (worst_d <= TOL_KNOT_SLOPE_REL)
    record("V11_hermite_exact_at_knots", ok,
           f"knot VALUES bit-exact over all {n} knots (max |dE| = {worst_v:.3e}, "
           f"must be 0.0); knot SLOPES to {worst_d:.3e} relative, one f64 "
           f"multiply-divide round trip (tol {TOL_KNOT_SLOPE_REL:.1e})")

    # ---- V12: Hermite bound holds against the exact model ------------------
    stated_E = mpf(repr(herm["max_abs_error_E_hartree"]))
    stated_F = mpf(repr(herm["max_abs_error_F_hartree_per_bohr"]))
    if args.full:
        ivs = list(range(n - 1))
    else:
        ivs = sorted(set(rng.sample(range(n - 1), min(120, n - 1))
                         + [0, 1, n - 3, n - 2]))
    tests = []
    for i in ivs:
        for t in (0.2113248654051871, 0.5, 0.7886751345948129,
                  rng.uniform(0.02, 0.98), rng.uniform(0.02, 0.98)):
            tests.append(kn[i] + t * (kn[i + 1] - kn[i]))
    tstr = [nstr(mpf(repr(x)), DPS) for x in tests]
    emit(f"  ... re-evaluating the model at {len(tstr)} interior points for "
         "the Hermite bound")
    tres = pmap(_w_EF, tstr)
    maxE = maxF = 0.0
    whereE = whereF = tests[0]
    for x, (ee, ff) in zip(tests, tres):
        hv, hd = hermite_f64(kn, vals, ders, x)
        de = abs(hv - float(mpf(ee)))
        df = abs(-hd - float(mpf(ff)))
        if de > maxE:
            maxE, whereE = de, x
        if df > maxF:
            maxF, whereF = df, x
    okE = mpf(repr(maxE)) <= stated_E * HERMITE_SLACK
    okF = mpf(repr(maxF)) <= stated_F * HERMITE_SLACK
    record("V12_hermite_bound_holds", okE and okF,
           f"observed max |dE| = {maxE:.6e} at R = {whereE:.6f} vs stated "
           f"{float(stated_E):.6e}; observed max |dF| = {maxF:.6e} at R = "
           f"{whereF:.6f} vs stated {float(stated_F):.6e} "
           f"(slack {float(HERMITE_SLACK)}x, {len(tests)} points)")

    # ---- V14: the Hermite slopes ARE the model's dE/dR ---------------------
    # (V11 only tests internal consistency and V4 tests a different array, so
    # without this a corrupted hermite.derivatives array survives both.)
    ident = all(ders[i] == -F[i] for i in range(n))
    worst = 0.0
    where = fRs[0]
    for i, rs, fv in zip(fidx, fRs, fres):
        # fres holds F = -dE/dR; the Hermite slope must be dE/dR = -F
        d = abs(ders[i] - (-float(mpf(fv)))) / max(abs(ders[i]), 1.0)
        if d > worst:
            worst, where = d, rs
    record("V14_hermite_slopes_match_model", ident and worst <= TOL_F64_REL,
           f"hermite.derivatives == -F_hartree_per_bohr for all {n} knots: "
           f"{ident}; max relative |slope - dE/dR_fresh| = {worst:.3e} at "
           f"R = {s(mpf(where),12)} over {len(fRs)} points "
           f"(tol {TOL_F64_REL:.1e})")

    # ---- V13: exact strings agree with the f64 arrays ----------------------
    worst = 0.0
    for i in idxs:
        for key, arr in (("R_grid_bohr", R), ("E_hartree", E),
                         ("F_hartree_per_bohr", F),
                         ("E2_hartree_per_bohr2", E2j)):
            a = float(mpf(doc["exact"][key][i]))
            b = arr[i]
            worst = max(worst, abs(a - b) / max(abs(b), 1.0))
    same_n = all(len(doc["exact"][k]) == n for k in
                 ("R_grid_bohr", "E_hartree", "F_hartree_per_bohr",
                  "hermite_derivatives_hartree_per_bohr"))
    record("V13_exact_strings_agree_with_f64", worst <= TOL_F64_REL and same_n,
           f"max relative |float(exact string) - f64 array| = {worst:.3e} "
           f"over {len(idxs)} indices; array lengths consistent: {same_n}")

    # ---- V15: stored E'' is the model's second derivative ------------------
    e2idx = idxs if args.full else idxs[::2]
    e2Rs = [doc["exact"]["R_grid_bohr"][i] for i in e2idx]
    e2res = pmap(_w_E2, e2Rs)
    worst = 0.0
    where = e2Rs[0]
    for i, rs, v in zip(e2idx, e2Rs, e2res):
        d = abs(float(mpf(v)) - E2j[i]) / max(abs(E2j[i]), 1.0)
        if d > worst:
            worst, where = d, rs
    d_re = abs(float(diff(H.energy_route_a, mpf(doc["exact"]["R_e"]), 2))
               - doc["E2_at_R_e"]) / abs(doc["E2_at_R_e"])
    record("V15_E2_matches_model",
           worst <= TOL_F64_REL and d_re <= TOL_F64_REL,
           f"max relative |E2_json - E2_fresh| = {worst:.3e} at R = "
           f"{s(mpf(where),12)} over {len(e2Rs)} points; E2_at_R_e relative "
           f"diff {d_re:.3e} (tol {TOL_F64_REL:.1e})")

    # ---- V16: envelope table structure and monotonicity --------------------
    env = doc["max_curvature_up_to_E"]
    rungs = env["rungs"]
    need_r = ["eps_above_min_hartree", "E_total_hartree", "R_in_bohr",
              "R_out_bohr", "range_flag", "max_abs_E2_hartree_per_bohr2",
              "argmax_R_bohr", "dt_per_sqrt_mu"]
    keys_ok = all(all(k in r for k in need_r) for r in rungs)
    et_inc = all(rungs[i]["E_total_hartree"] < rungs[i + 1]["E_total_hartree"]
                 for i in range(len(rungs) - 1))
    # the safety-critical property: the envelope may never fall as energy rises
    env_mono = all(rungs[i]["max_abs_E2_hartree_per_bohr2"]
                   <= rungs[i + 1]["max_abs_E2_hartree_per_bohr2"]
                   for i in range(len(rungs) - 1))
    rin_dec = all(rungs[i]["R_in_bohr"] >= rungs[i + 1]["R_in_bohr"]
                  for i in range(len(rungs) - 1))
    bnd = [r for r in rungs if r["range_flag"] == "bound"]
    rout_inc = all(bnd[i]["R_out_bohr"] < bnd[i + 1]["R_out_bohr"]
                   for i in range(len(bnd) - 1))
    flags_ok = all(r["range_flag"] in ("bound", "outer_beyond", "unbound")
                   for r in rungs)
    # a rung's own E_total must equal E(R_e) + eps
    eps_ok = all(abs((r["E_total_hartree"] - r["eps_above_min_hartree"])
                     - doc["E_at_R_e"]) < 1e-12 for r in rungs)
    starts_at_min = abs(rungs[0]["eps_above_min_hartree"]) < 1e-30
    enough_rungs = len(rungs) >= 20
    ok = (keys_ok and et_inc and env_mono and rin_dec and rout_inc
          and flags_ok and eps_ok and starts_at_min and enough_rungs)
    record("V16_envelope_wellformed", ok,
           f"{len(rungs)} rungs (>=20: {enough_rungs}); keys complete: "
           f"{keys_ok}; E_total strictly increasing: {et_inc}; max|E2| "
           f"NON-DECREASING: {env_mono}; R_in non-increasing: {rin_dec}; "
           f"R_out increasing over {len(bnd)} bound rungs: {rout_inc}; "
           f"flags valid: {flags_ok}; E_total == E(R_e)+eps: {eps_ok}; "
           f"first rung at the minimum: {starts_at_min}")

    # ---- V17: the turning points really are turning points -----------------
    rsel = sorted(set([0, 1, len(rungs) // 3, len(rungs) // 2,
                       2 * len(rungs) // 3, len(rungs) - 2, len(rungs) - 1]))
    tp_pts, tp_want = [], []
    for i in rsel:
        r = rungs[i]
        tp_pts.append(repr(r["R_in_bohr"]))
        tp_want.append(r["E_total_hartree"])
        if r["range_flag"] == "bound":
            tp_pts.append(repr(r["R_out_bohr"]))
            tp_want.append(r["E_total_hartree"])
    tp_got = pmap(_w_E, tp_pts)
    worst_tp = 0.0
    for got, want in zip(tp_got, tp_want):
        worst_tp = max(worst_tp, abs(float(mpf(got)) - want))
    # a turning point must also bracket: E just inside is lower
    inside_ok = True
    for i in rsel:
        r = rungs[i]
        if r["range_flag"] == "bound" and r["R_out_bohr"] - r["R_in_bohr"] > 1e-6:
            mid = mpf(repr(0.5 * (r["R_in_bohr"] + r["R_out_bohr"])))
            if not float(H.energy_route_a(mid)) <= r["E_total_hartree"]:
                inside_ok = False
    record("V17_envelope_turning_points_correct",
           worst_tp <= 1e-9 and inside_ok,
           f"max |E(R_turn) - E_total| = {worst_tp:.3e} over {len(tp_pts)} "
           f"turning points on {len(rsel)} sampled rungs (tol 1e-9); range "
           f"interior verified accessible: {inside_ok}")

    # ---- V18: the tabulated envelope does not UNDERSTATE the curvature -----
    # Understating is the unsafe direction: it yields a dt that is too large.
    # Scan the accessible range independently and demand the table covers it.
    NSCAN = 32
    scan_pts, scan_slice = [], []
    for i in rsel:
        r = rungs[i]
        lo_r = mpf(repr(r["R_in_bohr"]))
        hi_r = mpf(repr(r["R_out_bohr"])) if r["range_flag"] == "bound" \
            else mpf(repr(R[-1]))
        if hi_r <= lo_r:
            hi_r = lo_r
        st = len(scan_pts)
        for k in range(NSCAN):
            x = lo_r + (hi_r - lo_r) * mpf(k) / (NSCAN - 1)
            scan_pts.append(nstr(x, DPS))
        scan_slice.append((st, st + NSCAN))
    emit(f"  ... independently scanning {len(scan_pts)} points for the "
         "curvature envelope")
    scan_vals = pmap(_w_E2, scan_pts)
    worst_under = 0.0
    worst_over = 0.0
    for i, (a0, a1) in zip(rsel, scan_slice):
        smax = max(abs(mpf(scan_vals[k])) for k in range(a0, a1))
        tab = mpf(repr(rungs[i]["max_abs_E2_hartree_per_bohr2"]))
        if smax > tab:
            worst_under = max(worst_under, float((smax - tab) / smax))
        else:
            worst_over = max(worst_over, float((tab - smax) / max(smax, mpf(1))))
    record("V18_envelope_curvature_not_understated",
           worst_under <= 1e-12 and worst_over <= 1e-6,
           f"independent {NSCAN}-point scan of {len(rsel)} rungs: worst "
           f"UNDERSTATEMENT by the table = {worst_under:.3e} (must be ~0, this "
           f"is the unsafe direction); worst overstatement = {worst_over:.3e}")

    # ---- V19: the tabulated dt helper matches its own formula --------------
    worst_dt = 0.0
    for r in rungs:
        want = float(2 * mp.pi / (64 * mp.sqrt(
            mpf(repr(r["max_abs_E2_hartree_per_bohr2"])))))
        worst_dt = max(worst_dt, abs(want - r["dt_per_sqrt_mu"]) / abs(want))
    dt_dec = all(rungs[i]["dt_per_sqrt_mu"] >= rungs[i + 1]["dt_per_sqrt_mu"]
                 for i in range(len(rungs) - 1))
    record("V19_envelope_dt_consistent", worst_dt <= 1e-14 and dt_dec,
           f"max relative |dt_per_sqrt_mu - 2pi/(64 sqrt(max|E2|))| = "
           f"{worst_dt:.3e} over all {len(rungs)} rungs; dt non-increasing "
           f"with energy: {dt_dec}")

    # ---- exit contract -----------------------------------------------------
    emit()
    emit("-" * 78)
    ran = set(results)
    missing = [c for c in REQUIRED if c not in ran]
    failed = [c for c, ok in results.items() if not ok]
    extra = sorted(ran - set(REQUIRED))
    emit(f"required checks : {len(REQUIRED)}")
    emit(f"checks that ran : {len(ran)}" + (f"  (extra: {extra})" if extra else ""))
    if missing:
        emit(f"MISSING CHECKS  : {missing}")
        emit("REFUSE: a required check did not run. exit 2")
        return 2
    if failed:
        emit(f"FAILED CHECKS   : {failed}")
        emit("FAIL. exit 1")
        return 1
    emit("all required checks ran and PASSED. exit 0")
    return 0


if __name__ == "__main__":
    sys.exit(main())
