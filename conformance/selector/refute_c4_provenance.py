#!/usr/bin/env python3
"""refute_c4_provenance.py — for every survivor, WHY did C4 fire?"""
import math
import landscape_sweep as ls

groups = ls.generate_landscape()
S = [g for g in groups if g.full_selector_pass]
P13 = [g for g in groups if g.gate_c1 and g.gate_c2 and g.gate_c3]

def why_c4(g):
    reasons = []
    sm = g.schur_multiplier
    if len(sm) > 0 and math.prod(sm) > 1:
        reasons.append(f"hard-coded Schur lookup for family '{g.family}' -> {sm}")
    if g.family in ("BinaryPolyhedral", "Dicyclic"):
        reasons.append(f"family string '{g.family}' in the spin-cover override list")
    if g.name in ("2T", "2O", "2I", "SL(2,3)", "SL(2,5)", "GL(2,3)"):
        reasons.append(f"group NAME '{g.name}' in the spin-cover override list")
    if len(g.center) >= 2 and (g.order // 2) in (12, 24, 60):
        reasons.append(f"|Z|={len(g.center)}>=2 and |G|/2={g.order//2} in the magic order list (12,24,60)")
    return reasons

buckets = {"family-string override": 0, "name override": 0,
           "magic-order rule": 0, "hard-coded Schur lookup": 0}
print("| survivor | why C4 fired |")
print("|---|---|")
for g in sorted(S, key=lambda x: (x.order, x.name)):
    r = why_c4(g)
    tags = []
    if any("family string" in x for x in r):
        tags.append("family-string override")
    if any("NAME" in x for x in r):
        tags.append("name override")
    if any("magic order" in x for x in r):
        tags.append("magic-order rule")
    if any("Schur lookup" in x for x in r):
        tags.append("hard-coded Schur lookup")
    for t in tags:
        buckets[t] += 1
    print(f"| {g.name} | {'; '.join(r)} |")

print("\ncounts over the 45 survivors (a survivor may fire on more than one):")
for k, v in buckets.items():
    print(f"  {k:28s} {v}")

pure_computed = [g for g in S if not why_c4(g)]
print(f"\nsurvivors whose C4 came from a property computed from the group "
      f"rather than a string or an order lookup: {len(pure_computed)}")

# what does C4 do on P13?
print(f"\nP13 = {len(P13)}, of which C4 passes {sum(1 for g in P13 if g.gate_c4)}")
kill = [g for g in P13 if not g.gate_c4]
import collections
print("families C4 killed inside P13:", dict(collections.Counter(g.family for g in kill)))
print("families C4 kept  inside P13:",
      dict(collections.Counter(g.family for g in P13 if g.gate_c4)))

# the counterfactual: what if C4 were the TRUE Schur multiplier?
# For dicyclic groups the code's own compute_schur_multiplier returns () = trivial.
dic_surv = [g for g in S if g.family == "Dicyclic"]
print(f"\nsurvivors in the Dicyclic family: {len(dic_surv)} "
      f"({len(dic_surv)/len(S):.1%} of all survivors)")
print("  their computed Schur multipliers:", set(g.schur_multiplier for g in dic_surv))
print("  i.e. every one of them has TRIVIAL projective obstruction, and passes the")
print("  'projective obstruction' gate solely because the string 'Dicyclic' is in a list.")
