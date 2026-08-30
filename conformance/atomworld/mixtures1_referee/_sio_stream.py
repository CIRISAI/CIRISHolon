"""SiO's real cost, measured without ever allocating its matrix.

The other pairs were measured by building the operator and timing a matvec.  SiO
cannot be measured that way here: 1.97e8 nonzeros is about 3.2 GB of COO plus
2.4 GB of CSR, on a 31 GB machine shared with four other campaigns, and a
measurement that costs a sibling lane its pool is not a measurement worth
having.

So it is measured by STREAMING.  `route_a_elements` is a generator; walking it
and doing the same multiply-add the working-precision matvec does, without
storing anything, gives the exact nonzero count and the exact per-element rate
at flat memory.  Those two numbers are all a feasibility call needs, because the
working-precision matvec IS that walk -- `hp_cache` is off above 3000
determinants, so nothing is stored there either.
"""
import os
import resource
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "elements1"))

from mpmath import mp, mpf                                    # noqa: E402
import basis2                                                 # noqa: E402
import elements_core as EC                                    # noqa: E402
import fci as F                                               # noqa: E402

mp.dps = 60


def stream(Z1, Z2, R, label, report_every=2_000_000):
    t0 = time.time()
    atoms = [(Z1, (mpf(0), mpf(0), mpf(0))),
             (Z2, (mpf(0), mpf(0), mpf(R)))]
    mol = EC.molecule(atoms, table=basis2.STO3G_18)
    C, _ = F.lowdin_orbitals(mol["S"])
    h, g = F.mo_integrals(mol, C)
    ne, norb = mol["nelec"], mol["nbf"]
    na = (ne + ne % 2) // 2
    sp = F.DetSpace(norb, na, ne - na)
    print("%s: nbf=%d nelec=%d ndet=%s   integrals+MO %.0fs"
          % (label, norb, ne, format(sp.ndet, ","), time.time() - t0),
          flush=True)
    c = [mpf(1)] * sp.ndet          # the vector the matvec would multiply
    acc = mpf(0)
    n = 0
    t1 = time.time()
    for (i, j, v) in F.route_a_elements(sp, h, g):
        acc += v * c[j]             # exactly the matvec's inner operation
        n += 1
        if n % report_every == 0:
            el = time.time() - t1
            print("   %s elements  %.0fs  %.2f us/element  rss %.2f GB"
                  % (format(n, ","), el, el / n * 1e6,
                     resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
                     / 1048576.0), flush=True)
    el = time.time() - t1
    print("%s: nnz = %s   one HP matvec = %.0f s   %.2f us/element   "
          "rss %.2f GB"
          % (label, format(n, ","), el, el / n * 1e6,
             resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1048576.0),
          flush=True)
    return n, el


if __name__ == "__main__":
    stream(14, 8, "2.90", "SiO")
