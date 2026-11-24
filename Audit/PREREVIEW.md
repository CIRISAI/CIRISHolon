# The pre-review check — what a draft owes before a reviewer is asked

*2026-11-23. Three drafts went to adversarial review that day and all three came back
REDESIGN (`conformance/water_observatory/SEAM_NS0_REVIEW.md`,
`conformance/crystal/ANOMALY1_REVIEW.md`, `conformance/omega/COSMOS_TWO_PORTION.md`). The
lead's word: "this is the ratchet working. We need to bank this into our core engine." This
file says what was banked where, and how a draft passes.*

## The order of gates for a new campaign

1. **Write the draft with its `prereview` blocks.** `python3 Audit/prereview_check.py DRAFT.md`
   must print PASSED. It checks the draft's own arithmetic and nothing else.
2. **Adversarial review**, by a reader not anchored on the author (the house rule). The
   review is recorded beside the draft.
3. **The freeze.** `python3 Audit/prereg_audit.py X_PREREG.md` must print ADMITTED (words:
   witnesses, misfit citations, gate rows, the plants' sector statement). CI runs this one.

Step 1 does not replace step 2. It removes from step 2 what a program can find.

## The convention

A draft carries one or more fenced blocks with the info string `prereview`, each a JSON
object or a list of them. The full grammar, field by field, is the header of
`Audit/prereview_check.py`; this is the smallest draft that passes:

```
{"kind": "stake", "id": "S1", "quantity": "defect",
 "margin": 0.02, "margin_reason": "the reading's standard error on the carrier",
 "regions": [{"name": "MET", "hi": 0.10}, {"name": "KILL", "lo": 0.30}],
 "separates": [["closed", "open"]],
 "table": [{"hypothesis": "closed", "label": "one law", "reads": [0.00, 0.05], "expect": "MET"},
           {"hypothesis": "open", "label": "damping changed", "reads": [0.40, 0.50], "expect": "KILL"}]}
{"kind": "plant", "id": "P-closed", "stake": "S1", "lands": 0.03, "must": "MET"}
{"kind": "plant", "id": "P-open", "stake": "S1", "lands": 0.45, "must": "KILL"}
{"kind": "branches", "outcomes": {"gate": ["pass", "fail"], "S1": ["MET", "BETWEEN", "KILL"]},
 "branches": [{"id": "void", "when": {"gate": ["fail"]}},
              {"id": "a", "when": {"gate": ["pass"], "S1": ["MET"]}},
              {"id": "b", "when": {"gate": ["pass"], "S1": ["BETWEEN"]}},
              {"id": "k", "when": {"gate": ["pass"], "S1": ["KILL"]}}]}
```

(Shown as a plain fence so that this file is not itself read as a draft; in a draft the
objects go in a list inside a fence whose info string is `prereview`.) A draft that
reserves a region for a later blind stake adds a `reserve` block and `points` blocks; a
draft that cites numbers adds a `numbers` block; files it will create go in `new_files`.

## What it refuses, and the misfit each refusal names

| check | refuses | misfit |
|---|---|---|
| a | a stake with no table under two hypotheses; a bar within the stated margin of any cell; a row that does not land where the draft says; two hypotheses the stake must separate reaching one region; a region no row reaches | M-SORTS-NOT-SEPARATES, M-JOINT-PASS-REGION |
| b | a stake with no plant; a plant with no paper value; a paper value outside, or within the margin of, the region it must reach | M-PLANT-OBS |
| c | an outcome of the declared space that enters no branch, or two; a branch nothing enters | (the house rule: exactly one branch per outcome) |
| d | a file named in backticks that is not in the tree and not declared new; a cited number that does not occur in the file it is cited to | M-STALE-INSTRUMENT |
| e | a reserve with no declared points; a reserve that does not say bare or shifted, or states no shift, or does not say what theory already fixes within it; any declared point, bare or shifted, within the reserve; prose that reserves a region with no block | M-RESERVE-BREACHED, M-NAMED-POINT-SHIFTED |

## What it cannot do

It cannot tell whether the table's numbers are right, whether the hypotheses are the right
ones, whether the declared outcome space is the real one, whether a cited number means in
its file what the draft says or was measured in the regime it is spent in, whether the
declared points are all the points, or anything about prior art or blindness. Those are the
review's. Numbers with a unit that sit in prose away from any file are counted and listed,
and refused only under `--strict-numbers`, because prose parsing that refuses is how a gate
gets switched off.

## Proof that it fires

`python3 Audit/test_prereview_check.py`: one good draft that passes; thirteen planted bad
drafts, each one edit away from the good one and each refused for its own reason; and
three fixtures in `Audit/prereview_fixtures/` that transcribe the two reviewed drafts as
they were written. SEAM-NS-0 is refused for the reading astride both bars, the two readings
sharing a region, and the open plant failing on paper. ANOMALY-1 is refused for kills
beside cells of its own table and for a reserve that states neither which mass it means nor
a shift; with the review's shift written in, every one of its campaign points falls within
its own reserve.

## Not wired into CI yet, on purpose

No draft in the tree carries a block, so a blocking gate today would refuse all of them,
which is refusing backward. To wire it, forward only, add to `engine/ci-gates.sh` beside
gate 9c:

```
prev_fail=0; prev_seen=0
for d in ../conformance/*/*_PREREG_DRAFT*.md; do
  [ -f "$d" ] || continue
  grep -q '^```prereview' "$d" || continue        # drafts older than the convention
  prev_seen=$((prev_seen+1))
  python3 ../Audit/prereview_check.py "$d" >/dev/null 2>&1 || { prev_fail=1; echo "  pre-review check refuses: $(basename "$d")"; }
done
python3 ../Audit/test_prereview_check.py >/dev/null 2>&1 || prev_fail=1
[ "$prev_fail" -eq 0 ] && ok "pre-review check: its tests pass and every draft carrying a block passes (saw $prev_seen)" \
  || no "pre-review check"
```

That form gates the tool's own tests at once and each draft from the day it adopts the
convention. Making the block mandatory for drafts dated after a named day is the lead's call.
