"""The MIXTURES-1 referee's binding to the ELEMENTS-1 machinery.

ELEMENTS-1's `runner.py` and `fci.py` are reused rather than reimplemented --
they are the code whose integrals, three FCI routes and Temple certificates have
already been graded against the banked H2 referee, and a second copy would be a
second thing that can drift.  What this module does is bind them to THIS
campaign's declared model and THIS campaign's cache, and it does both
explicitly, because both are ways one campaign's numbers could enter the other's
artifact without anyone choosing that.

TWO BINDINGS, AND WHY EACH IS EXPLICIT.

1. THE TABLE.  `runner`'s functions take `table=None`, which means "the
   ELEMENTS-1 table, Z = 1..10", and `build_curves.py` -- the stage machinery
   this campaign reuses UNMODIFIED -- calls them without a table, because it was
   written when there was only one.  A forgotten `table=` would not raise: for a
   second-row species it would silently drop the 3s and 3p shells and report a
   HIGHER energy, perfectly converged for the wrong model.

   So the DEFAULT is rebound rather than the call sites edited:
   `elements_core.STO3G_SHELLS` becomes the Z = 1..18 table in this process.
   `assert_bound()` checks both that the rebinding happened AND that it was
   distinguishable -- a check that the table is right is worth nothing if the
   two tables would hash the same, and it says so if they ever do.

2. THE CACHE.  `runner.CACHE` is a module global pointing at ELEMENTS-1's
   45 MB of solves.  It is redirected here.  The basis fingerprint would refuse
   a cross-campaign record anyway -- the two tables hash differently -- but
   "refused" and "never offered" are different states, and a shared directory
   would put 11314 records one fingerprint change away from being adopted.

THE TAG NAMESPACES ARE SEPARATE FROM THE START.

ELEMENTS-1 filed three kinds of record -- certified dual-route point,
single-route stencil energy, spin -- under one tag namespace, and for its heavy
species the stencil precision equalled the grid precision, so a stencil at a
grid point produced that grid point's key.  It cost two Li2 geometries their
certificates without any downstream check noticing, because the assemble only
asked whether a record existed.  Here every kind is prefixed at the point where
the tag is made, and there is no un-prefixed form to forget.
"""
import hashlib
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
_E1 = os.path.join(HERE, "elements1")
for _p in (HERE, _E1):
    if _p not in sys.path:
        sys.path.insert(0, _p)

from mpmath import mp                                          # noqa: E402
import basis2                                                  # noqa: E402
import elements_core as EC                                     # noqa: E402
import runner as R                                             # noqa: E402
import species2 as SP2                                         # noqa: E402

TABLE = basis2.STO3G_18
FINGERPRINT = basis2.fingerprint()

CACHE = os.path.join(HERE, "cache")
R.CACHE = CACHE
os.makedirs(CACHE, exist_ok=True)

# THE TABLE BINDING, DONE ONCE, WHERE FORGETTING IT IS IMPOSSIBLE.
#
# Every entry point in `runner.py` takes `table=None`, and None means "the
# ELEMENTS-1 table, Z = 1..10".  `build_curves.py` -- the stage machinery this
# campaign reuses unmodified -- calls them without a table, because it was
# written when there was only one.  A forgotten `table=` would not raise: for a
# second-row species it would silently drop the 3s and 3p shells and report a
# HIGHER energy, perfectly converged for the wrong model.
#
# So rather than pass the table at 40 call sites in code that must not be
# edited, the DEFAULT is rebound: `elements_core.STO3G_SHELLS` becomes the
# Z = 1..18 table in this process.  `basis_fingerprint(None)` then hashes it
# too, so cache records carry this campaign's fingerprint automatically and a
# record from either campaign is refused by the other.
#
# ORDER MATTERS AND IS THE ONE FRAGILE THING HERE: basis2 BUILDS the 18-element
# table out of the 10-element one, so the table must exist before the rebinding.
# `basis2` is imported above, which is what guarantees it; the snapshot below
# keeps ELEMENTS-1's original reachable so the two fingerprints can still be
# compared, and `assert_bound()` fails if it ever stops being distinguishable.
ELEMENTS1_SHELLS = EC.STO3G_SHELLS          # snapshot BEFORE the rebinding
ELEMENTS1_FINGERPRINT = EC.basis_fingerprint(ELEMENTS1_SHELLS)
EC.STO3G_SHELLS = TABLE
_SYM = dict(EC.ELEMENT_SYMBOL)
_SYM.update({Z: sym for Z, sym in SP2.SYMBOL.items() if Z > 10})
EC.ELEMENT_SYMBOL = _SYM

