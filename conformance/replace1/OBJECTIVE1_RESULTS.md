# OBJECTIVE-1 — READ: branch (e) by the letter (PI-3 fails: its price came in at `+0.0069`, under the staked window). Without PI-3 the read is (b), as the prereg predicted from a banked number. Q1 and Q4 are killed at the one cell §4 named, Q3 is as staked, and PI-1 and PI-2 pass. In substance, on every defined 293 K cell the carried set is `{H}` or empty. No other block is carried anywhere on the fluid. The budget decides whether the tier exists, not what it carries.

*2026-09-26. Prereg `OBJECTIVE1_PREREG.md` (frozen alone, `743f106`; "Notes on building"
appended by the build, no stake moved). Instrument: `holon-closure::removable::Admission::ordered`
as banked by REPLACE-1 (`979984f`), whose `Step` now also records every standing candidate's
price. Driver `engine/crates/holon-closure/examples/objective1/`. The read is
`objective1_read.txt`: 101 s on `taskset -c 18-20`. Every one of the 123 cells (64 fluid, 3
chains, 56 plant) agrees with its `β = +∞` path. `cargo test --release -p holon-closure` is
green (43 unit tests + 5 plants; 3 walk-sized tests ignored, as in REPLACE-1).*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **Q1** invariance, fluid 293 K | the survivor set is the same over every defined cell on each seed (`{H}`); kill: two defined cells on one seed differ | seed 0: `{H}` on all 8 defined cells. Seed 2: `{H}` on all 12. **Seed 1: `{H}` on 7, `{}` at `(1 ps, β 0.05)`**, where `H`'s price is `+0.0390 < 0.05`. This is the cell §4.2 named before the run | **KILL** (predicted) |
| **Q2** 400 K, reported | — | only 0.5 ps is defined (full `R² +0.0068`). There the count block is the last standing (`+0.0092 ± 0.0044`), `{C}` at `β 0.005` and `{}` above it. 1, 2 and 5 ps are UNDEFINED | reported |
| **Q3** the chains | the process sector survives at every `β`; DMA survives at `≤ 0.01` and is dropped at `0.02`; framing kill if DMA survives at 0.02 or is dropped at 0.005 | `β 0.005 {DMA, proc}`, `0.01 {DMA, proc}`, `0.02 {proc}`. DMA's price is `+0.0164 ± 0.0081` (null `−0.0013`), the same as REPLACE-1. The process sector's price is `+0.3902`, or `+0.5754` alone | **AS STAKED** (level-relative) |
| **Q4** seeds | survivor sets and removal order agree across the three seeds wherever all three are defined; an order divergence within `√(se₁² + se₂²)` on each diverging seed is a tie | survivor sets agree on 7 of 8 jointly-defined cells and **differ at `(1 ps, β 0.05)`**: `{H} / {} / {H}`. Orders: seed 1 removes `C` first and seeds 0 and 2 remove `S` first, at 0.5 ps and at 1 ps. Seed 0 vs seed 1 at 0.5 ps is a tie. Seed 1 vs seed 2 at 0.5 ps is **not** a tie (`\|d\| 0.0033` vs SE `0.0027` on seed 2), and neither pair is a tie at 1 ps | **KILL** |
| **PI-1** unit amplitude | `planted` survives at every `β ≤ 0.05` and is dropped at 0.2 (1 ps) | survives at `0.005 … 0.05`, dropped at `0.2`; `β* = +0.1837` | **PASS** |
| **PI-2** shuffled dictionary | nothing kept at any `β` and any lag | nothing kept on all 16 cells; largest step price `+0.0002` | **PASS** |
| **PI-3** amplitude `0.25` | survives at `{0.005, 0.01}`, dropped at `{0.02, 0.05}` (1 ps) | price **`+0.0069 ± 0.0076`**: survives at `0.005` only | **FAIL** |
| **BRANCH** | | | **(e) by the letter.** Without PI-3 it would be **(b)**: Q1 killed, Q3 as staked, and Q4 killed too |

## 1. The fluid grid

Survivor set per cell. `H` = `(ρ, j_L)`, `C` = `(n33, n50, nb)`, `S` = `(q, s2)`. **U** marks an
undefined cell (full-dictionary `R² ≤ 0`), which is not graded.

