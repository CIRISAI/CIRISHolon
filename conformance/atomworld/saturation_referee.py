#!/usr/bin/env python3
"""
saturation_referee.py -- the 50-digit referee for SATURATION-1.

WHAT THIS IS
  An independent, high-precision arbiter for the three-body term of the
  many-body expansion (MBE) for hydrogen, and for its order-4 gauge:

      E1            = E(H),  one electron, one STO-3G 1s, one proton
      V2(r)         = E(H2; r) - 2 E1                        [pair term]
      dE3(r12,r13,r23)
                    = E(H3) - sum_pairs V2(r_ij) - 3 E1      [triple term]
      dE4(geometry) = E(H4) - sum_pairs V2 - sum_triples dE3 - 4 E1

  All subsystem energies are the electronic ground state in the subsystem's
  minimal-Sz sector plus the classical nuclear repulsion, computed by full CI
  in the STO-3G minimal basis from closed-form Gaussian integrals in mpmath.
  Exact-in-model, not a prediction of experiment.

  Sectors (per the prereg's "Scope"):
    H  : 1 electron,  2 spin orbitals, Sz=+1/2 ->  1 determinant
    H2 : 2 electrons, 4 spin orbitals, Sz= 0   ->  4 determinants
    H3 : 3 electrons, 6 spin orbitals, Sz=+1/2 ->  9 determinants
    H4 : 4 electrons, 8 spin orbitals, Sz= 0   -> 36 determinants
  The minimal-|Sz| block contains one component of every spin multiplet that
  can be formed, so the block's lowest eigenvalue IS the global ground state.
  <S^2> is computed and reported for every point so the multiplicity is a
  MEASURED label, never an assumption.

INDEPENDENCE (gate R1's premise)
  This file shares no code with the Rust engine (holon-chem).  It shares no
  code with h2_core.py either: the integrals here are three-dimensional and
  freshly written, the CI is a general determinant CI over an arbitrary number
  of s functions, and the eigensolver is a cyclic Jacobi written here.
  h2_core.py is imported ONLY inside --selftest, as an external referee for
  this file's E(H) and E2(r).

  The only thing deliberately shared with every other implementation is the
  MODEL DEFINITION -- the STO-3G hydrogen contraction below.  A basis set is
  an input, not a derivation.

PRECISION
  Working precision is 80 decimal digits; results are reported to 50.  The
  30-digit margin exists because dE3 is a difference of energies of order 1 Ha
  whose true value at wide separation is ~1e-50: the MBE consistency selftest
  needs the cancellation to survive.  --selftest re-runs a point at 110 dps
  and requires agreement to 1e-50, so the 50 reported digits are demonstrated,
  not asserted.

USAGE
  python3 saturation_referee.py --selftest
  python3 saturation_referee.py --grid   [--out h3_referee.json]
  python3 saturation_referee.py --h4     [--out h4_referee.json]
  python3 saturation_referee.py --point 1.3886940 1.3886940 1.3886940
"""

import argparse
import json
import os
import random
import sys
import time
from itertools import combinations
from multiprocessing import Pool

from mpmath import mp, mpf, sqrt, exp, erf, pi, nstr

# ---------------------------------------------------------------------------
# Precision.  DPS_OUT is what is staked; DPS_WORK is the arithmetic headroom.
# ---------------------------------------------------------------------------
DPS_OUT = 50
DPS_WORK = 80
mp.dps = DPS_WORK

# ---------------------------------------------------------------------------
# MODEL DEFINITION (an input, not a derivation): STO-3G hydrogen 1s.
# Stored as strings so they re-materialise at whatever precision is active.
# ---------------------------------------------------------------------------
STO3G_H_EXPONENTS = ("3.42525091", "0.62391373", "0.16885540")
STO3G_H_COEFFS = ("0.15432897", "0.53532814", "0.44463454")

MODEL_NAME = "H_n/STO-3G/FCI (minimal-|Sz| block, BO nuclei)"

# The equilibrium separation of the ELEMENTS-1 H2 curve, used to place the
# staked H4 set.  A geometry parameter, not a fitted constant.
R_E = "1.3886940"

# Staking seed for the pseudo-random block of the H3 grid.
GRID_SEED = 20260828


# ===========================================================================
# 1.  Closed-form s-type Gaussian integrals in three dimensions.
# ===========================================================================
def _d2(A, B):
    """Squared Euclidean distance between two 3-vectors of mpf."""
    return (A[0] - B[0]) ** 2 + (A[1] - B[1]) ** 2 + (A[2] - B[2]) ** 2


