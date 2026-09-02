#!/bin/bash
# Workspace invariants, each one empirically derived rather than assumed.
# Run from the workspace root.
set -u; fail=0
ok(){ printf "  PASS  %s\n" "$1"; }
no(){ printf "  FAIL  %s\n" "$1"; fail=1; }

# 1. The no_std core must build for native and BOTH wasm targets, on its own.
#    Use -p, never --workspace: a sibling's features can pull in a C++ build script.
#
#    Same disease as gate 9's, caught in the same pass: fracture/impact/runtime/descriptor
#    are all `#[cfg(feature = "alloc")]`, and this loop only ever built DEFAULT features —
#    so the three-target BUILD guarantee never compiled the adaptive-fracture line either,
#    on any target, even though gate 9 now tests it. Fixing the test line while leaving
#    this one blind would leave half the hole open. Both feature sets, both matter: default
#    is the true no_std/no-alloc floor (a consumer with no allocator needs THIS to build);
#    alloc is what fracture/impact actually need.
for T in "" "--target wasm32-unknown-unknown" "--target wasm32-wasip1"; do
  cargo build -q -p ciris-sim-core $T 2>/dev/null && ok "core builds ${T:-native}" || no "core builds ${T:-native}"
  cargo build -q -p ciris-sim-core --features alloc $T 2>/dev/null \
    && ok "core builds ${T:-native} (alloc)" || no "core builds ${T:-native} (alloc)"
done

# 2. The core's dependency graph must be EXACTLY the permitted set.
#    Rationale: the invariant is not "zero deps" — that threshold was calibrated against
#    a stub. It is "nothing that can unify with a sibling's features, pull an allocator,
#    or require a C/C++ toolchain". `libm` is permitted because it is the no_std math
#    implementation the core needs (fabs/sqrt in the eigensolver) and its entire graph is
#    itself, with no transitive dependencies. An allowlist keeps the teeth a count loses:
#    this fails on ANY new name, including an innocuous-looking one.
ALLOWED="libm"
got=$(cargo tree -p ciris-sim-core --edges normal --prefix none 2>/dev/null \
      | grep -v '^$' | sed 's/ v[0-9].*//' | grep -v '^ciris-sim-core' | sort -u | tr '\n' ' ' | sed 's/ $//')
[ "$got" = "$ALLOWED" ] && ok "core deps are exactly {$ALLOWED}" || no "core deps are {$got}, expected {$ALLOWED}"

# 3. Default features must not pull an inference engine: the physics core has to be
#    buildable on a machine with no cmake and no C++ toolchain.
d=$(cargo tree -p ciris-nl --edges normal --prefix none 2>/dev/null | grep -cE "llama-cpp|rten")
[ "$d" -eq 0 ] && ok "default build pulls no engine" || no "default build pulls $d engine crates"

# 4. The browser path must actually compile for wasm.
cargo build -q -p ciris-nl --features web --target wasm32-unknown-unknown 2>/dev/null \
  && ok "web feature -> wasm32-unknown-unknown" || no "web feature -> wasm32-unknown-unknown"

# 5. Grammar invariants, and the closed-set decode filter used by the browser path.
cargo test -q -p ciris-nl 2>/dev/null >/dev/null && ok "grammar tests" || no "grammar tests"
cargo test -q -p ciris-nl --features web 2>/dev/null >/dev/null \
  && ok "web decode-filter tests" || no "web decode-filter tests"

# 6. The native path must still build (it is feature-gated and easy to break silently).
cargo build -q -p ciris-nl --features native 2>/dev/null \
  && ok "native feature builds" || no "native feature builds"

# 7. The interactive fracture gate must execute in native tests and compile as the same
#    raw Rust module loaded by the browser. Rendering is screenshot-gated in the release
#    artifact; this protects the physics/WASM half on every commit.
cargo test -q -p holon-ball-game 2>/dev/null >/dev/null \
  && ok "holon ball/material fracture tests" || no "holon ball/material fracture tests"
cargo build -q -p holon-ball-game --release --target wasm32-unknown-unknown 2>/dev/null \
  && ok "holon ball game -> wasm32-unknown-unknown" || no "holon ball game -> wasm32-unknown-unknown"

# 8. The multiscale sandbox. Its tests carry the certifier equivalence gate (incremental
#    vs the shipped `certify_runtime_adaptive`, bit-for-bit over one model), the five
#    planted mutants that gate has to catch, the ledger arithmetic that fixes the zoom
#    ladder, and the energy/landing gates on the solver. Same shape as gate 7: physics
#    and WASM on every commit, rendering screenshot-gated in the artifact.
cargo test -q -p holon-sandbox --release 2>/dev/null >/dev/null \
  && ok "holon sandbox certifier/ledger/solver gates" || no "holon sandbox certifier/ledger/solver gates"
cargo build -q -p holon-sandbox --release --target wasm32-unknown-unknown 2>/dev/null \
  && ok "holon sandbox -> wasm32-unknown-unknown" || no "holon sandbox -> wasm32-unknown-unknown"

# 9. The no_std core's own test suite (sectors, runtime, relativity, linalg, sparse,
#    locality, descriptor, regplus, impact, bridge, dynamics, fracture, material, field,
#    data, twin_probe, quantum_link, curvature, mechanical, gaps, holon, homogenization,
#    structure — one #[test] module per file) never actually ran under this script: gate
#    1 only builds the crate for three targets, it never tests it. ciris-sim-core IS a
#    member of this workspace, so -p reaches it directly (unlike gate 11's two
#    standalone crates, which need --manifest-path).
#
#    --features alloc is NOT optional: the crate's default feature set is EMPTY, and
#    fracture/impact/runtime/descriptor are all `#[cfg(feature = "alloc")]` — measured
#    118 tests default vs 165 under alloc, and ZERO of them fracture::/impact:: either
#    way under default. A plain `-p ciris-sim-core` here ran green while covering none
#    of the adaptive-fracture line (research-manager-2, 2026-08-24) — the gate must name
#    the feature it means to test, not the crate's cheapest build. `std` stays untested
#    by this gate; that is a separate, smaller gap (`--all-features` would close it) and
#    is not what gate 1's no_std build guarantee depends on.
#
#    --release, same reason gate 8 (holon-sandbox) takes it: impact.rs's three-leg
#    convergence test is a float-heavy numeric solver that Rust's debug profile does not
#    optimize, and it is where the suite's cost concentrates (~5 of 165 tests carry it;
#    compilation itself is cheap in either profile). This is a profile argument, not a
#    timing one deliberately — a same-environment debug-vs-release pair was going to be
#    quoted here and got pulled: two readings taken under different, uncontrolled
#    concurrent build load are not a comparison (research-manager-2, on its own numbers,
#    2026-08-24; the same defect it had just caught in an unrelated warm-start probe).
#    Debug-vs-release is a speed knob here, not a correctness one — the assertions are
#    the same either way.
#
#    SELF-VERIFYING, not self-describing (team-lead's ruling): the defect above was a
#    PROSE claim of coverage next to a COMMAND that did not deliver it — "the command
#    exits 0" cannot distinguish "fracture/impact ran and passed" from "fracture/impact
#    were never compiled in", which is exactly how 47 tests and two modules stayed
#    invisible while this gate ran green. `--list` enumerates the test binary's contents
#    without running anything, so a feature-flag regression that silently drops a module
#    fails THIS assertion instead of only living in a comment that nobody re-checks.
alloc_list=$(cargo test -q -p ciris-sim-core --features alloc --release --lib -- --list 2>/dev/null)
if printf '%s\n' "$alloc_list" | grep -q 'fracture::' && printf '%s\n' "$alloc_list" | grep -q 'impact::'; then
  ok "ciris-sim-core alloc build compiles in fracture::/impact::"
