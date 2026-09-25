#!/usr/bin/env python3
"""SLOW-3 (SLOW3_PREREG.md): the integration fixed, selected on the synthetic and TRAIN only, frozen, read on
fresh walks.

  slow3_train.py select --cache DIR --out slow3/selection.json   # PS-3 screen + 2-fold CV on seeds 0/1 (never seed 2)
  slow3_train.py freeze --cache DIR --selection slow3/selection.json --frozen slow3/frozen   # train, save, sha256 manifest
  slow3_train.py read   --cache DIR --frozen slow3/frozen --out slow3/slow3_read.json         # loads frozen models, trains nothing

The cache holds slow2_compare.py `features` npz files named T293_seed0, T293_seed1, synthetic (select, freeze) and
T293_seed3, T293_seed4, T400_seed0 (read). T293_seed2 is never opened: the loaders below refuse its name.
Everything reused (the SHELL input, the dictionary, the ridge, the VAMP core, SLOW-2's read(), R0, D's recipe) is
imported from slow2_compare.py, slow1_search.py and reason_search0b.py, not copied.
"""
import sys, os, json, argparse, time, hashlib
import numpy as np
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE); sys.path.insert(0, os.path.join(HERE, "..", "reasoning"))
import slow2_compare as s2
from slow2_compare import Walk, Std, lobe, ridge_r2_torch, torch_setup, read, r0, cubic_r2, lagpairs, TAU, H, STRIDE
from slow1_search import r2_fit
from reason_search0b import vamp

FORBIDDEN = {"T293_seed2"}
LAMS = [0.3, 1.0, 3.0]                  # the declared grid
LAM_REPORT = [0.0]                      # addition B: reported, not selectable
SPIB_VARIANTS = {"S1": (20, 1e-2), "S2": (20, 1e-3), "S3": (50, 1e-2), "S4": (50, 1e-3)}
SEEDS = [0, 1, 2]
TIE = 5e-4
if os.environ.get("SLOW3_SMOKE"):   # code-path smoke only (1 epoch, 1 seed): NOT A SELECTION
    SEEDS = [0]
EARLY_END, LATE_START = 1000, 1250      # EARLY: t < 1000 (t + 5 ps < 25 ps); LATE: t >= 1250; between: closure only
EPOCHS = 1 if os.environ.get("SLOW3_SMOKE") else 20


def say(*a): print(*a, flush=True)


def ld(cache, k):
    assert k not in FORBIDDEN, f"SLOW-3 never opens {k}"
    return Walk(dict(np.load(os.path.join(cache, f"{k}.npz"))), k)


def fit_m(w): return [i for i in range(w.n) if i % 5 != 0]
def val_m(w): return [i for i in range(w.n) if i % 5 == 0]


# ---------------------------------------------------------------- rows for F-fixed
def rows3(walks, mols_of, std_rich):
    """x(t), x(t + tau), x(t + 2 tau), C(t) = (v, |v|^2, q), y(t) = |r(t + 5 ps) - r(t)|, molecule half, era."""
    X0, Xt, X2, C, Y, HF, ER = [], [], [], [], [], [], []
    for w in walks:
        R = std_rich(w.rich()); ts = np.arange(0, w.T - H, STRIDE)
        era = np.where(ts < EARLY_END, 0, np.where(ts >= LATE_START, 1, -1))
        for i in mols_of(w):
            X0.append(R[ts, i]); Xt.append(R[ts + TAU, i]); X2.append(R[ts + 2 * TAU, i]); v = w.V[ts, i]
            C.append(np.column_stack([v, (v ** 2).sum(1), w.F19[ts, i, 0]]))
            Y.append(np.linalg.norm(w.P[ts + H, i] - w.P[ts, i], axis=1))
            HF.append(np.full(len(ts), i % 2)); ER.append(era)
    c = lambda L: np.concatenate(L).astype(np.float32)
    return c(X0), c(Xt), c(X2), c(C), c(Y), np.concatenate(HF).astype(np.int64), np.concatenate(ER).astype(np.int64)


