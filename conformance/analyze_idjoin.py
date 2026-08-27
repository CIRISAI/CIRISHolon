#!/usr/bin/env python3
"""Frozen adjudicator for the S2 backbone (B1/B2/B3''/B4') on idjoin_probe CSVs.

B1: sham per-grain divergence exactly 0 at all frames, join always full.
B2: probed pair exactly 0 at all pre-probe frames (< 240).
B3'': per-sector onsets at 1% of that sector's max (onset_analyzer.threshold_onset);
      left onset < right onset and gap >= 10 frames.
B4': median per-frame growth ratio of total divergence (divL+divR) over the RISE
     EPOCH: frames >= 245 from the total's 1%-of-max onset to its first frame at
     >= 90% of max. A saturating expander reads 1.0 on any plateau-including
     window (the selftest's planted K=1.078 was MISSED by the masked-window form
     -- same plateau-domination mechanism as D-MATERIALIZE's dust, opposite end),
     so the plateau is excluded by construction. Rise epoch < 20 frames => VOID
     (unposable, dead-or-instant rise). Band: K <= 1.05.
Usage: analyze_idjoin.py <sham.csv> <probed.csv> | --selftest"""
import sys, csv, json
import numpy as np
from onset_analyzer import threshold_onset

def cols(path):
    rows = list(csv.reader(open(path))); hdr = rows[0]
    D = np.array([[float(x) for x in r] for r in rows[1:]])
    c = {h: i for i, h in enumerate(hdr)}
    return D, c

def adjudicate(sham, probed):
    Ds, cs = sham; Dp, cp = probed
    b1 = bool(np.all(Ds[:, cs['divL']] == 0) and np.all(Ds[:, cs['divR']] == 0)
              and np.all(Ds[:, cs['only_a']] == 0) and np.all(Ds[:, cs['only_b']] == 0))
    pre = Dp[Dp[:, cp['frame']] < 240]
    b2 = bool(np.all(pre[:, cp['divL']] == 0) and np.all(pre[:, cp['divR']] == 0))
    oL, _ = threshold_onset(Dp[:, cp['divL']]); oR, _ = threshold_onset(Dp[:, cp['divR']])
    gap = None if (oL is None or oR is None) else int(oR - oL)
    b3 = bool(oL is not None and oR is not None and oL < oR and gap >= 10)
    tot = Dp[:, cp['divL']] + Dp[:, cp['divR']]
    fr = Dp[:, cp['frame']]
    post = fr >= 245
    onset_i = np.argmax(post & (tot > 0.01 * tot.max()))
    top_i = np.argmax(post & (tot >= 0.90 * tot.max()))
    rise = np.arange(len(tot))
    rise = (rise >= onset_i) & (rise <= top_i)
    n_rise = int(rise.sum())
    g = tot[1:][rise[1:]] / np.maximum(tot[:-1][rise[1:]], 1e-300)
    K = float(np.median(g)) if n_rise >= 20 else float('nan')
    b4 = None if n_rise < 20 else bool(K <= 1.05)
    return {"B1_sham": b1, "B2_preprobe": b2,
            "B3_lightcone": {"pass": b3, "left": oL, "right": oR, "gap": gap},
            "B4_K": {"pass": b4, "K": K, "rise_frames": n_rise}}

def selftest():
    f = np.arange(2400)
    def mk(divL, divR, only=0):
        return (np.stack([f, divL, divR, np.full(2400, only), np.full(2400, only)], 1),
                {"frame":0, "divL":1, "divR":2, "only_a":3, "only_b":4})
    z = np.zeros(2400)
    rng = np.random.default_rng(1)
    dL = np.zeros(2400); dL[324:] = np.exp(0.01*np.arange(2400-324)); dL = np.minimum(dL, 50)
    dR = np.zeros(2400); dR[1047:] = np.exp(0.01*np.arange(2400-1047)); dR = np.minimum(dR, 50)
    r = adjudicate(mk(z, z), mk(dL, dR))
    ok = r["B1_sham"] and r["B2_preprobe"] and r["B3_lightcone"]["pass"] and r["B4_K"]["pass"] is True
    print(f"planted-truth (onsets 324/1047, saturating growth): {'PASS all four' if ok else 'BAND ERROR'} {r}")
    assert ok
    dLs = dL.copy(); dLs[100] = 1e-9              # pre-probe dust -> B2 fires
    r = adjudicate(mk(z, z), mk(dLs, dR)); print(f"planted pre-probe dust: {'FIRE B2' if not r['B2_preprobe'] else 'MISSED'}")
    assert not r["B2_preprobe"]
    dR2 = np.zeros(2400); dR2[330:] = np.exp(0.01*np.arange(2400-330)); dR2 = np.minimum(dR2, 50)
    r = adjudicate(mk(z, z), mk(dL, dR2)); print(f"planted gap=6: {'FIRE B3' if not r['B3_lightcone']['pass'] else 'MISSED'} gap={r['B3_lightcone']['gap']}")
    assert not r["B3_lightcone"]["pass"]
    dLe = np.zeros(2400); dLe[324:] = np.exp(0.075*np.arange(2400-324)); dLe = np.minimum(dLe, 1e60)
    r = adjudicate(mk(z, z), mk(dLe, dR)); print(f"planted K=1.078: {'FIRE B4' if not r['B4_K']['pass'] else 'MISSED'} K={r['B4_K']['K']:.4f}")
    assert not r["B4_K"]["pass"]
    zs = z.copy(); zs[7] = 5e-16
    r = adjudicate(mk(zs, z), mk(dL, dR)); print(f"planted sham ULP: {'FIRE B1' if not r['B1_sham'] else 'MISSED'}")
    assert not r["B1_sham"]
    instant = np.zeros(2400); instant[324:] = 50.0
    r = adjudicate(mk(z, z), mk(instant, z)); print(f"planted instant rise: {'VOID B4' if r['B4_K']['pass'] is None else 'MISSED'} rise_frames={r['B4_K']['rise_frames']}")
    assert r["B4_K"]["pass"] is None
    print("selftest verdict: all four arms PASS planted truth, FIRE their planted violations,")
    print("and B4 VOIDs on an unposable instant rise. Two-sided, prongs named.")

if __name__ == "__main__":
    if sys.argv[1] == "--selftest": selftest()
    else:
        r = adjudicate(cols(sys.argv[1]), cols(sys.argv[2]))
        print(json.dumps(r, indent=2))
