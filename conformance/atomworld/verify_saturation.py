#!/usr/bin/env python3
"""
verify_saturation.py -- standalone re-check of the SATURATION-1 referee JSONs.

It never trusts a number in h3_referee.json / h4_referee.json.  Every value it
compares is either recomputed fresh in this process from saturation_referee.py,
or re-derived from OTHER stored numbers by the defining identity.  The
recomputation runs at a DIFFERENT working precision (70 dps, not the referee's
80), so agreement at 1e-45 is also evidence that the 50 published digits do not
depend on the arithmetic headroom.

STAKED SPOT-CHECK RULE (frozen): the H3 subset is the six geometries at indices
0, 13, 26, 39, 52, 65 -- a fixed stride of 13 through the ordered staked list,
which lands one point in every block (A0 anchor, B near-linear, C boundary
shell, and three in D random).  The H4 subset is indices 0 and 3 (the r_e and
1.5 r_e tetrahedra).  --full re-checks every geometry in both files.

EXIT CONTRACT (house law: a missing check must refuse)
  0  every REQUIRED check ran and passed
  1  a required check ran and FAILED
  2  a required check did not run, or a contract is malformed / unreadable

Usage:
  python3 verify_saturation.py
  python3 verify_saturation.py --full
  python3 verify_saturation.py --h3 PATH --h4 PATH
"""

import argparse
import json
import os
import sys
from multiprocessing import Pool

from mpmath import mp, mpf, nstr

DPS_VERIFY = 70
mp.dps = DPS_VERIFY

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import saturation_referee as R  # noqa: E402
import h2_core as H2  # noqa: E402

TOL = mpf("1e-45")          # recomputation agreement
TOL_ID = mpf("1e-48")       # algebraic identity among stored strings
SPOT_H3 = [0, 13, 26, 39, 52, 65]
SPOT_H4 = [0, 3]

REQUIRED = [
    "V1_h3_contract_wellformed",
    "V2_h3_sides_in_domain",
    "V3_h3_spot_recomputed",
    "V4_h3_dE3_identity_from_stored",
    "V5_E_H_atom_matches_h2_core",
    "V6_h3_permutation_symmetry",
    "V7_h3_all_doublet",
    "V8_h4_contract_wellformed",
    "V9_h4_spot_recomputed",
    "V10_h4_dE4_identity_from_stored",
    "V11_shell_probe_recomputed",
]


def _init(dps):
    mp.dps = dps


def _w_h3(sides):
    d = R.de3_from_sides(sides[0], sides[1], sides[2], detail=True)
    return nstr(d["E"], 60), nstr(d["dE3"], 60), [nstr(x, 60) for x in d["V2"]]


def _w_h3_perm(sides):
    return nstr(R.de3_from_sides(sides[0], sides[1], sides[2]), 60)


