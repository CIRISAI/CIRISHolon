#!/usr/bin/env python3
"""Bit-identity baseline for the l <= 1 path, captured BEFORE the d extension.

# Why this exists

The d extension touches two lines of `elements_core.py` — a new `CART[2]` key, and
`_self_overlap` generalised from a two-branch special case to a formula in `l`. Both are
argued to be inert below `l = 2`: `CART[2]` is a dict key nothing existing reads, and the
general formula reduces to the current expressions at `l = 0` and `l = 1` term for term.

An argument is not a measurement. This captures what the second row actually computes now,
so the claim can be checked as EXACT EQUALITY rather than believed. A tolerance would be
the wrong instrument: none of the changes can touch an `l <= 1` path, so the correct
prediction is not "agrees to 1e-40" but "is the same number", and any tolerance wide enough
to write down is wide enough to hide a change that should not exist.

Every integral is emitted as an exact decimal string at the working precision, not as a
float, so the comparison never rounds through f64.

Usage:
    python3 second_row_baseline.py > second_row_baseline.txt
"""

import sys

from mpmath import mp, mpf, nstr

import elements_core as E

# The referee's working precision. Fixed here rather than inherited, so the baseline and
# the check are the same arithmetic even if a caller has changed it.
mp.dps = 60

# Species chosen to exercise every l <= 1 path the extension could disturb: an s-only pair,
# an s/p mix on two centres, and a p-heavy homonuclear. Small on purpose — the point is
# coverage of code paths, not of chemistry, and mpmath at 60 digits is not cheap.
CASES = [
    ("H2", [(1, ("0.0", "0.0", "0.0")), (1, ("0.0", "0.0", "1.4"))]),
    ("LiH", [(3, ("0.0", "0.0", "0.0")), (1, ("0.0", "0.0", "3.0"))]),
    ("HF", [(1, ("0.0", "0.0", "0.0")), (9, ("0.0", "0.0", "1.7"))]),
    ("N2", [(7, ("0.0", "0.0", "0.0")), (7, ("0.0", "0.0", "2.1"))]),
]

# Digits emitted per number. Below mp.dps so the last place is not printing noise from the
# guard digits, and far above anything a real change would hide under.
DIGITS = 50


def emit(tag, name, value):
    print(f"{tag} {name} {nstr(value, DIGITS, strip_zeros=False)}")


def main():
    print(f"# second-row bit-identity baseline, mp.dps = {mp.dps}, {DIGITS} digits emitted")
    print("# captured BEFORE the d extension to elements_core.py")
    print("# columns: KIND species index... value")
    for name, atoms in CASES:
        m = E.molecule([(Z, [mpf(x) for x in c]) for Z, c in atoms])
        n = m["nbf"]
        print(f"nbf {name} {n}")
        emit("enuc", name, m["E_nuc"])
        for i in range(n):
            for j in range(i + 1):
                emit(f"S:{i}:{j}", name, m["S"][i][j])
                emit(f"T:{i}:{j}", name, m["T"][i][j])
                emit(f"V:{i}:{j}", name, m["V"][i][j])
        # The two-electron array is the expensive half, so a deterministic diagonal slice
        # rather than all n^4: (ij|ij) over the unique pairs touches every shell quartet
        # class that the one-electron loop does not.
        for i in range(n):
            for j in range(i + 1):
                emit(f"G:{i}:{j}", name, m["eri"][i][j][i][j])
        # The raw shell norms are the cheapest single number that catches a change to the
        # normalisation path, which is exactly what the per-component factor touches.
        for k, sh in enumerate(m["shells"]):
            emit(f"raw:{k}", name, sh.raw_norm)
    print("# end", file=sys.stderr)


if __name__ == "__main__":
    main()
