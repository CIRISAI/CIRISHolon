"""MIXTURES-1's species module, standing in for ELEMENTS-1's.

WHY IT IS NAMED `species.py` AND NOT SOMETHING CLEARER.

`build_curves.py` -- the whole stage machinery, every guard it learned the hard
way, the assembler, the gates -- does `import species as SP`.  Putting THIS
directory ahead of the ELEMENTS-1 one on `sys.path` makes that import resolve
here, and the entire pipeline then runs on this campaign's species with no edit
to a single line of the code that is already graded against the banked H2
referee.

The alternative was to fork `build_curves.py`, and forking it would have
duplicated the run lock, the pool guard, the merge-not-narrow repair, the
grid_provenance regeneration, the no-defaulted-spin-column refusal and the
zero-probe refusal -- every one of which exists because it once failed.  A fork
inherits the guards and then stops inheriting the fixes.

WHAT THIS FILE MUST PROVIDE, and the check that it does: everything the shared
code reads out of `species`.  `test_species_shim.py` asserts the two modules'
public surfaces match, so a name added to ELEMENTS-1's is a failure here rather
than an AttributeError three hours into a pool.

GROUND Sz AND THE DEGENERACY RULE ARE THE SAME FUNCTIONS, imported from the
ELEMENTS-1 module by explicit path rather than re-typed.  They are campaign-
independent mathematics -- a spin-S multiplet appears in every sector with
|Sz| <= S and none above -- and two copies of that is two things that can drift.
"""
import importlib.util
import os
import sys

from mpmath import mpf

HERE = os.path.dirname(os.path.abspath(__file__))
_E1 = os.path.join(HERE, "elements1")
if _E1 not in sys.path:
    sys.path.append(_E1)

import curve                                                   # noqa: E402
import species2 as SP2                                         # noqa: E402
# Importing m1core here is not incidental.  `build_curves.py` imports `species`
# before it builds anything, so this is the earliest point in the shared
# pipeline that runs code this campaign owns -- and m1core is where the two
# bindings live (the declared table becomes the DEFAULT table, and the cache is
# redirected).  Both have to be in force before the first molecule is built, in
# the parent process, so that forked pool workers inherit them.
import m1core as _M                                            # noqa: E402,F401


def _load_elements1_species():
    """ELEMENTS-1's species module, loaded by PATH under another name.

    A plain `import species` from here would find this file.  The two modules
    have the same name on purpose -- that is the whole mechanism -- so the
    original has to be reached explicitly.
    """
    spec = importlib.util.spec_from_file_location(
        "elements1_species", os.path.join(_E1, "species.py"))
    m = importlib.util.module_from_spec(spec)
    sys.modules["elements1_species"] = m
    spec.loader.exec_module(m)
    return m


_E1SP = _load_elements1_species()

# Campaign-independent mathematics, shared rather than re-typed.
sz_sectors = _E1SP.sz_sectors
ground_spin_from_sectors = _E1SP.ground_spin_from_sectors
sparse_subset = _E1SP.sparse_subset
GRID_DPS = _E1SP.GRID_DPS

ATOMS = list(range(1, 19))

# --------------------------------------------------------------------------
# DENSITY IS A COMPUTE DECISION, and these are the measured costs it is made
# from -- not the determinant counts, which understate the spread by an order
# of magnitude, and not where anything interesting is expected to be.
#
# Measured at one geometry each (`_cost_probe.py`), on this machine, at dps 60:
#
#   pair   nbf  ndet     nnz        AO integrals   one working-precision matvec
#   Ar2    18   1        1          152 s          0.00 s
#   NeAr   14   1        1           59 s          0.00 s
#   HCl    10   100      1.0e4       15 s          0.03 s
#   ClF    14   196      3.8e4       58 s          0.10 s
#   Cl2    18   324      1.0e5      154 s          0.27 s
#   N2     10   14,400   8.8e6       19 s         78.69 s   <- the calibration
#
# For the four cheap pairs the AO integrals ARE the cost and the determinant
# space is free, so density is limited by nbf, not by ndet: Ar2 and Cl2 cost
# about 155 s per point however many knots there are.  S2, NaH and SiO are the
# other regime, where the matvec dominates; they get ELEMENTS-1's sparse
# treatment, which is FEWER points and never cheaper ones.
#
# The windows are NOT written here.  They come from `species2.window()`, a
# function of one declared number per pair, so a grid is regenerated from the
# rule rather than promised by a table.
# --------------------------------------------------------------------------
_DENSITY = {
    #        nbase  nsplit  heavy  routeC   sparse
    "HCl":  (100,   2,      False, "all",   None),
    "ClF":  (90,    2,      False, "all",   None),
    "Cl2":  (60,    1,      False, "all",   None),
    "Ar2":  (60,    1,      False, "all",   None),
    "NeAr": (60,    1,      False, "all",   None),
    "S2":   (20,    2,      True,  "none",  dict(well_stride=2, tail_stride=3)),
    "NaH":  (20,    2,      True,  "none",  dict(well_stride=2, tail_stride=3)),
    # SiO carries the MINIMAL rule instead of a stride subset -- see the note
    # above the rule itself.  The stride parameters are kept so the file still
    # declares the grid it decimates FROM.
    "SiO":  (20,    2,      True,  "none",  dict(well_stride=2, tail_stride=3,
                                                 minimal=True)),
}


