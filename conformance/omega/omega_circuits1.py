#!/usr/bin/env python3
"""OMEGA-CIRCUITS-1 instrument.  Prereg ADMITTED and hash-frozen BEFORE this
file existed: OMEGA_CIRCUITS1_PREREG_DRAFT.md, sha256 in FREEZE.sha256.

Reads the rent of a view -- the frozen CROSS-FACE-1 minimum per-step displaced
mass of a repair that holds the view closed -- on the CIRISHolon engine's own
qubit-circuit substrates, and checks it against values staked in the freeze
from circuit coefficients alone.

Exact throughout.  Amplitudes live in the engine's own ring
`ledger::Cyc = (c0 + c1*w + c2*w^2 + c3*w^3) * 2^{-m/2}`, w = e^{i*pi/4},
mirrored coefficient-for-coefficient.  Born weights land in Z[1/2][sqrt2] and
are carried as exact `Quad` values p + q*sqrt2 with Fraction p, q.  Every
quantity a gate decides on is exact; floats appear only in printed decimals.
"""
import sys, os, json, random, subprocess
from fractions import Fraction as F

HERE = os.path.dirname(os.path.abspath(__file__))
REPORT = {}
G1_ROWS = []
PLANTS = []


# ===================================================================== ring
# Cyc: coefficients over 1, w, w^2, w^3 with w^4 = -1, times 2^{-m/2}.
def cyc_mul_poly(a, b):
    r = [0, 0, 0, 0]
    for i in range(4):
        ai = a[i]
        if ai == 0:
            continue
        for j in range(4):
            bj = b[j]
            if bj == 0:
                continue
            k = i + j
            if k < 4:
                r[k] += ai * bj
            else:
                r[k - 4] -= ai * bj
    return r


def cyc_conj(a):
    """w -> w^{-1}: conj(1)=1, conj(w)=-w^3, conj(w^2)=-w^2, conj(w^3)=-w."""
    return [a[0], -a[3], -a[2], -a[1]]


def cyc_mul_w(a, k):
    """multiply the polynomial by w^k, k in 0..7."""
    r = [0, 0, 0, 0]
    for i in range(4):
        if a[i] == 0:
            continue
        j = i + k
        s = 1
        while j >= 4:
            j -= 4
            s = -s
        r[j] += s * a[i]
    return r


class Quad:
    """p + q*sqrt2, exact."""
    __slots__ = ("p", "q")

    def __init__(self, p=0, q=0):
        self.p = F(p)
        self.q = F(q)

    def __add__(self, o):
        o = quad(o)
        return Quad(self.p + o.p, self.q + o.q)

    def __sub__(self, o):
        o = quad(o)
        return Quad(self.p - o.p, self.q - o.q)

    def __neg__(self):
        return Quad(-self.p, -self.q)

    def __mul__(self, o):
        o = quad(o)
        return Quad(self.p * o.p + 2 * self.q * o.q, self.p * o.q + self.q * o.p)

    def sign(self):
        p, q = self.p, self.q
        if p == 0 and q == 0:
            return 0
        if p >= 0 and q >= 0:
            return 1
        if p <= 0 and q <= 0:
            return -1
        # opposite signs: compare p^2 with 2 q^2
        if p > 0:                      # q < 0 : p + q*sqrt2 > 0 iff p^2 > 2q^2
            return 1 if p * p > 2 * q * q else (0 if p * p == 2 * q * q else -1)
        else:                          # p < 0 < q
            return 1 if 2 * q * q > p * p else (0 if 2 * q * q == p * p else -1)

    def __eq__(self, o):
        o = quad(o)
        return self.p == o.p and self.q == o.q

    def __lt__(self, o):
        return (self - quad(o)).sign() < 0

    def __le__(self, o):
        return (self - quad(o)).sign() <= 0

    def __hash__(self):
        return hash((self.p, self.q))

    def is_rat(self):
        return self.q == 0

    def __float__(self):
        return float(self.p) + float(self.q) * 1.4142135623730951

    def __repr__(self):
        if self.q == 0:
            return str(self.p)
        return f"({self.p} + {self.q}*sqrt2)"


def quad(x):
    if isinstance(x, Quad):
        return x
    return Quad(x, 0)


ZERO, ONE = Quad(0), Quad(1)


def qmax(vals):
    b = vals[0]
    for v in vals[1:]:
        if b < v:
            b = v
    return b


# ============================================================== statevector
# A state is (amps, m): amps a list of 2^n coefficient lists, global 2^{-m/2}.
# Qubit q occupies bit (n-1-q) of the basis index, so |s_0 s_1 ... s_{n-1}>
# reads left to right, matching the freeze's notation.
def bit(s, q, n):
    return (s >> (n - 1 - q)) & 1


def basis_state(n, s):
    amps = [[0, 0, 0, 0] for _ in range(1 << n)]
    amps[s][0] = 1
    return [amps, 0]