def carried_xfit(torch, f, C, y, hf, er):
    """(i) + (ii) + addition A: a linear ridge head on [C, f], fitted on (even molecules, EARLY) and scored on
    (odd, LATE), and the reverse; R^2(y | C, f) - R^2(y | C), the second detached."""
    cA = (hf == 0) & (er == 0); cB = (hf == 1) & (er == 1); out = f.sum() * 0.0; k = 0   # a batch too small for both cells adds 0
    for a, b in ((cA, cB), (cB, cA)):
        if int(a.sum()) < 32 or int(b.sum()) < 32: continue
        Xf = torch.cat([C, f], 1)
        out = out + ridge_r2_torch(torch, Xf[a], y[a], Xf[b], y[b]) - ridge_r2_torch(torch, C[a], y[a], C[b], y[b]).detach(); k += 1
    return out / max(k, 1)


def train_ffixed(rows, k_in, seed, lam, device, epochs=EPOCHS, batch=2048, lr=5e-4):
    torch = torch_setup(seed)
    from deeptime.decomposition.deep import vamp_score
    X0, Xt, X2, C, Y = [torch.as_tensor(a).to(device) for a in rows[:5]]
    Y = (Y - Y.mean()) / Y.std(); HF = torch.as_tensor(rows[5]).to(device); ER = torch.as_tensor(rows[6]).to(device)
    net = lobe(torch, k_in).to(device); opt = torch.optim.Adam(net.parameters(), lr=lr)
    g = torch.Generator(device="cpu"); g.manual_seed(seed); hist = []
    for ep in range(epochs):
        perm = torch.randperm(len(X0), generator=g).to(device); sc = []; cr = []
        for b in range(0, len(X0), batch):
            idx = perm[b:b + batch]
            if len(idx) < 64: continue
            net.train(); f0 = net(X0[idx]); ft = net(Xt[idx]); f2 = net(X2[idx])
            v1 = vamp_score(f0, ft, method="VAMP2", epsilon=1e-6, mode="regularize")
            v2 = vamp_score(f0, f2, method="VAMP2", epsilon=1e-6, mode="regularize")
            loss = -0.5 * (v1 + v2)
            if lam:
                dr = carried_xfit(torch, f0, C[idx], Y[idx], HF[idx], ER[idx]); loss = loss - lam * dr; cr.append(float(dr.detach()))
            opt.zero_grad(); loss.backward(); opt.step(); sc.append(0.5 * float((v1 + v2).detach()))
        hist.append((ep, float(np.mean(sc)), float(np.mean(cr)) if cr else None))
    net.eval(); return net, hist


def train_d(rows, k_in, seed, device, epochs=EPOCHS, batch=2048, lr=5e-4):
    """D exactly as slow2_compare.train_vampnet(use_deeptime_fit=True): the same RNG sequence, the net returned."""
    torch = torch_setup(seed)
    from deeptime.decomposition.deep import VAMPNet
    X0, Xt = [torch.as_tensor(a).to(device) for a in rows[:2]]
    net = lobe(torch, k_in).to(device)
    g = torch.Generator(device="cpu"); g.manual_seed(seed)
    vn = VAMPNet(net, device=device, learning_rate=lr, score_method="VAMP2", score_mode="regularize", epsilon=1e-6)
    for ep in range(epochs):
        perm = torch.randperm(len(X0), generator=g).to(device)
        for b in range(0, len(X0), batch):
            idx = perm[b:b + batch]
            if len(idx) < 64: continue
            vn.partial_fit((X0[idx], Xt[idx]))
    net.eval(); return net, []


def net_out(net, std_m, std_s, device):
    import torch
    def out(w):
        with torch.no_grad():
            R = (w.rich().astype(np.float32) - std_m) / std_s; T, n, k = R.shape; o = []
            flat = torch.as_tensor(R.reshape(-1, k))
            for b in range(0, len(flat), 1 << 16): o.append(net(flat[b:b + (1 << 16)].to(device)).cpu().numpy())
            return np.concatenate(o).reshape(T, n, 2)
    return out


