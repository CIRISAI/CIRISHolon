#!/usr/bin/env python3
"""COSMOS-BASE: the organised share of dark energy fixed as a ratio of a dynamics to its base.

Companion to COSMOS_BASE.md. Every number that note quotes is printed here.
numpy only, single thread, no data fetched. It IMPORTS from cosmos_two_portion.py
(same folder): the grid (a_g, z_g, lna), the star-formation history PSI, solve(),
chi2(), best_om(), fit_cpl(), crossing(), maha(), the DESI DR2 vector (14 numbers) and
the four published (w0, wa) points, and the fiducial constants. Nothing is tuned here:
every share is a ratio of published densities, written down and hashed BEFORE any fit.

NOT BLIND: see the note's disclosure. The enumeration is post hoc with respect to DR2.
"""
import hashlib
import math
import os
import sys

os.environ.setdefault("OMP_NUM_THREADS", "1")
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cosmos_two_portion as C  # noqa: E402

P = print

# ---------------------------------------------------------------- inventory (published)
# Planck 2018 base-LCDM, TT,TE,EE+lowE+lensing (arXiv 1807.06209, Table 2):
OBH2, S_OBH2 = 0.02237, 0.00015
OCH2, S_OCH2 = 0.1200, 0.0012
H, S_H = 0.6736, 0.0054
OM_M, OM_L = 0.3153, 0.6847
OB = OBH2 / H ** 2
ODM = OCH2 / H ** 2
# Stars: two published inventories, both FROM MEMORY (not re-read at source):
#   Fukugita & Peebles 2004 (ApJ 616, 643): stars + remnants ~ 0.0027 +- 0.0005 (h = 0.7)
#   Madau & Dickinson 2014 (ARA&A 52, 415): rho_* ~ 5.8e8 Msun/Mpc^3 with return fraction 0.27
RHOCRIT_H2 = 2.775e11  # Msun / Mpc^3 / h^2
OSTAR_FP, S_OSTAR_FP = 0.0027, 0.0005
OSTAR_MD = 5.8e8 / (RHOCRIT_H2 * H ** 2)

# ---------------------------------------------------------------- 1. the enumeration
ENUM = """ENUMERATION (written before any fit; hashed)
dynamics-over-base ratios r, each from the wager's roles (DM capacity, LM writer, DE record):
 r1  all baryons over dark matter            Omega_b / Omega_dm     'DM is the base, light matter the dynamics on top'
 r2  all baryons over all matter             Omega_b / Omega_m      'DM+LM is the base'; the writer's part of it
 r3  stars over dark matter                  Omega_* / Omega_dm     only the WRITERS count as dynamics
 r4  stars over all matter                   Omega_* / Omega_m      the same, on the joint base
 r5  stars over baryons                      Omega_* / Omega_b      'the dynamics on top are by tier': writers among light matter
 r6  all matter over dark energy             Omega_m / Omega_DE     the base against the record
 r7  dark matter over dark energy            Omega_dm / Omega_DE    the capacity against the record
 r8  baryons over dark energy                Omega_b / Omega_DE     the writer against the record
 r9  stars over dark energy                  Omega_* / Omega_DE     the writers against the record
what the ratio is a ratio OF (a fork):
 T1  r = rho_org / rho_DE           the organised share itself:            f = r
 T2  r = rho_org / rho_static       organised over static:                 f = r / (1 + r)
 T3  r = rho_org / rho_base         organised over its base (r1..r5 only): f = r * Omega_base / Omega_DE
when the ratio holds (a fork):
 LA  at all times, the organised portion diluting with a matter-like base: rho_org ~ a^-3, w_org = 0
 LS  at all times as organised/static with matter-like dynamics and base:  rho_org constant, w_org = -1 (a constant)
 L2  today only; the growth law is the record's rent ledger, lam = 1.671 H0 (not diluted)
 LD  today only; the record is what stars have written, diluting as matter, no decay (lam = 0)
 L1  today only; the record is what stars have written, held for ever, not diluted (lam = 0)
forks: 9 ratios x (T1, T2) + 5 ratios x T3 = 23 shares, x 5 laws = 115 readings.
The stance already names this choice: 'dark-balance-intensive' is the per-unit (relative to
the base) reading and 'dark-balance-extensive' the grand total; they are staked there as RIVALS.
"""


