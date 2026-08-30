"""
elements_core.py -- general STO-3G (s and p) integral engine at >= 50 digits,
from first principles, by the McMurchie-Davidson (Hermite Gaussian) recursions.

DERIVATION (closed-form mathematics, implemented here from the formulae):
  Gaussian product theorem; the Hermite expansion coefficients E_t^{ij} and
  their two-term recursions; the auxiliary Hermite-Coulomb integrals R^n_{tuv}
  and their downward-in-n recursions; the Boys function F_n.  Overlap, kinetic,
  nuclear attraction and two-electron repulsion follow.  Every recursion is
  written out in the docstring of the function that implements it.

MODEL DEFINITION (an input, not a derivation):
  The STO-3G contraction for H..Ne -- exponents and coefficients below.  A basis
  set IS a model choice.  The decimals are the published STO-3G table (Basis Set
  Exchange, "Data from Gaussian09", revision 2018-06-19) ROUNDED TO 8 DECIMAL
  PLACES, which is the convention of the banked H2 referee (h2_core.py): its
  hydrogen row is reproduced here character for character, so the H2 regression
  is an identity and not an approximation.  The full published 10-significant-
  digit values are retained below as STO3G_SHELLS_FULL for provenance and for
  the basis-definition sensitivity diagnostic; they are NOT used in any gate.

No pyscf / psi4 / any quantum chemistry package is imported.  Arithmetic is
mpmath at the ambient working precision, so this file is safe to call from
mpmath.diff (which raises mp.prec internally).
"""

import math
from mpmath import mp, mpf, sqrt, exp, erf, pi, gammainc, gamma

