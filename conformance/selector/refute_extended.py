#!/usr/bin/env python3
"""refute_extended.py — run the frozen SELECTOR-5 gauntlet and the frozen
RULE-B label on a population that includes the families the census omitted."""
import pickle, time, collections
import numpy as np
import landscape_sweep as ls
import refute_lib as rl
import refute_population as rp

t0 = time.time()


def say(s):
    print(f"[{time.time()-t0:7.1f}s] {s}", flush=True)


say("building extension ...")
ext = rp.build_extension()
say(f"extension built: {len(ext)} raw groups")

base = ls.generate_landscape()
say(f"base census: {len(base)}")

seen, merged = {}, []
for g in base:
    sig = g.structural_signature()
    if sig not in seen:
        seen[sig] = g
        merged.append(g)
added = 0
for i, g in enumerate(ext):
    sig = g.structural_signature()
    if sig not in seen:
        seen[sig] = g
        merged.append(g)
        added += 1
    if i % 200 == 0:
        say(f"  merging {i}/{len(ext)} (new so far {added})")
merged.sort(key=lambda g: (g.order, g.name))
say(f"merged population: {len(merged)} ({added} new isomorphism types)")

c = collections.Counter(g.order for g in merged)
say("coverage at the 2-group orders: " +
    ", ".join(f"|G|={o}:{c[o]}" for o in (8, 16, 32, 64, 96, 128)))

say("running the frozen gauntlet ...")
S = [g for g in merged if g.full_selector_pass]
P123 = [g for g in merged if g.gate_c1 and g.gate_c2 and g.gate_c3]
say(f"C1+C2+C3 pool: {len(P123)}; full-gauntlet survivors: {len(S)}")

say("computing RULE-B ...")
labB = {}
for i, g in enumerate(merged):
    v, note = rl.rule_b_sm(g)
    labB[g.name] = v
    if i % 200 == 0:
        say(f"  RULE-B {i}/{len(merged)}")
say("RULE-B done")


def rate(pool):
    n = len(pool)
    a = sum(1 for g in pool if g.is_lie_type)
    b = sum(1 for g in pool if labB[g.name])
    return n, a, a / n, b, b / n


print("\n| pool | n | RULE-A SM | rate | RULE-B SM | rate |")
print("|---|---|---|---|---|---|")
for nm, pool in (("P0 all", merged), ("P123", P123), ("S survivors", S)):
    n, a, ra, b, rb = rate(pool)
    print(f"| {nm} | {n} | {a} | {ra:.4f} | {b} | {rb:.4f} |")

print("\nNEW survivors (not in the original 430):")
orig = {g.name for g in base}
nnew = 0
for g in sorted(S, key=lambda x: (x.order, x.name)):
    if g.name not in orig:
        nnew += 1
        print(f"  |G|={g.order:4d} {g.name:34s} family={g.family:18s} "
              f"RULE-B={labB[g.name]}  |Z|={len(g.center)}")
print(f"  ({nnew} new survivors)")

print("\nC4 provenance across the whole merged population:")
prov = collections.Counter()
for g in merged:
    if not g.gate_c4:
        continue
    if g.family in ("BinaryPolyhedral", "Dicyclic"):
        prov["family-string override"] += 1
    elif g.name in ("2T", "2O", "2I", "SL(2,3)", "SL(2,5)", "GL(2,3)"):
        prov["name override"] += 1
    elif len(g.schur_multiplier) > 0 and np.prod(g.schur_multiplier) > 1:
        prov["hard-coded Schur lookup"] += 1
    elif len(g.center) >= 2 and (g.order // 2) in (12, 24, 60):
        prov["magic-order rule"] += 1
    else:
        prov["computed from the group"] += 1
for k, v in prov.most_common():
    print(f"  {k:28s} {v}")

offlist = [g for g in merged
           if g.family in ("Metacyclic", "AffineSemidirect", "Extraspecial")]
print(f"\ngroups outside the eleven hard-coded families: {len(offlist)}")
print(f"  C4 fires on {sum(1 for g in offlist if g.gate_c4)} of them — all through "
      f"the magic-order rule, since compute_schur_multiplier returns the empty "
      f"tuple for every unlisted family.")

pickle.dump({"names": [g.name for g in merged], "labB": labB,
             "survivors": [g.name for g in S]},
            open("refute_extended.pkl", "wb"))
open("refute_extended.DONE", "w").write("done\n")
say("DONE")