# --------------------------------------------------------------------------
# SiO's MINIMAL grid, staked before any SiO energy exists.
#
# Measured (FEASIBILITY.md): SiO is 196,889,056 nonzero Hamiltonian elements,
# one working-precision matvec is 2378 s, and a certified point is 20-91 hours.
# The lead's ruling on the frozen R2 stake is the ELEMENTS-1 precedent: resolve
# by GRID, never by precision.  Digits do not bend; knots do.
#
# THE RULE, AND WHY IT IS BLIND TO EVERY RESULT.  Five knots, chosen from the
# staked window alone:
#
#     the first grid point            -- the repulsive wall
#     the first knot in the window    -- the near side of the well
#     the middle knot in the window   -- the well
#     the last knot in the window     -- the far side of the well
#     the last grid point             -- the separated-atom limit
#
# The window is a design input frozen before any energy was computed, and the
# base grid is a function of R_ref and the rule.  So the subset is a function of
# those two and nothing else, and anyone can regenerate it.  It is manifestly
# not a choice of which points to show: the well window brackets where a
# minimum would be IF there is one, which is the question, not the answer.
#
# WHAT FIVE KNOTS CANNOT SUPPORT, said here rather than discovered downstream:
# no stencil, so no referee-grade F or E2, and no Newton-refined R_e.  Those
# columns must ship as a DECLARED ABSENCE or be marked interpolant-grade over
# five points, which is a weak number honestly labelled.  The ENERGIES and the
# SPIN AUDIT are exact-in-model at the same 50 digits as every other pair --
# which is the whole of "a sparse exact referee is a referee".
# --------------------------------------------------------------------------
MINIMAL_SPECIES = ("SiO",)


def minimal_subset(grid, well):
    """Wall, three window knots, asymptote -- from the window and grid alone."""
    from mpmath import mp as _mp
    lo, hi = mpf(well[0]), mpf(well[1])
    inwin = [i for i, rs in enumerate(grid) if lo <= mpf(rs) <= hi]
    keep = {0, len(grid) - 1}
    if inwin:
        keep |= {inwin[0], inwin[len(inwin) // 2], inwin[-1]}
    return [grid[i] for i in sorted(keep)]


def _build():
    out = {}
    for name, d in SP2.STAKED.items():
        nbase, nsplit, heavy, routeC, sparse = _DENSITY[name]
        w = SP2.window(SP2.R_ref_bohr(name), d["negative"])
        e = dict(Z1=SP2.Z_OF[d["a"]], Z2=SP2.Z_OF[d["b"]],
                 rmin=w["rmin"], rmax=w["rmax"], well=w["well"],
                 nbase=nbase, nsplit=nsplit, heavy=heavy, routeC=routeC,
                 R_ref_angstrom=d["R_ref_angstrom"],
                 grid_rule=("NEGATIVE" if d["negative"] else "BOUND"),
                 note=("NEGATIVE CONTROL: gate E1 stakes no well deeper than "
                       "1e-4 hartree anywhere on this grid"
                       if d["negative"] else None))
        if sparse:
            e["sparse"] = sparse
        out[name] = e
    return out


DIATOMICS = _build()

E2_ORDER = [n for n in SP2.E2_ORDER if n in DIATOMICS]
E2_UNBOUND = list(SP2.E2_UNBOUND)


def grid_for(name):
    """The staked grid, built at a pinned precision so the same species yields
    the same grid whatever precision the caller happens to be in."""
    from mpmath import mp as _mp
    d = DIATOMICS[name]
    old = _mp.dps
    _mp.dps = GRID_DPS
    try:
        g = curve.build_grid(d["rmin"], d["rmax"], d["nbase"], d["well"],
                             d["nsplit"])
        if d.get("sparse"):
            if d["sparse"].get("minimal"):
                g = minimal_subset(g, d["well"])
            else:
                g = sparse_subset(g, d["well"], d["sparse"])
        return g
    finally:
        _mp.dps = old


if __name__ == "__main__":
    print("%-5s %-7s %-7s %-15s %-6s %-6s %s"
          % ("pair", "rmin", "rmax", "well", "knots", "heavy", "ndet"))
    for name in DIATOMICS:
        d = DIATOMICS[name]
        g = grid_for(name)
        print("%-5s %-7s %-7s %-15s %-6d %-6s %s"
              % (name, d["rmin"], d["rmax"], "[%s, %s]" % d["well"], len(g),
                 "yes" if d["heavy"] else "no",
                 format(SP2.ndet(name)[0], ",")))
    print("\ntotal staked geometries: %d"
          % sum(len(grid_for(n)) for n in DIATOMICS))
