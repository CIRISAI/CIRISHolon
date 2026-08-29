"""
emit_engine.py -- write the referee output in the ELEMENTS-1 ENGINE lane's
schema: engine/crates/holon-chem/tests/data/elements1/{<PAIR>.json, atoms.json}.

Every number is a PLAIN FIXED-POINT DECIMAL STRING.  No exponent notation
anywhere, because the engine's exact-decimal comparator asserts against 'e' and
'E', and because a JSON number is an f64 the moment anything parses it -- which
would throw away the digits past the 17th that are the entire point.

Two conventions worth stating out loud, since both have bitten this campaign:

  * F_hartree_per_bohr is the FORCE, -dE/dR.  It is POSITIVE on the repulsive
    wall (the pair pushes apart) and negative inside the well.  This file's H2
    first grid point reads +10.59..., matching the banked h2_potential.json's
    own F column sign for sign.  An earlier version of the referee's stencil
    returned -dE/dR where dE/dR was meant and every force came out mirrored;
    the sign is therefore asserted here rather than assumed.

  * Digits are emitted only where they are earned.  Each value carries at most
    the significant digits the eigenvalue certificate covers at its geometry,
    written in fixed point; the engine's 64-place comparator reads the unwritten
    places as zero, which cannot affect a comparison at 1e-10.  Nothing is
    padded out to look more precise than it is.
"""

import json
import os
import sys
from decimal import Decimal, ROUND_HALF_EVEN, getcontext

from mpmath import mp, mpf, nstr

import curve as CV
import elements_core as EC
import runner as R
import species as SP

getcontext().prec = 200

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_OUT = os.path.join(HERE, "engine_handoff", "elements1")
MAX_FRAC = 64          # the engine comparator's fixed-point fractional width
MAX_INT = 4            # ... and its integer width
SIG = 50


def fixed_str(x, sig=SIG, max_frac=MAX_FRAC):
    """Plain fixed-point decimal, no exponent, at most `sig` significant digits."""
    x = mpf(x)
    if x == 0:
        return "0." + "0" * 1
    d = Decimal(mp.nstr(x, sig + 2, strip_zeros=False))
    exp10 = int(mp.floor(mp.log10(abs(x))))
    frac = max(1, min(max_frac, sig - 1 - exp10))
    d = d.quantize(Decimal(1).scaleb(-frac), rounding=ROUND_HALF_EVEN)
    s = format(d, "f")
    if "." not in s:
        s += ".0"
    if "e" in s or "E" in s:
        raise AssertionError("exponent notation leaked into %r" % s)
    intpart = s.lstrip("-").split(".")[0]
    if len(intpart) > MAX_INT:
        raise AssertionError("%s needs %d integer digits, comparator allows %d"
                             % (s, len(intpart), MAX_INT))
    return s


def derivative_quality(rec):
    """(route, F uncertainty, E2 uncertainty).  NEVER returns zero.

    Where the raised-precision stencil covers every knot, the bound is the
    energy certificate PROPAGATED through the difference formula -- a stencil
    amplifies the value uncertainty by sum|w|/h and sum|w|/h^2, factors of 2e11
    and 6.5e22 at h = 1e-11, so quoting the energy's own bound for a derivative
    column would understate it by twenty-odd orders.

    Where the column comes from the local interpolant instead, the bound is the
    largest thing actually measured: the interpolant-versus-stencil deviation
    where there are stencil geometries, and always at least the spread between
    two interpolant window widths, which needs no stencil at all.

    A derivative column may not declare 0.0.  An exact zero is a MISSING bound
    wearing a number: it survives a presence check and then reads as "perfect",
    which would have this file's consumer grade the referee's interpolant as if
    it were exact.  The energy column has always carried a real bound; the
    derivative columns now do too.
    """
    d = rec["diagnostics"]
    route = rec["provenance"]["derivative_route"]
    ebound = mpf(d.get("energy_uncertainty_total")
                 or d.get("eigen_temple_bound_max") or 0)
    if route.startswith("stencil at every grid point"):
        uF, uE2 = CV.stencil_derivative_bounds(ebound)
        return route, uF, uE2
    cand1 = [mpf(d.get("interpolant_window_spread_d1") or 0)]
    cand2 = [mpf(d.get("interpolant_window_spread_d2") or 0)]
    if d.get("n_stencil_geometries"):
        cand1.append(mpf(d.get("fd_vs_interpolant_d1_max_abs") or 0))
        cand2.append(mpf(d.get("fd_vs_interpolant_d2_max_abs") or 0))
    return route, max(cand1), max(cand2)