def apply_gate(st, g, n):
    amps, m = st
    kind = g[0]
    if kind == "H":
        q = g[1]
        b = 1 << (n - 1 - q)
        for s in range(1 << n):
            if s & b:
                continue
            a0, a1 = amps[s], amps[s | b]
            amps[s] = [a0[i] + a1[i] for i in range(4)]
            amps[s | b] = [a0[i] - a1[i] for i in range(4)]
        st[1] = m + 1
    elif kind in ("S", "Sdg", "T", "Tdg", "Z", "X"):
        q = g[1]
        b = 1 << (n - 1 - q)
        if kind == "X":
            for s in range(1 << n):
                if s & b:
                    continue
                amps[s], amps[s | b] = amps[s | b], amps[s]
            return
        k = {"S": 2, "Sdg": 6, "T": 1, "Tdg": 7, "Z": 4}[kind]
        for s in range(1 << n):
            if s & b:
                amps[s] = cyc_mul_w(amps[s], k)
    elif kind == "CX":
        c, t = g[1], g[2]
        bc, bt = 1 << (n - 1 - c), 1 << (n - 1 - t)
        for s in range(1 << n):
            if (s & bc) and not (s & bt):
                amps[s], amps[s | bt] = amps[s | bt], amps[s]
    elif kind == "CZ":
        c, t = g[1], g[2]
        bc, bt = 1 << (n - 1 - c), 1 << (n - 1 - t)
        for s in range(1 << n):
            if (s & bc) and (s & bt):
                amps[s] = cyc_mul_w(amps[s], 4)
    else:
        raise ValueError(f"unknown gate {g}")


def evolve(n, circuit, s):
    st = basis_state(n, s)
    for g in circuit:
        apply_gate(st, g, n)
    return st


def born_column(n, circuit, s):
    """|<s'|U|s>|^2 for every s', exact Quad."""
    amps, m = evolve(n, circuit, s)
    den = F(1, 1 << m)
    out = []
    for a in amps:
        p = cyc_mul_poly(a, cyc_conj(a))
        assert p[2] == 0 and p[3] == -p[1], f"Born weight not real: {p}"
        out.append(Quad(F(p[0]) * den, F(p[1]) * den))
    return out


def born_kernel(n, circuit):
    """K[s][s'] exact."""
    return [born_column(n, circuit, s) for s in range(1 << n)]


# ==================================================== Clifford conjugation
# Heisenberg push of a Pauli's (x, z) bit-vectors through the circuit; signs
# are irrelevant because only X-parts are used.  T/Tdg are refused, exactly as
# adaptive.rs refuses them.
def conjugate(n, circuit, x, z):
    x, z = list(x), list(z)
    for g in circuit:
        k = g[0]
        if k == "H":
            q = g[1]
            x[q], z[q] = z[q], x[q]
        elif k in ("S", "Sdg"):
            q = g[1]
            z[q] ^= x[q]
        elif k in ("X", "Z"):
            pass
        elif k == "CX":
            c, t = g[1], g[2]
            x[t] ^= x[c]
            z[c] ^= z[t]
        elif k == "CZ":
            c, t = g[1], g[2]
            z[t] ^= x[c]
            z[c] ^= x[t]
        else:
            raise ValueError("non-Clifford step has no (A, V): " + str(g))
    return x, z


def vec_to_int(v, n):
    r = 0
    for q in range(n):
        if v[q]:
            r |= 1 << (n - 1 - q)
    return r


def structural_AV(n, circuit):
    """A's columns and V's generators, from the CIRCUIT alone (no kernel)."""
    Acols = []
    for i in range(n):
        e = [0] * n
        e[i] = 1
        x, _ = conjugate(n, circuit, e, [0] * n)
        Acols.append(vec_to_int(x, n))
    Vgens = []
    for i in range(n):
        e = [0] * n
        e[i] = 1
        x, _ = conjugate(n, circuit, [0] * n, e)
        Vgens.append(vec_to_int(x, n))
    return Acols, Vgens


# ================================================ F2 linear algebra on ints
def span_basis(vecs):
    basis = []
    for v in vecs:
        for b in basis:
            v = min(v, v ^ b)
        if v:
            basis.append(v)
            basis.sort(reverse=True)
    return basis


def in_span(v, basis):
    for b in basis:
        v = min(v, v ^ b)
    return v == 0


def dim(vecs):
    return len(span_basis(vecs))


def subgroup_elements(basis):
    out = [0]
    for b in basis:
        out += [x ^ b for x in out]
    return sorted(out)


# ===================================================================== views
class View:
    def __init__(self, name, n, fn, nblocks, linear_B=None):
        self.name = name
        self.n = n
        self.fn = fn
        self.nblocks = nblocks
        self.B = linear_B          # list of n-bit masks, one per output bit
        self.blocks = [[] for _ in range(nblocks)]
        for s in range(1 << n):
            self.blocks[fn(s)].append(s)

    def apply_B(self, s):
        r = 0
        m = len(self.B)
        for i, mask in enumerate(self.B):
            r |= (bin(s & mask).count("1") & 1) << (m - 1 - i)
        return r


def view_full(n):
    return View(f"full basis (m={n})", n, lambda s: s, 1 << n,
                linear_B=[1 << (n - 1 - q) for q in range(n)])


def view_marginal(n, qubits):
    B = [1 << (n - 1 - q) for q in qubits]
    m = len(qubits)

    def f(s):
        r = 0
        for i, q in enumerate(qubits):
            r |= bit(s, q, n) << (m - 1 - i)
        return r
    return View(f"marginal on {qubits}", n, f, 1 << m, linear_B=B)


def view_parity(n, qubits):
    mask = 0
    for q in qubits:
        mask |= 1 << (n - 1 - q)
    return View(f"parity of {qubits}", n,
                lambda s: bin(s & mask).count("1") & 1, 2, linear_B=[mask])


def view_weight_threshold(n, qubits, thr=1):
    def f(s):
        w = sum(bit(s, q, n) for q in qubits)
        return 0 if w <= thr else 1
    return View(f"weight-threshold(<= {thr}) on {qubits}", n, f, 2)


def view_hamming(n):
    return View("Hamming weight", n, lambda s: bin(s).count("1"), n + 1)


def view_product(name, n, va, vb):
    """(va, vb) as one view; block index = va*nb + vb."""
    nb = vb.nblocks
    return View(name, n, lambda s: va.fn(s) * nb + vb.fn(s), va.nblocks * nb)


