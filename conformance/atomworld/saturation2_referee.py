#!/usr/bin/env python3
"""
saturation2_referee.py -- the 50-digit referee for SATURATION-2, the (O, H, H)
three-body surface.

WHAT THIS IS
  An independent, high-precision arbiter for the heteronuclear three-body term
  of the many-body expansion:

      E(A)            one atom, its own minimal-|Sz| sector
      V2_AB(r)        = E(AB; r) - E(A) - E(B)                 [pair term]
      dE3(O,H,H)      = E(OHH) - sum_pairs V2 - E(O) - 2 E(H)
                      = E(OHH) + E(O) + 2 E(H)
                        - E(OH; x) - E(OH; y) - E(HH; z)       [triple term]

  All subsystem energies are the electronic ground state in the subsystem's
  minimal-|Sz| sector plus the classical nuclear repulsion, computed by full CI
  in the declared STO-3G minimal basis from closed-form Gaussian integrals in
  mpmath.  Exact-in-model, not a prediction of experiment.

  Sectors, and why the minimal one suffices:
    H   :  1 electron,  1 spatial orbital  (Sz = 1/2)  ->    1 determinant
    O   :  8 electrons, 5 spatial orbitals (Sz = 0)    ->   25 determinants
    H2  :  2 electrons, 2 spatial orbitals (Sz = 0)    ->    4 determinants
    OH  :  9 electrons, 6 spatial orbitals (Sz = 1/2)  ->   90 determinants
    OHH : 10 electrons, 7 spatial orbitals (Sz = 0)    ->  441 determinants
  The minimal-|Sz| block contains one component of EVERY spin multiplet the
  electron count can form, so that block's lowest eigenvalue is the global
  ground state.  <S^2> is computed and REPORTED at every geometry, so the
  multiplicity is a measured label and never an assumption (M-PARITY-PROTECT).

INDEPENDENCE (gate R1's premise)
  This file shares no code with the Rust engine (holon-chem).  It shares its
  integrals, its determinant CI and its certified eigensolver with
  `elements1_referee/`, BY IMPORT rather than by copy -- the ELEMENTS-1 lane's
  machinery is the committed bank, and a second transcription of a bank is how
  a bank stops being one.  What is written here is the (O, H, H) geometry
  construction, the many-body decomposition, the staked geometry set and the
  comparison.

  The only thing deliberately shared with the engine is the MODEL DEFINITION --
  Z, the STO-3G contraction.  A basis set is an input, not a derivation.

THE STAKED GEOMETRY RULE, STATED RESULT-BLIND
  Every geometry below is a function of the DECLARED domain constants alone
  (R_DOM_LO, R_HI, C_LO, C_HI, fixed by `s2_domain.py`'s truncation and fence
  measurements before any referee energy existed) and of a fixed integer
  ladder.  Nothing in the set consults an energy, a minimum, a bond length or
  an angle:

    SIDES  = a six-rung geometric ladder from the staked domain floor to the
             truncation radius, rounded to two decimals;
    ANGLES = the fence C_LO, the collinear edge C_HI, and two interior values
             at the third-points of [C_LO, C_HI];
    SET    = every (x, y) with x <= y drawn from SIDES x SIDES, crossed with
             ANGLES.

  That is 21 * 4 = 84 geometries, above the prereg's >= 48, and it spans the
  domain by construction: compact (both sides at the floor), bent (interior
  angles), linear (c = C_HI), stretched (both sides at R_HI), near-boundary
  (y = R_HI) and closed (c = C_LO) all appear as named rows of the ladder
  rather than as points someone picked after looking.

USAGE
  python3 saturation2_referee.py --time            # cost of one point, no cache
  python3 saturation2_referee.py --selftest        # precision demonstration
  python3 saturation2_referee.py --grid [--out water_referee.json]
  python3 saturation2_referee.py --point X Y C
"""

import argparse
import json
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "elements1_referee"))

from mpmath import mp, mpf, nstr, sqrt  # noqa: E402

import elements_core as E  # noqa: E402
import fci as F  # noqa: E402

