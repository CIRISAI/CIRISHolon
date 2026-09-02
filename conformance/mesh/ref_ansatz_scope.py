"""What SPACE does ref_invariants.py search? Measured basis-independently, not asserted.

The lead's question, and it is the right one: if the solve ran over translation-invariant
charts only, staggered momenta sit OUTSIDE the ansatz and "zero spurious" would be true of a
smaller space than the sentence implies -- "we looked where they could not be" rather than
"we looked and they are absent."

THE ANSATZ IS THE FULL SITE-DEPENDENT SPACE. The unknown in `invariant_space` is w[c][d],
one free variable per (cell, direction): all 6L^2 of them, nothing assumed translation-
invariant. The collapse to one value per (direction, line) is a DERIVED consequence of the
dynamics, not a restriction on the search -- put a single particle at cell c0 in direction d0
and conservation reads w[c0+DIR[d0]][d0] = w[c0][d0] directly, because single-particle states
are alone in their (N,P) fiber and every sector-preserving C fixes them.

A staggered or per-line invariant is exactly a POSITION-DEPENDENT weight, and it survives
that collapse whenever its sign pattern is constant along each direction's own lines -- which
is how HPP's per-line momenta appear below. So such functionals are inside the space, and the
question "are there any for FHP-I" is one the solver is able to answer either way.

WHAT IS REPORTED, and why it is not the naive count. Counting "how many returned basis vectors
look position-dependent" is BASIS-DEPENDENT: the SVD returns an arbitrary orthonormal basis of
the nullspace, so it can report zero flat vectors for a space that plainly contains mass. The
basis-independent quantity is the DIMENSION of the translation-invariant SUBSPACE of the
invariant space, obtained by re-solving with the weights forced constant across each
direction's lines. `dim - dim_TI` is then the number of genuinely position-dependent invariants,
and it cannot be moved by a change of basis.
"""
import numpy as np
from lattice_common import DIR, fhp_i
from ref_invariants import line_labels

HPPD = [[1, 0], [0, 1], [-1, 0], [0, -1]]


def _rows(L, C, dirs, translation_invariant):
    """Conservation rows, either over one weight per (direction, line) or per direction."""
    n = len(dirs)
    labs = line_labels(L, dirs)
    if translation_invariant:
        col = lambda d, c: d
        total = n
    else:
        offset, total = [], 0
        for _, k in labs:
            offset.append(total)
            total += k
        col = lambda d, c: offset[d] + labs[d][0][c]
    rows = []
    for c in range(L * L):
        for s in range(len(C)):
            if C[s] == s:
                continue
            r = np.zeros(total)
            for d in range(n):
                r[col(d, c)] += ((C[s] >> d) & 1) - ((s >> d) & 1)
            if r.any():
                rows.append(r)
    return np.array(rows) if rows else np.zeros((0, total)), total


def dims(L, C, dirs):
    out = []
    for ti in (False, True):
        A, total = _rows(L, C, dirs, ti)
        if A.shape[0] == 0:
            out.append(total)
            continue
        sv = np.linalg.svd(A, compute_uv=False)
        rank = int((sv > 1e-9 * sv[0] * max(A.shape)).sum())
        out.append(total - rank)
    return out[0], out[1]


if __name__ == "__main__":
    hc = list(range(16)); hc[5], hc[10] = 10, 5
    print("dim      = the full site-dependent invariant space (the ansatz: one weight per cell and direction)")
    print("dim_TI   = its translation-invariant subspace")
    print("dim-dim_TI = invariants that are GENUINELY POSITION-DEPENDENT -- the staggered shape\n")
    print(f"{'system':<34} {'L':>3} {'dim':>5} {'dim_TI':>7} {'position-dependent':>19}")
    for L in (4, 6, 8):
        d, t = dims(L, list(range(64)), DIR)
        print(f"{'identity collision (streaming)':<34} {L:>3} {d:>5} {t:>7} {d-t:>19}")
    for L in (4, 6, 8, 12):
        d, t = dims(L, hc, HPPD)
        print(f"{'HPP-4':<34} {L:>3} {d:>5} {t:>7} {d-t:>19}")
    for L in (4, 6, 8, 10, 12, 16):
        d, t = dims(L, fhp_i(), DIR)
        print(f"{'FHP-I':<34} {L:>3} {d:>5} {t:>7} {d-t:>19}")
