"""Every function defined in this lane, and who calls it.

The inert-FIELD audit asked what is emitted and never read.  This is the same
question about CODE: what is defined, tested, and called by nothing.  It was
written the day `_install_safe()` turned out to be exactly that -- correct,
covered by a test, and on no path any job took.
"""
import ast
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MODS = [f for f in sorted(os.listdir(HERE))
        if f.endswith(".py") and not f.startswith("_")]
SRC = {m: open(os.path.join(HERE, m)).read() for m in MODS}

defs = []
for m, s in SRC.items():
    try:
        tree = ast.parse(s)
    except SyntaxError:
        continue
    for node in ast.walk(tree):
        if isinstance(node, ast.FunctionDef):
            defs.append((m, node.name, node.lineno))

TESTS = {m for m in MODS if m.startswith("test_")}
unreferenced, tests_only = [], []
for m, name, ln in defs:
    if (name.startswith("test_") or m.startswith("test_")
            or name in ("main", "__init__", "__call__")):
        continue                          # test helpers are used by their test
    hits = {}
    for mm, ss in SRC.items():
        # REFERENCES, not calls: a worker handed to Pool.map is never called
        # by name, and a cross-module call wears its module prefix.  Counting
        # only bare `name(` was this audit's own first defect.
        # a FULL dotted chain: `self.B.sigma_f64` is a reference, and the
        # single-qualifier version of this pattern said it was not.  Every
        # loosening here came from a false positive it produced.
        n = len(re.findall(r"(?<![\w.])(?:[A-Za-z_]\w*\.)*%s\b"
                           % re.escape(name), ss))
        if mm == m:
            n -= 1                       # its own definition
        if n > 0:
            hits[mm] = n
    live = {k: v for k, v in hits.items() if k not in TESTS}
    if not hits:
        unreferenced.append((m, name, ln))
    elif not live:
        tests_only.append((m, name, ln, sorted(hits)))

print("%d functions across %d modules" % (len(defs), len(MODS)))
print("\nCALLED BY NOTHING AT ALL (%d):" % len(unreferenced))
for m, n, ln in unreferenced:
    print("   %s:%d  %s" % (m, ln, n))
print("\nCALLED ONLY BY TESTS -- the `_install_safe` shape (%d):"
      % len(tests_only))
for m, n, ln, who in tests_only:
    print("   %s:%d  %s   (only %s)" % (m, ln, n, ", ".join(who)))
sys.exit(1 if tests_only else 0)
