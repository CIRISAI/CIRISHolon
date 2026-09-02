"""LG_PREREG §5.4 — the closure defect belongs to the LATTICE, not to the collision law.

Enumerates ALL sector-preserving collision laws on FHP-6 and measures the k=1
block-chart witness rate under each. M-ONE-MODEL-DELTA: a defect measured against
one chosen law earns "worse than that law"; exhausting the alternatives is what
earns the unconditional statement.
"""
import itertools, math
from collections import defaultdict
from lattice_common import DIR, FIBERS, SUCC, MOVABLE

NONTRIVIAL = [v for v in FIBERS.values() if len(v) > 1]


def contrib(cell, s, b, L, C):
    """Block -> (N,Px,Py) landed by cell `cell` holding state `s`, after C then S."""
    out = defaultdict(lambda: [0, 0, 0])
    i, j = cell
    for d in range(6):
        if C[s] >> d & 1:
            ii, jj = (i + DIR[d][0]) % L, (j + DIR[d][1]) % L
            acc = out[(ii // b, jj // b)]
            acc[0] += 1
            acc[1] += DIR[d][0]
            acc[2] += DIR[d][1]
    return {k: tuple(v) for k, v in out.items()}


def rate(b, L, C):
    """Exact k=1 witness rate: position within a block is all that matters."""
    hit = tot = 0
    for i in range(b):
        for j in range(b):
            for s in MOVABLE:
                tot += 1
                if contrib((i, j), s, b, L, C) != contrib((i, j), SUCC[s], b, L, C):
                    hit += 1
    return hit / tot


if __name__ == "__main__":
    order = math.prod(math.factorial(len(v)) for v in NONTRIVIAL)
    print("sector-preserving collision group order:", order)
    seen = set()
    for choice in itertools.product(*[list(itertools.permutations(v)) for v in NONTRIVIAL]):
        C = list(range(64))
        for orig, perm in zip(NONTRIVIAL, choice):
            for a, b_ in zip(orig, perm):
                C[a] = b_
        seen.add(round(rate(8, 64, C), 12))
    print(f"distinct k=1 defect rates over all {order} laws at b=8:", sorted(seen))
    print("geometric bound W(8) =", 1 - (6 * 6) / 64)
