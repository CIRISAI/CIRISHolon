"""
curve.py -- grids, derivatives, minima, the piecewise cubic Hermite renderer
contract and the timestep envelope.  Schema-compatible with the banked
h2_potential.json, extended with per-species provenance.

Derivatives.  Two routes, and both are reported:
  * a local high-order Newton divided-difference interpolant through a window of
    computed grid points, differentiated analytically;
  * a central finite-difference stencil evaluated at RAISED precision, which
    costs extra energy evaluations and is therefore used everywhere for the
    cheap species and as a spot cross-check for the expensive ones.
The measured difference between the two is reported per species as
force_interpolant_vs_fd; on H2 both are additionally checked against the banked
referee's own mpmath.diff values.
"""

from mpmath import mp, mpf, sqrt, nstr, pi


# ---------------------------------------------------------------------------
# Grids.  Uniform in u = R^{-1/4}: the cubic Hermite error over an interval goes
# like |E''''| h^4 / 384, and over most of the range E'''' is dominated by the
# nuclear repulsion's d^4(1/R)/dR^4 = 24 R^{-5}; equidistributing h^4 R^{-5}
# needs h proportional to R^{5/4}, which is exactly uniform spacing in R^{-1/4}.
# The well window is refined once more up front.
# ---------------------------------------------------------------------------
GRID_DECIMALS = 12
STENCIL_DECIMALS = 11          # the stencil step is 10^-STENCIL_DECIMALS

# ---------------------------------------------------------------------------
# GEOMETRIES TRAVEL AS EXACT DECIMAL STRINGS, never as binary floats.
#
# A grid is a design choice, so nothing is lost by putting its points on a short
# decimal lattice -- and a great deal is gained.  A binary mpf built at dps 60
# is not the same number as the one a reader gets by parsing its printed form
# at dps 95, so a float-valued grid silently means different geometries in
# different parts of the pipeline, and the published R would not be the R the
# energy was computed at.  Strings are re-materialised at whatever precision is
# active, exactly as h2_core stores its basis constants.
#
# (The banked h2_potential.json does not do this: its stored R is rounded to 50
# digits while its E was evaluated at the unrounded value.  That, plus the file's
# own rounding, is why its stored E strings sit about 5e-50 from what h2_core
# itself computes -- see the R2 discussion in the report.)
# ---------------------------------------------------------------------------
def dec_str(x, dec=GRID_DECIMALS):
    """The exact decimal string for x, rounded to `dec` places."""
    x = mpf(x)
    n = int(mp.floor(abs(x) * mpf(10) ** dec + mpf("0.5")))
    s = str(n).rjust(dec + 1, "0")
    return ("-" if x < 0 else "") + s[:-dec] + "." + s[-dec:]


def dec_shift(s, k, unit_dec=STENCIL_DECIMALS):
    """s + k * 10^-unit_dec, exactly, in decimal string arithmetic."""
    neg = s.startswith("-")
    ip, fp = s.lstrip("-").split(".")
    dec = len(fp)
    if unit_dec > dec:
        raise ValueError("shift finer than the string's resolution")
    n = int(ip + fp) * (-1 if neg else 1)
    n += k * 10 ** (dec - unit_dec)
    out = str(abs(n)).rjust(dec + 1, "0")
    return ("-" if n < 0 else "") + out[:-dec] + "." + out[-dec:]


def dec_lerp(a, b, t, dec=GRID_DECIMALS):
    return dec_str(mpf(a) + mpf(t) * (mpf(b) - mpf(a)), dec)


def build_grid(rmin, rmax, nbase, well=None, nsplit=1):
    """Returns the grid as a sorted list of exact decimal STRINGS."""
    rmin, rmax = mpf(rmin), mpf(rmax)
    u_hi = rmin ** mpf("-0.25")
    u_lo = rmax ** mpf("-0.25")
    us = [u_hi + (u_lo - u_hi) * mpf(i) / (nbase - 1) for i in range(nbase)]
    base = [u ** mpf(-4) for u in us]
    base[0], base[-1] = rmin, rmax
    grid = []
    for i, r in enumerate(base):
        grid.append(r)
        if i + 1 < len(base) and well is not None:
            a, b = r, base[i + 1]
            if a >= mpf(well[0]) and b <= mpf(well[1]):
                for k in range(1, nsplit + 1):
                    grid.append(a + (b - a) * mpf(k) / (nsplit + 1))
    return sorted(set(dec_str(r) for r in grid), key=lambda s: mpf(s))


