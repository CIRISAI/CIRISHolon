"""
plants.py -- the two plants of the ELEMENTS-1 freeze, run and scored.

Per M-PLANT-SECTOR each plant's carrier is asserted NONZERO in the sector the
plant acts on BEFORE the plant is scored; a plant on an empty sector VOIDs
rather than passing.  Both assertions below are measurements, not comments:

  (i)  the Z-mutation acts on the sector "nuclear charge as it enters the
       Hamiltonian".  The carrier assertion is that changing Z by one actually
       moves the nuclear-attraction matrix -- max |dV| over the AO block, which
       must be nonzero before the energy shift means anything.  The mutation is
       applied to the CHARGE ONLY, with the basis held fixed, so what it proves
       is that the pipeline reads Z as a charge and not merely as a lookup key
       into the basis table (that key is what plant (ii) protects).

  (ii) the basis-mutation acts on the sector "the declared contraction".  The
       carrier assertion is that the perturbed coefficient belongs to an element
       PRESENT in the test species and that perturbing it actually moves the AO
       overlap matrix.  Perturbing, say, a nitrogen 2p coefficient and testing on
       H2 would be a plant on an empty sector, and would VOID.

A missed plant VOIDs.
"""

import copy
import json
import os
import sys

from mpmath import mp, mpf, nstr

import elements_core as E
import fci as F
import runner as R

HERE = os.path.dirname(os.path.abspath(__file__))
R2_TOL_STR = "1e-10"             # the referee gate the plants must fire


def r2_tol():
    """Materialised at the working precision, not at import (see curve._D1_W)."""
    return mpf(R2_TOL_STR)


def _energy(atoms, table=None, na=None, nb=None):
    mol = E.molecule(atoms, table=table)
    C, _ = F.lowdin_orbitals(mol["S"])
    h, g = F.mo_integrals(mol, C)
    nel = mol["nelec"]
    if na is None:
        na, nb = (nel + nel % 2) // 2, nel // 2
    sp = F.DetSpace(mol["nbf"], na, nb)
    r = F.solve_certified(F.RouteAOp(sp, h, g), tol_digits=mp.dps - 8,
                          max_outer=9)
    return r["energy"] + mol["E_nuc"], mol, r


def _nuclear_block(atoms, Zs, table=None):
    """The AO nuclear-attraction matrix with charges overridden."""
    shells, labels = E.build_basis(atoms, table)
    nuclei = [(tuple(mpf(x) for x in c), Zs[i])
              for i, (Z, c) in enumerate(atoms)]
    S, T, V, _ = E.ao_integrals(shells, nuclei, want_eri=False)
    return S, V


def plant_z(verbose=True):
    """(i) the Z-mutation: charge off by one, basis held fixed."""
    mp.dps = R.DPS
    R2_TOL = r2_tol()
    out = []
    cases = [("N atom", [(7, (0, 0, 0))], 7, 8, None, None),
             ("N2 at 2.1", [(7, (0, 0, 0)), (7, (0, 0, mpf("2.1")))],
              7, 8, 7, 7)]
    for (nm, atoms, Z0, Z1, na, nb) in cases:
        atoms = [(Z, tuple(mpf(x) for x in c)) for Z, c in atoms]
        # ---- carrier assertion: does one unit of Z move the operator at all?
        Zs0 = [Z for Z, _ in atoms]
        Zs1 = list(Zs0)
        Zs1[0] = Z1
        S, V0 = _nuclear_block(atoms, Zs0)
        _, V1 = _nuclear_block(atoms, Zs1)
        dV = max(abs(V0[i][j] - V1[i][j]) for i in range(len(V0))
                 for j in range(len(V0)))
        carrier_ok = dV > mpf("1e-30")

        # ---- baseline and mutated energies
        E0, mol0, _ = _energy(atoms, na=na, nb=nb)

        mutated = [(Z1 if i == 0 else Z, c) for i, (Z, c) in enumerate(atoms)]
        # charge-only: keep the ORIGINAL basis for atom 0
        table = dict(E.STO3G_SHELLS)
        table[Z1] = E.STO3G_SHELLS[Z0]
        E1, mol1, _ = _energy(mutated, table=table, na=na, nb=nb)

        shift = abs(E1 - E0)
        rec = dict(case=nm, Z_from=Z0, Z_to=Z1,
                   carrier_max_abs_dV=nstr(dV, 8), carrier_nonzero=carrier_ok,
                   E_baseline=R.s(E0), E_mutated=R.s(E1),
                   shift_hartree=nstr(shift, 12),
                   referee_tolerance=nstr(R2_TOL, 3),
                   orders_of_magnitude_above_tolerance=(
                       float(mp.log(shift / R2_TOL) / mp.log(10))
                       if shift > 0 else None),
                   fired=bool(shift > R2_TOL * 1000),
                   verdict=("VOID (empty sector)" if not carrier_ok
                            else "FIRED" if shift > R2_TOL * 1000 else "MISSED"))
        out.append(rec)
        if verbose:
            print("  (i)  %-10s Z %d->%d  carrier |dV| = %s  shift = %s Ha  "
                  "(%.1f orders above %s)  %s"
                  % (nm, Z0, Z1, nstr(dV, 5), nstr(shift, 8),
                     rec["orders_of_magnitude_above_tolerance"] or 0,
                     nstr(R2_TOL, 2), rec["verdict"]))
    return out