# ---------------------------------------------------------------- SPIB-fixed
def train_spib3(walks, mols_fit, mols_val, std_rich, seed, device, workdir, K, beta, n_tica=4):
    """SLOW-2's E with initial labels from k-means on the top n_tica TICA components (lag tau*) of the standardised
    input, fitted on the fitting trajectories. The schedule and the collapse handling are SLOW-2's."""
    torch = torch_setup(seed)
    from spib.spib import SPIB
    from spib.utils import TimeLaggedDataset, DataNormalize
    from deeptime.clustering import KMeans
    from deeptime.decomposition import TICA
    trajs_fit, trajs_val = [], []
    for w in walks:
        R = w.rich()
        trajs_fit += [R[:w.T - H + TAU, i].astype(np.float32) for i in mols_fit(w)]
        trajs_val += [R[:w.T - H + TAU, i].astype(np.float32) for i in mols_val(w)]
    mean, sd = std_rich.m32, std_rich.s32
    z = lambda X: ((X - mean) / sd).astype(np.float64)
    tica = TICA(lagtime=TAU, dim=n_tica).fit([z(x) for x in trajs_fit]).fetch_model()
    proj = lambda X: tica.transform(z(X))
    allp = np.concatenate([proj(x) for x in trajs_fit])
    rng = np.random.default_rng(seed); sub = allp[rng.choice(len(allp), min(100000, len(allp)), replace=False)]
    km = KMeans(n_clusters=K, fixed_seed=seed, max_iter=200).fit(sub).fetch_model()
    lab = lambda X: km.transform(proj(X)).astype(np.int64)
    Lf = [lab(x) for x in trajs_fit]; Lv = [lab(x) for x in trajs_val]
    ds_f = TimeLaggedDataset(trajs_fit, Lf, None, lagtime=TAU, subsampling_timestep=STRIDE, output_dim=K, device=device)
    ds_v = TimeLaggedDataset(trajs_val, Lv, None, lagtime=TAU, subsampling_timestep=STRIDE, output_dim=K, device=device)
    os.makedirs(workdir, exist_ok=True)
    model = SPIB(K, (trajs_fit[0].shape[1],), encoder_type="Nonlinear", z_dim=2, lagtime=TAU, beta=beta, learning_rate=1e-3,
                 device=device, path=os.path.join(workdir, f"spib_s{seed}"), UpdateLabel=True, neuron_num1=128, neuron_num2=128,
                 data_transform=DataNormalize(mean, sd)).to(device)
    import contextlib, io
    buf = io.StringIO(); collapsed = False
    with contextlib.redirect_stdout(buf):
        try:
            model.fit(ds_f, ds_v, batch_size=2048, tolerance=float("inf"), patience=4, refinements=8, index=seed)
        except ValueError:
            collapsed = True; model.eval()
    model.eval()
    info = dict(final_states=int(model.output_dim), collapsed=collapsed, refinements_done=len(model.convergence_history),
                initial_occupied=int(len(np.unique(np.concatenate(Lf)))))
    return model, info


def spib_out(model):
    def out(w):
        R = w.rich(); T, n, k = R.shape
        _, _, zm, _ = model.transform(R.reshape(-1, k).astype(np.float32), batch_size=1 << 15, to_numpy=True)
        return zm.reshape(T, n, 2).astype(np.float32)
    return out


# ---------------------------------------------------------------- f1 as saved parameters
def f1_params(train_outs):
    A = np.concatenate([lagpairs(o, TAU)[0] for o in train_outs]); B = np.concatenate([lagpairs(o, TAU)[1] for o in train_outs])
    m = vamp(A, B); return dict(mean=np.asarray(m["mean"], float), scale=np.asarray(m["scale"], float), w=np.asarray(m["W0"] @ m["U"][:, 0], float))


def f1_from(p): return lambda o: ((o - p["mean"]) / p["scale"]) @ p["w"]


# ---------------------------------------------------------------- one candidate: build, PS-3, CV
def make(kind, cand, walks_fit, std, seed, device, workdir, rows_cache):
    """kind 'F' (cand = lambda), 'D' (cand None) or 'SPIB' (cand = variant); returns (out_fn, model, info)."""
    key = (id(walks_fit[0]), len(walks_fit))
    if kind in ("F", "D") and key not in rows_cache:
        rows_cache[key] = rows3(walks_fit, lambda w: rows_cache["mols"](w), std)
    if kind == "F":
        net, hist = train_ffixed(rows_cache[key], rows_cache[key][0].shape[1], seed, cand, device)
        return net_out(net, std.m32, std.s32, device), net, dict(hist=hist)
    if kind == "D":
        net, _ = train_d(rows_cache[key], rows_cache[key][0].shape[1], seed, device)
        return net_out(net, std.m32, std.s32, device), net, {}
    K, beta = SPIB_VARIANTS[cand]
    model, info = train_spib3(walks_fit, rows_cache["mols"], rows_cache["vmols"], std, seed, device, workdir, K, beta)
    return spib_out(model), model, info