# ---------------------------------------------------------------------------
# Newton divided-difference interpolant and its first two derivatives.
#   P(x) = sum_i c_i N_i(x),  N_i(x) = prod_{j<i} (x - x_j)
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
    return lo, min(lo + 2 * half + 1, n)


def interpolant_derivs(grid, vals, half=6):
    """(E', E'') at every grid point from a local Newton interpolant."""
    n = len(grid)
    half = min(half, (n - 1) // 2)
    d1, d2 = [], []
    for i in range(n):
        lo, hi = window(i, n, half)
        xs, ys = grid[lo:hi], vals[lo:hi]
        cs = newton_coeffs(xs, ys)
        _, p1, p2 = newton_derivs(xs, cs, grid[i])
        d1.append(p1)
        d2.append(p2)
    return d1, d2


# ---------------------------------------------------------------------------
# Central finite differences at raised precision.  With working precision
# 10^-P and step h the first derivative from the 8th-order stencil carries a
# truncation error ~ h^8 |E^(9)| / 630 and a roundoff error ~ 10^-P / h; the
# second derivative's roundoff is ~ 10^-P / h^2.  h = 10^-(P/12) balances them
# and leaves far more than 50 digits.
# ---------------------------------------------------------------------------
# The weights are stored as exact RATIONALS and divided at the working
# precision inside fd_derivs, never as mpf constants at module level.  An
# mpmath constant built at import time carries whatever mp.dps happened to be
# then -- 15 by default -- and stays that inaccurate however high the precision
# is raised afterwards.  Written as mpf constants, these weights made the second
# derivative wrong by six orders of magnitude while every energy feeding it was
# right to 90 digits; the closed-form stencil test in test_fci.py exists to
# catch exactly that.  Same reason h2_core.py stores its basis as strings.
_D1_W = ((-4, 1, 280), (-3, -4, 105), (-2, 1, 5), (-1, -4, 5),
         (1, 4, 5), (2, -1, 5), (3, 4, 105), (4, -1, 280))
_D2_W = ((-4, -1, 560), (-3, 8, 315), (-2, -1, 5), (-1, 8, 5), (0, -205, 72),
         (1, 8, 5), (2, -1, 5), (3, 8, 315), (4, -1, 560))

# The value error of a cubic Hermite peaks at t = 1/2 and the derivative error
# at t = 1/2 +/- 1/(2 sqrt 3); both extrema are sampled exactly.
def hermite_test_ts():
    q = 1 / (2 * mp.sqrt(3))
    return [mpf(1) / 2 - q, mpf(1) / 2, mpf(1) / 2 + q]


def stencil_step():
    """The central-difference step, materialised at the working precision."""
    return mpf(10) ** -STENCIL_DECIMALS


def stencil_weight_sums():
    """(sum|w| for the first-derivative stencil, and for the second).

    A finite difference amplifies whatever uncertainty the FUNCTION VALUES
    carry: an energy known to eps yields a first derivative known only to
    sum|w| * eps / h and a second to sum|w| * eps / h^2.  With h = 1e-11 those
    are factors of 2e11 and 6.5e22, so the derivative columns are nowhere near
    as well determined as the energies they come from, and saying so is the
    whole point of shipping an uncertainty with them.
    """
    a = sum(abs(mpf(n) / d) for _, n, d in _D1_W)
    b = sum(abs(mpf(n) / d) for _, n, d in _D2_W)
    return a, b


def stencil_derivative_bounds(energy_bound, h=None):
    """(F bound, E2 bound) implied by an energy uncertainty of energy_bound."""
    if h is None:
        h = stencil_step()
    h = mpf(h)
    a, b = stencil_weight_sums()
    e = mpf(energy_bound)
    return a * e / h, b * e / (h * h)


def fd_derivs(vals9, h=None):
    """(E', E'') from the nine energies at R-4h .. R+4h, 8th order central."""
    if h is None:
        h = stencil_step()
    h = mpf(h)
    m = {k: vals9[k + 4] for k in range(-4, 5)}
    d1 = sum(mpf(a) / b * m[k] for k, a, b in _D1_W) / h
    d2 = sum(mpf(a) / b * m[k] for k, a, b in _D2_W) / (h * h)
    return d1, d2


# ---------------------------------------------------------------------------
# Minimum location, by bisection on E' inside the bracketing grid interval.
# ---------------------------------------------------------------------------
def count_extrema(d1, skip_ends=0):
    """Sign changes of E' between consecutive grid points.

    skip_ends drops the outermost interval at each end.  A local interpolant is
    at its worst at the first and last knot, where the evaluation point sits on
    the edge of its window rather than inside it; in the dissociation tail, where
    E' is itself of the size of that error, an endpoint can carry the wrong sign
    and manufacture an extremum that is not in the model.  (Measured on H2: the
    interpolant reads E'(10 bohr) = -1.5e-7 where the raised-precision stencil
    reads +3.8e-8.)"""
    mins = maxs = 0
    lo, hi = skip_ends, len(d1) - 1 - skip_ends
    for i in range(lo, hi):
        if d1[i] < 0 and d1[i + 1] > 0:
            mins += 1
        if d1[i] > 0 and d1[i + 1] < 0:
            maxs += 1
    return mins, maxs


# ---------------------------------------------------------------------------
# Piecewise cubic Hermite (the renderer contract), identical in form to the bank
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


# ---------------------------------------------------------------------------
# Timestep envelope.  The renderer must set dt from the stiffest CURVATURE it
# can reach at its current maximum pair energy, not from E'' at the minimum.
# ---------------------------------------------------------------------------
def turning_points(Efun, Etot, Rmin, Rmax, Rref, Easym, btol=None):
    if btol is None:
        btol = mpf(10) ** (-(mp.dps - 12))

    def f(R):
        return Efun(R) - Etot

    if f(Rmin) <= 0:
        return None, None, "inner_beyond"
    lo, hi = Rmin, Rref
    while hi - lo > btol:
        mid = (lo + hi) / 2
        if f(mid) > 0:
            lo = mid
        else:
            hi = mid
    R_in = (lo + hi) / 2
    if Easym is not None and Etot >= Easym:
        return R_in, None, "unbound"
    if f(Rmax) <= 0:
        return R_in, None, "outer_beyond"
    lo, hi = Rref, Rmax
    while hi - lo > btol:
        mid = (lo + hi) / 2
        if f(mid) < 0:
            lo = mid
        else:
            hi = mid
    return R_in, (lo + hi) / 2, "bound"


def build_envelope(grid, Evals, E2vals, Efun, Rref, Easym, rungs=None,
                   nsteps=64):
    """One rung per total pair energy: the maximum |E''| reachable at it.

    dt = 2 pi / (nsteps * sqrt(max|E''| / mu)); the tabulated dt_per_sqrt_mu
    carries no mass constant, because the electronic model is Born-Oppenheimer
    and knows no nuclear mass.
    """
    Emin = min(Evals)
    Etop = max(Evals)
    if rungs is None:
        eps = [mpf(0)]
        e = mpf("1e-6")
        while e <= (Etop - Emin):
            eps.append(e)
            e *= mpf(10) ** mpf("0.125")
        rungs = eps
    out = []
    for de in rungs:
        Etot = Emin + de
        if Etot > Etop:
            continue
        R_in, R_out, flag = turning_points(Efun, Etot, grid[0], grid[-1], Rref,
                                           Easym)
        if R_in is None:
            continue
        acc = [i for i in range(len(grid))
               if Evals[i] <= Etot and grid[i] >= R_in]
        if not acc:
            continue
        best = max(acc, key=lambda i: abs(E2vals[i]))
        m = abs(E2vals[best])
        out.append(dict(eps=de, Etot=Etot, R_in=R_in, R_out=R_out, flag=flag,
                        maxE2=m, argmax=grid[best],
                        dt_per_sqrt_mu=(2 * pi / (nsteps * sqrt(m)))
                        if m > 0 else None))
    return out


# Removed as superseded, not as clutter: `find_minimum`, `hermite_errors` and
# `fd_offsets` were second implementations of quantities the pipeline publishes
# -- the minimum comes from `newton_minimum` in build_curves, the Hermite error
# is measured inline at the probe points, and the stencil offsets come from
# `dec_shift`.  A spare implementation of a published number, tested by nothing
# and run by nothing, is a claim about the model that nobody has checked.
