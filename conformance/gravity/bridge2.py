#!/usr/bin/env python3
"""BRIDGE-2 instrument. Prereg GRAVITY_BRIDGE2_PREREG.md frozen at 4e26345
before this file. Model and dynamics imported BYTE-IDENTICAL from bridge1
(the passed rungs B3/B4/plants re-run unchanged); only the four re-staked
rungs are new. Exact arithmetic throughout."""
import sys
import numpy as np
from fractions import Fraction

sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
from bridge1 import (Model, base_graph, refined_graph, CLASS, KAPPA, MUL, INV,
                     rho2, markov_route, central_parity_dist)

# ---------------------------------------------------------- B1' machinery
def sector_overlap(M, s_empty, s_occ):
    """Exact normalized gauge-sector overlap omega between the evolved
    empty and occupied states (occupation label stripped; spinor comps
    paired)."""
    re0, im0, _ = s_empty
    re1, im1, _ = s_occ
    half = M.M // 2
    ipr = 0
    ipi = 0
    n0 = 0
    n1 = 0
    for m in range(half):
        a0 = re0[m].astype(object); b0 = im0[m].astype(object)
        a1 = re1[m + half].astype(object); b1 = im1[m + half].astype(object)
        ipr += int(np.sum(a0 * a1 + b0 * b1))
        ipi += int(np.sum(a0 * b1 - b0 * a1))
        n0 += int(np.sum(a0 * a0 + b0 * b0))
        n1 += int(np.sum(a1 * a1 + b1 * b1))
    if n0 == 0 or n1 == 0:
        return None
    return Fraction(ipr * ipr + ipi * ipi, n0 * n1)

def comb_states(mk, occ, k_steps):
    """Corrected combinatorial route (M9 fix: flux phase i^{KAPPA}, and the
    same 8-dim parity-amplitude recursion as bridge1's, kept for parity
    distributions AND reused for the comb-side overlap)."""
    edges, plaq, loop, p0, e_star = mk()
    members = [[p for p, w in enumerate(plaq) if any(we == ei for we, _ in w)]
               for ei in range(len(edges))]
    amp = {0: (Fraction(1), Fraction(0))}
    def phase_mul(z, t):
        a, b = z
        t %= 4
        if t == 0:
            return (a, b)
        if t == 1:
            return (-b, a)
        if t == 2:
            return (-a, -b)
        return (b, -a)
    KAP_R2 = int(KAPPA[1])  # class index 1 = {r2}: the M9 fix (i^2, not i^1)
    for _ in range(k_steps):
        if occ:
            amp = {s ^ (1 << p0): z for s, z in amp.items()}
        amp = {s: phase_mul(z, KAP_R2 if (s >> p0) & 1 else 0)
               for s, z in amp.items()}
        for ei in range(len(edges)):
            flip = 0
            for p in members[ei]:
                flip ^= 1 << p
            nxt = {}
            for s, (a, b) in amp.items():
                ra, rb = nxt.get(s, (Fraction(0), Fraction(0)))
                nxt[s] = (ra + a, rb + b)
                s2 = s ^ flip
                ra, rb = nxt.get(s2, (Fraction(0), Fraction(0)))
                nxt[s2] = (ra - b, rb + a)
            amp = nxt
        amp = {s: phase_mul(z, KAP_R2 * bin(s).count("1")) for s, z in amp.items()}
    return amp

def comb_parity_dist(amp):
    w = {}
    for s, (a, b) in amp.items():
        par = bin(s).count("1") & 1
        w[par] = w.get(par, Fraction(0)) + a * a + b * b
    tot = sum(w.values())
    return {p: v / tot for p, v in w.items() if v / tot != 0}

