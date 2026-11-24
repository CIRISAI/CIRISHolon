#!/usr/bin/env python3
"""COSMOS-BOUND-FREE: the organised portion split into a BOUND part (follows matter)
and a FREE part (cannot be held, smooth over the horizon). A paper campaign.

numpy only, single thread, seconds. Prints every number COSMOS_BOUND_FREE.md quotes.
No data is read. It imports the grid, the star-formation history and the two-portion
solver from cosmos_two_portion.py, and the Planck inventory from cosmos_base.py.

ORDER-OF-MAGNITUDE LINEAR THEORY. Not a Boltzmann code. Not reviewed.

NOT computed here, on purpose (owed from the review running beside this note):
  (R1) the crossing redshift of a record written at the star-formation rate and then
       diluting, as a function of the dilution exponent;
  (R2) the sign and size of the ISW response to a clustering portion, with the
       published all-sky amplitude in place of the one quoted from memory.
"""
import math
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cosmos_two_portion as T  # noqa: E402
import cosmos_base as B  # noqa: E402

P = print
trapz = getattr(np, "trapezoid", None) or np.trapz
OM, OR, H = B.OM_M, T.OM_R, B.H
ODE = 1.0 - OM - OR
OB, ODM = B.OB, B.ODM

# from COSMOS_CMB_CHANNEL.md section 3 and 5 (R2 may move the first of these)
P_LIM = 0.063                 # |P| at two sigma from the all-sky ISW amplitude (value FROM MEMORY there)
P_REACH = (0.016, 0.024)      # one-sigma reach today, DES Y6 against the CMB
P_FUTURE = (0.004, 0.006)     # a future half-per-cent amplitude measurement


# ------------------------------------------------------------------ 1. theorem (b), as numbers
def sec1():
    P("1. THEOREM (b) AS NUMBERS: a fluid below its sound horizon does not cluster")
    P("   quasi-static, sub-sound-horizon:  delta_F / delta_m  ~  (1 + w) (3/2) Omega_m (a H / c_s k)^2")
    c_over_H0 = 2997.92458  # Mpc/h
    w = -0.8
    for cs2 in (1.0, 1e-2, 1e-4, 1e-6):
        ks = 1.0 / (math.sqrt(cs2) * c_over_H0)
        row = []
        for k in (0.005, 0.02, 0.1):
            r = (1 + w) * 1.5 * OM * (1.0 / (math.sqrt(cs2) * c_over_H0 * k)) ** 2
            row.append(f"k = {k} h/Mpc: {min(r, (1 + w)):.2e}")
        P(f"   c_s^2 = {cs2:g}: sound horizon k_s = {ks:.2e} h/Mpc; effective bias at w = {w}: " + "; ".join(row))
    P("   (capped at (1 + w), the value for c_s = 0; a w = -1 portion has (1 + w) = 0 and no perturbation at all)")
    P("   the scales the void and ISW measurements use are k ~ 0.005 to 0.05 h/Mpc")


# ------------------------------------------------------------------ 2. the inventory
NW = 4 * math.pi * 1e-9 / T.C_LIGHT / (3.0 * (100e3 * H / T.MPC_M) ** 2 / (8 * math.pi * T.G_NEWT) * T.C_LIGHT ** 2)