STO3G_SHELLS = {
    1: (  # H
        (0, ("3.42525091", "0.62391373", "0.16885540"), ("0.15432897", "0.53532814", "0.44463454")),
    ),
    2: (  # He
        (0, ("6.36242139", "1.15892300", "0.31364979"), ("0.15432897", "0.53532814", "0.44463454")),
    ),
    3: (  # Li
        (0, ("16.11957475", "2.93620066", "0.79465049"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("0.63628975", "0.14786005", "0.04808868"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("0.63628975", "0.14786005", "0.04808868"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    4: (  # Be
        (0, ("30.16787069", "5.49511531", "1.48719265"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("1.31483311", "0.30553894", "0.09937075"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("1.31483311", "0.30553894", "0.09937075"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    5: (  # B
        (0, ("48.79111318", "8.88736217", "2.40526704"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("2.23695614", "0.51982050", "0.16906176"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("2.23695614", "0.51982050", "0.16906176"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    6: (  # C
        (0, ("71.61683735", "13.04509632", "3.53051216"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("2.94124936", "0.68348310", "0.22228992"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("2.94124936", "0.68348310", "0.22228992"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    7: (  # N
        (0, ("99.10616896", "18.05231239", "4.88566024"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("3.78045588", "0.87849664", "0.28571437"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("3.78045588", "0.87849664", "0.28571437"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    8: (  # O
        (0, ("130.70932140", "23.80886605", "6.44360831"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("5.03315132", "1.16959612", "0.38038896"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("5.03315132", "1.16959612", "0.38038896"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    9: (  # F
        (0, ("166.67913400", "30.36081233", "8.21682067"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("6.46480325", "1.50228124", "0.48858849"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("6.46480325", "1.50228124", "0.48858849"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
    10: (  # Ne
        (0, ("207.01560700", "37.70815124", "10.20529731"), ("0.15432897", "0.53532814", "0.44463454")),
        (0, ("8.24631512", "1.91626629", "0.62322927"), ("-0.09996723", "0.39951283", "0.70011547")),
        (1, ("8.24631512", "1.91626629", "0.62322927"), ("0.15591627", "0.60768372", "0.39195739")),
    ),
}

# full published values (10 significant digits), kept for provenance
STO3G_SHELLS_FULL = {
    1: (
        (0, ('0.3425250914E+01', '0.6239137298E+00', '0.1688554040E+00'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
    ),
    2: (
        (0, ('0.6362421394E+01', '0.1158922999E+01', '0.3136497915E+00'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
    ),
    3: (
        (0, ('0.1611957475E+02', '0.2936200663E+01', '0.7946504870E+00'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.6362897469E+00', '0.1478600533E+00', '0.4808867840E-01'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.6362897469E+00', '0.1478600533E+00', '0.4808867840E-01'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    4: (
        (0, ('0.3016787069E+02', '0.5495115306E+01', '0.1487192653E+01'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.1314833110E+01', '0.3055389383E+00', '0.9937074560E-01'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.1314833110E+01', '0.3055389383E+00', '0.9937074560E-01'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    5: (
        (0, ('0.4879111318E+02', '0.8887362172E+01', '0.2405267040E+01'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.2236956142E+01', '0.5198204999E+00', '0.1690617600E+00'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.2236956142E+01', '0.5198204999E+00', '0.1690617600E+00'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    6: (
        (0, ('0.7161683735E+02', '0.1304509632E+02', '0.3530512160E+01'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.2941249355E+01', '0.6834830964E+00', '0.2222899159E+00'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.2941249355E+01', '0.6834830964E+00', '0.2222899159E+00'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    7: (
        (0, ('0.9910616896E+02', '0.1805231239E+02', '0.4885660238E+01'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.3780455879E+01', '0.8784966449E+00', '0.2857143744E+00'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.3780455879E+01', '0.8784966449E+00', '0.2857143744E+00'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    8: (
        (0, ('0.1307093214E+03', '0.2380886605E+02', '0.6443608313E+01'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.5033151319E+01', '0.1169596125E+01', '0.3803889600E+00'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.5033151319E+01', '0.1169596125E+01', '0.3803889600E+00'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    9: (
        (0, ('0.1666791340E+03', '0.3036081233E+02', '0.8216820672E+01'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.6464803249E+01', '0.1502281245E+01', '0.4885884864E+00'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.6464803249E+01', '0.1502281245E+01', '0.4885884864E+00'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
    10: (
        (0, ('0.2070156070E+03', '0.3770815124E+02', '0.1020529731E+02'), ('0.1543289673E+00', '0.5353281423E+00', '0.4446345422E+00')),
        (0, ('0.8246315120E+01', '0.1916266291E+01', '0.6232292721E+00'), ('-0.9996722919E-01', '0.3995128261E+00', '0.7001154689E+00')),
        (1, ('0.8246315120E+01', '0.1916266291E+01', '0.6232292721E+00'), ('0.1559162750E+00', '0.6076837186E+00', '0.3919573931E+00')),
    ),
}

ELEMENT_SYMBOL = {1: "H", 2: "He", 3: "Li", 4: "Be", 5: "B",
                  6: "C", 7: "N", 8: "O", 9: "F", 10: "Ne"}

# Nuclear masses: DECLARED INPUTS (measured, like m_e).  Atomic mass of the most
# abundant isotope, in unified atomic mass units, CODATA/AME2020 values.  They
# enter NOTHING electronic -- the model is Born-Oppenheimer and knows no nuclear
# mass -- and are carried only so the renderer contract can form a reduced mass.
ISOTOPE_MASS_U = {
    1: "1.00782503207", 2: "4.00260325415", 3: "7.01600343666",
    4: "9.01218306500", 5: "11.00930536000", 6: "12.00000000000",
    7: "14.00307400443", 8: "15.99491461957", 9: "18.99840316273",
    10: "19.99244017617",
}
AMU_IN_ELECTRON_MASSES = "1822.888486209"

MODEL_NAME = "ELEMENTS1/STO-3G/FCI"


def basis_fingerprint(table=None):
    """A hash of the ENTIRE declared basis table.

    Cached energies are keyed by species and geometry, not by the model, so
    changing one exponent would otherwise leave every stale result in place and
    silently mix two models in one artifact.  Every cache record carries this,
    and a record whose fingerprint does not match the table in force is refused.
    """
    import hashlib
    t = STO3G_SHELLS if table is None else table
    h = hashlib.sha256()
    for Z in sorted(t):
        h.update(b"Z%d" % Z)
        for (l, exps, coefs) in t[Z]:
            h.update(b"|%d|" % l)
            h.update("|".join(exps).encode())
            h.update(b"|")
            h.update("|".join(coefs).encode())
    return h.hexdigest()[:16]


# ===========================================================================
# Boys function  F_n(t) = int_0^1 u^{2n} exp(-t u^2) du
# ===========================================================================
def boys0(t):
    """F_0(t) = sqrt(pi/(4t)) erf(sqrt(t)), t>0; 1 at t=0.

    Reproduced VERBATIM from the banked H2 referee h2_core.boys0 so that every
    s-only integral this engine computes is bit-comparable with the bank.
    """
    t = mpf(t)
    if t == 0:
        return mpf(1)
    if t < mpf(10) ** (-(mp.dps // 2 + 4)):
        s = mpf(0)
        term = mpf(1)
        n = 0
        while True:
            contrib = term / (2 * n + 1)
            s += contrib
            if abs(contrib) < mpf(10) ** (-(mp.dps + 10)):
                break
            n += 1
            term *= -t / n
        return s
    return sqrt(pi / (4 * t)) * erf(sqrt(t))


def boys_series(n, t):
    """F_n(t) = sum_{k>=0} (-t)^k / (k! (2n+2k+1)); absolutely convergent.

    The alternating terms peak near k = t and the result decays like
    t^{-(n+1/2)}, so the cancellation can be far worse than the naive e^t
    estimate.  The working precision is therefore raised by a guard that is
    MEASURED from the run itself (max|term| / |sum|) and the sum repeated if the
    first guess was short.  Only the test and seeding paths reach this function
    at large t; boys() uses it only for t < 1/2, where there is no cancellation.
    """
    n, t = int(n), mpf(t)
    prec0 = mp.dps
    guard = int(float(t) / 2.302585093) + 20
    for _ in range(6):
        mp.dps = prec0 + guard
        try:
            tt = mpf(t)
            s = mpf(0)
            term = mpf(1)
            mx = mpf(0)
            k = 0
            cut = mpf(10) ** (-(prec0 + guard - 5))
            while True:
                contrib = term / (2 * n + 2 * k + 1)
                s += contrib
                mx = max(mx, abs(contrib))
                if abs(contrib) < cut * max(abs(s), mpf(1)) and k > int(t) + 2:
                    break
                k += 1
                term *= -tt / k
                if k > 200000:
                    raise RuntimeError("boys_series failed to converge")
            need = 0 if s == 0 else int(mp.log(mx / abs(s)) / mp.log(10)) + 25
        finally:
            mp.dps = prec0
        if need <= guard:
            return +s
        guard = need + 10
    raise RuntimeError("boys_series guard did not settle")


def boys_gamma(n, t):
    """F_n(t) = gamma(n+1/2, t) / (2 t^{n+1/2}), the lower incomplete gamma."""
    n, t = int(n), mpf(t)
    return gammainc(mpf(n) + mpf(1) / 2, 0, t) / (2 * t ** (mpf(n) + mpf(1) / 2))


def boys(n, t):
    """F_n at the working precision.  n=0 takes the banked closed form exactly.

    For n>0 the series is used where it converges quickly (small t) and the
    incomplete-gamma closed form otherwise.  boys_down() is an independent
    cross-check (see test_integrals.py).
    """
    n = int(n)
    t = mpf(t)
    if n == 0:
        return boys0(t)
    if t == 0:
        return mpf(1) / (2 * n + 1)
    if t < mpf(1) / 2:
        return boys_series(n, t)
    return boys_gamma(n, t)


def boys_down(nmax, t):
    """All of F_0..F_nmax by the stable DOWNWARD recursion, an independent route.

        F_n(t) = (2 t F_{n+1}(t) + exp(-t)) / (2n + 1)

    Seeded at n = nmax + 24 (where the seed's error is damped by the product of
    the 1/(2n+1) factors below it).  The seed is the absolutely convergent
    series, or -- once exp(-t) has fallen below the working precision, so that
    the incomplete gamma IS the complete one -- Gamma(n+1/2) / (2 t^{n+1/2}).
    Neither seed calls gammainc, so this route is independent of boys_gamma.
    """
    nmax, t = int(nmax), mpf(t)
    if t == 0:
        return [mpf(1) / (2 * n + 1) for n in range(nmax + 1)]
    top = nmax + 24
    if t > mp.dps * 2.302585093 + 60:
        f = gamma(mpf(top) + mpf(1) / 2) / (2 * t ** (mpf(top) + mpf(1) / 2))
    else:
        f = boys_series(top, t)
    et = exp(-t)
    out = [None] * (top + 1)
    out[top] = f
    for n in range(top - 1, -1, -1):
        f = (2 * t * f + et) / (2 * n + 1)
        out[n] = f
    return out[: nmax + 1]


# ===========================================================================
# McMurchie-Davidson Hermite expansion coefficients, one Cartesian direction.
#
#   x_A^i x_B^j exp(-a x_A^2) exp(-b x_B^2) = sum_{t=0}^{i+j} E_t^{ij} Lambda_t(p, x_P)
#
# with Lambda_t the t-th Hermite Gaussian about P.  Recursions (Helgaker-Taylor):
#   E_0^{00}   = exp(-mu X_AB^2),          mu = ab/p,  p = a+b
#   E_t^{i+1,j} = (1/(2p)) E_{t-1}^{ij} + X_PA E_t^{ij} + (t+1) E_{t+1}^{ij}
#   E_t^{i,j+1} = (1/(2p)) E_{t-1}^{ij} + X_PB E_t^{ij} + (t+1) E_{t+1}^{ij}
#   E_t^{ij}   = 0 for t < 0 or t > i+j
# ===========================================================================
def hermite_E(i, j, a, b, Ax, Bx):
    """Return the list [E_0^{ij}, ..., E_{i+j}^{ij}] for one direction."""
    a, b, Ax, Bx = mpf(a), mpf(b), mpf(Ax), mpf(Bx)
    p = a + b
    mu = a * b / p
    Px = (a * Ax + b * Bx) / p
    XPA, XPB, XAB = Px - Ax, Px - Bx, Ax - Bx
    half_p = 1 / (2 * p)

    cur = [exp(-mu * XAB * XAB)]            # E^{00}
    for _ in range(i):                      # raise i
        nxt = [mpf(0)] * (len(cur) + 1)
        for t in range(len(cur)):
            v = cur[t]
            if v == 0:
                continue
            nxt[t + 1] += half_p * v
            nxt[t] += XPA * v
            if t >= 1:
                nxt[t - 1] += t * v
        cur = nxt
    for _ in range(j):                      # raise j
        nxt = [mpf(0)] * (len(cur) + 1)
        for t in range(len(cur)):
            v = cur[t]
            if v == 0:
                continue
            nxt[t + 1] += half_p * v
            nxt[t] += XPB * v
            if t >= 1:
                nxt[t - 1] += t * v
        cur = nxt
    return cur[: i + j + 1]


# ===========================================================================
# Hermite-Coulomb auxiliary integrals
#   R^n_{tuv}(alpha, P-C) with R^n_{000} = (-2 alpha)^n F_n(alpha |P-C|^2)
#   R^n_{t+1,u,v} = t R^{n+1}_{t-1,u,v} + X R^{n+1}_{t,u,v}
#   R^n_{t,u+1,v} = u R^{n+1}_{t,u-1,v} + Y R^{n+1}_{t,u,v}
#   R^n_{t,u,v+1} = v R^{n+1}_{t,u,v-1} + Z R^{n+1}_{t,u,v}
# Returns a dict {(t,u,v): R^0_{tuv}} for all t+u+v <= L.
# ===========================================================================
def hermite_R(L, alpha, PCx, PCy, PCz):
    L = int(L)
    alpha = mpf(alpha)
    X, Y, Z = mpf(PCx), mpf(PCy), mpf(PCz)
    T = alpha * (X * X + Y * Y + Z * Z)
    Fn = [boys(n, T) for n in range(L + 1)]
    m2a = -2 * alpha
    R = {}
    pw = mpf(1)
    for n in range(L + 1):
        R[(n, 0, 0, 0)] = pw * Fn[n]
        pw *= m2a
    for tot in range(1, L + 1):
        for t in range(tot + 1):
            for u in range(tot - t + 1):
                v = tot - t - u
                for n in range(L - tot + 1):
                    if t > 0:
                        val = X * R[(n + 1, t - 1, u, v)]
                        if t > 1:
                            val += (t - 1) * R[(n + 1, t - 2, u, v)]
                    elif u > 0:
                        val = Y * R[(n + 1, t, u - 1, v)]
                        if u > 1:
                            val += (u - 1) * R[(n + 1, t, u - 2, v)]
                    else:
                        val = Z * R[(n + 1, t, u, v - 1)]
                        if v > 1:
                            val += (v - 1) * R[(n + 1, t, u, v - 2)]
                    R[(n, t, u, v)] = val
    return {(t, u, v): R[(0, t, u, v)]
            for t in range(L + 1) for u in range(L + 1 - t)
            for v in range(L + 1 - t - u)}


# ===========================================================================
# Cartesian Gaussian primitive normalisation
#   N(a,l,m,n) = (2a/pi)^{3/4} (4a)^{(l+m+n)/2} / sqrt((2l-1)!!(2m-1)!!(2n-1)!!)
# For l=m=n=0 this is (2a/pi)^{3/4}, identical to the banked h2_core.prim_norm.
# ===========================================================================
def _dfact(k):
    """(2k-1)!! with (-1)!! = 1."""
    r = 1
    while k > 1:
        r *= 2 * k - 1
        k -= 1
    return r


def prim_norm(a, lmn=(0, 0, 0)):
    a = mpf(a)
    base = (2 * a / pi) ** mpf(0.75)
    l, m, n = lmn
    L = l + m + n
    if L == 0:
        return base
    d = _dfact(l) * _dfact(m) * _dfact(n)
    return base * (4 * a) ** (mpf(L) / 2) / sqrt(mpf(d))


CART = {0: ((0, 0, 0),),
        1: ((1, 0, 0), (0, 1, 0), (0, 0, 1))}


class Shell:
    """A contracted Cartesian Gaussian shell.

    The primitive normalisation N(a,l) is deliberately kept OUTSIDE the stored
    coefficient and applied inside each integral, in the same product order the
    banked H2 referee uses, so that the s-only path here is the bank's
    arithmetic expression term for term and not merely its value.

    The contraction is renormalised so <chi|chi> = 1 at the working precision --
    standard practice, and exactly what h2_core.sto3g_h does (the tabulated
    STO-3G coefficients are rounded, so the raw contraction is not normalised).
    """

    __slots__ = ("center", "l", "prims", "raw_norm")

    def __init__(self, center, l, exps, coefs):
        self.center = tuple(mpf(c) for c in center)
        self.l = int(l)
        lmn = (self.l, 0, 0)
        pr = [(mpf(e), mpf(c), prim_norm(e, lmn)) for e, c in zip(exps, coefs)]
        raw = mpf(0)
        for a, ca, na in pr:
            for b, cb, nb in pr:
                raw += ca * cb * _self_overlap(a, na, b, nb, self.l)
        self.raw_norm = raw
        sc = 1 / sqrt(raw)
        self.prims = [(a, c * sc, nn) for a, c, nn in pr]

    def ncart(self):
        return len(CART[self.l])


def _self_overlap(a, na, b, nb, l):
    """<g_a|g_b> for two primitives, same centre, same pure angular momentum l.

    For l = 0 this is h2_core.prim_overlap(a, x, b, x) expression for
    expression: N_a N_b (pi/p)^{3/2} exp(0), with exp(0) exactly 1.
    """
    a, b = mpf(a), mpf(b)
    p = a + b
    v = na * nb * (pi / p) ** mpf(1.5)
    return v if l == 0 else v / (2 * p)


def build_basis(atoms, table=None):
    """atoms: [(Z, (x,y,z)), ...] -> (shells, basis_labels).

    Shell order per atom follows the STO-3G table order (1s, then 2s, then 2p),
    and Cartesian components within a p shell are x, y, z.
    """
    table = STO3G_SHELLS if table is None else table
    shells, labels = [], []
    for iat, (Z, ctr) in enumerate(atoms):
        for l, exps, coefs in table[Z]:
            sh = Shell(ctr, l, exps, coefs)
            shells.append(sh)
            for lmn in CART[l]:
                labels.append((iat, Z, l, lmn))
    return shells, labels


# ===========================================================================
# Shell-pair data: primitive-pair Gaussian products and their Hermite tables.
# ===========================================================================
class PrimPair:
    __slots__ = ("p", "P", "c", "n", "herm", "herm_signed", "a", "b")

    def __init__(self, a, ca, na, A, b, cb, nb, B, la, lb):
        self.a, self.b = a, b
        p = a + b
        self.p = p
        self.P = tuple((a * A[k] + b * B[k]) / p for k in range(3))
        self.c = ca * cb          # contraction coefficients only
        self.n = na * nb          # primitive normalisations, kept separate
        Edir = []
        for k in range(3):
            d = {}
            for i in range(la + 1):
                for j in range(lb + 1):
                    d[(i, j)] = hermite_E(i, j, a, b, A[k], B[k])
            Edir.append(d)
        self.herm, self.herm_signed = [], []
        for lmnA in CART[la]:
            rowH, rowS = [], []
            for lmnB in CART[lb]:
                ex = Edir[0][(lmnA[0], lmnB[0])]
                ey = Edir[1][(lmnA[1], lmnB[1])]
                ez = Edir[2][(lmnA[2], lmnB[2])]
                terms, sterms = [], []
                for t, vt in enumerate(ex):
                    if vt == 0:
                        continue
                    for u, vu in enumerate(ey):
                        if vu == 0:
                            continue
                        tu = vt * vu
                        for v, vv in enumerate(ez):
                            if vv == 0:
                                continue
                            w = tu * vv
                            terms.append((t, u, v, w))
                            sterms.append((t, u, v, -w if (t + u + v) & 1 else w))
                rowH.append(terms)
                rowS.append(sterms)
            self.herm.append(rowH)
            self.herm_signed.append(rowS)


def shell_pair(shA, shB):
    return [PrimPair(a, ca, na, shA.center, b, cb, nb, shB.center, shA.l, shB.l)
            for a, ca, na in shA.prims for b, cb, nb in shB.prims]


# ===========================================================================
# One-electron integrals over shell pairs.
# ===========================================================================
def shell_overlap(shA, shB, pairs=None):
    """S_{ab} = (pi/p)^{3/2} E_0^{ij}(x) E_0^{ij}(y) E_0^{ij}(z), contracted.

    Written so that for two s shells the arithmetic is the banked
    h2_core.prim_overlap expression term for term.
    """
    pairs = shell_pair(shA, shB) if pairs is None else pairs
    na, nb = shA.ncart(), shB.ncart()
    out = [[mpf(0)] * nb for _ in range(na)]
    for pp in pairs:
        base = pp.n * (pi / pp.p) ** mpf(1.5)
        for i in range(na):
            for j in range(nb):
                acc = mpf(0)
                for (t, u, v, w) in pp.herm[i][j]:
                    if t == 0 and u == 0 and v == 0:
                        acc += w
                out[i][j] += pp.c * (base * acc)
    return out


def _s1d(i, j, a, b, Ax, Bx):
    """1-D overlap int x_A^i x_B^j exp(-a x_A^2 - b x_B^2) dx = E_0^{ij} sqrt(pi/p)."""
    if i < 0 or j < 0:
        return mpf(0)
    return hermite_E(i, j, a, b, Ax, Bx)[0] * sqrt(pi / (mpf(a) + mpf(b)))


def shell_kinetic(shA, shB):
    """T = -1/2 <a|nabla^2|b>, from  d^2/dx^2 G_j = 4b^2 G_{j+2} - 2b(2j+1) G_j
    + j(j-1) G_{j-2}, applied direction by direction on the 1-D overlaps."""
    na, nb = shA.ncart(), shB.ncart()
    out = [[mpf(0)] * nb for _ in range(na)]
    A, B = shA.center, shB.center
    for a, ca, na in shA.prims:
        for b, cb, nb in shB.prims:
            cc = ca * cb
            nn = na * nb
            for i, lA in enumerate(CART[shA.l]):
                for j, lB in enumerate(CART[shB.l]):
                    s = [_s1d(lA[k], lB[k], a, b, A[k], B[k]) for k in range(3)]
                    tk = []
                    for k in range(3):
                        jj = lB[k]
                        val = (4 * b * b * _s1d(lA[k], jj + 2, a, b, A[k], B[k])
                               - 2 * b * (2 * jj + 1) * s[k]
                               + (jj * (jj - 1)) * _s1d(lA[k], jj - 2, a, b,
                                                        A[k], B[k]))
                        tk.append(-val / 2)
                    out[i][j] += cc * (nn * (tk[0] * s[1] * s[2]
                                             + s[0] * tk[1] * s[2]
                                             + s[0] * s[1] * tk[2]))
    return out


def shell_nuclear(shA, shB, nuclei, pairs=None):
    """V = -sum_C Z_C (2 pi / p) sum_tuv E_tuv R^0_{tuv}(p, P-C)."""
    pairs = shell_pair(shA, shB) if pairs is None else pairs
    na, nb = shA.ncart(), shB.ncart()
    L = shA.l + shB.l
    out = [[mpf(0)] * nb for _ in range(na)]
    for pp in pairs:
        for (C, Z) in nuclei:
            R = hermite_R(L, pp.p, pp.P[0] - C[0], pp.P[1] - C[1],
                          pp.P[2] - C[2])
            zf = -mpf(Z) * pp.n * (2 * pi / pp.p)
            for i in range(na):
                for j in range(nb):
                    acc = mpf(0)
                    for (t, u, v, w) in pp.herm[i][j]:
                        acc += w * R[(t, u, v)]
                    out[i][j] += pp.c * (zf * acc)
    return out


def shell_eri(shA, shB, shC, shD, pab=None, pcd=None):
    """(ab|cd) chemist notation, McMurchie-Davidson:

      (ab|cd) = 2 pi^{5/2} / (p q sqrt(p+q))
                sum_{tuv} E^{ab}_{tuv} sum_{t'u'v'} (-1)^{t'+u'+v'} E^{cd}_{t'u'v'}
                R^0_{t+t', u+u', v+v'}(alpha = pq/(p+q), P-Q)

    For four s shells this reduces term for term to the banked h2_core.prim_eri.
    """
    pab = shell_pair(shA, shB) if pab is None else pab
    pcd = shell_pair(shC, shD) if pcd is None else pcd
    na, nb = shA.ncart(), shB.ncart()
    nc, nd = shC.ncart(), shD.ncart()
    L = shA.l + shB.l + shC.l + shD.l
    out = [[[[mpf(0)] * nd for _ in range(nc)] for _ in range(nb)]
           for _ in range(na)]
    for bp in pab:
        p, P = bp.p, bp.P
        for kp in pcd:
            q, Q = kp.p, kp.P
            pq = p + q
            alpha = p * q / pq
            R = hermite_R(L, alpha, P[0] - Q[0], P[1] - Q[1], P[2] - Q[2])
            pref = (bp.n * kp.n * 2 * pi ** mpf(2.5)
                    / (p * q * sqrt(pq)))
            cprod = bp.c * kp.c
            for i in range(na):
                for j in range(nb):
                    HB = bp.herm[i][j]
                    if not HB:
                        continue
                    for k in range(nc):
                        for l in range(nd):
                            HK = kp.herm_signed[k][l]
                            acc = mpf(0)
                            for (t, u, v, w) in HB:
                                sub = mpf(0)
                                for (t2, u2, v2, w2) in HK:
                                    sub += w2 * R[(t + t2, u + u2, v + v2)]
                                acc += w * sub
                            out[i][j][k][l] += cprod * (pref * acc)
    return out


# ===========================================================================
# Full AO integral assembly.
# ===========================================================================
def ao_integrals(shells, nuclei, screen=None, want_eri=True):
    """Return S, T, V (n x n) and eri[i][j][k][l] in chemist notation (ij|kl).

    screen: Schwarz threshold |(ab|cd)| <= sqrt((ab|ab)(cd|cd)).  Default is
    10^-(dps+15), far below anything that can move the 50th digit of an energy
    of order 10^2 hartree; pass 0 to disable entirely.
    """
    if screen is None:
        screen = mpf(10) ** (-(mp.dps + 15))
    else:
        screen = mpf(screen)

    ns = len(shells)
    off, n = [], 0
    for sh in shells:
        off.append(n)
        n += sh.ncart()

    pairs = {}
    for i in range(ns):
        for j in range(i + 1):
            pairs[(i, j)] = shell_pair(shells[i], shells[j])

    S = [[mpf(0)] * n for _ in range(n)]
    T = [[mpf(0)] * n for _ in range(n)]
    V = [[mpf(0)] * n for _ in range(n)]
    for i in range(ns):
        for j in range(i + 1):
            pp = pairs[(i, j)]
            s = shell_overlap(shells[i], shells[j], pp)
            t = shell_kinetic(shells[i], shells[j])
            v = shell_nuclear(shells[i], shells[j], nuclei, pp)
            for a in range(shells[i].ncart()):
                for b in range(shells[j].ncart()):
                    ia, jb = off[i] + a, off[j] + b
                    S[ia][jb] = S[jb][ia] = s[a][b]
                    T[ia][jb] = T[jb][ia] = t[a][b]
                    V[ia][jb] = V[jb][ia] = v[a][b]
    if not want_eri:
        return S, T, V, None

    # Schwarz bounds per shell pair
    Q = {}
    for i in range(ns):
        for j in range(i + 1):
            d = shell_eri(shells[i], shells[j], shells[i], shells[j],
                          pairs[(i, j)], pairs[(i, j)])
            m = mpf(0)
            for a in range(shells[i].ncart()):
                for b in range(shells[j].ncart()):
                    m = max(m, abs(d[a][b][a][b]))
            Q[(i, j)] = sqrt(m)

    eri = [[[[mpf(0)] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]
    nskip = 0
    for i in range(ns):
        for j in range(i + 1):
            ij = i * (i + 1) // 2 + j
            for k in range(ns):
                for l in range(k + 1):
                    kl = k * (k + 1) // 2 + l
                    if kl > ij:
                        continue
                    if screen > 0 and Q[(i, j)] * Q[(k, l)] < screen:
                        nskip += 1
                        continue
                    blk = shell_eri(shells[i], shells[j], shells[k], shells[l],
                                    pairs[(i, j)], pairs[(k, l)])
                    for a in range(shells[i].ncart()):
                        ia = off[i] + a
                        for b in range(shells[j].ncart()):
                            jb = off[j] + b
                            for c in range(shells[k].ncart()):
                                kc = off[k] + c
                                for d in range(shells[l].ncart()):
                                    ld = off[l] + d
                                    val = blk[a][b][c][d]
                                    for (P, Qq, Rr, Ss) in (
                                            (ia, jb, kc, ld), (jb, ia, kc, ld),
                                            (ia, jb, ld, kc), (jb, ia, ld, kc),
                                            (kc, ld, ia, jb), (ld, kc, ia, jb),
                                            (kc, ld, jb, ia), (ld, kc, jb, ia)):
                                        eri[P][Qq][Rr][Ss] = val
    ao_integrals.last_screened = nskip
    return S, T, V, eri


def nuclear_repulsion(nuclei):
    e = mpf(0)
    for i in range(len(nuclei)):
        Ci, Zi = nuclei[i]
        for j in range(i):
            Cj, Zj = nuclei[j]
            d = sqrt(sum((Ci[k] - Cj[k]) ** 2 for k in range(3)))
            e += mpf(Zi) * mpf(Zj) / d
    return e


def molecule(atoms, table=None, screen=None):
    """atoms: [(Z,(x,y,z)),...] -> dict with basis, integrals, E_nuc."""
    shells, labels = build_basis(atoms, table)
    nuclei = [(tuple(mpf(x) for x in c), Z) for Z, c in atoms]
    S, T, V, eri = ao_integrals(shells, nuclei, screen)
    n = len(labels)
    Hcore = [[T[i][j] + V[i][j] for j in range(n)] for i in range(n)]
    return dict(shells=shells, labels=labels, nuclei=nuclei, S=S, T=T, V=V,
                eri=eri, Hcore=Hcore, nbf=n, E_nuc=nuclear_repulsion(nuclei),
                nelec=sum(Z for Z, _ in atoms), atoms=atoms)
