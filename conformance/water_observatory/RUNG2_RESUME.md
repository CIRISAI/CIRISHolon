# Rung-2 (fluid-element tier) lane — RESUME

**State: COMPLETE. Verdict banked. No computation is running and none is pending.**

## Verdict in one line

The fluid-element tier is NOT certified; `RUNG2_PREREG.md` branch **(d) — inadmissible
carrier** — and the 1 km face does not flip. `RUNG2_RESULTS.md` is the reading.

## What is banked (committed, green)

| thing | where |
|---|---|
| the stakes, frozen before the instrument | `RUNG2_PREREG.md` (`aee5317`, ADMITTED by `Audit/prereg_audit.py`) |
| the instrument | `engine/crates/holon-lens/src/field.rs` — 16 tests, every plant of §7 |
| the runner | `engine/crates/holon-lens/examples/rung2.rs` |
| the measured results | `RUNG2_RESULTS.md` |
| the log they summarise | `rung2_chart.log` (2,129 lines) |
| G1's digest check | `rung2_g1_digests.log` (23 files OK, exit 0) |

Reproduce, from a clean checkout:

```
cargo test -p holon-lens                      # instrument + plants, no holon-render needed
cd /home/emoore/holon-artifacts/census-traj && \
  sha256sum -c <repo>/conformance/water_observatory/census_traj_manifest.sha256
cargo run --release -p holon-lens --example rung2 -- \
  /home/emoore/holon-artifacts/census-traj fenced hydrogen
```

`holon-lens` has ZERO dependencies, so this lane never needed `holon-render` to compile.
The whole campaign is 320,000 frames read and 18.0e6 chart evaluations; it runs in about a
minute on one core, so nothing here was ever detached and there is no done-marker to find.

## The four freeze defects, so nobody re-finds them

All in `RUNG2_RESULTS.md` §5, none repaired in place:

1. **R1** refuses a whole trajectory for one atom-frame 0.0418 bohr outside a *soft* wall
   (1 of 3,840,000). Hydrogen seed `0x53415425` is excluded and its counterfactual reading
   was deliberately not computed. A successor freeze owes a tolerance equal to the wall's
   softness.
2. **G8's staked form is too weak.** Monotone collision counts do not establish the
   refinement hypothesis; `field::refines` does, and it is what caught P-6.
3. **G9b/G9c are not computable** from the trajectory artifact — no forces, no intervention
   ledger in the dump. Marked UNDISCHARGED, not failed.
4. **The freeze's own blind control is degenerate** (`BlindIndex`: constant membership, zero
   transport). `BlindLabel` is the control every G7 number uses.

## What is owed, and by whom

Nothing by this lane. The band's fence now carries numbers (`RUNG2_RESULTS.md` §6) and the
exit is reported UNDETERMINED under the freeze's own pre-committed branch. Two successor
routes are named there — a ≥400-atom carrier with a format v2, or a continuum-native tier
on its own dynamics — and **neither may be composed into this ladder without its own
freeze**.
