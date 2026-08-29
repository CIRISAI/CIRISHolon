"""Which fields does the referee EMIT that nothing is obliged to READ?

The engine lane found a worst_residual that its PairMeta had recorded and
emitted for the whole campaign with nothing ever checking it: a curve whose
Davidson hit its iteration cap would have shipped looking healthy, with the
evidence sitting in a field no consumer had to read.  The diagnostic was
present and inert.

This is the reciprocal audit on this side: walk every key the emitter writes
and every key the verifier and the emitter's own guards actually consume, and
list the difference.  A field in the gap is not necessarily wrong -- some are
provenance for a human -- but each one should be there because someone decided
it is provenance, not because nobody noticed it was never checked.
"""
import json
import os
import re

HERE = os.path.dirname(os.path.abspath(__file__))
DROP = os.path.join(HERE, "engine_handoff", "elements1")


def walk(obj, prefix=""):
    out = set()
    if isinstance(obj, dict):
        for k, v in obj.items():
            p = "%s.%s" % (prefix, k) if prefix else k
            out.add(p)
            if isinstance(v, (dict, list)):
                out |= walk(v, p)
    elif isinstance(obj, list) and obj and isinstance(obj[0], dict):
        out |= walk(obj[0], prefix + "[]")
    return out


emitted = set()
for f in ("H2.json", "F2.json", "atoms.json", "manifest.json"):
    p = os.path.join(DROP, f)
    if os.path.exists(p):
        emitted |= walk(json.load(open(p)))

# THE WRITER IS NOT A READER.  Counting emit_engine.py as a reader made every
# key it names literally look consumed -- which is every key it writes, which
# is the whole file.  The audit then reported only the keys built somewhere
# else, and would have called a brand-new inert block "read" on the strength of
# the line that wrote it.  Readers are the standalone verifier and the engine
# lane's own test, which are the two things obliged to consume this drop.
READER_FILES = [os.path.join(HERE, "verify_elements.py"),
                "/home/emoore/CIRISHolon/engine/crates/holon-chem/tests/pair.rs"]
readers = ""
for f in READER_FILES:
    if os.path.exists(f):
        readers += open(f).read()
    else:
        print("NOTE: reader not found, audit is over the rest: %s" % f)

# A key can be consumed in two different places and they are not the same
# promise.  READ means a downstream consumer takes it out of the emitted file.
# GUARDED means the emitter refuses to write the file when it is wrong, which
# protects the drop but leaves nothing checking it afterwards.  Only "neither"
# is inert -- and lumping the two together (or counting the writer as a reader,
# which this audit did until today) hides which promise a field actually has.
emitter = open(os.path.join(HERE, "emit_engine.py")).read()
guard_zones = ""
lines = emitter.splitlines()
for i, ln in enumerate(lines):
    if "raise AssertionError" in ln:
        guard_zones += "\n".join(lines[max(0, i - 6):i + 8]) + "\n"

read, guarded, inert = [], [], []
for key in sorted(emitted):
    leaf = key.split(".")[-1].replace("[]", "")
    if not leaf or leaf.endswith("note") or leaf.endswith("_note"):
        continue
    pat = r'["\']%s["\']' % re.escape(leaf)
    if re.search(pat, readers):
        read.append(key)
    elif re.search(pat, guard_zones):
        guarded.append(key)
    else:
        inert.append(key)

print("keys emitted: %d" % len(emitted))
print("  read by a consumer of the file: %d" % len(read))
print("  guarded at write time only:     %d" % len(guarded))
for k in guarded:
    print("     %s" % k)
print("  NEITHER read nor guarded:       %d" % len(inert))
for k in inert:
    print("     %s" % k)
