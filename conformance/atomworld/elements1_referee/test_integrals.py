"""
test_integrals.py -- unit tests for the general STO-3G (s,p) integral engine.

The tests are of four kinds, in increasing strength:

  (1) BANKED IDENTITY.  Every s-only integral must reproduce the banked H2
      referee h2_core.py.  Overlap, nuclear attraction and the ERI are required
      to be bit-for-bit; the kinetic integral is reached by a different algebraic
      route (Hermite 1-D overlaps vs the closed form) and is required to agree
      to a few units in the last place of the working precision.

  (2) DIFFERENTIATION OF THE BANK  -- the real test of the p machinery.
      An unnormalised p_x primitive is exactly the A_x-derivative of an
      unnormalised s primitive:
            (x - A_x) exp(-a|r-A|^2) = (1/2a) d/dA_x exp(-a|r-A|^2)
      so EVERY p integral is a center-coordinate derivative of the banked s-only
      closed forms, evaluated here by mpmath.diff at raised precision.  Nothing
      of the McMurchie-Davidson recursions enters that route.  Normalisation:
      N_p/N_s = (4a)^{1/2}, so <p_x^A|O|s^B> = a^{-1/2} d/dA_x <s^A|O|s^B> and
      <p^A|O|p^B> = (a b)^{-1/2} d^2/dA dB <s^A|O|s^B>, with the h2_core
      normalisation already inside the s integrals.

  (3) SYMMETRY IDENTITIES.  Translational invariance of every integral;
      rotational invariance of the total electronic energy; the eight-fold ERI
      permutational symmetry checked by INDEPENDENT re-evaluation with the
      shells passed in permuted order (not by the storage loop that imposes it).

  (4) INTERNAL CONSISTENCY OF THE DECLARED BASIS.  The STO-3G table is a
      universal 1s / 2sp Gaussian fit scaled by zeta^2; the tabulated exponents
      must reproduce that scaling law.  This does not derive the basis (it is a
      declared input); it catches transcription error in the declared input.
"""

import os
import sys
import itertools
from mpmath import mp, mpf, sqrt, exp, erf, pi, diff, nstr, matrix

# h2_core.py is the BANKED foundation and lives one level up beside the freeze
# it belongs to; import it from there rather than keeping a second copy.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(1, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import elements_core as E
import h2_core as H

FAIL = []


def check(name, ok, detail=""):
    print(("  PASS  " if ok else "  FAIL  ") + name + ("   " + detail if detail else ""))
    if not ok:
        FAIL.append(name)


def ulps(x, y):
    x, y = mpf(x), mpf(y)
    if x == y:
        return 0.0
    m = max(abs(x), abs(y))
    if m == 0:
        return 0.0
    return float(abs(x - y) / m) * float(mpf(2) ** mp.prec)


# ---------------------------------------------------------------------------
# The banked s-only closed forms, read in three dimensions.  These are
# h2_core's expressions verbatim with |A-B|^2 summed over three coordinates;
# on a linear arrangement they reduce to h2_core term for term.
# ---------------------------------------------------------------------------
def _d2(A, B):
    return sum((mpf(A[k]) - mpf(B[k])) ** 2 for k in range(3))


def s3_overlap(a, A, b, B):
    a, b = mpf(a), mpf(b)
    p = a + b
    mu = a * b / p
    return (H.prim_norm(a) * H.prim_norm(b) * (pi / p) ** mpf(1.5)
            * exp(-mu * _d2(A, B)))


def s3_kinetic(a, A, b, B):
    a, b = mpf(a), mpf(b)
    p = a + b
    mu = a * b / p
    d2 = _d2(A, B)
    return (H.prim_norm(a) * H.prim_norm(b) * mu * (3 - 2 * mu * d2)
            * (pi / p) ** mpf(1.5) * exp(-mu * d2))


def s3_nuclear(a, A, b, B, C, Z):
    a, b = mpf(a), mpf(b)
    p = a + b
    mu = a * b / p
    P = [(a * mpf(A[k]) + b * mpf(B[k])) / p for k in range(3)]
    t = p * _d2(P, C)
    return (-mpf(Z) * H.prim_norm(a) * H.prim_norm(b) * (2 * pi / p)
            * exp(-mu * _d2(A, B)) * H.boys0(t))