def select(cache, outp, device):
    t0 = time.time(); R = dict(meta=dict(device=device, lams=LAMS, spib=SPIB_VARIANTS, seeds=SEEDS, tie=TIE, epochs=EPOCHS), ps3={}, cv={})
    syn = ld(cache, "synthetic"); tr = {0: ld(cache, "T293_seed0"), 1: ld(cache, "T293_seed1")}
    sy_fit = syn.sub([i for i in range(syn.n) if i % 4 != 0]); sy_te = syn.sub([i for i in range(syn.n) if i % 4 == 0])
    std_syn = Std(sy_fit.rich()); wd = os.path.join(os.path.dirname(os.path.abspath(outp)), "spib_work")
    cands = [("F", l) for l in LAMS + LAM_REPORT] + [("SPIB", v) for v in SPIB_VARIANTS] + [("D", None)]
    name = lambda k, c: f"F lam={c:g}" if k == "F" else (f"SPIB {c}" if k == "SPIB" else "D VAMPnet")
    # PS-3 on the synthetic, 3 torch seeds
    say("# PS-3 screen on the synthetic (fit i mod 4 != 0, read i mod 4 == 0), torch seeds 0, 1, 2")
    rc = {"mols": lambda w: range(w.n), "vmols": lambda w: [i for i in range(w.n) if i % 5 == 0]}
    rc_sp = {"mols": lambda w: [i for i in range(w.n) if i % 5 != 0], "vmols": lambda w: [i for i in range(w.n) if i % 5 == 0]}
    for k, c in cands:
        nm = name(k, c); sc = []
        for s in SEEDS:
            fn, _, info = make(k, c, [sy_fit], std_syn, s, device, wd + "_ps3", rc_sp if k == "SPIB" else rc)
            f1 = f1_from(f1_params([fn(sy_fit)])); f = f1(fn(sy_te)).reshape(-1); p = sy_te.planted.reshape(-1)
            sc.append(dict(cubic=cubic_r2(f, p), linear=r2_fit(f[:, None], p), info={k2: v for k2, v in info.items() if k2 != "hist"}))
        R["ps3"][nm] = sc
        say(f"  PS-3 {nm:14s}: cubic R^2 by seed {[round(x['cubic'], 3) for x in sc]} (linear {[round(x['linear'], 3) for x in sc]}) "
            f"-> {'ADMISSIBLE' if all(x['cubic'] >= 0.9 for x in sc) else 'not admissible'}"
            + (f"  states {[x['info'].get('final_states') for x in sc]}" if k == "SPIB" else "") + f" [{time.time() - t0:.0f} s]")
    # 2-fold CV on TRAIN seeds 0 and 1
    say("# 2-fold CV on TRAIN (train on seed a, read on seed b): R^2(Z + q + F) - R^2(Z + q), the stake's statistic")
    stds = {a: Std(tr[a].rich()) for a in (0, 1)}
    rcs = {a: {"mols": fit_m, "vmols": val_m} for a in (0, 1)}
    for k, c in cands:
        nm = name(k, c); rows = []
        for a, b in ((0, 1), (1, 0)):
            for s in SEEDS:
                fn, _, info = make(k, c, [tr[a]], stds[a], s, device, wd + f"_cv{a}", rcs[a])
                f1 = f1_from(f1_params([fn(tr[a])])); r = read(tr[b], fn(tr[b]), f1)
                rows.append(dict(train=a, test=b, seed=s, inc_q=r["inc_q"], inc_Z=r["inc_Z"], PS2=r["PS2"], sigma1=r["sigma1"][1],
                                 load_q=r["loading"]["q"], q_carry=r["q_carry"], info={k2: v for k2, v in info.items() if k2 != "hist"}))
        R["cv"][nm] = rows; v = [x["inc_q"] for x in rows]
        say(f"  CV {nm:14s}: inc_q mean {np.mean(v):+.4f} [{min(v):+.4f}, {max(v):+.4f}] | fold 0->1 {np.mean([x['inc_q'] for x in rows if x['train'] == 0]):+.4f} 1->0 {np.mean([x['inc_q'] for x in rows if x['train'] == 1]):+.4f} "
            f"| inc_Z {np.mean([x['inc_Z'] for x in rows]):+.4f} | PS2 max {max(x['PS2'] for x in rows):+.4f} | sigma1 {np.mean([x['sigma1'] for x in rows]):.3f} "
            f"| load q {np.mean([x['load_q'] for x in rows]):.3f} [{time.time() - t0:.0f} s]")
    R["choice"] = choose(R, name)
    R["meta"]["seconds"] = time.time() - t0
    json.dump(R, open(outp, "w"), indent=1, default=lambda x: float(x) if isinstance(x, np.floating) else (int(x) if isinstance(x, np.integer) else str(x)))
    return R


