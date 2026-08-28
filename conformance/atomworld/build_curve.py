"""
build_curve.py -- produce the H2/STO-3G/FCI potential curve, forces, structural
gates, the piecewise cubic Hermite renderer contract, and h2_potential.json.

Everything numeric here comes from h2_core.py.  Nothing is quoted from
literature: the dissociation asymptote, R_e and D_e are all computed in-model.
"""

import json
import os
import sys
import time
from multiprocessing import Pool

from mpmath import mp, mpf, nstr, diff

DPS = 60
REPORT_DIGITS = 50
mp.dps = DPS

import h2_core as H  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
NPROC = 16

# Hermite accuracy targets driving adaptive refinement of the grid.
E_TARGET = mpf("1e-9")        # hartree
F_TARGET = mpf("5e-7")        # hartree/bohr
MAX_REFINE_PASSES = 6


def s(x, n=REPORT_DIGITS):
    return nstr(mpf(x), n, strip_zeros=False)


# ---------------------------------------------------------------------------
# BASE GRID
# Uniform in u = R^{-1/4}.  Reason (derivation, not taste): the cubic Hermite
# error on an interval is |E''''| h^4 / 384, and over most of the range E'''' is
# dominated by the nuclear repulsion's d^4(1/R)/dR^4 = 24 R^{-5}.  Equidistributing
# h^4 R^{-5} needs h proportional to R^{5/4}, which is exactly uniform spacing
# in u = R^{-1/4}.  The well region [1.0, 2.2] is refined once more up front, per
# the brief's "denser near the minimum"; adaptive refinement then handles the
# regions where the ELECTRONIC part of E'''' is not negligible.
# ---------------------------------------------------------------------------
R_MIN, R_MAX = mpf("0.3"), mpf("10")
N_BASE = 260
WELL_LO, WELL_HI = mpf("1.0"), mpf("2.2")


def build_grid():
    u_hi = R_MIN ** mpf("-0.25")
    u_lo = R_MAX ** mpf("-0.25")
    us = [u_hi + (u_lo - u_hi) * mpf(i) / (N_BASE - 1) for i in range(N_BASE)]
    base = [u ** mpf(-4) for u in us]
    base[0], base[-1] = R_MIN, R_MAX
    grid = []
    for i, r in enumerate(base):
        grid.append(r)
        if i + 1 < len(base):
            a, b = r, base[i + 1]
            if a >= WELL_LO and b <= WELL_HI:
                grid.append((a + b) / 2)
    grid.sort()
    return grid


# ---------------------------------------------------------------------------
# Worker: both FCI routes plus the exact first derivative at one R.
# ---------------------------------------------------------------------------
def _work(rs):
    mp.dps = DPS
    R = mpf(rs)
    ea, eb = H.energy_both(R)
    d1 = diff(H.energy_route_a, R)
    return (nstr(ea, DPS), nstr(eb, DPS), nstr(d1, DPS))


def _work_d4(rs):
    mp.dps = DPS
    return nstr(diff(H.energy_route_a, mpf(rs), 4), 30)


def _work_dcheck(rs):
    """dE/dR at dps 60 and at dps 95 -- an accuracy check on the primary force."""
    R = mpf(rs)
    mp.dps = DPS
    d60 = diff(H.energy_route_a, R)
    mp.dps = 95
    d95 = diff(H.energy_route_a, mpf(rs))
    mp.dps = DPS
    return nstr(d60, DPS), nstr(abs(d60 - d95), 10)


def evaluate(points, cache):
    """Evaluate any point not already in cache; cache maps R-string -> triple."""
    todo = [p for p in points if p not in cache]
    if todo:
        with Pool(NPROC) as pool:
            for k, v in zip(todo, pool.map(_work, todo)):
                cache[k] = v
    return cache


# ---------------------------------------------------------------------------
# Local high-order Lagrange interpolant derivative.
#   l_i'(x) = sum_{k != i} [ 1/(x_i - x_k) * prod_{j != i,k} (x - x_j)/(x_i - x_j) ]
# valid at nodes as well as between them.
# ---------------------------------------------------------------------------
def lagrange_deriv(xs, ys, x):
    n = len(xs)
    tot = mpf(0)
    for i in range(n):
        acc = mpf(0)
        for k in range(n):
            if k == i:
                continue
            term = 1 / (xs[i] - xs[k])
            for j in range(n):
                if j == i or j == k:
                    continue
                term *= (x - xs[j]) / (xs[i] - xs[j])
            acc += term
        tot += ys[i] * acc
    return tot


def pmap_(fn, args):
    if not args:
        return []
    with Pool(min(NPROC, len(args))) as pool:
        return pool.map(fn, args)


def _work_Eonly(rs):
    mp.dps = DPS
    return nstr(H.energy_route_a(mpf(rs)), DPS)


def _work_d2(rs):
    mp.dps = DPS
    return nstr(diff(H.energy_route_a, mpf(rs), 2), DPS)


def _work_turning(payload):
    """Classical turning points at a total pair energy, by bisection on E(R)-E_tot.

    Returns (R_in, R_out or "", flag) where flag is one of
      "bound"        -- both turning points inside [R_MIN, R_MAX]
      "outer_beyond" -- bound, but the outer turning point lies past R_MAX
      "unbound"      -- E_tot >= the dissociation asymptote; no outer turning point
    """
    mp.dps = DPS
    etot_s, re_s = payload
    etot, Re = mpf(etot_s), mpf(re_s)
    btol = mpf("1e-30")

    def f(R):
        return H.energy_route_a(R) - etot

    lo, hi = R_MIN, Re                      # inner: E decreasing, f(lo) > 0 > f(hi)
    if f(lo) <= 0:
        return (nstr(R_MIN, DPS), "", "inner_beyond")
    while hi - lo > btol:
        mid = (lo + hi) / 2
        if f(mid) > 0:
            lo = mid
        else:
            hi = mid
    R_in = (lo + hi) / 2

    e_asym = 2 * H.h_atom_energy()
    if etot >= e_asym:
        return (nstr(R_in, DPS), "", "unbound")
    if f(R_MAX) <= 0:
        return (nstr(R_in, DPS), "", "outer_beyond")
    lo, hi = Re, R_MAX                      # outer: E increasing, f(lo) < 0 < f(hi)
    while hi - lo > btol:
        mid = (lo + hi) / 2
        if f(mid) < 0:
            lo = mid
        else:
            hi = mid
    return (nstr(R_in, DPS), nstr((lo + hi) / 2, DPS), "bound")


