#!/usr/bin/env python3
"""
mutation_test.py -- plant defects and confirm verify_atom_core.py refuses.

A gate that cannot fail on a planted defect is not a gate.  Each mutation below
names the check it is supposed to trip and the exit code it must produce.
Exits nonzero if any mutation went undetected.
"""
import copy
import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "h2_potential.json")
VERIFY = os.path.join(HERE, "verify_atom_core.py")


def run(doc, tmpdir, verify=VERIFY, extra=()):
    path = os.path.join(tmpdir, "mutant.json")
    with open(path, "w") as f:
        json.dump(doc, f)
    p = subprocess.run([sys.executable, verify, "--json", path, *extra],
                       capture_output=True, text=True)
    return p.returncode, p.stdout


# (name, expected exit, expected check to trip or None, mutator)
def m_energy(d):
    d["E_hartree"][0] += 1e-9
    d["exact"]["E_hartree"][0] = repr(d["E_hartree"][0])
    d["hermite"]["values_hartree"][0] = d["E_hartree"][0]


def m_force(d):
    d["F_hartree_per_bohr"][0] *= 1.000001
    d["exact"]["F_hartree_per_bohr"][0] = repr(d["F_hartree_per_bohr"][0])


def m_missing_key(d):
    del d["R_e"]


def m_slope(d):
    d["hermite"]["derivatives_hartree_per_bohr"][0] *= 1.01


def m_Re(d):
    d["R_e"] *= 1.01
    d["exact"]["R_e"] = repr(d["R_e"])


def m_De(d):
    d["D_e"] *= 1.001


def m_second_minimum(d):
    # carve a dimple into the outer wall: a second minimum in dE/dR
    n = len(d["F_hartree_per_bohr"])
    for i in range(n - 60, n - 40):
        d["F_hartree_per_bohr"][i] = abs(d["F_hartree_per_bohr"][i]) + 1e-6
        d["hermite"]["derivatives_hartree_per_bohr"][i] = \
            -d["F_hartree_per_bohr"][i]


def m_above_asymptote(d):
    # push the tail above the asymptote: kills "approaches from below"
    for i in range(len(d["E_hartree"])):
        if d["R_grid_bohr"][i] > 6.0:
            d["E_hartree"][i] = d["E_asymptote"] + 1e-7
            d["hermite"]["values_hartree"][i] = d["E_hartree"][i]


def m_asymptote(d):
    d["E_asymptote"] *= 1.0001


def m_hermite_bound(d):
    d["hermite"]["max_abs_error_E_hartree"] = 1e-14


def m_E2(d):
    d["E2_hartree_per_bohr2"][0] *= 1.01
    d["exact"]["E2_hartree_per_bohr2"][0] = repr(d["E2_hartree_per_bohr2"][0])


def m_envelope_nonmonotone(d):
    r = d["max_curvature_up_to_E"]["rungs"]
    r[-1]["max_abs_E2_hartree_per_bohr2"] = \
        0.5 * r[0]["max_abs_E2_hartree_per_bohr2"]


def m_envelope_understates(d):
    # halve the whole envelope: monotonicity and the dt formula stay self
    # consistent, so ONLY an independent recomputation can catch this
    import math
    for r in d["max_curvature_up_to_E"]["rungs"]:
        r["max_abs_E2_hartree_per_bohr2"] *= 0.5
        r["dt_per_sqrt_mu"] = 2 * math.pi / (
            64 * math.sqrt(r["max_abs_E2_hartree_per_bohr2"]))


def m_envelope_turning_point(d):
    d["max_curvature_up_to_E"]["rungs"][0]["R_in_bohr"] *= 1.01


def m_envelope_dt(d):
    d["max_curvature_up_to_E"]["rungs"][3]["dt_per_sqrt_mu"] *= 1.01


def m_del_envelope(d):
    del d["max_curvature_up_to_E"]


def m_del_E2(d):
    del d["E2_hartree_per_bohr2"]


def m_grid_short(d):
    for k in ("R_grid_bohr", "E_hartree", "F_hartree_per_bohr"):
        d[k] = d[k][:150]
    for k in ("knots_bohr", "values_hartree", "derivatives_hartree_per_bohr"):
        d["hermite"][k] = d["hermite"][k][:150]
    d["E2_hartree_per_bohr2"] = d["E2_hartree_per_bohr2"][:150]
    for k in ("R_grid_bohr", "E_hartree", "F_hartree_per_bohr",
              "E2_hartree_per_bohr2"):
        d["exact"][k] = d["exact"][k][:150]


