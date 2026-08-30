#!/usr/bin/env python3
"""refute_funnels.py — two alternative funnels of comparable simplicity that
select a DIFFERENT famous physics family from the same 430 groups."""
import math
import numpy as np
import landscape_sweep as ls
import refute_lib as rl


# ------------------------------------------------------------- group helpers

def subgroup_closure(G, gens):
    S = set(gens) | {G.IDE}
    changed = True
    while changed:
        changed = False
        new = set()
        for x in S:
            for y in S:
                z = int(G.MUL[x, y])
                if z not in S:
                    new.add(z)
                    changed = True
        S |= new
    return S


def commutators_with(G, H):
    out = set()
    for h in H:
        for g in range(G.order):
            out.add(int(G.MUL[G.MUL[h, g], G.MUL[G.INV[h], G.INV[g]]]))
    return subgroup_closure(G, out)


def is_nilpotent(G):
    cur = set(range(G.order))
    for _ in range(12):
        nxt = commutators_with(G, cur)
        if nxt == cur:
            return False
        cur = nxt
        if len(cur) == 1:
            return True
    return False


def exponent(G):
    return max(G.element_orders)


def quotient_by_center_elem_abelian(G):
    Z = G.center
    p = len(Z)
    if p < 2 or any(p % q == 0 and p != q for q in (2, 3, 5, 7, 11)):
        pass
    if not all(x in Z for x in G.commutator_subgroup):
        return False
    # (gZ)^p = Z for all g
    for g in range(G.order):
        cur, x = 1, g
        while cur < p:
            x = int(G.MUL[x, g])
            cur += 1
        if x not in Z:
            return False
    return True


def is_prime(m):
    return m > 1 and all(m % d for d in range(2, int(m ** 0.5) + 1))


def is_extraspecial(G):
    Z = G.center
    if not is_prime(len(Z)):
        return False
    if set(G.commutator_subgroup) != set(Z):
        return False
    return quotient_by_center_elem_abelian(G)


# ------------------------------------------------ real representations (FS)

def fs_and_kernels(G):
    """returns list of (degree, fs_indicator, kernel_bitmask) per irreducible"""
    chars, degs, sizes, CLS, reps = rl.character_table(G)
    ok, why = rl.validate_table(G, chars, degs, sizes)
    if not ok:
        return None
    degs_i = np.round(degs).astype(int)
    kers = rl.kernels(G, chars, degs, CLS)
    sq = np.array([CLS[int(G.MUL[int(r), int(r)])] for r in reps])
    out = []
    for t in range(len(degs_i)):
        nu = float(np.real(np.sum(sizes * chars[t][sq]) / G.order))
        out.append((int(degs_i[t]), round(nu), kers[t]))
    return out


def has_faithful_real_rep_deg_le(G, D, info):
    """faithful representation of total degree <= D, every constituent
    realisable over R (Frobenius-Schur indicator +1)."""
    idbit = 1 << G.IDE
    real = [(d, k) for (d, nu, k) in info if nu == 1 and d <= D]
    full = (1 << G.order) - 1
    states = {(D, full)}
    frontier = [(D, full)]
    while frontier:
        nxt = []
        for (b, k) in frontier:
            if k == idbit:
                return True
            for (d, kk) in real:
                if d <= b:
                    st = (b - d, k & kk)
                    if st not in states:
                        states.add(st)
                        nxt.append(st)
        frontier = nxt
    return any(k == idbit for (_, k) in states)


# ------------------------------------------------------- crystallographic set

def crystallographic_targets():
    z2 = ls.build_cyclic(2)
    reps = [ls.build_dihedral(3), ls.build_dihedral(4), ls.build_dihedral(6),
            ls.build_alternating(4), ls.build_symmetric(4)]
    out = list(reps) + [ls.build_direct_product(z2, r) for r in reps[:5]]
    return {g.structural_signature() for g in out}


def main():
    groups = ls.generate_landscape()
    print(f"population: {len(groups)}\n", flush=True)

    info = {}
    for g in groups:
        if len(g.commutator_subgroup) > 1:
            info[g.name] = fs_and_kernels(g)

    cryst = crystallographic_targets()
    nonab_cryst = {s for s in cryst}

    # ---------------- funnel X: the Pauli / Weyl-Heisenberg tower
    X = []
    for g in groups:
        x1 = len(g.commutator_subgroup) > 1
        if not x1:
            continue
        x2 = is_nilpotent(g)
        x3 = is_prime(len(g.center))
        x4 = exponent(g) <= len(g.center) ** 2 if x3 else False
        if x1 and x2 and x3 and x4:
            X.append(g)
    tX = [g for g in X if is_extraspecial(g)]
    poolX = [g for g in groups if len(g.commutator_subgroup) > 1]
    baseX = sum(1 for g in poolX if is_extraspecial(g))
    print("FUNNEL X — 'the Pauli / Weyl-Heisenberg tower' (extraspecial p-groups)")
    print("  X1 nonabelian | X2 nilpotent | X3 |Z(G)| prime | X4 exponent <= |Z(G)|^2")
    print(f"  survivors: {len(X)}   on-target (extraspecial): {len(tX)}"
          f"   precision {len(tX)/max(len(X),1):.4f}")
    print(f"  base rate of target in the post-X1 pool: {baseX}/{len(poolX)} = {baseX/len(poolX):.4f}")
    print(f"  survivors: {sorted(g.name for g in X)}\n")

    # ---------------- funnel Y: the 32 crystallographic point groups
    Y = []
    for g in groups:
        y1 = len(g.commutator_subgroup) > 1
        if not y1:
            continue
        y2 = set(g.element_orders) <= {1, 2, 3, 4, 6}
        if not y2:
            continue
        inf = info.get(g.name)
        if inf is None:
            continue
        y3 = has_faithful_real_rep_deg_le(g, 3, inf)
        if y1 and y2 and y3:
            Y.append(g)
    tY = [g for g in Y if g.structural_signature() in nonab_cryst]
    poolY = [g for g in groups if len(g.commutator_subgroup) > 1]
    baseY = sum(1 for g in poolY if g.structural_signature() in nonab_cryst)
    print("FUNNEL Y — 'the crystallographic point groups' (all of solid-state physics)")
    print("  Y1 nonabelian | Y2 element orders in {1,2,3,4,6} | Y3 faithful real rep of degree <= 3")
    print(f"  survivors: {len(Y)}   on-target (nonabelian crystallographic point group): {len(tY)}"
          f"   precision {len(tY)/max(len(Y),1):.4f}")
    print(f"  base rate of target in the post-Y1 pool: {baseY}/{len(poolY)} = {baseY/len(poolY):.4f}")
    print(f"  survivors: {sorted(g.name for g in Y)}")
    print(f"  off-target: {sorted(g.name for g in Y if g.structural_signature() not in nonab_cryst)}\n")

    S = [g for g in groups if g.full_selector_pass]
    print("FUNNEL C (SELECTOR-5, for comparison)")
    print(f"  survivors: {len(S)}   on-target (RULE-A): 44   precision 0.9778")
    print(f"  base rate of RULE-A target in the whole population: "
          f"{sum(1 for g in groups if g.is_lie_type)}/{len(groups)} = "
          f"{sum(1 for g in groups if g.is_lie_type)/len(groups):.4f}")

    # overlap
    print(f"\noverlap X n C = {sorted(set(g.name for g in X) & set(g.name for g in S))}")
    print(f"overlap Y n C = {sorted(set(g.name for g in Y) & set(g.name for g in S))}")


main()
