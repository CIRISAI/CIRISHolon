"""
h2_core.py -- H2 / STO-3G / FCI potential energy core, from first principles.

DERIVATION (what is closed-form mathematics, implemented here from the formulae):
  Gaussian product theorem, and the closed-form s-type primitive integrals
  (overlap, kinetic, nuclear attraction via the Boys function F0, two-electron
  repulsion).  See docstrings on each function for the exact expression used.

MODEL DEFINITION (what is an input, not a derivation):
  The STO-3G contraction for hydrogen -- three exponents and three coefficients.
  A basis set IS a model choice; the decimal constants below define the model.
  Nothing else is quoted: the hydrogen-atom energy, R_e, D_e and the dissociation
  asymptote are all computed from this code.

No pyscf / psi4 / any quantum chemistry package is imported.  Arithmetic is
mpmath at the ambient working precision, so this file is safe to call from
mpmath.diff (which raises precision internally).
"""

import math
from mpmath import mp, mpf, sqrt, exp, erf, pi, matrix, eigsy

# ---------------------------------------------------------------------------
# MODEL DEFINITION: STO-3G hydrogen 1s contraction.
# Stored as strings so they are re-materialised at whatever precision is
# currently active (mpmath.diff raises mp.prec temporarily).
# ---------------------------------------------------------------------------
STO3G_H_EXPONENTS = ("3.42525091", "0.62391373", "0.16885540")
STO3G_H_COEFFS = ("0.15432897", "0.53532814", "0.44463454")

MODEL_NAME = "H2/STO-3G/FCI"


