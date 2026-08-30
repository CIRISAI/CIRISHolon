#!/usr/bin/env python3
"""heldout.py — SELECTOR-6 H1: the held-out census, run, and scoring.

Predictions P1..P5 are committed in HELDOUT_PREDICTIONS.md (commit 2b6d94a),
BEFORE this file built a single held-out table.  Nothing here is revised after
a held-out number is seen.

Population, per that document:
  (a) every isomorphism type of orders 65..71, gated against A000001;
  (b) the six-group 2-swamp panel: ES32+, ES32-, Q64, Q128, ES128+, ES128-,
      where the order-128 extraspecials substitute for the order-64 pair the
      commission named, which does not exist (extraspecial 2-groups have order
      2^(1+2n) = 8, 32, 128).
"""
import itertools
import json
import os
import sys
import time
import collections

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import s4core as S4
import census as C
import selector6 as S6
import ruleb as RL

SEED = 20260830
DRAWS = 100_000
t0 = time.time()
RNG = np.random.default_rng(SEED)


def say(s):
    print(f"[{time.time()-t0:8.1f}s] {s}", flush=True)


# ------------------------------------------------------------------- the panel

def central_product_z2(G, H):
    """(G x H) / <(z_G, z_H)>, for G, H with a unique central involution."""
    def unique_involution(K):
        z = [g for g in K.center if g != K.IDE and K.orders[g] == 2]
        assert len(z) == 1, f"expected a unique central involution, got {len(z)}"
        return z[0]
    zg, zh = unique_involution(G), unique_involution(H)
    n, m = G.n, H.n
    pair = lambda a, b: a * m + b
    ident = {}
    reps = []
    for a in range(n):
        for b in range(m):
            k = pair(a, b)
            if k in ident:
                continue
            k2 = pair(int(G.MUL[a, zg]), int(H.MUL[b, zh]))
            r = len(reps)
            ident[k] = r
            ident[k2] = r
            reps.append((a, b))
    N = len(reps)
    T = np.zeros((N, N), dtype=np.int16)
    for i, (a1, b1) in enumerate(reps):
        for j, (a2, b2) in enumerate(reps):
            T[i, j] = ident[pair(int(G.MUL[a1, a2]), int(H.MUL[b1, b2]))]
    return C.Grp(T)


def dihedral8():
    n = 4
    M = np.zeros((8, 8), dtype=np.int16)
    for i1 in range(2):
        for j1 in range(n):
            for i2 in range(2):
                for j2 in range(n):
                    M[i1 * n + j1, i2 * n + j2] = ((i1 + i2) % 2) * n + \
                        ((j1 * (-1) ** i2 + j2) % n)
    return C.Grp(M)


def dicyclic(k):
    """Dic_k of order 4k; <a,b | a^2k=1, b^2=a^k, bab^-1=a^-1>"""
    N = 4 * k
    m = 2 * k
    M = np.zeros((N, N), dtype=np.int16)
    for i1 in range(2):
        for j1 in range(m):
            for i2 in range(2):
                for j2 in range(m):
                    if i2 == 0:
                        j3, i3 = (j1 + j2) % m, i1
                    else:
                        j3, i3 = (j2 - j1) % m, (i1 + 1) % 2
                    if i1 == 1 and i2 == 1:
                        j3 = (j3 + k) % m
                    M[i1 * m + j1, i2 * m + j2] = i3 * m + j3
    return C.Grp(M)


def is_extraspecial(G):
    Z = set(G.center)
    if len(Z) != 2:
        return False
    if set(G.derived()) != Z:
        return False
    p = 2
    for g in range(G.n):
        if int(G.MUL[g, g]) not in Z:
            return False
    return True


