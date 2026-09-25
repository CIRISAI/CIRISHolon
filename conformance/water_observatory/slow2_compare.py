#!/usr/bin/env python3
"""SLOW-2 (SLOW2_PREREG.md): SLOW-1's search against TICA, VAMP, VAMPnet and SPIB, and a VAMPnet trained
jointly on closure and the carried target (ours, integrated).

  slow2_compare.py features --walk DIR --out F.npz        # the per-walk cache (CPU, one thread)
  slow2_compare.py features --synthetic --out F.npz       # PS-3's carrier
  slow2_compare.py run --cache DIR [--out R.json]          # every arm, every plant, the verdict

The dictionary, the loaders, the ridge regression, the VAMP core and the synthetic are imported from
slow1_search.py and reason_search0b.py, not copied. Walks are read-only.
"""
import sys, os, math, json, argparse, time
import numpy as np
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, "..", "reasoning"))
import slow1_search as s1
from slow1_search import load, dictionary, ridge_r2, r2_fit, synthetic, STRUCT, HIST
from reason_search0b import vamp

DT_FS = 20.0
RD = lambda ps: int(round(ps * 1000 / DT_FS))
TAU = RD(1.0); H = RD(5.0); T0 = RD(10.0); STRIDE = 5
LAGS_PS = [1, 2, 5, 10, 20]
NNB = 12


# ---------------------------------------------------------------- inputs
def shell(P, L):
    """(T, n, 78): the 12 nearest-neighbour distances in ascending order and the 66 cosines between the unit
    vectors to neighbours j < k (distance order) - a permutation-invariant encoding of the neighbour set."""
    T, n, _ = P.shape; iu = np.triu_indices(NNB, 1); out = np.zeros((T, n, NNB + len(iu[0])), np.float32)
    for t in range(T):
        X = P[t] % L; d = X[None, :, :] - X[:, None, :]; d -= L * np.round(d / L)
        r = np.sqrt((d ** 2).sum(2)); np.fill_diagonal(r, np.inf)
        nn = np.argsort(r, axis=1)[:, :NNB]
        rr = np.take_along_axis(r, nn, axis=1); u = np.take_along_axis(d, nn[:, :, None], axis=1) / rr[:, :, None]
        c = np.einsum("ijk,ilk->ijl", u, u)
        out[t, :, :NNB] = rr; out[t, :, NNB:] = c[:, iu[0], iu[1]]
    return out


def features(P, V, L):
    hist_rd = [RD(h) for h in s1.HIST_PS]
    feats, xi, pk = dictionary(P, V, L, hist_rd)
    F19 = np.stack(feats, 1).astype(np.float32)          # (T, n, 19) in slow1's column order
    return dict(P=P.astype(np.float64), V=V.astype(np.float32), F19=F19, SH=shell(P, L), L=np.float64(L), xi=np.float64(xi))


# ---------------------------------------------------------------- per-walk views
class Walk:
    def __init__(self, d, name):
        self.name = name; self.P = d["P"]; self.V = d["V"]; self.F19 = d["F19"]; self.SH = d["SH"]; self.L = float(d["L"])
        self.planted = d["planted"] if "planted" in d else None
        self.T, self.n = self.P.shape[:2]
    def struct(self):
        S = self.F19[:, :, STRUCT]
        return S if self.planted is None else np.concatenate([S, self.planted[:, :, None]], 2)
    def rich(self):
        R = np.concatenate([self.SH, self.F19[:, :, STRUCT]], 2)
        return R if self.planted is None else np.concatenate([R, self.planted[:, :, None]], 2)
    def shuffled(self, seed=11):
        rng = np.random.default_rng(seed); w = Walk.__new__(Walk); w.__dict__.update(self.__dict__); w.name = self.name + " (time-shuffled)"
        perms = [rng.permutation(self.T) for _ in range(self.n)]
        take = lambda A: np.stack([A[perms[i], i] for i in range(self.n)], 1)
        w.P = take(self.P); w.V = take(self.V); w.F19 = take(self.F19); w.SH = take(self.SH)
        w.planted = take(self.planted) if self.planted is not None else None
        return w
    def sub(self, mols):
        w = Walk.__new__(Walk); w.__dict__.update(self.__dict__); w.P = self.P[:, mols]; w.V = self.V[:, mols]
        w.F19 = self.F19[:, mols]; w.SH = self.SH[:, mols]; w.planted = self.planted[:, mols] if self.planted is not None else None
        w.n = len(mols); return w


def lagpairs(X, lag):
    """X (T, n, k): the pooled (t, t + lag) pairs over molecules."""
    return X[:-lag].reshape(-1, X.shape[2]), X[lag:].reshape(-1, X.shape[2])


