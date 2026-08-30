"""Gate R1: every atom H through Ar, dual route, 50 digits, ground spin DERIVED.

WHY THE FIRST ROW IS RECOMPUTED HERE RATHER THAN IMPORTED.

ELEMENTS-1 already published H..Ne at 50 digits.  Those numbers are not reused,
for a reason worth stating: they were computed under a DIFFERENT declared table
(Z = 1..10), and this campaign's table is Z = 1..18.  A basis fingerprint that
differs is a refusal, and honouring it means recomputing rather than deciding by
hand that the difference does not matter for an isolated hydrogen atom.

But it obviously SHOULDN'T matter -- adding argon to a table cannot change a
hydrogen atom -- and that turns an inconvenience into a free and rather strong
check.  If the ten first-row energies come back bit-identical to ELEMENTS-1's,
then the table extension provably touched nothing it should not have: not the
integral code paths the new shells exercise, not the shell-pair loop, not the
normalisation.  If any one of them moved, something in the extension reached
back into the first row and the second row's numbers cannot be trusted either.
That comparison is made below and its result is part of the artifact.

GROUND SPIN IS DERIVED TWICE, from two readings that share only the solver:
once from which Sz sectors are degenerate (a spin-S multiplet appears in every
sector with |Sz| <= S and none above), and once from <S^2> of the converged
vector in the sector that attains the minimum.  H commutes with S^2, so a
subspace method can converge cleanly, with a small residual and a tight bound,
onto a spin-EXCITED state, and nothing it reports about itself will show that.
The two derivations disagreeing is the only thing that would.
"""
import json
import os
import sys
import time
from multiprocessing import Pool

from mpmath import mp, mpf

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(HERE, "elements1"))

import basis2                                                  # noqa: E402
import m1core as M                                             # noqa: E402
import species2 as SP2                                         # noqa: E402
import species as SP1                                          # noqa: E402

NPROC = int(os.environ.get("NPROC", "6"))
ATOMS = list(range(1, 19))
E1_ATOMS = os.path.join(HERE, "elements1", "elements_atoms.json")


def work(Z):
    mp.dps = M.DPS
    return Z, M.run_point(("atom", Z), M.atom_tag(Z), want_C=True, want_B=True)


def main():
    t0 = time.time()
    only = [int(a) for a in sys.argv[1:] if a.isdigit()]
    todo = only or ATOMS
    out = dict(model="MIXTURES1/STO-3G/FCI",
               basis_fingerprint=M.FINGERPRINT,
               basis="STO-3G, Z = 1..18, s and p shells only; see basis2.py",
               working_precision_dps=M.DPS, precision_digits=M.REPORT,
               units=dict(E="hartree"),
               note=("Exact-in-model full CI in the declared STO-3G minimal "
                     "basis.  NOT a prediction of experiment.  Ground total "
                     "spin is derived from the Sz degeneracy pattern AND from "
                     "<S^2>, and the two derivations are reported separately."),
               atoms={})
    with Pool(min(NPROC, len(todo))) as p:
        results = dict(p.map(work, todo))

    for Z in todo:
        res = results[Z]
        mp.dps = M.DPS
        en = {int(k): mpf(v["E_A"]) for k, v in res["sectors"].items()}
        two_s, emin, hits = SP1.ground_spin_from_sectors(en, mpf("1e-40"))
        low = min(hits)
        sec = res["sectors"][str(low)]
        two_s_s2 = sec.get("two_S_from_S2")
        agree = (two_s_s2 is not None and two_s_s2 == two_s)
        worst = max(float(v["dev_AB"]) for v in res["sectors"].values())
        sym = SP2.SYMBOL[Z]
        out["atoms"][sym] = dict(
            Z=Z, symbol=sym, nbf=res["nbf"], nelec=res["nelec"],
            mass_u=(basis2.ISOTOPE_MASS_U.get(Z)
                    or str(__import__("elements_core").ISOTOPE_MASS_U[Z])),
            two_S_from_degeneracy=two_s, two_S_from_S2_expectation=two_s_s2,
            spin_derivations_agree=bool(agree),
            S2_expectation=sec.get("S2_expectation"),
            S2_deviation_from_exact=sec.get("S2_deviation_from_exact"),
            ground_S=("%d/2" % two_s) if two_s % 2 else str(two_s // 2),
            multiplicity=two_s + 1, degenerate_sectors=hits,
            E=M.R.s(emin), max_dev_AB=worst,
            E_C=res.get("E_C"), dev_AC=res.get("dev_AC"),
            route_C_fock_dim=res.get("route_C_fock_dim"),
            sectors=res["sectors"], t_total=res["t_total"])
        print("%-3s Z=%2d nbf=%-2d 2S=%d (mult %d)  E = %s  |A-B|<=%.1e  "
              "spin routes %s"
              % (sym, Z, res["nbf"], two_s, two_s + 1, M.R.s(emin, 32), worst,
                 "AGREE" if agree else "DISAGREE (%s vs %s)"
                 % (two_s, two_s_s2)), flush=True)

    # ---- the free check: did extending the table move the first row? -------
    if os.path.exists(E1_ATOMS) and not only:
        e1 = json.load(open(E1_ATOMS))["atoms"]
        moved, same, absent = [], [], []
        for sym, rec in sorted(out["atoms"].items()):
            if rec["Z"] > 10:
                continue
            if sym not in e1:
                absent.append(sym)
                continue
            if e1[sym]["E"] == rec["E"]:
                same.append(sym)
            else:
                moved.append((sym, e1[sym]["E"], rec["E"]))
        out["first_row_unchanged_by_table_extension"] = dict(
            identical=same, moved=[m[0] for m in moved], absent=absent,
            meaning=("Every first-row energy recomputed under the Z = 1..18 "
                     "table, compared to ELEMENTS-1's published value at the "
                     "same 50 digits.  Adding the second row cannot change an "
                     "isolated first-row atom, so anything in `moved` means "
                     "the extension reached back into code the first row uses "
                     "and the second row's numbers are not trustworthy "
                     "either."))
        print("\nfirst row under the extended table: %d of %d bit-identical "
              "to ELEMENTS-1%s"
              % (len(same), len(same) + len(moved) + len(absent),
                 "" if not moved else "  !! MOVED: %s" % [m[0] for m in moved]))
        for sym, a, b in moved:
            print("   %-3s ELEMENTS-1 %s\n       MIXTURES-1 %s" % (sym, a, b))

    out["elapsed_s"] = time.time() - t0
    p = os.path.join(HERE, "mixtures_atoms.json")
    if only and os.path.exists(p):
        prev = json.load(open(p))
        prev["atoms"].update(out["atoms"])
        prev["elapsed_s"] = prev.get("elapsed_s", 0) + out["elapsed_s"]
        out = prev
    with open(p, "w") as f:
        json.dump(out, f, indent=1)
    print("\nwrote mixtures_atoms.json (%d atoms) in %.1fs"
          % (len(out["atoms"]), time.time() - t0))


if __name__ == "__main__":
    main()