# ============================================================== face reading
def face_P(n, K, view):
    """P_ij = Pr[X=i, Y=j], exact."""
    N = view.nblocks
    P = [[ZERO] * N for _ in range(N)]
    w = F(1, 1 << n)
    for i, blk in enumerate(view.blocks):
        row = P[i]
        for s in blk:
            col = K[s]
            for sp in range(1 << n):
                k = col[sp]
                if k.p or k.q:
                    j = view.fn(sp)
                    row[j] = row[j] + k * quad(w)
    return P


def marginals(P):
    N = len(P)
    mu = [ZERO] * N
    nu = [ZERO] * N
    for i in range(N):
        for j in range(N):
            mu[i] = mu[i] + P[i][j]
            nu[j] = nu[j] + P[i][j]
    return mu, nu


def rent(P):
    """Theorem 1: W = 1 - sum_i max_j P_ij."""
    tot = ZERO
    for row in P:
        tot = tot + qmax(row)
    return ONE - tot


def transfer(P, mu):
    N = len(P)
    M = [[ZERO] * N for _ in range(N)]
    for i in range(N):
        if mu[i].p == 0 and mu[i].q == 0:
            continue
        inv = quad_inv(mu[i])
        for j in range(N):
            M[i][j] = P[i][j] * inv
    return M


def quad_inv(a):
    """1/(p+q sqrt2) = (p - q sqrt2)/(p^2 - 2q^2)."""
    d = a.p * a.p - 2 * a.q * a.q
    assert d != 0
    return Quad(a.p / d, -a.q / d)


def lam_exact(M, mu):
    """Exact lambda = ||M - Pi|| on 1^perp, with the method that certified it."""
    N = len(M)
    # (1) M == Pi
    if all(M[i][j] == mu[j] for i in range(N) for j in range(N)):
        return ZERO, "EXACT(M=Pi)"
    # (2) convolution over F_2^m (block index IS the group element)
    m = N.bit_length() - 1
    uniform = all(x == mu[0] for x in mu)
    if (1 << m) == N and uniform:
        conv = True
        f = [M[0][j] for j in range(N)]
        for x in range(N):
            for y in range(N):
                if M[x][y] != f[x ^ y]:
                    conv = False
                    break
            if not conv:
                break
        if conv:
            best = ZERO
            for u in range(1, N):
                acc = ZERO
                for z in range(N):
                    if bin(u & z).count("1") & 1:
                        acc = acc - f[z]
                    else:
                        acc = acc + f[z]
                a = acc if acc.sign() >= 0 else -acc
                if best < a:
                    best = a
            return best, "EXACT(convolution on F_2^m)"
    # (3) projector test: B = (M-Pi)^*(M-Pi) in l^2(mu); B == 0 -> 0,
    #     B idempotent and nonzero -> 1.  No structure assumed.
    A = [[M[i][j] - mu[j] for j in range(N)] for i in range(N)]
    B = [[ZERO] * N for _ in range(N)]
    for i in range(N):
        inv = quad_inv(mu[i])
        for j in range(N):
            acc = ZERO
            for k in range(N):
                acc = acc + mu[k] * A[k][i] * A[k][j]
            B[i][j] = acc * inv
    if all(B[i][j] == ZERO for i in range(N) for j in range(N)):
        return ZERO, "EXACT(B=0)"
    B2 = [[ZERO] * N for _ in range(N)]
    for i in range(N):
        for j in range(N):
            acc = ZERO
            for k in range(N):
                acc = acc + B[i][k] * B[k][j]
            B2[i][j] = acc
    if all(B2[i][j] == B[i][j] for i in range(N) for j in range(N)):
        return ONE, "EXACT(B idempotent, nonzero)"
    return None, "NOT QUANTIZED (B neither 0 nor idempotent)"


