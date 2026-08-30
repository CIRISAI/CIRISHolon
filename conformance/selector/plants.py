#!/usr/bin/env python3
"""plants.py — SELECTOR-6's three plants, demonstrated before the primary.

Each plant's carrier is asserted nonzero in the sector the plant acts on BEFORE
the plant is scored; a plant on an empty sector VOIDs (M-PLANT-SECTOR).  A plant
that does not fire VOIDs the campaign — it is not written up as a curiosity.

  (i)   tag blindness      — permuting names and serials moves no verdict.
  (ii)  isomorphic pairs   — re-presentation gives bit-identical verdicts;
                             a corrupted table is REFUSED, not scored.
  (iii) null integrity     — the E1 null reports nothing on a random label
                             vector at the staked threshold in >= 99% of trials.
"""
import json
import os
import sys
import time

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import s4core as S4
import census as C
import selector6 as S6
import ruleb as RL          # the PINNED, tag-free RULE-B (make_ruleb.py)

RNG = np.random.default_rng(20260830)
t0 = time.time()
FAIL = []


def say(s):
    print(f"[{time.time()-t0:7.1f}s] {s}", flush=True)


def check(cond, msg):
    if not cond:
        FAIL.append(msg)
        say(f"    PLANT FAILED: {msg}")
    return cond


# ------------------------------------------------------------------ machinery

def score(MUL, name="?"):
    """The full per-group verdict, from a multiplication table alone."""
    G = C.Grp(np.asarray(MUL))
    W = S4.build_world(name, np.asarray(G.MUL, dtype=np.int64),
                       np.asarray(G.INV, dtype=np.int64))
    W["MUL"] = np.asarray(G.MUL, dtype=np.int64)
    W["INV"] = np.asarray(G.INV, dtype=np.int64)
    members, perms, orbit_of, reps, sizes, powers = S6.build_family(W)
    per_rung, calls, audits = S6.run_group(
        W, members, perms, orbit_of, reps, sizes, powers,
        deadline=time.time() + S6.GROUP_WALL)
    sel, kstar, selcount = S6.verdict(per_rung, len(members))
    return dict(select=sel, kstar=kstar, sel=selcount, F=len(members),
                per_rung={k: (v["sel"], v["voids"]) for k, v in per_rung.items()},
                audits=audits)


def relabel(MUL, perm):
    """Re-present a group: relabel elements by `perm` (perm[old] = new)."""
    n = len(perm)
    M = np.asarray(MUL)
    out = np.zeros((n, n), dtype=np.int64)
    inv = np.empty(n, dtype=np.int64)
    inv[perm] = np.arange(n)
    for a in range(n):
        for b in range(n):
            out[a, b] = perm[M[inv[a], inv[b]]]
    return out


def validator(MUL):
    """The census validator: refuse anything that is not a group table."""
    try:
        G = C.Grp(np.asarray(MUL))
    except ValueError as e:
        return False, f"refused: {e}"
    if not G.check_associative():
        return False, "refused: non-associative"
    return True, "admitted"


# ---------------------------------------------------------------- plant (ii)b

def plant_refusal(tables):
    say("PLANT (ii) REFUSAL — a corrupted table must be refused, not scored")
    carrier = 0
    refused = 0
    silent = 0
    for nm, M in tables:
        M = np.asarray(M)
        n = M.shape[0]
        ok, _ = validator(M)
        if not check(ok, f"{nm}: the UNcorrupted table was not admitted"):
            continue
        for trial in range(3):
            a, b = int(RNG.integers(n)), int(RNG.integers(n))
            old = int(M[a, b])
            new = int(RNG.integers(n))
            if new == old:
                new = (old + 1) % n
            Mc = M.copy()
            Mc[a, b] = new
            # the defect must be OBSERVABLE: a corruption that leaves a valid
            # group would be a bad plant, not a passing one
            latin = (len(set(Mc[a].tolist())) == n and
                     len(set(Mc[:, b].tolist())) == n)
            carrier += 1
            good, why = validator(Mc)
            if good:
                silent += 1
                say(f"    {nm}: corruption at ({a},{b}) {old}->{new} was SILENT "
                    f"(still a valid group table) — latin={latin}")
            else:
                refused += 1
    check(carrier > 0, "refusal plant has an empty carrier")
    check(silent == 0, f"{silent} corruptions were not caught by the validator")
    say(f"    carrier {carrier} corruptions, refused {refused}, silent {silent}")
    return refused, silent


# ----------------------------------------------------------------- plant (ii)a

def plant_representation(pairs):
    say("PLANT (ii) RE-PRESENTATION — one group, two presentations, one verdict")
    for nm, Ma, Mb in pairs:
        ra, rb = score(Ma, nm + "-a"), score(Mb, nm + "-b")
        same = (ra["select"] == rb["select"] and ra["kstar"] == rb["kstar"]
                and ra["sel"] == rb["sel"] and ra["F"] == rb["F"])
        say(f"    {nm:28s} A:(select={ra['select']}, k*={ra['kstar']}, "
            f"sel={ra['sel']}, F={ra['F']})  B:(select={rb['select']}, "
            f"k*={rb['kstar']}, sel={rb['sel']}, F={rb['F']})  identical={same}")
        check(same, f"{nm}: verdicts differ between presentations")