SCALARS = ("E_asymptote", "R_e", "D_e")
COLUMNS = ("R_grid_bohr", "E_hartree", "F_hartree_per_bohr",
           "E2_hartree_per_bohr2")


def diff_against_existing(path, new):
    """What changed since the file on disk -- COMPUTED, not recalled.

    A delivery note that says "nothing else changed" is an assertion, and an
    assertion in place of a check is how this campaign's defects have all
    looked.  One of mine said exactly that while LiH's 138 forces and 138
    curvatures had all moved and its derivative route had gone from interpolant
    to full stencil; the other lane caught it by diffing rather than reading.
    So the diff is computed here and the note quotes it.
    """
    if not os.path.exists(path):
        return ["NEW FILE"]
    try:
        old = json.load(open(path))
    except Exception:
        return ["unreadable previous file"]
    out = []
    for k in COLUMNS:
        a, b = old.get(k), new.get(k)
        if a != b:
            n = sum(1 for i in range(min(len(a or []), len(b or [])))
                    if a[i] != b[i])
            out.append("%s: %d of %d entries changed" % (k, n, len(b or [])))
    for k in SCALARS:
        if old.get(k) != new.get(k):
            out.append("%s: %s -> %s" % (k, old.get(k), new.get(k)))
    op = (old.get("derivative_provenance") or {})
    np_ = (new.get("derivative_provenance") or {})
    for k in sorted(set(op) | set(np_)):
        if op.get(k) != np_.get(k):
            out.append("derivative_provenance.%s: %s -> %s"
                       % (k, op.get(k), np_.get(k)))
    if (old.get("spin") or {}) != (new.get("spin") or {}):
        out.append("spin block changed")
    # KEYS, not just values.  The named lists above cannot see a block being
    # ADDED or REMOVED, so this diff once reported "no change" on six files
    # that had all just gained a whole new top-level block -- the same shape as
    # the defect it was written to catch, one level out.  Enumerate the key
    # sets and compare everything not already covered.
    added = sorted(set(new) - set(old))
    dropped = sorted(set(old) - set(new))
    if added:
        out.append("keys ADDED: %s" % ", ".join(added))
    if dropped:
        out.append("keys REMOVED: %s" % ", ".join(dropped))
    covered = set(COLUMNS) | set(SCALARS) | {"derivative_provenance", "spin"}
    for k in sorted((set(old) & set(new)) - covered):
        if old.get(k) != new.get(k):
            out.append("%s changed" % k)
    return out or ["no change"]