else
  no "ciris-sim-core alloc build compiles in fracture::/impact:: (feature flag regression?)"
fi
cargo test -q -p ciris-sim-core --features alloc --release 2>/dev/null >/dev/null \
  && ok "ciris-sim-core test suite (alloc) passes" || no "ciris-sim-core test suite (alloc) passes"

# 10. The committed viewer wasm IS what the source builds. pages.yml ships the
#     committed binary verbatim with no Rust toolchain in CD, so "what ships is what
#     was gated" holds only if this comparison holds — a 503-byte counterexample sat
#     in the tree until the Jules triage (JULES_3D_TRIAGE.md F6) found that nothing
#     anywhere compared the artifact to its source.
#
#     2026-08-24 postmortem (the "206-byte cross-machine delta"): this gate was never
#     wrong and the two machines were never in disagreement. The committed binary had
#     been built, and then committed, from a WORKING TREE that had uncommitted changes
#     staged in a sibling crate (ciris-sim-core: fracture.rs/impact.rs) belonging to a
#     concurrent, unrelated task sharing this checkout — so the commit shipped a wasm
#     that no clean checkout of its own claimed source can reproduce. CI's checkout is
#     always clean, so it correctly rejected the binary; a "local" rebuild done in the
#     same contaminated tree just reproduced the same contamination and looked like
#     agreement.
#
#     HERMETIC BY CONSTRUCTION (fixed 2026-08-24): the gate used to run build-web.sh
#     straight at the tracked path and `git checkout --` it on failure — a lane-visible
#     mechanism for destroying another lane's uncommitted work in this shared tree
#     (this is the one known mechanism behind a lane's WIP going missing during the
#     outage window; attribution to this gate specifically was never provable, but the
#     mechanism was real and is now gone). The gate now builds to a throwaway scratch
#     path via build-web.sh's HOLON_SANDBOX_WASM_OUT override and diffs bytes straight
#     out of `git show` — it never writes to, and never runs `git checkout` on, the
#     tracked file. This retires the interim rule that ci-gates.sh may only run in a
#     clean worktree; the shared tree's contamination is still a bug in whatever writes
#     uncommitted changes across lanes, but it can no longer be THIS gate's fault.
built_wasm=$(mktemp)
trap 'rm -f "$built_wasm"' EXIT
HOLON_SANDBOX_WASM_OUT="$built_wasm" bash crates/holon-sandbox/build-web.sh >/dev/null 2>&1
if git show "HEAD:./crates/holon-sandbox/viewer/holon_sandbox.wasm" 2>/dev/null | cmp -s - "$built_wasm"; then
  ok "holon sandbox committed wasm matches its source"
else
  # Diagnostic on failure: a blind byte-mismatch cannot be debugged from a CI log.
  echo "    committed: $(git show "HEAD:./crates/holon-sandbox/viewer/holon_sandbox.wasm" 2>/dev/null | sha256sum | cut -c1-16) ($(git show "HEAD:./crates/holon-sandbox/viewer/holon_sandbox.wasm" 2>/dev/null | wc -c) bytes)"
  echo "    built:     $(sha256sum "$built_wasm" | cut -c1-16) ($(wc -c < "$built_wasm") bytes)"
  echo "    rustc:     $(rustc -V)  host: $(rustc -vV | grep host)"
  no "holon sandbox committed wasm matches its source (rerun build-web.sh and commit, FROM A CLEAN TREE)"
fi
rm -f "$built_wasm"
trap - EXIT

# 10a1. MISFIT REGISTRY INTEGRITY: every M- id cited anywhere in the
#      conformance record exists in MISFITS.md, and registry ids are
#      unique. A ghost citation is a broken cross-reference in the
#      record's load-bearing index (see also the cross-reference gate).
reg_fail=0
reg_ids=$(grep -oE '\*\*(M-[A-Z0-9-]+)\*\*' ../conformance/gravity/MISFITS.md | tr -d '*' | sort)
[ "$(echo "$reg_ids" | wc -l)" -eq "$(echo "$reg_ids" | sort -u | wc -l)" ] || { reg_fail=1; echo "  duplicate misfit ids in registry"; }
for id in $(grep -rhoE '\bM-[A-Z0-9-]+\b' ../conformance/ ../Audit/ 2>/dev/null | sort -u); do
  echo "$reg_ids" | grep -qx "$id" || { reg_fail=1; echo "  ghost misfit citation: $id"; }
done
[ "$reg_fail" -eq 0 ] && ok "misfit registry: unique ids, no ghost citations" \
  || no "misfit registry: unique ids, no ghost citations"

# 10a3. NO SESSION-KEYED PATHS IN COMMITTED INSTRUMENTS (M-STALE-INSTRUMENT,
#      widened 2026-09-01): a committed instrument citing a per-session
#      scratchpad resolves from inside the session that wrote it and dies
#      with it -- the defect is invisible to every working-tree check and
#      fails only when followed from OUTSIDE. Executable files only; prose
#      records may cite such paths as history.
sess_hits=$(grep -rln "claude-1000" ../conformance --include="*.py" --include="*.sh" --include="*.rs" 2>/dev/null || true)
if [ -n "$sess_hits" ]; then
  echo "$sess_hits" | sed 's/^/  session-keyed path in: /'
  no "no session-keyed scratch paths in committed instruments"
else
  ok "no session-keyed scratch paths in committed instruments"
fi

# 10a2. CONFORMANCE REPRODUCIBILITY (added after an external re-review found
#      three banked verdicts whose instruments were working-tree-only, one
#      of them never committed at all -- M-STALE-INSTRUMENT). Every gravity
#      instrument must IMPORT from the committed tree, and the fast ones
#      must fresh-run green.
repro_fail=0
( cd ../conformance/gravity && python3 - <<'PYEOF'
import sys, importlib
sys.path.insert(0, ".")
for mod in ("bridge1","bridge5","bridge6","bridge7","wilson1","wilson2",
            "local1","local2","closure2","closure3","einstein_adm1","punctured_torus"):
    importlib.import_module(mod)
PYEOF
) >/dev/null 2>&1 || { repro_fail=1; echo "  prong: instrument imports failed"; }
( cd ../conformance/gravity && timeout 600 python3 einstein_adm1.py >/dev/null 2>&1 ) \
  || { repro_fail=1; echo "  prong: einstein_adm1 fresh-run failed"; }