# ---------------------------------------------------------------- the declared domain
#
# These four numbers are the table's box, and they are INPUTS here.  Their
# provenance is `examples/s2_domain.rs`.  The truncation radius is 15 and NOT
# the 14 a coarse shell sweep first licensed: re-swept at five times the
# resolution and with the angle carried down to c = 0.002, the b = 14 shell
# reads 1.0091e-5 Ha against a 1e-5 stake, where the coarse sweep had said
# 9.71e-6 -- a grid maximum understates its own supremum, and every shell's
# worst reading sat at the sweep's own closed-angle floor.  At b = 15 the worst
# point stops being the near-collinear chain and becomes a stretched H2 with the
# oxygen 12 bohr away, 3.54e-6, and that tail is ALGEBRAIC: `s2_dispersion.rs`
# staked R^-6 for it and measured -5.01, the quadrupole-quadrupole law rather
# than the dispersion one.  It is purely three-body because an isolated hydrogen
# is spherical in this basis and carries no quadrupole, so no O-H pair term can
# hold any of it; replacing the oxygen with closed-shell NEON removes the
# algebraic sector entirely.  The closed-angle corner stays smooth down to c = 0.005,
# so the fence at 0.05 is above anything the surface does.

R_DOM_LO = mpf("0.9")          # staked domain floor in either O-H side
R_HI = mpf("15.0")             # truncation radius on the LARGER O-H side
C_LO = mpf("0.05")             # closed-angle fence, c = sqrt(1 - cos theta)
C_HI = sqrt(mpf(2))            # the collinear edge, theta = 180 degrees

DPS = 60
REPORT = 50

# Below this route-A gap the ground state is treated as degenerate and the second
# CI route is declared unavailable rather than attempted.  See `solve_atoms`.
# In hartree, and set well above the solver's own certified accuracy so it names
# a physical degeneracy rather than a numerical one.
DEGENERACY_GAP = mpf("1e-6")

CACHE = os.path.join(HERE, "s2_runs", "referee_cache")

Z_O, Z_H = 8, 1


def s(x, n=REPORT):
    return nstr(mpf(x), n, strip_zeros=False)


# The engine-side comparator's fixed-point width (`tests/common/mod.rs`,
# FRAC_DIGITS). The columns below are rendered to fit it exactly.
GATE_FRAC_DIGITS = 60


def sfix(x, n=REPORT):
    """The gate's view of a number: PLAIN fixed point, never exponent notation,
    and never wider than the comparator can hold.

    Two separate constraints, both learned by being violated.

    NO EXPONENTS.  The engine-side gate compares in exact decimal
    (`tests/common/mod.rs`'s `decimal_minus_f64`) rather than parsing the referee
    into an f64 first, and that comparator refuses exponent notation on purpose:
    a silent `parse::<f64>()` of a 50-digit referee is exactly the contamination
    it exists to avoid.

    NO MORE THAN `GATE_FRAC_DIGITS` PLACES.  `dE3` runs down to 5.7e-23 on the
    dissociated shells, and 50 SIGNIFICANT digits of a number that small is 72
    DECIMAL PLACES -- wider than the comparator's fixed-point buffer, which
    refuses rather than silently truncating.  So the fraction is trimmed here, to
    the comparator's own width, and the trimming costs at most 1e-60 against a
    stake of 1e-10.

    `rows` keeps the full significant-digit strings; this is the gate's column
    view and says so.
    """
    t = nstr(mpf(x), n, strip_zeros=False, min_fixed=-(10 ** 6), max_fixed=10 ** 6)
    if "." not in t:
        return t
    head, frac = t.split(".", 1)
    return head + "." + frac[:GATE_FRAC_DIGITS]


# ---------------------------------------------------------------- the point solver