# ---------------------------------------------------------------------------
# Boys function of order zero.
#   F0(t) = int_0^1 exp(-t u^2) du
#         = sqrt(pi/(4t)) * erf(sqrt(t))     for t > 0
#         = 1                                for t = 0
# ---------------------------------------------------------------------------
def boys0(t):
    t = mpf(t)
    if t == 0:
        return mpf(1)
    # Guard the removable singularity with the Maclaurin series when t is so
    # small that the closed form is a ratio of two vanishing quantities.
    if t < mpf(10) ** (-(mp.dps // 2 + 4)):
        # F0(t) = sum_{n>=0} (-t)^n / (n! (2n+1))
        s = mpf(0)
        term = mpf(1)
        n = 0
        while True:
            contrib = term / (2 * n + 1)
            s += contrib
            if abs(contrib) < mpf(10) ** (-(mp.dps + 10)):
                break
            n += 1
            term *= -t / n
        return s
    return sqrt(pi / (4 * t)) * erf(sqrt(t))


# ---------------------------------------------------------------------------
# Normalisation of a primitive s-type Gaussian  g_a(r) = N_a exp(-a |r - A|^2)
#   N_a = (2a/pi)^{3/4}
# ---------------------------------------------------------------------------
def prim_norm(a):
    return (2 * mpf(a) / pi) ** mpf(0.75)


# ---------------------------------------------------------------------------
# Primitive integrals over NORMALISED s-type Gaussians.
# Centres are 1-D coordinates on the internuclear axis (all functions are s,
# so only the squared separations enter, and the molecule is linear).
# Gaussian product theorem:
#   exp(-a|r-A|^2) exp(-b|r-B|^2) = K_ab exp(-p|r-P|^2)
#   p = a + b,  P = (aA + bB)/p,  K_ab = exp(-(ab/p) |A-B|^2)
# ---------------------------------------------------------------------------
def prim_overlap(a, A, b, B):
    """S = N_a N_b (pi/p)^{3/2} K_ab"""
    a, b, A, B = mpf(a), mpf(b), mpf(A), mpf(B)
    p = a + b
    mu = a * b / p
    d2 = (A - B) ** 2
    return prim_norm(a) * prim_norm(b) * (pi / p) ** mpf(1.5) * exp(-mu * d2)


def prim_kinetic(a, A, b, B):
    """T = N_a N_b mu (3 - 2 mu |A-B|^2) (pi/p)^{3/2} K_ab,  mu = ab/p"""
    a, b, A, B = mpf(a), mpf(b), mpf(A), mpf(B)
    p = a + b
    mu = a * b / p
    d2 = (A - B) ** 2
    return (prim_norm(a) * prim_norm(b) * mu * (3 - 2 * mu * d2)
            * (pi / p) ** mpf(1.5) * exp(-mu * d2))


def prim_nuclear(a, A, b, B, C, Z):
    """V = -Z N_a N_b (2 pi / p) K_ab F0(p |P-C|^2)"""
    a, b, A, B, C = mpf(a), mpf(b), mpf(A), mpf(B), mpf(C)
    p = a + b
    mu = a * b / p
    P = (a * A + b * B) / p
    d2 = (A - B) ** 2
    t = p * (P - C) ** 2
    return (-mpf(Z) * prim_norm(a) * prim_norm(b) * (2 * pi / p)
            * exp(-mu * d2) * boys0(t))


def prim_eri(a, A, b, B, c, C, d, D):
    """
    Chemist notation (ab|cd) = int int a(1) b(1) r12^{-1} c(2) d(2).
      = N_a N_b N_c N_d * 2 pi^{5/2} / (p q sqrt(p+q))
        * K_ab K_cd * F0( pq/(p+q) |P-Q|^2 )
    """
    a, b, c, d = mpf(a), mpf(b), mpf(c), mpf(d)
    A, B, C, D = mpf(A), mpf(B), mpf(C), mpf(D)
    p = a + b
    q = c + d
    P = (a * A + b * B) / p
    Q = (c * C + d * D) / q
    Kab = exp(-(a * b / p) * (A - B) ** 2)
    Kcd = exp(-(c * d / q) * (C - D) ** 2)
    t = (p * q / (p + q)) * (P - Q) ** 2
    pref = (prim_norm(a) * prim_norm(b) * prim_norm(c) * prim_norm(d)
            * 2 * pi ** mpf(2.5) / (p * q * sqrt(p + q)))
    return pref * Kab * Kcd * boys0(t)


# ---------------------------------------------------------------------------
# Contracted basis: one STO-3G 1s function per hydrogen.
# The contraction is renormalised so that <chi|chi> = 1 exactly at the working
# precision (standard practice; the tabulated coefficients are rounded).
# ---------------------------------------------------------------------------
def sto3g_h(center):
    """Return (center, [(alpha, coeff), ...]) with the contraction normalised."""
    prims = [(mpf(e), mpf(c)) for e, c in zip(STO3G_H_EXPONENTS, STO3G_H_COEFFS)]
    raw = mpf(0)
    for a, ca in prims:
        for b, cb in prims:
            raw += ca * cb * prim_overlap(a, center, b, center)
    scale = 1 / sqrt(raw)
    return (mpf(center), [(a, c * scale) for a, c in prims], raw)


def contraction_raw_norm():
    """<chi|chi> BEFORE renormalisation -- a diagnostic on the tabulated data."""
    _, _, raw = sto3g_h(0)
    return raw


# ---------------------------------------------------------------------------
# Contracted AO integrals for an arbitrary set of s-type contracted functions.
# ---------------------------------------------------------------------------
def ao_integrals(basis, nuclei):
    """
    basis  : list of (center, [(alpha, coeff), ...])
    nuclei : list of (center, Z)
    Returns S, T, V (n x n nested lists) and eri[i][j][k][l] in chemist notation.
    """
    n = len(basis)
    S = [[mpf(0)] * n for _ in range(n)]
    T = [[mpf(0)] * n for _ in range(n)]
    V = [[mpf(0)] * n for _ in range(n)]

    for i in range(n):
        Ai, pi_ = basis[i][0], basis[i][1]
        for j in range(i + 1):
            Aj, pj_ = basis[j][0], basis[j][1]
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
            for k in range(n):
                for l in range(k + 1):
                    if (i * (i + 1) // 2 + j) < (k * (k + 1) // 2 + l):
                        continue
                    acc = mpf(0)
                    for a, ca in basis[i][1]:
                        for b, cb in basis[j][1]:
                            for c, cc in basis[k][1]:
                                for d, cd in basis[l][1]:
                                    acc += (ca * cb * cc * cd * prim_eri(
                                        a, basis[i][0], b, basis[j][0],
                                        c, basis[k][0], d, basis[l][0]))
                    for (p, q, r, s_) in ((i, j, k, l), (j, i, k, l),
                                          (i, j, l, k), (j, i, l, k),
                                          (k, l, i, j), (l, k, i, j),
                                          (k, l, j, i), (l, k, j, i)):
                        eri[p][q][r][s_] = acc
    return S, T, V, eri


# ---------------------------------------------------------------------------
# Hydrogen atom in this same basis -- the in-model dissociation half-asymptote.
# One electron, one contracted 1s, one proton: E = <chi|h|chi> / <chi|chi>.
# ---------------------------------------------------------------------------
def h_atom_energy():
    basis = [sto3g_h(0)[:2]]
    S, T, V, _ = ao_integrals(basis, [(mpf(0), 1)])
    return (T[0][0] + V[0][0]) / S[0][0]


def asymptote():
    """In-model dissociation limit: two non-interacting STO-3G hydrogen atoms."""
    return 2 * h_atom_energy()


# ---------------------------------------------------------------------------
# H2 at internuclear separation R: AO integrals, symmetry MOs, MO integrals.
# ---------------------------------------------------------------------------
def h2_mo_integrals(R):
    R = mpf(R)
    basis = [sto3g_h(0)[:2], sto3g_h(R)[:2]]
    nuclei = [(mpf(0), 1), (R, 1)]
    S, T, V, eri = ao_integrals(basis, nuclei)
    Hcore = [[T[i][j] + V[i][j] for j in range(2)] for i in range(2)]

    # Symmetry-determined MOs for a homonuclear diatomic in a minimal basis:
    #   sigma_g = (chi_A + chi_B) / sqrt(2(1+S)),  sigma_u = (chi_A - chi_B)/sqrt(2(1-S))
    s = S[0][1]
    cg = 1 / sqrt(2 * (1 + s))
    cu = 1 / sqrt(2 * (1 - s))
    C = [[cg, cu], [cg, -cu]]          # C[ao][mo]

    hmo = [[mpf(0)] * 2 for _ in range(2)]
    for p in range(2):
        for q in range(2):
            acc = mpf(0)
            for i in range(2):
                for j in range(2):
                    acc += C[i][p] * C[j][q] * Hcore[i][j]
            hmo[p][q] = acc

    gmo = [[[[mpf(0)] * 2 for _ in range(2)] for _ in range(2)] for _ in range(2)]
    for p in range(2):
        for q in range(2):
            for r in range(2):
                for t_ in range(2):
                    acc = mpf(0)
                    for i in range(2):
                        for j in range(2):
                            for k in range(2):
                                for l in range(2):
                                    acc += (C[i][p] * C[j][q] * C[k][r] * C[l][t_]
                                            * eri[i][j][k][l])
                    gmo[p][q][r][t_] = acc

    return dict(S=S, T=T, V=V, eri=eri, Hcore=Hcore, C=C, hmo=hmo, gmo=gmo,
                S_AB=s, R=R)


def nuclear_repulsion(R):
    return 1 / mpf(R)


# ---------------------------------------------------------------------------
# ROUTE (a): singlet-sector 2x2 CI in the {(sigma_g)^2, (sigma_u)^2} basis.
#   H11 = 2 h_gg + (gg|gg)
#   H22 = 2 h_uu + (uu|uu)
#   H12 = (gu|gu)                       [Slater-Condon, doubly-substituted]
# The open-shell singlet is ungerade and does not couple to these two.
# ---------------------------------------------------------------------------
def fci_route_a(mo):
    h, g = mo["hmo"], mo["gmo"]
    H11 = 2 * h[0][0] + g[0][0][0][0]
    H22 = 2 * h[1][1] + g[1][1][1][1]
    H12 = g[0][1][0][1]
    tr = H11 + H22
    disc = sqrt((H11 - H22) ** 2 + 4 * H12 ** 2)
    return (tr - disc) / 2


# ---------------------------------------------------------------------------
# ROUTE (b): brute-force exact diagonalisation of the 4-spin-orbital Fock space
# (dimension 16), restricted to the 2-electron block (dimension 6).
# Spin orbitals: 0=(g,alpha) 1=(g,beta) 2=(u,alpha) 3=(u,beta).
#   H = sum_pq h_pq a+_p a_q + 1/2 sum_pqrs (pq|rs) a+_p a+_r a_s a_q
# with spin deltas on (p,q) and (r,s).  Fermionic signs are applied explicitly
# by the ladder operators; no Slater-Condon rule is used anywhere in this route.
# ---------------------------------------------------------------------------
def _spatial(p):
    return p >> 1


def _spin(p):
    return p & 1


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


def _apply_string(det, ops):
    """ops applied right to left; each op is ('c', p) create or ('a', p) annihilate."""
    sign = 1
    cur = det
    for kind, p in reversed(ops):
        res = _create(cur, p) if kind == "c" else _annihilate(cur, p)
        if res is None:
            return None
        s, cur = res
        sign *= s
    return sign, cur


def fock_dets(nso=4, nelec=2):
    return [d for d in range(1 << nso) if bin(d).count("1") == nelec]


def fci_route_b(mo, return_all=False):
    h, g = mo["hmo"], mo["gmo"]
    dets = fock_dets()
    idx = {d: i for i, d in enumerate(dets)}
    n = len(dets)
    H = matrix(n, n)

    for d in dets:
        col = idx[d]
        # one-electron
        for p in range(4):
            for q in range(4):
                if _spin(p) != _spin(q):
                    continue
                hv = h[_spatial(p)][_spatial(q)]
                if hv == 0:
                    continue
                res = _apply_string(d, [("c", p), ("a", q)])
                if res is None:
                    continue
                sg, nd = res
                H[idx[nd], col] += sg * hv
        # two-electron
        for p in range(4):
            for q in range(4):
                if _spin(p) != _spin(q):
                    continue
                for r in range(4):
                    for s_ in range(4):
                        if _spin(r) != _spin(s_):
                            continue
                        gv = g[_spatial(p)][_spatial(q)][_spatial(r)][_spatial(s_)]
                        if gv == 0:
                            continue
                        res = _apply_string(d, [("c", p), ("c", r), ("a", s_), ("a", q)])
                        if res is None:
                            continue
                        sg, nd = res
                        H[idx[nd], col] += mpf(sg) * gv / 2

    # symmetrise defensively (should already be symmetric to working precision)
    asym = mpf(0)
    for i in range(n):
        for j in range(n):
            asym = max(asym, abs(H[i, j] - H[j, i]))
    Hs = matrix(n, n)
    for i in range(n):
        for j in range(n):
            Hs[i, j] = (H[i, j] + H[j, i]) / 2

    evals, _ = eigsy(Hs)
    vals = sorted([evals[i] for i in range(n)])
    if return_all:
        return vals[0], vals, asym
    return vals[0]


# ---------------------------------------------------------------------------
# Total energies
# ---------------------------------------------------------------------------
def energy_route_a(R):
    mo = h2_mo_integrals(R)
    return fci_route_a(mo) + nuclear_repulsion(R)


def energy_route_b(R):
    mo = h2_mo_integrals(R)
    return fci_route_b(mo) + nuclear_repulsion(R)


def energy_both(R):
    mo = h2_mo_integrals(R)
    nr = nuclear_repulsion(R)
    return fci_route_a(mo) + nr, fci_route_b(mo) + nr


# default primary evaluator (route a; agreement with route b is gate G1)
def E(R):
    return energy_route_a(R)
