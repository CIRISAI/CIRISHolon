"""The MIXTURES-1 standalone re-check.

    python3 verify2.py [--quick]

Most of the sections are ELEMENTS-1's, called on this campaign's artifact: dual
route agreement, the certificates, the emergent negatives, the atoms, the spin
audit at every geometry, the grid regenerated from its own declared rule, and
the record's own verdicts. They are reused rather than reimplemented for the
same reason the stage machinery is: each one exists because something once got
past its absence.

WHAT IS ADDED HERE is what this campaign stakes and ELEMENTS-1 does not: gate
E2's ordering hypothesis, the D1 bridge marking, and the scope bound.

AND A WORK COUNT, ASSERTED RATHER THAN REPORTED.

The failure this guards against is a verifier that runs no checks and prints
"ALL CHECKS PASSED". It is not hypothetical -- an empty species list, a section
that returns early on a missing key, a loop over a dict that is not there, and
every one of them exits 0. So every section declares the MINIMUM number of
checks it must have run, the count is compared against it, and a section that
comes in under its floor FAILS with the shortfall named. A verifier that cannot
say how much work it did has not said anything.
"""
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_E1 = os.path.join(HERE, "elements1")
sys.path.insert(0, HERE)
if _E1 not in sys.path:
    sys.path.append(_E1)

import species as SP                                           # noqa: E402
import m1core as M                                             # noqa: E402
import species2 as SP2                                         # noqa: E402
import verify_elements as VE                                   # noqa: E402
from mpmath import mp, mpf                                     # noqa: E402

MODEL = "MIXTURES1/STO-3G/FCI"
VE.HERE = HERE                      # read THIS campaign's artifact

# ---------------------------------------------------------------------------
# A counting wrapper around the shared `check`, so a section that quietly does
# nothing is a failure rather than a silence.
# ---------------------------------------------------------------------------
COUNT = [0]
_real_check = VE.check


def counting_check(name, ok, detail=""):
    COUNT[0] += 1
    return _real_check(name, ok, detail)


VE.check = counting_check
check = counting_check


def section(label, fn, floor, *args):
    """Run one section and require it to have done at least `floor` checks."""
    before = COUNT[0]
    fn(*args)
    did = COUNT[0] - before
    if did < floor:
        _real_check("[%s] ran at least %d checks" % (label, floor), False,
                    "ran %d -- a section that does nothing still exits 0, so "
                    "the shortfall is the finding" % did)
        COUNT[0] += 1
    return did


# ---------------------------------------------------------------------------
def v_e2_ordering(pot):
    """Gate E2: the in-model well-depth ordering, in its broad strokes.

    The prereg stakes N2 > SiO > HCl > ClF > S2 > Cl2 > NaH >> (Ar2, NeAr), and
    N2 comes from the OTHER campaign's table -- so this is scored across the two
    and says which side each number came from. A gross inversion is branch (b):
    reported and investigated, never massaged.
    """
    print("\n[E2] the emergent chemical contrast (two-branch, structural)")
    depths = {}
    for name in SP2.E2_ORDER + SP2.E2_UNBOUND:
        rec = pot["species"].get(name)
        if rec is None:
            continue
        ex = rec["exact"]
        d = ex.get("D_e")
        depths[name] = mpf(d) if d not in (None, "unbound") else None
    have = [n for n in SP2.E2_ORDER if depths.get(n) is not None]
    check("at least two bound pairs have a well depth to order", len(have) >= 2,
          "have %s" % (have or "none"))
    inversions = []
    for i in range(len(have) - 1):
        a, b = have[i], have[i + 1]
        if depths[a] < depths[b]:
            inversions.append("%s (%s) shallower than %s (%s)"
                              % (a, VE.R.s(depths[a], 8), b,
                                 VE.R.s(depths[b], 8)))
    check("the staked ordering holds over the pairs present", not inversions,
          "; ".join(inversions) if inversions
          else " > ".join(have) + "  (branch (a))")
    if inversions:
        VE.NOTE.append("E2 branch (b): %d inversion(s) against the staked "
                       "ordering -- REPORTED, to be investigated, not "
                       "massaged: %s" % (len(inversions),
                                         "; ".join(inversions)))
    for name in SP2.E2_UNBOUND:
        if name in depths:
            check("%s is unbound, as E2 and E1 both stake" % name,
                  depths[name] is None,
                  "D_e = %s" % ("unbound" if depths[name] is None
                                else VE.R.s(depths[name], 8)))


def v_scope(pot):
    """The referee's model stops at argon, and the drop says so."""
    print("\n[S] scope: what this referee can and cannot grade")
    import basis2
    check("the declared table is Z = 1..18", max(basis2.STO3G_18) == 18,
          "max Z = %d" % max(basis2.STO3G_18))
    check("no declared shell has l > 1 (no d functions anywhere)",
          max(l for sh in basis2.STO3G_18.values()
              for (l, _, _) in sh) <= 1)
    fired = False
    try:
        basis2.shells_for(19)
    except KeyError:
        fired = True
    check("an out-of-scope element RAISES rather than building a smaller "
          "basis that would look converged", fired)
    zs = set()
    for name in pot["species"]:
        d = SP.DIATOMICS.get(name)
        if d:
            zs.add(d["Z1"])
            zs.add(d["Z2"])
    check("every element in the artifact is inside the declared model",
          all(z in basis2.STO3G_18 for z in zs),
          "elements used: %s" % sorted(zs))