# ---------------------------------------------------------------- linear arms (A, B, C)
class Std:
    """TRAIN's per-column mean and standard deviation, accumulated in float64 over a list of (..., k) arrays."""
    def __init__(self, Xs):
        Xs = Xs if isinstance(Xs, list) else [Xs]; k = Xs[0].shape[-1]; n = 0; s1 = np.zeros(k); s2 = np.zeros(k)
        for X in Xs:
            Y = X.reshape(-1, k)
            for b in range(0, len(Y), 1 << 18):
                Yb = Y[b:b + (1 << 18)].astype(np.float64); n += len(Yb); s1 += Yb.sum(0); s2 += (Yb ** 2).sum(0)
        self.m = s1 / n; self.s = np.maximum(np.sqrt(np.maximum(s2 / n - self.m ** 2, 0)), 1e-9)
        self.m32 = self.m.astype(np.float32); self.s32 = self.s.astype(np.float32)
    def __call__(self, X): return (X.astype(np.float32) - self.m32) / self.s32


def arm_linear(kind, walks):
    """fit on the list of walks (their molecules pooled); return out(walk) -> (T, n, 2)."""
    Xs = [w.struct() for w in walks]; std = Std(Xs)
    if kind == "A":
        A = np.concatenate([lagpairs(x, TAU)[0] for x in Xs]); B = np.concatenate([lagpairs(x, TAU)[1] for x in Xs])
        m = vamp(A, B); W = m["W0"] @ m["U"][:, :2]
        return lambda w: (((w.struct() - m["mean"]) / m["scale"]) @ W).astype(np.float32)
    from deeptime.decomposition import TICA, VAMP
    trajs = [std(x[:, i]).astype(np.float64) for x in Xs for i in range(x.shape[1])]
    est = TICA(lagtime=TAU, dim=2) if kind == "B" else VAMP(lagtime=TAU, dim=2)
    model = est.fit(trajs).fetch_model()
    def out(w):
        X = std(w.struct()); T, n, k = X.shape
        return model.transform(X.reshape(-1, k).astype(np.float64)).reshape(T, n, -1)[:, :, :2].astype(np.float32)
    return out


# ---------------------------------------------------------------- nets (D, E, F)
def torch_setup(seed):
    os.environ.setdefault("CUBLAS_WORKSPACE_CONFIG", ":4096:8")
    import torch
    torch.manual_seed(seed); np.random.seed(seed); torch.use_deterministic_algorithms(True)
    return torch


def rows_for(walks, mols_of, std_rich):
    """training rows: t in [0, T - H) at stride STRIDE; x0 = rich(t), xt = rich(t + TAU), Z(t), y(t), molecule id."""
    X0, Xt, Z, Y, M = [], [], [], [], []; gid = 0
    for w in walks:
        R = std_rich(w.rich()); ts = np.arange(0, w.T - H, STRIDE)
        for i in mols_of(w):
            X0.append(R[ts, i]); Xt.append(R[ts + TAU, i]); v = w.V[ts, i]
            Z.append(np.column_stack([v, (v ** 2).sum(1)])); Y.append(np.linalg.norm(w.P[ts + H, i] - w.P[ts, i], axis=1))
            M.append(np.full(len(ts), gid + i))
        gid += w.n
    c = lambda L: np.concatenate(L).astype(np.float32)
    return c(X0), c(Xt), c(Z), c(Y), np.concatenate(M)


def lobe(torch, k_in):
    nn = torch.nn
    return nn.Sequential(nn.Linear(k_in, 128), nn.ReLU(), nn.Linear(128, 128), nn.ReLU(), nn.Linear(128, 2))


def ridge_r2_torch(torch, Xa, ya, Xb, yb):
    m = Xa.mean(0); s = Xa.std(0) + 1e-6; A = (Xa - m) / s; B = (Xb - m) / s
    W = torch.linalg.solve(A.T @ A + 1e-3 * len(A) * torch.eye(A.shape[1], device=A.device), A.T @ (ya - ya.mean()))
    pred = B @ W + ya.mean(); return 1 - ((yb - pred) ** 2).sum() / ((yb - yb.mean()) ** 2).sum()


def carried_in_batch(torch, f, Z, y, mid):
    """held-out (by molecule parity) R^2(y | Z, f) - R^2(y | Z), averaged over the two directions."""
    ev = (mid % 2) == 0; od = ~ev; out = 0.0
    for a, b in ((ev, od), (od, ev)):
        Xf = torch.cat([Z, f], 1)
        out = out + ridge_r2_torch(torch, Xf[a], y[a], Xf[b], y[b]) - ridge_r2_torch(torch, Z[a], y[a], Z[b], y[b]).detach()
    return out / 2