# Fukugita & Peebles 2004, "The Cosmic Energy Inventory" (astro-ph/0406095), Table 1, h = 0.7.
# ALL FROM MEMORY of that table; not re-read at source in this run.
INV = [
    # name, log10 |Omega|, w, where, can it be dark energy
    ("starlight + reprocessed infrared (optical 10^-5.8, far-IR 10^-6.1)", -5.7, 1 / 3, "FREE", "no: radiation, dilutes as a^-4"),
    ("stellar neutrinos (core collapse 10^-5.5, burning 10^-6.8)", -5.5, 1 / 3, "FREE", "no: radiation-like, a^-4"),
    ("gravitational waves, massive black-hole mergers", -7.5, 1 / 3, "FREE", "no: radiation, a^-4"),
    ("gravitational waves, stellar binaries", -9.0, 1 / 3, "FREE", "no: radiation, a^-4"),
    ("X and gamma rays", -7.9, 1 / 3, "FREE", "no: radiation, a^-4"),
    ("cosmic rays and magnetic fields", -8.3, 1 / 3, "BOUND (to galaxies)", "no: w >= 0"),
    ("kinetic energy of the intergalactic medium", -8.0, 0.0, "BOUND (follows gas)", "no: w ~ 0"),
    ("white dwarfs (rest mass)", math.log10(3.6e-4), 0.0, "BOUND", "no: matter"),
    ("neutron stars (rest mass)", math.log10(5e-5), 0.0, "BOUND", "no: matter"),
    ("stellar black holes (rest mass)", math.log10(7e-5), 0.0, "BOUND", "no: matter in GR"),
    ("massive black holes (rest mass)", -5.4, 0.0, "BOUND", "no: matter in GR"),
    ("nuclear binding released by stars (the metals' deficit)", -5.2, None, "the SOURCE of the free radiation", "it is a deficit, not a component"),
    ("binding released by dissipative settling (remnants, nuclei)", -4.9, None, "the SOURCE; mostly neutron stars and black holes", "a deficit, not a component"),
]


def sec2():
    P("")
    P("2. THE INVENTORY: what structure formation writes, and where it goes (Fukugita & Peebles 2004, FROM MEMORY)")
    P(f"   1 nW m^-2 sr^-1 of isotropic background = {NW:.2e} of the critical density (h = {H})")
    free = bound_e = bound_m = 0.0
    for name, lg, w, where, de in INV:
        om = 10 ** lg
        ws = "deficit" if w is None else f"w = {w:.2f}"
        P(f"   {name:<66s} {om:9.2e}  {ws:<9s} {where}; dark energy? {de}")
        if w is None:
            continue
        if where.startswith("FREE"):
            free += om
        elif "rest mass" in name:
            bound_m += om
        else:
            bound_e += om
    # heat of the gas, computed: (3/2) kT / (mu m_p c^2) times its density parameter
    mu_mp = 0.6 * 938.272e3  # keV
    icm = 0.0018 * 1.5 * 5.0 / mu_mp
    whim = 0.040 * 1.5 * 0.1 / mu_mp
    P(f"   heat of intracluster gas (Omega 0.0018, kT 5 keV)                 {icm:9.2e}  w ~ 0     BOUND (in the wells)")
    P(f"   heat of the warm-hot intergalactic gas (Omega 0.040, kT 0.1 keV)  {whim:9.2e}  w ~ 0     BOUND (in the filaments)")
    bound_e += icm + whim
    rel = 10 ** -5.2 + 10 ** -4.9
    P(f"   TOTALS: free, radiation-like, today {free:.2e}; bound as energy (heat, motion, cosmic rays) {bound_e:.2e}; bound as remnant rest mass {bound_m:.2e}")
    P(f"   total binding released (nuclear + settling) {rel:.2e}; the free part is {free / rel:.2f} of it today (the rest was redshifted away, or is the settling of black holes that radiated little)")
    P(f"   free total against dark energy: {free / ODE:.1e}; everything ever released against dark energy: {rel / ODE:.1e}")
    P("   EVERY free item has w = 1/3 and every bound item w >= 0: each dilutes FASTER than a constant. None can be dark energy.")
    return free, bound_e, bound_m, rel


# ------------------------------------------------------------------ 3. the sourced smooth field
def lcdm_tau():
    E = np.sqrt(OR * T.a_g ** -4 + OM * T.a_g ** -3 + ODE)
    return E, T.cumtrapz0(1.0 / E, T.lna)


