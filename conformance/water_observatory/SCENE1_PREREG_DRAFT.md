# SCENE-1 — the first scene certificate, on water: molecules generated under a fluid cell, certified against walks the generator never saw. PREREGISTRATION (DRAFT)

*DRAFT of 2026-11-11, revised the same day (budgets now measured, not chosen; G-T moved out).
NOT FROZEN. Nothing here is a freeze until the lead strikes this line and commits the file
alone under the name `SCENE1_PREREG.md`. At this draft: no SCENE-1 script exists, no reference extract has been
made, no generator has been fitted, and no statistic named below has been computed on any
walk. The walks themselves are not fresh — see §"What is already public".*

## Why

Issue #3 asks for a certificate that a GENERATED scene may stand in for a real one, for named
observables, inside a named envelope, with a named refusal outside it and `unresolved` where
the reference cannot judge. `lean/CIRISHolon/SceneCert.lean` (`e655aab`) states what the
object is: an exact section law plus a transport cost on an ensemble, with four
preconditions for stacking it under a tier. Those are theorems about the form. This campaign
is the first measurement of whether the form bites on a carrier where every piece exists:

- a coarse view the fluid tier really uses (the cell chart);
- a fine tier that can be sampled (the rigid operator's walks), so "reality" is in hand;
- a downstream consumer already measured to read one scene statistic and not others
  (SLOW-1: tetrahedral order `q` carries 5 ps mobility at 293 K, `+0.028` R², and nothing
  at 400 K);
- a known place for the marginals fence to stand: `q` is a three-body quantity, and a
  generator matched on pair distances alone is not obliged to reproduce it.

Water is the rehearsal. The object is the certificate, and the twin's streets, a mock
catalogue under a density field, and an atomistic box under a coarse-grained state are the
same eight parts on other carriers.

## What is already public (said before anything is computed)

- All six walks have been read by SLOW-1, SLOW-2 or SLOW-3. `q`'s distribution on them has
  been looked at. None has been read for a generator, a transport cost, a block-counted
  floor or a section law.
- SLOW-1's `+0.028` and the 400 K contrast are banked. The mobility consumer below is
  SLOW-1's own linear read, refitted on TRAIN only.
- The literature says pair-matched reconstructions of water under-represent tetrahedral
  order. The direction of stake S3 is therefore expected. Its size on this operator, and
  whether the instrument sees it, are not known.

## Carrier and inputs

- **The scene.** One readout of the rigid operator: 250 oxygens in a periodic box of edge
  `19.58` Å (`36.99` bohr). Oxygen positions only; this carrier holds no hydrogens.
- **The coarse view `v`.** The engine's cell chart at `--cells 5`: the box cut into
  5 × 5 × 5 cells of edge `3.92` Å, and the reading is the integer oxygen count in each of
  the 125 cells. A PARENT is one such 125-vector, taken from a real readout.
- **Walks** (read-only, all at 20 fs readouts, 2,501 readouts = 50 ps, loaded with
  `slow1_search.load`):
  - **TRAIN** (generators are fitted here and nowhere else): `replace0/slow1/T293_seed{0,1}`.
  - The **REFERENCE** (the certificate's "reality"; no generator sees it):
    `replace0/slow1/T293_seed2`, `replace0/slow3/T293_seed{3,4}`.
  - **THIN CLASS**: `replace0/slow1/T400_seed0`. One walk at 400 K.
  - **UNCOVERED CLASS**: none is loaded. 350 K and 500 K are named, and nothing at them
    is in the reference.
- **The reference extract (the first step, and the only one that touches a walk file).**
  Every 25th readout (0.5 ps) of each walk, 101 frames per walk: oxygen positions as
  float32, written to `scene1/reference_extract.npz` with a manifest of each source walk's
  byte length and digest. Every later step reads the extract and never a walk. The extract
  is committed, so the campaign can be re-run from the repository alone; the six walks are
  untracked files on one machine (`305 MB + 232 MB`) and the extract is what makes the
  record independent of them.
- **Environment.** Python 3 with numpy only, on a CPU. No engine binary, no torch.
  CPU work is pinned with `taskset` to one core class, and no wall clock is compared
  (M-PLACEMENT-LOTTERY). Every generator is seeded from an integer and the seed is in the
  output's name.

## The eight parts of issue #3, as built here

### Part 2 — the section law (EXACT)

For every generated scene `x` under parent `c`: `v(x) = c`, all 125 counts, as integers.

- Checked on every scene of every generator, every seed. Not on a sample, not on average.
- The reader reports EVERY violation (scene, cell, expected, found), not the first
  (M-FIRST-VIOLATION-ONLY).
- A generator with one violating scene is `refused` on the section law, and its match is
  printed and marked NOT READ.
- Lean: `Generator` carries the law as a field; `under_tier` is why it is checked first.

### Part 5 — the declared coupling

Per-molecule observables, each computed from a scene's oxygen positions with
`slow1_search.structure` and a nearest-distance pass:

| observable | what it is | level | read by |
|---|---|---|---|
| `d1` | distance to the nearest oxygen | pair | — (reported) |
| `d5` | distance to the fifth-nearest oxygen | pair | — (reported) |
| `n33` | oxygens within 3.3 Å | pair | consumer **C-DENS** |
| `q` | tetrahedral order from the four nearest | three-body | consumer **C-MOB** |

- **C-MOB**: SLOW-1's linear read of 5 ps displacement from `q`, `φ(q) = a·q + b`, with `a`
  and `b` fitted on TRAIN. Lipschitz constant `K = |a|`.
- **C-DENS**: the local-density reading `n33` itself. `K = 1`.
- **Decoration (a label, no certificate owed):** the orientation of a molecule's shell in
  the box frame, and anything about hydrogens.
- A certificate is owed for `q` under C-MOB and for `n33` under C-DENS. `d1` and `d5` are
  scored and printed with no verdict: nothing declared reads them.

### Part 1 — the match

For each (observable, consumer) pair that owes a certificate, between the generator's
ensemble and REFERENCE:

- **W1**: exact 1-D Wasserstein-1 (sorted pairing, with the larger sample quantile-thinned
  to the smaller).
- **Quantile defects** at levels `{0.1, 0.5, 0.9}` and, where the floor resolves them,
  `{0.01, 0.99}`.
- **The mean difference**, printed beside every W1. It carries no verdict. It is there so
  that a case where the mean passes and the distribution refuses is visible as such.

### The budgets: measured, not chosen; two rules, both printed, one staked

1. **Noise-floor rule (staked).** A scene is certified when REFERENCE cannot tell it from
   reality better than reality can be told from itself. From 1,000 block-bootstrap splits
   of TRAIN into two halves, each component of the match (W1 and each quantile defect, per
   owed observable) gets its real-against-real median. The budget `β₀` of every component
   is that median times ONE common multiplier `m`, where `m` is the smallest value at
   which all components of a pair pass together in 95 % of the splits. Calibrating the
   components jointly is what keeps reality from being refused one time in three by ten
   separate 5 % tests. No number here is anyone's choice except the 95 %.
2. **Consumer rule (reported).** `stacked_le` bounds the downstream error by
   `K · cost + n · ε`. `β_C(o) = ε_C / K_C` is the mismatch at which the scene's term
   equals the tier's own, with `ε_C` the consumer's mean absolute residual per molecule on
   TRAIN. C-DENS has no residual of its own and gets no consumer budget.

Both are computed from TRAIN alone and written to `scene1/budgets.json`, which is
committed alone before any generator is scored (M-BAR-FROM-THE-READ).

**Sharpness, printed beside every verdict:** the ratio `β_C / β₀`. It says how many times
reality's own scatter this consumer could absorb. A ratio above 10 means the certificate
is stricter than its only declared consumer needs, by that factor; the verdict stands and
the sentence is printed with it (M-VACUOUS-SUCCESS). This is a live possibility for C-MOB,
whose R² is `0.03`.

### Part 4 — the resolution floor: two rules, compared

- **Naive rule.** A quantile level `p` is resolved if at least 30 samples lie beyond it.
- The **block rule.** REFERENCE is cut into time blocks of length `L_b` per walk. A level is
  resolved if at least 30 blocks each hold at least 30 samples beyond it, counting one
  sample per molecule per block (the block's first frame).
- **`L_b` is set by a gate on TRAIN, not chosen.** The integrated autocorrelation time
  `τ_int` of the indicator `1[q > its 0.9 quantile]` is measured per molecule on TRAIN and
  averaged. `L_b` is the smallest of `{2.5, 5, 10}` ps with `τ_int ≤ L_b / 2`. If none
  qualifies, `L_b = 10` ps and the excess is stated beside every verdict.
- **Consequence, stated now.** REFERENCE is three walks of 50 ps: 60, 30 or 15 blocks. At
  `L_b = 10` ps the 293 K class itself has 15 blocks and is `unresolved`. That is a legal
  outcome (branch (u)) and it names what would resolve it: three more 50 ps walks.
- THIN CLASS is one walk: 20, 10 or 5 blocks. It cannot reach 30 at any `L_b`.

### Part 3 — the envelope

Classes are (temperature, waters, cells). The certificate is earned on
`(293 K, 250, 5)` only.

- `(400 K, 250, 5)` is declared COVERED BUT THIN: expected verdict `unresolved`.
- `(350 K, ·, ·)` and `(500 K, ·, ·)` are outside: expected verdict `refused`, by name,
  without the match being consulted.

### The verdict

`SceneCert.verdict`, in its order: section law, then envelope, then floor, then match. The
reader implements that function and nothing else decides a verdict.

### Part 6 — determinism and random access

- Same parent and seed give the same scene, byte for byte, on two runs in two processes.
- Any one parent's scene can be generated without generating any other's.
- Cross-platform identity and a WASM build are NOT tested here.

### Part 7 — composition

On REFERENCE frames, with `gen` the generated scene under each real frame's own parent:

- The coupling pairs molecules cell by cell. The section law makes the counts equal, so
  the pairing always exists; within a cell, molecules are paired in sorted order of `q`.
- `truth` is the real molecule's 5 ps displacement; `coarse` is C-MOB.
- Measured: `Σ |coarse(q(gen_i)) − truth(real_i)|` against `K · cost + n · ε`, where `ε` is
  now the maximum absolute residual on those frames, so `htier` holds by construction.
- **The counter-arm.** The same bound with the cost taken on `n33` instead of `q` — a
  certified observable the consumer does not read. Precondition (1) of `stacked_le` is
  then unmet (M-BOUND-HYPOTHESIS-UNMET), and whether the inequality survives is reported.

### Part 8 — witnesses for a refusal

Every `refused` verdict writes `scene1/witness_<generator>_<observable>.json`: the
observable, the level, both quantile functions on a 201-point grid, the budget that broke,
and its source rule.

## Generators

Every one takes (parent, seed), places each cell's count of oxygens within that cell, and so
satisfies the section law by construction. That is the claim the section check tests.

| id | rule | fitted on | expected |
|---|---|---|---|
| **G-U** | uniform in the cell | nothing | refused on every pair |
| **G-X** | uniform in the cell, rejecting any oxygen closer than `r_min` to one already placed; `r_min` = TRAIN's 0.01 quantile of `d1` | TRAIN (`r_min`) | refused on `q` |
| **G-P** | reverse Monte Carlo on TRAIN's `g_OO(r)` out to 6 Å, moves confined to the oxygen's own cell, started from G-X, stopped at 2,000 attempted moves per oxygen | TRAIN (`g_OO`) | certified on `n33`, refused on `q` |


- Parents: the 101 extracted frames of each REFERENCE walk, 303 parents.
- Seeds: seed 0 on every parent, which is the match ensemble: 303 scenes, 75,750
  molecules. Seeds 1 and 2 on every 10th parent (31 parents) as well, for the section
  law's every-seed claim: 365 scenes per generator in all.
- The generator **G-T is not in this freeze.** A generator matched on `g_OO` and on the `q` histogram
  is the candidate that might pass, and it is the expensive one (§Cost). It is owed as
  SCENE-1b, frozen after this read, with a faster move.
- G-P's stopping rule is a move count, not a convergence criterion. A generator
  that has not converged is a worse generator, and the certificate is what says so.

## Stakes (drafted by the instrument's author; the lead's to set)

- **S1 (the certificate can say yes).** P-REAL is `certified` on both owed pairs,
  with W1 ≤ `β₀`. **Kill:** P-REAL is refused on either. A certificate that
  refuses reality is broken and nothing below is read.
- **S2 (the certificate can say no).** G-U is `refused` on both owed pairs with
  W1 ≥ `3 β₀`. **Kill:** G-U is certified on either pair.
- **S3 (the marginals fence, in the field).** G-P is `certified` on (`n33`, C-DENS) and
  `refused` on (`q`, C-MOB), with G-P's W1 on `q` at least `3 ×` its W1 on `q` for P-REAL.
  **Kill:** G-P is certified on `q` (W1 ≤ `β₀`) — then pair matching
  under a cell constraint fixes tetrahedral order on this operator and the fence does not
  stand here.
- **S4 (the stacked bound).** The measured downstream error is ≤ `K · cost + n · ε` on
  every generator: EXACT, a theorem, so a violation is an instrument fault. Reported
  beside it: the ratio `K · cost / (n · ε)`. If that ratio is below `0.01` on every
  generator, S4 is VACUOUS for the scene term and is said to be.
- **S5 (the floor rules differ).** At level `0.99` the naive rule resolves and the block
  rule does not, on REFERENCE. **Kill:** the two rules agree at every level in
  `{0.01, 0.1, 0.5, 0.9, 0.99}` — then block counting buys nothing at this sample size.

## Plants

Each runs before any generator is scored, and its result is printed ahead of every stake
line. Each names its carrier and the sector it acts on.

- **P-REAL** (carrier: REFERENCE split by walk, seed 2 against seeds 3 and 4; sector: the
  owed observables' own distributions). Real against real. Must be `certified`.
- **P-SHIFT** (carrier: REFERENCE; sector: the per-cell counts). Each parent is paired with
  the real scene 50 frames later in the same walk. The match is real against real; the
  counts differ. Must be `refused` on the section law, with every violating cell listed,
  and the match marked NOT READ. Fires only if at least 1 cell count differs in at least
  95 % of pairs.
- **P-ONE** (carrier: G-X's ensemble; sector: one cell of one scene). One oxygen of one
  scene out of 365 is moved to the neighbouring cell. Must be `refused`, naming that scene
  and those 2 cells.
- **P-MARG** (carrier: REFERENCE's per-molecule table of the six cosines among the four
  nearest neighbours; sector: the dependence among those six columns). Each cosine column
  is permuted independently across molecules within a block. `q` is `1 − ⅜ Σ (cos + ⅓)²`,
  a sum of one term per column, so every column's distribution AND the mean of `q` are
  unchanged exactly. Must read: mean difference of `q` below `10⁻⁹`, and W1 on `q`
  `refused` at ≥ `3 β₀`. This is the twin's correlated outage, planted.
- **P-DUP** (carrier: REFERENCE with every extracted frame repeated 10 times; sector: the
  sample count). The naive rule's deepest resolved level must move; the block rule's must
  not move at all.
- **P-ENV** (carrier: THIN CLASS and a declared 350 K request; sector: the class label).
  THIN CLASS must return `unresolved`; the 350 K request must return `refused` with the
  class named, and the reader must not have opened a scene to say so.
- **P-DET** (carrier: G-P; sector: the seed). Parent 17, seed 1, generated in two separate
  processes, one of them generating nothing else: identical bytes, 1 digest.

**Carrier-sector statement (M-PLANT-SECTOR).** The sector the plant acts on is nonzero in
its carrier for each plant, and for the two where that is a fact about the liquid rather
than a construction it is measured on TRAIN before the plant is scored: P-SHIFT needs
cell counts that change in 1 ps (the oxygen MSD at 2.5 ps is `1.4–1.7` Å² against a cell
edge of `3.92` Å, and the fire condition above refuses the plant if they do not), and
P-MARG needs the six cosines to be dependent (printed as the ratio of `q`'s variance
before and after permutation on TRAIN; the plant is VOID if that ratio is within 5 % of 1).

## Branches (evaluated in this order)

- **(e)** A plant fails, or S1's kill fires. Nothing is read. The failure is the finding.
- **(u)** The block gate sets `L_b = 10` ps and the 293 K class is `unresolved`. The
  plants and the section law are reported; the match verdicts are not issued; the walks
  that would resolve the class are named.
- **(a)** S1–S5 met. The certificate says yes to reality, no to noise, separates a
  pair-matched scene from a real one on the three-body observable, and its floor rules
  differ where they should.
- **(b)** S1 and S2 met, at least one of S3–S5 not. Stated with the clause that failed.
  If S3's kill fires: the fence is real as a theorem and absent on this carrier.

## Order

1. This file, frozen by the lead, committed alone.
2. `scene1_extract.py` writes the reference extract and its manifest. Committed alone.
3. `scene1.py` is built. TRAIN calibration: `a`, `b`, `ε`, `r_min`, `g_OO`, `τ_int`,
   `L_b`, `m`, `β₀`, `β_C`. `scene1/budgets.json` committed alone.
4. Every plant runs and prints.
5. Generators run. Section law, then verdicts.
6. The lead starts the read of the stake lines.

If step 3 cannot be built as written, a dated note is appended here under "Notes on
building" and committed alone before step 4.

## Cost

- Extract: six text files of 35 MB parsed once.
- Calibration and plants: nearest-neighbour passes over at most 606 frames of 250 oxygens.
- G-P: `365 scenes × 250 oxygens × 2,000 moves` = `1.8 × 10⁸` attempted moves. Timed on
  2026-11-11 on synthetic positions (no walk read) at `177 µs` per move in plain numpy on
  one core: about `9` core-hours, parallel over scenes.
- G-T, as it would naively be written, timed at `5.3 ms` per move (recomputing `q` for
  about 30 neighbours twice per move): about `270` core-hours for the same ensemble. That
  is why it is not in this freeze (M-CHEAPER-THAN-ITS-PRICE: the figure is a timing of a
  synthetic run, not of the campaign).
- Step 3 re-times 10 real scenes and prints the extrapolation before step 5 is launched.
- No engine run. No new walk, unless branch (u) asks for three.

## What this does not test

- **Not reality.** The reference is the rigid operator's own walks. This certifies a generator
  against a simulation it did not see, which is the rehearsal of the certificate, not
  water against experiment.
- **The reference's own sampling error.** Three walks from one operator. The certificate
  is against that sample.
- **Dynamics.** Generated scenes are static. No generated scene is stepped by the engine,
  so nothing is said about whether a certified scene relaxes as a real one does.
- **Hydrogens, orientation, energy.** The carrier is oxygen-only.
- **Cross-platform determinism and WASM** (issue #3 part 6, second half).
- **Any other class.** One temperature, one size, one cell chart.

---
witness: `CIRISHolon.SceneCert.Generator` (the section law, Part 2)
witness: `CIRISHolon.SceneCert.stacked_le` (S4 and the budget rule)
witness: `CIRISHolon.SceneCert.marginals_match_joint_differs` (S3, P-MARG)
witness: `CIRISHolon.SceneCert.verdict_certified_iff` (the verdict's order)
witness: `CIRISHolon.SceneCert.cost_triangle` (not exercised here: no distillation chain)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-BAR-FROM-THE-READ, M-VACUOUS-SUCCESS, M-FIRST-VIOLATION-ONLY, M-BOUND-HYPOTHESIS-UNMET, M-PLACEMENT-LOTTERY, M-CHEAPER-THAN-ITS-PRICE, M-FLOOR-UNSTAKED, M-BASE-RATE-OMITTED, M-PROVENANCE-OVERREACH, M-HOMOG, M-VOLUME-SCALE, M-ONE-MODEL-DELTA. These are the registered ids this text contacts.

- M-PLANT-OBS: every plant is built for THIS reader and runs before any generator is scored.
- M-BAR-FROM-THE-READ: every budget is measured on TRAIN and committed before a generator is scored. None is chosen.
- M-VACUOUS-SUCCESS: the sharpness ratio and S4's ratio name the two ways a pass here could mean less than it says.
- M-FIRST-VIOLATION-ONLY: the section reader lists every violating cell of every scene.
- M-BOUND-HYPOTHESIS-UNMET: the counter-arm runs `stacked_le` with its factoring hypothesis deliberately unmet and reports it as such.
- M-FLOOR-UNSTAKED: the resolution floor is staked (S5) and planted (P-DUP), under two rules.
- M-BASE-RATE-OMITTED: the mean difference and the real-against-real W1 are printed beside every generator's W1.
- M-PROVENANCE-OVERREACH: the extract's manifest carries each walk's byte length and sha256. A digest names bytes; that those bytes are SLOW-1's and SLOW-3's walks rests on the paths and on those campaigns' own records.
- M-HOMOG: "local" density means a 3.3 Å sphere on one molecule, not a claim of spatial locality on a graph family.
- M-VOLUME-SCALE: one box size on one cell lattice. Nothing is claimed about convergence in either.
- M-ONE-MODEL-DELTA: G-P stands for "pair-matched under a cell constraint at 2,000 moves per oxygen", not for reverse Monte Carlo as a method. No Markov-state model is built.
