#!/usr/bin/env python3
"""
cosmos_two_portion.py -- the arithmetic behind COSMOS_TWO_PORTION.md.

The 2026-11-23 amendment to the cosmological wager (TIERS.md section 4):
    rho_DE(a) = rho_static + rho_org(a)
a STATIC portion and an ORGANISED portion. This script prices that family
with the instruments the sibling seed already wrote, and prints every number
the note quotes. Nothing here is fitted to data that is not already on the
record, and nothing is fetched.

Carried over from CIRISOntology (read-only; credited, re-implemented here
with numpy alone so the gate needs no scipy):
  scratchpad/temporal-share/DE_LEDGER_MODEL.py   the rent ledger, the CPL
      projection, the DESI DR2 BAO vector (arXiv:2503.14738 Table IV) and the
      CMB-lite acoustic point, the four DR2 (w0, wa) summary points
  scratchpad/flow_pricing.py / FLOW_PRICING_RESULTS.md   the Landauer stock
      table in densities, the star-formation power caps, the FIRAS figure
  scratchpad/preferred-frame-results.md   the two dark-balance branches' w(z)

Run:  python3 conformance/omega/cosmos_two_portion.py
Deps: python3 stdlib + numpy. Single thread.
"""
import os
os.environ.setdefault("OMP_NUM_THREADS", "1")
os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")
import math
import numpy as np

# ---------------------------------------------------------------- constants
KB = 1.380649e-23
LN2 = math.log(2.0)
C_LIGHT = 2.99792458e8
MPC_M = 3.0856775814913673e22
G_NEWT = 6.67430e-11
A_RAD = 7.565733e-16            # J m^-3 K^-4
T_CMB = 2.725
H0_FID = 67.4                   # km/s/Mpc   (the record's fiducial)
OM_M_FID = 0.315
OM_R = 9.24e-5
OM_L = 0.6847                   # the flow study's Omega_Lambda
H0_SI = H0_FID * 1e3 / MPC_M
U_CRIT = 3.0 * H0_SI ** 2 / (8 * math.pi * G_NEWT) * C_LIGHT ** 2
RHO_DE = OM_L * U_CRIT          # J/m^3, the record's target (5.2509e-10)


def P(s=""):
    print(s)


def dex(x):
    return math.log10(x)


# ---------------------------------------------------------------- the ledger
SFH = (0.015, 2.7, 2.9, 5.6)    # Madau & Dickinson 2014 eq. 15 (the record's A2)
Z_START = 20.0
N_GRID = 4000
lna = np.linspace(math.log(1e-6), 0.0, N_GRID)
a_g = np.exp(lna)
z_g = 1.0 / a_g - 1.0
PSI = np.where(z_g > Z_START, 0.0, SFH[0] * (1 + z_g) ** SFH[1] / (1.0 + ((1 + z_g) / SFH[2]) ** SFH[3]))


def cumtrapz0(y, x):
    out = np.zeros_like(y)
    out[1:] = np.cumsum(0.5 * (y[1:] + y[:-1]) * np.diff(x))
    return out


def solve(f, lam, om_m=OM_M_FID, variant="A", n_iter=120, tol=1e-12):
    """rho_DE(a)/rho_DE(1) = (1 - f) + f * rho_org(a)/rho_org(1).
    rho_org is the record's rent ledger dn/dtau = psi - lam n (variant A: not
    diluted; variant D: diluted as a^-3). f = 1 is the record's own model;
    f = 0 is a constant."""
    om_de = 1.0 - om_m - OM_R
    rho = np.ones_like(a_g)
    org = np.ones_like(a_g)
    for _ in range(n_iter):
        E = np.sqrt(OM_R * a_g ** -4 + om_m * a_g ** -3 + om_de * rho)
        tau = cumtrapz0(1.0 / E, lna)
        dec = np.exp(-lam * (tau - tau[-1]))
        if variant == "A":
            n = dec * cumtrapz0(PSI / dec, tau)
        else:
            n = a_g ** -3 * dec * cumtrapz0(PSI * a_g ** 3 / dec, tau)
        org = np.maximum(n / n[-1], 1e-300)
        new = (1.0 - f) + f * org
        m = a_g > 1e-3
        done = np.max(np.abs(np.log(new[m] / rho[m]))) < tol
        rho = new
        if done:
            break
    E = np.sqrt(OM_R * a_g ** -4 + om_m * a_g ** -3 + om_de * rho)
    tau = cumtrapz0(1.0 / E, lna)
    w_eff = -1.0 - np.gradient(np.log(rho), lna) / 3.0
    w_org = -1.0 - np.gradient(np.log(org), lna) / 3.0
    share = f * org / rho                     # the organised share of dark energy at each epoch
    return dict(E=E, tau=tau, rho=rho, org=org, w=w_eff, w_org=w_org, share=share, om_m=om_m, f=f, lam=lam)


