#!/usr/bin/env python3
"""size_cost.py — SELECTOR-6 cost model, measured on worst-case shapes.

SIZING ONLY.  This computes the SHAPE of the work (family size, gauge-orbit
count, per-rung view grouping) and never calls `separates`, so no verdict, no
selected set and no label is produced for any group.  Its output is the input to
B1's budget declaration, per the lead's requirement that the budget be DERIVED
from the aggregation rather than asserted.

Cost model.  At rung k the criterion groups candidates by the single-step reading
`view o p`; only candidates sharing a reading can fail to separate, so only those
pairs pay for a rho-BFS.  With the design's aggregation -- home candidates range
over gauge-orbit representatives, partners range over the whole family -- the
per-group BFS-call count is

    calls(G) = sum over rungs k of  sum over view-groups g of
               (#orbit reps in g) * (|g| - 1)

and each call is bounded by BUDGET rho-nodes.
"""
import os
import sys
import time
import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import s4core as S4            # the frozen criterion, import-safe (make_s4core.py)
import landscape_sweep as ls


def family_and_orbits(W):
    """Full dressed family F(G) = {GAUGE[g] . step^d}, deduplicated, with the
    gauge-orbit partition.  Returns (keys, orbit_of, orbit_sizes, reps)."""
    n, N = W["n"], W["N"]
    divs = S4.divisors(W["ordstep"])
    powers = {}
    x = W["idp"].copy()
    for d in range(1, max(divs) + 1):
        x = W["step"][x]
        if d in divs:
            powers[d] = x.astype(np.int32)
    index = {}
    members = []                       # (d, g) label per distinct permutation
    for d in divs:
        for g in range(n):
            key = W["GAUGE"][g][powers[d]].astype(np.int32).tobytes()
            if key not in index:
                index[key] = len(members)
                members.append((d, g))
    # gauge orbits: conjugating GAUGE[g].step^d by GAUGE[x] gives GAUGE[xgx^-1].step^d
    MUL, INV = W["MUL"], W["INV"]
    orbit_of = [-1] * len(members)
    reps, sizes = [], []
    for i, (d, g) in enumerate(members):
        if orbit_of[i] >= 0:
            continue
        orb = set()
        for x_ in range(n):
            gg = int(MUL[MUL[x_, g], INV[x_]])
            key = W["GAUGE"][gg][powers[d]].astype(np.int32).tobytes()
            orb.add(index[key])
        lab = len(reps)
        for j in orb:
            orbit_of[j] = lab
        reps.append(i)
        sizes.append(len(orb))
    return members, powers, index, orbit_of, reps, sizes


def measure(name, MUL, INV, cap_family=None):
    t0 = time.time()
    W = S4.build_world(name, np.asarray(MUL), np.asarray(INV))
    W["MUL"], W["INV"] = np.asarray(MUL), np.asarray(INV)
    n, N = W["n"], W["N"]
    members, powers, index, orbit_of, reps, sizes = family_and_orbits(W)
    F = len(members)
    if cap_family is not None and F > cap_family:
        return dict(name=name, n=n, N=N, ordstep=W["ordstep"], F=F,
                    orbits=len(reps), calls=None, skipped=True,
                    secs=time.time() - t0)
    is_rep = np.zeros(F, bool)
    for i in reps:
        is_rep[i] = True

    total_calls = 0
    per_rung = []
    for k in range(5):
        view = W["V"][k]
        if int(view.max()) == 0:
            per_rung.append((1, F, 0, 0, True))   # analytic rung: no BFS at all
            continue
        groups = {}
        for i, (d, g) in enumerate(members):
            perm = W["GAUGE"][g][powers[d]]
            groups.setdefault(view[perm].tobytes(), []).append(i)
        # Which home candidates take the CHEAP cached path?  run_rung uses the
        # divisor-pair cache exactly when the view is gauge-invariant and the
        # home candidate's act vocabulary is EMPTY; otherwise it pays a rho-BFS.
        ginv = S4.gauge_invariant_view(view, W["GAUGE"])
        cheap = 0
        bfs = 0
        for mem in groups.values():
            if len(mem) == 1:
                continue
            for i in mem:
                if not is_rep[i]:
                    continue
                d, g = members[i]
                perm = W["GAUGE"][g][powers[d]].astype(np.int32)
                cyc, _ = S4.knobs_partial_sections(perm, view, N)
                if ginv and not cyc:
                    cheap += len(mem) - 1
                else:
                    bfs += len(mem) - 1
        total_calls += bfs
        per_rung.append((len(groups), max(len(v) for v in groups.values()),
                         bfs, cheap, ginv))
    return dict(name=name, n=n, N=N, ordstep=W["ordstep"], F=F, orbits=len(reps),
                calls=total_calls, per_rung=per_rung, skipped=False,
                secs=time.time() - t0)