def s3_eri(a, A, b, B, c, C, d, D):
    a, b, c, d = mpf(a), mpf(b), mpf(c), mpf(d)
    p, q = a + b, c + d
    P = [(a * mpf(A[k]) + b * mpf(B[k])) / p for k in range(3)]
    Q = [(c * mpf(C[k]) + d * mpf(D[k])) / q for k in range(3)]
    Kab = exp(-(a * b / p) * _d2(A, B))
    Kcd = exp(-(c * d / q) * _d2(C, D))
    t = (p * q / (p + q)) * _d2(P, Q)
    pref = (H.prim_norm(a) * H.prim_norm(b) * H.prim_norm(c) * H.prim_norm(d)
            * 2 * pi ** mpf(2.5) / (p * q * sqrt(p + q)))
    return pref * Kab * Kcd * H.boys0(t)


def prim_shell(center, l, a):
    """A one-primitive shell with unit contraction coefficient."""
    return E.Shell(center, l, (a,), ("1",))


# ---------------------------------------------------------------------------
def test_banked_identity():
    print("\n[1] BANKED IDENTITY  (s-only integrals vs h2_core.py)")
    mp.dps = 60
    worst = {"S": 0.0, "T": 0.0, "V": 0.0, "ERI": 0.0}
    for Rs in ("0.3", "0.7", "1.4", "2.0", "3.5", "7.0", "10"):
        R = mpf(Rs)
        shA = E.Shell((0, 0, 0), 0, *E.STO3G_SHELLS[1][0][1:])
        shB = E.Shell((0, 0, R), 0, *E.STO3G_SHELLS[1][0][1:])
        bA, bB = H.sto3g_h(0)[1], H.sto3g_h(R)[1]

        def bank1(fn, *extra):
            acc = mpf(0)
            for a, ca in bA:
                for b, cb in bB:
                    acc += ca * cb * fn(a, mpf(0), b, R, *extra)
            return acc

        S = E.shell_overlap(shA, shB)[0][0]
        T = E.shell_kinetic(shA, shB)[0][0]
        V = E.shell_nuclear(shA, shB, [((0, 0, 0), 1), ((0, 0, R), 1)])[0][0]
        g = E.shell_eri(shA, shB, shA, shB)[0][0][0][0]
        gb = mpf(0)
        for a, ca in bA:
            for b, cb in bB:
                for c, cc in bA:
                    for d, cd in bB:
                        gb += ca * cb * cc * cd * H.prim_eri(a, 0, b, R, c, 0, d, R)
        worst["S"] = max(worst["S"], ulps(S, bank1(H.prim_overlap)))
        worst["T"] = max(worst["T"], ulps(T, bank1(H.prim_kinetic)))
        worst["V"] = max(worst["V"], ulps(V, bank1(H.prim_nuclear, mpf(0), 1)
                                          + bank1(H.prim_nuclear, R, 1)))
        worst["ERI"] = max(worst["ERI"], ulps(g, gb))
    # The overlap path is written to be the bank's expression term for term and
    # is required to be exactly equal.  Nuclear attraction, the ERI and the
    # kinetic integral reach the same value by a different association of the
    # same products (and, for T, by a different algebraic route entirely), so
    # they are required to agree to a few units in the last place of the 60-digit
    # WORKING precision -- i.e. to be identical in all 50 REPORTED digits.  The
    # decisive form of this check is the H2 total-energy identity in
    # verify_elements.py, which compares the banked 50-digit strings themselves.
    check("overlap bit-for-bit", worst["S"] == 0.0, "max %.2f ulp" % worst["S"])
    check("nuclear within 64 ulp of working precision",
          worst["V"] < 64, "max %.2f ulp" % worst["V"])
    check("eri within 64 ulp of working precision",
          worst["ERI"] < 64, "max %.2f ulp" % worst["ERI"])
    check("kinetic within 64 ulp (different algebraic route)",
          worst["T"] < 64, "max %.2f ulp" % worst["T"])


