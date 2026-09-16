# RUNG 2 — AMENDMENT 1: the density field binned at its own Poisson scale, the grid given its third axis, and the grid ladder derived rather than listed

*Written 2026-09-16, committed alone, BEFORE any trajectory is read under it. Amends
`RUNG2_PREREG.md` (frozen `aee5317`) in three named places and nowhere else. The freeze's
reading (`RUNG2_RESULTS.md`, branch (d)) stands as banked and is not re-graded; the freeze's
exact chart is RUN BESIDE the amended chart on every carrier from now on, as the instrument's
own control, so the two readings sit side by side and neither replaces the other.*

## Why an amendment, from the freeze's own arithmetic

The freeze defines a fluid element by G2: a cell holds `≥ 100` atoms so that its Poisson
density fluctuation `1/√N ≤ 0.10`. It also reads the density field as **exact integer
occupancy** (§2.3: *"occupancy is already integral"*) and tests closure by the **collision
form** (G5: two frames with identical readings must have identical successors).

Those three together cannot certify anything. At `⟨n⟩ = 100` a cell's occupancy wanders over
`±20` values; four such cells have `~10^5` distinct exact readings, against a few hundred
frames. Readings all but never repeat, and the verdict is **VOID by counting — no collisions, or
far too few to meet G4's 200 — at any box size, on any dynamics.** (Plant PA-1 sharpened
this sentence: a frame-to-frame-correlated walk repeats a few readings, 73 in 600 frames,
so "none" overstated it; "below the work count" is exact and is the grade the real 2×1 chart
received.) The trend is already in
the record: on the 3D carrier (`replace0/rung2_occ/`, read 2026-09-16) collisions fall
`1,498 → 77 → 9 → 0` as occupancy goes `64 → 32 → 16 → 5`. The freeze's G2 (large cells) and
G4 (`≥ 200` informative transitions) pull against each other at the exact-occupancy rung.
This is a fault in the instrument's letter, found by reading it against the carrier it was
written to reach, and it is the same shape as `LIQUID2_AMENDMENT_1.md`: a rule correct on the
scene it was staked on (`N = 12`, where occupancies `0–12` repeat freely) and wrong on the
scene it was for.

The momentum and energy fields do not have this fault, and the reason is instructive: the
freeze **binned** them, at scales *"derived from protocol constants only"* — `Δp`, the thermal
momentum; `Δe`, one thermal quantum. The density field was left exact because at `N = 12`
exact was fine. The repair is to give density the same treatment its siblings already have.

## The three changes

### A1 — the density field is binned at its own Poisson scale

> **Δn = √⟨n⟩**, where `⟨n⟩ = N_atoms / cells` is exact arithmetic. Bin index is
> `floor(n / Δn)`, ties to `−∞`, as for `Δp` and `Δe`.

The derivation is the freeze's own: G2's `100` comes from `1/√N ≤ 0.10`, so the freeze
already says a fluid element's density is known to one Poisson standard deviation. `Δn` is
that deviation. It is parameter-free (no measured input), it parallels `Δp` and `Δe` exactly,
and it is what makes a fluid element's *reading* match its *definition*. At `⟨n⟩ = 100`,
`Δn = 10`: a cell's density is read to the `10 %` the freeze defined it by.

The exact chart is strictly finer than the binned one, so `refines(exact, binned)` must hold
on every trajectory — an instrument self-check, added to the plants below. And the exact chart
is **still run and still printed** on every carrier: on the freeze's own `N = 12` scene it is
the reading of record, and on any other it is the control that shows what binning bought.

### A2 — the grid has a third axis

> A cell grid is `(n_x, n_y, n_z)`. On a carrier with `dims = 2` the instrument sets
> `n_z = 1` and the chart is the freeze's, bit for bit. On a `dims = 3` carrier a cell is a
> box, not a column through the box.

The freeze's cells are `(n_x, n_y)` because the certified scene was two-dimensional; §2.1 says
*"an Eulerian grid over the box"* and a column through a 3D box is not that. The 2026-09-16
reading on the 3D carrier declared its cells as columns and was honest about it; this makes
the chart what §2.1 meant.

