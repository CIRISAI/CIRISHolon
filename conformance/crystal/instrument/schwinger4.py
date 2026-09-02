#!/usr/bin/env python3
"""SCHWINGER-4 — the residual interaction between two screened static pairs (GF0).

Prereg: conformance/crystal/SCHWINGER4_PREREG.md, frozen 2026-09-02 and committed
alone BEFORE this file existed. Node GF0 of the fold below the atom (OBJECT.md).

THE MODEL is the mirrored SCHWINGER-3 driver's, plus static unit charges entering as
the Gauss law's background flux: the Coulomb term becomes Σ_n (L_n + ε_n)² with
ε_n = Σ_{k≤n} Q^ext_k, and expanding the square adds ONE site-diagonal MPO entry,
c_k q_k with c_k = 2 Σ_{n=k}^{N−2} ε_n, plus a constant Σ ε_n². Nothing else changes.

THE DRIVER IS THE BANKED ONE, mechanically. `dmrg_schwinger.py` beside this file is
SCHWINGER-3's instrument, hashed in PROVENANCE.sha256, and its bytes are not touched.
This file does not copy its DMRG sweep by hand: at import it takes the SOURCE of the
banked `dmrg()`, verifies the mirror's sha256 against the pin, substitutes exactly the
one MPO-construction line (refusing if that line is not found exactly once), and
compiles the result as `dmrg_ext`. That is plant (iii) of the prereg in constructive
form — the extension cannot run on a driver that is not the banked one.

THE EXACT REFERee for plant (i) is SCHWINGER-1's `build()` (CIRISOntology
scratchpad/crystal/schwinger.py, mirrored here as `ed_build` with the same static term).

Modes:
    gauge                  plants (i) and (iii): ED vs DMRG at N=12, x=4; carrier nonzero
    pilot X N CHI          time one vacuum and one two-pair ground state (sizing only)
    staked4 X              the frozen grid for one column, checkpointed per configuration
    plant-coulomb-off      plant (ii): the x=4 column under the mutation; G1 must refuse (a)
    analyze                the gates, from checkpoints; prints the results-document numbers
"""
import hashlib
import inspect
import json
import os
import sys
import time
from itertools import combinations
from math import sqrt

import numpy as np
from scipy.sparse import lil_matrix
from scipy.sparse.linalg import LinearOperator, eigsh

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import dmrg_schwinger as ds  # noqa: E402  (the banked driver, untouched)
from dmrg_schwinger import I2, SM, SP, site_ops  # noqa: E402

# ------------------------------------------------------------------ the frozen grid

GRID = {
    9.0: {"N": 84, "s": 2, "d": [2, 3, 4, 6, 8, 10, 12, 14, 16, 20, 24], "chi64_at": [8, 12, 24],
          "M64": 0.646645, "flip": True},
    4.0: {"N": 56, "s": 2, "d": [2, 3, 4, 6, 8, 10, 12, 14, 16], "chi64_at": [6, 10, 16],
          "M64": 0.681667, "flip": False},
}
CHI = 40
FIT_DMIN = 8
NOISE_FLOOR = 1e-3
BAND = (0.8, 1.2)
R2_MIN = 0.99


def kappa_pred(x):
    return GRID[x]["M64"] / sqrt(x)


# ------------------------------------------------------------ plant (iii): the driver

def _pinned_sha():
    pin = open(os.path.join(HERE, "PROVENANCE.sha256")).read()
    for line in pin.splitlines():
        if "dmrg_schwinger.py" in line:
            return line.split()[0]
    raise SystemExit("REFUSED: PROVENANCE.sha256 carries no pin for dmrg_schwinger.py")


def _driver_gauge():
    have = hashlib.sha256(open(os.path.join(HERE, "dmrg_schwinger.py"), "rb").read()).hexdigest()
    want = _pinned_sha()
    if have != want:
        raise SystemExit(f"REFUSED: dmrg_schwinger.py sha256 {have[:12]} != pinned {want[:12]}; "
                         "the driver is not the banked SCHWINGER-3 instrument")
    src = inspect.getsource(ds.dmrg)
    head = "def dmrg(n, x, chi, n_sweeps=14, penalty=None, w_pen=None, mutate=None, seed=7,\n         init=None):"
    line = "    ws = mpo(n, x, mutate)\n"
    if src.count(head) != 1 or src.count(line) != 1:
        raise SystemExit("REFUSED: the banked dmrg() does not carry the one signature and the one "
                         "MPO line this extension substitutes; the driver moved")
    src2 = src.replace(head, "def dmrg_ext(n, x, chi, charges, n_sweeps=14, penalty=None, w_pen=None, "
                             "mutate=None, seed=7,\n         init=None):")
    src2 = src2.replace(line, "    ws = mpo_ext(n, x, charges, mutate)[0]\n")
    return have, src2