| seed | lag | `R²` full | β 0.005 | 0.01 | 0.02 | 0.05 |
|---|---|---|---|---|---|---|
| T293_seed0 | 0.5 ps | `+0.0922` | {H} | {H} | {H} | {H} |
| T293_seed0 | 1 ps | `+0.0488` | {H} | {H} | {H} | {H} |
| T293_seed0 | 2 ps | `−0.0030` | U {H} | U {H} | U {} | U {} |
| T293_seed0 | 5 ps | `−0.0267` | U {} | U {} | U {} | U {} |
| T293_seed1 | 0.5 ps | `+0.0807` | {H} | {H} | {H} | {H} |
| T293_seed1 | 1 ps | `+0.0252` | {H} | {H} | {H} | **{}** |
| T293_seed1 | 2 ps | `−0.0098` | U {} | U {} | U {} | U {} |
| T293_seed1 | 5 ps | `−0.0251` | U {} | U {} | U {} | U {} |
| T293_seed2 | 0.5 ps | `+0.1081` | {H} | {H} | {H} | {H} |
| T293_seed2 | 1 ps | `+0.0546` | {H} | {H} | {H} | {H} |
| T293_seed2 | 2 ps | `+0.0093` | {H} | {H} | {H} | {H} |
| T293_seed2 | 5 ps | `−0.0770` | U {C} | U {} | U {} | U {} |
| T400_seed0 | 0.5 ps | `+0.0068` | {C} | {} | {} | {} |
| T400_seed0 | 1 ps | `−0.0043` | U {} | U {} | U {} | U {} |
| T400_seed0 | 2 ps | `−0.0132` | U {} | U {} | U {} | U {} |
| T400_seed0 | 5 ps | `−0.0050` | U {} | U {} | U {} | U {} |

**The paths** (`β = +∞`). Each step's price is followed by its nulls (shifted / swapped). Every
step's standing prices are in the read.

| seed, lag | step 1 | step 2 | step 3 | `β*` (S, C, H) |
|---|---|---|---|---|
| s0 0.5 ps | S `−0.0035` (`−0.0003 / −0.0042`) | C `−0.0022` | H `+0.1078 ± 0.0101` (`+0.0005 / +0.0003`) | `−0.0035, −0.0022, +0.1078` |
| s0 1 ps | S `−0.0015` | C `−0.0010` | H `+0.0622 ± 0.0153` (`−0.0005 / −0.0036`) | `−0.0015, −0.0010, +0.0622` |
| s0 2 ps (U) | S `−0.0068` | C `−0.0054` | H `+0.0199 ± 0.0090` | `−0.0068, −0.0054, +0.0199` |
| s1 0.5 ps | C `−0.0025` | S `−0.0020` | H `+0.0942 ± 0.0138` (`−0.0081 / −0.0007`) | `−0.0020, −0.0025, +0.0942` |
| s1 1 ps | C `−0.0031` | S `−0.0017` | H `+0.0390 ± 0.0073` (`−0.0066 / −0.0048`) | `−0.0017, −0.0031, +0.0390` |
| s1 2 ps (U) | C `−0.0039` | S `−0.0014` | H `+0.0047 ± 0.0052` | `−0.0014, −0.0039, +0.0047` |
| s2 0.5 ps | S `−0.0016` | C `+0.0007` | H `+0.1547 ± 0.0256` (`−0.0128 / −0.0080`) | `−0.0016, +0.0007, +0.1547` |
| s2 1 ps | S `−0.0030` | C `+0.0033` | H `+0.1039 ± 0.0195` (`−0.0094 / −0.0104`) | `−0.0030, +0.0033, +0.1039` |
| s2 2 ps | S `−0.0025` | C `−0.0030` | H `+0.0734 ± 0.0141` (`−0.0155 / −0.0127`) | `−0.0025, −0.0025, +0.0734` |
| 400 K 0.5 ps | S `−0.0001` | H `+0.0012` | C `+0.0092 ± 0.0044` | `−0.0001, +0.0092, +0.0012` |

At 5 ps every path's prices are `≤ +0.0091`, all on undefined cells. At 1 ps, REPLACE-1's
Amendment 1 paths are reproduced to the fourth decimal: every step, price and null.

**How far the invariance extends.** On every defined 293 K cell, the largest `β*` of any
dropped block is `+0.0033` (C, seed 2, 1 ps), and the smallest `β*` of `H` is `+0.0390`
(seed 1, 1 ps). So for every budget in **`(0.0033, 0.039)`** the carried set is `{H}` on every
defined cell of every seed at 0.5 and 1 ps, and at 2 ps on seed 2. That is an order of
magnitude of budget and a factor of 2 to 4 in cadence. Outside that window the set does not
change to another block. It goes to EMPTY at `H`'s own price. No block other than `H` survives
on any defined 293 K cell at any swept budget. The prereg's content clause ("some block's price
lies INSIDE the sweep and the set still does not change") is met by no dropped block, since all
their prices sit below 0.005. The only price inside the sweep is `H`'s own on seed 1, and there
the set changes.

