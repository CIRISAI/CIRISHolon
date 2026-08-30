"""The cache's three record kinds, and the key collision that let one of them
wear another's identity.

THE DEFECT THIS SUITE EXISTS FOR.  run_point, energy_only and spin_only all
write into one cache.  spin_only's tags were prefixed; energy_only's were not,
and for the three HEAVY species the stencil precision equals the grid precision
(FD_DPS_CHEAP == R.DPS == 60), so a stencil evaluated AT a grid point produced
that grid point's key exactly.

It failed two ways and only one was loud:

  loud   -- energy_only reading a run_point record raises KeyError: 'E'.  That
            is what stopped the N2 and CO stencil stages, and Li2's before them.
  silent -- when the stencil got there first, its single-route record sat under
            the certified point's key and run_point returned it from the cache
            without looking at its shape.  Li2 carried two (R = 7.490376922377
            and R = 18.000000000000): no route B, no Temple bound, no spin
            sector, no dev_AB -- and the assemble only asked whether a record
            EXISTED, so nothing downstream could tell.

Both halves of the repair are tested here, and each is tested by making the
FAILING case happen rather than by asserting the fixed one works:

  1. the namespaces are disjoint -- and the test shows they were NOT, by
     rebuilding the old un-prefixed tag and finding the collision on the real
     staked grids;
  2. cache_get refuses a record of the wrong kind -- fired in both directions;
  3. a kind clash RAISES rather than returning None, because None means "stale,
     recompute" and recomputing under a clashing key would overwrite the other
     kind's record.  A repair that destroys data is worse than the fault.
"""
import json
import os
import shutil
import tempfile

import build_curves as B
import runner as R
import species as SP

FAILED = []
CHECKS = [0]


def check(cond, label):
    CHECKS[0] += 1
    if not cond:
        FAILED.append(label)
        print("  FAIL  %s" % label)
    else:
        print("  ok    %s" % label)


def fires(fn, exc, label):
    CHECKS[0] += 1
    try:
        fn()
    except exc:
        print("  ok    %s (fired)" % label)
        return
    except Exception as e:
        FAILED.append(label)
        print("  FAIL  %s (raised %s, wanted %s)"
              % (label, type(e).__name__, exc.__name__))
        return
    FAILED.append(label)
    print("  FAIL  %s (did not fire)" % label)


