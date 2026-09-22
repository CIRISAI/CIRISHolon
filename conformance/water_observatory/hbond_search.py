#!/usr/bin/env python3
"""HBOND-SEARCH-1 (TSCAN_HBOND_SEARCH_PREREG.md): the network rung on an all-atom walk.
Per molecule per frame: bonds donated, bonds accepted (O-O < 3.5 A, angle H-O...O < 30 deg),
COM velocity (3), mean O-H stretch rate, H-O-H angle rate. Molecules are the ensemble.
  hbond_search.py plants | hbond_search.py read DIR"""
import sys, os, math, numpy as np
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__))); sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "reasoning"))
from molsearch import load, molecules, features as mol_features, M_O, M_H
from reason_search0b import vamp, heldout, blocks
BOHR_A = 0.529177
BOND = [0, 1]; VEL = [2, 3, 4]; INT = [5, 6]
def hbonds(P, z, L):
    mols = molecules(z); n = len(mols); T = len(P)
    O = np.array([m[0] for m in mols]); H1 = np.array([m[1] for m in mols]); H2 = np.array([m[2] for m in mols])
    don = np.zeros((T, n)); acc = np.zeros((T, n))
    rc = 3.5 / BOHR_A; cos30 = math.cos(math.radians(30))
    for t in range(T):
        PO = P[t, O]; d = PO[:, None, :] - PO[None, :, :]; d -= L * np.round(d / L); r = np.linalg.norm(d, axis=2); np.fill_diagonal(r, 1e9)
        close = r < rc
        for Hidx in (H1, H2):
            v = P[t, Hidx] - PO; v -= L * np.round(v / L); vn = v / np.linalg.norm(v, axis=1)[:, None]
            # angle at O_i between O_i->H_i and O_i->O_j: cos = vn_i . (d_ij/r_ij) where d_ij = O_j - O_i = -d[i,j]
            cosang = np.einsum('ik,ijk->ij', vn, -d / r[..., None])
            b = close & (cosang > cos30)
            don[t] += b.sum(1); acc[t] += b.sum(0)
    return don, acc
def features(P, V, z, L, dt):
    don, acc = hbonds(P, z, L); mf = mol_features(P, V, z, dt)
    out = []
    for i, f in enumerate(mf):
        out.append(np.column_stack([don[:, i], acc[:, i], f[:, 6], f[:, 7], f[:, 8], 0.5 * (f[:, 3] + f[:, 4]), f[:, 5]]))
    return out