def build_panel():
    D8, Q8 = dihedral8(), dicyclic(2)
    es32p = central_product_z2(D8, D8)
    es32m = central_product_z2(D8, Q8)
    es128p = central_product_z2(es32p, D8)
    es128m = central_product_z2(es32m, D8)
    panel = [("ES32+", es32p), ("ES32-", es32m),
             ("Q64", dicyclic(16)), ("Q128", dicyclic(32)),
             ("ES128+", es128p), ("ES128-", es128m)]
    say("panel verification (each group must be what it is named):")
    for nm, G in panel:
        if nm.startswith("ES"):
            ok = is_extraspecial(G)
            invol = sum(1 for g in range(G.n) if G.orders[g] == 2)
            say(f"  {nm:8s} |G|={G.n:4d}  extraspecial={ok}  |Z|={len(G.center)}  "
                f"involutions={invol}")
            assert ok, f"{nm} is not extraspecial"
        else:
            invol = [g for g in range(G.n) if G.orders[g] == 2]
            say(f"  {nm:8s} |G|={G.n:4d}  involutions={len(invol)} "
                f"(generalized quaternion needs exactly 1)")
            assert len(invol) == 1, f"{nm} has {len(invol)} involutions"
        assert G.check_associative(), f"{nm} table is not associative"
    # the two types must be non-isomorphic
    for a, b in (("ES32+", "ES32-"), ("ES128+", "ES128-")):
        Ga = dict(panel)[a]
        Gb = dict(panel)[b]
        iso = C.isomorphism(Ga, Gb)
        say(f"  {a} vs {b}: isomorphic={iso is not None} (must be False)")
        assert iso is None, f"{a} and {b} came out isomorphic"
    return panel


# ------------------------------------------------------------- held-out census

