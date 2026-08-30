# ELEMENTS-1 REFEREE — resume state

Scratchpad lane: `/tmp/claude-1000/-home-emoore-CIRISOntology/4cf4fa5c-aaa3-4173-83b9-978cb75c887f/scratchpad/elements_referee`
Repo drop:       `/home/emoore/CIRISHolon/engine/crates/holon-chem/tests/data/elements1/`  (pathspec commits only, NO push)
Cache backup:    `$HOME/elements1_referee_backup/cache` — /tmp is not durable.
Sibling lane:    `../mixtures_referee` (MIXTURES-1; imports this one via a symlink)

## Landed (six of nine, committed, engine-graded, byte-stable)

H2 · He2 · LiH · HF · F2 · Ne2 — plus `atoms.json` and `manifest.json`.
Last pin: **56a8141**. Re-emitting after the 2026-08-30 cache migration
reproduced all six species byte-for-byte, which is what licensed the migration.

## THE DEFECT THIS PASS FOUND, AND WHY IT MATTERS TO ANYONE READING LATER

Three record kinds shared ONE tag namespace: `run_point` (dual-route certified),
`energy_only` (single-route stencil energy), `spin_only`.  Only spin was
prefixed.  For the three HEAVY species the stencil precision equals the grid
precision (`FD_DPS_CHEAP == R.DPS == 60`), so a stencil evaluated AT a grid point
produced that grid point's key exactly.

- **loud** — `energy_only` reading a point record raises `KeyError: 'E'`.  That
  is what killed the N2/CO stencil stage (`after_energies.FAILED`) and Li2's.
- **silent** — when the stencil got there first, its single-route record sat
  under the certified point's key and `run_point` returned it from cache without
  looking at its shape.  **Li2 had two** (R = 7.490376922377, 18.000000000000):
  no route B, no Temple bound, no spin sector, no `dev_AB`.

Repaired two ways, both connected, each with a demonstrated failing case:
1. per-kind tag namespaces (`fd_` for stencil energies, as `spin_` already was)
   + `_migrate_fd_tags.py`, which moved 9470 records and vacated exactly the two
   contaminated Li2 keys;
2. `cache_get(..., kind=)` checks the shape against what the caller will read.
   A clash **raises** — `None` means "stale, recompute", and recomputing under a
   clashing key would overwrite the other kind's record.

`test_cache_kinds.py` — 23 checks, 0 FAIL, including the old tag demonstrated
colliding on 12 heavy stencil centres.

Two more found by the byte-stability check that the repair required:
- **Ne2 assembled with `n_probes = 0`, `max_abs_error = 0.0`** — a zero
  uncertainty that is a MISSING measurement.  Never reached the engine drop
  (Ne2.json carries no hermite block).  `assemble()` now refuses it; the refusal
  fired on Ne2 first run.  F2 likewise had only 110 of 399 probes cached.
- **CO R=9.0 died on `ArpackNoConvergence` with no fallback.**  The f64 seed
  asked for `tol=0` — machine precision on all six vectors — and at a
  dissociation limit the low levels are degenerate to ~1e-14.  `fci.py` now has
  a four-rung seed ladder (rung (a) unchanged and tried first, so every existing
  geometry computes bit-identically; then ARPACK partials, a wider Krylov basis
  at seed-grade tolerance, then Jacobi-preconditioned LOBPCG from the
  lowest-diagonal block — deterministic, no RNG).  Also fixes a latent
  IndexError: the old code accepted a one-vector ARPACK partial then indexed
  `w[o[1]]`.  `test_seed_ladder.py` — 10 checks, 0 FAIL, every rung forced.

## Owed (three) — CORRECTED STATE

| species | state |
|---|---|
| Li2 | **23 of 42 grid points valid**, not 42.  17 carry `basis_fingerprint: null` (they predate the stamp; `cache_get` refuses them) and 2 were the silent collisions.  19 dual-route recomputes owed, then spin/stencil/hermite/probe. |
| N2  | 13 of 13 grid points done.  Owed: stencil (was blocked by the defect), hermite, spin ×13, recertify, probe. |
| CO  | 12 of 13 grid points done; **R = 9.000000000000 owed** and needs the seed ladder.  Then the same tail as N2. |

## Live

`run_owed.sh` — detached, `setsid`, per-step `.DONE`/`.FAILED` markers, log
`run_owed.log`.  Steps, in order, each skipped if its marker exists:

    owed_hermite_landed  --hermite F2 Ne2      (closes the re-derivability gap)
    owed_li2_energies    --energies Li2        (19 points)
    owed_stencils        --stencil Li2 N2
    owed_hermite_heavy   --hermite Li2 N2
    owed_spin            --spin Li2 N2         (55 geometries)
    owed_recertify       --recertify Li2 N2
    owed_probe           --probe Li2 N2
    owed_assemble        --assemble Li2 N2

Re-running `./run_owed.sh` after a kill resumes at the first missing marker.

`run_co.sh` — CO's whole tail, queued behind the chain: it waits for
`run_owed.DONE`, refuses to start if any `owed_*.FAILED` appeared, then runs
`--energies --stencil --hermite --spin --recertify --probe --assemble` for CO.
It is a SEPARATE file rather than more steps in `run_owed.sh` because bash reads
a script incrementally from a byte offset: editing the file a running shell is
in the middle of makes it resume at the wrong place.  CO's R = 9 is the natural
demonstration of the seed ladder, and the rung it lands on is recorded in that
geometry's own `seed` field.

**Both chains export `OMP_NUM_THREADS=1` and friends.**  Unset, scipy's bundled
OpenBLAS builds a spin-waiting pool of 63 threads in EVERY worker — 756 for a
12-worker pool, 148% CPU per worker for 100% of work, six cores of a shared
machine burned on nothing.  Measured before and after: `nlwp` 63 → 1, per-worker
CPU 148% → 87%, 781 fewer threads on the box from this lane alone.  It must be
set before numpy is imported, so it lives in the shell script.

Per species, to land: `--probe` → `--assemble` → `emit_engine.py
engine_handoff/elements1` → `verify_elements.py --quick` (must exit 0) → copy to
the repo path → `git commit -F <msgfile> -- <pathspec>` → tell the lead the pin.

## Machine

32 cores, ~31 GB.  Other campaigns take 6–12 and one takes 9.6 on its own.
Keep pools at 12.  Oversubscription looks exactly like "these determinants are
expensive".

## Standing rules for this lane

- Never trade digits for knots: a sparse exact referee is a referee.
- Every claim about a sweep must say where the claim would fail and whether the
  sweep goes there.
- Every emitted key must have a reader (`_inert_audit.py`); nine are deliberate
  human provenance, and that is a different status from unread.
- A guard must demonstrate a failing case, and must be shown to be CONNECTED.
- **A cache key must say what KIND of record it holds.**  Two record types in
  one namespace is not a naming problem; it is a data-corruption channel that
  presents as a stale-looking file.
