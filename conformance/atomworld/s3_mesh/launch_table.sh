#!/usr/bin/env bash
# SATURATION-3 table launch, under the full provenance discipline.
#
# M-PROVENANCE-OVERREACH is the misfit this script exists to satisfy, and its lesson is
# specific: a header that printed the binary's TRUE sha256 beside "repo HEAD = <hash>" was
# MORE CONFIDENTLY WRONG than the timestamps it replaced, because the build immediately
# before had FAILED and the bytes were an earlier build. Every byte in that header was
# true; the relationship it implied was not.
#
# So this script:
#   1. BUILDS FIRST and captures the build's exit status;
#   2. REFUSES TO LAUNCH on a failed build, rather than printing a warning beside a stale
#      binary and starting a multi-hour run anyway;
#   3. pins the binary's sha256 (the thing actually measured);
#   4. records HEAD and whether the tree is DIRTY -- both labelled as what they are;
#   5. lets the BINARY echo its own parameters, which is the half that closes the gap: the
#      header pins the bytes, the echo says what those bytes were asked to do.
#
# Detached per the standing rule: setsid, done-markers, RESUME. Session death must only
# kill narration, never computation.
set -uo pipefail

if [ $# -lt 2 ]; then
    echo "usage: $0 <label> <s3_tables args...>" >&2
    echo "   e.g: $0 hhcl --species H,H,Cl --x 2.0:6.0 ... --out .../hhcl.tbl" >&2
    exit 64
fi

LABEL="$1"; shift
ENGINE=/home/emoore/CIRISHolon/engine
OUTDIR=/home/emoore/CIRISHolon/engine/output/saturation3
BIN="$ENGINE/target/release/s3_tables"
LOG="$OUTDIR/$LABEL.log"
mkdir -p "$OUTDIR"

# ---- 0: SNAPSHOT EVERY SCRIPT THIS RUN WILL READ, then re-exec from the snapshot.
#
# Bash reads a script INCREMENTALLY BY BYTE OFFSET. Editing a running script moves the
# ground under its read position: the remainder executes from the new bytes at the old
# offset. gpu-prod lost a summary block to exactly this and the twelve readings under it
# had to be recomputed by hand.
#
# TWO exposures here, and the second is the one that matters:
#
#   1. THIS FILE, for as long as the build below takes. Minutes, but real.
#   2. `run_conditions.sh`, which the DETACHED WRAPPER sources when the binary returns --
#      possibly HOURS later. That file lives in `conformance/lib/` and was published for
#      every lane in the fleet to use, so it is not merely mutable, it is EXPECTED to
#      change. A multi-hour run that reads a shared helper at exit is reading whatever the
#      fleet happened to leave there, including a half-written file.
#
# The existing header pinned the BINARY's sha256 and said nothing about the scripts, so a
# mutated launcher or helper was invisible to the provenance -- M-PROVENANCE-OVERREACH's
# shape in the one place that header did not look. Both are now copied read-only into a
# run-scoped directory, sha256'd into the header, and the run reads ONLY the copies.
# LIMIT, stated rather than guarded: the snapshot dir is per-LABEL and is cleared on
# launch, so launching the same label twice concurrently would clear the first run's copies.
# On Linux the running bash holds the inodes open and survives it, but the provenance dir
# would then describe the second run. One run per label is the existing assumption; this
# does not add a lock, because an invented lock is a new thing that can be wrong.
SNAPDIR="$OUTDIR/$LABEL.launch"
LIBSRC=/home/emoore/CIRISHolon/conformance/lib/run_conditions.sh
if [ "${S3_LAUNCH_SNAPSHOT:-}" != "1" ]; then
    rm -rf "$SNAPDIR"; mkdir -p "$SNAPDIR"
    cp "$0" "$SNAPDIR/launch_table.sh" || { echo "cannot snapshot the launcher" >&2; exit 71; }
    cp "$LIBSRC" "$SNAPDIR/run_conditions.sh" || { echo "cannot snapshot run_conditions.sh" >&2; exit 71; }
    chmod 444 "$SNAPDIR/launch_table.sh" "$SNAPDIR/run_conditions.sh"
    export S3_LAUNCH_SNAPSHOT=1
    exec bash "$SNAPDIR/launch_table.sh" "$LABEL" "$@"
fi

# From here on we ARE the snapshot, and every path below reads the snapshot's copies.
source "$SNAPDIR/run_conditions.sh"
LAUNCHER_SHA=$(sha256sum "$SNAPDIR/launch_table.sh" | awk '{print $1}')
CONDITIONS_SHA=$(sha256sum "$SNAPDIR/run_conditions.sh" | awk '{print $1}')

# ---- 1 & 2: build, capture status, refuse on failure.
echo "building s3_tables (release)..." >&2
BUILD_OUT=$(cd "$ENGINE" && nice -n 19 cargo build --release -p holon-tables --bin s3_tables -j 6 2>&1)
BUILD_STATUS=$?
if [ $BUILD_STATUS -ne 0 ]; then
    echo "=== LAUNCH REFUSED ===" | tee "$LOG"
    echo "build exit status: $BUILD_STATUS" | tee -a "$LOG"
    echo "$BUILD_OUT" | tail -30 | tee -a "$LOG"
    echo "" | tee -a "$LOG"
    echo "The binary on disk is STALE relative to HEAD. Launching it would start a" | tee -a "$LOG"
    echo "multi-hour run whose provenance header would assert a relationship nothing" | tee -a "$LOG"
    echo "verified -- M-PROVENANCE-OVERREACH exactly. Fix the build and re-launch." | tee -a "$LOG"
    exit 70
fi

# ---- 3, 4: the header. Measured facts and labelled inferences, kept apart.
SHA=$(sha256sum "$BIN" | cut -d' ' -f1)
HEAD_HASH=$(cd "$ENGINE" && git rev-parse HEAD)
DIRTY=$(cd "$ENGINE" && git status --porcelain | wc -l)

{
    echo "=== SATURATION-3 TABLE LAUNCH: $LABEL ==="
    echo "launched          $(date -Is)"
    echo "binary            $BIN"
    echo "binary sha256     $SHA                     [MEASURED]"
    echo "build exit status $BUILD_STATUS                     [MEASURED — 0, so the bytes above are of HEAD's source]"
    echo "repo HEAD         $HEAD_HASH  [MEASURED]"
    if [ "$DIRTY" -gt 0 ]; then
        echo "working tree      DIRTY, $DIRTY path(s)          [MEASURED]"
        echo "                  ^ the binary was built from the WORKING TREE, not from HEAD."
        echo "                    HEAD above is context, NOT a claim that these bytes are HEAD's."
    else
        echo "working tree      clean                      [MEASURED]"
        echo "                  so the binary corresponds to HEAD [INFERRED from clean+build-ok]"
    fi
    echo "launcher sha256   $LAUNCHER_SHA  [MEASURED — snapshot, read-only]"
    echo "conditions sha256 $CONDITIONS_SHA  [MEASURED — snapshot, read-only]"
    echo "                  ^ both copied into $SNAPDIR and executed from there, so an edit"
    echo "                    to the shared originals cannot reach this run."
    echo "command           s3_tables $*"
    run_conditions "at launch"
    echo "=== end header; the binary's own parameter echo follows ==="
    echo ""
} | tee "$LOG"

# ---- 5: run detached; the binary echoes its parameters as IT parsed them.
rm -f "$OUTDIR/$LABEL.DONE"
setsid nice -n 19 bash -c "
    source '$SNAPDIR/run_conditions.sh'
    '$BIN' $* >> '$LOG' 2>&1
    rc=\$?
    echo \"exit=\$rc\" >> '$LOG'
    run_conditions 'at exit' >> '$LOG' 2>&1
    touch '$OUTDIR/$LABEL.DONE'
" < /dev/null > /dev/null 2>&1 &

echo "detached. log: $LOG   marker: $OUTDIR/$LABEL.DONE"