( cd ../conformance/gravity && timeout 600 python3 punctured_torus.py >/dev/null 2>&1 ) \
  || { repro_fail=1; echo "  prong: punctured_torus fresh-run failed"; }
[ "$repro_fail" -eq 0 ] && ok "gravity instruments import and fast campaigns fresh-run green" \
  || no "gravity instruments import and fast campaigns fresh-run green"

# 10b. holon-zx: the magic tier's canonicalizer, composed (quizx simplifies,
#      holon evaluates). The DEFAULT build is gated here — it must compile
#      and pass with no external dependency, because the whole point of the
#      separate crate is that `holon`'s core stays zero-dep and 65 KB. The
#      `--features zx` build pulls quizx from git and is NOT run in CI: a
#      network fetch is not a gate, it is a flake. Its amplitude certificate
#      (crates/holon-zx/tests/composed.rs) runs locally and its result is
#      recorded in conformance/BENCHMARKS.md entry twenty.
cargo test -q -p holon-zx --release 2>/dev/null >/dev/null \
  && ok "holon-zx default (no-dep passthrough) builds and tests" \
  || no "holon-zx default (no-dep passthrough) builds and tests"

# 11. holon-swarm and holon-mesh each USED TO carry their own empty `[workspace]` table,
#     which made `-p holon-swarm`/`-p holon-mesh` from this root resolve to nothing — no
#     such package existed in this workspace's graph — so neither crate had ever been
#     reached by this script. Both are now real `members` (holon-mesh path-depends on
#     holon-swarm, so cargo refuses two workspace roots in the same graph if only one
#     joins — holon-mesh lane, measured: "multiple workspace roots found"; both joined
#     together, both empty tables removed). Plain -p reaches each directly now. Same
#     shape as gates 7/8: run the tests, then build the release artifact the crate
#     actually ships (a native bin; neither claims a wasm target).
#
#     SELF-VERIFYING (same question asked of gate 9, per team-lead's ruling — "if it can
#     pass while reaching neither crate, it has gate 9's disease"): unlike ciris-sim-core's
#     src-level `#[cfg(test)] mod tests`, these crates' interesting coverage lives in
#     tests/*.rs integration files, whose functions list with BARE names, not a
#     file-derived prefix (measured: tests/determinism.rs and tests/mutation.rs both
#     contribute unprefixed names to `--list`, so a module-prefix grep like gate 9's would
#     not distinguish "reached" from "not reached" here). A nonzero test COUNT is the
#     right assertion for THIS failure mode — "-p resolves to nothing" or "the crate
#     builds but the test binaries collect zero tests" both show up as 0, and unlike an
#     exact count it does not go red every time a test is legitimately added.
n_swarm=$(cargo test -q -p holon-swarm -- --list 2>/dev/null | grep -c ': test$')
[ "${n_swarm:-0}" -gt 0 ] \
  && ok "holon-swarm reaches $n_swarm tests" \
  || no "holon-swarm reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q -p holon-swarm 2>/dev/null >/dev/null \
  && ok "holon-swarm determinism/mutation tests" || no "holon-swarm determinism/mutation tests"
cargo build -q -p holon-swarm --release 2>/dev/null \
  && ok "holon-swarm swarm_bench builds" || no "holon-swarm swarm_bench builds"

# holon: THE recursive data object (planes/ledger/chart/certificate/arena).
# Its tiers certify against the QASM-suite reference tiers; the recursion,
# conditioning, and ledger-ring tests are the design lock's standing evidence.
cargo test -q -p holon 2>/dev/null >/dev/null \
  && ok "holon object: tier conformance, ledger ring, conditioning, recursion" \
  || no "holon object: tier conformance, ledger ring, conditioning, recursion"

# holon-qasm: the stratified QASM simulator. The in-crate three-way tier
# conformance (each cheap tier vs the statevector carrier, exact) plus
# planted-mutation detection is the CI backbone of the QASM suite; the
# externally-refereed record (qiskit ground truth, 12 pre-registered arms,
# all CONFIDENCE) lives upstream in CIRISOntology scratchpad/qasm.
cargo test -q -p holon-qasm 2>/dev/null >/dev/null \
  && ok "holon-qasm tier conformance + mutation detection" \
  || no "holon-qasm tier conformance + mutation detection"

n_mesh=$(cargo test -q -p holon-mesh -- --list 2>/dev/null | grep -c ': test$')
[ "${n_mesh:-0}" -gt 0 ] \
  && ok "holon-mesh reaches $n_mesh tests" \
  || no "holon-mesh reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q -p holon-mesh 2>/dev/null >/dev/null \
  && ok "holon-mesh mutation/bit-identity tests" || no "holon-mesh mutation/bit-identity tests"
cargo build -q -p holon-mesh --release 2>/dev/null \
  && ok "holon-mesh mesh_bench builds" || no "holon-mesh mesh_bench builds"

# holon-tables: SATURATION-3's G1 tier — table generation sharded across workers,
# bit-identical across shard counts, corruption convicted by an exact digest.
#
# WHAT IS ENFORCED HERE AND WHAT IS NOT, because the difference is the whole point.
# This crate's suite has two halves with very different costs:
#
#   --lib        the partition, the traversal, the digest algebra and the conviction
#                cases. Pure arithmetic, no chemistry, runs in milliseconds. ENFORCED.
#   tests/g1_gate.rs
#                the gate itself: every node is a REAL all-electron FCI solve, so the
#                suite is ~126 s in release and minutes-to-hours in debug. NOT enforced
#                here — it is a campaign run, recorded in
#                conformance/atomworld/SATURATION3_RESULTS.md, and a human runs it.
#
# Advertising the gate as CI-enforced when CI cannot afford to run it would be exactly
# the "recorded commitment presented as a proof" this repo refuses elsewhere. The
# vacuity count below covers BOTH halves (--list collects without running), so a crate
# that stopped reaching its gate still shows up as a drop in the count.
n_tables=$(cargo test -q -p holon-tables -- --list 2>/dev/null | grep -c ': test$')
[ "${n_tables:-0}" -gt 0 ] \
  && ok "holon-tables reaches $n_tables tests" \
  || no "holon-tables reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q -p holon-tables --lib 2>/dev/null >/dev/null \
  && ok "holon-tables merge-law digest: fold invariance, conviction, no false positives" \
  || no "holon-tables merge-law digest: fold invariance, conviction, no false positives"
cargo build -q -p holon-tables --release --tests 2>/dev/null \
  && ok "holon-tables G1 gate builds (run is a campaign run, not a CI gate)" \
  || no "holon-tables G1 gate builds (run is a campaign run, not a CI gate)"
# The LEASED path IS enforced: it runs the mesh through holon-resource with a REAL worker
# probe (spawn/run/join, not a capacity reading) on a 32-node H3 grid in seconds, and it is
# the only place the lease books are checked against real concurrent work. Its
# leasing_does_not_change_the_table case is the one that would catch the resource layer
# reaching the numbers.
cargo test -q -p holon-tables --test leased 2>/dev/null >/dev/null \
  && ok "holon-tables leased path: real worker probe, books balance, table unchanged" \
  || no "holon-tables leased path: real worker probe, books balance, table unchanged"
