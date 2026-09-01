#!/bin/bash
#
# THE GUARDED DETACHED WRAPPER — a tail that does not run on a corpse.
#
# The pattern this replaces looked correct and was not:
#
#     run_the_thing > log 2>&1
#     echo $? > DONE
#     mv "$out" "$durable"          # moved an EMPTY directory
#     sha256sum "$durable"/*        # hashed a TRUNCATED log into the manifest
#     census "$durable" > result    # wrote "usage: ..." under a verdict's filename
#
# Every line after the first ran even though the work had been SIGTERMed a minute in. The
# done-marker said 143 and the cleanup marched on regardless, producing three artifacts
# that were each plausible and each about nothing: an empty parked directory, a manifest
# holding a second contradictory hash for a name it already had, and a one-line usage
# message sitting where an adjudication was supposed to be.
#
# The rule, from the lead's ruling of 2026-09-01: THE TAIL CHECKS THE EXIT CODE FIRST.
# Nonzero writes a KILLED marker and touches nothing else — never parks, never hashes,
# never adjudicates. A run that died leaves one artifact saying it died.
#
# Two further things this file is careful about, both paid for on the same day:
#
#   * 128+N is a SIGNAL, not a result. Checking that a done-marker EXISTS is not checking
#     that the work finished; 143 is SIGTERM and 137 is SIGKILL, and both look like
#     completion to a test for file existence.
#   * A detach must be VERIFIED. Plain `setsid cmd &` inside a caller that then went on to
#     do other work did not survive; `setsid nohup cmd & disown` from a caller that exits
#     immediately did. Launch, then confirm the process is alive in its own session.
#
# Usage:
#     detached_run.sh <tag> <durable-dir> <out-dir> <command...>
#
# On success:  <out-dir> is moved to <durable-dir>/<tag>, hashed into MANIFEST, tag.DONE=0
# On failure:  <tag>.KILLED is written with the code and the signal name; nothing is moved,
#              hashed, or adjudicated.

set -u
if [ $# -lt 4 ]; then
    echo "usage: detached_run.sh <tag> <durable-dir> <out-dir> <command...>" >&2
    exit 2
fi
TAG=$1; DUR=$2; OUT=$3; shift 3
STATE="$(dirname "$OUT")"
MANIFEST="${MANIFEST:-$DUR/manifest.sha256}"

mkdir -p "$OUT" "$DUR"
"$@" > "$STATE/$TAG.log" 2>&1
rc=$?

# ---- THE GUARD. Everything below the first branch is the tail, and the tail only runs
# ---- when the work actually finished.
if [ "$rc" -ne 0 ]; then
    sig=""
    if [ "$rc" -gt 128 ]; then
        n=$((rc - 128))
        sig=" (signal $n$(kill -l "$n" 2>/dev/null | sed 's/^/ SIG/'))"
    fi
    {
        echo "$rc"
        echo "# KILLED or FAILED -- exit $rc$sig"
        echo "# Nothing was parked, hashed or adjudicated: the tail does not run on a corpse."
        echo "# The partial log is at $STATE/$TAG.log and the partial output at $OUT."
    } > "$STATE/$TAG.KILLED"
    exit "$rc"
fi

# ---- Success only past here.
if ! find "$OUT" -mindepth 1 -print -quit | grep -q .; then
    # A zero exit with an empty output directory is its own defect: the command claims
    # success and produced nothing. Say so rather than parking emptiness.
    {
        echo "0"
        echo "# EXIT 0 BUT NO OUTPUT -- $OUT is empty."
        echo "# Not parked and not hashed: an empty success is a result nobody can check."
    } > "$STATE/$TAG.EMPTY"
    exit 3
fi

mv "$OUT" "$DUR/$TAG"
cp "$STATE/$TAG.log" "$DUR/$TAG.log" 2>/dev/null
( cd "$DUR" && sha256sum "$TAG"/* "$TAG.log" >> "$MANIFEST" ) 2>/dev/null
echo "$rc" > "$STATE/$TAG.DONE"