def solve_atoms(atoms, want_B=True, tol_digits=None, two_sz=None):
    """Ground-state total energy of an arbitrary nuclear arrangement.

    `atoms` is [(Z, (x, y, z)), ...].  The Sz sector defaults to the minimal one
    the electron count allows, which is the declared choice and the one that
    still contains every multiplet.

    `two_sz` overrides it.  That is what the spin-resolution pass uses: a
    multiplet of total spin S appears, at the same energy, in every sector with
    |Sz| <= S and in none above, so E_min(Sz = 1) - E_min(Sz = 0) is exactly the
    gap between the ground state and the lowest TRIPLET.  A positive gap says
    the ground state is a resolved singlet; a zero gap says singlet and triplet
    are degenerate there and the multiplicity can only be REPORTED.
    """
    mol = E.molecule(atoms)
    nelec, norb = mol["nelec"], mol["nbf"]
    if two_sz is None:
        two_sz = nelec % 2
    na = (nelec + two_sz) // 2
    nb = nelec - na
    if tol_digits is None:
        tol_digits = mp.dps - 8

    C, sev = F.lowdin_orbitals(mol["S"])
    hA, gA = F.mo_integrals(mol, C)
    sp = F.DetSpace(norb, na, nb)
    rA = F.solve_certified(F.RouteAOp(sp, hA, gA), tol_digits=tol_digits)
    out = dict(
        nbf=norb,
        nelec=nelec,
        two_sz=two_sz,
        ndet=sp.ndet,
        E_nuc=s(mol["E_nuc"]),
        E=rA["energy"] + mol["E_nuc"],
        resid_A=nstr(rA["resid"], 6),
        bound_A=nstr(rA["bound_temple"], 6) if rA["bound_temple"] is not None else None,
        overlap_min_eigenvalue=nstr(min(sev), 6),
    )
    vec = rA.get("vector")
    if vec is None:
        raise RuntimeError(
            "the solver returned no vector; the spin audit cannot be skipped silently"
        )
    s2 = F.spin_squared(sp, vec)
    twoS, s2dev = F.spin_from_s2(s2)
    out["S2_expectation"] = nstr(s2, 12)
    out["two_S_from_S2"] = twoS
    out["S2_deviation_from_exact"] = nstr(s2dev, 6)
    # ---- the second route, and where it is DECLARED UNAVAILABLE --------------
    #
    # Route B re-solves in a randomly rotated orbital basis, which is the
    # referee's own independence check: two CI routes over the same integrals
    # landing on one number.  It cannot be run everywhere, and the reason is
    # physical rather than budgetary.
    #
    # At a DISSOCIATED geometry -- oxygen with its hydrogens eight bohr away --
    # the ground state is near-degenerate (O's 3P times two hydrogen doublets),
    # so the Temple bound has no gap to certify against and the rotated route
    # grinds.  MEASURED on x = y = 8.545, c = 0.959: route A converges in about
    # 36 s at dps 30 and 45 and roughly a minute at 60; route A PLUS route B was
    # still running after twenty minutes and had to be killed.  A referee that
    # stalls on a tenth of its staked set is not a referee.
    #
    # So route B is skipped where route A's own gap says it cannot be certified,
    # and the skip is RECORDED per geometry rather than absorbed.  The decision is
    # made from a quantity route A already computed, before route B is paid for,
    # and it is deterministic: no wall clock is consulted.
    gap = rA.get("gap")
    out["gap_A"] = nstr(gap, 6) if gap is not None else None
    if want_B and gap is not None and gap < DEGENERACY_GAP:
        out["route_B_skipped"] = (
            "ground state is near-degenerate: route-A gap %s < %s Ha, so the "
            "Temple bound has nothing to certify against and the rotated route "
            "does not converge" % (nstr(gap, 6), nstr(DEGENERACY_GAP, 3))
        )
    elif want_B:
        Q = F.rotation_matrix(norb)
        hB, gB = F.mo_integrals(mol, F.rotate_orbitals(C, Q))
        rB = F.solve_certified(F.RouteBOp(sp, hB, gB), tol_digits=tol_digits)
        EB = rB["energy"] + mol["E_nuc"]
        out["E_B"] = s(EB)
        out["dev_AB"] = nstr(abs(out["E"] - EB), 6)
        out["resid_B"] = nstr(rB["resid"], 6)
    return out


# ---------------------------------------------------------------- the run lock
#
# WHY THIS EXISTS, and it is not hypothetical.  A --grid run was relaunched
# against a corrected domain while the previous one was still going, and for
# twenty minutes two processes wrote the same log and were both aimed at the same
# output file -- so the stale run, finishing later, would have OVERWRITTEN the
# corrected artifact with a staked set built to the wrong R_HI.  Nothing would
# have looked broken: the file would have existed, parsed, and been wrong.
#
# The sibling `elements1_referee/` has the same guard and a test that fires it
# against a live process, which is where the idea comes from.


