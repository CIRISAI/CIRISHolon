#!/usr/bin/env python3
"""refute_c4.py — what C4 actually computes, and the survivor-set diff."""
import re, math
import landscape_sweep as ls

groups = ls.generate_landscape()
by = {g.name: g for g in groups}
S = [g for g in groups if g.full_selector_pass]

# --- survivor set diff against SELECTOR5_RESULTS.md section 5
md = open("SELECTOR5_RESULTS.md").read()
sec5 = md.split("## 5. Complete Inventory")[1].split("---")[0]
claimed = []
for line in sec5.splitlines():
    m = re.match(r"\|\s*(\d+)\s*\|\s*\$?([^|]+?)\$?\s*\|", line)
    if m and m.group(1).isdigit():
        raw = m.group(2).strip()
        raw = raw.replace("\\text{", "").replace("}", "").replace("{", "")
        raw = raw.replace("\\times", "x").replace("_", "_").replace("$", "").replace("\\", "")
        raw = raw.replace(" ", "")
        claimed.append((int(m.group(1)), raw))
print(f"claimed survivors parsed: {len(claimed)}")
actual = sorted((g.order, g.name) for g in S)
def norm(s):
    return s.replace("Dic", "Dic").replace("Delta", "Delta").replace("Q_8", "Q_8")
cl = sorted(claimed)
print("\norder | claimed (RESULTS.md)      | actual (script)")
for i in range(max(len(cl), len(actual))):
    a = f"{cl[i][0]:5d} {cl[i][1]:24s}" if i < len(cl) else " " * 30
    b = f"{actual[i][0]:5d} {actual[i][1]}" if i < len(actual) else ""
    mark = "" if (i < len(cl) and i < len(actual) and cl[i][0] == actual[i][0]) else "  <-- order mismatch"
    print(f"{a} | {b}{mark}")

# --- C4 dissection
print("\n\n=== C4 dissection ===")
fams = {}
for g in groups:
    fams.setdefault(g.family, []).append(g)
for fam in sorted(fams):
    gs = fams[fam]
    n4 = sum(1 for g in gs if g.gate_c4)
    schur = sum(1 for g in gs if len(g.schur_multiplier) > 0 and math.prod(g.schur_multiplier) > 1)
    spin = sum(1 for g in gs if g.is_spin_cover)
    print(f"{fam:18s} n={len(gs):4d}  C4={n4:4d}  via nontrivial-Schur={schur:4d}  via spin-cover={spin:4d}")

dp = fams["DirectProduct"]
pred = [g for g in dp if g.order in (24, 48, 120) and len(g.center) >= 2]
c4dp = [g for g in dp if g.gate_c4]
print(f"\nDirectProduct: C4 passers = {len(c4dp)}; "
      f"groups with |G| in (24,48,120) and |Z|>=2 = {len(pred)}; "
      f"sets identical = {set(g.name for g in c4dp) == set(g.name for g in pred)}")
print("orders of DirectProduct C4 passers:", sorted(set(g.order for g in c4dp)))

dic = fams["Dicyclic"]
print(f"\nDicyclic: {len(dic)} groups, C4 passers = {sum(1 for g in dic if g.gate_c4)}, "
      f"all with computed Schur multiplier = {set(g.schur_multiplier for g in dic)}")

print("\n=== generalized quaternion Q_{2^k} = Dic_{2^{k-2}} ===")
for n in (2, 4, 8, 16, 32):
    g = by.get(f"Dic_{n}")
    if g:
        print(f"  Q_{4*n:4d} = Dic_{n:2d}: C1={g.gate_c1} C2={g.gate_c2} C3={g.gate_c3} "
              f"C4={g.gate_c4} orient={g.orientation_index:.3f}  -> {'PASS' if g.full_selector_pass else 'KILLED'}")

print("\n=== extraspecial / Heisenberg present in population ===")
for nm in ("D_8", "Q_8", "UT(3,3)", "Delta(27)", "UT(3,5)"):
    g = by.get(nm)
    if g:
        print(f"  {nm:10s} |G|={g.order:4d} aliases={g.aliases} C1={g.gate_c1} C3={g.gate_c3} "
              f"C4={g.gate_c4} -> {'PASS' if g.full_selector_pass else 'KILLED'}")
