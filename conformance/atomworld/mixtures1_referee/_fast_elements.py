"""Is SiO's cost reducible?  A hoisted element generator, and the proof that it
yields the SAME STREAM.

WHY THIS EXISTS.  SiO has 1.97e8 nonzero Hamiltonian elements, 22x N2's, and the
working-precision matvec walks every one of them at a measured ~12 us each.  That
is about 40 minutes per matvec and tens of hours per geometry, which would make
gate D1's SiO stake a multi-week job.  Reporting "infeasible" without testing the
obvious remedy would be a weak finding, so the remedy is tested.

WHAT IS ACTUALLY REDUNDANT in `route_a_elements`.  The generator's outer loop is
over alpha strings and its inner loop over beta strings, and it correctly hoists
the ALPHA excitation structure out of the beta loop.  It does not hoist anything
that depends on the beta string alone, and there are four such things:

  1. `same = [j for j in oa if j != m]` is rebuilt for every (alpha, beta) pair
     though it depends on the alpha string only;
  2. the alpha-double VALUE `g[m][p][n][q] - g[m][q][n][p]` is recomputed for
     every beta string though it contains no beta index -- nbs times too often;
  3. the whole beta single/double enumeration, `_excite` calls included, is
     redone for every ALPHA string -- nas times too often.  The beta-double
     value is entirely alpha-independent, so it is recomputed nas times as well;
  4. worst, the mixed alpha/beta block calls `_excite(sb, n, q)` once per
     (alpha string, beta string, alpha single, beta single) -- that is one
     `_excite` per NONZERO, where |asing| x |bsing| of them share the same
     answer.  This is the dominant block: for SiO it is 1089 of the 1486
     couplings per determinant, about 73% of the work.

WHAT IS NOT CHANGED.  Not one arithmetic expression.  Every value is computed by
the same expression with the same association order -- `sgn * (g[a] - g[b])`
stays `sgn * (g[a] - g[b])`, evaluated once instead of nas times -- and every
element is yielded in the same ORDER.  That is what makes the equivalence
checkable rather than arguable, and it is checked below by comparing the two
streams element for element, including the exact mpf values, on real operators.

WHAT THIS FILE IS NOT.  It is not applied to `fci.py`.  That file is producing
every referee number in two live campaigns, and an optimisation to it is the
lead's call, not this lane's.  What is here is the patch and the measurement to
decide with.
"""
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.append(os.path.join(HERE, "elements1"))

from mpmath import mp, mpf                                     # noqa: E402
import basis2                                                  # noqa: E402
import elements_core as EC                                     # noqa: E402
import fci as F                                                # noqa: E402


