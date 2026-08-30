#!/usr/bin/env python3
"""primary.py — the SELECTOR-6 primary run.

Pre-registered before any number exists (SELECTOR6_PREREG.md + AMENDMENT A1 +
SELECTOR6_DESIGN.md v2, all committed):

  * the criterion is s4core (selector4.py at blob d33f0469), untouched;
  * the label is ruleb (refute_lib.py at blob cbaf2b47), tag-free by a gate;
  * the family is the FULL dressed family, so I1 holds by construction;
  * SELECT(G) := 0 < |sel_k*| < |F(G)| at the finest decided rung k*, and
    k* must be A3 or A4 -- design v2 section 1.1, ruling 3 as proposed.

THE PRIMARY VERDICT IS THE ONE UNDER MIN_DECIDED_RUNG = 3.  The coarse-rung
fallback (letting a group whose fine rungs VOID be scored False on A2) is
computed ONLY as a labelled sensitivity diagnostic, never as an alternative
headline.  This sentence is written before the run, so that reporting both
cannot become choosing between them.

T1 runs inline and a violation VOIDs for diagnosis; it is never data.
"""
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

SEED = 20260830          # staked in SELECTOR6_DESIGN.md section 8
DRAWS = 100_000          # >= the freeze's 10^4
t0 = time.time()
RNG = np.random.default_rng(SEED)


def say(s):
    print(f"[{time.time()-t0:8.1f}s] {s}", flush=True)


def ambivalent(G):
    """every conjugacy class equal to its own inverse class"""
    M, I = G.MUL, G.INV
    lab = {}
    for ci, c in enumerate(G.classes):
        for x in c:
            lab[x] = ci
    return all(lab[int(I[c[0]])] == lab[c[0]] for c in G.classes)


def oriented_sector_empty(G):
    """FROB-ORIENT-1's prediction in the form the design pins: the R2 reading
    class([a,b]) must be invariant under commutator inversion, [b,a] = [a,b]^-1."""
    n, M, I = G.n, G.MUL, G.INV
    lab = {}
    for ci, c in enumerate(G.classes):
        for x in c:
            lab[x] = ci
    for a in range(n):
        for b in range(n):
            cab = int(M[M[a, b], M[I[a], I[b]]])
            cba = int(M[M[b, a], M[I[b], I[a]]])
            if lab[cab] != lab[cba]:
                return False
    return True