def plant_basis(rel="1e-6", verbose=True, cases="all"):
    """(ii) the basis-mutation: one contraction coefficient perturbed at 1e-6."""
    mp.dps = R.DPS
    R2_TOL = r2_tol()
    rel = mpf(rel)
    out = []
    want = cases
    # (species atoms, na, nb, which element, which shell, which primitive)
    cases = [
        ("H2 at 1.4", [(1, (0, 0, 0)), (1, (0, 0, mpf("1.4")))], 1, 1,
         1, 0, 1, "hydrogen 1s, middle primitive"),
        ("N2 at 2.1", [(7, (0, 0, 0)), (7, (0, 0, mpf("2.1")))], 7, 7,
         7, 2, 0, "nitrogen 2p, first primitive"),
        ("EMPTY-SECTOR CONTROL: H2, nitrogen coefficient",
         [(1, (0, 0, 0)), (1, (0, 0, mpf("1.4")))], 1, 1,
         7, 2, 0, "nitrogen 2p perturbed, but no nitrogen in H2"),
    ]
    if want == "cheap":
        cases = [c for c in cases if "N2" not in c[0]]
    for (nm, atoms, na, nb, Zmut, ish, ip, what) in cases:
        atoms = [(Z, tuple(mpf(x) for x in c)) for Z, c in atoms]
        table = {k: tuple(tuple(x) for x in v)
                 for k, v in E.STO3G_SHELLS.items()}
        l, exps, coefs = table[Zmut][ish]
        new = list(coefs)
        new[ip] = nstr(mpf(coefs[ip]) * (1 + rel), 20, strip_zeros=False)
        shells = list(table[Zmut])
        shells[ish] = (l, exps, tuple(new))
        table[Zmut] = tuple(shells)

        # ---- carrier assertion: does the perturbation reach the integrals?
        S0, _ = _nuclear_block(atoms, [Z for Z, _ in atoms])
        S1, _ = _nuclear_block(atoms, [Z for Z, _ in atoms], table=table)
        dS = max(abs(S0[i][j] - S1[i][j]) for i in range(len(S0))
                 for j in range(len(S0)))
        present = Zmut in [Z for Z, _ in atoms]
        carrier_ok = bool(present and dS > mpf("1e-30"))

        E0, _, _ = _energy(atoms, na=na, nb=nb)
        E1, _, _ = _energy(atoms, table=table, na=na, nb=nb)
        shift = abs(E1 - E0)
        rec = dict(case=nm, perturbed=what, relative_perturbation=nstr(rel, 3),
                   element_present=present, carrier_max_abs_dS=nstr(dS, 8),
                   carrier_nonzero=carrier_ok,
                   E_baseline=R.s(E0), E_mutated=R.s(E1),
                   shift_hartree=nstr(shift, 12),
                   referee_tolerance=nstr(R2_TOL, 3),
                   fired=bool(shift > R2_TOL),
                   verdict=("VOID (empty sector, as designed)"
                            if not carrier_ok
                            else "FIRED" if shift > R2_TOL else "MISSED"))
        out.append(rec)
        if verbose:
            print("  (ii) %-46s |dS| = %-12s shift = %s Ha  %s"
                  % (nm, nstr(dS, 4), nstr(shift, 8), rec["verdict"]))
    return out


def main():
    print("PLANTS -- ELEMENTS-1")
    print("\nplant (i)  the Z-mutation")
    z = plant_z()
    print("\nplant (ii) the basis-mutation")
    b = plant_basis()
    res = dict(basis_fingerprint=E.basis_fingerprint(),
               plant_i_Z_mutation=z, plant_ii_basis_mutation=b)
    live_i = [r for r in z if r["carrier_nonzero"]]
    live_ii = [r for r in b if r["carrier_nonzero"]]
    ok = (live_i and all(r["fired"] for r in live_i)
          and live_ii and all(r["fired"] for r in live_ii))
    res["all_live_plants_fired"] = bool(ok)
    with open(os.path.join(HERE, "elements_plants.json"), "w") as f:
        json.dump(res, f, indent=1)
    print("\nall live plants fired: %s   (wrote elements_plants.json)" % ok)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
