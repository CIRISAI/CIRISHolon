"""EVERY linear invariant of the tier's own motion, found by solving for them.

The lead suggested staking Zanetti's staggered momentum invariants (Phys. Rev. A 40,
1989) in advance. Rather than stake a remembered formula, this solves for the COMPLETE
space of linear invariants and reports its dimension: anything beyond mass and the two
momentum components is a spurious invariant, and each one is an EXTRA exactly-closed
view the headline claim would have to account for.

THE SYSTEM, DERIVED RATHER THAN ASSUMED. Write L(x) = sum_{c,d} w[c][d] x_d(c). Then
L(Tx) = sum_{c,d} w[c+DIR[d]][d] C(x)_d(c). Every single-particle state is alone in its
(N,P) fiber, so C fixes all of them, and those states ALONE force
w[c+DIR[d]][d] = w[c][d]: the weight is constant along the lines in direction d. That
collapses the unknowns from 6L^2 to one per (direction, line), and what remains is a
per-cell condition on the collision, imposed for every local state it moves.

THE INSTRUMENT IS GAUGED IN BOTH DIRECTIONS and must be, or "FHP-I has none" would be a
statement about the solver. The identity collision must return one invariant per
(direction, line); HPP-4 must return its textbook per-line momenta.
"""
import numpy as np
from lattice_common import DIR, fhp_i


def line_labels(L, dirs):
    """For each direction, the orbit index of each cell under stepping in that direction.

    Computed by walking the orbit rather than by a closed form, so it stays correct for
    any direction set and any L.
    """
    out = []
    for a, b in dirs:
        lab = [-1] * (L * L)
        k = 0
        for start in range(L * L):
            if lab[start] >= 0:
                continue
            i, j = divmod(start, L)
            while lab[i * L + j] < 0:
                lab[i * L + j] = k
                i, j = (i + a) % L, (j + b) % L
            k += 1
        out.append((lab, k))
    return out


def invariant_space(L, C, dirs=DIR, tol=1e-9, return_basis=False):
    n = len(dirs)
    labs = line_labels(L, dirs)
    offset, total = [], 0
    for _, k in labs:
        offset.append(total)
        total += k
    rows = []
    for c in range(L * L):
        cols = [offset[d] + labs[d][0][c] for d in range(n)]
        for s in range(len(C)):
            if C[s] == s:
                continue
            r = np.zeros(total)
            for d in range(n):
                r[cols[d]] += ((C[s] >> d) & 1) - ((s >> d) & 1)
            if r.any():
                rows.append(r)
    if not rows:
        return (total, np.eye(total)) if return_basis else total
    A = np.array(rows)
    u, sv, vt = np.linalg.svd(A)
    rank = int((sv > tol * sv[0] * max(A.shape)).sum()) if sv.size else 0
    dim = total - rank
    return (dim, vt[rank:]) if return_basis else dim


if __name__ == "__main__":
    HPPD = [[1, 0], [0, 1], [-1, 0], [0, -1]]
    hc = list(range(16)); hc[5], hc[10] = 10, 5
    print("FHP-I (the campaign's collision), on the hex torus:")
    for L in (4, 6, 8, 10, 12, 16):
        d = invariant_space(L, fhp_i())
        print(f"  L={L:<3} linear invariants: {d:>3}   spurious beyond mass+2 momenta: {d-3:>3}")
    print("\nGAUGE 1 -- identity collision (streaming alone): must be 6L, one per line.")
    for L in (4, 6, 8):
        d = invariant_space(L, list(range(64)))
        print(f"  L={L:<3} invariants: {d:>3}   6L = {6*L:>3}   {'as expected' if d == 6*L else 'MISMATCH'}")
    print("\nGAUGE 2 -- HPP-4: must find its textbook per-line momenta (2L+1).")
    for L in (4, 6, 8, 12):
        d = invariant_space(L, hc, HPPD)
        print(f"  L={L:<3} invariants: {d:>3}   2L+1 = {2*L+1:>3}   spurious {d-3:>3}   "
              f"{'as expected' if d == 2*L+1 else 'MISMATCH'}")