DRIVER_SHA, _DMRG_EXT_SRC = _driver_gauge()
_ns = dict(np=np, eigsh=eigsh, LinearOperator=LinearOperator)
_ns["mpo_ext"] = None  # bound below, after mpo_ext is defined
exec(compile(_DMRG_EXT_SRC, "<dmrg_ext: banked dmrg() with the MPO line substituted>", "exec"), _ns)


def dmrg_ext(n, x, chi, charges, **kw):
    _ns["mpo_ext"] = mpo_ext
    return _ns["dmrg_ext"](n, x, chi, charges, **kw)


# ------------------------------------------------------------------ the static term

def background_flux(n, charges):
    """ε_k = Σ_{j≤k} Q_j on link k (k = 0..n−2; the last entry is unused)."""
    eps = np.zeros(n)
    for k in range(n):
        eps[k] = sum(q for (s, q) in charges if s <= k)
    return eps


def site_potential(n, eps):
    """c_k = 2 Σ_{m=k}^{n−2} ε_m — the one diagonal entry the static charges add."""
    c = np.zeros(n)
    tail = 0.0
    for k in range(n - 1, -1, -1):
        if k <= n - 2:
            tail += eps[k]
        c[k] = 2.0 * tail
    return c


def mpo_ext(n, x, charges, mutate=None, lam=None):
    """The banked MPO (channels as in dmrg_schwinger.mpo) plus c_k q_k on the diagonal.

    `coulomb-off` is plant (ii): the dynamical Coulomb channel (3) and its diagonal
    a·q² are zeroed, leaving free staggered fermions in the static site potential.
    Returns (ws, constant) with constant = Σ ε_n², so energies are the true W."""
    if lam is None:
        lam = 20.0 * (x + 1.0)
    eps = background_flux(n, charges)
    c = site_potential(n, eps)
    const = float(np.sum(eps[: n - 1] ** 2))
    ws = []
    for l in range(n):
        a = float(n - 1 - l)
        q = site_ops(n, l)
        w = np.zeros((6, 6, 2, 2))
        w[0, 0] = I2
        w[0, 1] = SP
        w[1, 5] = x * SM
        w[0, 2] = SM
        w[2, 5] = x * SP
        if mutate == "coulomb-off":
            diag = 0.0 * q
        else:
            w[0, 3] = q
            w[3, 3] = I2
            w[3, 5] = 2.0 * a * q
            diag = a * (q @ q)
        w[0, 4] = q
        w[4, 4] = I2
        w[4, 5] = 2.0 * lam * q
        w[0, 5] = diag + lam * (q @ q) + c[l] * q
        w[5, 5] = I2
        ws.append(w)
    return ws, const


# ---------------------------------------------------------- plant (i): the ED referee

