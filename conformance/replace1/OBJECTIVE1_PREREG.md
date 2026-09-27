# OBJECTIVE-1 — is a tier's CARRIED SET invariant across the observer's budget and cadence? PREREGISTRATION

*Frozen 2026-09-26, committed alone before any code. The stakes are the lead's. `OBJECT.md`
CORRECTION 2026-09-25 made a tier "a Closed view of the tier below THAT THE TIER ABOVE
CARRIES — one its conservation law cannot remove within its budget", and REPLACE-1 put that
into the engine as `Admission::ordered` (backward elimination with a joint price,
`REPLACE1_AMENDMENT_1.md`) and read it at ONE budget (`β = 0.02`) and ONE cadence (lag `1 ps`).
The definition then has two observer's knobs in it. If the survivor set is the same across a
range of budgets and lags, the level is there whoever set the tolerance (Wimsatt's robustness:
invariance across independent means of access); if it moves, the observer stays in the
definition, and the record should say for which tier and over what range. Nothing here is new
statistics: backward elimination is Efroymson (1960), and a regularisation path read for the
stability of the selected set is Meinshausen–Bühlmann's stability selection (2010). What is
ours is the carrier, the target (the level above's next state) and the stakes frozen before
the sweep.*

## 1. The sweep

- **Fluid.** `Admission::ordered` on the SLOW-1 walks `T293_seed0`, `T293_seed1`, `T293_seed2`
  (and `T400_seed0` as the contrast), dictionary = `{hydrodynamic block (ρ_k, j_L),
  count block (n33, n50, nb), structural block (q, s2)}` exactly as REPLACE-1 Amendment 1
  built it (`fluid_dict`: nothing kept outside the dictionary, target `(ρ_k, j_L)` at the next
  lag, `Ridge::ORDER1`, `Folds::TimeBlocks { k: 5 }`, nulls = the time shift and the chain two
  over), over `β ∈ {0.005, 0.01, 0.02, 0.05}` × lag `∈ {0.5, 1, 2, 5} ps` (25, 50, 100, 250
  readouts at 20 fs): 16 cells per seed.
- **Chains.** The accord chains (207 chains, 444 transitions, REPLACE-1 G3b's carrier and
  question: target = the next thought's process sector, dictionary = `{process sector,
  conscience (DMA) block}`, chain folds `i mod 4`, the permutation null only) over
  `β ∈ {0.005, 0.01, 0.02}` at lag 1 thought: 3 cells.
- **Per cell, reported:** the survivor set; the removal order; each step's price, SE and
  nulls; and the target's held-out `R²` from the FULL dictionary. **A cell is UNDEFINED where
  that `R² ≤ 0`**: the target is not predictable there, every block is removable by
  arithmetic, and no invariance claim is made on it (M-VACUOUS-SUCCESS). Undefined cells are
  reported and never graded.
- **Per (seed, lag), reported beside the grid:** the full removal path (the gate run with the
  budget set to `+∞`, so every block is removed and priced), and from it each block's `β*` —
  the largest budget at which it survives, i.e. the price of the step that removes it, or the
  largest earlier step's price if that is higher. Also reported: the step prices of EVERY
  standing candidate at every step (not only the one removed), with SEs, so that Q4's
  "within their SEs" can be read.

## 2. Stakes

- **Q1 — invariance on the fluid.** Over every DEFINED cell at 293 K, the survivor set is the
  same on each seed: the hydrodynamic block alone. **Kill:** the survivor set differs between
  two defined cells on the same seed. Reported beside it: each dropped block's `β*`, so the
  reader sees how far the invariance extends. A block with price `0.003` is dropped at every
  `β ≥ 0.005` by arithmetic; the claim has content only if some block's price lies INSIDE the
  sweep `[0.005, 0.05]` and the survivor set still does not change. The sweep's lower bound
  `0.005` is above every dropped price REPLACE-1 read at 1 ps (`≤ 0.0033`), and the stake is
  honest about that: at 1 ps the only prices inside the sweep are the survivor's own.
- **Q2 — the contrast.** At 400 K the survivor set over defined cells is reported. No stake:
  the target's `R²` there is near zero by ORDER-1, and most or all of its cells are expected
  UNDEFINED.
- **Q3 — the chains.** Over the defined cells the process sector survives at every `β`; the
  conscience block survives at `β ≤ 0.01` and is dropped at `β = 0.02` (its price is
  `+0.0164`, REPLACE-1 §re-read). The chains' carried set is therefore NOT budget-invariant
  across this range, and the stake says so in advance: Q3 is the honest negative, the
  level-relative case. **Kill (of the framing, not the campaign):** the conscience block
  survives at `β = 0.02`, or is dropped at `β = 0.005`.
- **Q4 — seed invariance.** The fluid survivor set and removal ORDER agree across the three
  293 K seeds at every lag and `β` where all three cells are defined. The order may differ
  where the two prices that decide it are within their SEs: at the first step where two seeds'
  orders diverge, if on each diverging seed the removed candidate's price and the price of the
  candidate the other seed removed differ by `≤ √(se₁² + se₂²)` (their jackknife SEs at that
  step), the divergence is a tie and is reported, not killed. **Kill:** the survivor sets
  differ across seeds at a cell defined on all three, or the orders differ outside that
  tie rule.

## 3. Plants (carrier `slow1/T293_seed0`, lag `1 ps` unless named; read at every `β` whether or not the cell is defined)

| plant | must |
|---|---|
| **PI-1** | REPLACE-1's PR-1 field — ORDER-1's PO-4 re-derived for the engine: 5 ps memory, 1 ps latency, noise `0.05`, plant seed 5, **unit amplitude**, banked price `+0.1041` — added to the dictionary as a fourth block `planted`, on the planted carrier (the density carries the field): the planted block SURVIVES at every `β ∈ {0.005, 0.01, 0.02, 0.05}` and is DROPPED when `β = 0.2` — the budget dependence a carried-but-finite variable must show, so the sweep can see a change when there is one |
| **PI-2** | the time-shuffled dictionary: REPLACE-1 PR-4's permutation (PCG64 seed 11) of the readouts, applied to every dictionary block of every chain of `T293_seed0`, the TARGET NOT shuffled: ordered admission keeps NOTHING at any `β` in the sweep, at every lag (every step price `≤ 0.005`) |
| **PI-3** | the same field at amplitude **`0.25`**, with predicted price `+0.012` to `+0.018` (below): the planted block SURVIVES at `β ∈ {0.005, 0.01}` and is DROPPED at `β ∈ {0.02, 0.05}` — a change INSIDE the sweep, exhibited |

**PI-3's amplitude, declared from a plant and not from a read** (M-BAR-FROM-THE-READ). The
lead's text reads "price `+0.012` (half amplitude)". The two do not agree under the plant's own
arithmetic, and this freeze keeps the PRICE, which is what the stake grades, and sets the
amplitude from PR-1's banked `+0.1041` at unit amplitude. The plant adds `a · sd(ρ) · Z(t)` to
each density column, so the planted share of a density column's variance is
`f = a² / (1 + a²)` (`½` at `a = 1`). Two models of the increment, both anchored at `+0.1041`:
(i) proportional to `f`: `0.208 f`; (ii) the share of `Z(t + 1 ps)` the hydrodynamic block
cannot already extract from `Z(t)` (correlation `e^{−1/5}`), `0.313 f (1 − 0.67 f)`. At
`a = ½` (`f = 0.2`) they give `+0.042` and `+0.054` — a plant that survives at `β = 0.02`
under both, so the lead's "dropped at `{0.02, 0.05}`" would fail by construction. At
**`a = 0.25`** (`f = 0.0588`) they give **`+0.012`** and **`+0.018`**, both inside
`(0.01, 0.02]`, the window PI-3's must requires. `a = 0.25` is
declared here; no amplitude is read or tuned.

Carrier-sector statement (M-PLANT-SECTOR): PI-1 and PI-3 act on `T293_seed0`'s density columns
`ρ_k` (the target's own sector; the planted field is nonzero in that carrier by construction,
unit and quarter amplitude of `sd(ρ)`); PI-2 acts on every dictionary block of `T293_seed0`
and leaves the target untouched, so the sector it must empty is the whole dictionary.

## 4. What is known before the run (arithmetic on numbers already banked; stated so that none of it is later read as a finding)

1. **The `β` axis is arithmetic on the path.** Ordered admission's removal path does not depend
   on `β` — `β` only chooses where the path stops (the first step whose price exceeds `β`).
   One run per (seed, lag) at `β = +∞` therefore determines every `β` cell. The sweep runs each
   cell anyway, as staked, and checks it against the path; the only information the `β` axis
   carries is where the path's step prices fall relative to `{0.005, 0.01, 0.02, 0.05}`.
2. **Q1 is killed at 1 ps on seed 1 by a banked number.** REPLACE-1 read the hydrodynamic
   block's removal price at 1 ps as `+0.0622 / +0.0390 / +0.1039`. On seed 1, `+0.0390 < 0.05`,
   so at `(β = 0.05, 1 ps)` the survivor set is EMPTY, against `{hydrodynamic}` at
   `(β = 0.02, 1 ps)`; both cells are defined (full `R² ≈ 0.03`). Q1's kill line fires there
   unless the engine no longer reproduces its own banked read. The collision between the
   sweep's upper end and a banked price is recorded here, before the run, and the stake is
   not moved. What the run adds to Q1 is the
   other lags, and seeds 0 and 2.
3. **Q3 is arithmetic on a banked number.** With two blocks, the first step removes the
   cheaper; the conscience block's price `+0.0164` is below `0.02` and above `0.01`. Q3 as
   staked follows unless the engine's chains read has moved.
4. **Q4 at 1 ps is already known to diverge in order**: seeds 0 and 2 remove `(q, s2)` first,
   seed 1 the count block first, every one of those prices `≤ 0.0033` in magnitude. Whether
   that divergence is a tie under Q4's rule is not known (REPLACE-1 did not record the price
   of the candidate NOT removed); the run reads it.
5. **At 5 ps** REPLACE-1 read every candidate's `R²` negative, and PR-1 at `+0.003`; the 5 ps
   cells are expected UNDEFINED.

Predicted branch from 2 and 3 alone: **(b)**, with Q3 as staked. The run can move that only
if the engine does not reproduce REPLACE-1's own numbers.

## 5. Branches

- **(a)** Q1 and Q4 met, Q3 as staked → the fluid element's carried set is budget- and
  cadence-invariant over the defined range (level-objective for that tier), and the chains'
  is not (level-relative).
- **(b)** Q1 killed → the fluid's carried set is budget-relative too; the grid and each
  block's `β*` say over what range it holds.
- **(c)** Q3's framing killed → the chains' prices are not where REPLACE-1 read them.
- **(e)** a plant fails → nothing is read.

A Q4 kill with Q1 met is reported as its own line (seed-relative) and read under (a)'s
conditions as not met.

## 6. Cost and placement

No new trajectory. The SLOW-1 walks and the accord chains on disk; the dictionaries as
REPLACE-1 built them (about 15 s per walk); per (seed, lag) one path and four budgets, each
at most six increments with two nulls. Minutes. Run pinned with `taskset -c 18-20` (cores 0–17
and 21–27 belong to other campaigns); no wall-clock number is graded.

## 7. What this does not test

Any other tier; any other dictionary (the network block is not on these walks, REPLACE-1 §4.1);
any other `k`; the fine model; RESPONSE-1's walks; whether the survivor set is invariant to the
REGRESSION (ridge penalty, fold count) — a third observer's knob, named here and left out.

---
witness: `Closed` (Object.lean) and `StatClosure.lean` for the certificate's composition; the survivor sets are measured
**misfits:** M-VACUOUS-SUCCESS, M-BAR-FROM-THE-READ, M-JOINT-PASS-REGION, M-PLANT-OBS, M-PLANT-SECTOR, M-PLACEMENT-LOTTERY, M-STALE-INSTRUMENT, M-COND-PROBE — contacted, cited. M-VACUOUS-SUCCESS: a cell whose target is unpredictable (full-dictionary `R² ≤ 0`) is UNDEFINED and never graded, and PI-2 is read at every cell regardless. M-BAR-FROM-THE-READ: no bar here is set from a read; §4 lists every place where a banked number already decides a cell, and PI-3's amplitude is set from PR-1's banked plant price. M-JOINT-PASS-REGION: PI-1 (survives on `[0.005, 0.05]`, dropped at `0.2`) and PI-3 (changes inside the sweep) exhibit both outcomes the sweep grades on the carrier of record. M-PLANT-OBS: PI-1..PI-3 are re-derived with the engine's own plant (`walk::plant_po4`), not replayed. M-PLACEMENT-LOTTERY: the pin is stated; nothing graded depends on it. M-STALE-INSTRUMENT: the instrument is `Admission::ordered` at the commit that banked REPLACE-1 (`979984f`), extended only to record the prices of the candidates not removed. M-COND-PROBE: contacted by keyword only ("inside the sweep"); no operator is applied after a step, the gate reads the walks as run.

---

### Notes on building

*Appended by the build (2026-09-26) after the code existed and the numbers came in. No stake is
moved here. The read is `objective1_read.txt`; the verdict is `OBJECTIVE1_RESULTS.md`.*

**Where the code lives.** `engine/crates/holon-closure/examples/objective1/main.rs` (the sweep;
it takes REPLACE-1's chains reader by `#[path]`, and its dictionaries, question and nulls are
copied from `examples/replace1` unchanged). One engine change: `removable::Step` gains
`alternatives` — the price of every candidate standing at that step, the removed one included —
which is what Q4's tie rule needs. `Admission::ordered`'s decisions are untouched (every
REPLACE-1 test passes unchanged). A new unit test, `the_budget_only_chooses_where_the_path_stops`,
checks §4.1's arithmetic in CI: survivors at any `β` equal the path's running-maximum rule.

