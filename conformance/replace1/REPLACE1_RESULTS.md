# REPLACE-1 — READ: branch (c) by the letter. The engine reproduces every banked number it was asked for (R1″ `0.180`, ORDER-1's eighteen, HBOND's three, the conscience sector's `8.90`, ACUITY-1's 76 removals), and G3 is killed on both halves. The cause is in the stake as written, not in the statistic.

*2026-09-26. Prereg `REPLACE1_PREREG.md` (frozen alone, `1855204`; "Notes on building"
appended by the build, no stake moved). Instruments: `holon-closure::removable` (the gate),
`holon-lens::staggered` (the charts), `holon-lens::walk` (the readers and dictionaries),
`holon::sector` (the light-cone removal, now a call to the gate). Driver
`engine/crates/holon-closure/examples/replace1/`; the read is `replace1_read.txt`, 142 s on
`taskset -c 18-20`. The first full read and a re-read after the final lint fixes are
byte-identical apart from timings. Tests: `engine/crates/holon-closure/tests/replace1_plants.rs`
and the new unit tests in `removable`, `staggered` and `sector`, all green with the three
crates' suites.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **G1** the chart | staggered chart, 36-cycle pooled `D ≤ 0.2`, `α ∈ [0.8, 1.25]`, engine = banked `0.180 ± 0.01` | **`D = 0.1800`** (banked `0.180`, difference `< 10⁻⁴`), jackknife SE `0.058` (banked `0.058`), `α = 1.03`, `R² = 0.97`. Pooled tables 33 of 33 lines identical to `r1pp_pooled_L200.txt` | **MET** |
| **G2** the drops | structural and network blocks DROPPED at `β = 0.02`, increments and nulls `≤ 0.01`, ORDER-1 and HBOND reproduced to `0.005` | `Q` (ORDER-1's column): all 18 banked numbers (3 seeds × 2 lags × increment, shifted null, swapped null) reproduced to the printed fourth decimal. Structural block `−0.0123 … +0.0003`. Network block on HBOND's carriers `−0.0001 / −0.0008 / +0.0000` (banked `−0.000 / −0.001 / +0.000`). All DROPPED. Max increment `+0.0003`, max null `+0.0049` | **MET** |
| **G3a** the keeps, fluid | hydrodynamic block CARRIED against its own next state, the others kept | at 1 ps (the lag declared in code before the read) **`+0.0087 / +0.0094 / +0.0091`** against `β = 0.02`: **DROPPED** on all three seeds. At 0.1 ps it is carried (`+0.20 … +0.23`). With `Q` or `Q + s2` kept instead of the full block it is carried at 1 ps (`+0.04 … +0.10`) | **KILL** |
| **G3b** the keeps, chains | conscience block CARRIED toward the next process sector; banked `8.9×` to 10 % | `8.90×` reproduced exactly (`1.576 / 0.177`; the whole `reason_search0b` table matches). The gate on the stated target: increment **`+0.0164 ± 0.0081`**, under `β` → **DROPPED** | **KILL** |
| **G4** the circuit case | `sector`'s removal through `removable` at `β = 10⁻¹²`: removed set identical (kill on any difference); 76 removed, worst `2.6 × 10⁻¹⁵` | 27 instances, **76 removed**, **0 differences** (against the old loop kept verbatim, and against the gate's own dropped list), worst change **`2.55 × 10⁻¹⁵`** | **MET** |
| **G5** refusal by price | `β` below the block's increment: REFUSED by name with the price; above it: dropped, and the defect rises by the price | `REFUSED: dropping 'hydrodynamic block' costs the level above +0.0087 against a budget of +0.0000` on 3 of 3 seeds. At `β = 0.05` it is dropped, and the independently read closure defect rises by exactly the price (`|rise − price| = 0`) | **MET** |
| **PR-1 … PR-5** | | all fire (§2) | **PASS** |
| **BRANCH** | | | **(c) by the letter**: G3 killed. The branch's stated cause ("the statistic does not reproduce the banked reads") does **not** hold (§3) |

`TIERS.md` and `STANCE.md` are not edited. The prereg writes them only under branch (a).

## 1. The numbers against the banked reads