def ed_build(n, x, charges, mutate=None):
    """SCHWINGER-1's `build()` (CIRISOntology scratchpad/crystal/schwinger.py), with the
    static background flux inside the square. Half-filling (Q = 0) sector."""
    states = []
    for occ in combinations(range(n), n // 2):
        b = 0
        for k in occ:
            b |= 1 << k
        states.append(b)
    states.sort()
    index = {b: i for i, b in enumerate(states)}
    eps = background_flux(n, charges)
    H = lil_matrix((len(states), len(states)))
    for i, b in enumerate(states):
        diag = 0.0
        acc = 0.0
        for k in range(n - 1):
            sz = 2.0 * ((b >> k) & 1) - 1.0
            stag = 1.0 if k % 2 == 0 else -1.0
            acc += 0.5 * (sz + stag)
            l_val = acc + eps[k] if mutate != "coulomb-off" else 0.0
            diag += l_val * l_val
        if mutate == "coulomb-off":
            # the static site potential survives the mutation, as in the MPO
            c = site_potential(n, eps)
            for k in range(n):
                sz = 2.0 * ((b >> k) & 1) - 1.0
                stag = 1.0 if k % 2 == 0 else -1.0
                diag += c[k] * 0.5 * (sz + stag)
            diag += float(np.sum(eps[: n - 1] ** 2))
        H[i, i] = diag
        for k in range(n - 1):
            k2 = k + 1
            if ((b >> k) & 1) != ((b >> k2) & 1):
                j = index[b ^ (1 << k) ^ (1 << k2)]
                H[j, i] += x
    return H.tocsr()


def ed_energy(n, x, charges, mutate=None):
    H = ed_build(n, x, charges, mutate)
    vals = eigsh(H, k=1, which="SA")[0]
    return float(vals[0])


# ------------------------------------------------------------- configurations

def pair(p, s, sign=+1):
    return [(p, +1 * sign), (p + s, -1 * sign)]


def positions(n, s, d):
    """The first pair's site so the two-pair span (2s + d) is centred on the chain."""
    span = 2 * s + d
    p = (n - span) // 2
    return p, p + s + d


def energy(n, x, chi, charges, mutate=None, init=None):
    e, mps = dmrg_ext(n, x, chi, charges, mutate=mutate, init=init)
    const = mpo_ext(n, x, charges, mutate)[1]
    return float(e) + const, mps


# ----------------------------------------------------------------- checkpoints

def ckpt_path(tag):
    return os.path.join(HERE, "..", f"ckpt4_{tag}.npz")


def loadavg():
    try:
        return os.getloadavg()[0]
    except OSError:
        return float("nan")


def run_point(tag, n, x, chi, charges, mutate=None):
    ck = ckpt_path(tag)
    if os.path.exists(ck):
        z = np.load(ck, allow_pickle=True)
        e = float(z["e"])
        print(f"{tag:28s} E={e:.10f}  [checkpoint]", flush=True)
        return e
    t0 = time.time()
    e, _ = energy(n, x, chi, charges, mutate)
    np.savez(ck, e=e, charges=np.array(charges, dtype=object), n=n, x=x, chi=chi,
             mutate=str(mutate), seconds=time.time() - t0)
    print(f"{tag:28s} E={e:.10f}  {time.time() - t0:8.1f}s  loadavg {loadavg():.1f}", flush=True)
    return e


def column(x, mutate=None, chi=CHI, prefix=""):
    """Every configuration of one column at one χ; returns the energies keyed by label."""
    g = GRID[x]
    n, s = g["N"], g["s"]
    mut = f"-{mutate}" if mutate else ""
    key = lambda name: f"{prefix}x{x}_N{n}_chi{chi}{mut}_{name}"
    out = {}
    out["E0"] = run_point(key("E0"), n, x, chi, [], mutate)
    for d in g["d"]:
        p1, p2 = positions(n, s, d)
        for p in (p1, p2):
            lab = f"E1_p{p}"
            if lab not in out:
                out[lab] = run_point(key(lab), n, x, chi, pair(p, s), mutate)
        out[f"E2_d{d}"] = run_point(key(f"E2_d{d}"), n, x, chi, pair(p1, s) + pair(p2, s), mutate)
        if g["flip"] and d >= FIT_DMIN and mutate is None and chi == CHI:
            out[f"E2f_d{d}"] = run_point(key(f"E2f_d{d}"), n, x, chi, pair(p1, s) + pair(p2, s, -1), mutate)
    return out


def v_of_d(x, out, flipped=False):
    g = GRID[x]
    n, s = g["N"], g["s"]
    vs = {}
    for d in g["d"]:
        p1, p2 = positions(n, s, d)
        lab = f"E2f_d{d}" if flipped else f"E2_d{d}"
        if lab not in out:
            continue
        vs[d] = out[lab] - out[f"E1_p{p1}"] - out[f"E1_p{p2}"] + out["E0"]
    return vs


def fit(vs):
    """OLS of ln|V| on d over the window, above the stated floor. Returns (κ, R², npts)."""
    pts = [(d, abs(v)) for d, v in sorted(vs.items()) if d >= FIT_DMIN and abs(v) >= NOISE_FLOOR]
    if len(pts) < 5:
        return None, None, len(pts)
    ds_ = np.array([p[0] for p in pts], float)
    ys = np.log(np.array([p[1] for p in pts]))
    A = np.vstack([np.ones_like(ds_), ds_]).T
    coef, res, *_ = np.linalg.lstsq(A, ys, rcond=None)
    pred = A @ coef
    ss_res = float(np.sum((ys - pred) ** 2))
    ss_tot = float(np.sum((ys - ys.mean()) ** 2))
    r2 = 1.0 - ss_res / ss_tot if ss_tot > 0 else 0.0
    return -float(coef[1]), r2, len(pts)


def g1(x, kappa, r2, npts):
    kp = kappa_pred(x)
    if kappa is None or npts < 5:
        return "VOID(fewer than 5 fit points)"
    if r2 < R2_MIN:
        return f"VOID(R²={r2:.4f} < {R2_MIN})"
    ratio = kappa / kp
    if BAND[0] <= ratio <= BAND[1]:
        return "a"
    return "b" if ratio < BAND[0] else "c"


# ----------------------------------------------------------------------- modes

def mode_gauge():
    print(f"plant (iii): driver sha256 {DRIVER_SHA[:16]} matches PROVENANCE.sha256; one line substituted")
    n, x, chi = 12, 4.0, 48
    s = 2
    p1, p2 = 4, 8
    cfgs = {"E0": [], "E1": pair(p1, s), "E2(d=2)": pair(p1, s) + pair(p2, s)}
    worst = 0.0
    es = {}
    for name, ch in cfgs.items():
        e_ed = ed_energy(n, x, ch)
        e_dm, _ = energy(n, x, chi, ch)
        es[name] = e_dm
        diff = abs(e_ed - e_dm)
        worst = max(worst, diff)
        print(f"plant (i) {name:8s} ED {e_ed:.10f}  DMRG {e_dm:.10f}  |diff| {diff:.2e}")
    carrier = es["E1"] - es["E0"]
    print(f"carrier: E1 - E0 = {carrier:.6f}  ({'nonzero' if carrier > 0.1 else 'VACUOUS'})")
    ok = worst <= 1e-6 and carrier > 0.1
    print(f"gauge verdict: {'PASS' if ok else 'FAIL'} (worst |diff| {worst:.2e} vs 1e-6; carrier {carrier:.3f} vs 0.1)")
    # the mutation is a real defect: it must move the ED energy visibly in this sector
    e_mut = ed_energy(n, x, cfgs["E2(d=2)"], mutate="coulomb-off")
    print(f"plant (ii) carrier: E2 under coulomb-off {e_mut:.6f} vs {es['E2(d=2)']:.6f} "
          f"(|shift| {abs(e_mut - es['E2(d=2)']):.3f})")
    sys.exit(0 if ok else 1)


def mode_pilot(x, n, chi):
    s = 2
    p1, p2 = positions(n, s, 8)
    for name, ch in (("E0", []), ("E2(d=8)", pair(p1, s) + pair(p2, s))):
        t0 = time.time()
        e, _ = energy(n, x, chi, ch)
        print(f"pilot x={x} N={n} chi={chi} {name:8s} E={e:.8f}  {time.time() - t0:.1f}s  loadavg {loadavg():.1f}",
              flush=True)


def mode_staked(x):
    g = GRID[x]
    print(f"# SCHWINGER-4 column x={x} N={g['N']} chi={CHI}; driver sha256 {DRIVER_SHA[:16]}; "
          f"kappa_pred {kappa_pred(x):.6f}/site", flush=True)
    out40 = column(x, chi=CHI)
    # the screening premise: s=3 against s=2 at the first pair's own position
    n, s = g["N"], g["s"]
    p1, _ = positions(n, s, g["d"][0])
    e1_s3 = run_point(f"x{x}_N{n}_chi{CHI}_E1s3_p{p1}", n, x, CHI, pair(p1, 3))
    # chi = 64 checks at the staked d
    out64 = {"E0": run_point(f"x{x}_N{n}_chi64_E0", n, x, 64, [])}
    for d in g["chi64_at"]:
        q1, q2 = positions(n, s, d)
        for p in (q1, q2):
            lab = f"E1_p{p}"
            if lab not in out64:
                out64[lab] = run_point(f"x{x}_N{n}_chi64_{lab}", n, x, 64, pair(p, s))
        out64[f"E2_d{d}"] = run_point(f"x{x}_N{n}_chi64_E2_d{d}", n, x, 64, pair(q1, s) + pair(q2, s))
    json.dump({"x": x, "chi40": out40, "chi64": out64, "E1_s3": e1_s3, "driver_sha256": DRIVER_SHA},
              open(os.path.join(HERE, "..", f"schwinger4_x{x}.json"), "w"), indent=1)
    print("COLUMN_DONE", flush=True)


def mode_plant_coulomb_off():
    x = 4.0
    out = column(x, mutate="coulomb-off", chi=CHI)
    vs = v_of_d(x, out)
    k, r2, npts = fit(vs)
    verdict = g1(x, k, r2, npts)
    print(f"plant (ii) coulomb-off x={x}: V(2)={vs.get(2, float('nan')):.4e} "
          f"(carrier {'nonzero' if abs(vs.get(2, 0.0)) > 1e-3 else 'VACUOUS'}); "
          f"kappa_fit={k} R2={r2} n={npts} -> G1 reads {verdict}: "
          f"{'FIRES' if verdict != 'a' else 'MISSED'}", flush=True)
    json.dump({"vs": {str(d): v for d, v in vs.items()}, "kappa": k, "r2": r2, "g1": verdict},
              open(os.path.join(HERE, "..", "schwinger4_plant_coulomb_off.json"), "w"), indent=1)


def mode_analyze():
    report = {}
    for x in GRID:
        f = os.path.join(HERE, "..", f"schwinger4_x{x}.json")
        if not os.path.exists(f):
            print(f"x={x}: no column file yet")
            continue
        z = json.load(open(f))
        g = GRID[x]
        n, s = g["N"], g["s"]
        out40, out64 = z["chi40"], z["chi64"]
        vs = v_of_d(x, out40)
        # chi-premise
        voids = []
        for d in g["chi64_at"]:
            q1, q2 = positions(n, s, d)
            v64 = out64[f"E2_d{d}"] - out64[f"E1_p{q1}"] - out64[f"E1_p{q2}"] + out64["E0"]
            band = max(1e-4, 0.05 * abs(vs[d]))
            gap = abs(v64 - vs[d])
            print(f"x={x} chi-premise d={d}: V40={vs[d]:.6e} V64={v64:.6e} |gap|={gap:.2e} band={band:.2e} "
                  f"{'ok' if gap <= band else 'VOID'}")
            if gap > band:
                voids.append(d)
        # screening premise
        p1, _ = positions(n, s, g["d"][0])
        de2 = out40[f"E1_p{p1}"] - out40["E0"]
        de3 = z["E1_s3"] - out40["E0"]
        screened = abs(de3 - de2) < 0.5 * de2
        print(f"x={x} screening: E1(s=2)-E0={de2:.6f} E1(s=3)-E0={de3:.6f} -> {'screened' if screened else 'VOID (string)'}")
        for d in sorted(vs):
            print(f"x={x} d={d:2d} V={vs[d]:+.6e}")
        k, r2, npts = fit(vs)
        verdict = g1(x, k, r2, npts)
        if voids or not screened:
            verdict = "VOID(premise)"
        print(f"x={x}: kappa_fit={k} kappa_pred={kappa_pred(x):.6f} ratio={(k / kappa_pred(x)) if k else float('nan'):.4f} "
              f"R2={r2} npts={npts} -> G1 branch {verdict}")
        rep = {"kappa_fit": k, "kappa_pred": kappa_pred(x), "r2": r2, "npts": npts, "g1": verdict,
               "chi_voids": voids, "screened": screened, "V": {str(d): v for d, v in vs.items()}}
        if g["flip"]:
            vf = v_of_d(x, out40, flipped=True)
            kf, r2f, nf = fit(vf)
            signs_flip = all(np.sign(vf[d]) == -np.sign(vs[d]) for d in vf if d >= FIT_DMIN)
            print(f"x={x} G2 sign reading: every sign flips = {signs_flip}; kappa_flip={kf} (|Δ|/kappa_pred = "
                  f"{(abs(kf - k) / kappa_pred(x)) if (kf and k) else float('nan'):.4f})")
            rep["g2"] = {"all_signs_flip": bool(signs_flip), "kappa_flip": kf, "r2": r2f}
        report[str(x)] = rep
    json.dump(report, open(os.path.join(HERE, "..", "schwinger4_analysis.json"), "w"), indent=1)


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else ""
    if mode == "gauge":
        mode_gauge()
    elif mode == "pilot":
        mode_pilot(float(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]))
    elif mode == "staked4":
        mode_staked(float(sys.argv[2]))
    elif mode == "plant-coulomb-off":
        mode_plant_coulomb_off()
    elif mode == "analyze":
        mode_analyze()
    else:
        raise SystemExit(__doc__)
