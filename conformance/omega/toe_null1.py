#!/usr/bin/env python3
"""TOE-NULL-1 verifier: exact, self-contained. See TOE_NULL1_RESULTS.md.
Checks: (1) 48 closed reversible dynamics on the 6-state/3-fiber model with
macro-type census 8/24/16; (2) the view-lattice rent spectra of the three
designated representative worlds are 0/50/63 total over 203 views (0/172/195
nonzero); (3) the STASIS THEOREM instance: among ALL 720 permutations,
exactly the identity has zero rent on every view of the lattice."""
import sys
from fractions import Fraction as F
from itertools import permutations

S = list(range(6)); FIB = {0:0,1:0,2:1,3:1,4:2,5:2}; mu = F(1,6)

def all_partitions(seq):
    if not seq: yield []
    else:
        first, rest = seq[0], seq[1:]
        for part in all_partitions(rest):
            for i in range(len(part)):
                yield part[:i] + [[first]+part[i]] + part[i+1:]
            yield [[first]] + part

PARTS = sorted({tuple(tuple(sorted(b)) for b in sorted(map(sorted, pt)))
                for pt in all_partitions(S)})

def rent(p, blocks):
    lab = {s: bi for bi, b in enumerate(blocks) for s in b}
    P = {}
    for s in S:
        k = (lab[s], lab[p[s]]); P[k] = P.get(k, F(0)) + mu
    return 1 - sum(max(P.get((i, j), F(0)) for j in range(len(blocks)))
                   for i in range(len(blocks)))

def closed(p):
    img = {}
    for s in S:
        i, j = FIB[s], FIB[p[s]]
        if img.setdefault(i, j) != j: return False
    return len(set(img.values())) == 3

def macro_order(p):
    m = {i: FIB[p[[s for s in S if FIB[s]==i][0]]] for i in range(3)}
    x, k = (0,1,2), 0
    while True:
        x = tuple(m[i] for i in x); k += 1
        if x == (0,1,2): return k

CP = [p for p in permutations(S) if closed(p)]
census = {}
for p in CP: census[macro_order(p)] = census.get(macro_order(p), 0) + 1
assert len(CP) == 48 and census == {1: 8, 2: 24, 3: 16}, (len(CP), census)

reps = {o: next(p for p in CP if macro_order(p) == o) for o in (1, 2, 3)}
tot = {o: sum(rent(p, [list(b) for b in pt]) for pt in PARTS) for o, p in reps.items()}
nz = {o: sum(1 for pt in PARTS if rent(p, [list(b) for b in pt]) != 0) for o, p in reps.items()}
assert (tot[1], tot[2], tot[3]) == (0, 50, 63), tot
assert (nz[1], nz[2], nz[3]) == (0, 172, 195), nz

zero_everywhere = [p for p in permutations(S)
                   if all(rent(p, [list(b) for b in pt]) == 0 for pt in PARTS)]
assert zero_everywhere == [tuple(S)], "stasis theorem instance failed"

print("TOE-NULL-1: 48 dynamics (8/24/16); spectra 0/50/63 over 203 views "
      "(0/172/195 nonzero); identity is the UNIQUE lattice-wide zero among 720. "
      "ALL CHECKS PASS")
