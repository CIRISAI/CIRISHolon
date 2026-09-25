# SLOW-3 — the integration fixed and retried, selected on training data only, read on fresh walks. PREREGISTRATION

*Frozen 2026-09-25, committed alone. At this commit, `slow3_train.py` does not exist. No SLOW-3
fix, λ, SPIB variant or selection number has been computed. The two fresh walks (T293 seeds 3
and 4) were launched at 10:53 the same day and are about 1.2 days from producing a readable
walk. Nothing in them has been read, and nothing will be until the models are frozen (§Order).
The stakes are the lead's (2026-09-25), written here as given. Where the carrier could not hold
a stake exactly as worded, §"What could not be built as written" says what and why, and no
stake's number moves.*

## Why

SLOW-2 (`SLOW2_RESULTS.md`, branch (c)) killed its own integration. F, a VAMPnet trained on
VAMP-2 plus λ · the carried increment (λ = 10), failed PS-3 on the synthetic, scoring 0.684
against a bar of 0.9. On HELD-OUT it carried less than the plain VAMPnet D: +0.0076 beyond
velocity and `q`, against D's +0.0128. The in-batch carried term had fitted a 5 ps predictor
that neither generalised nor stayed slow. λ = 1 was reported without a stake. It passed PS-3
on the synthetic (0.920) and read +0.0137. SPIB collapsed to two states and failed PS-3. A
linear read of the 84 raw inputs (R0) carried +0.0194 beyond velocity and `q`.

**The discipline problem, which is the reason this campaign exists.** SLOW-2's HELD-OUT seed
(T293 seed 2) and its CONTROL (T400 seed 0) have both been read. A fix tuned while those
numbers are in view would pass by construction. So:

- the fix and its selection rule are declared here;
- the selection uses **only** the synthetic plant walk and the TRAINING seeds 0 and 1;
- the stake is read on **two fresh walks** (T293 seeds 3 and 4), which did not exist when
  this was written.

**What is already public, said before anything is computed.**

- SLOW-2's HELD-OUT numbers are listed above.
- λ = 1 on SLOW-2's objective passes PS-3 on the synthetic. That was seen on the synthetic
  and on seed 2. Seed 2 is not used here in any way.
- The lead's grid {0.3, 1, 3} brackets λ = 1. The grid is the lead's, and it is not re-chosen.

## Carrier and inputs (as SLOW-2, except where stated)

- **Walks** (read-only): the rigid operator, 250 waters, 2,501 readouts at 20 fs = 50 ps,
  oxygen positions and velocities, loaded with `slow1_search.load`.
  - **TRAIN** = `replace0/slow1/T293_seed{0,1}` (SLOW-1's walks).
  - **TEST (the stake)** = `replace0/slow3/T293_seed{3,4}`: FRESH. They are launched by
    `slow3.sh` exactly as SLOW-1's arms were (`slow1.sh`'s flags: `--cells 5 --settle 1000
    --settle-rigid 1000 --readouts 2500 --readout-fs 20`, the same stiffness file,
    `--workers 1 --temperature-k 293`), one per E-core, on cores 16 and 17.
  - **CONTROL** = `replace0/slow1/T400_seed0`. It has already been seen. It is read
    unchanged and reported for the contrast only, with no stake.
  - **Never used:** `T293_seed2`. It enters no fit, no selection and no read. The feature
    cache SLOW-3 builds contains only seeds 0, 1, 3, 4, T400 and the synthetic.