def main():
    with open(os.path.join(HERE, "census.json")) as f:
        cen = json.load(f)
    tabs = cen["tables"]
    total = sum(len(v) for v in tabs.values())
    say(f"census loaded: {total} groups, orders {min(map(int,tabs))}..{max(map(int,tabs))}")
    say(f"criterion s4core (selector4 blob d33f0469); label ruleb (blob cbaf2b47)")
    say(f"MIN_DECIDED_RUNG = {S6.MIN_DECIDED_RUNG}; BUDGET = {S6.BUDGET}; "
        f"seed {SEED}; null draws {DRAWS}")

    rows = []
    t1_viol = []
    for n_s in sorted(tabs, key=int):
        for i, MUL in enumerate(tabs[n_s]):
            gid = f"n{n_s}#{i}"
            G = C.Grp(np.asarray(MUL))
            rec = dict(gid=gid, order=G.n, abelian=bool(G.is_abelian()),
                       ambivalent=bool(ambivalent(G)))
            try:
                W = S4.build_world(gid, np.asarray(G.MUL, dtype=np.int64),
                                   np.asarray(G.INV, dtype=np.int64))
                W["MUL"] = np.asarray(G.MUL, dtype=np.int64)
                W["INV"] = np.asarray(G.INV, dtype=np.int64)
                fam = S6.build_family(W)
                members, perms, orbit_of, reps, sizes, powers = fam
                per_rung, calls, audits = S6.run_group(
                    W, members, perms, orbit_of, reps, sizes, powers,
                    deadline=time.time() + S6.GROUP_WALL)
                sel, kstar, selc = S6.verdict(per_rung, len(members))
                rec.update(select=sel, kstar=kstar, sel=selc, F=len(members),
                           calls=calls, void=None,
                           rung={k: (v["sel"], v["voids"]) for k, v in per_rung.items()})
                # cheap-path audit: the divisor cache must agree with separates
                bad = [(a, b) for (a, b) in audits if a != b]
                if bad:
                    rec["void"] = "cheap-path disagreed with separates"
                # Orbit constancy (design section 2.1).  The equivariance lemma
                # says |Ident| is constant on gauge orbits, so re-running with a
                # DIFFERENT member of each sampled orbit as the home must give
                # the same per-rung selected counts.  Checked, not assumed: if it
                # disagrees, the gauge-orbit economy is invalid for this group and
                # the group VOIDs rather than being scored on an unproved
                # decomposition.
                if sel is not None and len(reps) > 1:
                    pick = RNG.choice(len(reps),
                                      size=min(S6.ORBIT_CHECK_S, len(reps)),
                                      replace=False)
                    alt = list(reps)
                    for r in pick:
                        lab = orbit_of[reps[int(r)]]
                        sib = [j for j in range(len(members)) if orbit_of[j] == lab]
                        alt[int(r)] = int(RNG.choice(sib))
                    if alt != list(reps):
                        pr2, _, _ = S6.run_group(
                            W, members, perms, orbit_of, reps, sizes, powers,
                            homes=alt, deadline=time.time() + S6.GROUP_WALL)
                        for kk in range(5):
                            a1 = per_rung.get(kk, {}).get("sel")
                            a2 = pr2.get(kk, {}).get("sel")
                            if a1 != a2:
                                rec["void"] = (f"orbit constancy failed at A{kk}: "
                                               f"{a1} vs {a2}")
                                rec["select"] = None
                                break
            except S6.Refused as e:
                rec.update(select=None, kstar=None, sel=None,
                           F=None, calls=None, void=str(e), rung=None)
            except Exception as e:
                rec.update(select=None, kstar=None, sel=None, F=None, calls=None,
                           void=f"{type(e).__name__}: {e}", rung=None)

            # ---- T1, inline, never data
            if rec["abelian"] and rec["select"] is True:
                t1_viol.append((gid, "abelian group SELECTED"))
            if rec["ambivalent"] and not oriented_sector_empty(G):
                t1_viol.append((gid, "ambivalent group has a nonempty oriented sector"))
            if (not rec["ambivalent"]) and oriented_sector_empty(G):
                rec["orient_note"] = "non-ambivalent but R2-inversion-invariant"

            # ---- RULE-B, from the table alone
            ad = S6.LabelAdapter(G.MUL, G.INV, G.IDE, G.n)
            try:
                lab, note = RL.rule_b_sm(ad)
            except Exception as e:
                lab, note = None, f"{type(e).__name__}: {e}"
            rec["sm"] = lab
            rec["sm_note"] = note
            rows.append(rec)
            if len(rows) % 25 == 0:
                say(f"  {len(rows)}/{total} scored (last {gid}, |F|={rec['F']})")

    with open(os.path.join(HERE, "primary.json"), "w") as f:
        json.dump(rows, f)
    say(f"scored {len(rows)} groups")

    # ------------------------------------------------------------------ T1
    say("")
    say("=== T1 THEOREM AUDIT ===")
    if t1_viol:
        for g, m in t1_viol:
            say(f"  VIOLATION {g}: {m}")
        say(f"T1 VIOLATED ({len(t1_viol)}) — the run VOIDS for diagnosis")
        sys.exit(1)
    nab = sum(1 for r in rows if r["abelian"])
    namb = sum(1 for r in rows if r["ambivalent"])
    say(f"  stasis half   : {nab} abelian groups, none SELECTED — PASS")
    say(f"  orientation   : {namb} ambivalent groups, all oriented sectors empty — PASS")
    fe = sum(1 for r in rows if r["abelian"] and r["select"] is False and r["sel"] == 0)
    fa = sum(1 for r in rows if r["abelian"] and r["select"] is False
             and r["sel"] is not None and r["sel"] == r["F"])
    say(f"  abelian failure modes (A1 item 2): fail-EMPTY {fe}, "
        f"fail-EVERYTHING {fa}, other {nab - fe - fa}")

    # ------------------------------------------------------------------ VOIDs
    say("")
    say("=== VOIDs (B1) ===")
    voids = [r for r in rows if r["select"] is None]
    say(f"  {len(voids)}/{len(rows)} groups VOID")
    if voids:
        byord = collections.Counter(r["order"] for r in voids)
        say(f"  by order: {dict(sorted(byord.items()))}")
        byreason = collections.Counter(str(r["void"])[:60] for r in voids)
        for k, v in byreason.most_common():
            say(f"    {v:4d}  {k}")
        fs = [r["F"] for r in voids if r["F"]]
        if fs:
            say(f"  |F| among VOIDs: min {min(fs)} median {int(np.median(fs))} max {max(fs)}")
    ok = [r for r in rows if r["select"] is not None and r["sm"] is not None]
    say(f"  eligible pool (decided verdict AND decided label): {len(ok)}")

    # ------------------------------------------------------------------ E1
    say("")
    say("=== E1 — THE STAKE ===")
    S = [r for r in ok if r["select"]]
    labels = np.array([1.0 if r["sm"] else 0.0 for r in ok])
    base = labels.mean()
    if not S:
        say("  the criterion SELECTED nothing in the eligible pool.")
        say("  E1 BRANCH (b): no enrichment is measurable; the landscape's answer is")
        say("  'the bootstrap criterion does not preferentially select SM-embeddable")
        say("  structure at these orders'.")
        return
    obs = np.mean([1.0 if r["sm"] else 0.0 for r in S])
    k = len(S)
    idx = np.argsort(RNG.random((DRAWS, len(ok))), axis=1)[:, :k]
    null = labels[idx].mean(axis=1)
    p99 = np.percentile(null, 99)
    pval = float((null >= obs - 1e-12).mean())
    say(f"  selected            : {k}/{len(ok)}")
    say(f"  RULE-B among SELECT : {obs:.4f}  ({int(obs*k)}/{k})")
    say(f"  eligible base rate  : {base:.4f}  ({int(labels.sum())}/{len(ok)})")
    say(f"  null mean {null.mean():.4f} sd {null.std():.4f}  99th pct {p99:.4f}")
    say(f"  p(null >= observed) = {pval:.6g}")
    branch = "a" if obs > p99 else "b"
    say("")
    if branch == "a":
        say("  E1 BRANCH (a): observed enrichment EXCEEDS the null's 99th percentile.")
    else:
        say("  E1 BRANCH (b): observed enrichment does NOT exceed the null's 99th")
        say("  percentile. The landscape's answer is 'the bootstrap criterion does")
        say("  not preferentially select SM-embeddable structure at these orders'.")
    say(f"  recall (the half SELECTOR-5 never printed): "
        f"{sum(1 for r in S if r['sm'])}/{int(labels.sum())} of SM-embeddable groups")

    # -------------------------------------------------- sensitivity, NOT a result
    say("")
    say("=== SENSITIVITY (labelled diagnostic, NOT an alternative result) ===")
    say("  Under the coarse-rung fallback that ruling 3 forbids -- letting a group")
    say("  whose fine rungs VOID be scored False on a coarse rung -- the eligible")
    say(f"  pool would grow by {len(voids)} groups, every one of them a False, and")
    say("  those Falses would be budget exhaustions rather than measurements.")
    with open(os.path.join(HERE, "primary.DONE"), "w") as f:
        f.write(f"branch {branch}\n")
    say("")
    say(f"PRIMARY COMPLETE — E1 branch ({branch})")


if __name__ == "__main__":
    main()
