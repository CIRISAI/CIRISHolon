"""
build_atoms.py -- ground-state energies of H through Ne, every Sz sector, three
routes, with the ground total spin DERIVED from the degeneracy pattern.

Writes elements_atoms.json.
"""

import json
import os
import sys
import time
from multiprocessing import Pool

from mpmath import mp, mpf, nstr

import elements_core as EC
import species as SP
import runner as R

HERE = os.path.dirname(os.path.abspath(__file__))
NPROC = int(os.environ.get("NPROC", "10"))


def work(Z):
    mp.dps = R.DPS
    return R.run_point(("atom", Z), "atom_Z%d" % Z, want_C=True, want_B=True)


def main():
    t0 = time.time()
    out = dict(model="ELEMENTS1/STO-3G/FCI",
               working_precision_dps=R.DPS, precision_digits=R.REPORT,
               units=dict(E="hartree"),
               note=("Exact-in-model full CI in the declared STO-3G minimal "
                     "basis, from closed-form McMurchie-Davidson Gaussian "
                     "integrals in mpmath.  NOT a prediction of experiment.  "
                     "Ground total spin is derived from the Sz degeneracy "
                     "pattern, not assumed."),
               atoms={})
    with Pool(min(NPROC, len(SP.ATOMS))) as p:
        results = p.map(work, SP.ATOMS)
    for Z, res in zip(SP.ATOMS, results):
        mp.dps = R.DPS
        en = {int(k): mpf(v["E_A"]) for k, v in res["sectors"].items()}
        tol = mpf("1e-40")
        two_s, emin, hits = SP.ground_spin_from_sectors(en, tol)
        # A SECOND, independent derivation of the same multiplicity: <S^2> of
        # the converged CI vector in the sector that attains the minimum.  The
        # first reading comes from which Sz sectors are degenerate, the second
        # from the vector itself, and they use no common machinery beyond the
        # solver.  Agreement is what rules out the failure mode where a subspace
        # method converges cleanly inside the wrong spin sector, since H
        # commutes with S^2 and the residual cannot see it.
        low = min(hits)
        sec = res["sectors"][str(low)]
        two_s_s2 = sec.get("two_S_from_S2")
        s2_ok = (two_s_s2 is not None and two_s_s2 == two_s)
        rec = dict(Z=Z, symbol=EC.ELEMENT_SYMBOL[Z], nbf=res["nbf"],
                   nelec=res["nelec"],
                   mass_u=EC.ISOTOPE_MASS_U[Z],
                   ground_two_Sz=two_s,
                   two_S_from_degeneracy=two_s,
                   two_S_from_S2_expectation=two_s_s2,
                   S2_expectation=sec.get("S2_expectation"),
                   S2_deviation_from_exact=sec.get("S2_deviation_from_exact"),
                   spin_derivations_agree=bool(s2_ok),
                   ground_S=("%d/2" % two_s) if two_s % 2 else str(two_s // 2),
                   multiplicity=two_s + 1,
                   degenerate_sectors=hits,
                   E=R.s(emin),
                   sectors={k: v for k, v in res["sectors"].items()},
                   E_C=res.get("E_C"), dev_AC=res.get("dev_AC"),
                   route_C_fock_dim=res.get("route_C_fock_dim"),
                   t_total=res["t_total"])
        worst = max(float(v["dev_AB"]) for v in res["sectors"].values())
        rec["max_dev_AB"] = worst
        out["atoms"][EC.ELEMENT_SYMBOL[Z]] = rec
        print("%-3s Z=%2d  2S=%d (mult %d)  E = %s   |A-B|<=%.1e  "
              "<S^2>=%-14s 2S both routes: %s"
              % (EC.ELEMENT_SYMBOL[Z], Z, two_s, two_s + 1, R.s(emin, 32),
                 worst, sec.get("S2_expectation"),
                 "AGREE" if s2_ok else "DISAGREE (%s vs %s)"
                 % (two_s, two_s_s2)))
    out["elapsed_s"] = time.time() - t0
    with open(os.path.join(HERE, "elements_atoms.json"), "w") as f:
        json.dump(out, f, indent=1)
    print("\nwrote elements_atoms.json in %.1fs" % out["elapsed_s"])


if __name__ == "__main__":
    main()