# The reaper against REAL long-running holders. Its false-positive count against genuine work
# is a measurement, and zero is the only acceptable answer -- a reaper that reaps live holders
# is worse than the leak it prevents. It also self-calibrates grace from each worker's OWN
# observed step, which is what took the count from 1115 to 0.
cargo test -q -p holon-tables --test live_reaper 2>/dev/null >/dev/null \
  && ok "holon-tables live reaper: zero false reaps against real generation" \
  || no "holon-tables live reaper: zero false reaps against real generation"

# holon-resource: RESOURCE-1's leasing tier. Unlike holon-tables, this one is FULLY
# enforced here — its whole suite is arithmetic and bookkeeping with no chemistry in
# it, so it runs in milliseconds and there is no excuse for CI not to run the plants.
#
# The plants are the point (RESOURCE_DESIGN D13: a rule that has never fired has never
# been demonstrated to gate). Each was mutation-checked when written — remove rung 3
# and D10 fires; make Receipt round instead of refuse and D8 fires; remove the depth
# check and D7 fires — so these are tests that can fail, not tests that pass.
n_res=$(cargo test -q -p holon-resource -- --list 2>/dev/null | grep -c ': test$')
[ "${n_res:-0}" -gt 0 ] \
  && ok "holon-resource reaches $n_res tests" \
  || no "holon-resource reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q -p holon-resource 2>/dev/null >/dev/null \
  && ok "holon-resource: all nine RESOURCE_DESIGN plants (D2,D4,D5,D7,D8,D9,D10,D12,D3b)" \
  || no "holon-resource: all nine RESOURCE_DESIGN plants (D2,D4,D5,D7,D8,D9,D10,D12,D3b)"

# holon-lens: trajectory lenses, the closure census, and the blind phase classifier --
# census-lens's instrument crate. Pure arithmetic over recorded trajectories (no
# chemistry, no I/O beyond fixtures), so the whole suite runs in seconds and is fully
# enforced here, plants included (tests/plants.rs -- gates that have fired).
n_lens=$(cargo test -q -p holon-lens -- --list 2>/dev/null | grep -c ': test$')
[ "${n_lens:-0}" -gt 0 ] \
  && ok "holon-lens reaches $n_lens tests" \
  || no "holon-lens reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q --release -p holon-lens 2>/dev/null >/dev/null \
  && ok "holon-lens census/classifier gates pass (plants included)" \
  || no "holon-lens census/classifier gates pass (plants included)"

# holon-md: threaded molecular dynamics -- leased workers, chunked evaluation, and the
# claim that matters is BIT-IDENTITY to the serial run (tests/bit_identity.rs). Release
# because the identity test propagates real dynamics; still tens of seconds, not hours.
n_md=$(cargo test -q -p holon-md -- --list 2>/dev/null | grep -c ': test$')
[ "${n_md:-0}" -gt 0 ] \
  && ok "holon-md reaches $n_md tests" \
  || no "holon-md reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q --release -p holon-md 2>/dev/null >/dev/null \
  && ok "holon-md threaded/serial bit-identity gates pass" \
  || no "holon-md threaded/serial bit-identity gates pass"

# holon-device: ONE definition of RESOURCE_DESIGN D0's device class, `no_std` and
# dependency-free, sitting under both `holon-chem` (which ships into the browser and
# cannot take a std-only dep) and `holon-resource` (which sits under everything and
# cannot take a heavy one).
#
# A REAL INVOCATION rather than a CRATE_ALLOW entry, because gate 13's own text says an
# allowlist entry needs an owner and an exit and this crate needs neither: two tests, no
# dependencies, milliseconds. Both halves matter and both are here — every class round-
# trips through its tag (so a class added without one fails HERE rather than inside a
# table six months later), and an unknown tag is REFUSED rather than defaulted (so a
# foreign artifact cannot be admitted into a bit-gated table under this build's class).
n_dev=$(cargo test -q -p holon-device -- --list 2>/dev/null | grep -c ': test$')
[ "${n_dev:-0}" -gt 0 ] \
  && ok "holon-device reaches $n_dev tests" \
  || no "holon-device reaches 0 tests (gate 9's disease: passing without covering anything)"
cargo test -q -p holon-device 2>/dev/null >/dev/null \
  && ok "holon-device: the device class round-trips, and an unknown tag is refused" \
  || no "holon-device: the device class round-trips, and an unknown tag is refused"

