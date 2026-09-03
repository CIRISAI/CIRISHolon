# Re-banking checklist — the thick-restart arithmetic-regime boundary

**Owner:** lead (solver lane). **Boundary:** the commit carrying `tier.rs`'s thick restart,
2026-09-02. **Cause line, carried verbatim by every re-banking commit under this boundary:**
*re-banked under the thick-restart regime*.

This file is the successor of `REBANK_4884704.md` and follows its ruling exactly: the regime
is ACCEPTED, not reverted; artifacts re-bank per lane; the shared cause line is what kills
the each-lane-blames-its-own-last-commit failure.

## What changed, and why it was not optional

`tier.rs`'s `davidson_in` restarts THICK. When the subspace fills (48 vectors), it now
collapses onto the lowest `DAVIDSON_RESTART_KEEP = 16` Ritz vectors — carrying their images
forward as the same linear combinations of the sigma vectors already in hand — plus the
deflated residual. It used to keep exactly two: the current Ritz vector and the residual
deflated against it, and it recomputed the image with a fresh sigma application on the next
pass.

**The measured reason.** On a near-degenerate ground manifold the old restart threw the
manifold's partner states away every 48 iterations and the diagonal preconditioner had to
rebuild them from the residual. Measured on the (O,O) curve at 2,025 determinants
(`examples/pair_price.rs`, `HOLON_SOLVE_TIMING=1`, eight knots, this machine, 2026-09-02):

| restart | knot iterations, inward to outward | curve |
|---|---|---|
| old ({x, r⊥}) | 264 18 29 50 398 922 **5000 (cap, residual 2.6e-6)** 672 945 | 307 s |
| thick, keep 8 | 18 29 50 165 574 937 238 264 | 164 s |
| **thick, keep 16 (shipped)** | 18 29 53 163 406 469 241 247 | 143 s |

The cap row is the point: the old solver **silently shipped an unconverged knot**, labelled
`IterationCap` in the artifact and nothing else. The keep sweep and the subspace sweep
(48/96/128) are in `DAVIDSON_RESTART_KEEP`'s own doc comment; 16 at the priced 48-vector
subspace is the setting, and the budget is unchanged because 16 < 48.

A second, independent change rides the same boundary and moves **no bits**:
`lanes.rs`'s `MIN_ROWS_PER_SHARD` fell from 2,048 to 128, so pair-sized spaces shard across
threads at all (measured: one (O,O) sigma application 16–19 ms → 8.5–10.7 ms; the eight-knot
curve 143 s → 71 s). Sharding is row-disjoint and `lanes_gauge.rs` pins one shard against
many to the bit. Its trap — a producer's own worker pool multiplying with the kernel's
threads — is closed by `lanes::set_lane_threads_for_pool`, now called by every pooled
producer in `examples/`.

## Two classes (inherited from `REBANK_4884704.md`, unchanged)

| class | action |
|---|---|
| **CURRENT-ENGINE-OUTPUT** — the artifact is what this engine emits today | re-bank, carrying the cause line |
| **FROZEN CONTROL** — the artifact's whole value is being a snapshot of a prior state | retire-as-discharged; never re-bank |

## The enumeration

Empirical, from a full `holon-chem` and `holon-render` release run at the boundary, plus the
page's own gate. **Measured per artifact, never inferred from a rule.**

| artifact | class | moved? | measurement | action |
|---|---|---|---|---|
| `holon-chem/tests/data/s2/s2_water_table.txt` | current-engine-output | **YES** | 26,386 of 105,105 nodes; median 3.98e-13, p99 7.22e-12, max 1.73e-7 Ha — 4,400× inside the campaign's declared 7.68e-4 | re-banked; superseded bytes kept as `s2_water_table.stale_pre17bc115.txt`; the record and the new ~1e-7 long-range floor are in `SATURATION2_RESULTS.md` |
| `docs/workbench/tables/s2_water_table.txt` | served copy of the above | **YES** | byte-identical to the re-banked table (the smoke gate requires it) | re-copied; the page's SHA-256 pin updated |
| `holon-render/tests/data/many_body_identity.receipt` | current-engine-output | **YES** | 6 of 40 pinned entries, all forces, on one atom of each scene; ≤1.07e-7 Ha/bohr against a receipt whose scale is 0.788 | re-banked via `HOLON_MANY_BODY_RECEIPT=write` |
| `docs/workbench/tables/HO.json` | current-engine-output | **NO** (numerically) | 0 of 192 energy knots differ; every scalar bit-identical; only `generation_ms` changed | re-emitted anyway (the field is part of the file); pin updated |
| `docs/workbench/tables/O2.json` | current-engine-output | n/a — first emission | 2,025 determinants, 192 knots, 1,785 s, exit `converged`, uncertainty 9.99e-11 | emitted UNDER the new regime; it is the curve the old restart could not emit without capping a knot |
| `docs/workbench/law_probe.json` | current-engine-output | **NO** | the wasm/native bit-identity probe re-ran clean against the rebuilt artifact | untouched |
| `docs/atoms/tables/*.json`, `docs/unified/tables/*.json` (HCl, Cl2) | current-engine-output | see status below | 100 and 324 determinants | governed by their own lane's gates |
| `docs/atoms/tables/*.stale_pre4884704.json` | **frozen control** | — | snapshots of the prior regime | never re-banked |
| `conformance/atomworld/*_referee/*.json` | current-engine-output | see status below | referee tables read by tolerance gates | tolerance gates cannot see moves of this size; re-bank only where an identity gate fires |

## Re-bank in DEPENDENCY ORDER, or the second bank is stale

The many-body receipt was banked once, correctly, from a build that already carried the new
solver — and it went RED again on the next full run. The cause was not the solver: the
`quartet` scenes READ the `(O,H,H)` table (`tests/common/quartet.rs` loads
`s2_water_table.txt`), and the receipt had been written before the re-banked table was
installed, so it had pinned the new solver against the old table. **An artifact that reads
another artifact must be re-banked after it, not beside it.** Banked again in order, the
receipt reproduces itself (`many_body_identity` 3/3).

The same ordering governs the page: the served copy of the table, then the SHA-256 pins in
`app.js`, then the smoke gate.

## Status of the full runs at the boundary

| suite | result |
|---|---|
| `cargo test --release -p holon-chem` | **252 tests, 0 failures** — including the identity gate `water.rs::the_committed_table_is_this_build_s_own_output` on the re-banked table, the 50-digit external referee `water_referee.rs` (R1), and every pair/element/dimer/ion bank |
| `cargo test --release -p holon-render` | green after the receipt was re-banked in dependency order (`many_body_identity` 3/3); the full `--no-fail-fast` run is the record |
| `docs/workbench/smoke.mjs` | **376 checks, 0 failures**, including the new arms: the shipped (H,O) and (O,O) curves ADMITTED by the bank's provenance gate, the water door's bit-exact push, the fence count dropping by exactly the (O,H,H) family, and a STEPPING O:2H scene with its census |

**No external referee moved.** `water_referee.json` and `referee_h2_sto3g_fci.json` are
frozen by definition and were not touched; the gates that read them pass on the new solver,
which is the check that says the regime moved the engine's own arithmetic and not its
agreement with an outside number.
