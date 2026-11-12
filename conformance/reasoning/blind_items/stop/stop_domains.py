#!/usr/bin/env python3
"""Write stop_scenario_domains.tsv: one row per scenario with its split, pair, context key, setting and domain.

Standard library only. Reads stop_vocab.json. The context-to-domain table below is stated by the writer;
no domain is inferred from words. Exits non-zero if any check fails. Does not touch the item files:
it reads only their `scenario` and `split` fields and their bytes for the checksum comparison.

Run:  python3 stop_domains.py
"""
import collections, hashlib, json, os, sys

HERE = os.path.dirname(os.path.abspath(__file__))
SPLITS = ["fit", "explore", "confirm", "reserve"]
ITEM_FILES = [f"stop_items_{s}.jsonl" for s in SPLITS]

# (split, context key) -> domain. The names are those of meta.domains_per_split in stop_vocab.json.
TABLE = {
    "fit": {
        "hospitals": ["h1", "h2", "h3", "h4", "h5", "h6", "h7"],
        "parcel and warehouse logistics": ["l1", "l2", "l3", "l4", "l5", "l6"],
        "schools": ["s1", "s2", "s3", "s4", "s5", "s6"],
        "city services": ["c1", "c2", "c3", "c4", "c5", "c6"],
    },
    "explore": {
        "farming": ["f1", "f2", "f3", "f4", "f5"],
        "libraries": ["b1", "b2", "b3", "b4", "b5"],
    },
    "confirm": {
        "shipping and ports": ["p1", "p2", "p3", "p4", "p5"],
        "laboratories": ["a1", "a2", "a3", "a4", "a5"],
        "housing": ["t1", "t2", "t3", "t4", "t5"],
        "sports clubs": ["g1", "g2", "g3", "g4", "g5"],
    },
    "reserve": {
        "mountain and cave rescue": ["r1", "r2", "r3", "r4", "r5"],
        "mines and quarries": ["r6", "r7", "r8", "r9", "r10"],
        "relief camps": ["r11", "r12", "r13", "r14", "r15"],
        "forestry": ["r16", "r17", "r18", "r19", "r20"],
        "museums": ["r21", "r22", "r23", "r24", "r25"],
        "theatre": ["r26", "r27", "r28", "r29", "r30"],
    },
}

fails = []


def check(name, ok, detail=""):
    print(("PASS  " if ok else "FAIL  ") + name + (("  " + detail) if detail else ""))
    if not ok:
        fails.append(name)


vocab = json.load(open(os.path.join(HERE, "stop_vocab.json"), encoding="utf-8"))
declared = vocab["meta"]["domains_per_split"]

# the table itself: each context in exactly one domain; domain names are the declared ones
ctx_dom = {}
dup = []
for split, doms in TABLE.items():
    for dom, keys in doms.items():
        for k in keys:
            if (split, k) in ctx_dom:
                dup.append((split, k))
            ctx_dom[(split, k)] = dom
check("each context maps to exactly one domain", not dup, str(dup))
check("the table's domain names are exactly the declared ones, per split",
      all(sorted(TABLE[s]) == sorted(declared[s]) for s in SPLITS) and sorted(TABLE) == sorted(declared))

rows = []
missing = []
for pair in sorted(vocab["pairs"]):
    for sc in vocab["pairs"][pair]["scenarios"]:
        key = (sc["split"], sc["context"])
        if key not in ctx_dom:
            missing.append((sc["id"], key))
            continue
        rows.append((sc["id"], sc["split"], pair, sc["context"], sc["slots"]["setting"], ctx_dom[key]))
check("every scenario's context is in the table", not missing, str(missing[:10]))
used = {(r[1], r[3]) for r in rows}
check("every context in the table is used by a scenario", used == set(ctx_dom), str(sorted(set(ctx_dom) - used)))
check("one row per scenario, 335 in all", len(rows) == 335 and len({r[0] for r in rows}) == 335, str(len(rows)))
check("each domain is one of its split's declared domains", all(r[5] in declared[r[1]] for r in rows))
sett = collections.defaultdict(set)
for r in rows:
    sett[(r[1], r[3])].add(r[4])
check("a context has one setting text", all(len(v) == 1 for v in sett.values()))
check("no tab or newline inside a field", all("\t" not in f and "\n" not in f for r in rows for f in r))

# against the items: scenario ids and splits
item_split = {}
for name in ITEM_FILES:
    for line in open(os.path.join(HERE, name), encoding="utf-8"):
        it = json.loads(line)
        item_split.setdefault(it["scenario"], set()).add(it["split"])
check("the scenarios of the table are exactly the scenarios of the items", set(item_split) == {r[0] for r in rows})
check("each scenario's split in the table equals its split in the items",
      all(item_split.get(r[0]) == {r[1]} for r in rows))

# the four item files are byte-for-byte those of SHA256SUMS
sums = {}
for line in open(os.path.join(HERE, "SHA256SUMS")):
    hx, name = line.split()
    sums[name] = hx
bad = [n for n in ITEM_FILES if hashlib.sha256(open(os.path.join(HERE, n), "rb").read()).hexdigest() != sums.get(n)]
check("the four item files' sha256 are those of SHA256SUMS", not bad, str(bad))
for n in ITEM_FILES:
    print("     ", sums.get(n), n)

if fails:
    print("FAILED:", len(fails), "- nothing written")
    sys.exit(1)

with open(os.path.join(HERE, "stop_scenario_domains.tsv"), "w", encoding="utf-8", newline="") as f:
    f.write("scenario\tsplit\tpair\tcontext\tsetting\tdomain\n")
    for r in rows:
        f.write("\t".join(r) + "\n")

print()
print("rows written:", len(rows))
cnt = collections.Counter((r[1], r[5]) for r in rows)
nctx = collections.Counter((s, d) for (s, k), d in ctx_dom.items())
print("%-8s %-32s %9s %9s" % ("split", "domain", "contexts", "scenarios"))
for s in SPLITS:
    for d in TABLE[s]:
        print("%-8s %-32s %9d %9d" % (s, d, nctx[(s, d)], cnt[(s, d)]))
    print("%-8s %-32s %9s %9d" % (s, "(all)", "", sum(cnt[(s, d)] for d in TABLE[s])))
print("ALL CHECKS PASS")