def boys0(t):
    """
    F0(t) = int_0^1 exp(-t u^2) du = sqrt(pi/(4t)) erf(sqrt(t)),  F0(0) = 1.
    The closed form is a 0/0 ratio as t -> 0, so the Maclaurin series
    F0(t) = sum_n (-t)^n / (n! (2n+1)) takes over near the origin.
    """
    t = mpf(t)
    if t == 0:
        return mpf(1)
    if t < mpf(10) ** (-(mp.dps // 2 + 4)):
        s = mpf(0)
        term = mpf(1)
        n = 0
        while True:
            contrib = term / (2 * n + 1)
            s += contrib
            if abs(contrib) < mpf(10) ** (-(mp.dps + 10)):
                return s
            n += 1
            term *= -t / n
    return sqrt(pi / (4 * t)) * erf(sqrt(t))


def prim_norm(a):
    """Normalisation of g_a(r) = N_a exp(-a |r-A|^2):  N_a = (2a/pi)^(3/4)."""
    return (2 * mpf(a) / pi) ** mpf(0.75)


def prim_overlap(a, A, b, B):
    """S = N_a N_b (pi/p)^(3/2) exp(-mu |A-B|^2),  p = a+b,  mu = ab/p."""
    p = a + b
    mu = a * b / p
    return prim_norm(a) * prim_norm(b) * (pi / p) ** mpf(1.5) * exp(-mu * _d2(A, B))


def prim_kinetic(a, A, b, B):
    """T = N_a N_b mu (3 - 2 mu |A-B|^2) (pi/p)^(3/2) exp(-mu |A-B|^2)."""
    p = a + b
    mu = a * b / p
    d2 = _d2(A, B)
    return (prim_norm(a) * prim_norm(b) * mu * (3 - 2 * mu * d2)
            * (pi / p) ** mpf(1.5) * exp(-mu * d2))


def prim_nuclear(a, A, b, B, C, Z):
    """V = -Z N_a N_b (2 pi / p) exp(-mu |A-B|^2) F0(p |P-C|^2)."""
    p = a + b
    mu = a * b / p
    P = ((a * A[0] + b * B[0]) / p,
         (a * A[1] + b * B[1]) / p,
         (a * A[2] + b * B[2]) / p)
    return (-mpf(Z) * prim_norm(a) * prim_norm(b) * (2 * pi / p)
            * exp(-mu * _d2(A, B)) * boys0(p * _d2(P, C)))


def prim_eri(a, A, b, B, c, C, d, D):
    """
    Chemist notation (ab|cd) = int int a(1)b(1) r12^-1 c(2)d(2)
      = N_a N_b N_c N_d 2 pi^(5/2) / (p q sqrt(p+q))
        exp(-mu_ab|A-B|^2) exp(-mu_cd|C-D|^2) F0( pq/(p+q) |P-Q|^2 ).
    """
    p = a + b
    q = c + d
    P = ((a * A[0] + b * B[0]) / p,
         (a * A[1] + b * B[1]) / p,
         (a * A[2] + b * B[2]) / p)
    Q = ((c * C[0] + d * D[0]) / q,
         (c * C[1] + d * D[1]) / q,
         (c * C[2] + d * D[2]) / q)
    pref = (prim_norm(a) * prim_norm(b) * prim_norm(c) * prim_norm(d)
            * 2 * pi ** mpf(2.5) / (p * q * sqrt(p + q)))
    return (pref * exp(-(a * b / p) * _d2(A, B)) * exp(-(c * d / q) * _d2(C, D))
            * boys0((p * q / (p + q)) * _d2(P, Q)))


# ===========================================================================
# 2.  Contracted basis and AO integrals.
# ===========================================================================
def sto3g_h(center):
    """One normalised STO-3G 1s at a 3-vector centre: (centre, [(alpha, c), ...])."""
    prims = [(mpf(e), mpf(c)) for e, c in zip(STO3G_H_EXPONENTS, STO3G_H_COEFFS)]
    raw = mpf(0)
    for a, ca in prims:
        for b, cb in prims:
            raw += ca * cb * prim_overlap(a, center, b, center)
    scale = 1 / sqrt(raw)
    return (tuple(center), [(a, c * scale) for a, c in prims])


def ao_integrals(basis, nuclei):
    """S, T, V (n x n) and eri[i][j][k][l] in chemist notation, 8-fold symmetric."""
    n = len(basis)
    S = [[mpf(0)] * n for _ in range(n)]
    T = [[mpf(0)] * n for _ in range(n)]
    V = [[mpf(0)] * n for _ in range(n)]

    for i in range(n):
        Ai, pi_ = basis[i]
        for j in range(i + 1):
            Aj, pj_ = basis[j]
            s = t = v = mpf(0)
            for a, ca in pi_:
                for b, cb in pj_:
                    w = ca * cb
                    s += w * prim_overlap(a, Ai, b, Aj)
                    t += w * prim_kinetic(a, Ai, b, Aj)
                    for C, Z in nuclei:
                        v += w * prim_nuclear(a, Ai, b, Aj, C, Z)
            S[i][j] = S[j][i] = s
            T[i][j] = T[j][i] = t
            V[i][j] = V[j][i] = v

    eri = [[[[None] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]
    for i in range(n):
        for j in range(i + 1):
            ij = i * (i + 1) // 2 + j
            for k in range(n):
                for l in range(k + 1):
                    if ij < k * (k + 1) // 2 + l:
                        continue
                    acc = mpf(0)
                    Ai, pi_ = basis[i]
                    Aj, pj_ = basis[j]
                    Ak, pk_ = basis[k]
                    Al, pl_ = basis[l]
                    for a, ca in pi_:
                        for b, cb in pj_:
                            wab = ca * cb
                            for c, cc in pk_:
                                for d, cd in pl_:
                                    acc += (wab * cc * cd
                                            * prim_eri(a, Ai, b, Aj, c, Ak, d, Al))
                    for (p, q, r, s_) in ((i, j, k, l), (j, i, k, l),
                                          (i, j, l, k), (j, i, l, k),
                                          (k, l, i, j), (l, k, i, j),
                                          (k, l, j, i), (l, k, j, i)):
                        eri[p][q][r][s_] = acc
    return S, T, V, eri


# ===========================================================================
# 3.  Symmetric eigensolver: cyclic Jacobi, written here.
# ===========================================================================
def jacobi_eigen(Ain):
    """
    Cyclic Jacobi for a real symmetric matrix given as a list of lists.
    Returns (evals ascending, evecs) with evecs[k] the k-th eigenvector (a list),
    ordered to match evals.  Deterministic: fixed sweep order, no pivoting on
    magnitude beyond a fixed skip threshold.
    """
    n = len(Ain)
    A = [row[:] for row in Ain]
    Vm = [[mpf(1) if i == j else mpf(0) for j in range(n)] for i in range(n)]
    scale = mpf(0)
    for i in range(n):
        for j in range(n):
            if abs(A[i][j]) > scale:
                scale = abs(A[i][j])
    if scale == 0:
        return [mpf(0)] * n, [[mpf(1) if i == j else mpf(0) for j in range(n)]
                              for i in range(n)]
    eps = mpf(10) ** (-(mp.dps + 6)) * scale

    for _sweep in range(200):
        off = mpf(0)
        for i in range(n):
            for j in range(i + 1, n):
                off += A[i][j] ** 2
        if sqrt(off) <= eps:
            break
        for p in range(n - 1):
            for q in range(p + 1, n):
                apq = A[p][q]
                if abs(apq) <= eps * mpf(10) ** (-2):
                    continue
                theta = (A[q][q] - A[p][p]) / (2 * apq)
                if theta >= 0:
                    t = 1 / (theta + sqrt(theta ** 2 + 1))
                else:
                    t = -1 / (-theta + sqrt(theta ** 2 + 1))
                c = 1 / sqrt(t ** 2 + 1)
                s = t * c
                tau = s / (1 + c)
                h = t * apq
                A[p][p] -= h
                A[q][q] += h
                A[p][q] = A[q][p] = mpf(0)
                for k in range(n):
                    if k == p or k == q:
                        continue
                    akp = A[k][p]
                    akq = A[k][q]
                    A[k][p] = A[p][k] = akp - s * (akq + tau * akp)
                    A[k][q] = A[q][k] = akq + s * (akp - tau * akq)
                for k in range(n):
                    vkp = Vm[k][p]
                    vkq = Vm[k][q]
                    Vm[k][p] = vkp - s * (vkq + tau * vkp)
                    Vm[k][q] = vkq + s * (vkp - tau * vkq)
    else:
        raise RuntimeError("Jacobi failed to converge in 200 sweeps")

    order = sorted(range(n), key=lambda k: A[k][k])
    evals = [A[k][k] for k in order]
    evecs = [[Vm[i][k] for i in range(n)] for k in order]
    return evals, evecs


def residual_inf(H, v, lam):
    """max_i | (H v)_i - lam v_i |."""
    n = len(H)
    r = mpf(0)
    for i in range(n):
        acc = mpf(0)
        row = H[i]
        for j in range(n):
            acc += row[j] * v[j]
        d = abs(acc - lam * v[i])
        if d > r:
            r = d
    return r


# ===========================================================================
# 4.  Orthonormalisation and integral transformation.
# ===========================================================================
def loewdin(S):
    """Symmetric orthogonaliser X = S^(-1/2), from the Jacobi decomposition of S."""
    n = len(S)
    ev, evec = jacobi_eigen(S)
    smallest = ev[0]
    if smallest <= 0:
        raise ValueError("overlap matrix is not positive definite: %s" % nstr(smallest, 8))
    X = [[mpf(0)] * n for _ in range(n)]
    for k in range(n):
        w = 1 / sqrt(ev[k])
        uk = evec[k]
        for i in range(n):
            wu = w * uk[i]
            for j in range(n):
                X[i][j] += wu * uk[j]
    return X, smallest


def transform_1e(Hc, X):
    n = len(X)
    tmp = [[mpf(0)] * n for _ in range(n)]
    for i in range(n):
        for q in range(n):
            acc = mpf(0)
            for j in range(n):
                acc += Hc[i][j] * X[j][q]
            tmp[i][q] = acc
    out = [[mpf(0)] * n for _ in range(n)]
    for p in range(n):
        for q in range(n):
            acc = mpf(0)
            for i in range(n):
                acc += X[i][p] * tmp[i][q]
            out[p][q] = acc
    return out


def transform_2e(eri, X):
    """Four quarter transformations; O(n^5) rather than O(n^8)."""
    n = len(X)

    def zeros():
        return [[[[mpf(0)] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]

    g1 = zeros()
    for i in range(n):
        for j in range(n):
            for k in range(n):
                for s in range(n):
                    acc = mpf(0)
                    for l in range(n):
                        acc += eri[i][j][k][l] * X[l][s]
                    g1[i][j][k][s] = acc
    g2 = zeros()
    for i in range(n):
        for j in range(n):
            for r in range(n):
                for s in range(n):
                    acc = mpf(0)
                    for k in range(n):
                        acc += g1[i][j][k][s] * X[k][r]
                    g2[i][j][r][s] = acc
    g3 = zeros()
    for i in range(n):
        for q in range(n):
            for r in range(n):
                for s in range(n):
                    acc = mpf(0)
                    for j in range(n):
                        acc += g2[i][j][r][s] * X[j][q]
                    g3[i][q][r][s] = acc
    g4 = zeros()
    for p in range(n):
        for q in range(n):
            for r in range(n):
                for s in range(n):
                    acc = mpf(0)
                    for i in range(n):
                        acc += g3[i][q][r][s] * X[i][p]
                    g4[p][q][r][s] = acc
    return g4


# ===========================================================================
# 5.  Determinant CI over an orthonormal spatial basis.
#     Spin-orbital index p:  spatial = p >> 1,  spin = p & 1 (0 alpha, 1 beta).
#     |D> = a+_{p1} ... a+_{pN} |vac> with p1 < ... < pN (bitmask, ascending).
# ===========================================================================
def _annihilate(det, p):
    if not (det >> p) & 1:
        return None
    sign = -1 if bin(det & ((1 << p) - 1)).count("1") & 1 else 1
    return sign, det ^ (1 << p)


def _create(det, p):
    if (det >> p) & 1:
        return None
    sign = -1 if bin(det & ((1 << p) - 1)).count("1") & 1 else 1
    return sign, det | (1 << p)


def sz_block(norb, n_alpha, n_beta):
    """All determinants with the given alpha/beta counts, in ascending bitmask order."""
    dets = []
    alphas = [c for c in combinations(range(norb), n_alpha)]
    betas = [c for c in combinations(range(norb), n_beta)]
    for A in alphas:
        ma = 0
        for i in A:
            ma |= 1 << (2 * i)
        for B in betas:
            mb = 0
            for i in B:
                mb |= 1 << (2 * i + 1)
            dets.append(ma | mb)
    dets.sort()
    return dets


def build_hamiltonian(h, g, dets, nso):
    """
    H = sum_pq h_pq a+_p a_q + 1/2 sum_pqrs (pq|rs) a+_p a+_r a_s a_q,
    with spin deltas on (p,q) and (r,s).  Fermionic phases come from the
    ladder operators; no Slater-Condon case analysis is used anywhere.
    """
    idx = {d: i for i, d in enumerate(dets)}
    n = len(dets)
    H = [[mpf(0)] * n for _ in range(n)]

    for d in dets:
        col = idx[d]
        # one-electron: a+_p a_q
        for q in range(nso):
            aq = _annihilate(d, q)
            if aq is None:
                continue
            sq, d1 = aq
            sq_spin = q & 1
            hq = h[q >> 1]
            for p in range(nso):
                if (p & 1) != sq_spin:
                    continue
                hv = hq[p >> 1]
                if hv == 0:
                    continue
                cp = _create(d1, p)
                if cp is None:
                    continue
                sp, nd = cp
                H[idx[nd]][col] += (sq * sp) * hv
        # two-electron: a+_p a+_r a_s a_q  (rightmost acts first)
        for q in range(nso):
            aq = _annihilate(d, q)
            if aq is None:
                continue
            sq, d1 = aq
            qs = q & 1
            for s in range(nso):
                asx = _annihilate(d1, s)
                if asx is None:
                    continue
                ss, d2 = asx
                ssp = s & 1
                for r in range(nso):
                    if (r & 1) != ssp:
                        continue
                    cr = _create(d2, r)
                    if cr is None:
                        continue
                    sr, d3 = cr
                    for p in range(nso):
                        if (p & 1) != qs:
                            continue
                        cp = _create(d3, p)
                        if cp is None:
                            continue
                        sp, nd = cp
                        gv = g[p >> 1][q >> 1][r >> 1][s >> 1]
                        if gv == 0:
                            continue
                        H[idx[nd]][col] += mpf(sq * ss * sr * sp) * gv / 2
    return H


def s2_matrix(dets, norb, n_alpha, n_beta):
    """
    The S^2 operator as a dense matrix on the determinant block.
      S^2 = S_- S_+ + Sz^2 + Sz,  S_+ = sum_i a+_{i,alpha} a_{i,beta}
    Built as A^T A + (Sz^2+Sz) I with A the matrix of S_+ from this block to the
    Sz+1 block, which is what S_- = S_+^dagger means.
    """
    m = len(dets)
    sz = mpf(n_alpha - n_beta) / 2
    S2 = [[mpf(0)] * m for _ in range(m)]
    if n_beta > 0:
        up = sz_block(norb, n_alpha + 1, n_beta - 1)
        uidx = {d: i for i, d in enumerate(up)}
        A = [[mpf(0)] * m for _ in range(len(up))]
        for j, d in enumerate(dets):
            for i in range(norb):
                a = _annihilate(d, 2 * i + 1)
                if a is None:
                    continue
                sa, d1 = a
                c = _create(d1, 2 * i)
                if c is None:
                    continue
                sc, d2 = c
                A[uidx[d2]][j] += mpf(sa * sc)
        for i in range(m):
            for j in range(i + 1):
                acc = mpf(0)
                for k in range(len(up)):
                    acc += A[k][i] * A[k][j]
                S2[i][j] = S2[j][i] = acc
    for i in range(m):
        S2[i][i] += sz * sz + sz
    return S2


def spin_resolved_spectrum(H, S2M, lam=None):
    """
    [H, S^2] = 0, so diagonalising H + lambda S^2 with a large lambda splits any
    accidental degeneracy BY SPIN and returns simultaneous eigenvectors.  Each
    state's energy is then its Rayleigh quotient against H itself -- no large
    cancellation, so the 50 reported digits survive.
    Returns [(E, <S^2>), ...] ascending in E.
    """
    m = len(H)
    if lam is None:
        lam = mpf(1000)
    M = [[H[i][j] + lam * S2M[i][j] for j in range(m)] for i in range(m)]
    _, evec = jacobi_eigen(M)
    out = []
    for v in evec:
        nrm = sqrt(sum(x ** 2 for x in v))
        v = [x / nrm for x in v]
        E = mpf(0)
        s2 = mpf(0)
        for i in range(m):
            hi = mpf(0)
            si = mpf(0)
            for j in range(m):
                hi += H[i][j] * v[j]
                si += S2M[i][j] * v[j]
            E += v[i] * hi
            s2 += v[i] * si
        out.append((E, s2))
    out.sort(key=lambda t: t[0])
    return out


def s_squared(psi, dets, norb, n_alpha, n_beta):
    """
    <S^2> = || S_+ psi ||^2 + Sz^2 + Sz,  with S_+ = sum_i a+_{i,alpha} a_{i,beta}.
    psi is a coefficient list aligned with dets, assumed normalised.
    """
    acc = {}
    for c, d in zip(psi, dets):
        if c == 0:
            continue
        for i in range(norb):
            a = _annihilate(d, 2 * i + 1)
            if a is None:
                continue
            sa, d1 = a
            cr = _create(d1, 2 * i)
            if cr is None:
                continue
            sc, d2 = cr
            acc[d2] = acc.get(d2, mpf(0)) + mpf(sa * sc) * c
    nrm2 = mpf(0)
    for v in acc.values():
        nrm2 += v ** 2
    sz = mpf(n_alpha - n_beta) / 2
    return nrm2 + sz ** 2 + sz


# ===========================================================================
# 6.  System energies.
# ===========================================================================
def nuclear_repulsion(centers):
    acc = mpf(0)
    for i in range(len(centers)):
        for j in range(i):
            acc += 1 / sqrt(_d2(centers[i], centers[j]))
    return acc


def system_energy(centers, detail=False, sector=None, spin_resolve=False):
    """
    Ground-state total energy of n hydrogens at the given 3-vector centres:
    FCI in the minimal-|Sz| block (or the explicit `sector` = (n_alpha, n_beta))
    plus classical nuclear repulsion.
    """
    n = len(centers)
    basis = [sto3g_h(c) for c in centers]
    nuclei = [(tuple(c), 1) for c in centers]
    S, T, V, eri = ao_integrals(basis, nuclei)
    Hcore = [[T[i][j] + V[i][j] for j in range(n)] for i in range(n)]
    X, s_min = loewdin(S)
    h = transform_1e(Hcore, X)
    g = transform_2e(eri, X)

    if sector is None:
        n_alpha = (n + 1) // 2
        n_beta = n // 2
    else:
        n_alpha, n_beta = sector
    dets = sz_block(n, n_alpha, n_beta)
    Hm = build_hamiltonian(h, g, dets, 2 * n)

    asym = mpf(0)
    m = len(dets)
    for i in range(m):
        for j in range(i):
            d = abs(Hm[i][j] - Hm[j][i])
            if d > asym:
                asym = d
        for j in range(m):
            Hm[i][j] = Hm[j][i] = (Hm[i][j] + Hm[j][i]) / 2

    evals, evecs = jacobi_eigen(Hm)
    e_elec = evals[0]
    v0 = evecs[0]
    nrm = sqrt(sum(x ** 2 for x in v0))
    v0 = [x / nrm for x in v0]
    total = e_elec + nuclear_repulsion(centers)

    if not detail:
        return total
    out = {
        "E": total,
        "E_elec": e_elec,
        "E_nuc": nuclear_repulsion(centers),
        "ndet": m,
        "S2": s_squared(v0, dets, n, n_alpha, n_beta),
        "residual": residual_inf(Hm, v0, e_elec),
        "h_asymmetry": asym,
        "S_min_eig": s_min,
        "evals": evals,
    }
    if spin_resolve:
        S2M = s2_matrix(dets, n, n_alpha, n_beta)
        spec = spin_resolved_spectrum(Hm, S2M)
        enuc = out["E_nuc"]
        out["spectrum"] = [(E + enuc, s2) for (E, s2) in spec]
        sing = [E + enuc for (E, s2) in spec if abs(s2) < mpf("1e-30")]
        out["E_S0"] = sing[0] if sing else None
        out["S2_of_ground_resolved"] = spec[0][1]
        out["block_min_check"] = abs((spec[0][0] + enuc) - total)
    return out


_E1_CACHE = {}


def e_h_atom():
    """E(H): one electron, one contracted 1s, one proton."""
    key = mp.dps
    if key not in _E1_CACHE:
        _E1_CACHE[key] = system_energy([(mpf(0), mpf(0), mpf(0))])
    return _E1_CACHE[key]


def e_h_atom_closedform():
    """(T + V)/S on the single contracted function -- no CI, no eigensolver."""
    basis = [sto3g_h((mpf(0), mpf(0), mpf(0)))]
    S, T, V, _ = ao_integrals(basis, [((mpf(0), mpf(0), mpf(0)), 1)])
    return (T[0][0] + V[0][0]) / S[0][0]


_E2_CACHE = {}


def e_h2(r):
    """E(H2; r), total (electronic ground state of the Sz=0 block + 1/r)."""
    r = mpf(r)
    key = (mp.dps, mp.nstr(r, mp.dps))
    if key not in _E2_CACHE:
        _E2_CACHE[key] = system_energy([(mpf(0), mpf(0), mpf(0)),
                                        (r, mpf(0), mpf(0))])
    return _E2_CACHE[key]


def v2(r):
    """The pair term V2(r) = E(H2;r) - 2 E(H)."""
    return e_h2(r) - 2 * e_h_atom()


def exchange_J(r):
    """
    The H2 singlet-triplet gap J(r) = E(triplet) - E(singlet) at separation r.
    At wide separation this is the Heisenberg exchange coupling, and it is what
    the three-body term reduces to there (see selftest T13).
    """
    r = mpf(r)
    c = [(mpf(0), mpf(0), mpf(0)), (r, mpf(0), mpf(0))]
    return system_energy(c, sector=(2, 0)) - system_energy(c, sector=(1, 1))


# ---------------------------------------------------------------------------
# Geometry: three sides -> three coplanar centres.
# ---------------------------------------------------------------------------
def sites_from_sides(r12, r13, r23):
    r12, r13, r23 = mpf(r12), mpf(r13), mpf(r23)
    A = (mpf(0), mpf(0), mpf(0))
    B = (r12, mpf(0), mpf(0))
    x = (r12 ** 2 + r13 ** 2 - r23 ** 2) / (2 * r12)
    y2 = r13 ** 2 - x ** 2
    if y2 < 0:
        if y2 > -mpf(10) ** (-(mp.dps - 12)) * (r13 ** 2 + 1):
            y2 = mpf(0)
        else:
            raise ValueError("sides violate the triangle inequality: %s %s %s"
                             % (nstr(r12, 12), nstr(r13, 12), nstr(r23, 12)))
    return [A, B, (x, sqrt(y2), mpf(0))]


def de3_from_sides(r12, r13, r23, detail=False):
    centers = sites_from_sides(r12, r13, r23)
    d = system_energy(centers, detail=True)
    e1 = e_h_atom()
    v = [v2(r12), v2(r13), v2(r23)]
    de3 = d["E"] - (v[0] + v[1] + v[2]) - 3 * e1
    if not detail:
        return de3
    d["dE3"] = de3
    d["V2"] = v
    d["E1"] = e1
    return d


def de4_from_positions(positions, detail=False, spin_resolve=False):
    """dE4 = E(H4) - 4 E1 - sum_pairs V2 - sum_triples dE3."""
    n = len(positions)
    assert n == 4
    d = system_energy(positions, detail=True, spin_resolve=spin_resolve)
    e1 = e_h_atom()
    dist = {}
    for i in range(4):
        for j in range(i):
            dist[(j, i)] = sqrt(_d2(positions[i], positions[j]))
    pair_sum = mpf(0)
    pairs = []
    for k in sorted(dist):
        val = v2(dist[k])
        pairs.append((k, dist[k], val))
        pair_sum += val
    triples = []
    trip_sum = mpf(0)
    for t in combinations(range(4), 3):
        a, b, c = t
        s12 = dist[(a, b)]
        s13 = dist[(a, c)]
        s23 = dist[(b, c)]
        val = de3_from_sides(s12, s13, s23)
        triples.append((t, (s12, s13, s23), val))
        trip_sum += val
    de4 = d["E"] - 4 * e1 - pair_sum - trip_sum
    if not detail:
        return de4
    d["dE4"] = de4
    if spin_resolve and d.get("E_S0") is not None:
        d["dE4_S0"] = d["E_S0"] - 4 * e1 - pair_sum - trip_sum
    d["pairs"] = pairs
    d["triples"] = triples
    d["pair_sum"] = pair_sum
    d["triple_sum"] = trip_sum
    d["E1"] = e1
    return d


# ===========================================================================
# 7.  The staked geometry set.
# ===========================================================================
STAKING_RULE = """\
FROZEN STAKING RULE for the H3 grid (seed 20260828).  Four blocks, emitted in
this order, deduplicated on the exact side triple (first occurrence wins):

  A0  ANCHORS (2) -- the two unambiguous geometries disclosed in the prereg's
      feasibility paragraph, so the record carries a direct f64-vs-50-digit
      comparison at a point whose prior was seen before the freeze:
        (r_e, r_e, r_e)        equilateral at r_e
        (r_e, r_e, 2*r_e)      symmetric linear at r_e (exactly collinear)
      with r_e = 1.3886940 bohr.

  A   EQUILATERALS (12) -- sides taken from the fixed ladder
        0.90 1.00 1.20 1.3886940 1.60 2.00 2.50 3.00 3.75 4.60 5.75 7.00
      spanning the domain from the inner wall to the outer wall.

  B   NEAR-LINEAR (12) -- for each (a,b) in the fixed ladder
        (0.90,0.90) (1.3886940,1.3886940) (1.00,2.00)
        (1.50,3.00) (2.00,2.50) (3.00,3.50)
      and each squeeze delta in {1e-3, 1e-6}: sides (a, b, (a+b)*(1-delta)).
      These sit an arbitrarily controlled distance from the collinear wall.

  C   BOUNDARY SHELL (12) -- r23 = 7.0 exactly (the domain's outer wall), with
        (r12,r13) in {(3.55,3.55) (3.60,3.60) (4.00,3.50) (4.50,4.50)
                      (5.00,2.50) (5.50,2.00) (6.00,1.50) (6.00,6.00)
                      (6.50,1.00) (7.00,0.90) (7.00,3.50) (7.00,7.00)}.
      Every pair satisfies r12+r13 > 7.0.  This block is gate T2's shell.

  D   SEEDED PSEUDO-RANDOM (32) -- random.Random(20260828); repeatedly draw
      u1,u2,u3 independently uniform on [0.90, 7.00], sort ascending to
      (s0,s1,s2), and ACCEPT iff s0 + s1 > s2 * (1 + 1e-6) (strict triangle
      inequality with a margin).  Keep the first 32 acceptances, in draw order.
      No draw is discarded for any reason except that inequality, so the block
      is manifestly not cherry-picked: re-running the seed reproduces it.
"""

EQ_LADDER = ["0.90", "1.00", "1.20", R_E, "1.60", "2.00",
             "2.50", "3.00", "3.75", "4.60", "5.75", "7.00"]

NEARLIN_PAIRS = [("0.90", "0.90"), (R_E, R_E), ("1.00", "2.00"),
                 ("1.50", "3.00"), ("2.00", "2.50"), ("3.00", "3.50")]
NEARLIN_DELTAS = ["1e-3", "1e-6"]

SHELL_PAIRS = [("3.55", "3.55"), ("3.60", "3.60"), ("4.00", "3.50"),
               ("4.50", "4.50"), ("5.00", "2.50"), ("5.50", "2.00"),
               ("6.00", "1.50"), ("6.00", "6.00"), ("6.50", "1.00"),
               ("7.00", "0.90"), ("7.00", "3.50"), ("7.00", "7.00")]

DOMAIN_LO = "0.90"
DOMAIN_HI = "7.00"
N_RANDOM = 32


def _fmt_side(x):
    """Sides are exact decimal inputs; render them at output precision."""
    return nstr(mpf(x), DPS_OUT, strip_zeros=False)


def staked_geometries():
    """Return [(block, (s12, s13, s23) as decimal strings), ...] per STAKING_RULE."""
    out = []
    seen = set()

    def add(block, a, b, c):
        key = (_fmt_side(a), _fmt_side(b), _fmt_side(c))
        if key in seen:
            return
        seen.add(key)
        out.append((block, key))

    re_ = mpf(R_E)
    add("A0-anchor", re_, re_, re_)
    add("A0-anchor", re_, re_, 2 * re_)

    for a in EQ_LADDER:
        add("A-equilateral", mpf(a), mpf(a), mpf(a))

    for (a, b) in NEARLIN_PAIRS:
        for dl in NEARLIN_DELTAS:
            c = (mpf(a) + mpf(b)) * (1 - mpf(dl))
            add("B-nearlinear", mpf(a), mpf(b), c)

    for (a, b) in SHELL_PAIRS:
        add("C-shell", mpf(a), mpf(b), mpf("7.00"))

    rng = random.Random(GRID_SEED)
    lo, hi = float(DOMAIN_LO), float(DOMAIN_HI)
    kept = 0
    guard = 0
    while kept < N_RANDOM:
        guard += 1
        if guard > 100000:
            raise RuntimeError("random block failed to fill")
        u = sorted(rng.uniform(lo, hi) for _ in range(3))
        s0, s1, s2 = (mpf(repr(x)) for x in u)
        if s0 + s1 <= s2 * (1 + mpf("1e-6")):
            continue
        before = len(out)
        add("D-random", s0, s1, s2)
        if len(out) > before:
            kept += 1
    return out


def h4_geometries():
    """
    The six staked F1 geometries: regular tetrahedron, square, 60-degree rhombus,
    each at edge a = r_e and a = 1.5 r_e.  Positions are exact-in-model 3-vectors.
    """
    out = []
    for tag, mult in (("r_e", "1"), ("1.5r_e", "1.5")):
        a = mpf(R_E) * mpf(mult)
        s3 = sqrt(mpf(3))
        # regular tetrahedron, edge a
        tet = [(mpf(0), mpf(0), mpf(0)),
               (a, mpf(0), mpf(0)),
               (a / 2, a * s3 / 2, mpf(0)),
               (a / 2, a * s3 / 6, a * sqrt(mpf(2) / 3))]
        # square, side a (diagonal a sqrt2)
        sq = [(mpf(0), mpf(0), mpf(0)),
              (a, mpf(0), mpf(0)),
              (a, a, mpf(0)),
              (mpf(0), a, mpf(0))]
        # 60-degree rhombus, side a: two equilateral triangles sharing an edge
        # (short diagonal a, long diagonal a sqrt3)
        rh = [(mpf(0), mpf(0), mpf(0)),
              (a, mpf(0), mpf(0)),
              (3 * a / 2, a * s3 / 2, mpf(0)),
              (a / 2, a * s3 / 2, mpf(0))]
        out.append(("tetrahedron@%s" % tag, tet))
        out.append(("square@%s" % tag, sq))
        out.append(("rhombus60@%s" % tag, rh))
    return out


# ===========================================================================
# 8.  Workers and drivers.
# ===========================================================================
# The boundary-shell probe for gate T2.  The staked C block samples the shell
# r23 = 7.0 at twelve points; this probe walks the shell's COMPACT edge, the
# collinear line r12 + r13 = 7.0 exactly, where the two short sides are as
# short as the shell allows.  It is a probe, not part of the staked set, and is
# labelled as such in the JSON.
SHELL_PROBE_T = ["0.90", "1.20", "1.60", "2.00", "2.50", "3.00", "3.50"]


def _init_worker(dps):
    mp.dps = dps


def _w_h3(item):
    block, sides = item
    d = de3_from_sides(sides[0], sides[1], sides[2], detail=True)
    return {
        "block": block,
        "sides_bohr": list(sides),
        "E_H3": nstr(d["E"], DPS_OUT, strip_zeros=False),
        "dE3": nstr(d["dE3"], DPS_OUT, strip_zeros=False),
        "V2": [nstr(x, DPS_OUT, strip_zeros=False) for x in d["V2"]],
        "S2_ground": nstr(d["S2"], 20, strip_zeros=False),
        "eig_residual": nstr(d["residual"], 6),
        "H_asymmetry": nstr(d["h_asymmetry"], 6),
        "S_min_eig": nstr(d["S_min_eig"], 12),
        "ndet": d["ndet"],
    }


def _w_h4(item):
    name, pos = item
    d = de4_from_positions(pos, detail=True, spin_resolve=True)
    de3s = [abs(t[2]) for t in d["triples"]]
    de3_max = max(de3s)
    de4 = d["dE4"]
    de4_s0 = d.get("dE4_S0")
    return {
        "name": name,
        "positions_bohr": [[nstr(c, DPS_OUT, strip_zeros=False) for c in p]
                           for p in pos],
        "pair_distances_bohr": [nstr(p[1], DPS_OUT, strip_zeros=False)
                                for p in d["pairs"]],
        "E_H4": nstr(d["E"], DPS_OUT, strip_zeros=False),
        "dE4": nstr(de4, DPS_OUT, strip_zeros=False),
        # The Sz=0 block's ground state is NOT always a singlet at compact H4
        # geometries.  Both readings are on the record; "E_H4" / "dE4" are the
        # BLOCK minimum (the system's actual ground state), "..._S0" are the
        # lowest S=0 state, obtained by simultaneous diagonalisation with S^2.
        "E_H4_S0": nstr(d["E_S0"], DPS_OUT, strip_zeros=False),
        "dE4_S0": nstr(de4_s0, DPS_OUT, strip_zeros=False),
        "S2_of_block_ground": nstr(d["S2_of_ground_resolved"], 20),
        "block_ground_is_singlet": bool(abs(d["S2_of_ground_resolved"])
                                        < mpf("1e-30")),
        "ratio_abs_dE4_S0_over_max_abs_dE3": nstr(abs(de4_s0) / de3_max, 20),
        "V2_sum": nstr(d["pair_sum"], DPS_OUT, strip_zeros=False),
        "dE3_sum": nstr(d["triple_sum"], DPS_OUT, strip_zeros=False),
        "dE3_per_triple": [nstr(t[2], DPS_OUT, strip_zeros=False)
                           for t in d["triples"]],
        "dE3_max_abs": nstr(de3_max, DPS_OUT, strip_zeros=False),
        "ratio_abs_dE4_over_max_abs_dE3": nstr(abs(de4) / de3_max, 20),
        "ratio_abs_dE4_over_abs_sum_dE3": nstr(abs(de4) / abs(d["triple_sum"]), 20),
        # plant (i)'s carrier: the two-separated-dimers vs bound-H4 gap.
        "E_minus_two_dimers_at_r_e":
            nstr(d["E"] - 2 * e_h2(mpf(R_E)), DPS_OUT, strip_zeros=False),
        "E_S0_minus_two_dimers_at_r_e":
            nstr(d["E_S0"] - 2 * e_h2(mpf(R_E)), DPS_OUT, strip_zeros=False),
        "eig_residual": nstr(d["residual"], 6),
        "ndet": d["ndet"],
    }


def shell_probe_geometries():
    out = []
    for t in SHELL_PROBE_T:
        a = mpf(t)
        b = mpf("7.00") - a
        out.append(("T2-shell-probe", (_fmt_side(a), _fmt_side(b), _fmt_side("7.00"))))
    return out


def run_grid(out_path, jobs):
    geoms = staked_geometries()
    probe = shell_probe_geometries()
    t0 = time.time()
    c0 = time.process_time()
    with Pool(jobs, initializer=_init_worker, initargs=(DPS_WORK,)) as pool:
        rows = pool.map(_w_h3, geoms, chunksize=1)
        probe_rows = pool.map(_w_h3, probe, chunksize=1)
    wall = time.time() - t0
    for i, r in enumerate(rows):
        r["i"] = i
    doc = {
        "model": MODEL_NAME.replace("H_n", "H3"),
        "sector": "doublet, Sz=+1/2 block, 9 determinants",
        "precision_digits": DPS_OUT,
        "working_precision_dps": DPS_WORK,
        "units": {"sides": "bohr", "E": "hartree", "dE3": "hartree"},
        "definition": ("dE3(r12,r13,r23) = E(H3) - sum_pairs [E2(r_ij) - 2 E(H)] "
                       "- 3 E(H); every subsystem energy is the FCI ground state "
                       "of its minimal-|Sz| block plus classical nuclear repulsion"),
        "staking_seed": GRID_SEED,
        "staking_rule": STAKING_RULE,
        "domain": {"side_lo_bohr": DOMAIN_LO, "side_hi_bohr": DOMAIN_HI,
                   "constraint": "strict triangle inequality"},
        "E_H_atom": nstr(e_h_atom(), DPS_OUT, strip_zeros=False),
        "r_e_bohr": R_E,
        "n_geometries": len(rows),
        "boundary_shell_probe": {
            "what": ("gate T2's tail-truncation systematic. Walks the compact "
                     "edge of the r23 = 7.0 shell: the exactly-collinear line "
                     "r12 + r13 = 7.0. NOT part of the staked set; a probe."),
            "shell_max_abs_dE3_over_staked_C_block":
                nstr(max(abs(mpf(r["dE3"])) for r in rows
                         if r["block"] == "C-shell"), DPS_OUT, strip_zeros=False),
            "shell_max_abs_dE3_over_probe":
                nstr(max(abs(mpf(r["dE3"])) for r in probe_rows),
                     DPS_OUT, strip_zeros=False),
            "outer_corner_dE3_at_7_7_7":
                nstr([mpf(r["dE3"]) for r in rows
                      if r["sides_bohr"][0].startswith("7.00")
                      and r["block"] == "A-equilateral"][0],
                     DPS_OUT, strip_zeros=False),
            "geometries": probe_rows,
        },
        "cost": {"wall_seconds": round(wall, 2),
                 "driver_cpu_seconds": round(time.process_time() - c0, 2),
                 "worker_processes": jobs,
                 "note": "compute cost only; no calendar terms"},
        "geometries": rows,
    }
    with open(out_path, "w") as f:
        json.dump(doc, f, indent=1)
        f.write("\n")
    return doc, wall


def run_h4(out_path, jobs):
    geoms = h4_geometries()
    t0 = time.time()
    with Pool(min(jobs, len(geoms)), initializer=_init_worker,
              initargs=(DPS_WORK,)) as pool:
        rows = pool.map(_w_h4, geoms, chunksize=1)
    wall = time.time() - t0
    for i, r in enumerate(rows):
        r["i"] = i
    doc = {
        "model": MODEL_NAME.replace("H_n", "H4"),
        "sector": ("Sz=0 block, 36 determinants. WARNING, MEASURED: the block's "
                   "ground state is NOT a singlet at every compact geometry in "
                   "this set. Both readings are carried: E_H4/dE4 are the block "
                   "minimum (the system's true ground state), E_H4_S0/dE4_S0 are "
                   "the lowest S=0 state, separated by simultaneous "
                   "diagonalisation of H and S^2. The prereg's 'singlet block' "
                   "wording is ambiguous between the two; the engine must say "
                   "which it computes."),
        "two_dimers_reference_2xE2_at_r_e":
            nstr(2 * e_h2(mpf(R_E)), DPS_OUT, strip_zeros=False),
        "precision_digits": DPS_OUT,
        "working_precision_dps": DPS_WORK,
        "units": {"positions": "bohr", "E": "hartree", "dE4": "hartree"},
        "definition": ("dE4 = E(H4) - 4 E(H) - sum_{6 pairs} V2(r_ij) "
                       "- sum_{4 triples} dE3(triple); the dE3 values are this "
                       "referee's own, computed directly (never interpolated)"),
        "staked_set": ("r_e tetrahedron, r_e square, r_e 60-degree rhombus, "
                       "and the same three at 1.5 r_e, r_e = %s bohr" % R_E),
        "E_H_atom": nstr(e_h_atom(), DPS_OUT, strip_zeros=False),
        "r_e_bohr": R_E,
        "cost": {"wall_seconds": round(wall, 2), "worker_processes": min(jobs, len(geoms)),
                 "note": "compute cost only; no calendar terms"},
        "geometries": rows,
    }
    with open(out_path, "w") as f:
        json.dump(doc, f, indent=1)
        f.write("\n")
    return doc, wall


# ===========================================================================
# 9.  Selftest.
# ===========================================================================
def _sep_geom(d):
    """Equilateral triangle of side d."""
    return (mpf(d), mpf(d), mpf(d))


def selftest():
    """
    Every check is REQUIRED.  A check that does not run is a failure (exit 2),
    per the house exit contract.
    """
    import h2_core as H2  # external referee; imported only here

    required = [
        "T1_E_H_matches_closed_form",
        "T2_E_H_matches_h2_core",
        "T3_E2_matches_h2_core",
        "T4_MBE_zero_at_far_separation",
        "T5_H2_plus_far_H_additive",
        "T6_dE3_totally_symmetric",
        "T7_ground_is_doublet",
        "T8_eigen_residual_small",
        "T9_jacobi_matches_mpmath_eigsy",
        "T10_precision_stable_at_110dps",
        "T11_H4_zero_at_far_separation",
        "T12_anchors_match_disclosed_f64",
        "T13_dE3_at_20_bohr_is_spin_frustration",
        "T14_dE4_two_dimers_is_four_body",
    ]
    results = {}
    failures = []

    def record(name, ok, msg):
        results[name] = (ok, msg)
        print("  %-34s %s   %s" % (name, "PASS" if ok else "FAIL", msg))
        if not ok:
            failures.append(name)

    print("saturation_referee selftest  (working %d dps, output %d digits)"
          % (DPS_WORK, DPS_OUT))
    print()

    e1_ci = e_h_atom()
    e1_cf = e_h_atom_closedform()
    d = abs(e1_ci - e1_cf)
    record("T1_E_H_matches_closed_form", d < mpf("1e-70"),
           "|CI - (T+V)/S| = %s ; E(H) = %s" % (nstr(d, 4), nstr(e1_ci, 20)))

    e1_ref = H2.h_atom_energy()
    d = abs(e1_ci - e1_ref)
    record("T2_E_H_matches_h2_core", d < mpf("1e-45"),
           "|this - h2_core| = %s" % nstr(d, 4))

    re_ = mpf(R_E)
    e2_here = e_h2(re_)
    e2_ref = H2.energy_route_a(re_)
    d = abs(e2_here - e2_ref)
    record("T3_E2_matches_h2_core", d < mpf("1e-45"),
           "|this - h2_core| = %s ; E2(r_e) = %s ; V2(r_e) = %s"
           % (nstr(d, 4), nstr(e2_here, 20), nstr(v2(re_), 20)))

    # The MBE must close to the referee's ARITHMETIC precision once the physics
    # is dead.  40 bohr, not 20: at 20 bohr the three-body term is still a real
    # (if minute) physical quantity -- see T13, which measures and explains it.
    far = de3_from_sides(*_sep_geom("40.0"))
    record("T4_MBE_zero_at_far_separation", abs(far) < mpf("1e-40"),
           "dE3(40,40,40) = %s  (arithmetic noise floor at %d dps)"
           % (nstr(far, 6), DPS_WORK))

    # H2 at r_e with a third H at 25 bohr: the 9-determinant H3 machinery must
    # reproduce h2_core's 2x2 closed form plus one free atom.
    third = mpf("25.0")
    s12 = re_
    s13 = third
    s23 = sqrt(third ** 2 + re_ ** 2 - 2 * third * re_ * mpf("0.3"))  # off-axis
    e3 = system_energy(sites_from_sides(s12, s13, s23))
    d = abs(e3 - (e2_ref + e1_ref) - (v2(s13) + v2(s23)))
    record("T5_H2_plus_far_H_additive", d < mpf("1e-30"),
           "|E(H3) - E2(r_e) - E(H) - V2(far pairs)| = %s" % nstr(d, 4))

    sides = (mpf("1.1"), mpf("2.3"), mpf("3.1"))
    vals = []
    for perm in ((0, 1, 2), (0, 2, 1), (1, 0, 2), (1, 2, 0), (2, 0, 1), (2, 1, 0)):
        vals.append(de3_from_sides(sides[perm[0]], sides[perm[1]], sides[perm[2]]))
    spread = max(vals) - min(vals)
    record("T6_dE3_totally_symmetric", spread < mpf("1e-45"),
           "max-min over the 6 permutations = %s ; dE3 = %s"
           % (nstr(spread, 4), nstr(vals[0], 20)))

    dd = de3_from_sides(*_sep_geom(R_E), detail=True)
    d = abs(dd["S2"] - mpf(3) / 4)
    record("T7_ground_is_doublet", d < mpf("1e-40"),
           "|<S^2> - 3/4| = %s at the r_e equilateral" % nstr(d, 4))

    record("T8_eigen_residual_small", dd["residual"] < mpf("1e-60"),
           "||Hv - Ev||_inf = %s ; H asymmetry = %s"
           % (nstr(dd["residual"], 4), nstr(dd["h_asymmetry"], 4)))

    # Jacobi against mpmath's own symmetric eigensolver on a real H3 Hamiltonian.
    from mpmath import matrix as _matrix, eigsy as _eigsy
    centers = sites_from_sides(*_sep_geom("1.6"))
    n = 3
    basis = [sto3g_h(c) for c in centers]
    S, T, V, eri = ao_integrals(basis, [(tuple(c), 1) for c in centers])
    Hc = [[T[i][j] + V[i][j] for j in range(n)] for i in range(n)]
    X, _ = loewdin(S)
    Hm = build_hamiltonian(transform_1e(Hc, X), transform_2e(eri, X),
                           sz_block(3, 2, 1), 6)
    m = len(Hm)
    M = _matrix(m, m)
    for i in range(m):
        for j in range(m):
            M[i, j] = (Hm[i][j] + Hm[j][i]) / 2
    ev_ref = sorted(_eigsy(M)[0][i] for i in range(m))
    ev_here, _ = jacobi_eigen([[(Hm[i][j] + Hm[j][i]) / 2 for j in range(m)]
                               for i in range(m)])
    d = max(abs(ev_here[i] - ev_ref[i]) for i in range(m))
    record("T9_jacobi_matches_mpmath_eigsy", d < mpf("1e-45"),
           "max |lambda_here - lambda_eigsy| over 9 roots = %s" % nstr(d, 4))

    lo = de3_from_sides(mpf("1.1"), mpf("2.3"), mpf("3.1"))
    saved = mp.dps
    mp.dps = 110
    _E1_CACHE.clear()
    _E2_CACHE.clear()
    hi = de3_from_sides(mpf("1.1"), mpf("2.3"), mpf("3.1"))
    mp.dps = saved
    _E1_CACHE.clear()
    _E2_CACHE.clear()
    d = abs(lo - hi)
    record("T10_precision_stable_at_110dps", d < mpf("1e-50"),
           "|dE3@80dps - dE3@110dps| = %s" % nstr(d, 4))

    # Four ATOMS mutually 40 bohr apart: no permanent moments, no in-basis
    # polarisability, no overlap -- the order-4 term must vanish to arithmetic
    # precision.  (Four atoms arranged as two MOLECULES do NOT: see T14.)
    big = mpf("40.0")
    s3 = sqrt(mpf(3))
    tet40 = [(mpf(0), mpf(0), mpf(0)),
             (big, mpf(0), mpf(0)),
             (big / 2, big * s3 / 2, mpf(0)),
             (big / 2, big * s3 / 6, big * sqrt(mpf(2) / 3))]
    d4 = de4_from_positions(tet40)
    record("T11_H4_zero_at_far_separation", abs(d4) < mpf("1e-40"),
           "dE4(40-bohr tetrahedron of atoms) = %s" % nstr(d4, 6))

    eq = de3_from_sides(*_sep_geom(R_E))
    lin = de3_from_sides(re_, re_, 2 * re_)
    d_eq = abs(eq - mpf("0.858071"))
    d_lin = abs(lin - mpf("0.354728"))
    d_e1 = abs(e1_ci - mpf("-0.466581850"))
    d_v2 = abs(v2(re_) - mpf("-0.204142352"))
    ok = (d_e1 < mpf("1e-9") and d_v2 < mpf("1e-9")
          and d_eq < mpf("1e-6") and d_lin < mpf("1e-6"))
    record("T12_anchors_match_disclosed_f64", ok,
           "E(H)=%s (d=%s) V2(r_e)=%s (d=%s) dE3_eq=%s (d=%s) dE3_lin=%s (d=%s)"
           % (nstr(e1_ci, 12), nstr(d_e1, 3), nstr(v2(re_), 12), nstr(d_v2, 3),
              nstr(eq, 12), nstr(d_eq, 3), nstr(lin, 12), nstr(d_lin, 3)))

    # ---- what the 20-bohr trimer actually is -------------------------------
    # f64 calls dE3(20,20,20) machine zero (-2.3e-15, the prereg's disclosed
    # probe).  This referee resolves it 14 decades lower and it is NOT zero:
    # the equilateral trimer is spin-frustrated.  Mapping the wide-separation
    # limit onto a Heisenberg trimer H = J sum_{i<j} (S_i.S_j - 1/4) gives
    #   E(H3, doublet) - sum_pairs E(H2, singlet)  =  +3J/2
    # with J = E_triplet - E_singlet the H2 exchange gap at the same distance.
    # The prediction carries no free parameter.
    d20 = de3_from_sides(*_sep_geom("20.0"))
    J20 = exchange_J(mpf("20.0"))
    pred = 3 * J20 / 2
    rel = abs(d20 - pred) / abs(d20)
    record("T13_dE3_at_20_bohr_is_spin_frustration",
           d20 > mpf("1e-40") and rel < mpf("1e-10"),
           "dE3(20,20,20) = %s ; 3J/2 = %s ; relative deviation = %s"
           % (nstr(d20, 12), nstr(pred, 12), nstr(rel, 4)))

    # ---- and what two far-apart DIMERS are ---------------------------------
    # Two H2 molecules at 30 bohr still interact: each carries a permanent
    # quadrupole.  A bare H atom in this basis has no permanent moment and no
    # polarisability, so every pair and every triple of atoms is dead there --
    # the whole molecule-molecule interaction lands at order 4.  This is a
    # REPORTED context value for gate F1, and the reason the F1 ratio must be
    # read at compact geometries only.
    def _two_dimers(Rd):
        Rd = mpf(Rd)
        return de4_from_positions([(mpf(0), mpf(0), mpf(0)), (re_, mpf(0), mpf(0)),
                                   (Rd, mpf(0), mpf(0)), (Rd + re_, mpf(0), mpf(0))])
    a30 = _two_dimers("30.0")
    a60 = _two_dimers("60.0")
    from mpmath import log as _log
    slope = _log(abs(a60 / a30)) / _log(mpf(2))
    record("T14_dE4_two_dimers_is_four_body",
           a30 > mpf("1e-12") and mpf("-5.5") < slope < mpf("-4.5"),
           "dE4(30 bohr) = %s ; dE4(60 bohr) = %s ; log-log slope = %s "
           "(quadrupole-quadrupole is -5)"
           % (nstr(a30, 12), nstr(a60, 12), nstr(slope, 8)))

    print()
    missing = [k for k in required if k not in results]
    if missing:
        print("REFUSED: required checks did not run: %s" % ", ".join(missing))
        return 2
    if failures:
        print("SELFTEST FAILED: %s" % ", ".join(failures))
        return 1
    print("SELFTEST PASSED: %d/%d required checks" % (len(required), len(required)))
    return 0


# ===========================================================================
# 10.  CLI
# ===========================================================================
def main():
    here = os.path.dirname(os.path.abspath(__file__))
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--grid", action="store_true", help="write h3_referee.json")
    ap.add_argument("--h4", action="store_true", help="write h4_referee.json")
    ap.add_argument("--point", nargs=3, metavar=("R12", "R13", "R23"))
    ap.add_argument("--out", default=None)
    ap.add_argument("--jobs", type=int, default=min(16, os.cpu_count() or 1))
    ap.add_argument("--list-geometries", action="store_true")
    args = ap.parse_args()

    if args.selftest:
        sys.path.insert(0, here)
        return selftest()

    if args.list_geometries:
        for i, (b, s) in enumerate(staked_geometries()):
            print("%3d  %-14s %s" % (i, b, "  ".join(nstr(mpf(x), 12) for x in s)))
        return 0

    if args.point:
        d = de3_from_sides(*args.point, detail=True)
        print("sides      %s" % "  ".join(args.point))
        print("E(H)       %s" % nstr(d["E1"], DPS_OUT))
        print("E(H3)      %s" % nstr(d["E"], DPS_OUT))
        print("V2 terms   %s" % "  ".join(nstr(x, 20) for x in d["V2"]))
        print("dE3        %s" % nstr(d["dE3"], DPS_OUT))
        print("<S^2>      %s" % nstr(d["S2"], 20))
        print("residual   %s" % nstr(d["residual"], 6))
        return 0

    rc = 0
    if args.grid:
        out = args.out or os.path.join(here, "h3_referee.json")
        doc, wall = run_grid(out, args.jobs)
        print("wrote %s : %d geometries, wall %.2f s on %d workers"
              % (out, doc["n_geometries"], wall, args.jobs))
    if args.h4:
        out = args.out or os.path.join(here, "h4_referee.json")
        doc, wall = run_h4(out, args.jobs)
        print("wrote %s : %d geometries, wall %.2f s"
              % (out, len(doc["geometries"]), wall))
    if not (args.grid or args.h4):
        ap.print_help()
        rc = 2
    return rc


if __name__ == "__main__":
    sys.exit(main())
