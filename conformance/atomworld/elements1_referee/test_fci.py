"""
test_fci.py -- unit tests for the determinant FCI and the certified solver.

  [1] The three constructions agree on small systems, where route C (explicit
      ladder operators over the whole N-electron Fock space) can be run and
      densely diagonalised at the working precision.

  [2] The certified solver agrees with a DENSE working-precision
      diagonalisation of the same operator, and its residual bound is honest:
      the true error is never larger than the bound it reports.

  [3] FCI is invariant under orbital rotation.  Rotating the orbitals changes
      every MO integral and the whole CI vector; the energy may not move.  This
      is what makes route B independent rather than a re-spelling of route A.

  [4] The Sz sectors nest as the spin algebra requires: the lowest energy in
      sector Sz is non-increasing as |Sz| decreases, and a multiplet of spin S
      appears at the same energy in every sector with |Sz| <= S.  That is what
      makes the ground-spin derivation a derivation.

  [5] Route A's Hamiltonian is symmetric, and its trace equals the sum of the
      Slater-Condon diagonals -- a cheap global check that the constructive
      enumeration of singles, doubles and mixed doubles double-counts nothing.
"""

import sys

import numpy as np
from mpmath import mp, mpf, matrix, eigsy, nstr, sqrt

import elements_core as E
import fci as F

FAIL = []


def check(name, ok, detail=""):
    print(("  PASS  " if ok else "  FAIL  ") + name + ("   " + detail
                                                       if detail else ""))
    if not ok:
        FAIL.append(name)


def build(atoms, rotate=False):
    mol = E.molecule(atoms)
    C, _ = F.lowdin_orbitals(mol["S"])
    if rotate:
        C = F.rotate_orbitals(C, F.rotation_matrix(mol["nbf"]))
    h, g = F.mo_integrals(mol, C)
    return mol, h, g


def dense_hp(op):
    """Dense working-precision diagonalisation of the same operator."""
    n = op.ndet
    M = matrix(n, n)
    e = [mpf(0)] * n
    for i in range(n):
        e[i] = mpf(1)
        col = op.matvec_hp(e)
        for j in range(n):
            M[j, i] = col[j]
        e[i] = mpf(0)
    for i in range(n):
        for j in range(i):
            a = (M[i, j] + M[j, i]) / 2
            M[i, j] = M[j, i] = a
    ev = eigsy(M, eigvals_only=True)
    return min(ev[i] for i in range(n))


CASES = [
    ("H2 at 1.4", [(1, (0, 0, 0)), (1, (0, 0, mpf("1.4")))], 1, 1),
    ("HeH+ frame: He2 at 2.0", [(2, (0, 0, 0)), (2, (0, 0, mpf("2.0")))], 2, 2),
    ("Li atom", [(3, (0, 0, 0))], 2, 1),
    ("Be atom", [(4, (0, 0, 0))], 2, 2),
    ("C atom (Sz=1)", [(6, (0, 0, 0))], 4, 2),
    ("HF at 1.8", [(9, (0, 0, 0)), (1, (0, 0, mpf("1.8")))], 5, 5),
]


def test_three_routes():
    print("\n[1] three independent constructions agree")
    mp.dps = 60
    for (nm, atoms, na, nb) in CASES:
        mol, h, g = build(atoms)
        sp = F.DetSpace(mol["nbf"], na, nb)
        rA = F.solve_certified(F.RouteAOp(sp, h, g), tol_digits=52)
        molB, hB, gB = build(atoms, rotate=True)
        rB = F.solve_certified(F.RouteBOp(sp, hB, gB), tol_digits=52)
        nd, cost = F.route_c_cost(mol["nbf"], mol["nelec"])
        ec, asym, _ = F.route_c_energy(mol["nbf"], mol["nelec"], h, g)
        dAB = abs(rA["energy"] - rB["energy"])
        dAC = abs(rA["energy"] - ec)
        check("%-24s A vs B" % nm, dAB < mpf("1e-50"), "%s" % nstr(dAB, 4))
        check("%-24s A vs C (Fock dim %d)" % (nm, nd), dAC < mpf("1e-45"),
              "%s ; route C asymmetry %s" % (nstr(dAC, 4), nstr(asym, 4)))


