#!/usr/bin/env python3
"""PT-2OI instrument.  Prereg ADMITTED and frozen before this file
(PT2OI_PREREG.md at HEAD), with the recon module binary_groups.py vendored
beside it and imported here.

Every staked number in STAKED below is a LITERAL transcribed from the frozen
prereg.  The instrument MEASURES independently -- by enumerating the punctured
torus over the group -- and compares against those literals.  Nothing in the
gate path recomputes the prediction from the group, or the gate would be
circular.

Mass is the conjugacy CLASS.  The deficit angle is a derived label and, at 2O,
NOT injective: two classes share deficit pi, one realizable and one exactly
empty (gate L1b).  Classes are therefore keyed by (deficit, class size), which
the recon verified is unique in both groups.
"""
import os
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from binary_groups import (ANGLE_PHI, ANGLE_SQRT2, Grp, build_2I, build_2O)

# ==========================================================================
# THE FROZEN PREDICTIONS -- literals from PT2OI_PREREG.md, not recomputed
# ==========================================================================
STAKED = {
    "2O": {
        "order": 48,
        "n_classes": 8,
        "configs": 2304,
        # (deficit, class size) -> occupancy of the 2304 configurations
        "rungs": {("0", 1): 384, ("2*pi/3", 8): 864, ("pi", 6): 672,
                  ("4*pi/3", 8): 288, ("2*pi", 1): 96},
        # classes staked EXACTLY ZERO -- note ("pi", 12): the split-pi gate
        "empty": {("pi/2", 6), ("3*pi/2", 6), ("pi", 12)},
        "split_pi": {"occupied": ("pi", 6, 672), "empty": ("pi", 12)},
        "closed_torus_live": 384,
        "closed_torus_empty_sectors": 7,
    },
    "2I": {
        "order": 120,
        "n_classes": 9,
        "configs": 14400,
        "rungs": {("0", 1): 1080, ("2*pi/5", 12): 2400, ("2*pi/3", 20): 4320,
                  ("4*pi/5", 12): 720, ("pi", 30): 1920, ("6*pi/5", 12): 2400,
                  ("4*pi/3", 20): 720, ("8*pi/5", 12): 720, ("2*pi", 1): 120},
        "empty": set(),                      # 2I is PERFECT: no forbidden sector
        "split_pi": None,
        "closed_torus_live": 1080,
        "closed_torus_empty_sectors": 8,
    },
}
LADDER = {"D4": 2, "2T": 3, "2O": 5, "2I": 9}

PEAK = {"base_cells": 0, "refined_cells": 0, "certificate_cells": 0}


# ==========================================================================
# helpers
# ==========================================================================
def apply_perm(psi, perm):
    """pushforward: out[perm[i]] += psi[i]  (pt2t.py's form, kept)"""
    out = np.zeros_like(psi)
    np.add.at(out, perm, psi)
    return out


def sector_weight(psi, tgt):
    """exact sum of squares inside / over all, with an explicit overflow
    guard so int64 arithmetic is provably exact here."""
    m = int(np.abs(psi).max())
    assert m * m * psi.size < 2 ** 63 - 1, "int64 overflow in sector_weight"
    w = psi.astype(np.int64) ** 2
    return int(w[tgt].sum()), int(w.sum())


class Model:
    """the once-punctured one-vertex torus over G, dense on |G|^2."""

    def __init__(self, tag, builder, angle_table):
        elems, rmul, val, ring = builder()
        self.tag = tag
        self.G = G = Grp(tag, elems, rmul, val, ring)
        self.n = n = G.n
        self.N = N = n * n
        PEAK["base_cells"] = max(PEAK["base_cells"], N)
        M, I = G.MUL, G.INV
        self.GA = np.arange(N) % n
        self.GB = np.arange(N) // n
        self.IDX = lambda a, b: a + n * b
        self.PUNCT = M[M[self.GA, self.GB], M[I[self.GA], I[self.GB]]]
        self.T_MAP = self.IDX(self.GA, M[self.GA, self.GB])
        self.S_MAP = self.IDX(self.GB, M[M[self.GB, self.GA], I[self.GB]])
        # M-NONBIJECTIVE-STEP: bijectivity by exhaustion BEFORE any gate
        self.bijective = (len(set(self.T_MAP.tolist())) == N
                          and len(set(self.S_MAP.tolist())) == N)
        # class key: (deficit, class size) -- unique in both groups (recon)
        self.key = {}
        for c in range(G.ncls):
            members = [g for g in range(n) if G.CLS[g] == c]
            self.key[c] = (angle_table[G.scalar(members[0])][0], len(members))
        assert len(set(self.key.values())) == G.ncls, "class key not unique"

    def gauge_perm(self, x):
        M, I = self.G.MUL, self.G.INV
        return self.IDX(M[M[x, self.GA], I[x]], M[M[x, self.GB], I[x]])

    def gauss_project(self, psi):
        out = np.zeros_like(psi)
        for x in range(self.n):
            out += apply_perm(psi, self.gauge_perm(x))
        return out

    def gauss_holds(self, psi):
        return all(np.array_equal(apply_perm(psi, self.gauge_perm(x)), psi)
                   for x in range(self.n))

    def step(self, psi):
        return apply_perm(apply_perm(psi, self.T_MAP), self.S_MAP)


