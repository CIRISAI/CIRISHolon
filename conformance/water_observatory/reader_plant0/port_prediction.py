#!/usr/bin/env python3
"""READER-PLANT-0: the PREDICTION, from a python port of the record's relaxation reader.

What this is. A port of `fit_relaxation` and `align_cycles`
(engine/crates/holon-lens/src/field.rs) fed synthetic mode series. Its printed output,
saved beside it as `port_prediction.out`, is the PREDICTION that the Rust harness
(engine/crates/holon-lens/examples/reader_plant0.rs, calling the real functions) is graded
against. It reads no campaign walk and no banked series; every input is typed below and
cited in READER_PLANT0_PREREG_DRAFT.md.

Who wrote it. The classifier port (the window cut and the rebound rule, `label` below) is
the adversarial reviewer's scratch script of 2026-11-23 (the one behind SEAM_NS0_REVIEW.md
finding 2), kept line for line in `label_reviewer` and its loop in `reviewer_block`, which
reprints the reviewer's numbers with the reviewer's seed. Everything else (the port of the
Gauss-Newton exponential, of `align_cycles`, the further signal forms, the third noise arm,
path B) was added by the draft's author the same day, BEFORE any Rust was built or run.

Run: OMP_NUM_THREADS=1 python3 port_prediction.py > port_prediction.out   (numpy only)
"""
import numpy as np

DT = 10.0082e-15          # the readout cadence of the arms (replace0_response1*.sh)
RELAX = 314               # readouts per cycle
CYCLES = 12
TRIALS = 2000
G_U, W_U = 2.0e12, 8.8e12           # the planted underdamped form (the reviewer's rounding of the banked 200 m/s fits)
L1, L2 = 5.5e12, 1.04e13            # the record's two 50 m/s rates, as an overdamped second-order mode
L_SINGLE = 1.0e13                   # a single exponential near the banked 50 m/s current rate
G_E, W_E = 5.0e12, 5.9e12           # the EQUILIBRIUM law: a least-squares fit of exp(-Gt)(cos wt + G/w sin wt) to the banked density acf
                                    # (replace0/view_search_scout432_rigid.txt, mean of three seeds, ten lags; rms 0.023), read 2026-11-23
SNR_LOW = (6.4, 6.8, 9.4)           # banked aligned SNR, 50 m/s seeds 0, 2, 1
SNR_HIGH = (29.4, 38.4, 76.1)       # banked aligned SNR, 200 m/s seeds 2, 0, 1
T = np.arange(RELAX) * DT

def form(name):
    if name == 'U':   return np.exp(-G_U * T) * (np.cos(W_U * T) - G_U / W_U * np.sin(W_U * T))   # current after a velocity kick
    if name == 'E':   return np.exp(-G_E * T) * (np.cos(W_E * T) - G_E / W_E * np.sin(W_E * T))   # the current the equilibrium law implies (fluctuation-dissipation)
    if name == 'Uc':  return np.exp(-G_U * T) * np.cos(W_U * T)                                    # PR-2's own form
    if name == 'O2':  return (L2 * np.exp(-L2 * T) - L1 * np.exp(-L1 * T)) / (L2 - L1)             # overdamped second-order current
    if name == 'O1':  return np.exp(-L_SINGLE * T)                                                 # the reader's own notion (PR-1)
    if name == 'N':   return np.zeros(RELAX)
    raise ValueError(name)

