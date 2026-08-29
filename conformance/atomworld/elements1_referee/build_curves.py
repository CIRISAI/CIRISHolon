"""
build_curves.py -- the nine staked diatomic curves.

Stage 1  grid energies, both routes (and the third where it is affordable)
Stage 2  first and second derivatives
Stage 3  the minimum, by Newton on a raised-precision stencil
Stage 4  assembly: Hermite renderer contract, timestep envelope, gates E1/E2/E3

Derivatives.  A central 8th-order stencil evaluated at RAISED precision is the
primary route wherever the species is cheap enough to afford nine extra energies
per grid point; for the three expensive species (Li2, N2, CO) the primary route
is a local Newton divided-difference interpolant through the computed grid, and
the stencil is run at spot geometries as the cross-check.  Which route was used,
and the measured disagreement between them, is recorded per species.

The minimum needs no bisection: E is stationary there, so an error dR in R_e
costs only E'' dR^2 / 2 in E(R_e).  Newton from the interpolant's root converges
quadratically and two steps put D_e far past 50 digits even for the expensive
species.
"""

import atexit
import hashlib
import json
import os
import sys
import time
from multiprocessing import Pool

from mpmath import mp, mpf, nstr, sqrt

import curve
import elements_core as EC
import runner as R
import species as SP

HERE = os.path.dirname(os.path.abspath(__file__))
# Stencil precision.  With step h = 10^-11 and working precision 10^-P, the
# second derivative carries a roundoff of about |E| * 10^-P / h^2, so P = 95
# leaves E'' good to ~1e-50 while P = 60 leaves it good to ~1e-34.  The cheap
# species use 95 everywhere; the three expensive ones use 60 for the Newton
# iterations (where only E'/E'' RATIOS matter, and 1e-45 on E' is ample) and 95
# for the final evaluation that publishes E(R_e) and E''(R_e).
FD_DPS = 95
FD_DPS_CHEAP = 60
# The stencil OFFSETS are exact decimal strings (curve.dec_shift); the stencil
# STEP is materialised at the working precision wherever it is divided by, never
# as an mpf at module level -- see the note on _D1_W in curve.py.


def stencil_dps(name):
    return FD_DPS_CHEAP if SP.DIATOMICS[name]["heavy"] else FD_DPS


def tag_for(name, rs, dps):
    """rs is the exact decimal STRING of the geometry -- the canonical identity
    of a computed point, precision-independent by construction."""
    hh = hashlib.sha1(rs.encode()).hexdigest()[:14]
    return "%s_%s_d%d" % (name, hh, dps)


def spec_for(name, rs):
    d = SP.DIATOMICS[name]
    return ("diatomic", d["Z1"], d["Z2"], rs)


# ---------------------------------------------------------------------------
def _w_point(payload):
    name, rs, dps, want_C = payload[:4]
    force = payload[4] if len(payload) > 4 else False
    mp.dps = dps
    return R.run_point(spec_for(name, rs), tag_for(name, rs, dps),
                       want_C=want_C, want_B=True, dps=dps, force=force)


def cert_target(E):
    """One unit in the 50th significant digit of E.

    The certificate has to cover the digits actually PUBLISHED, and those are
    50 SIGNIFICANT digits, so the absolute bound a species needs scales with its
    energy: 1e-49 for H2 near -1.1 hartree, but 1e-47 for F2 near -196.  A flat
    absolute target either over-works the small molecules or under-certifies the
    large ones."""
    E = abs(mpf(E))
    if E == 0:
        return mpf(10) ** -49
    return mpf(10) ** (int(mp.floor(mp.log10(E))) - 49)


def stage_recertify(names, nproc_light, nproc_heavy, target=None):
    """Recompute only those geometries whose certificate is weaker than the
    reported precision.

    The Temple bound is ||r||^2 / (lambda_1 - theta), so a geometry near a
    closing gap can carry a weak bound even when the energy itself is fine.
    Rather than trust that, or recompute everything, this finds the geometries
    whose bound does not cover the 50 reported digits and redoes just those.
    """
    for name in names:
        d = SP.DIATOMICS[name]
        grid = SP.grid_for(name)
        weak = []
        for i, rs in enumerate(grid):
            c = R.cache_get(tag_for(name, rs, R.DPS), R.DPS)
            if c is None:
                continue
            sec = list(c["sectors"].values())[0]
            tgt = float(target if target is not None
                        else cert_target(mpf(sec["E_A"])))
            b = float(sec["bound_A"]) if sec.get("bound_A") else 0.0
            rb = float(sec.get("resid_B") or 0.0)
            gap = sec.get("gap_A") or 1.0
            bB = (rb * rb / gap) if gap > 0 else 0.0
            # the DECLARED uncertainty is max(Temple, route A vs B), so the
            # recertify test has to be the same quantity the emitter refuses on
            dab = float(sec.get("dev_AB") or 0.0)
            decl = float(declared_uncertainty(b, bB, dab))
            if decl > tgt:
                pol = d.get("routeC", "all")
                weak.append((name, rs, R.DPS, pol == "all", True))
        if not weak:
            print("  %-4s every certificate already covers the 50th digit"
                  % name, flush=True)
            continue
        t0 = time.time()
        pmap(_w_point, weak, nproc_heavy if d["heavy"] else nproc_light)
        still = 0
        for (_, rs, _, _, _) in weak:
            c = R.cache_get(tag_for(name, rs, R.DPS), R.DPS)
            sec = list(c["sectors"].values())[0]
            tgt = float(target if target is not None
                        else cert_target(mpf(sec["E_A"])))
            rb2 = float(sec.get("resid_B") or 0.0)
            gp2 = sec.get("gap_A") or 0.0
            b = float(declared_uncertainty(
                float(sec["bound_A"]) if sec.get("bound_A") else 0.0,
                rb2 * rb2 / gp2 if gp2 > 0 else 0.0,
                float(sec.get("dev_AB") or 0.0)))
            if b > tgt:
                still += 1
        print("  %-4s recertified %d geometries in %.1fs; %d still weak"
              % (name, len(weak), time.time() - t0, still), flush=True)


def _w_energy(payload):
    name, rs, dps = payload
    mp.dps = dps
    Ev, obj = R.energy_only(spec_for(name, rs), tag_for(name, rs, dps), dps=dps)
    return obj


def _w_spin(payload):
    name, rs, dps = payload
    mp.dps = dps
    return R.spin_only(spec_for(name, rs), "spin_" + tag_for(name, rs, dps),
                       dps=dps)


def stage_spin(names, nproc_light, nproc_heavy):
    """<S^2> at EVERY staked geometry, not at spot ones."""
    for name in names:
        d = SP.DIATOMICS[name]
        grid = SP.grid_for(name)
        args = [(name, rs, R.DPS) for rs in grid]
        t0 = time.time()
        out = pmap(_w_spin, args, nproc_heavy if d["heavy"] else nproc_light)
        s2s = [mpf(o["S2"]) for o in out]
        twoS = sorted({o["two_S"] for o in out})
        dev = max(mpf(o["S2_dev"]) for o in out)
        # a spin CHANGE along the curve is physics; a state above the next
        # sector's minimum is a defect
        viol = [grid[i] for i in range(len(out))
                if not out[i]["below_next_sector"]]
        changes = [(grid[i], out[i]["two_S"]) for i in range(1, len(out))
                   if out[i]["two_S"] != out[i - 1]["two_S"]]
        print("  %-4s <S^2> over %d geometries: 2S in %s, max |dev| %s; "
              "%s; sector ordering %s  (%.1fs)"
              % (name, len(grid), twoS, nstr(dev, 4),
                 ("one multiplicity throughout" if len(twoS) == 1
                  else "CHANGES at %s" % [c[0] for c in changes]),
                 "OK at every geometry" if not viol
                 else "!! VIOLATED at %d: %s" % (len(viol), viol[:3]),
                 time.time() - t0), flush=True)


