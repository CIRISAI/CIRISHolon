# CRYO-H-O — run state

Worktree: /tmp/claude-1000/cryo-wt  (branch cryo-h-o, off 16f6602)
Prereg:   conformance/atomworld/CRYO_HO_PREREG.md @ fc7b6a0 (ADMITTED, frozen before any instrument existed)
Results:  conformance/atomworld/CRYO_HO_RESULTS.md

## Runs, all detached (setsid + done-marker). Run state is untracked; cited logs are committed.

| marker | log | arm / gates | status |
|---|---|---|---|
| arm1_dimer.DONE | arm1_dimer.log | ARM 1 G1/G2/G3, plants P1/P2 | COMPLETE |
| (none) | arm1_tail.log | ARM 1 POST-DATA tail + floor control + 0.25 well refinement | COMPLETE |
| o2_spin.DONE | o2_spin.log | ARM 2 G5/G6, plant P3, + POST-DATA <S^2> along the curve | COMPLETE |
| compress.DONE | compress.log | ARM 3 G9/G10, plant P2, V4/V5, unit fence | COMPLETE |
| quench_hydrogen.DONE | quench_hydrogen.log | ARM 1 G4 + classifier, plants P4/P4b, V1 leg 2 | COMPLETE |
| order_probe.DONE | order_probe.log | POST-DATA order-channel diagnostic (jitter sweep, interior_atoms) | COMPLETE |
| quench_oxygen.DONE | quench_oxygen.log | ARM 2 G7/G8 + classifier, V1 leg 2 | LONG — the 96-knot O-O curve is 2025 determinants a knot |

## Dead, kept and marked
quench_hydrogen.NO_H3_TABLE.log — the first hydrogen run, before the H3 three-body surface
was loaded. It read a 7-to-12 atom aggregate at every rung including 300 K, against the
banked 44 x H2. Kept because a record that deletes what it got wrong cannot be audited.

## Rebuild / re-run
cd /tmp/claude-1000/cryo-wt/engine
cargo build --release -p holon-chem -p holon-render --examples
./target/release/examples/cryo_h2_dimer      > output/cryo/arm1_dimer.log
./target/release/examples/cryo_h2_tail       > output/cryo/arm1_tail.log
./target/release/examples/cryo_o2_spin       > output/cryo/o2_spin.log
./target/release/examples/cryo_h_compress    > output/cryo/compress.log
./target/release/examples/cryo_quench hydrogen > output/cryo/quench_hydrogen.log
./target/release/examples/cryo_quench oxygen   > output/cryo/quench_oxygen.log
./target/release/examples/cryo_order_probe   > output/cryo/order_probe.log

Every runner is deterministic: seeds and protocol are consts, not flags.
