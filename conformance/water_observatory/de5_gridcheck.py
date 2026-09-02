#!/usr/bin/env python3
"""Water's check: do the live-vs-served dE3 disagreements sort by DISTANCE FROM THE
NEAREST GRID NODE, or by the scf_converged flag?

Grid mapping copied from water.rs `eval_grid` / `eval_sorted`, not re-derived:
    x, y = sorted O-H sides;  u = clamp((x^2+y^2-z^2)/(2xy), -1, 1);  c = sqrt(max(1-u,0))
    tx = tau_of_r(max(x,R_LO)) * (NR-1)      tau_of_r(r) = ln(1+(r-R_LO)k)/a, k=(e^a-1)/(R_HI-R_LO)
    ty = tau_of_r(max(y,R_LO)) * (NR-1)
    tc = (max(c,C_LO)-C_LO)/(C_HI-C_LO) * (NU-1)
Distance to the nearest node on an axis is |t - round(t)| in [0, 0.5]; 0 = on a node,
0.5 = cell centre, which is where a tensor-product interpolant's error peaks.
"""
import math, re, sys, statistics as st

R_LO, R_HI, A = 0.7, 15.0, 3.0
C_LO, C_HI = 0.05, math.sqrt(2.0)
NR, NU = 65, 49
K = (math.exp(A) - 1.0) / (R_HI - R_LO)

def tau_of_r(r): return math.log(1.0 + (r - R_LO) * K) / A

rows = []
pat = re.compile(r"^\s*\((\d),(\d),(\d)\)\s+(\d+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+(true|false)\s+([-+][\d.e+-]+)\s+([-+][\d.e+-]+)\s+([-+][\d.e+-]+)\s*$")
for line in open(sys.argv[1]):
    m = pat.match(line)
    if not m:
        continue
    r1, r2, z = float(m.group(5)), float(m.group(6)), float(m.group(7))
    scf = m.group(8) == "true"
    diff = abs(float(m.group(11)))
    x, y = (r1, r2) if r1 <= r2 else (r2, r1)
    u = max(-1.0, min(1.0, (x*x + y*y - z*z) / (2.0*x*y)))
    c = math.sqrt(max(1.0 - u, 0.0))
    tx = min(max(tau_of_r(max(x, R_LO)) * (NR-1), 0.0), NR-1.0)
    ty = min(max(tau_of_r(max(y, R_LO)) * (NR-1), 0.0), NR-1.0)
    tc = min(max((max(c, C_LO) - C_LO)/(C_HI - C_LO) * (NU-1), 0.0), NU-1.0)
    d = [abs(t - round(t)) for t in (tx, ty, tc)]
    rows.append(dict(diff=diff, scf=scf, D=max(d), dx=d[0], dy=d[1], dc=d[2],
                     tc=tc, edge=(NU-1) - tc, x=x, y=y, z=z))

def summ(rs):
    if not rs: return "        --            --      --"
    v = sorted(r["diff"] for r in rs)
    return f"{len(v):>4}   {st.median(v):>10.3e}   {v[-1]:>10.3e}"

print(f"n = {len(rows)} (O,H,H) triples\n")

print("=== BY DISTANCE FROM THE NEAREST GRID NODE (D = max over the three axes) ===")
print("  D bin            n       median|diff|     max|diff|")
edges = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5001]
for lo, hi in zip(edges, edges[1:]):
    sel = [r for r in rows if lo <= r["D"] < hi]
    print(f"  [{lo:.1f},{min(hi,0.5):.1f})   {summ(sel)}")

print("\n=== BY THE scf_converged FLAG (the original, convicted comparison) ===")
print("  flag             n       median|diff|     max|diff|")
for f, name in ((True, "converged "), (False, "NOT conv. ")):
    print(f"  {name}    {summ([r for r in rows if r['scf'] is f])}")

