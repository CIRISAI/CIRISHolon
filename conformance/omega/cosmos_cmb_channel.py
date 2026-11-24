#!/usr/bin/env python3
"""COSMOS, THE PERTURBATION CHANNEL: every number of COSMOS_CMB_CHANNEL.md.

The two-portion picture (TIERS.md section 4, the amendment of 2026-11-23):
    rho_DE(a) = rho_static + rho_org(a),   share today f = rho_org/rho_DE
The static portion is smooth. The organised portion is taken to follow structure:
    delta rho_org = b * rho_org * delta_m          (b: its linear bias against matter)
so the Poisson source becomes rho_m delta_m (1 + eps(a)), with
    eps(a) = P * [Omega_DE/Omega_m](a) * [rho_org(a)/rho_org(1)] / [rho_DE(a)/rho_DE(1)],   P = f * b.

ORDER-OF-MAGNITUDE LINEAR THEORY, not a Boltzmann code. Stated approximations:
  - sub-horizon, quasi-static, no anisotropic stress: k^2 Phi = -4 pi G a^2 (total density perturbation);
  - the background is flat LCDM (Omega_m = 0.315); the organised share's effect on H(a) is ignored here
    (COSMOS_TWO_PORTION.md prices it), and rho_org(a) is a power law a^(-3(1 + w_org)), so
    eps(a) = P * (Omega_DE0/Omega_m0) * a^n with n = -3 w_org;
  - the measured amplitudes are ratios to an LCDM template built on the same galaxy-traced matter field:
        A_kappa(z) = 1 + eps(z)                      (the potential itself)
        A_ISW(z)   = [(1 + eps)(g' - 1) + n eps] / (g - 1)    (the potential's rate; g = dln delta/dln a)
    with g' the growth rate under the modified source and g the LCDM one.

No dataset, map or likelihood is read. The published amplitudes below are summary numbers typed in from
the papers named in the note (each labelled there as read at abstract, read through a search summary,
or from memory). numpy only; single thread; seconds.
"""
import numpy as np

OM, ODE = 0.315, 0.685
R0 = ODE / OM


def P(*a):
    print(*a)


_CACHE = {}
_NA = 1200
_X = np.linspace(np.log(1e-3), 0.0, _NA)


def _solve(Pfb, n):
    """delta'' + (2 + H'/H) delta' = 1.5 Om(a) (1 + eps(a)) delta in x = ln a, RK4 on a fixed grid, plain floats."""
    key = (round(Pfb, 12), n)
    if key in _CACHE:
        return _CACHE[key]
    import math
    x = _X; h = float(x[1] - x[0]); c = Pfb * R0

    def rhs(xx, d, dp):
        a3 = math.exp(3 * xx)
        om = OM / (OM + ODE * a3)
        return dp, -(2.0 - 1.5 * om) * dp + 1.5 * om * (1.0 + c * math.exp(n * xx)) * d

    d = dp = math.exp(float(x[0]))
    ds = [d]; gs = [1.0]
    for i in range(_NA - 1):
        xi = float(x[i])
        a1, b1 = rhs(xi, d, dp)
        a2, b2 = rhs(xi + h / 2, d + h / 2 * a1, dp + h / 2 * b1)
        a3_, b3 = rhs(xi + h / 2, d + h / 2 * a2, dp + h / 2 * b2)
        a4, b4 = rhs(xi + h, d + h * a3_, dp + h * b3)
        d += h / 6 * (a1 + 2 * a2 + 2 * a3_ + a4); dp += h / 6 * (b1 + 2 * b2 + 2 * b3 + b4)
        ds.append(d); gs.append(dp / d)
    _CACHE[key] = (np.array(ds), np.array(gs))
    return _CACHE[key]


def growth(Pfb, n, a_out):
    ds, gs = _solve(Pfb, n)
    xo = np.log(np.atleast_1d(a_out))
    return np.interp(xo, _X, ds), np.interp(xo, _X, gs)


def eps_at(Pfb, n, z):
    a = 1.0 / (1.0 + z)
    return Pfb * R0 * a ** n


def amps(Pfb, n, z):
    a = 1.0 / (1.0 + z)
    d0, g0 = growth(0.0, n, a)
    d1, g1 = growth(Pfb, n, a)
    e = eps_at(Pfb, n, z)
    a_isw = ((1 + e) * (g1[0] - 1) + n * e) / (g0[0] - 1)
    return dict(eps=e, A_kappa=1 + e, A_ISW=a_isw, growth_ratio=d1[0] / d0[0], g0=g0[0], g1=g1[0])