**Cadence.** The defined region is 0.5–1 ps on all three seeds, and 2 ps on seed 2 only. `H`'s
price falls with lag on every seed: `0.108 → 0.062` (s0), `0.094 → 0.039` (s1),
`0.155 → 0.104 → 0.073` (s2). The lag axis therefore tests invariance over a factor of 2, or 4
on seed 2. The target stops being predictable before the cadence can be varied further.

## 2. The chains

| β | survivors | removal order | survivors' removal prices |
|---|---|---|---|
| 0.005 | {DMA, proc} | — | proc `+0.3902 ± 0.0178` (null `−0.0139`), DMA `+0.0164 ± 0.0081` (null `−0.0013`) |
| 0.01 | {DMA, proc} | — | the same |
| 0.02 | {proc} | DMA at `+0.0164` | proc `+0.5754 ± 0.0269` (null `−0.0163`) |

Full-dictionary `R² +0.5622`. The composition changes inside the sweep, at DMA's price, with
the process sector still standing. This is the level-relative case the stake named in advance.
It is arithmetic on REPLACE-1's banked `+0.0164`, which the engine reproduces.

## 3. Plants (T293_seed0; dictionary `{H, C, S, P}` on the planted walk)

| plant | 1 ps path | survives at β 0.005 / 0.01 / 0.02 / 0.05 / 0.2 | must | |
|---|---|---|---|---|
| **PI-1** `a = 1` | S `−0.0002`, C `+0.0009`, H `+0.0410`, P `+0.1837 ± 0.0321` (P over H: `+0.1041`, PR-1's banked price exactly) | ✓ ✓ ✓ ✓ ✗ | ✓ ✓ ✓ ✓ ✗ | PASS |
| **PI-2** shuffled dictionary | every path's largest step price `+0.0002`; every cell UNDEFINED (`R² −0.007 … −0.012`) | nothing, 16 of 16 cells | nothing | PASS |
| **PI-3** `a = 0.25` | S `−0.0012`, C `−0.0010`, **P `+0.0069 ± 0.0076`**, H `+0.0657` | ✓ ✗ ✗ ✗ ✗ | ✓ ✓ ✗ ✗ ✗ | **FAIL** |

The PI-1 carrier at other lags shows that the sweep can see a change when there is one. At
2 ps, `{H, P}` becomes `{P}` at `β 0.02` (H `+0.0159`). At 1 ps, `{H, P}` becomes `{P}` at
`β 0.05` (H `+0.0410`). PI-3 does change inside the sweep (it survives at 0.005 and is dropped at
0.01), but at a price its own SE does not separate from zero. At 0.5 ps and 2 ps it is removed
at every budget (`+0.0018`, `+0.0015`).

## 4. What the prereg got wrong

1. **PI-3's amplitude.** The freeze replaced the lead's "half amplitude" with `a = 0.25` from two
   price models anchored on PR-1's `+0.1041`, which predicted `+0.012` and `+0.018`. The read is
   `+0.0069`. The price falls faster than either model; why is not read here.
   **And the must was not stakeable at this carrier's resolution.** A window `(0.01, 0.02]`
   graded with a five-block jackknife SE of `0.008` is a coin toss even at the right amplitude.
   A plant meant to change inside the sweep needs a price several SEs from both neighbouring
   budgets. On this carrier that means a price near `0.03` with `β` points at 0.01 and 0.05.
   Whether the lead's half amplitude would have landed there was not read, and is not read here.
2. **§4 missed half of what it knew.** It predicted Q1's kill at `(seed 1, 1 ps, β 0.05)` but
   not that the same banked `+0.0390` kills Q4's survivor-set clause at that cell.
3. **The definedness rule reads the wrong `R²`.** The full dictionary's held-out `R²` sits
   BELOW the survivor's own price wherever the dropped blocks only add noise to the ridge. At
   seed 0, 2 ps, the full `R²` is `−0.0030` while `H` alone is priced at `+0.0199` over
   nothing, so the cell was excused. Read by the best `R²` along the path, it would be defined
   and would add a second Q1 kill (`{H}` → `{}` at `β 0.02`). Under the rule as staked it is
   not graded. The rule is the lead's, and the finding is stated here without regrading.
4. **Q4's order clause grades noise-level orderings.** Every divergence is between `C` and `S`
   at step 1, at prices within `±0.0045` of zero and far below every budget. Their
   differences (`0.0033`, `0.0034`, `0.0075`) exceed the combined jackknife SEs (`0.0027`,
   `0.0028`), so by the rule they are not ties. The rule is right to call them. But the order
   of two blocks that are both removed at every budget has no consequence for the carried set.
5. **The lag axis is shorter than it looked.** The target is predictable (full `R² > 0`) only to
   1 ps on seeds 0 and 1, and to 2 ps on seed 2. REPLACE-1 said 5 ps was vacuous. It did not
   say 2 ps would be too.
6. **PI-2 is read entirely on undefined cells.** A shuffled dictionary cannot predict the target,
   so the definedness rule would have excused every one of its cells. The plant was declared
   to be read regardless, and it passes regardless. It is a null of the whole procedure, not a
   test of the invariance claim.

## 5. The read, for the lead

- **Fluid (293 K).** Ordered admission's carried set on the fluid element is `{H}` on every
  defined cell for every budget in `(0.0033, 0.039)`, on all three seeds, over 0.5–1 ps. The
  set never switches to a different block. It only goes empty at `H`'s own price (`0.039` to
  `0.155`, falling with lag). The budget decides whether the tier is carried at all, not what
  it carries. That is weaker than Q1's letter (one set over the whole sweep) and stronger than
  "budget-relative" in the chains' sense.