def sqrt_lo(q, D=1 << 40):
    from math import isqrt
    a, b = q.numerator, q.denominator
    return F(isqrt((a * D * D) // b), D)


def sqrt_hi(q, D=1 << 40):
    return sqrt_lo(q, D) + F(1, D)


def theorem2(P, W, lam, mu, name):
    """G1: W >= (1 - mu_max) - lam*(sum sigma_i)*sigma_max, exactly."""
    N = len(P)
    mumax = qmax(mu)
    L = (ONE - mumax) - W
    if L.sign() == 0:
        return "PASS(TIGHT: equality)"
    if L.sign() < 0:
        return "PASS(strict: ceiling < rent)"
    if lam is None:
        return "INCONCLUSIVE(lambda not certified)"
    if lam == ZERO:
        return "FIRE(certified violation, lambda=0)"
    assert all(x.is_rat() for x in mu), "non-uniform irrational mu unsupported"
    if all(x == mu[0] for x in mu):
        # uniform mu: (sum sigma_i)*sigma_max = N*sigma^2 = 1 - 1/N, exact,
        # so the whole bound is (1 - 1/N)(1 - lam) with no square root at all.
        sub = lam * (ONE - mu[0])
        if L <= sub:
            return "PASS(TIGHT: equality)" if L == sub else "PASS(exact)"
        return "FIRE(certified violation)"
    s2 = [x.p * (1 - x.p) for x in mu]
    sub_lo = lam * quad(sum(sqrt_lo(x) for x in s2)) * quad(max(sqrt_lo(x) for x in s2))
    if L <= sub_lo:
        return "PASS(exact)"
    sub_hi = lam * quad(sum(sqrt_hi(x) for x in s2)) * quad(max(sqrt_hi(x) for x in s2))
    if sub_hi < L:
        return "FIRE(certified violation)"
    return "INCONCLUSIVE(enclosure too loose)"


def read_face(n, K, view, tag):
    P = face_P(n, K, view)
    mu, nu = marginals(P)
    assert all(mu[i] == nu[i] for i in range(len(mu))), \
        f"{tag}: view marginal not stationary"
    W = rent(P)
    M = transfer(P, mu)
    lam, how = lam_exact(M, mu)
    G1_ROWS.append((tag, theorem2(P, W, lam, mu, tag)))
    return W, lam, how, mu, M


def predicted_h(n, circuit, view):
    """h = dim(BA(ker B) + BV), from the CIRCUIT alone."""
    Acols, Vgens = structural_AV(n, circuit)
    m = len(view.B)
    kerB = [s for s in range(1 << n) if view.apply_B(s) == 0]
    gens = []
    for s in kerB:
        As = 0
        for q in range(n):
            if bit(s, q, n):
                As ^= Acols[q]
        gens.append(view.apply_B(As))
    for v in Vgens:
        gens.append(view.apply_B(v))
    return dim(gens), m


def dyadic(h):
    return ONE - Quad(F(1, 1 << h), 0)


# ============================================================== the circuits
def U_S():          return 1, [("S", 0)]
def U_H():          return 1, [("H", 0)]
def U_HTH():        return 1, [("H", 0), ("T", 0), ("H", 0)]
def U_CX():         return 2, [("CX", 0, 1)]
def U_SWAP():       return 2, [("CX", 0, 1), ("CX", 1, 0), ("CX", 0, 1)]
def U_H0_2():       return 2, [("H", 0)]
def U_GHZ():        return 3, [("H", 0), ("CX", 0, 1), ("CX", 1, 2)]
def U_W():          return 3, [("CX", 0, 1), ("CX", 0, 2)]
def U_H01():        return 3, [("H", 0), ("H", 1)]
def U_H012():       return 3, [("H", 0), ("H", 1), ("H", 2)]
def U_HTH2():       return 2, [("H", 0), ("T", 0), ("H", 0),
                               ("H", 1), ("T", 1), ("H", 1)]
def U_W_H3():       return 4, [("CX", 0, 1), ("CX", 0, 2), ("H", 3)]


def U_TEL():
    # adaptive.rs::teleportation_works_for_every_seed, deferred:
    #   M(0)/M(1) + IfBit corrections -> CX(1,2) and CZ(0,2).
    return 3, [("H", 0), ("H", 1), ("CX", 1, 2), ("CX", 0, 1), ("H", 0),
               ("CX", 1, 2), ("CZ", 0, 2)]


def U_REP():
    # adaptive.rs::repetition_code_syndrome_cycle_corrects, deferred:
    #   M(3) + IfBit X(1) -> CX(3,1); the unused M(4) dropped.
    return 5, [("X", 1), ("CX", 0, 3), ("CX", 1, 3), ("CX", 1, 4),
               ("CX", 2, 4), ("CX", 3, 1)]


# ================================================================ G0 / stakes
def check_G0(n, K, tag, permutation=False):
    ok = True
    for s in range(1 << n):
        tot = ZERO
        for x in K[s]:
            tot = tot + x
        if tot != ONE:
            ok = False
    for sp in range(1 << n):
        tot = ZERO
        for s in range(1 << n):
            tot = tot + K[s][sp]
        if tot != ONE:
            ok = False
    if permutation:
        img = []
        for s in range(1 << n):
            nz = [j for j in range(1 << n) if K[s][j] != ZERO]
            if len(nz) != 1:
                ok = False
            else:
                img.append(nz[0])
        if len(set(img)) != (1 << n):
            ok = False
    return ok


STAKES = []      # (id, side, circuit-name, view-name, staked W, staked lam)


def stake(idn, side, W, lam):
    STAKES.append((idn, side, W, lam))
    return W, lam


def run_stakes(log):
    """Every staked face of sections 3 and 4, read exactly."""
    rows = []
    faces = []

    n, c = U_S();     faces.append(("C1", "derivation", n, c, view_full(1),
                                    ZERO, ONE, True))
    n, c = U_H();     faces.append(("C2", "derivation", n, c, view_full(1),
                                    Quad(F(1, 2)), ZERO, False))
    n, c = U_CX();    faces.append(("C3", "derivation", n, c, view_parity(2, [0, 1]),
                                    Quad(F(1, 2)), ZERO, True))
    n, c = U_SWAP();  faces.append(("C4", "derivation", n, c, view_parity(2, [0, 1]),
                                    ZERO, ONE, True))
    n, c = U_H0_2();  faces.append(("C5", "derivation", n, c, view_full(2),
                                    Quad(F(1, 2)), ONE, False))
    n, c = U_W();     faces.append(("C9", "derivation", n, c,
                                    view_weight_threshold(3, [0, 1, 2]),
                                    Quad(F(1, 4)), Quad(F(1, 2)), True))
    n, c = U_H01();   faces.append(("C10", "derivation", n, c,
                                    view_weight_threshold(3, [0, 1, 2]),
                                    Quad(F(3, 8)), Quad(F(1, 4)), False))
    n, c = U_HTH();   faces.append(("C12", "derivation", n, c, view_full(1),
                                    Quad(F(2, 4), F(-1, 4)), Quad(0, F(1, 2)), False))

    n, c = U_GHZ();   faces.append(("C6", "held-out", n, c, view_full(3),
                                    Quad(F(1, 2)), ONE, False))
    n, c = U_GHZ();   faces.append(("C7", "held-out", n, c, view_parity(3, [0, 1, 2]),
                                    Quad(F(1, 2)), ZERO, False))
    n, c = U_GHZ();   faces.append(("C8", "held-out", n, c, view_marginal(3, [1, 2]),
                                    Quad(F(1, 2)), ONE, False))
    n, c = U_H012();  faces.append(("C11", "held-out", n, c,
                                    view_weight_threshold(3, [0, 1, 2]),
                                    Quad(F(1, 2)), ZERO, False))
    n, c = U_HTH2();  faces.append(("C13", "held-out", n, c, view_full(2),
                                    Quad(F(5, 8), F(-2, 8)), Quad(0, F(1, 2)), False))
    n, c = U_W_H3()
    vw = view_weight_threshold(4, [0, 1, 2])
    vb = view_marginal(4, [3])
    faces.append(("C14", "held-out", n, c,
                  view_product("weight-threshold(0,1,2) x s3", 4, vw, vb),
                  Quad(F(5, 8)), Quad(F(1, 2)), False))
    n, c = U_TEL();   faces.append(("T1", "held-out", n, c, view_full(3),
                                    Quad(F(7, 8)), ZERO, False))
    n, c = U_TEL();   faces.append(("T2", "held-out", n, c, view_parity(3, [0, 1, 2]),
                                    Quad(F(1, 2)), ZERO, False))
    n, c = U_TEL();   faces.append(("T3", "held-out", n, c, view_marginal(3, [0, 1]),
                                    Quad(F(3, 4)), ZERO, False))
    n, c = U_TEL();   faces.append(("T4", "held-out", n, c, view_hamming(3),
                                    Quad(F(5, 8)), ZERO, False))
    n, c = U_REP();   faces.append(("R1v", "held-out", n, c, view_marginal(5, [0, 1, 2]),
                                    Quad(F(1, 2)), ONE, True))
    n, c = U_REP();   faces.append(("R2v", "held-out", n, c, view_marginal(5, [3, 4]),
                                    Quad(F(3, 4)), ZERO, True))
    n, c = U_REP();   faces.append(("R3v", "held-out", n, c, view_full(5),
                                    ZERO, ONE, True))

    kcache = {}
    for (idn, side, n, circ, view, Wstake, lamstake, perm) in faces:
        key = (n, tuple(circ))
        if key not in kcache:
            kcache[key] = born_kernel(n, circ)
        K = kcache[key]
        g0 = check_G0(n, K, idn, permutation=perm)
        W, lam, how, mu, M = read_face(n, K, view, idn)
        # structural prediction, from the circuit alone, where it applies
        hpred = None
        if view.B is not None and all(g[0] not in ("T", "Tdg") for g in circ):
            h, m = predicted_h(n, circ, view)
            hpred = (h, m, dyadic(h))
        okW = (W == Wstake)
        okL = (lam is not None and lam == lamstake)
        rows.append(dict(id=idn, side=side, view=view.name, W=W, Wstake=Wstake,
                         lam=lam, lamstake=lamstake, how=how, g0=g0,
                         hpred=hpred, okW=okW, okL=okL, N=view.nblocks))
        log(f"  {idn:4s} [{side:10s}] {view.name:34s} "
            f"W = {W!r:22s} staked {Wstake!r:22s} {'OK ' if okW else 'MISS'}   "
            f"lam = {lam!r:14s} staked {lamstake!r:14s} "
            f"{'OK ' if okL else 'MISS'}  [{how}]"
            + (f"  h={hpred[0]}/{hpred[1]} -> 1-2^-h = {hpred[2]!r}" if hpred else "")
            + ("" if g0 else "   G0 VOID"))
    return rows


# ============================================== S2: the quantization sweep
# Exact integer arithmetic (every Clifford Born weight is dyadic), numpy int64
# only as a fast exact integer matmul.  Nothing here is floating point.
import numpy as np


def random_clifford(n, depth, rng):
    g = []
    while len(g) < depth:
        k = rng.choice(["H", "S", "X", "Z", "CX", "CX"])
        if k == "CX":
            if n < 2:
                continue
            a, b = rng.sample(range(n), 2)
            g.append(("CX", a, b))
        else:
            g.append((k, rng.randrange(n)))
    return g


def random_linear_view(n, m, rng):
    while True:
        B = [rng.randrange(1, 1 << n) for _ in range(m)]
        if dim(B) == m:
            return B


def kernel_int(n, circuit):
    """Clifford Born kernel as integers over a common power-of-two denominator.
    Returns (Knum, kden_log2) with K = Knum / 2^kden."""
    K = born_kernel(n, circuit)
    kd = 0
    for row in K:
        for x in row:
            assert x.is_rat(), "sweep is Clifford-only; a T gate leaked in"
            if x.p:
                d = x.p.denominator
                assert d & (d - 1) == 0
                kd = max(kd, d.bit_length() - 1)
    Knum = [[int(x.p * (1 << kd)) for x in row] for row in K]
    return Knum, kd


def sweep_pair(n, circuit, Bmasks, Knum, kd, Acols, Vgens):
    """One (circuit, linear view) pair, entirely in exact integers.
    Returns (ok_struct, ok_W, ok_lam, lam, h, W)."""
    m = len(Bmasks)
    N = 1 << m

    def Bs(s):
        r = 0
        for i, mask in enumerate(Bmasks):
            r |= (bin(s & mask).count("1") & 1) << (m - 1 - i)
        return r

    # --- predicted h from the CIRCUIT alone
    gens = []
    for s in range(1 << n):
        if Bs(s) == 0:
            As = 0
            for q in range(n):
                if bit(s, q, n):
                    As ^= Acols[q]
            gens.append(Bs(As))
    for v in Vgens:
        gens.append(Bs(v))
    h = dim(gens)

    # --- measured P over a common denominator 2^(n+kd)
    Pnum = [[0] * N for _ in range(N)]
    for s in range(1 << n):
        i = Bs(s)
        row = Knum[s]
        Pi = Pnum[i]
        for sp in range(1 << n):
            v = row[sp]
            if v:
                Pi[Bs(sp)] += v
    Pden = (1 << n) * (1 << kd)
    tot = sum(max(r) for r in Pnum)
    W = F(1) - F(tot, Pden)
    ok_W = (W == F(1) - F(1, 1 << h))

    # --- lambda, exact, with no structure assumed:
    #     A = M - Pi over denominator Dc; B = A^T A (uniform mu).
    blk = 1 << (n - m)
    Mden = blk * (1 << kd)                    # M = Pnum / Mden
    Dc = Mden * N
    lam = lam_int_quantized(Pnum, Mden, N)
    ok_lam = (lam == (0 if h == m else 1))

    # --- Theorem C1's structure, MEASURED on the kernel
    Vb = span_basis(list(Vgens))
    Vel = subgroup_elements(Vb)
    val = 1 << (kd - len(Vb)) if kd >= len(Vb) else None
    ok_struct = True
    supp0 = [j for j in range(1 << n) if Knum[0][j]]
    if len(supp0) != len(Vel):
        ok_struct = False
    else:
        c0 = min(supp0)
        for s in range(1 << n):
            As = 0
            for q in range(n):
                if bit(s, q, n):
                    As ^= Acols[q]
            want = sorted(c0 ^ As ^ v for v in Vel)
            got = sorted(j for j in range(1 << n) if Knum[s][j])
            if want != got:
                ok_struct = False
                break
            vals = {Knum[s][j] for j in got}
            if len(vals) != 1:
                ok_struct = False
                break
    return ok_struct, ok_W, ok_lam, lam, h, W


def lam_int_quantized(Pnum, Mden, N):
    """The sweep's exact integer lambda test, factored out so it can be
    controlled: 0, 1, or None (= not quantized)."""
    Dc = Mden * N
    A = np.array([[Pnum[i][j] * N - Mden for j in range(N)] for i in range(N)],
                 dtype=np.int64)
    BB = A.T @ A
    if not BB.any():
        return 0
    if np.array_equal(BB @ BB, (Dc * Dc) * BB):
        return 1
    return None


def detector_control(log):
    """DETECTOR, NOT VERDICT: S2's exact integer lambda test must be able to
    return 'not quantized'.  Fed C9's transfer ([[3/4,1/4],[1/4,3/4]],
    lambda = 1/2) and C10's ([[5/8,3/8],[3/8,5/8]], lambda = 1/4) it must say
    None; fed a permutation and fed Pi it must say 1 and 0."""
    cases = [("C9 transfer (lambda = 1/2)", [[3, 1], [1, 3]], 4, 2, None),
             ("C10 transfer (lambda = 1/4)", [[5, 3], [3, 5]], 8, 2, None),
             ("identity (lambda = 1)", [[1, 0], [0, 1]], 1, 2, 1),
             ("Pi (lambda = 0)", [[1, 1], [1, 1]], 2, 2, 0)]
    ok = True
    for name, Pn, Md, N, want in cases:
        got = lam_int_quantized(Pn, Md, N)
        good = (got == want)
        ok = ok and good
        log(f"  control: {name:30s} detector says {str(got):5s} "
            f"want {str(want):5s} {'OK' if good else 'FAIL'}")
    return ok


def run_sweep(log, seed=20260828):
    rng = random.Random(seed)
    plan = [(2, 400, 6), (3, 800, 10), (4, 1000, 12)]
    tot = bad_struct = bad_W = bad_lam = 0
    inter = []
    lam_hist = {0: 0, 1: 0}
    W_hist = {}
    for (n, ncirc, nview) in plan:
        for _ in range(ncirc):
            circ = random_clifford(n, 12, rng)
            Knum, kd = kernel_int(n, circ)
            Acols, Vgens = structural_AV(n, circ)
            for _ in range(nview):
                m = rng.randint(1, n)
                B = random_linear_view(n, m, rng)
                ok_s, ok_W, ok_l, lam, h, W = sweep_pair(
                    n, circ, B, Knum, kd, Acols, Vgens)
                tot += 1
                if not ok_s:
                    bad_struct += 1
                if not ok_W:
                    bad_W += 1
                    if len(inter) < 8:
                        inter.append(("W", n, circ, B, h, W))
                if not ok_l:
                    bad_lam += 1
                    if len(inter) < 8:
                        inter.append(("lam", n, circ, B, h, lam))
                if lam in lam_hist:
                    lam_hist[lam] += 1
                else:
                    if len(inter) < 8:
                        inter.append(("NOT-QUANTIZED", n, circ, B, h, lam))
                W_hist[W] = W_hist.get(W, 0) + 1
        log(f"  n={n}: {ncirc} circuits x {nview} views done "
            f"(running total {tot} pairs)")
    log(f"  pairs read                    : {tot}")
    log(f"  Theorem C1 coset structure    : {tot - bad_struct}/{tot} exact")
    log(f"  W == 1 - 2^-h                 : {tot - bad_W}/{tot} exact")
    log(f"  lambda == predicted 0/1       : {tot - bad_lam}/{tot} exact")
    log(f"  lambda histogram              : 0 -> {lam_hist[0]}, 1 -> {lam_hist[1]}, "
        f"other -> {tot - lam_hist[0] - lam_hist[1]}")
    log("  rent spectrum observed        : "
        + ", ".join(f"{k}({v})" for k, v in sorted(W_hist.items())))
    for row in inter:
        log(f"  MISS {row}")
    return dict(total=tot, bad_struct=bad_struct, bad_W=bad_W, bad_lam=bad_lam,
                unquantized=tot - lam_hist[0] - lam_hist[1],
                spectrum=sorted(str(k) for k in W_hist))


# ===================================================================== plants
def run_plants(log):
    out = {}

    # (i) best-model -> average-model substitution, on C9
    n, c = U_W()
    K = born_kernel(n, c)
    v = view_weight_threshold(3, [0, 1, 2])
    P = face_P(n, K, v)
    W_true = rent(P)
    tot = ZERO
    for row in P:
        acc = ZERO
        for x in row:
            acc = acc + x
        tot = tot + acc * quad(F(1, len(row)))
    W_plant = ONE - tot
    fired = (W_plant != W_true)
    out["(i) mean-for-max on C9"] = (str(W_true), str(W_plant), fired)
    log(f"  plant (i)   C9 rent  true {W_true!r}  planted {W_plant!r}  "
        f"{'FIRES' if fired else 'SILENT'}")

    # (ii) drop the spread subgroup BV, on C2
    n, c = U_H()
    view = view_full(1)
    Acols, Vgens = structural_AV(n, c)
    kerB = [s for s in range(1 << n) if view.apply_B(s) == 0]
    gens_true, gens_plant = [], []
    for s in kerB:
        As = 0
        for q in range(n):
            if bit(s, q, n):
                As ^= Acols[q]
        gens_true.append(view.apply_B(As))
        gens_plant.append(view.apply_B(As))
    for vv in Vgens:
        gens_true.append(view.apply_B(vv))
    h_true, h_plant = dim(gens_true), dim(gens_plant)
    fired = (h_true != h_plant)
    out["(ii) drop BV on C2"] = (str(dyadic(h_true)), str(dyadic(h_plant)), fired)
    log(f"  plant (ii)  C2 rent  true {dyadic(h_true)!r}  planted "
        f"{dyadic(h_plant)!r}  {'FIRES' if fired else 'SILENT'}")

    # (iii) aggregate-lambda headline on the product face C5
    n, c = U_H0_2()
    K = born_kernel(n, c)
    v = view_full(2)
    P = face_P(n, K, v)
    mu, _ = marginals(P)
    W_true = rent(P)
    lam, _how = lam_exact(transfer(P, mu), mu)
    N = v.nblocks
    W_plant = (ONE - quad(F(1, N))) * (ONE - lam)
    fired = (W_plant != W_true)
    out["(iii) aggregate-lambda on C5"] = (str(W_true), str(W_plant), fired)
    log(f"  plant (iii) C5 rent  true {W_true!r}  planted {W_plant!r}  "
        f"{'FIRES' if fired else 'SILENT'}")

    # (iv) non-unital step: adaptive.rs Step::Reset on one qubit
    Kreset = [[ONE, ZERO], [ONE, ZERO]]
    g0 = check_G0(1, Kreset, "reset")
    out["(iv) Reset (non-unital) at G0"] = ("unital", "non-unital", not g0)
    log(f"  plant (iv)  Reset column sums  G0 = {'PASS' if g0 else 'VOID'}  "
        f"{'FIRES' if not g0 else 'SILENT'}")
    return out


# ============================================ G2: agreement with the engine
ENGINE_CIRCUITS = {
    "U_CX": U_CX, "U_SWAP": U_SWAP, "U_H0_2": U_H0_2, "U_GHZ": U_GHZ,
    "U_W": U_W, "U_H01": U_H01, "U_H012": U_H012, "U_W_H3": U_W_H3,
    "U_TEL": U_TEL, "U_REP": U_REP,
}


def predict_determinacy(n, circuit):
    """From (c0, A, V): per input s, per qubit, Some(bit) or None."""
    Acols, Vgens = structural_AV(n, circuit)
    Vel = subgroup_elements(span_basis(list(Vgens)))
    amps, _m = evolve(n, circuit, 0)
    supp0 = [j for j in range(1 << n) if any(amps[j])]
    c0 = min(supp0)
    out = {}
    for s in range(1 << n):
        As = 0
        for q in range(n):
            if bit(s, q, n):
                As ^= Acols[q]
        coset = [c0 ^ As ^ v for v in Vel]
        row = []
        for q in range(n):
            bits = {bit(x, q, n) for x in coset}
            row.append(bits.pop() if len(bits) == 1 else None)
        out[s] = row
    return out


def run_engine_gate(log):
    exe = os.path.join(HERE, "engine_probe", "target", "release", "engine_probe")
    if not os.path.exists(exe):
        log("  G2 VOID: engine probe binary not built")
        return dict(status="VOID", reason="probe not built")
    try:
        raw = subprocess.run([exe], capture_output=True, text=True, timeout=300)
    except Exception as e:                                    # pragma: no cover
        log(f"  G2 VOID: engine probe failed: {e}")
        return dict(status="VOID", reason=str(e))
    if raw.returncode != 0:
        log(f"  G2 VOID: engine probe exit {raw.returncode}: {raw.stderr[:200]}")
        return dict(status="VOID", reason=raw.stderr[:200])
    eng = json.loads(raw.stdout)
    tot = mism = 0
    misses = []
    for name, fn in ENGINE_CIRCUITS.items():
        n, circ = fn()
        pred = predict_determinacy(n, circ)
        got = eng[name]
        for s in range(1 << n):
            row = got[str(s)] if isinstance(got, dict) else got[s]
            for q in range(n):
                tot += 1
                e = row[q]                  # -1 indeterminate, else 0/1
                p = pred[s][q]
                if (e < 0 and p is not None) or (e >= 0 and p != e):
                    mism += 1
                    if len(misses) < 6:
                        misses.append((name, s, q, e, p))
    log(f"  G2 engine agreement: {tot - mism}/{tot} (qubit, input) readings match "
        f"the (c0, A, V) prediction")
    for m in misses:
        log(f"     MISMATCH {m}")
    return dict(status="PASS" if mism == 0 else "FAIL", total=tot, mismatch=mism,
                misses=misses)


# ======================================================================= main
def main():
    logf = open(os.path.join(HERE, "omega_circuits1.log"), "w")

    def log(s=""):
        print(s, flush=True)
        logf.write(s + "\n")
        logf.flush()

    log("=" * 78)
    log("OMEGA-CIRCUITS-1 -- the frozen CROSS-FACE-1 rent law on the engine's")
    log("own qubit circuits.  Prereg frozen before this file; sha256 in")
    log("FREEZE.sha256.  Exact arithmetic throughout.")
    log("=" * 78)

    log("\nA.  STAKED FACES (sections 3 and 4 of the freeze)")
    log("-" * 78)
    rows = run_stakes(log)

    log("\nB.  S2 -- the quantization sweep")
    log("-" * 78)
    ctl = detector_control(log)
    log(f"  detector control: {'PASS' if ctl else 'FAIL -- S2 is VOID'}")
    sweep = run_sweep(log)
    sweep["detector_control"] = ctl

    log("\nC.  PLANTS")
    log("-" * 78)
    plants = run_plants(log)

    log("\nD.  G2 -- agreement with the engine's own PackedTableau")
    log("-" * 78)
    g2 = run_engine_gate(log)

    log("\nE.  G1 -- Theorem 2 on every view read")
    log("-" * 78)
    bad = [r for r in G1_ROWS if r[1].startswith("FIRE")]
    for tag, verdict in G1_ROWS:
        log(f"  {tag:5s} {verdict}")
    log(f"  G1: {len(G1_ROWS) - len(bad)}/{len(G1_ROWS)} pass, "
        f"{len(bad)} certified violations")

    log("\nF.  VERDICTS")
    log("-" * 78)
    der = [r for r in rows if r["side"] == "derivation"]
    hel = [r for r in rows if r["side"] == "held-out"]
    dW = sum(1 for r in der if r["okW"]) 
    hW = sum(1 for r in hel if r["okW"])
    dL = sum(1 for r in der if r["okL"])
    hL = sum(1 for r in hel if r["okL"])
    hpred_ok = sum(1 for r in rows if r["hpred"] and r["hpred"][2] == r["W"])
    hpred_n = sum(1 for r in rows if r["hpred"])
    log(f"  G0  : {'PASS' if all(r['g0'] for r in rows) else 'VOID on some face'}")
    log(f"  G1  : {'PASS' if not bad else 'FIRE'}  ({len(G1_ROWS)} views)")
    log(f"  G2  : {g2['status']}")
    log(f"  S1  : held-out rents {hW}/{len(hel)} exact, "
        f"held-out lambdas {hL}/{len(hel)} exact  "
        f"-> {'branch (a)' if hW == len(hel) and hL == len(hel) else 'branch (b)'}")
    log(f"        (derivation-side {dW}/{len(der)} rents, {dL}/{len(der)} lambdas)")
    log(f"        closed form 1-2^-h from the circuit alone: {hpred_ok}/{hpred_n}")
    log(f"  S2  : detector control {'PASS' if sweep['detector_control'] else 'FAIL'}; "
        f"{sweep['total']} pairs; structure "
        f"{sweep['total'] - sweep['bad_struct']}/{sweep['total']}, "
        f"W {sweep['total'] - sweep['bad_W']}/{sweep['total']}, "
        f"lambda {sweep['total'] - sweep['bad_lam']}/{sweep['total']}, "
        f"unquantized {sweep['unquantized']}  "
        f"-> {'branch (a)' if sweep['bad_W'] == 0 and sweep['bad_lam'] == 0 and sweep['unquantized'] == 0 else 'branch (b)'}")
    s3 = [r for r in rows if r["id"] in ("C9", "C10", "C12")]
    log(f"  S3  : intermediate-lambda faces {sum(1 for r in s3 if r['okW'] and r['okL'])}/3 "
        f"exact -> {'branch (a)' if all(r['okW'] and r['okL'] for r in s3) else 'branch (b)'}")
    s4ids = ("C13", "C14", "C5", "C6", "C8", "R1v")
    s4 = [r for r in rows if r["id"] in s4ids]
    log(f"  S4  : discriminator faces {sum(1 for r in s4 if r['okW'])}/{len(s4)} "
        f"took the product-law value -> "
        f"{'branch (a)' if all(r['okW'] for r in s4) else 'branch (b)/(c)'}")
    rv = {r["id"]: r["W"] for r in rows if r["id"] in ("R1v", "R2v", "R3v")}
    distinct = len({str(v) for v in rv.values()})
    log(f"  R1  : U_rep is a bijection (micro alpha = 1 for every view); the "
        f"three view rents are {[str(rv.get(k)) for k in ('R1v','R2v','R3v')]} "
        f"-> {distinct} distinct values -> "
        f"{'branch (a): rival refuted' if distinct == 3 else 'branch (b)'}")
    log(f"  B3  : every reading exhaustive over 2^n states, every view marginal "
        f"verified stationary (asserted in read_face)")
    log(f"  plants: {sum(1 for v in plants.values() if v[2])}/{len(plants)} fire")
    logf.close()


if __name__ == "__main__":
    main()
