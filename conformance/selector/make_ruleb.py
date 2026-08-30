#!/usr/bin/env python3
"""make_ruleb.py — extract RULE-B at the version that actually produced the
SELECTOR-5 refutation's numbers, and prove it is tag-free.

WHY THIS EXISTS.  SELECTOR6_DESIGN.md pinned RULE-B at
conformance/selector/refute_lib.py blob 5ddebd48.  That blob is BROKEN, and it
was already broken when it was pinned:

  1. It short-circuits on `G.family` -- a CONSTRUCTION TAG -- returning True for
     six family strings without computing anything.  FROZEN_LABEL_RULE.md and
     SELECTOR6_PREREG.md both forbid construction tags as inputs to the label.
     This is M-TAG-AS-PROPERTY, the misfit whose founding case is the refuted C4,
     reintroduced into the corrective instrument's own label.
  2. `triv = fast_lin_index(np.ones(len(CLS)))` uses the per-ELEMENT class-label
     array where the per-CLASS vector is required, so the label raises ValueError
     on every group that reaches it -- every group with no faithful irrep of
     degree <= 2.  Measured: it crashes on A4, S5, UT(3,5), Delta(27), F21 and A5,
     and answers Q8 and D8 only through defect 1.
  3. The abelian branch's `return (rk <= 4), ...` was deleted, so the rank
     obstruction stated as a theorem in FROZEN_LABEL_RULE.md no longer returns.
  4. `linmul.get((d, ...), None)` silently skips a missing transition where the
     original indexed directly; a missing key can now only UNDER-count achievable
     kernels, and it does so without a sound.

The version pinned here is blob cbaf2b4 (committed at 3b97c29), which is the
code that produced every number in SELECTOR5_REFUTATION.md.  Measured: correct on
all eight independently-checkable cases, and identical when driven through a
tag-free adapter -- because it never reads a tag.

Run:  python3 make_ruleb.py
"""
import hashlib
import os
import subprocess
import sys

REPO = "/home/emoore/CIRISHolon"
PINNED_COMMIT = "3b97c29"   # the commit whose refute_lib.py blob is cbaf2b4
PINNED_SHA256 = "af00cf2c3512735e41e2d83e205119f577b732f1759ed5f9efb50cd009eb361b"
OUT = os.path.join(REPO, "conformance/selector/ruleb.py")

# Any attribute that exists only because of how a group was CONSTRUCTED.
TAG_TOKENS = ("G.family", ".family", "is_lie_type", "G.name", ".aliases",
              "G.notes")

BANNER = '''"""ruleb.py — GENERATED.  Do not edit.

RULE-B, extracted by make_ruleb.py from git blob {blob} (sha256 {sha}) --
the version that produced SELECTOR5_REFUTATION.md's numbers, before the edit
that made the label read a construction tag.  Regenerate, never edit.
"""
'''


def main():
    oid = subprocess.run(
        ["git", "-C", REPO, "rev-parse", f"{PINNED_COMMIT}:conformance/selector/refute_lib.py"],
        capture_output=True, check=True).stdout.decode().strip()
    src = subprocess.run(["git", "-C", REPO, "cat-file", "blob", oid],
                         capture_output=True, check=True).stdout.decode()
    got = hashlib.sha256(src.encode()).hexdigest()
    if got != PINNED_SHA256:
        sys.exit(f"ABORT: blob {oid} hashes {got}, expected {PINNED_SHA256}")

    # THE TAG-FREEDOM GATE: mechanise "construction tags are FORBIDDEN as inputs"
    hits = [(i + 1, l.strip()) for i, l in enumerate(src.split("\n"))
            for tok in TAG_TOKENS if tok in l and not l.strip().startswith("#")]
    if hits:
        for ln, txt in hits:
            print(f"  line {ln}: {txt}")
        sys.exit(f"ABORT: RULE-B reads a construction tag ({len(hits)} sites)")

    text = BANNER.format(blob=oid[:8], sha=PINNED_SHA256[:16] + "…") + src
    with open(OUT, "w") as f:
        f.write(text)
    print(f"wrote {OUT}")
    print(f"  source blob  {oid}  sha256 {PINNED_SHA256}")
    print(f"  derived      sha256 {hashlib.sha256(text.encode()).hexdigest()}")
    print(f"  tag-freedom gate: PASS (no construction tag read anywhere)")

    sys.path.insert(0, os.path.dirname(OUT))
    import ruleb
    for name in ("rule_b_sm", "character_table", "validate_table", "kernels",
                 "det_characters"):
        if not hasattr(ruleb, name):
            sys.exit(f"ABORT: ruleb is missing {name}")
    print("  all RULE-B entry points present")


if __name__ == "__main__":
    main()
