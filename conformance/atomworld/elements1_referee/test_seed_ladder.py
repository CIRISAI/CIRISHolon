"""The double-precision seed ladder, exercised by making every rung fail.

WHY THERE IS A LADDER.  solve_certified does its real work at the working
precision; ARPACK's only job is to hand it a starting subspace and a rough gap.
But the call asked for `tol=0` -- MACHINE precision on all k vectors -- and at a
dissociation limit the lowest levels are degenerate to about 1e-14, so ARPACK
cannot separate them and the request fails for a reason that says nothing about
the quality of the seed.  CO at R = 9 bohr (14400 determinants, dissociated to
C + O, both open shell) failed both the k=6 tol=0 attempt and the k=2 tol=1e-12
retry, and stage 1 died there with nothing left to try.

WHAT IS CHECKED HERE, AND WHAT IS NOT.  These tests force the rungs to fail and
require the ladder to keep producing a usable subspace -- the plumbing.  The
NATURAL failing case is CO at 9 bohr itself, which costs a 14400-determinant
solve; it is exercised by the campaign rather than by this suite, and the rung
each geometry actually used is recorded in its `seed` field so the artifact says
so rather than this file promising it.

A rung is tried only when the one above it produced nothing, so any geometry
that succeeded before the ladder existed computes bit-identically now.  That is
tested too, because "additive" is a claim about behaviour and not a comment.
"""
import numpy as np
import scipy.sparse.linalg as spl

import fci as F

FAILED = []
CHECKS = [0]


def check(cond, label):
    CHECKS[0] += 1
    if not cond:
        FAILED.append(label)
        print("  FAIL  %s" % label)
    else:
        print("  ok    %s" % label)


class DegenerateOp(object):
    """A symmetric matrix whose four lowest eigenvalues are degenerate to 1e-15
    -- the spectral shape that defeats a tol=0 request, in miniature."""

    def __init__(self, n=240, seed=11):
        rng = np.random.RandomState(seed)
        Q, _ = np.linalg.qr(rng.randn(n, n))
        lam = np.arange(n, dtype=float) * 0.7 + 3.0
        lam[:4] = lam[0] + np.array([0.0, 1e-15, 2e-15, 3e-15])
        self.M = (Q * lam) @ Q.T
        self.M = (self.M + self.M.T) / 2
        self.ndet = n
        self.diag_f64 = np.diag(self.M).copy()
        self.exact = np.sort(np.linalg.eigvalsh(self.M))

    def matvec_f64(self, c):
        return self.M @ c


def _lop(op):
    return spl.LinearOperator((op.ndet, op.ndet), matvec=op.matvec_f64,
                              dtype=float)


def _no_convergence(*a, **kw):
    raise spl.ArpackNoConvergence("forced", np.array([]), np.zeros((0, 0)))


def _partial_of_one(op):
    def f(*a, **kw):
        w, V = np.linalg.eigh(op.M)
        raise spl.ArpackNoConvergence("forced partial",
                                      w[:1], V[:, :1])
    return f


def test_rung_a_unchanged():
    print("\n1. rung (a) is what it always was")
    op = DegenerateOp()
    w, V, seed = F._f64_seed(op, _lop(op), op.ndet, 6)
    check(seed == "sparse f64 Lanczos (ARPACK)",
          "an ordinary problem still takes rung (a): %r" % seed)
    check(abs(np.sort(w)[0] - op.exact[0]) < 1e-8,
          "and its lowest eigenvalue is right (%.3e)"
          % abs(np.sort(w)[0] - op.exact[0]))


