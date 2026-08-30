#!/usr/bin/env python3
"""
heldout_sweep.py — Frozen SELECTOR-5 evaluation on held-out orders |G| in [129, 256].

Tests the forward prediction of the frozen SELECTOR-4 bootstrap criteria (C1..C4)
and evaluates enrichment over the held-out population under both RULE-A and RULE-B
(subgroup of SU(3) x SU(2) x U(1)).
"""
import sys, os, time, math, collections
import numpy as np

import landscape_sweep as ls
from landscape_sweep import FiniteGroup
import refute_lib as rl

def build_heldout_groups():
    groups = []
    print("Building held-out groups for orders 129..256...", flush=True)

    # 1. Cyclic groups Z_n (n = 129..256)
    for n in range(129, 257):
        groups.append(ls.build_cyclic(n))

    # 2. Dihedral groups D_{2n} (2n = 130..256, n = 65..128)
    for n in range(65, 129):
        groups.append(ls.build_dihedral(n))

    # 3. Dicyclic / Binary Dihedral groups Dic_n (4n = 132..256, n = 33..64)
    for n in range(33, 65):
        groups.append(ls.build_dicyclic(n))

    # 4. Frobenius groups F_{p, k} of order p * k in [129, 256]
    primes = [p for p in range(5, 256) if all(p % d != 0 for d in range(2, int(p**0.5) + 1))]
    for p in primes:
        for k in range(2, p):
            if (p - 1) % k == 0:
                ord_g = p * k
                if 129 <= ord_g <= 256:
                    groups.append(ls.build_frobenius(p, k))

    # 5. SU(3) finite subgroups Delta(3n^2) in [129, 256]
    # Delta(3 * 7^2) = Delta(147)
    # Delta(3 * 8^2) = Delta(192)
    # Delta(3 * 9^2) = Delta(243)
    for n in (7, 8, 9):
        ord_g = 3 * n * n
        if 129 <= ord_g <= 256:
            groups.append(ls.build_delta_3n2(n))

    # 6. Direct product gauge combinations with chiral cores
    core_2t = ls.build_binary_tetrahedral()       # order 24
    core_2o = ls.build_binary_octahedral()         # order 48
    core_2i = ls.build_binary_icosahedral()        # order 120
    core_a4 = ls.build_alternating(4)              # order 12
    core_d27 = ls.build_delta_3n2(3)               # order 27

    # Z_m x 2T (order 24m in 129..256 -> m = 6..10)
    for m in range(6, 11):
        zm = ls.build_cyclic(m)
        groups.append(ls.build_direct_product(zm, core_2t))

    # Z_m x 2O (order 48m in 129..256 -> m = 3..5)
    for m in range(3, 6):
        zm = ls.build_cyclic(m)
        groups.append(ls.build_direct_product(zm, core_2o))

    # Z_2 x 2I (order 240)
    groups.append(ls.build_direct_product(ls.build_cyclic(2), core_2i))

    # Z_m x A4 (order 12m in 129..256 -> m = 11..21)
    for m in range(11, 22):
        zm = ls.build_cyclic(m)
        groups.append(ls.build_direct_product(zm, core_a4))

    # Z_m x Delta(27) (order 27m in 129..256 -> m = 5..9)
    for m in range(5, 10):
        zm = ls.build_cyclic(m)
        groups.append(ls.build_direct_product(zm, core_d27))

    # Z_m x Dic_n
    for n in (3, 4, 5, 7):
        dic = ls.build_dicyclic(n) # order 4n
        for m in range(2, 257 // (4 * n) + 1):
            if 129 <= 4 * n * m <= 256:
                zm = ls.build_cyclic(m)
                groups.append(ls.build_direct_product(zm, dic))

    # 7. Semidihedral / Modular Maximal-Cyclic
    for n in (7, 8): # orders 128, 256
        if 2**n >= 129 and 2**n <= 256:
            groups.append(ls.build_semidihedral(n))
            groups.append(ls.build_modular_group(n))

    print(f"Generated {len(groups)} candidate groups in held-out range [129, 256].", flush=True)
    return groups

def run_heldout_experiment():
    t0 = time.time()
    candidates = [g for g in build_heldout_groups() if g is not None]

    # Deduplicate by structural signature
    seen = {}
    heldout = []
    for g in candidates:
        sig = g.structural_signature()
        if sig not in seen:
            seen[sig] = g
            heldout.append(g)
    heldout.sort(key=lambda g: (g.order, g.name))
    print(f"Unique non-isomorphic held-out groups: {len(heldout)}\n", flush=True)

    # Evaluate Gates
    c1_pass = [g for g in heldout if g.gate_c1]
    c2_pass = [g for g in heldout if g.gate_c2]
    c3_pass = [g for g in heldout if g.gate_c3]
    c4_pass = [g for g in heldout if g.gate_c4]

    p123 = [g for g in heldout if g.gate_c1 and g.gate_c2 and g.gate_c3]
    survivors = [g for g in heldout if g.full_selector_pass]

    # Evaluate RULE-B
    print("Evaluating RULE-B embedding in SU(3) x SU(2) x U(1)...", flush=True)
    rule_b = {}
    for i, g in enumerate(heldout):
        is_sm, _ = rl.rule_b_sm(g)
        rule_b[g.name] = is_sm
        if (i + 1) % 50 == 0:
            print(f"  Processed {i+1}/{len(heldout)} groups for RULE-B...", flush=True)

    # Gate breakdown table
    print("\n" + "="*80)
    print("HELDOUT EXPERIMENT RESULTS (|G| in [129, 256])")
    print("="*80)
    print(f"Total Held-Out Groups: {len(heldout)}")
    print(f"Gate C1 (Commutator Defect > 0) Pass:   {len(c1_pass):4d} / {len(heldout)} ({len(c1_pass)/len(heldout):.2%})")
    print(f"Gate C2 (Order Spectrum Entropy) Pass:  {len(c2_pass):4d} / {len(heldout)} ({len(c2_pass)/len(heldout):.2%})")
    print(f"Gate C3 (Orientation Index > 0) Pass:   {len(c3_pass):4d} / {len(heldout)} ({len(c3_pass)/len(heldout):.2%})")
    print(f"Gate C4 (Projective Schur Cover) Pass:  {len(c4_pass):4d} / {len(heldout)} ({len(c4_pass)/len(heldout):.2%})")
    print(f"C1 + C2 + C3 Pool:                      {len(p123):4d} / {len(heldout)} ({len(p123)/len(heldout):.2%})")
    print(f"FULL SELECTOR-4 GAUNTLET SURVIVORS:     {len(survivors):4d} / {len(heldout)} ({len(survivors)/len(heldout):.2%})")
    print("="*80)

    # Enrichment and Precision Analysis
    def pool_stats(name, pool):
        n = len(pool)
        if n == 0:
            return f"| {name:20s} | 0 | 0.0% | 0 | 0.0% |"
        sm_a = sum(1 for g in pool if g.is_lie_type)
        sm_b = sum(1 for g in pool if rule_b[g.name])
        return (f"| {name:20s} | {n:4d} | {sm_a:4d} ({sm_a/n:6.2%}) | {sm_b:4d} ({sm_b/n:6.2%}) |")

    print("\n| Pool                 |   N  | RULE-A SM (Tag)  | RULE-B SM (Embed) |")
    print("|----------------------|------|-------------------|-------------------|")
    print(pool_stats("P0 (All Held-Out)", heldout))
    print(pool_stats("P1 (C1 Non-Abelian)", c1_pass))
    print(pool_stats("P13 (C1 + C3)", [g for g in heldout if g.gate_c1 and g.gate_c3]))
    print(pool_stats("P123 (C1+C2+C3)", p123))
    print(pool_stats("S (Full Gauntlet)", survivors))

    print("\n" + "-"*80)
    print("SURVIVOR CENSUS (Held-Out Orders 129..256):")
    print("-"*80)
    for g in sorted(survivors, key=lambda x: (x.order, x.name)):
        print(f"  |G|={g.order:3d} | {g.name:32s} | family={g.family:18s} | RULE-B={rule_b[g.name]}")

    elapsed = time.time() - t0
    print(f"\nHeld-out sweep completed in {elapsed:.2f}s.")

if __name__ == "__main__":
    run_heldout_experiment()
