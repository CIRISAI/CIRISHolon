#!/usr/bin/env python3
"""THE BANKED REPAIRS — standing analyzers for D-MATERIALIZE and D-CHAN-DRIFT.

1. threshold_onset: onset = first frame a response exceeds THRESH_FRAC of its own
   maximum (default 1%). Never a zero-baseline `>0` onset — materialization dust
   (ULP..1e-7 content changes under re-certification) sits below any physical
   response by orders of magnitude, and a relative threshold is scale-free.
2. oneway_premise: an arm premised on a one-way channel verifies the premise
   IN-JOB — the reverse reading must sit at ≤ PREMISE_K × its floor; else the
   arm is VOID-PREMISE (unposable this epoch), never adjudicated against bands.
"""
import csv, sys
THRESH_FRAC = 0.01
PREMISE_K = 10.0   # reverse ≤ 10× floor: "the realized channel is one-way here"

def threshold_onset(series, frames=None):
    mx = max(series)
    if mx <= 0: return None, mx
    for i, v in enumerate(series):
        if v > THRESH_FRAC * mx:
            return (frames[i] if frames else i), mx
    return None, mx

def analyze_sector_file(path, cols, probe_frame):
    rows = list(csv.reader(open(path)))[1:]
    frames = [int(r[0]) for r in rows]
    post = [i for i, f in enumerate(frames) if f >= probe_frame]
    out = {}
    for name, fn in cols.items():
        series = [fn(rows[i]) for i in post]
        onset, mx = threshold_onset(series, [frames[i] for i in post])
        out[name] = (onset, mx)
    return out

def oneway_premise(reverse, floor):
    return reverse <= PREMISE_K * max(floor, 1e-12)

if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "pernode":   # omega-1 arm_K: cols div_pos_l, div_pos_r
        r = analyze_sector_file(sys.argv[2],
            {"left": lambda x: float(x[3]), "right": lambda x: float(x[4])}, 240)
    else:                    # omega-2 arm_agg: |ke_b - ke_a| per sector
        r = analyze_sector_file(sys.argv[2],
            {"left": lambda x: abs(float(x[3]) - float(x[1])),
             "right": lambda x: abs(float(x[4]) - float(x[2]))}, 240)
    oL, oR = r["left"][0], r["right"][0]
    print(f"{sys.argv[2]}: left onset={oL}  right onset={oR}  gap={None if None in (oL,oR) else oR-oL}"
          f"  (1% of max; maxL={r['left'][1]:.3e} maxR={r['right'][1]:.3e})")