def emit_pair(name, rec, outdir):
    """One <PAIR>.json from an assembled species record."""
    ex = rec["exact"]
    bound = rec["bound"]
    out = {
        "model": "%s/STO-3G/FCI" % name,
        "R_grid_bohr": list(ex["R_grid_bohr"]),
        "E_hartree": [fixed_str(v) for v in ex["E_hartree"]],
        "F_hartree_per_bohr": [fixed_str(v) for v in ex["F_hartree_per_bohr"]],
        "E2_hartree_per_bohr2": [fixed_str(v)
                                 for v in ex["E2_hartree_per_bohr2"]],
        "E_asymptote": fixed_str(ex["E_asymptote"]),
        "R_e": (ex.get("R_e_exact_decimal") or fixed_str(ex["R_e"]))
               if bound else "unbound",
        "D_e": fixed_str(ex["D_e"]) if bound else "unbound",
    }
    # Extra keys, ignorable: E is referee-grade everywhere, but F and E2 are
    # only referee-grade where the stencil covers every knot.  Comparing F at
    # 1e-10 against an interpolant-derived column would fail on the REFEREE's
    # error, not the engine's, so the uncertainty travels with the numbers.
    sp = rec.get("spin")
    if sp is None:
        raise AssertionError(
            "%s: refusing to emit without a spin audit. An unverified spin "
            "sector is not a passing check, it is an absent one." % name)
    # A multiplicity CHANGE along the curve is physics and is declared, not
    # refused -- refusing it would refuse F2, whose triplet genuinely drops
    # below its singlet past about 4 bohr.  What is refused is a state sitting
    # above the next Sz sector's minimum, which is a solver failure whichever
    # spin wins.
    if sp.get("spin_resolved_out_to_bohr") is None:
        raise AssertionError(
            "%s: the ground multiplicity is not resolved at ANY geometry" % name)
    if sp.get("sector_ordering_violations"):
        raise AssertionError(
            "%s: E_min(Sz) > E_min(Sz+1) at %s -- the solver returned a state "
            "that is not the lowest in its sector"
            % (name, sp["sector_ordering_violations"][:3]))
    out["spin"] = sp
    route, uF, uE2 = derivative_quality(rec)
    eu = mpf(rec["diagnostics"].get("energy_uncertainty_total") or 0)
    if eu <= 0:
        raise AssertionError(
            "%s: refusing to declare a zero ENERGY uncertainty. Temple's bound "
            "is 0 for a one-determinant space because the eigensolve there is "
            "exact -- but the integrals and the transformation feeding it are "
            "not, and the measured route A vs route B deviation bounds those."
            % name)
    # A declared bound that does not cover the digits being PRINTED is honest
    # but useless: the file would carry 50 significant digits of E while saying
    # it is only good to 17.  Refuse rather than ship a self-contradicting file.
    tgt = mpf(rec["diagnostics"].get("certificate_target") or 0)
    if tgt > 0 and eu > tgt:
        raise AssertionError(
            "%s: declared energy uncertainty %s exceeds one unit in the 50th "
            "significant digit (%s); the file would print more digits than it "
            "claims. Recertify the weak geometries first."
            % (name, nstr(eu, 6), nstr(tgt, 6)))
    if uF <= 0 or uE2 <= 0:
        raise AssertionError(
            "%s: refusing to declare a zero derivative uncertainty (F %s, "
            "E2 %s). A zero bound is a missing bound that passes a presence "
            "check." % (name, uF, uE2))
    # WHICH DERIVATIVE THIS IS, where the two stop being the same quantity.
    #
    # This F column is d/dR of the LOWEST EIGENVALUE, taken numerically from
    # energies along the curve.  An analytic Hellmann-Feynman force is
    # <v|H'|v>, the slope of the BRANCH its eigenvector sits on.  Away from a
    # degeneracy those are the same number.  Inside one they are not: every
    # member of a degenerate level is an eigenvector, the branches have
    # different slopes, and which member a solver lands on depends on its
    # orbital basis -- so an analytic force is basis-dependent exactly where the
    # energy is not.  The engine lane measured this on F2: through an unrotated
    # basis its forces match this column to 1e-12..1e-15 at all 134 geometries,
    # through its rotated production basis they differ by 3.4e-10 at 7.738 bohr,
    # same geometry, same energy, different branch.
    #
    # So the geometries where the multiplicity is unresolved are named here.  A
    # consumer comparing analytic derivatives should expect to disagree at
    # those, for a reason that is about what a derivative IS and not about
    # either side's arithmetic.
    # An ABSENT resolution column used to fall back to "resolved everywhere",
    # which is the campaign's own defect shape written into this file: the
    # emitted `geometries_where_multiplicity_is_unresolved` would have been an
    # empty list meaning "we did not look", indistinguishable from an empty
    # list meaning "we looked and found none".  This lane WRITES that column,
    # so its absence is a broken pipeline, not a permitted state.
    rbg = sp.get("resolved_by_geometry")
    if rbg is None:
        raise AssertionError(
            "%s: the spin block has no per-geometry resolution column. An "
            "empty list of unresolved geometries would then mean 'nobody "
            "looked' while reading as 'none found'." % name)
    if len(rbg) != len(out["R_grid_bohr"]):
        raise AssertionError(
            "%s: the resolution column covers %d of %d geometries"
            % (name, len(rbg), len(out["R_grid_bohr"])))
    unres = [rs for i, rs in enumerate(out["R_grid_bohr"]) if not rbg[i]]
    # THE GRID, AND WHY IT HAS THE KNOTS IT HAS.
    #
    # N2 and CO are emitted on a SPARSE STAKED SUBSET, and a reader is entitled
    # to be suspicious of a sparse curve: choosing which points to show is the
    # oldest way to make a curve behave.  So the rule travels with the file in
    # staked parameters only, and -- more to the point -- it is REGENERATED
    # here from those parameters and compared to the emitted grid, so "anyone
    # can reproduce this subset" is a check rather than a promise.
    d = SP.DIATOMICS[name]
    full = CV.build_grid(d["rmin"], d["rmax"], d["nbase"], d["well"],
                         d["nsplit"])
    gp = {
        "rule": ("uniform in R^(-1/4) between rmin and rmax from nbase points, "
                 "refined by nsplit inside the staked well window"),
        "staked_parameters": {"rmin_bohr": d["rmin"], "rmax_bohr": d["rmax"],
                              "nbase": d["nbase"], "nsplit": d["nsplit"],
                              "well_window_bohr": list(d["well"])},
        "full_grid_knots": len(full),
        "emitted_knots": len(out["R_grid_bohr"]),
    }
    if d.get("sparse"):
        regen = SP.sparse_subset(full, d["well"], d["sparse"])
        gp["subset_rule"] = (
            "keep index i when i %% stride == 0, stride = well_stride inside "
            "the staked well window and tail_stride outside it, always keeping "
            "the first and last knot; the window and the strides are design "
            "inputs frozen before any energy was computed, so the subset is a "
            "function of the grid rule alone and consults no result")
        gp["subset_parameters"] = dict(d["sparse"])
        gp["why_sparse"] = (
            "a %d-determinant geometry is not affordable at full grid density "
            "on a shared machine; the response is FEWER points, never cheaper "
            "ones -- every knot here is the same exact-in-model full CI at the "
            "same working precision, dual-route and certified"
            % rec.get("ndet", 0))
        gp["regenerated_from_the_rule_matches"] = (
            [str(x) for x in regen] == [str(x) for x in out["R_grid_bohr"]])
        if not gp["regenerated_from_the_rule_matches"]:
            raise AssertionError(
                "%s: the emitted grid is NOT the subset its own stated rule "
                "produces (%d emitted vs %d regenerated). A sparse grid whose "
                "rule does not reproduce it is a chosen grid."
                % (name, len(out["R_grid_bohr"]), len(regen)))
    else:
        gp["subset_rule"] = "none -- every knot of the staked grid is emitted"
        if len(out["R_grid_bohr"]) != len(full):
            raise AssertionError(
                "%s: %d knots emitted from a %d-knot staked grid with no "
                "subset rule declared" % (name, len(out["R_grid_bohr"]),
                                          len(full)))
    out["grid_provenance"] = gp
    out["derivative_provenance"] = {
        "quantity": "d/dR of the lowest eigenvalue, taken numerically from "
                    "energies; NOT an analytic <v|H'|v> branch derivative",
        "branch_dependence": ("inside a degenerate level the two differ: every "
                              "member is an eigenvector, the branches have "
                              "different slopes, and which one an analytic "
                              "force follows depends on the orbital basis, so "
                              "it is basis-dependent exactly where the energy "
                              "is not"),
        "geometries_where_multiplicity_is_unresolved": unres,
        "route": route,
        "F_max_abs_uncertainty_hartree_per_bohr": fixed_str(uF, sig=8),
        "F_uncertainty_scope": ("applies where the multiplicity is resolved; at "
                                "the geometries listed above the comparison "
                                "against an analytic force is not a comparison "
                                "of the same quantity"),
        "E2_max_abs_uncertainty_hartree_per_bohr2": fixed_str(uE2, sig=8),
        "E_max_abs_uncertainty_hartree":
            fixed_str(rec["diagnostics"].get("energy_uncertainty_total") or 0,
                      sig=8),
        "basis": ("STO-3G, 8 decimals, ties to even; fingerprint %s"
                  % rec["provenance"].get("basis_fingerprint")),
    }
    # the R grid is already an exact short decimal; assert it, do not reformat
    for r in out["R_grid_bohr"]:
        if "e" in r or "E" in r or "." not in r:
            raise AssertionError("R grid entry %r is not a plain decimal" % r)
    n = len(out["R_grid_bohr"])
    for k in ("E_hartree", "F_hartree_per_bohr", "E2_hartree_per_bohr2"):
        if len(out[k]) != n:
            raise AssertionError("%s: %s has %d entries, grid has %d"
                                 % (name, k, len(out[k]), n))
    # sign convention assertion: the force is repulsive at the innermost point
    if Decimal(out["F_hartree_per_bohr"][0]) <= 0:
        raise AssertionError("%s: F at the innermost geometry is %s; the force "
                             "must be positive on the repulsive wall"
                             % (name, out["F_hartree_per_bohr"][0]))
    # Put the grid block LAST.  The consumer reads these files by scanning for
    # quoted keys rather than by deserialising, so a new block ahead of the old
    # ones is a needless hazard even when no key names collide.
    out["grid_provenance"] = out.pop("grid_provenance")
    p = os.path.join(outdir, "%s.json" % name)
    changes = diff_against_existing(p, out)
    with open(p, "w") as f:
        json.dump(out, f, indent=1)
    return p, n, changes