# 12. A CROSS-REFERENCE IS A WARRANT ONLY IF ITS TARGET EXISTS (team-lead's ruling,
#     2026-08-24). Q10_PREREG.md §10 cites "M1-M6 carry over from Q9's brief unchanged" —
#     there is no Q9 file anywhere in the repository, so that citation warrants nothing,
#     and nothing here caught it before a human went looking. Prereg/record docs
#     (sim_engine/*.md) cite other artifacts constantly; this gate makes "does the
#     citation resolve" a mechanical fact instead of a claim nobody rechecks.
#
#     THE DISCRIMINATOR (same question gate 11 had to answer for itself): a mention of a
#     filename in prose is not a citation, and a heuristic that cannot tell them apart
#     "looks rigorous and checks nothing". This repo's docs already use two
#     SYNTACTICALLY distinct, unambiguous forms for a real citation — a backtick-quoted
#     path (`` `FILE.md` ``) or a markdown link (`[text](path)`) — never bare prose ("see
#     X" appears nowhere in this corpus, measured). Only those two forms are extracted;
#     everything else is left alone as prose, which is also why "Q9's brief" (bare prose,
#     no literal filename) is NOT and cannot be caught by this gate — that is a real,
#     stated limit, not an oversight: making it catchable requires citing by literal
#     filename, which is exactly the fix this gate incentivizes rather than performs.
#
#     RESOLUTION is a PATH-COMPONENT SUFFIX match against `git ls-files` (the tracked
#     set — what a clean checkout actually has, immune to local untracked clutter),
#     not a literal relative-path resolution: this corpus cites files by bare basename
#     or partial path far more often than by a path resolvable from the citing file
#     (measured: a strict two-basis relative-path resolver flagged 105 "broken"
#     references, of which 94 were real files under a different directory — e.g.
#     `regplus.rs` for `crates/ciris-sim-core/src/regplus.rs`, `Core/ModeChart.lean` for
#     the Lean tree's `CIRISOntology/Core/ModeChart.lean`). A suffix match on path
#     components (not a raw string suffix, so `rt.rs` cannot match `part.rs`) resolves
#     all of those correctly and still catches a genuine miss: a literal `../MISSION.md`
#     does not suffix-match `sim_engine/MISSION.md` (no tracked path contains a literal
#     `..` segment), so a wrong-directory citation is not silently rescued into a pass.
#     Existence only, deliberately: `RESUME.md` resolves against any of nine same-named
#     files in the tree, and this gate does not attempt to pick the right one — that is
#     a real, separate limitation (a citation can resolve to a WRONG same-named file),
#     named here rather than hidden.
#
#     THE ALLOWLIST is per (file, reference), not per filename, and every entry is one of
#     THREE NAMED POLICY CATEGORIES — not an unexplained one-off, so the next reference of
#     a kind already seen is covered by a stated rule rather than re-litigated:
#       1. FOREIGN REPO. FSD_GRAPH_PHYSICS_ENGINE.md declares itself "Repo: CIRISClient" in
#          its own header and cites that OTHER repo's planned files (attract.rs,
#          geometry.rs, plasma.rs, tendrils.rs, ../MISSION.md).
#       2. DEPENDENCY INTERNALS / EXTERNAL ASSET. NL_BRIDGE.md's
#          build.rs/llama.rs/onnx_registry.rs/rten_registry.rs name files INSIDE the
#          llama-cpp-2/rten dependency crates, not this repo; tokenizer.json is a
#          downloaded 11.4MB model asset, deliberately not committed (see
#          chief-of-staff-2's finding on quoted-identity artifacts).
#       3. CROSS-BRANCH PROVENANCE: target verified present on a named branch.
#          MESH_DESIGN.md's M-G11 cites "`JULES_3D_TRIAGE.md` §3.3" as the SOURCE of a
#          finding it already restates inline — not a "go read this" pointer the reader
#          still needs. `git show salvage/jules-3d:sim_engine/JULES_3D_TRIAGE.md` confirms
#          the file exists there; that branch's own header says "Nothing here lands on
#          main... this branch is a parts shelf", so its absence from main is by design,
#          not decay. Team-lead's ruling: the gate's rule is "a cross-reference is a
#          warrant only if its target EXISTS" — it does, just not on this branch — which
#          is a different failure than a citation to something NEVER WRITTEN (Q9's brief,
#          T1's shorthand), and the gate must not conflate them. The citation itself
#          should eventually name the branch inline (mesh's edit to make, tracked
#          separately, not blocking this gate).
#     Scoped per-reference, not globally by filename, so a genuinely missing `build.rs`
#     in some OTHER document is still caught — `build.rs` is too common a real filename
#     to exempt everywhere.
#
#     MESSAGE-ONLY CONTENT IS NOT RECORD: Q10's M1-M6 exist only as an instruction inside
#     agent conversations (named in a board brief, never written to a file), and four
#     Fable-limit deaths today destroyed exactly that kind of content mid-lane. A citation
#     into a lane's conversation is a citation into something that can vanish without
#     warning; a citation into a tracked file is the only kind this gate — or any
#     mechanical check — can ever stand behind.
declare -A REF_ALLOW=(
  ["sim_engine/FSD_GRAPH_PHYSICS_ENGINE.md::attract.rs"]=1
  ["sim_engine/FSD_GRAPH_PHYSICS_ENGINE.md::geometry.rs"]=1
  ["sim_engine/FSD_GRAPH_PHYSICS_ENGINE.md::plasma.rs"]=1
  ["sim_engine/FSD_GRAPH_PHYSICS_ENGINE.md::tendrils.rs"]=1
  ["sim_engine/FSD_GRAPH_PHYSICS_ENGINE.md::../MISSION.md"]=1
  ["sim_engine/NL_BRIDGE.md::build.rs"]=1
  ["sim_engine/NL_BRIDGE.md::llama.rs"]=1
  ["sim_engine/NL_BRIDGE.md::onnx_registry.rs"]=1
  ["sim_engine/NL_BRIDGE.md::rten_registry.rs"]=1
  ["sim_engine/NL_BRIDGE.md::tokenizer.json"]=1
  # Category 3, cross-branch provenance (see the comment block above):
  ["sim_engine/MESH_DESIGN.md::JULES_3D_TRIAGE.md"]=1
)
repo_root=$(git rev-parse --show-toplevel 2>/dev/null)
ref_fail=0
if [ -n "$repo_root" ]; then
  tracked=$(git -C "$repo_root" ls-files)
  for f in *.md; do
    refs=$( { grep -oE '`[A-Za-z0-9_./+-]+\.(md|lean|rs|json)`' "$f" | tr -d '`'; \
              grep -oE '\]\([^)[:space:]]+\.(md|lean|rs|json)\)' "$f" | sed -E 's/^\]\(//; s/\)$//'; } \
            | sort -u )
    while IFS= read -r r; do
      [ -z "$r" ] && continue
      case "$r" in http://*|https://*) continue ;; esac
      [ -n "${REF_ALLOW["sim_engine/$f::$r"]:-}" ] && continue
      esc=$(printf '%s' "$r" | sed 's/[.[\*^$]/\\&/g')
      # herestring, not a pipe: grep -q exits at first match, and the pipe was
      # spraying "printf: write error: Broken pipe" across every verify log.
      if ! grep -qE "(^|/)${esc}\$" <<< "$tracked"; then
        # DECOUPLING RULE: these docs migrated from CIRISAI/CIRISOntology and
        # legitimately cite files tracked THERE (Core/*.lean, campaign
        # records). UPSTREAM_MANIFEST.txt is a committed snapshot of the
        # upstream `git ls-files`; refresh it when upstream moves a cited
        # file. A citation resolving against neither tree is still a failure.
        if ! grep -qE "(^|/)${esc}\$" UPSTREAM_MANIFEST.txt 2>/dev/null; then
          echo "    sim_engine/$f: \`$r\` -- no tracked file (local or upstream manifest) matches"
          ref_fail=1
        fi
      fi
    done <<< "$refs"
  done