def main():
    z2 = ls.build_cyclic(2)
    cases = [
        ("Z_32", ls.build_cyclic(32)), ("Z_48", ls.build_cyclic(48)),
        ("Z_60", ls.build_cyclic(60)), ("Z_63", ls.build_cyclic(63)),
        ("Z_2^5", ls.build_direct_product(ls.build_direct_product(
            ls.build_direct_product(ls.build_direct_product(z2, z2), z2), z2), z2)),
        ("D_32", ls.build_dihedral(16)), ("D_48", ls.build_dihedral(24)),
        ("D_60", ls.build_dihedral(30)), ("D_62", ls.build_dihedral(31)),
        ("Dic_8(Q32)", ls.build_dicyclic(8)), ("Dic_12", ls.build_dicyclic(12)),
        ("Dic_15", ls.build_dicyclic(15)),
        ("SD_32", ls.build_semidihedral(5)), ("M_32", ls.build_modular_group(5)),
        ("Delta(48)", ls.build_delta_3n2(4)), ("GL(2,3)", ls.build_gl2_gf3()),
        ("F_39", ls.build_frobenius(13, 3)), ("F_55", ls.build_frobenius(11, 5)),
        ("F_57", ls.build_frobenius(19, 3)),
        ("Z_5xA_4", ls.build_direct_product(ls.build_cyclic(5), ls.build_alternating(4))),
        ("Z_2x2T", ls.build_direct_product(z2, ls.build_binary_tetrahedral())),
        ("S_3xZ_10", ls.build_direct_product(ls.build_dihedral(3), ls.build_cyclic(10))),
        ("A_5", ls.build_alternating(5)),
    ]
    print(f"{'group':12s}{'|G|':>5s}{'N':>7s}{'ordstep':>8s}{'|F|':>6s}"
          f"{'orb':>5s}{'realBFS':>9s}{'secs':>7s}  per rung: groups/maxgroup/bfs/cheap")
    rows = []
    for nm, g in cases:
        if g is None:
            continue
        r = measure(nm, g.MUL, g.INV)
        rows.append(r)
        pr = "" if r["skipped"] else " ".join(
            f"A{k}:g{a}/m{b}/bfs{c}/cheap{d}{'' if e else '/NOTGINV'}"
            for k, (a, b, c, d, e) in enumerate(r["per_rung"]))
        print(f"{r['name']:12s}{r['n']:5d}{r['N']:7d}{r['ordstep']:10d}{r['F']:7d}"
              f"{r['orbits']:8d}{str(r['calls']):>11s}{r['secs']:8.1f}   {pr}", flush=True)
    ok = [r for r in rows if not r["skipped"]]
    print()
    print(f"worst |F|         : {max(r['F'] for r in ok)} "
          f"({max(ok, key=lambda r: r['F'])['name']})")
    print(f"worst orbit count : {max(r['orbits'] for r in ok)} "
          f"({max(ok, key=lambda r: r['orbits'])['name']})")
    print(f"worst REAL BFS    : {max(r['calls'] for r in ok)} "
          f"({max(ok, key=lambda r: r['calls'])['name']})")
    print(f"worst cheap-path  : {max(sum(x[3] for x in r['per_rung']) for r in ok)}")
    print(f"worst sizing secs : {max(r['secs'] for r in ok):.1f} "
          f"({max(ok, key=lambda r: r['secs'])['name']})")


if __name__ == "__main__":
    main()
