"""ruleb.py — GENERATED.  Do not edit.

RULE-B, extracted by make_ruleb.py from git blob cbaf2b47 (sha256 af00cf2c3512735e…) --
the version that produced SELECTOR5_REFUTATION.md's numbers, before the edit
that made the label read a construction tag.  Regenerate, never edit.
"""
#!/usr/bin/env python3
"""refute_lib.py — adversarial instrument for SELECTOR-5.

Computes, FROM THE GROUP and never from a construction tag:
  * the complex character table (Burnside-Dixon, numeric simultaneous
    diagonalisation of the class algebra),
  * the kernel of every irreducible representation,
  * the determinant character of every irreducible representation,
  * RULE-B: whether G embeds in SU(3) x SU(2) x U(1).

RULE-B is frozen in FROZEN_LABEL_RULE.md (sha256
0a4b956587135da4d57786285728dfc131ca477abbc3317800921bb3bad0d913).
"""
import math
import numpy as np


# ---------------------------------------------------------------- characters

def class_data(G):
    n = G.order
    classes = G.classes
    r = len(classes)
    CLS = np.empty(n, dtype=np.int64)
    reps = []
    for i, c in enumerate(classes):
        for x in c:
            CLS[x] = i
        reps.append(min(c))
    sizes = np.array([len(c) for c in classes], dtype=float)
    return classes, CLS, np.array(reps, dtype=np.int64), sizes


def class_matrices(G, classes, CLS, reps):
    """A[i][j][k] = a_ijk with K_i K_j = sum_k a_ijk K_k."""
    r = len(classes)
    MUL, INV = G.MUL, G.INV
    A = np.zeros((r, r, r))
    for i in range(r):
        xs = np.fromiter(sorted(classes[i]), dtype=np.int64)
        prod = MUL[INV[xs][:, None], reps[None, :]]     # (|C_i|, r)
        cl = CLS[prod]
        for k in range(r):
            A[i, :, k] = np.bincount(cl[:, k], minlength=r)
    return A


def character_table(G, seed=20260830):
    """Returns (chars, degrees, sizes, CLS, reps).  chars[t, i] = chi_t(g_i)."""
    classes, CLS, reps, sizes = class_data(G)
    r = len(classes)
    n = float(G.order)
    if r == G.order:                       # abelian: characters are the duals
        A = class_matrices(G, classes, CLS, reps)
    else:
        A = class_matrices(G, classes, CLS, reps)
    Ms = [A[i].T for i in range(r)]        # M_i[k, j] = a_ijk
    rng = np.random.default_rng(seed)
    for attempt in range(8):
        c = rng.normal(size=r) + 1j * rng.normal(size=r)
        M = np.tensordot(c, np.array(Ms), axes=(0, 0))
        w, V = np.linalg.eig(M)
        # separation check
        ws = np.sort_complex(w)
        if r == 1 or np.min(np.abs(np.diff(ws))) > 1e-7:
            break
    chars = np.zeros((r, r), dtype=complex)
    degs = np.zeros(r)
    Marr = np.array(Ms)
    for t in range(r):
        v = V[:, t]
        p = int(np.argmax(np.abs(v)))
        omega = np.einsum('ikj,j->ik', Marr, v)[:, p] / v[p]
        s = np.sum(np.abs(omega) ** 2 / sizes)
        d = math.sqrt(n / s.real)
        degs[t] = d
        chars[t] = d * omega / sizes
    order = np.argsort(np.round(degs, 6), kind='stable')
    return chars[order], degs[order], sizes, CLS, reps


def validate_table(G, chars, degs, sizes, tol=1e-5):
    n = float(G.order)
    if abs(np.sum(np.round(degs) ** 2) - n) > 1e-6:
        return False, "degree sum"
    if np.max(np.abs(degs - np.round(degs))) > 1e-4:
        return False, "non-integer degree"
    W = chars * sizes[None, :]
    gram = W @ chars.conj().T
    if np.max(np.abs(gram - n * np.eye(len(degs)))) > 1e-3 * n:
        return False, "orthogonality"
    return True, "ok"


# ------------------------------------------------------- kernels and det char