def v_d1_bridge(dropdir):
    """S2 and SiO carry the bridge marking in their own files, or say why not."""
    print("\n[D1] the bridge's reference values are marked as such")
    n = 0
    for name in SP2.D1_BRIDGE:
        p = os.path.join(dropdir, "%s.json" % name)
        if not os.path.exists(p):
            check("%s is emitted so the bridge has a reference" % name, False,
                  "absent from the drop -- gate D1 cannot be scored against a "
                  "file that is not there")
            continue
        obj = json.load(open(p))
        b = obj.get("d1_bridge_reference")
        check("%s says in its own header that it is the D1 reference" % name,
              isinstance(b, dict) and "gate" in b,
              (b or {}).get("gate", "MISSING")[:60])
        n += 1
    if not n:
        VE.NOTE.append("gate D1 has no reference file yet; S2 and SiO are the "
                       "staked overlap species and SiO's feasibility is "
                       "measured-and-negative (see FEASIBILITY.md)")


def v_model_label(pot, atoms):
    print("\n[M] the artifact says which campaign it is")
    check("the assembled potential carries this campaign's model",
          pot.get("model") == MODEL, "%r" % pot.get("model"))
    check("the atoms record carries this campaign's model",
          atoms.get("model") == MODEL, "%r" % atoms.get("model"))
    check("the table in force is this campaign's",
          pot.get("basis_fingerprint", M.FINGERPRINT) == M.FINGERPRINT,
          M.FINGERPRINT)
    fr = atoms.get("first_row_unchanged_by_table_extension")
    check("the first row was compared against ELEMENTS-1's published values",
          isinstance(fr, dict) and fr.get("identical"),
          "%d identical, %d moved" % (len((fr or {}).get("identical") or []),
                                      len((fr or {}).get("moved") or [])))
    check("and none of them moved -- so extending the table touched nothing "
          "the first row uses", not (fr or {}).get("moved"),
          "moved: %s" % ((fr or {}).get("moved") or "none"))


# ---------------------------------------------------------------------------
def main():
    quick = "--quick" in sys.argv
    drop = os.path.join(HERE, "engine_handoff", "mixtures1")
    mp.dps = VE.R.DPS
    print("=" * 74)
    print("verify2.py -- MIXTURES-1 standalone re-check"
          + ("  [quick]" if quick else ""))
    print("=" * 74)

    src = ("elements_potential.json"
           if os.path.exists(os.path.join(HERE, "elements_potential.json"))
           else "elements_potential_partial.json")
    if not os.path.exists(os.path.join(HERE, src)):
        print("no assembled curves yet -- run curves2.py --assemble")
        return 1
    pot = VE.load(src)
    atoms = VE.load("mixtures_atoms.json")
    print("\nsource: %s   species present: %s"
          % (src, ", ".join(sorted(pot["species"])) or "none"))
    for nm, why in (pot.get("incomplete_species") or {}).items():
        print("      INCOMPLETE: %s" % why)
    if not pot["species"]:
        print("\nno species assembled: nothing to verify. Exiting 1 rather "
              "than passing vacuously.")
        return 1

    npair = len(pot["species"])
    section("M", v_model_label, 5, pot, atoms)
    section("S", v_scope, 4, pot)
    section("V1", VE.v1_dual_route, npair, pot)
    section("V3", VE.v3_certificates, npair, pot)
    missing = [nm for nm in SP2.E2_UNBOUND if nm not in pot["species"]]
    if missing:
        print("\n(gate E1 cannot be scored: %s absent)" % missing)
    else:
        section("V4", VE.v4_gate_e1, len(SP2.E2_UNBOUND), pot)
    section("V6", VE.v6_atoms, 3, atoms)
    section("V8", VE.v8_spin, npair, pot)
    if os.path.isdir(drop):
        section("V10", VE.v10_grid_provenance, npair, drop)
        section("D1", v_d1_bridge, 0, drop)
    else:
        print("\n(no engine drop at %s -- V10 and D1 not scored)"
              % os.path.relpath(drop, HERE))
    section("V11", VE.v11_record_verdicts, npair, pot, atoms)
    section("E2", v_e2_ordering, 2, pot)

    if VE.NOTE:
        print("\nNOTES")
        for n in VE.NOTE:
            print("  - " + n)
    print("\n" + "=" * 74)
    print("%d checks run over %d species" % (COUNT[0], npair))
    if COUNT[0] < 10 * npair:
        print("REFUSING TO PASS: %d checks for %d species is too few to mean "
              "anything. A verifier that cannot say how much work it did has "
              "not said anything." % (COUNT[0], npair))
        return 1
    if VE.FAIL:
        print("FAILED (%d): %s" % (len(VE.FAIL), "; ".join(VE.FAIL)))
        return 1
    print("ALL CHECKS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