def sec3():
    P("")
    P("3. THE ONLY CONSISTENT FORM: a smooth negative-pressure field with a source set by the writing rate")
    P("   d rho_F/dt + 3 H (1 + w_F) rho_F = Q(t),  Q = alpha c^2 psi(t)   (psi: Madau & Dickinson 2014 eq. 15)")
    E, tau = lcdm_tau()
    hub_yr = 977.8e9 / (100 * H)                       # 1/H0 in years
    rho_formed = trapz(T.PSI, tau) * hub_yr          # Msun / Mpc^3, all stars ever formed
    rhoc = B.RHOCRIT_H2 * H ** 2
    of = rho_formed / rhoc
    ae3 = trapz(T.PSI * T.a_g ** 3, tau) / trapz(T.PSI, tau)
    zmean = trapz(T.PSI * T.z_g, tau) / trapz(T.PSI, tau)
    P(f"   all stars ever formed: {rho_formed:.2e} Msun/Mpc^3 = {of:.4f} of critical (surviving, at a return fraction of 0.27: {of * 0.73:.4f}; cosmos_base.py has {B.OSTAR_MD:.4f})")
    P(f"   mass-weighted mean formation redshift {zmean:.2f}; mean a_e^3 = {ae3:.3f}")
    P("   (a) SIZE with w_F = -1 (not diluted once written): share today f = alpha x Omega_formed / Omega_DE")
    for lab, al in (("alpha = 0.007 (hydrogen burning)", 0.007), ("alpha = Omega_b/Omega_dm", OB / ODM), ("alpha = 1 (the whole rest mass)", 1.0)):
        P(f"       {lab:<34s} f = {al * of / ODE:.2e}")
    for f in (OB / ODM, ODM / ODE, OM / ODE):
        P(f"       f = {f:.3f} needs alpha = {f * ODE / of:.1f} (grams of field per gram of star formed)")
    P("   (b) SIGN with w_F = -1: the sum of a constant and a portion that only accumulates")
    for f in (OB / ODM, OM / ODE, 1.0):
        s = T.solve(f, 0.0, om_m=OM, variant="A")
        cz = T.crossing(s)
        P(f"       f = {f:.3f}: w_eff(z = 0) = {T.at_z(s, 'w', 0.0):+.4f}, w_eff(z = 0.5) = {T.at_z(s, 'w', 0.5):+.4f}, w_eff(z = 1) = {T.at_z(s, 'w', 1.0):+.4f}; crossing: {cz}")
    P("       phantom at every epoch, and never above -1: the OPPOSITE of DR2's sign today (w0 > -1). A source alone does not give the shape.")
    P("   (c) SINK = matter (the field is paid for by what forms it): matter no longer dilutes exactly as a^-3")
    for f in (OB / ODM, ODM / ODE, OM / ODE):
        d = f * ODE * ae3
        P(f"       f = {f:.3f}: matter missing today {d:.4f} of critical = {100 * d / OM:.1f} % of all matter = {100 * d / OB:.0f} % of the baryons (if the baryons pay) = {100 * d / ODM:.1f} % of the dark matter (if it pays)")
    P("       so the matter density inferred early (CMB) EXCEEDS the late one by that much; and if the baryons pay, that fraction of them is gone")
    P("   (d) SINK = the static portion: d(rho_static + rho_F)/dt = -3 H (1 + w_F) rho_F; with w_F = -1 the sum is a constant.")
    P("       Nothing is added to a constant on the background, and both parts are uniform: a relabelling.")
    P("   (e) NO SINK: the total stress-energy is not conserved; not general relativity.")
    P("   (f) the shape needs BOTH a source (the rise) AND a dilution or decay of the field (the fall): 1 + w_eff = share x (1 + w_F) - Q / (3 H rho_DE)")
    P("       the crossing is where Q = 3 H (1 + w_F) rho_F. With w_F = 0 the record already has it: z = 0.818 at f = 0.186 (COSMOS_BASE.md section 4).")
    P("       Its dependence on the dilution exponent, and whether it is independent of the amplitude: OWED FROM THE REVIEW (R1), not computed here.")
    return of, ae3