- **The binary.** SLOW-1's binary (`engine/target/release/examples/replace0` in the main
  checkout, built 2026-09-20 from the source of `65d4b2c`) **no longer exists**; the whole
  `engine/target` directory is gone. It was rebuilt from `git archive 65d4b2c engine`
  (toolchain 1.95.0) with one line changed. `SEEDS` is extended from three entries to five
  (`0x…4533`, `0x…4534`), because the binary panics on `--seed 3` otherwise. That panic was
  the first launch attempt, logged in `replace0/slow3/slow3_launch_attempt1_panic.log`.
  Seeds 0–2 are untouched, and the same line is committed in `replace0.rs`.
  - The binary's sha256 is printed in the launch log. A hash names bytes, and it does not
    name the source that produced them (M-PROVENANCE-OVERREACH). The claim "the same
    operator SLOW-1 ran" therefore rests on a measurement, not on the hash:
    `slow3_provenance.sh` re-runs SLOW-1's seed 0 with the rebuilt binary for 20 readouts
    and compares its `rigid.walk` and `rigid.vwalk` with SLOW-1's, byte for byte.
  - If they differ, the fresh arms are reported as runs of a rebuilt operator, and the
    difference is stated beside every stake.
- **Input.** SHELL, 84 columns: the 12 nearest-oxygen distances, the 66 pairwise cosines,
  and the six STRUCT columns, exactly as in `slow2_compare.shell` and `features`.
  Standardisation uses TRAIN's moments (seeds 0 and 1).
- **Lag and output.** τ* = 1 ps (50 readouts), with a 2-D output, as in SLOW-2. `f1` is the
  top linear-VAMP singular function of the arm's 2-D output at τ*, fitted on TRAIN
  (`slow2_compare.f1_map`).
- **Training rows.** Origins `t ∈ [0, T − 250)` at a stride of 5 readouts. The nets fit on
  molecules `i mod 5 ≠ 0` of every training walk, as in SLOW-2.
- **Device.** The nets train on the GPU, an NVIDIA GeForce RTX 4090 Laptop (cuda), with
  deterministic algorithms and `CUBLAS_WORKSPACE_CONFIG=:4096:8`. Torch seeds are 0, 1 and
  2, and an arm's value is the mean over the three with its min–max spread. The numbers
  are this device class's (M-DEVICE-CLASS); a CPU is not claimed to reproduce them
  bitwise.
- **CPU pinning.** CPU work is pinned with `taskset -c 18-20`, one thread per process.
  Cores 0–15 and 21–27 run other campaigns. No wall clock is compared across core classes.
- **Environment.** The Python environment used is
  `/tmp/claude-1000/…/scratchpad/venv` (torch 2.14.0+cu130, deeptime 0.4.5, spib 1.1.0,
  numpy 2.5.3). It is session-keyed (M-STALE-INSTRUMENT). The frozen weights are committed
  in the repository, not left in the scratchpad. The package versions are recorded so the
  read can be rebuilt in any environment that has them.

## The fix: F-fixed

These are the lead's four points, (i)–(iv), and two additions of mine. Each addition is marked
and has its reason.

The lobe, optimiser, batch, epochs and rows are D's: an MLP 84 → 128 → 128 → 2 with ReLU,
Adam at 5 × 10⁻⁴, batch 2,048, 20 epochs, fixed, with the final weights used. The loss per
batch is

  **L = −½ [VAMP2_τ*(batch) + VAMP2_2τ*(batch)] − λ · ΔR²_xfit(batch)**

Each part of that loss:

- **(iii) Closure at τ* and at 2τ*.** Each row carries `x(t)`, `x(t + τ*)` and
  `x(t + 2τ*)`. `vamp_score` (VAMP-2, `regularize`, ε = 10⁻⁶) is taken at both lags on
  the same batch and averaged. The average keeps the closure term on SLOW-2's scale, so
  λ means what it meant there. An output that predicts one lag ahead but decays by
  2 ps is penalised.
- **(ii) A linear head only.** The carried target `y = |r(t + 5 ps) − r(t)|`, standardised
  over TRAIN, is predicted from `[C, f]` by a closed-form ridge (10⁻³, standardised,
  differentiable). The net cannot carry a 5 ps predictor in a nonlinear head, because
  there is none.
  - *Said plainly:* SLOW-2's F head was already this linear ridge. (ii) is kept as an
    explicit constraint. It is not new.
