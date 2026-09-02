"""The FHP-6 local state space, shared by every reference script in this node.

Derived from CIRISOntology/Core/Lattice.lean: six axial directions, 64 local
states, the (N,P) label, and its 53 fibers. Nothing here is dynamics.
"""
from collections import defaultdict

DIR = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)]


def lab(s):
    """The conserved (N, Px, Py) label of local state `s` — Lattice.lean's `np`."""
    n = px = py = 0
    for k in range(6):
        if s >> k & 1:
            n += 1
            px += DIR[k][0]
            py += DIR[k][1]
    return (n, px, py)


FIBERS = defaultdict(list)
for _s in range(64):
    FIBERS[lab(_s)].append(_s)
FIBERS = dict(FIBERS)

#: cyclic successor within a state's own fiber — the LG_PREREG §6.2 fiber move.
SUCC = {}
for _v in FIBERS.values():
    if len(_v) > 1:
        for _a, _b in zip(_v, _v[1:] + _v[:1]):
            SUCC[_a] = _b

#: the 20 states lying in a fiber of dimension >= 2, i.e. the ones a fiber move can touch.
MOVABLE = sorted(SUCC)


def fhp_i(deterministic=True):
    """FHP-I as a permutation of the 64 local states.

    The 3-cycle on the head-on fiber {9,18,36} (Lattice.lean's `three_route_sector`)
    and the swap on the three-body fiber {21,42}. Identity on the other 50 states,
    which is forced: a sector-preserving map is the identity on every fiber of
    dimension 1, and there are 44 of those.
    """
    c = list(range(64))
    if deterministic:
        c[9], c[18], c[36] = 18, 36, 9
    else:
        c[9], c[18], c[36] = 36, 9, 18
    c[21], c[42] = 42, 21
    return c
