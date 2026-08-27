# q8-mps G4 certificate run — RESUME

**Started:** 2026-08-23 (this session). **Command:**
```
setsid cargo run --release --manifest-path crates/q8-mps/Cargo.toml --example g4_certificate \
  > output/q8_mps/g4_certificate.log 2>&1 &
```

**What it's doing:** `examples/g4_certificate.rs` — the chi ladder {16,32,64,128,256} at all 12
(N,U) configurations (N in {8,10,12}, U/t in {0,1,4,16}), 60 DMRG runs total. Fits
`dE = c*eps^p` (log-log OLS) calibrated on N=8,10, held out on N=12, per `Q8_MPS_PREREG.md` §5.
Runs concurrently with the separate `full_grid_gates` job (both single-threaded, 32 cores
available, no expected resource contention beyond wall-clock sharing).

**Expected runtime:** the smaller chi rungs (16,32,64,128) are markedly cheaper than 256, so
total cost per (N,U) is roughly 1.3-1.6x a single chi=256 run, not 5x. `full_grid_gates` (12
chi=256-only runs, PLUS a q-seam exact call each) ran roughly 60-90s per configuration; expect
this job to run somewhat longer in aggregate given the extra rungs but each individual rung
below 256 completes quickly — no single-point stall expected.

**Done marker:** none written automatically by this job (it's `cargo run`, not `cargo test`,
so there's no pass/fail exit code semantics the way `full_grid_gates` has) — completion is the
process exiting and `g4_certificate.log` containing the `=== G4 VERDICT ===` block. Check with
`pgrep -af g4_certificate` and `tail output/q8_mps/g4_certificate.log`.

**On resume:** the log's raw-points table at the end (or as far as it got) has every
`(N,U,chi,epsilon,dE)` tuple computed so far — usable even from a partial/killed run, since
each line is independent and printed as it completes (not buffered to the end). The FIT/VERDICT
block only appears once all 60 points are collected (the fit is a single pass over everything at
the end, not incremental) — a truncated log has data but no verdict yet.