class RunLocked(RuntimeError):
    pass


def _lock_path(out):
    return out + ".lock"


def _alive(pid):
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


def acquire_run_lock(out, force=False):
    """Refuse to start when another live process is aimed at the same output.

    A lock left behind by a dead process is STALE and is taken over with a note,
    because refusing forever on a crash would be a guard that costs more than it
    saves.  A lock held by a live process is a refusal.
    """
    p = _lock_path(out)
    if os.path.exists(p):
        try:
            with open(p) as fh:
                held = json.load(fh)
            pid = int(held.get("pid", -1))
        except Exception:
            pid, held = -1, {}
        if pid > 0 and _alive(pid) and pid != os.getpid():
            if not force:
                raise RunLocked(
                    "another run (pid %d, started %s) is already aimed at %s. "
                    "Two runs writing one artifact is how a stale staked set "
                    "overwrites a corrected one. Kill it, or pass --force-lock."
                    % (pid, held.get("started", "?"), out)
                )
            print("# --force-lock: taking over a lock held by live pid %d" % pid)
        else:
            print("# stale lock from pid %s taken over" % pid)
    os.makedirs(os.path.dirname(os.path.abspath(p)) or ".", exist_ok=True)
    with open(p, "w") as fh:
        json.dump(
            dict(pid=os.getpid(), started=time.strftime("%Y-%m-%d %H:%M:%S"), out=out),
            fh,
        )
    return p


def release_run_lock(p):
    try:
        os.remove(p)
    except OSError:
        pass


# ---------------------------------------------------------------- the cache


def _cache_path(tag):
    return os.path.join(CACHE, "%s__dps%d.json" % (tag.replace("/", "_"), mp.dps))


def cached(tag, fn, force=False):
    os.makedirs(CACHE, exist_ok=True)
    p = _cache_path(tag)
    if not force and os.path.exists(p):
        with open(p) as fh:
            rec = json.load(fh)
        if rec.get("basis_fingerprint") != E.basis_fingerprint():
            raise RuntimeError(
                "cache record %s was written against a different basis; refusing" % tag
            )
        rec["E"] = mpf(rec["E_str"])
        return rec
    t0 = time.time()
    rec = fn()
    rec["tag"] = tag
    rec["dps"] = mp.dps
    rec["seconds"] = time.time() - t0
    rec["basis_fingerprint"] = E.basis_fingerprint()
    rec["E_str"] = s(rec["E"])
    # Round the FRESH value to the reported width too. Otherwise a first run and a
    # cache-hit run return different numbers for the same geometry, and the artifact
    # stops being reproducible from its own cache.
    rec["E"] = mpf(rec["E_str"])
    out = dict(rec)
    out["E"] = out["E_str"]
    with open(p, "w") as fh:
        json.dump(out, fh, indent=1, sort_keys=True)
    return rec


# ---------------------------------------------------------------- the species


def atom_O(force=False):
    return cached("O", lambda: solve_atoms([(Z_O, (mpf(0), mpf(0), mpf(0)))]), force)


def atom_H(force=False):
    return cached("H", lambda: solve_atoms([(Z_H, (mpf(0), mpf(0), mpf(0)))]), force)


def pair(Za, Zb, r, force=False):
    r = mpf(r)
    tag = "pair_%d_%d_%s" % (Za, Zb, nstr(r, 20, strip_zeros=False))
    return cached(
        tag,
        lambda: solve_atoms(
            [(Za, (mpf(0), mpf(0), mpf(0))), (Zb, (mpf(0), mpf(0), r))]
        ),
        force,
    )


def ohh_sites(x, y, u):
    """The engine's own placement: O at the origin, H1 along +x at `x`, H2 at
    `y` from the origin with `cos(theta_HOH) = u`.  Written the same way here
    so the two implementations differ in arithmetic and not in geometry."""
    x, y, u = mpf(x), mpf(y), mpf(u)
    sn = sqrt(max(0, 1 - u * u))
    return [
        (Z_O, (mpf(0), mpf(0), mpf(0))),
        (Z_H, (x, mpf(0), mpf(0))),
        (Z_H, (y * u, y * sn, mpf(0))),
    ]