**Declared in code before the read:**
- `β = +∞` for the path (`Admission::ordered` with `f64::INFINITY` removes every block).
- `β*` of a block = the running maximum of the step prices up to its removal (§1).
- "Survivor set" = the blocks NOT removed, whatever their verdict. A survivor whose null exceeds
  `β` is `Unresolved` in the gate's verdict, but it was not removed; none occurred.
- Q4's tie rule reads the path's step (`β = +∞`), which is the cell's step, since a cell's
  order is a prefix of its path (checked on every cell).
- PI-1 and PI-3's dictionary on the planted carrier is `{H, C, S, planted}`, all on the planted
  walk (the density columns carry the field), nulls as the fluid's; plant seed 5 for both.
- PI-2 reads the full sweep (four lags, four budgets).
- The chains' "lag" column prints `0 ps`; it is lag 1 thought.

**Where the frozen text and the machine disagree** (in the results as "what the prereg got wrong"):
1. PI-3's amplitude arithmetic over-predicted its price by about 2×: read `+0.0069 ± 0.0076`
   against a predicted `+0.012` to `+0.018`. Neither model survives the read.
2. §4 listed Q1's advance kill at `(seed 1, 1 ps, β 0.05)` but not that the same banked
   number kills Q4's survivor-set clause at the same cell (seeds 0 and 2 keep `{H}`).
3. The definedness rule reads the FULL dictionary's `R²`, which the two dropped blocks drag
   below the hydrodynamic block's own: seed 0 at 2 ps is UNDEFINED at `−0.0030` while `H`
   alone, over nothing kept, is priced at `+0.0199`.
