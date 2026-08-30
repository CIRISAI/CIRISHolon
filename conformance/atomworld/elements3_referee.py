#!/usr/bin/env python3
"""The 50-digit referee, extended to d shells and to Z = 19..54.

# What lives here and what lives in the shared core

Two edits were needed inside `elements1_referee/elements_core.py` and they are the whole
footprint there: a `CART[2]` key, and `_self_overlap` generalised from a two-branch special
case to a formula in `l`. Both are inert below `l = 2`, and
`elements1_referee/second_row_baseline.txt` checks that as EXACT equality on 424 values
rather than as an argument.

Everything else is here, so the shared core stays small and the referee lane has one file
to review rather than a diff spread through theirs:

* the per-component Cartesian normalisation;
* the 5x6 spherical projection;
* the Z = 19..54 basis table.

# The convention, stated end to end

The engine's basis DECLARES five spherical components per d shell (AMENDMENT A1.1 of
ELEMENTS3_PREREG; `md::SPHERICAL_D`). A referee grades the declared model, so it applies
the same projection in the same component order. Grading a six-component basis against a
five-component engine would produce a disagreement that is neither side's error.

# Why the normalisation needs fixing rather than inheriting

`prim_norm` is already per-component-correct — it carries the double factorial — but
`Shell` stores only the `(l, 0, 0)` component's value and uses it for every component of
the shell. For s that is trivially right; for p it is right by accident, because
`(1,0,0)`, `(0,1,0)` and `(0,0,1)` share a normalisation; for d it is WRONG, because
`(2,0,0)` and `(1,1,0)` differ by exactly sqrt(3).

Rather than change how `Shell` stores its primitives — which would touch the l <= 1 path
and put the bit-identity result at risk for no gain — the factor is applied afterwards, as
a per-basis-function scale. It is identically 1 for every `l <= 1` component, so it cannot
move a second-row number even in principle.
"""

import importlib.util
import json
import os
import sys

from mpmath import mp, mpf, sqrt

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "elements1_referee"))

import elements_core as E  # noqa: E402


# ---------------------------------------------------------------- the basis table

def _generator():
    spec = importlib.util.spec_from_file_location(
        "elements3_transcribe", os.path.join(HERE, "elements3_transcribe.py")
    )
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


# `ShellKind` to angular momentum, matching `elements::ShellKind::l`.
_KIND_L = {"S1": 0, "S2": 0, "P2": 1, "S3": 0, "P3": 1, "D3": 2,
           "S4": 0, "P4": 1, "D4": 2, "S5": 0, "P5": 1}

_REGISTRY = os.path.join(
    HERE, "..", "..", "engine", "crates", "holon-chem", "src")


def heavy_table():
    """`{Z: ((l, exps, coeffs), ...)}` for Z = 1..54, in the engine's basis order.

    # Parsed from the engine's DECLARATION, not regenerated beside it

    The first version of this built the heavy rows from the same pinned tabulation and the
    same generator the Rust registry was emitted from, at the same eight-decimal rounding,
    and reasoned that it must therefore agree. It did not: krypton came out 6.3e-7 hartree
    from the engine, two hundred times f64 noise.

    The cause was ONE rounding tie. The 2p contraction's leading coefficient is
    0.155916275 in the tabulation, exactly half way; this rounded it to 0.15591628 and the
    registry declares 0.15591627. Both are defensible roundings of the same source and they
    are different bases, so the referee was grading a model the engine does not have.

    A referee grades the DECLARED model. The declaration is `elements.rs`, so that is what
    is read -- exponents and coefficients both, by parsing the constants and the species
    blocks. Regenerating the same numbers independently is how a referee comes to disagree
    with its subject about something neither of them got wrong.

    Z = 1..18 keep the referee's own `STO3G_SHELLS` unchanged, so the second row still
    grades against exactly what it graded against before.
    """
    import re

    src = open(os.path.join(_REGISTRY, "elements.rs")).read()
    sto = open(os.path.join(_REGISTRY, "sto3g.rs")).read()

    consts = {}
    for m in re.finditer(r"pub const (C_[A-Z0-9_]+): \[f64; 3\] = \[([^\]]+)\];", src):
        consts[m.group(1)] = tuple(x.strip() for x in m.group(2).split(","))
    # C_1S is declared BY REFERENCE to the hydrogen constant, which is the whole point of
    # the crate's hydrogen-by-reference discipline -- so it is resolved the same way.
    m = re.search(r"pub const H_COEFFS: \[f64; 3\] = \[([^\]]+)\];", sto)
    consts["C_1S"] = tuple(x.strip() for x in m.group(1).split(","))

    table = dict(E.STO3G_SHELLS)
    # Anchored at line start on BOTH ends. Unanchored, the match runs from the indented
    # `Species::HYDROGEN` inside `impl Species` -- whose terminator is indented too -- past
    # it and on to the first line-start `};`, silently swallowing potassium's whole block.
    for blk in re.finditer(
        r"^pub const [A-Z]+: Species = Species \{\n(.*?)^\};", src, re.S | re.M
    ):
        body = blk.group(1)
        zm = re.search(r"\bz: (\d+),", body)
        if not zm:
            continue
        z = int(zm.group(1))
        if z < 19:
            continue
        rows = []
        for sh in re.finditer(
            r"kind: ShellKind::(\w+),\s*alpha: \[([^\]]+)\],\s*coeff: (\w+),", body
        ):
            kind, alpha, cname = sh.group(1), sh.group(2), sh.group(3)
            assert cname in consts, f"Z={z}: {kind} names an unknown constant {cname}"
            rows.append((_KIND_L[kind],
                         tuple(x.strip() for x in alpha.split(",")),
                         consts[cname]))
        assert rows, f"Z={z}: no shells parsed out of the registry"
        table[z] = tuple(rows)
    missing = [z for z in range(19, 55) if z not in table]
    assert not missing, f"registry parse missed Z = {missing}"
    return table


