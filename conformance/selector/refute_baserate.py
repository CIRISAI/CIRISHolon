#!/usr/bin/env python3
"""refute_baserate.py — the base-rate / tautology test for SELECTOR-5."""
import json, sys, numpy as np
import landscape_sweep as ls
import refute_lib as rl

def pools(groups):
    P0 = groups
    P1 = [g for g in groups if g.gate_c1]
    P13 = [g for g in groups if g.gate_c1 and g.gate_c3]
    P123 = [g for g in groups if g.gate_c1 and g.gate_c2 and g.gate_c3]
    S = [g for g in groups if g.full_selector_pass]
    return {"P0 (all)": P0, "P1 (post-C1)": P1, "P13 (post-C1+C3)": P13,
            "P123 (pool C4 acts on)": P123, "S (survivors)": S}

def main():
    groups = ls.generate_landscape()
    print(f"population: {len(groups)} groups", flush=True)
    labB, notes = {}, {}
    for i, g in enumerate(groups):
        v, note = rl.rule_b_sm(g)
        labB[g.name] = v
        notes[g.name] = note
        if v is None:
            print(f"  !! RULE-B undecided for {g.name}: {note}", flush=True)
    print("RULE-B computed for all groups", flush=True)

    P = pools(groups)
    rows = []
    for pname, pool in P.items():
        a = sum(1 for g in pool if g.is_lie_type)
        b = sum(1 for g in pool if labB[g.name])
        rows.append((pname, len(pool), a, a/len(pool), b, b/len(pool)))
    print("\n| pool | n | RULE-A SM | rate | RULE-B SM | rate |")
    print("|---|---|---|---|---|---|")
    for r in rows:
        print(f"| {r[0]} | {r[1]} | {r[2]} | {r[3]:.4f} | {r[4]} | {r[5]:.4f} |")

    # disagreements between the published label and the honest one
    da = [g.name for g in groups if g.is_lie_type and not labB[g.name]]
    db = [g.name for g in groups if (not g.is_lie_type) and labB[g.name]]
    print(f"\nRULE-A says SM, RULE-B says NOT ({len(da)}): {sorted(da)[:40]}")
    print(f"RULE-A says NOT, RULE-B says SM ({len(db)}): {sorted(db)[:40]}")

    # permutation control on P123
    rng = np.random.default_rng(20260830)
    pool = P["P123 (pool C4 acts on)"]
    S = P["S (survivors)"]
    k = len(S)
    for rule, lab in (("RULE-A", lambda g: g.is_lie_type), ("RULE-B", lambda g: labB[g.name])):
        vals = np.array([1.0 if lab(g) else 0.0 for g in pool])
        obs = float(np.mean([1.0 if lab(g) else 0.0 for g in S]))
        draws = np.array([vals[rng.choice(len(vals), size=k, replace=False)].mean()
                          for _ in range(200000)])
        p = float((draws >= obs - 1e-12).mean())
        print(f"\n{rule}: observed precision on S = {obs:.4f} ({sum(1 for g in S if lab(g))}/{k})")
        print(f"  base rate in P123 = {vals.mean():.4f}; permutation mean = {draws.mean():.4f}, "
              f"sd = {draws.std():.4f}, max = {draws.max():.4f}")
        print(f"  p(random 45-subset of P123 reaches this precision) = {p:.6g}")

    json.dump({"ruleB": {k: v for k, v in labB.items()}}, open("refute_labels.json", "w"))

main()
