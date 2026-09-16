# The fluid-element chart on the 3D carrier — occupancy rung, both arms of REPLACE-0, frozen instrument unchanged

*Read 2026-09-16 at zero compute, on data already banked: REPLACE-0's oxygen walks for both
arms of three seeds (`transport_seed0..2/{flexible,rigid}.walk`, 151 readouts at 20 fs,
~3 ps each), written as HLNTRAJ1 v1 by `walk2traj.py` and read by
`holon-lens/examples/rung2.rs` exactly as frozen at `aee5317` — every bar printed below is
the freeze's. Log: `rung2.log`. The instrument was not modified.*

## Why this was read, and what it is

`OBJECT.md`'s weight table names one mover for separating its readings B and C: *the 3D
carrier re-running rungs 1 and 2*. The lead had priced the fluid tier at ~59,000 core-hours;
that is the price of a CLASSICAL comparison (Green–Kubo viscosity on the flexible model),
not the price the object levies. The object's price for a tier is design rule 1's battery —
a closure defect against a named coarse view, non-expanding, non-vacuous — and it is a
short-time test on trajectories that exist. This is that test at the rung the banked data
can carry: **the density field over cells**, `Rung::Occ`, on the 128-water periodic box.

## Declared before the numbers — what these files are and are not

- `TrajWriter::create` asserts `n_atoms ≤ 16` (the v1 bond bitset is C(16,2) bits). The
  reader has no such check and the occupancy chart never reads bonds, so a v1 file with
  128 oxygens and an EMPTY bond set is read exactly. **Bonds are empty: rung 1 must never
  be pointed at these files.**
- **Velocities are zero.** The `Mom` and `Ene` rungs are degenerate copies of `Occ` (checked:
  identical collision counts and defects) and are NOT readings. Only `Spatial Occ` lines mean
  anything and only those are quoted.
- The grid ladder is the freeze's, staked for N = 12 in 2D; read here at N = 128 in 3D the
  cells are COLUMNS through the box (x, y only). Occupancies 128 / 64 / 32 / 16 / 5.3.
- Positions are the walks' unwrapped oxygens wrapped into the box; `cell_series` refuses
  atoms outside and refused none.

## The reading

| grid | cells | mean occupancy | transport | arm | D_A (mean of 3 seeds) | informative (per seed) | verdict |
|---|---|---|---|---|---|---|---|
| 1x1 | 1×1 | 128.0 | 0.00 | flexible | 0.000 | 150, 150, 150 | VoidVacuous |
| 1x1 | 1×1 | 128.0 | 0.00 | rigid | 0.000 | 150, 150, 150 | VoidVacuous |
| 2x1 | 2×1 | 64.0 | 0.62 | flexible | 0.667 | 150, 149, 149 | VoidWorkCount |
| 2x1 | 2×1 | 64.0 | 0.67 | rigid | 0.689 | 150, 150, 150 | VoidWorkCount |
| 2x2 | 2×2 | 32.0 | 0.89 | flexible | 0.920 | 93, 73, 88 | VoidWorkCount |
| 2x2 | 2×2 | 32.0 | 0.90 | rigid | 0.946 | 86, 76, 85 | VoidWorkCount |
| 4x2 | 4×2 | 16.0 | 0.97 | flexible | 0.952 | 18, 11, 4 | VoidWorkCount |
| 4x2 | 4×2 | 16.0 | 0.95 | rigid | 0.952 | 22, 11, 16 | VoidWorkCount |
| 6x4 | 6×4 | 5.3 | 1.00 | flexible | n/a (no collisions) | 0, 0, 0 | VoidWorkCount |
| 6x4 | 6×4 | 5.3 | 1.00 | rigid | 1.000 | 2, 0, 2 | VoidWorkCount |

Species/arity constant on every file (G9a). Transport fraction ≥ 0.57 on every grid past
1×1, so no chart past the whole box is frozen (G3). Zero refusals.

## What it says

1. **G2 admissibility is NO on every grid, by arithmetic.** The freeze's fluid element needs
   ≥ 100 atoms per cell AND ≥ 4 cells AND fluctuation ≤ 0.10. At 128 oxygens the only grid
   with 100 per cell has one cell. **A 128-water box cannot host a fluid-element chart the
   freeze would admit; the smallest that can is ≥ 400 waters at the coarsest admissible
   grid, and the fluctuation bar will push it higher.** This is a size fence, now measured
   rather than inferred, and it is the concrete meaning of "the next tier is not buildable
   on this carrier".
2. **Rung 1's disjointness reproduces on the 3D carrier at the density rung.** The whole-box
   chart (1×1) is closed and VOID-vacuous, as the freeze demands. The first dynamic chart
   (2×1, two half-boxes) has `D_A` = 0.62–0.74 on every arm and every seed — thirty times
   the budget β = 0.02 — and every finer chart runs out of collisions. Every dynamic chart is
   out of budget; the only closed chart is vacuous.
   **Fence:** at 151 readouts the 2×1 chart has 149–150 informative transitions against the
   freeze's 200, so the instrument grades it `VoidWorkCount`, not `NotClosed`, and that is
   the correct grade. **Staked now, before any longer run:** at ≥ 200 informative
   transitions the 2×1 verdict is `NotClosed`; a defect of 0.65 does not move to 0.02 with
   more data.
3. **The rigid operator's density chart is as open as the fine model's, to within the seed
   spread** (0.689 vs 0.667 at 2×1; same transport fraction; same collapse of collisions up
   the ladder). On the one chart the banked data can carry, the coarse self and the fine self
   agree — which is the recursive self-comparison at this rung, and it is a pass for the
   operator on faithfulness, not a pass for the chart on closure.

## What is NOT read, and what each would cost

| rung | needs | status | price |
|---|---|---|---|
| Occ, ≥ 200 informative | more readouts | staked `NotClosed` above | one REPLACE-0 flexible arm re-run with pos+vel banked, ~22 core-h; or LIQUID-2's three arms, ~60 core-h |
| Mom, Ene | velocities banked (v1 layout with empty bonds suffices) | not banked by any run to date | same run as above |
| rung 1 (the network) | hydrogens and bonds; v1 cannot hold C(384,2) | blocked on trajectory format v2 (the "16-atom cap" the fluid band's exit names) | engine work, then the same run |
| a fluid-element chart the freeze ADMITS | ≥ 400 waters, likely more | not this carrier | a new box, priced at the size phase before any arm |

No weight in `OBJECT.md` moves on a VOID reading. What moves B against C is the same chart
at ≥ 200 informative transitions, and its expected grade is written above so the run can be
judged against it.