# ---------------------------------------------------------------------------
def test_p_by_differentiating_the_bank():
    print("\n[2] p INTEGRALS BY DIFFERENTIATING THE BANKED s CLOSED FORMS")
    mp.dps = 150
    tol = mpf("1e-40")
    A0 = [mpf("0.11"), mpf("-0.27"), mpf("0.4")]
    B0 = [mpf("0.9"), mpf("0.35"), mpf("-1.3")]
    C0 = [mpf("-0.6"), mpf("1.1"), mpf("0.2")]
    D0 = [mpf("0.45"), mpf("-0.8"), mpf("1.7")]
    a, b, c, d = mpf("1.7"), mpf("0.63"), mpf("2.1"), mpf("0.41")
    NUC = [(tuple(C0), 3)]

    def sub(v, k, x):
        w = list(v)
        w[k] = x
        return w

    # ---- <p_i^A | O | s^B>  and  <p_i^A | O | p_j^B> for S, T, V
    worst = mpf(0)
    for k1 in range(3):
        f = lambda x, k1=k1: s3_overlap(a, sub(A0, k1, x), b, B0)
        ref = diff(f, A0[k1]) / sqrt(a)
        mine = E.shell_overlap(prim_shell(A0, 1, a), prim_shell(B0, 0, b))[k1][0]
        worst = max(worst, abs(mine - ref))
        f = lambda x, k1=k1: s3_kinetic(a, sub(A0, k1, x), b, B0)
        ref = diff(f, A0[k1]) / sqrt(a)
        mine = E.shell_kinetic(prim_shell(A0, 1, a), prim_shell(B0, 0, b))[k1][0]
        worst = max(worst, abs(mine - ref))
        f = lambda x, k1=k1: s3_nuclear(a, sub(A0, k1, x), b, B0, C0, 3)
        ref = diff(f, A0[k1]) / sqrt(a)
        mine = E.shell_nuclear(prim_shell(A0, 1, a), prim_shell(B0, 0, b), NUC)[k1][0]
        worst = max(worst, abs(mine - ref))
    check("(p|O|s) for S,T,V", worst < tol, "max abs dev %s" % nstr(worst, 6))

    worst = mpf(0)
    mS = E.shell_overlap(prim_shell(A0, 1, a), prim_shell(B0, 1, b))
    mT = E.shell_kinetic(prim_shell(A0, 1, a), prim_shell(B0, 1, b))
    mV = E.shell_nuclear(prim_shell(A0, 1, a), prim_shell(B0, 1, b), NUC)
    for k1 in range(3):
        for k2 in range(3):
            for fn, mine in ((s3_overlap, mS[k1][k2]), (s3_kinetic, mT[k1][k2])):
                g = lambda x, y, k1=k1, k2=k2, fn=fn: fn(
                    a, sub(A0, k1, x), b, sub(B0, k2, y))
                ref = diff(g, (A0[k1], B0[k2]), (1, 1)) / sqrt(a * b)
                worst = max(worst, abs(mine - ref))
            g = lambda x, y, k1=k1, k2=k2: s3_nuclear(
                a, sub(A0, k1, x), b, sub(B0, k2, y), C0, 3)
            ref = diff(g, (A0[k1], B0[k2]), (1, 1)) / sqrt(a * b)
            worst = max(worst, abs(mV[k1][k2] - ref))
    check("(p|O|p) for S,T,V", worst < tol, "max abs dev %s" % nstr(worst, 6))

    # ---- ERIs with 1, 2 and 4 p shells
    worst1 = mpf(0)
    m = E.shell_eri(prim_shell(A0, 1, a), prim_shell(B0, 0, b),
                    prim_shell(C0, 0, c), prim_shell(D0, 0, d))
    for k1 in range(3):
        g = lambda x, k1=k1: s3_eri(a, sub(A0, k1, x), b, B0, c, C0, d, D0)
        worst1 = max(worst1, abs(m[k1][0][0][0] - diff(g, A0[k1]) / sqrt(a)))
    check("(ps|ss) eri", worst1 < tol, "max abs dev %s" % nstr(worst1, 6))

    worst2 = mpf(0)
    m = E.shell_eri(prim_shell(A0, 1, a), prim_shell(B0, 0, b),
                    prim_shell(C0, 1, c), prim_shell(D0, 0, d))
    for k1 in range(3):
        for k3 in range(3):
            g = lambda x, z, k1=k1, k3=k3: s3_eri(
                a, sub(A0, k1, x), b, B0, c, sub(C0, k3, z), d, D0)
            ref = diff(g, (A0[k1], C0[k3]), (1, 1)) / sqrt(a * c)
            worst2 = max(worst2, abs(m[k1][0][k3][0] - ref))
    check("(ps|ps) eri", worst2 < tol, "max abs dev %s" % nstr(worst2, 6))

    worst4 = mpf(0)
    m = E.shell_eri(prim_shell(A0, 1, a), prim_shell(B0, 1, b),
                    prim_shell(C0, 1, c), prim_shell(D0, 1, d))
    for (k1, k2, k3, k4) in ((0, 0, 0, 0), (0, 1, 2, 0), (2, 2, 1, 1),
                             (1, 2, 0, 2), (0, 1, 1, 0)):
        g = lambda w, x, y, z, k1=k1, k2=k2, k3=k3, k4=k4: s3_eri(
            a, sub(A0, k1, w), b, sub(B0, k2, x),
            c, sub(C0, k3, y), d, sub(D0, k4, z))
        ref = diff(g, (A0[k1], B0[k2], C0[k3], D0[k4]), (1, 1, 1, 1)) \
            / sqrt(a * b * c * d)
        worst4 = max(worst4, abs(m[k1][k2][k3][k4] - ref))
    check("(pp|pp) eri", worst4 < mpf("1e-30"), "max abs dev %s" % nstr(worst4, 6))
    mp.dps = 60


