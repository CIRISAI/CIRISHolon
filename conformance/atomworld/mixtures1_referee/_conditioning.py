"""Can the referee compute on the grid the rule produced?

A grid's inner edge is set by the RULE, which knows only R_ref.  Whether the
basis is still usable there is a separate question and it is not a question
about energies: at small separation a minimal basis on two heavy centres goes
linearly dependent, the overlap matrix's smallest eigenvalue collapses, and the
Lowdin orthogonalisation that every route runs through amplifies whatever is
left.  ELEMENTS-1's heaviest negative, Ne2, starts at 1.80 bohr with Z = 10; the
rule applied to Ar2 asks for 1.42 bohr with Z = 18, which is a different regime
and has to be measured rather than assumed.

THIS IS A RESULT-BLIND CHECK.  It reads the overlap matrix -- basis and geometry
only.  No Hamiltonian, no electron count, no energy.  Raising an inner edge
because the basis is singular there is a statement about the BASIS; it would
only stop being result-blind if the edge moved because of what an energy did.
"""
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "elements1"))

from mpmath import mp, mpf, nstr                              # noqa: E402
import basis2                                                 # noqa: E402
import species2 as SP2                                        # noqa: E402
import elements_core as EC                                    # noqa: E402
import fci as F                                               # noqa: E402
import curve                                                  # noqa: E402

# DECLARED IN ADVANCE: the conditioning floor.  A grid point is admissible when
# the overlap matrix's smallest eigenvalue is at least this.  1e-6 is three
# orders above the 1e-9 at which a 50-digit Lowdin root starts losing digits it
# cannot get back, and it is BELOW every eigenvalue ELEMENTS-1 actually ran on
# (its worst, Ne2 at 1.80 bohr, is reported by this script for comparison).
SMIN_FLOOR = mpf("1e-6")


def smin(Z1, Z2, R, table):
    mp.dps = 60
    atoms = [(Z1, (mpf(0), mpf(0), mpf(0))),
             (Z2, (mpf(0), mpf(0), mpf(R)))]
    mol = EC.molecule(atoms, table=table, screen=None)
    _, sev = F.lowdin_orbitals(mol["S"])
    return min(sev), mol["nbf"]


def scan(name, Z1, Z2, r0, table, step="0.02"):
    """The smallest R on a 2-decimal grid at which the floor still holds."""
    r = mpf(r0)
    seen = []
    for _ in range(80):
        rs = curve.dec_str(r, 2)
        t0 = time.time()
        s, nbf = smin(Z1, Z2, rs, table)
        seen.append((rs, s))
        print("    %-6s R=%-7s  s_min = %-14s  (%.0fs)"
              % (name, rs, nstr(s, 6), time.time() - t0), flush=True)
        if s >= SMIN_FLOOR:
            return rs, s, seen
        r = r + mpf(step)
    return None, None, seen


if __name__ == "__main__":
    tab = basis2.STO3G_18
    print("conditioning floor: s_min >= %s\n" % nstr(SMIN_FLOOR, 3))
    print("ELEMENTS-1's inner edges, for comparison (its own table):")
    for nm, Z1, Z2, r in (("Ne2", 10, 10, "1.80"), ("F2", 9, 9, "1.60"),
                          ("N2", 7, 7, "1.40"), ("Li2", 3, 3, "2.60")):
        s, nbf = smin(Z1, Z2, r, EC.STO3G_SHELLS)
        print("    %-5s R=%-6s nbf=%-3d s_min = %s"
              % (nm, r, nbf, nstr(s, 6)), flush=True)
    print("\nMIXTURES-1, at the inner edge the rule produced:")
    out = {}
    for nm in SP2.STAKED:
        g = SP2.grid_spec(nm)
        s, nbf = smin(g["Z1"], g["Z2"], g["rmin"], tab)
        ok = s >= SMIN_FLOOR
        print("    %-5s R=%-7s nbf=%-3d s_min = %-14s %s"
              % (nm, g["rmin"], nbf, nstr(s, 6), "ok" if ok else "BELOW FLOOR"),
              flush=True)
        out[nm] = (g["rmin"], s, ok)
    bad = [nm for nm, v in out.items() if not v[2]]
    if bad:
        print("\nraising the inner edge for %s, by the declared rule:"
              % ", ".join(bad))
        for nm in bad:
            g = SP2.grid_spec(nm)
            r, s, _ = scan(nm, g["Z1"], g["Z2"], g["rmin"], tab)
            print("    %-5s admissible inner edge: %s (s_min %s)"
                  % (nm, r, nstr(s, 6) if s else "none found"), flush=True)
    else:
        print("\nevery staked inner edge is admissible as the rule placed it")