# ---------------------------------------------------------------------------
# 1.  THE COLLISION, DEMONSTRATED ON THE REAL STAKED GRIDS.
#
# The old energy tag was tag_for(name, rs, dps) with no prefix.  Rebuild it and
# count how many staked geometries it lands on a certified point's key -- for
# the heavy species, every stencil centre.
# ---------------------------------------------------------------------------
def test_old_tags_collided():
    print("\n1. the un-prefixed energy tag, on the staked grids")
    point_keys = {}
    for name in SP.DIATOMICS:
        for rs in SP.grid_for(name):
            point_keys[B.tag_for(name, rs, R.DPS)] = (name, rs)
    heavy_hits, light_hits, still_colliding = [], [], []
    n_centres = [0]
    for name in SP.DIATOMICS:
        d = SP.DIATOMICS[name]
        grid = SP.grid_for(name)
        dps = B.stencil_dps(name)
        if d["heavy"]:
            n = len(grid)
            centres = [grid[i] for i in
                       sorted(set([0, n // 3, (2 * n) // 3, n - 1]))]
        else:
            centres = grid
        for rs in centres:
            if B.tag_for(name, rs, dps) in point_keys:      # the old tag
                (heavy_hits if d["heavy"] else light_hits).append((name, rs))
            if B.fd_tag(name, rs, dps) in point_keys:       # the new one
                still_colliding.append((name, rs))
            n_centres[0] += 1
    check(len(heavy_hits) > 0,
          "the OLD tag DID collide on the heavy species (%d stencil centres: "
          "%s)" % (len(heavy_hits),
                   ", ".join("%s@%s" % h for h in heavy_hits[:4])))
    check(not light_hits,
          "the old tag never collided on the light species (stencil dps 95 "
          "!= grid dps 60) -- which is why this went three species without "
          "being seen")
    check(not still_colliding,
          "no fd_ tag lands on a staked point key (%d stencil centres over "
          "%d species)%s"
          % (n_centres[0], len(SP.DIATOMICS), "" if not still_colliding
             else "; %r" % still_colliding[:4]))
    # And the probe geometries, which are interior and were the OTHER reader.
    interior = 0
    for name in SP.DIATOMICS:
        gs = SP.grid_for(name)
        for (_, rs) in B.hermite_probe_points(name, gs,
                                              SP.DIATOMICS[name]["heavy"]):
            interior += 1
            if B.fd_tag(name, rs, R.DPS) in point_keys:
                still_colliding.append((name, rs))
    check(not still_colliding,
          "no Hermite probe's fd_ tag lands on a staked point key "
          "(%d probes)" % interior)


# ---------------------------------------------------------------------------
# 2.  cache_get REFUSES THE WRONG SHAPE, IN BOTH DIRECTIONS.
# ---------------------------------------------------------------------------
def test_kind_guard():
    print("\n2. cache_get's kind guard, fired in both directions")
    tmp = tempfile.mkdtemp(prefix="cachekind_")
    old = R.CACHE
    R.CACHE = tmp
    try:
        point = dict(tag="t", dps=60, sectors={"0": dict(E_A="-1.0")})
        energy = dict(tag="t", dps=60, E="-1.0")
        spin = dict(tag="t", dps=60, E="-1.0", S2="0.0", two_S=0)

        R.cache_put("K_point", 60, dict(point))
        R.cache_put("K_energy", 60, dict(energy))
        R.cache_put("K_spin", 60, dict(spin))

        check(R.record_kind(point) == "point", "record_kind reads a point")
        check(R.record_kind(energy) == "energy", "record_kind reads an energy")
        check(R.record_kind(spin) == "spin",
              "record_kind reads a spin record (which also carries 'E')")

        check(R.cache_get("K_point", 60, kind="point") is not None,
              "the matching kind is returned")
        check(R.cache_get("K_energy", 60, kind="energy") is not None,
              "the matching kind is returned (energy)")
        check(R.cache_get("K_spin", 60, kind="spin") is not None,
              "the matching kind is returned (spin)")

        # THE LOUD HALF: what killed the N2 and CO stencil stages.
        fires(lambda: R.cache_get("K_point", 60, kind="energy"),
              R.CacheKindClash,
              "a point record asked for as an energy (the KeyError: 'E' that "
              "stopped N2 and CO)")
        # THE SILENT HALF: what put two uncertified points into Li2.
        fires(lambda: R.cache_get("K_energy", 60, kind="point"),
              R.CacheKindClash,
              "an energy record asked for as a certified point (the two Li2 "
              "geometries that shipped with no route B and no Temple bound)")
        fires(lambda: R.cache_get("K_energy", 60, kind="spin"),
              R.CacheKindClash, "an energy record asked for as spin")

        # A CLASH MUST NOT LOOK LIKE A MISS.  None means "recompute", and a
        # recompute under a clashing key overwrites the other kind's record.
        n0 = R.cache_get.refused
        try:
            R.cache_get("K_point", 60, kind="energy")
        except R.CacheKindClash:
            pass
        check(R.cache_get.refused == n0,
              "a kind clash is not counted as a stale-basis refusal")
        check(R.cache_get.kind_clashes > 0,
              "kind clashes are counted separately (%d)"
              % R.cache_get.kind_clashes)

        # ABSENT still means absent.
        check(R.cache_get("K_nothing", 60, kind="point") is None,
              "a key that does not exist still returns None")

        # And an UNGUARDED read still returns whatever is there -- the guard is
        # opt-in per call site, so this test also records that every call site
        # in build_curves.py opts in.
        check(R.cache_get("K_point", 60) is not None,
              "an unguarded read is unchanged")
    finally:
        R.CACHE = old
        shutil.rmtree(tmp, ignore_errors=True)


# ---------------------------------------------------------------------------
# 3.  EVERY CACHE READ IN THE PIPELINE DECLARES ITS KIND.
#
# The guard being correct is not the same as the guard being connected; this
# campaign shipped a pool guard that was correct, tested, and called by nothing.
# ---------------------------------------------------------------------------
def test_every_read_declares_a_kind():
    print("\n3. every cache read on the pipeline's path declares a kind")
    import re
    here = os.path.dirname(os.path.abspath(__file__))
    src = open(os.path.join(here, "build_curves.py")).read()
    calls = re.findall(r"R\.cache_get\((?:[^()]|\([^()]*\))*\)", src,
                       re.S)
    check(len(calls) >= 8, "found the cache reads in build_curves.py (%d)"
          % len(calls))
    bare = [c for c in calls if "kind=" not in c]
    check(not bare, "no bare cache_get in build_curves.py"
          + ("" if not bare else "; bare: %r" % bare))
    rsrc = open(os.path.join(here, "runner.py")).read()
    for fn in ("run_point", "spin_only", "energy_only"):
        body = rsrc.split("def %s(" % fn, 1)[1].split("\ndef ", 1)[0]
        check("kind=" in body.split("cache_get(", 1)[1].split(")", 1)[0],
              "runner.%s declares its kind" % fn)


# ---------------------------------------------------------------------------
# 4.  THE LIVE CACHE IS PARTITIONED.
# ---------------------------------------------------------------------------
def test_live_cache_partition():
    print("\n4. the live cache: prefix and shape agree, record by record")
    if not os.path.isdir(R.CACHE):
        print("  (no live cache here; skipped)")
        return
    bad, n = [], 0
    for fn in os.listdir(R.CACHE):
        if not fn.endswith(".json"):
            continue
        n += 1
        try:
            obj = json.load(open(os.path.join(R.CACHE, fn)))
        except Exception:
            bad.append((fn, "unreadable"))
            continue
        k = R.record_kind(obj)
        want = ("spin" if fn.startswith("spin_")
                else "energy" if fn.startswith("fd_") else "point")
        if k != want:
            bad.append((fn, "%s in the %s namespace" % (k, want)))
    check(not bad, "all %d live records match their namespace%s"
          % (n, "" if not bad else "; %d do not: %r" % (len(bad), bad[:5])))


if __name__ == "__main__":
    test_old_tags_collided()
    test_kind_guard()
    test_every_read_declares_a_kind()
    test_live_cache_partition()
    print("\n%d checks, %d FAIL" % (CHECKS[0], len(FAILED)))
    for f in FAILED:
        print("   FAILED: %s" % f)
    raise SystemExit(1 if FAILED else 0)
