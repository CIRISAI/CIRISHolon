# SATURATION-1 D1 — detached runs

Launched from `engine/crates/holon-render/examples/quench.rs` with the protocol
frozen in `conformance/atomworld/SATURATION1_RESULTS.md` (commit 277ce67).

```
cd engine
./target/release/examples/quench pair    > output/quench_pair.log
./target/release/examples/quench mbe3    > output/quench_mbe3.log
./target/release/examples/quench plant3  > output/quench_plant3.log
# post-hoc, labelled, NOT the staked plant -- the localisation arms:
./target/release/examples/quench plant3b 1396790273 1396790274 > output/quench_plant3b.log
./target/release/examples/quench plant3c 1396790273 1396790274 > output/quench_plant3c.log
```

Seeds are given in DECIMAL on the command line; the staked eight are `0x53415401`
through `0x53415408`, i.e. 1396790273 to 1396790280.

Each writes one line per seed and two criterion lines at the end. A run is
complete when the `# worst energy drift / bound` line is present; the wrappers
also touch `output/quench_<arm>.DONE`. Session death kills only the narration:
every arm is `setsid`-detached and every number it needs is a `const` in the
runner, so a killed run is re-run, never resumed.

Wall cost on this machine (load average ~65): ~20-40 s per seed for the
pair-only arm, ~5-10x that with the three-body loop on. The three-body table is
solved ONCE per process (~2-6 s) and copied into each seed's scene.
