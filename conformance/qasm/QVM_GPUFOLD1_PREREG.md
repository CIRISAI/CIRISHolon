# QVM-GPUFOLD-1 — the budgeted prefix folded on the device: PREREGISTRATION

*Frozen 2026-09-23, committed alone before any code. QVM-ACUITY-1 read branch (a) and named
"the GPU branch fold" as the next freeze (`QVM_ACUITY1_PREREG.md` §4 (a); results §4 "Owed").
The budgeted sum's hard part is a fold of an evaluated PREFIX of a declared branch order,
and its S4 mesh on seven E-cores read `0.27×` at `S = 4` and `0.25×` at `S = 8` against a
seven-core floor of `0.143`. `holon-gpu` already folds a batch of affine branch descriptors
on CUDA with the ring taken out of the reduction (the host aligns every branch to one
denominator exponent and the device sums `i128` coefficient lanes, so the schedule is not
an input), and its `tests/determinism.rs` holds it to `holon::mesh::fold_amplitude` as a
struct. This freeze wires that fold UNDER the budgeted sum and prices it. Nothing about the
budget moves: the stopping decision is `BudgetPlan::stop_at(ε)`, a pure function of the
plan, and it stays on the CPU; the device only folds the prefix the plan names.*

## 1. Instances

ACUITY-1's family (`random_circuit`: `t` T/T†-gates placed at random among `20n` Clifford
gates over `{h, s, sdg, x, z, cx}`), one seed per cell, the branch source
`acuity::certified_source_for` (the certified scalar bound — the one that truncates), `y`
the first draw with a live amplitude (ACUITY-1's `y_with_signal`). G1/G2 grid:
`n ∈ {12, 16, 20, 24}` × `t ∈ {16, 20, 24, 28}` (`N = 108 … 2,916`). G3 sizes: `n = 24`,
`t ∈ {24, 28, 32}` → `N = expected_branches(t) = 972, 2,916, 8,748` (`3^7 · 2^2` at `t = 32`;
the register is `n + t ≤ 56` wires, under the descriptor's 64-wire cap). The device is the
box's RTX 4090 Laptop GPU (16 GB), device ordinal 0, declared, not detected.

## 2. Stakes, each with its kill

- **G1 — bit-identity, at every prefix.** For every instance and EVERY prefix length
  `k ∈ [0, N]` of the plan's order, the GPU fold of `order[..k]` is the SAME `Cyc` STRUCT
  (coefficients and exponent, `==`) as `holon::mesh::fold_amplitude` over the same prefix at
  `S = 8`; and at every `ε ∈ {0, 10⁻¹, 10⁻², 10⁻³, 10⁻⁴, 10⁻⁶}` and the relative rungs
  `ε · 2^{−n/2}` for the same three exponents, the budgeted result on the GPU backend
  carries the same `(value, remainder, k, N, order)` as on the CPU mesh backend. EXACT.
  The upload's `parity_uniform` flag is reported per instance (`gpu.rs`'s header: struct
  equality is guaranteed when every branch shares one exponent parity and only observed
  otherwise). **Kill:** one `(instance, k)` with unequal structs. If structs differ and the
  values are equal after `mesh::canonicalize` on both sides, G1 is still KILLED as staked
  and the reading is "value-identical, representation not": no canonicalisation is applied
  after the fact to rescue it.
- **G2 — the device class is part of the artifact.** Every budgeted result carries a
  `DeviceClass` (`holon-device`, the one definition); the backend is chosen EXPLICITLY by
  the caller (`FoldBackend`), never auto-detected, and a GPU backend that cannot open its
  device fails loudly rather than falling back to the host. A CPU-mesh run of the same plan
  carries `Cpu`, and asking it to stand as a `Gpu` artifact (`require_class(Gpu)`) is
  REFUSED; a table that mixes classes is refused. EXACT. **Kill:** one CPU-produced result
  admitted as `Gpu`, or one silent fallback.
- **G3 — the device pays, round trip included.** `ε = 0` (the full prefix `k = N`), `n = 24`,
  `N ∈ {972, 2,916, 8,748}`. Arms: the CPU mesh at `S = 8` on `taskset -c 21-27`
  (seven E-cores, oversubscribed exactly as ACUITY-1's S4 was, so the two are the same arm);
  the GPU per-query fold with the batch resident — launch, kernel, and the device→host copy
  of the block partials with the host's final lane sum — which is the ROUND TRIP, INCLUDED
  in the staked number; the kernel alone (CUDA events) reported beside it so the round
  trip's share is separate; and the COLD query (descriptor decode + host→device upload +
  the query) reported beside both. 25 repetitions each; min, median and the max/min spread
  on every row (SATURATION-3 G2 found the round trip carried a `1.97×` spread, so the
  median is read beside the min, not replaced by it). Stake: GPU round-trip wall ≤ `0.25×`
  the CPU `S = 8` wall at `N = 8,748`, on the min AND on the median. **Kill:** `> 0.75×` at
  `N = 8,748` on the median. The cold ratio is reported with no stake; it is named if it
  exceeds `1.0×`. The CPU arm's loadavg is printed on every row.

## 3. Plants, each of which must fire before a stake is read

| plant | must |
|---|---|
| **PG-1** | a corrupted lane ON THE DEVICE: one `i128` coefficient limb of one live branch's resident base overwritten after upload; the device fold then disagrees with the CPU twin (`holon_gpu::cpu::fold_packed` over the same descriptors) and the audit names the prefix position, the branch and the lane |
| **PG-2** | a scrambled schedule: the same prefix folded at five block/grid shapes (`determinism.rs`'s `SHAPES`) and uploaded in a permuted branch order returns the same struct |
| **PG-3** | a planted wrong branch (one term's sign flipped, its bound unchanged — ACUITY-1's PQ-4): the GPU-backend `ε = 0` sum disagrees with the dense statevector referee by `2|term|` to `10⁻⁹`, which exceeds `R_k = 0`; and at every `ε ∈ {10⁻², 10⁻³, 10⁻⁴}` whose prefix contains the plant, the disagreement exceeds the certified `R_k` |

## 4. Branches

- **(a)** G1–G3 met → the budgeted sum has a device backend that is the same artifact up to
  its declared class, and pays; banked; the next freeze is the marginal (`2^{|L|−|S|}`
  amplitudes against one resident batch) and the `t ≥ 36` sizes where the host fold is minutes.
- **(b)** G1 killed → the device fold is not the same struct; the parity reading decides
  whether the remedy is the upload's exponent choice or the kernel; nothing else is read.
- **(c)** G3 killed → the device does not pay at these sizes; the finding is where the
  wall goes (launch, copy, kernel), from the three reported numbers.
- **(d)** G3 between stake and kill → the ratio is the number, reported, not banked as met.
- **(e)** a plant fails → nothing is read.

## 5. Cost

G1: 16 instances × every prefix — `Σ k ≈ N²/2 ≈ 4.3·10⁶` branch amplitudes at `t = 28` on the
CPU side (cached per branch), and one upload per prefix on the device: minutes on cores
21–27. G3: seconds per arm. Source construction at `t = 32` (8,748 branch evolutions on 56
wires) dominates, once per size.

## 6. What this does not test

The marginal; circuits outside Clifford+T; any device other than ordinal 0 of this box; the
face ring and qutrit tiers; a GPU-side stopping rule (there is none: the stop is the plan's).

---
witness: `lanes_shardedFold_invariant` (MergeLaw.lean) for G1 and PG-2 — the merge law at its lane carrier, any two shardings agree; none for G2, G3, PG-1, PG-3 (measured gates)
**misfits:** M-CHEAPER-THAN-ITS-PRICE, M-DEVICE-CLASS, M-IDLE-CALIBRATED-TIMEOUT, M-PARITY-PROTECT, M-PLACEMENT-LOTTERY, M-PLANT-OBS, M-PLANT-SECTOR, M-VACUOUS-SUCCESS, M-VALIDATED-NOT-WIRED — contacted by keyword, cited. M-VALIDATED-NOT-WIRED is this campaign's reason to exist: `holon-gpu`'s fold is gated in isolation (`determinism.rs`) and has never run under the budget, so G1 is read through the budgeted entry point the campaign calls, not through `GpuBatch` directly. M-VACUOUS-SUCCESS: G1 prints its comparison COUNT (instances × prefixes, and how many prefixes had a nonzero fold) and is refused if that count is below `Σ (N + 1)` over the grid.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier and the sector the plant acts on is nonzero in that carrier by construction — PG-1 corrupts a branch whose amplitude at `y` is nonzero (a zero branch's rotated base is never summed), PG-2 acts on a prefix with a nonzero fold, PG-3 flips the earliest branch in the declared order that is live at `y`.
