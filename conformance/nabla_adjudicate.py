#!/usr/bin/env python3
"""NABLA adjudicator — the engine's dynamical connection, derived-then-measured.

Connection: mass-weighted transport of INTENSIVE fields (momx/mass, ke/mass)
between half-cell-shifted charts; loop = plaquette G -> +x -> +xy -> +y -> G.
Derived (exact for UNIFORM weights): loop operator = (I+Dx/4)(I+Dy/4), so
H_pred = (Dx+Dy)phi/4 + DxDy phi/16.  The MEASURED loop uses the scene's own
mass field as weights, so agreement is a law, not an identity.

Arms (bands in the prereg):
  V1 derived-holonomy law: Pearson(H_meas, H_pred) per field + slope band
  V2 state-dependence localizes: Spearman(|H_meas - H_pred|, mass contrast)
  V3 commutation with the step: Pearson(dH_meas, dH_pred) across sample pairs
Validity mask: all loop-source masses > 0 and source |phi| > 0 somewhere;
< 100 valid cell-samples => VOID."""
import sys, csv, json
import numpy as np

NX, NY = 32, 16

def load(d):
    F = {}
    for r in list(csv.reader(open(f"{d}/fields.csv")))[1:]:
        f, ch, c = int(r[0]), int(r[1]), int(r[2])
        F.setdefault(f, {}).setdefault(ch, {"mass": np.zeros(NX*NY), "momx": np.zeros(NX*NY), "ke": np.zeros(NX*NY)})
        F[f][ch]["mass"][c], F[f][ch]["momx"][c], F[f][ch]["ke"][c] = float(r[3]), float(r[4]), float(r[5])
    return F

def grids(F1):
    return [{k: v[k].reshape(NY, NX) for k in v} for ch, v in sorted(F1.items())]

def leg(phi, m, axis, forward):
    """Mass-weighted transport of intensive phi along a half-cell leg."""
    if axis == 'x':
        a, b = (slice(None), slice(0, -1)), (slice(None), slice(1, None))
    else:
        a, b = (slice(0, -1), slice(None)), (slice(1, None), slice(None))
    if not forward: a, b = b, a
    num = m[a]*phi[a] + m[b]*phi[b]; den = m[a] + m[b]
    out = np.full_like(phi, np.nan)
    tgt = (slice(None), slice(0, -1)) if axis == 'x' else (slice(0, -1), slice(None))
    if not forward: tgt = (slice(None), slice(1, None)) if axis == 'x' else (slice(1, None), slice(None))
    with np.errstate(invalid='ignore', divide='ignore'):
        out[tgt] = num / den
    return out

def loop_holonomy(g):
    """g: list of 4 chart dicts (mass/momx/ke as NY x NX). Returns per-field
    (H_meas, H_pred, phi0, contrast) on interior cells (nan elsewhere)."""
    out = {}
    for fld in ("momx", "ke"):
        with np.errstate(invalid='ignore', divide='ignore'):
            phi0 = g[0][fld] / g[0]["mass"]
        p = leg(phi0, g[0]["mass"], 'x', True)
        p = leg(p, g[1]["mass"], 'y', True)
        p = leg(p, g[2]["mass"], 'x', False)
        p = leg(p, g[3]["mass"], 'y', False)
        H = p - phi0
        f = np.nan_to_num(phi0, nan=0.0)
        Dx = np.full_like(f, np.nan); Dy = np.full_like(f, np.nan)
        Dx[:, 1:-1] = f[:, 2:] - 2*f[:, 1:-1] + f[:, :-2]
        Dy[1:-1, :] = f[2:, :] - 2*f[1:-1, :] + f[:-2, :]
        Dxy = np.full_like(f, np.nan)
        Dxy[1:-1, 1:-1] = (Dx[2:, 1:-1] - 2*Dx[1:-1, 1:-1] + Dx[:-2, 1:-1])
        Hp = Dx/4 + Dy/4 + Dxy/16
        m = g[0]["mass"]
        mn_i = np.minimum.reduce([m[:-2, :-2], m[:-2, 2:], m[2:, :-2], m[2:, 2:], m[1:-1, 1:-1]])
        mx_i = np.maximum.reduce([m[:-2, :-2], m[:-2, 2:], m[2:, :-2], m[2:, 2:], m[1:-1, 1:-1]])
        ctr = np.full_like(f, np.nan); mnf = np.full_like(f, np.nan)
        with np.errstate(invalid='ignore', divide='ignore'):
            ctr[1:-1, 1:-1] = (mx_i - mn_i) / (mx_i + mn_i)
        mnf[1:-1, 1:-1] = mn_i
        valid = (~np.isnan(H)) & (~np.isnan(Hp)) & (np.abs(f) > 0) & (mnf > 0)
        valid[0, :] = valid[-1, :] = False; valid[:, 0] = valid[:, -1] = False
        out[fld] = (H[valid], Hp[valid], ctr[valid], int(valid.sum()))
    return out

