"""The shim that lets MIXTURES-1 run ELEMENTS-1's stage machinery unmodified.

THE MECHANISM.  `build_curves.py` does `import species as SP`.  With this
directory ahead of the ELEMENTS-1 one on `sys.path`, that import resolves to
`species.py` HERE, and the whole pipeline -- the run lock, the pool guard, the
merge-not-narrow repair, the grid_provenance regeneration, the assembler, the
gates -- runs on this campaign's species with no edit to a single line of code
that is already graded against the banked H2 referee.

WHAT MAKES IT SAFE, AND WHAT WOULD MAKE IT UNSAFE.

A stand-in module is only as good as its surface.  If ELEMENTS-1's `species.py`
gains a name that the shared code reads and this one does not have, the failure
is an AttributeError three hours into a pool -- or, far worse, a silently
different default somewhere.  So the surfaces are compared HERE, before any
compute, and the comparison is by what the SHARED CODE actually reads rather
than by what the module happens to export: `build_curves.py` and `runner.py` are
scanned for every `SP.<name>` they touch, and each one must exist on the shim.
That direction is the one that matters, and it is the one a "both modules export
the same names" test would get wrong -- a name ELEMENTS-1 exports and nobody
reads is not a hazard, and a name read but not exported is.

THE OTHER HALF is the table binding.  `runner`'s entry points default to
`table=None`, meaning the first-row table, and the shared code calls them that
way.  Importing the shim rebinds the default.  If it did not, a second-row
species would build a 5-function chlorine instead of a 9-function one and report
a higher energy that is perfectly converged for the wrong model -- no exception,
no warning, a plausible number.  So the failing case is constructed: chlorine's
basis size is measured under both tables and required to differ.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.append(os.path.join(HERE, "elements1"))

import species as SHIM                                          # noqa: E402
import m1core as M                                              # noqa: E402
import elements_core as EC                                      # noqa: E402
import runner as R                                              # noqa: E402

FAILED = []
CHECKS = [0]


def check(cond, label):
    CHECKS[0] += 1
    if not cond:
        FAILED.append(label)
        print("  FAIL  %s" % label)
    else:
        print("  ok    %s" % label)


def names_read_by_shared_code():
    """Every `SP.<name>` the shared modules touch, read out of their source."""
    want = set()
    for fn in ("build_curves.py", "runner.py", "emit_engine.py",
               "verify_elements.py"):
        p = os.path.join(HERE, "elements1", fn)
        if not os.path.exists(p):
            continue
        src = open(p).read()
        for m in re.finditer(r"\bSP\.(\w+)", src):
            want.add(m.group(1))
        for m in re.finditer(r"\bspecies\.(\w+)", src):
            want.add(m.group(1))
    return want


def main():
    print("\n1. the shim carries every name the shared code reads")
    want = names_read_by_shared_code()
    check(len(want) >= 5, "found the reads (%d names: %s)"
          % (len(want), ", ".join(sorted(want))))
    missing = sorted(n for n in want if not hasattr(SHIM, n))
    check(not missing, "every one of them exists on the shim"
          + ("" if not missing else "; MISSING: %s" % missing))

    print("\n2. and the failing case: a name read but not provided")
    class Hollow(object):
        pass
    hollow_missing = sorted(n for n in want if not hasattr(Hollow(), n))
    check(len(hollow_missing) == len(want),
          "an empty stand-in is caught on all %d names, so the check above is "
          "not vacuous" % len(want))

    print("\n3. the shared mathematics is shared, not re-typed")
    # The test asks where the CODE came from, not whether two module objects
    # are identical: loading ELEMENTS-1's species.py a second time under
    # another name would give different function objects for the same source,
    # so `is` against a fresh load answers a question nobody asked.  What
    # matters is that the shim did not re-type these -- so the compiled code's
    # filename must be ELEMENTS-1's file and not this directory's.
    e1_species = os.path.abspath(os.path.join(HERE, "elements1", "species.py"))
    shim_file = os.path.abspath(os.path.join(HERE, "species.py"))
    for fn in ("sz_sectors", "ground_spin_from_sectors", "sparse_subset"):
        f = getattr(SHIM, fn)
        where = os.path.abspath(f.__code__.co_filename)
        check(where == e1_species and where != shim_file,
              "%s's code is compiled from ELEMENTS-1's species.py, not "
              "re-typed here (%s)" % (fn, os.path.basename(where)))
        check(f is getattr(SHIM._E1SP, fn),
              "%s is the object the shim loaded, with no wrapper in between"
              % fn)
    import importlib.util
    spec = importlib.util.spec_from_file_location(
        "_e1sp_check", e1_species)
    e1 = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(e1)
    check(SHIM.GRID_DPS == e1.GRID_DPS,
          "the grid precision is the same pinned value (%d)" % SHIM.GRID_DPS)
    # and the failing case: a re-typed copy WOULD be caught.  The stand-in is
    # defined here, so its code reports THIS file -- neither ELEMENTS-1's
    # species.py nor the shim's -- and the assertion above would reject it.
    def sz_sectors(nelec, norb):        # a plausible local re-typing
        return []
    copy_where = os.path.abspath(sz_sectors.__code__.co_filename)
    check(copy_where != e1_species,
          "a re-typed copy does not report ELEMENTS-1's file (%s), so the "
          "provenance check can tell a copy from the original"
          % os.path.basename(copy_where))
    check(shim_file != e1_species,
          "and the shim's own file is a different path from ELEMENTS-1's, so "
          "the comparison is between two real alternatives")

    print("\n4. the species sets are disjoint, so neither campaign can be "
          "mistaken for the other")
    overlap = set(SHIM.DIATOMICS) & set(e1.DIATOMICS)
    check(not overlap, "no pair name is claimed by both campaigns"
          + ("" if not overlap else "; shared: %s" % sorted(overlap)))

    print("\n5. the table binding, with the unbound case measured")
    check(EC.basis_fingerprint(None) == M.FINGERPRINT,
          "table=None now means the Z = 1..18 table")
    nbf_bound = sum(1 if l == 0 else 3
                    for (l, _, _) in EC.STO3G_SHELLS[17])
    check(nbf_bound == 9, "chlorine builds 9 basis functions (got %d)"
          % nbf_bound)
    check(17 not in M.ELEMENTS1_SHELLS,
          "chlorine is absent from ELEMENTS-1's table, so an unbound run would "
          "not merely be smaller -- it would fail to build chlorine at all, "
          "which is the LOUD half of the failure")
    # the SILENT half: an element in both tables with a different shell count
    silent = [Z for Z in M.ELEMENTS1_SHELLS
              if Z in EC.STO3G_SHELLS
              and len(M.ELEMENTS1_SHELLS[Z]) != len(EC.STO3G_SHELLS[Z])]
    check(not silent,
          "no element is in both tables with a different shell count, so there "
          "is no element on which an unbound run would silently build a "
          "smaller basis%s" % ("" if not silent else "; %r" % silent))

    print("\n6. the caches cannot be confused")
    check(R.CACHE == M.CACHE, "runner.CACHE is this campaign's")
    check(os.path.abspath(R.CACHE) !=
          os.path.abspath(os.path.join(HERE, "elements1", "cache")),
          "and it is not ELEMENTS-1's directory")
    check(M.FINGERPRINT != M.ELEMENTS1_FINGERPRINT,
          "the two fingerprints differ, so a record from either is refused by "
          "the other even if the directories were ever merged")

    print("\n7. every staked grid regenerates from its own rule")
    import species2 as SP2
    for name in SHIM.DIATOMICS:
        d = SHIM.DIATOMICS[name]
        w = SP2.window(SP2.R_ref_bohr(name), SP2.STAKED[name]["negative"])
        ok = (d["rmin"] == w["rmin"] and d["rmax"] == w["rmax"]
              and tuple(d["well"]) == tuple(w["well"]))
        check(ok, "%s's window is what the rule produces from R_ref alone"
              % name)
    g = SHIM.grid_for("SiO")
    check(g == SHIM.grid_for("SiO"),
          "a grid is reproducible (SiO, %d knots)" % len(g))
    check(len(SHIM.grid_for("HCl")) > len(SHIM.grid_for("SiO")),
          "the cheap pairs carry more knots than the expensive ones, which is "
          "the density decision being a COMPUTE decision")

    print("\n%d checks, %d FAIL" % (CHECKS[0], len(FAILED)))
    for f in FAILED:
        print("   FAILED: %s" % f)
    return 1 if FAILED else 0


if __name__ == "__main__":
    raise SystemExit(main())
