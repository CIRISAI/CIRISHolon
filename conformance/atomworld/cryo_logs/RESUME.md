# CRYO-H-O — run state

**Every path below is repo-root-relative.** Gate 10a3's rule applied to a record rather
than to an instrument: the re-run recipe must work in any checkout, and where the run
actually happened is PROVENANCE, stated once here and never baked into a command.

*Run provenance, for this record only: the campaign ran in a `cryo-h-o` worktree under the
session's scratch directory, off `16f6602`. That directory is gone the moment the session
is; nothing below depends on it.*

Prereg:  `conformance/atomworld/CRYO_HO_PREREG.md` @ `fc7b6a0`, ADMITTED by
         `Audit/prereg_audit.py` before any instrument in this campaign existed.
Results: `conformance/atomworld/CRYO_HO_RESULTS.md`
Logs:    `conformance/atomworld/cryo_logs/` (this directory — the committed copies).
Run state lived in `engine/output/cryo/`, which is untracked by design.

## The runs

| log (committed here) | arm / gates | status |
|---|---|---|
| `arm1_dimer.log` | ARM 1 G1/G2/G3, plants P1/P2 | COMPLETE |
| `arm1_tail.log` | ARM 1 POST-DATA tail + floor control + 0.25 bohr well refinement | COMPLETE |
| `o2_spin.log` | ARM 2 G5/G6, plant P3, POST-DATA `<S^2>` along the curve | COMPLETE |
| `compress.log` | ARM 3 G9/G10, plant P2, V4/V5, the unit fence | COMPLETE |
| `quench_hydrogen.log` | ARM 1 G4 + classifier, plants P4/P4b, V1 leg 2 | COMPLETE |
| `quench_oxygen.log` | ARM 2 G7/G8 + classifier, V1 leg 2 | COMPLETE (curve 841.5 s, then 15 runs) |
| `order_probe.log` | POST-DATA order-channel diagnostic (jitter sweep, `interior_atoms`) | COMPLETE |

## Dead, kept and marked

`quench_hydrogen.NO_H3_TABLE.log` — the first hydrogen run, before the H3 three-body
surface was loaded. It read a 7-to-12 atom aggregate at every rung including 300 K,
against the banked 44 x H2. Kept because a record that deletes what it got wrong cannot
be audited.

## Re-run, from any checkout

```sh
cd "$(git rev-parse --show-toplevel)/engine" || exit 1
cargo build --release -p holon-chem -p holon-render --examples
mkdir -p output/cryo
for r in cryo_h2_dimer cryo_h2_tail cryo_o2_spin cryo_h_compress cryo_order_probe; do
    ./target/release/examples/$r > "output/cryo/$r.log" 2>&1
done
./target/release/examples/cryo_quench hydrogen > output/cryo/quench_hydrogen.log 2>&1
./target/release/examples/cryo_quench oxygen   > output/cryo/quench_oxygen.log 2>&1
```

`git rev-parse --show-toplevel` prints nothing outside a checkout, and `cd ""` succeeds
into `$HOME` — which is why the `|| exit 1` is there and why the recipe is written with
it. That failure mode was measured elsewhere in this tree today; it is not hypothetical.

Every runner is deterministic: seeds, box, schedule and knot count are `const`s in the
source, not flags. `cryo_quench` takes one argument, the arm.