# ---------------------------------------------------------------- the reader, ported
def fit_relaxation(y, dt, noise):
    """Returns (label, points, rate). label in OVER / UNDER / REF."""
    n = len(y); a0 = y[0]
    if abs(a0) < 3.0 * noise or n < 4: return 'REF', 0, np.nan
    dead = 2.0 * noise; cut = n
    below = np.abs(y) < dead
    for i in range(3, max(n - 4, 0)):
        if below[i:i + 5].all(): cut = i; break
    cut = min(max(cut, 4), n)
    seg = y[:cut]
    rebound = np.any((np.sign(seg) != np.sign(a0)) & (np.abs(seg) > max(4.0 * noise, 0.1 * abs(a0))))
    if rebound: return 'UNDER', cut, np.nan
    idx = np.nonzero(np.abs(seg) > 3.0 * noise)[0]
    if len(idx) < 3: return 'REF', cut, np.nan
    x = idx * dt; ly = np.log(np.abs(seg[idx])); m = len(idx)
    sx, sy, sxx, sxy = x.sum(), ly.sum(), (x * x).sum(), (x * ly).sum()
    lam = -(m * sxy - sx * sy) / (m * sxx - sx * sx)
    amp = np.exp((sy + lam * sx) / m) * np.sign(a0)
    ts = np.arange(cut) * dt
    for _ in range(30):
        e = np.exp(-lam * ts); r = seg - amp * e
        j0, j1 = e, -amp * ts * e
        a, b, c = (j0 * j0).sum(), (j0 * j1).sum(), (j1 * j1).sum()
        r0, r1 = (j0 * r).sum(), (j1 * r).sum()
        det = a * c - b * b
        if abs(det) < 1e-300: break
        da = (r0 * c - r1 * b) / det; dl = (a * r1 - b * r0) / det
        amp += da; lam += dl
        if abs(da) < 1e-12 * max(abs(amp), 1e-300) and abs(dl) < 1e-12 * max(abs(lam), 1e-300): break
    return 'OVER', cut, lam

