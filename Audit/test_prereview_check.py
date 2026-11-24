#!/usr/bin/env python3
"""The pre-review check is a gate, and gates prove they can fire.

One GOOD draft that must pass, and one planted bad draft per check, each differing from
the good one in exactly one place and each required to be refused FOR THAT REASON (a
refusal for some other reason would not show the check works). Then the two drafts that
were sent back by review on 2026-11-23, transcribed into the convention as they were
written, each required to be refused for what its reviewer found.

Run: python3 Audit/test_prereview_check.py
"""
import copy
import json
import pathlib
import sys
import tempfile

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import prereview_check as P  # noqa: E402

HERE = pathlib.Path(__file__).resolve().parent

GOOD = [
    {"kind": "stake", "id": "S1", "quantity": "defect", "margin": 0.02,
     "margin_reason": "the reading's standard error on the carrier, from the banked record",
     "regions": [{"name": "MET", "hi": 0.10}, {"name": "KILL", "lo": 0.30}],
     "separates": [["closed", "open"]],
     "table": [
         {"hypothesis": "closed", "label": "one law", "reads": [0.00, 0.05], "expect": "MET"},
         {"hypothesis": "open", "label": "damping changed threefold", "reads": [0.40, 0.50], "expect": "KILL"},
         {"hypothesis": "weak", "label": "a small second mode", "reads": 0.20, "expect": "BETWEEN"}]},
    {"kind": "plant", "id": "P-closed", "stake": "S1", "lands": 0.03, "must": "MET"},
    {"kind": "plant", "id": "P-open", "stake": "S1", "lands": 0.45, "must": "KILL"},
    {"kind": "branches",
     "outcomes": {"gate": ["pass", "fail"], "S1": ["MET", "BETWEEN", "KILL"]},
     "branches": [
         {"id": "void", "when": {"gate": ["fail"]}},
         {"id": "a", "when": {"gate": ["pass"], "S1": ["MET"]}},
         {"id": "b", "when": {"gate": ["pass"], "S1": ["BETWEEN"]}},
         {"id": "k", "when": {"gate": ["pass"], "S1": ["KILL"]}}]},
    {"kind": "numbers", "cited": [
        {"value": "0.180", "source": "conformance/x/RECORD_RESULTS.md"},
        {"value": "1,500 m/s", "source": "literature: a named paper"}]},
    {"kind": "new_files", "paths": ["conformance/x/instrument.py"]},
    {"kind": "reserve", "id": "next-rung",
     "region": {"x": {"lo": 1.5, "hi": 25}, "mass": {"lo": 0, "lo_open": True}},
     "meaning": {"x": "bare", "mass": "shifted"},
     "shifts": {"mass": "mass + 1/(8*sqrt(x))"},
     "known_inside": "first-order theory fixes the small-mass surface; the stake goes where it fails"},
    {"kind": "points", "of": "campaign", "label": "S1",
     "points": [{"x": 0.7, "mass": 0.0}, {"x": 40, "mass": 0.0}]},
]

PROSE = """# A synthetic draft

The banked closure is `0.180` (`conformance/x/RECORD_RESULTS.md`). The instrument will be
`conformance/x/instrument.py`. A region must stay unread for the next rung.

"""


def draft(objs, prose=PROSE):
    return prose + "```prereview\n" + json.dumps(objs, indent=1) + "\n```\n"


def run(text, strict=False):
    """Check `text` as a draft inside a small tree that holds the one record it cites."""
    with tempfile.TemporaryDirectory() as d:
        root = pathlib.Path(d)
        (root / "conformance" / "x").mkdir(parents=True)
        (root / "conformance" / "x" / "RECORD_RESULTS.md").write_text("closure D = 0.180 +- 0.058\n")
        p = root / "conformance" / "x" / "DRAFT.md"
        p.write_text(text)
        return P.check(p, root, strict)[0]


def edit(fn):
    objs = copy.deepcopy(GOOD)
    fn(objs)
    return objs


def by(objs, kind, ident=None):
    return next(o for o in objs if o["kind"] == kind and (ident is None or o.get("id") == ident))


def bar_on_a_cell(o):       # (a) the kill moved to sit beside the open row's low end
    by(o, "stake")["regions"][1]["lo"] = 0.39


def astride(o):             # (a) the open reading lands astride the kill, as SEAM-NS-0's did
    by(o, "stake")["table"][1]["reads"] = [0.25, 0.36]


def same_region(o):         # (a) both hypotheses read MET: the quantity does not separate them
    by(o, "stake")["table"][1].update(reads=[0.02, 0.06], expect="MET")
    o[:] = [x for x in o if x.get("id") != "P-open"]


def no_table(o):            # (a) a stake with no separation table at all
    by(o, "stake")["table"] = []


def plant_wrong_side(o):    # (b) the open plant, on paper, does not reach the kill
    by(o, "plant", "P-open")["lands"] = 0.25


def no_plant(o):            # (b) a stake with no plant
    o[:] = [x for x in o if x["kind"] != "plant"]


def uncovered(o):           # (c) BETWEEN enters no branch
    b = by(o, "branches")
    b["branches"] = [x for x in b["branches"] if x["id"] != "b"]


def doubled(o):             # (c) a kill also enters the void branch
    by(o, "branches")["branches"][0]["when"] = {"S1": ["KILL"]}
    by(o, "branches")["branches"].append({"id": "void2", "when": {"gate": ["fail"], "S1": ["MET", "BETWEEN"]}})