def heldout_census(lo=65, hi=71):
    pin = C.load_pin()
    groups = {}
    # rebuild the primary census's lower orders as extension inputs (cheap: the
    # cyclic-extension constructor needs every group of order n/p)
    say("rebuilding orders 1..%d as extension inputs" % (hi // 2))
    with open(os.path.join(HERE, "census.json")) as f:
        prev = json.load(f)["tables"]
    for k, v in prev.items():
        groups[int(k)] = [C.Grp(np.asarray(t)) for t in v]
    counts, voids = {}, []
    for n in range(lo, hi + 1):
        found, fps = [], {}
        ncand = 0
        for p in sorted(C.factorize(n)):
            m = n // p
            for N in groups.get(m, []):
                for T in C.cyclic_extensions(N, p):
                    ncand += 1
                    try:
                        G = C.Grp(T)
                    except ValueError:
                        continue
                    fp = G.fingerprint
                    b = fps.setdefault(fp, [])
                    if any(C.isomorphism(G, H) is not None for H in b):
                        continue
                    b.append(G)
                    found.append(G)
        for G in found:
            assert G.check_associative(), f"non-associative at order {n}"
        groups[n] = found
        counts[n] = len(found)
        want = pin[n]
        flag = "" if len(found) == want else f"   <-- S1 VOID (pin {want})"
        if len(found) != want:
            voids.append(n)
        say(f"order {n:3d}: {len(found):3d} types from {ncand:5d} candidates"
            f"  (pin {want}){flag}")
    return groups, counts, voids


# -------------------------------------------------------------------- scoring

def score_group(gid, G):
    rec = dict(gid=gid, order=G.n, abelian=bool(G.is_abelian()))
    try:
        W = S4.build_world(gid, np.asarray(G.MUL, dtype=np.int64),
                           np.asarray(G.INV, dtype=np.int64))
        W["MUL"] = np.asarray(G.MUL, dtype=np.int64)
        W["INV"] = np.asarray(G.INV, dtype=np.int64)
        members, perms, orbit_of, reps, sizes, powers = S6.build_family(W)
        per_rung, calls, audits = S6.run_group(
            W, members, perms, orbit_of, reps, sizes, powers,
            deadline=time.time() + S6.GROUP_WALL)
        sel, kstar, selc = S6.verdict(per_rung, len(members))
        bad = [(a, b) for (a, b) in audits if a != b]
        rec.update(select=sel, kstar=kstar, sel=selc, F=len(members),
                   calls=calls, void="cheap-path disagreed" if bad else None)
        if bad:
            rec["select"] = None
    except S6.Refused as e:
        rec.update(select=None, kstar=None, sel=None, F=None, calls=None,
                   void=str(e))
    except Exception as e:
        rec.update(select=None, kstar=None, sel=None, F=None, calls=None,
                   void=f"{type(e).__name__}: {e}")
    ad = S6.LabelAdapter(G.MUL, G.INV, G.IDE, G.n)
    try:
        rec["sm"], rec["sm_note"] = RL.rule_b_sm(ad)
    except Exception as e:
        rec["sm"], rec["sm_note"] = None, f"{type(e).__name__}: {e}"
    return rec


def main():
    say("H1 — held-out run. Predictions committed at 2b6d94a, before this built anything.")
    panel = build_panel()
    say("")
    groups, counts, s1voids = heldout_census()
    say("")
    say(f"S1 on held-out orders 65..71: "
        f"{'PASS, zero VOIDs' if not s1voids else 'VOID at ' + str(s1voids)}")
    say(f"held-out complete-order groups: {sum(counts.values())}")

    rows = []
    for n in sorted(counts):
        for i, G in enumerate(groups[n]):
            rows.append(score_group(f"h{n}#{i}", G))
            say(f"  {rows[-1]['gid']:10s} |G|={rows[-1]['order']:4d} "
                f"abelian={rows[-1]['abelian']!s:5s} select={rows[-1]['select']!s:5s} "
                f"sel={rows[-1]['sel']} F={rows[-1]['F']} sm={rows[-1]['sm']}")
    say("")
    say("panel:")
    for nm, G in panel:
        r = score_group(nm, G)
        r["panel"] = True
        rows.append(r)
        say(f"  {nm:10s} |G|={r['order']:4d} abelian={r['abelian']!s:5s} "
            f"select={r['select']!s:5s} sel={r['sel']} F={r['F']} sm={r['sm']} "
            f"void={r['void']}")

    json.dump(rows, open(os.path.join(HERE, "heldout.json"), "w"))

    # ------------------------------------------------------------- P1..P5
    say("")
    say("=" * 70)
    say("H1 SCORING — against HELDOUT_PREDICTIONS.md, unrevised")
    say("=" * 70)
    decided = [r for r in rows if r["select"] is not None]
    dis = [r for r in decided if r["select"] != (not r["abelian"])]
    say(f"P1 (SELECT == non-abelian, zero exceptions): "
        f"{len(decided)} decided, {len(dis)} disagreements "
        f"-> {'CONFIRMED' if not dis else 'FALSIFIED'}")
    for r in dis:
        say(f"    DISAGREEMENT {r['gid']}: abelian={r['abelian']} "
            f"select={r['select']} sel={r['sel']} F={r['F']}")

    pool = [r for r in decided if r["sm"] is not None]
    S = [r for r in pool if r["select"]]
    if S and len(pool) > len(S):
        labels = np.array([1.0 if r["sm"] else 0.0 for r in pool])
        obs = float(np.mean([1.0 if r["sm"] else 0.0 for r in S]))
        idx = np.argsort(RNG.random((DRAWS, len(pool))), axis=1)[:, :len(S)]
        null = labels[idx].mean(axis=1)
        p99 = float(np.percentile(null, 99))
        say(f"P2 (aggregate null extends): selected {len(S)}/{len(pool)}, "
            f"RULE-B among selected {obs:.4f}, base {labels.mean():.4f}, "
            f"null 99th {p99:.4f} -> "
            f"{'CONFIRMED' if obs <= p99 else 'FALSIFIED'}")
        say(f"P3 (miss stays downward): observed {obs:.4f} vs base "
            f"{labels.mean():.4f} -> "
            f"{'CONFIRMED' if obs <= labels.mean() else 'FALSIFIED'}")
    else:
        say("P2/P3: pool degenerate (all selected or none) — reported, not scored")

    say("P4 (panel labels):")
    want = {"ES32+": False, "ES32-": False, "ES128+": False, "ES128-": False,
            "Q64": True, "Q128": True}
    p4 = True
    for nm, exp in want.items():
        got = next((r["sm"] for r in rows if r["gid"] == nm), None)
        ok = (got == exp)
        p4 &= ok
        say(f"    {nm:8s} predicted RULE-B={exp!s:5s} got {got!s:5s} "
            f"-> {'ok' if ok else 'MISS'}")
    say(f"P4 -> {'CONFIRMED' if p4 else 'FALSIFIED'}")

    v_complete = [r for r in rows if not r.get("panel") and r["select"] is None]
    v_panel = [r for r in rows if r.get("panel") and r["select"] is None]
    say(f"P5 (budgets): VOIDs at orders 65..71 = {len(v_complete)} "
        f"(predicted 0) -> {'CONFIRMED' if not v_complete else 'FALSIFIED'}")
    say(f"    panel VOIDs = {len(v_panel)}"
        + (f" ({', '.join(r['gid'] for r in v_panel)}) — anticipated for the "
           f"order-128 members, not a failure" if v_panel else ""))
    open(os.path.join(HERE, "heldout.DONE"), "w").write("done\n")
    say("H1 COMPLETE")


if __name__ == "__main__":
    main()