def comb_overlap(a_empty, a_occ):
    ipr = Fraction(0)
    ipi = Fraction(0)
    n0 = Fraction(0)
    n1 = Fraction(0)
    keys = set(a_empty) | set(a_occ)
    for s in keys:
        a0, b0 = a_empty.get(s, (Fraction(0), Fraction(0)))
        a1, b1 = a_occ.get(s, (Fraction(0), Fraction(0)))
        ipr += a0 * a1 + b0 * b1
        ipi += a0 * b1 - b0 * a1
        n0 += a0 * a0 + b0 * b0
        n1 += a1 * a1 + b1 * b1
    return (ipr * ipr + ipi * ipi) / (n0 * n1)

# ---------------------------------------------------------- B6' enumerator
def charged_support_enumerator(mk):
    """Independent closed-form route: comp-(0,0) amplitude of the charged
    vacuum = sum over (g_c,g1,g2,g3) preimages with weight
    (rho2(g1) rho2(g2)^T)_{00} (both singlet seeds folded in)."""
    edges, plaq, loop, p0, e_star = mk()
    pw = [8 ** e for e in range(len(edges))]
    def cfg_of(gs):
        # edge value: tail vertex's g LEFT, head's g^{-1} RIGHT (bridge1's
        # joint_action convention, transcribed independently)
        vmap = {"c": gs[0], 1: gs[1], 2: gs[2], 3: gs[3]}
        tot = 0
        for i, (a, b) in enumerate(edges):
            d = MUL[vmap[a], INV[vmap[b]]]
            tot += int(d) * pw[i]
        return tot
    from collections import defaultdict
    w = defaultdict(int)
    for gc in range(8):
        for g1 in range(8):
            for g2 in range(8):
                wt = int((rho2(g1) @ rho2(g2).T)[0, 0])
                if wt == 0:
                    continue
                for g3 in range(8):
                    w[cfg_of((gc, g1, g2, g3))] += wt
    return sum(1 for v in w.values() if v != 0)

def uncharged_support_enumerator(mk):
    edges, plaq, loop, p0, e_star = mk()
    pw = [8 ** e for e in range(len(edges))]
    def cfg_of(gs):
        vmap = {"c": gs[0], 1: gs[1], 2: gs[2], 3: gs[3]}
        tot = 0
        for i, (a, b) in enumerate(edges):
            d = MUL[vmap[a], INV[vmap[b]]]
            tot += int(d) * pw[i]
        return tot
    seen = set()
    for gc in range(8):
        for g1 in range(8):
            for g2 in range(8):
                for g3 in range(8):
                    seen.add(cfg_of((gc, g1, g2, g3)))
    return len(seen)