def hh_distance(x, y, u):
    x, y, u = mpf(x), mpf(y), mpf(u)
    return sqrt(max(0, x * x + y * y - 2 * x * y * u))


def _ohh_tag(x, y, u, suffix=""):
    return "ohh%s_%s_%s_%s" % (
        suffix,
        nstr(mpf(x), 20, strip_zeros=False),
        nstr(mpf(y), 20, strip_zeros=False),
        nstr(mpf(u), 20, strip_zeros=False),
    )


def water(x, y, u, force=False, want_B=True):
    return cached(
        _ohh_tag(x, y, u),
        lambda: solve_atoms(ohh_sites(x, y, u), want_B=want_B),
        force,
    )


def water_triplet(x, y, u, force=False):
    """The same geometry in the Sz = 1 sector: 245 determinants, and the lowest
    state there IS the lowest triplet.  Route B is not run — this energy is used
    only as the upper half of a GAP, and the gap's tolerance is orders above the
    two routes' own disagreement."""
    return cached(
        _ohh_tag(x, y, u, "_sz1"),
        lambda: solve_atoms(ohh_sites(x, y, u), want_B=False, two_sz=2),
        force,
    )


# How far above the singlet the lowest triplet must sit before the ground state
# is called RESOLVED rather than degenerate.  In hartree, and set at the scale
# the sandbox's own dynamics could tell apart rather than at the solver's noise
# floor -- a gap of 1e-30 is a degeneracy for every purpose this campaign has.
SPIN_RESOLVED_GAP = mpf("1e-6")


def de3(x, y, u, force=False, want_B=True):
    """The three-body term and every part it is built from."""
    z = hh_distance(x, y, u)
    w = water(x, y, u, force=force, want_B=want_B)
    eo, eh = atom_O(force), atom_H(force)
    p1 = pair(Z_O, Z_H, x, force)
    p2 = pair(Z_O, Z_H, y, force)
    p3 = pair(Z_H, Z_H, z, force)
    t = water_triplet(x, y, u, force=force)
    gap = t["E"] - w["E"]
    d = w["E"] + eo["E"] + 2 * eh["E"] - p1["E"] - p2["E"] - p3["E"]
    return dict(
        E_OHH_triplet=s(t["E"]),
        spin_gap=s(gap, 20),
        spin_resolved=bool(gap > SPIN_RESOLVED_GAP),
        ndet_OHH_triplet=t["ndet"],
        x=s(x, 20),
        y=s(y, 20),
        u=s(u, 20),
        z=s(z),
        E_OHH=s(w["E"]),
        E_O=s(eo["E"]),
        E_H=s(eh["E"]),
        E_OH_x=s(p1["E"]),
        E_OH_y=s(p2["E"]),
        E_HH_z=s(p3["E"]),
        V2_OH_x=s(p1["E"] - eo["E"] - eh["E"]),
        V2_OH_y=s(p2["E"] - eo["E"] - eh["E"]),
        V2_HH_z=s(p3["E"] - 2 * eh["E"]),
        dE3=s(d),
        ndet_OHH=w["ndet"],
        S2_OHH=w["S2_expectation"],
        two_S_OHH=w["two_S_from_S2"],
        S2_OH=p1["S2_expectation"],
        two_S_OH=p1["two_S_from_S2"],
        S2_O=eo["S2_expectation"],
        two_S_O=eo["two_S_from_S2"],
        resid_OHH=w["resid_A"],
        bound_OHH=w["bound_A"],
        gap_OHH=w.get("gap_A"),
        dev_AB_OHH=w.get("dev_AB"),
        route_B_skipped=w.get("route_B_skipped"),
        seconds_OHH=w.get("seconds"),
    )


# ---------------------------------------------------------------- the staked set


def staked_sides():
    """A six-rung geometric ladder from the staked domain floor to the
    truncation radius, rounded to two decimals.  A function of R_DOM_LO and
    R_HI and nothing else."""
    n = 6
    out = []
    for i in range(n):
        t = mpf(i) / (n - 1)
        r = R_DOM_LO * (R_HI / R_DOM_LO) ** t
        out.append(mpf(nstr(r, 4, strip_zeros=False)))
    return out