def at_z(sol, key, zq):
    return float(np.interp(zq, z_g[::-1], np.asarray(sol[key])[::-1]))


def crossing(sol, key="w"):
    """the redshift at which w crosses -1, searched in 0 < z < 3; None if it does not."""
    m = (z_g > 1e-4) & (z_g < 3.0)
    zz, ww = z_g[m][::-1], (np.asarray(sol[key])[m] + 1.0)[::-1]
    s = np.sign(ww)
    idx = np.where(s[1:] * s[:-1] < 0)[0]
    if len(idx) == 0:
        return None
    i = idx[0]
    return float(zz[i] - ww[i] * (zz[i + 1] - zz[i]) / (ww[i + 1] - ww[i]))


def fit_cpl(sol, a_lo=0.3):
    """the record's projection: least squares of ln rho on the CPL form over
    a in [0.3, 1], weighted by dark energy's share of the total density."""
    m = (a_g >= a_lo) & (a_g <= 1.0)
    aa = a_g[m]
    y = np.log(sol["rho"][m]) + 3.0 * np.log(aa)
    X = np.vstack([-3.0 * np.log(aa), -3.0 * np.log(aa) - 3.0 * (1.0 - aa)]).T
    om_de = 1.0 - sol["om_m"] - OM_R
    E2 = OM_R * aa ** -4 + sol["om_m"] * aa ** -3 + om_de * sol["rho"][m]
    W = np.sqrt(om_de * sol["rho"][m] / E2)
    b, *_ = np.linalg.lstsq((X.T * W).T, y * W, rcond=None)
    return float(b[0]), float(b[1])


# ------------------------------------------------- DESI DR2, as on the record
BAO_DV = [(0.295, 7.942, 0.075)]
BAO_MH = [(0.510, 13.588, 0.167, 21.863, 0.425, -0.459), (0.706, 17.351, 0.177, 19.455, 0.330, -0.404),
          (0.934, 21.576, 0.152, 17.641, 0.193, -0.416), (1.321, 27.601, 0.318, 14.176, 0.221, -0.434),
          (1.484, 30.512, 0.760, 12.817, 0.516, -0.500), (2.330, 38.988, 0.531, 8.632, 0.101, -0.431)]
CMB_ZSTAR, CMB_DM, CMB_SIG = 1089.92, 94.31, 0.15


def _data():
    d, blocks, zs, kinds = [], [], [], []
    for z, v, s in BAO_DV:
        d.append(v); blocks.append(np.array([[s ** 2]])); zs.append(z); kinds.append("V")
    for z, mm, sm, h, sh, r in BAO_MH:
        d += [mm, h]; blocks.append(np.array([[sm ** 2, r * sm * sh], [r * sm * sh, sh ** 2]])); zs += [z, z]; kinds += ["M", "H"]
    n = len(d) + 1
    Ci = np.zeros((n, n)); i = 0
    for b in blocks:
        k = b.shape[0]; Ci[i:i + k, i:i + k] = np.linalg.inv(b); i += k
    Ci[-1, -1] = CMB_SIG ** -2
    return np.array(d + [CMB_DM]), Ci, zs, kinds


DATA, CINV, DZ, DK = _data()


def shape(sol):
    """the model's 14 observables up to the factor c/(H0 r_d)."""
    E = sol["E"]
    chi = cumtrapz0(1.0 / (a_g ** 2 * E), a_g)
    chi = chi[-1] - chi
    v = []
    for z, kind in zip(DZ, DK):
        aq = 1.0 / (1.0 + z)
        ch, Eq = np.interp(aq, a_g, chi), np.interp(aq, a_g, E)
        v.append(ch if kind == "M" else (1.0 / Eq if kind == "H" else (z * ch ** 2 / Eq) ** (1.0 / 3.0)))
    v.append(np.interp(1.0 / (1.0 + CMB_ZSTAR), a_g, chi))
    return np.array(v)


def chi2(sol, data=None):
    """chi^2 on the 13 BAO numbers and the acoustic point, profiled over c/(H0 r_d)."""
    d = DATA if data is None else data
    v = shape(sol)
    vCv, vCd = v @ CINV @ v, v @ CINV @ d
    return float(d @ CINV @ d - vCd ** 2 / vCv)