# ------------------------------------------------------------------- run
def run(tag, mk, with_spinors, k_max=4):
    M = Model(mk, with_spinors)
    rep = {}
    registry = []
    s0 = M.vacuum(0)
    s1 = M.vacuum(1)
    registry += [("vac0", s0), ("vac1", s1)]

    # B4 replication (unchanged stake) + B1' overlaps
    want_omega = {2: Fraction(1), 4: Fraction(1)}
    b1_ok = True
    b4_ok = True
    want_delta = {1: Fraction(0), 2: Fraction(1, 2),
                  3: Fraction(0), 4: Fraction(1, 2)}
    for k in range(1, k_max + 1):
        s0 = M.step(s0)
        s1 = M.step(s1)
        registry += [(f"T{k}e", s0), (f"T{k}o", s1)]
        om_ev = sector_overlap(M, s0, s1)
        om_cb = comb_overlap(comb_states(mk, 0, k), comb_states(mk, 1, k))
        if om_ev != om_cb:
            b1_ok = False
            rep[f"B1_route_k{k}"] = f"FIRE ev={om_ev} comb={om_cb}"
        if k in want_omega and om_ev != want_omega[k]:
            b1_ok = False
            rep[f"B1_omega_k{k}"] = f"FIRE {om_ev} want 1"
        if k in (1, 3) and not (om_ev is not None and om_ev < 1):
            b1_ok = False
            rep[f"B1_strict_k{k}"] = f"FIRE {om_ev} not < 1"
        # B4 (occupied trajectory), unchanged
        q = central_parity_dist(M, s1)
        c = markov_route(mk, 1, k)
        delta = sum(abs(q.get(p, Fraction(0)) - c.get(p, Fraction(0)))
                    for p in (0, 1)) / 2
        if k in want_delta and delta != want_delta[k]:
            b4_ok = False
        print(f"    k={k}: omega={om_ev}  delta={delta}")
    rep["B1'"] = "PASS" if b1_ok else "FIRE"
    rep["B4"] = "PASS" if b4_ok else "FIRE"

    # B2' at k=2 (M11 fix)
    sup = M.vacuum_occ_superposed()
    st = M.step(M.step(sup))
    registry.append(("sup_T2", st))
    def coherence(state):
        re, im, _ = state
        half = M.M // 2
        tot = 0
        for m in range(half):
            a0 = re[m].astype(object); b0 = im[m].astype(object)
            a1 = re[m + half].astype(object); b1 = im[m + half].astype(object)
            tot += int(np.sum(a0 * a1 + b0 * b1))
        return tot
    c_before = coherence(st)
    after = M.u_geo_mat(st, M.plaq[M.p0])
    registry.append(("B2_after", after))
    ctrl_p = next(p for p in range(len(M.plaq)) if p != M.p0)
    after_c = M.u_geo_mat(st, M.plaq[ctrl_p])
    registry.append(("B2_ctrl", after_c))
    c_after = coherence(after)
    c_ctrl = coherence(after_c)
    rep["B2'"] = ("PASS" if (c_after != c_before and c_ctrl == c_before)
                  else f"FIRE {c_before}->{c_after} ctrl {c_ctrl}")

    # B3 full-scope Gauss (unchanged stake)
    bad = [nm for nm, stx in registry for held, _ in [M.gauss_holds(stx)]
           if not held]
    rep["B3"] = "PASS" if not bad else f"FIRE {bad}"

    # B6' two-route supports
    if tag == "base":
        cal = uncharged_support_enumerator(mk)
        rep["B6_cal"] = "PASS" if cal == 512 else f"FIRE cal {cal} vs 512"
        if with_spinors:
            en = charged_support_enumerator(mk)
            meas = int(np.count_nonzero(M.vacuum(0)[0][0]))
            rep["B6'"] = ("PASS" if en == meas
                          else f"FIRE enum {en} vs measured {meas}")
    print(f"[{tag}] " + "  ".join(f"{k}={v}" for k, v in sorted(rep.items())))
    return rep

def plants():
    from bridge1 import Model as Mo
    Mb = Mo(base_graph, with_spinors=False, broken_action=True)
    Mk = Mo(base_graph, with_spinors=False)
    held, why = Mk.gauss_holds(Mb.vacuum(0))
    p1 = not held
    print(f"[plant i] wrong-side action -> B3 {'FIRES' if p1 else 'MISSED'} ({why})")
    Mp = Mo(base_graph, with_spinors=False, broken_pump=True)
    held, why = Mp.gauss_holds(Mp.u_pump(Mp.vacuum(1)))
    p2 = not held
    print(f"[plant ii] non-central pump -> B3 {'FIRES' if p2 else 'MISSED'} ({why})")
    return p1 and p2

if __name__ == "__main__":
    r1 = run("base", base_graph, with_spinors=True, k_max=4)
    r2 = run("refined", refined_graph, with_spinors=False, k_max=2)
    b5 = all(r1[k] == "PASS" and r2[k] == "PASS" for k in ("B1'", "B4"))
    print(f"[B5' refinement] {'PASS' if b5 else 'FIRE'}")
    ok = plants()
    gates = all(v == "PASS" for r in (r1, r2) for v in r.values()) and b5
    print(f"\nVERDICT: gates {'ALL PASS' if gates else 'FIRED'}; "
          f"plants {'both FIRE' if ok else 'VOID - plant missed'}")
    sys.exit(0 if (gates and ok) else 1)