**G1 and PR-5: the charts.** `staggered::closure_read` runs `r1_closure_test.py` line for line
behind one interface (`FluidChart::Cell { lag }`, `FluidChart::Staggered { h_angstrom }`). Seed
0's per-arm table matches the banked `r1_closure_test.txt` on **33 of 33 lines, to the printed
digit**. The 36-cycle pool matches `r1pp_pooled_L200.txt` on 33 of 33 lines. R1″ from the engine:
`D_lead 0.180`, SE `0.058`, `D_tail 0.689`, `α 1.03`, `R² 0.97`, `|D_lead − D_tail| 0.509`
(SE `0.170`). All equal the bank. On the same pool the cell chart reads `0.310`. The two-sided
floor is reported and not graded. On the pooled 8-cell read the staggered chart sits `0.205`
below its floor of `0.385` and the cell chart `0.032` below its floor of `0.342`. At 16 cells the
cell chart is `0.280` above its floor.

**G2: ORDER-1 through the gate** (target `(ρ_k, j^L_k)`, kept `(ρ, j^L)`, 5 time blocks):

| 293 K seed | lag | `Q` increment (bank) | shifted null (bank) | swapped null (bank) | structural block | verdict |
|---|---|---|---|---|---|---|
| 0 | 1 ps | `−0.0032` (`−0.0032`) | `−0.0008` (`−0.0008`) | `−0.0001` (`−0.0001`) | `−0.0025` | DROPPED |
| 0 | 5 ps | `−0.0126` (`−0.0126`) | `−0.0148` (`−0.0148`) | `−0.0066` (`−0.0066`) | `−0.0123` | DROPPED |
| 1 | 1 ps | `−0.0007` (`−0.0007`) | `+0.0001` (`+0.0001`) | `−0.0035` (`−0.0035`) | `−0.0048` | DROPPED |
| 1 | 5 ps | `−0.0049` (`−0.0049`) | `−0.0022` (`−0.0022`) | `+0.0049` (`+0.0049`) | `−0.0104` | DROPPED |
| 2 | 1 ps | `−0.0033` (`−0.0033`) | `−0.0047` (`−0.0047`) | `−0.0026` (`−0.0026`) | `+0.0003` | DROPPED |
| 2 | 5 ps | `−0.0089` (`−0.0089`) | `−0.0144` (`−0.0144`) | `−0.0008` (`−0.0008`) | `−0.0076` | DROPPED |

The kept set's own `R²` (`0.0513 / 0.0299 / 0.0543` at 1 ps) also matches ORDER-1's "H alone".
The 400 K arm, reported, gives `−0.0004` and `+0.0001` (bank: the same). Each structural field
alone (`s2`, `n33`, `n50`, `nb`) is dropped, with increments between `−0.005` and `+0.0034`.

**G2: the network block**, read on HBOND-SEARCH-1's own all-atom carriers against HBOND's target
(`Δv_com` over 50 fs, velocity kept, molecule folds):

| carrier | `R²` velocity alone (bank) | increment (bank) | nulls shifted / swapped |
|---|---|---|---|
| `molsearch1` (299 K) | `0.3992` (`0.399`) | `−0.0001` (`−0.000`) | `−0.0006 / −0.0011` |
| `molsearch1_warm_396K` | `0.3510` (`0.351`) | `−0.0008` (`−0.001`) | `−0.0001 / −0.0002` |
| `molsearch2_5ps` | `0.3988` (`0.399`) | `+0.0000` (`+0.000`) | `−0.0002 / −0.0001` |

**G3a: the fluid.** The hydrodynamic block `(ρ, j^L)` is the candidate. The target is its own
next state. The declared structural block is kept.

| 293 K seed | 0.1 ps (reported) | **1 ps (graded)** | 5 ps (reported) |
|---|---|---|---|
| 0 | `+0.2195`, CARRIED | **`+0.0087 ± 0.0051`, DROPPED** | `−0.0063` |
| 1 | `+0.2336`, CARRIED | **`+0.0094 ± 0.0039`, DROPPED** | `−0.0022` |
| 2 | `+0.2012`, CARRIED | **`+0.0091 ± 0.0016`, DROPPED** | `+0.0010` |

