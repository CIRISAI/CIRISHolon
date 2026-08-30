#!/usr/bin/env python3
"""census.py — SELECTOR-6's census: every isomorphism type of order 1..63.

COMPLETENESS IS A THEOREM, NOT A FAMILY LIST.  Every group with a nontrivial
abelianization has a subgroup of prime index in G/G', whose preimage is normal
of prime index in G; so every NON-PERFECT group of order n is a cyclic extension
of a group of order n/p by Z_p, for some prime p | n.  Building those extensions
recursively from the trivial group is therefore complete for the non-perfect
groups at every order.  The only perfect group of order 2..63 is A5 (order 60),
which is added by hand with its perfection verified rather than asserted.

Cyclic extension.  Given N and a prime p, every extension 1 -> N -> G -> Z_p -> 1
is realised as G = { n t^i } with

    (a t^i)(b t^j) = a . alpha^i(b) . [c if i+j >= p else e] . t^((i+j) mod p)

for alpha in Aut(N) and c in N with alpha(c) = c and alpha^p = conj_c.  Both
conditions are checked per candidate; associativity of every ADMITTED table is
then verified exhaustively (never assumed from the theorem).

S1 gate: per-order count must equal the pinned A000001 value, else that order
VOIDs for every claim.  The pin is cross-audited by two internal theorems
(exact abelian counts from partition products; Holder's formula on squarefree
orders) so that a pinned constant is not an unchecked constant.
"""
import itertools
import json
import os
import sys
import time
from math import gcd

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
PIN = os.path.join(HERE, "A000001.pin")
MAXORD = int(os.environ.get('SELECTOR6_MAXORD', '63'))

t0 = time.time()


def say(s):
    print(f"[{time.time()-t0:8.1f}s] {s}", flush=True)


# --------------------------------------------------------------- group object

