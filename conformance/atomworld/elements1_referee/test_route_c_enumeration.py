"""Route C's determinant enumeration: the same list, without the 2**nso walk.

THE DEFECT. `route_c_energy` built its determinant list as

    [d for d in range(1 << nso) if bin(d).count("1") == nelec]

-- walk every nso-bit integer, keep the ones with the right population count.
That is 2**nso iterations whatever the answer's size.  `route_c_cost`, which
decides whether route C is affordable, models `math.comb(nso, nelec)`: the size
of the ANSWER.

So the budget guard was blindest exactly where the answer was smallest.  A
CLOSED SHELL is that case by definition -- every spin orbital occupied, one
determinant, and a search space of 2**nso:

    Ar2   nso = 36, nelec = 36  ->  1 determinant, 68,719,476,736 iterations
    NeAr  nso = 28, nelec = 28  ->  1 determinant,     268,435,456 iterations

Ar2 passed the check with a modelled cost of 4.2e5 against a 4.0e7 budget, and
then held eight workers at 98% CPU for fifty minutes without finishing a single
geometry.  Nothing failed; nothing warned; the job simply did not advance, which
is the hardest shape to notice from outside.

THE REPAIR is to choose the bit patterns instead of filtering for them, which
makes the enumeration O(number of determinants) -- what `route_c_cost` was
modelling all along.  The list is SORTED so the order is bit-for-bit the
filter's, because `idx` maps determinant to matrix row: a different order is a
different matrix, with the same eigenvalue and different everything else, and
there is no reason to accept that risk for nothing.

WHAT IS CHECKED HERE.  That the two enumerations produce the IDENTICAL LIST,
order included, over a range of sizes; that the closed-shell cases the filter
could not finish are now instant; that the cost model is honest about what it
assumes; and -- the one that matters -- that a route C ENERGY computed the new
way equals one that was computed and cached the old way, to all 50 digits.
"""
import itertools
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import fci as F                                                # noqa: E402

FAILED = []
CHECKS = [0]


def check(cond, label):
    CHECKS[0] += 1
    if not cond:
        FAILED.append(label)
        print("  FAIL  %s" % label)
    else:
        print("  ok    %s" % label)


def old_enumeration(nso, nelec):
    """The filter, kept here so the equivalence is against the real thing."""
    return [d for d in range(1 << nso) if bin(d).count("1") == nelec]


def test_identical_list():
    print("\n1. the same list, order included")
    for nso, ne in ((4, 2), (8, 4), (10, 6), (12, 6), (14, 7), (16, 8),
                    (20, 10)):
        old = old_enumeration(nso, ne)
        new = F.route_c_determinants(nso, ne)
        check(old == new,
              "nso=%d nelec=%d: %d determinants, identical" % (nso, ne,
                                                               len(old)))
    print("\n   and the edges")
    check(F.route_c_determinants(6, 0) == [0], "zero electrons is one empty "
                                               "determinant")
    check(F.route_c_determinants(6, 6) == [(1 << 6) - 1],
          "a full shell is one determinant, all bits set")
    check(F.route_c_determinants(6, 7) == [],
          "more electrons than spin orbitals is empty, not an error")


def test_closed_shells_are_instant():
    print("\n2. the cases the filter could not finish")
    for nso, ne, walk in ((28, 28, 268435456), (36, 36, 68719476736)):
        t = time.time()
        d = F.route_c_determinants(nso, ne)
        el = time.time() - t
        check(len(d) == 1 and el < 1.0,
              "nso=%d nelec=%d -> %d determinant in %.4fs; the filter would "
              "have walked %s integers" % (nso, ne, len(d), el,
                                           format(walk, ",")))


def test_cost_model_is_now_honest():
    print("\n3. the cost model, and what it assumes")
    # It models the determinant count.  That is only an honest model of the
    # work because the enumeration is now O(nd).  Assert the relationship the
    # comment claims, so a regression in the enumeration shows up here.
    for norb, nelec in ((10, 18), (14, 26), (18, 36), (18, 34)):
        nd, cost = F.route_c_cost(norb, nelec)
        t = time.time()
        got = len(F.route_c_determinants(2 * norb, nelec))
        el = time.time() - t
        check(got == nd,
              "norb=%d nelec=%d: the model's nd (%d) is the number actually "
              "enumerated" % (norb, nelec, nd))
        check(el < 5.0,
              "  and enumerating them took %.3fs, not 2**%d steps"
              % (el, 2 * norb))
    nd, cost = F.route_c_cost(18, 36)
    check(cost < 4.0e7,
          "Ar2's modelled cost (%.1e) is under the budget -- which is now TRUE "
          "rather than merely modelled" % cost)


def test_energy_unchanged():
    print("\n4. a route C energy, against one computed the old way")
    # The cached record was written before this change; recomputing it now
    # exercises the new enumeration through the whole route.
    import build_curves as B
    import runner as R
    import species as SP
    from mpmath import mp, mpf
    import elements_core as EC
    mp.dps = R.DPS
    done = 0
    for name in ("H2", "LiH", "HF"):
        if name not in SP.DIATOMICS:
            continue
        for rs in SP.grid_for(name):
            c = R.cache_get(B.tag_for(name, rs, R.DPS), R.DPS, kind="point")
            if not c or not c.get("E_C"):
                continue
            d = SP.DIATOMICS[name]
            atoms = [(d["Z1"], (mpf(0), mpf(0), mpf(0))),
                     (d["Z2"], (mpf(0), mpf(0), mpf(rs)))]
            mol = EC.molecule(atoms)
            C, _ = F.lowdin_orbitals(mol["S"])
            h, g = F.mo_integrals(mol, C)
            ec, _asym, _n = F.route_c_energy(mol["nbf"], mol["nelec"], h, g)
            check(R.s(ec + mol["E_nuc"]) == c["E_C"],
                  "%s at R=%s: route C energy identical to the cached one at "
                  "all %d digits" % (name, rs, R.REPORT))
            done += 1
            break
        if done >= 2:
            break
    if not done:
        check(False, "at least one cached route C energy was available to "
                     "check against -- with none, this section proves nothing")


if __name__ == "__main__":
    test_identical_list()
    test_closed_shells_are_instant()
    test_cost_model_is_now_honest()
    test_energy_unchanged()
    FLOOR = 20
    print("\n%d checks, %d FAIL" % (CHECKS[0], len(FAILED)))
    if CHECKS[0] < FLOOR:
        print("   REFUSING TO PASS: %d checks is below this suite's floor of "
              "%d." % (CHECKS[0], FLOOR))
        raise SystemExit(1)
    for f in FAILED:
        print("   FAILED: %s" % f)
    raise SystemExit(1 if FAILED else 0)