def _w_h4(pos):
    p = [tuple(mpf(c) for c in q) for q in pos]
    d = R.de4_from_positions(p, detail=True, spin_resolve=True)
    return nstr(d["E"], 60), nstr(d["dE4"], 60), nstr(d["E_S0"], 60)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--h3", default=os.path.join(HERE, "h3_referee.json"))
    ap.add_argument("--h4", default=os.path.join(HERE, "h4_referee.json"))
    ap.add_argument("--full", action="store_true")
    ap.add_argument("--jobs", type=int, default=min(16, os.cpu_count() or 1))
    args = ap.parse_args()

    results = {}
    failures = []

    def record(name, ok, msg):
        results[name] = ok
        print("  %-32s %s   %s" % (name, "PASS" if ok else "FAIL", msg))
        if not ok:
            failures.append(name)

    try:
        h3 = json.load(open(args.h3))
        h4 = json.load(open(args.h4))
    except Exception as exc:                       # unreadable contract -> refuse
        print("REFUSED: cannot read a referee contract: %s" % exc)
        return 2

    print("verify_saturation  (recomputing at %d dps against %d-digit strings)"
          % (DPS_VERIFY, h3.get("precision_digits", -1)))
    print()

    # ---- V1 --------------------------------------------------------------
    need = ("model", "precision_digits", "working_precision_dps", "staking_seed",
            "staking_rule", "E_H_atom", "n_geometries", "geometries",
            "boundary_shell_probe")
    ok = all(k in h3 for k in need)
    ok = ok and h3["n_geometries"] == len(h3["geometries"]) >= 64
    ok = ok and h3["precision_digits"] == 50
    for g in h3["geometries"]:
        for k in ("sides_bohr", "E_H3", "dE3", "V2", "S2_ground", "ndet"):
            if k not in g:
                ok = False
        if g.get("ndet") != 9:
            ok = False
        try:
            mpf(g["E_H3"]); mpf(g["dE3"]); [mpf(x) for x in g["V2"]]
        except Exception:
            ok = False
    record("V1_h3_contract_wellformed", ok,
           "%d geometries, %d digits, seed %s, 9 determinants each"
           % (h3.get("n_geometries", -1), h3.get("precision_digits", -1),
              h3.get("staking_seed")))

    # ---- V2 --------------------------------------------------------------
    lo, hi = mpf("0.9"), mpf("7.0")
    bad = []
    for g in h3["geometries"]:
        s = sorted(mpf(x) for x in g["sides_bohr"])
        if not (lo <= s[0] and s[2] <= hi):
            bad.append((g["i"], "outside domain"))
        if s[0] + s[1] < s[2]:
            bad.append((g["i"], "triangle inequality"))
    record("V2_h3_sides_in_domain", not bad,
           "all %d side triples inside [0.9, 7.0] and realisable"
           % len(h3["geometries"]) if not bad else str(bad[:4]))

    # ---- V3 --------------------------------------------------------------
    idx = list(range(len(h3["geometries"]))) if args.full else SPOT_H3
    jobs = [tuple(mpf(x) for x in h3["geometries"][i]["sides_bohr"]) for i in idx]
    with Pool(min(args.jobs, len(jobs)), initializer=_init,
              initargs=(DPS_VERIFY,)) as pool:
        got = pool.map(_w_h3, jobs, chunksize=1)
    worst = mpf(0)
    worst_at = None
    for i, (E, d3, v) in zip(idx, got):
        g = h3["geometries"][i]
        for a, b in ((E, g["E_H3"]), (d3, g["dE3"])) + tuple(zip(v, g["V2"])):
            dd = abs(mpf(a) - mpf(b))
            if dd > worst:
                worst, worst_at = dd, i
    record("V3_h3_spot_recomputed", worst <= TOL,
           "%d geometries recomputed, worst |delta| = %s at index %s (tol %s)"
           % (len(idx), nstr(worst, 4), worst_at, nstr(TOL, 2)))

    # ---- V4: the defining identity, from stored strings only -------------
    e1 = mpf(h3["E_H_atom"])
    worst = mpf(0)
    worst_at = None
    for g in h3["geometries"]:
        lhs = mpf(g["dE3"])
        rhs = mpf(g["E_H3"]) - sum(mpf(x) for x in g["V2"]) - 3 * e1
        dd = abs(lhs - rhs)
        if dd > worst:
            worst, worst_at = dd, g["i"]
    record("V4_h3_dE3_identity_from_stored", worst <= TOL_ID,
           "dE3 == E(H3) - sum V2 - 3 E(H) on all %d rows, worst = %s at %s"
           % (len(h3["geometries"]), nstr(worst, 4), worst_at))

    # ---- V5 --------------------------------------------------------------
    d = abs(e1 - H2.h_atom_energy())
    record("V5_E_H_atom_matches_h2_core", d < TOL,
           "|stored E(H) - h2_core| = %s" % nstr(d, 4))

    # ---- V6: dE3 is totally symmetric in its three sides ------------------
    g = h3["geometries"][SPOT_H3[3]]
    s = [mpf(x) for x in g["sides_bohr"]]
    perms = [(s[a], s[b], s[c]) for (a, b, c) in
             ((0, 1, 2), (0, 2, 1), (1, 0, 2), (1, 2, 0), (2, 0, 1), (2, 1, 0))]
    with Pool(6, initializer=_init, initargs=(DPS_VERIFY,)) as pool:
        vals = [mpf(x) for x in pool.map(_w_h3_perm, perms, chunksize=1)]
    spread = max(vals) - min(vals)
    d = abs(vals[0] - mpf(g["dE3"]))
    record("V6_h3_permutation_symmetry", spread <= TOL and d <= TOL,
           "index %d: spread over 6 permutations = %s, vs stored = %s"
           % (g["i"], nstr(spread, 4), nstr(d, 4)))

    # ---- V7 --------------------------------------------------------------
    bad = [g["i"] for g in h3["geometries"]
           if abs(mpf(g["S2_ground"]) - mpf(3) / 4) > mpf("1e-18")]
    record("V7_h3_all_doublet", not bad,
           "<S^2> = 3/4 on all %d geometries" % len(h3["geometries"])
           if not bad else "not a doublet at %s" % bad[:6])

    # ---- V8 --------------------------------------------------------------
    need = ("model", "precision_digits", "E_H_atom", "geometries",
            "two_dimers_reference_2xE2_at_r_e")
    ok = all(k in h4 for k in need) and len(h4["geometries"]) == 6
    for g in h4["geometries"]:
        for k in ("E_H4", "dE4", "E_H4_S0", "dE4_S0", "V2_sum", "dE3_sum",
                  "dE3_per_triple", "positions_bohr", "ndet"):
            if k not in g:
                ok = False
        if g.get("ndet") != 36:
            ok = False
    record("V8_h4_contract_wellformed", ok,
           "%d geometries, 36 determinants each" % len(h4.get("geometries", [])))

    # ---- V9 --------------------------------------------------------------
    idx4 = list(range(len(h4["geometries"]))) if args.full else SPOT_H4
    jobs = [h4["geometries"][i]["positions_bohr"] for i in idx4]
    with Pool(min(args.jobs, len(jobs)), initializer=_init,
              initargs=(DPS_VERIFY,)) as pool:
        got = pool.map(_w_h4, jobs, chunksize=1)
    worst = mpf(0)
    worst_at = None
    for i, (E, d4, es0) in zip(idx4, got):
        g = h4["geometries"][i]
        for a, b in ((E, g["E_H4"]), (d4, g["dE4"]), (es0, g["E_H4_S0"])):
            dd = abs(mpf(a) - mpf(b))
            if dd > worst:
                worst, worst_at = dd, g["name"]
    record("V9_h4_spot_recomputed", worst <= TOL,
           "%d geometries recomputed, worst |delta| = %s at %s"
           % (len(idx4), nstr(worst, 4), worst_at))

    # ---- V10 -------------------------------------------------------------
    e1 = mpf(h4["E_H_atom"])
    worst = mpf(0)
    worst_at = None
    for g in h4["geometries"]:
        trip = sum(mpf(x) for x in g["dE3_per_triple"])
        checks = [
            (mpf(g["dE3_sum"]), trip),
            (mpf(g["dE4"]),
             mpf(g["E_H4"]) - 4 * e1 - mpf(g["V2_sum"]) - mpf(g["dE3_sum"])),
            (mpf(g["dE4_S0"]),
             mpf(g["E_H4_S0"]) - 4 * e1 - mpf(g["V2_sum"]) - mpf(g["dE3_sum"])),
        ]
        for lhs, rhs in checks:
            dd = abs(lhs - rhs)
            if dd > worst:
                worst, worst_at = dd, g["name"]
    record("V10_h4_dE4_identity_from_stored", worst <= TOL_ID,
           "dE4 == E(H4) - 4 E(H) - sum V2 - sum dE3 on all 6 rows and both "
           "spin readings, worst = %s at %s" % (nstr(worst, 4), worst_at))

    # ---- V11 -------------------------------------------------------------
    probe = h3["boundary_shell_probe"]["geometries"]
    jobs = [tuple(mpf(x) for x in p["sides_bohr"]) for p in probe]
    with Pool(min(args.jobs, len(jobs)), initializer=_init,
              initargs=(DPS_VERIFY,)) as pool:
        got = pool.map(_w_h3, jobs, chunksize=1)
    worst = mpf(0)
    for p, (E, d3, v) in zip(probe, got):
        worst = max(worst, abs(mpf(E) - mpf(p["E_H3"])),
                    abs(mpf(d3) - mpf(p["dE3"])))
    stated = mpf(h3["boundary_shell_probe"]["shell_max_abs_dE3_over_probe"])
    recomputed_max = max(abs(mpf(d3)) for (_, d3, _) in got)
    ok = worst <= TOL and abs(stated - recomputed_max) <= TOL
    record("V11_shell_probe_recomputed", ok,
           "%d shell points, worst |delta| = %s ; stated shell max %s"
           % (len(probe), nstr(worst, 4), nstr(stated, 10)))

    print()
    missing = [k for k in REQUIRED if k not in results]
    if missing:
        print("REFUSED: required checks did not run: %s" % ", ".join(missing))
        return 2
    if failures:
        print("VERIFY FAILED: %s" % ", ".join(failures))
        return 1
    print("VERIFY PASSED: %d/%d required checks" % (len(REQUIRED), len(REQUIRED)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