def emit_atoms(atoms, outdir):
    syms = [EC.ELEMENT_SYMBOL[Z] for Z in SP.ATOMS]
    out = {"symbols": syms,
           "E_hartree": [fixed_str(atoms["atoms"][s]["E"]) for s in syms]}
    p = os.path.join(outdir, "atoms.json")
    with open(p, "w") as f:
        json.dump(out, f, indent=1)
    return p


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    strict = "--loose" not in sys.argv
    outdir = args[0] if args else DEFAULT_OUT
    os.makedirs(outdir, exist_ok=True)
    mp.dps = R.DPS

    with open(os.path.join(HERE, "elements_atoms.json")) as f:
        atoms = json.load(f)
    p = emit_atoms(atoms, outdir)
    print("wrote %s  (%d symbols)" % (p, len(SP.ATOMS)))

    src = None
    for cand in ("elements_potential.json", "elements_potential_partial.json"):
        if os.path.exists(os.path.join(HERE, cand)):
            src = cand
            break
    if src is None:
        print("no assembled curves yet")
        return 1
    with open(os.path.join(HERE, src)) as f:
        pot = json.load(f)
    print("source: %s" % src)
    fp = EC.basis_fingerprint()
    have = []
    for name in ("H2", "LiH", "Li2", "HF", "N2", "F2", "CO", "He2", "Ne2"):
        rec = pot["species"].get(name)
        if rec is None:
            print("  %-4s not assembled yet" % name)
            continue
        got = rec["provenance"].get("basis_fingerprint")
        if got != fp:
            stale = os.path.join(outdir, "%s.json" % name)
            if os.path.exists(stale):
                os.remove(stale)
            print("  %-4s REFUSED: assembled under basis %s, current is %s "
                  "-- recompute before handing it over" % (name, got, fp))
            continue
        # WHAT STRICT MODE REFUSES, AND WHAT IT NO LONGER DOES.
        #
        # It still refuses a zero uncertainty on any column, a stale basis
        # fingerprint, exponent notation, an over-wide integer part, mismatched
        # array lengths, and a force that is not repulsive at the innermost
        # geometry.  Those are defects in the FILE and stay defects whoever
        # reads it.
        #
        # It no longer refuses an interpolant-derived F or E2 column.  That
        # guard existed to protect a consumer that graded F against a flat
        # 1e-10; the engine lane's reworked gate now grades every column
        # against the uncertainty the file declares for it, so the declaration
        # does that work and the guard would only block an honest handoff.
        # Withholding a column whose stated worth is accurate is its own kind
        # of misreport.
        p, n, changes = emit_pair(name, rec, outdir)
        cd = rec["diagnostics"].get("certified_significant_digits_min")
        print("  %-4s %3d geometries -> %s   (>= %s certified significant "
              "digits at every geometry)" % (name, n, os.path.basename(p), cd))
        for c in changes:
            print("         %s" % c)
        have.append(name)
    # A coverage manifest, so the consumer reads what is present and what is
    # owed from the DATA rather than hardcoding a list that can drift out of
    # step with the files beside it.
    ALL = ["H2", "LiH", "Li2", "HF", "N2", "F2", "CO", "He2", "Ne2"]
    man = {
        "model": "ELEMENTS1/STO-3G/FCI",
        "basis_fingerprint": fp,
        "basis": "STO-3G, 8 decimals, ties to even; hydrogen verbatim from the "
                 "banked H2 referee h2_core.py",
        "atoms": "atoms.json (all ten, H through Ne, referee-grade)",
        "staked_nine": ALL,
        "staked_nine_note": ("the species the freeze names. present + owed is "
                             "this set, always: a pair may move from present to "
                             "owed, it may not leave both. Coverage read purely "
                             "from a manifest can shrink silently when a file "
                             "goes missing, because the thing the gate checks "
                             "against shrank with it."),
        "pairs_present": have,
        "pairs_owed": [x for x in ALL if x not in have],
        "grading": {
            "E_hartree": "referee-grade for every emitted pair; the declared "
                         "bound is max(Temple, measured route A vs route B)",
            "F_hartree_per_bohr": "grade varies per pair; read "
                                  "derivative_provenance.route and gate against "
                                  "the declared uncertainty, not a flat bound",
            "E2_hartree_per_bohr2": "as F",
        },
        "rules": [
            "no column declares a zero uncertainty; a zero is a missing bound "
            "that passes a presence check",
            "every number is a plain fixed-point decimal string, never a JSON "
            "number and never exponent notation",
            "R_grid_bohr is authoritative and is an exact short decimal, so "
            "the published R IS the R the energy was computed at",
        ],
    }
    with open(os.path.join(outdir, "manifest.json"), "w") as f:
        json.dump(man, f, indent=1)
    print("wrote manifest.json  (present %s | owed %s)"
          % (",".join(have) or "-", ",".join(man["pairs_owed"]) or "-"))
    print("\n%d of 9 pairs emitted%s" % (len(have),
                                         "  [PREVIEW, not referee-grade]"
                                         if not strict else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