# ------------------------------------------------------------------ 4. the bound fraction
def sec4(free, bound_e, bound_m, rel):
    P("")
    P("4. THE BOUND FRACTION beta: P = f x beta x b, against |P| < %.3f (pending R2)" % P_LIM)
    for f in (0.19, 0.39, 0.46):
        P(f"   f = {f:.2f}: beta <= {P_LIM / f:.2f} at b = 1; <= {P_LIM / f / 2:.2f} at b = 2; <= {P_LIM / f / 5:.3f} at b = 5 (clusters)")
    P("   the inventory's bound fraction, by four rules written down before the division:")
    rules = [
        ("(i)   by energy today: bound heat, motion and cosmic rays over bound + free radiation", bound_e / (bound_e + free)),
        ("(ii)  by energy today, counting remnant rest mass as bound", (bound_e + bound_m) / (bound_e + bound_m + free)),
        ("(iii) by binding released: what stayed as a deficit in compact objects (settling) over all released", 10 ** -4.9 / rel),
        ("(iv)  by entropy (Egan & Lineweaver 2010, FROM MEMORY: black holes 3e104 k, everything free under 1e90 k)", 1.0 - 1e90 / 3.1e104),
    ]
    for lab, beta in rules:
        row = []
        for f in (0.19, 0.39, 0.46):
            pv = f * beta * 1.0
            st = "ALLOWED" if pv < P_LIM else "EXCLUDED"
            row.append(f"f = {f:.2f}: P = {pv:.4f} {st}")
        P(f"   {lab}")
        P(f"         beta = {beta:.4f};  at b = 1: " + "; ".join(row))
    b1 = rules[0][1]
    P(f"   rule (i) lands inside the allowed range with no tuning: beta = {b1:.4f} against a ceiling of {P_LIM / 0.46:.2f} to {P_LIM / 0.19:.2f}.")
    for f in (0.19, 0.39, 0.46):
        pv = f * b1
        P(f"       f = {f:.2f}: P = {pv:.4f}; today's one-sigma reach {P_REACH[0]} to {P_REACH[1]}: {P_REACH[0] / pv:.1f} to {P_REACH[1] / pv:.1f} times under; a future half-per-cent measurement {P_FUTURE[0]} to {P_FUTURE[1]}: {P_FUTURE[0] / pv:.1f} to {P_FUTURE[1] / pv:.1f}")
    P("   rules (ii), (iii) and (iv) land OUTSIDE it at every share. Four rules, one inside: beta is a fork, not a derivation.")


# ------------------------------------------------------------------ 5. the free part's own observable
def background(w0, wa, om=OM):
    a = np.exp(np.linspace(math.log(1e-3), 0.0, 6000))
    ode = 1.0 - om - OR
    de = a ** (-3 * (1 + w0 + wa)) * np.exp(-3 * wa * (1 - a))
    E2 = OR * a ** -4 + om * a ** -3 + ode * de
    return a, np.sqrt(E2)


def growth(w0, wa, om=OM):
    a, E = background(w0, wa, om)
    x = np.log(a)
    dlnE = np.gradient(np.log(E), x)
    omm = om * a ** -3 / E ** 2
    d = np.zeros_like(a)
    p = np.zeros_like(a)
    d[0], p[0] = a[0], a[0]

    def rhs(i0, frac, dd, pp):
        de_ = dlnE[i0] + frac * (dlnE[i0 + 1] - dlnE[i0])
        om_ = omm[i0] + frac * (omm[i0 + 1] - omm[i0])
        return pp, -(2.0 + de_) * pp + 1.5 * om_ * dd

    for i in range(len(a) - 1):
        h = x[i + 1] - x[i]
        k1 = rhs(i, 0.0, d[i], p[i])
        k2 = rhs(i, 0.5, d[i] + 0.5 * h * k1[0], p[i] + 0.5 * h * k1[1])
        k3 = rhs(i, 0.5, d[i] + 0.5 * h * k2[0], p[i] + 0.5 * h * k2[1])
        k4 = rhs(i, 1.0, d[i] + h * k3[0], p[i] + h * k3[1])
        d[i + 1] = d[i] + h * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]) / 6
        p[i + 1] = p[i] + h * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]) / 6
    f = p / d
    # comoving distance in units of c/H0
    z = 1.0 / a - 1.0
    chi = np.zeros_like(a)
    integ = 1.0 / (a * E)            # d chi / d ln a  (with sign handled below)
    chi[::-1][1:] = np.cumsum(0.5 * (integ[::-1][1:] + integ[::-1][:-1]) * np.abs(np.diff(x[::-1])))
    return dict(a=a, z=z, E=E, D=d, f=f, chi=chi)


