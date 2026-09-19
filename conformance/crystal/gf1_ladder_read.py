#!/usr/bin/env python3
"""GF1's ladder read (GF1_PREREG.md §3, S1/S2/S4 as re-sized by GF1_AMENDMENT_1.md).
Assembles every admitted point, then:
  S1 - at each x, the successive differences M2(N) - M2(N-Δ) per unit Δ are constant to 10 %
       from N = 16 up; the limit c(x) is the last difference per site. Cannot fail, only refuse.
  S2 - M2/N < 0.05 at x = 0.25 (read at the largest admitted N). Kill: >= 0.05.
  S4 - M2_box(10) < 10 at every x (read at the largest admitted N >= 12; the spread over N shown).
  S3 - retired by Amendment 1's C1: M2_nl = M2 by theorem; branch (b) entered on that finding.
"""
import json, glob, os, sys
D = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), "gf1_ladder")
XS = [0.015625, 0.0625, 0.25, 1.0, 4.0, 16.0]; NS = [8, 12, 16, 24, 32, 48]
pts = {}
for x in XS:
    for n in NS:
        tag = f"x{x:g}_N{n}"
        adm = os.path.join(D, tag + ".admitted"); ref = os.path.join(D, tag + ".refused")
        if os.path.exists(adm):
            chi = int(open(adm).read().strip()); d = json.load(open(os.path.join(D, f"{tag}_chi{chi}.json")))
            pts[(x, n)] = d
        elif os.path.exists(ref):
            pts[(x, n)] = None
print("# GF1 ladder read")
print(f"# {sum(1 for v in pts.values() if v)} admitted, {sum(1 for v in pts.values() if v is None)} refused, {len(XS)*len(NS)-len(pts)} not yet read")
print(f"{'x':>6} {'N':>3} {'chi':>3} {'var/site':>10} {'M2':>14} {'M2/N':>9} {'M2_box':>10} {'box':>3} {'s':>6}")
for (x, n), d in sorted(pts.items()):
    if d is None: print(f"{x:>6g} {n:>3}  REFUSED at chi = 11"); continue
    print(f"{x:>6g} {n:>3} {d['chi']:>3} {d['variance_per_site']:>10.2e} {d['m2']:>14.9f} {d['m2_per_site']:>9.5f} {d['m2_box']:>10.6f} {d['box']:>3} {d['seconds_m2']:>6.1f}")
verdict = {}
print("\n## S1 - the density's convergence at each coupling")
for x in XS:
    row = [(n, pts[(x, n)]['m2']) for n in NS if pts.get((x, n))]
    if len(row) < 3: print(f"x={x:g}: {len(row)} points, not readable yet"); continue
    diffs = [((row[i][1] - row[i-1][1]) / (row[i][0] - row[i-1][0]), row[i][0]) for i in range(1, len(row))]
    from16 = [d for d in diffs if d[1] >= 16]
    spread = (max(d[0] for d in from16) - min(d[0] for d in from16)) / abs(from16[-1][0]) if from16 and from16[-1][0] != 0 else float('nan')
    ok = len(from16) >= 2 and spread <= 0.10
    verdict[('S1', x)] = ok
    print(f"x={x:g}: per-site differences " + ", ".join(f"{d[0]:.5f}@N{d[1]}" for d in diffs) + f"; spread from N=16 up {100*spread:.1f} % -> {'CONVERGED, c(x) = %.5f' % from16[-1][0] if ok else 'REFUSED (not constant to 10 %)'}")
print("\n## S2 - the density at strong coupling")
for x in [0.25, 0.0625, 0.015625]:
    row = [(n, pts[(x, n)]) for n in NS if pts.get((x, n))]
    if not row: continue
    n, d = row[-1]; ok = d['m2_per_site'] < 0.05
    if x == 0.25: verdict['S2'] = ok
    label = "the prereg's stake" if x == 0.25 else ("branch (c)'s extension" if x == 0.0625 else "a LABELLED EXTRA, not a gate")
    print(f"x={x:g}, N={n}: M2/N = {d['m2_per_site']:.5f} vs 0.05 -> {'under' if ok else 'OVER'} ({label})")
if 'S2' in verdict and not verdict['S2']: print("S2 at x=0.25: KILL as staked; branch (c): the ladder is extended to x = 0.0625 before any verdict")
print("\n## S4 - the price of a ten-site box at every coupling")
for x in XS:
    row = [(n, pts[(x, n)]) for n in NS if n >= 12 and pts.get((x, n))]
    if not row: print(f"x={x:g}: no N >= 12 point yet"); continue
    vals = [d['m2_box'] for _, d in row]; n, d = row[-1]; ok = d['m2_box'] < 10; verdict[('S4', x)] = ok
    print(f"x={x:g}: M2_box(10) at N={n} = {d['m2_box']:.5f} (over N: {min(vals):.4f}..{max(vals):.4f}), price bound 2^M2 = {2**d['m2_box']:.1f} -> {'MET' if ok else 'KILL: a ten-site box costs more than 2^10'}")
print("\n## S3 - retired: M2_nl = M2 by GF1_AMENDMENT_1 C1 (parity), branch (b) entered on that finding")
s1 = [v for k, v in verdict.items() if k[0] == 'S1']; s4 = [v for k, v in verdict.items() if k[0] == 'S4']
if 'S2' in verdict and len(s1) == 4 and len(s4) == 4:
    print("\n## Branch:", "(d) S1 refuses at some coupling - VOID, volumes extended once" if not all(s1) else ("(c) S2 fails - ladder extended to x = 0.0625" if not verdict['S2'] else ("S4 kill at some coupling - Fold III's 'priced exactly' weakens to 'priced, and here is the price'" if not all(s4) else "(b) by S3's vacuity, with S1, S2, S4 all met: c(x) banked, the ten-site price banked, the area-law clause without a quantity")))