# ==========================================================================
# base-torus gates
# ==========================================================================
def record(rep, gate, tag, ok, msg):
    """one gate result per group, kept as a BOOLEAN plus its text, so the
    final verdict is never inferred by string-matching."""
    rep.setdefault(gate, []).append((tag, bool(ok), msg))


def base_gates(m, staked, rep, detail):
    G, n = m.G, m.n
    t = m.tag

    # ---- measurement: occupancy per class, by enumerating the torus -------
    occ = {}
    for c in range(G.ncls):
        occ[m.key[c]] = int(np.count_nonzero(G.CLS[m.PUNCT] == c))
    assert sum(occ.values()) == m.N

    # ---- G0 --------------------------------------------------------------
    idc = int(G.CLS[G.e])
    psi0 = m.gauss_project((G.CLS[m.PUNCT] == idc).astype(np.int64))
    g0 = bool(np.count_nonzero(psi0)) and m.gauss_holds(psi0)
    record(rep, "G0", t, g0, "PASS" if g0 else "FIRE")

    # ---- L1: occupancies equal the frozen literals EXACTLY ---------------
    bad = []
    for k, want in staked["rungs"].items():
        got = occ.get(k)
        if got != want:
            bad.append((k, got, want))
    for k in staked["empty"]:
        got = occ.get(k)
        if got != 0:
            bad.append((k, got, 0))
    # every class must be accounted for by the staked table
    covered = set(staked["rungs"]) | staked["empty"]
    if set(occ) != covered:
        bad.append(("class-set", sorted(occ), sorted(covered)))
    # the Gauss-projected sector must be nonzero exactly where occupancy is
    for c in range(G.ncls):
        live = bool(np.count_nonzero(
            m.gauss_project((G.CLS[m.PUNCT] == c).astype(np.int64))))
        if live != (occ[m.key[c]] > 0):
            bad.append((m.key[c], "sector/occupancy disagree", live))
    record(rep, "L1", t, not bad,
           f"PASS ({len(staked['rungs'])} rungs, occupancies exact)"
           if not bad else f"FIRE {bad!r}")
    detail.append(f"  [{t}] occupancies measured over {m.N} configs, "
                  f"{len(staked['rungs'])} rungs staked, "
                  f"{len(staked['empty'])} classes staked empty")
    for k in sorted(occ, key=lambda k: -occ[k]):
        flag = "RUNG" if occ[k] else "empty"
        detail.append(f"      deficit {k[0]:>8}  size {k[1]:>3}  "
                      f"occupancy {occ[k]:>6}   {flag}")

    # ---- L1b: the split-pi gate (2O only) --------------------------------
    if staked["split_pi"]:
        d, sz, want = staked["split_pi"]["occupied"]
        ed, esz = staked["split_pi"]["empty"]
        ok = (occ.get((d, sz)) == want and occ.get((ed, esz)) == 0)
        record(rep, "L1b", t, ok,
               f"{'PASS' if ok else 'FIRE'} "
               f"(pi/size{sz}={occ.get((d, sz))}, "
               f"pi/size{esz}={occ.get((ed, esz))})")

    # ---- L2: class conservation, and B3 on every trajectory state --------
    if not m.bijective:
        record(rep, "L2", t, False, "FIRE (base step not bijective)")
        record(rep, "B3", t, False, "FIRE")
        return occ
    # (a) pointwise, exhaustive over every configuration
    ptwise = bool(np.array_equal(G.CLS[m.PUNCT[m.T_MAP]], G.CLS[m.PUNCT])
                  and np.array_equal(G.CLS[m.PUNCT[m.S_MAP]], G.CLS[m.PUNCT]))
    # (b) sector-weight ratio along the trajectory, on EVERY realizable rung
    drift, states = [], []
    for c in range(G.ncls):
        if occ[m.key[c]] == 0:
            continue
        tgt = G.CLS[m.PUNCT] == c
        psi = m.gauss_project(tgt.astype(np.int64))
        w0 = sector_weight(psi, tgt)
        for k in range(1, 7):
            psi = m.step(psi)
            states.append(psi)
            w = sector_weight(psi, tgt)
            if w[0] * w0[1] != w0[0] * w[1]:
                drift.append((m.key[c], k, w, w0))
    l2 = ptwise and not drift
    record(rep, "L2", t, l2,
           f"PASS (class conserved pointwise on all {m.N} configs; "
           f"weight ratio held on every rung)" if l2
           else f"FIRE ptwise={ptwise} drift={drift!r}")
    b3 = all(m.gauss_holds(s) for s in states)
    record(rep, "B3", t, b3,
           f"{'PASS' if b3 else 'FIRE'} ({len(states)} trajectory states)")
    return occ