# ------------------------------------------------------------------ plant (i)

def plant_tag_blindness(tables):
    say("PLANT (i) TAG BLINDNESS — permuting names and serials moves no verdict")
    base = [(nm, score(M, nm)) for nm, M in tables]
    check(any(r["select"] is not None for _, r in base),
          "tag-blindness carrier is empty (no decided verdict to move)")
    order = RNG.permutation(len(tables))
    shuffled = [(f"anonymous_{int(RNG.integers(10**6))}", tables[i][1]) for i in order]
    got = {}
    for nm, M in shuffled:
        got[C.Grp(np.asarray(M)).fingerprint] = score(M, nm)
    moved = 0
    for nm, r in base:
        fp = C.Grp(np.asarray(dict(tables)[nm])).fingerprint
        g = got[fp]
        if (g["select"], g["kstar"], g["sel"], g["F"]) != \
           (r["select"], r["kstar"], r["sel"], r["F"]):
            moved += 1
            say(f"    {nm}: verdict MOVED under renaming")
    check(moved == 0, f"{moved} verdicts moved under renaming")
    say(f"    {len(base)} groups renamed and reordered, {moved} verdicts moved")


# ----------------------------------------------------------------- plant (iii)

def plant_null_integrity(npool=140, nsel=45, trials=300, draws=10000):
    """The staked E1 null, run against a label vector that is itself a uniform
    random relabelling.  Since `obs` is then one more exchangeable draw from the
    null's own distribution, the firing rate must sit at the threshold, ~1%."""
    say("PLANT (iii) NULL INTEGRITY — the null must say nothing when there is nothing")
    rng = np.random.default_rng(4242)
    fired = 0
    spreads = []
    for _ in range(trials):
        labels = (rng.random(npool) < 0.6).astype(float)   # MEANINGLESS labels
        obs = labels[rng.permutation(npool)[:nsel]].mean()
        # `draws` equal-size subsets, vectorised: argsort of random keys
        idx = np.argsort(rng.random((draws, npool)), axis=1)[:, :nsel]
        null = labels[idx].mean(axis=1)
        spreads.append(null.std())
        if obs > np.percentile(null, 99):
            fired += 1
    rate = fired / trials
    say(f"    {trials} trials x {draws} draws on random labels: "
        f"fired {fired} ({rate:.2%}); staked ceiling 1%, tolerance 2%")
    say(f"    mean null spread {np.mean(spreads):.4f} "
        f"(a degenerate null would read 0)")
    check(np.mean(spreads) > 1e-6, "null-integrity carrier is degenerate "
                                   "(the null does not move at all)")
    check(rate <= 0.02, f"null fired on random labels at {rate:.2%}")


# ----------------------------------------------------------------------- main

def main():
    with open(os.path.join(HERE, "census.json")) as f:
        cen = json.load(f)
    tabs = cen["tables"]
    say(f"census loaded: {sum(len(v) for v in tabs.values())} groups")

    # a small, declared panel: cheap orders, spanning abelian and nonabelian
    panel = []
    for n in ("6", "8", "10", "12", "16", "20", "21", "24"):
        for i, M in enumerate(tabs.get(n, [])[:4]):
            panel.append((f"n{n}#{i}", np.asarray(M)))
    say(f"panel: {len(panel)} groups")

    # plant (ii) refusal
    plant_refusal(panel[:12])
    say("")

    # plant (ii) re-presentation: >= 8 pairs, A4/Delta(12) mandatory
    import landscape_sweep as ls
    z2 = ls.build_cyclic(2)
    pairs = [("A_4 vs Delta(12) [MANDATORY]",
              np.asarray(ls.build_alternating(4).MUL),
              np.asarray(ls.build_delta_3n2(2).MUL)),
             ("D_12 vs Z_2xD_6",
              np.asarray(ls.build_dihedral(6).MUL),
              np.asarray(ls.build_direct_product(z2, ls.build_dihedral(3)).MUL)),
             ("D_20 vs Z_2xD_10",
              np.asarray(ls.build_dihedral(10).MUL),
              np.asarray(ls.build_direct_product(z2, ls.build_dihedral(5)).MUL))]
    for nm, M in panel[:6]:
        n = M.shape[0]
        p = RNG.permutation(n)
        pairs.append((f"{nm} vs relabelled", M, relabel(M, p)))
    plant_representation(pairs)
    say("")

    plant_tag_blindness(panel[:10])
    say("")

    plant_null_integrity()
    say("")

    if FAIL:
        say(f"PLANTS FAILED ({len(FAIL)}):")
        for m in FAIL:
            say(f"   - {m}")
        sys.exit(1)
    say("ALL PLANTS FIRED AS STAKED")
    with open(os.path.join(HERE, "plants.DONE"), "w") as f:
        f.write("ok\n")


if __name__ == "__main__":
    main()
