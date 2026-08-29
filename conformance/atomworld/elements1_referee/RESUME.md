# ELEMENTS-1 REFEREE — resume state

Scratchpad lane: `/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/elements_referee`
Repo drop:       `/home/emoore/CIRISHolon/engine/crates/holon-chem/tests/data/elements1/`  (pathspec commits only, NO push)

## Landed (six of nine, committed, engine-graded, byte-stable)

H2 · He2 · LiH · HF · F2 · Ne2 — plus `atoms.json` and `manifest.json`.
Last pin: **56a8141** (grid_provenance added; every number byte-identical). Re-emitting today reproduced all eight files byte-for-byte.
`verify_elements.py --quick` → 263 checks, 0 FAIL, exit 0.

## Owed (three)

| species | state | note |
|---|---|---|
| Li2 | grid 42/42 done (dual-route max\|A−B\| 7.13e-54); stencils + spin + recertify running | open-shell dissociation, expect a multiplicity crossing |
| N2  | grid 9/13 on the sparse staked subset | 13-knot subset, staked rule in `species.py:sparse_subset` |
| CO  | queued behind N2 | under the corrected oxygen 130.70932140 |

Per species, in order: `--stencil --hermite --spin --recertify` → `--probe` →
`--assemble` → `emit_engine.py engine_handoff/elements1` →
`verify_elements.py --quick` (must exit 0) → copy to the repo path →
`git commit -F <msgfile> -- <pathspec>` → tell elements-engine the new pin.

## Live processes (all detached with setsid; session death kills narration only)

- `build_curves.py --energies N2 CO`                              → `heavy_stage1.log`
- `build_curves.py --stencil --hermite --spin --recertify Li2`    → `li2_rest.log`
- `watch_progress.sh`                                             → `progress.log` (10-min lines; shouts STALLED after 90 min of no new cache record with pools alive)
- test suites → `suites.log`, marker `suites.DONE`

## Machine

32 cores. Other campaigns take ~6. Keep my two pools at 12 workers each.
Oversubscription is the failure mode to watch: it looks exactly like "these
determinants are expensive".

## Guards added this pass (all with a demonstrated failing case)

1. **Run lock** (`locks/<species>.<stage>.lock`) — refuses a second pool on work
   already running; probes the holder pid; `ALLOW_DUPLICATE_RUN=1` escapes.
   Fired against a real running pool, exit 3. Two identical pools had been
   running the six landed species for three hours, 28 of 32 cores, both healthy.
2. **Pool wrapper** — `pmap` wraps its callable in `_Safe` unconditionally.
   `_install_safe()` was called by NOTHING in `main()`, so every pool this
   campaign ran was unprotected against the unpicklable `ArpackNoConvergence`
   that kills the result-handler thread and leaves the workers looking healthy.
3. **In-process guard self-test** — `selftest_pool_guard()` fires the real
   failure through the real `pmap` in the process about to do the work, in a
   daemon thread so a missing guard times out instead of hanging the job.
4. **merge-not-narrow** — `--assemble He2` used to REPLACE the accumulated
   `elements_potential_partial.json` with one species. It did, today. Repaired
   and re-emitted identical.
5. **V9 in the verifier** — asserts guards 1–4 are on the path `main()` takes.
   Demonstrated failing by disconnecting them.

6. **grid_provenance** — every pair file carries the grid rule in staked
   parameters, and the emitter REGENERATES the subset from that rule and
   refuses the file if it differs. Dense species carry the same block and are
   refused if their knot count is not the full grid's.
7. **The diff sees keys** — `diff_against_existing` reported "no change" on six
   files that had each just gained a top-level block. It now enumerates key
   sets.
8. **No defaulted spin column** — an absent `resolved_by_geometry` used to fall
   back to "resolved everywhere"; it is now a refusal.
9. **V10** — rebuilds each emitted grid from the file's OWN declared parameters
   and subset rule. Its sparse branch is exercised on a synthetic N2 drop
   (`test_verify_sections.py`) because N2 lands last.
10. **The audit stopped counting the writer as a reader** — `_inert_audit.py`
   had `emit_engine.py` in its reader set, so every key the emitter names
   literally looked consumed. Corrected, the buckets are read / guarded-at-
   write / inert; V8 now re-derives every spin summary from the per-geometry
   columns, and the six remaining inert keys are prose, allowlisted in
   `prose_fields.txt` — which is a separate file BECAUSE naming them inside
   the verifier laundered them out of the inert bucket.

Tests: `test_runlock.py` (5), `test_pmap_safety.py` (3, incl. the unguarded
hang), `test_emit_refusals.py` (9 refusals, each fired on purpose),
`test_integrals.py`, `test_fci.py`. All are in `run_final.sh`.

## Chains running detached (markers, not narration)

- `after_li2.sh` → waits for the Li2 pool, runs `--probe` then `--assemble`,
  touches `after_li2.DONE` (or `.FAILED`). Log: `after_li2.log`.
- `after_energies.sh` → waits for the N2/CO grid pool, runs
  `--stencil --hermite --spin --recertify`, `--probe`, `--assemble` for both,
  touches `after_energies.DONE` (or `.FAILED`). Log: `after_energies.log`.

After each lands: emit → verify (`--quick`, must exit 0) → copy to the repo →
pathspec commit → tell elements-engine the new pin.

## Standing rules for this lane

- Never trade digits for knots: a sparse exact referee is a referee.
- Every claim about a sweep must say where the claim would fail and whether the
  sweep goes there.
- Every emitted key must have a reader (`_inert_audit.py`); nine are deliberate
  human provenance, and that is a different status from unread.
- A guard must demonstrate a failing case, and must be shown to be CONNECTED.