# ---------------------------------------------------------------------------
# Newton divided-difference form of a local interpolant, and its first two
# derivatives.  Used for the E'' cross-check: building the coefficients is
# O(n^2) and each evaluation O(n), where the Lagrange l_i''(x) route is O(n^4).
#   P(x)   = sum_i c_i N_i(x),   N_i(x) = prod_{j<i} (x - x_j)
#   N_{i+1}   = N_i (x - x_i)
#   N'_{i+1}  = N'_i (x - x_i) + N_i
#   N''_{i+1} = N''_i (x - x_i) + 2 N'_i
# ---------------------------------------------------------------------------
def newton_coeffs(xs, ys):
    c = list(ys)
    n = len(xs)
    for j in range(1, n):
        for i in range(n - 1, j - 1, -1):
            c[i] = (c[i] - c[i - 1]) / (xs[i] - xs[i - j])
    return c


def newton_derivs(xs, cs, x):
    P = P1 = P2 = mpf(0)
    N, N1, N2 = mpf(1), mpf(0), mpf(0)
    for i in range(len(cs)):
        P += cs[i] * N
        P1 += cs[i] * N1
        P2 += cs[i] * N2
        if i < len(cs) - 1:
            d = x - xs[i]
            N2 = N2 * d + 2 * N1
            N1 = N1 * d + N
            N = N * d
    return P, P1, P2


def window(idx, n, half):
    lo = max(0, min(idx - half, n - 2 * half - 1))
    return lo, lo + 2 * half + 1


# ---------------------------------------------------------------------------
# Piecewise cubic Hermite
# ---------------------------------------------------------------------------
def hermite_interval(x0, x1, y0, y1, d0, d1, x):
    h = x1 - x0
    t = (x - x0) / h
    t2, t3 = t * t, t * t * t
    val = ((2 * t3 - 3 * t2 + 1) * y0 + (t3 - 2 * t2 + t) * h * d0
           + (-2 * t3 + 3 * t2) * y1 + (t3 - t2) * h * d1)
    der = ((6 * t2 - 6 * t) * y0 + (3 * t2 - 4 * t + 1) * h * d0
           + (-6 * t2 + 6 * t) * y1 + (3 * t2 - 2 * t) * h * d1) / h
    return val, der


def hermite_eval(knots, vals, ders, x):
    lo, hi = 0, len(knots) - 1
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if knots[mid] <= x:
            lo = mid
        else:
            hi = mid
    return hermite_interval(knots[lo], knots[lo + 1], vals[lo], vals[lo + 1],
                            ders[lo], ders[lo + 1], x)


# Test abscissae for measuring the Hermite bound.  The leading cubic-Hermite
# error is proportional to t^2(1-t)^2, so the VALUE error peaks at t = 1/2 and
# the DERIVATIVE error, proportional to t(1-t)(1-2t), peaks at t = 1/2 +/- 1/(2 sqrt 3).
# Both extrema are sampled exactly; the rest brackets them.
_TSQ = 1 / (2 * mp.sqrt(3))
TEST_TS = [mpf(1) / 8, mpf(1) / 2 - _TSQ, mpf(1) / 3, mpf(1) / 2,
           mpf(2) / 3, mpf(1) / 2 + _TSQ, mpf(7) / 8]


def hermite_interval_errors(grid, Ea, dE, cache):
    """Per-interval (maxErrE, maxErrF), evaluating the exact model at TEST_TS."""
    tests = []
    for i in range(len(grid) - 1):
        for t in TEST_TS:
            tests.append(nstr(grid[i] + t * (grid[i + 1] - grid[i]), DPS))
    evaluate(tests, cache)
    out = []
    k = 0
    for i in range(len(grid) - 1):
        eE = eF = mpf(0)
        for _ in TEST_TS:
            rr = mpf(tests[k])
            ex_E = mpf(cache[tests[k]][0])
            ex_F = -mpf(cache[tests[k]][2])
            hv, hd = hermite_interval(grid[i], grid[i + 1], Ea[i], Ea[i + 1],
                                      dE[i], dE[i + 1], rr)
            eE = max(eE, abs(hv - ex_E))
            eF = max(eF, abs(-hd - ex_F))
            k += 1
        out.append((eE, eF))
    return out, tests


