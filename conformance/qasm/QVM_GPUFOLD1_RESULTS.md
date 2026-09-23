# QVM-GPUFOLD-1 — READ: the budgeted prefix folds on the device as the same struct at every prefix, carries its class, and runs at 0.022–0.025× the S = 8 mesh at N = 8,748 with the round trip in — branch (a), with the cold path and the parity regime named

*2026-09-23. Prereg `conformance/qasm/QVM_GPUFOLD1_PREREG.md` (frozen alone, `78e8c57`; nothing
in it moved). Instruments: `holon::acuity` (`FoldBackend`, `DeviceFold`, `AffineBranches`,
`budgeted_amplitude_on`, `fold_prefix_on`; `Budgeted.device`, `require_class`, `single_class`),
`holon_gpu::acuity` (`GpuBranchFold`, `CpuTwinFold`), `GpuBatch::{fold_timed, read_base,
expected_base, plant_base_limb}`; tests `engine/crates/holon-gpu/tests/qvm_gpufold_plants.rs`
(5 green, G3 `#[ignore]`d and run by hand four times). Device: NVIDIA GeForce RTX 4090 Laptop
GPU, driver 580.142 / CUDA 13.0, the fold PTX from nvcc 12.0 JIT-compiled at load — CUDA WAS
available. Host arm: `taskset -c 21-27` (seven E-cores), box loadavg 9–28 from other campaigns
on cores 0–23.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **G1** same `Cyc` struct at every prefix and every ε | GPU fold of `order[..k]` `==` the S = 8 mesh for all `k ∈ [0, N]` on `n ∈ {12,16,20,24}` × `t ∈ {16,20,24,28}`; the budgeted `(value, R_k, k, N, order)` equal at 9 ε rungs; no canonicalisation | **17,296 prefix comparisons (= Σ(N+1), the work count asserted), 11,519 of them with a nonzero fold, 0 struct mismatches**; 144 ε rows equal in every field, `R_k` bit-for-bit | **MET** |
| **G2** the class travels with the artifact | CPU run refused as Gpu; mixed table refused; no silent fallback | CPU mesh, the legacy `budgeted_amplitude_with`, and the CPU twin through the SAME `Device` door all carry `cpu` and are refused as `gpu` while holding the identical struct; a mixed table is refused; device ordinal 7 errors at the door; a 68-wire register is refused at construction | **MET** |
| **G3** round-trip wall ≤ 0.25× the S = 8 mesh at N = 8,748, min and median; kill > 0.75× | four runs, table §1 | median **0.0068–0.0247**, min **0.0075–0.0278** at N = 8,748 | **MET** (by ≥ 9× on the worst reading) |

Plants: **PG-1** fires (a limb XOR'd on the card after upload — low limb of lane 0, then high
limb of lane 2 — is convicted by the CPU twin by prefix position 102, branch 156, lane and
word; the fold moves; a fresh upload is acquitted). **PG-2** fires (4 prefixes × 4 upload
orders — as planned, reversed, rotated, shuffled — × 5 launch shapes = 80 folds, one struct
per prefix, equal to the mesh; two of the four prefixes are budget-truncated, `k = 213` of
324 and `k = 1,458` of 2,916). **PG-3** fires exactly as ACUITY-1's PQ-4 did: the sign-flipped
branch moves the device sum by `2|term| = 3.125·10⁻²` against `R_k = 0`, and at `t = 12,
ε = 10⁻²` by `2.34·10⁻²` against a certified `R_k = 7.81·10⁻³` — the same numbers PQ-4 read on
the host, and the planted source folds to the same struct on both backends.

**Branch (a).** The budgeted sum has a device backend that is the same artifact up to its
declared class, and pays.

## 1. G3 — the table

`n = 24`, `ε = 0` (`k = N`), 25 repetitions per arm, milliseconds, **min / median** (max/min
spread in the notes). "GPU warm" is the budgeted call on the device backend with the batch
resident: host stop decision, launch, kernel, device→host copy of the block partials, host lane
sum — the ROUND TRIP IS IN IT. "Kernel" is the CUDA-event time of the launch alone. Four runs
with the committed code (r1–r3 before the packed-twin column was added, r4 after; r4 is the
only one with it).