class Grp:
    __slots__ = ("n", "MUL", "INV", "IDE", "_ord", "_cls", "_ctr", "_der", "_fp",
                 "_iso")

    def __init__(self, MUL):
        self.MUL = np.asarray(MUL, dtype=np.int16)
        self.n = self.MUL.shape[0]
        n = self.n
        ide = -1
        for g in range(n):
            if all(self.MUL[g, x] == x for x in range(n)):
                ide = g
                break
        if ide < 0:
            raise ValueError("no identity")
        self.IDE = ide
        INV = np.full(n, -1, dtype=np.int16)
        for a in range(n):
            for b in range(n):
                if self.MUL[a, b] == ide:
                    INV[a] = b
                    break
        if (INV < 0).any():
            raise ValueError("missing inverse")
        self.INV = INV
        self._ord = self._cls = self._ctr = self._der = self._fp = None
        self._iso = None

    # ---- basic invariants
    @property
    def orders(self):
        if self._ord is None:
            n, M, ide = self.n, self.MUL, self.IDE
            out = np.empty(n, dtype=np.int16)
            for g in range(n):
                o, cur = 1, g
                while cur != ide:
                    cur = int(M[cur, g])
                    o += 1
                out[g] = o
            self._ord = out
        return self._ord

    @property
    def classes(self):
        if self._cls is None:
            n, M, I = self.n, self.MUL, self.INV
            lab = np.full(n, -1, dtype=np.int16)
            out = []
            for g in range(n):
                if lab[g] >= 0:
                    continue
                orb = {int(M[M[x, g], I[x]]) for x in range(n)}
                for h in orb:
                    lab[h] = len(out)
                out.append(sorted(orb))
            self._cls = out
        return self._cls

    @property
    def center(self):
        if self._ctr is None:
            n, M = self.n, self.MUL
            self._ctr = [g for g in range(n)
                         if all(M[g, x] == M[x, g] for x in range(n))]
        return self._ctr

    def subgroup(self, gens):
        M, ide = self.MUL, self.IDE
        S = {ide}
        frontier = [ide]
        gl = list(gens)
        while frontier:
            nxt = []
            for x in frontier:
                for a in gl:
                    y = int(M[x, a])
                    if y not in S:
                        S.add(y)
                        nxt.append(y)
            frontier = nxt
        return S

    def derived(self, elems=None):
        M, I = self.MUL, self.INV
        E = range(self.n) if elems is None else list(elems)
        comm = set()
        for a in E:
            for b in E:
                comm.add(int(M[M[a, b], M[I[a], I[b]]]))
        return self.subgroup(comm)

    @property
    def derived_series(self):
        if self._der is None:
            cur = set(range(self.n))
            out = []
            while True:
                nxt = self.derived(cur)
                out.append(len(nxt))
                if nxt == cur or len(nxt) == 1:
                    break
                cur = nxt
            self._der = out
        return self._der

    def abelianization_profile(self):
        """The multiset of element orders of G/G'.

        For a finite ABELIAN group the element-order multiset is a COMPLETE
        isomorphism invariant, since |{a : a^(p^k) = e}| = p^(sum_i min(l_i, k))
        determines the partition l of each primary part.  So this is sound as a
        fingerprint component -- and unlike an invariant-factor extraction it
        cannot loop.  An earlier draft divided a running order by the largest
        remaining element order; on Z_2 x Z_2 that largest order becomes 1 and
        the division never terminates.  The census hung at order 4.
        """
        Gp = self.derived()
        M = self.MUL
        cos, cid = [], {}
        for g in range(self.n):
            if g in cid:
                continue
            c = frozenset(int(M[g, x]) for x in Gp)
            k = len(cos)
            cos.append(c)
            for x in c:
                cid[x] = k
        m = len(cos)
        amul = np.zeros((m, m), dtype=np.int32)
        for i, ci in enumerate(cos):
            gi = next(iter(ci))
            for j, cj in enumerate(cos):
                gj = next(iter(cj))
                amul[i, j] = cid[int(M[gi, gj])]
        ide = cid[self.IDE]
        aord = []
        for i in range(m):
            o, cur = 1, i
            while cur != ide:
                cur = int(amul[cur, i])
                o += 1
            aord.append(o)
        return tuple(sorted(aord))

    @property
    def fingerprint(self):
        if self._fp is None:
            ordv = self.orders
            cls = self.classes
            cls_sig = sorted((len(c), int(ordv[c[0]])) for c in cls)
            ctr = self.center
            ctr_sig = sorted(int(ordv[g]) for g in ctr)
            # power-map statistic: for each class, (size, order, order of square)
            M = self.MUL
            pw = sorted((len(c), int(ordv[c[0]]), int(ordv[int(M[c[0], c[0]])]))
                        for c in cls)
            self._fp = (self.n,
                        tuple(sorted(int(x) for x in ordv)),
                        tuple(cls_sig),
                        tuple(ctr_sig),
                        tuple(pw),
                        len(self.derived()),
                        tuple(self.derived_series),
                        self.abelianization_profile())
        return self._fp

    def is_abelian(self):
        return len(self.derived()) == 1

    def check_associative(self):
        M = self.MUL
        n = self.n
        A = np.asarray(M, dtype=np.int32)
        for a in range(n):
            left = A[A[a, :], :]            # (a*b)*c
            right = A[a, A]                 # a*(b*c)
            if not np.array_equal(left, right):
                return False
        return True


# ---------------------------------------------------------------- isomorphism

def minimal_gens(G):
    n = G.n
    gens, cur = [], {G.IDE}
    order_desc = sorted(range(n), key=lambda g: -int(G.orders[g]))
    for g in order_desc:
        if g not in cur:
            gens.append(g)
            cur = G.subgroup(gens)
            if len(cur) == n:
                break
    return gens


def word_table(G, gens):
    """each element as a word (tuple of generator indices), by BFS"""
    M = G.MUL
    words = {G.IDE: ()}
    frontier = [G.IDE]
    while frontier:
        nxt = []
        for x in frontier:
            for gi, a in enumerate(gens):
                y = int(M[x, a])
                if y not in words:
                    words[y] = words[x] + (gi,)
                    nxt.append(y)
        frontier = nxt
    return words


def _map_from_images(G, H, words, images):
    M = H.MUL
    phi = {}
    for e, w in words.items():
        cur = H.IDE
        for gi in w:
            cur = int(M[cur, images[gi]])
        phi[e] = cur
    return phi


def iso_data(G):
    """generators, words and subgroup sizes -- computed ONCE per group, not once
    per candidate partner (that recomputation was the census's first bottleneck)."""
    if G._iso is None:
        gens = minimal_gens(G)
        words = word_table(G, gens)
        sub_sizes = [len(G.subgroup(gens[:i + 1])) for i in range(len(gens))]
        G._iso = (gens, words, sub_sizes)
    return G._iso