def at(g, key, zq):
    return float(np.interp(zq, g["z"][::-1], g[key][::-1]))


def lensing(g, n_eff=-1.0, L=100.0, zstar=1090.0):
    """Limber, C_L^kk ~ int dz/E [ (1+z) (chi* - chi)/chi* ]^2 D^2 (L/chi)^n_eff, with chi* extended
    by matter + radiation above the grid; only the RATIO between two models is used."""
    a, z, E, D, chi = g["a"], g["z"], g["E"], g["D"], g["chi"]
    # chi from the grid's first point (z = 999) to z* is common to both models to first order
    chis = chi[0] + 0.0
    m = z < 30.0
    zz, EE, DD, cc = z[m][::-1], E[m][::-1], D[m][::-1], chi[m][::-1]
    cc = np.maximum(cc, 1e-6)
    W = ((1 + zz) * (chis - cc) / chis) ** 2 * DD ** 2 * (L / cc) ** n_eff / EE
    return float(trapz(W, zz))


CANDS = [
    ("A: Omega_m/Omega_DE under the rent ledger", -0.768, -0.739),
    ("   Omega_dm/Omega_DE under the rent ledger", -0.804, -0.622),
    ("B: Omega_b/Omega_dm, star-written and diluting", -0.863, -0.347),
    ("   Omega_b/Omega_dm under the rent ledger", -0.904, -0.303),
    ("   DR2's own point with DESY5 (for scale)", -0.752, -0.86),
]

# errors, ALL FROM MEMORY unless said: fractional one-sigma
ERR = dict(
    s8_now=("sigma8 from DESI DR1 full shape, 0.841 +- 0.034 (from memory)", 0.040),
    fs8_now=("f sigma8 in one DESI DR1 bin (from memory)", 0.08),
    fs8_fut=("f sigma8, DESI final survey, one bin (forecast, from memory)", 0.015),
    fs8_euc=("f sigma8, Euclid, one bin (forecast, from memory)", 0.010),
    lens_now=("CMB lensing amplitude, ACT DR6 1.013 +- 0.023 (arXiv 2503.14452, as read by COSMOS_CMB_CHANNEL.md)", 0.023),
    lens_so=("CMB lensing amplitude, Simons Observatory (forecast, from memory)", 0.010),
    lens_s4=("CMB lensing amplitude, CMB-S4 (forecast, from memory)", 0.005),
    isw_now=("all-sky ISW amplitude, 1.00 +- 0.25 (Planck 2015 XXI, from memory; R2 owes the published value)", 0.25),
    isw_cv=("all-sky ISW amplitude at the cosmic-variance limit, S/N about 7.5 (from memory)", 0.13),
)