def choose(R, name):
    out = {}
    for kind, grid, order in (("F", LAMS, lambda c: c), ("SPIB", list(SPIB_VARIANTS), lambda c: list(SPIB_VARIANTS).index(c))):
        rows = []
        for c in grid:
            nm = name(kind, c); ps = [x["cubic"] for x in R["ps3"][nm]]; cv = float(np.mean([x["inc_q"] for x in R["cv"][nm]]))
            rows.append(dict(cand=c, name=nm, ps3_min=min(ps), admissible=all(p >= 0.9 for p in ps), cv=cv))
        adm = [r for r in rows if r["admissible"]]
        if adm:
            best = max(r["cv"] for r in adm)
            tied = sorted([r for r in adm if best - r["cv"] < TIE], key=lambda r: order(r["cand"]))
            pick = tied[0]; why = f"admissible {[r['name'] for r in adm]}; highest CV {best:+.4f}; within {TIE} of it {[r['name'] for r in tied]} -> {pick['name']}"
        elif kind == "F":
            pick = max(rows, key=lambda r: r["ps3_min"]); why = f"NO lambda admissible: G1 FAILS (the kill). Frozen for the record: highest min PS-3, {pick['name']}"
        else:
            pick = max(rows, key=lambda r: r["cv"]); why = f"NO SPIB variant admissible: SPIB-fixed NOT READ (G3). Frozen for the record: highest CV, {pick['name']}"
        out[kind] = dict(table=rows, pick=pick["cand"], admissible=bool(adm), why=why)
        say(f"# SELECTION {kind}: " + " | ".join(f"{r['name']}: PS-3 min {r['ps3_min']:.3f} {'adm' if r['admissible'] else 'NOT adm'}, CV {r['cv']:+.4f}" for r in rows))
        say(f"   -> {why}")
    return out


# ---------------------------------------------------------------- freeze
def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as fh:
        for b in iter(lambda: fh.read(1 << 20), b""): h.update(b)
    return h.hexdigest()


def freeze(cache, selp, fdir, device):
    import torch
    t0 = time.time(); sel = json.load(open(selp)); lam = float(sel["choice"]["F"]["pick"]); var = sel["choice"]["SPIB"]["pick"]
    os.makedirs(fdir, exist_ok=True)
    tr = [ld(cache, "T293_seed0"), ld(cache, "T293_seed1")]; std = Std([w.rich() for w in tr])
    sh_tr = [w.shuffled() for w in tr]; std_sh = Std([w.rich() for w in sh_tr])
    np.savez(os.path.join(fdir, "std_train.npz"), m=std.m, s=std.s); np.savez(os.path.join(fdir, "std_train_shuffled.npz"), m=std_sh.m, s=std_sh.s)
    wd = os.path.join(fdir, "..", "spib_work_freeze"); meta = dict(lam=lam, spib_variant=var, spib_K_beta=SPIB_VARIANTS[var], seeds=SEEDS, arms={})
    rc = {"mols": fit_m, "vmols": val_m}; rc_sh = {"mols": fit_m, "vmols": val_m}
    for kind, cand, tag in (("D", None, "D"), ("F", lam, "F"), ("SPIB", var, "SPIB")):
        for s in SEEDS:
            fn, model, info = make(kind, cand, tr, std, s, device, wd, rc)
            p = f1_params([fn(w) for w in tr]); np.savez(os.path.join(fdir, f"{tag}_s{s}_f1.npz"), **p)
            path = os.path.join(fdir, f"{tag}_s{s}.pt")
            torch.save(model.state_dict() if kind != "SPIB" else model, path)
            meta["arms"].setdefault(tag, {})[str(s)] = {k: v for k, v in info.items() if k != "hist"}
            say(f"  frozen {tag} seed {s} [{time.time() - t0:.0f} s] {meta['arms'][tag][str(s)]}")
        # PS-4: trained on time-shuffled TRAIN, torch seed 0
        fn, model, info = make(kind, cand, sh_tr, std_sh, 0, device, wd + "_ps4", rc_sh)
        torch.save(model.state_dict() if kind != "SPIB" else model, os.path.join(fdir, f"{tag}_ps4_s0.pt"))
        meta["arms"][tag]["ps4"] = {k: v for k, v in info.items() if k != "hist"}
        say(f"  frozen {tag} PS-4 (time-shuffled TRAIN) [{time.time() - t0:.0f} s]")
    import torch as _t, deeptime, spib as _sp
    meta["env"] = dict(torch=_t.__version__, deeptime=deeptime.__version__, numpy=np.__version__, device=device,
                       gpu=_t.cuda.get_device_name(0) if device == "cuda" else None, selection_sha256=sha(selp))
    json.dump(meta, open(os.path.join(fdir, "freeze_meta.json"), "w"), indent=1, default=str)
    files = sorted(f for f in os.listdir(fdir) if f != "MANIFEST.sha256")
    with open(os.path.join(fdir, "MANIFEST.sha256"), "w") as fh:
        for f in files: fh.write(f"{sha(os.path.join(fdir, f))}  {f}\n")
    say(f"# FROZEN: {len(files)} files, manifest sha256 {sha(os.path.join(fdir, 'MANIFEST.sha256'))} [{time.time() - t0:.0f} s]")