def align_cycles(series, cycles, relax, first):
    series = series - series.mean()
    per, sds = [], []
    for c in range(cycles):
        s, e = first + c * relax, first + (c + 1) * relax
        if e > len(series) or relax < 8: break
        sds.append(series[e - relax // 4:e].std())
        per.append((1.0 if c % 2 == 0 else -1.0) * series[s:e])
    per = np.array(per); mean = per.mean(axis=0)
    return mean, mean[relax - relax // 4:].std(), float(np.mean(sds)), per

# ---------------------------------------------------------------- noise, unit SD
def noise_white(rng, shape): return rng.normal(0, 1, shape)
def noise_ar1(rng, shape, tau=50e-15):
    e = rng.normal(0, 1, shape); a = np.exp(-DT / tau); x = np.empty(shape); x[..., 0] = e[..., 0]
    for i in range(1, shape[-1]): x[..., i] = a * x[..., i - 1] + np.sqrt(1 - a * a) * e[..., i]
    return x
SUB = 16
def osc_velocity(rng, shape, two_gamma, w0sq):
    """Velocity of x'' + two_gamma x' + w0sq x = white force, sampled every DT (semi-implicit
    Euler, SUB substeps), the thermal noise a mode of that law carries; unnormalised."""
    h = DT / SUB; x = np.zeros(shape[:-1]); v = np.zeros(shape[:-1]); out = np.empty(shape)
    burn = 400 * SUB
    for k in range(burn + shape[-1] * SUB):
        v += h * (-two_gamma * v - w0sq * x) + np.sqrt(h) * rng.normal(0, 1, shape[:-1])
        x += h * v
        if k >= burn and (k - burn) % SUB == 0: out[..., (k - burn) // SUB] = v
    return out
_sd_cache = {}
def noise_fdt(rng, shape, name):
    if name in ('U', 'Uc'): tg, w0 = 2 * G_U, W_U * W_U + G_U * G_U
    elif name == 'E':       tg, w0 = 2 * G_E, W_E * W_E + G_E * G_E
    elif name == 'O2':      tg, w0 = L1 + L2, L1 * L2
    else: return noise_ar1(rng, shape, tau=1.0 / L_SINGLE) if name == 'O1' else noise_white(rng, shape)
    if name not in _sd_cache:
        _sd_cache[name] = osc_velocity(np.random.default_rng(20261123), (64, 4000), tg, w0).std()
    return osc_velocity(rng, shape, tg, w0) / _sd_cache[name]
def noise(rng, kind, shape, name):
    return noise_white(rng, shape) if kind == 'W' else noise_ar1(rng, shape) if kind == 'C' else noise_fdt(rng, shape, name)

# ---------------------------------------------------------------- the runs
def tally(labels, pts, rates):
    n = len(labels); o = labels.count('OVER'); u = labels.count('UNDER'); r = labels.count('REF')
    po = [p for l, p in zip(labels, pts) if l == 'OVER']; ro = [x for l, x in zip(labels, rates) if l == 'OVER']
    return o / n, u / n, r / n, (int(np.median(po)) if po else 0), (float(np.median(ro)) if ro else float('nan'))

def path_a(rng, name, kind, snr):
    y = form(name)[None, :] + noise(rng, kind, (TRIALS, RELAX), name) / snr
    res = [fit_relaxation(row, DT, 1.0 / snr) for row in y]
    return tally([r[0] for r in res], [r[1] for r in res], [r[2] for r in res])

def path_b(rng, name, kind, snr):
    """The reader as it ran: twelve cycles of alternating sign after a pre-kick row, aligned,
    the noise read by the reader from its own tails. Per-cycle noise is set so the EXPECTED
    aligned SNR is `snr` (sigma_cycle = sqrt(12)/snr for a unit kick)."""
    sig = form(name); sc = np.sqrt(CYCLES) / snr
    nz = noise(rng, kind, (TRIALS, 1 + CYCLES * RELAX), name) * sc
    lab, pts, rates, snrs, pc_over, pc_n = [], [], [], [], 0, 0
    for tr in range(TRIALS):
        s = nz[tr].copy()
        for c in range(CYCLES): s[1 + c * RELAX:1 + (c + 1) * RELAX] += (1.0 if c % 2 == 0 else -1.0) * sig
        mean, nse, nse_pc, per = align_cycles(s, CYCLES, RELAX, 1)
        l, p, r = fit_relaxation(mean, DT, nse); lab.append(l); pts.append(p); rates.append(r); snrs.append(abs(mean[0]) / nse)
        if tr < 200:
            for cyc in per:
                pc_n += 1; pc_over += fit_relaxation(cyc, DT, nse_pc)[0] == 'OVER'
    return tally(lab, pts, rates) + (float(np.median(snrs)), pc_over / max(pc_n, 1))

def label_reviewer(y, noise, n=RELAX):      # the reviewer's function, verbatim apart from the name
    a0 = y[0]
    if abs(a0) < 3 * noise: return 'REF', 0
    cut = n
    for i in range(3, n - 4):
        if np.all(np.abs(y[i:i + 5]) < 2 * noise): cut = i; break
    cut = max(cut, 4)
    reb = np.any((np.sign(y[:cut]) != np.sign(a0)) & (np.abs(y[:cut]) > max(4 * noise, 0.1 * abs(a0))))
    return ('UNDER' if reb else 'OVER'), cut

def reviewer_block():
    rng = np.random.default_rng(7); dt = 10e-15; n = 314; t = np.arange(n) * dt
    j = np.exp(-G_U * t) * (np.cos(W_U * t) - G_U / W_U * np.sin(W_U * t))
    def ar(sig, tau=0.0):
        e = rng.normal(0, 1, n)
        if tau > 0:
            a = np.exp(-dt / tau); x = np.zeros(n); x[0] = e[0]
            for i in range(1, n): x[i] = a * x[i - 1] + np.sqrt(1 - a * a) * e[i]
            e = x
        return sig * e
    for tau in (0.0, 50e-15):
        for snr in (74, 38, 20, 13, 9.4, 6.4):
            c = {'UNDER': 0, 'OVER': 0, 'REF': 0}; cuts = []
            for _ in range(2000):
                l, cu = label_reviewer(j + ar(1 / snr, tau), 1 / snr); c[l] += 1; cuts.append(cu)
            print(f"  noise corr {tau*1e15:.0f} fs  SNR {snr:5.1f}: {c}  median pts {int(np.median(cuts))}")

if __name__ == '__main__':
    print("READER-PLANT-0 port prediction; numpy", np.__version__, "; trials per cell", TRIALS)
    print("\n[0] noise-free landings (the label a clean series gets when the stated noise is 1/SNR)")
    for name in ('U', 'Uc', 'E', 'O2', 'O1'):
        print("  " + name + ": " + "  ".join(f"SNR {s}: {fit_relaxation(form(name), DT, 1.0/s)[0]}/{fit_relaxation(form(name), DT, 1.0/s)[1]}" for s in SNR_LOW + SNR_HIGH))
    f = form('U'); print(f"  E: first opposite lobe {form('E').min():+.4f} at {T[form('E').argmin()]*1e15:.0f} fs"); print(f"  U: first opposite lobe {f.min():+.4f} of the kick at {T[f.argmin()]*1e15:.0f} fs; O2: {form('O2').min():+.4f} at {T[form('O2').argmin()]*1e15:.0f} fs")
    print("\n[1] the reviewer's block, reprinted with the reviewer's seed (path A, known noise, dt = 10 fs)")
    reviewer_block()
    rng = np.random.default_rng(20261123)
    print("\n[2] path A (one 314-readout series, the noise passed to the reader as known): form noise SNR | p_over p_under p_ref | median pts | median rate /s")
    for kind in ('W', 'C'):
        for snr in SNR_LOW + SNR_HIGH:
            o, u, r, p, lam = path_a(rng, 'U', kind, snr); print(f"  U  {kind} {snr:5.1f} | {o:.3f} {u:.3f} {r:.3f} | {p:3d} | {lam:.3e}")
    print("\n[3] path B (twelve cycles aligned, the reader's own tail noise): form noise SNR | p_over p_under p_ref | median pts | median rate /s | median realised SNR | per-cycle p_over")
    table = {}; tableu = {}
    for name in ('U', 'Uc', 'E', 'O2', 'O1'):
        for kind in ('W', 'C', 'F'):
            for snr in SNR_LOW + SNR_HIGH:
                o, u, r, p, lam, rs, pc = path_b(rng, name, kind, snr); table[(name, kind, snr)] = o; tableu[(name, kind, snr)] = u
                print(f"  {name:2s} {kind} {snr:5.1f} | {o:.3f} {u:.3f} {r:.3f} | {p:3d} | {lam:.3e} | {rs:5.1f} | {pc:.3f}")
    for snr in SNR_LOW:
        o, u, r, p, lam, rs, pc = path_b(rng, 'N', 'W', snr); print(f"  N  W {snr:5.1f} | {o:.3f} {u:.3f} {r:.3f} | {p:3d} | {lam:.3e} | {rs:5.1f} | {pc:.3f}")
    print("\n[4] the graded quantity J = p_over(6.4) x p_over(6.8) x p_over(9.4), path B: the chance the banked triple of labels (three OVERDAMPED) arises from the planted form")
    for name in ('U', 'Uc', 'E', 'O2', 'O1'):
        print("  " + name + ": " + "  ".join(f"{kind}: {table[(name,kind,6.4)]*table[(name,kind,6.8)]*table[(name,kind,9.4)]:.3f}" for kind in ('W', 'C', 'F')))
    print("\n[5] K = p_under(29.4) x p_under(38.4) x p_under(76.1), path B: the chance the banked 200 m/s triple of labels (three UNDERDAMPED) arises from the planted form")
    for name in ('U', 'Uc', 'E', 'O2', 'O1'):
        print("  " + name + ": " + "  ".join(f"{kind}: {tableu[(name,kind,29.4)]*tableu[(name,kind,38.4)]*tableu[(name,kind,76.1)]:.3f}" for kind in ('W', 'C', 'F')))
    print("done.")