# ---------------------------------------------------------------------------
def main():
    t_start = time.time()
    log = []

    def emit(line=""):
        print(line, flush=True)
        log.append(line)

    emit("=" * 78)
    emit("H2 / STO-3G / FCI  --  exact-in-model potential core")
    emit("=" * 78)
    emit(f"working precision : mp.dps = {DPS}")
    emit(f"reported digits   : {REPORT_DIGITS}")
    emit(f"contraction <chi|chi> before renormalisation = "
         f"{s(H.contraction_raw_norm(), 25)}")
    emit("  (the contraction is renormalised to 1; the tabulated STO-3G "
         "coefficients are rounded)")
    emit()

    # ---- in-model asymptote -------------------------------------------------
    e_h = H.h_atom_energy()
    e_asym = 2 * e_h
    emit("IN-MODEL DISSOCIATION ASYMPTOTE (derived here, not quoted)")
    emit(f"  E_H  (STO-3G, 1 electron, 1 proton) = {s(e_h)}")
    emit(f"  E_asymptote = 2 E_H                 = {s(e_asym)}")
    emit()

    # ---- grid + adaptive refinement ----------------------------------------
    grid = build_grid()
    emit(f"BASE GRID: {len(grid)} points, R in [{s(grid[0],6)}, {s(grid[-1],6)}] bohr")
    cache = {}

    def load(g):
        keys = [nstr(r, DPS) for r in g]
        evaluate(keys, cache)
        return ([mpf(cache[k][0]) for k in keys],
                [mpf(cache[k][1]) for k in keys],
                [mpf(cache[k][2]) for k in keys])

    Ea, Eb, dE = load(grid)
    emit(f"  evaluated in {time.time()-t_start:.1f}s")
    emit()
    emit("ADAPTIVE REFINEMENT  (targets: |dE| <= "
         f"{s(E_TARGET,3)} Ha, |dF| <= {s(F_TARGET,3)} Ha/bohr)")
    for p in range(MAX_REFINE_PASSES):
        errs, _ = hermite_interval_errors(grid, Ea, dE, cache)
        bad = [i for i, (ee, ef) in enumerate(errs)
               if ee > E_TARGET or ef > F_TARGET]
        mE = max(e for e, _ in errs)
        mF = max(f for _, f in errs)
        emit(f"  pass {p}: n={len(grid):4d}  max|dE|={s(mE,6)}  "
             f"max|dF|={s(mF,6)}  intervals over target: {len(bad)}")
        if not bad:
            break
        newpts = [(grid[i] + grid[i + 1]) / 2 for i in bad]
        grid = sorted(grid + newpts)
        Ea, Eb, dE = load(grid)
    else:
        emit("  WARNING: refinement did not converge within "
             f"{MAX_REFINE_PASSES} passes")
    Fv = [-d for d in dE]
    hs = [grid[i + 1] - grid[i] for i in range(len(grid) - 1)]
    emit(f"  FINAL GRID: {len(grid)} points; spacing h: min {s(min(hs),6)} "
         f"max {s(max(hs),6)}")
    nwell = sum(1 for r in grid if WELL_LO <= r <= WELL_HI)
    emit(f"  points in well region [{s(WELL_LO,3)}, {s(WELL_HI,3)}]: {nwell}")
    emit(f"  elapsed {time.time()-t_start:.1f}s")
    emit()

    gates = {}

    # ---- G1: route agreement ------------------------------------------------
    diffs = [abs(a - b) for a, b in zip(Ea, Eb)]
    worst = max(diffs)
    iworst = diffs.index(worst)
    rel = max(abs(a - b) / max(abs(a), mpf(1)) for a, b in zip(Ea, Eb))
    tol = mpf(10) ** (-(DPS - 8))
    gates["G1_route_agreement"] = bool(worst < tol)
    emit("G1  ROUTE AGREEMENT  (2x2 singlet CI  vs  16-dim Fock-space ED, "
         "2-electron block)")
    emit(f"  compared at all {len(grid)} grid points")
    emit(f"  max |E_a - E_b| = {s(worst,6)}  at R = {s(grid[iworst],12)}")
    emit(f"  max relative    = {s(rel,6)}")
    emit(f"  tolerance       = {s(tol,6)}   -> "
         f"{'PASS' if gates['G1_route_agreement'] else 'FAIL'}")
    emit()

    # ---- G2: exactly one minimum -------------------------------------------
    sign_changes = []
    for i in range(len(dE) - 1):
        if dE[i] < 0 <= dE[i + 1]:
            sign_changes.append(("min", i))
        elif dE[i] > 0 >= dE[i + 1]:
            sign_changes.append(("max", i))
    n_min = sum(1 for k, _ in sign_changes if k == "min")
    n_max = sum(1 for k, _ in sign_changes if k == "max")
    gates["G2_exactly_one_minimum"] = bool(n_min == 1 and n_max == 0)
    emit("G2  EXACTLY ONE MINIMUM  (sign changes of dE/dR on the grid)")
    emit(f"  minima (dE/dR: - -> +) = {n_min}   maxima (+ -> -) = {n_max}")
    emit(f"  -> {'PASS' if gates['G2_exactly_one_minimum'] else 'FAIL'}")
    emit()

    imin = [i for k, i in sign_changes if k == "min"][0]

    # ---- R_e by exact bisection on dE/dR of the EXACT function --------------
    emit("R_e BY BISECTION")
    lo, hi = grid[imin], grid[imin + 1]
    flo = diff(H.energy_route_a, lo)
    fhi = diff(H.energy_route_a, hi)
    assert flo < 0 < fhi, "bracket lost"
    btol = mpf(10) ** (-(REPORT_DIGITS - 5))
    nit = 0
    while hi - lo > btol:
        mid = (lo + hi) / 2
        fm = diff(H.energy_route_a, mid)
        if fm < 0:
            lo = mid
        else:
            hi = mid
        nit += 1
    Re_exact = (lo + hi) / 2
    emit(f"  bracket from grid : [{s(grid[imin],14)}, {s(grid[imin+1],14)}]")
    emit(f"  bisection on the EXACT dE/dR: {nit} iterations, "
         f"final bracket width {s(hi-lo,6)}")
    emit(f"  R_e = {s(Re_exact)} bohr")
    emit(f"  residual dE/dR(R_e) = {s(diff(H.energy_route_a, Re_exact), 6)}")

    half = 6
    wlo, whi = window(imin, len(grid), half)
    wx, wy = grid[wlo:whi], Ea[wlo:whi]
    lo2, hi2 = grid[imin], grid[imin + 1]
    for _ in range(400):
        if hi2 - lo2 <= btol:
            break
        mid = (lo2 + hi2) / 2
        if lagrange_deriv(wx, wy, mid) < 0:
            lo2 = mid
        else:
            hi2 = mid
    Re_interp = (lo2 + hi2) / 2
    dRe = abs(Re_exact - Re_interp)
    emit(f"  cross-check: bisection on the derivative of a degree-{whi-wlo-1} "
         f"local Lagrange")
    emit(f"    interpolant ({whi-wlo} nodes, R in [{s(wx[0],8)}, {s(wx[-1],8)}])")
    emit(f"    R_e (interpolant) = {s(Re_interp, 30)}")
    emit(f"    |R_e(exact) - R_e(interpolant)| = {s(dRe, 6)}")
    emit(f"  precision of R_e: bisection bracket {s(hi-lo,3)}; the two "
         f"independent locators agree to {s(dRe,3)}")
    emit()

    # ---- D_e ---------------------------------------------------------------
    E_at_Re_a, E_at_Re_b = H.energy_both(Re_exact)
    De = e_asym - E_at_Re_a
    E2_at_Re = diff(H.energy_route_a, Re_exact, 2)
    emit("WELL DEPTH (in-model)")
    emit(f"  E(R_e)      = {s(E_at_Re_a)}")
    emit(f"  route b     = {s(E_at_Re_b)}   |a-b| = {s(abs(E_at_Re_a-E_at_Re_b),6)}")
    emit(f"  E_asymptote = {s(e_asym)}")
    emit(f"  D_e = E_asym - E(R_e) = {s(De)} hartree")
    emit(f"      = {s(De*mpf('27.211386245988'),20)} eV   "
         "[eV is a unit conversion, not a model result]")
    emit("  CAVEAT: these are EXACT-IN-MODEL values for STO-3G FCI. They are")
    emit("  not predictions of experiment and are not gated against experiment.")
    emit()

    # ---- G3: approach to the asymptote from below --------------------------
    tail = [(r, e) for r, e in zip(grid, Ea) if r >= Re_exact]
    below = all(e < e_asym for _, e in tail)
    mono = all(tail[i][1] < tail[i + 1][1] for i in range(len(tail) - 1))
    gap_end = e_asym - Ea[-1]
    gate_from_below = bool(below and mono and gap_end > 0)
    gates["G3_asymptote_from_below"] = gate_from_below
    emit("G3  E(R) -> E_asymptote FROM BELOW  (for R >= R_e)")
    emit(f"  E(R) < E_asymptote at all {len(tail)} tail points : {below}")
    emit(f"  E strictly increasing on the tail                 : {mono}")
    emit(f"  E_asym - E(R=10) = {s(gap_end,8)}  (> 0 means approached from below)")
    for rr in ("2", "3", "5", "7", "10"):
        ee = H.energy_route_a(mpf(rr))
        emit(f"    R = {rr:>4}  E_asym - E = {s(e_asym-ee, 8)}")
    emit(f"  -> {'PASS' if gate_from_below else 'FAIL'}")
    emit()

    # ---- G4: divergence as R -> 0 ------------------------------------------
    emit("G4  E(R) -> +infinity AS R -> 0  (nuclear repulsion dominates)")
    small = ["0.1", "0.03", "0.01", "0.003", "0.001"]
    svals = [H.energy_route_a(mpf(x)) for x in small]
    prods = [mpf(x) * v for x, v in zip(small, svals)]
    for x, v, pr in zip(small, svals, prods):
        emit(f"    R = {x:>6}  E = {s(v,20):>28}   R*E = {s(pr,20)}")
    monotone_up = all(svals[i] > svals[i - 1] for i in range(1, len(svals)))
    approaches_one = abs(prods[-1] - 1) < mpf("1e-2")
    positive_growth = all(p > 0 for p in prods)
    gate_div = bool(monotone_up and approaches_one and positive_growth)
    gates["G4_diverges_at_zero"] = gate_div
    emit(f"  E increases monotonically as R decreases  : {monotone_up}")
    emit(f"  R*E(R) -> 1, i.e. the divergence is the 1/R rate: "
         f"R*E = {s(prods[-1],12)} at R = 0.001")
    emit(f"  -> {'PASS' if gate_div else 'FAIL'}")
    emit()

    # ---- forces -------------------------------------------------------------
    emit("FORCES  F(R) = -dE/dR")
    emit("  PRIMARY: exact numerical differentiation of E(R) at working precision.")
    probe = ["0.3", nstr(Re_exact, 30), "4.5", "10.0"]
    with Pool(min(NPROC, len(probe))) as pool:
        dchk = pool.map(_work_dcheck, probe)
    worst_dd = mpf(0)
    emit("  self-consistency of the primary force (recomputed at mp.dps = 95):")
    for rr, (d60, dd) in zip(probe, dchk):
        worst_dd = max(worst_dd, mpf(dd))
        emit(f"    R = {rr[:14]:>14}  dE/dR = {s(mpf(d60),20):>26}  "
             f"|d(60) - d(95)| = {dd}")
    emit(f"  -> the primary force is converged to {s(worst_dd,3)} absolute.")
    F_interp = []
    for i in range(len(grid)):
        wlo, whi = window(i, len(grid), 4)
        F_interp.append(-lagrange_deriv(grid[wlo:whi], Ea[wlo:whi], grid[i]))
    d_interp = [abs(a - b) for a, b in zip(Fv, F_interp)]
    F_fd = [None] * len(grid)
    for i in range(1, len(grid) - 1):
        h1 = grid[i] - grid[i - 1]
        h2 = grid[i + 1] - grid[i]
        d = (-h2 / (h1 * (h1 + h2)) * Ea[i - 1]
             + (h2 - h1) / (h1 * h2) * Ea[i]
             + h1 / (h2 * (h1 + h2)) * Ea[i + 1])
        F_fd[i] = -d
    d_fd = [abs(Fv[i] - F_fd[i]) for i in range(1, len(grid) - 1)]
    max_interp = max(d_interp)
    max_fd = max(d_fd)
    i_ip = d_interp.index(max_interp)
    i_fd = d_fd.index(max_fd) + 1
    emit("  CHECK 1 -- derivative of a degree-8 local Lagrange interpolant "
         "at every node:")
    emit(f"    max |F_primary - F_lagrange| = {s(max_interp,6)} at "
         f"R = {s(grid[i_ip],10)}")
    emit("  CHECK 2 -- 3-point central finite difference of the raw curve "
         "(non-uniform):")
    emit(f"    max |F_primary - F_centraldiff| = {s(max_fd,6)} at "
         f"R = {s(grid[i_fd],10)}")
    emit("  Both discrepancies are the truncation error OF THE CHECK "
         "(O(h^8) and O(h^2)),")
    emit("  not of the primary force, which is converged to "
         f"{s(worst_dd,3)} above.")
    emit()

    # ---- Hermite representation and its bound ------------------------------
    emit("PIECEWISE CUBIC HERMITE  (renderer contract, C1)")
    emit(f"  knots = the {len(grid)} grid points; values = E; slopes = dE/dR.")
    emit("  At a knot the basis collapses to 1*y0, so the VALUE is reproduced")
    emit("  identically; the SLOPE is reconstructed as (h*d0)/h, one "
         "multiply-divide")
    emit("  round trip, so it is exact only to working round-off (and, in the "
         "emitted")
    emit("  f64 contract, to a few ulp).  Measured:")
    knot_err_v = mpf(0)
    knot_err_d = mpf(0)
    for i in range(len(grid) - 1):
        v, d = hermite_interval(grid[i], grid[i + 1], Ea[i], Ea[i + 1],
                                dE[i], dE[i + 1], grid[i])
        knot_err_v = max(knot_err_v, abs(v - Ea[i]))
        knot_err_d = max(knot_err_d, abs(d - dE[i]))
    knot_err = max(knot_err_v, knot_err_d)
    emit(f"    max |E_hermite(knot) - E(knot)|      = {s(knot_err_v,6)}")
    emit(f"    max |dE_hermite(knot) - dE/dR(knot)| = {s(knot_err_d,6)}")
    errs, tests = hermite_interval_errors(grid, Ea, dE, cache)
    bE = max(e for e, _ in errs)
    bF = max(f for _, f in errs)
    iE = max(range(len(errs)), key=lambda i: errs[i][0])
    iF = max(range(len(errs)), key=lambda i: errs[i][1])
    emit(f"  BOUND, measured against the exact model at "
         f"{len(TEST_TS)*(len(grid)-1)} interior points "
         f"({len(TEST_TS)} per interval, including the analytic error extrema "
         "t = 1/2 and t = 1/2 +/- 1/(2 sqrt 3)):")
    emit(f"    max |E_hermite - E_exact| = {s(bE,8)} hartree        "
         f"(interval near R = {s(grid[iE],10)})")
    emit(f"    max |F_hermite - F_exact| = {s(bF,8)} hartree/bohr   "
         f"(interval near R = {s(grid[iF],10)})")
    emit(f"    relative to the well depth D_e: {s(bE/De,6)}")
    emit("    This is a MEASURED maximum over that stated test set, not a "
         "proven a-priori bound.")
    emit("  a-priori corroboration,  |E - Hermite| <= max|E''''| h^4 / 384 :")
    probe4 = ["0.3", "0.5", "1.0", nstr(Re_exact, 12), "2.0", "4.0", "4.5", "10.0"]
    with Pool(min(NPROC, len(probe4))) as pool:
        d4s = pool.map(_work_d4, probe4)
    for rr, d4 in zip(probe4, d4s):
        rrm = mpf(rr)
        j = min(range(len(grid) - 1), key=lambda k: abs(grid[k] - rrm))
        j = min(j, len(grid) - 2)
        hh = grid[j + 1] - grid[j]
        bnd = abs(mpf(d4)) * hh ** 4 / 384
        emit(f"    R = {rr:>14}  E'''' = {s(mpf(d4),10):>16}  h = {s(hh,6)}  "
             f"bound = {s(bnd,6)}")
    f64_eps = mpf(2) ** -52 * max(abs(e) for e in Ea)
    emit(f"  f64 storage round-off on E is <= {s(f64_eps,6)}, which is "
         f"{s(f64_eps/bE,4)} of the")
    emit("  Hermite bound -- so emitting the contract in doubles does not "
         "degrade it.")
    emit()

    # ---- E''(R) on the grid ------------------------------------------------
    emit("SECOND DERIVATIVE  E''(R)  (contract amendment: curvature envelope)")
    emit("  PRIMARY: exact numerical differentiation of E(R) at working precision.")
    gkeys = [nstr(r, DPS) for r in grid]
    E2 = [mpf(x) for x in pmap_(_work_d2, gkeys)]
    emit(f"  evaluated at all {len(grid)} grid points")
    # CHECK 1: second derivative of a degree-8 local Newton interpolant
    E2_interp = []
    for i in range(len(grid)):
        wlo, whi = window(i, len(grid), 4)
        wx, wy = grid[wlo:whi], Ea[wlo:whi]
        cs = newton_coeffs(wx, wy)
        _, p1, p2 = newton_derivs(wx, cs, grid[i])
        E2_interp.append(p2)
    d_ip = [abs(a - b) for a, b in zip(E2, E2_interp)]
    m_ip = max(d_ip)
    i_ip = d_ip.index(m_ip)
    r_ip = max(abs(a - b) / max(abs(a), mpf(1)) for a, b in zip(E2, E2_interp))
    # CHECK 2: 3-point second finite difference of the raw curve (non-uniform)
    #   f'' ~ 2[h2 f(i-1) - (h1+h2) f(i) + h1 f(i+1)] / (h1 h2 (h1+h2))
    E2_fd = [None] * len(grid)
    for i in range(1, len(grid) - 1):
        h1 = grid[i] - grid[i - 1]
        h2 = grid[i + 1] - grid[i]
        E2_fd[i] = (2 * (h2 * Ea[i - 1] - (h1 + h2) * Ea[i] + h1 * Ea[i + 1])
                    / (h1 * h2 * (h1 + h2)))
    d_fd2 = [abs(E2[i] - E2_fd[i]) for i in range(1, len(grid) - 1)]
    m_fd2 = max(d_fd2)
    i_fd2 = d_fd2.index(m_fd2) + 1
    r_fd2 = max(abs(E2[i] - E2_fd[i]) / max(abs(E2[i]), mpf(1))
                for i in range(1, len(grid) - 1))
    emit("  CHECK 1 -- 2nd derivative of a degree-8 local Newton interpolant "
         "at every node:")
    emit(f"    max |E2_primary - E2_interp| = {s(m_ip,6)} at R = "
         f"{s(grid[i_ip],10)}   (max relative {s(r_ip,6)})")
    emit("  CHECK 2 -- 3-point second finite difference of the raw curve:")
    emit(f"    max |E2_primary - E2_fd| = {s(m_fd2,6)} at R = "
         f"{s(grid[i_fd2],10)}   (max relative {s(r_fd2,6)})")
    emit("  A 3-point second difference is only O(h) accurate on a NON-uniform")
    emit("  grid (O(h^2) uniform), so CHECK 2's discrepancy is expected to be the")
    emit("  larger of the two.  Both are the truncation error of the check.")
    emit(f"  E''(R_e) = {s(E2_at_Re,20)}   E''(R_MIN=0.3) = {s(E2[0],20)}")
    emit(f"  stiffness ratio wall/well = {s(abs(E2[0]/E2_at_Re),10)} -- this is why")
    emit("  a timestep set from E''(R_e) alone is unsafe on the repulsive wall.")
    emit()

    # ---- curvature envelope table ------------------------------------------
    emit("CURVATURE ENVELOPE  max_curvature_up_to_E")
    emit("  For a total pair energy E_tot, the classically accessible range is")
    emit("  {R : E(R) <= E_tot}; the table reports max |E''| over that range.")
    eps_max = (Ea[0] - E_at_Re_a) * (1 - mpf("1e-15"))
    eps_min = mpf("1e-5")
    N_RUNG = 80
    eps_list = [mpf(0)]
    for k in range(N_RUNG):
        t = mpf(k) / (N_RUNG - 1)
        eps_list.append(eps_min * (eps_max / eps_min) ** t)
    eps_list.append(De)                      # the dissociation threshold itself
    seen, eps_rungs = set(), []
    for e in sorted(eps_list):
        k = nstr(e, 30)
        if k not in seen:
            seen.add(k)
            eps_rungs.append(e)
    etots = [E_at_Re_a + e for e in eps_rungs]
    emit(f"  ladder: {len(eps_rungs)} rungs, geometric in (E_tot - E(R_e)) from "
         f"{s(eps_min,3)} to {s(eps_max,8)} Ha,")
    emit(f"  plus the exact minimum (0) and the dissociation threshold "
         f"D_e = {s(De,12)}.")
    emit("  The top rung is the energy whose INNER turning point is the tabulated")
    emit(f"  domain edge R = {s(R_MIN,4)}; above it the model is not tabulated here.")

    tp = pmap_(_work_turning, [(nstr(e, DPS), nstr(Re_exact, DPS))
                               for e in etots])
    # verify the turning points really sit at E_tot, and gather E'' there
    tp_pts = []
    for a, b, flag in tp:
        tp_pts.append(a)
        if b:
            tp_pts.append(b)
    tp_pts = sorted(set(tp_pts))
    tpE = dict(zip(tp_pts, pmap_(_work_Eonly, tp_pts)))
    tpD2 = dict(zip(tp_pts, pmap_(_work_d2, tp_pts)))
    worst_tp = mpf(0)
    for (a, b, flag), et in zip(tp, etots):
        worst_tp = max(worst_tp, abs(mpf(tpE[a]) - et))
        if b:
            worst_tp = max(worst_tp, abs(mpf(tpE[b]) - et))
    emit(f"  turning points located by bisection; max |E(R_turn) - E_tot| = "
         f"{s(worst_tp,6)}")

    # coarse argmax over grid points inside the range, plus the endpoints
    coarse = []
    for (a, b, flag), et in zip(tp, etots):
        R_in = mpf(a)
        R_hi = mpf(b) if b else R_MAX
        gridonly = [(abs(E2[i]), r, "grid") for i, r in enumerate(grid)
                    if R_in <= r <= R_hi]
        cands = [(abs(mpf(tpD2[a])), R_in, "R_in")] + gridonly
        if b:
            cands.append((abs(mpf(tpD2[b])), mpf(b), "R_out"))
        cands.sort(key=lambda z: z[0])
        g_max = max([c[0] for c in gridonly], default=mpf(0))
        coarse.append((cands[-1], R_in, R_hi, flag, g_max))

    # local refinement around each coarse argmax (the grid may straddle the peak)
    NREF = 13
    ref_pts, ref_slice = [], []
    for (best, R_in, R_hi, flag, g_max) in coarse:
        c = best[1]
        j = min(range(len(grid)), key=lambda k: abs(grid[k] - c))
        lo_r = max(R_in, grid[max(j - 1, 0)])
        hi_r = min(R_hi, grid[min(j + 1, len(grid) - 1)])
        start = len(ref_pts)
        for k in range(NREF):
            x = lo_r + (hi_r - lo_r) * mpf(k) / (NREF - 1)
            ref_pts.append(nstr(x, DPS))
        ref_slice.append((start, start + NREF))
    emit(f"  refining the argmax of |E''| with {NREF} points per rung "
         f"({len(ref_pts)} evaluations) ...")
    ref_vals = pmap_(_work_d2, ref_pts)

    rows = []
    gain = mpf(0)
    at_Rin = 0
    ctl_gain = mpf(0)          # positive control: refinement vs GRID-ONLY
    ctl_fired = 0
    ref_reach = mpf(1)         # how close refinement points get to the answer
    for (best, R_in, R_hi, flag, g_max), (a0, a1) in zip(coarse, ref_slice):
        bv, br, bwhere = best
        rmax = mpf(0)
        for k in range(a0, a1):
            v = abs(mpf(ref_vals[k]))
            rmax = max(rmax, v)
            if v > bv:
                bv, br, bwhere = v, mpf(ref_pts[k]), "refined"
        gain = max(gain, (bv - best[0]) / max(best[0], mpf("1e-30")))
        if g_max > 0:
            g = (max(rmax, g_max) - g_max) / g_max
            if g > 0:
                ctl_fired += 1
            ctl_gain = max(ctl_gain, g)
        ref_reach = min(ref_reach, rmax / bv)
        if abs(br - R_in) < mpf("1e-12"):
            at_Rin += 1
        rows.append((bv, br, bwhere, R_in, R_hi, flag))
    emit(f"  refinement raised the FULL coarse candidate set by at most "
         f"{s(gain*100,4)}%")
    emit(f"  the maximum sits AT the inner turning point on {at_Rin} of "
         f"{len(rows)} rungs, which is why:")
    emit("    |E''| falls monotonically from the wall through R_e, and the inner")
    emit("    turning point is always <= R_e, so the range's left endpoint is its")
    emit("    stiffest point.  The endpoint is computed exactly, so there is")
    emit("    nothing left for refinement to find -- the 0% is a fact about the")
    emit("    curve, not a scan that cannot fire.  Positive control:")
    emit(f"    refinement beats a GRID-ONLY candidate set on {ctl_fired} of "
         f"{len(rows)} rungs, by up to {s(ctl_gain*100,4)}%,")
    emit(f"    and its best point reaches >= {s(ref_reach*100,6)}% of the answer "
         "on every rung.")

    # dt helper, mass-free:  dt = sqrt(mu) * 2 pi / (64 sqrt(maxE''))
    dt_per_sqrt_mu = [2 * mp.pi / (64 * mp.sqrt(bv)) for bv, *_ in rows]

    envelope = []
    for e, et, (bv, br, bwhere, R_in, R_hi, flag), dts in zip(
            eps_rungs, etots, rows, dt_per_sqrt_mu):
        envelope.append(dict(
            eps_above_min_hartree=float(e),
            E_total_hartree=float(et),
            R_in_bohr=float(R_in),
            R_out_bohr=(float(R_hi) if flag == "bound" else None),
            range_flag=flag,
            max_abs_E2_hartree_per_bohr2=float(bv),
            argmax_R_bohr=float(br),
            dt_per_sqrt_mu=float(dts),
        ))

    emit("  sample rungs:")
    emit(f"    {'eps (Ha)':>13} {'E_tot (Ha)':>14} {'R_in':>10} {'R_out':>10} "
         f"{'max|E2|':>13} {'flag':>13}")
    for k in (0, 1, len(rows) // 4, len(rows) // 2, 3 * len(rows) // 4,
              len(rows) - 2, len(rows) - 1):
        bv, br, bwhere, R_in, R_hi, flag = rows[k]
        ro = s(R_hi, 8) if flag == "bound" else "-"
        emit(f"    {s(eps_rungs[k],8):>13} {s(etots[k],10):>14} "
             f"{s(R_in,8):>10} {ro:>10} {s(bv,8):>13} {flag:>13}")
    emit()

    # ---- G5: envelope monotone --------------------------------------------
    mono_env = all(rows[i][0] <= rows[i + 1][0] for i in range(len(rows) - 1))
    gates["G5_envelope_monotone"] = bool(mono_env)
    emit("G5  CURVATURE ENVELOPE IS NON-DECREASING IN ENERGY")
    emit("  the accessible set grows with E_tot, so its max |E''| cannot fall")
    emit(f"  non-decreasing across all {len(rows)} rungs: {mono_env}")
    emit(f"  span: {s(rows[0][0],10)} at the minimum -> {s(rows[-1][0],10)} at "
         f"the top rung ({s(rows[-1][0]/rows[0][0],6)}x)")
    emit(f"  -> {'PASS' if mono_env else 'FAIL'}")
    emit()

    # ---- G6: turning points correct and monotone ---------------------------
    rin_dec = all(rows[i][3] > rows[i + 1][3] for i in range(len(rows) - 1))
    bnd = [(i, r) for i, r in enumerate(rows) if r[5] == "bound"]
    rout_inc = all(bnd[i][1][4] < bnd[i + 1][1][4] for i in range(len(bnd) - 1))
    tp_ok = worst_tp < mpf("1e-25")
    gate_tp = bool(rin_dec and rout_inc and tp_ok)
    gates["G6_turning_points_consistent"] = gate_tp
    emit("G6  TURNING POINTS CORRECT AND MONOTONE")
    emit(f"  max |E(R_turn) - E_tot| = {s(worst_tp,6)} (< 1e-25: {tp_ok})")
    emit(f"  R_in strictly decreasing in E_tot: {rin_dec}")
    emit(f"  R_out strictly increasing over the {len(bnd)} bound rungs: {rout_inc}")
    emit(f"  -> {'PASS' if gate_tp else 'FAIL'}")
    emit()

    # ---- JSON contract -----------------------------------------------------
    emit("EMITTING RENDERER CONTRACT")
    doc = {
        "model": H.MODEL_NAME,
        "precision_digits": REPORT_DIGITS,
        "working_precision_dps": DPS,
        "units": {"R": "bohr", "E": "hartree", "F": "hartree/bohr"},
        "note": ("Exact-in-model: full CI in the STO-3G minimal basis, computed "
                 "from closed-form Gaussian integrals in mpmath. NOT a prediction "
                 "of experiment. The dissociation asymptote, R_e and D_e are all "
                 "computed by this code, none are quoted."),
        "R_grid_bohr": [float(r) for r in grid],
        "E_hartree": [float(e) for e in Ea],
        "F_hartree_per_bohr": [float(f) for f in Fv],
        "E2_hartree_per_bohr2": [float(x) for x in E2],
        "R_e": float(Re_exact),
        "D_e": float(De),
        "E2_at_R_e": float(E2_at_Re),
        "max_curvature_up_to_E": {
            "purpose": ("timestep envelope: the renderer must set dt from the "
                        "stiffest CURVATURE it can reach at the current maximum "
                        "pair energy, not from E''(R_e). The repulsive wall is "
                        f"{float(abs(E2[0]/E2_at_Re)):.1f}x stiffer than the well "
                        "bottom over this table's range."),
            "lookup": ("take the FIRST rung whose E_total_hartree is >= your "
                       "current maximum pair energy, i.e. round UP. Using the "
                       "rung below understates the stiffness and yields a dt "
                       "that is too large."),
            "dt_formula": ("dt = 2*pi/(64*sqrt(max_abs_E2/mu)) = sqrt(mu) * "
                           "dt_per_sqrt_mu, with mu the reduced mass in the "
                           "renderer's units. dt_per_sqrt_mu is tabulated so no "
                           "mass constant enters this contract: the electronic "
                           "model is Born-Oppenheimer and knows no nuclear mass."),
            "domain": ("E_total above the top rung is NOT tabulated: its inner "
                       "turning point lies below R = 0.3 bohr, off the grid. A "
                       "renderer reaching that energy must refuse or clamp."),
            "accessible_range": ("{R : E(R) <= E_total}. range_flag 'bound' has "
                                 "both turning points on the grid; 'outer_beyond' "
                                 "is bound with R_out past R = 10; 'unbound' has "
                                 "E_total >= the dissociation asymptote and no "
                                 "outer turning point. In the last two cases "
                                 "max|E''| is still attained on the grid, since "
                                 "|E''| decays to zero beyond R = 10."),
            "rungs": envelope,
        },
        "E_asymptote": float(e_asym),
        "E_at_R_e": float(E_at_Re_a),
        "hermite": {
            "kind": "piecewise_cubic_hermite_C1",
            "knots_bohr": [float(r) for r in grid],
            "values_hartree": [float(e) for e in Ea],
            "derivatives_hartree_per_bohr": [float(d) for d in dE],
            "max_abs_error_E_hartree": float(bE),
            "max_abs_error_F_hartree_per_bohr": float(bF),
            "error_test": ("measured max over 7 interior points per interval "
                           "against the exact model, including the analytic "
                           "error extrema t=1/2 and t=1/2+/-1/(2 sqrt 3)"),
            "eval": ("t=(x-x0)/h; E = (2t^3-3t^2+1)y0 + (t^3-2t^2+t)h d0 + "
                     "(-2t^3+3t^2)y1 + (t^3-t^2)h d1; F = -dE/dx"),
        },
        "exact": {
            "R_grid_bohr": [s(r) for r in grid],
            "E_hartree": [s(e) for e in Ea],
            "F_hartree_per_bohr": [s(f) for f in Fv],
            "E2_hartree_per_bohr2": [s(x) for x in E2],
            "hermite_derivatives_hartree_per_bohr": [s(d) for d in dE],
            "E2_at_R_e": s(E2_at_Re),
            "max_curvature_rung_E_total": [s(x) for x in etots],
            "max_curvature_rung_max_abs_E2": [s(r[0]) for r in rows],
            "max_curvature_rung_R_in": [s(r[3]) for r in rows],
            "R_e": s(Re_exact),
            "D_e": s(De),
            "E_asymptote": s(e_asym),
            "E_H_atom": s(e_h),
            "E_at_R_e": s(E_at_Re_a),
            "hermite_max_abs_error_E": s(bE, 12),
            "hermite_max_abs_error_F": s(bF, 12),
        },
        "gates": gates,
        "diagnostics": {
            "route_agreement_max_abs": s(worst, 12),
            "route_agreement_tolerance": s(tol, 12),
            "primary_force_convergence_abs": s(worst_dd, 12),
            "force_check_vs_lagrange_max_abs": s(max_interp, 12),
            "force_check_vs_central_fd_max_abs": s(max_fd, 12),
            "R_e_exact_vs_interpolant_abs_diff": s(dRe, 12),
            "R_e_bisection_bracket_width": s(hi - lo, 12),
            "E2_check_vs_newton_interpolant_max_abs": s(m_ip, 12),
            "E2_check_vs_newton_interpolant_max_rel": s(r_ip, 12),
            "E2_check_vs_second_fd_max_abs": s(m_fd2, 12),
            "E2_check_vs_second_fd_max_rel": s(r_fd2, 12),
            "envelope_n_rungs": len(rows),
            "envelope_refinement_gain_frac": s(gain, 12),
            "envelope_refinement_control_gain_frac": s(ctl_gain, 12),
            "envelope_refinement_control_rungs_fired": ctl_fired,
            "envelope_refinement_min_reach_frac": s(ref_reach, 12),
            "envelope_max_at_inner_turning_point": at_Rin,
            "envelope_turning_point_residual": s(worst_tp, 12),
            "envelope_stiffness_ratio_wall_over_well": s(abs(E2[0] / E2_at_Re), 12),
            "hermite_knot_value_error": s(knot_err_v, 12),
            "hermite_knot_slope_error": s(knot_err_d, 12),
            "contraction_raw_norm": s(H.contraction_raw_norm(), 25),
            "n_grid": len(grid),
            "n_minima": n_min,
            "n_maxima": n_max,
        },
    }
    out = os.path.join(HERE, "h2_potential.json")
    with open(out, "w") as f:
        json.dump(doc, f, indent=1)
    emit(f"  wrote {out}  ({os.path.getsize(out)} bytes)")
    emit()

    all_pass = all(gates.values())
    emit("GATE SUMMARY")
    for k, v in gates.items():
        emit(f"  {k:<28} {'PASS' if v else 'FAIL'}")
    emit(f"  overall: {'ALL PASS' if all_pass else 'FAILURE PRESENT'}")
    emit(f"total wall time {time.time()-t_start:.1f}s")

    with open(os.path.join(HERE, "build_curve.log"), "w") as f:
        f.write("\n".join(log) + "\n")
    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