def sec5():
    P("")
    P("5. THE FREE PART'S OWN OBSERVABLE: a smooth evolving portion acts on growth, lensing and the ISW effect only through H(a)")
    P(f"   same Omega_m = {OM}, same H0, same early amplitude; linear growth; CPL (w0, wa) as COSMOS_BASE.md section 4 gives them")
    g0 = growth(-1.0, 0.0)
    l0 = lensing(g0)
    out = []
    for lab, w0, wa in CANDS:
        g = growth(w0, wa)
        s8 = g["D"][-1] / g0["D"][-1] - 1.0
        fs = [at(g, "f", z) * at(g, "D", z) / (at(g0, "f", z) * at(g0, "D", z)) - 1.0 for z in (0.3, 0.5, 0.8, 1.0)]
        isw = [(at(g, "f", z) - 1.0) / (at(g0, "f", z) - 1.0) for z in (0.3, 0.5, 1.0)]
        le = lensing(g) / l0 - 1.0
        hz = [at(g, "E", z) / at(g0, "E", z) - 1.0 for z in (0.5, 1.0)]
        P(f"   {lab:<48s} (w0, wa) = ({w0}, {wa})")
        P(f"       sigma8 {100 * s8:+.2f} %;  f sigma8 at z = 0.3/0.5/0.8/1.0: " + "/".join(f"{100 * v:+.2f}" for v in fs) + " %"
          f";  CMB lensing power {100 * le:+.2f} %;  H at z = 0.5/1.0: {100 * hz[0]:+.2f}/{100 * hz[1]:+.2f} %")
        P(f"       ISW amplitude (potential decay rate per unit structure) at z = 0.3/0.5/1.0: " + "/".join(f"{v:.3f}" for v in isw))
        out.append((lab, s8, fs, isw, le))
    P("   against the errors (one sigma, fractional):")
    for k, (lab, e) in ERR.items():
        P(f"       {k:<9s} {e:.3f}  {lab}")
    P("   significance of each candidate's shift, in standard errors:")
    for lab, s8, fs, isw, le in out:
        fmax = max(abs(v) for v in fs)
        imean = abs(isw[1] - 1.0)
        P(f"   {lab.strip():<46s} sigma8 now {abs(s8) / ERR['s8_now'][1]:.2f}; f sigma8 now {fmax / ERR['fs8_now'][1]:.2f}, DESI final {fmax / ERR['fs8_fut'][1]:.2f}, Euclid {fmax / ERR['fs8_euc'][1]:.2f};"
          f" lensing now {abs(le) / ERR['lens_now'][1]:.2f}, SO {abs(le) / ERR['lens_so'][1]:.2f}, S4 {abs(le) / ERR['lens_s4'][1]:.2f};"
          f" ISW now {imean / ERR['isw_now'][1]:.2f}, at the cosmic-variance limit {imean / ERR['isw_cv'][1]:.2f}")
    P("   (the background itself separates A and B from a constant by 2.7 to 4.2 of DR2's standard errors: COSMOS_BASE.md section 9)")
    return out