def spearman(x, y):
    rx = np.argsort(np.argsort(x)); ry = np.argsort(np.argsort(y))
    return float(np.corrcoef(rx, ry)[0, 1])

def adjudicate(F):
    frames = sorted(F)
    per = {"momx": [], "ke": []}
    for f in frames:
        h = loop_holonomy(grids(F[f]))
        for fld in per:
            per[fld].append(h[fld])
    out = {}
    for fld in per:
        Hm = np.concatenate([p[0] for p in per[fld]])
        Hp = np.concatenate([p[1] for p in per[fld]])
        ct = np.concatenate([p[2] for p in per[fld]])
        n = len(Hm)
        if n < 100:
            out[fld] = {"n": n, "pass": None}; continue
        r = float(np.corrcoef(Hm, Hp)[0, 1])
        slope = float(np.polyfit(Hp, Hm, 1)[0])
        dev = np.abs(Hm - Hp)
        sp = spearman(dev, ct)
        out[fld] = {"n": n, "V1_pearson": r, "V1_slope": slope,
                    "V1_pass": bool(r >= 0.9 and 0.8 <= slope <= 1.25),
                    "V2_spearman_dev_vs_contrast": sp, "V2_pass": bool(sp >= 0.5)}
    # V3: per-cell deltas between consecutive samples on the FULL grid
    d3 = {}
    for fld in ("momx", "ke"):
        dm, dp = [], []
        prevH = None
        for f in frames:
            g = grids(F[f])
            with np.errstate(invalid='ignore', divide='ignore'):
                phi0 = g[0][fld] / g[0]["mass"]
            p = leg(phi0, g[0]["mass"], 'x', True); p = leg(p, g[1]["mass"], 'y', True)
            p = leg(p, g[2]["mass"], 'x', False); p = leg(p, g[3]["mass"], 'y', False)
            H = p - phi0
            fz = np.nan_to_num(phi0, nan=0.0)
            Dx = np.full_like(fz, np.nan); Dy = np.full_like(fz, np.nan)
            Dx[:, 1:-1] = fz[:, 2:] - 2*fz[:, 1:-1] + fz[:, :-2]
            Dy[1:-1, :] = fz[2:, :] - 2*fz[1:-1, :] + fz[:-2, :]
            Dxy = np.full_like(fz, np.nan)
            Dxy[1:-1, 1:-1] = (Dx[2:, 1:-1] - 2*Dx[1:-1, 1:-1] + Dx[:-2, 1:-1])
            Hp = Dx/4 + Dy/4 + Dxy/16
            if prevH is not None:
                dHm, dHp = H - prevH[0], Hp - prevH[1]
                ok = (~np.isnan(dHm)) & (~np.isnan(dHp))
                dm.append(dHm[ok]); dp.append(dHp[ok])
            prevH = (H, Hp)
        dm, dp = np.concatenate(dm), np.concatenate(dp)
        live = np.abs(dp) > 0
        d3[fld] = {"n": int(live.sum()),
                   "V3_pearson": float(np.corrcoef(dm[live], dp[live])[0, 1]) if live.sum() >= 100 else None}
        d3[fld]["V3_pass"] = None if live.sum() < 100 else bool(d3[fld]["V3_pearson"] >= 0.8)
    return {"static": out, "delta": d3}

if __name__ == "__main__":
    print(json.dumps(adjudicate(load(sys.argv[1])), indent=2))