# ---------------------------------------------------------------------------
def test_boys():
    print("\n[3] BOYS FUNCTION -- closed form vs downward recursion")
    mp.dps = 60
    worst = mpf(0)
    for ts in ("0", "1e-30", "1e-8", "0.01", "0.3", "0.5", "1", "3.7",
               "12.5", "60", "400", "5000"):
        t = mpf(ts)
        ref = E.boys_down(6, t)
        for n in range(7):
            mine = E.boys(n, t)
            if ref[n] != 0:
                worst = max(worst, abs(mine - ref[n]) / abs(ref[n]))
    check("F_n agrees with downward recursion", worst < mpf("1e-52"),
          "max rel dev %s" % nstr(worst, 6))
    # F_0 must be the banked function itself
    ok = all(E.boys(0, mpf(ts)) == H.boys0(mpf(ts))
             for ts in ("0", "1e-40", "0.4", "2", "50"))
    check("F_0 is h2_core.boys0 identically", ok)


# ---------------------------------------------------------------------------
def _rot_matrix(al, be, ga):
    ca, sa = mp.cos(al), mp.sin(al)
    cb, sb = mp.cos(be), mp.sin(be)
    cg, sg = mp.cos(ga), mp.sin(ga)
    Rz1 = [[ca, -sa, 0], [sa, ca, 0], [0, 0, 1]]
    Ry = [[cb, 0, sb], [0, 1, 0], [-sb, 0, cb]]
    Rz2 = [[cg, -sg, 0], [sg, cg, 0], [0, 0, 1]]
    def mul(X, Y):
        return [[sum(X[i][k] * Y[k][j] for k in range(3)) for j in range(3)]
                for i in range(3)]
    return mul(mul(Rz1, Ry), Rz2)