def route_a_elements_hoisted(space, h, g):
    """Same stream, same order, same values, without the redundant work."""
    nbs = space.nbs
    aidx, bidx = space.aidx, space.bidx

    # ---- beta structure, built ONCE instead of once per alpha string --------
    # `bevents[ib]` replays the original's interleaving exactly: for each beta
    # single, the single itself and then every double built on top of it.
    bevents = []
    bmix = []
    for ib in range(nbs):
        ob, vb = space.bocc[ib], space.bvir[ib]
        sb = space.bstr[ib]
        ev = []
        mx = []
        for m in ob:
            for p in vb:
                r1 = F._excite(sb, m, p)
                s1, sgn1 = r1[1], r1[0]
                jb = bidx[s1]
                same = [j for j in ob if j != m]
                ev.append((0, jb, m, p, sgn1, same))
                for n in ob:
                    if n <= m:
                        continue
                    for q in vb:
                        if q <= p:
                            continue
                        r2 = F._excite(s1, n, q)
                        if r2 is None:
                            continue
                        # alpha-independent: computed once, not nas times
                        ev.append((1, bidx[r2[1]],
                                   sgn1 * r2[0] * (g[m][p][n][q]
                                                   - g[m][q][n][p])))
        # the mixed block enumerates the same beta singles in the same order
        for n in ob:
            for q in vb:
                r = F._excite(sb, n, q)
                mx.append((n, q, bidx[r[1]], r[0]))
        bevents.append(ev)
        bmix.append(mx)

    for ia in range(space.nas):
        oa, va = space.aocc[ia], space.avir[ia]
        sa = space.astr[ia]
        asing = []
        adoub = []
        for m in oa:
            same_a = [j for j in oa if j != m]        # hoisted out of the ib loop
            for p in va:
                r1 = F._excite(sa, m, p)
                s1, sgn1 = r1[1], r1[0]
                asing.append((aidx[s1], m, p, sgn1, same_a))
                for n in oa:
                    if n <= m:
                        continue
                    for q in va:
                        if q <= p:
                            continue
                        r2 = F._excite(s1, n, q)
                        if r2 is None:
                            continue
                        # beta-independent: computed once, not nbs times
                        adoub.append((aidx[r2[1]],
                                      sgn1 * r2[0] * (g[m][p][n][q]
                                                      - g[m][q][n][p])))
        base_a = ia * nbs
        for ib in range(nbs):
            ob = space.bocc[ib]
            I = base_a + ib
            yield (I, I, F._sc_diag(oa, ob, h, g))
            for (ja, m, p, sgn, same_a) in asing:
                yield (ja * nbs + ib, I,
                       sgn * F._sc_single(m, p, same_a, ob, h, g))
            for (ja, val) in adoub:
                yield (ja * nbs + ib, I, val)
            for ev in bevents[ib]:
                if ev[0] == 0:
                    _, jb, m, p, sgn1, same_b = ev
                    yield (base_a + jb, I,
                           sgn1 * F._sc_single(m, p, same_b, oa, h, g))
                else:
                    yield (base_a + ev[1], I, ev[2])
            mx = bmix[ib]
            for (ja, m, p, sgna, _sa) in asing:
                jbase = ja * nbs
                for (n, q, jb, sgnb) in mx:
                    yield (jbase + jb, I, sgna * sgnb * g[m][p][n][q])


# ---------------------------------------------------------------------------
def build(Z1, Z2, R, table=None):
    mp.dps = 60
    atoms = [(Z1, (mpf(0), mpf(0), mpf(0))),
             (Z2, (mpf(0), mpf(0), mpf(R)))]
    mol = EC.molecule(atoms, table=table or basis2.STO3G_18)
    C, _ = F.lowdin_orbitals(mol["S"])
    h, g = F.mo_integrals(mol, C)
    ne, norb = mol["nelec"], mol["nbf"]
    na = (ne + ne % 2) // 2
    return F.DetSpace(norb, na, ne - na), h, g


def compare(label, Z1, Z2, R):
    """Element for element, value for value, in order."""
    sp, h, g = build(Z1, Z2, R)
    t0 = time.time()
    ref = list(F.route_a_elements(sp, h, g))
    t_ref = time.time() - t0
    t0 = time.time()
    fast = list(route_a_elements_hoisted(sp, h, g))
    t_fast = time.time() - t0
    ok = len(ref) == len(fast)
    first_bad = None
    if ok:
        for k in range(len(ref)):
            a, b = ref[k], fast[k]
            if a[0] != b[0] or a[1] != b[1] or a[2] != b[2]:
                ok = False
                first_bad = (k, a, b)
                break
    print("%-6s ndet=%-8s nnz=%-12s  ref %7.2fs  hoisted %7.2fs  "
          "speedup %5.2fx   %s"
          % (label, format(sp.ndet, ","), format(len(ref), ","),
             t_ref, t_fast, (t_ref / t_fast) if t_fast else 0,
             "IDENTICAL STREAM" if ok else "MISMATCH at %r" % (first_bad,)),
          flush=True)
    return ok, t_ref, t_fast, len(ref)


if __name__ == "__main__":
    cases = [
        ("HCl", 1, 17, "2.40"),
        ("ClF", 17, 9, "3.10"),
        ("Cl2", 17, 17, "3.80"),
        ("N2", 7, 7, "2.10"),
    ]
    want = sys.argv[1:]
    allok = True
    for (lab, Z1, Z2, R) in cases:
        if want and lab not in want:
            continue
        ok, _, _, _ = compare(lab, Z1, Z2, R)
        allok = allok and ok
    print("\n%s" % ("every stream identical" if allok
                    else "A STREAM DIFFERED -- the hoisting is not equivalent"))
    raise SystemExit(0 if allok else 1)