# ---------------------------------------------------------------- normalisation

def cart_factor(lmn):
    """`prim_norm(a, lmn) / prim_norm(a, (L, 0, 0))`, which is independent of `a`.

    From the double-factorial form, the exponent-dependent parts cancel and only
    `sqrt(dfact(L) / (dfact(l) dfact(m) dfact(n)))` survives. Identically 1 whenever
    `L <= 1`, and `sqrt(3)` for the three d products against the three d squares.
    """
    l, m, n = lmn
    L = l + m + n
    num = E._dfact(L)
    den = E._dfact(l) * E._dfact(m) * E._dfact(n)
    return sqrt(mpf(num) / mpf(den))


def cart_scales(labels):
    """One factor per basis function, in basis order."""
    return [cart_factor(lmn) for (_iat, _Z, _l, lmn) in labels]


# ---------------------------------------------------------------- the projection

def _sqrt3_over_2():
    return sqrt(mpf(3)) / 2


def spherical_d():
    """The 5x6 matrix, in the component order of `CART[2]`.

    Derived rather than copied from the engine, so agreement between the two is evidence
    rather than a shared transcription. The six normalised Cartesian functions are not
    orthogonal — the three squares overlap each other at exactly 1/3 — so the combinations
    are normalised against that metric:

        d(z^2)     = (2 g_zz - g_xx - g_yy) / 2
        d(x^2-y^2) = sqrt(3)/2 (g_xx - g_yy)
        d(xy), d(xz), d(yz)  are already orthonormal

    and the direction removed is exactly `g_xx + g_yy + g_zz`, the spherically symmetric
    one.
    """
    h = mpf(1) / 2
    s3h = _sqrt3_over_2()
    zero = mpf(0)
    one = mpf(1)
    return [
        [-h, -h, one, zero, zero, zero],   # z^2
        [zero, zero, zero, zero, one, zero],  # xz
        [zero, zero, zero, zero, zero, one],  # yz
        [s3h, -s3h, zero, zero, zero, zero],  # x^2 - y^2
        [zero, zero, zero, one, zero, zero],  # xy
    ]


def projection(shells):
    """The `n_sph x n_cart` matrix: identity per s/p shell, `spherical_d()` per d shell."""
    d5 = spherical_d()
    n_cart = sum(sh.ncart() for sh in shells)
    n_sph = sum(5 if sh.l == 2 else sh.ncart() for sh in shells)
    P = [[mpf(0)] * n_cart for _ in range(n_sph)]
    r = c = 0
    for sh in shells:
        if sh.l == 2:
            for a in range(5):
                for b in range(6):
                    P[r + a][c + b] = d5[a][b]
            r += 5
            c += 6
        else:
            k = sh.ncart()
            for i in range(k):
                P[r + i][c + i] = mpf(1)
            r += k
            c += k
    assert (r, c) == (n_sph, n_cart)
    return P


def _apply2(P, M):
    """`P M P^T`."""
    n, m = len(P), len(P[0])
    PM = [[mpf(0)] * m for _ in range(n)]
    for a in range(n):
        Pa = P[a]
        for k in range(m):
            w = Pa[k]
            if w:
                Mk = M[k]
                row = PM[a]
                for j in range(m):
                    row[j] += w * Mk[j]
    out = [[mpf(0)] * n for _ in range(n)]
    for a in range(n):
        for b in range(n):
            acc = mpf(0)
            Pb = P[b]
            for j in range(m):
                w = Pb[j]
                if w:
                    acc += PM[a][j] * w
            out[a][b] = acc
    return out


