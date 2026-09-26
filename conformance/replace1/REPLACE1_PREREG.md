# REPLACE-1 — the fluid-element tier on the staggered chart, with a removability gate: PREREGISTRATION

*Frozen 2026-09-26, committed alone before any code. Two weeks of measurement said what the
fluid-element tier should be, and this freeze puts it into the engine as code with tests: (i)
the tier's chart is density in cells and momentum on the faces — the chart the conservation
law chose over the closure score (`RESPONSE1_AMENDMENT_5.md`; R1″ `D = 0.180 ± 0.058` on the
graded 36-cycle average against the cell chart's `0.62`); (ii) the tier carries ONLY what the
level above cannot remove within its budget — a REMOVABILITY GATE, the operational form of
"a tier is a closed view the tier above carries" (`OBJECT.md` CORRECTION 2026-09-25), which
already exists for circuits as the light-cone removal in `holon::sector` (`QVM_ACUITY1_RESULTS`,
S1: removed gates change the observable by `≤ 10⁻¹²`) and is here generalised to any tier. The
structural and network columns are the first things it must DROP with a measured price
(`ORDER1_RESULTS.md` §6: increments `≤ 0`; `TSCAN_HBOND_SEARCH_RESULTS.md`: increment exactly
zero). Nothing here is new physics — the staggered grid is the MAC grid of 1965 — and the
freeze says so; what is new to this engine is a coarse tier that declares what it does not
carry and refuses when a drop's price exceeds its budget.*

## 1. The code

- **`holon-closure::removable`** (new module): given a trajectory's per-step readings, a
  DICTIONARY of candidate columns, a TARGET (the level above's next state — the columns it
  predicts), a lag, and a budget `β`: for each candidate column (or declared block), the
  carried increment = R² of the target from {kept set + candidate} minus R² from {kept set},
  held out (train/test split declared), with a RE-PAIRED NULL (the candidate from a
  time-shifted or partner-swapped source) and a jackknife SE; the column is CARRIED if the
  increment exceeds `β` and the null does not, DROPPED otherwise, with the pair (increment,
  null) as its price certificate. Output: `Admission { carried, dropped, prices, se }`. The
  statistic is `slow1_search.py`'s S2 increment and `hbond_search.py`'s S3 increment, now one
  engine function, with the plants below re-derived for it.
- **The staggered chart** in the fluid element's tier code (`holon-lens`'s chart machinery /
  `rung2`'s integral form): density per cell, momentum on the cell faces at `h = 0.25 Å`
  from the face centre (the value Amendment 5 fixed and did not tune), the integral-form
  continuity leg reading occupancy change from face momenta; the cell chart kept beside it
  as the reference, both behind one interface.
- **The tier's admission**: the fluid-element tier's carried set is what `removable` returns
  on the RESPONSE-1 and SLOW-1 walks against the target `(ρ, j)` at the next window; the
  structural block (q, s2, density counts, bond count) and the network block are
  candidates; the certificate is banked with the tier.

## 2. Stakes

- **G1 — the chart.** The staggered chart's continuity defect on the three 200 m/s longitudinal
  arms, 36-cycle aligned average, `D ≤ 0.2` with `α ∈ [0.8, 1.25]` — reproducing R1″'s banked
  `0.180 ± 0.058` from the engine's own code path, not the Python reader, to `±0.01`. **Kill:**
  the engine reads a different number than the banked one by more than `0.01` (the port is
  wrong), or `D > 0.3`.
- **G2 — the drops.** On the SLOW-1 293 K walks, target `(ρ_k, j_k)` at 1 ps and 5 ps, budget
  `β = 0.02`: the structural block and the network block are DROPPED with increments `≤ 0.01`
  and nulls `≤ 0.01`, reproducing ORDER-1's and HBOND's numbers to `±0.005`. **Kill:** either
  block is admitted, or the price disagrees with the banked read by more than `0.005`.
- **G3 — the keeps.** The hydrodynamic block itself, tested as a candidate against a target of
  its own next state with the others kept, is CARRIED (the gate must not drop what closes and
  carries); and on the reasoning chains (`reason_search0b`'s 444 transitions, target = the next
  thought's process sector), the conscience block is CARRIED with the banked `8.9 ×`-above-null
  reproduced to `10 %`. **Kill:** a carried block dropped.
- **G4 — the circuit case is the same function.** `holon::sector`'s light-cone removal expressed
  through `removable` (candidate = a T-gate, target = the observable, budget `10⁻¹²`) reproduces
  ACUITY-1's S1 (76 removed on the grid's marginals, worst change `2.6 × 10⁻¹⁵`; every kept gate
  moves the amplitude by `> 10⁻³`). **Kill:** any difference in the removed set.
- **G5 — refusal by price.** With the budget set BELOW a carried block's increment (e.g. `β = 0`
  on the hydrodynamic block), the tier REFUSES by name to drop it and reports the price;
  with the budget set above it, it drops it and the closure defect rises by the price. The
  refusal path is exercised, not assumed.