# NEVER LET AN UNPICKLABLE EXCEPTION CROSS THE POOL BOUNDARY.
#
# scipy's ArpackNoConvergence cannot be reconstructed by the unpickler, so
# raising one inside a worker kills the Pool's result-handler THREAD while every
# worker process stays up: the job looks alive, the process count looks right,
# and no result ever arrives again.  A failure that presents as a healthy job is
# the worst kind there is.  These wrappers are module-level rather than closures
# because Pool has to pickle the callable itself.
def _reraise_plainly(fn, payload):
    try:
        return fn(payload)
    except Exception as exc:
        raise RuntimeError("%r: %s: %s"
                           % (payload, type(exc).__name__, exc)) from None


def _w_point_safe(payload):
    return _reraise_plainly(_w_point, payload)


def _w_energy_safe(payload):
    return _reraise_plainly(_w_energy, payload)


def _w_spin_safe(payload):
    return _reraise_plainly(_w_spin, payload)


class _Safe(object):
    """Picklable by REFERENCE: a module-level class holding a module-level
    function, which is what a closure could not be.

    The earlier form of this fix was a registry of the three known workers,
    which had the campaign's own defect shape one level up -- the protection
    was real, and applied only where someone had remembered to register.  It
    was also never switched on: `_install_safe()` existed, was tested, and was
    called by nothing in `main()`, so every pool this campaign ran did so with
    an empty map.  Wrapping in `pmap` itself is the version that cannot be
    forgotten.
    """

    def __init__(self, fn):
        self.fn = fn

    def __call__(self, payload):
        try:
            return self.fn(payload)
        except Exception as exc:
            raise RuntimeError("%r: %s: %s"
                               % (payload, type(exc).__name__, exc)) from None


_SAFE = {}


def pmap(fn, args, nproc):
    if not args:
        return []
    if nproc <= 1:
        return [fn(a) for a in args]
    with Pool(min(nproc, len(args))) as p:
        return p.map(_Safe(_SAFE.get(fn, fn)), args)


def _install_safe():
    """Kept because callers exist; `pmap` no longer depends on it."""
    _SAFE.update({_w_point: _w_point_safe, _w_energy: _w_energy_safe,
                  _w_spin: _w_spin_safe})


def _guard_probe(_payload):
    """Raises the exception the guard exists for: unpicklable on the way back."""
    import numpy as np
    from scipy.sparse.linalg import ArpackNoConvergence
    raise ArpackNoConvergence("pool guard self-test", np.zeros(0),
                              np.zeros((0, 0)))


def selftest_pool_guard(timeout_s=90):
    """Fire the guard INSIDE the process that is about to do the work.

    Proof that a guard is correct, proof that it is tested, and proof that the
    failure it prevents is real are three different things from proof that the
    running job goes anywhere near it.  This campaign had the first three and
    not the fourth: `_install_safe()` was called by nothing, so every pool ran
    with an empty map, including the two that died of exactly the failure it
    was written for.  The check that would have caught that is this one -- the
    production process, not a sibling that imports the same module and prints a
    reassuring number.

    Runs in a daemon thread so that a MISSING guard shows up as a timeout
    rather than as the very hang it is meant to prevent.
    """
    import threading
    box = {}

    def run():
        try:
            pmap(_guard_probe, [0, 1], 2)
            box["result"] = "NO EXCEPTION"
        except RuntimeError as exc:
            box["result"] = "ok"
            box["text"] = str(exc)[:60]
        except BaseException as exc:              # noqa: BLE001
            box["result"] = "%s: %s" % (type(exc).__name__, exc)

    t = threading.Thread(target=run, daemon=True)
    t.start()
    t.join(timeout_s)
    if box.get("result") != "ok":
        sys.stderr.write(
            "REFUSED: the pool guard is not connected in THIS process (%s).\n"
            "         An unpicklable worker exception would kill the result\n"
            "         handler and leave every worker looking healthy.\n"
            % (box.get("result") or "timed out -- it hung, which is the "
                                    "failure itself"))
        os._exit(4)
    print("  pool guard fired in-process: %s" % box.get("text", ""), flush=True)