fi
# 9c. THE PREREG AUDIT (ported from CIRISOntology after it was NOT used for
#     BRIDGE-5/6/SCHWINGER-2 -- all three retro-refused). Every NEW freeze
#     must pass Audit/prereg_audit.py: witnesses resolve in lean/, misfit
#     registry contact is cited, gates carry numeric-or-EXACT criteria, and
#     plants state the sector their carrier must be nonzero in. Historical
#     freezes are FROZEN HISTORY: they cannot be edited to comply, so they
#     are exempted BY NAME with their retro-refusals recorded in the results
#     documents. Adding a name to this list requires a results document
#     explaining why.
# (Path bug caught by direct test: the first version globbed from engine/
#  where conformance/ does not exist, audited ZERO files, and passed
#  vacuously -- while the commit banking it claimed BRIDGE-2 passed. It is
#  REFUSED like the others. A gate that inspects zero artifacts must refuse,
#  and now does.)
#
# SCOPE, WIDENED 2026-09-01 (tower lane), because the previous glob was the same
# defect one level up from the path bug recorded above. It named TWO directories --
# gravity and crystal -- while the tree has SIX, so four whole campaign families
# (atomworld, omega, selector, water_observatory) were audited by nobody. The gate
# was green and the question "what would this command NOT catch?" had the answer
# "most of the freezes". The DIMER-1 freeze that prompted the check was itself
# outside it. Measured before widening: atomworld (6), selector (4) and
# water_observatory (3) all pass unchanged; omega has two that do not, named below.
#
# STILL UNCOVERED, stated rather than left silent: `engine/*_PREREG.md` (Q7, Q7B,
# Q8_MPS, Q10, Q_SEAM). All five are frozen history from before this audit existed
# and all five retro-refuse; covering them would mean five more exemptions and zero
# coverage gain, which is what the file above calls suppression. Owner: the Q-series
# lane. Exit: a results document recording the five retro-refusals, after which the
# glob takes `../engine/*_PREREG.md` too.
PREREG_EXEMPT="GRAVITY_BRIDGE0_PREREG.md GRAVITY_BRIDGE0_V2_PREREG.md GRAVITY_BRIDGE1_PREREG.md GRAVITY_BRIDGE2_PREREG.md GRAVITY_BRIDGE3_PREREG.md GRAVITY_BRIDGE5_PREREG.md GRAVITY_BRIDGE6_PREREG.md CROSSFACE1_PREREG.md OMEGA_CIRCUITS1_PREREG.md"
# CROSSFACE1 / OMEGA_CIRCUITS1: added with the widening, NOT to hide them. Both
# retro-refuse on one contact each -- 'repair'/'maintenance' -> M-MAINTENANCE-LENS
# uncited -- and both are frozen, so they cannot be edited to comply. Owner: the
# omega lane. Exit: record the retro-refusal in the omega results documents (the
# same remedy the seven gravity names took) and delete these two entries. An
# allowlist entry is legitimate only when the break has an owner and an exit; these
# have both, and the break is one citation line, not a defect in either campaign.
audit_fail=0
audit_seen=0
for pre in ../conformance/*/*_PREREG.md; do
  [ -f "$pre" ] || continue
  audit_seen=$((audit_seen+1))
  base=$(basename "$pre")
  case " $PREREG_EXEMPT " in *" $base "*) continue;; esac
  python3 ../Audit/prereg_audit.py "$pre" >/dev/null 2>&1 || { audit_fail=1; echo "  prereg audit refuses: $base"; }
done
[ "$audit_seen" -gt 0 ] || { audit_fail=1; echo "  prereg audit saw ZERO preregs -- a vacuous gate is a failed gate"; }
[ "$audit_fail" -eq 0 ] && ok "every non-exempt prereg passes the prereg audit (saw $audit_seen)" \
  || no "every non-exempt prereg passes the prereg audit"

[ "$ref_fail" -eq 0 ] && ok "prereg cross-references resolve to a tracked file" \
  || no "prereg cross-references resolve to a tracked file (see missing targets above)"

# 13. EVERY CRATE THE WORKSPACE KNOWS ABOUT MUST HAVE A STATED INVOCATION IN THIS
#     SCRIPT (team-lead's ruling, 2026-08-24), or a documented reason it does not yet.
#     k2-judge went looking for a way to build-verify h3ere2-eval and could not find one
#     here — that absence is the finding this gate closes.
#
#     THE MECHANISM, corrected before landing rather than after (chief-of-staff-2 relayed
#     "`cargo build -p h3ere2-eval` resolves to nothing, not an error" without running it;
#     it does error, loudly, exit 101, `package ID specification did not match any
#     packages` — measured here first). What actually resolves to nothing is the natural
#     BUILD-EVERYTHING command: `cargo build --workspace` exits 0 over a crate `cargo
#     metadata` does not even list, silently, because h3ere2-eval is in `exclude`. `-p`
#     on a name that is not a package always fails loudly; the hazard is a crate nobody
#     ever names at all.
#
#     "REACHABLE" CANNOT MEAN "IN members": four crates are excluded on purpose
#     (engine-compare pulls Rapier and would falsify the core's zero-allocation
#     isolation; wasm-probe and ciris-sim-component each carry their own target/profile
#     configuration a plain `-p` would silently get wrong; h3ere2-eval pulls
#     llama-cpp-2's cmake/C++ build). Forcing them into `members` to satisfy this gate
#     would be exactly the wrong fix -- the isolation gates exist to keep them out. Each
#     is checked via `--manifest-path crates/<dir>/Cargo.toml` from its own directory
#     instead, or exempted with its OWN documented exclude reason if that invocation
#     would need something this script cannot yet supply.
#
#     THE AUDIT uses each crate's real `name =` field, not its directory name -- caught
#     immediately that `crates/wasm-probe`'s package is `ciris-sim-wasm-probe`, which a
#     directory-name check would have missed exactly the way this gate exists to prevent.
#
#     THE FINDING IS BIGGER THAN THE CASE THAT PROMPTED IT: seven crates had zero
#     invocation anywhere in this script. THREE are workspace MEMBERS -- q8-mps, q-seam,
#     sphere-demo -- trivially reachable by a plain `-p` that nobody had ever called. The
#     other FOUR are the excluded set. CRATE_ALLOW is per-crate, dated, and every entry
#     names why, not just that:
#       - q8-mps: DEFERRED. Its grid is live and hours deep; a gate must never run
#         `--ignored` full-grid tests, which are a multi-hour job with no business inside
#         a gate script. Revisit once the grid completes; the eventual entry runs the
#         fast suite only.
#       - q-seam, sphere-demo: uncovered, ownership untriaged (chief-of-staff-2,
#         2026-08-24). q-seam is q8-mps's exact-reference dependency and is real,
#         worth covering once the grid clears.
#       - engine-compare, wasm-probe (pkg ciris-sim-wasm-probe), ciris-sim-component:
#         each crate's OWN documented `exclude` reason in this file's Cargo.toml already
#         states why a naive invocation here would be wrong (std/alloc unification risk;
#         a tri-target profile a plain build would silently misconfigure; a WIT ABI with
#         its own release profile) -- restated per-crate rather than re-litigated.
#
#     h3ere2-eval -- the case that prompted the whole gate -- is NOT in CRATE_ALLOW, and
#     the reason it briefly was is worth keeping: a cold `--manifest-path` build here
#     first reproduced the same E0433/E0599 k2-judge had found from inside the K2 lane
#     (`ciris_nl::chat`/`Session::generate` both called, neither existing), which made a
#     plain "excluded, not yet covered" allowlist reason a lie, while a blocking gate
#     entry would have reddened every OTHER lane's push over a crate none of them touch.
#     Team-lead's ruling on the discriminator, worth restating wherever this pattern
#     recurs: AN ALLOWLIST ENTRY IS LEGITIMATE ONLY WHEN THE BREAK HAS AN OWNER AND AN
#     EXIT; WITHOUT BOTH IT IS SUPPRESSION. The entry that shape produced named k2-judge
#     as owner and "converts to a real invocation the moment the crate builds" as exit --
#     and the exit fired during THIS SAME construction (`71ff2b6`, "the generator was
#     recoverable, and the rebuild is proven byte-identical"), so what ships below is the
#     real gate 13's audit was always meant to end in, not the allowlist entry.
declare -A CRATE_ALLOW=(
  ["holon-gpu"]="excluded: requires an NVIDIA GPU (cudarc + nvcc-compiled PTX); GitHub runners have none. Tested on the 4090 dev box: 12/12 determinism tests incl. struct-level shard invariance vs the CPU mesh (crates/holon-gpu/GPU.md). Owner: gpu-mesh lane / team-lead. Exit: a CI runner with a GPU, at which point this entry converts to a real invocation."
  ["q8-mps"]="PARTIALLY COVERED (2026-09-01): gate 16b runs --test c2_tdvp_gates (the C2 carrier). The DEFERRAL still stands for the rest of the crate -- a live full-grid run, hours deep, and a gate must never run --ignored full-grid tests. Owner: the C2/tower lane. Exit: once the grid completes, this entry converts to a plain -p q8-mps. (Backticks removed: inside a double-quoted bash string they are COMMAND SUBSTITUTION -- the prose executed itself and corrupted this entry, failing the coverage audit.)"
  ["q-seam"]="uncovered. Owner: team-lead. Exit: real invocation or crate retirement, decided at the fence-ledger triage (FENCES.md F-4) — this entry was suppression by the gate's own ruling until it carried these two fields."
  ["sphere-demo"]="uncovered. Owner: team-lead. Exit: real invocation or crate retirement, decided at the fence-ledger triage (FENCES.md F-4)."
  ["engine-compare"]="excluded: pulls Rapier, needs std+alloc, would falsify the core's zero-allocation isolation gates"
  ["ciris-sim-wasm-probe"]="excluded: carries its own target/profile configuration for the tri-target bit-identity probe"
  ["ciris-sim-component"]="excluded: WIT adapter with its own release profile and dependency graph"
)
crate_fail=0
for d in crates/*/; do
  d="${d%/}"
  [ -f "$d/Cargo.toml" ] || continue
  dirname=$(basename "$d")
  pkgname=$(grep -m1 '^name = ' "$d/Cargo.toml" | sed -E 's/^name = "(.*)"$/\1/')
  [ -z "$pkgname" ] && pkgname="$dirname"
  [ -n "${CRATE_ALLOW["$pkgname"]:-}" ] && continue
  if grep -qE -- "-p $pkgname([[:space:]]|\$)" "${BASH_SOURCE[0]}" \
     || grep -qE -- "--manifest-path crates/$dirname/" "${BASH_SOURCE[0]}"; then
    :
  else
    echo "    crates/$dirname (package \"$pkgname\") has no build/test invocation anywhere in ci-gates.sh"
    crate_fail=1
  fi