def number_not_in_file(o):  # (d) cited to a file that does not hold it
    by(o, "numbers")["cited"][0]["value"] = "0.181"


def path_missing(o):        # (d) the instrument is named but neither exists nor is declared new
    o[:] = [x for x in o if x["kind"] != "new_files"]


def point_in_reserve(o):    # (e) a massless point at x = 4: inside, once the shift is applied
    by(o, "points")["points"].append({"x": 4, "mass": 0.0})


def reserve_unstated(o):    # (e) the reserve does not say bare or shifted
    by(o, "reserve")["meaning"] = {"x": "bare"}


def reserve_by_honour(o):   # (e) the prose reserves a region and no block declares it
    o[:] = [x for x in o if x["kind"] not in ("reserve", "points")]


BAD = [
    ("a: a bar beside a cell", bar_on_a_cell, "within the margin"),
    ("a: a reading astride a bar", astride, "within the margin"),
    ("a: two hypotheses in one region", same_region, "does not separate"),
    ("a: no separation table", no_table, "separation table needs"),
    ("b: a plant on the wrong side, on paper", plant_wrong_side, "ON PAPER it lands"),
    ("b: a stake with no plant", no_plant, "no plant is declared"),
    ("c: an outcome with no branch", uncovered, "enters NO branch"),
    ("c: an outcome with two branches", doubled, "enters 2 branches"),
    ("d: a number not in its cited file", number_not_in_file, "does not occur in"),
    ("d: a named file that does not exist", path_missing, "not in the tree"),
    ("e: a campaign point inside the reserve", point_in_reserve, "INSIDE the reserved region after the shift"),
    ("e: a reserve that does not say bare or shifted", reserve_unstated, "bare parameter or the shifted"),
    ("e: a reserve kept by honour", reserve_by_honour, "declares no reserve block"),
]


def main():
    failures, checked = [], 0

    good = run(draft(GOOD))
    if good:
        failures.append(f"the GOOD draft was refused, so no refusal below isolates anything: {good}")

    for name, fn, needle in BAD:
        errs = run(draft(edit(fn)))
        checked += 1
        if not errs:
            failures.append(f"NOT REFUSED: {name}")
        elif not any(needle in e for e in errs):
            failures.append(f"REFUSED FOR ANOTHER REASON: {name}: wanted {needle!r}, got {errs}")

    # no block at all: the state of every draft written before this convention
    if not any("no ```prereview block" in e for e in run(PROSE)):
        failures.append("a draft with no prereview block was not refused for lacking one")
    checked += 1

    # a block that is not JSON must be refused as such, not read as empty
    if not any("is not JSON" in e for e in run(PROSE + "```prereview\n{not json\n```\n")):
        failures.append("a malformed block was not refused as malformed")
    checked += 1

    # --strict-numbers: a unit-bearing number in prose with no file nearby and no entry
    loose = draft(GOOD, PROSE + "The gap is 3,200 m/s at short wavelength.\n\n")
    if run(loose):
        failures.append("a loose number refused the draft without --strict-numbers")
    if not any("--strict-numbers" in e for e in run(loose, strict=True)):
        failures.append("a loose number did not refuse the draft under --strict-numbers")
    checked += 2

    # an ordered branch list (first match wins) must be gradable as ordered
    ordered = edit(lambda o: by(o, "branches").update(
        first_match=True,
        branches=[{"id": "void", "when": {"gate": ["fail"]}},
                  {"id": "a", "when": {"S1": ["MET"]}},
                  {"id": "k", "when": {"S1": ["KILL"]}},
                  {"id": "b", "when": {}}]))
    if run(draft(ordered)):
        failures.append(f"an ordered branch list was refused: {run(draft(ordered))}")
    checked += 1

    # ---- the two drafts of 2026-11-23, as they were written (fixtures beside this file)
    want = {
        "prereview_fixtures/SEAM_NS0_as_drafted.md": [
            ("the reading astride both bars", "S1: the bar MET hi = 0.2 is within the margin"),
            ("the two readings in one region", "does not separate"),
            ("the open plant failing on paper", "plant P2 on S1: ON PAPER it lands"),
        ],
        "prereview_fixtures/ANOMALY1_as_drafted.md": [
            ("a kill beside a cell of its own table", "S2: the bar KILL lo = 0.16 is within the margin"),
            ("the banked-referee stake astride its bars", "S1: the bar KILL lo = 0.22 is within the margin"),
            ("a reserve that does not say which mass it means", "bare parameter or the shifted"),
            ("a reserve with no stated shift", "no \"shifts\""),
        ],
        "prereview_fixtures/ANOMALY1_with_the_shift.md": [
            ("the campaign's own points inside its reserve", "INSIDE the reserved region after the shift"),
        ],
    }
    for rel, needs in want.items():
        errs, _ = P.check(HERE / rel, P.ROOT)
        checked += 1
        for what, needle in needs:
            if not any(needle in e for e in errs):
                failures.append(f"{rel}: not refused for {what} (wanted {needle!r})")

    print(f"pre-review checks exercised: {checked}  (planted bad drafts: {len(BAD)})")
    if checked < len(BAD) + 7:
        failures.append("this suite did less work than its own table")
    if failures:
        print("\nFAILED")
        for f in failures:
            print(f"    - {f}")
        return 1
    print("\nPASSED: the good draft passes; every planted defect is refused for its own reason; "
          "both drafts of 2026-11-23 are refused for what their reviewers found")
    return 0


if __name__ == "__main__":
    sys.exit(main())
