"""Every key this campaign emits, split into read / guarded-at-write / inert.

STANDING QUESTION 3, and the trap in it.

The question is "does anything read this?", and the trap is that the natural way
to answer it puts the WRITER in the reader set. `emit2.py` names every key it
writes, literally, in the dict it builds -- so a grep for the key name across
the source finds it, and every field looks consumed. ELEMENTS-1's audit had
exactly this defect and reported all its keys as read; correcting it turned up
six that nothing consumed.

So the writer is excluded here by construction, and the reader set is only the
things that would actually USE a field: the verifier, the tests, and the
engine-side consumers. What comes out is three buckets, and the middle one
matters as much as the outside ones:

  read              -- something downstream reads it
  guarded-at-write  -- nothing reads it, but the emitter REFUSES a file whose
                       value is wrong. That is not an unread field; it is a
                       field whose reader is an assertion.
  inert             -- nothing reads it and nothing checks it. Every one of
                       these must be deliberate human provenance and named in
                       `prose_fields2.txt`, which is a SEPARATE FILE because
                       naming a field inside the auditor would launder it out
                       of this bucket.

An inert field that is not on the allowlist fails the audit.
"""
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
WRITER = "emit2.py"
READERS = ["verify2.py", "test_basis_matches_engine.py",
           "test_species_shim.py", "species2.py", "m1core.py", "basis2.py",
           "species.py", "curves2.py"]
# The engine side is a reader too, and not being able to see it is a fact worth
# reporting rather than an excuse to call a field read.
ENGINE = ["/home/emoore/CIRISHolon/engine/crates/holon-chem/tests",
          "/home/emoore/CIRISHolon/engine/crates/holon-chem/src"]
ALLOWLIST = os.path.join(HERE, "prose_fields2.txt")


def emitted_keys():
    """Every JSON key `emit2.py` writes, from its source."""
    src = open(os.path.join(HERE, WRITER)).read()
    keys = set()
    # dict literals with string keys, and obj["key"] = ... assignments
    for m in re.finditer(r'"([a-z][a-z0-9_]{2,})"\s*:', src):
        keys.add(m.group(1))
    for m in re.finditer(r'\[\s*"([a-z][a-z0-9_]{2,})"\s*\]\s*=', src):
        keys.add(m.group(1))
    return keys


def guarded_keys():
    """Keys the emitter refuses a file over -- their reader is an assertion."""
    src = open(os.path.join(HERE, WRITER)).read()
    out = set()
    for m in re.finditer(r'if\s+([a-z_]*\.?get\(\s*"([a-z0-9_]+)"|[a-z_]+\['
                         r'\s*"([a-z0-9_]+)"\s*\])', src):
        out.add(m.group(2) or m.group(3))
    # the explicit refusals in this file's flow
    for m in re.finditer(r'\.get\("([a-z0-9_]+)"\)\s*!=', src):
        out.add(m.group(1))
    return out


def reader_text():
    parts = []
    for fn in READERS:
        p = os.path.join(HERE, fn)
        if os.path.exists(p):
            parts.append(open(p).read())
    seen_engine = False
    for d in ENGINE:
        if not os.path.isdir(d):
            continue
        for root, _, files in os.walk(d):
            for f in files:
                if f.endswith((".rs", ".json", ".txt")):
                    try:
                        parts.append(open(os.path.join(root, f),
                                          errors="ignore").read())
                        seen_engine = True
                    except OSError:
                        pass
    return "\n".join(parts), seen_engine


def main():
    keys = emitted_keys()
    guarded = guarded_keys()
    text, seen_engine = reader_text()
    allow = set()
    if os.path.exists(ALLOWLIST):
        allow = {l.split("#")[0].strip() for l in open(ALLOWLIST)
                 if l.split("#")[0].strip()}

    # A KEY IS READ WHEN IT IS NAMED AS A KEY, not when its letters occur.
    #
    # The first version matched on a word boundary, and half this manifest's
    # key names are ordinary English -- "model", "gate", "kill", "scope",
    # "stake", "rules", "atoms", "basis", "pairs", "precision".  Every one of
    # them appears somewhere in a source tree of that size, so the audit
    # reported them read and the INERT bucket came out almost empty.  An audit
    # that cannot come out non-empty is not an audit.
    #
    # A consumer names a JSON field as a quoted string or as an attribute
    # access, so that is what counts.
    read, gwrite, inert = [], [], []
    for k in sorted(keys):
        pat = r'["\'\[]%s["\'\]]|\.%s\b|get\(\s*["\']%s["\']' % (
            re.escape(k), re.escape(k), re.escape(k))
        if re.search(pat, text):
            read.append(k)
        elif k in guarded:
            gwrite.append(k)
        else:
            inert.append(k)

    print("writer excluded from the reader set: %s" % WRITER)
    print("readers scanned: %s%s"
          % (", ".join(READERS),
             " + the engine tree" if seen_engine
             else "  (ENGINE TREE NOT FOUND -- a field it reads would be "
                  "misreported as inert)"))
    print()
    print("READ (%d): %s" % (len(read), ", ".join(read)))
    print()
    print("GUARDED AT WRITE (%d): %s" % (len(gwrite), ", ".join(gwrite)))
    print()
    print("INERT (%d): %s" % (len(inert), ", ".join(inert)))
    stray = [k for k in inert if k not in allow]
    print()
    if stray:
        print("FAIL: %d inert field(s) not named in %s:"
              % (len(stray), os.path.basename(ALLOWLIST)))
        for k in stray:
            print("   %s" % k)
        print("An inert field is either prose written for a person -- in which "
              "case name it -- or it is a field nobody wanted.")
        return 1
    print("PASS: every inert field is named as deliberate human provenance "
          "(%d allowlisted)" % len(allow))
    return 0


if __name__ == "__main__":
    sys.exit(main())