def staked_angles():
    """The fence, the collinear edge, and the two third-points between them."""
    return [C_LO, C_LO + (C_HI - C_LO) / 3, C_LO + 2 * (C_HI - C_LO) / 3, C_HI]


def staked_geometries():
    """(x, y, u) with x <= y, every pair of the side ladder crossed with every
    staked angle.  84 geometries; the prereg stakes >= 48."""
    sides = staked_sides()
    out = []
    for i, x in enumerate(sides):
        for y in sides[i:]:
            for c in staked_angles():
                out.append((x, y, 1 - c * c))
    return out


def family_of(x, y, c):
    """The named region a staked geometry falls in.  A function of the declared
    constants and the geometry alone; it labels the row, it does not select it."""
    sides = staked_sides()
    tags = []
    if x == sides[0] and y == sides[0]:
        tags.append("compact")
    if y == R_HI:
        tags.append("near-boundary")
    if x >= sides[3]:
        tags.append("stretched")
    if c == C_HI:
        tags.append("linear")
    if c == C_LO:
        tags.append("closed")
    if C_LO < c < C_HI:
        tags.append("bent")
    return "+".join(tags) if tags else "interior"


# ---------------------------------------------------------------- the modes


def cmd_time(args):
    mp.dps = args.dps
    sides = staked_sides()
    print("# staked side ladder:", [s(v, 6) for v in sides])
    print("# staked angles c   :", [s(v, 6) for v in staked_angles()])
    print("# staked geometries :", len(staked_geometries()))
    x, y, c = mpf("1.5"), mpf("2.0"), mpf("1.0")
    for label, fn in [
        ("H   ", lambda: solve_atoms([(Z_H, (mpf(0), mpf(0), mpf(0)))])),
        ("O   ", lambda: solve_atoms([(Z_O, (mpf(0), mpf(0), mpf(0)))])),
        ("H2  ", lambda: solve_atoms(
            [(Z_H, (mpf(0), mpf(0), mpf(0))), (Z_H, (mpf(0), mpf(0), mpf("1.4")))])),
        ("OH  ", lambda: solve_atoms(
            [(Z_O, (mpf(0), mpf(0), mpf(0))), (Z_H, (mpf(0), mpf(0), mpf("1.9")))])),
        ("OHH ", lambda: solve_atoms(ohh_sites(x, y, 1 - c * c))),
    ]:
        t0 = time.time()
        r = fn()
        print(
            "  %s ndet %4d  E = %s  resid %s  <S2> %s  2S %s  %.1f s"
            % (label, r["ndet"], s(r["E"], 22), r["resid_A"],
               r["S2_expectation"], r["two_S_from_S2"], time.time() - t0)
        )


def cmd_point(args):
    mp.dps = args.dps
    x, y, c = mpf(args.point[0]), mpf(args.point[1]), mpf(args.point[2])
    rec = de3(x, y, 1 - c * c, force=args.force)
    print(json.dumps(rec, indent=1, sort_keys=True))


def cmd_selftest(args):
    """The reported digits are DEMONSTRATED: one geometry is recomputed at a
    higher working precision and the two must agree past the reporting width."""
    x, y, c = mpf("1.5"), mpf("2.0"), mpf("1.0")
    mp.dps = args.dps
    a = de3(x, y, 1 - c * c, force=True)
    mp.dps = args.dps + 30
    b = de3(x, y, 1 - c * c, force=True)
    dev = abs(mpf(a["dE3"]) - mpf(b["dE3"]))
    print("  dE3 @ dps %d : %s" % (args.dps, a["dE3"]))
    print("  dE3 @ dps %d : %s" % (args.dps + 30, b["dE3"]))
    print("  deviation   : %s" % nstr(dev, 6))
    ok = dev < mpf(10) ** (-(REPORT - 5))
    print("  SELFTEST %s" % ("PASS" if ok else "FAIL"))
    return 0 if ok else 1


def cmd_grid(args):
    mp.dps = args.dps
    lock = acquire_run_lock(args.out, force=args.force_lock)
    try:
        return _grid_body(args)
    finally:
        release_run_lock(lock)