print("\n=== THE TWO-WAY TABLE: is the flag flat WITHIN a grid-distance bin? ===")
print("  D bin        flag             n       median|diff|     max|diff|")
for lo, hi in zip(edges, edges[1:]):
    for f, name in ((True, "converged "), (False, "NOT conv. ")):
        sel = [r for r in rows if lo <= r["D"] < hi and r["scf"] is f]
        print(f"  [{lo:.1f},{min(hi,0.5):.1f})   {name}   {summ(sel)}")

print("\n=== THE WORST TEN DISAGREEMENTS, and where they sit ===")
print("   |diff|      scf     D     d_x    d_y    d_c   tc (of 48)  cells from c-edge   x      y      z")
for r in sorted(rows, key=lambda r: -r["diff"])[:10]:
    print(f"  {r['diff']:.3e}  {str(r['scf']):>5}  {r['D']:.3f}  {r['dx']:.3f}  {r['dy']:.3f}  "
          f"{r['dc']:.3f}   {r['tc']:6.3f}        {r['edge']:6.3f}      {r['x']:.3f}  {r['y']:.3f}  {r['z']:.3f}")

# Spearman rank correlation, no scipy.
def rank(v):
    order = sorted(range(len(v)), key=lambda i: v[i])
    rk = [0.0]*len(v); i = 0
    while i < len(order):
        j = i
        while j+1 < len(order) and v[order[j+1]] == v[order[i]]: j += 1
        avg = (i + j)/2.0 + 1.0
        for k in range(i, j+1): rk[order[k]] = avg
        i = j+1
    return rk
def spearman(a, b):
    ra, rb = rank(a), rank(b)
    n = len(a); ma, mb = st.mean(ra), st.mean(rb)
    num = sum((x-ma)*(y-mb) for x, y in zip(ra, rb))
    den = math.sqrt(sum((x-ma)**2 for x in ra) * sum((y-mb)**2 for y in rb))
    return num/den if den else float("nan")

d = [r["diff"] for r in rows]
print("\n=== RANK CORRELATION with |diff| (Spearman; n = %d) ===" % len(rows))
for name, key in (("grid distance D    ", "D"), ("d_c (angle axis)   ", "dc"),
                  ("d_x                ", "dx"), ("d_y                ", "dy"),
                  ("cells from c-edge  ", "edge")):
    print(f"  {name}  rho = {spearman(d, [r[key] for r in rows]):+.4f}")
print(f"  scf flag (1=NOT conv) rho = {spearman(d, [0.0 if r['scf'] else 1.0 for r in rows]):+.4f}")

# ---- The decisive cut the worst-ten table pointed at, not the one we set out to test.
# water.rs on the c axis: "The collinear H-O-H end, theta = 180 degrees, u = -1. A hard
# edge of the geometry: u < -1 violates the triangle inequality, so there is no node
# beyond it and THE LAST INTERVAL USES THE ONE-SIDED NODE SLOPE."  So the prediction is
# not cell-centre error but BOUNDARY error, and it is separable: last c-cell vs interior.
LAST = (NU - 1) - 1.0   # tc > 47 is inside the final interval
print("\n=== THE CUT THAT ACTUALLY SEPARATES: the final c-axis interval ===")
print("  (water.rs: past the last c node there is no node, so that interval is one-sided)")
print("  region                          n       median|diff|     max|diff|")
inlast = [r for r in rows if r["tc"] > LAST]
inter  = [r for r in rows if r["tc"] <= LAST]
print(f"  final c interval (tc>47)   {summ(inlast)}")
print(f"  interior                   {summ(inter)}")
print("\n  and the flag WITHIN the final interval, which is where the big numbers live:")
for f, name in ((True, "converged "), (False, "NOT conv. ")):
    print(f"    {name}  {summ([r for r in inlast if r['scf'] is f])}")
print("  the flag WITHIN the interior:")
for f, name in ((True, "converged "), (False, "NOT conv. ")):
    print(f"    {name}  {summ([r for r in inter if r['scf'] is f])}")