def load_frozen(fdir, device):
    import torch
    bad = [l for l in open(os.path.join(fdir, "MANIFEST.sha256")) if sha(os.path.join(fdir, l.split()[1])) != l.split()[0]]
    assert not bad, f"frozen files do not match the manifest: {bad}"
    meta = json.load(open(os.path.join(fdir, "freeze_meta.json"))); st = np.load(os.path.join(fdir, "std_train.npz"))
    sts = np.load(os.path.join(fdir, "std_train_shuffled.npz")); arms = {}
    m32, s32 = st["m"].astype(np.float32), st["s"].astype(np.float32)
    for tag in ("D", "F", "SPIB"):
        def mk(path, m, s):
            if tag == "SPIB":
                model = torch.load(path, weights_only=False, map_location=device); model.eval(); return spib_out(model)
            net = lobe(torch, len(m)).to(device); net.load_state_dict(torch.load(path, map_location=device)); net.eval()
            return net_out(net, m, s, device)
        arms[tag] = dict(seeds={s: (mk(os.path.join(fdir, f"{tag}_s{s}.pt"), m32, s32), f1_from(dict(np.load(os.path.join(fdir, f"{tag}_s{s}_f1.npz")))))
                                for s in SEEDS},
                         ps4=mk(os.path.join(fdir, f"{tag}_ps4_s0.pt"), sts["m"].astype(np.float32), sts["s"].astype(np.float32)))
    return meta, arms


