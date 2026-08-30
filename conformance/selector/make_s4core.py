#!/usr/bin/env python3
"""make_s4core.py — derive an importable copy of the frozen SELECTOR-4 criterion.

`conformance/omega/selector4.py` cannot be imported: line 113 is

    LOG = open(__file__.replace(".py", ".log"), "w")

at module scope, so `import selector4` TRUNCATES `conformance/omega/selector4.log`,
which is the committed SELECTOR-4 run record (117 lines, commit 25eae60).

SELECTOR-6 must run the frozen criterion without reimplementing it and without
destroying its predecessor's evidence.  So the criterion is extracted from the
PINNED GIT BLOB (never from the working tree, which another lane may edit) and
exactly one line is neutralised.  The transformation is then VERIFIED to be that
one line and nothing else: any other difference aborts.

Run:  python3 make_s4core.py           # writes s4core.py, verifies, prints hashes
"""
import hashlib
import subprocess
import sys
import os

REPO = "/home/emoore/CIRISHolon"
PINNED_BLOB = "d33f0469376e26ffebb83f9fb83a8580455670df"
PINNED_SHA256 = "0c1752158bcc9d31cbaae93ef7b9930ec2233618234bbc9af19853b9f8e82a0e"
OUT = os.path.join(REPO, "conformance/selector/s4core.py")

OLD = 'LOG = open(__file__.replace(".py", ".log"), "w")'
NEW = ('LOG = open(os.devnull, "w")  '
       '# SELECTOR-6: neutralised by make_s4core.py; see its docstring')

BANNER = '''"""s4core.py — GENERATED.  Do not edit.

The frozen SELECTOR-4 criterion, extracted by make_s4core.py from git blob
{blob} (sha256 {sha}) with exactly one line replaced so that importing it
cannot truncate conformance/omega/selector4.log.  Regenerate, never edit.
"""
import os
'''


def main():
    src = subprocess.run(["git", "-C", REPO, "cat-file", "blob", PINNED_BLOB],
                         capture_output=True, check=True).stdout.decode()
    got = hashlib.sha256(src.encode()).hexdigest()
    if got != PINNED_SHA256:
        sys.exit(f"ABORT: pinned blob hashes {got}, expected {PINNED_SHA256}")

    lines = src.split("\n")
    hits = [i for i, l in enumerate(lines) if l.strip() == OLD]
    if len(hits) != 1:
        sys.exit(f"ABORT: expected exactly one log-open line, found {len(hits)}")
    out_lines = list(lines)
    out_lines[hits[0]] = NEW

    # verify the transformation is EXACTLY that one line
    diff = [(i, a, b) for i, (a, b) in enumerate(zip(lines, out_lines)) if a != b]
    if len(diff) != 1 or diff[0][0] != hits[0]:
        sys.exit(f"ABORT: transformation touched {len(diff)} lines, expected 1")
    if len(out_lines) != len(lines):
        sys.exit("ABORT: line count changed")

    body = "\n".join(out_lines)
    text = BANNER.format(blob=PINNED_BLOB[:8], sha=PINNED_SHA256[:16] + "…") + body
    with open(OUT, "w") as f:
        f.write(text)

    print(f"wrote {OUT}")
    print(f"  source blob   {PINNED_BLOB}  sha256 {PINNED_SHA256}")
    print(f"  derived file  sha256 {hashlib.sha256(text.encode()).hexdigest()}")
    print(f"  single change at line {hits[0]+1}:")
    print(f"    -  {OLD}")
    print(f"    +  {NEW}")

    # prove the import is now inert with respect to the predecessor's record
    log_path = os.path.join(REPO, "conformance/omega/selector4.log")
    before = os.path.getsize(log_path)
    sys.path.insert(0, os.path.dirname(OUT))
    import s4core  # noqa: F401
    after = os.path.getsize(log_path)
    if before != after:
        sys.exit(f"ABORT: importing s4core changed selector4.log ({before} -> {after})")
    print(f"  import verified inert: selector4.log unchanged at {before} bytes")
    for name in ("build_world", "separates", "knobs_partial_sections", "closed_at",
                 "cycles_of", "inv_perm", "acts_are_bijections", "divisors",
                 "knobs_view_aligned", "gauge_invariant_view", "refines"):
        if not hasattr(s4core, name):
            sys.exit(f"ABORT: s4core is missing {name}")
    print("  all criterion entry points present")


if __name__ == "__main__":
    main()