- **(i) Cross-fitted: disjoint molecules AND disjoint time origins.** Each training walk's
  molecules are split into halves by molecule index (even and odd). Its origins are split
  into EARLY (`t + 5 ps < 25 ps`, i.e. `t < 1,000` readouts) and LATE (`t ≥ 1,250`). The
  rows with `1,000 ≤ t < 1,250` would have a 5 ps window crossing the split. They enter
  the closure term only.
  - The head is fitted on (even molecules, EARLY) and scored on (odd molecules, LATE), then
    fitted on (odd, LATE) and scored on (even, EARLY). The two scores are averaged. The
    off-diagonal rows enter the closure term only.
  - `ΔR²_xfit = R²_eval(y | C, f) − R²_eval(y | C)`. The second term is detached. It is a
    constant of the net.
  - *What is new over SLOW-2:* SLOW-2's head was held out by molecule index within the
    batch. Its fit rows and score rows therefore shared the same time windows. Molecules
    in one shell at one time move together, so that split does not hold out the collective
    part of the 5 ps displacement. The time split does, and with the guard band no fit
    row's 5 ps window reaches a score row's.
- **Addition A (mine): the conditioning set is `C = (vx, vy, vz, |v|², q)`, not velocity
  alone.** The stake statistic is the increment beyond velocity AND `q`. SLOW-2's F was
  rewarded for re-finding `q`, which the stake then subtracts; its objective targeted a
  different statistic from the one it was judged on. Conditioning on `q` makes the loss
  the training-time form of the stake's own statistic. `q` is a SHELL column, so no new
  input enters.
- **(iv) λ from the declared grid {0.3, 1, 3}, by the selection rule below.**
- **Addition B (mine), reported and not selectable: λ = 0 through F-fixed's code** (the
  two-lag closure alone). It separates what (iii) does from what the carried term does.

## SPIB-fixed

**The SLOW-2 diagnosis (building note 4).** k-means with K = 10 on the 85 standardised raw
columns does not split on the one planted column. SPIB then merges the fast labels until one
or two states remain.

**The fix.** Initial labels come from k-means on the **top four TICA components** (lag τ*,
`deeptime.decomposition.TICA`) of the standardised SHELL input, fitted on the SPIB fitting
rows. Clustering a kinetic projection rather than raw coordinates is the field's standard
state-space construction (TICA before k-means, as in Pérez-Hernández et al. 2013). The
SPIB authors' guidance is that the initial partition should be finer than the expected
number of metastable states, and that β controls how many survive; a smaller β keeps more.
The declared variants are:

| variant | K (initial states) | β |
|---|---|---|
| S1 | 20 | 10⁻² |
| S2 | 20 | 10⁻³ |
| S3 | 50 | 10⁻² |
| S4 | 50 | 10⁻³ |

Everything else is SLOW-2's E:

- the package `spib` 1.1.0, a nonlinear encoder with 2 × 128 hidden units, z_dim 2,
  lagtime 50, learning rate 10⁻³, batch 2,048;
- the fixed refinement schedule from SLOW-2's building note 1 (`tolerance = ∞`,
  `patience = 4`, 8 refinements, 45 epochs);
- a collapse to one state is caught and read as trained (building note 2);
- the output is `z_mean`.

## The selection rule (declared; uses ONLY the synthetic and TRAIN seeds 0 and 1)

It is the same rule for the F-fixed λ and for the SPIB-fixed variant.

1. **PS-3 screen, on the synthetic walk** (`slow1_search.synthetic()`, SLOW-2's PS-3
   exactly). The planted OU log-mobility is the 85th column. The arm is fitted on particles
   `i mod 4 ≠ 0` and read on `i mod 4 = 0`. The score is the planted column's R² on
   `(f1, f1², f1³)`. A candidate is **admissible** only if the score is **≥ 0.9 on 3/3
   torch seeds** (0, 1, 2).