done
[ "$crate_fail" -eq 0 ] && ok "every non-exempt crate has a stated invocation in ci-gates.sh" \
  || no "every non-exempt crate has a stated invocation in ci-gates.sh"

# 14. h3ere2-eval's own build+test, from its own directory, satisfying gate 13's audit
#     for real rather than by exemption -- K2's instrument, once the crate this gate
#     found broken during its own construction. `--lib` carries 13 real unit tests
#     (scramble/path/blocks -- weight-multiset, determinism, seeded-path properties),
#     none of them needing model weights; the three `bin/` targets (generate, labelqual,
#     paths) are build-checked by the same command with none filtered out, so a compile
#     break in any of them still fails this gate the way the original hazard needed.
cargo test -q --manifest-path crates/h3ere2-eval/Cargo.toml 2>/dev/null >/dev/null \
  && ok "h3ere2-eval builds and its unit tests pass" || no "h3ere2-eval builds and its unit tests pass"

# 15. holon-render-3d, HEADLESS ONLY -- the 3D atom world's conservation gates, run where
#     there is no GPU.
#
#     `--no-default-features --features headless` is the whole point of the feature split
#     and not a convenience: the default (`native`) links bevy_render, wgpu and winit and
#     needs X11/Wayland/ALSA development headers, none of which a gate script should
#     require. `headless` enables `bevy/std` and nothing else, so what this compiles is
#     bevy_app + bevy_ecs + the physics -- and the gates it asserts (energy, momentum, the
#     capture plant, the drag ledger) are properties of the physics, which is exactly why
#     they can be asserted without pixels.
#
#     `--manifest-path` because the crate is deliberately OUTSIDE this workspace (see the
#     `exclude` block in Cargo.toml: Bevy's feature set would unify with the no_std core's
#     and falsify gates 1-3). This invocation is also what satisfies gate 13's audit for
#     the crate, rather than a CRATE_ALLOW entry -- there is no break here to own and no
#     exit to wait for, so an allowlist entry would be suppression.
cargo test -q --manifest-path crates/holon-render-3d/Cargo.toml --release \
  --no-default-features --features headless 2>/dev/null >/dev/null \
  && ok "holon-render-3d headless gates pass" || no "holon-render-3d headless gates pass"

# 16. the atom world's two workspace crates, which the coverage audit rightly
#     refused to see uncovered: holon-chem (the browser's own STO-3G FCI chemistry,
#     pinned 1e-12 to the 50-digit referee; the ELEMENTS-1 cross-check stays
#     #[ignore]d until the elements-referee files are committed and is not what this
#     gate waives) and holon-render (the ledger-gated shell whose conservation gates
#     are one-per-law; holon-render-3d consumes it as an rlib and is gate 15).
cargo test -q --release -p holon-chem 2>/dev/null >/dev/null \
  && ok "holon-chem referee/FCI/pair/E1 gates pass" || no "holon-chem referee/FCI/pair/E1 gates pass"
cargo test -q --release -p holon-render 2>/dev/null >/dev/null \
  && ok "holon-render ledger/amendment/cluster gates pass" || no "holon-render ledger/amendment/cluster gates pass"

# 16b. THE C2 CARRIER (WB-8.3). The tower's third node is real-time MPS electronic
#      dynamics, and holon-chem's C2_MpsTdvp is a thin shell over q8-mps's integrator --
#      so the gate that means anything is the integrator's, not the shell's.
#
#      NAMED TEST TARGET, on purpose. q8-mps's CRATE_ALLOW entry above defers the crate
#      because its full-grid tests are a multi-hour job that must never run inside a gate
#      script; `--test c2_tdvp_gates` cannot reach them (they are `#[ignore]`d and live in
#      other test binaries), so this covers the new carrier without touching the deferral's
#      reason. The price measurement in the same file is `#[ignore]`d for the same class of
#      reason one level down: a wall clock on a heterogeneous box is a placement lottery
#      (M-PLACEMENT-LOTTERY) and has no business deciding a gate.
#
#      --release is not a speed preference: the battery integrates ~1400 TDVP substeps
#      including a 512-step reference trajectory, which is ~30 s debug and ~3 s optimised.
cargo test -q --release -p q8-mps --test c2_tdvp_gates 2>/dev/null >/dev/null \
  && ok "q8-mps C2 TDVP gates pass (exactness, order, 4 planted defects fire)" \
  || no "q8-mps C2 TDVP gates pass (exactness, order, 4 planted defects fire)"