## 3. Plants

| plant | must |
|---|---|
| **PR-1** | a planted carried column (a synthetic field wired into the target with 5 ps memory at unit amplitude — ORDER-1's PO-4 as re-derived under its Amendment 1) is ADMITTED with increment `≥ 0.05` and null `≤ 0.01` |
| **PR-2** | a planted uncarried column (the same field, de-correlated from the target by a partner swap) is DROPPED with increment `≤ 0.01` |
| **PR-3** | a column that is a linear copy of a kept column is DROPPED with increment `0` to `10⁻¹⁰` (redundancy is a drop, not a keep) |
| **PR-4** | the time-shuffled dictionary admits nothing (`≤ 0.01` on every candidate) |
| **PR-5** | the staggered chart and the cell chart, both through the engine's interface, reproduce `r1_closure_test.py`'s per-arm tables on `response1_L200_seed0` to the printed digit |

## 4. Branches

- **(a)** G1–G5 met → the fluid-element tier runs on the staggered chart with an admission
  certificate; the tier's carried set is `(ρ, j)` with the structural and network columns
  dropped at a measured price; the QVM's removal is the same function; banked in the
  engine's gates, the stance and TIERS.md.
- **(b)** G1 killed by the port → the Python reader and the engine disagree; the disagreement
  is the finding and nothing is banked until it is resolved.
- **(c)** G2 or G3 killed → the removability statistic does not reproduce the banked reads;
  the statistic is wrong or the reads were; resolved before anything is banked.
- **(e)** a plant fails → nothing is read.

## 5. Cost

No new trajectory. Rust in `holon-closure`, `holon-lens` and `holon`; the reads run on walks on
disk in minutes; CI carries the plants.

## 6. What this does not test

New physics; the fine model (its seeds are running); the tier at any other `k` or `T`; the
game.

---
witness: `Closed` (Object.lean) and `StatClosure.lean` for the certificate's composition; the staggered chart's closure is measured (R1″); the removability statistic is measured against banked reads
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-JOINT-PASS-REGION, M-CHEAPER-THAN-ITS-PRICE, M-STALE-INSTRUMENT, M-MAINTENANCE-LENS, M-FLOOR-UNSTAKED, M-PLACEMENT-LOTTERY, M-VACUOUS-SUCCESS, M-VALIDATED-NOT-WIRED, M-COND-PROBE, M-HOMOG, M-PARITY-PROTECT, M-DEVICE-CLASS — contacted by keyword, cited. M-PLANT-OBS: every plant is re-derived for the engine's own function on the carrier of record (PR-1 at ORDER-1 Amendment 1's unit amplitude). M-JOINT-PASS-REGION: PR-1 passes G2's admission rule and G5's refusal rule on the same carrier before the freeze is read.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a synthetic field on `ρ_k`, a partner-swapped copy, a linear copy of a kept column, a time-shuffled dictionary, `response1_L200_seed0`), and the sector the plant acts on is nonzero in that carrier by construction.

---

### Notes on building

*Appended by the build (2026-09-26) after the code existed and the numbers came in. No stake is
moved here. These are the choices the frozen text left open, with the order they were made in,
and the places where the frozen text and the machine disagree. The read is
`replace1_read.txt`; the verdict is `REPLACE1_RESULTS.md`.*

**Where the code lives.** `holon-closure::removable` (the gate: `increment`, `admit`,
`admit_certified`, `refuse_drop`, `closure_r2`, the held-out VAMP score, a numpy-compatible
PCG64); `holon-lens::staggered` (the cell and staggered charts behind `FluidChart`, the
integral-form continuity leg, the 36-cycle pool, the leave-one-cycle-out jackknife, the
two-sided floor); `holon-lens::walk` (the `rigid.walk`/`atoms.walk` readers, SLOW-1's `q`,
counts and `s2`, ORDER-1's modes and chains, HBOND's counts, the PO-4 plant);
`holon::sector` (the light-cone removal now taken by `removable::admit_certified`). `holon`
takes `holon-closure` as a second runtime dependency (std, zero dependencies of its own);
`holon-closure` takes `holon-lens` and `holon` as DEV-dependencies only, for the driver
`examples/replace1` and `tests/replace1_plants.rs`.

