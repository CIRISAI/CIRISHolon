#!/bin/bash
# Refresh the committed referee source from this working copy, and SAY what
# moved.  Two copies of anything drift; the answer is to make the refresh
# mechanical and the diff visible, not to remember.
#
# The one file that is deliberately NOT synced is verify_elements.py's import
# shim: the committed copy adds the parent directory to sys.path so it can
# import the banked h2_core.py, which does not exist beside this working copy.
D="$(cd "$(dirname "$0")" && pwd)"
R=/home/emoore/CIRISHolon/conformance/atomworld/elements1_referee
FILES="elements_core.py fci.py curve.py runner.py species.py build_curves.py
       build_atoms.py emit_engine.py verify_elements.py plants.py
       _inert_audit.py _dead_guard_audit.py prose_fields.txt run_final.sh
       test_integrals.py test_fci.py test_runlock.py test_pmap_safety.py
       test_emit_refusals.py test_verify_sections.py RESUME.md README.md
       elements_atoms.json elements_plants.json"
changed=0
for f in $FILES; do
  [ -f "$D/$f" ] || { [ -f "$R/$f" ] && echo "  only in repo: $f"; continue; }
  case "$f" in
    verify_elements.py|test_integrals.py)
      # These two ALWAYS differ from the working copy by their import shim, so
      # cmp would report them every run.  Copy and re-shim, and say which it is.
      cp "$D/$f" "$R/$f"; echo "  refreshed + re-shimmed: $f"; continue ;;
  esac
  if ! cmp -s "$D/$f" "$R/$f"; then echo "  updated: $f"; cp "$D/$f" "$R/$f"; changed=$((changed+1)); fi
done
for f in elements_potential.json elements_potential_partial.json; do
  if [ -f "$D/$f" ] && ! cmp -s "$D/$f" "$R/$f"; then echo "  updated: $f"; cp "$D/$f" "$R/$f"; changed=$((changed+1)); fi
done
# re-apply the two import shims the repo copy needs and the working copy does not
python3 - "$R" <<'PY'
import os, sys
R = sys.argv[1]
p = os.path.join(R, "verify_elements.py"); s = open(p).read()
old = "sys.path.insert(0, HERE)"
add = ("sys.path.insert(0, HERE)\n"
       "# h2_core.py is the BANKED foundation and lives one level up beside the\n"
       "# freeze it belongs to.  Import it from there rather than keeping a copy\n"
       "# here: two copies of a bank is how a bank stops being one.\n"
       "sys.path.insert(1, os.path.dirname(HERE))")
if "os.path.dirname(HERE))" not in s:
    open(p, "w").write(s.replace(old, add, 1)); print("  re-applied: verify_elements.py h2_core import shim")
p = os.path.join(R, "test_integrals.py"); s = open(p).read()
old = """import sys
import itertools
"""
add = """import os
import sys
import itertools

# h2_core.py is the BANKED foundation and lives one level up beside the freeze
# it belongs to; import it from there rather than keeping a second copy.
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(1, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
"""
if "os.path.dirname(os.path.dirname" not in s:
    open(p, "w").write(s.replace(old, add, 1)); print("  re-applied: test_integrals.py h2_core import shim")
PY
echo "$changed file(s) refreshed; now run the repo copy:"
echo "  cd $R && python3 verify_elements.py --quick"
