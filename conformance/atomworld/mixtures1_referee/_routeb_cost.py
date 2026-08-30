"""Route A enumerates matrix elements; route B contracts over strings.  Which
one actually costs less at the working precision?

The feasibility of SiO was estimated from route A's element count, because that
is the number the campaign's notes are written in.  But `fci.py` carries a
SECOND formulation -- the spin-summed generator (unitary-group) route -- whose
sigma is built from <K|E_pq|J> string couplings rather than from an enumeration
of nonzeros.  If its working-precision matvec scales differently, the whole
feasibility question was asked about the wrong route.

Both are measured on the same operator at the same precision.  Nothing here is
an argument about complexity; it is two timings.
"""
import os, sys, time
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE); sys.path.append(os.path.join(HERE, "elements1"))
from mpmath import mp, mpf
import basis2, elements_core as EC, fci as F
mp.dps = 60

CASES = [("HCl", 1, 17, "2.40"), ("Cl2", 17, 17, "3.80"), ("N2", 7, 7, "2.10")]
if len(sys.argv) > 1:
    CASES = [c for c in CASES if c[0] in sys.argv[1:]]

for lab, Z1, Z2, R in CASES:
    t = time.time()
    atoms = [(Z1, (mpf(0), mpf(0), mpf(0))), (Z2, (mpf(0), mpf(0), mpf(R)))]
    mol = EC.molecule(atoms, table=basis2.STO3G_18)
    C, _ = F.lowdin_orbitals(mol["S"])
    hA, gA = F.mo_integrals(mol, C)
    Q = F.rotation_matrix(mol["nbf"])
    hB, gB = F.mo_integrals(mol, F.rotate_orbitals(C, Q))
    ne, norb = mol["nelec"], mol["nbf"]
    na = (ne + ne % 2) // 2
    sp = F.DetSpace(norb, na, ne - na)
    print("%s nbf=%d ndet=%s  integrals %.0fs" % (lab, norb, format(sp.ndet, ","),
          time.time() - t), flush=True)
    t = time.time(); opA = F.RouteAOp(sp, hA, gA); tbA = time.time() - t
    t = time.time(); opB = F.RouteBOp(sp, hB, gB); tbB = time.time() - t
    v = [mpf(1)] * sp.ndet
    t = time.time(); opA.matvec_hp(v); tA = time.time() - t
    t = time.time(); opB.matvec_hp(v); tB = time.time() - t
    print("  build A %7.1fs  B %7.1fs      matvec_hp A %8.2fs  B %8.2fs   "
          "B/A = %.2f" % (tbA, tbB, tA, tB, tB / tA if tA else 0), flush=True)
