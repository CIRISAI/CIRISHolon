#!/usr/bin/env python3
"""REASON-SEARCH-0 re-read under Amendment 1: parent->child chains from accord_traces.jsonl."""
import sys, os, json, math, collections
import numpy as np
sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "water_observatory"))
from view_search import inv_sqrt
SRC = "/home/emoore/RATCHET/release/data_scrubbed_v1/accord_traces.jsonl"
NAMES = ["plaus", "align", "k_eff", "corr_risk", "depth", "log_tok", "act_SPEAK"]   # PONDER is SPEAK's complement once TOOL is dropped (PR-B on building)
DMA = [0, 1, 2, 3]; PROC = [4, 5, 6]; DEPTH = [4]
def g(r, *ks):
    v = r
    for k in ks: v = v.get(k) if isinstance(v, dict) else None
    return v
def feat(r):
    d = [r.get("csdma_plausibility_score"), r.get("dsdma_domain_alignment"), r.get("idma_k_eff"), r.get("idma_correlation_risk")]
    if any(not isinstance(v, (int, float)) or isinstance(v, bool) for v in d): return None
    a = r.get("selected_action")
    if a not in ("SPEAK", "PONDER"): return None
    return d + [float(r["thought_depth"]), math.log(max(r.get("tokens_output") or 1, 1)), 1.0 if a == "SPEAK" else 0.0]
def load():
    R = [json.loads(l) for l in open(SRC)]; by = {r["thought_id"]: r for r in R}
    child = {r["thought_id"]: g(r, "action_result", "follow_up_thought_id") for r in R if g(r, "action_result", "follow_up_thought_id") in by}
    parents = set(child.values()); roots = [t for t in child if t not in parents]
    chains = []
    for t in roots:
        seq = [t]
        while t in child: t = child[t]; seq.append(t)
        X = [feat(by[s]) for s in seq]
        # keep the longest prefix of consecutive usable rows
        rows = []
        for x in X:
            if x is None: break
            rows.append(x)
        if len(rows) >= 2: chains.append(np.array(rows))
    return chains
def pairs(chains):
    A = np.concatenate([c[:-1] for c in chains]); B = np.concatenate([c[1:] for c in chains]); return A, B
def vamp(A, B, cols=None, ridge_rel=1e-3):
    if cols is not None: A = A[:, cols]; B = B[:, cols]
    mean = 0.5 * (A.mean(0) + B.mean(0)); scale = np.maximum(np.concatenate([A, B]).std(0), 1e-12)
    A = (A - mean) / scale; B = (B - mean) / scale; N = len(A)
    C00, C0t, Ctt = A.T @ A / N, A.T @ B / N, B.T @ B / N
    ridge = ridge_rel * np.trace(C00) / len(C00); W0, Wt = inv_sqrt(C00, ridge), inv_sqrt(Ctt, ridge)
    U, s, Vt = np.linalg.svd(W0 @ C0t @ Wt)
    return dict(s=s, U=U, Vt=Vt, W0=W0, Wt=Wt, mean=mean, scale=scale, C00=C00, C0t=C0t)
def heldout(trA, trB, teA, teB, k, cols=None):
    m = vamp(trA, trB, cols)
    A = teA[:, cols] if cols is not None else teA; B = teB[:, cols] if cols is not None else teB
    A = (A - m["mean"]) / m["scale"]; B = (B - m["mean"]) / m["scale"]
    PA = A @ (m["W0"] @ m["U"][:, :k]); PB = B @ (m["Wt"] @ m["Vt"][:k, :].T)
    PA -= PA.mean(0); PB -= PB.mean(0); N = len(PA)
    Caa, Cab, Cbb = PA.T @ PA / N, PA.T @ PB / N, PB.T @ PB / N
    r = 1e-6 * (np.trace(Caa) + np.trace(Cbb)) / (2 * k)
    return float(np.linalg.norm(inv_sqrt(Caa, r) @ Cab @ inv_sqrt(Cbb, r), "fro") ** 2)