def _apply4(P, G):
    """The four-index array, one axis at a time.

    Four sweeps of `O(n_cart^3 n_sph)` rather than one contraction of
    `O(n_cart^4 n_sph^4)`. At mpmath precision the difference is the whole feasibility of
    the thing.
    """
    n, m = len(P), len(P[0])
    cur = G
    for axis in range(4):
        d = len(cur), len(cur[0]), len(cur[0][0]), len(cur[0][0][0])
        e = list(d)
        e[axis] = n
        nxt = [[[[mpf(0)] * e[3] for _ in range(e[2])] for _ in range(e[1])]
               for _ in range(e[0])]
        for i0 in range(e[0]):
            for i1 in range(e[1]):
                for i2 in range(e[2]):
                    for i3 in range(e[3]):
                        idx = [i0, i1, i2, i3]
                        acc = mpf(0)
                        for k in range(d[axis]):
                            w = P[idx[axis]][k]
                            if not w:
                                continue
                            src = list(idx)
                            src[axis] = k
                            acc += cur[src[0]][src[1]][src[2]][src[3]] * w
                        nxt[i0][i1][i2][i3] = acc
        cur = nxt
    return cur


# ---------------------------------------------------------------- the driver

def molecule(atoms, want_eri=True, screen=None):
    """The declared-basis integrals: Cartesian assembly, per-component scale, projection.

    Returns the same shape `elements_core.molecule` does, with `nbf` the SPHERICAL count.
    """
    table = heavy_table()
    shells, labels = E.build_basis([(Z, [mpf(x) for x in c]) for Z, c in atoms], table)
    nuclei = [(tuple(mpf(x) for x in c), Z) for Z, c in atoms]
    S, T, V, eri = E.ao_integrals(shells, nuclei, screen, want_eri=want_eri)

    # 1. the per-component Cartesian normalisation, which Shell does not carry
    f = cart_scales(labels)
    n = len(f)
    for M in (S, T, V):
        for i in range(n):
            for j in range(n):
                M[i][j] *= f[i] * f[j]
    if want_eri:
        for i in range(n):
            for j in range(n):
                for k in range(n):
                    for l in range(n):
                        eri[i][j][k][l] *= f[i] * f[j] * f[k] * f[l]

    # 2. the projection to the declared five components per d shell
    if any(sh.l == 2 for sh in shells):
        P = projection(shells)
        S, T, V = _apply2(P, S), _apply2(P, T), _apply2(P, V)
        if want_eri:
            eri = _apply4(P, eri)
        labels = [lb for lb in labels if lb[2] != 2][: len(P)]

    n = len(S)
    Hcore = [[T[i][j] + V[i][j] for j in range(n)] for i in range(n)]
    return dict(shells=shells, nuclei=nuclei, S=S, T=T, V=V, eri=eri, Hcore=Hcore,
                nbf=n, E_nuc=E.nuclear_repulsion(nuclei),
                nelec=sum(Z for Z, _ in atoms), atoms=atoms)


def closed_shell_energy(m):
    """The energy of the ALL-OCCUPIED closed-shell determinant, exactly.

    # Why this particular species is refereeable and most are not

    A 50-digit FCI needs an eigensolve over the determinant space, which is what stops the
    referee well short of the freeze's 3e4-determinant threshold. But krypton and xenon in
    this basis are ONE determinant: every orbital the basis provides is doubly occupied.
    A determinant with no empty orbital is unique up to a phase, so its energy is invariant
    under orbital rotation and is therefore a pure function of the AO integrals -- no
    eigensolve, no SCF, and nothing chosen.

    That makes exactly the two atoms E1 asserts "exactly" the two that a referee can reach
    at full precision, which is a coincidence worth using rather than stepping over.

    With every orbital occupied the AO density is `D = 2 S^-1`, since `C C^T = S^-1` for a
    square coefficient matrix satisfying `C^T S C = I`. The rest is the closed-shell
    expression.
    """
    n = m["nbf"]
    S = mp.matrix(n, n)
    for i in range(n):
        for j in range(n):
            S[i, j] = m["S"][i][j]
    Sinv = mp.inverse(S)
    D = [[2 * Sinv[i, j] for j in range(n)] for i in range(n)]

    e1 = mpf(0)
    for u in range(n):
        for v in range(n):
            e1 += D[v][u] * m["Hcore"][u][v]

    e2 = mpf(0)
    g = m["eri"]
    for u in range(n):
        for v in range(n):
            duv = D[v][u]
            if not duv:
                continue
            for l in range(n):
                for sg in range(n):
                    dls = D[sg][l]
                    if not dls:
                        continue
                    e2 += duv * dls * (g[u][v][l][sg] - g[u][sg][l][v] / 2) / 2
    return e1 + e2 + m["E_nuc"]


if __name__ == "__main__":
    from mpmath import nstr

    mp.dps = int(os.environ.get("REF_DPS", "50"))
    for arg in sys.argv[1:] or ["36"]:
        z = int(arg)
        m = molecule([(z, ("0.0", "0.0", "0.0"))])
        e = closed_shell_energy(m)
        # One record line per atom: symbol-free, so the consumer keys on Z and cannot
        # mis-attribute a row by a label this file made up.
        print(f"atom {z} {m['nbf']} {mp.dps} {nstr(e, 40)}", flush=True)
