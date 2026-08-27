#!/usr/bin/env python3
"""Frozen adjudicator for N3 (tier closure within budget) and N4 (view privilege).

Operates on tier_closure_probe.rs output. All bands frozen in OMEGA_KILL3_PREREG.md.
Coarse divergence for N3 = standardized L2 of the three declared velocity-moment
views (momx/S0, momy/S1, ke/S2); comx is reported, unstaked. N4 compares per-view
growth ratios g_v = median(div_v, frames 1200-2399) / median(div_v, frames 60-300):
declared = the three pure conserved directions, control = the 64 random unit combos
in the same standardized moment space."""
import sys, csv
import numpy as np

def load(swap_csv, scales_txt):
    rows = list(csv.reader(open(swap_csv))); hdr = rows[0]
    D = np.array([[float(x) for x in r] for r in rows[1:]])
    col = {h: i for i, h in enumerate(hdr)}
    S = [float(l.split('=')[1]) for l in open(scales_txt)]
    micro = D[:, col['micro_div']]
    decl = np.stack([D[:, col['d_momx']]/S[0], D[:, col['d_momy']]/S[1], D[:, col['d_ke']]/S[2]])
    rand = np.stack([D[:, col[f'rand_{r:02d}']] for r in range(64)])
    return micro, decl, rand

def n3_adjudicate(micro, coarse):
    """coarse = standardized L2 of the 3 declared views, per frame."""
    i   = coarse[0] <= 1e-9 * micro.max()
    ii  = np.median(coarse[1200:] / np.maximum(micro[1200:], 1e-300)) <= 0.5
    m   = coarse > 0.01 * coarse.max()
    g   = coarse[1:][m[1:]] / np.maximum(coarse[:-1][m[1:]], 1e-300)
    iii = np.median(g) <= 1.05
    pose = np.median(micro[1200:]) >= 0.25 * np.median(micro[:60])   # sustained, else VOID
    return {"pass": bool(i and ii and iii), "posable": bool(pose),
            "conjuncts": (bool(i), bool(ii), bool(iii)),
            "coarse_over_micro": float(np.median(coarse[1200:]/np.maximum(micro[1200:],1e-300))),
            "growth_median": float(np.median(g))}

def n4_adjudicate(decl, rand):
    """Growth ratios; declared must sit in the closed tail of the random ensemble."""
    g = lambda v: np.median(v[1200:2400]) / max(np.median(v[60:300]), 1e-300)
    gd = np.array([g(v) for v in decl]); gr = np.array([g(v) for v in rand])
    passed = bool(np.all(gd < np.percentile(gr, 25)))
    fired  = bool(np.any(gd > np.median(gr)))
    return {"pass": passed, "fired": fired, "declared_ratios": [float(x) for x in gd],
            "random_pctiles": {p: float(np.percentile(gr, p)) for p in (5, 25, 50, 75, 95)}}

if __name__ == "__main__":
    micro, decl, rand = load(sys.argv[1], sys.argv[2])
    coarse = np.sqrt((decl**2).sum(0))
    import json
    print(json.dumps({"N3": n3_adjudicate(micro, coarse), "N4": n4_adjudicate(decl, rand)}, indent=2))
