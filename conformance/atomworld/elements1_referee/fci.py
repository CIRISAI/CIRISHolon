"""
fci.py -- exact-in-model full configuration interaction, three independent
constructions, with a rigorously certified high-precision eigenvalue.

ORBITALS.  The FCI energy is invariant under any nonsingular transformation of
the orbital space, so no SCF is needed and none is done: the orbitals are the
Loewdin symmetric orthogonalisation S^{-1/2} of the AOs.  Route B additionally
applies a fixed orthogonal rotation Q to those orbitals, which changes every MO
integral and the entire CI vector while leaving the energy invariant -- so
routes A and B disagree in EVERY intermediate quantity and must agree in the
one number that is physical.

ROUTE A (primary).  Determinant CI in the Sz-restricted determinant basis
(alpha string x beta string), matrix elements by the Slater-Condon rules with
explicit excitation-rank classification and bit-counted phases.

ROUTE B (independent).  The spin-summed generator (unitary-group / Handy)
formulation
      H = sum_pq h'_pq E_pq + 1/2 sum_pqrs (pq|rs) E_pq E_rs,
      h'_pq = h_pq - 1/2 sum_r (pr|rq),     E_pq = sum_sigma a+_p,sigma a_q,sigma
built from one-electron string coupling coefficients only.  No excitation rank
is ever computed, same-spin and opposite-spin double excitations are never
distinguished, and no Slater-Condon rule appears.  Run in the ROTATED orbital
basis.

ROUTE C (a third, representational, witness -- small species only).  Brute-force
Fock space: determinants are ALL N-electron occupations of the 2*norb spin
orbitals (every Sz sector at once), and every matrix element is produced by
applying explicit creation/annihilation operator strings with bit-counted
fermionic signs, exactly as the banked h2_core route (b) does.  Diagonalised
densely.  This is the generalisation of the banked H2 referee's second route.

EIGENSOLVER AND ITS CERTIFICATE.  A double-precision solve provides a starting
vector; the eigenpair is then refined at the working precision by Newton /
inverse iteration, the correction equation being solved in double precision on
the orthogonal complement of the current vector (where H - lambda is positive
definite, so conjugate gradients apply).  Each outer step multiplies the vector
accuracy by roughly the double-precision solve tolerance, and the Rayleigh
quotient squares that.  Convergence is not asserted, it is CERTIFIED: for a
symmetric H the Rayleigh quotient theta and residual r = Hc - theta c obey
      min_i |theta - lambda_i| <= ||r||,     and (Temple)  lambda_0 >= theta -
      ||r||^2 / (lambda_1 - theta)
so the reported bound is computed from the run's own numbers at the working
precision.  The refinement is shared by routes A and B; it cannot manufacture
agreement between two different operators, because the certificate it returns is
computed from the operator it was given.
"""

import array
import itertools
import warnings
import math
from mpmath import mp, mpf, sqrt, matrix, eigsy, nstr

import numpy as np


# ===========================================================================
# Orbitals
# ===========================================================================
def _mat(A):
    n = len(A)
    M = matrix(n, len(A[0]))
    for i in range(n):
        for j in range(len(A[0])):
            M[i, j] = A[i][j]
    return M


def lowdin_orbitals(S):
    """C = S^{-1/2}, so that C^T S C = 1.  Symmetric, canonical, SCF-free."""
    n = len(S)
    evals, U = eigsy(_mat(S))
    C = [[mpf(0)] * n for _ in range(n)]
    inv = [1 / sqrt(evals[k]) for k in range(n)]
    for i in range(n):
        for j in range(n):
            acc = mpf(0)
            for k in range(n):
                acc += U[i, k] * inv[k] * U[j, k]
            C[i][j] = acc
    return C, [evals[k] for k in range(n)]


def _lcg_stream(seed):
    """A tiny deterministic integer generator -- no library RNG, so the rotation
    is reproducible exactly at any precision and on any platform."""
    x = seed
    while True:
        x = (6364136223846793005 * x + 1442695040888963407) % (1 << 64)
        yield x


def rotation_matrix(n, seed=20260828):
    """A fixed orthogonal n x n matrix, by Gram-Schmidt on a deterministic
    integer matrix.  Used only to prove orbital invariance of the FCI energy."""
    g = _lcg_stream(seed)
    A = [[mpf(next(g) % 2000001) / 1000000 - 1 for _ in range(n)]
         for _ in range(n)]
    Q = []
    for i in range(n):
        v = list(A[i])
        for u in Q:
            d = sum(u[k] * v[k] for k in range(n))
            v = [v[k] - d * u[k] for k in range(n)]
        nv = sqrt(sum(x * x for x in v))
        Q.append([x / nv for x in v])
    return [[Q[j][i] for j in range(n)] for i in range(n)]


def rotate_orbitals(C, Q):
    n, m = len(C), len(Q)
    return [[sum(C[i][k] * Q[k][j] for k in range(m)) for j in range(m)]
            for i in range(n)]