Diagnostics at 1 ps, added after the graded read and not graded. The block's increment with
other kept sets:

| kept | seed 0 | seed 1 | seed 2 |
|---|---|---|---|
| nothing | `+0.0622` | `+0.0390` | `+0.1039` |
| `Q` | `+0.0617` | `+0.0426` | `+0.0996` |
| `Q + s2` | `+0.0588` | `+0.0434` | `+0.1014` |
| `n33 + n50 + nb` | `+0.0130` | `+0.0070` | `+0.0111` |

The coarse density COUNTS alone predict `(ρ, j^L)` one picosecond ahead with `R² 0.02–0.05`, about
what `(ρ, j^L)` itself does. They are a second chart of the density mode. With them kept,
`(ρ, j^L)` is removable within `β`. With `q` and `s2` kept, it is carried.

**G3b: the chains.** 207 chains and 444 transitions, as banked. The held-out table of
`reason_search0b` is reproduced exactly: depth `0.998 / 0.010`, DMA `1.576 / 0.177 = 8.90`,
process `1.757 / 0.096`, all `3.260 / 0.530`. The null is a numpy-identical permutation, so this
is the bank itself and not a re-draw. The gate on the prereg's stated target (next thought's
process sector, this thought's process sector kept) gives the conscience block an increment of
`+0.0164 ± 0.0081` (`R² 0.546 → 0.562`), with the permutation null at `−0.0013` and the
time-shifted null at `+0.0160`. It is DROPPED at `β = 0.02`. The time-shifted null is not a null
on these units: rolling a two-row chain by one row gives each origin the target's own row, and
107 of the 207 chains have two rows.

**G4.** Per instance, the removed sets are 1/2/1/3/4/2/4/3/1 at `n = 12`, 3/1/3/3/0/6/3/3/7 at
`n = 16` and 2/1/2/4/2/3/6/1/5 at `n = 20`. The total is 76, the same as ACUITY-1's. Each removed
gate is dropped on its own and re-refereed; the worst change is `2.55 × 10⁻¹⁵`, and dropping
every removed gate at once stays within `10⁻¹²` too. For the kept-gate clause, the best kept T
moves the amplitude by more than `10⁻³` on **25 of 27** instances under ACUITY-1's own method (the
first four kept T at `n = 20`: `8.4 × 10⁻⁴` at `n 20 t 12 s 2`, `2.7 × 10⁻¹⁸` at `n 20 t 16 s 3`).
Trying every kept T, it does so on **27 of 27**. Individually, **150 of 248** kept T gates move
it by more than `10⁻³`.

## 2. Plants (carrier of record `slow1/T293_seed0` unless named)