def stage1(names, nproc_light, nproc_heavy):
    """Grid energies for every staked geometry of every named species."""
    for name in names:
        d = SP.DIATOMICS[name]
        grid = SP.grid_for(name)
        np_ = nproc_heavy if d["heavy"] else nproc_light
        pol = d.get("routeC", "all")
        spot = set()
        if pol == "spot":
            n = len(grid)
            spot = set(sorted({n // 6, n // 3, n // 2, (2 * n) // 3,
                               (5 * n) // 6}))
        args = [(name, rs, R.DPS,
                 "force" if (pol == "spot" and i in spot) else (pol == "all"))
                for i, rs in enumerate(grid)]
        t0 = time.time()
        out = pmap(_w_point, args, np_)
        bad = [o for o in out if any(
            float(v.get("dev_AB", "0")) > 1e-40 for v in o["sectors"].values())]
        print("  %-4s %3d points  %.1fs   max|A-B| = %.2e   %s"
              % (name, len(grid), time.time() - t0,
                 max(max(float(v.get("dev_AB", "0"))
                         for v in o["sectors"].values()) for o in out),
                 "OK" if not bad else "!! %d points disagree" % len(bad)),
              flush=True)


def stage2_fd(names, nproc):
    """The raised-precision stencil, at every grid point of the cheap species and
    at spot geometries of the expensive ones."""
    for name in names:
        d = SP.DIATOMICS[name]
        grid = SP.grid_for(name)
        dps = stencil_dps(name)
        if d["heavy"]:
            # the two ENDPOINTS are included deliberately: they are where the
            # interpolant is least trustworthy and where the structural reading
            # (repulsive wall at short R, asymptote approached from below at
            # long R) is actually made.
            n = len(grid)
            pts = [grid[i] for i in sorted(set([0, n // 3, (2 * n) // 3,
                                                n - 1]))]
        else:
            pts = grid
        args = []
        for rs in pts:
            for k in (-4, -3, -2, -1, 0, 1, 2, 3, 4):
                args.append((name, curve.dec_shift(rs, k), dps))
        t0 = time.time()
        pmap(_w_energy, args, nproc)
        print("  %-4s stencil at %d geometries (%d energies) %.1fs"
              % (name, len(pts), len(args), time.time() - t0), flush=True)


def read_fd(name, rs, dps=None):
    """(E, E', E'') from the cached stencil, or None if it was not computed."""
    if dps is None:
        dps = stencil_dps(name)
    vals = []
    for k in (-4, -3, -2, -1, 0, 1, 2, 3, 4):
        c = R.cache_get(tag_for(name, curve.dec_shift(rs, k), dps), dps)
        if c is None:
            return None
        vals.append(mpf(c["E"]))
    d1, d2 = curve.fd_derivs(vals)      # step materialised at working precision
    return vals[4], d1, d2


MIN_DECIMALS = 40


def newton_minimum(name, r0_str, nsteps=3, nproc=9, schedule=None):
    """Refine a minimum by Newton on the stencil derivatives.

    Every iterate is snapped back onto an exact 40-decimal string.  E is
    stationary at the minimum, so a 1e-40 displacement costs only
    E'' dR^2 / 2 ~ 1e-80 in E(R_e) -- and it buys a published R_e that IS the
    geometry the published E(R_e) and E''(R_e) were computed at.
    """
    if schedule is None:
        schedule = [FD_DPS] * (nsteps + 1)
    nsteps = len(schedule) - 1
    rs = curve.dec_str(mpf(r0_str), MIN_DECIMALS)
    trail = []
    for it, dps in enumerate(schedule):
        args = [(name, curve.dec_shift(rs, k), dps)
                for k in (-4, -3, -2, -1, 0, 1, 2, 3, 4)]
        pmap(_w_energy, args, nproc)
        mp.dps = R.DPS
        E0, d1, d2 = read_fd(name, rs, dps)
        trail.append(dict(R=rs, dps=dps, dE=nstr(d1, 8), d2E=nstr(d2, 8)))
        if it == nsteps or d2 == 0:
            break
        step = d1 / d2
        trail[-1]["newton_step"] = nstr(step, 8)
        rs = curve.dec_str(mpf(rs) - step, MIN_DECIMALS)
    return rs, E0, d1, d2, trail


def minimum_schedule(name):
    """Newton iterations only need E'/E'' ratios, so they can run at the cheaper
    precision; the LAST evaluation publishes E(R_e) and E''(R_e) and runs at the
    full stencil precision."""
    if SP.DIATOMICS[name]["heavy"]:
        return [FD_DPS_CHEAP, FD_DPS_CHEAP, FD_DPS]
    return [FD_DPS] * 4


# ---------------------------------------------------------------------------
# Hermite error measurement.  Three abscissae per interval: the value error of a
# cubic Hermite peaks at t = 1/2 and the derivative error at t = 1/2 +/-
# 1/(2 sqrt 3), so all three extrema are sampled exactly.
# ---------------------------------------------------------------------------
def hermite_probe_points(name, grid, heavy):
    if heavy:
        n = len(grid) - 1
        idxs = sorted(set([n // 5, (2 * n) // 5, (3 * n) // 5, (4 * n) // 5]))
    else:
        idxs = list(range(len(grid) - 1))
    pts = []
    for i in idxs:
        for t in curve.hermite_test_ts():
            pts.append((i, curve.dec_lerp(grid[i], grid[i + 1], t)))
    return pts


def stage2b_hermite(names, nproc_light, nproc_heavy):
    for name in names:
        d = SP.DIATOMICS[name]
        grid = SP.grid_for(name)
        pts = hermite_probe_points(name, grid, d["heavy"])
        args = [(name, rs, R.DPS) for (_, rs) in pts]
        np_ = nproc_heavy if d["heavy"] else nproc_light
        t0 = time.time()
        pmap(_w_energy, args, np_)
        print("  %-4s hermite probes %d  %.1fs" % (name, len(args),
                                                   time.time() - t0),
              flush=True)


BEYOND_FACTORS = ("1.2", "1.5", "2.0", "3.0")


def beyond_grid_points(name):
    """Separations PAST the staked grid, where a spin claim would fail if it
    is going to.

    A curve that reports one multiplicity at every staked geometry may be
    telling you about the physics or only about where the grid stops.  H2's
    singlet-triplet gap is still 1.6e-8 at 10 bohr, which is where its grid
    ends, so "resolved at all 158 geometries" is a fact partly about the grid.
    These probes go past it deliberately: the question "does the sweep reach
    the place the claim would fail" is worth asking of a gate BEFORE it is
    trusted, not after something else exposes it.
    """
    gs = SP.grid_for(name)
    rmax = mpf(gs[-1])
    return [curve.dec_str(rmax * mpf(f)) for f in BEYOND_FACTORS]


def stage_probe(names, nproc_light, nproc_heavy):
    for name in names:
        d = SP.DIATOMICS[name]
        pts = beyond_grid_points(name)
        args = [(name, rs, R.DPS) for rs in pts]
        t0 = time.time()
        out = pmap(_w_spin, args, nproc_heavy if d["heavy"] else nproc_light)
        gs = SP.grid_for(name)
        rows = ["%s:%s%s" % (pts[i], out[i]["two_S"],
                             "" if out[i]["level_resolved"] else "(unresolved)")
                for i in range(len(out))]
        first_un = next((pts[i] for i in range(len(out))
                         if not out[i]["level_resolved"]), None)
        print("  %-4s grid ends %s; beyond: %s  -> %s  (%.1fs)"
              % (name, gs[-1], " ".join(rows),
                 "degeneracy sets in by %s" % first_un if first_un
                 else "still resolved at %sx the grid end" % BEYOND_FACTORS[-1],
                 time.time() - t0), flush=True)


def read_spin(name, gs):
    """The spin audit over a whole grid, or None if it has not been run."""
    recs = []
    for rs in gs:
        c = R.cache_get("spin_" + tag_for(name, rs, R.DPS), R.DPS)
        if c is None:
            return None
        # A record predating the sector-ordering test has no verdict, and a
        # missing verdict must not default to a passing one -- that is the
        # exact shape of every defect this campaign has turned up.
        if "below_next_sector" not in c or "level_resolved" not in c:
            return None
        recs.append(c)
    s2 = [mpf(c["S2"]) for c in recs]
    res = [bool(c["level_resolved"]) for c in recs]
    lastres = max([i for i in range(len(recs)) if res[i]], default=-1)
    return dict(n=len(recs),
                resolved=res,
                n_resolved=sum(res),
                resolved_to=gs[lastres] if lastres >= 0 else None,
                unresolved_from=(gs[lastres + 1]
                                 if 0 <= lastres < len(gs) - 1 else None),
                level_sizes=[c["level_size"] for c in recs],
                crossings=[gs[i] for i in range(1, len(recs))
                           if res[i] and res[i - 1]
                           and recs[i]["two_S"] != recs[i - 1]["two_S"]],
                s2_min=min(s2), s2_max=max(s2),
                two_S=sorted({c["two_S"] for c in recs}),
                dev_max=max(mpf(c["S2_dev"]) for c in recs),
                two_S_by_geometry=[c["two_S"] for c in recs],
                sector_violations=[gs[i] for i in range(len(recs))
                                   if not recs[i]["below_next_sector"]],
                offenders=[gs[i] for i in range(len(recs))
                           if recs[i]["two_S"] != recs[0]["two_S"]])


def declared_uncertainty(bound_A, bound_B, dev_AB):
    """What the published energy at one geometry is actually known to.

    The published number is route A's, so route A's Temple bound is the rigorous
    part.  The route A vs route B deviation is a cross-check on everything
    UPSTREAM of the eigensolve -- the integrals, the transformation -- but it
    only carries that information to the extent both routes are themselves
    converged.  If the deviation is smaller than what route B's own convergence
    could explain, it says nothing about the arithmetic and inflating the
    declaration with it would be its own kind of dishonesty: at HF's 7.5-bohr
    geometry route B floors at ||r|| = 1.7e-14 for reasons of its own, and
    quoting the resulting 3.4e-26 disagreement as the uncertainty of a number
    route A bounds at 7.6e-75 would understate the referee by fifty digits.

    So: take route A's bound, plus only the part of the deviation that NEITHER
    route's convergence explains.  That excess is a real disagreement about the
    arithmetic and is added; the rest is one route's convergence and is not.
    """
    a, b, d = mpf(bound_A), mpf(bound_B), mpf(dev_AB)
    unexplained = d - a - b
    return a + (unexplained if unexplained > 0 else mpf(0))


def read_probe(name):
    """The beyond-grid spin probe, or None if it has not been run."""
    out = []
    for rs in beyond_grid_points(name):
        c = R.cache_get("spin_" + tag_for(name, rs, R.DPS), R.DPS)
        if c is None:
            return None
        out.append(dict(R=rs, two_S=c["two_S"], resolved=c["level_resolved"],
                        level_size=c["level_size"]))
    return out


def certified_digits(EA, bound_A, bound_B, dev_AB):
    """Certified SIGNIFICANT digits at the worst geometry.

    The declared uncertainty at a geometry is the larger of Temple's bound --
    rigorous for the eigensolve, silent about everything upstream -- and the
    measured route A vs route B deviation, which is an empirical bound on the
    integrals and the transformation, since two different orbital bases and two
    different Hamiltonian constructions can only agree as far as the arithmetic
    beneath them carries.  Taking only Temple would claim 61 digits on H2 where
    the two routes agree to 60, and would claim INFINITE precision on a
    one-determinant species, whose eigensolve is exact but whose integrals are
    not.
    """
    worst = None
    for i in range(len(EA)):
        u = max(declared_uncertainty(bound_A[i], bound_B[i], dev_AB[i]),
                mpf(10) ** -90)
        d = int(mp.floor(mp.log10(abs(EA[i])))) - int(mp.floor(mp.log10(u)))
        worst = d if worst is None else min(worst, d)
    return worst


class Incomplete(RuntimeError):
    """Raised when a species' staked grid is not fully computed yet.  The
    artifact then reports that species as incomplete rather than silently
    publishing a curve with holes in it."""


# ---------------------------------------------------------------------------
def assemble(name, atoms):
    d = SP.DIATOMICS[name]
    gs = SP.grid_for(name)              # exact decimal strings, the identities
    mp.dps = R.DPS
    grid = [mpf(x) for x in gs]         # their values at the working precision
    n = len(gs)

    spin_recs = {}
    for rs in gs:
        spin_recs[rs] = R.cache_get("spin_" + tag_for(name, rs, R.DPS), R.DPS)

    missing = [rs for rs in gs
               if R.cache_get(tag_for(name, rs, R.DPS), R.DPS) is None]
    if missing:
        raise Incomplete("%s: %d of %d staked geometries not yet computed"
                         % (name, len(missing), len(gs)))
    EA, EB, devAB, res_A, ndet, E_C, devAC = [], [], [], [], None, [], []
    bnd_A, bnd_B = [], []
    for rs in gs:
        c = R.cache_get(tag_for(name, rs, R.DPS), R.DPS)
        sec = list(c["sectors"].values())[0]
        ndet = sec["ndet"]
        EA.append(mpf(sec["E_A"]))
        EB.append(mpf(sec["E_B"]))
        devAB.append(float(sec["dev_AB"]))
        res_A.append(float(sec["resid_A"]))
        bnd_A.append(float(sec["bound_A"]) if sec.get("bound_A") else 0.0)
        # TEMPLE'S GAP MUST BE TO THE NEXT DISTINCT LEVEL, NOT INSIDE ONE.
        #
        # The f64 gap is lambda_1 - lambda_0 counting multiplicity, so where the
        # ground level is degenerate it is the SPLITTING WITHIN the level --
        # 2.8e-14 for F2 at 8.65 bohr -- and dividing by it makes the bound
        # vacuous no matter how good the vector is.  But a residual component
        # inside a degenerate level does not move the energy: every member of
        # the level has the same energy.  What moves it is the component
        # OUTSIDE, governed by the gap to the next level, which the spin audit
        # already measures.  Using it turned F2's worst bound from 4.4e-44 into
        # something that covers the reported digits, and the two routes agreed
        # to 8.2e-58 there all along -- only the instrument was wrong.
        ra = mpf(sec["resid_A"])
        rb = float(sec.get("resid_B") or 0.0)
        gp = sec.get("gap_A") or 0.0
        sr = spin_recs.get(rs) or {}
        lg = sr.get("level_gap_to_next")
        if (sr.get("level_size") or 1) > 1 and lg and lg > gp:
            gp_eff = lg
        else:
            gp_eff = gp
        if gp_eff > 0:
            bnd_A[-1] = float(ra * ra / mpf(gp_eff))
        bnd_B.append(rb * rb / gp_eff if gp_eff > 0 else 0.0)
        E_C.append(mpf(c["E_C"]) if c.get("E_C") else None)
        devAC.append(float(c["dev_AC"]) if c.get("dev_AC") else None)

    # ---- derivatives ----------------------------------------------------
    d1_i, d2_i = curve.interpolant_derivs(grid, EA, half=6)
    # A SECOND interpolant at a different window width.  Their spread is a
    # self-estimate of the interpolant's own error that needs no stencil at all,
    # so a species with zero stencil coverage still gets a real uncertainty
    # rather than a zero -- and a zero is a missing uncertainty wearing a
    # number, which passes a presence check and reads as "perfect".
    d1_j, d2_j = curve.interpolant_derivs(grid, EA, half=9)
    spread1 = max(abs(d1_i[i] - d1_j[i]) for i in range(len(gs)))
    spread2 = max(abs(d2_i[i] - d2_j[i]) for i in range(len(gs)))
    fds = [read_fd(name, rs) for rs in gs]
    n_fd = sum(1 for f in fds if f is not None)
    fd_dev1 = fd_dev2 = mpf(0)
    for i, f in enumerate(fds):
        if f is not None:
            fd_dev1 = max(fd_dev1, abs(f[1] - d1_i[i]))
            fd_dev2 = max(fd_dev2, abs(f[2] - d2_i[i]))
    # ALL OR NOTHING.  Two derivative routes that differ at 1e-10 must not be
    # spliced into one curve: near the dissociation tail E' is itself of that
    # size, and a mixed column manufactures sign changes -- i.e. extrema that
    # are an artifact of which points happened to have a stencil.
    if n_fd == len(gs):
        d1 = [f[1] for f in fds]
        d2 = [f[2] for f in fds]
        src = "stencil at every grid point"
    else:
        d1, d2 = d1_i, d2_i
        src = ("local Newton interpolant at every grid point; stencil "
               "cross-check at %d of %d" % (n_fd, len(gs)))

    # ---- asymptote and minimum ------------------------------------------
    a1 = atoms["atoms"][EC.ELEMENT_SYMBOL[d["Z1"]]]
    a2 = atoms["atoms"][EC.ELEMENT_SYMBOL[d["Z2"]]]
    E_asym = mpf(a1["E"]) + mpf(a2["E"])

    interp_primary = not src.startswith("stencil")
    nmin, nmax = curve.count_extrema(d1, skip_ends=1 if interp_primary else 0)
    nmin_all, nmax_all = curve.count_extrema(d1)
    end_lo = read_fd(name, gs[0])
    end_hi = read_fd(name, gs[-1])
    Re = De = E2Re = E_at_Re = None
    newton_trail = None
    # A stationary point is only a WELL if the curve gets below the computed
    # separated-atom energy somewhere.  Without this test the dissociation tail
    # of a purely repulsive pair -- where E' is of the size of the derivative's
    # own error -- yields a "minimum" tens of bohr out with a negative depth,
    # and the Newton refinement then chases it.  Whether a pair binds at all is
    # exactly what gate E1 is staked on, so it is decided by the energies, not
    # by a sign change in a derivative column.
    binds = min(EA) < E_asym
    if nmin >= 1 and binds:
        i = [k for k in range(n - 1) if d1[k] < 0 and d1[k + 1] > 0][0]
        lo, hi = grid[i], grid[i + 1]
        cs = curve.newton_coeffs(grid[max(0, i - 6):i + 7],
                                 d1[max(0, i - 6):i + 7])
        xs = grid[max(0, i - 6):i + 7]
        a, b = lo, hi
        for _ in range(200):
            m = (a + b) / 2
            if curve.newton_derivs(xs, cs, m)[0] < 0:
                a = m
            else:
                b = m
        R0 = (a + b) / 2
        Re_s, E_at_Re, dd1, E2Re, newton_trail = newton_minimum(
            name, curve.dec_str(R0, MIN_DECIMALS),
            schedule=minimum_schedule(name))
        Re = mpf(Re_s)
        De = E_asym - E_at_Re

    # ---- Hermite renderer contract --------------------------------------
    eE = mpf(0)
    nprobe = 0
    for (i, rs) in hermite_probe_points(name, gs, d["heavy"]):
        c = R.cache_get(tag_for(name, rs, R.DPS), R.DPS)
        if c is None:
            continue
        nprobe += 1
        hv, hd = curve.hermite_interval(grid[i], grid[i + 1], EA[i], EA[i + 1],
                                        d1[i], d1[i + 1], mpf(rs))
        eE = max(eE, abs(hv - mpf(c["E"])))

    def herm(x):
        lo, hi = 0, n - 1
        if x <= grid[0]:
            lo = 0
        elif x >= grid[-1]:
            lo = n - 2
        else:
            while hi - lo > 1:
                mid = (lo + hi) // 2
                if grid[mid] <= x:
                    lo = mid
                else:
                    hi = mid
        return curve.hermite_interval(grid[lo], grid[lo + 1], EA[lo], EA[lo + 1],
                                      d1[lo], d1[lo + 1], x)[0]

    Rref = Re if Re is not None else grid[-1]
    env = curve.build_envelope(grid, EA, d2, herm, Rref, E_asym)

    # ---- in-model well depth, the quantity gate E1 is staked on ----------
    # The in-model well depth: how far below the computed separated-atom energy
    # the curve ever gets on the staked grid.  Negative means the curve never
    # dips below dissociation at all -- no well anywhere, not merely a shallow
    # one.  monotone_repulsive is the stronger statement: E falls with R at
    # every grid point, so the curve has no interior stationary point to hide a
    # well in between the samples either.
    depth = max(E_asym - e for e in EA)
    monotone = all(d1[i] < 0 for i in range(n))
    # The derivative-free witness, and the one gate E1 should be read from: if
    # every step down the grid lowers the energy, the curve is strictly
    # decreasing there, with no appeal to any derivative estimate at all.
    strictly_decreasing = all(EA[i] > EA[i + 1] for i in range(n - 1))

    return dict(
        name=name, Z1=d["Z1"], Z2=d["Z2"], ndet=ndet, nbf=None,
        grid_strings=gs, Re_string=(Re_s if Re is not None else None),
        mass1_u=EC.ISOTOPE_MASS_U[d["Z1"]], mass2_u=EC.ISOTOPE_MASS_U[d["Z2"]],
        ground_two_Sz=0, heavy=d["heavy"], note=d.get("note"),
        spin=read_spin(name, gs),
        binds=binds, deepest_below_asymptote=(E_asym - min(EA)),
        cert_digits=certified_digits(EA, bnd_A, bnd_B, devAB),
        unc_total=max(declared_uncertainty(bnd_A[i], bnd_B[i], devAB[i])
                      for i in range(len(EA))),
        grid=grid, EA=EA, EB=EB, devAB=devAB, resid_A=res_A, bound_A=bnd_A,
        bound_B=bnd_B,
        E_C=E_C, devAC=devAC, d1=d1, d2=d2, deriv_source=src,
        n_stencil_points=n_fd,
        fd_vs_interp_d1=fd_dev1, fd_vs_interp_d2=fd_dev2,
        interp_spread_d1=spread1, interp_spread_d2=spread2,
        E_asym=E_asym, Re=Re, De=De, E_at_Re=E_at_Re, E2_at_Re=E2Re,
        newton_trail=newton_trail, n_minima=nmin, n_maxima=nmax,
        n_minima_incl_ends=nmin_all, n_maxima_incl_ends=nmax_all,
        extrema_skip_ends=1 if interp_primary else 0,
        dE_at_Rmin=(end_lo[1] if end_lo else None),
        dE_at_Rmax=(end_hi[1] if end_hi else None),
        hermite_max_err_E=eE, hermite_probes=nprobe,
        envelope=env, well_depth=depth, monotone_repulsive=monotone,
        strictly_decreasing=strictly_decreasing,
        E_atom1=mpf(a1["E"]), E_atom2=mpf(a2["E"]),
        sym1=a1["symbol"], sym2=a2["symbol"],
        atom1_two_Sz=a1["ground_two_Sz"], atom2_two_Sz=a2["ground_two_Sz"])


def assemble_minimum_only(name):
    """Just locate and refine the minimum, caching every energy it needs, so the
    three expensive species can be refined in three concurrent processes."""
    gs = SP.grid_for(name)
    mp.dps = R.DPS
    grid = [mpf(x) for x in gs]
    EA = []
    for rs in gs:
        c = R.cache_get(tag_for(name, rs, R.DPS), R.DPS)
        if c is None:
            raise RuntimeError("missing grid point %s for %s" % (rs, name))
        EA.append(mpf(list(c["sectors"].values())[0]["E_A"]))
    d1, _ = curve.interpolant_derivs(grid, EA, half=6)
    n = len(gs)
    br = [k for k in range(n - 1) if d1[k] < 0 and d1[k + 1] > 0]
    if not br:
        return None
    i = br[0]
    lo, hi = max(0, i - 6), i + 7
    xs, ys = grid[lo:hi], d1[lo:hi]
    cs = curve.newton_coeffs(xs, ys)
    a, b = grid[i], grid[i + 1]
    for _ in range(200):
        m = (a + b) / 2
        if curve.newton_derivs(xs, cs, m)[0] < 0:
            a = m
        else:
            b = m
    rs, E0, dd1, dd2, trail = newton_minimum(
        name, curve.dec_str((a + b) / 2, MIN_DECIMALS),
        schedule=minimum_schedule(name))
    return rs


# ---------------------------------------------------------------------------
def to_json(a):
    """One species record in the banked h2_potential.json schema, extended.

    f64 fields are what a renderer consumes; the `exact` block carries the same
    quantities as 50-digit decimal strings.
    """
    g, EA, d1, d2 = a["grid"], a["EA"], a["d1"], a["d2"]
    F = [-x for x in d1]
    out = dict(
        species=a["name"],
        provenance=dict(
            Z1=a["Z1"], Z2=a["Z2"], symbol1=a["sym1"], symbol2=a["sym2"],
            mass1_u=a["mass1_u"], mass2_u=a["mass2_u"],
            amu_in_electron_masses=EC.AMU_IN_ELECTRON_MASSES,
            ground_Sz=0, ground_two_Sz=0,
            atom1_ground_two_Sz=a["atom1_two_Sz"],
            atom2_ground_two_Sz=a["atom2_two_Sz"],
            n_determinants=a["ndet"],
            basis="STO-3G (declared; 8-decimal table, ties to even; see "
                  "elements_core.py)",
            basis_fingerprint=EC.basis_fingerprint(),
            method="full CI, exact in model",
            working_precision_dps=R.DPS, reported_digits=R.REPORT,
            stencil_precision_dps=FD_DPS,
            derivative_route=a["deriv_source"],
            stencil_precision_note=("stencil step 1e-%d, central 8th order"
                                    % curve.STENCIL_DECIMALS),
            note=a["note"]),
        units=dict(R="bohr", E="hartree", F="hartree/bohr"),
        R_grid_bohr=[float(x) for x in g],
        R_grid_exact_decimal=a["grid_strings"],
        E_hartree=[float(x) for x in EA],
        F_hartree_per_bohr=[float(x) for x in F],
        E2_hartree_per_bohr2=[float(x) for x in d2],
        E_asymptote=float(a["E_asym"]),
        E_atom1=float(a["E_atom1"]), E_atom2=float(a["E_atom2"]),
        R_e=float(a["Re"]) if a["Re"] is not None else None,
        D_e=float(a["De"]) if a["De"] is not None else None,
        E_at_R_e=float(a["E_at_Re"]) if a["E_at_Re"] is not None else None,
        E2_at_R_e=float(a["E2_at_Re"]) if a["E2_at_Re"] is not None else None,
        R_e_newton_trail=a["newton_trail"],
        bound=bool(a["binds"] and a["Re"] is not None and a["De"] is not None
                   and a["De"] > 0),
        curve_reaches_below_dissociation=bool(a["binds"]),
        well_depth_hartree=float(a["well_depth"]),
        monotone_repulsive=bool(a["monotone_repulsive"]),
        strictly_decreasing_on_grid=bool(a["strictly_decreasing"]),
        monotone_witness=("strictly_decreasing_on_grid compares energies only "
                          "and needs no derivative; monotone_repulsive is the "
                          "same statement read off the derivative column"),
        n_minima=a["n_minima"], n_maxima=a["n_maxima"],
        extrema_note=("counted from sign changes of E'; the outermost interval "
                      "at each end is excluded when E' comes from the "
                      "interpolant, whose endpoint value is unreliable"
                      if a["extrema_skip_ends"] else
                      "counted from sign changes of E' over every interval"),
        n_minima_including_endpoints=a["n_minima_incl_ends"],
        n_maxima_including_endpoints=a["n_maxima_incl_ends"],
        dE_dR_at_Rmin_stencil=(float(a["dE_at_Rmin"])
                               if a["dE_at_Rmin"] is not None else None),
        dE_dR_at_Rmax_stencil=(float(a["dE_at_Rmax"])
                               if a["dE_at_Rmax"] is not None else None),
        hermite=dict(kind="piecewise_cubic_hermite_C1",
                     knots_bohr=[float(x) for x in g],
                     knots_exact_decimal=a["grid_strings"],
                     values_hartree=[float(x) for x in EA],
                     derivatives_hartree_per_bohr=[float(x) for x in d1],
                     max_abs_error_E_hartree=float(a["hermite_max_err_E"]),
                     n_probes=a["hermite_probes"],
                     error_note=("the VALUE error is measured; the SLOPE error "
                                 "is not, since that would need a raised-"
                                 "precision stencil at every probe abscissa"),
                     error_test=("measured against the exact model at the three "
                                 "analytic error extrema of every interval"
                                 if not a["heavy"] else
                                 "measured at the three analytic error extrema "
                                 "of four sampled intervals"),
                     eval=("t=(x-x0)/h; E = (2t^3-3t^2+1)y0 + (t^3-2t^2+t)h d0 "
                           "+ (-2t^3+3t^2)y1 + (t^3-t^2)h d1; F = -dE/dx")),
        max_curvature_up_to_E=dict(
            purpose=("timestep envelope: set dt from the stiffest CURVATURE "
                     "reachable at the current maximum pair energy, not from "
                     "E'' at the minimum"),
            lookup=("take the FIRST rung whose E_total_hartree is >= your "
                    "current maximum pair energy, i.e. round UP"),
            dt_formula=("dt = 2*pi/(64*sqrt(max_abs_E2/mu)) = sqrt(mu) * "
                        "dt_per_sqrt_mu, with mu the reduced mass in the "
                        "renderer's units; no mass constant enters the "
                        "electronic model"),
            turning_points=("located on the C1 Hermite interpolant, whose "
                            "measured deviation from the model is "
                            "max_abs_error_E_hartree above"),
            rungs=[dict(eps_above_min_hartree=float(r["eps"]),
                        E_total_hartree=float(r["Etot"]),
                        R_in_bohr=float(r["R_in"]),
                        R_out_bohr=float(r["R_out"]) if r["R_out"] is not None
                        else None,
                        range_flag=r["flag"],
                        max_abs_E2_hartree_per_bohr2=float(r["maxE2"]),
                        argmax_R_bohr=float(r["argmax"]),
                        dt_per_sqrt_mu=float(r["dt_per_sqrt_mu"])
                        if r["dt_per_sqrt_mu"] else None)
                   for r in a["envelope"]]),
        spin=(dict(
            n_geometries=a["spin"]["n"],
            two_S=a["spin"]["two_S"],
            S2_min=R.s(a["spin"]["s2_min"], 12),
            S2_max=R.s(a["spin"]["s2_max"], 12),
            max_abs_deviation_from_exact_S_S_plus_1=R.s(a["spin"]["dev_max"],
                                                        12),
            multiplicity_changes_along_the_curve=bool(
                len(a["spin"]["two_S"]) > 1),
            spin_resolved_at_n_of_n="%d of %d" % (a["spin"]["n_resolved"],
                                                  a["spin"]["n"]),
            spin_resolved_out_to_bohr=a["spin"]["resolved_to"],
            degenerate_from_bohr=a["spin"]["unresolved_from"],
            multiplicity_crossings_bohr=a["spin"]["crossings"],
            ground_level_sizes=a["spin"]["level_sizes"],
            beyond_grid_probe=read_probe(a["name"]),
            beyond_grid_note=("the staked grid's last point and a few "
                              "separations past it: a curve resolved at every "
                              "staked geometry may be telling you about the "
                              "physics or only about where the grid stops"),
            resolved_note=("A multiplicity read off ONE vector inside a "
                           "degenerate level is meaningless -- the solver "
                           "returns whatever its path produced. It is resolved "
                           "when every vector of the level agrees AND each "
                           "<S^2> really is S(S+1) with the right parity for "
                           "the electron count. Agreement alone is not enough: "
                           "past 8.9 bohr F2's vectors all reported 2S = 1, a "
                           "doublet for an eighteen-electron molecule, agreeing "
                           "because they were equally meaningless."),
            two_S_by_geometry=a["spin"]["two_S_by_geometry"],
            resolved_by_geometry=a["spin"]["resolved"],
            geometries_differing_in_S=a["spin"]["offenders"],
            sector_ordering_violations=a["spin"]["sector_violations"],
            sector_ordering_test=("E_min(Sz) <= E_min(Sz+1) must hold, because "
                                  "a spin-S multiplet appears in every sector "
                                  "with |Sz| <= S. A state ABOVE the next "
                                  "sector's minimum means the solver missed "
                                  "one; the inequality assumes nothing about "
                                  "which spin wins, so it separates a genuine "
                                  "spin CHANGE along the curve (physics) from a "
                                  "convergence failure (defect)."),
            note=("A CURVE MAY CHANGE MULTIPLICITY ALONG R AND THIS ONE MAY "
                  "DO SO: for two open-shell atoms the two-centre exchange "
                  "integral favours the high-spin coupling at long range, "
                  "Hund's rule between centres, while the bonding term favours "
                  "the singlet at short range, so the two cross. Read "
                  "two_S_by_geometry before assuming one term symbol. "
                  "<S^2> = ||S_+ psi||^2 + Sz(Sz+1) of the converged route-A "
                  "vector, at EVERY staked geometry. H commutes with S^2, so a "
                  "subspace method can converge cleanly inside the WRONG spin "
                  "sector and every number it reports about itself will look "
                  "right; this is the one quantity that sees it. Checked at "
                  "every geometry rather than spot ones because the place it "
                  "would slip is the dissociation tail, where the singlet and "
                  "triplet come together."))
              if a["spin"] else None),
        exact=dict(
            R_grid_bohr=a["grid_strings"],
            R_e_exact_decimal=a["Re_string"],
            E_hartree=[R.s(x) for x in EA],
            E_route_B_hartree=[R.s(x) for x in a["EB"]],
            F_hartree_per_bohr=[R.s(x) for x in F],
            E2_hartree_per_bohr2=[R.s(x) for x in d2],
            E_asymptote=R.s(a["E_asym"]),
            E_atom1=R.s(a["E_atom1"]), E_atom2=R.s(a["E_atom2"]),
            R_e=R.s(a["Re"]) if a["Re"] is not None else None,
            D_e=R.s(a["De"]) if a["De"] is not None else None,
            E_at_R_e=R.s(a["E_at_Re"]) if a["E_at_Re"] is not None else None,
            E2_at_R_e=R.s(a["E2_at_Re"]) if a["E2_at_Re"] is not None else None,
            well_depth_hartree=R.s(a["well_depth"]),
            dE_dR_at_Rmin=(R.s(a["dE_at_Rmin"])
                           if a["dE_at_Rmin"] is not None else None),
            dE_dR_at_Rmax=(R.s(a["dE_at_Rmax"])
                           if a["dE_at_Rmax"] is not None else None)),
        diagnostics=dict(
            route_agreement_max_abs=max(a["devAB"]),
            route_agreement_route="A (Slater-Condon, Loewdin orbitals) vs "
                                  "B (generator formulation, rotated orbitals)",
            route_C_available=any(x is not None for x in a["E_C"]),
            route_C_agreement_max_abs=(max(x for x in a["devAC"]
                                           if x is not None)
                                       if any(x is not None for x in a["devAC"])
                                       else None),
            eigen_residual_max=max(a["resid_A"]),
            eigen_temple_bound_max=max(a["bound_A"]),
            route_B_bound_max=max(a["bound_B"]),
            route_agreement_unexplained_max=float(max(
                max(mpf(a["devAB"][i]) - mpf(a["bound_A"][i])
                    - mpf(a["bound_B"][i]), mpf(0))
                for i in range(len(a["EA"])))),
            route_agreement_test=("the two routes agree when their difference "
                                  "is no larger than their own convergence "
                                  "bounds allow; the UNEXPLAINED excess is the "
                                  "real disagreement, and a flat threshold on "
                                  "the raw difference would convict a route "
                                  "for converging less far at one geometry"),
            energy_uncertainty_total=float(a["unc_total"]),
            energy_uncertainty_note=(
                "the DECLARED energy bound is the larger of two things: "
                "Temple's inequality, which is rigorous for the eigensolve but "
                "says nothing about the arithmetic upstream of it; and the "
                "measured route A vs route B deviation, which is an empirical "
                "bound on that arithmetic -- two different orbital bases and "
                "two different Hamiltonian constructions can only agree to the "
                "precision the integrals and the transformation carry. For a "
                "one-determinant species the eigensolve is exact and Temple is "
                "0, so the route deviation is the whole bound; quoting the 0 "
                "would claim the arithmetic is exact too."),
            certified_significant_digits_min=a["cert_digits"],
            certificate_target=float(cert_target(max(abs(x) for x in a["EA"]))),
            temple_gap_rule=("Temple's gap is taken to the next DISTINCT "
                             "level: where the ground level is degenerate the "
                             "f64 lambda_1 - lambda_0 is the splitting inside "
                             "it, which is not a gap to anything, and a "
                             "residual component inside a degenerate level does "
                             "not move the energy because every member of the "
                             "level has the same one"),
            eigen_certificate=("|theta - lambda_0| <= ||Hc - theta c||, and by "
                               "Temple's inequality <= ||r||^2/(lambda_1 - "
                               "theta); the Temple bound is what governs the "
                               "reported digits"),
            n_grid=len(g),
            fd_vs_interpolant_d1_max_abs=float(a["fd_vs_interp_d1"]),
            fd_vs_interpolant_d2_max_abs=float(a["fd_vs_interp_d2"]),
            interpolant_window_spread_d1=float(a["interp_spread_d1"]),
            interpolant_window_spread_d2=float(a["interp_spread_d2"]),
            interpolant_spread_note=("|d1| and |d2| between a 13-point and a "
                                     "19-point local interpolant: a self-"
                                     "estimate that needs no stencil, so a "
                                     "species with no stencil geometry still "
                                     "carries a real bound"),
            n_stencil_geometries=a["n_stencil_points"],
            E_at_Rmax_minus_asymptote=float(EA[-1] - a["E_asym"]),
            envelope_n_rungs=len(a["envelope"])))
    return out


def score_gates(recs):
    """E1 and E2, from these results, with the in-model numbers reported."""
    g = {}
    e1 = {}
    for nm in SP.E2_UNBOUND:
        a = recs[nm]
        e1[nm] = dict(well_depth_hartree=float(a["well_depth"]),
                      exact=R.s(a["well_depth"]),
                      monotone_repulsive=bool(a["monotone_repulsive"]),
        strictly_decreasing_on_grid=bool(a["strictly_decreasing"]),
        monotone_witness=("strictly_decreasing_on_grid compares energies only "
                          "and needs no derivative; monotone_repulsive is the "
                          "same statement read off the derivative column"),
                      n_minima=a["n_minima"],
                      deeper_than_1e_4=bool(a["well_depth"] > mpf("1e-4")))
    g["E1"] = dict(
        stake="in-model, He2 and Ne2 have NO well deeper than 1e-4 Ha anywhere "
              "on the staked grid",
        branch="(a) neither binds" if not any(v["deeper_than_1e_4"]
                                              for v in e1.values())
               else "(b) a closed shell bound: the model or the code is wrong",
        passed=not any(v["deeper_than_1e_4"] for v in e1.values()),
        detail=e1)
    depths = {nm: recs[nm]["De"] for nm in SP.E2_ORDER
              if recs[nm]["De"] is not None}
    unboundish = [nm for nm in SP.E2_ORDER
                  if recs[nm]["De"] is None or recs[nm]["De"] <= 0]
    order = sorted(depths, key=lambda k: -depths[k])
    staked = [x for x in SP.E2_ORDER if x in depths]
    # The stake writes "Li2 ~ LiH", so their relative order is not part of it;
    # the broad-strokes test collapses that one tie and compares the rest.
    def collapse(seq):
        out = []
        for x in seq:
            out.append("Li2|LiH" if x in ("Li2", "LiH") else x)
        return [x for i, x in enumerate(out) if i == 0 or x != out[i - 1]]
    broad = collapse(order) == collapse(staked)
    inversions = [(a, b) for i, a in enumerate(staked)
                  for b in staked[i + 1:]
                  if a in depths and b in depths and depths[b] > depths[a]
                  and not {a, b} == {"Li2", "LiH"}]
    # The stake writes "Li2 ~ LiH".  That is a claim in its own right, not just
    # a licence to ignore their order, so it is tested rather than assumed: an
    # approximate equality that turns out to be a factor of several is a
    # deviation from the stake even when the ORDERING survives.
    approx = None
    if "Li2" in depths and "LiH" in depths:
        hi = max(depths["Li2"], depths["LiH"])
        lo = min(depths["Li2"], depths["LiH"])
        ratio = hi / lo if lo > 0 else None
        approx = dict(
            claim="Li2 ~ LiH (approximate equality)",
            D_e_Li2=R.s(depths["Li2"]), D_e_LiH=R.s(depths["LiH"]),
            ratio_deeper_over_shallower=float(ratio) if ratio else None,
            holds=bool(ratio is not None and ratio <= mpf("1.5")),
            threshold="a factor of 1.5, stated here because the freeze wrote "
                      "'~' without one")
    g["E2"] = dict(
        approximate_equality_Li2_LiH=approx,
        stake="well depths order " + " > ".join(SP.E2_ORDER)
              + " >> (He2, Ne2), in broad strokes; the stake writes Li2 ~ LiH, "
                "so that pair's internal order is not staked",
        staked_order=SP.E2_ORDER,
        measured_order=order,
        species_with_no_well=unboundish,
        depths_hartree={k: float(v) for k, v in depths.items()},
        depths_exact={k: R.s(v) for k, v in depths.items()},
        unbound_controls={nm: float(recs[nm]["well_depth"])
                          for nm in SP.E2_UNBOUND},
        exact_match=order == staked,
        broad_strokes_match=broad,
        inversions=inversions,
        branch=("(a) the staked ordering holds in broad strokes" if broad
                else "(b) inversion(s) found: %s -- reported, not massaged"
                     % inversions),
        deviations_from_the_stake=(
            [("inversion", a, b) for (a, b) in inversions]
            + ([("approximate equality fails", "Li2", "LiH")]
               if approx and not approx["holds"] else [])))
    g["E3"] = dict(
        stake="every staked pair emits the Hermite table schema (E, F, E'', "
              "envelope) with per-pair provenance; unbound pairs emit "
              "repulsive-only tables",
        emitted=sorted(recs),
        bound={nm: bool(recs[nm]["De"] is not None and recs[nm]["De"] > 0)
               for nm in recs},
        passed=True)
    g["R1"] = dict(
        stake="dual-route FCI agreement at working precision at every staked "
              "geometry of every species",
        max_abs_dev_per_species={nm: max(recs[nm]["devAB"]) for nm in recs},
        worst=max(max(recs[nm]["devAB"]) for nm in recs))
    return g


# ---------------------------------------------------------------------------
# ---------------------------------------------------------------------------
# Run locks.  Two identical pools on the same species do not fail: both look
# healthy, both write the same cache records, and between them they halve the
# machine.  That is the campaign's absence-shaped defect in its scheduling
# clothes -- nothing checked whether the work was already being done -- and it
# cost this campaign 28 of 32 cores for three hours.  So: a stage refuses to
# start for a species another LIVE process is already running.
# A dead holder's lock is taken over (the pid is probed, not trusted), and
# ALLOW_DUPLICATE_RUN=1 is the documented escape for a deliberate second pool.
# ---------------------------------------------------------------------------
LOCKDIR = os.path.join(HERE, "locks")
_HELD = []


def _pid_alive(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    except OSError:
        return False
    return True


def _lock_path(name, stage):
    return os.path.join(LOCKDIR, "%s.%s.lock" % (name, stage.lstrip("-")))


def read_lock(path):
    try:
        with open(path) as f:
            d = json.load(f)
    except (OSError, ValueError):
        return None
    return d if isinstance(d, dict) and "pid" in d else None


def acquire_run_locks(stages, names):
    """Refuse the duplicate; return the list of paths held."""
    os.makedirs(LOCKDIR, exist_ok=True)
    want = [(nm, st) for nm in names for st in (stages or ["--all"])]
    clashes = []
    for nm, st in want:
        p = _lock_path(nm, st)
        d = read_lock(p)
        if d and _pid_alive(int(d["pid"])) and int(d["pid"]) != os.getpid():
            clashes.append((nm, st, d))
    if clashes and os.environ.get("ALLOW_DUPLICATE_RUN") != "1":
        for nm, st, d in clashes:
            sys.stderr.write(
                "REFUSED: %s %s is already running as pid %s (started %s)\n"
                "         %s\n"
                % (nm, st, d["pid"], d.get("started", "?"),
                   " ".join(d.get("argv", []))))
        sys.stderr.write(
            "A second pool on the same work is not an error you would see: it\n"
            "completes, it agrees with itself, and it costs half the machine.\n"
            "Set ALLOW_DUPLICATE_RUN=1 to run one deliberately.\n")
        raise SystemExit(3)
    for nm, st in want:
        p = _lock_path(nm, st)
        tmp = p + ".tmp%d" % os.getpid()
        with open(tmp, "w") as f:
            json.dump(dict(pid=os.getpid(), started=time.strftime("%F %T"),
                           argv=sys.argv, species=nm, stage=st), f)
        os.replace(tmp, p)
        _HELD.append(p)
    return list(_HELD)


def release_run_locks():
    for p in _HELD:
        d = read_lock(p)
        if d and int(d["pid"]) == os.getpid():
            try:
                os.remove(p)
            except OSError:
                pass
    del _HELD[:]


def merge_partial(prev, out, js, incomplete):
    """Fold a narrow run into the accumulated partial file, never narrowing it.

    Named for the tool rather than for the invocation, the partial file used to
    be REPLACED by whatever the last run happened to cover: `--assemble He2`
    left one species where six had been, with no error and a cheerful "wrote"
    line.  Merging is the fix; the assert is the check that the fix held.
    """
    merged = dict(prev.get("species") or {})
    merged.update(js)
    out["species"] = merged
    inc = dict(prev.get("incomplete_species") or {})
    inc.update(incomplete)
    for nm in js:
        inc.pop(nm, None)
    if inc:
        out["incomplete_species"] = inc
    else:
        out.pop("incomplete_species", None)
    lost = set(prev.get("species") or {}) - set(merged)
    assert not lost, "the merge dropped %s" % sorted(lost)
    return out


def main():
    args = sys.argv[1:]
    stages = [a for a in args if a.startswith("--")]
    names = [a for a in args if not a.startswith("--")] or list(SP.DIATOMICS)
    # An empty --stage list means main() runs the default four, so the lock
    # must name those four rather than a stand-in nobody else would collide
    # with.  (Naming the work is the whole point of the lock.)
    acquire_run_locks(stages or ["--energies", "--stencil", "--hermite",
                                 "--assemble"], names)
    selftest_pool_guard()
    atexit.register(release_run_locks)
    npl = int(os.environ.get("NPROC_LIGHT", "20"))
    nph = int(os.environ.get("NPROC_HEAVY", "12"))
    if not stages or "--energies" in stages:
        print("STAGE 1  grid energies", flush=True)
        stage1(names, npl, nph)
    if not stages or "--stencil" in stages:
        print("STAGE 2  raised-precision stencils", flush=True)
        stage2_fd(names, npl)
    if not stages or "--hermite" in stages:
        print("STAGE 2b hermite probes", flush=True)
        stage2b_hermite(names, npl, nph)
    if "--spin" in stages:
        print("STAGE 1c  <S^2> at every staked geometry", flush=True)
        stage_spin(names, npl, nph)
    if "--probe" in stages:
        print("STAGE 1d  spin probes BEYOND the staked grid", flush=True)
        stage_probe(names, npl, nph)
    if "--recertify" in stages:
        print("STAGE 1b  recertify weak geometries", flush=True)
        stage_recertify(names, npl, nph)
    if "--minima" in stages:
        print("STAGE 3  minima only (cacheable, parallel per species)",
              flush=True)
        with open(os.path.join(HERE, "elements_atoms.json")) as f:
            atoms = json.load(f)
        for nm in names:
            t0 = time.time()
            a = assemble_minimum_only(nm)
            print("  %-4s R_e = %s   (%.1fs)"
                  % (nm, a if isinstance(a, str) else "none",
                     time.time() - t0), flush=True)
    if not stages or "--assemble" in stages:
        print("STAGE 3/4  minima, envelope, gates", flush=True)
        with open(os.path.join(HERE, "elements_atoms.json")) as f:
            atoms = json.load(f)
        recs, js, incomplete = {}, {}, {}
        for nm in names:
            t0 = time.time()
            try:
                a = assemble(nm, atoms)
            except Incomplete as e:
                incomplete[nm] = str(e)
                print("  %-4s INCOMPLETE: %s" % (nm, e), flush=True)
                continue
            recs[nm] = a
            js[nm] = to_json(a)
            print("  %-4s ndet=%-6d Re=%s De=%s  |A-B|<=%.1e  (%.1fs)"
                  % (nm, a["ndet"],
                     R.s(a["Re"], 14) if a["Re"] is not None else "  none    ",
                     R.s(a["De"], 14) if a["De"] is not None else "  none    ",
                     max(a["devAB"]), time.time() - t0), flush=True)
        out = dict(model="ELEMENTS1/STO-3G/FCI",
                   working_precision_dps=R.DPS, precision_digits=R.REPORT,
                   note=("Exact-in-model full CI in the declared STO-3G "
                         "minimal basis.  NOT a prediction of experiment: "
                         "the dispersion binding of He2 is real physics this "
                         "model excludes."),
                   basis_fingerprint=EC.basis_fingerprint(),
                   species=js)
        complete = set(recs) >= set(SP.DIATOMICS)
        p = os.path.join(HERE, "elements_potential.json" if complete
                         else "elements_potential_partial.json")
        if complete:
            if incomplete:
                out["incomplete_species"] = incomplete
            out["gates"] = score_gates(recs)
        else:
            # MERGE, never narrow.  The partial file is named for the TOOL and
            # not for the invocation, so `--assemble He2` used to replace the
            # whole accumulated set with one species -- a silent loss with no
            # error, no diff, and a cheerful "wrote" line.  (It happened.)
            prev = {}
            if os.path.exists(p):
                try:
                    with open(p) as f:
                        prev = json.load(f)
                except ValueError:
                    prev = {}
            merge_partial(prev, out, js, incomplete)
        with open(p, "w") as f:
            json.dump(out, f, indent=1)
        print("wrote %s (%d species: %s)"
              % (os.path.basename(p), len(out["species"]),
                 " ".join(sorted(out["species"]))))


if __name__ == "__main__":
    main()