def isomorphism(G, H):
    """explicit isomorphism G -> H as a list, or None"""
    if G.n != H.n or G.fingerprint != H.fingerprint:
        return None
    n = G.n
    gens, words, sub_sizes = iso_data(G)
    Ho = H.orders
    cands = [[h for h in range(n) if Ho[h] == G.orders[g]] for g in gens]

    res = [None]

    def rec(i, images):
        if res[0] is not None:
            return
        if i == len(gens):
            phi = _map_from_images(G, H, words, images)
            if len(set(phi.values())) != n:
                return
            GM, HM = G.MUL, H.MUL
            for a in range(n):
                pa = phi[a]
                for b in range(n):
                    if HM[pa, phi[b]] != phi[int(GM[a, b])]:
                        return
            res[0] = [phi[x] for x in range(n)]
            return
        for h in cands[i]:
            if len(H.subgroup(images + [h])) != sub_sizes[i]:
                continue
            rec(i + 1, images + [h])
            if res[0] is not None:
                return

    rec(0, [])
    return res[0]


# --------------------------------------------------------------- automorphisms

def automorphisms(G):
    n = G.n
    gens, words, sub_sizes = iso_data(G)
    Go = G.orders
    cands = [[h for h in range(n) if Go[h] == Go[g]] for g in gens]
    M = G.MUL
    out = []

    def rec(i, images):
        if i == len(gens):
            phi = _map_from_images(G, G, words, images)
            if len(set(phi.values())) != n:
                return
            arr = np.empty(n, dtype=np.int16)
            for k, v in phi.items():
                arr[k] = v
            # verify homomorphism exactly
            ok = True
            for a in range(n):
                if not np.array_equal(M[arr[a], arr], arr[M[a]]):
                    ok = False
                    break
            if ok:
                out.append(arr)
            return
        for h in cands[i]:
            if len(G.subgroup(images + [h])) != sub_sizes[i]:
                continue
            rec(i + 1, images + [h])

    rec(0, [])
    return out


# ------------------------------------------------------------ cyclic extension

def cyclic_extensions(N, p):
    """all G with normal N of index p, as MUL tables"""
    n = N.n
    M, I, ide = N.MUL, N.INV, N.IDE
    out = []
    for alpha in automorphisms(N):
        # alpha^p
        ap = alpha.copy()
        for _ in range(p - 1):
            ap = alpha[ap]
        for c in range(n):
            if alpha[c] != c:
                continue
            # alpha^p == conj_c ?
            if not np.array_equal(ap, M[M[c, np.arange(n)], I[c]]):
                continue
            # powers of alpha
            pw = [np.arange(n, dtype=np.int16)]
            for _ in range(p - 1):
                pw.append(alpha[pw[-1]])
            NN = n * p
            T = np.zeros((NN, NN), dtype=np.int16)
            for i in range(p):
                ai = pw[i]
                for j in range(p):
                    k = i + j
                    carry = (k >= p)
                    kk = k % p
                    # rows a+n*i, cols b+n*j
                    block = M[:, ai]                      # block[a,b] = a*alpha^i(b)
                    if carry:
                        block = M[block, c]
                    T[i * n:(i + 1) * n, j * n:(j + 1) * n] = block + n * kk
            out.append(T)
    return out


# ------------------------------------------------------------------ base cases

def cyclic_group(m):
    a = np.arange(m)
    return Grp((a[:, None] + a[None, :]) % m)


def alt5():
    perms = [p for p in itertools.permutations(range(5))
             if sum(1 for i in range(5) for j in range(i + 1, 5) if p[i] > p[j]) % 2 == 0]
    idx = {p: i for i, p in enumerate(perms)}
    n = len(perms)
    M = np.zeros((n, n), dtype=np.int16)
    for i, a in enumerate(perms):
        for j, b in enumerate(perms):
            M[i, j] = idx[tuple(a[b[k]] for k in range(5))]
    return Grp(M)


# ------------------------------------------------------------------- the audits

def partitions(k):
    """p(k), by the standard DP."""
    dp = [1] + [0] * k
    for part in range(1, k + 1):
        for v in range(part, k + 1):
            dp[v] += dp[v - part]
    return dp[k]


def factorize(n):
    f, d = {}, 2
    while d * d <= n:
        while n % d == 0:
            f[d] = f.get(d, 0) + 1
            n //= d
        d += 1
    if n > 1:
        f[n] = f.get(n, 0) + 1
    return f


