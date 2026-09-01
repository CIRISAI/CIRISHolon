#!/bin/bash
# Refresh the COMMITTED MIXTURES-1 referee from the WORKING copy, and say what
# moved.  Two copies of anything drift; the answer is to make the refresh
# mechanical and the diff visible, not to remember.
#
# THE DIRECTION IS HARDCODED AND THE SAME-DIRECTORY CASE IS REFUSED.
# The first version took the destination from `dirname $0`, so running the copy
# that lives in the working directory made source and destination the same path.
# It compared every file with itself, reported "0 files refreshed", and exited 0.
# A sync that silently does nothing is worse than one that fails, because the
# next thing you do is trust the destination.
# M-STALE-INSTRUMENT (widened): the old default was a per-session scratchpad
# path that dies with the session that wrote it. There is no durable default;
# the working dir must be named explicitly.
if [ -z "${MIXTURES1_WORKING:-}" ]; then
  echo "REFUSING: MIXTURES1_WORKING is unset and the old default was a dead" >&2
  echo "per-session scratchpad path. Export your live working dir, e.g.:" >&2
  echo "  export MIXTURES1_WORKING=/path/to/your/mixtures_referee" >&2
  exit 2
fi
D="$MIXTURES1_WORKING"
R="${MIXTURES1_REPO:-/home/emoore/CIRISHolon/conformance/atomworld/mixtures1_referee}"
FILES="README.md RESUME.md FEASIBILITY.md basis2.py species2.py species.py
       m1core.py build_atoms2.py curves2.py emit2.py env.sh run_pairs.sh
       test_basis_matches_engine.py test_species_shim.py verify2.py
       _cost_probe.py _sio_stream.py _conditioning.py _rss_guard.sh
       _fast_elements.py _routeb_cost.py mixtures_atoms.json
       _inert_audit2.py prose_fields2.txt sync_repo_copy.sh"
[ -d "$D" ] || { echo "working copy not found: $D"; exit 1; }
[ -d "$R" ] || { echo "repo copy not found: $R"; exit 1; }
if [ "$(cd "$D" && pwd -P)" = "$(cd "$R" && pwd -P)" ]; then
  echo "REFUSING: source and destination are the same directory"
  echo "  $D"
  exit 2
fi
changed=0
for f in $FILES; do
  [ -f "$D/$f" ] || { [ -f "$R/$f" ] && echo "  only in repo: $f"; continue; }
  if ! cmp -s "$D/$f" "$R/$f"; then
    echo "  updated: $f"; cp "$D/$f" "$R/$f"; changed=$((changed+1))
  fi
done
# The elements1 symlink is NOT synced: in the working copy it points at the
# scratchpad lane, in the repo at the committed ELEMENTS-1 referee beside it.
# Two different correct answers to the same question.
[ -L "$R/elements1" ] || ln -sfn ../elements1_referee "$R/elements1"
echo "$changed file(s) refreshed"
echo "now verify the repo copy:"
echo "  cd $R && . ./env.sh && python3 test_basis_matches_engine.py && python3 test_species_shim.py"