def solve_P(target, n, z, key, lo=-3.0, hi=3.0):
    f = lambda p: amps(p, n, z)[key] - target
    a, b = lo, hi
    fa, fb = f(a), f(b)
    if fa * fb > 0:
        return float("nan")
    for _ in range(34):
        m = 0.5 * (a + b); fm = f(m)
        if fa * fm <= 0:
            b, fb = m, fm
        else:
            a, fa = m, fm
    return 0.5 * (a + b)


def K_slope(n, z, dP=1e-4):
    """d(A_ISW)/d(A_kappa) at eps -> 0: the forced relation between the two amplitudes."""
    lo, hi = amps(-dP, n, z), amps(dP, n, z)
    return (hi["A_ISW"] - lo["A_ISW"]) / (hi["A_kappa"] - lo["A_kappa"])


LAWS = [(3.0, "organised density constant (w_org = -1)"),
        (1.98, "the rent ledger today (w_org = -0.660)"),
        (1.39, "the ledger diluted as matter (w_org = -0.463)"),
        (0.0, "diluting exactly as matter (w_org = 0)")]

# published summary amplitudes (the note says how each was read)
PUB = dict(
    kov19=("supervoids DES Y3 + BOSS, A_ISW (arXiv 1811.07812)", 5.2, 1.6),
    kov19d=("supervoids DES Y3 alone, A_ISW (arXiv 1811.07812)", 4.1, 2.0),
    kov22=("DES Y3 voids, A_kappa (arXiv 2203.11306)", 0.79, 0.12),
    hangv=("DESI Legacy voids, A_ISW (arXiv 2105.11936)", -0.10, 0.69),
    hangc=("DESI Legacy clusters, A_ISW (arXiv 2105.11936)", 1.52, 0.72),
    hangk=("DESI Legacy superstructures, A_kappa (as cited by arXiv 2203.11306)", 0.811, 0.057),
    nc16=("BOSS voids and superclusters, A_ISW (Nadathur and Crittenden 2016)", 1.64, 0.53),
    sart=("DESI LRG voids, A_kappa (A&A 700, A17, 2025)", 1.016, 0.054),
    plk=("ISW x large-scale structure, A (Planck 2015 XXI; from memory)", 1.00, 0.25),
)