def _grid_body(args):
    geoms = staked_geometries()
    if args.limit:
        geoms = geoms[: args.limit]
    rows = []
    t0 = time.time()
    for i, (x, y, u) in enumerate(geoms):
        c = sqrt(1 - u)
        rec = de3(x, y, u, want_B=not args.no_route_b)
        rec["c"] = s(c, 20)
        rec["family"] = family_of(x, y, c)
        rows.append(rec)
        # `s(..., 8)` and NOT a slice of the 50-digit string: truncating the string
        # cuts the EXPONENT off, and a correct 2.93e-18 then reads as 2.93.
        print(
            "  [%3d/%3d] x=%s y=%s c=%s  %-28s dE3 = %14s  2S %s  (%.0f s elapsed)"
            % (i + 1, len(geoms), s(x, 6), s(y, 6), s(c, 6), rec["family"],
               s(mpf(rec["dE3"]), 8), rec["two_S_OHH"], time.time() - t0),
            flush=True,
        )
    # The gate's own view: parallel columns of plain decimals, prefixed so that the
    # key search in `tests/common/mod.rs` cannot land on a same-named field inside
    # `rows` instead. `rows` stays for a human reading the file.
    eo, eh = atom_O(), atom_H()
    out = dict(
        producer="saturation2_referee.py",
        campaign="SATURATION-2",
        model="(O,H,H)/STO-3G/FCI, minimal-|Sz| sector",
        dps=mp.dps,
        reported_digits=REPORT,
        basis_fingerprint=E.basis_fingerprint(),
        domain=dict(
            R_DOM_LO=s(R_DOM_LO, 20), R_HI=s(R_HI, 20),
            C_LO=s(C_LO, 20), C_HI=s(C_HI, 20),
        ),
        staked_sides=[s(v, 20) for v in staked_sides()],
        staked_angles=[s(v, 20) for v in staked_angles()],
        n_geometries=len(rows),
        col_E_O=sfix(eo["E"]),
        col_E_H=sfix(eh["E"]),
        col_x=[sfix(mpf(r["x"]), 20) for r in rows],
        col_y=[sfix(mpf(r["y"]), 20) for r in rows],
        col_u=[sfix(mpf(r["u"]), 20) for r in rows],
        col_c=[sfix(mpf(r["c"]), 20) for r in rows],
        col_z=[sfix(mpf(r["z"])) for r in rows],
        col_E_OHH=[sfix(mpf(r["E_OHH"])) for r in rows],
        col_E_OH_x=[sfix(mpf(r["E_OH_x"])) for r in rows],
        col_E_OH_y=[sfix(mpf(r["E_OH_y"])) for r in rows],
        col_E_HH_z=[sfix(mpf(r["E_HH_z"])) for r in rows],
        col_dE3=[sfix(mpf(r["dE3"])) for r in rows],
        col_family=[r["family"] for r in rows],
        col_two_S_OHH=[str(r["two_S_OHH"]) for r in rows],
        col_dual_route=["0" if r.get("route_B_skipped") else "1" for r in rows],
        col_spin_gap=[sfix(mpf(r["spin_gap"]), 20) for r in rows],
        col_spin_resolved=["1" if r["spin_resolved"] else "0" for r in rows],
        rows=rows,
    )
    with open(args.out, "w") as fh:
        json.dump(out, fh, indent=1, sort_keys=True)
    print("wrote %s (%d geometries, %.0f s)" % (args.out, len(rows), time.time() - t0))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dps", type=int, default=DPS)
    ap.add_argument("--time", action="store_true")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--grid", action="store_true")
    ap.add_argument("--point", nargs=3)
    ap.add_argument("--out", default=os.path.join(HERE, "water_referee.json"))
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--force", action="store_true")
    ap.add_argument("--no-route-b", action="store_true")
    ap.add_argument(
        "--force-lock",
        action="store_true",
        help="take over a run lock held by a LIVE process; see acquire_run_lock",
    )
    args = ap.parse_args()
    if args.time:
        return cmd_time(args) or 0
    if args.selftest:
        return cmd_selftest(args)
    if args.point:
        return cmd_point(args) or 0
    if args.grid:
        return cmd_grid(args) or 0
    ap.print_help()
    return 1


if __name__ == "__main__":
    sys.exit(main())