- **Chains.** The composition changes at `0.0164` with the process sector still standing. This
  is level-relative in the full sense, as staked.
- **400 K.** At the one defined cell the last block standing is `C`, not `H`, at a price
  (`+0.0092 ± 0.0044`) inside the sweep. On this dictionary the hot fluid has no carried
  hydrodynamic block. It is reported, not staked.
- **What is owed.** A re-run of PI-3 at an amplitude set so its price sits several SEs inside a
  sweep interval. Also the definedness rule read against the path's best `R²` as well as the
  full dictionary's. Both are amendments for the lead to freeze alone before any re-read. This
  build moves neither.

---
witness: `Closed` (Object.lean) and `StatClosure.lean`, cited in words; the survivor sets are measured
**misfits:** M-VACUOUS-SUCCESS (the undefined cells; PI-2 read entirely on them), M-BAR-FROM-THE-READ (§4 of the prereg: Q1's and Q3's cells decided by banked numbers, stated before the run), M-PLANT-OBS (PI-3's amplitude re-derived and wrong), M-PLANT-SECTOR, M-JOINT-PASS-REGION (PI-1 exhibits both outcomes; PI-3 does not resolve its window) — contacted, cited.

## The lead's reading (2026-09-26): the metaphysical question answered by a window, not a yes or no

By the letter, (e): PI-3's price (`+0.0069 ± 0.0076`) sat inside its own error bar, so the plant
that was to exhibit a change inside the sweep could not; that is `M-BAR-FROM-THE-READ`'s
sibling (a plant priced by a model rather than on the carrier, M-PLANT-OBS's sixth occurrence)
and is recorded, not re-run. Beside the letter, the grid says one thing on every defined cell,
three seeds, every lag at which the target is predictable: **the fluid element's carried set is
the hydrodynamic block or nothing. No budget and no cadence ever makes it something else.** The
budget decides WHETHER the tier is carried — the set empties exactly at the hydrodynamic
block's own price, `+0.039` on the weakest seed — never WHAT it carries. The invariance holds
for every budget between the dropped blocks' prices (`≤ 0.0033`) and the carried block's
(`≥ 0.039`): a window of one decade, on this box, at this wavelength, and the two edges of the
window are physical numbers (what structure costs to drop; what hydrodynamics costs to drop),
not choices. On the chains the conscience block's price (`+0.016`) lies inside the sweep and the
carried set changes with the budget as staked: level-relative, as predicted.

So the position the record supports is sharpened, not settled: **a tier's content is
objective inside a window whose edges are the prices of its carried and dropped sectors, and
observer-relative outside it.** Where the window is wide (the fluid: a decade), the level is
a fact about the liquid; where it is narrow or empty (the chains: the conscience block's price
is one SE from its null), the observer's budget is part of what the tier is. That is a
measurable notion of how objective a level is — the width of its window — and it is the
first number on this record that bears on the metaphysics rather than on the method.