def ratios():
    r = {}
    r["r1"] = ("Omega_b/Omega_dm", OB / ODM, (OB / ODM) * math.hypot(S_OBH2 / OBH2, S_OCH2 / OCH2), "dm")
    r["r2"] = ("Omega_b/Omega_m", OB / OM_M, (OB / OM_M) * math.hypot(S_OBH2 / OBH2, 0.0073 / OM_M), "m")
    for k, base, ob in (("r3", "dm", ODM), ("r4", "m", OM_M), ("r5", "b", OB)):
        lo, hi = OSTAR_FP / ob, OSTAR_MD / ob
        r[k] = ("Omega_*/Omega_" + base, 0.5 * (lo + hi), 0.5 * (hi - lo), base)
    r["r6"] = ("Omega_m/Omega_DE", OM_M / OM_L, (OM_M / OM_L) * 0.0073 / OM_M * (1 + OM_M / OM_L), None)
    r["r7"] = ("Omega_dm/Omega_DE", ODM / OM_L, (ODM / OM_L) * 0.03, None)
    r["r8"] = ("Omega_b/Omega_DE", OB / OM_L, (OB / OM_L) * 0.03, None)
    r["r9"] = ("Omega_*/Omega_DE", 0.5 * (OSTAR_FP + OSTAR_MD) / OM_L, 0.5 * (OSTAR_MD - OSTAR_FP) / OM_L, None)
    return r


OBASE = {"dm": ODM, "m": OM_M, "b": OB}


def shares(r):
    out = []
    for k, (name, val, err, base) in r.items():
        out.append((k + ".T1", name + " as the share", val))
        out.append((k + ".T2", name + " as organised/static", val / (1 + val)))
        if base:
            out.append((k + ".T3", name + " as organised/base", val * OBASE[base] / OM_L))
    return out


# ---------------------------------------------------------------- laws
def sol_law(law, f, om):
    if law == "L2":
        return C.solve(f, 1.671, om_m=om, variant="A", n_iter=60, tol=1e-9)
    if law == "LD":
        return C.solve(f, 0.0, om_m=om, variant="D", n_iter=60, tol=1e-9)
    if law == "L1":
        return C.solve(f, 0.0, om_m=om, variant="A", n_iter=60, tol=1e-9)
    raise ValueError(law)


def best(law, f):
    om, c2 = C.golden(lambda o: C.chi2(sol_law(law, f, o)), 0.22, 0.45, tol=4e-4)
    return om, c2


def growth(om_cl, om_sm, om_de_fn, b_org=0.0, om_org_fn=None):
    """linear growth D(a=1)/D_LCDM-normalised later. delta'' + (2 + dlnH/dlna) delta' = 1.5 * source * delta,
    source = [Om_cl a^-3 + b_org * rho_org(a)] / E^2. om_sm: smooth matter-like density (a^-3) in units of crit.
    om_de_fn(a): the dark-energy density / crit today (total, incl. any organised part not listed in om_sm)."""
    lna = np.linspace(math.log(1e-3), 0.0, 6000)
    a = np.exp(lna)
    E2 = C.OM_R * a ** -4 + (om_cl + om_sm) * a ** -3 + om_de_fn(a)
    dlnH = np.gradient(0.5 * np.log(E2), lna)
    org = om_org_fn(a) if om_org_fn is not None else 0.0
    src = 1.5 * (om_cl * a ** -3 + b_org * org) / E2
    d, v = a[0], a[0]
    h = lna[1] - lna[0]
    D = np.empty_like(a)
    for i in range(len(a)):
        D[i] = d
        if i + 1 == len(a):
            break
        # midpoint (RK2) with coefficients interpolated linearly
        k1d, k1v = v, -(2 + dlnH[i]) * v + src[i] * d
        dm, vm = d + 0.5 * h * k1d, v + 0.5 * h * k1v
        cm = 0.5 * (dlnH[i] + dlnH[i + 1])
        sm = 0.5 * (src[i] + src[i + 1])
        d, v = d + h * vm, v + h * (-(2 + cm) * vm + sm * dm)
    return a, D