def ho(chains, k, cols, null=False, seed=0):
    rng = np.random.default_rng(seed); out = []
    for j in range(4):
        tr = [c for i, c in enumerate(chains) if i % 4 != j]; te = [c for i, c in enumerate(chains) if i % 4 == j]
        trA, trB = pairs(tr); teA, teB = pairs(te)
        if null: trB = trB[rng.permutation(len(trB))]; teB = teB[rng.permutation(len(teB))]   # cross-chain re-pairing
        out.append(heldout(trA, trB, teA, teB, k, cols))
    return float(np.mean(out))
def blocks(A, B, ca, cb):
    keep = ca + cb; m = vamp(A, B, keep)
    K = np.linalg.solve(m["C00"] + 1e-6 * np.trace(m["C00"]) / len(keep) * np.eye(len(keep)), m["C0t"])
    ia = list(range(len(ca))); ib = list(range(len(ca), len(keep)))
    kab = np.linalg.norm(K[np.ix_(ia, ib)], "fro"); kba = np.linalg.norm(K[np.ix_(ib, ia)], "fro")
    return math.sqrt(kab ** 2 + kba ** 2) / np.linalg.norm(K, "fro"), kab, kba
if __name__ == "__main__":
    chains = load(); A, B = pairs(chains)
    print(f"# REASON-SEARCH-0 under Amendment 1: {len(chains)} chains (length dist {collections.Counter(min(len(c),8) for c in chains).most_common(7)}), {len(A)} parent->child transitions, dictionary {A.shape[1]}")
    inc = float(np.mean(np.abs((B[:, 4] - A[:, 4]) - 1.0) < 1e-9)); print(f"   links are real: depth increments by one on {inc:.3f} of transitions")
    m = vamp(A, B); print(f"   lag-0 check: max |σ−1| = {np.abs(vamp(A, A)['s'] - 1).max():.1e} (PR-B, < 1e-2)")
    print(f"   dictionary top σ: {np.array2string(m['s'], precision=3)}")
    for i in range(3):
        w = m["W0"] @ m["U"][:, i]; top = np.argsort(-np.abs(w))[:3]; print(f"   σ_{i+1} = {m['s'][i]:.3f}: " + ", ".join(f"{NAMES[j]} {w[j]:+.2f}" for j in top))
    rows = []
    for name, k, cols in [("depth alone", 1, DEPTH), ("DMA sector", 4, DMA), ("process sector", 3, PROC), ("all", 7, None)]:
        real = ho(chains, k, cols); bound = ho(chains, k, None); nul = ho(chains, k, cols, null=True)
        rows.append((name, k, real, bound, nul)); print(f"   {name:16} k={k}  held-out {real:.3f}  bound_k {bound:.3f}  fraction {real/bound:.2f}  re-paired null {nul:.3f}  real/null {real/max(nul,1e-9):.2f}")
    cross, kpd, kdp = blocks(A, B, PROC, DMA)
    print(f"   blocks: cross ‖K_PD‖/‖K‖ = {cross:.3f}; ‖K_P→D‖ = {kpd:.3f}, ‖K_D→P‖ = {kdp:.3f}, ratio = {kpd/max(kdp,1e-9):.2f}")
    d = rows[0][2]; dma_r = rows[1][2] / max(rows[1][4], 1e-9); allnull = rows[3][4] / rows[3][3]
    print(f"   PR-A: re-paired null of the full dictionary {allnull:.2f} of the bound (stake < 0.5) -> {'PASS' if allnull < 0.5 else 'FAIL'}")
    print(f"   S1 depth alone ≥ 0.9: {d:.3f} -> {'MET' if d >= 0.9 else ('KILL' if d < 0.5 else 'between')}")
    print(f"   S2 cross-block < 0.2 (kill ≥ 0.4): {cross:.3f} -> {'MET' if cross < 0.2 else ('KILL' if cross >= 0.4 else 'between')}")
    print(f"   S3 DMA real/null ≥ 3 (kill < 1.5): {dma_r:.2f} -> {'MET' if dma_r >= 3 else ('KILL' if dma_r < 1.5 else 'between')}")
    r = kpd / max(kdp, 1e-9); print(f"   S4 P→D / D→P ≥ 2 (kill ≤ 0.5): {r:.2f} -> {'MET' if r >= 2 else ('KILL' if r <= 0.5 else 'undecided')}")