def train_vampnet(train_rows, k_in, seed, lam, device, epochs=20, batch=2048, lr=5e-4, use_deeptime_fit=False, log=None):
    """D (lam = 0 through deeptime's VAMPNet.partial_fit) and F (lam >= 0 through our step with deeptime's vamp_score)."""
    torch = torch_setup(seed)
    from deeptime.decomposition.deep import VAMPNet, vamp_score
    X0, Xt, Z, Y, M = [torch.as_tensor(a).to(device) for a in train_rows]
    Y = (Y - Y.mean()) / Y.std(); Mi = torch.as_tensor(train_rows[4]).to(device)
    net = lobe(torch, k_in).to(device)
    g = torch.Generator(device="cpu"); g.manual_seed(seed)
    if use_deeptime_fit:
        vn = VAMPNet(net, device=device, learning_rate=lr, score_method="VAMP2", score_mode="regularize", epsilon=1e-6)
    else:
        opt = torch.optim.Adam(net.parameters(), lr=lr)
    hist = []
    for ep in range(epochs):
        perm = torch.randperm(len(X0), generator=g).to(device); sc = []; cr = []
        for b in range(0, len(X0), batch):
            idx = perm[b:b + batch]
            if len(idx) < 64: continue
            if use_deeptime_fit:
                vn.partial_fit((X0[idx], Xt[idx])); continue
            net.train(); f0 = net(X0[idx]); ft = net(Xt[idx])
            vs = vamp_score(f0, ft, method="VAMP2", epsilon=1e-6, mode="regularize")
            loss = -vs
            if lam:
                dr = carried_in_batch(torch, f0, Z[idx], Y[idx], Mi[idx]); loss = loss - lam * dr; cr.append(float(dr.detach()))
            opt.zero_grad(); loss.backward(); opt.step(); sc.append(float(vs.detach()))
        if not use_deeptime_fit: hist.append((ep, float(np.mean(sc)), float(np.mean(cr)) if cr else None))
    if use_deeptime_fit:
        tr_ = np.asarray(getattr(vn, 'train_scores', [])); hist = [(len(tr_), float(tr_[-1][1]) if len(tr_) else None, None)]
    net.eval()
    def out(w, std_rich):
        with torch.no_grad():
            R = std_rich(w.rich()); T, n, k = R.shape; o = []
            flat = torch.as_tensor(R.reshape(-1, k))
            for b in range(0, len(flat), 1 << 16): o.append(net(flat[b:b + (1 << 16)].to(device)).cpu().numpy())
            return np.concatenate(o).reshape(T, n, 2)
    return out, hist


def train_spib(walks, mols_fit, mols_val, std_rich, seed, device, workdir, beta=0.01, K=10):
    torch = torch_setup(seed)
    from spib.spib import SPIB
    from spib.utils import TimeLaggedDataset, DataNormalize
    from deeptime.clustering import KMeans
    trajs_fit, trajs_val = [], []
    for w in walks:
        R = w.rich()
        trajs_fit += [R[:w.T - H + TAU, i].astype(np.float32) for i in mols_fit(w)]
        trajs_val += [R[:w.T - H + TAU, i].astype(np.float32) for i in mols_val(w)]
    allfit = np.concatenate(trajs_fit); mean, sd = std_rich.m32, std_rich.s32
    rng = np.random.default_rng(seed); sub = allfit[rng.choice(len(allfit), min(100000, len(allfit)), replace=False)]
    km = KMeans(n_clusters=K, fixed_seed=seed, max_iter=200).fit(((sub - mean) / sd).astype(np.float64)).fetch_model()
    lab = lambda X: km.transform(((X - mean) / sd).astype(np.float64)).astype(np.int64)
    Lf = [lab(x) for x in trajs_fit]; Lv = [lab(x) for x in trajs_val]
    ds_f = TimeLaggedDataset(trajs_fit, Lf, None, lagtime=TAU, subsampling_timestep=STRIDE, output_dim=K, device=device)
    ds_v = TimeLaggedDataset(trajs_val, Lv, None, lagtime=TAU, subsampling_timestep=STRIDE, output_dim=K, device=device)
    os.makedirs(workdir, exist_ok=True)
    model = SPIB(K, (allfit.shape[1],), encoder_type="Nonlinear", z_dim=2, lagtime=TAU, beta=beta, learning_rate=1e-3,
                 device=device, path=os.path.join(workdir, f"spib_s{seed}"), UpdateLabel=True, neuron_num1=128, neuron_num2=128,
                 data_transform=DataNormalize(mean, sd)).to(device)
    import contextlib, io
    buf = io.StringIO()
    collapsed = False
    with contextlib.redirect_stdout(buf):
        try:   # a fixed schedule: refine every 5 epochs, 8 refinements, 45 epochs (Notes on building)
            model.fit(ds_f, ds_v, batch_size=2048, tolerance=float("inf"), patience=4, refinements=8, index=seed)
        except ValueError:   # the package raises when its labels collapse to one state; the encoder as trained is read (Notes on building)
            collapsed = True; model.eval()
    lines = buf.getvalue().splitlines(); tail = [l for l in lines if l.strip()][-4:]
    n_states = int(model.output_dim)
    def out(w, _std_rich=None):
        R = w.rich(); T, n, k = R.shape
        _, _, zm, _ = model.transform(R.reshape(-1, k).astype(np.float32), batch_size=1 << 15, to_numpy=True)
        return zm.reshape(T, n, 2).astype(np.float32)
    return out, dict(final_states=n_states, collapsed=collapsed, refinements_done=len(model.convergence_history), epochs=sum(1 for l in lines if l.startswith('Epoch')), log_tail=tail)