def main():
    # ---- 1. enumeration and its hash, BEFORE anything is fitted
    P(ENUM)
    P("sha256 of the enumeration text: " + hashlib.sha256(ENUM.encode()).hexdigest())
    P()
    r = ratios()
    P("2. THE RATIOS (Planck 2018 base-LCDM: Obh2 %.5f, Och2 %.4f, h %.4f, Om %.4f, OL %.4f)" % (OBH2, OCH2, H, OM_M, OM_L))
    P("   Omega_b = %.5f, Omega_dm = %.5f; Omega_* = %.4f (Fukugita & Peebles, memory) to %.4f (Madau & Dickinson, memory)" % (OB, ODM, OSTAR_FP, OSTAR_MD))
    for k, (name, val, err, base) in r.items():
        P("   %s  %-20s = %.4f +- %.4f" % (k, name, val, err))
    sh = shares(r)
    P("   the %d shares f of dark energy these give:" % len(sh))
    for k, name, f in sh:
        P("     %-6s f = %.5f   (%s)" % (k, f, name))
    P()

    # ---- 2. the two all-time laws, by algebra
    P("3. THE ALL-TIME LAWS ARE RELABELLINGS (algebra, checked numerically)")
    P("   LA: rho_org ~ a^-3 gives E^2 = (Om + f*ODE) a^-3 + (1-f) ODE + rad: LCDM with Om' = Om + f*ODE, at every f.")
    P("   LS: organised/static fixed with both parts matter-like in ratio: rho_org is constant: a constant, at every f.")
    base_sol = C.solve(0.0, 1.671, om_m=OM_M, n_iter=5)
    c2_fid = C.chi2(base_sol)
    om0, c2_const = C.golden(lambda o: C.chi2(C.solve(0.0, 1.0, om_m=o, n_iter=5)), 0.22, 0.45, tol=4e-4)
    P("   a constant, Om free: chi2 = %.3f at Om = %.4f   (at Om = %.4f fixed: %.3f)" % (c2_const, om0, OM_M, c2_fid))
    for f in (r["r1"][1], r["r6"][1]):
        # LA with the TRUE matter density fixed at Planck's: the background's effective matter density is Om + f*ODE
        ome = OM_M + f * OM_L
        c2 = C.chi2(C.solve(0.0, 1.0, om_m=ome, n_iter=5))
        P("   LA at f = %.4f with the matter density held at %.4f: effective Om' = %.4f, chi2 = %.1f  (%.1f above the constant)" % (f, OM_M, ome, c2, c2 - c2_const))
    P("   LA with Om free: chi2 = %.3f at every f (the fit returns Om' = %.4f and cannot see f)." % (c2_const, om0))
    P("   LA under 1 + w = share x (1 + w_org) with w_org = 0: 1 + w = share(a) > 0 at every epoch: never phantom, no crossing.")
    for f in (r["r1"][1], r["r8"][1]):
        P("     f = %.4f: w(0) = %.3f;  w(z=1) = %.3f;  w(z=2) = %.3f" % (
            f, -1 + f, -1 + f * 8 / (1 - f + 8 * f), -1 + f * 27 / (1 - f + 27 * f)))
    P()

    # ---- 3. today-only laws at the forced shares
    P("4. THE FIT AT EACH FORCED SHARE (14 DR2 numbers; Om free; no share is free)")
    laws = ("L2", "LD", "L1")
    grid = np.concatenate([[0.0], np.geomspace(1e-4, 1.0, 33)])
    prof = {}
    for law in laws:
        prof[law] = np.array([best(law, f)[1] for f in grid])
        i = int(np.argmin(prof[law]))
        P("   law %s: free-share best chi2 = %.3f at f = %.3f (%.3f against the constant); f = 1: %.3f" % (
            law, prof[law][i], grid[i], prof[law][i] - c2_const, prof[law][-1]))
        ok = grid[prof[law] <= prof[law][i] + 1.0]
        P("          window (within 1 of its best): f from %.4f to %.4f" % (ok.min(), ok.max()))
        better = grid[prof[law] <= c2_const - 1.0]
        P("          window (at least 1 BETTER than a constant): %s" % ("f from %.3f to %.3f" % (better.min(), better.max()) if len(better) else "empty"))
    P("   the general two-parameter form on the same numbers: chi2 7.160 (the sibling record; not recomputed here)")
    P()
    P("   %-7s %-9s | %s" % ("share", "f", "  ".join("%-34s" % (l + ": chi2 (vs const) w0 wa zc") for l in laws)))
    table = {}
    for k, name, f in sh:
        row = []
        for law in laws:
            om, c2 = best(law, f)
            s = sol_law(law, f, om)
            w0, wa = C.fit_cpl(s)
            zc = C.crossing(s)
            table[(k, law)] = (f, c2, w0, wa, zc, om)
            row.append("%7.3f (%+6.3f) %+.3f %+.3f %s" % (c2, c2 - c2_const, w0, wa, ("%.2f" % zc) if zc else " -- "))
        P("   %-7s %-9.5f | %s" % (k, f, "  ".join("%-34s" % x for x in row)))
    P()
    P("   against the four published DR2 (w0, wa) points (standard errors, assumed correlation %.1f), law L2 and law LD:" % C.RHO_W)
    for k in ("r1.T1", "r1.T2", "r2.T1", "r5.T1", "r6.T1", "r6.T2", "r7.T1", "r8.T1", "r3.T1"):
        for law in ("L2", "LD"):
            f, c2, w0, wa, zc, om = table[(k, law)]
            P("     %-6s %s f = %.4f: " % (k, law, f) + ", ".join("%s %.1f" % (row[0].replace("DESI+CMB", "D+C"), C.maha(w0, wa, row)) for row in C.DESI))
    P("     a constant (w0 = -1, wa = 0): " + ", ".join("%s %.1f" % (row[0].replace("DESI+CMB", "D+C"), C.maha(-1.0, 0.0, row)) for row in C.DESI))
    P()

    # ---- 4. look-elsewhere
    P("5. LOOK-ELSEWHERE: how many of the enumerated shares land in a window, against chance")
    rng = np.random.default_rng(20261123)
    fvals = np.array([f for _, _, f in sh])
    for law in laws:
        i = int(np.argmin(prof[law]))
        for label, mask in (("within 1 of the free-share best", prof[law] <= prof[law][i] + 1.0),
                            ("at least 1 better than a constant", prof[law] <= c2_const - 1.0)):
            if not mask.any():
                P("   %s, %s: window empty; hits 0 of %d" % (law, label, len(fvals)))
                continue
            lo, hi = grid[mask].min(), grid[mask].max()
            hits = int(((fvals >= lo) & (fvals <= hi)).sum())
            p_log = max(0.0, (math.log10(hi) - math.log10(max(lo, 1e-4))) / 4.0)
            p_uni = (hi - max(lo, 1e-4)) / (1 - 1e-4)
            n = len(fvals)
            P("   %s, %s [%.4f, %.4f]: hits %d of %d; one random ratio lands with p = %.2f (log-uniform), %.2f (uniform);" % (law, label, lo, hi, hits, n, p_log, p_uni))
            P("        expected hits by chance %.1f / %.1f; P(at least one) = %.3f / %.3f" % (n * p_log, n * p_uni, 1 - (1 - p_log) ** n, 1 - (1 - p_uni) ** n))
    # supernova quick-look window on L2: Mahalanobis <= 1 for each SN combination
    fine = np.linspace(0.02, 1.0, 50)
    mm = []
    for f in fine:
        om, _ = best("L2", f)
        w0, wa = C.fit_cpl(sol_law("L2", f, om))
        mm.append([C.maha(w0, wa, row) for row in C.DESI[1:]])
    mm = np.array(mm)
    for j, row in enumerate(C.DESI[1:]):
        ok = fine[mm[:, j] <= 1.0]
        if len(ok):
            hits = [k for k, _, f in sh if ok.min() <= f <= ok.max()]
            P("   L2 quick-look window for %s (within 1 standard error): f from %.2f to %.2f; shares landing: %s" % (row[0], ok.min(), ok.max(), ", ".join(hits) or "none"))
        else:
            P("   L2 quick-look window for %s: empty (closest %.2f at f = %.2f)" % (row[0], mm[:, j].min(), fine[int(np.argmin(mm[:, j]))]))
    P()

    # ---- 5. by tier
    P("6. BY TIER: does one rule (organised/static = dynamics/base) give the sizes the record has measured?")
    kT_mol, s_mol = 0.325, 0.099
    P("   molecule: the dropped vibrations at the handoff, +%.3f +- %.3f kT a water (replace0/README.md), an ENERGY" % (kT_mol, s_mol))
    P("     over the rigid unit's kinetic energy 6 x kT/2 = 3 kT:      %.3f +- %.3f" % (kT_mol / 3, s_mol / 3))
    P("     over the vibrations' classical thermal energy 3 kT:        %.3f +- %.3f" % (kT_mol / 3, s_mol / 3))
    P("     by count of degrees of freedom: 3 over 6 = %.3f;  3 over 9 = %.3f" % (3 / 6, 3 / 9))
    P("   fluid element: prices are increments of R^2 to the next-state target (OBJECTIVE1_RESULTS.md)")
    P("     the dropped blocks read -0.0031, -0.0017, -0.0030 and +0.0033: consistent with ZERO, of either sign")
    P("     the largest of them over the smallest carried price: 0.0033 / 0.0390 = %.3f   (a window's edges, not a measured size)" % (0.0033 / 0.0390))
    P("     by count: structural (q, s2) and count, 3 fields, over 5 conserved densities = %.3f; over 4 (no energy) = %.3f" % (3 / 5, 3 / 4))
    P("   hydrogen-bond network: increment to the momentum law exactly 0")
    cands = {"molecule, energy/3kT": (kT_mol / 3, s_mol / 3), "fluid, price edge ratio": (0.0033 / 0.0390, None)}
    P("   cosmic ratios within one standard error of the molecule's %.3f +- %.3f:" % cands["molecule, energy/3kT"])
    lo, hi = kT_mol / 3 - s_mol / 3, kT_mol / 3 + s_mol / 3
    inwin = [(k, name, f) for k, name, f in sh if lo <= f <= hi]
    for k, name, f in inwin:
        P("     %-6s f = %.4f  (%s)" % (k, f, name))
    p_log = (math.log10(hi) - math.log10(lo)) / 4.0
    P("     that window [%.3f, %.3f] catches one random log-uniform ratio in [1e-4, 1] with p = %.3f: expected %.1f of %d by chance; found %d" % (lo, hi, p_log, len(sh) * p_log, len(sh), len(inwin)))
    P("   a rule by COUNT gives 0.5 (molecule) and 0.6 (fluid); a rule by ENERGY gives 0.108 (molecule) and has no fluid reading;")
    P("   a rule by PRICE gives <= 0.085 (fluid), 0 (network), and has no molecule reading in the same unit.")
    P("   No one rule is measured at two tiers in one unit.")
    P()

    # ---- 6. the writer's budget in the base's terms
    P("7. WHO PAYS, IN THE BASE'S TERMS")
    for k in ("r1.T1", "r8.T1", "r5.T1", "r3.T1"):
        f = dict((a, c) for a, _, c in sh)[k]
        oorg = f * OM_L
        P("   %s f = %.4f: the organised portion is %.4f of critical: %.1f x all baryons, %.0f to %.0f x the rest mass of every star formed" % (
            k, f, oorg, oorg / OB, oorg / OSTAR_MD, oorg / OSTAR_FP))
    P("   fusion pays 0.007 of a star's rest mass: a star-written record of that size is unpaid by %.1f to %.1f dex at f = Omega_b/Omega_dm." % (
        math.log10(r["r1"][1] * OM_L / (0.007 * OSTAR_MD)), math.log10(r["r1"][1] * OM_L / (0.007 * OSTAR_FP))))
    P("   under LA nothing is written after the start: no rate, no writer, no bill - and it is matter (section 3).")
    P()

    # ---- 7. perturbations, linear theory, order of magnitude
    P("8. THE PERTURBATION CHANNEL AT A FORCED SIZE (linear growth; order of magnitude, not a Boltzmann code)")
    ode = lambda a: OM_L * np.ones_like(a)
    a, D0 = growth(OM_M, 0.0, ode)
    P("   reference: LCDM at Om = %.4f; growth from a = 0.001 to today, D(1)/D(0.001) = %.1f" % (OM_M, D0[-1] / D0[0]))

    def rep(label, om_cl, om_sm, fn, b=0.0, orgfn=None):
        _, D = growth(om_cl, om_sm, fn, b, orgfn)
        P("   %-78s sigma8 ratio %.3f" % (label, D[-1] / D0[-1]))
        return D[-1] / D0[-1]

    for f in (r["r1"][1], r["r8"][1], r["r3"][1] * ODM / OM_L):
        osm = f * OM_L
        # LA, the background held at the measured Om_eff = 0.3153: clustering matter is Om - f*ODE'
        # here ODE' is the TOTAL dark energy today (static + organised) = 1 - (Om - osm) ... keep total matter-like = OM_M
        rep("LA SMOOTH, f*ODE = %.4f of the measured matter-like density does not cluster:" % osm, OM_M - osm, osm, lambda a, o=osm: (OM_L) * np.ones_like(a))
        rep("LA CLUSTERING like matter, same f: identical to LCDM by construction:", OM_M, 0.0, ode)
    need = None
    for frac in np.linspace(0.0, 0.05, 51):
        _, D = growth(OM_M * (1 - frac), OM_M * frac, ode)
        if D[-1] / D0[-1] <= 0.95 and need is None:
            need = frac
    P("   a smooth matter-like fraction of %.3f of matter lowers sigma8 by 5 %% (the size of the larger published S8 offsets, from memory)" % need)
    P("     that is an organised share f = %.4f of dark energy; Omega_b/Omega_dm asks %.0f x that." % (need * OM_M / OM_L, r["r1"][1] / (need * OM_M / OM_L)))
    # today-only laws: smooth organised portion, background only
    for k, law in (("r1.T1", "LD"), ("r1.T1", "L2"), ("r6.T1", "L2"), ("r8.T1", "LD")):
        f, c2, w0, wa, zc, om = table[(k, law)]
        s = sol_law(law, f, OM_M)
        rho = lambda a, s=s: (1 - OM_M - C.OM_R) * np.interp(a, C.a_g, s["rho"])
        rep("%s %s f = %.3f, organised portion SMOOTH (background only), Om = %.4f:" % (k, law, f, OM_M), OM_M, 0.0, rho)
        org = lambda a, s=s, f=f: (1 - OM_M - C.OM_R) * f * np.interp(a, C.a_g, s["org"])
        rep("%s %s f = %.3f, organised portion CLUSTERING with bias 1:" % (k, law, f), OM_M, 0.0, rho, 1.0, org)
    P("   Poisson source of a clustering organised portion relative to matter's, today: f x ODE/Om x b = %.3f x b at f = Omega_b/Omega_dm" % (r["r1"][1] * OM_L / OM_M))
    P("   the perturbation note's all-sky bound |f x b| < 0.063 gives b <= %.2f at f = %.3f and b <= %.2f at f = %.3f" % (
        0.063 / r["r1"][1], r["r1"][1], 0.063 / r["r6"][1], r["r6"][1]))
    P("   a star-written record the stars can pay for at fusion efficiency: Omega_* x 0.007 / Omega_DE = %.1e to %.1e" % (
        OSTAR_FP * 0.007 / OM_L, OSTAR_MD * 0.007 / OM_L))
    P("   published summaries used for scale, all FROM MEMORY: Planck S8 0.832 +- 0.013; KiDS-1000 0.759 +- 0.024; DES Y3 0.776 +- 0.017;")
    P("   KiDS-Legacy 0.815 +- 0.016; Planck's ISW amplitude about 1.0 +- 0.25.")
    P()

    # ---- 8. the stake's numbers
    P("9. NUMBERS FOR A STAKE WITH THE SHARE FIXED")
    for k, law in (("r1.T1", "LD"), ("r1.T1", "L2"), ("r1.T2", "L2"), ("r2.T1", "L2"), ("r6.T1", "L2"), ("r6.T2", "L2"), ("r7.T1", "L2")):
        f, c2, w0, wa, zc, om = table[(k, law)]
        P("   %-6s %s f = %.4f: chi2 %.3f (%+.3f vs constant); Om = %.4f; w0 = %+.3f, wa = %+.3f; crossing %s" % (
            k, law, f, c2, c2 - c2_const, om, w0, wa, ("z = %.3f" % zc) if zc else "none"))
    P("   separations between the predictions, in standard errors of a DR2 combination (assumed correlation %.1f):" % C.RHO_W)
    pts = {"constant": (-1.0, 0.0)}
    for lab, key in (("A: Om/ODE under L2", ("r6.T1", "L2")), ("B: Ob/Odm under LD", ("r1.T1", "LD")), ("B': Ob/Odm under L2", ("r1.T1", "L2"))):
        f, c2, w0, wa, zc, om = table[key]
        pts[lab] = (w0, wa)
    names = list(pts)
    for row in (C.DESI[1], C.DESI[3]):
        for i in range(len(names)):
            for j in range(i + 1, len(names)):
                fake = (row[0], pts[names[j]][0], row[2], pts[names[j]][1], row[4])
                P("     %-22s %-20s to %-20s %.1f" % (row[0] + ":", names[i], names[j], C.maha(pts[names[i]][0], pts[names[i]][1], fake)))
    P()
    P("done.")


if __name__ == "__main__":
    main()
