"""Every refusal in the emitter, fired on purpose.

The emitter's job is to refuse a file that would mislead, and each refusal here
was written after a real near-miss: a declared zero uncertainty, an absent spin
audit, a solver state above the next sector's minimum, a resolution column that
was missing and defaulted to "resolved everywhere".  None of them had a test
that made it fire, which by this campaign's own rule makes them indistinguish-
able from refusals that cannot.
"""
import copy
import json
import os
import shutil
import tempfile

import emit_engine as EE

HERE = os.path.dirname(os.path.abspath(__file__))
SRC = os.path.join(HERE, "elements_potential_partial.json")
if not os.path.exists(SRC):
    SRC = os.path.join(HERE, "elements_potential.json")
POT = json.load(open(SRC))
SPECIES = "F2" if "F2" in POT["species"] else sorted(POT["species"])[0]


def _fresh_out():
    return tempfile.mkdtemp(prefix="emit_test_")


def _emit(rec, name=SPECIES):
    out = _fresh_out()
    try:
        return EE.emit_pair(name, rec, out)
    finally:
        shutil.rmtree(out, ignore_errors=True)


def _fires(rec, want, label):
    try:
        _emit(rec)
    except AssertionError as e:
        assert want in str(e), "%s: wrong refusal: %s" % (label, e)
        print("  %s -> refused" % label)
        return
    raise AssertionError("%s: THE REFUSAL DID NOT FIRE" % label)


def test_intact_record_emits():
    p, n, changes = _emit(copy.deepcopy(POT["species"][SPECIES]))
    assert os.path.basename(p) == SPECIES + ".json"
    print("  the intact record emits (%d geometries) -- the control" % n)


def test_missing_resolution_column_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["spin"].pop("resolved_by_geometry", None)
    _fires(rec, "no per-geometry resolution column",
           "spin block with no resolution column")


def test_short_resolution_column_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["spin"]["resolved_by_geometry"] = \
        rec["spin"]["resolved_by_geometry"][:-3]
    _fires(rec, "resolution column covers", "resolution column three short")


def test_missing_spin_audit_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["spin"] = None
    _fires(rec, "refusing to emit without a spin audit", "no spin audit at all")


def test_unresolved_everywhere_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["spin"]["spin_resolved_out_to_bohr"] = None
    _fires(rec, "not resolved at ANY geometry", "multiplicity resolved nowhere")


def test_sector_ordering_violation_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["spin"]["sector_ordering_violations"] = [["2.0", "planted"]]
    _fires(rec, "the lowest in its sector", "a state above the next sector")


def test_zero_energy_uncertainty_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["diagnostics"]["energy_uncertainty_total"] = "0"
    _fires(rec, "refusing to declare a zero ENERGY uncertainty",
           "a declared zero energy uncertainty")


def test_uncertainty_wider_than_the_printed_digits_is_refused():
    rec = copy.deepcopy(POT["species"][SPECIES])
    rec["diagnostics"]["energy_uncertainty_total"] = "1e-20"
    _fires(rec, "exceeds one unit in the 50th",
           "a bound that does not cover the printed digits")


def test_a_grid_that_its_rule_does_not_reproduce_is_refused():
    """The sparse-subset guarantee, fired: doctor the grid and the emitter must
    notice that the stated rule no longer produces it."""
    import species as SP
    rec = copy.deepcopy(POT["species"][SPECIES])
    d = dict(SP.DIATOMICS[SPECIES])
    d["sparse"] = dict(well_stride=2, tail_stride=3)
    old = SP.DIATOMICS[SPECIES]
    SP.DIATOMICS[SPECIES] = d
    try:
        _fires(rec, "NOT the subset its own stated rule",
               "a grid its declared rule does not reproduce")
    finally:
        SP.DIATOMICS[SPECIES] = old


if __name__ == "__main__":
    fns = [v for k, v in sorted(globals().items()) if k.startswith("test_")]
    for f in fns:
        f()
    print("test_emit_refusals: %d passed" % len(fns))