| N | run | CPU mesh S=8 | GPU warm (round trip in) | GPU kernel | round trip = warm − kernel (med) | **ratio min** | **ratio med** | loadavg |
|---|---|---|---|---|---|---|---|---|
| 972 | r1 | 4.43 / 7.00 | 0.318 / 0.330 | 0.322 | 0.008 | 0.072 | 0.047 | 9.5 |
| 972 | r2 | 2.45 / 3.59 | 0.307 / 0.332 | 0.321 | 0.011 | 0.125 | 0.092 | 9.0 |
| 972 | r3 | 5.70 / 8.93 | 0.313 / 0.332 | 0.320 | 0.011 | 0.055 | 0.037 | 12.4 |
| 972 | r4 | 2.65 / 2.80 | 0.307 / 0.335 | 0.322 | 0.012 | 0.116 | 0.120 | 28.2 |
| 2,916 | r1 | 6.28 / 6.99 | 0.423 / 0.440 | 0.424 | 0.017 | 0.067 | 0.063 | 9.1 |
| 2,916 | r2 | 8.52 / 12.49 | 0.423 / 0.442 | 0.425 | 0.017 | 0.050 | 0.035 | 9.2 |
| 2,916 | r3 | 20.85 / 26.00 | 0.426 / 0.445 | 0.429 | 0.016 | 0.020 | 0.017 | 15.5 |
| 2,916 | r4 | 9.00 / 10.68 | 0.424 / 0.440 | 0.425 | 0.016 | 0.047 | 0.041 | 26.6 |
| **8,748** | r1 | 20.97 / 24.74 | 0.584 / 0.601 | 0.582 | 0.019 | **0.028** | **0.024** | 8.8 |
| **8,748** | r2 | 24.49 / 27.07 | 0.583 / 0.601 | 0.586 | 0.015 | **0.024** | **0.022** | 9.3 |
| **8,748** | r3 | 80.04 / 91.59 | 0.599 / 0.625 | 0.591 | 0.034 | **0.0075** | **0.0068** | 20 |
| **8,748** | r4 | 23.21 / 24.33 | 0.582 / 0.600 | 0.583 | 0.017 | **0.025** | **0.025** | 24.4 |

Beside the stake, reported and not staked:

- **Cold, as the harness splits it** (evict, then the budgeted call pays gather + host→device
  upload + the query; the descriptors already decoded): median `5.3–8.4 ms` at N = 8,748 →
  `0.09–0.30×` the mesh.
- **Cold, as the PREREG defined it** (descriptor decode + upload + query — a fresh backend for
  ONE amplitude): median `68–108 ms` at N = 8,748 → **`1.17–3.95×` the mesh. It EXCEEDS
  `1.0×`, which the prereg said would be named: it is named here.** The whole excess is the
  decode — `holon::Affine` cloned out of the source and serialised through `canon_key`, then
  decoded — at `≈ 7.5–12 µs` per branch, `65–105 ms` for 8,748. It is y-independent and paid
  once per circuit, like the plan, so the device wins from the SECOND amplitude a circuit is
  asked for; for a single amplitude the host mesh is the cheaper route at these sizes.
- **The tightest honest CPU arm** (`cpu.rs`: the SAME packed descriptors folded on the host at
  S = 8, r4 only): `9.9 ms` median at N = 8,748 against the mesh's `24.3` — the descriptor
  packing alone buys `2.5×` — and the GPU warm wall is `0.061×` of THAT, `0.10×` at 2,916 and
  `0.26×` at 972. So at the stake's size the device beats even the arm that is not staked, by
  16×; at N = 972 it would not meet a 0.25 stake against it.

## 2. What the prereg got wrong, once the code existed

1. **"Cold" lumped two costs of different kinds.** §2 G3 defined the cold query as decode +
   upload + query. The decode is per CIRCUIT (it depends on neither `y` nor `ε`), the upload is
   per PREFIX, the query per AMPLITUDE; they amortise over different things, and lumping them
   hid that the decode alone costs more than the whole host fold (§1). The harness reports all
   three; the prereg's lumped number is reported too, above `1.0×`, as it said it would be.
