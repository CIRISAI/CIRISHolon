# q8-mps full-grid gate run — RESUME (re-run after Amendment 2)

**Superseded run:** the original `N=8,10,12` grid stalled 85+ min on `N=12`'s exact reference
(resource contention, not an algorithmic defect — see `Q8_MPS_PREREG.md` Amendment 2 and
`examples/probe_n12.rs`). Both jobs were KILLED 2026-08-24 on Eric's direct instruction via
team-lead; their logs are preserved as `output/q8_mps/{full_grid_gates,g4_certificate}.log`
with `.DONE` marked `KILLED` (not a crash).

**This run:** `N ∈ {8,10}` only, per Amendment 2. Launched SERIALLY (not concurrently with
`g4_certificate`), per the new binding scheduling rule the amendment adds: probe the
environment before launching, serialize by default. Environment probed first (`free -h`,
`uptime`, `ps aux --sort=-%cpu`) — confirmed persistent desktop load (Firefox/Chrome, ~1.6+
cores sustained) but no other heavy job running.

**Command:**
```
setsid cargo test --release --manifest-path crates/q8-mps/Cargo.toml --test full_grid_gates \
  -- --ignored --nocapture > output/q8_mps/full_grid_gates.log 2>&1 &
```

**Done marker:** `output/q8_mps/full_grid_gates.DONE` (overwritten from the killed run's marker
once this run completes) — contains the exit code.

**On resume:** tail the log; `g4_certificate` is launched only AFTER this one's DONE marker
appears (serialized, not concurrent) — see `RESUME_G4.md`.