# ==========================================================================
# L3 -- 2O: DENSE both-edges lift
# ==========================================================================
def l3_dense(m, detail):
    G, n, M, I = m.G, m.n, m.G.MUL, m.G.INV
    NR = n ** 4
    PEAK["refined_cells"] = max(PEAK["refined_cells"], NR)
    ar = np.arange(NR, dtype=np.int64)
    A1 = ar % n
    A2 = (ar // n) % n
    B1 = (ar // n ** 2) % n
    B2 = ar // n ** 3
    del ar
    RIDX = lambda a1, a2, b1, b2: a1 + n * a2 + n * n * b1 + n ** 3 * b2
    GA = M[A1, A2]
    GB = M[B1, B2]
    PUNCT = M[M[GA, GB], M[I[GA], I[GB]]]

    T_REF = RIDX(A1, A2, M[GA, B1], B2)
    _c = GB
    S_REF = RIDX(B1, B2, M[M[_c, A1], I[_c]], M[M[_c, A2], I[_c]])
    # bijectivity FIRST (M-NONBIJECTIVE-STEP)
    bij = bool(np.bincount(T_REF, minlength=NR).max() == 1
               and np.bincount(S_REF, minlength=NR).max() == 1)
    if not bij:
        return False, "FIRE (refined lift not bijective)"

    PERMS = [
        lambda x: RIDX(M[x, A1], M[A2, I[x]], M[x, B1], M[B2, I[x]]),
        lambda x: RIDX(M[A1, I[x]], M[x, A2], B1, B2),
        lambda x: RIDX(A1, A2, M[B1, I[x]], M[x, B2]),
    ]

    def r_gauss_project(psi):
        for pf in PERMS:
            acc = np.zeros_like(psi)
            for x in range(n):
                acc += apply_perm(psi, pf(x))
            psi = acc
        return psi

    def r_gauss_holds(psi):
        return all(np.array_equal(apply_perm(psi, pf(x)), psi)
                   for pf in PERMS for x in range(n))

    def r_step(psi):
        return apply_perm(apply_perm(psi, T_REF), S_REF)

    # spectrum identical to the base
    r_occ, base_occ = {}, {}
    for c in range(G.ncls):
        r_occ[m.key[c]] = int(np.count_nonzero(G.CLS[PUNCT] == c))
        base_occ[m.key[c]] = int(np.count_nonzero(G.CLS[m.PUNCT] == c))
    r_live = {k for k, v in r_occ.items() if v}
    b_live = {k for k, v in base_occ.items() if v}
    same = r_live == b_live
    # every refined occupancy must be exactly |G|^2 times the base one
    scaled = all(r_occ[k] == base_occ[k] * n * n for k in r_occ)
    # liveness read the way PT-2T read it: through the refined Gauss projector
    proj_live = set()
    for c in range(G.ncls):
        if np.count_nonzero(r_gauss_project(
                (G.CLS[PUNCT] == c).astype(np.int64))):
            proj_live.add(m.key[c])
    same &= (proj_live == b_live)
    detail.append(f"  [2O] refined {NR} configs; live classes {len(r_live)} "
                  f"(projector agrees: {proj_live == b_live}); "
                  f"occupancy = base x |G|^2 exactly: {scaled}")

    # conservation + refined Gauss on trajectory states
    drift, b3 = [], True
    live_cls = [c for c in range(G.ncls) if base_occ[m.key[c]]]
    for c in (live_cls[0], live_cls[-1]):
        tgt = G.CLS[PUNCT] == c
        psi = r_gauss_project(tgt.astype(np.int64))
        w0 = sector_weight(psi, tgt)
        for k in range(1, 4):
            psi = r_step(psi)
            if not r_gauss_holds(psi):
                b3 = False
            w = sector_weight(psi, tgt)
            if w[0] * w0[1] != w0[0] * w[1]:
                drift.append((m.key[c], k))
    ok = same and scaled and not drift and b3
    return ok, (f"PASS (dense {NR}: bijective, spectrum identical, "
                "occupancies x|G|^2, conserved, refined Gauss held)" if ok else
                f"FIRE same={same} scaled={scaled} drift={drift} B3={b3}")


# ==========================================================================
# L3 -- 2I: the PULLBACK CERTIFICATE (lemmas by exhaustion at <= |G|^3)
# ==========================================================================
def l3_certificate(m, detail):
    G, n, M, I = m.G, m.n, m.G.MUL, m.G.INV
    PEAK["certificate_cells"] = max(PEAK["certificate_cells"], 2 * n * n)
    ar = np.arange(n)
    L = {}

    # L-A: every fiber of pi(a1,a2,b1,b2) = (a1a2, b1b2) has size |G|^2
    L["L-A"] = bool((np.bincount(M.ravel(), minlength=n) == n).all())
    # L-B: pi invariant under both edge-midpoint gauge actions
    L["L-B"] = all(np.array_equal(M[M[ar[:, None], I[x]], M[x, ar[None, :]]], M)
                   for x in range(n))
    # L-C: pi intertwines the vertex gauge with simultaneous conjugation
    L["L-C"] = all(np.array_equal(M[M[x, ar[:, None]], M[ar[None, :], I[x]]],
                                  M[M[x, M], I[x]]) for x in range(n))
    # L-D: the edge-midpoint actions are FREE
    L["L-D"] = all(int(M[a, I[x]]) != a
                   for x in range(n) if x != G.e for a in range(n))
    # L-E: both Dehn lifts intertwine with the base maps
    L["L-E"] = (all(np.array_equal(M[M[g, ar[:, None]], ar[None, :]], M[g, M])
                    for g in range(n))
                and all(np.array_equal(
                    M[M[M[c, ar[:, None]], I[c]], M[M[c, ar[None, :]], I[c]]],
                    M[M[c, M], I[c]]) for c in range(n)))
    # L-F: both lifts bijective (left translation / conjugation, explicit
    #      inverses), blockwise by exhaustion
    L["L-F"] = (all(len(set(M[g, :].tolist())) == n for g in range(n))
                and all(len(set(M[M[c, ar], I[c]].tolist())) == n
                        for c in range(n)))
    detail.append(f"  [2I] certificate lemmas at <= |G|^3 = {n**3}: "
                  + ", ".join(f"{k}={v}" for k, v in L.items()))
    ok = all(L.values())
    return ok, ("PASS (pullback certificate: L-A..L-F all exhaustive, dense "
                f"{n**4} run NOT required)" if ok else
                f"FIRE {[k for k, v in L.items() if not v]}")


# ==========================================================================
# plants
# ==========================================================================
def plant_closed_torus(m, staked):
    """(i) closed-torus control: filling the puncture ([g_a,g_b] = 1) must
    leave EXACTLY the identity sector live, with the staked live count."""
    G = m.G
    idc = int(G.CLS[G.e])
    closed_mask = (m.PUNCT == G.e)
    carrier = m.gauss_project(closed_mask.astype(np.int64))
    live = int(np.count_nonzero(closed_mask))
    # ASSERT THE CARRIER NONZERO IN THE SECTOR IT ACTS ON, BEFORE SCORING
    in_sector = int(carrier[G.CLS[m.PUNCT] == idc].sum())
    if in_sector == 0:
        return False, "VOID (carrier dead in the identity sector)"
    empties = [c for c in range(G.ncls)
               if c != idc and int(np.count_nonzero(
                   closed_mask & (G.CLS[m.PUNCT] == c)))]
    n_empty = G.ncls - 1 - len(empties)
    fired = (live == staked["closed_torus_live"]
             and not empties
             and n_empty == staked["closed_torus_empty_sectors"])
    return fired, (f"live twin={live} (staked {staked['closed_torus_live']}), "
                   f"{n_empty}/{G.ncls - 1} other sectors exactly empty "
                   f"(staked {staked['closed_torus_empty_sectors']}), "
                   f"carrier weight in identity sector={in_sector}")


def plant_broken_twist(m):
    """(ii) broken twist: a non-covariant twist must break Gauss on a
    single-orbit carrier (PT-2T's construction)."""
    G = m.G
    s_el = G.idx[(0, 0, 2, 0, 0, 0, 0, 0)]              # i
    rot = G.idx[(1, 0, 1, 0, 1, 0, 1, 0)]               # (1+i+j+k)/2
    delta = np.zeros(m.N, dtype=np.int64)
    delta[m.IDX(s_el, G.e)] = 1
    carrier = m.gauss_project(delta)
    if not np.any(carrier):                      # carrier nonzero, asserted
        return False, "VOID (carrier dead)"
    bad = apply_perm(carrier, m.IDX(m.GA, G.MUL[np.full(m.N, rot), m.GB]))
    fired = not m.gauss_holds(bad)
    return fired, (f"carrier orbit weight={int(carrier.sum())}, "
                   f"Gauss after twist held={not fired}")


# ==========================================================================
def main():
    rep, detail = {}, []
    models = {}
    for tag, builder, table in (("2O", build_2O, ANGLE_SQRT2),
                                ("2I", build_2I, ANGLE_PHI)):
        m = Model(tag, builder, table)
        assert m.n == STAKED[tag]["order"], "group order != staked"
        assert m.G.ncls == STAKED[tag]["n_classes"], "class count != staked"
        assert m.N == STAKED[tag]["configs"], "config count != staked"
        assert m.bijective, f"{tag} base Dehn lifts not bijective"
        models[tag] = m
        detail.append(f"[{tag}] order {m.n}, {m.G.ncls} classes, {m.N} configs, "
                      f"base lifts bijective by exhaustion")
        base_gates(m, STAKED[tag], rep, detail)

    ok3, msg3 = l3_dense(models["2O"], detail)
    record(rep, "L3", "2O", ok3, msg3)
    ok3, msg3 = l3_certificate(models["2I"], detail)
    record(rep, "L3", "2I", ok3, msg3)

    # the ladder as MEASURED: rungs = classes with nonzero occupancy
    got = {}
    for t, m in models.items():
        got[t] = sum(1 for c in range(m.G.ncls)
                     if np.count_nonzero(m.G.CLS[m.PUNCT] == c))
    ladder_ok = all(got[t] == LADDER[t] for t in got)

    print("\n".join(detail))
    print(f"\nLADDER (measured): D4=2 -> 2T=3 -> 2O={got['2O']} -> "
          f"2I={got['2I']}   staked {LADDER}  "
          f"{'OK' if ladder_ok else 'MISMATCH'}")
    print("\nGATES: " + "  ".join(
        f"{k}=[{'; '.join(f'{tag}:{msg}' for tag, _, msg in v)}]"
        for k, v in sorted(rep.items())))

    ok = True
    for tag in ("2O", "2I"):
        f, msg = plant_closed_torus(models[tag], STAKED[tag])
        print(f"[plant i/{tag}] closed-torus control: {msg} -> "
              f"{'FIRES' if f else 'MISSED'}")
        ok &= f
    for tag in ("2O", "2I"):
        f, msg = plant_broken_twist(models[tag])
        print(f"[plant ii/{tag}] broken twist: {msg} -> "
              f"{'FIRES' if f else 'MISSED'}")
        ok &= f

    hard = all(passed for v in rep.values() for _, passed, _ in v)
    fired_gates = [f"{k}/{tag}: {msg}" for k, v in sorted(rep.items())
                   for tag, passed, msg in v if not passed]
    if fired_gates:
        print("\nFIRED GATES:\n  " + "\n  ".join(fired_gates))
    print(f"\nRESOURCES: peak base array {PEAK['base_cells']} cells, "
          f"peak refined array {PEAK['refined_cells']} cells "
          f"({PEAK['refined_cells'] * 8 / 2**20:.1f} MiB per int64 vector), "
          f"certificate peak {PEAK['certificate_cells']} cells; "
          f"2I dense {120**4} NEVER allocated")
    print(f"\nVERDICT: {'ALL PASS' if (hard and ladder_ok) else 'FIRED'}; "
          f"plants {'all FIRE' if ok else 'VOID'}")
    sys.exit(0 if (hard and ladder_ok and ok) else 1)


if __name__ == "__main__":
    main()