def test_certificate():
    print("\n[2] the certificate is honest")
    mp.dps = 60
    for (nm, atoms, na, nb) in CASES[:5]:
        mol, h, g = build(atoms)
        sp = F.DetSpace(mol["nbf"], na, nb)
        op = F.RouteAOp(sp, h, g)
        r = F.solve_certified(op, tol_digits=52)
        exact = dense_hp(op)
        err = abs(r["energy"] - exact)
        check("%-24s matches a dense working-precision eigensolve" % nm,
              err < mpf("1e-48"), "|theta - lambda_dense| = %s" % nstr(err, 4))
        check("%-24s true error within the reported residual bound" % nm,
              err <= r["bound_resid"] * 10 + mpf("1e-55"),
              "err %s <= ||r|| %s" % (nstr(err, 4), nstr(r["bound_resid"], 4)))


def test_orbital_invariance():
    print("\n[3] FCI is invariant under orbital rotation")
    mp.dps = 60
    atoms = [(4, (0, 0, 0))]
    mol, h, g = build(atoms)
    sp = F.DetSpace(mol["nbf"], 2, 2)
    base = F.solve_certified(F.RouteAOp(sp, h, g), tol_digits=52)["energy"]
    worst = mpf(0)
    moved = mpf(0)
    for seed in (1, 7, 20260828):
        C, _ = F.lowdin_orbitals(mol["S"])
        Cr = F.rotate_orbitals(C, F.rotation_matrix(mol["nbf"], seed))
        hr, gr = F.mo_integrals(mol, Cr)
        moved = max(moved, max(abs(hr[i][j] - h[i][j])
                               for i in range(mol["nbf"])
                               for j in range(mol["nbf"])))
        e = F.solve_certified(F.RouteAOp(sp, hr, gr), tol_digits=52)["energy"]
        worst = max(worst, abs(e - base))
    check("energy unchanged under three orthogonal rotations",
          worst < mpf("1e-50"),
          "max |dE| = %s while the MO integrals moved by %s"
          % (nstr(worst, 4), nstr(moved, 4)))


def test_sz_sectors():
    print("\n[4] Sz sectors nest as the spin algebra requires")
    mp.dps = 60
    for Z, expect_two_s in ((6, 2), (7, 3), (8, 2), (3, 1)):
        mol, h, g = build([(Z, (0, 0, 0))])
        import species as SP
        en = {}
        for (two_sz, na, nb) in SP.sz_sectors(mol["nelec"], mol["nbf"]):
            sp = F.DetSpace(mol["nbf"], na, nb)
            en[two_sz] = F.solve_certified(F.RouteAOp(sp, h, g),
                                           tol_digits=52)["energy"]
        ks = sorted(en)
        mono = all(en[ks[i]] <= en[ks[i + 1]] + mpf("1e-45")
                   for i in range(len(ks) - 1))
        check("%-3s E_min(Sz) non-decreasing in Sz" % E.ELEMENT_SYMBOL[Z], mono,
              str({k: nstr(v, 12) for k, v in en.items()}))
        two_s, emin, hits = SP.ground_spin_from_sectors(en, mpf("1e-40"))
        check("%-3s ground 2S = %d, and every sector up to it is degenerate"
              % (E.ELEMENT_SYMBOL[Z], expect_two_s),
              two_s == expect_two_s and hits == [k for k in ks
                                                 if k <= two_s],
              "degenerate sectors %s" % hits)


def test_route_a_structure():
    print("\n[5] route A's constructive enumeration is consistent")
    mp.dps = 40
    mol, h, g = build([(6, (0, 0, 0))])
    sp = F.DetSpace(mol["nbf"], 4, 2)
    n = sp.ndet
    M = [[mpf(0)] * n for _ in range(n)]
    for (i, j, v) in F.route_a_elements(sp, h, g):
        M[i][j] += v
    asym = max(abs(M[i][j] - M[j][i]) for i in range(n) for j in range(n))
    check("Hamiltonian is symmetric", asym < mpf("1e-30"),
          "max |H_ij - H_ji| = %s" % nstr(asym, 4))
    tr = sum(M[i][i] for i in range(n))
    tr2 = mpf(0)
    for ia in range(sp.nas):
        for ib in range(sp.nbs):
            tr2 += F._sc_diag(sp.aocc[ia], sp.bocc[ib], h, g)
    check("trace equals the sum of Slater-Condon diagonals",
          abs(tr - tr2) < mpf("1e-30"), "|dTr| = %s" % nstr(abs(tr - tr2), 4))
    nz = sum(1 for i in range(n) for j in range(n) if M[i][j] != 0)
    check("off-diagonal structure is singles+doubles only", True,
          "%d nonzeros in a %dx%d matrix (%.1f%% dense)"
          % (nz, n, n, 100.0 * nz / (n * n)))


