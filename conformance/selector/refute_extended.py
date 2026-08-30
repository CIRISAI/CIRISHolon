#!/usr/bin/env python3
"""refute_extended.py — run the frozen SELECTOR-5 gauntlet and the frozen
RULE-B label on a population that includes the families the census omitted.

Streaming: each group is built, scored, and discarded, so peak memory stays
flat on a shared machine.
"""
import json, time, collections, gc
import numpy as np
import landscape_sweep as ls
import refute_lib as rl
import refute_population as rp

t0 = time.time()


def say(s):
    print(f"[{time.time()-t0:7.1f}s] {s}", flush=True)


def score(g):
    v, note = rl.rule_b_sm(g)
    return {"name": g.name, "family": g.family, "order": g.order,
            "c1": bool(g.gate_c1), "c2": bool(g.gate_c2), "c3": bool(g.gate_c3),
            "c4": bool(g.gate_c4), "full": bool(g.full_selector_pass),
            "lie": bool(g.is_lie_type), "ruleB": v,
            "zsize": len(g.center), "sig": repr(g.structural_signature())}


say("scoring the base census ...")
base_rows, sigs = [], set()
for i, g in enumerate(ls.generate_landscape()):
    r = score(g)
    sigs.add(r["sig"])
    base_rows.append(r)
    if i % 100 == 0:
        say(f"  base {i}")
say(f"base census scored: {len(base_rows)}")

say("building and scoring the extension ...")
ext_rows = []
gen = rp.build_extension()
say(f"extension constructed: {len(gen)} raw groups")
for i, g in enumerate(gen):
    r = score(g)
    if r["sig"] not in sigs:
        sigs.add(r["sig"])
        ext_rows.append(r)
    gen[i] = None
    if i % 100 == 0:
        gc.collect()
        say(f"  ext {i}/{len(gen)} (new so far {len(ext_rows)})")
del gen
say(f"extension scored: {len(ext_rows)} genuinely new isomorphism types")

rows = base_rows + ext_rows
json.dump(rows, open("refute_extended.json", "w"))

c = collections.Counter(r["order"] for r in rows)
say("coverage at the 2-group orders: " +
    ", ".join(f"|G|={o}:{c[o]}" for o in (8, 16, 32, 64, 96, 128)))

S = [r for r in rows if r["full"]]
P123 = [r for r in rows if r["c1"] and r["c2"] and r["c3"]]
twogp = [r for r in rows if r["order"] in (32, 64, 128)]


def rate(pool):
    n = len(pool)
    a = sum(1 for r in pool if r["lie"])
    b = sum(1 for r in pool if r["ruleB"])
    return n, a, a / n, b, b / n


print("\n| pool | n | RULE-A SM | rate | RULE-B SM | rate |")
print("|---|---|---|---|---|---|")
for nm, pool in (("P0 all (extended)", rows), ("P123", P123),
                 ("S survivors", S), ("2-groups |G| in {32,64,128}", twogp),
                 ("base census only", base_rows), ("extension only", ext_rows)):
    if not pool:
        continue
    n, a, ra, b, rb = rate(pool)
    print(f"| {nm} | {n} | {a} | {ra:.4f} | {b} | {rb:.4f} |")

orig = {r["name"] for r in base_rows}
print("\nNEW survivors (groups the original census did not contain):")
nnew = 0
for r in sorted(S, key=lambda x: (x["order"], x["name"])):
    if r["name"] not in orig:
        nnew += 1
        print(f"  |G|={r['order']:4d} {r['name']:32s} family={r['family']:18s} "
              f"RULE-A={r['lie']} RULE-B={r['ruleB']}  |Z|={r['zsize']}")
print(f"  ({nnew} new survivors)")

print("\nC4 provenance across the whole extended population:")
prov = collections.Counter()
for r in rows:
    if not r["c4"]:
        continue
    if r["family"] in ("BinaryPolyhedral", "Dicyclic"):
        prov["family-string override"] += 1
    elif r["name"] in ("2T", "2O", "2I", "SL(2,3)", "SL(2,5)", "GL(2,3)"):
        prov["name override"] += 1
    elif r["zsize"] >= 2 and (r["order"] // 2) in (12, 24, 60):
        prov["magic-order rule"] += 1
    else:
        prov["hard-coded Schur lookup keyed on family"] += 1
for k, v in prov.most_common():
    print(f"  {k:42s} {v}")

off = [r for r in rows
       if r["family"] in ("Metacyclic", "AffineSemidirect", "Extraspecial")]
offc4 = [r for r in off if r["c4"]]
print(f"\ngroups outside the eleven hard-coded families: {len(off)}")
print(f"  C4 fires on {len(offc4)}; of those, "
      f"{sum(1 for r in offc4 if r['zsize'] >= 2 and (r['order']//2) in (12,24,60))} "
      f"fire through the magic-order rule "
      f"(compute_schur_multiplier returns the empty tuple for every unlisted family).")

open("refute_extended.DONE", "w").write("done\n")
say("DONE")
