#!/usr/bin/env python3
import csv, hashlib, json, re, sys, difflib, collections
IN, OUT = sys.argv[1], sys.argv[2]
def facts(n): return list(csv.reader(open(f"{IN}/{n}_facts.tsv", encoding="utf-8"), delimiter="\t", quoting=csv.QUOTE_NONE))
def items(n): return [json.loads(l) for l in open(f"{OUT}/{n}_items.jsonl", encoding="utf-8")]
BAN = ["tries to", "attempts to", "attest", "authoriz", "supersede", "quorum", "standing"]
SOFT = ["try to", "legitimately", "without permission", "properly", "merely", "finally", "sneak", "withdraw", " role", "policy", "family", "window"]
for n in ("guard", "story", "policy"):
    F = facts(n); hdr, rows = F[0], F[1:]; L = items(n)
    cnt = collections.Counter((o["item"], o["phrasing"]) for o in L)
    ok = all(cnt[(r[0], p)] == 1 for r in rows for p in (1, 2)) and len(L) == 2 * len(rows)
    print(n, "rows", len(rows), "lines", len(L), "two-per-item:", ok)
    for b in BAN: print("   banned %-12r hits: %d" % (b, sum(b in o["text"].lower() for o in L)))
    for c in hdr:
        h = sum(1 for o in L if re.search(r"\b%s\b" % re.escape(c), o["text"]))
        if h: print("   column name %r appears as a word in %d texts" % (c, h))
    for b in SOFT:
        h = sorted({o["item"] for o in L if b in o["text"].lower()})
        if h: print("   soft word %r in %d items e.g. %s" % (b, len(h), h[:6]))
    for k in ("ill_posed", "source_leak_removed", "unsure"):
        print("   flagged %s: %d items" % (k, len({o["item"] for o in L if o[k]})), sorted({o["item"] for o in L if o[k]})[:40] if n != "policy" else "")
    print("   invented non-empty:", len({o["item"] for o in L if o["invented"]}))
    qs = collections.Counter((o["phrasing"], o["text"].split("\n")[-1]) for o in L); print("   final lines:", dict(qs))
    if n == "guard":
        by = {(o["item"], o["phrasing"]): o["text"] for o in L}
        bad = []; multi = []; spans = []
        for i in range(0, len(rows), 2):
            if rows[i][1] != rows[i+1][1]: print("   NOT A PAIR", rows[i][0], rows[i+1][0])
            for p in (1, 2):
                a, b = by[(rows[i][0], p)].split(), by[(rows[i+1][0], p)].split()
                pre = 0
                while pre < min(len(a), len(b)) and a[pre] == b[pre]: pre += 1
                suf = 0
                while suf < min(len(a), len(b)) - pre and a[-1-suf] == b[-1-suf]: suf += 1
                spans.append((len(a)-pre-suf, len(b)-pre-suf))
                ops = [x for x in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes() if x[0] != "equal"]
                if len(ops) != 1: multi.append((rows[i][0], p, len(ops)))
                if a == b: bad.append((rows[i][0], p))
        print("   pairs x phrasings:", len(spans), "identical-by-mistake:", bad)
        print("   differing span (words, member1/member2) max:", max(s[0] for s in spans), max(s[1] for s in spans))
        print("   pairs where a word-level diff finds MORE than one changed run inside that span:", len(multi), multi)
    if n == "policy":
        by = collections.defaultdict(list)
        for o in L: by[o["item"]].append(o)
        miss = 0
        for r in rows:
            nums = set(re.findall(r"\d+", " ".join(r[1:9])))
            for o in by[r[0]]:
                have = set(re.findall(r"\d+", o["text"]))
                if not nums <= have: miss += 1
                if r[7] not in o["text"] or r[1] not in o["text"]: miss += 1
                assert o["frame"] == int(hashlib.sha256(r[0].encode()).hexdigest()[:8], 16) % 6 + (1 if o["phrasing"] == 1 else 7)
        print("   renderings missing a number, the proposal text or the period:", miss)
        W = {"one day": 86400, "three days": 259200, "seven days": 604800}
        def pos(r):
            s = int(r[6]); w = W[r[7]]
            return "at 0 seconds" if s == 0 else "1 second before end" if s == w - 1 else "exactly at end" if s == w else "OTHER"
        fr = lambda r: int(hashlib.sha256(r[0].encode()).hexdigest()[:8], 16) % 6
        tab = collections.Counter((fr(r), pos(r)) for r in rows)
        P = ["at 0 seconds", "1 second before end", "exactly at end"]
        print("   rows per (frame pair, time position):")
        print("   | frames (p1/p2) | " + " | ".join(P) + " | total |")
        for k in range(6):
            print("   | %d / %d | " % (k + 1, k + 7) + " | ".join(str(tab[(k, x)]) for x in P) + " | %d |" % sum(tab[(k, x)] for x in P))
        print("   | total | " + " | ".join(str(sum(tab[(k, x)] for k in range(6))) for x in P) + " | %d |" % len(rows))
        print("   OTHER positions:", sum(v for (k, x), v in tab.items() if x == "OTHER"))
        tw = collections.Counter((fr(r), r[1]) for r in rows)
        print("   rows per (frame pair, proposal kind):", sorted(tw.items()))
        print("   unresolved braces:", sum("{" in o["text"] for o in L))