# 15b. THE 3D SHELL IN THE CONFIGURATION THE DEPLOY ACTUALLY SHIPS.
#
#     Gate 15 above runs holon-render-3d `--no-default-features --features headless`,
#     which is right for what it tests and is why it could not see this: `hud.rs`,
#     `pick.rs` and `render.rs` are all `#[cfg(feature = "render")]`, so the headless
#     build never compiles them. The wasm the site serves is built WITH render.
#
#     MEASURED CONSEQUENCE, and it is why this gate exists rather than being a tidy-up.
#     `Boundary::Periodic` landed at 4bec9e2 and two `match` expressions in hud.rs were
#     never extended. From that commit until this one, holon-render-3d DID NOT COMPILE for
#     wasm32 -- and both of the mechanisms that should have said so were green:
#
#       * gate 15 passed, because the feature it selects excludes the broken modules;
#       * pages.yml built the artifact under `continue-on-error: true`, so the deploy
#         proceeded and the failure was swallowed.
#
#     The artifacts are never committed (they are ~40 MB each), so every deploy in that
#     window shipped docs/atoms3d with no wasm at all. The page fences itself honestly
#     when neither backend loads, which is the one thing that went right; nobody was told
#     the shell had stopped building.
#
#     `cargo check`, not `build`: this catches the type errors that actually broke it
#     without paying for codegen and wasm-bindgen on every gate run. The full artifact
#     build stays in pages.yml where it belongs.
if cargo check -q --manifest-path crates/holon-render-3d/Cargo.toml \
     --target wasm32-unknown-unknown --features render 2>/dev/null; then
  ok "holon-render-3d compiles for wasm WITH render (the configuration the site ships)"
else
  echo "    the headless gate above cannot see this: hud/pick/render are cfg(feature = \"render\")"
  echo "    reproduce: cargo check --manifest-path crates/holon-render-3d/Cargo.toml --target wasm32-unknown-unknown --features render"
  no "holon-render-3d compiles for wasm WITH render (the configuration the site ships)"
fi

# 17. THE BROWSER ARTIFACT, RUN RATHER THAN BUILT.
#
#     Gate 1 compiles for wasm. Nothing until now RAN a wasm artifact, and that gap has
#     hidden two defects that every native gate was green through:
#
#       * a rebuilt viewer wasm TRAPPED -- `RuntimeError: memory access out of bounds` on
#         `holon_bank_generate_pair(1, 2, 96)`, which is helium, two orbitals, the cheapest
#         solve the sandbox does -- while `cargo test` was fully green for holon-chem and
#         holon-render against the identical source. Cause: a wasm stack overflow at
#         wasm-ld's 1 MiB default, from `Box::new(<big array literal>)` materialising 685 KB
#         on the stack before moving it to the heap. Fixed at 3b37b8e by building a boxed
#         slice through `vec!` instead; no stack bump was needed.
#       * the unified viewer read `holon_atom_z` -- the atom's Z COORDINATE in bohr -- as
#         its species, for months, guarded by a second export (`holon_set_atom_z`) that has
#         never existed. The guard was false, so the fallback ran and every atom drew as
#         hydrogen; the day anything exported that name, atoms would have been coloured by
#         their height in the box.
#
#     So this drives the actual `wasm32-unknown-unknown` artifact through the raw
#     extern "C" ABI, with the real shipped JSON tables parsed by a host: it solves H2,
#     changes a species and requires the scene to REFUSE to step until the new pair's curve
#     is banked, requires Cl2 to be refused for in-browser solving rather than spending
#     ninety-six seconds of a page's main thread, loads the shipped HCl and Cl2 tables
#     through the provenance door, fires the three provenance refusals and requires the
#     refused curve to be EVICTED, steps a mixed scene and requires both conservation gates,
#     and requires every `holon_*` call each of the four viewer copies makes to resolve.
#
#     DEMONSTRATED FAILING CASE (standing question 4): run against the trapping build
#     described above, this exits 1; against the good build it exits 0. It was not adopted
#     on the strength of passing.
#
#     The `command -v node` check FAILS rather than skips. A wasm gate that quietly skips
#     itself when the runner lacks node is the vacuous-success shape -- green because it
#     checked nothing -- which is the exact family this repository keeps catching. Node is
#     preinstalled on ubuntu-latest, so a missing one means the environment changed and
#     somebody should look.
if command -v node >/dev/null 2>&1; then
  node crates/holon-render/viewer/smoke.mjs >/dev/null 2>&1 \
    && ok "browser artifact runs (wasm ABI, shipped tables, provenance refusals)" \
    || no "browser artifact runs (wasm ABI, shipped tables, provenance refusals)"
else
  no "browser artifact gate needs node, which is not on PATH (NOT skipped: a wasm gate that skips itself is green for having checked nothing)"
fi

# 17b. THE WORKBENCH ARTIFACT, RUN RATHER THAN BUILT.
#
#     The atom viewer's smoke gate above drives the ABI. This one drives the WORKBENCH's
#     copy through the boot sequence its own page uses, and adds the checks that page is
#     making claims about: both conservation gates closing under a thermostatted run, the
#     hand's work landing in the ledger exactly and coming back out on release, the O-O
#     curve refused by the engine's own in-browser split with the code the page NAMES,
#     O-bearing triples refused AND counted (with the discriminating arm that proves the
#     count is not the uninformative "never looked" zero), and the seeded scene replaying
#     bit-identically.
#
#     It also runs two checks of a kind this repository has been short of. The first reads
#     the page's declared export list out of app.js and requires it to cover every
#     `holon_*` the page actually calls, so a call added without a declaration is caught
#     rather than discovered as an undefined function in a frame loop -- the unified
#     viewer's `holon_atom_z` incident is exactly this defect. The second is INVERTED: the
#     page fences gravity, the barostat and the phase classifier on the stated grounds that
#     no such export exists, and the gate FAILS if one ever appears, telling the next lane
#     to un-fence the panel. A fence justified by an absence rots into a lie the day the
#     absence ends, and nothing else in the tree would notice.
#
#     DEMONSTRATED FAILING CASES: five plants (a declared export that does not exist, a
#     call missing from the declaration, a SYNTHETIC label reaching the rendered page, a
#     fence-justifying absence ending, and a Math.random re-entering the telemetry path)
#     each fire their own prong; a sixth -- the O:2H preset dropping the H3 surface -- fires
#     the fence-count prong. The first version of that last check PASSED while its defect
#     was planted, because an earlier section of the same file had left a surface resident;
#     the block now runs on fresh instances, which is why it can fail at all.
if command -v node >/dev/null 2>&1; then
  node ../docs/workbench/smoke.mjs >/dev/null 2>&1 \
    && ok "workbench artifact runs (ledgered hand, fired fences, bit-identical replay)" \
    || { node ../docs/workbench/smoke.mjs 2>&1 | sed 's/^/    /'; \
         no "workbench artifact runs (ledgered hand, fired fences, bit-identical replay)"; }
else
  no "workbench artifact gate needs node, which is not on PATH (NOT skipped)"
fi

exit $fail