def abelian_count(n):
    r = 1
    for _, a in factorize(n).items():
        r *= partitions(a)
    return r


def holder_count(n):
    """Holder's formula: number of groups of squarefree order n."""
    f = factorize(n)
    if any(a > 1 for a in f.values()):
        return None
    primes = sorted(f)
    tot = 0
    for d in [d for d in range(1, n + 1) if n % d == 0]:
        m = n // d
        if gcd(d, m) != 1:
            continue
        prod = 1
        for p in primes:
            if d % p == 0:
                cnt = sum(1 for q in primes if m % q == 0 and (q - 1) % p == 0)
                prod *= (p ** cnt - 1) // (p - 1)
        tot += prod
    return tot


def load_pin():
    vals = {}
    with open(PIN) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            a, b = line.split()
            vals[int(a)] = int(b)
    return vals


# ------------------------------------------------------------------------ main

def main():
    pin = load_pin()
    say(f"A000001 pin loaded: {len(pin)} rows, A000001(63)={pin[63]}")

    groups = {1: [Grp(np.zeros((1, 1), dtype=np.int16))]}
    counts, voids = {1: 1}, []
    dup_demo = []

    for n in range(2, MAXORD + 1):
        found = []
        fps = {}
        ncand = 0
        for p in sorted(factorize(n)):
            m = n // p
            for N in groups[m]:
                for T in cyclic_extensions(N, p):
                    ncand += 1
                    try:
                        G = Grp(T)
                    except ValueError:
                        continue
                    fp = G.fingerprint
                    bucket = fps.setdefault(fp, [])
                    if any(isomorphism(G, H) is not None for H in bucket):
                        continue
                    bucket.append(G)
                    found.append(G)
        if n == 60:
            A = alt5()
            if len(A.derived()) != A.n:
                raise SystemExit("VOID: A5 is not perfect as constructed")
            fp = A.fingerprint
            bucket = fps.setdefault(fp, [])
            if not any(isomorphism(A, H) is not None for H in bucket):
                bucket.append(A)
                found.append(A)
                say("  order 60: A5 admitted by hand (perfection verified)")
        # associativity of every admitted table, exhaustively
        for G in found:
            if not G.check_associative():
                raise SystemExit(f"VOID: non-associative table admitted at order {n}")
        groups[n] = found
        counts[n] = len(found)
        want = pin[n]
        flag = "" if len(found) == want else f"   <-- S1 VOID (pin says {want})"
        if len(found) != want:
            voids.append(n)
        ab = sum(1 for G in found if G.is_abelian())
        abw = abelian_count(n)
        abflag = "" if ab == abw else f"  ABELIAN-AUDIT VOID (theorem says {abw})"
        say(f"order {n:3d}: {len(found):4d} types from {ncand:6d} candidates"
            f"  (abelian {ab}/{abw}){flag}{abflag}")

    say("")
    say("=== S1 GATE ===")
    tot = sum(counts.values())
    say(f"census total, orders 1..{MAXORD}: {tot}")
    say(f"pin total,    orders 1..{MAXORD}: {sum(pin[n] for n in range(1, MAXORD+1))}")
    say(f"orders VOIDed by S1: {voids if voids else 'NONE'}")

    say("")
    say("=== PIN CROSS-AUDIT, leg 2: exact abelian counts ===")
    bad = [n for n in range(1, MAXORD + 1)
           if sum(1 for G in groups[n] if G.is_abelian()) != abelian_count(n)]
    say(f"orders where the census abelian count != partition-product theorem: "
        f"{bad if bad else 'NONE'}")

    say("")
    say("=== PIN CROSS-AUDIT, leg 3: Holder on squarefree orders ===")
    hbad = []
    for n in range(1, MAXORD + 1):
        h = holder_count(n)
        if h is not None and h != pin[n]:
            hbad.append((n, h, pin[n]))
    say(f"squarefree orders where Holder != pin: {hbad if hbad else 'NONE'}")

    out = {str(n): [G.MUL.tolist() for G in groups[n]] for n in groups}
    with open(os.path.join(HERE, "census.json"), "w") as f:
        json.dump({"counts": counts, "voids": voids, "tables": out}, f)
    with open(os.path.join(HERE, "census.DONE"), "w") as f:
        f.write("done\n")
    say(f"census written ({tot} groups)")


if __name__ == "__main__":
    main()