2. **Two-fold cross-validated carried increment on TRAIN.** For each candidate:
   - train on seed 0 and read on seed 1, then train on seed 1 and read on seed 0, with
     torch seeds 0, 1 and 2 in each direction;
   - standardisation and `f1` come from the fold's own training seed;
   - the statistic is the stake's own: `R²(Z + q + F) − R²(Z + q)` on the read seed, by
     `slow2_compare.read` (4-fold, held out by molecule within the read walk);
   - the **CV score** is the mean over 2 folds × 3 torch seeds.
3. **Choice.** Among the admissible candidates, the one with the **highest CV score** is
   chosen.
   - A **tie** (CV scores within 0.0005 R² of each other) goes to the smaller λ. For SPIB, it
     goes to the variant earlier in the table (S1 before S2 before S3 before S4).
   - *How the lead's wording is read:* "the largest λ whose PS-3 passes … AND whose CV
     increment is highest; ties to the smaller λ" is taken as a filter (PS-3 on 3/3),
     then an argmax (CV), then a tie-break (smaller). Read literally, "largest" and "ties
     to the smaller" contradict each other. This reading honours both the plant and the
     tie-break.
4. **If no candidate is admissible:**
   - for F-fixed, **G1 fails** (the kill). The candidate with the highest minimum PS-3
     score is still frozen, and it is read and printed as NOT READ;
   - for SPIB-fixed, the candidate with the highest CV score is frozen and read, and it is
     marked NOT READ (G3 below).