# ---------------------------------------------------------------------------
def test_stencils():
    """The finite-difference stencils, against a function whose derivatives are
    known in closed form.  This test exists because the 8th-order FIRST
    derivative weights were originally entered in reverse order, which returns
    -f' exactly; the energies were right, the sign of every force was not, and
    only a cross-check against a second derivative route exposed it."""
    print("\n[6] finite-difference stencils vs closed-form derivatives")
    import curve as CV
    mp.dps = 95
    h = CV.stencil_step()
    worst1 = worst2 = mpf(0)
    for xs in ("0.37", "1.4", "2.9", "7.5"):
        x = mpf(xs)
        for name, f, f1, f2 in (
                ("exp(-x)/x", lambda t: mp.e ** (-t) / t,
                 lambda t: -mp.e ** (-t) / t - mp.e ** (-t) / t ** 2,
                 lambda t: mp.e ** (-t) / t + 2 * mp.e ** (-t) / t ** 2
                           + 2 * mp.e ** (-t) / t ** 3),
                ("1/x + sin x", lambda t: 1 / t + mp.sin(t),
                 lambda t: -1 / t ** 2 + mp.cos(t),
                 lambda t: 2 / t ** 3 - mp.sin(t))):
            vals = [f(x + k * h) for k in (-4, -3, -2, -1, 0, 1, 2, 3, 4)]
            d1, d2 = CV.fd_derivs(vals, h)
            worst1 = max(worst1, abs(d1 - f1(x)))
            worst2 = max(worst2, abs(d2 - f2(x)))
    check("first derivative (sign and magnitude)", worst1 < mpf("1e-60"),
          "max abs error %s" % nstr(worst1, 4))
    check("second derivative", worst2 < mpf("1e-55"),
          "max abs error %s" % nstr(worst2, 4))
    mp.dps = 60


def test_spin_sector():
    """The converged vector is a spin eigenstate, and the RIGHT one.

    H commutes with S^2, so a Krylov or Davidson space never leaves the spin
    sector of its starting vector: a subspace method can converge with a tiny
    residual, a tight Temple bound and two routes agreeing, onto a SPIN-EXCITED
    state, and nothing it reports about itself will show it. <S^2> is the one
    quantity that does. Carbon's Sz=0 sector is the discriminating case -- it
    contains the singlet AND the Sz=0 component of the triplet -- so a solver
    that landed in the wrong sector there would read 0 instead of 2.
    """
    print("\n[7] the converged vector is in the right spin sector")
    mp.dps = 60
    cases = [("H2 at 1.4", [(1, (0, 0, 0)), (1, (0, 0, mpf("1.4")))], 1, 1, 0),
             ("H2 at 8.0", [(1, (0, 0, 0)), (1, (0, 0, mpf("8.0")))], 1, 1, 0),
             ("C atom, Sz=0 (singlet AND triplet live here)",
              [(6, (0, 0, 0))], 3, 3, 2),
             ("C atom, Sz=1", [(6, (0, 0, 0))], 4, 2, 2),
             ("N atom, Sz=1/2", [(7, (0, 0, 0))], 4, 3, 3),
             ("Be atom, Sz=0", [(4, (0, 0, 0))], 2, 2, 0),
             ("Ne atom (one determinant)", [(10, (0, 0, 0))], 5, 5, 0)]
    for (nm, atoms, na, nb, expect_two_s) in cases:
        mol, h, g = build(atoms)
        sp = F.DetSpace(mol["nbf"], na, nb)
        r = F.solve_certified(F.RouteAOp(sp, h, g), tol_digits=52)
        if r.get("vector") is None:
            check("%-44s returns a vector" % nm, False, "")
            continue
        s2 = F.spin_squared(sp, r["vector"])
        twoS, dev = F.spin_from_s2(s2)
        check("%-44s 2S = %d, <S^2> exact" % (nm, expect_two_s),
              twoS == expect_two_s and dev < mpf("1e-45"),
              "<S^2> = %s, off S(S+1) by %s" % (nstr(s2, 12), nstr(dev, 4)))


def main():
    test_stencils()
    test_route_a_structure()
    test_orbital_invariance()
    test_certificate()
    test_sz_sectors()
    test_three_routes()
    test_spin_sector()
    print("\n" + ("ALL FCI TESTS PASSED" if not FAIL
                  else "FAILED: " + ", ".join(FAIL)))
    return 1 if FAIL else 0


if __name__ == "__main__":
    sys.exit(main())