MUTATIONS = [
    ("E value off by 1e-9", 1, "V3_energies_match_model", m_energy),
    ("F value off by 1 ppm", 1, "V4_forces_match_model", m_force),
    ("required key R_e deleted", 2, "V1_contract_wellformed", m_missing_key),
    ("hermite slope off by 1%", 1, "V14_hermite_slopes_match_model", m_slope),
    ("R_e moved by 1%", 1, "V9_R_e_is_stationary_minimum", m_Re),
    ("D_e off by 0.1%", 1, "V10_D_e_consistent", m_De),
    ("second minimum carved in", 1, "V6_exactly_one_minimum", m_second_minimum),
    ("tail pushed above asymptote", 1, "V7_asymptote_from_below",
     m_above_asymptote),
    ("asymptote off by 1e-4 rel", 1, "V5_asymptote_in_model", m_asymptote),
    ("hermite bound understated", 1, "V12_hermite_bound_holds", m_hermite_bound),
    ("grid truncated to 150 points", 2, "V1_contract_wellformed", m_grid_short),
    ("E2 value off by 1%", 1, "V15_E2_matches_model", m_E2),
    ("envelope made non-monotone", 1, "V16_envelope_wellformed",
     m_envelope_nonmonotone),
    ("envelope halved, self-consistently", 1,
     "V18_envelope_curvature_not_understated", m_envelope_understates),
    ("envelope turning point moved 1%", 1,
     "V17_envelope_turning_points_correct", m_envelope_turning_point),
    ("envelope dt off by 1%", 1, "V19_envelope_dt_consistent", m_envelope_dt),
    ("max_curvature_up_to_E deleted", 2, "V1_contract_wellformed",
     m_del_envelope),
    ("E2 array deleted", 2, "V1_contract_wellformed", m_del_E2),
]


def main():
    with open(SRC) as f:
        base = json.load(f)
    tmpdir = tempfile.mkdtemp(prefix="atom_mut_")
    failures = []
    print("=" * 78)
    print("mutation_test.py -- planted defects must be detected")
    print("=" * 78)

    rc, out = run(copy.deepcopy(base), tmpdir)
    print(f"  control (unmutated): exit {rc}  "
          f"{'OK' if rc == 0 else 'UNEXPECTED -- control must pass'}")
    if rc != 0:
        failures.append("control")

    for name, want_rc, want_check, mut in MUTATIONS:
        doc = copy.deepcopy(base)
        mut(doc)
        rc, out = run(doc, tmpdir)
        tripped = f"[FAIL] {want_check}" in out or (
            want_rc == 2 and ("REFUSE" in out))
        ok = (rc == want_rc) and tripped
        print(f"  [{'OK ' if ok else 'MISS'}] {name:<34} exit {rc} "
              f"(want {want_rc}), {want_check} tripped: {tripped}")
        if not ok:
            failures.append(name)

    # A check that silently does not run must refuse, not pass.
    vpath = os.path.join(tmpdir, "verify_missing_check.py")
    shutil.copy(VERIFY, vpath)
    shutil.copy(os.path.join(HERE, "h2_core.py"),
                os.path.join(tmpdir, "h2_core.py"))
    import re
    vsrc = open(vpath).read()
    vsrc, nsub = re.subn(r"(REQUIRED = \[.*?)\n\]",
                         r'\1\n    "VXX_a_check_that_never_runs",\n]',
                         vsrc, count=1, flags=re.S)
    assert nsub == 1, "could not inject a never-running check into REQUIRED"
    open(vpath, "w").write(vsrc)
    rc, out = run(copy.deepcopy(base), tmpdir, verify=vpath)
    ok = rc == 2 and "MISSING CHECKS" in out
    print(f"  [{'OK ' if ok else 'MISS'}] a required check never runs        "
          f"exit {rc} (want 2), refuse-on-missing: {'MISSING CHECKS' in out}")
    if not ok:
        failures.append("missing-check refusal")

    shutil.rmtree(tmpdir, ignore_errors=True)
    print("-" * 78)
    if failures:
        print(f"UNDETECTED MUTATIONS: {failures}")
        return 1
    print(f"all {len(MUTATIONS) + 1} planted defects detected, control passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