def kernels(G, chars, degs, CLS):
    """Bitmask over group elements, per irreducible."""
    n = G.order
    out = []
    for t in range(len(degs)):
        keep = np.abs(chars[t] - degs[t]) < 1e-4
        mask = 0
        cls_keep = np.flatnonzero(keep)
        sel = np.isin(CLS, cls_keep)
        for g in np.flatnonzero(sel):
            mask |= (1 << int(g))
        out.append(mask)
    return out


def elem_order(G, g):
    o, cur = 1, g
    while cur != G.IDE:
        cur = int(G.MUL[cur, g])
        o += 1
    return o


def det_characters(G, chars, degs, CLS, reps):
    """det of each irrep, as a complex vector over classes."""
    r = len(degs)
    dets = np.zeros((r, r), dtype=complex)
    # precompute powers of each class rep
    powidx = []
    for k in range(r):
        g = int(reps[k])
        m = elem_order(G, g)
        seq, cur = [], G.IDE
        for _ in range(m):
            seq.append(CLS[cur])
            cur = int(G.MUL[cur, g])
        powidx.append((m, np.array(seq)))
    for t in range(r):
        for k in range(r):
            m, seq = powidx[k]
            vals = chars[t][seq]                       # chi(g^0..g^{m-1})
            j = np.arange(m)
            w = np.exp(-2j * np.pi * np.outer(j, j) / m)
            mult = np.real_if_close((w @ vals) / m)
            mult = np.round(np.real(mult)).astype(int)
            e = int(np.sum(j * mult)) % m
            dets[t, k] = np.exp(2j * np.pi * e / m)
    return dets


# ------------------------------------------------------------------- RULE-B

def _lin_index(dets_row, lin_chars, tol=1e-4):
    for idx, lc in lin_chars:
        if np.max(np.abs(dets_row - lc)) < tol:
            return idx
    return None


def rule_b_sm(G, cache=None):
    """RULE-B: does G embed in SU(3) x SU(2) x U(1)?  Returns (bool, note)."""
    n = G.order
    idbit = 1 << G.IDE
    # abelian shortcut: embeds iff rank <= 4 (rank(SU3 x SU2 x U1) = 4)
    if len(G.commutator_subgroup) == 1:
        rk = 0
        for p in (2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47,
                  53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107,
                  109, 113, 127):
            if n % p:
                continue
            eo = G.element_orders
            cnt = sum(1 for o in eo if o in (1, p))
            rk = max(rk, round(math.log(cnt, p)))
        return (rk <= 4), f"abelian rank {rk}"

    chars, degs, sizes, CLS, reps = character_table(G)
    ok, why = validate_table(G, chars, degs, sizes)
    if not ok:
        return None, f"character table failed: {why}"
    degs_i = np.round(degs).astype(int)
    kers = kernels(G, chars, degs, CLS)
    dets = det_characters(G, chars, degs, CLS, reps)
    lin = [(t, chars[t]) for t in range(len(degs_i)) if degs_i[t] == 1]
    lin_idx_of = {}
    for t in range(len(degs_i)):
        li = _lin_index(dets[t], lin)
        if li is None:
            return None, "det character not linear (numeric failure)"
        lin_idx_of[t] = li
    # multiplication table on linear characters
    linmul = {}
    for (a, ca) in lin:
        for (b, cb) in lin:
            linmul[(a, b)] = _lin_index(ca * cb, lin)
    triv = _lin_index(np.ones(len(degs_i)), lin)
    full = (1 << n) - 1

    def achievable(budget):
        """kernels of reps of total degree <= budget with trivial determinant"""
        seen = {(budget, full, triv)}
        frontier = [(budget, full, triv)]
        res = set()
        while frontier:
            nxt = []
            for (b, k, d) in frontier:
                if d == triv:
                    res.add(k)
                for t in range(len(degs_i)):
                    if degs_i[t] <= b:
                        st = (b - degs_i[t], k & kers[t], linmul[(d, lin_idx_of[t])])
                        if st not in seen:
                            seen.add(st)
                            nxt.append(st)
            frontier = nxt
        return res

    K3 = achievable(3)
    K2 = achievable(2)
    K1 = set(kers[t] for (t, _) in lin) | {full}
    for k3 in K3:
        for k2 in K2:
            a = k3 & k2
            if a == idbit:
                return True, "embeds"
            for k1 in K1:
                if a & k1 == idbit:
                    return True, "embeds"
    return False, "no embedding"