# ---------------------------------------------------------------- the common read
def f1_map(train_outs):
    """the top singular function of the linear VAMP on an arm's 2-D output at tau, fitted on TRAIN."""
    A = np.concatenate([lagpairs(o, TAU)[0] for o in train_outs]); B = np.concatenate([lagpairs(o, TAU)[1] for o in train_outs])
    m = vamp(A, B); w = m["W0"] @ m["U"][:, 0]
    return lambda o: ((o - m["mean"]) / m["scale"]) @ w


def cubic_r2(f, y):
    return r2_fit(np.column_stack([f, f ** 2, f ** 3]), y)


def read(w, o, f1, full=True):
    """closure, the carried statistics, loadings and PS-1/PS-2 of an arm's output o (T, n, 2) on walk w."""
    res = {}
    res["sigma1"] = {ps: float(vamp(*lagpairs(o, RD(ps)))["s"][0]) for ps in LAGS_PS if RD(ps) < w.T // 2}
    if not full: return res
    f = f1(o); T, n = w.T, w.n; rows = slice(T0, T - H)
    y = [np.linalg.norm(w.P[T0 + H:, i] - w.P[T0:T - H, i], axis=1) for i in range(n)]
    v = w.V[rows]; Zc = [np.column_stack([v[:, i], (v[:, i] ** 2).sum(1)]) for i in range(n)]
    S = w.F19[rows][:, :, STRUCT]; Hh = w.F19[rows][:, :, HIST]; Oc = o[rows]; fc = f[rows]; q = w.F19[rows][:, :, [0]]
    R84 = None
    folds = [[i for i in range(n) if i % 4 == j] for j in range(4)]
    def R2(blocks_of, perm=None):
        """blocks_of: list of per-molecule (rows, k) arrays or 'Z'; perm maps target molecule to donor for the non-Z blocks."""
        perm = perm or list(range(n)); sc = []
        for te in folds:
            tr = [i for i in range(n) if i not in te]
            X = lambda idx: np.column_stack([np.concatenate([Zc[i] for i in idx])] + [np.concatenate([b[perm[i]] for i in idx]) for b in blocks_of])
            sc.append(ridge_r2(X(tr), np.concatenate([y[i] for i in tr]), X(te), np.concatenate([y[i] for i in te])))
        return float(np.mean(sc))
    per = lambda A: [A[:, i] for i in range(n)]
    Qb, Sb, Hb, Ob, fb = per(q), per(S), per(Hh), per(Oc), [fc[:, i:i + 1] for i in range(n)]
    base = dict(Z=R2([]), Zq=R2([Qb]), ZS=R2([Sb]), ZSH=R2([Sb, Hb]))
    res["R2"] = base
    res["q_carry"] = base["Zq"] - base["Z"]
    res["inc_Z"] = R2([Ob]) - base["Z"]; res["inc_q"] = R2([Qb, Ob]) - base["Zq"]
    res["inc_S"] = R2([Sb, Ob]) - base["ZS"]; res["inc_SH"] = R2([Sb, Hb, Ob]) - base["ZSH"]
    res["f1_inc_Z"] = R2([fb]) - base["Z"]; res["f1_inc_q"] = R2([Qb, fb]) - base["Zq"]
    der = [(i + n // 2) % n for i in range(n)]
    res["PS2"] = R2([Ob], perm=der) - base["Z"]
    F = fc.reshape(-1); cols = w.F19[rows].reshape(-1, 19)
    load = {"q": [0], "s2": [17], "s2w": [18], "density": [1, 2], "nb": [3], "h1": [4], "h2": [5], "h5": [6], "h10": [7], "STRUCT6": STRUCT}
    res["loading"] = {k: r2_fit(cols[:, c], F) for k, c in load.items()}
    vv = v.reshape(-1, 3); res["loading"]["MOM"] = r2_fit(np.column_stack([vv, (vv ** 2).sum(1)]), F)
    res["PS1_f1_on_MOM"] = res["loading"]["MOM"]
    # the direction of f1 in STRUCT's standardised coordinates (linear arms are exactly linear there)
    Sz = (cols[:, STRUCT] - cols[:, STRUCT].mean(0)) / np.maximum(cols[:, STRUCT].std(0), 1e-9)
    res["dir_struct"] = np.linalg.lstsq(np.column_stack([Sz, np.ones(len(Sz))]), F, rcond=None)[0][:6].tolist()
    return res


def r0(w):
    """reference R0: the 84 rich columns directly in the ridge, beyond velocity and q (no dynamics, no stake)."""
    T, n = w.T, w.n; rows = slice(T0, T - H); R = w.rich()[rows]
    y = [np.linalg.norm(w.P[T0 + H:, i] - w.P[T0:T - H, i], axis=1) for i in range(n)]
    v = w.V[rows]; folds = [[i for i in range(n) if i % 4 == j] for j in range(4)]
    def R2(extra):
        sc = []
        for te in folds:
            tr = [i for i in range(n) if i not in te]
            X = lambda idx: np.concatenate([np.column_stack([v[:, i], (v[:, i] ** 2).sum(1), w.F19[rows][:, i, [0]]] + ([R[:, i]] if extra else [])) for i in idx])
            sc.append(ridge_r2(X(tr), np.concatenate([y[i] for i in tr]), X(te), np.concatenate([y[i] for i in te])))
        return float(np.mean(sc))
    return R2(True) - R2(False)


# ---------------------------------------------------------------- the campaign
def run(cache, outp, device, lam_stake=10.0, quick=False):
    t_start = time.time()
    ld = lambda k: Walk(dict(np.load(os.path.join(cache, f"{k}.npz"))), k)
    tr = [ld("T293_seed0"), ld("T293_seed1")]; ho = ld("T293_seed2"); c4 = ld("T400_seed0")
    syn = ld("synthetic")
    if quick:   # a code-path exercise, NOT A READING
        tr = [w.sub(list(range(40))) for w in tr]; ho = ho.sub(list(range(40))); c4 = c4.sub(list(range(40)))
    fit_m = lambda w: [i for i in range(w.n) if i % 5 != 0]; val_m = lambda w: [i for i in range(w.n) if i % 5 == 0]
    std_rich = Std([w.rich() for w in tr])
    R = dict(meta=dict(device=device, tau_rd=TAU, horizon_rd=H, t0_rd=T0, stride=STRIDE, lam_stake=lam_stake, quick=quick,
                       n_train=[w.n for w in tr], T=ho.T), arms={}, plants={})
    def say(*a):
        print(*a, flush=True)
    # PS-1 (data): MOM own sigma1 at 1 ps
    for w in [ho, c4]:
        R["plants"].setdefault("PS1_MOM_sigma1", {})[w.name] = float(vamp(*lagpairs(w.F19[:, :, 8:11], TAU))["s"][0])
    say(f"# PS-1 data: MOM own sigma1 at 1 ps {R['plants']['PS1_MOM_sigma1']}")
    R["R0"] = {w.name: r0(w) for w in [ho, c4]}
    say(f"# R0 (84 rich columns in the ridge, beyond velocity and q; no stake): {R['R0']}")

    def evaluate(name, outs_fn, seed=None, extra=None):
        train_outs = [outs_fn(w) for w in tr]; f1 = f1_map(train_outs)
        r = {w.name: read(w, outs_fn(w), f1) for w in [ho, c4]}
        if extra: r.update(extra)
        R["arms"].setdefault(name, {})[str(seed)] = r
        h = r[ho.name]; c = r[c4.name]
        say(f"  {name} seed {seed}: HELD-OUT sigma1(1ps) {h['sigma1'][1]:.3f} inc_Z {h['inc_Z']:+.4f} inc_q {h['inc_q']:+.4f} inc_S {h['inc_S']:+.4f} "
            f"inc_SH {h['inc_SH']:+.4f} | load q {h['loading']['q']:.3f} STRUCT6 {h['loading']['STRUCT6']:.3f} MOM {h['loading']['MOM']:.4f} | PS2 {h['PS2']:+.4f} "
            f"|| 400K sigma1 {c['sigma1'][1]:.3f} inc_q {c['inc_q']:+.4f} inc_Z {c['inc_Z']:+.4f} PS2 {c['PS2']:+.4f} load q {c['loading']['q']:.3f} "
            f"[{time.time() - t_start:.0f} s]")
        return r

    # ---- plants PS-3 and PS-4 per arm, built first
    sy_fit = syn.sub([i for i in range(syn.n) if i % 4 != 0]); sy_te = syn.sub([i for i in range(syn.n) if i % 4 == 0])
    std_syn = Std(sy_fit.rich())
    def ps3(name, out_fn):
        fo = out_fn(sy_fit); f1 = f1_map([fo]); f = f1(out_fn(sy_te)).reshape(-1); p = sy_te.planted.reshape(-1)
        r = dict(cubic=cubic_r2(f, p), linear=r2_fit(f[:, None], p)); R["plants"].setdefault("PS3", {})[name] = r
        say(f"  PS-3 {name}: planted on (f1, f1^2, f1^3) R^2 {r['cubic']:.3f} (linear {r['linear']:.3f}) -> {'PASS' if r['cubic'] >= 0.9 else 'FAIL'}")
    sh_tr = [w.shuffled() for w in tr]; sh_ho = ho.shuffled()
    std_sh = Std([w.rich() for w in sh_tr])
    def ps4(name, out_fn):
        s = read(sh_ho, out_fn(sh_ho), None, full=False)["sigma1"]; mx = max(s.values())
        R["plants"].setdefault("PS4", {})[name] = dict(sigma1=s, max=mx)
        say(f"  PS-4 {name}: time-shuffled own sigma1 {{{', '.join(f'{k}: {v:.3f}' for k, v in s.items())}}} max {mx:.3f} -> {'PASS' if mx < 0.2 else 'FAIL'}")

    say("# LINEAR ARMS A, B, C (fitted on TRAIN, frozen)")
    for k, nm in (("A", "A SLOW-1 search"), ("B", "B TICA"), ("C", "C VAMP")):
        fn = arm_linear(k, tr); evaluate(nm, fn, seed=None)
        ps3(nm, arm_linear(k, [sy_fit])); ps4(nm, arm_linear(k, sh_tr))
        fn4 = arm_linear(k, [c4]); f1_4 = f1_map([fn4(c4)])
        R["arms"][nm]["refit400"] = read(c4, fn4(c4), f1_4)
        say(f"  {nm} refitted on 400 K itself (no stake): sigma1 {R['arms'][nm]['refit400']['sigma1'][1]:.3f} inc_q {R['arms'][nm]['refit400']['inc_q']:+.4f} load q {R['arms'][nm]['refit400']['loading']['q']:.3f}")
    rows_tr = rows_for(tr, fit_m, std_rich); k_in = rows_tr[0].shape[1]
    rows_sy = rows_for([sy_fit], lambda w: range(w.n), std_syn)
    rows_sh = rows_for(sh_tr, fit_m, std_sh)
    seeds = [0, 1, 2]
    wd = os.path.join(os.path.dirname(os.path.abspath(outp)), "spib_work")
    ep = 3 if quick else 20
    say(f"# NETS on {device}: {len(rows_tr[0])} fitting rows x {k_in} inputs; {ep} epochs")
    for s in seeds:
        fn, hist = train_vampnet(rows_tr, k_in, s, 0.0, device, epochs=ep, use_deeptime_fit=True)
        evaluate("D VAMPnet", lambda w: fn(w, std_rich), seed=s, extra=dict(train_log=hist))
    for s in seeds:
        fn, info = train_spib(tr, fit_m, val_m, std_rich, s, device, wd)
        evaluate("E SPIB", fn, seed=s, extra=dict(train_log=info)); say(f"    SPIB seed {s}: final states {info['final_states']}")
    for lam in [lam_stake, 0.0, 1.0, 100.0]:
        nm = f"F integrated lam={lam:g}" + (" (STAKE)" if lam == lam_stake else "")
        for s in seeds:
            fn, hist = train_vampnet(rows_tr, k_in, s, lam, device, epochs=ep)
            evaluate(nm, lambda w: fn(w, std_rich), seed=s, extra=dict(train_log=hist))
    say("# PLANTS PS-3 and PS-4 for the nets (training seed 0)")
    fnD, _ = train_vampnet(rows_sy, rows_sy[0].shape[1], 0, 0.0, device, epochs=ep, use_deeptime_fit=True); ps3("D VAMPnet", lambda w: fnD(w, std_syn))
    fnE, _ = train_spib([sy_fit], lambda w: [i for i in range(w.n) if i % 5 != 0], lambda w: [i for i in range(w.n) if i % 5 == 0], std_syn, 0, device, wd + "_ps3")
    ps3("E SPIB", fnE)
    fnF, _ = train_vampnet(rows_sy, rows_sy[0].shape[1], 0, lam_stake, device, epochs=ep); ps3("F integrated", lambda w: fnF(w, std_syn))
    fnD, _ = train_vampnet(rows_sh, k_in, 0, 0.0, device, epochs=ep, use_deeptime_fit=True); ps4("D VAMPnet", lambda w: fnD(w, std_sh))
    fnE, _ = train_spib(sh_tr, fit_m, val_m, std_sh, 0, device, wd + "_ps4"); ps4("E SPIB", fnE)
    fnF, _ = train_vampnet(rows_sh, k_in, 0, lam_stake, device, epochs=ep); ps4("F integrated", lambda w: fnF(w, std_sh))
    R["meta"]["seconds"] = time.time() - t_start
    json.dump(R, open(outp, "w"), indent=1, default=lambda x: float(x) if isinstance(x, (np.floating,)) else str(x))
    verdict(R, ho.name, c4.name)
    return R


def verdict(R, HO, C4):
    A = R["arms"]; P = R["plants"]
    def agg(nm, key, walk):
        v = [r[walk][key] for s, r in A[nm].items() if s != "refit400"]; return float(np.mean(v)), float(min(v)), float(max(v))
    def plants_ok(nm):
        fails = []
        for s, r in A[nm].items():
            if s == "refit400": continue
            for wk in (HO, C4):
                if not r[wk]["PS1_f1_on_MOM"] < 0.05: fails.append(f"PS-1 (f1 on MOM {r[wk]['PS1_f1_on_MOM']:.3f}) seed {s} {wk}")
                if not r[wk]["PS2"] <= 0.01: fails.append(f"PS-2 ({r[wk]['PS2']:+.4f}) seed {s} {wk}")
        base = nm.split(" lam")[0].replace(" (STAKE)", "")
        key = "F integrated" if base.startswith("F integrated") else nm   # PS-3/PS-4 are run on the STAKE lambda only; other lambdas inherit that row
        if key in P.get("PS3", {}) and not P["PS3"][key]["cubic"] >= 0.9: fails.append(f"PS-3 ({P['PS3'][key]['cubic']:.3f})")
        if key in P.get("PS4", {}) and not P["PS4"][key]["max"] < 0.2: fails.append(f"PS-4 ({P['PS4'][key]['max']:.3f})")
        if not all(v < 0.2 for v in P["PS1_MOM_sigma1"].values()): fails.append("PS-1 data (MOM own sigma1)")
        return fails
    print("\n# ARMS (mean [min, max] over training seeds for D, E, F)")
    for nm in A:
        m = agg(nm, "inc_q", HO); z = agg(nm, "inc_Z", HO); c = agg(nm, "inc_q", C4); lq = agg(nm, "loading", HO) if False else None
        loads = [r[HO]["loading"]["q"] for s, r in A[nm].items() if s != "refit400"]
        s1 = [(r[HO]["sigma1"].get(1) or r[HO]["sigma1"].get("1")) for s, r in A[nm].items() if s != "refit400"]
        print(f"{nm:34s} sigma1 {np.mean(s1):.3f} | inc_Z {z[0]:+.4f} | inc_q {m[0]:+.4f} [{m[1]:+.4f}, {m[2]:+.4f}] | 400K inc_q {c[0]:+.4f} | load q {np.mean(loads):.3f} | plants {'PASS' if not plants_ok(nm) else 'FAIL: ' + '; '.join(plants_ok(nm))}")
    lb = A["B TICA"]["None"][HO]["loading"]["q"]; lc = A["C VAMP"]["None"][HO]["loading"]["q"]
    c1 = lb >= 0.8 and lc >= 0.8
    dv = {k: np.array(A[k]["None"][HO]["dir_struct"]) for k in ("A SLOW-1 search", "B TICA", "C VAMP")}
    cs = lambda u, v: abs(u @ v) / (np.linalg.norm(u) * np.linalg.norm(v))
    print(f"   |cos| of the leading directions in STRUCT (standardised): A-B {cs(dv['A SLOW-1 search'], dv['B TICA']):.4f}, A-C {cs(dv['A SLOW-1 search'], dv['C VAMP']):.4f}, B-C {cs(dv['B TICA'], dv['C VAMP']):.4f}")
    print(f"C1: TICA f1 loads {lb:.3f} on q, VAMP {lc:.3f} -> {'MET: SLOW-1s search added nothing over TICA on this problem' if c1 else 'NOT MET for ' + ', '.join(n for n, l in (('TICA', lb), ('VAMP', lc)) if l < 0.8)}")
    Fst = [nm for nm in A if "(STAKE)" in nm][0]
    learned = ["D VAMPnet", "E SPIB", Fst]
    best = max(learned, key=lambda nm: agg(nm, "inc_q", HO)[0]); bm = agg(best, "inc_q", HO); b4 = agg(best, "inc_q", C4)
    ps2 = [A[best][s][HO]["PS2"] for s in A[best] if s != "refit400"]
    c2 = bm[0] >= 0.02 and all(p <= 0.01 for p in ps2) and b4[0] <= 0.005
    print(f"C2: best learned arm {best}: inc_q {bm[0]:+.4f} [{bm[1]:+.4f}, {bm[2]:+.4f}] (bar >= 0.02), PS-2 max {max(ps2):+.4f}, 400 K {b4[0]:+.4f} (bar <= 0.005) -> {'MET' if c2 else 'NOT MET'}")
    meets = {nm: (agg(nm, "inc_q", HO)[0] >= 0.02 and all(A[nm][s][HO]['PS2'] <= 0.01 for s in A[nm] if s != 'refit400') and agg(nm, "inc_q", C4)[0] <= 0.005) for nm in learned}
    fm, em = agg(Fst, "inc_q", HO), agg("E SPIB", "inc_q", HO)
    spread = (fm[2] - fm[1]) + (em[2] - em[1])
    c3 = fm[0] >= em[0]
    print(f"C3: F {fm[0]:+.4f} vs SPIB {em[0]:+.4f}: {'MET' if c3 else 'NOT MET'}; |difference| {abs(fm[0]-em[0]):.4f} vs combined spread {spread:.4f} -> {'separated' if abs(fm[0]-em[0]) > spread else 'tied within the training-seed spread'}")
    fF = plants_ok(Fst); kill = (not c3) or bool(fF)
    print(f"Integration kill: {'FIRES' if kill else 'does not fire'} ({'F below SPIB' if not c3 else ''}{'; F plant failures: ' + '; '.join(fF) if fF else ''})")
    # SLOW2_PREREG.md: (e) is a plant failing on THE ARM A BRANCH WOULD READ; an arm failing a plant is not read,
    # and F failing a plant fires the integration kill. (a) needs F read; (b) needs D or E read and meeting C2.
    ok = {nm: not plants_ok(nm) for nm in learned}
    for nm in learned:
        if not ok[nm]: print(f"   {nm}: NOT READ (plant failure: {'; '.join(plants_ok(nm))})")
    would = [nm for nm in learned if meets[nm]]
    if any(not ok[nm] for nm in would): br = "(e) a plant fails on the arm the branch would read: " + ", ".join(nm for nm in would if not ok[nm])
    elif meets[Fst] and c3 and ok[Fst] and ok["E SPIB"]: br = "(a) the integrated instrument finds something q does not carry and beats SPIB on the carried test"
    elif meets["D VAMPnet"] or meets["E SPIB"] or (meets[Fst] and not c3): br = "(b) the field's tool wins"
    else: br = "(c) no learned arm meets C2: tetrahedral order is all the structure carries on this operator at these lags"
    print(f"BRANCH: {br}" + (f"; the integration kill fires ({'; '.join(fF)})" if fF else "") + ("  [NOT A READING: quick]" if R["meta"].get("quick") else ""))


if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("mode", choices=["features", "run", "verdict"])
    ap.add_argument("--walk"); ap.add_argument("--synthetic", action="store_true"); ap.add_argument("--out")
    ap.add_argument("--cache"); ap.add_argument("--device", default="cuda"); ap.add_argument("--quick", action="store_true")
    a = ap.parse_args()
    if a.mode == "features":
        if a.synthetic:
            sy = synthetic(); d = features(sy["P"], sy["V"], sy["L"]); d["planted"] = sy["planted"].astype(np.float32)
        else:
            P, V, L, dt, meta = load(a.walk); assert abs(dt - DT_FS) < 1e-9; d = features(P, V, L)
        np.savez(a.out, **d); print(f"features -> {a.out}: " + ", ".join(f"{k} {np.shape(v)}" for k, v in d.items()))
    elif a.mode == "run":
        run(a.cache, a.out, a.device, quick=a.quick)
    else:
        R = json.load(open(a.cache)); verdict(R, "T293_seed2", "T400_seed0")
