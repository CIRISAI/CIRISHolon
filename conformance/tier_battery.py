#!/usr/bin/env python3
"""THE TIER VALIDATION BATTERY — standing machinery, born from Omega-KILL-3's
N3/N4 misses and their misfit pass. Runs on any tier_closure_probe output dir.

Arms (all bands frozen in OMEGA_TIER2_PREREG.md; unit discipline per D-UNITS,
frame-zero discipline per D-FRAME-ZERO):
  T-construction  PREMISE: meta.txt's PRE-step raw-moment L2 <= 1e-12 (else VOID)
  T-budget        median growth ratio of standardized 3-view coarse L2 over its
                  RISE EPOCH (1%-of-max onset to first frame >= 90% of max; the
                  plateau-including form is refused -- plateau domination, the
                  analyze_idjoin lesson) <= 1.05; rise < 20 frames => VOID
  T-levels        Spearman(div_v(f), P_v(f)) >= 0.8 at f in {300, 1200} over all
                  67 views — the conditioning-ceiling level law
                  (witness: sum_perturb_le gives the ceiling; the correlation is
                  the empirical law that realized divergences track it)
  T-organize      momx growth residual g/g_ceiling > random-view residual p75
  T-protect       ke growth residual < random-view residual p25
All divergences standardized by the run's own frozen scales; residual =
(median div 1200-2399 / median div 60-300) / (P(1200)/P(300))."""
import sys, csv, json, re
import numpy as np
from analyze_tier import load

M = (1 << 64) - 1
def _sm64(x):
    z = (x + 0x9E3779B97F4A7C15) & M
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & M
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & M
    return z ^ (z >> 31)

def weights():
    W = np.zeros((64, 9))
    for r in range(64):
        for j in range(9):
            key = ((r * 0x9E3779B97F4A7C15) ^ (j * 0xD1B54A32D192ED03)) & M
            W[r, j] = ((_sm64(key) >> 11) / (1 << 53)) * 2.0 - 1.0
        W[r] /= np.linalg.norm(W[r])
    return np.vstack([np.eye(9)[:3], W])          # rows 0-2 = declared directions

def spearman(x, y):
    rx = np.argsort(np.argsort(x)); ry = np.argsort(np.argsort(y))
    return float(np.corrcoef(rx, ry)[0, 1])

def adjudicate_arrays(views, P300, P1200, prestep_l2):
    """views: 67 x T standardized divergences (rows 0-2 declared momx/momy/ke)."""
    out = {}
    out["T_construction"] = {"prestep_l2": prestep_l2,
                             "premise_ok": bool(prestep_l2 <= 1e-12)}
    coarse = np.sqrt((views[:3] ** 2).sum(0))
    onset_i = int(np.argmax(coarse > 0.01 * coarse.max()))
    top_i = int(np.argmax(coarse >= 0.90 * coarse.max()))
    rise = (np.arange(len(coarse)) >= onset_i) & (np.arange(len(coarse)) <= top_i)
    n_rise = int(rise.sum())
    gr = coarse[1:][rise[1:]] / np.maximum(coarse[:-1][rise[1:]], 1e-300)
    K = float(np.median(gr)) if n_rise >= 20 else float("nan")
    out["T_budget"] = {"K": K, "rise_frames": n_rise,
                       "pass": None if n_rise < 20 else bool(K <= 1.05)}
    lv = {}
    for f, P in ((300, P300), (1200, P1200)):
        lvl = np.array([np.median(v[max(f - 40, 1):f + 40]) for v in views])
        lv[f] = spearman(lvl, P)
    out["T_levels"] = {"spearman": lv, "pass": bool(all(v >= 0.8 for v in lv.values()))}
    g = np.array([np.median(v[1200:2400]) / max(np.median(v[60:300]), 1e-300) for v in views])
    resid = g / (P1200 / P300)
    rr = resid[3:]
    out["T_organize"] = {"momx_resid": float(resid[0]), "rand_p75": float(np.percentile(rr, 75)),
                         "pass": bool(resid[0] > np.percentile(rr, 75))}
    out["T_protect"] = {"ke_resid": float(resid[2]), "rand_p25": float(np.percentile(rr, 25)),
                        "momy_resid": float(resid[1]),
                        "pass": bool(resid[2] < np.percentile(rr, 25))}
    return out

def adjudicate_dir(d):
    micro, decl, rand = load(f"{d}/swap.csv", f"{d}/swap_scales.txt")
    views = np.vstack([decl, rand])
    S = np.array([float(l.split('=')[1]) for l in open(f"{d}/swap_scales.txt")])
    A = {}
    for row in list(csv.reader(open(f"{d}/swap_coherence.csv")))[1:]:
        A.setdefault(int(row[0]), [0.0] * 9)[int(row[1])] = float(row[3])
    Wf = np.abs(weights())
    P300, P1200 = Wf @ (np.array(A[300]) / S), Wf @ (np.array(A[1200]) / S)
    pre = float(re.search(r"raw-moment L2 = (\S+)", open(f"{d}/meta.txt").read()).group(1))
    return adjudicate_arrays(views, P300, P1200, pre)

if __name__ == "__main__":
    print(json.dumps({d: adjudicate_dir(d) for d in sys.argv[1:]}, indent=2))
