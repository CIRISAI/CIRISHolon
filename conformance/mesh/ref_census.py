"""LG_PREREG §5.1 — the census classifies the dynamics.

Reproduces Core/Lattice.lean's sector_count (53) and sector_dims (44/7/2) and
reads off what they say about the collision law: a sector-preserving map permutes
within fibers and can do nothing else, so the whole space of REG+ collision laws
on FHP-6 is the product of the fibers' symmetric groups.
"""
import math
from collections import defaultdict
from lattice_common import FIBERS, lab, fhp_i

if __name__ == "__main__":
    dims = defaultdict(int)
    for v in FIBERS.values():
        dims[len(v)] += 1
    print("sectors", len(FIBERS), "dimension histogram", dict(sorted(dims.items())))
    print("dimension-3 sectors:", {k: v for k, v in FIBERS.items() if len(v) == 3})
    print("dimension-2 sectors:", {k: v for k, v in FIBERS.items() if len(v) == 2})
    print("collision group order:",
          math.prod(math.factorial(len(v)) for v in FIBERS.values()))
    C = fhp_i()
    acts = [s for s in range(64) if C[s] != s]
    print("FHP-I acts on:", acts, "-> labels", [lab(s) for s in acts])
    print("FHP-I is a bijection:", sorted(C) == list(range(64)))
    print("FHP-I is sector-preserving:", all(lab(C[s]) == lab(s) for s in range(64)))