tot = sum(1 for r in rows if r["diff"] >= 1e-4)
inl = sum(1 for r in inlast if r["diff"] >= 1e-4)
print(f"\n  disagreements at or above 1e-4 Ha: {inl} of {tot} are in the final c interval")
print(f"  the final interval is {len(inlast)} of {len(rows)} triples ({100.0*len(inlast)/len(rows):.0f}% of the sample)")

# ---- SATURATION-3 already located this. Its RESULTS name a near-collinear ELECTRONIC
# STATE CROSSING ("Seam 1") at c ~= 1.4128, theta ~= 174.9 deg, whose bending slope turns
# by ~400x across ~1e-4 of the c axis against a grid spacing of 0.0284 -- and it explicitly
# REFUTED the one-sided-slope explanation on planted data ("the one-sided slope costs
# nothing on smooth data. It is not the mechanism."). So the test is not "is it the last
# cell" but "is it that CORNER".
SEAM_C = 1.4128
print("\n=== AGAINST SATURATION-3's SEAM 1 (c ~= 1.4128, theta ~= 174.9 deg) ===")
print("   |diff|      c        theta(deg)   c - c_seam    in last cell")
for r in sorted(rows, key=lambda r: -r["diff"])[:8]:
    x, y, z = r["x"], r["y"], r["z"]
    u = max(-1.0, min(1.0, (x*x + y*y - z*z)/(2.0*x*y)))
    c = math.sqrt(max(1.0-u, 0.0))
    th = math.degrees(math.acos(u))
    print(f"  {r['diff']:.3e}  {c:.4f}   {th:8.3f}     {c-SEAM_C:+.4f}      {r['tc']>LAST}")
near = [r for r in rows if r["tc"] > LAST]
if near:
    cs = []
    for r in near:
        x, y, z = r["x"], r["y"], r["z"]
        u = max(-1.0, min(1.0, (x*x+y*y-z*z)/(2.0*x*y)))
        cs.append(math.sqrt(max(1.0-u, 0.0)))
    print(f"\n  the {len(near)} final-interval triples span c = {min(cs):.4f} .. {max(cs):.4f}")
    print(f"  SATURATION-3's corner sits at c = {SEAM_C}, grid spacing there {(C_HI-C_LO)/(NU-1):.4f}")

# ---- The base rate the cut needs (M-BASE-RATE-OMITTED): a region that holds all the big
# disagreements proves nothing until the region's own share of the sample is stated.
print("\n=== BASE-RATED: the near-collinear band, denominator included ===")
def theta(r):
    x, y, z = r["x"], r["y"], r["z"]
    u = max(-1.0, min(1.0, (x*x+y*y-z*z)/(2.0*x*y)))
    return math.degrees(math.acos(u))
for lo in (170, 160, 150, 140, 120, 90, 0):
    band = [r for r in rows if theta(r) >= lo]
    big = [r for r in band if r["diff"] >= 1e-4]
    allbig = [r for r in rows if r["diff"] >= 1e-4]
    print(f"  theta >= {lo:3d} deg:  {len(band):>4} of {len(rows)} triples "
          f"({100.0*len(band)/len(rows):>5.1f}% of the sample)   "
          f"holds {len(big)} of {len(allbig)} disagreements >= 1e-4   median|diff| {summ(band).split()[1]}")
band = [r for r in rows if theta(r) >= 150]
rest = [r for r in rows if theta(r) < 150]
print(f"\n  theta >= 150 deg   {summ(band)}")
print(f"  theta <  150 deg   {summ(rest)}")
print("\n  and the scf flag INSIDE the near-collinear band (theta >= 150):")
for f, name in ((True, "converged "), (False, "NOT conv. ")):
    print(f"    {name}  {summ([r for r in band if r['scf'] is f])}")
print("  the scf flag OUTSIDE it (theta < 150):")
for f, name in ((True, "converged "), (False, "NOT conv. ")):
    print(f"    {name}  {summ([r for r in rest if r['scf'] is f])}")
print(f"\n  Spearman rho(|diff|, theta) = {spearman([r['diff'] for r in rows], [theta(r) for r in rows]):+.4f}")
