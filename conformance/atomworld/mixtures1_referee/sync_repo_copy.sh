#!/bin/bash
# Refresh the committed MIXTURES-1 referee from the working copy, and SAY what
# moved.  Two copies of anything drift; the answer is to make the refresh
# mechanical and the diff visible, not to remember.
#
# The `elements1` symlink is NOT synced: in the working copy it points at the
# scratchpad lane, and in the repo it points at the COMMITTED ELEMENTS-1 referee
# beside it.  Those are two different correct answers to the same question, and
# copying one over the other would silently repoint a whole campaign.
D="${MIXTURES1_WORKING:-/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/mixtures_referee}"
R="$(cd "$(dirname "$0")" && pwd)"
FILES="README.md RESUME.md basis2.py species2.py species.py m1core.py
       build_atoms2.py curves2.py emit2.py env.sh run_pairs.sh
       test_basis_matches_engine.py test_species_shim.py
       _cost_probe.py _sio_stream.py _conditioning.py _rss_guard.sh
       FEASIBILITY.md _fast_elements.py _routeb_cost.py mixtures_atoms.json"
[ -d "$D" ] || { echo "working copy not found: $D"; exit 1; }
changed=0
for f in $FILES; do
  [ -f "$D/$f" ] || { [ -f "$R/$f" ] && echo "  only in repo: $f"; continue; }
  if ! cmp -s "$D/$f" "$R/$f"; then
    echo "  updated: $f"; cp "$D/$f" "$R/$f"; changed=$((changed+1))
  fi
done
[ -L "$R/elements1" ] || ln -sfn ../elements1_referee "$R/elements1"
echo "$changed file(s) refreshed; now run the repo copy:"
echo "  cd $R && . ./env.sh && python3 test_basis_matches_engine.py && python3 test_species_shim.py"
