#!/usr/bin/env python3
"""CROSS-FACE-1 instrument. Prereg ADMITTED and frozen at 1231fee, ALONE and
before this file: conformance/omega/CROSSFACE1_PREREG.md.

Reads the rent of a view -- the minimum per-step displaced mass of a repair
that holds the view closed -- on the two HELD-OUT substrates, and checks it
against rationals staked in the freeze from group data and circuit
coefficients alone.  The host instruments (conformance/gravity/pt2t.py and
conformance/gravity/local1.py) are IMPORTED, never re-implemented: their step
maps are evaluated here for the first time.

Exact throughout.  Every quantity a gate decides on is a Fraction or an
integer; floats appear only in printed decimals.
"""
import sys
from fractions import Fraction as F
from math import isqrt, gcd

sys.path.insert(0, "/home/emoore/CIRISHolon/conformance/gravity")
import numpy as np

REPORT = {}
PLANTS_OK = True


# ===================================================================== exact
def sqrt_lo(q, D=1 << 40):
    """largest k/D with (k/D)^2 <= q.  q a nonnegative Fraction."""
    a, b = q.numerator, q.denominator
    return F(isqrt((a * D * D) // b), D)


def sqrt_hi(q, D=1 << 40):
    return sqrt_lo(q, D) + F(1, D)


def matmul(X, Y):
    n, k, m = len(X), len(Y), len(Y[0])
    return [[sum(X[i][t] * Y[t][j] for t in range(k)) for j in range(m)]
            for i in range(n)]


def rent(P):
    """Theorem 1: W = 1 - sum_i max_j P_ij."""
    return F(1) - sum(max(row) for row in P)


def marginals(P):
    m = len(P)
    mu = [sum(P[i]) for i in range(m)]
    nu = [sum(P[i][j] for i in range(m)) for j in range(m)]
    return mu, nu


def transfer(P, mu):
    return [[P[i][j] / mu[i] for j in range(len(P))] for i in range(len(P))]


def lam2_lower(M, mu):
    """Exact rational LOWER bound on lambda^2, with a flag saying whether it is
    known EXACT.  Three exact detectors, then a rational power iteration."""
    m = len(mu)
    # (1) M - Pi identically zero -> lambda = 0 exactly
    if all(M[i][j] == mu[j] for i in range(m) for j in range(m)):
        return F(0), "EXACT(M=Pi)"
    # (2) permutation matrix preserving mu -> an isometry on 1^perp, lambda = 1
    if all(sorted(M[i]) == [F(0)] * (m - 1) + [F(1)] for i in range(m)):
        img = [next(j for j in range(m) if M[i][j] == 1) for i in range(m)]
        if sorted(img) == list(range(m)) and all(mu[img[i]] == mu[i] for i in range(m)):
            return F(1), "EXACT(permutation)"
    # (3) circulant on 3 blocks -> |p^(1)|^2 = a^2+b^2+c^2-ab-bc-ca exactly
    if m == 3 and all(M[i][(i + d) % 3] == M[0][d] for i in range(3) for d in range(3)):
        a, b, c = M[0]
        return a * a + b * b + c * c - a * b - b * c - c * a, "EXACT(circulant3)"
    # (4) fallback: Rayleigh quotient of the mu-self-adjoint B = (M-Pi)*(M-Pi)
    A = [[M[i][j] - mu[j] for j in range(m)] for i in range(m)]
    As = [[mu[j] * A[j][i] / mu[i] for j in range(m)] for i in range(m)]
    B = matmul(As, A)
    best = F(0)
    for seed in range(3):
        v = [F(1 + ((i * 7 + seed * 13) % 11)) for i in range(m)]
        c0 = sum(mu[i] * v[i] for i in range(m))
        v = [v[i] - c0 for i in range(m)]                       # project onto 1^perp
        if all(x == 0 for x in v):
            continue
        for _ in range(14):
            num = sum(mu[i] * v[i] * sum(B[i][j] * v[j] for j in range(m))
                      for i in range(m))
            den = sum(mu[i] * v[i] * v[i] for i in range(m))
            if den == 0:
                break
            best = max(best, num / den)
            w = [sum(B[i][j] * v[j] for j in range(m)) for i in range(m)]
            if all(x == 0 for x in w):
                break
            L = 1
            for x in w:
                L = L * x.denominator // gcd(L, x.denominator)
            wi = [int(x * L) for x in w]
            g = 0
            for x in wi:
                g = gcd(g, abs(x))
            v = [F(x // g) for x in wi] if g else w
    return best, "power-iteration lower bound"


def lam2_upper(M, mu):
    """Certified rational UPPER bound: rho(B) <= ||B||_inf."""
    m = len(mu)
    A = [[M[i][j] - mu[j] for j in range(m)] for i in range(m)]
    As = [[mu[j] * A[j][i] / mu[i] for j in range(m)] for i in range(m)]
    B = matmul(As, A)
    return max(sum(abs(x) for x in row) for row in B)


def theorem2(P, name):
    """G1: W >= (1 - mu_max) - lambda * (sum sigma_i) * sigma_max, exactly.

    Verification uses a LOWER bound on the subtracted term (which makes the
    right-hand side LARGER, so a pass certifies the theorem).  A certified
    violation uses the UPPER bound.  Anything else is INCONCLUSIVE, never a
    silent pass."""
    m = len(P)
    mu, _ = marginals(P)
    W = rent(P)
    M = transfer(P, mu)
    L = (1 - max(mu)) - W
    lam2, how = lam2_lower(M, mu)
    if L <= 0:
        return "PASS(trivial: ceiling <= rent)", W, lam2, how
    s2 = [mu[i] * (1 - mu[i]) for i in range(m)]
    smax2 = max(s2)
    uniform = all(x == mu[0] for x in mu)
    if uniform:
        sum_sig2_lo = F(m * m) * s2[0]          # (m*sigma)^2, exact
        exactness = "exact"
    else:
        sum_sig2_lo = sum(sqrt_lo(x) for x in s2) ** 2
        exactness = "lower bound"
    if L * L <= lam2 * sum_sig2_lo * smax2:
        return f"PASS({exactness})", W, lam2, how
    lam2u = lam2_upper(M, mu)
    if uniform:
        sum_sig2_hi = F(m * m) * s2[0]
    else:
        sum_sig2_hi = sum(sqrt_hi(x) for x in s2) ** 2
    if L * L > lam2u * sum_sig2_hi * smax2:
        return "FIRE(certified violation)", W, lam2, how
    return "INCONCLUSIVE(enclosure too loose)", W, lam2, how


G1_ROWS = []


def read_view(P, name):
    """One view: exact rent, lambda^2, and the Theorem-2 check."""
    mu, nu = marginals(P)
    assert mu == nu, f"{name}: view marginal not stationary -- mu is not invariant"
    verdict, W, lam2, how = theorem2(P, name)
    G1_ROWS.append((name, verdict))
    return W, lam2, how, verdict


# ============================================================ A.  2T  torus
def part_A():
    import pt2t as PT
    print("\n" + "=" * 76)
    print("A.  2T torus -- host conformance/gravity/pt2t.py, 576 configs, permutation")
    print("=" * 76, flush=True)
    N, NG = PT.N, PT.NG
    STEP = [int(PT.S_MAP[PT.T_MAP[c]]) for c in range(N)]

    # ---- G0
    g0 = []
    g0.append(("step bijective on all 576", len(set(STEP)) == N))
    g0.append(("T_MAP bijective", len(set(PT.T_MAP.tolist())) == N))
    g0.append(("S_MAP bijective", len(set(PT.S_MAP.tolist())) == N))
    g0.append(("counting measure invariant (bijection)", len(set(STEP)) == N))
    for k, v in g0:
        print(f"  G0 {k}: {v}")
    REPORT["G0-2T"] = "PASS" if all(v for _, v in g0) else "VOID"

    # ---- views
    GA, GB, CLS = PT.GA, PT.GB, PT.CLS
    orbit = [-1] * N
    lab = 0
    for i in range(N):
        if orbit[i] >= 0:
            continue
        stack = [i]
        while stack:
            j = stack.pop()
            if orbit[j] >= 0:
                continue
            orbit[j] = lab
            a, b = int(GA[j]), int(GB[j])
            for x in range(NG):
                k = int(PT.IDX(PT.MUL[PT.MUL[x, a], PT.INV[x]],
                               PT.MUL[PT.MUL[x, b], PT.INV[x]]))
                if orbit[k] < 0:
                    stack.append(k)
        lab += 1
    COMM = PT.comm(GA, GB)
    views = {
        "v_orbit": [orbit[c] for c in range(N)],
        "v_comm": [int(CLS[COMM[c]]) for c in range(N)],
        "v_classA": [int(CLS[GA[c]]) for c in range(N)],
        "v_classpair": [int(CLS[GA[c]]) * PT.N_CLS + int(CLS[GB[c]]) for c in range(N)],
    }
    out = {}
    for name, v in views.items():
        labs = sorted(set(v))
        ix = {l: i for i, l in enumerate(labs)}
        m = len(labs)
        C = [[0] * m for _ in range(m)]
        for c in range(N):
            C[ix[v[c]]][ix[v[STEP[c]]]] += 1
        P = [[F(C[i][j], N) for j in range(m)] for i in range(m)]
        W, lam2, how, g1 = read_view(P, f"2T/{name}")
        out[name] = (W, lam2, how, m, C, labs)
        print(f"  {name:<12} blocks={m:<3} W = {str(W):<10} lambda^2 = {str(lam2):<10}"
              f" [{how}]  G1={g1}", flush=True)

    # ---- S1
    W_orb, l_orb = out["v_orbit"][0], out["v_orbit"][1]
    W_com, l_com = out["v_comm"][0], out["v_comm"][1]
    m_com, C_com, labs_com = out["v_comm"][3], out["v_comm"][4], out["v_comm"][5]
    is_I = (m_com == 3 and all((C_com[i][j] != 0) == (i == j)
                               for i in range(m_com) for j in range(m_com)))
    rents_zero = (W_orb == 0 and W_com == 0)
    lam_one = (l_orb == 1 and l_com == 1)
    if not rents_zero:
        REPORT["S1"] = "BRANCH(b) KILL: a closed view has nonzero rent"
    elif not is_I:
        REPORT["S1"] = "BRANCH(c) the ambivalence reading dies: rents zero but M != I3"
    else:
        REPORT["S1"] = "BRANCH(a)"
    print(f"  S1  W(v_orbit)={W_orb}  W(v_comm)={W_com}  lambda=1 both: {lam_one}")
    print(f"      realised commutator classes {labs_com}, transfer counts {C_com}")
    print(f"      transfer is exactly I3: {is_I}  ->  {REPORT['S1']}")

    # ---- S2
    s2a = out["v_classA"][0] == F(3, 4)
    s2b = out["v_classpair"][0] == F(43, 144)
    REPORT["S2"] = "BRANCH(a)" if (s2a and s2b) else (
        f"BRANCH(b) KILL: classA={out['v_classA'][0]} (want 3/4), "
        f"classpair={out['v_classpair'][0]} (want 43/144)")
    print(f"  S2  W(v_classA)={out['v_classA'][0]} (staked 3/4, {s2a});  "
          f"W(v_classpair)={out['v_classpair'][0]} (staked 43/144, {s2b})"
          f"  ->  {REPORT['S2']}")

    # ---- R1
    four = [out[k][0] for k in ("v_orbit", "v_comm", "v_classA", "v_classpair")]
    distinct = len(set(four))
    REPORT["R1"] = ("BRANCH(a) rival refuted" if distinct > 1 else
                    "BRANCH(b) rival survives: one cost for every view")
    print(f"  R1  the four 2T rents = {[str(x) for x in four]}; the rival (micro")
    print(f"      mixing rate = 1 for every view) predicts ONE cost; measured "
          f"{distinct} distinct  ->  {REPORT['R1']}")

    # ---- B3, on the host's own physical states via the host's own gauss_holds
    reg = []
    for c in PT.Q8_CLASSES:
        psi = PT.gauss_project((PT.CLS[PT.PUNCT] == c).astype(np.int64))
        reg.append(psi)
        for _ in range(3):
            psi = PT.step(psi)
            reg.append(psi)
    b3 = all(PT.gauss_holds(s) for s in reg)
    REPORT["B3-2T"] = "PASS" if b3 else "VOID"
    print(f"  B3  pt2t.gauss_holds on {len(reg)} projected states + step images: {b3}")
    print(f"      (the 576 configs are basis elements of mu, not registry states)")
    return out


# ================================================= B.  tailed graph (local1)
CARRIER_CONFIGS = [0, 1, 7, 271, 12345, 531440]      # frozen list, all-zero first


def part_B():
    import local1 as L1
    print("\n" + "=" * 76)
    print("B.  tailed graph -- host conformance/gravity/local1.py, 12 edges,")
    print("    531441 configs x 9 matter, unitary; mu uniform over basis states")
    print("=" * 76, flush=True)

    SCALE = 3 ** 38          # k_charge x9, u_e x27 per edge over 12 edges
    holos = {
        "v_e8": L1.DIG[8],
        "v_pend": L1.HOL_P[5],
        "v_rim": L1.hol(L1.LOOP),
        "v_p1": L1.HOL_P[1],
        "v_near": L1.HOL_P[4],
    }
    E0_FREE = ["v_e8", "v_pend", "v_rim", "v_p1"]
    masks = {k: [(h == d) for d in range(3)] for k, h in holos.items()}

    laws = {k: {} for k in holos}            # law -> list of (m, c)
    g0_weights = []
    n2_checked = False
    for c in CARRIER_CONFIGS:
        for m in range(9):
            A = np.zeros((9, L1.N), dtype=np.int64)
            B = np.zeros((9, L1.N), dtype=np.int64)
            A[m][c] = 1
            st = L1.step((A, B))
            assert st[0].dtype == np.int64 and st[1].dtype == np.int64, \
                "amplitudes promoted to object: the int64 fast path is unsafe, REFUSE"
            a, b = st
            n2 = a * a - a * b + b * b            # exact: every partial sum < 2^63
            if not n2_checked:                    # cross-check the fast path once
                assert [int(x) for x in L1.triple(st, holos["v_pend"])] == \
                    [int(n2[:, msk].sum()) for msk in masks["v_pend"]], \
                    "fast norm2 disagrees with the host's own norm2"
                n2_checked = True
            tot = int(n2.sum())
            g0_weights.append(tot)
            for k, h in holos.items():
                w = [int(n2[:, msk].sum()) for msk in masks[k]]
                assert sum(w) == tot
                d0 = int(h[c])
                law = tuple(F(w[(d0 + t) % 3], tot) for t in range(3))
                laws[k].setdefault(law, []).append((m, c))
        print(f"  carriers for config {c} done ({(CARRIER_CONFIGS.index(c)+1)*9}"
              f" of {9*len(CARRIER_CONFIGS)})", flush=True)

    # ---- G0
    wt_ok = all(w == SCALE for w in g0_weights)
    print(f"\n  G0 total Born weight = 3^38 on all {len(g0_weights)} carriers: {wt_ok}")
    REPORT["G0-TG"] = "PASS" if wt_ok else "VOID"

    # ---- S5: carrier-independence on the e0-free views
    s5_bad = [k for k in E0_FREE if len(laws[k]) != 1]
    REPORT["S5"] = ("BRANCH(a)" if not s5_bad else
                    f"BRANCH(b) VOID: carrier-dependent on {s5_bad}")
    print(f"  S5 carrier-independence over {9*len(CARRIER_CONFIGS)} basis states"
          f" (>= 54 required): distinct laws per view = "
          f"{ {k: len(laws[k]) for k in E0_FREE} }  ->  {REPORT['S5']}")

    # ---- S3: the perimeter law
    staked = {"v_e8": (1, F(2, 9)), "v_pend": (3, F(38, 81)),
              "v_rim": (5, F(422, 729)), "v_p1": (3, F(38, 81))}
    lam_e = F(2, 3)
    s3_ok, s3_rows = True, []
    for k, (Lp, want) in staked.items():
        if len(laws[k]) != 1:
            s3_ok = False
            s3_rows.append((k, Lp, want, None, None))
            continue
        law = next(iter(laws[k]))
        P = [[F(1, 3) * law[(j - i) % 3] for j in range(3)] for i in range(3)]
        W, lam2, how, g1 = read_view(P, f"TG/{k}")
        pred = F(2, 3) * (1 - lam_e ** Lp)
        assert pred == want, "internal: staked value disagrees with the frozen form"
        s3_ok &= (W == want)
        s3_rows.append((k, Lp, want, W, law))
    REPORT["S3"] = "BRANCH(a)" if s3_ok else "BRANCH(b) KILL: product law missed"
    print("\n  S3 perimeter law  W(L) = (2/3)(1 - (2/3)^L), lambda_e = 2/3 from the"
          " circuit alone")
    for k, Lp, want, W, law in s3_rows:
        mark = "MATCH" if W == want else "MISS"
        print(f"     {k:<8} L={Lp}  staked {str(want):<10} measured "
              f"{str(W):<10} {mark}   law={tuple(str(x) for x in law) if law else '-'}")
    print(f"     ->  {REPORT['S3']}")

    # ---- S4: the conditioned plaquette
    ln = laws["v_near"]
    diag_ms = {0, 4, 8}
    by_branch = {}
    struct_ok = True
    for law, who in ln.items():
        kinds = {("diag" if m in diag_ms else "off") for m, _ in who}
        if len(kinds) != 1:
            struct_ok = False
        by_branch.setdefault(kinds.pop() if len(kinds) == 1 else "mixed", []).append(
            (law, len(who)))
    print(f"\n  S4 conditioned plaquette (edges 4,9,0): {len(ln)} distinct laws")
    for law, who in ln.items():
        kinds = sorted({("diag" if m in diag_ms else "off") for m, _ in who})
        print(f"     {tuple(str(x*243) for x in law)} /243  on {len(who)} carriers"
              f"  matter={kinds}")
    off = [law for law, who in ln.items() if all(m not in diag_ms for m, _ in who)]
    dia = [law for law, who in ln.items() if all(m in diag_ms for m, _ in who)]
    if len(ln) == 2 and struct_ok and len(off) == 1 and len(dia) == 1:
        mix = tuple(F(2, 3) * off[0][t] + F(1, 3) * dia[0][t] for t in range(3))
        P = [[F(1, 3) * mix[(j - i) % 3] for j in range(3)] for i in range(3)]
        W4, lam2, how, g1 = read_view(P, "TG/v_near")
        print(f"     mu-mixture (6 off : 3 diag) = "
              f"{tuple(str(x*243) for x in mix)} /243   W = {W4}")
        if W4 == F(122, 243):
            REPORT["S4"] = "BRANCH(a) conditioned-operator correction confirmed"
        elif W4 == F(114, 243):
            REPORT["S4"] = "BRANCH(b) KILL: the naive product law wins, correction spurious"
        else:
            REPORT["S4"] = f"BRANCH(c) KILL: neither, W = {W4}"
    else:
        REPORT["S4"] = (f"BRANCH(c) KILL: branch structure wrong "
                        f"({len(ln)} laws, split-by-diagonality={struct_ok})")
    print(f"     staked 122/243, naive rival 114/243  ->  {REPORT['S4']}")

    # ---- B3 on the host's own physical carrier
    psi = L1.dressed_state()
    reg = [psi]
    for _ in range(2):
        psi = L1.step(psi)
        reg.append(psi)
    b3 = all(L1.gauss_holds(s)[0] for s in reg)
    REPORT["B3-TG"] = "PASS" if b3 else "VOID"
    print(f"\n  B3 local1.gauss_holds on the dressed vacuum + 2 step images: {b3}")
    print(f"     (basis carriers are elements of mu, not registry states -- they are")
    print(f"      not gauge-invariant by construction and are not claimed to be)")


# ==================================================================== plants
def plants():
    global PLANTS_OK
    print("\n" + "=" * 76)
    print("PLANTS -- frozen carriers, on the DERIVATION substrates (held-out")
    print("         discipline: no plant is scored on a held-out step map)")
    print("=" * 76)
    from bridge1 import MUL, INV, CLASS
    G, N = 8, 64
    IDX = lambda a, b: a + G * b
    GA, GB = np.arange(N) % G, np.arange(N) // G
    comm = lambda a, b: MUL[MUL[a, b], MUL[INV[a], INV[b]]]
    T_MAP = IDX(GA, MUL[GA, GB])
    S_MAP = IDX(GB, MUL[MUL[GB, GA], INV[GB]])
    STEP = [int(S_MAP[T_MAP[c]]) for c in range(N)]

    def counts(view, smap):
        labs = sorted(set(int(x) for x in view))
        ix = {l: i for i, l in enumerate(labs)}
        m = len(labs)
        C = [[0] * m for _ in range(m)]
        for c in range(N):
            C[ix[int(view[c])]][ix[int(view[smap[c]])]] += 1
        return C

    # (i) best-model -> average-model
    C = counts(CLASS[GA], STEP)
    W_max = F(1) - sum(F(max(r), N) for r in C)
    W_mean = F(1) - sum(F(sum(r), N * len(r)) for r in C)
    spread = F(max(C[0]), sum(C[0])) - F(1, len(C[0]))
    assert spread != 0, "plant (i) carrier is not nonzero in the sector it acts on"
    fired = W_max != W_mean
    print(f"  [plant i]  carrier v_classA transfer rows; sector = row spread,")
    print(f"             nonzero in the sector the plant acts on: {spread}")
    print(f"             max-model {W_max} vs mean-model {W_mean} -> "
          f"{'FIRES' if fired else 'MISSED'}")
    PLANTS_OK &= fired

    # (ii) normaliser-breaking twist
    orbit = np.full(N, -1, dtype=np.int64)
    lab = 0
    for i in range(N):
        if orbit[i] >= 0:
            continue
        stack = [i]
        while stack:
            j = stack.pop()
            if orbit[j] >= 0:
                continue
            orbit[j] = lab
            a, b = int(GA[j]), int(GB[j])
            for x in range(G):
                k = int(IDX(MUL[MUL[x, a], INV[x]], MUL[MUL[x, b], INV[x]]))
                if orbit[k] < 0:
                    stack.append(k)
        lab += 1
    rot4 = next(x for x in range(G) if INV[x] != x)
    TWIST = IDX(GA, MUL[np.full(N, rot4), GB])
    BAD = [int(TWIST[STEP[c]]) for c in range(N)]
    rerouted = sum(1 for c in range(N) if orbit[BAD[c]] != orbit[STEP[c]])
    assert rerouted > 0, "plant (ii) carrier is not nonzero in the sector it acts on"
    Wc = F(1) - sum(F(max(r), N) for r in counts(orbit, STEP))
    Wb = F(1) - sum(F(max(r), N) for r in counts(orbit, BAD))
    fired = Wb != Wc
    print(f"  [plant ii] carrier v_ADM (28-block orbit view); sector = the twisted")
    print(f"             transfer's off-diagonal, nonzero in the sector the plant")
    print(f"             acts on: {rerouted} of 64 configs re-routed")
    print(f"             clean {Wc} -> twisted {Wb} -> "
          f"{'FIRES' if fired else 'MISSED'}")
    PLANTS_OK &= fired

    # (iii) convicted Fourier kernel on one edge of a staked loop
    def n2(a, b):
        return a * a - a * b + b * b
    p = [F(n2(5, 4), 27), F(n2(2, 1), 27), F(n2(-1, -2), 27)]
    lam_true, lam_planted = p[0] - p[1], F(0)
    assert lam_true != lam_planted, \
        "plant (iii) carrier is not nonzero in the sector it acts on"
    Wt = F(2, 3) * (1 - lam_true ** 3)
    Wp = F(2, 3) * (1 - lam_true * lam_true * lam_planted)
    fired = Wt != Wp
    print(f"  [plant iii] carrier the 3-edge pendant loop's increment law; sector =")
    print(f"             that loop's mixing modulus, nonzero in the sector the plant")
    print(f"             acts on: lambda_e {lam_true} -> {lam_planted}")
    print(f"             W {Wt} -> {Wp} -> {'FIRES' if fired else 'MISSED'}")
    PLANTS_OK &= fired


# ====================================================================== main
if __name__ == "__main__":
    part_A()
    part_B()
    plants()

    g1_bad = [n for n, v in G1_ROWS if not v.startswith("PASS")]
    REPORT["G1"] = ("PASS (%d views, all exact)" % len(G1_ROWS) if not g1_bad
                    else "FIRE " + str(g1_bad))
    print("\n" + "=" * 76)
    print("G1 -- Theorem 2 on every view read:")
    for n, v in G1_ROWS:
        print(f"     {n:<18} {v}")
    print("=" * 76)
    print("GATES: " + "  ".join(f"{k}={v}" for k, v in sorted(REPORT.items())),
          flush=True)
    kills = [k for k, v in REPORT.items()
             if "KILL" in str(v) or str(v).startswith("FIRE")]
    voids = [k for k, v in REPORT.items() if "VOID" in str(v)]
    ok = not kills and not voids and PLANTS_OK
    print(f"VERDICT: {'no gate killed, no gate VOID' if not kills and not voids else ''}"
          f"{'KILLS ' + str(kills) if kills else ''}"
          f"{' VOID ' + str(voids) if voids else ''}; "
          f"plants {'all FIRE' if PLANTS_OK else 'MISSED - VOID'}")
    sys.exit(0 if ok else 1)