def main():
    P("COSMOS, THE PERTURBATION CHANNEL. Linear theory, order of magnitude. Every number of COSMOS_CMB_CHANNEL.md.")
    P(f"Omega_m = {OM}, Omega_DE = {ODE}; Omega_DE/Omega_m today = {R0:.3f}")
    for z in (0.0, 0.3, 0.5, 0.7, 2.0):
        a = 1 / (1 + z)
        P(f"   z = {z:.1f}: Omega_DE/Omega_m = {R0 * a ** 3:.4f};  Omega_m(a) = {OM / (OM + ODE * a ** 3):.3f}")

    P("\n1. THE SOURCE: eps(a) = P x (Omega_DE0/Omega_m0) x a^n,  P = f x b,  n = -3 w_org")
    for n, name in LAWS:
        P(f"   n = {n:4.2f}  {name}: eps per unit P at z = 0, 0.5, 0.7, 2: "
          + ", ".join(f"{eps_at(1.0, n, z):.3f}" for z in (0, 0.5, 0.7, 2.0)))

    P("\n2. THE SIGN, AND THE SLOPE K = -d(A_ISW)/d(A_kappa) (growth feedback included)")
    d0, g0 = growth(0.0, 3.0, [1.0, 1 / 1.3, 1 / 1.5, 1 / 1.7])
    P("   LCDM growth rate g at z = 0, 0.3, 0.5, 0.7: " + ", ".join(f"{g:.3f}" for g in g0)
      + "   (potential decays as g - 1: " + ", ".join(f"{g - 1:+.3f}" for g in g0) + ")")
    for n, name in LAWS:
        ks = [-K_slope(n, z) for z in (0.3, 0.5, 0.7)]
        P(f"   n = {n:4.2f}: K at z = 0.3, 0.5, 0.7 = " + ", ".join(f"{k:+.2f}" for k in ks) + f"   ({name})")
    P("   K > 0 means: a portion that FOLLOWS matter (P > 0) deepens potentials (A_kappa > 1) and WEAKENS the ISW signal (A_ISW < 1);")
    P("   an ISW EXCESS needs P < 0, a portion that pools where matter is not. K < 0 reverses that.")
    P("   A_ISW at z = 0.5 for P = +0.05, +0.10, +0.20, +0.40 (n = 3 | n = 1.98 | n = 0):")
    for pv in (0.05, 0.10, 0.20, 0.40):
        P(f"      P = {pv:+.2f}: " + " | ".join(f"{amps(pv, n, 0.5)['A_ISW']:+.2f}" for n in (3.0, 1.98, 0.0))
          + "     A_kappa: " + " | ".join(f"{amps(pv, n, 0.5)['A_kappa']:.3f}" for n in (3.0, 1.98, 0.0)))
    for n in (3.0, 1.98):
        pz = solve_P(0.0, n, 0.5, "A_ISW", lo=0.0, hi=2.0)
        P(f"   the ISW signal changes SIGN (voids read hot) above P = {pz:+.3f} at z = 0.5, n = {n}")

    P("\n3. WHAT THE PUBLISHED AMPLITUDES ASK OF P (z = 0.5 unless said)")
    req = {}
    for n in (3.0, 1.98):
        for tgt in (2.0, 5.2, 4.1):
            req[(n, tgt)] = solve_P(tgt, n, 0.5, "A_ISW", lo=-3.0, hi=0.0)
        P(f"   n = {n}: A_ISW = 2 needs P = {req[(n, 2.0)]:+.3f};  5.2 needs {req[(n, 5.2)]:+.3f};  4.1 needs {req[(n, 4.1)]:+.3f}")
    for n in (3.0, 1.98):
        k = PUB["kov22"]
        pk = (k[1] - 1) / eps_at(1.0, n, 0.5); spk = k[2] / eps_at(1.0, n, 0.5)
        P(f"   n = {n}: DES void lensing A_kappa = {k[1]} +- {k[2]} reads P = {pk:+.3f} +- {spk:.3f}")
        s = PUB["sart"]
        ps = (s[1] - 1) / eps_at(1.0, n, 0.7); sps = s[2] / eps_at(1.0, n, 0.7)
        P(f"   n = {n}: DESI LRG void lensing A_kappa = {s[1]} +- {s[2]} (z ~ 0.7) reads P = {ps:+.3f} +- {sps:.3f}; |P| < {abs(ps) + 2 * sps:.3f} at two sigma")
        pl = PUB["plk"]
        lo = solve_P(pl[1] + 2 * pl[2], n, 0.5, "A_ISW", lo=-3.0, hi=0.0)
        hi = solve_P(pl[1] - 2 * pl[2], n, 0.5, "A_ISW", lo=0.0, hi=3.0)
        P(f"   n = {n}: the all-sky cross-correlation A = {pl[1]} +- {pl[2]} allows {lo:+.3f} < P < {hi:+.3f} at two sigma")
        hv = PUB["hangv"]
        ph = solve_P(hv[1], n, 0.5, "A_ISW", lo=0.0, hi=3.0)
        P(f"   n = {n}: DESI Legacy voids A_ISW = {hv[1]} +- {hv[2]} read as this mechanism is P = {ph:+.3f} (positive: the portion follows matter)")

    P("\n4. ONE MECHANISM, TWO AMPLITUDES: does the lensing reading predict the ISW reading on the same voids?")
    for label, kk, ii in (("DES supervoids", "kov22", "kov19"), ("DES supervoids (DES alone)", "kov22", "kov19d"),
                          ("DESI Legacy superstructures", "hangk", "hangv")):
        ak, sk = PUB[kk][1], PUB[kk][2]; ai, si = PUB[ii][1], PUB[ii][2]
        for n in (3.0, 1.98):
            p0 = (ak - 1) / eps_at(1.0, n, 0.5)
            pred = amps(p0, n, 0.5)["A_ISW"]
            dp = sk / eps_at(1.0, n, 0.5)
            spred = abs(amps(p0 + dp, n, 0.5)["A_ISW"] - amps(p0 - dp, n, 0.5)["A_ISW"]) / 2
            pull = (ai - pred) / np.hypot(si, spred)
            P(f"   {label:30s} n = {n}: A_kappa {ak} +- {sk} -> predicted A_ISW {pred:+.2f} +- {spred:.2f}; measured {ai:+.2f} +- {si}; apart by {pull:+.2f} sigma")
    P("   quadrant test (the sign alone, K > 0): A_kappa - 1 and A_ISW - 1 must have OPPOSITE signs")
    for label, kk, ii in (("DES supervoids", "kov22", "kov19"), ("DESI Legacy", "hangk", "hangv")):
        a, b = PUB[kk][1] - 1, PUB[ii][1] - 1
        P(f"      {label}: ({a:+.3f}, {b:+.2f}) -> {'opposite, as the mechanism requires' if a * b < 0 else 'SAME sign: not this mechanism'}")

    P("\n5. GROWTH AND S8 (same early normalisation; lensing reads the potential at the lenses, z ~ 0.4)")
    for n in (3.0, 1.98):
        for pv in (-0.10, -0.03, 0.03, 0.10):
            m0 = amps(pv, n, 0.0); ml = amps(pv, n, 0.4)
            P(f"   n = {n}, P = {pv:+.2f}: sigma8 of matter today x{m0['growth_ratio']:.4f};  lensing-read amplitude x{ml['growth_ratio'] * ml['A_kappa']:.4f}")
    s8_des, e_des, s8_cmb, e_cmb = 0.789, 0.012, 0.832, 0.013
    off = s8_des / s8_cmb - 1
    eoff = abs(s8_des / s8_cmb) * np.hypot(e_des / s8_des, e_cmb / s8_cmb)
    P(f"   DES Y6 3x2pt {s8_des} +- {e_des} against the CMB's {s8_cmb} +- {e_cmb} (the CMB value from memory): offset {100 * off:+.1f} % +- {100 * eoff:.1f} %")
    for n in (3.0, 1.98):
        f = lambda p: amps(p, n, 0.4)["growth_ratio"] * amps(p, n, 0.4)["A_kappa"] - (1 + off)
        a_, b_ = -1.0, 0.0
        for _ in range(34):
            m = 0.5 * (a_ + b_)
            if f(a_) * f(m) <= 0: b_ = m
            else: a_ = m
        pS = 0.5 * (a_ + b_)
        dpp = 1e-3
        sens = (amps(dpp, n, 0.4)["growth_ratio"] * amps(dpp, n, 0.4)["A_kappa"] - 1) / dpp
        P(f"   n = {n}: that offset read as this mechanism is P = {pS:+.4f};  sensitivity d(amplitude)/dP = {sens:.3f};"
          f"  one-sigma reach of a {100 * eoff:.1f} % measurement: |P| = {eoff / sens:.4f}")

    P("\n6. LENSING OF THE CMB (A_L) AND THE LOW MULTIPOLES")
    for n in (3.0, 1.98):
        e2 = eps_at(1.0, n, 2.0); e1 = eps_at(1.0, n, 1.0)
        P(f"   n = {n}: eps per unit P at the lensing kernel's z ~ 2: {e2:.4f} (z ~ 1: {e1:.4f}); A_L - 1 ~ 2 eps: "
          f"Planck 2018's 1.180 would need P ~ {0.09 / e2:+.1f} (z ~ 2) or {0.09 / e1:+.2f} (z ~ 1); ACT DR6's 1.013 +- 0.023 allows |P| < {(0.0065 + 0.023) / e1:.2f} (z ~ 1)")
    for n in (3.0, 1.98):
        P(f"   n = {n}: late ISW amplitude at z = 0.3 for P = +0.02, +0.05, +0.10: "
          + ", ".join(f"{amps(pv, n, 0.3)['A_ISW']:+.2f}" for pv in (0.02, 0.05, 0.10))
          + "  (a portion that follows matter LOWERS the late ISW power at low multipoles)")

    P("\n7. THE PAYABLE SIZE: what P can be")
    caps = [("Landauer stock, largest self-consistent pairing", 1e-4), ("writer's budget, hydrogen burning", 2.7e-5),
            ("writer's budget, every gram of new stars to energy", 3.9e-3), ("Landauer stock, smallest", 10 ** -11.8)]
    P("   (a) large scales: the organised portion's fluctuation is its mean times the linear bias of what hosts it (mass conservation of the")
    P("       portion: however concentrated it is inside galaxies, its large-scale contrast is the hosts' bias), so b is 1 to 2, at most ~5 for clusters")
    for name, fcap in caps:
        for b in (1.0, 2.0, 5.0):
            pv = fcap * b
            P(f"       {name:52s} f = {fcap:.1e}, b = {b:.0f}: P = {pv:.1e}; eps today {pv * R0:.1e}; A_kappa - 1 at z = 0.5 (n = 3): {eps_at(pv, 3.0, 0.5):.1e}")
    P("   (b) in a supervoid the portion's own contrast is at most about one either way: ABSENT at the least (density cannot go below zero; that is")
    P("       P > 0, a higher hill, a WEAKER signal), doubled at the most if it avoids matter (it then fills the under-dense half of space; P < 0, a")
    P("       stronger signal). So |eps_void| <= f x (Omega_DE/Omega_m)(z) / |matter deficit|, whatever b is called:")
    for dm in (0.2, 0.3, 0.5):
        for fcap in (1e-4, 3.9e-3, 0.2, 0.5):
            e = -fcap * R0 / (1.5 ** 3) / dm
            k = -K_slope(3.0, 0.5)
            P(f"       matter deficit {dm:.1f}, f = {fcap:.1e}: |eps_void| <= {abs(e):.2e} at z = 0.5;  |A_ISW - 1| <= {abs(k * e):.2e} (K = {k:.2f}, n = 3)")
    P("   (c) b required by each published amplitude at each share (n = 3, z = 0.5 for voids; S8 at z = 0.4)")
    asks = [("supervoid ISW excess 5.2", req[(3.0, 5.2)]), ("supervoid ISW excess 2", req[(3.0, 2.0)]),
            ("DES void lensing 0.79", (0.79 - 1) / eps_at(1.0, 3.0, 0.5)), ("S8 offset, DES Y6", None)]
    f_ = lambda p: amps(p, 3.0, 0.4)["growth_ratio"] * amps(p, 3.0, 0.4)["A_kappa"] - (1 + off)
    a_, b_ = -1.0, 0.0
    for _ in range(34):
        m = 0.5 * (a_ + b_)
        if f_(a_) * f_(m) <= 0: b_ = m
        else: a_ = m
    asks[3] = ("S8 offset, DES Y6", 0.5 * (a_ + b_))
    for name, pv in asks:
        P(f"       {name:28s} P = {pv:+.3f}:  b = " + ", ".join(f"{pv / fc:+.1e} (f = {fc:.1e})" for fc in (1e-4, 2.7e-5, 3.9e-3))
          + ";  " + ", ".join(f"{pv / fc:+.2f} (f = {fc})" for fc in (0.2, 0.4, 0.7)))

    P("\n8. THE LARGE SHARE THAT REPAIRS THE SHAPE, IF IT FOLLOWED MATTER (b = 1): already excluded")
    pl, sa = PUB["plk"], PUB["sart"]
    for fsh in (0.2, 0.4, 0.7):
        for n in (3.0, 1.98):
            m5, m7 = amps(fsh, n, 0.5), amps(fsh, n, 0.7)
            P(f"   f = {fsh}, b = 1, n = {n}: A_ISW(z = 0.5) = {m5['A_ISW']:+.2f} against {pl[1]} +- {pl[2]}: {(m5['A_ISW'] - pl[1]) / pl[2]:+.1f} sigma;"
              f"  A_kappa(z = 0.7) = {m7['A_kappa']:.3f} against {sa[1]} +- {sa[2]}: {(m7['A_kappa'] - sa[1]) / sa[2]:+.1f} sigma")
    for n in (3.0, 1.98):
        lo = solve_P(pl[1] + 2 * pl[2], n, 0.5, "A_ISW", lo=-3.0, hi=0.0); hi = solve_P(pl[1] - 2 * pl[2], n, 0.5, "A_ISW", lo=0.0, hi=3.0)
        P(f"   n = {n}: with {lo:+.3f} < P < {hi:+.3f}, a share of 0.2, 0.4, 0.7 may have |b| at most "
          + ", ".join(f"{max(abs(lo), abs(hi)) / fsh:.2f}" for fsh in (0.2, 0.4, 0.7)))

    P("\n8b. THE ONE SMALL WINDOW WHERE THE READINGS DO NOT CONTRADICT EACH OTHER: P near -0.05 (a portion that AVOIDS matter)")
    for pv in (-0.04, -0.05, -0.06):
        for n in (3.0, 1.98):
            m5, m4, m7 = amps(pv, n, 0.5), amps(pv, n, 0.4), amps(pv, n, 0.7)
            P(f"   P = {pv:+.2f}, n = {n}: A_ISW(z = 0.5) = {m5['A_ISW']:+.2f} ({(m5['A_ISW'] - 1.0) / 0.25:+.1f} sigma of the all-sky 1.00 +- 0.25; {(m5['A_ISW'] - 1.64) / 0.53:+.1f} of BOSS's 1.64 +- 0.53;"
              f" {(m5['A_ISW'] - 5.2) / 1.6:+.1f} of DES's 5.2 +- 1.6);  A_kappa(0.7) = {m7['A_kappa']:.3f} ({(m7['A_kappa'] - 1.016) / 0.054:+.1f} of DESI LRG voids);"
              f"  lensing-read amplitude x{m4['growth_ratio'] * m4['A_kappa']:.3f}")
    P("   at the shares that repair the shape that is b = P/f: " + ", ".join(f"{-0.05 / fsh:+.2f} (f = {fsh})" for fsh in (0.2, 0.4, 0.7)))

    P("\n9. THE STAKE'S SEPARATION, ON PAPER: the point (A_kappa, A_ISW) for one catalogue at z = 0.5")
    P("   smooth constant, or the organised portion at any payable size:  (1.000, +1.00)")
    for pv in (-0.40, -0.20, -0.10, 0.10, 0.20):
        r = [amps(pv, n, 0.5) for n in (3.0, 1.98)]
        P(f"   P = {pv:+.2f}: A_kappa = {r[0]['A_kappa']:.3f} | {r[1]['A_kappa']:.3f};  A_ISW = {r[0]['A_ISW']:+.2f} | {r[1]['A_ISW']:+.2f}   (n = 3 | n = 1.98)")
    k3, k2 = -K_slope(3.0, 0.5), -K_slope(1.98, 0.5)
    P(f"   the wedge: A_ISW - 1 = -K (A_kappa - 1) with K between {min(k2, k3):.2f} and {max(k2, k3):.2f} at z = 0.5 (n from 1.98 to 3)")
    P("   with sigma(A_kappa) = 0.06 and sigma(A_ISW) = 0.7 (the sizes of the DESI Legacy numbers above), a lensing deficit of 0.2 is 3.3 sigma,")
    for pv in (-0.20,):
        e = eps_at(1.0, 3.0, 0.5)
        ak = 0.8
        p3, p2 = (ak - 1) / eps_at(1.0, 3.0, 0.5), (ak - 1) / eps_at(1.0, 1.98, 0.5)
        P(f"   and at A_kappa = 0.80 the wedge predicts A_ISW between {amps(p2, 1.98, 0.5)['A_ISW']:+.2f} and {amps(p3, 3.0, 0.5)['A_ISW']:+.2f}:"
          f" {(amps(p2, 1.98, 0.5)['A_ISW'] - 1) / 0.7:.1f} to {(amps(p3, 3.0, 0.5)['A_ISW'] - 1) / 0.7:.1f} sigma from 1")
    P("   the published pairs scored on the stake's two statistics, AFTER THE FACT and so not counted:")
    for label, kk, ii in (("DES supervoids", "kov22", "kov19"), ("DESI Legacy", "hangk", "hangv")):
        P(f"      {label}: t_kappa = {abs(PUB[kk][1] - 1) / PUB[kk][2]:.2f}, t_ISW = {abs(PUB[ii][1] - 1) / PUB[ii][2]:.2f}")
    P(f"   if the all-sky amplitude's error were 0.4 and not 0.25, f = 0.2 with b = 1 (n = 3) would sit {(amps(0.2, 3.0, 0.5)['A_ISW'] - 1.0) / 0.4:+.1f} sigma away")

    P("\n10. HOW FAR UNDER, CHANNEL BY CHANNEL, AT THE LARGEST LANDAUER SHARE (f = 1e-4)")
    P(f"   background: 1 + w = 3.6e-05 against sigma(w0) = 0.055 on the record (COSMOS_TWO_PORTION.md): {0.055 / 3.6e-5:,.0f} times under")
    for n in (3.0, 1.98):
        dpp = 1e-3
        sens = (amps(dpp, n, 0.4)["growth_ratio"] * amps(dpp, n, 0.4)["A_kappa"] - 1) / dpp
        reach = eoff / sens
        for bb in (1.0, 2.0):
            P(f"   clustering, n = {n}, b = {bb:.0f}: P = {1e-4 * bb:.0e} against a one-sigma reach of {reach:.4f}: {reach / (1e-4 * bb):,.0f} times under;"
              f"  hydrogen-burning share: {reach / (2.7e-5 * bb):,.0f};  every-gram share: {reach / (3.9e-3 * bb):.1f}")
        P(f"   n = {n}: a future 0.5 % amplitude measurement reaches |P| = {0.005 / sens:.4f}")
    P("\ndone.")


if __name__ == "__main__":
    main()
