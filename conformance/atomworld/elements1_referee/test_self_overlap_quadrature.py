#!/usr/bin/env python3
"""Pin `_self_overlap` against NUMERICAL QUADRATURE, not against another closed form.

# Why quadrature and not algebra

`_self_overlap` was generalised from a two-branch special case to `v * _dfact(l) / (2p)**l`
so that d shells normalise correctly. That is right only if `_dfact(l)` means `(2l-1)!!`
(1, 1, 3, 15) rather than the ordinary double factorial of its argument (1, 1, 2, 3), which
is what most libraries named `dfact` or `double_factorial` compute.

Under the wrong reading every d self-overlap is off by exactly 3/2 -- and it would be
SILENT. A uniform factor on a normalisation produces no NaN and no refusal; it produces
converged d energies that are wrong, which is precisely the failure mode the old
`v if l == 0 else v / (2 * p)` had and which this generalisation was written to remove.

The check is therefore against a numerical integral of the actual defining integrand, not
against a second closed form: **two closed forms can share a mistake, and a quadrature
cannot share it with either.** (The point, and the hazard, are mixtures-referee's.)

    <x^l | x^l> at one centre  =  N_a N_b (pi/p)^{3/2} * (2l-1)!! / (2p)^l

so the ratio to the l = 0 value is `(2l-1)!! / (2p)^l`, which is what is measured here.

Usage: `python3 test_self_overlap_quadrature.py`
"""

import sys

from mpmath import mp, mpf, quad, exp, inf, sqrt, pi

import elements_core as E

mp.dps = 50

# Two exponent pairs, so a coincidence at one `p` cannot pass.
CASES = [(mpf("1.3"), mpf("0.7")), (mpf("5.0"), mpf("2.0"))]
LMAX = 3


def one_d_moment(l, p):
    """int x^{2l} exp(-p x^2) dx over the whole line, by quadrature."""
    return quad(lambda x: x ** (2 * l) * exp(-p * x**2), [-inf, inf])


def main():
    bad = 0
    print(f"{'l':>2} {'p':>6} {'quadrature ratio':>26} {'_self_overlap ratio':>26} {'|delta|':>10}")
    for a, b in CASES:
        p = a + b
        # The quadrature truth: the 3-D integral factorises, and only the x direction
        # carries the angular momentum, so the ratio to l = 0 is the 1-D moment ratio.
        base = one_d_moment(0, p)
        for l in range(LMAX + 1):
            want = one_d_moment(l, p) / base

            # What the function under test says, with the primitive normalisations
            # divided back out so only the l-dependence remains.
            na = E.prim_norm(a, (l, 0, 0))
            nb = E.prim_norm(b, (l, 0, 0))
            na0 = E.prim_norm(a, (0, 0, 0))
            nb0 = E.prim_norm(b, (0, 0, 0))
            got = (
                E._self_overlap(a, na, b, nb, l)
                / E._self_overlap(a, na0, b, nb0, 0)
                * (na0 * nb0)
                / (na * nb)
            )

            d = abs(got - want)
            flag = "" if d < mpf("1e-30") else "   <-- MISMATCH"
            if flag:
                bad += 1
            print(f"{l:>2} {float(p):>6.2f} {mp.nstr(want, 20):>26} {mp.nstr(got, 20):>26} {float(d):>10.2e}{flag}")

    # The specific value the hazard turns on, stated so a reader sees the number that
    # separates the two conventions rather than only a pass.
    print()
    print(f"_dfact(2) = {E._dfact(2)}  -- must be 3 ((2k-1)!!). The ordinary double")
    print("factorial of the argument would give 2, and every d self-overlap would be")
    print("wrong by exactly 3/2, silently.")
    if E._dfact(2) != 3 or E._dfact(3) != 15:
        print("REFUSED: _dfact does not mean (2k-1)!!")
        bad += 1

    # THE PLANT. Passing on the correct convention proves nothing unless the check can
    # fail on the wrong one, so the wrong one is installed and the test must refuse it.
    print()
    real = E._dfact
    try:
        def ordinary_double_factorial(k):
            """k!! -- what almost every library named dfact computes. 1, 1, 2, 3, 8."""
            r = 1
            while k > 1:
                r *= k
                k -= 2
            return r

        E._dfact = ordinary_double_factorial
        a, b = CASES[0]
        p = a + b
        base = one_d_moment(0, p)
        want = one_d_moment(2, p) / base
        na, nb = E.prim_norm(a, (2, 0, 0)), E.prim_norm(b, (2, 0, 0))
        na0, nb0 = E.prim_norm(a, (0, 0, 0)), E.prim_norm(b, (0, 0, 0))
        got = (
            E._self_overlap(a, na, b, nb, 2)
            / E._self_overlap(a, na0, b, nb0, 0)
            * (na0 * nb0)
            / (na * nb)
        )
        caught = abs(got - want) > mpf("1e-30")
        print(
            f"plant (ordinary double factorial): l=2 gives {mp.nstr(got, 12)} against the "
            f"quadrature's {mp.nstr(want, 12)}"
        )
        if caught:
            print(f"  -> CAUGHT, off by {mp.nstr(want / got, 6)}x")
        else:
            print("  -> PLANT MISSED: the wrong convention passed the quadrature check")
            bad += 1
    finally:
        E._dfact = real

    print()
    print("REFUSED" if bad else "OK: self-overlap matches quadrature for l = 0..3, and the wrong convention is caught")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