**The statistic, spelled out.** The three banked readers differed only in pooling and split, so
the gate names both: `Ridge::ORDER1` (λ = 10⁻³ N on standardised inputs, target standardised by
its training SD, residual and total sums pooled over blocks and columns) with
`Folds::TimeBlocks { k: 5 }` (`linspace` edges, training pairs within the lag of the test block
dropped) for the fluid; `Ridge::HBOND` (raw target, mean of per-fold `R²`) with
`Folds::Units { k: 4 }` for the molecules. The jackknife SE is leave-one-held-out-block.

**The nulls.** Time-shifted: the candidate rolled by half its unit's length (ORDER-1's). Partner-
swapped: the next seed's chains (ORDER-1's PO-2) on the fluid; molecule `i + n/2` on HBOND's
carriers; on the chains, whose units are two to eight rows long and cannot be deranged, every
origin's candidate row re-paired by a PCG64 permutation (seed 0) — numpy's `default_rng`
ported (SeedSequence, PCG64, `permutation`) and checked against numpy's output, so the banked
re-paired nulls reproduce to the permutation rather than to a re-draw.

**Redundancy (PR-3).** A ridge fit with a copy of a kept column splits the weight between them
and reads an increment of the order of the penalty, not `0`. The gate therefore declares a
candidate column REDUNDANT when it is an affine function of the kept columns over every row it
reads (relative residual `≤ 10⁻⁹` after Gram–Schmidt) and does not fit it; a wholly redundant
candidate's increment is exactly `0`.

**The verdict rule.** CARRIED iff the increment exceeds `β` and neither null does; DROPPED iff
the increment is within `β`; a candidate whose increment AND a null both exceed `β` is
`Unresolved` — listed with the dropped (§1's "DROPPED otherwise") and refused by `refuse_drop`,
since its price exceeds the budget.

**Declared in code before the graded read** (the frozen text leaves each open):
- `β = 0.02` for every admission, the chains included (§2 states a budget only for G2).
- G2's structural block is the five coarse fields at the tier's `k` — `q`, `s2`, `n33`, `n50`,
  `nb` (ten columns per chain). ORDER-1's banked number is `Q` alone, so `Q`'s price is the one
  compared with the bank; the block's own price is new.
- G3 (fluid) is graded at 1 ps, G2's shorter lag, with 0.1 ps and 5 ps reported; "the others
  kept" is the declared structural block. The network block is not on these walks (below).
- G3 (chains): kept = the thought's process sector, target = the next thought's process sector,
  candidate = the DMA (conscience) block, chain folds `i mod 4`, ORDER-1's ridge convention.
- G5 is exercised on the 1 ps hydrodynamic admissions of G3.
- PR-1 is ORDER-1's PO-4 at unit amplitude (Amendment 1's rule): τ = 5 ps, latency 1 ps, noise
  `0.05`, plant seed 5, with this engine's Gaussian (Box–Muller on PCG64), not numpy's ziggurat —
  re-derived, not replayed (M-PLANT-OBS). PR-2's partner is the chain two over (another axis).
  PR-4 is one PCG64 permutation (seed 11) of the readouts applied to every chain of PR-1's
  carrier, the planted field and the structural block included.

**Added after a read, reported and not graded:** the G3 fluid diagnostics (the hydrodynamic
block's 1 ps increment with other kept sets), and G4's second column (every kept T tried at
`n = 20`, where ACUITY-1's test tried the first four).

**Where the frozen text and the machine disagree** (each is in the results as "what the prereg
got wrong"):
1. **G2's network block cannot be built on the SLOW-1 walks**: they carry oxygen positions and
   velocities only, no hydrogen, so no donor–acceptor geometry. HBOND's banked number was read on
   its own all-atom walks against its own target (`Δv` of the molecule over 50 fs, velocity
   kept). The network block is read there, on those carriers, against that target.
2. **G3 names no lag, and its "others kept" contains a second chart of the density**: `n33`,
   `n50` and `nb` coarse-grained at `k` carry the density mode. A per-candidate gate then drops
   EACH of two redundant blocks — the structural block given `(ρ, jL)` (G2) and `(ρ, jL)` given
   the structural block (G3 at 1 ps).
3. **G3's 8.9× is a closure number, not a carried one**: `reason_search0b`'s DMA row is the DMA
   block's own next state (VAMP held-out score against a re-paired null), not the next thought's
   process sector.
4. **G4's "every kept gate moves the amplitude by > 10⁻³"** is not what ACUITY-1 measured (the
   best kept T per instance, the first four at `n = 20`).
5. **§0's "the cell chart's 0.62"**: on the graded pool (8 cells, 36 cycles) the cell chart reads
   `0.310`; `0.616` is the 4-cell pool.