def golden(fn, lo, hi, tol=2e-4):
    g = (math.sqrt(5) - 1) / 2
    c, d = hi - g * (hi - lo), lo + g * (hi - lo)
    fc, fd = fn(c), fn(d)
    while hi - lo > tol:
        if fc < fd:
            hi, d, fd = d, c, fc
            c = hi - g * (hi - lo); fc = fn(c)
        else:
            lo, c, fc = c, d, fd
            d = lo + g * (hi - lo); fd = fn(d)
    x = 0.5 * (lo + hi)
    return x, fn(x)


def best_om(f, lam, data=None):
    om, c2 = golden(lambda om: chi2(solve(f, lam, om_m=om, n_iter=60, tol=1e-9), data), 0.22, 0.45, tol=4e-4)
    return om, c2


DESI = [("DESI+CMB", -0.42, 0.21, -1.75, 0.58), ("DESI+CMB+Pantheon+", -0.838, 0.055, -0.62, 0.205),
        ("DESI+CMB+Union3", -0.667, 0.088, -1.09, 0.29), ("DESI+CMB+DESY5", -0.752, 0.057, -0.86, 0.215)]
RHO_W = -0.9                     # the record's ASSUMED w0-wa correlation, not published per combination


def maha(w0, wa, row):
    _, w0d, s0, wad, sa = row
    C = np.array([[s0 ** 2, RHO_W * s0 * sa], [RHO_W * s0 * sa, sa ** 2]])
    d = np.array([w0 - w0d, wa - wad])
    return math.sqrt(float(d @ np.linalg.inv(C) @ d))