def test_ladder_falls_through_to_lobpcg():
    print("\n2. every ARPACK rung forced to fail; the ladder must still seed")
    op = DegenerateOp()
    real = spl.eigsh
    spl.eigsh = _no_convergence
    try:
        w, V, seed = F._f64_seed(op, _lop(op), op.ndet, 6)
    finally:
        spl.eigsh = real
    check("LOBPCG" in seed, "the last resort names itself: %r" % seed)
    check(len(w) >= 2, "at least two eigenvalues came back (%d)" % len(w))
    check(V.shape[0] == op.ndet, "the vectors have the right length")
    err = abs(np.sort(np.asarray(w).ravel())[0] - op.exact[0])
    check(err < 1e-6, "the seed's lowest eigenvalue is usable (err %.2e)" % err)
    v = np.asarray(V)[:, int(np.argmin(np.asarray(w).ravel()))]
    v = v / np.linalg.norm(v)
    rq = float(v @ op.matvec_f64(v))
    check(abs(rq - op.exact[0]) < 1e-6,
          "and its Rayleigh quotient matches (err %.2e)"
          % abs(rq - op.exact[0]))


def test_partial_of_one_does_not_crash():
    print("\n3. an ARPACK partial carrying ONE vector")
    # The pre-ladder code accepted a partial of length >= 1 and then indexed
    # w[o[1]] to form the gap -- an IndexError on exactly the geometries the
    # partial path exists for.  The ladder requires two before it accepts one.
    op = DegenerateOp()
    real = spl.eigsh
    spl.eigsh = _partial_of_one(op)
    try:
        w, V, seed = F._f64_seed(op, _lop(op), op.ndet, 6)
    finally:
        spl.eigsh = real
    check(len(np.asarray(w).ravel()) >= 2,
          "a one-vector partial is not accepted as a subspace; the ladder "
          "went on to %r and returned %d" % (seed, len(np.asarray(w).ravel())))


def test_seed_without_a_diagonal():
    print("\n4. the last resort with no diagonal to precondition with")
    op = DegenerateOp()
    op.diag_f64 = None                  # RouteBOp can be in this state
    real = spl.eigsh
    spl.eigsh = _no_convergence
    try:
        w, V, seed = F._f64_seed(op, _lop(op), op.ndet, 6)
    finally:
        spl.eigsh = real
    check(len(np.asarray(w).ravel()) >= 2,
          "no diagonal, no preconditioner, still a subspace (%r)" % seed)


def test_solve_certified_end_to_end_through_the_ladder():
    print("\n5. solve_certified itself, with ARPACK removed underneath it")
    op = DegenerateOp(n=120, seed=5)
    real = spl.eigsh
    spl.eigsh = _no_convergence
    try:
        r = F.solve_certified(_HpWrap(op), tol_digits=20, max_outer=6)
    finally:
        spl.eigsh = real
    err = abs(float(r["energy"]) - op.exact[0])
    check(err < 1e-14,
          "the certified solve converges from the last-resort seed "
          "(err %.2e, ||r|| %.2e)" % (err, float(r["resid"])))


class _HpWrap(object):
    """DegenerateOp with the working-precision matvec solve_certified needs."""

    def __init__(self, op):
        from mpmath import mpf
        self._op = op
        self.ndet = op.ndet
        self.diag_f64 = op.diag_f64
        self._rows = [[mpf(float(x)) for x in row] for row in op.M]

    def matvec_f64(self, c):
        return self._op.matvec_f64(c)

    def matvec_hp(self, c):
        from mpmath import mpf
        out = []
        for row in self._rows:
            acc = mpf(0)
            for j in range(len(c)):
                if c[j] != 0:
                    acc += row[j] * c[j]
            out.append(acc)
        return out


if __name__ == "__main__":
    test_rung_a_unchanged()
    test_ladder_falls_through_to_lobpcg()
    test_partial_of_one_does_not_crash()
    test_seed_without_a_diagonal()
    test_solve_certified_end_to_end_through_the_ladder()
    print("\n%d checks, %d FAIL" % (CHECKS[0], len(FAILED)))
    for f in FAILED:
        print("   FAILED: %s" % f)
    raise SystemExit(1 if FAILED else 0)
