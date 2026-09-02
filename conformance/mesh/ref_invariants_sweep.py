"""Which REG+ collision laws carry SPURIOUS linear invariants? All 4608, at L=6.

The lead suggested staking Zanetti's staggered momentum invariants in advance. The
complete-invariant solver returns 3 for FHP-I -- mass and the two momenta, nothing
spurious -- so rather than stake or dismiss a remembered formula, this sweeps the WHOLE
group and reports the distribution. A law with more than 3 carries extra exactly-closed
views; the sweep says which laws those are, so "FHP-I has none" is placed inside a map
rather than asserted alone.
"""
import itertools, numpy as np
from collections import Counter
from lattice_common import DIR, FIBERS, fhp_i
from ref_invariants import invariant_space

NONTRIVIAL = [v for v in FIBERS.values() if len(v) > 1]

def laws():
    for choice in itertools.product(*[list(itertools.permutations(v)) for v in NONTRIVIAL]):
        C = list(range(64))
        for orig, perm in zip(NONTRIVIAL, choice):
            for a, b in zip(orig, perm):
                C[a] = b
        yield C

if __name__ == "__main__":
    L = 6
    hist = Counter()
    example = {}
    n = 0
    for C in laws():
        d = invariant_space(L, C)
        hist[d] += 1
        example.setdefault(d, [s for s in range(64) if C[s] != s])
        n += 1
    print(f"L={L}, {n} sector-preserving collision laws swept")
    print(f"{'invariants':>11} {'laws':>7} {'spurious':>9}   acts on (first example)")
    for d in sorted(hist):
        print(f"{d:>11} {hist[d]:>7} {d-3:>9}   {example[d][:12]}")
    print()
    fi = invariant_space(L, fhp_i())
    print(f"FHP-I itself: {fi} invariants, {fi-3} spurious")
    print(f"laws with NO spurious linear invariant: {hist[3]} of {n} "
          f"({100.0*hist[3]/n:.1f}%)")
