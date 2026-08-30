"""One-shot cache migration: move every single-route stencil energy record into
its own "fd_" tag namespace.

WHY THIS IS A MIGRATION AND NOT A RECOMPUTE.  The records are correct
computations -- the same route-A energy at the same geometry at the same
precision -- that were merely FILED under a key that another kind of record also
claimed.  Renaming the file moves it to the key read_fd now looks under; nothing
about a number changes, which is why the six landed species must still emit
byte-for-byte, and that is checked rather than asserted.

WHAT IT REPAIRS.  Where a stencil record had landed on a grid point's key (the
three heavy species only -- for the light ones the stencil runs at dps 95 and
the namespaces never met), the rename VACATES that key.  The grid point then
reads as uncomputed and is recomputed as a proper dual-route certified point,
which is the outcome that was wanted all along.  Li2 had two.
"""
import json
import os
import shutil
import sys

CACHE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "cache")


def kind_of(obj):
    if "sectors" in obj:
        return "point"
    if "S2" in obj:
        return "spin"
    if "E" in obj:
        return "energy"
    return None


def main(apply_it):
    moves, vacated, counts, skipped = [], [], {}, 0
    for fn in sorted(os.listdir(CACHE)):
        if not fn.endswith(".json"):
            continue
        if fn.startswith("spin_") or fn.startswith("fd_"):
            counts[kind_of(json.load(open(os.path.join(CACHE, fn))))] = \
                counts.get(kind_of(json.load(open(os.path.join(CACHE, fn)))), 0) + 1
            continue
        try:
            obj = json.load(open(os.path.join(CACHE, fn)))
        except Exception:
            skipped += 1
            continue
        k = kind_of(obj)
        counts[k] = counts.get(k, 0) + 1
        if k != "energy":
            continue
        moves.append((fn, "fd_" + fn))
        # was this key ALSO a grid point's key?  ask species/build_curves.
        if not fn.startswith("atom_"):
            vacated.append(fn)
    print("cache records by kind (pre-migration):", counts,
          "unreadable:", skipped)
    print("energy records to move: %d" % len(moves))

    # which of the vacated keys are staked GRID points -- the silent collisions
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import species as SP
    import build_curves as B
    import runner as R
    grid_keys = {}
    for name in SP.DIATOMICS:
        for rs in SP.grid_for(name):
            grid_keys[R._key(B.tag_for(name, rs, R.DPS), R.DPS) + ".json"] = \
                (name, rs)
    hits = [(fn, grid_keys[fn]) for fn, _ in moves if fn in grid_keys]
    print("of those, sitting on a STAKED GRID POINT's key: %d" % len(hits))
    for fn, (nm, rs) in hits:
        print("   %-6s R=%s   -> the grid point becomes uncomputed and will be"
              " recomputed dual-route" % (nm, rs))

    clobber = [(a, b) for a, b in moves
               if os.path.exists(os.path.join(CACHE, b))]
    if clobber:
        print("REFUSING: %d destination names already exist" % len(clobber))
        return 2
    if not apply_it:
        print("(dry run; pass --apply)")
        return 0
    for a, b in moves:
        pa, pb = os.path.join(CACHE, a), os.path.join(CACHE, b)
        obj = json.load(open(pa))
        obj["tag"] = "fd_" + obj.get("tag", "")
        obj["record_kind"] = "energy"
        tmp = pb + ".tmp%d" % os.getpid()
        with open(tmp, "w") as f:
            json.dump(obj, f)
        os.replace(tmp, pb)
        os.remove(pa)
    print("moved %d records" % len(moves))
    return 0


if __name__ == "__main__":
    sys.exit(main("--apply" in sys.argv))