5. **Freeze.** The chosen λ and variant are trained on TRAIN (seeds 0 and 1, molecules
   `i mod 5 ≠ 0`) with torch seeds 0, 1 and 2. Everything below is saved under
   `conformance/water_observatory/slow3/frozen/`, and a sha256 manifest is committed
   before either fresh walk has `run.done`:
   - those weights;
   - D (SLOW-2's setting, retrained identically);
   - the PS-4 models (each of D, F-fixed and SPIB-fixed trained on time-shuffled TRAIN,
     numpy seed 11, torch seed 0);
   - TRAIN's standardisation;
   - each arm's `f1` map.

   The read loads these files and trains nothing.

## Arms on TEST (fresh seeds 3 and 4) and CONTROL (T400 seed 0)

- **D, the plain VAMPnet**, unchanged from SLOW-2 (deeptime `VAMPNet.partial_fit`,
  VAMP-2, the same lobe and settings). It is the reference.
- **F-fixed** at the selected λ.
- **SPIB-fixed**, the selected variant (one of S1–S4, 2-D `z_mean`).
- **R0**, the linear ridge of the 84 SHELL columns (`slow2_compare.r0`), beyond velocity
  and `q`. It is the bar any learned slow variable has to justify itself against. It is
  fitted within the evaluation walk, as in SLOW-2.
- Reported beside them, with no stake:
  - A (SLOW-1's linear search on STRUCT, fitted on TRAIN);
  - `q`'s own carry beyond velocity, the base rate (M-BASE-RATE-OMITTED);
  - the λ = 0 F-fixed arm.

The measured statistics are SLOW-2's `read()`, unchanged:

- own σ₁ at 1, 2, 5, 10 and 20 ps;
- the increments beyond velocity, beyond velocity and `q`, beyond STRUCT, and beyond STRUCT
  and the histories;
- `f1`'s loadings;
- PS-1 and PS-2.

Every number is printed per fresh seed, and as the mean over the two.

## Stakes (graded on the mean of TEST seeds 3 and 4)

Here "F-fixed's increment" means the mean over seeds 3 and 4 of the mean over torch seeds
0, 1 and 2 of `R²(Z + q + F) − R²(Z + q)`. The same holds for D and SPIB-fixed. R0's value
is its mean over seeds 3 and 4.

- **G1 (plants).** F-fixed passes **all four plants**, as defined in §Plants:
  - **PS-1**: f1's R² on MOM **< 0.05** on seeds 3, 4 and CONTROL, for every torch seed.
    MOM's own σ₁ at 1 ps is **< 0.2** on each of those walks.
  - **PS-2**: the re-paired increment is **≤ 0.01** on seeds 3, 4 and CONTROL, for every
    torch seed.
  - **PS-3**: **≥ 0.9** on the synthetic at the selected λ, on **3/3 torch seeds**. These
    are the selection's own runs.
  - **PS-4**: the time-shuffled own σ₁ is **< 0.2** at every lag in {1, 2, 5, 10, 20} ps on
    shuffled seed 3 and on shuffled seed 4.
- **G2 (carries).** Both parts must hold:
  - **G2a**: F-fixed's increment is **≥ +0.02**;
  - **G2b**: it is **≥ D's increment**.
- **G3 (against the field's tool).** F-fixed's increment is **≥ SPIB-fixed's**. If
  SPIB-fixed fails any plant, it is not read, G3 is printed as "SPIB-fixed not read", and
  branch (a) cannot be declared.
- **G4 (against the raw regression).** F-fixed's increment is **≥ R0 − 0.005**: a slow
  variable that carries as much as the raw regression does, in 2 dimensions instead of 84.
- **Kill:** G1 fails, OR G2a (+0.02) is missed.
- **Reported beside every stake (G1–G4), with no stake:**
  - the per-seed values, and the minimum over (seed × torch seed);
  - whether each difference in G2b and G3 exceeds the two arms' combined min–max spread
    over torch seeds (if it does not, the result is stated as "tied within the spread");
  - CONTROL's increment, reported for the contrast. SLOW-2's C2 bar there was ≤ +0.005,
    and it is printed beside that bar, graded on nothing.

## Plants

Every plant runs on every learned arm (D, F-fixed, SPIB-fixed). Each names its carrier and
the sector it acts on, which is nonzero in that carrier by construction. The PS-3 numbers
are printed in the selection's output, before any fresh walk exists. PS-1, PS-2 and PS-4 are
printed at the read, ahead of every stake line.

- **PS-1** (carrier: seeds 3, 4 and CONTROL; sector: MOM). As in SLOW-2: MOM's own σ₁ at
  1 ps is **< 0.2**, and f1's R² on MOM is **< 0.05**.
- **PS-2** (carrier: seeds 3, 4 and CONTROL; sector: the arm's output paired to another
  molecule's future). The re-paired null uses `π(i) = (i + 125) mod 250`. The increment
  beyond velocity must be **≤ 0.01**.
- **PS-3** (carrier: SLOW-1's synthetic, 128 particles × 2,501 readouts; sector: the planted
  column, the OU log-mobility with a quarter-variance noise). The score must be **≥ 0.9**
  on `(f1, f1², f1³)` on **3/3 torch seeds**. This is stricter than SLOW-2, which ran
  torch seed 0 only.
- **PS-4** (carrier: shuffled seeds 3 and 4; sector: the arm's input, shuffled). Each
  molecule's whole record is permuted in time (numpy seed 11). The model trained on
  shuffled TRAIN (frozen, torch seed 0) is applied, and its own σ₁ must be **< 0.2** at
  every lag in {1, 2, 5, 10, 20} ps.
- An arm that fails a plant is **not read**: its numbers are printed and marked NOT READ.

## Branches

The lead's letters are kept, and there is no (d). The order of evaluation is (e), then (c),
then (a) or (b).

- **(e)** A plant fails on F-fixed. Nothing is read from F-fixed, and the integration kill
  fires. The failure is the finding.
- **(c)** G2a is missed: the carried structure beyond `q` is **below +0.02 on this operator
  at these lags, now on fresh data**. The kill fires.
- **(a)** G1–G4 are all met. **The integration works and beats the field's tool on fresh
  data.**
- **(b)** G1 and G2a are met, and at least one of G2b, G3 or G4 is not, or SPIB-fixed is not
  read. The result is stated with the clause that failed. For example, if G2b fails: "the
  plain VAMPnet carries as much; the integration adds nothing over it".
  - This clause is mine. The lead's (b) names G3 and G4. G2b is part of G2, and a G2b miss
    with +0.02 met is not (c)'s sentence, so it is read under (b) and named there.

## Order (the discipline, restated as a sequence)

1. The fresh walks are launched (done, 10:53).
2. This prereg is committed alone.
3. `slow3_train.py` is built, and the selection runs on the synthetic and TRAIN.
4. The chosen models are frozen, and the manifest is committed alone.
5. The fresh walks finish.
6. The lead starts the read.

If anything in step 3 cannot be built as written here, a dated note is appended to this file
under "Notes on building" and committed alone, before step 4.

## What this does not test

- **Water.** This is the rigid operator, oxygen-only, with 250 waters.
- **Experiment.** There is no experimental referee.
- **Softness or GNN propensity as a contest.** That is a different target (not the 5 ps increment).
- **Hyperparameter search beyond the declared grids.** A loss for F-fixed or SPIB-fixed at
  these settings is a loss at these settings.
- **Seed 2 and 400 K as tests.** Both have been seen. The 400 K walk is a contrast here,
  not a test.

## What could not be built as written (said before any number)

1. **"Cross-fitted on one half of the molecules/origins."** This is built as the product
   split (molecule half × EARLY/LATE origins with a 5 ps guard band), for the reason given
   in (i). Either split alone leaves a shared-window or shared-molecule path.
2. **"The same settle, same binary."** The binary could not be the same file, because it no
   longer exists. It was rebuilt from SLOW-1's source commit, with the seed table
   extended. The provenance check stands in for "same", and its result is reported
   whatever it is.
3. **"Largest λ … highest CV … ties to the smaller."** This is read as the filter, argmax
   and tie-break in the selection rule, step 3.
4. **PS-4 on the fresh seeds** needs a model trained on shuffled TRAIN. That model is
   trained and frozen before the read, like every other model, so the read trains nothing.

---
witness: none (a measured campaign: its gates are numeric, its closure algebra is the VAMP core cited in words; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-HOMOG, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS, M-MAX-OVER-SUCCESSES, M-ONE-MODEL-DELTA, M-BASE-RATE-OMITTED, M-PROVENANCE-OVERREACH, M-STALE-INSTRUMENT. These are the registered ids this text contacts.

- M-PLANT-OBS: every plant is re-derived for THIS reader and each arm. PS-3 runs before the fresh walks exist; the rest run ahead of every stake line.
- M-HOMOG: "neighbour shell" is a molecule's twelve nearest oxygens, not a claim about spatial locality on a graph family.
- M-PLACEMENT-LOTTERY: CPU work is pinned with `taskset` to cores 18–20 (one core class), and no wall clock is compared.
- M-DEVICE-CLASS: the nets are GPU artifacts. The torch-seed spread is their uncertainty, not bit identity.
- M-MAX-OVER-SUCCESSES: the selection's argmax is a choice among admissible candidates, never a bound on the others. Every candidate's row is printed.
- M-ONE-MODEL-DELTA: G3 earns "against SPIB at these settings" only. No Markov-state model is built.
- M-BASE-RATE-OMITTED: `q`'s own carry is printed beside every increment.
- M-PROVENANCE-OVERREACH: the binary's hash names bytes. "The same operator as SLOW-1" is claimed only if the byte-for-byte walk comparison says so.
- M-STALE-INSTRUMENT: the Python environment is session-keyed. The weights, the manifest and the instrument are committed, and the package versions are recorded.

Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier and sector in its own row. Each sector is nonzero in its carrier by construction: the planted column carries the OU log-mobility, and the re-paired and time-shuffled nulls act on the arm's own output and input, which are nonzero on every real walk.
