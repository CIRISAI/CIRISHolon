"""Measure, do not estimate, what one geometry of each staked pair costs.

The determinant COUNT is not the cost; the cost is the number of nonzero
Hamiltonian elements the working-precision matvec walks, and how fast it walks
them.  The prereg's feasibility map is stated in determinants, which for the
mixed second row understates the spread by an order of magnitude: SiO has 9x
N2's determinants and 22x its nonzeros.

This builds the operator at ONE geometry per pair and times one f64 matvec and
one working-precision matvec.  From those two numbers and the outer-iteration
count a real solve uses, the per-geometry cost follows without running one.
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
Z = {"H": 1, "C": 6, "N": 7, "O": 8, "F": 9, "Ne": 10,
     "Na": 11, "Si": 14, "S": 16, "Cl": 17, "Ar": 18}
R0 = {"N2": "2.10", "S2": "3.60", "NaH": "3.60", "SiO": "2.90",
      "Cl2": "3.80", "HCl": "2.40", "ClF": "3.10", "Ar2": "7.00",
      "NeAr": "6.50"}
PAIRS = {"N2": ("N", "N"), "Cl2": ("Cl", "Cl"), "HCl": ("H", "Cl"),
         "ClF": ("Cl", "F"), "Ar2": ("Ar", "Ar"), "NeAr": ("Ne", "Ar"),
         "S2": ("S", "S"), "NaH": ("Na", "H"), "SiO": ("Si", "O")}


def rss_gb():
    return resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1048576.0


def probe(name):
    a, b = PAIRS[name]
    tab = basis2.STO3G_18
    atoms = [(Z[a], (mpf(0), mpf(0), mpf(0))),
             (Z[b], (mpf(0), mpf(0), mpf(R0[name])))]
    t0 = time.time()
    mol = EC.molecule(atoms, table=tab)
    C, _ = F.lowdin_orbitals(mol["S"])
    h, g = F.mo_integrals(mol, C)
    t_int = time.time() - t0
    ne, norb = mol["nelec"], mol["nbf"]
    na = (ne + ne % 2) // 2
    nb = ne - na
    sp = F.DetSpace(norb, na, nb)
    t0 = time.time()
    op = F.RouteAOp(sp, h, g)
    t_build = time.time() - t0
    nnz = op.csr.nnz
    import numpy as np
    v = np.ones(sp.ndet) / (sp.ndet ** 0.5)
    t0 = time.time()
    for _ in range(3):
        op.matvec_f64(v)
    t_f64 = (time.time() - t0) / 3
    vh = [mpf(1) / mp.sqrt(sp.ndet)] * sp.ndet
    t0 = time.time()
    op.matvec_hp(vh)
    t_hp = time.time() - t0
    print("%-5s nbf=%-3d ne=%-3d ndet=%-9s nnz=%-12s  int %6.1fs  build %7.1fs"
          "  f64 %8.4fs  HP %9.2fs  hp_cache=%s  rss %.2f GB"
          % (name, norb, ne, format(sp.ndet, ","), format(nnz, ","),
             t_int, t_build, t_f64, t_hp,
             "yes" if op.hp_cache is not None else "NO", rss_gb()),
          flush=True)
    return dict(name=name, ndet=sp.ndet, nnz=nnz, t_hp=t_hp, t_f64=t_f64,
                t_build=t_build, t_int=t_int)


if __name__ == "__main__":
    want = sys.argv[1:] or ["Ar2", "NeAr", "HCl", "ClF", "Cl2", "N2", "S2",
                            "NaH", "SiO"]
    out = []
    for nm in want:
        try:
            out.append(probe(nm))
        except MemoryError:
            print("%-5s MemoryError" % nm, flush=True)
        except Exception as e:
            print("%-5s %s: %s" % (nm, type(e).__name__, e), flush=True)
    print()
    ref = next((r for r in out if r["name"] == "N2"), None)
    if ref:
        print("scaled to N2, whose real grid points measured 2354-10893 s "
              "(route A + route B, 9 outer iterations):")
        for r in out:
            k = r["t_hp"] / ref["t_hp"]
            print("  %-5s HP matvec %8.2fx N2  ->  a grid point costs about "
                  "%s" % (r["name"], k,
                          ("%.0f s" % (k * 4700)) if k * 4700 < 86400
                          else "%.1f DAYS" % (k * 4700 / 86400)))