def read(feats, dt, label=""):
    n = len(feats); T = len(feats[0])
    print(f"# HBOND-SEARCH-1{label}: {n} molecules x {T} rows at {dt:.2f} fs; mean bonds donated {np.mean([f[:,0].mean() for f in feats]):.2f}, accepted {np.mean([f[:,1].mean() for f in feats]):.2f}")
    folds = [[i for i in range(n) if i % 4 == j] for j in range(4)]
    def pairs(fs, lag): return np.concatenate([f[:-lag] for f in fs]), np.concatenate([f[lag:] for f in fs])
    EXTRA_LAGS = tuple(int(x) for x in os.environ.get("HBOND_EXTRA_LAGS","").split(",") if x)   # labelled extra: beyond the prereg's 100 fs, on the 5 ps walk only
    for tau in (5, 20, 50, 100) + EXTRA_LAGS:
        lag = max(1, int(round(tau / dt)))
        A, B = pairs(feats, lag); m = vamp(A, B)
        s_bond = vamp(A, B, BOND)["s"][0]; s_vel = vamp(A, B, VEL)["s"][0]; s_int = vamp(A, B, INT)["s"][0]
        cross, kbv, kvb = blocks(A, B, BOND, VEL)
        print(f"   lag {tau:4d} fs{' [EXTRA, beyond the prereg]' if tau > 100 else ''}: dictionary σ {np.array2string(m['s'][:5], precision=3)} | own top σ: bonds {s_bond:.3f}, velocity {s_vel:.3f}, internal {s_int:.3f} | bond-velocity cross {cross:.3f} (B→V {kbv:.3f}, V→B {kvb:.3f})")
        if tau == 100: print(f"   S1 bond state closed at 100 fs (own σ ≥ 0.5, kill < 0.2): {s_bond:.3f} -> {'MET' if s_bond >= 0.5 else ('KILL' if s_bond < 0.2 else 'between')}")
        if tau == 5: print(f"   S2 bond-velocity decoupled at 5 fs (cross < 0.2, kill ≥ 0.4): {cross:.3f} -> {'MET' if cross < 0.2 else ('KILL' if cross >= 0.4 else 'between')}")
    # S3: the bond state predicts the momentum change over 50 fs (held out by molecule) vs a re-paired null
    lag = max(1, int(round(50 / dt))); rng = np.random.default_rng(3)
    def r2_bond_to_dv(shuffle):
        out = []
        for te in folds:
            tr = [i for i in range(n) if i not in te]
            Xtr = np.concatenate([feats[i][:-lag][:, BOND + VEL] for i in tr]); Ytr = np.concatenate([feats[i][lag:, VEL] - feats[i][:-lag, VEL] for i in tr])
            Xte = np.concatenate([feats[i][:-lag][:, BOND + VEL] for i in te]); Yte = np.concatenate([feats[i][lag:, VEL] - feats[i][:-lag, VEL] for i in te])
            if shuffle: Ytr = Ytr[rng.permutation(len(Ytr))]; Yte = Yte[rng.permutation(len(Yte))]
            m_, s_ = Xtr.mean(0), Xtr.std(0) + 1e-12; A = (Xtr - m_) / s_; W = np.linalg.solve(A.T @ A + 1e-3 * len(A) * np.eye(A.shape[1]), A.T @ (Ytr - Ytr.mean(0)))
            pred = ((Xte - m_) / s_) @ W + Ytr.mean(0); out.append(1 - ((Yte - pred) ** 2).sum() / ((Yte - Yte.mean(0)) ** 2).sum())
        return float(np.mean(out))
    # the velocity's own contribution is removed by comparing bonds+velocity against velocity alone
    def r2_cols(cols):
        out = []
        for te in folds:
            tr = [i for i in range(n) if i not in te]
            Xtr = np.concatenate([feats[i][:-lag][:, cols] for i in tr]); Ytr = np.concatenate([feats[i][lag:, VEL] - feats[i][:-lag, VEL] for i in tr])
            Xte = np.concatenate([feats[i][:-lag][:, cols] for i in te]); Yte = np.concatenate([feats[i][lag:, VEL] - feats[i][:-lag, VEL] for i in te])
            m_, s_ = Xtr.mean(0), Xtr.std(0) + 1e-12; A = (Xtr - m_) / s_; W = np.linalg.solve(A.T @ A + 1e-3 * len(A) * np.eye(A.shape[1]), A.T @ (Ytr - Ytr.mean(0)))
            pred = ((Xte - m_) / s_) @ W + Ytr.mean(0); out.append(1 - ((Yte - pred) ** 2).sum() / ((Yte - Yte.mean(0)) ** 2).sum())
        return float(np.mean(out))
    r_bv = r2_cols(BOND + VEL); r_v = r2_cols(VEL); r_b = r2_cols(BOND); r_null = r2_bond_to_dv(True)
    print(f"   S3 Δv over 50 fs: R² from bonds+velocity {r_bv:.3f}, velocity alone {r_v:.3f}, bonds alone {r_b:.3f}, re-paired null {r_null:.3f}; bonds' increment over velocity {r_bv - r_v:+.3f}; bonds alone / null {r_b/max(abs(r_null),1e-9):.1f} -> {'MET' if r_b >= 3*abs(r_null) and r_b > 0 else ('KILL' if r_b < 1.5*abs(r_null) else 'between')}")
def plants():
    rng = np.random.default_rng(2); dt = 1.0; T = 500; n = 32; feats = []
    for i in range(n):
        x = np.zeros((T, 7)); b = np.array([2.0, 2.0]); v = rng.normal(size=3) * 1e-4
        for t in range(T):
            if rng.random() < 0.02: b = np.clip(b + rng.choice([-1, 1], size=2), 0, 4)
            v = 0.98 * v + 0.02e-4 * (b[0] - b[1]) + rng.normal(size=3) * 3e-6   # a planted bond->force coupling
            x[t, :2] = b; x[t, 2:5] = v; x[t, 5:] = rng.normal(size=2) * 1e-5
        feats.append(x)
    print("PH-1 (synthetic: bonds flip at 2 % per fs, a planted bond->force coupling):"); read(feats, dt, label=" plant")
    A, B = np.concatenate([f[:-1] for f in feats]), np.concatenate([f[1:] for f in feats]); e = np.abs(vamp(A, A)["s"] - 1).max()
    print(f"PH-2: lag 0 max |σ−1| = {e:.1e} -> {'PASS' if e < 1e-2 else 'FAIL'}")
if __name__ == "__main__":
    if sys.argv[1] == "plants": plants(); sys.exit(0)
    P, V, z, L, dt = load(sys.argv[2]); read(features(P, V, z, L, dt), dt, label=f" on {sys.argv[2]}")
