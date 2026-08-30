#!/usr/bin/env python3
"""refute_dedup.py — is the 430 a census of isomorphism types, and is the
gauntlet an isomorphism invariant?

The deduplication key `structural_signature()` includes `schur_multiplier`,
which `compute_schur_multiplier` computes by branching on `G.family` -- a
CONSTRUCTION TAG.  So two isomorphic groups built under different tags can
receive different keys, survive deduplication as separate 'non-isomorphic'
entries, and receive DIFFERENT gauntlet verdicts.
"""
import itertools
import numpy as np
import landscape_sweep as ls


def tagless_signature(g):
    return (g.order,
            tuple(sorted(g.element_orders)),
            tuple(sorted(len(c) for c in g.classes)),
            len(g.commutator_subgroup),
            len(g.center),
            g.num_non_ambivalent_classes)


def isomorphic(G, H, limit=400000):
    """explicit isomorphism search on generators (small groups only)"""
    if G.order != H.order:
        return False
    n = G.order
    # find a small generating set of G
    def closure(Gg, gens):
        S = {Gg.IDE}
        frontier = [Gg.IDE]
        while frontier:
            nxt = []
            for x in frontier:
                for a in gens:
                    y = int(Gg.MUL[x, a])
                    if y not in S:
                        S.add(y)
                        nxt.append(y)
            frontier = nxt
        return S
    gens = []
    cur = {G.IDE}
    for g in sorted(range(n), key=lambda x: -G.element_orders[x]):
        if g not in cur:
            gens.append(g)
            cur = closure(G, gens)
            if len(cur) == n:
                break
    ordG = [G.element_orders[a] for a in gens]
    cands = [[h for h in range(n) if H.element_orders[h] == o] for o in ordG]
    tried = 0
    for combo in itertools.product(*cands):
        tried += 1
        if tried > limit:
            return None
        # build the map by BFS over words
        phi = {G.IDE: H.IDE}
        frontier = [G.IDE]
        ok = True
        while frontier and ok:
            nxt = []
            for x in frontier:
                for a, b in zip(gens, combo):
                    y = int(G.MUL[x, a])
                    v = int(H.MUL[phi[x], b])
                    if y in phi:
                        if phi[y] != v:
                            ok = False
                            break
                    else:
                        phi[y] = v
                        nxt.append(y)
                if not ok:
                    break
            frontier = nxt
        if ok and len(phi) == n and len(set(phi.values())) == n:
            if all(H.MUL[phi[x], phi[y]] == phi[int(G.MUL[x, y])]
                   for x in range(n) for y in range(n)):
                return True
    return False


groups = ls.generate_landscape()
buckets = {}
for g in groups:
    buckets.setdefault(tagless_signature(g), []).append(g)
dups = {k: v for k, v in buckets.items() if len(v) > 1}

print(f"population: {len(groups)} entries claimed non-isomorphic")
print(f"tagless-signature collisions: {len(dups)} buckets, "
      f"{sum(len(v) for v in dups.values())} entries\n")

confirmed = 0
for k, v in sorted(dups.items(), key=lambda kv: kv[0][0]):
    names = [g.name for g in v]
    verdicts = [g.full_selector_pass for g in v]
    c4 = [g.gate_c4 for g in v]
    schur = [g.schur_multiplier for g in v]
    fams = [g.family for g in v]
    iso = isomorphic(v[0], v[1])
    if iso:
        confirmed += 1
    split = "  *** SPLIT VERDICT ***" if len(set(verdicts)) > 1 else ""
    print(f"|G|={k[0]:4d}  {names}  families={fams}")
    print(f"          isomorphic={iso}  schur={schur}  C4={c4}  FULL={verdicts}{split}")

print(f"\nconfirmed-isomorphic duplicate buckets: {confirmed}")