# ---------------------------------------------------------------- the read
def read_fresh(cache, fdir, outp, device, test_names=("T293_seed3", "T293_seed4"), control="T400_seed0", dry=False):
    t0 = time.time(); meta, arms = load_frozen(fdir, device)
    sel = json.load(open(os.path.join(os.path.dirname(os.path.abspath(outp)), "selection.json")))
    tests = [ld(cache, k) for k in test_names]; c4 = ld(cache, control); walks = tests + [c4]
    tr = [ld(cache, "T293_seed0"), ld(cache, "T293_seed1")]
    R = dict(meta=dict(frozen=meta, dry=dry, tests=list(test_names), control=control), arms={}, plants={}, R0={}, q_carry={})
    say(f"# SLOW-3 READ{' (DRY: NOT A READING)' if dry else ''}: lam = {meta['lam']:g}, SPIB variant {meta['spib_variant']} {meta['spib_K_beta']}")
    for w in walks:
        R["plants"].setdefault("PS1_MOM_sigma1", {})[w.name] = float(vamp(*lagpairs(w.F19[:, :, 8:11], TAU))["s"][0])
    say(f"# PS-1 data: MOM own sigma1 at 1 ps {R['plants']['PS1_MOM_sigma1']}")
    # PS-4 first (time-shuffled test walks, the frozen shuffled-TRAIN models)
    for tag in ("D", "F", "SPIB"):
        for w in tests:
            sw = w.shuffled(); s = read(sw, arms[tag]["ps4"](sw), None, full=False)["sigma1"]
            R["plants"].setdefault("PS4", {}).setdefault(tag, {})[w.name] = dict(sigma1=s, max=max(s.values()))
            say(f"  PS-4 {tag} on shuffled {w.name}: own sigma1 {{{', '.join(f'{k}: {v:.3f}' for k, v in s.items())}}} max {max(s.values()):.3f} -> {'PASS' if max(s.values()) < 0.2 else 'FAIL'}")
    # linear arm A (reported), fitted on TRAIN
    fa = s2.arm_linear("A", tr); f1a = s2.f1_map([fa(w) for w in tr])
    for w in walks:
        R["arms"].setdefault("A SLOW-1 search", {}).setdefault("None", {})[w.name] = read(w, fa(w), f1a)
    for tag in ("D", "F", "SPIB"):
        for s in SEEDS:
            fn, f1 = arms[tag]["seeds"][s]
            for w in walks:
                r = read(w, fn(w), f1); R["arms"].setdefault(tag, {}).setdefault(str(s), {})[w.name] = r
                say(f"  {tag} seed {s} {w.name}: sigma1 {r['sigma1'][1]:.3f} inc_Z {r['inc_Z']:+.4f} inc_q {r['inc_q']:+.4f} inc_S {r['inc_S']:+.4f} "
                    f"| load q {r['loading']['q']:.3f} MOM {r['loading']['MOM']:.4f} | PS2 {r['PS2']:+.4f} [{time.time() - t0:.0f} s]")
    for w in walks:
        R["R0"][w.name] = r0(w); R["q_carry"][w.name] = R["arms"]["A SLOW-1 search"]["None"][w.name]["q_carry"]
    say(f"# R0 (84 SHELL columns in the ridge, beyond velocity and q): {R['R0']}")
    say(f"# q's own carry beyond velocity (base rate): {R['q_carry']}")
    R["ps3"] = {k: sel["ps3"][f"F lam={meta['lam']:g}"] if k == "F" else (sel["ps3"][f"SPIB {meta['spib_variant']}"] if k == "SPIB" else sel["ps3"]["D VAMPnet"]) for k in ("D", "F", "SPIB")}
    R["meta"]["seconds"] = time.time() - t0
    json.dump(R, open(outp, "w"), indent=1, default=lambda x: float(x) if isinstance(x, np.floating) else str(x))
    verdict(R)
    return R