DPS = R.DPS          # 60
REPORT = R.REPORT    # 50


# ---------------------------------------------------------------------------
# Tags.  Three kinds, three namespaces, made only by these functions.
# ---------------------------------------------------------------------------
def _h(s):
    return hashlib.sha1(s.encode()).hexdigest()[:14]


def point_tag(name, rs, dps):
    """A certified dual-route geometry.  `rs` is the exact decimal STRING."""
    return "%s_%s_d%d" % (name, _h(rs), dps)


def fd_tag(name, rs, dps):
    """A single-route stencil energy."""
    return "fd_" + point_tag(name, rs, dps)


def spin_tag(name, rs, dps):
    """An <S^2> reading."""
    return "spin_" + point_tag(name, rs, dps)


def atom_tag(Z):
    return "atom_Z%d" % Z


def spec_for(name, rs):
    d = SP2.STAKED[name]
    return ("diatomic", SP2.Z_OF[d["a"]], SP2.Z_OF[d["b"]], rs)


# ---------------------------------------------------------------------------
# The three workers.  Each passes the table; none has a default.
# ---------------------------------------------------------------------------
def run_point(spec, tag, want_C=True, want_B=True, dps=DPS, force=False,
              sectors=None, max_outer=9):
    mp.dps = dps
    return R.run_point(spec, tag, want_C=want_C, want_B=want_B, dps=dps,
                       table=TABLE, force=force, sectors=sectors,
                       max_outer=max_outer)


def energy_only(spec, tag, dps=DPS, force=False, max_outer=12):
    mp.dps = dps
    return R.energy_only(spec, tag, dps=dps, table=TABLE, force=force,
                         max_outer=max_outer)


def spin_only(spec, tag, dps=DPS, force=False):
    mp.dps = dps
    return R.spin_only(spec, tag, dps=dps, table=TABLE, force=force)


def cache_get(tag, dps, kind):
    return R.cache_get(tag, dps, TABLE, kind=kind)


# ---------------------------------------------------------------------------
# The binding check, run at import.
# ---------------------------------------------------------------------------
def assert_bound():
    """Both bindings, checked by making the unbound case visibly different.

    A check that the table is passed is worth nothing unless NOT passing it
    would produce a different answer, so that is what is measured: the number of
    basis functions the default table would build for chlorine against the
    number this one does.  If those were ever equal the check would be vacuous
    and would say so.
    """
    problems = []
    if R.CACHE != CACHE:
        problems.append("runner.CACHE is %r, not this campaign's" % R.CACHE)
    # The rebinding must have HAPPENED: table=None now means Z = 1..18.
    if EC.basis_fingerprint(None) != FINGERPRINT:
        problems.append("elements_core's default table is not this campaign's; "
                        "every call in build_curves.py that omits table= would "
                        "silently compute a first-row-only basis")
    if 17 not in EC.STO3G_SHELLS:
        problems.append("chlorine is absent from the rebound default table")
    # ...and it must have been DISTINGUISHABLE, or the check above is vacuous.
    if FINGERPRINT == ELEMENTS1_FINGERPRINT:
        problems.append("this campaign's table fingerprints the same as "
                        "ELEMENTS-1's; the binding check cannot detect "
                        "anything and has gone vacuous")
    if 17 in ELEMENTS1_SHELLS:
        problems.append("ELEMENTS-1's table already contained Z=17; the "
                        "second-row binding is not distinguishable")
    if basis2.NBF[17] != 9:
        problems.append("chlorine should carry 9 basis functions, has %d"
                        % basis2.NBF[17])
    if problems:
        raise RuntimeError("MIXTURES-1 binding is wrong: " + "; ".join(problems))
    return True


assert_bound()


if __name__ == "__main__":
    print("MIXTURES-1 referee binding")
    print("  table fingerprint  %s  (Z = 1..%d)" % (FINGERPRINT,
                                                    max(TABLE)))
    print("  ELEMENTS-1's       %s" % ELEMENTS1_FINGERPRINT)
    print("  table=None means   %s  (rebound: %s)"
          % (EC.basis_fingerprint(None),
             "yes" if EC.basis_fingerprint(None) == FINGERPRINT else "NO"))
    print("  cache              %s" % CACHE)
    print("  bindings checked   ok")
    print()
    print("  tag namespaces, on one geometry:")
    for f in (point_tag, fd_tag, spin_tag):
        print("    %-10s %s" % (f.__name__, f("Cl2", "3.76", DPS)))