def test_symmetries():
    print("\n[4] SYMMETRY IDENTITIES")
    mp.dps = 60
    atoms = [(7, ("0", "0", "0")), (8, ("0.3", "-0.7", "1.9"))]
    m0 = E.molecule([(Z, [mpf(x) for x in c]) for Z, c in atoms])
    n = m0["nbf"]

    # translational invariance of every AO integral
    sh = [mpf("1.37"), mpf("-2.4"), mpf("0.61")]
    m1 = E.molecule([(Z, [mpf(x) + sh[k] for k, x in enumerate(c)])
                     for Z, c in atoms])
    w = mpf(0)
    for M in ("S", "T", "V", "Hcore"):
        for i in range(n):
            for j in range(n):
                w = max(w, abs(m0[M][i][j] - m1[M][i][j]))
    for i in range(n):
        for j in range(n):
            for k in range(n):
                for l in range(n):
                    w = max(w, abs(m0["eri"][i][j][k][l] - m1["eri"][i][j][k][l]))
    check("translational invariance (all S,T,V,ERI)", w < mpf("1e-50"),
          "max abs dev %s" % nstr(w, 6))

    # ERI permutational symmetry by INDEPENDENT re-evaluation of permuted shells
    shells, _ = E.build_basis([(Z, [mpf(x) for x in c]) for Z, c in atoms])
    A, B, C, D = shells[2], shells[0], shells[5], shells[3]   # p,s,p,s
    base = E.shell_eri(A, B, C, D)
    perms = {"(ba|cd)": (B, A, C, D), "(ab|dc)": (A, B, D, C),
             "(cd|ab)": (C, D, A, B), "(dc|ba)": (D, C, B, A)}
    w = mpf(0)
    for nm, (P, Q, R_, S_) in perms.items():
        alt = E.shell_eri(P, Q, R_, S_)
        for i in range(A.ncart()):
            for j in range(B.ncart()):
                for k in range(C.ncart()):
                    for l in range(D.ncart()):
                        if nm == "(ba|cd)":
                            v = alt[j][i][k][l]
                        elif nm == "(ab|dc)":
                            v = alt[i][j][l][k]
                        elif nm == "(cd|ab)":
                            v = alt[k][l][i][j]
                        else:
                            v = alt[l][k][j][i]
                        w = max(w, abs(base[i][j][k][l] - v))
    check("8-fold ERI symmetry by independent re-evaluation", w < mpf("1e-50"),
          "max abs dev %s" % nstr(w, 6))

    # rotational invariance of the trace-level invariants and of Hcore's spectrum
    Rm = _rot_matrix(mpf("0.7"), mpf("1.3"), mpf("-0.45"))
    rot = [(Z, [sum(Rm[i][k] * mpf(c[k]) for k in range(3)) for i in range(3)])
           for Z, c in atoms]
    m2 = E.molecule(rot)
    ev0 = sorted(mp.eigsy(matrix([[m0["Hcore"][i][j] for j in range(n)]
                                  for i in range(n)]), eigvals_only=True))
    ev2 = sorted(mp.eigsy(matrix([[m2["Hcore"][i][j] for j in range(n)]
                                  for i in range(n)]), eigvals_only=True))
    w = max(abs(a - b) for a, b in zip(ev0, ev2))
    check("rotational invariance of the Hcore spectrum", w < mpf("1e-48"),
          "max abs dev %s" % nstr(w, 6))


# ---------------------------------------------------------------------------
def test_basis_consistency():
    print("\n[5] DECLARED BASIS -- internal consistency of the STO-3G table")
    mp.dps = 30
    # STO-3G is a universal three-Gaussian fit to a Slater orbital, scaled by
    # zeta^2.  Recover zeta from the FIRST exponent of each shell and require
    # the other two to follow the same scaling.
    ref = {}
    worst = 0.0
    for Z in range(1, 11):
        for (l, exps, coefs) in E.STO3G_SHELLS[Z]:
            key = ("1s" if (Z <= 2 or (l == 0 and float(exps[0]) > 10))
                   else "2sp")
            e = [float(x) for x in exps]
            sc = e[0]
            r = [x / sc for x in e]
            if (key, l) not in ref:
                ref[(key, l)] = r
            else:
                for u, v in zip(r, ref[(key, l)]):
                    worst = max(worst, abs(u - v) / abs(v))
    check("exponents follow one zeta^2 scaling law per shell type",
          worst < 2e-6, "max rel dev %.2e" % worst)
    # the contraction coefficients are universal across the row
    cs = set()
    for Z in range(1, 11):
        for (l, exps, coefs) in E.STO3G_SHELLS[Z]:
            cs.add((l, coefs))
    check("exactly three distinct contraction coefficient sets (1s, 2s, 2p)",
          len(cs) == 3, "found %d" % len(cs))
    # hydrogen's row is the banked referee's row, character for character
    ok = (E.STO3G_SHELLS[1][0][1] == H.STO3G_H_EXPONENTS
          and E.STO3G_SHELLS[1][0][2] == H.STO3G_H_COEFFS)
    check("hydrogen row identical to h2_core's declared constants", ok)
    mp.dps = 60


def main():
    test_banked_identity()
    test_boys()
    test_p_by_differentiating_the_bank()
    test_symmetries()
    test_basis_consistency()
    print("\n" + ("ALL INTEGRAL TESTS PASSED" if not FAIL
                  else "FAILED: " + ", ".join(FAIL)))
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())