def verdict(R):
    tests = R["meta"]["tests"]; C4 = R["meta"]["control"]; A = R["arms"]; P = R["plants"]
    def inc(tag, key="inc_q", walks=None):
        walks = walks or tests
        per = {w: [A[tag][str(s)][w][key] for s in SEEDS] for w in walks}
        allv = [v for w in walks for v in per[w]]
        mean = float(np.mean([np.mean(per[w]) for w in walks]))
        spread = float(np.mean([max(per[w]) - min(per[w]) for w in walks]))
        return mean, min(allv), max(allv), spread, {w: float(np.mean(per[w])) for w in walks}
    def plants(tag):
        f = []
        for s in SEEDS:
            for w in tests + [C4]:
                r = A[tag][str(s)][w]
                if not r["PS1_f1_on_MOM"] < 0.05: f.append(f"PS-1 f1 on MOM {r['PS1_f1_on_MOM']:.3f} seed {s} {w}")
                if not r["PS2"] <= 0.01: f.append(f"PS-2 {r['PS2']:+.4f} seed {s} {w}")
        for w, v in P["PS1_MOM_sigma1"].items():
            if not v < 0.2: f.append(f"PS-1 data MOM sigma1 {v:.3f} {w}")
        ps3 = [x["cubic"] for x in R["ps3"][tag]]
        if not all(p >= 0.9 for p in ps3): f.append(f"PS-3 {[round(p, 3) for p in ps3]}")
        for w, v in P["PS4"][tag].items():
            if not v["max"] < 0.2: f.append(f"PS-4 {v['max']:.3f} {w}")
        return f
    say("\n# ARMS on TEST (mean over the fresh seeds of the mean over torch seeds 0, 1, 2) and CONTROL")
    rows = {}
    for tag, nm in (("D", "D VAMPnet (reference)"), ("F", f"F-fixed lam={R['meta']['frozen']['lam']:g}"), ("SPIB", f"SPIB-fixed {R['meta']['frozen']['spib_variant']}")):
        m = inc(tag); z = inc(tag, "inc_Z"); c = inc(tag, walks=[C4]); pf = plants(tag); rows[tag] = (m, pf)
        say(f"{nm:28s} inc_q {m[0]:+.4f} [{m[1]:+.4f}, {m[2]:+.4f}] per seed {', '.join(f'{k} {v:+.4f}' for k, v in m[4].items())} | inc_Z {z[0]:+.4f} | "
            f"400K inc_q {c[0]:+.4f} | plants {'PASS' if not pf else 'FAIL: ' + '; '.join(pf)}")
    a = [A["A SLOW-1 search"]["None"][w]["inc_q"] for w in tests]
    r0m = float(np.mean([R["R0"][w] for w in tests]))
    say(f"{'A SLOW-1 search (no stake)':28s} inc_q {np.mean(a):+.4f} | R0 {r0m:+.4f} (per seed {', '.join(f'{w} {R['R0'][w]:+.4f}' for w in tests)}) | "
        f"q carry {np.mean([R['q_carry'][w] for w in tests]):+.4f} | 400K R0 {R['R0'][C4]:+.4f}")
    (fm, ff), (dm, df), (sm, sf) = rows["F"], rows["D"], rows["SPIB"]
    g1 = not ff; g2a = fm[0] >= 0.02; g2b = fm[0] >= dm[0]; g4 = fm[0] >= r0m - 0.005
    sp_read = not sf; g3 = fm[0] >= sm[0]
    tied = lambda x, y: abs(x[0] - y[0]) <= x[3] + y[3]
    say(f"G1 plants on F-fixed: {'MET' if g1 else 'NOT MET: ' + '; '.join(ff)}")
    say(f"G2a F-fixed inc_q {fm[0]:+.4f} >= +0.02: {'MET' if g2a else 'NOT MET'} (min over seed x torch seed {fm[1]:+.4f})")
    say(f"G2b F-fixed {fm[0]:+.4f} >= D {dm[0]:+.4f}: {'MET' if g2b else 'NOT MET'}{' (tied within the spread)' if tied(fm, dm) else ' (separated)'}")
    say(f"G3 F-fixed {fm[0]:+.4f} >= SPIB-fixed {sm[0]:+.4f}: " + (('MET' if g3 else 'NOT MET') + (' (tied within the spread)' if tied(fm, sm) else ' (separated)') if sp_read else f"SPIB-fixed NOT READ ({'; '.join(sf)}); by the letter {'>=' if g3 else '<'}"))
    say(f"G4 F-fixed {fm[0]:+.4f} >= R0 - 0.005 = {r0m - 0.005:+.4f}: {'MET' if g4 else 'NOT MET'}")
    kill = (not g1) or (not g2a)
    say(f"Kill: {'FIRES' if kill else 'does not fire'}")
    if not g1: br = "(e) a plant fails on F-fixed: nothing is read from it; the integration kill fires"
    elif not g2a: br = "(c) the carried structure beyond q is below +0.02 on this operator at these lags, now on fresh data"
    elif g2b and g3 and g4 and sp_read: br = "(a) the integration works and beats the field's tool on fresh data"
    else:
        miss = [n for n, ok in (("G2b: the plain VAMPnet carries as much; the integration adds nothing over it", g2b), ("G3", g3 and sp_read), ("G4", g4)) if not ok]
        br = "(b) G1 and G2a met; not met: " + "; ".join(miss) + ("" if sp_read else " (SPIB-fixed not read)")
    say(f"BRANCH: {br}" + ("  [DRY: NOT A READING]" if R["meta"].get("dry") else ""))


if __name__ == "__main__":
    ap = argparse.ArgumentParser(); ap.add_argument("mode", choices=["select", "freeze", "read", "verdict"])
    ap.add_argument("--cache"); ap.add_argument("--out"); ap.add_argument("--selection"); ap.add_argument("--frozen")
    ap.add_argument("--device", default="cuda"); ap.add_argument("--dry", action="store_true")
    a = ap.parse_args()
    if a.mode == "select": select(a.cache, a.out, a.device)
    elif a.mode == "freeze": freeze(a.cache, a.selection, a.frozen, a.device)
    elif a.mode == "read":
        if a.dry:   # code path only, on TRAIN walks standing in for the fresh ones: NOT A READING
            read_fresh(a.cache, a.frozen, a.out, a.device, test_names=("T293_seed0", "T293_seed1"), control="T293_seed1", dry=True)
        else: read_fresh(a.cache, a.frozen, a.out, a.device)
    else: verdict(json.load(open(a.out)))