2. **G1 never contacted the regime where struct equality is only observed.** The prereg made
   the upload's `parity_uniform` flag a reported quantity because `gpu.rs` guarantees struct
   equality only when every branch shares one exponent parity. On all 16 instances the flag
   came back `true` (and `exponent_uniform` `false`): Magic5's branch weights here share a
   parity. So G1 is MET in the regime where it is guaranteed by the arithmetic, and says
   nothing about mixed-parity sources, where `tests/determinism.rs` has already measured
   struct equality failing on 11 of 18 probes with values equal. A source that mixes parity
   under this backend is not covered by this read.
3. **G1's every-`k` CPU side is the mesh over the source's CACHED per-branch amplitudes**, not
   over the source: `N²/2` affine amplitudes at `t = 28` is what the cache avoids. The mesh
   folds the same values in the same order by the same law either way, and the 144 ε rows are
   folded over the source itself and agree; but the prereg said "over the same prefix" and
   the harness says exactly how.
4. **PG-2's carrier needed choosing, and it is a finding about the order, not the plant.** The
   first truncated rung tried (`ε = R_{N/2}`) folds to EXACTLY ZERO at `n = 12, t = 20`, and so
   does every budgeted prefix up to `N/2` there: the certified order front-loads the largest
   `|coeff·γ|`, which are the smallest-support branches, which are dead at most `y`. Across
   G1's grid only `11,519 / 17,296 = 67%` of prefixes fold to nonzero (`608 / 2,917` at
   `n = 12, t = 28`). The order is greedy-optimal for the BOUND (ACUITY-1's argument stands);
   it is not an order in which the value arrives early. PG-2 now cuts at the first `R_j`,
   `j ≥ N/2`, whose prefix is nonzero (`k = 213`, `1,458`).
5. **The CPU arm is contended, and the ratio is not a constant.** M-PLACEMENT-LOTTERY in its
   own words: `taskset` restricts and does not reserve. The mesh's median at N = 8,748 ran
   `24–92 ms` across four runs as loadavg ran 9–24; the GPU warm median ran `0.600–0.625 ms`.
   Every contended reading FLATTERS the device, so the numbers to quote are the least
   contended: r1/r2/r4, ratio `0.022–0.025` median. The verdict is safe from this — the worst
   reading clears the stake by 9× — but `0.007` (r3) is contention, not the device.
6. **The round trip is not where the wall is.** The prereg carried SATURATION-3's `1.97×`
   round-trip spread as the risk. Here the round trip is `8–34 µs` (`2–6%` of the warm wall),
   the kernel is `97%` of it, and the warm max/min spread was `1.10–1.25×` on three runs of
   four; r3's `10.7×` max/min is single outliers (`3.3–3.5 ms`) with the median unmoved. The
   kernel time grows `0.32 → 0.59 ms` for a `9×` growth in N — the card is far from saturated
   at 8,748 branches, one thread's Gauss–Jordan on a 56-wire register each.
7. **`holon` now has a runtime dependency.** G2 needs the budgeted result to carry the device
   class, and D0 says the class has one definition (`holon-device`). `holon`'s manifest said
   "Zero runtime deps"; it now says one, `holon-device` — `no_std`, zero-dep, featureless, so
   the feature-resolution argument in holon-render's and holon-tables' manifests still holds.
   `engine/Cargo.lock` also picked up a `holon-md` edge for a sibling crate whose manifest
   already named it (pre-existing drift, synced by cargo, not introduced here).

## 3. Owed

The marginal on one resident batch (`2^{|L|−|S|}` amplitudes, where the per-circuit decode is
amortised by construction); a cheaper decode (the affine state straight into the packed
descriptor, skipping `canon_key`, which is where the prereg-cold excess lives); a mixed-parity
source under this backend (item 2); the `t ≥ 36` sizes where the host fold is seconds; G3 on a
quiet box.

---
**misfits:** M-DEVICE-CLASS (G2 is its remedy, wired), M-PARITY-PROTECT (item 2's parity regime), M-PLACEMENT-LOTTERY (item 5), M-VACUOUS-SUCCESS (G1's count asserted), M-PLANT-SECTOR (PG-2's carrier, item 4), M-VALIDATED-NOT-WIRED (G1 read through the budgeted entry point), M-CHEAPER-THAN-ITS-PRICE, M-IDLE-CALIBRATED-TIMEOUT (loadavg printed on every row).