### A3 — the grid ladder is derived from the carrier, not listed

> Cells double: `2^k` cells for `k = 0, 1, 2, …` while `⟨n⟩ ≥ 1`, each doubling splitting
> the longest remaining axis. `2^0` is the vacuity control of G3, as `(1,1)` was.

The freeze listed five grids for `N = 12` (§2.5) to keep the grid from being a free
parameter. On a carrier of any other size the list must be derived by a rule stated before
the data, or it is a free parameter again. The doubling rule reproduces the freeze's first
four grids exactly (`1, 2, 4, 8` cells → occupancies `12, 6, 3, 1.5`); its fifth, `6×4`, is
not on the ladder and is kept in the frozen mode. On the 128-water 3D carrier the ladder is
`1, 2, 4, 8, 16, 32, 64, 128` cells (`128 … 1` per cell).

## What does not change

The collision form (G5), Leg B (G6), the vacuity fence (G3), the work count (G4 — `≥ 200`,
carried from the census unchanged), the position-blind controls and the separation bar (G7),
the refinement self-checks (G8, both forms), the species null (G9a), the momentum and energy
bins (§2.3), the budget `β = 0.02`, the admissibility bar (G2 — `≥ 100`, `≥ 4` cells,
fluctuation `≤ 0.10`), and every branch of §5. **Nothing this amendment adds lowers a bar.**

## What it makes reachable, stated as arithmetic

| carrier | cells | `⟨n⟩` | `Δn` | G2 |
|---|---|---|---|---|
| 128 waters (this one) | 4 (2×2×1) | 32 | 5.7 | NO — occupancy |
| **400 waters** | 4 (2×2×1) | 100 | 10 | **at the bar** — fluctuation `0.10` exactly |
| **800 waters** | 8 (2×2×2) | 100 | 10 | the first admissible **cube** |
| 1,000 waters | 8 (2×2×2) | 125 | 11.2 | inside, fluctuation `0.089` |

So the smallest carrier on which this instrument can return anything but VOID for the fluid
element is **400 waters at four cells**, and the first cubic one is 800. Those numbers are
what a box has to be, and they are the reason the instrument was amended before a box was
priced: on the freeze's exact chart a 1,000-water box would have returned `VoidNoCollisions`
and taught nothing.

## Plants, and what each must do

| plant | carrier | must |
|---|---|---|
| **PA-1** | a synthetic 400-atom random walk in a `2×2×1` grid | on the **exact** chart: VOID by counting (`VoidNoCollisions` or `VoidWorkCount`) — the fault, exhibited; on the **binned** chart: collisions `> 0` and G4 met on the same frames |
| **PA-2** | P-2's closed-by-construction trajectory | `CertifiedStrict` on the binned chart too — binning does not manufacture a defect |
| **PA-3** | P-3's hidden-variable trajectory | `NotClosed` on the binned chart — binning does not hide one |
| **PA-4** | any `dims = 2` trajectory | the `(n_x, n_y, 1)` chart's readings are **bit-identical** to the freeze's `(n_x, n_y)` — backward compatibility, exact |
| **PA-5** | every trajectory | `refines(exact, binned)` — the self-check; a violation convicts the binning, not the trajectory |
| **PA-6** | `N = 12`, `dims = 2` | the derived ladder's first four grids are the freeze's first four, cell for cell |

A plant that cannot fire is refused; each of these fires on its own carrier in
`holon-lens/src/field.rs`'s tests, and the results document cites the run.

## The first reading it will be pointed at

REPLACE-0's banked walks, both arms, three seeds — the same files as the 2026-09-16 exact
reading, so the two charts are compared on identical frames. Expected, stated now: G2 still
NO on every grid (128 waters admit no admissible cell); the `2×1×1` binned chart collects
more collisions than the exact one and its `D_A` is reported beside the exact `0.62–0.74`;
the `2×2×1` and `2×2×2` binned charts reach G4 where the exact ones did not. Whether any of
them is inside `β` is the reading, and it is not predicted.