def main():
    P("=" * 96)
    P("COSMOS, TWO PORTIONS: rho_DE(a) = rho_static + rho_org(a).  Every number of COSMOS_TWO_PORTION.md.")
    P("=" * 96)

    # ------------------------------------------------------------------ 0
    P("\n0. GATE: this re-implementation reproduces the record's own numbers at f = 1")
    P("   (record: DE_LEDGER_MODEL.md sections 2, 3, 4)")
    ok = True
    for lam, rec in ((0.0, -1.088), (1.0, -0.821), (2.0, -0.587)):
        s = solve(1.0, lam)
        got = at_z(s, "w", 0.0)
        ok &= abs(got - rec) < 2e-3
        P(f"   w(z=0) at lam = {lam:.0f} H0: {got:+.3f}   record {rec:+.3f}")
    s1 = solve(1.0, 1.0)
    w0, wa = fit_cpl(s1)
    ok &= abs(w0 + 0.683) < 2e-3 and abs(wa + 1.366) < 4e-3
    P(f"   CPL projection at lam = 1 H0: ({w0:+.3f}, {wa:+.3f})   record (-0.683, -1.366)")
    zc = crossing(s1)
    ok &= abs(zc - 0.386) < 4e-3
    P(f"   crossing of w = -1 at lam = 1 H0: z = {zc:.3f}   record 0.386")
    om_l, c_l = golden(lambda om: chi2(solve(0.0, 1.0, om_m=om, n_iter=3)), 0.22, 0.45)
    om_r, c_r = best_om(1.0, 1.671)
    ok &= abs(c_l - 10.603) < 0.02 and abs(c_r - 8.472) < 0.03
    P(f"   chi2 LCDM (BAO + CMB-lite, N = 14): {c_l:.3f} at Om = {om_l:.4f}   record 10.603 at 0.2954")
    P(f"   chi2 ledger f = 1, lam = 1.671 H0:  {c_r:.3f} at Om = {om_r:.4f}   record 8.472 at 0.3378")
    P(f"   GATE {'PASSES' if ok else 'FAILS'}")
    if not ok:
        raise SystemExit("the gate fails: nothing below may be read")

    # ------------------------------------------------------------------ 1
    P("\n1. THE MIXING IDENTITY (exact, any growth law)")
    P("   1 + w_eff(a) = share(a) * (1 + w_org(a)),   share(a) = rho_org(a) / rho_DE(a)")
    P("   so a static portion changes the SIZE of the departure from -1 and never its sign,")
    P("   and the redshift at which w crosses -1 does not move.")
    P("   check on the ledger at lam = 1.671 H0:")
    P("       f     w_eff(0)    1+w_eff(0)   share(0)*(1+w_org(0))   crossing z   max |identity residual|, a > 0.2")
    for f in (1.0, 0.5, 0.1, 1e-2, 1e-4):
        s = solve(f, 1.671)
        m = a_g > 0.2
        res = float(np.max(np.abs((1 + s["w"][m]) - s["share"][m] * (1 + s["w_org"][m]))))
        zc = crossing(s)
        P(f"   {f:7.0e}   {at_z(s,'w',0):+.6f}   {1+at_z(s,'w',0):+.3e}      {at_z(s,'share',0)*(1+at_z(s,'w_org',0)):+.3e}"
          f"          {zc:.3f}        {res:.1e}")

    # ------------------------------------------------------------------ 2
    P("\n2. THE GROWTH LAWS THE RECORD ADMITS, each alone (f = 1): the sign of 1 + w")
    P("   law                                                   w_org(z=0)   w_org(z=1)   crossing   source")
    rows = []
    s = solve(1.0, 0.0); rows.append(("L1 stock, held for ever (lam = 0), not diluted", at_z(s, "w", 0), at_z(s, "w", 1), crossing(s), "DE_LEDGER_MODEL.md sec 2"))
    s = solve(1.0, 1.671); rows.append(("L2 rent ledger, lam = 1.671 H0 (the DR2 fit), not diluted", at_z(s, "w", 0), at_z(s, "w", 1), crossing(s), "DE_LEDGER_MODEL.md sec 4"))
    s = solve(1.0, 1.0, variant="D"); rows.append(("L3 rent ledger, lam = 1 H0, DILUTED as a^-3 (A5 dropped)", at_z(s, "w", 0), at_z(s, "w", 1), crossing(s), "DE_LEDGER_MODEL.md sec 5, variant D"))
    for nm, a, b, c, src in rows:
        P(f"   {nm:55s} {a:+.3f}       {b:+.3f}       {('z = %.3f' % c) if c else 'none':9s}  {src}")
    P("   L4 Gough's HDIE: rho ~ a^1.8 for z > 1, a^-0.02 for z < 1    -0.993       -1.600       the break  DE_LEDGER_MODEL.md sec 2 (his CPL form: w0 = -1, wa = -0.6)")
    P(f"      (from the exponents: w = -1 + 0.02/3 = {-1+0.02/3:+.3f} late, -1 - 1.8/3 = {-1-1.8/3:+.3f} early)")
    P("   L5 per-unit balance (dark-balance-intensive), as computed     -0.979       -1.03..      z ~ 0.6    preferred-frame-results.md table (z = 0.02 row)")
    P("   L6 grand total (dark-balance-extensive), as computed          -0.931       -1.1..       z ~ 0.6    preferred-frame-results.md table (z = 0.02 row)")
    P("   L7 any thermal system priced at its OWN temperature: N kB T ln2 = T S, its own heat; dilutes as matter or radiation, w >= 0 (section 5)")

    # ------------------------------------------------------------------ 3
    P("\n3. WHAT SIZE OF ORGANISED PORTION THE DESI DR2 SUMMARY POINTS WOULD NEED (post hoc: DR2 was read first)")
    P("   3a. the ledger with a static portion, projected on (w0, wa) as the record projects it")
    fs = [1.0, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.05]
    lams = [round(0.25 * k, 2) for k in range(0, 17)]
    grid = {}
    for f in fs:
        for lam in lams:
            s = solve(f, lam, n_iter=60, tol=1e-10)
            grid[(f, lam)] = fit_cpl(s)
    P("       lam/H0 |  f = 1.0            f = 0.5            f = 0.2            ratio wa/(1+w0) at f = 1.0, 0.5, 0.2")
    for lam in (0.5, 1.0, 1.25, 1.5, 1.75, 2.0, 2.5, 3.0, 4.0):
        c = [grid[(f, lam)] for f in (1.0, 0.5, 0.2)]
        P(f"       {lam:5.2f}  | " + "   ".join(f"({a:+.3f}, {b:+.3f})" for a, b in c) + "   " + "  ".join(f"{b/(1+a):+.2f}" for a, b in c))
    P("   3b. nearest point of the two-portion family to each DR2 combination (the record's distance: correlation -0.9 ASSUMED)")
    P("       dataset               (w0, wa)          f = 1 only: lam, distance     with a static portion: f, lam, (w0, wa), distance")
    need = []
    for row in DESI:
        b1 = min(((maha(*grid[(1.0, lam)], row), lam) for lam in lams))
        b2 = min(((maha(*grid[(f, lam)], row), f, lam) for f in fs for lam in lams))
        w = grid[(b2[1], b2[2])]
        need.append(b2[1])
        P(f"       {row[0]:20s} ({row[1]:+.3f}, {row[3]:+.3f})   lam = {b1[1]:.2f}, {b1[0]:.1f} sigma          f = {b2[1]:.2f}, lam = {b2[2]:.2f}, ({w[0]:+.3f}, {w[1]:+.3f}), {b2[0]:.1f} sigma")
    P(f"       organised share the four points ask for: f = {min(need):.2f} to {max(need):.2f}  ({dex(min(need)):+.2f} to {dex(max(need)):+.2f} dex)")
    P("   3c. the same question on the record's likelihood (13 DR2 BAO numbers + the acoustic point), Om re-minimised")
    P("        f      best lam/H0    Om       chi2     chi2 - chi2(LCDM)   crossing z   1+w_eff(0)")
    prof = []
    for f in (1.0, 0.8, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1, 0.05):
        lam, _ = golden(lambda L: best_om(f, L)[1], 0.6, 5.0, tol=0.02)
        om, c2 = best_om(f, lam)
        prof.append((c2, f, lam, om))
        sb = solve(f, lam, om_m=om)
        P(f"       {f:4.2f}     {lam:6.2f}       {om:.4f}   {c2:7.3f}    {c2 - c_l:+.3f}             {crossing(sb):.3f}        {1+at_z(sb,'w',0):+.3f}")
    P(f"       0.00        any        {om_l:.4f}   {c_l:7.3f}    +0.000   (a constant)")
    cb, fb, lb, ob = min(prof)
    P(f"       best on this grid: f = {fb:.2f}, lam = {lb:.2f} H0, chi2 = {cb:.3f}; against f = 1: {cb - c_r:+.3f} for one more parameter;"
      f" against LCDM: {cb - c_l:+.3f} for two more")
    P(f"       spread of chi2 over f in [0.3, 1] at the best lam of each: {max(p[0] for p in prof if p[1] >= 0.3) - min(p[0] for p in prof if p[1] >= 0.3):.3f}"
      "   (BAO + CMB-lite does not fix f)")
    P("   3c'. PLANTS: can this instrument tell two portions from one, or from none? Noise-free data made from a planted model,")
    P("        DR2's own covariances, then the best fit of each rival (chi2 of the truth is 0; a rival's chi2 is what the data could tell)")
    P("        planted truth                         best one-portion ledger (f = 1)      best constant (f = 0)")
    for fp, lp, op in ((0.5, 1.93, 0.3222), (0.2, 2.64, 0.3119), (1.06e-4, 1.671, 0.2954)):   # the last: the largest Landauer share of section 4
        truth = 29.5 * shape(solve(fp, lp, om_m=op))
        lam1, _ = golden(lambda L: best_om(1.0, L, truth)[1], 0.3, 5.0, tol=0.02)
        c1 = best_om(1.0, lam1, truth)[1]
        c0 = golden(lambda om: chi2(solve(0.0, 1.0, om_m=om, n_iter=3), truth), 0.22, 0.45)[1]
        P(f"        f = {fp:8.1e}, lam = {lp:.2f}, Om = {op:.4f}     chi2 = {c1:8.3f} at lam = {lam1:.2f}          chi2 = {c0:.3e}")
    P("        a rival is told apart at two standard errors when its chi2 reaches 4; at three, 9")
    P("   3d. the dark-balance branches: a static portion moves them AWAY from the DR2 points")
    for nm, w0b in (("per-unit (intensive)", -0.979), ("grand total (extensive)", -0.931)):
        dev = 1 + w0b
        P(f"       {nm:24s} 1+w(0) at f = 1: {dev:.3f};  f needed for 1+w0 = 0.162 (Pantheon+), 0.248 (DESY5), 0.333 (Union3): "
          f"{0.162/dev:.1f}, {0.248/dev:.1f}, {0.333/dev:.1f}   (all above 1: not available)")
    P("   3e. Gough's form with a static portion (his exponents as the record quotes them, projected by the record's method)")
    P(f"       1 + w_eff(0) = f x 0.02/3 = f x {0.02/3:.4f}: at most {0.02/3:.4f}, for any f")
    for f in (1.0, 0.5, 0.1):
        org = np.where(a_g < 0.5, (a_g / 0.5) ** 1.8 * 0.5 ** -0.02, a_g ** -0.02)
        sol = dict(rho=(1 - f) + f * org, om_m=OM_M_FID)
        w0, wa = fit_cpl(sol)
        P(f"       f = {f:.1f}: CPL projection ({w0:+.3f}, {wa:+.3f})")

    # ------------------------------------------------------------------ 4
    P("\n4. WHAT SIZE THE LANDAUER BILL IS (the record's stock table, in densities: FLOW_PRICING_RESULTS.md section 2)")
    P(f"   target rho_DE = {RHO_DE:.4e} J/m^3 (Omega_L = {OM_L}, H0 = {H0_FID})")
    stock = [("stellar baryons, 1 bit each, stellar interior T", 1.38e-2, 1e7), ("stellar baryons, 1 bit each, stellar surface T", 1.38e-2, 5.8e3),
             ("all baryons, 1 bit each, warm IGM T", 2.52e-1, 1e6), ("stars, E&L entropy, stellar interior T", 9.99, 1e7),
             ("IGM gas, Sackur-Tetrode, warm", 57.7, 1e6), ("IGM gas, Sackur-Tetrode, hot", 60.2, 1e7), ("IGM gas, E&L entropy, hot", 74.6, 1e7),
             ("starlight photons at starlight T", 1.05e6, 5e3), ("dust photons at dust T", 1.05e6, 30.0), ("CMB photons at CMB T", 2.13e9, 2.73)]
    P("   pairing (count and temperature of the SAME degrees of freedom)   N [bits/m^3]   T [K]    N kB T ln2 [J/m^3]   f = share of rho_DE   dex")
    fl = []
    for nm, N, T in stock:
        e = N * KB * T * LN2
        fl.append(e / RHO_DE)
        P(f"   {nm:60s} {N:10.3e}  {T:8.2e}   {e:10.3e}          {e/RHO_DE:9.2e}      {dex(e/RHO_DE):+.1f}")
    P(f"   Landauer share of dark energy: f = {min(fl):.1e} to {max(fl):.1e}   ({dex(min(fl)):+.1f} to {dex(max(fl)):+.1f} dex)")
    matter = [x for (nm, N, T), x in zip(stock, fl) if "photons" not in nm]
    P(f"   matter pairings only (photons set aside, see section 5): f = {min(matter):.1e} to {max(matter):.1e}   ({dex(min(matter)):+.1f} to {dex(max(matter)):+.1f} dex)")
    P("   the older table in totals (DE_LEDGER_MODEL.md K4): 3.1 to 5.3 dex short, and 1.1 dex more once its volume is corrected: "
      f"{3.1+1.1:.1f} to {5.3+1.1:.1f} dex")
    P(f"   REQUIRED (section 3b) against AVAILABLE: {dex(min(need)):+.2f}..{dex(max(need)):+.2f} dex against {dex(max(fl)):+.1f}..{dex(min(fl)):+.1f} dex:"
      f" short by {dex(min(need)) - dex(max(fl)):.1f} to {dex(max(need)) - dex(min(fl)):.1f} dex")
    P(f"   what the split buys the dead leg: the stock no longer has to be 1.0 of dark energy but {min(need):.2f} to {max(need):.2f} of it:"
      f" {-dex(max(need)):.2f} to {-dex(min(need)):.2f} dex of a {-dex(max(fl)):.1f} to {-dex(min(fl)):.1f} dex shortfall")

    # ------------------------------------------------------------------ 5
    P("\n5. THE LARGEST PAIRINGS ARE ENERGY ALREADY COUNTED (an identity, not a new component)")
    u_cmb = A_RAD * T_CMB ** 4
    s_cmb_bits = (4.0 / 3.0) * A_RAD * T_CMB ** 3 / KB / LN2
    P(f"   CMB: N = S/(kB ln2) = {s_cmb_bits:.3e} bits/m^3; N kB T ln2 = T S = {s_cmb_bits*KB*T_CMB*LN2:.3e} J/m^3 = {s_cmb_bits*KB*T_CMB*LN2/u_cmb:.4f} x u_CMB"
      f"   (u_CMB = a T^4 = {u_cmb:.3e} J/m^3)")
    P("   For any thermal system priced at its own temperature the Landauer stock is T S, of the order of its own heat. That heat is already")
    P(f"   in the census as radiation or matter (Omega_r = {OM_R:.2e}: the CMB row above is 4/3 of the photon part of it), it dilutes, and its w is >= 0.")
    P("   The record found the same thing for the horizon row (T_dS S / V = rho_crit c^2 exactly). So the organised portion at the Landauer size")
    P("   is not a new, non-diluting component unless A5 (non-dilution) is added, and the record marks A5 IMPORTED, not derived.")

    # ------------------------------------------------------------------ 6
    P("\n6. WHAT THE LANDAUER-SIZE PORTION DOES TO w (the mixing identity with section 4's f)")
    s = solve(1.0, 1.671)
    d_dec = 1 + at_z(s, "w", 0.0)
    s0 = solve(1.0, 0.0)
    d_hold = 1 + at_z(s0, "w", 0.0)
    P(f"   1 + w_org(0): {d_dec:+.3f} for the ledger at lam = 1.671 H0;  {d_hold:+.3f} for a stock held for ever")
    for nm, f in (("largest pairing", max(fl)), ("largest matter pairing", max(matter)), ("smallest pairing", min(fl))):
        P(f"   {nm:24s} f = {f:.1e}:  1 + w_eff(0) = {f*d_dec:+.1e} (decaying)  or  {f*d_hold:+.1e} (held)")
    P("   reference sensitivities (the record's DR2 table): sigma(w0) = 0.055 to 0.21, sigma(wa) = 0.2 to 0.58")
    P(f"   the largest of these departures is {0.055/(max(fl)*d_dec):.0f} times smaller than the best sigma(w0) on the record")
    P("   textbook, from memory and not checked here: next-generation surveys aim at sigma(w0) of about 0.01 to 0.02 -- still "
      f"{0.01/(max(fl)*d_dec):.0f} times too coarse")

    # ------------------------------------------------------------------ 7
    P("\n7. WHO PAYS: the writer's budget and the waste heat, scaled from the record's flow study (FLOW_PRICING_RESULTS.md section 4)")
    lam = 1.671
    src = lam - 3.0 * d_dec          # psi0/n0 in units of H0, from w_org(0) = -1 + (lam - psi0/n0)/3
    P(f"   ledger at lam = {lam} H0: the source term today is psi0/n0 = lam - 3(1+w_org(0)) = {src:.3f} H0  (writing {src:.3f} of the stock per Hubble time)")
    p_need = src * H0_SI * RHO_DE     # W/m^3 for f = 1
    P(f"   writing power needed for an organised portion of share f: {p_need:.3e} x f  W/m^3")
    for nm, cap in (("every gram of new stars turned wholly to energy", 2.885e-30), ("every gram of new stars burned H -> He (0.7 %)", 2.019e-32)):
        P(f"   the record's cap, {nm:48s} {cap:.3e} W/m^3  =>  f <= {cap/p_need:.1e}  ({dex(cap/p_need):+.1f} dex)")
    P("   (this cap binds only if the writer pays for what it writes, which is the wager's own sentence: luminous matter is the writer)")
    y_full = (3.0 * RHO_DE) / (4.0 * u_cmb)
    P(f"   waste heat: the record's flow figure, 3 rho_DE per Hubble time into the CMB, gives y = {y_full:.2e} against FIRAS 1.5e-5 ({dex(y_full/1.5e-5):.1f} dex)")
    P(f"   a decaying organised portion sheds lam f rho_DE per Hubble time: y = {y_full:.2e} x f x lam/(3 H0)")
    f_firas = 1.5e-5 / (y_full * lam / 3.0)
    P(f"   IF that heat lands in the CMB, FIRAS allows f <= {f_firas:.1e} ({dex(f_firas):+.1f} dex) at lam = {lam} H0")
    P(f"   against the DR2-size portion (f = {min(need):.2f} to {max(need):.2f}): over by {dex(min(need)/f_firas):.1f} to {dex(max(need)/f_firas):.1f} dex")
    P(f"   against the Landauer sizes ({dex(min(fl)):+.1f} to {dex(max(fl)):+.1f} dex): from {dex(min(fl)/f_firas):+.1f} to {dex(max(fl)/f_firas):+.1f} dex relative to the limit")
    P("   IF instead the decayed record returns to the static portion, the sum only ever gains: d(rho_static + rho_org)/dt = source >= 0,")
    P("   so 1 + w_eff <= 0 at every epoch (phantom or constant, never above -1), which is the opposite of DR2's sign today.")
    P(f"   IF it goes to matter, it adds lam f Omega_DE = {lam*min(need)*(1-OM_M_FID-OM_R):.2f} to {lam*max(need)*(1-OM_M_FID-OM_R):.2f} of the critical density per Hubble time"
      f" (Omega_m = {OM_M_FID}); not priced here.")

    # ------------------------------------------------------------------ 8
    P("\n8. WHAT THE TWO-PORTION LEDGER FORBIDS (the wedge), for a stake")
    r_all = {lam: grid[(0.05, lam)][1] / (1 + grid[(0.05, lam)][0]) for lam in lams if lam >= 0.5}
    P("   ratio wa/(1+w0) of the family as f -> small (lam/H0: ratio): " + ", ".join(f"{k:.2f}: {v:+.2f}" for k, v in r_all.items() if k in (0.5, 1.0, 1.5, 2.0, 3.0, 4.0)))
    lo = max(r_all.values())
    P(f"   over lam in [0.5, 4] H0 and every f the family keeps w0 > -1, wa < 0 and wa/(1+w0) <= {lo:+.2f}")
    P("   the four DR2 points' own ratios: " + ", ".join(f"{r[0]}: {r[3]/(1+r[1]):+.2f}" for r in DESI))
    P("   the DR2 68 % interval of lam at f = 1 on the record: [1.33, 2.02] H0; ratios there at f = 0.5: "
      f"{grid[(0.5,1.25)][1]/(1+grid[(0.5,1.25)][0]):+.2f} (lam 1.25) to {grid[(0.5,2.0)][1]/(1+grid[(0.5,2.0)][0]):+.2f} (lam 2.0)")
    for lam in (1.33, 1.671, 2.02):
        P(f"   crossing of w = -1 at lam = {lam} H0: z = {crossing(solve(1.0, lam)):.3f} at f = 1, {crossing(solve(0.5, lam)):.3f} at f = 0.5, {crossing(solve(0.1, lam)):.3f} at f = 0.1")
    P("   the record's own CPL refit of this data has corr(w0, wa) = -0.99 with sigma 0.314 and 0.996: a degeneracy of slope "
      f"{-0.996/0.314:.1f}, and the line from (-1, 0) to its best fit (-0.508, -1.432) has slope {-1.432/(1-0.508):.2f}")
    P("   so moving f moves the family ALONG the direction the data constrain least.")

    # ------------------------------------------------------------------ 9
    P("\n9. IN THE DECLARATION STANDARD'S TERMS (holonc.py: carried iff price - 2 se > the consumer's budget)")
    budget = 0.055
    P(f"   consumer: a reader of the expansion history; budget {budget} in 1 + w0 (the best sigma(w0) on the record, DESI+CMB+Pantheon+)")
    P(f"   price of the organised portion at the largest Landauer size: {max(fl)*d_dec:.1e}  ->  reads DROPPED, by {dex(budget/(max(fl)*d_dec)):.1f} dex")
    P(f"   price at the DR2-asked sizes f = {min(need):.2f} to {max(need):.2f}: {min(need)*d_dec:.3f} to {max(need)*d_dec:.3f}, with no standard error on the record for f"
      " (section 3c: the data do not fix f)")
    for nm, w0d, s0, wad, sa in DESI:
        lower = (1 + w0d) - 2 * s0
        P(f"   read from the data instead, {nm:20s} 1 + w0 = {1+w0d:.3f} +- {s0:.3f}: less two standard errors {lower:+.3f} against {budget}: reads {'CARRIED' if lower > budget else 'DROPPED'}")
    # ----------------------------------------------------------------- 10
    P("\n10. THE RECORD'S NUMBERS THE NOTE QUOTES AND THIS SCRIPT DOES NOT RECOMPUTE")
    P("   general two-parameter (CPL) fit of the same 14 numbers: chi2 7.160, -3.44 against a constant   (DE_LEDGER_MODEL.md section 4)")
    P("   w(0) = -0.662 for the ledger at its fitted Om = 0.3378                                         (DE_LEDGER_MODEL.md section 4)")
    P("   the grand-total reading's frozen crossing: z = 0.59 +- 0.03                                     (Stance.lean, dark-balance-extensive)")
    P("   Gough's own CPL form: w0 = -1, wa = -0.6                                                        (DE_LEDGER_MODEL.md section 2)")
    P("   K4 in totals: 3.1 to 5.3 dex short; the volume correction: 1.1 dex                              (DE_LEDGER_MODEL.md section 6; FLOW_PRICING_RESULTS.md section 2.3)")
    P("   PROPOSED blanks of the stake (section 8 of the note; from DR2 or the record's stub, not bars): 3 sigma, 2 sigma, +2, 4, z in [0.45, 0.75]")
    P("\ndone.")


if __name__ == "__main__":
    main()
