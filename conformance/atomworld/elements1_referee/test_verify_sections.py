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


# ---------------------------------------------------------------------------
# V11: the record's own verdicts.  Each of these was carried and unconsumed
# until the engine lane's `converged()` finding prompted the same question
# here, so each one gets its failing case.
# ---------------------------------------------------------------------------
def _pot_and_atoms():
    import json as _json
    import os as _os
    HERE = _os.path.dirname(_os.path.abspath(__file__))
    src = _os.path.join(HERE, "elements_potential_partial.json")
    if not _os.path.exists(src):
        src = _os.path.join(HERE, "elements_potential.json")
    return (_json.load(open(src)),
            _json.load(open(_os.path.join(HERE, "elements_atoms.json"))))


def _v11(mutate=None, name=None):
    import copy
    pot, atoms = _pot_and_atoms()
    nm = name or ("F2" if "F2" in pot["species"] else
                  sorted(pot["species"])[0])
    pot = copy.deepcopy(pot)
    if mutate:
        mutate(pot["species"][nm])
    del V.FAIL[:]
    V.v11_record_verdicts(pot, atoms)
    out = list(V.FAIL)
    del V.FAIL[:]
    return out


def test_v11_passes_on_the_real_record():
    assert not _v11(), _v11()
    print("  V11 passes on the record as assembled -- the control")


def test_v11_fires_on_a_wrong_asymptote():
    def m(r):
        r["exact"]["E_asymptote"] = str(float(r["exact"]["E_asymptote"]) + 1e-40)
    f = _v11(m)
    assert any("asymptote" in x for x in f), f
    print("  asymptote off by 1e-40 -> fires")


def test_v11_fires_on_a_second_minimum():
    f = _v11(lambda r: r.__setitem__("n_minima", 2))
    assert any("extremum counts" in x for x in f), f
    print("  a second minimum -> fires")


def test_v11_fires_on_a_flipped_repulsive_verdict():
    f = _v11(lambda r: r.__setitem__("monotone_repulsive",
                                     not r["monotone_repulsive"]))
    assert any("monotone-repulsive" in x for x in f), f
    print("  flipped monotone-repulsive verdict -> fires")


def test_v11_fires_when_an_unbound_control_dips_below_dissociation():
    import json as _json
    pot, _ = _pot_and_atoms()
    unb = [k for k, v in pot["species"].items() if not v["bound"]]
    if not unb:
        print("  (no unbound control assembled yet)")
        return
    f = _v11(lambda r: r["diagnostics"].__setitem__(
        "E_at_Rmax_minus_asymptote", -1e-9), name=unb[0])
    assert any("ABOVE dissociation" in x for x in f), f
    print("  %s dipping below dissociation -> fires" % unb[0])


def test_v11_fires_when_the_third_route_silently_did_not_run():
    f = _v11(lambda r: r["diagnostics"].__setitem__("route_C_available", False))
    assert any("route-C" in x for x in f), f
    print("  third route absent while declared -> fires")


def test_v11_fires_when_the_R1_agreement_names_one_route_twice():
    f = _v11(lambda r: r["diagnostics"].__setitem__(
        "route_agreement_route", "A (Slater-Condon) vs A (Slater-Condon)"))
    assert any("two distinct routes" in x for x in f), f
    print("  R1 comparing a route with itself -> fires")


if __name__ == "__main__":
    fns = [v for k, v in sorted(globals().items()) if k.startswith("test_")]
    for f in fns:
        f()
    print("test_verify_sections: %d passed" % len(fns))
