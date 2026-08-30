#!/usr/bin/env python3
"""refute_population.py — build the group families SELECTOR-5's census omitted.

Principled, not cherry-picked: every SPLIT metacyclic group Z_m : Z_k with
mk <= 128 (this strictly generalises five of their eleven families -- Cyclic,
Dihedral, Frobenius, Semidihedral, ModularGroup), every Z_p^2 : Z_k and
Z_2^3 : Z_k realisable by a matrix action, and the extraspecial 2-groups of
order 32 and 128 (central products), which their census contains at order 8
only.
"""
import itertools, math
import numpy as np
import landscape_sweep as ls
from landscape_sweep import FiniteGroup


def from_table(name, family, elems, mulfn, is_lie=False, notes=""):
    idx = {e: i for i, e in enumerate(elems)}
    N = len(elems)
    MUL = np.zeros((N, N), dtype=np.int64)
    for i, a in enumerate(elems):
        for j, b in enumerate(elems):
            MUL[i, j] = idx[mulfn(a, b)]
    IDE = None
    for i in range(N):
        if all(MUL[i, j] == j for j in range(N)):
            IDE = i
            break
    assert IDE is not None, name
    INV = np.zeros(N, dtype=np.int64)
    for i in range(N):
        INV[i] = next(j for j in range(N) if MUL[i, j] == IDE)
    return FiniteGroup(name=name, family=family, order=N, MUL=MUL, INV=INV,
                       IDE=IDE, is_lie_type=is_lie, notes=notes)


def metacyclic(m, k, t):
    """Z_m : Z_k with generator acting as x -> x^t."""
    elems = [(a, b) for b in range(k) for a in range(m)]
    tp = [pow(t, b, m) for b in range(k)]
    def mul(x, y):
        return ((x[0] * 1 + tp[x[1]] * y[0]) % m, (x[1] + y[1]) % k)
    return from_table(f"Z_{m}:{t}:Z_{k}", "Metacyclic", elems, mul,
                      notes=f"split metacyclic Z_{m} : Z_{k}, action x -> x^{t}")


def mat_semidirect(p, d, M, k, tag):
    """(Z_p)^d : Z_k where the generator acts by matrix M in GL(d,p)."""
    elems = [(v, b) for b in range(k) for v in itertools.product(range(p), repeat=d)]
    Mp = [np.eye(d, dtype=np.int64)]
    for _ in range(k - 1):
        Mp.append((Mp[-1] @ M) % p)
    def mul(x, y):
        v = (np.array(x[0]) + Mp[x[1]] @ np.array(y[0])) % p
        return (tuple(int(c) for c in v), (x[1] + y[1]) % k)
    return from_table(tag, "AffineSemidirect", elems, mul,
                      notes=f"(Z_{p})^{d} : Z_{k}")


def mat_order(M, p):
    d = M.shape[0]
    A = M.copy() % p
    I = np.eye(d, dtype=np.int64) % p
    o = 1
    while not np.array_equal(A, I):
        A = (A @ M) % p
        o += 1
        if o > 400:
            return None
    return o


def extraspecial_2(n, plus=True):
    """Extraspecial 2-group of order 2^(2n+1): central product of n copies of D8
    (plus type) or (n-1) D8 with one Q8 (minus type).  Realised as the Pauli
    group modulo phases: elements (a, b, c) in F_2^n x F_2^n x F_2."""
    N = 2 ** (2 * n + 1)
    elems = [(a, b, c) for a in itertools.product(range(2), repeat=n)
             for b in itertools.product(range(2), repeat=n)
             for c in range(2)]
    def mul(x, y):
        a = tuple((x[0][i] + y[0][i]) % 2 for i in range(n))
        b = tuple((x[1][i] + y[1][i]) % 2 for i in range(n))
        c = (x[2] + y[2] + sum(x[1][i] * y[0][i] for i in range(n))) % 2
        return (a, b, c)
    G = from_table(f"ES2_{N}{'+' if plus else '-'}", "Extraspecial", elems, mul,
                   notes=f"extraspecial 2-group of order {N} (Pauli group mod phases)")
    return G


def build_extension():
    out = []
    # split metacyclic Z_m : Z_k
    for m in range(2, 129):
        for k in range(2, 129 // m + 1):
            seen_t = set()
            for t in range(1, m):
                if math.gcd(t, m) != 1:
                    continue
                if pow(t, k, m) != 1:
                    continue
                if t in seen_t:
                    continue
                seen_t.add(t)
                out.append(metacyclic(m, k, t))
    # (Z_p)^2 : Z_k
    for p in (2, 3, 5, 7, 11):
        mats = []
        for a in range(p):
            for b in range(p):
                for c in range(p):
                    for d in range(p):
                        if (a * d - b * c) % p == 0:
                            continue
                        mats.append(np.array([[a, b], [c, d]], dtype=np.int64))
        used = set()
        for M in mats:
            k = mat_order(M, p)
            if k is None or k < 2 or p * p * k > 128:
                continue
            key = (p, k, tuple(M.flatten()))
            if key in used:
                continue
            used.add(key)
            out.append(mat_semidirect(p, 2, M, k, f"Z_{p}^2:Z_{k}[{M[0,0]}{M[0,1]}{M[1,0]}{M[1,1]}]"))
    # (Z_2)^3 : Z_k  and (Z_2)^4 : Z_k
    for d in (3, 4):
        p = 2
        allm = []
        for bits in range(2 ** (d * d)):
            M = np.array([(bits >> i) & 1 for i in range(d * d)], dtype=np.int64).reshape(d, d)
            det = round(np.linalg.det(M)) % 2
            if det == 0:
                continue
            allm.append(M)
        used = set()
        for M in allm:
            k = mat_order(M, 2)
            if k is None or k < 2 or (2 ** d) * k > 128:
                continue
            sig = (k, tuple(M.flatten()))
            if sig in used:
                continue
            used.add(sig)
            out.append(mat_semidirect(2, d, M, k, f"Z_2^{d}:Z_{k}[{int(''.join(map(str,M.flatten())),2)}]"))
    # extraspecial 2-groups of order 32 and 128
    for n in (2, 3):
        out.append(extraspecial_2(n))
    return out


def merged_landscape():
    base = ls.generate_landscape()
    ext = build_extension()
    seen = {}
    merged = []
    for g in base:
        sig = g.structural_signature()
        if sig not in seen:
            seen[sig] = g
            merged.append(g)
    added = 0
    for g in ext:
        sig = g.structural_signature()
        if sig not in seen:
            seen[sig] = g
            merged.append(g)
            added += 1
        else:
            e = seen[sig]
            if g.name not in e.aliases and g.name != e.name:
                e.aliases.append(g.name)
    merged.sort(key=lambda g: (g.order, g.name))
    return merged, added


if __name__ == "__main__":
    m, added = merged_landscape()
    print(f"base 430 + {added} genuinely new isomorphism types = {len(m)}")
    import collections
    c = collections.Counter(g.family for g in m)
    for k in sorted(c):
        print(f"  {k:18s} {c[k]}")
    print("\norders 64 and 128 present:",
          sum(1 for g in m if g.order == 64), sum(1 for g in m if g.order == 128))