def mo_integrals(mol, C):
    """AO -> MO in four quarter transformations, O(n^5)."""
    n = mol["nbf"]
    H0, eri = mol["Hcore"], mol["eri"]
    h = [[sum(C[i][p] * sum(C[j][q] * H0[i][j] for j in range(n))
              for i in range(n)) for q in range(n)] for p in range(n)]
    t1 = [[[[mpf(0)] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]
    for j in range(n):
        for k in range(n):
            for l in range(n):
                for p in range(n):
                    acc = mpf(0)
                    for i in range(n):
                        acc += C[i][p] * eri[i][j][k][l]
                    t1[p][j][k][l] = acc
    t2 = [[[[mpf(0)] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]
    for p in range(n):
        for k in range(n):
            for l in range(n):
                for q in range(n):
                    acc = mpf(0)
                    for j in range(n):
                        acc += C[j][q] * t1[p][j][k][l]
                    t2[p][q][k][l] = acc
    t3 = [[[[mpf(0)] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]
    for p in range(n):
        for q in range(n):
            for l in range(n):
                for r in range(n):
                    acc = mpf(0)
                    for k in range(n):
                        acc += C[k][r] * t2[p][q][k][l]
                    t3[p][q][r][l] = acc
    g = [[[[mpf(0)] * n for _ in range(n)] for _ in range(n)] for _ in range(n)]
    for p in range(n):
        for q in range(n):
            for r in range(n):
                for s in range(n):
                    acc = mpf(0)
                    for l in range(n):
                        acc += C[l][s] * t3[p][q][r][l]
                    g[p][q][r][s] = acc
    return h, g


# ===========================================================================
# Determinant space: alpha strings x beta strings at fixed Sz.
# ===========================================================================
def strings(norb, nel):
    """All norb-bit masks of popcount nel, in increasing integer order."""
    return [d for d in range(1 << norb) if bin(d).count("1") == nel]


def occ_list(s, norb):
    return [i for i in range(norb) if (s >> i) & 1]


def _excite(s, m, p):
    """a+_p a_m |s>.  Returns (sign, s') or None if it annihilates."""
    if not (s >> m) & 1:
        return None
    s1 = s ^ (1 << m)
    if (s1 >> p) & 1:
        return None
    lo, hi = (m, p) if m < p else (p, m)
    mask = ((1 << hi) - 1) ^ ((1 << (lo + 1)) - 1)
    sign = -1 if bin(s1 & mask).count("1") & 1 else 1
    return sign, s1 | (1 << p)


class DetSpace:
    """Sz-restricted determinant basis.  Index = ia * nbs + ib."""

    def __init__(self, norb, nalpha, nbeta):
        self.norb, self.na, self.nb = norb, nalpha, nbeta
        self.astr = strings(norb, nalpha)
        self.bstr = strings(norb, nbeta)
        self.nas, self.nbs = len(self.astr), len(self.bstr)
        self.ndet = self.nas * self.nbs
        self.aidx = {s: i for i, s in enumerate(self.astr)}
        self.bidx = {s: i for i, s in enumerate(self.bstr)}
        self.aocc = [occ_list(s, norb) for s in self.astr]
        self.bocc = [occ_list(s, norb) for s in self.bstr]
        self.avir = [[i for i in range(norb) if not (s >> i) & 1]
                     for s in self.astr]
        self.bvir = [[i for i in range(norb) if not (s >> i) & 1]
                     for s in self.bstr]


# ===========================================================================
# ROUTE A -- Slater-Condon determinant CI.
# ===========================================================================
def _sc_diag(oa, ob, h, g):
    e = h[0][0] * 0
    for i in oa:
        e += h[i][i]
    for i in ob:
        e += h[i][i]
    for x in range(len(oa)):
        i = oa[x]
        for y in range(x):
            j = oa[y]
            e += g[i][i][j][j] - g[i][j][j][i]
    for x in range(len(ob)):
        i = ob[x]
        for y in range(x):
            j = ob[y]
            e += g[i][i][j][j] - g[i][j][j][i]
    for i in oa:
        for j in ob:
            e += g[i][i][j][j]
    return e


def _sc_single(m, p, same, other, h, g):
    """<D'|H|D> for a single excitation m->p in one spin channel.

    same  : orbitals occupied in the SAME spin channel, excluding m
    other : orbitals occupied in the other spin channel
    """
    v = h[m][p]
    for j in same:
        v += g[m][p][j][j] - g[m][j][j][p]
    for j in other:
        v += g[m][p][j][j]
    return v


def route_a_elements(space, h, g):
    """Yield (row, col, value) for the lower/upper-complete sparse H.

    Every connection is generated CONSTRUCTIVELY from each determinant (its own
    diagonal, its single and double excitations) and classified by excitation
    rank; the Slater-Condon rule for that rank supplies the value.
    """
    norb = space.norb
    nbs = space.nbs
    aidx, bidx = space.aidx, space.bidx
    for ia in range(space.nas):
        oa, va = space.aocc[ia], space.avir[ia]
        sa = space.astr[ia]
        # ---- alpha single and double excitations (beta spectator) ----
        asing = []          # (ja, m, p, sign)
        adoub = []          # (ja, m, n, p, q, sign)
        for m in oa:
            for p in va:
                r1 = _excite(sa, m, p)
                s1, sgn1 = r1[1], r1[0]
                asing.append((aidx[s1], m, p, sgn1))
                for n in oa:
                    if n <= m:
                        continue
                    for q in va:
                        if q <= p:
                            continue
                        r2 = _excite(s1, n, q)
                        if r2 is None:
                            continue
                        adoub.append((aidx[r2[1]], m, n, p, q, sgn1 * r2[0]))
        for ib in range(nbs):
            ob, vb = space.bocc[ib], space.bvir[ib]
            sb = space.bstr[ib]
            I = ia * nbs + ib
            yield (I, I, _sc_diag(oa, ob, h, g))
            for (ja, m, p, sgn) in asing:
                same = [j for j in oa if j != m]
                yield (ja * nbs + ib, I, sgn * _sc_single(m, p, same, ob, h, g))
            for (ja, m, n, p, q, sgn) in adoub:
                yield (ja * nbs + ib, I,
                       sgn * (g[m][p][n][q] - g[m][q][n][p]))
            # ---- beta single and double excitations (alpha spectator) ----
            for m in ob:
                for p in vb:
                    r1 = _excite(sb, m, p)
                    s1, sgn1 = r1[1], r1[0]
                    jb = bidx[s1]
                    same = [j for j in ob if j != m]
                    yield (ia * nbs + jb, I,
                           sgn1 * _sc_single(m, p, same, oa, h, g))
                    for n in ob:
                        if n <= m:
                            continue
                        for q in vb:
                            if q <= p:
                                continue
                            r2 = _excite(s1, n, q)
                            if r2 is None:
                                continue
                            yield (ia * nbs + bidx[r2[1]], I,
                                   sgn1 * r2[0] * (g[m][p][n][q]
                                                   - g[m][q][n][p]))
            # ---- mixed alpha/beta double excitations ----
            for (ja, m, p, sgna) in asing:
                for n in ob:
                    for q in vb:
                        r = _excite(sb, n, q)
                        yield (ja * nbs + bidx[r[1]], I,
                               sgna * r[0] * g[m][p][n][q])


# ===========================================================================
# ROUTE B -- spin-summed generator (unitary group) formulation.
# ===========================================================================
def _tri(p, q):
    return (p * (p + 1)) // 2 + q if p >= q else (q * (q + 1)) // 2 + p


def string_couplings(strs, idx, norb):
    """<K| E_pq |J> for one spin channel, as flat arrays.

    E_pq = a+_p a_q.  Entries carry the TRIANGULAR index of (p,q); the (p,q) and
    (q,p) entries land in the same slot, which is exactly the symmetrisation the
    two-electron contraction wants.
    """
    tri, src, tgt, sgn = [], [], [], []
    for j, s in enumerate(strs):
        for q in range(norb):
            if not (s >> q) & 1:
                continue
            tri.append(_tri(q, q)); src.append(j); tgt.append(j); sgn.append(1)
            for p in range(norb):
                if p == q or (s >> p) & 1:
                    continue
                sg, s2 = _excite(s, q, p)
                tri.append(_tri(p, q)); src.append(j)
                tgt.append(idx[s2]); sgn.append(sg)
    return (np.array(tri, dtype=np.int64), np.array(src, dtype=np.int64),
            np.array(tgt, dtype=np.int64), np.array(sgn, dtype=np.int64))


def hprime(h, g, norb):
    """h'_pq = h_pq - 1/2 sum_r (pr|rq); symmetric because (pr|rq) = (qr|rp)."""
    out = [[None] * norb for _ in range(norb)]
    for p in range(norb):
        for q in range(norb):
            acc = h[p][q]
            for r in range(norb):
                acc -= g[p][r][r][q] / 2
            out[p][q] = acc
    return out


class RouteB:
    """sigma = H c by  H = sum_pq h'_pq E_pq + 1/2 sum_pqrs (pq|rs) E_pq E_rs."""

    def __init__(self, space, h, g):
        self.sp = space
        norb = space.norb
        self.nt = norb * (norb + 1) // 2
        self.ac = string_couplings(space.astr, space.aidx, norb)
        self.bc = string_couplings(space.bstr, space.bidx, norb)
        hp = hprime(h, g, norb)
        self.hp_t = [None] * self.nt
        for p in range(norb):
            for q in range(p + 1):
                self.hp_t[_tri(p, q)] = hp[p][q]
        self.G_t = [[None] * self.nt for _ in range(self.nt)]
        for p in range(norb):
            for q in range(p + 1):
                for r in range(norb):
                    for s in range(r + 1):
                        self.G_t[_tri(p, q)][_tri(r, s)] = g[p][q][r][s]
        # double-precision mirrors
        self.hp_f = np.array([float(x) for x in self.hp_t])
        self.G_f = np.array([[float(self.G_t[i][j]) for j in range(self.nt)]
                             for i in range(self.nt)])

    # -- double precision, vectorised -------------------------------------
    def sigma_f64(self, c):
        sp = self.sp
        c = c.reshape(sp.nas, sp.nbs)
        D = np.zeros((self.nt, sp.nas, sp.nbs))
        t, s_, g_, sg = self.ac
        np.add.at(D.reshape(self.nt * sp.nas, sp.nbs),
                  t * sp.nas + g_, sg[:, None] * c[s_, :])
        Db = np.zeros((self.nt, sp.nbs, sp.nas))
        t, s_, g_, sg = self.bc
        np.add.at(Db.reshape(self.nt * sp.nbs, sp.nas),
                  t * sp.nbs + g_, sg[:, None] * c.T[s_, :])
        D += Db.transpose(0, 2, 1)
        G = (self.G_f @ D.reshape(self.nt, -1)).reshape(self.nt, sp.nas, sp.nbs)
        W = 0.5 * G
        out = np.zeros((sp.nas, sp.nbs))
        t, s_, g_, sg = self.ac
        contrib = sg[:, None] * (W[t, s_, :] + self.hp_f[t][:, None] * c[s_, :])
        np.add.at(out, g_, contrib)
        t, s_, g_, sg = self.bc
        contribT = sg[:, None] * (W.transpose(0, 2, 1)[t, s_, :]
                                  + self.hp_f[t][:, None] * c.T[s_, :])
        outT = np.zeros((sp.nbs, sp.nas))
        np.add.at(outT, g_, contribT)
        out += outT.T
        return out.reshape(-1)

    # -- working precision -------------------------------------------------
    def sigma_hp(self, c):
        sp = self.sp
        nas, nbs, nt = sp.nas, sp.nbs, self.nt
        D = [[[mpf(0)] * nbs for _ in range(nas)] for _ in range(nt)]
        t, s_, g_, sg = self.ac
        for k in range(len(t)):
            row_s = c[s_[k]]
            row_d = D[t[k]][g_[k]]
            if sg[k] > 0:
                for ib in range(nbs):
                    row_d[ib] += row_s[ib]
            else:
                for ib in range(nbs):
                    row_d[ib] -= row_s[ib]
        t, s_, g_, sg = self.bc
        for k in range(len(t)):
            Dk, jb, sb, sk = D[t[k]], g_[k], s_[k], sg[k]
            if sk > 0:
                for ia in range(nas):
                    Dk[ia][jb] += c[ia][sb]
            else:
                for ia in range(nas):
                    Dk[ia][jb] -= c[ia][sb]
        # G_t = sum_rs (pq|rs) D_rs
        G = [[[mpf(0)] * nbs for _ in range(nas)] for _ in range(nt)]
        for i in range(nt):
            Gi, Grow = G[i], self.G_t[i]
            for j in range(nt):
                w = Grow[j]
                if w == 0:
                    continue
                Dj = D[j]
                for ia in range(nas):
                    gi, dj = Gi[ia], Dj[ia]
                    for ib in range(nbs):
                        gi[ib] += w * dj[ib]
        half = mpf(1) / 2
        W = [[[half * G[i][ia][ib] for ib in range(nbs)] for ia in range(nas)]
             for i in range(nt)]
        out = [[mpf(0)] * nbs for _ in range(nas)]
        t, s_, g_, sg = self.ac
        for k in range(len(t)):
            ti, ia_s, ia_t, sk = t[k], s_[k], g_[k], sg[k]
            hpt, Wt, cs, ot = self.hp_t[ti], W[ti][ia_s], c[ia_s], out[ia_t]
            if sk > 0:
                for ib in range(nbs):
                    ot[ib] += Wt[ib] + hpt * cs[ib]
            else:
                for ib in range(nbs):
                    ot[ib] -= Wt[ib] + hpt * cs[ib]
        t, s_, g_, sg = self.bc
        for k in range(len(t)):
            ti, ib_s, ib_t, sk = t[k], s_[k], g_[k], sg[k]
            hpt, Wi = self.hp_t[ti], W[ti]
            if sk > 0:
                for ia in range(nas):
                    out[ia][ib_t] += Wi[ia][ib_s] + hpt * c[ia][ib_s]
            else:
                for ia in range(nas):
                    out[ia][ib_t] -= Wi[ia][ib_s] + hpt * c[ia][ib_s]
        return out


def spin_squared(space, c):
    """<S^2> for a normalised CI vector, as ||S_+ psi||^2 + Sz(Sz+1).

    WHY THIS EXISTS.  H commutes with S^2, so every Krylov and Davidson space
    built from a starting vector stays inside that vector's spin sector.  A
    subspace method can therefore converge beautifully -- small residual, tight
    Temple bound, dual routes agreeing -- onto a SPIN-EXCITED state, and every
    number it reports about itself will look right.  The engine lane hit exactly
    this from the other direction: a converged-looking Davidson sitting 0.07
    hartree above carbon's ground state because its Krylov space never left one
    spin sector.  A small residual is a statement about the subspace you are in,
    not about the spectrum.

    This is the cheap, decisive check.  S^2 = S_- S_+ + S_z(S_z+1), so for a
    normalised vector <S^2> = ||S_+ psi||^2 + S_z(S_z+1), and S_+ = sum_p
    a+_{p alpha} a_{p beta} is one pass over the determinants.  The answer must
    be S(S+1) for an integer or half-integer S; anything else means the vector is
    a spin-contaminated mixture rather than an eigenstate.
    """
    norb, nas, nbs = space.norb, space.nas, space.nbs
    two_sz = space.na - space.nb
    sz = mpf(two_sz) / 2
    base = sz * (sz + 1)
    if space.nb == 0 or space.na + 1 > norb:
        return base
    tgt = DetSpace(norb, space.na + 1, space.nb - 1)
    out = {}
    for ia in range(nas):
        sa = space.astr[ia]
        for ib in range(nbs):
            amp = c[ia * nbs + ib]
            if amp == 0:
                continue
            sb = space.bstr[ib]
            for p in range(norb):
                if not (sb >> p) & 1 or (sa >> p) & 1:
                    continue
                low = ((1 << p) - 1)
                par = (bin(sb & low).count("1") + bin(sa & low).count("1")) & 1
                k = (tgt.aidx[sa | (1 << p)] * tgt.nbs
                     + tgt.bidx[sb ^ (1 << p)])
                out[k] = out.get(k, mpf(0)) + (-amp if par else amp)
    return sum(v * v for v in out.values()) + base


def ground_level_spins(op, space, degen_tol=1e-9, kmax=8):
    """<S^2> of EVERY vector in the ground LEVEL, not of one arbitrary vector.

    Inside a degenerate level any basis is an eigenbasis, so no single vector's
    <S^2> means anything -- the solver returns whatever its path produced.  But
    if EVERY vector in the level reports the same multiplicity, the LEVEL is
    spin-pure and the multiplicity is resolved regardless of the degeneracy.
    Treating any degeneracy as unresolved throws that away: F2's triplet region
    is spatially two-fold because its pi orbitals are degenerate, and every
    vector there is a triplet, so a naive test would report F2's multiplicity as
    constant and miss the crossing entirely.  Only a level where the vectors
    DISAGREE -- singlet and triplet genuinely come together, as they do in F2's
    far tail -- is unanswerable, and that is reported rather than asserted.

    <S^2> is an O(1) quantity and the question is which integer S(S+1) it is, so
    double-precision eigenvectors are ample here; the reported ground-state
    <S^2> is still taken at the working precision elsewhere.
    """
    n = op.ndet
    if n == 1:
        return dict(level_size=1, two_S_in_level=[0], resolved=True,
                    gap_to_next=None, spin_pure=True)
    k = min(kmax, n - 1)
    if n <= 400:
        M = np.empty((n, n))
        e = np.zeros(n)
        for i in range(n):
            e[i] = 1.0
            M[:, i] = op.matvec_f64(e)
            e[i] = 0.0
        M = (M + M.T) / 2
        w, V = np.linalg.eigh(M)
    else:
        Lop = spl.LinearOperator((n, n), matvec=op.matvec_f64, dtype=float)
        try:
            w, V = spl.eigsh(Lop, k=k, which="SA", tol=0, maxiter=20000)
        except spl.ArpackNoConvergence as exc:
            if exc.eigenvalues is not None and len(exc.eigenvalues) >= 1:
                w, V = exc.eigenvalues, exc.eigenvectors
            else:
                return dict(level_size=1, two_S_in_level=[], resolved=False,
                            agree_on_value=False, is_a_spin_eigenvalue=False,
                            parity_matches_electron_count=False,
                            max_dev_from_S_S_plus_1=None, gap_to_next=None,
                            spin_pure=False)
    order = list(np.argsort(w))
    lam0 = w[order[0]]
    level = [i for i in order if abs(w[i] - lam0) <= degen_tol]
    outside = [i for i in order if i not in level]
    gap = float(w[outside[0]] - lam0) if outside else None
    twoS, worst_dev = [], mpf(0)
    for i in level:
        v = [mpf(float(x)) for x in V[:, i]]
        nrm = sqrt(_hp_dot(v, v))
        v = [x / nrm for x in v]
        t, dev = spin_from_s2(spin_squared(space, v))
        twoS.append(t)
        worst_dev = max(worst_dev, dev)
    # AGREEMENT IS NOT ENOUGH.  Inside a degenerate level the vectors are
    # arbitrary mixtures, and their <S^2> can land on the same value while
    # being no eigenvalue at all -- F2 past 8.9 bohr had every vector reporting
    # 2S = 1, a DOUBLET, for an eighteen-electron molecule.  They agreed
    # because they were all equally meaningless.  So the level counts as
    # resolved only if the vectors agree AND each <S^2> really is S(S+1);
    # otherwise the multiplicity is reported, not asserted.
    valid = worst_dev < mpf("1e-6")
    parity_ok = all((t % 2) == (space.na + space.nb) % 2 for t in twoS)
    return dict(level_size=len(level), two_S_in_level=sorted(set(twoS)),
                resolved=bool(len(set(twoS)) == 1 and valid and parity_ok),
                agree_on_value=len(set(twoS)) == 1,
                is_a_spin_eigenvalue=bool(valid),
                parity_matches_electron_count=bool(parity_ok),
                max_dev_from_S_S_plus_1=float(worst_dev),
                gap_to_next=gap, spin_pure=bool(len(set(twoS)) == 1 and valid))


def spin_from_s2(s2, tol=None):
    """Recover S from <S^2> = S(S+1), and how far off an exact S(S+1) it is."""
    s2 = mpf(s2)
    S = (sqrt(1 + 4 * s2) - 1) / 2
    twoS = int(mp.nint(2 * S))
    exact = mpf(twoS) / 2 * (mpf(twoS) / 2 + 1)
    return twoS, abs(s2 - exact)


# ===========================================================================
# ROUTE C -- brute-force Fock space with explicit ladder operators.
# The generalisation of the banked h2_core route (b): spin orbitals are
# interleaved p = 2*spatial + spin, every matrix element comes from applying an
# operator string with bit-counted fermionic signs, and no Slater-Condon rule
# and no excitation rank appear anywhere.  All Sz sectors are present at once.
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


def _apply_string(det, ops):
    sign, cur = 1, det
    for kind, p in reversed(ops):
        res = _create(cur, p) if kind == "c" else _annihilate(cur, p)
        if res is None:
            return None
        s, cur = res
        sign *= s
    return sign, cur


def route_c_cost(norb, nelec):
    """(Fock dimension, cost).  The cost is the LARGER of building H and of the
    dense symmetric diagonalisation that follows it -- an earlier version
    counted only the build and let LiH's 495-dimensional eigsy, at 1.2e8
    working-precision operations, run as if it were free."""
    nso = 2 * norb
    nd = math.comb(nso, nelec)
    # Every term here is a function of the DETERMINANT COUNT, which is only an
    # honest model of the work because `route_c_determinants` enumerates in
    # O(nd).  While the enumeration walked 2**nso integers this model was blind
    # to it -- see that function's note.  If the enumeration ever goes back to
    # filtering, this needs a `1 << nso` term or the guard stops guarding.
    return nd, max(nd * nelec * nelec * norb * norb, nd ** 3)


def route_c_determinants(nso, nelec):
    """Every nso-bit integer with exactly nelec bits set, in increasing order.

    THE ENUMERATION USED TO COST 2**nso REGARDLESS OF HOW FEW SURVIVED.

    It was `[d for d in range(1 << nso) if bin(d).count("1") == nelec]` -- walk
    every integer, keep the ones with the right population count.  That is
    2**nso iterations whatever the answer's size, and route_c_cost models the
    ANSWER (`math.comb(nso, nelec)`) rather than the SEARCH.  So the budget
    guard was blindest exactly where the answer was smallest:

        Ar2   nso = 36, nelec = 36  ->  1 determinant, 68,719,476,736 iterations
        NeAr  nso = 28, nelec = 28  ->  1 determinant,     268,435,456 iterations

    Ar2 passed the budget check with a modelled cost of 4.2e5 and then ran eight
    workers at 98% for fifty minutes without completing ONE geometry.  A guard
    that fires on the size of the result cannot see a search that is exponential
    in the basis and trivial in the result -- and a closed shell is precisely
    that case.

    Choosing the bit patterns directly makes the enumeration O(number of
    determinants), which is what route_c_cost was modelling all along.  The list
    is SORTED so the order is identical to the filter's, because `idx` maps
    determinant to row and a different order would be a different matrix -- same
    eigenvalue, different everything else, and no reason to accept the risk.
    """
    return sorted(sum(1 << p for p in c)
                  for c in itertools.combinations(range(nso), nelec))


def route_c_energy(norb, nelec, h, g):
    """Lowest eigenvalue of H in the full N-electron Fock space, dense."""
    nso = 2 * norb
    dets = route_c_determinants(nso, nelec)
    idx = {d: i for i, d in enumerate(dets)}
    n = len(dets)
    H = [[mpf(0)] * n for _ in range(n)]
    for d in dets:
        col = idx[d]
        occ = [p for p in range(nso) if (d >> p) & 1]
        for q in occ:
            sq = q & 1
            for p in range(sq, nso, 2):
                hv = h[p >> 1][q >> 1]
                if hv == 0:
                    continue
                res = _apply_string(d, [("c", p), ("a", q)])
                if res is None:
                    continue
                sg, nd_ = res
                H[idx[nd_]][col] += sg * hv
        for q in occ:
            sq = q & 1
            for s_ in occ:
                if s_ == q:
                    continue
                ss = s_ & 1
                for p in range(sq, nso, 2):
                    for r in range(ss, nso, 2):
                        gv = g[p >> 1][q >> 1][r >> 1][s_ >> 1]
                        if gv == 0:
                            continue
                        res = _apply_string(d, [("c", p), ("c", r),
                                                ("a", s_), ("a", q)])
                        if res is None:
                            continue
                        sg, nd_ = res
                        H[idx[nd_]][col] += mpf(sg) * gv / 2
    asym = mpf(0)
    M = matrix(n, n)
    for i in range(n):
        for j in range(n):
            asym = max(asym, abs(H[i][j] - H[j][i]))
            M[i, j] = (H[i][j] + H[j][i]) / 2
    if n == 1:
        return M[0, 0], asym, n
    ev = eigsy(M, eigvals_only=True)
    return min(ev[i] for i in range(n)), asym, n


# ===========================================================================
# Operator adapters
# ===========================================================================
import scipy.sparse as sps
import scipy.sparse.linalg as spl


def _tofloat4(g, n):
    return [[[[float(g[p][q][r][s]) for s in range(n)] for r in range(n)]
             for q in range(n)] for p in range(n)]


class RouteAOp:
    name = "A: determinant CI, Slater-Condon rules"

    def __init__(self, space, h, g, cache_hp=None):
        self.sp, self.h, self.g = space, h, g
        self.ndet = space.ndet
        n = space.norb
        hf = [[float(h[i][j]) for j in range(n)] for i in range(n)]
        gf = _tofloat4(g, n)
        # array.array keeps the COO compact: a determinant space of 14400 has
        # 8.8e6 nonzeros, which as Python lists of boxed scalars would cost most
        # of a gigabyte per worker.
        rows = array.array("i")
        cols = array.array("i")
        vals = array.array("d")
        for (i, j, v) in route_a_elements(space, hf, gf):
            rows.append(i); cols.append(j); vals.append(v)
        self.csr = sps.csr_matrix(
            (np.frombuffer(vals, dtype=np.float64),
             (np.frombuffer(rows, dtype=np.int32),
              np.frombuffer(cols, dtype=np.int32))),
            shape=(self.ndet, self.ndet))
        del rows, cols, vals
        self.diag_f64 = self.csr.diagonal()
        if cache_hp is None:
            cache_hp = self.ndet <= 3000
        self.hp_cache = None
        if cache_hp:
            r2, c2, v2 = [], [], []
            for (i, j, v) in route_a_elements(space, h, g):
                r2.append(i); c2.append(j); v2.append(v)
            self.hp_cache = (r2, c2, v2)

    def matvec_f64(self, c):
        return self.csr @ c

    def matvec_hp(self, c):
        out = [mpf(0)] * self.ndet
        if self.hp_cache is not None:
            r2, c2, v2 = self.hp_cache
            for k in range(len(v2)):
                out[r2[k]] += v2[k] * c[c2[k]]
        else:
            for (i, j, v) in route_a_elements(self.sp, self.h, self.g):
                out[i] += v * c[j]
        return out


def diagonal_f64(space, h, g):
    """<I|H|I> for every determinant, in double precision.

    Used ONLY as a Jacobi preconditioner for the double-precision correction
    equation.  A preconditioner cannot change the answer -- it changes how fast
    conjugate gradients reaches the same solution -- and the certificate the
    solver returns is computed from the operator it was actually given.  So it
    is legitimate for route B to precondition with this, even though the
    expression is route A's diagonal rule; what would be illegitimate, and is
    not done, is letting route A supply any part of route B's operator.
    """
    n = space.norb
    hf = [[float(h[i][j]) for j in range(n)] for i in range(n)]
    gf = _tofloat4(g, n)
    d = np.empty(space.ndet)
    for ia in range(space.nas):
        oa = space.aocc[ia]
        for ib in range(space.nbs):
            d[ia * space.nbs + ib] = _sc_diag(oa, space.bocc[ib], hf, gf)
    return d


class RouteBOp:
    name = "B: generator (unitary-group) CI in rotated orbitals"

    def __init__(self, space, h, g, precond=True):
        self.sp = space
        self.ndet = space.ndet
        self.B = RouteB(space, h, g)
        # A Jacobi preconditioner needs the diagonal.  For small spaces it is
        # read off this construction's own sigma, one unit vector at a time --
        # which also cross-checks diagonal_f64() against the generator algebra.
        # For large spaces that costs ndet sigma calls, so diagonal_f64() is
        # used instead; see its docstring for why a preconditioner cannot leak
        # into the answer.  Unpreconditioned, this correction equation needs
        # thousands of conjugate-gradient steps and dominates the whole run.
        if not precond:
            self.diag_f64 = None
        elif self.ndet <= 400:
            e = np.zeros(self.ndet)
            d = np.empty(self.ndet)
            for i in range(self.ndet):
                e[i] = 1.0
                d[i] = self.B.sigma_f64(e)[i]
                e[i] = 0.0
            self.diag_f64 = d
        else:
            self.diag_f64 = diagonal_f64(space, h, g)

    def matvec_f64(self, c):
        return self.B.sigma_f64(c)

    def matvec_hp(self, c):
        nas, nbs = self.sp.nas, self.sp.nbs
        cn = [c[ia * nbs:(ia + 1) * nbs] for ia in range(nas)]
        o = self.B.sigma_hp(cn)
        return [o[ia][ib] for ia in range(nas) for ib in range(nbs)]


# ===========================================================================
# Certified ground-state solve
# ===========================================================================
def _hp_dot(a, b):
    s = mpf(0)
    for i in range(len(a)):
        s += a[i] * b[i]
    return s


def _jd_correction(op, theta, Vf, rhat, cgtol):
    """Solve the correction equation on the orthogonal complement of the WHOLE
    current subspace, not just of the current vector.

    Deflating the near-degenerate partners is what makes this well-conditioned:
    restricted to span(V)^perp the operator H - theta is positive definite with
    condition number set by the gap to the first eigenvalue OUTSIDE the
    subspace, so conjugate gradients converges in tens of steps even where the
    ground state is nearly degenerate -- as it is at every dissociation limit,
    where the singlet and the triplet come together.
    """
    n = op.ndet

    def P(x):
        y = x
        for v in Vf:
            y = y - v * (v @ y)
        return y

    def A(x):
        y = P(x)
        y = op.matvec_f64(y) - theta * y
        return P(y)

    Aop = spl.LinearOperator((n, n), matvec=A, dtype=float)
    Mop = None
    if op.diag_f64 is not None:
        d = op.diag_f64 - theta
        d = np.where(np.abs(d) < 1e-10, 1e-10, d)
        Mop = spl.LinearOperator((n, n), matvec=lambda x: P(P(x) / d),
                                 dtype=float)
    b = P(rhat)
    t, info = spl.cg(Aop, b, rtol=cgtol, atol=0.0, maxiter=8000, M=Mop)
    return P(t), info


def _orthonormalise(V, v):
    """Modified Gram-Schmidt of v against V, at the working precision."""
    for _ in range(2):                     # twice, for numerical orthogonality
        for u in V:
            d = _hp_dot(u, v)
            v = [v[i] - d * u[i] for i in range(len(v))]
    nrm = sqrt(_hp_dot(v, v))
    if nrm == 0:
        return None
    return [x / nrm for x in v]


def _rayleigh_ritz(V, W):
    """Lowest Ritz pair of H restricted to span(V), given W = [H v for v in V].

    Returns (theta, c, r) with r = Hc - theta c, computed as a combination of
    the stored W -- so refining the Ritz vector costs no extra matvec.
    """
    m = len(V)
    M = matrix(m, m)
    for i in range(m):
        for j in range(m):
            M[i, j] = _hp_dot(V[i], W[j])
    for i in range(m):
        for j in range(i):
            avg = (M[i, j] + M[j, i]) / 2
            M[i, j] = M[j, i] = avg
    if m == 1:
        theta = M[0, 0]
        y = [mpf(1)]
    else:
        ev, U = eigsy(M)
        k = min(range(m), key=lambda i: ev[i])
        theta = ev[k]
        y = [U[i, k] for i in range(m)]
    n = len(V[0])
    c = [mpf(0)] * n
    Hc = [mpf(0)] * n
    for j in range(m):
        yj = y[j]
        if yj == 0:
            continue
        vj, wj = V[j], W[j]
        for i in range(n):
            c[i] += yj * vj[i]
            Hc[i] += yj * wj[i]
    r = [Hc[i] - theta * c[i] for i in range(n)]
    return theta, c, r


# THE DOUBLE-PRECISION SEED IS ONLY A SEED, AND MUST ALWAYS PRODUCE ONE.
#
# The working-precision Jacobi-Davidson loop below does the real work; ARPACK's
# job is to hand it a starting subspace and a rough gap.  But `tol=0` asks
# ARPACK for MACHINE precision on all k vectors, and at a dissociation limit the
# lowest levels are degenerate to about 1e-14 -- so it cannot separate them, and
# the request fails for a reason that has nothing to do with the quality of the
# seed.  CO at R = 9 bohr (14400 determinants, dissociated to C + O, both open
# shell) does exactly this: neither the k=6 tol=0 attempt nor the k=2 tol=1e-12
# retry converges, and stage 1 died on it with no fallback left.
#
# Each rung is tried only when the one above it produced nothing, so a geometry
# that succeeded on rung (a) computes bit-identically to before this ladder
# existed.  The rung actually used is recorded in the artifact's `seed` field,
# because a seed that needed the last resort is a fact about that geometry.
#
# The failing case is exercised in test_fci.py rather than assumed: rung (a) is
# forced to fail and the ladder is required to return a usable subspace.
def _lowest_diag_block(op, n, k):
    """Unit vectors on the k determinants of lowest diagonal: the Davidson
    start for a CI matrix, and deterministic (no RNG in a referee)."""
    d = getattr(op, "diag_f64", None)
    if d is None:
        idx = np.arange(min(k, n))
    else:
        idx = np.argsort(np.asarray(d, dtype=float))[:k]
    X = np.zeros((n, len(idx)))
    for c, i in enumerate(idx):
        X[i, c] = 1.0
    return X


def _f64_seed(op, Lop, n, k):
    """(eigenvalues, eigenvectors, label) -- at least two of each, always."""
    # (a) what every geometry before this ladder used
    try:
        w, V = spl.eigsh(Lop, k=k, which="SA", tol=0, maxiter=20000)
        return w, V, "sparse f64 Lanczos (ARPACK)"
    except spl.ArpackNoConvergence as exc:
        # (b) ARPACK's partial result carries the vectors that DID converge
        if exc.eigenvalues is not None and len(exc.eigenvalues) >= 2:
            return (exc.eigenvalues, exc.eigenvectors,
                    "sparse f64 Lanczos (ARPACK, partial: %d of %d converged)"
                    % (len(exc.eigenvalues), k))
    # (c) a wider Krylov basis and a tolerance appropriate to a SEED
    ncv = int(min(n - 1, max(2 * k + 1, 120)))
    for tol, mi in ((1e-10, 200000), (1e-8, 400000)):
        try:
            w, V = spl.eigsh(Lop, k=k, which="SA", tol=tol, ncv=ncv,
                             maxiter=mi)
            return w, V, ("sparse f64 Lanczos (ARPACK, ncv=%d, seed tol %g)"
                          % (ncv, tol))
        except spl.ArpackNoConvergence as exc:
            if exc.eigenvalues is not None and len(exc.eigenvalues) >= 2:
                return (exc.eigenvalues, exc.eigenvectors,
                        "sparse f64 Lanczos (ARPACK, ncv=%d, partial %d)"
                        % (ncv, len(exc.eigenvalues)))
    # (d) last resort: LOBPCG, Jacobi-preconditioned, from the lowest-diagonal
    # block.  It is the standard cure for a CI matrix and it always returns.
    X = _lowest_diag_block(op, n, max(k, 4))
    d = getattr(op, "diag_f64", None)
    M = None
    if d is not None:
        dv = np.asarray(d, dtype=float)
        shift = dv - dv.min() + 1.0
        M = spl.LinearOperator((n, n), matvec=lambda x: x / shift[:, None]
                               if x.ndim > 1 else x / shift, dtype=float)
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        w, V = spl.lobpcg(Lop, X, M=M, largest=False, tol=1e-9, maxiter=2000)
    w = np.asarray(w).ravel()
    if len(w) < 2:
        raise RuntimeError("f64 seed ladder exhausted: no usable subspace")
    return w, V, "LOBPCG, Jacobi-preconditioned, lowest-diagonal block"


def solve_certified(op, tol_digits=55, max_outer=6, verbose=False):
    """Ground-state energy of op at the working precision, with a certificate.

    Returns a dict: energy (mpf), residual norm, the two rigorous bounds, the
    per-iteration residual history, and the double-precision gap used.
    """
    n = op.ndet
    if n == 1:
        e = op.matvec_hp([mpf(1)])[0]
        # the vector is returned even here: a one-determinant space makes the
        # spin check trivial, but OMITTING it makes the check merely absent,
        # and an absent check reads the same as a passing one downstream
        return dict(energy=e, resid=mpf(0), bound_resid=mpf(0),
                    bound_temple=mpf(0), history=[], gap=None, outer=0,
                    subspace=1, vector=[mpf(1)],
                    seed="exact (one determinant)")
    # ---- double-precision seed
    if n <= 400:
        M = np.empty((n, n))
        e = np.zeros(n)
        for i in range(n):
            e[i] = 1.0
            M[:, i] = op.matvec_f64(e)
            e[i] = 0.0
        M = (M + M.T) / 2
        w, V = np.linalg.eigh(M)
        lam0, lam1, v0 = w[0], w[1], V[:, 0]
        seed = "dense f64 eigh"
    else:
        Lop = spl.LinearOperator((n, n), matvec=op.matvec_f64, dtype=float)
        k = min(6, n - 1)
        # ARPACK at tol=0 on a 14400-dimensional space with six requested
        # vectors does not always converge, and scipy's ArpackNoConvergence
        # cannot be pickled -- raised inside a worker it kills the Pool's result
        # thread and leaves the job LOOKING alive with every process still up.
        # Its partial result is perfectly usable: it carries the vectors that
        # did converge, and one is all the seed needs.
        w, V, seed = _f64_seed(op, Lop, n, k)
        o = np.argsort(w)
        lam0, lam1, v0 = w[o[0]], w[o[1]], V[:, o[0]]
    gap = float(lam1 - lam0)

    # ---- the working-precision subspace.  Seeded with the double-precision
    # ground vector, and with the first excited vector too where the two are
    # nearly degenerate, so the partner is deflated from the very first
    # correction rather than fought against.
    # Seed the subspace with the double-precision ground vector, and with the
    # next one or two where the spectrum is close, so the near-degenerate
    # partner is DEFLATED from the first correction rather than fought against.
    # Near a dissociation limit the singlet and triplet come together and the
    # correction equation's condition number is set by that gap; without the
    # extra seeds the residual stalls around 1e-25 and Temple's bound -- which
    # divides by the same gap -- then fails to cover the reported digits.
    #
    # (The earlier form of this test added EVERY remaining eigenvector on the
    # dense path, where eigh returns all n of them.  It never fired in the runs
    # that were kept, but it would have cost one working-precision matvec per
    # determinant the moment it did.)
    order = np.argsort(w)
    # Seed the subspace with the WHOLE near-degenerate cluster, not just the
    # ground vector, and deflate all of it in the correction equation.
    #
    # At a dissociation limit the singlet and the triplet come together -- HF at
    # 10 bohr has a gap of 1.0e-10 -- and there the residual stops being a
    # measure of convergence for a SINGLE vector: every direction inside the
    # cluster has nearly the same Rayleigh quotient, so a correction can lower
    # the Ritz value while RAISING the residual, which is exactly what route B
    # did (7.5e-14 -> 5.0e-10, then stalled). Temple's bound divides by that
    # same gap, so the certificate collapses with it. Converging the cluster as
    # a block fixes both.
    seeds = [V[:, order[0]]]
    lam_lo = w[order[0]]
    for k in range(1, len(order)):
        if len(seeds) >= 8:
            break
        if abs(w[order[k]] - lam_lo) < 1e-6 or gap < 1e-1 and k < 3:
            seeds.append(V[:, order[k]])
        elif k >= 3:
            break
    Vh, Wh, Vf = [], [], []
    for sv in seeds:
        u = _orthonormalise(Vh, [mpf(float(x)) for x in sv])
        if u is None:
            continue
        Vh.append(u)
        Wh.append(op.matvec_hp(u))
        f = np.array([float(x) for x in u])
        Vf.append(f / np.linalg.norm(f))

    tol = mpf(10) ** (-tol_digits)
    hist = []
    theta = rn = None
    c = None
    for it in range(max_outer):
        theta, c, r = _rayleigh_ritz(Vh, Wh)
        rn = sqrt(_hp_dot(r, r))
        hist.append(rn)
        if verbose:
            print("    outer %d (subspace %d): ||r|| = %s"
                  % (it, len(Vh), nstr(rn, 5)))
        if rn < tol or it == max_outer - 1:
            break
        # Stop early on a stall ONLY once the answer is already good enough.
        # The stopping rule has to be tied to what the artifact needs, not to
        # how fast the last step happened to go: at HF's 7.5-bohr geometry
        # route B improved slowly for three iterations, tripped a
        # rate-of-progress test and stopped at ||r|| = 5.4e-13, while route A on
        # the same geometry ran to 8.4e-44.  Both certificates were honest and
        # consistent -- the disagreement sat inside route B's own bound -- but
        # route B had quit a long way above the precision being published.
        covered = (rn * rn / mpf(gap) < tol) if gap and gap > 0 else (rn < tol)
        if covered and len(hist) >= 3 and hist[-1] > hist[-2] / 2:
            break
        # (An earlier version broke as soon as a step gained less than a factor
        # of ten.  That fired on geometries where the gap is closing toward
        # dissociation, stopping route A three iterations early and leaving a
        # Temple bound of 7e-47 -- fewer than the 50 digits being reported --
        # while route B, which happened not to trip the test, reached 4e-30 at
        # the same geometry.  The certificate caught it; the energies did not
        # look wrong.)
        rf = np.array([float(x / rn) for x in r])
        if not np.all(np.isfinite(rf)):
            break
        t, info = _jd_correction(op, float(theta), Vf, rf, 1e-14)
        if not np.all(np.isfinite(t)) or np.linalg.norm(t) == 0:
            break
        u = _orthonormalise(Vh, [mpf(float(x)) for x in t])
        if u is None:
            break
        Vh.append(u)
        Wh.append(op.matvec_hp(u))
        f = np.array([float(x) for x in u])
        Vf.append(f / np.linalg.norm(f))
    bound_resid = rn
    bound_temple = rn * rn / mpf(gap) if gap > 0 else None
    return dict(energy=theta, resid=rn, bound_resid=bound_resid,
                bound_temple=bound_temple, history=hist, gap=gap,
                outer=len(hist), subspace=len(Vh), seed=seed, vector=c)