# ------------------------------------------------------------------ 6. the haze, and the diffuse backgrounds
def sec6():
    P("")
    P("6. THE HAZE (a boundary case) AND A SCREEN OF THE DIFFUSE BACKGROUNDS")
    kpc = 3.0856775814913673e21  # cm
    vol = 2 * (4 / 3) * math.pi * (5.0 * 3.0 * 3.0) * kpc ** 3        # two lobes, semi-axes 5 x 3 x 3 kpc
    u_de = T.RHO_DE * 10.0                                            # erg / cm^3
    P(f"   dark-energy density {u_de:.2e} erg/cm^3 = {u_de / 1.602e-9:.2f} keV/cm^3; two lobes of 5 x 3 x 3 kpc: {vol:.2e} cm^3")
    e_de = u_de * vol
    P(f"   dark energy inside the bubbles' volume: {e_de:.1e} erg")
    for lab, e in (("galactic-wind estimate (arXiv 1505.03634)", 7.7e54), ("explosive estimate (arXiv 2203.02526 and others)", 1e56), ("jet model, both jets (as returned by search)", 2e57)):
        P(f"   the bubbles' energy, {lab}: {e:.1e} erg = 1/{e_de / e:.0f} of the dark energy in their own volume")
    ucrit = T.U_CRIT * 10.0 * (H * 100 / T.H0_FID) ** 2
    for n in (0.01, 0.003):
        for e in (1e55, 1e56):
            om = n * e / (1e6 * kpc) ** 3 / ucrit
            P(f"   one such pair per galaxy at {n} per Mpc^3, {e:.0e} erg each: {om:.1e} of critical = {om / ODE:.1e} of dark energy")
    P("   a BOUND record of ordinary matter and fields. An illustration. Nothing is claimed from it.")
    P("   the smooth backgrounds that are NOT fully explained, as candidates for a free written record (all FROM MEMORY):")
    # radio: T = 24.1 K (nu / 310 MHz)^-2.599 (Fixsen et al. 2011, ARCADE 2); nu I_nu at 1 GHz, times ~3 for the band
    Tr = 24.1 * (1000.0 / 310.0) ** -2.599
    nuI = 2 * T.KB * Tr * (1e9) ** 3 / T.C_LIGHT ** 2 * 1e9            # nW m^-2 sr^-1
    rows = [
        (f"radio synchrotron background excess (ARCADE 2: {Tr:.2f} K at 1 GHz; nu I_nu {nuI:.1e} nW, band of about 3 e-folds)", 3 * nuI),
        ("optical background excess (New Horizons: 8 +- 2 nW claimed in 2022; revised to about 3 +- 2, consistent with galaxies, in 2024)", 3.0),
        ("the whole optical + infrared background, for scale (about 60 to 100 nW)", 80.0),
        ("the unresolved gamma-ray background (about 1e-3 MeV cm^-2 s^-1 sr^-1 a decade, five decades)", 5 * 1e-3 * 1.602e-13 * 1e4 * 1e9),
    ]
    for lab, nw in rows:
        P(f"   {lab}: {nw:.1e} nW m^-2 sr^-1 = {nw * NW:.1e} of critical = {nw * NW / ODE:.1e} of dark energy; w = 1/3")
    P("   all radiation: w = 1/3, diluting as a^-4, and 5 to 11 orders under dark energy. None is the free portion.")


# ------------------------------------------------------------------ 7. the stake's numbers
def sec7(out):
    P("")
    P("7. THE STAKE'S SEPARATION, ON PAPER")
    P("   three readings of one void catalogue and one growth measurement, with the share FIXED (COSMOS_BASE.md's A or B):")
    P("     smooth constant:            background (w0, wa) = (-1, 0);  A_kappa = 1;  f sigma8 shift 0")
    for lab, s8, fs, isw, le in out[:1] + out[2:3]:
        f_share = OM / ODE if lab.startswith("A") else OB / ODM
        fmax = max(fs, key=abs)
        P(f"     {lab.strip()[:44]:<44s} FREE (beta = 0): A_kappa = 1.000; f sigma8 {100 * fmax:+.2f} %; CMB lensing {100 * le:+.2f} %")
        for beta in (0.0095, 0.1, 0.3, 1.0):
            pv = f_share * beta
            eps7 = pv * (ODE / OM) * (1 / 1.7) ** 3       # n = 3 at z = 0.7, as COSMOS_CMB_CHANNEL.md section 2
            P(f"         bound fraction {beta:<6g}: P = {pv:.4f}; void lensing A_kappa(z = 0.7) = {1 + eps7:.3f} = {eps7 / 0.054:+.1f} sigma of 1.016 +- 0.054; {eps7 / 0.02:+.1f} at a future +- 0.02")
    P("   so 'evolving AND smooth' and 'evolving AND following matter' are told apart by void lensing only at a bound fraction of")
    P("   about 0.3 or more for A, and about 0.7 or more for B, at a future error of 0.02; today only a wholly bound A is read.")
    P("   Below that the bound part is not read, and the free part is read only through the background and the growth rate.")


def main():
    P("COSMOS-BOUND-FREE  (linear theory; no data read; R1 and R2 owed from the review and not computed)")
    P(f"Omega_m = {OM}, Omega_DE = {ODE:.4f}, Omega_b = {OB:.4f}, Omega_dm = {ODM:.4f}, h = {H}")
    P("")
    sec1()
    free, be, bm, rel = sec2()
    sec3()
    sec4(free, be, bm, rel)
    out = sec5()
    sec6()
    sec7(out)
    P("")
    P("done.")


if __name__ == "__main__":
    main()