| plant | must | read | |
|---|---|---|---|
| **PR-1** PO-4 at unit amplitude (5 ps memory, 1 ps latency; re-derived, this engine's Gaussian) | ADMITTED, increment `≥ 0.05`, null `≤ 0.01` | 1 ps: **`+0.1041 ± 0.0127`**, nulls `+0.0035 / −0.0056`, CARRIED. 5 ps: `+0.0034`, dropped. Joint pass region exhibited: carried at `β = 0.02` and refused at `β = 0` on the same carrier (M-JOINT-PASS-REGION) | PASS |
| **PR-2** the same field from another axis's chain | DROPPED, `≤ 0.01` | `−0.0056` (1 ps), `−0.0344` (5 ps) | PASS |
| **PR-3** affine copies of kept columns (`2ρ_c − 3`; `½ρ_s − 2 j^L_s + 1`) | DROPPED, `0` to `10⁻¹⁰` | exactly `0.0000`; both columns declared redundant | PASS |
| **PR-4** the time-shuffled dictionary (one permutation for every chain) | admits nothing, `≤ 0.01` | planted, `Q`, structural block, hydrodynamic block: `−0.0010 … −0.0001` at both lags | PASS |
| **PR-5** both charts through one interface on `response1_L200_seed0` | the per-arm table to the printed digit | 33 of 33 lines identical | PASS |

In CI (no walks), the plants run on a synthetic hydrodynamic carrier. G4 runs at `n = 12, 16`
(21 + 29 removed), and G5 runs on a synthetic block. The walk-sized reads are `#[ignore]`d:
PR-5 with G1, G2 with PR-1..PR-4 on the carrier of record, and G4 at `n = 20`. This driver
output is their record. They were run by hand for this read and are green.

## 3. The branch, read

By the letter the branch is **(c)**: G3 is killed on both halves. The branch's text gives the
cause as "the removability statistic does not reproduce the banked reads; the statistic is wrong
or the reads were". **That is not what happened.** The statistic reproduces every banked read it
was pointed at, exactly or to the fourth decimal: ORDER-1's eighteen numbers, HBOND's three, the
conscience sector's `8.90`, R1″'s `0.180` and ACUITY-1's 76. Both kills come from G3 as written:

- **G3a** asks the gate to keep a block while the kept set holds a second chart of that block.
  A per-candidate gate tests each block against all the others, so it drops each of two
  redundant blocks. It dropped the structural block given `(ρ, j^L)` (G2), and it drops
  `(ρ, j^L)` given the structural block at 1 ps. A tier that applied both drops would lose the
  density entirely: the price of dropping both is `+0.04 … +0.10` (the "nothing kept" row), well
  over budget. **This is a defect of the gate as frozen, not of the fluid.** Admission has to be
  ordered or joint: admit, then test the next candidate against what has been admitted. §1
  describes the gate one candidate at a time, and the build followed it.
- **G3b** grades carriage with a closure number. The `8.9×` measures how well the DMA block
  predicts ITS OWN next state against a re-paired null. It does not measure what the DMA block
  adds to the next thought's process sector. On the target the stake names, the addition is
  `+0.016 ± 0.008`: real at 2σ, and under the only budget in the prereg. No budget for the chains
  was frozen.

Whether this is (c) or "none cleanly" is the lead's call. What the build can state is that the
engine port holds and the gate works as specified: it carries a planted field, drops redundancy
at exactly zero, refuses by price and matches the circuit removal gate for gate. What failed is
that the gate is specified one candidate at a time, and that two of the stakes asked the wrong
question of it.

## 4. What the prereg got wrong

1. **G2's network block on the SLOW-1 walks.** Those walks carry oxygens only, so there is no
   H-bond network to build. HBOND's banked number came from other walks with another target. The
   build read the network block where the number was banked; the prereg should have named those
   carriers.
2. **G3 names no lag**, and at 5 ps nothing is predictable: every candidate's `R²` is negative,
   and even the unit-amplitude plant reads `+0.003`. G2's 5 ps drops are therefore **vacuous**
   (M-VACUOUS-SUCCESS). At that lag the gate drops everything, including the hydrodynamic block.
3. **G3's "others kept" holds a second chart of the kept block** (the coarse density counts).
   Combined with a per-candidate gate, this kills G3a by construction at any lag where the counts
   chart the density as well as `ρ_k` does. The joint pass region of G2 and G3 was never exhibited
   before the freeze, which is M-JOINT-PASS-REGION again, in the freeze that cites it.
4. **G3b stakes a carriage verdict on a closure statistic** (`8.9×` is DMA → DMA), and freezes no
   budget for the chains.
5. **G4's "every kept gate moves the amplitude by > 10⁻³"** states more than ACUITY-1 measured.
   ACUITY-1's "27 of 27" does not reproduce under its own `n = 20` method (25 of 27), and per gate
   only 150 of 248 move it. The kill (the removed set) is unaffected.
6. **§0's "the cell chart's 0.62"**: on the graded pool the cell chart reads `0.310`. `0.616` is
   the 4-cell pool.
7. The stakes read the tier's admission (§1: "on the RESPONSE-1 and SLOW-1 walks") only on SLOW-1.
   RESPONSE-1 enters only through the chart.

## 5. What was built

- **`holon-closure::removable`**: `Admission { carried, dropped, prices, se }`; `increment` (held-
  out `R²` increment, the readers' conventions as `Ridge::ORDER1` / `Ridge::HBOND` and
  `Folds::TimeBlocks` / `Folds::Units`, jackknife over blocks, the redundancy guard); `admit` with
  `Nulls { shift, swap: Derange | Donor | Permute }`; `admit_certified` (a proven bound in place
  of a regression); `refuse_drop` (REFUSED by name and price); `closure_r2`;
  `vamp_heldout_full`; `NumpyPcg64` (numpy's `default_rng`, checked against numpy); `ou_field`.
  Pure std, zero dependencies, deterministic.
- **`holon-lens::staggered`**: `FluidChart`, `closure_read`, `pool`, `jackknife`,
  `two_sided_floor`, `format_grid`. **`holon-lens::walk`**: the two walk readers, `structure`,
  `pair_entropy_s2`, `fourier_features`, `coarse`, `chains`, `order_features`, `hbond_features`,
  `plant_po4`.
- **`holon::sector`**: `locate` takes its removal from `removable::admit_certified` at
  `REMOVAL_BUDGET = 10⁻¹²`, with the cone as each gate's certificate. `light_cone_admission`
  exposes the certificates. `locate`'s API is unchanged, and its tests and ACUITY-1's are green.
- `cargo test --release -p holon-closure -p holon-lens -p holon` is green under
  `taskset -c 18-20`. Clippy adds no warnings in any touched file. Clippy on `holon-lens` stops on
  a pre-existing `deny(clippy::eq_op)` at `census.rs:516` (`i += 1 - 1`), which is not touched
  here; the touched files were linted with that one lint allowed.

---
witness: `Closed` (Object.lean) and `StatClosure.lean`, cited in words; the gate's numbers are measured against banked reads
**misfits:** M-JOINT-PASS-REGION (G2 × G3a; recurs in the freeze that cites it), M-VACUOUS-SUCCESS (G2's 5 ps drops), M-PLANT-OBS (PR-1 re-derived; fires at `+0.104`), M-PLANT-SECTOR — contacted, cited.

## Re-read under Amendment 1

*2026-09-26. Amendment `REPLACE1_AMENDMENT_1.md` (frozen alone, `a99b690`). Instrument:
`Admission::ordered` in `holon-closure::removable` (backward elimination: remove the candidate
whose removal from the standing set costs least, stop when the cheapest removal exceeds `β`;
ties within `10⁻¹²` go to the first-declared; kept columns assembled in name order so the
arithmetic does not depend on declared order; the stop reads the price, the nulls are read at
every step and reported). `Admission::marginal` is the one-at-a-time record. The read is
`replace1_read_amend1.txt` (`examples/replace1 -- --only a1`). Declared in code before the
read: G3a's dictionary is `[(ρ, j_L), (n33, n50, nb), (q, s2)]` in that order with nothing
kept outside it; G3a's nulls are the time shift and the chain two over (another axis); G3b's
dictionary is `[process sector, conscience block]`. PR-6 and PR-7 are also in CI on the
synthetic carrier.*

| stake | as amended | read | verdict |
|---|---|---|---|
| **G3a** | survivors ∋ hydrodynamic or count block; removing the survivor REFUSED at `≥ 0.04`; `(q, s2)` removed at `≤ 0.01` | all three seeds: `(ρ, j_L)` survives alone, REFUSED at **`+0.0622 / +0.0390 / +0.1039`**; `(q, s2)` removed at `−0.0015 / −0.0017 / −0.0030` | **not killed; the `≥ 0.04` bar missed on seed 1 by `0.001`** |
| **G3b** | conscience block survives, removal price `≥ 0.02`, null `≤ 0.005` | survives (`β = 0.01`); removal price **`+0.0164 ± 0.0081`**, swapped null `−0.0013` | **not killed; the `≥ 0.02` bar missed** |
| **PR-6** | ordered keeps one of two copies at the column's price; marginal drops both | ordered removes copy A at `0.0000` and keeps B at `+0.1041`, the single column's price, to `10⁻¹²`; marginal drops both at `0.0000` | PASS |
| **PR-7** | removal order deterministic, independent of declared order (ties excepted) | re-run bit-identical; declared, reversed and rotated dictionaries give the same order, survivors and step prices (to `10⁻¹²`) | PASS |
| **G4 amplitude clause** | as ACUITY-1 measured it | 25 of 27 under ACUITY-1's four-gate method, 27 of 27 with every kept T; 150 of 248 gate by gate. Dated correction added to `QVM_ACUITY1_RESULTS.md` | reported |
| **BRANCH** | | | **none of (a)/(c)/(e) cleanly**: no kill fires and both plants pass, but a bar is missed on each half of G3. The lead's call. |

**Removal orders, with each step's price** (nulls shifted / swapped):

| seed | step 1 | step 2 | survivor (removal price) |
|---|---|---|---|
| T293_seed0 | `(q, s2)` `−0.0015` (`−0.0024 / −0.0047`) | counts `−0.0010` (`−0.0019 / −0.0045`) | `(ρ, j_L)` **`+0.0622 ± 0.0153`** (`−0.0005 / −0.0036`) |
| T293_seed1 | counts `−0.0031` (`−0.0025 / −0.0035`) | `(q, s2)` `−0.0017` (`−0.0005 / −0.0050`) | `(ρ, j_L)` **`+0.0390 ± 0.0073`** (`−0.0066 / −0.0048`) |
| T293_seed2 | `(q, s2)` `−0.0030` (`−0.0052 / −0.0055`) | counts `+0.0033` (`−0.0027 / −0.0087`) | `(ρ, j_L)` **`+0.1039 ± 0.0195`** (`−0.0094 / −0.0104`) |
| chains | none removed | — | process sector `+0.3902 ± 0.0178` (null `−0.0139`); conscience block **`+0.0164 ± 0.0081`** (null `−0.0013`) |

The marginal record, beside it: `(ρ, j_L)` `+0.0087 / +0.0094 / +0.0091`, counts
`+0.0002 / −0.0031 / +0.0045`, `(q, s2)` `−0.0015 / +0.0003 / −0.0030`, all dropped. Ordered
admission repairs exactly the defect the amendment names. The hydrodynamic block is carried on
every seed, the count block goes second or first, and nothing on this carrier is left in a
redundant pair.

**What is wrong in the amendment.**
1. **G3a's `≥ 0.04` bar was set from the numbers it grades.** The survivor's price after both
   other blocks are removed is, by construction, the "nothing kept" increment already in §1's
   diagnostic table: `+0.0622 / +0.0390 / +0.1039`. §3 of these results rounded that range to
   "`+0.04 … +0.10`". That rounding is **my error**, and the amendment took its lower end as the
   bar. Seed 1 then misses by the rounding. The bar tests nothing the read had not already
   shown.
2. **G3b's `≥ 0.02` price bar sits above the price already read.** With the process sector the
   only other block, ordered and marginal admission compute the same number, `+0.0164`, which
   was in the first read. The declared `β = 0.01` makes the block survive, and the price bar
   then fails it on a number that was known before the freeze.
3. **The branches do not cover "survives but a bar is missed"**, which is what both halves read.
   The kill lines ("both blocks removed"; "the conscience block removed") do not fire.
4. **G3a states no null and does not say how a null enters the stopping rule.** The build stops
   on the price alone and reports the time-shifted and chain-two-over nulls at every step. Every
   null read is `≤ 0.0139` in magnitude, so the choice changes nothing here.


## The lead's verdict (2026-09-26): BANKED AS (a), with G3's two bars recorded as the lead's

G1, G2, G4, G5 MET to the digit against the bank; every plant fires, including PR-6 (the
defect exhibited under marginal admission and absent under ordered) and PR-7 (determinism).
G3 in substance: the hydrodynamic block survives ordered admission ALONE on every seed and is
refused at `+0.062 / +0.039 / +0.104`; `(q, s2)` is removed at `≤ 0.003`; the count block never
survives; on the chains the conscience block survives at `+0.016` with a null of `−0.001`. The
two bars missed — `≥ 0.04` on seed 1 by `0.001`, `≥ 0.02` on the chains by `0.004` — were set
by the lead from a rounded quotation of the first read and from a number already read:
`M-BAR-FROM-THE-READ`, registered with this verdict as its third instance. They grade the
lead's rounding, not the gate. The engine is banked: the fluid-element tier runs on the
staggered chart with ORDERED admission and a price certificate, and the QVM's light-cone
removal is the same function. What is owed: the next freeze that uses the gate sets its bars
from a prior stake or a plant, never from a read.
