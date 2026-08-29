"""V10's sparse branch, exercised before the sparse species exist.

N2 and CO are the only pairs that will carry a subset rule, and they are the
last two to land -- so without this the sparse half of the check would ship
having never run, which by this campaign's rule is indistinguishable from a
check that cannot run.  Here it runs on a synthetic N2 drop: once correct, once
with a single knot removed, once with the block missing.
"""
import json
import os
import shutil
import tempfile

import curve as CV
import species as SP
import verify_elements as V


def _synth(outdir, name="N2", drop_knot=None, no_block=False):
    d = SP.DIATOMICS[name]
    full = CV.build_grid(d["rmin"], d["rmax"], d["nbase"], d["well"],
                         d["nsplit"])
    knots = [str(x) for x in SP.sparse_subset(full, d["well"], d["sparse"])]
    if drop_knot is not None:
        knots = knots[:drop_knot] + knots[drop_knot + 1:]
    j = {"model": "%s/STO-3G/FCI" % name, "R_grid_bohr": knots}
    if not no_block:
        j["grid_provenance"] = {
            "rule": "uniform in R^(-1/4) ...",
            "staked_parameters": {"rmin_bohr": d["rmin"], "rmax_bohr": d["rmax"],
                                  "nbase": d["nbase"], "nsplit": d["nsplit"],
                                  "well_window_bohr": list(d["well"])},
            "full_grid_knots": len(full),
            "emitted_knots": len(knots),
            "subset_parameters": dict(d["sparse"]),
        }
    with open(os.path.join(outdir, "%s.json" % name), "w") as f:
        json.dump(j, f)
    return len(full), len(knots)


def _run(**kw):
    d = tempfile.mkdtemp(prefix="v10_")
    try:
        nfull, nk = _synth(d, **kw)
        del V.FAIL[:]
        V.v10_grid_provenance(d)
        return list(V.FAIL), nfull, nk
    finally:
        shutil.rmtree(d, ignore_errors=True)
        del V.FAIL[:]


def test_sparse_branch_passes_on_the_real_subset():
    fails, nfull, nk = _run()
    assert not fails, fails
    print("  sparse branch passes on the rule's own subset "
          "(%d of %d knots)" % (nk, nfull))


def test_sparse_branch_fires_when_a_knot_is_removed():
    fails, _, _ = _run(drop_knot=5)
    assert fails, "V10 DID NOT FIRE on a grid its declared rule cannot produce"
    assert any("SUBSET rule reproduces" in f for f in fails), fails
    print("  one knot removed -> V10 fires:", fails[0][:58])


def test_missing_block_fires():
    fails, _, _ = _run(no_block=True)
    assert fails and any("declares its grid rule" in f for f in fails), fails
    print("  no grid_provenance -> V10 fires")


if __name__ == "__main__":
    fns = [v for k, v in sorted(globals().items()) if k.startswith("test_")]
    for f in fns:
        f()
    print("test_verify_sections: %d passed" % len(fns))
