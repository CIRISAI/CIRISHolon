# SLOW-2 — the search-then-select instrument against the field's tools, and a learned dictionary integrated into it. PREREGISTRATION

*Frozen 2026-09-25, committed alone before `slow2_compare.py` exists and before any baseline
number (TICA, VAMP, VAMPnet, SPIB, or the integrated net) is computed. The stakes are the
lead's (2026-09-25), written here as given. Where the carrier could not hold a stake exactly as
worded, §"What could not be built as written" says what and why, and no stake's number moves.*

## Why

SLOW-1 (`SLOW1_RESULTS.md`) read a RE-FINDING. On the rigid operator's 50 ps walks (250
waters, three seeds at 293 K and one at 400 K), the linear search's slow structural sector at
τ* = 1 ps loads 0.82–0.88 on tetrahedral order `q`, with own σ₁ 0.30–0.35. Beyond velocity and
every other candidate it carries +0.028 R² of the next-5-ps displacement at 293 K and +0.0002
at 400 K. Two questions are left open, and this campaign asks both:

1. **Did the programme's search add anything over the published linear tools?** TICA
   (Pérez-Hernández, Paul, Giorgino, De Fabritiis & Noé, J. Chem. Phys. 139, 015102, 2013)
   and VAMP (Wu & Noé, J. Nonlinear Sci. 30, 23, 2020), run from `deeptime` (Hoffmann,
   Scherpf, Kawai et al., Mach. Learn.: Sci. Technol. 3, 015009, 2022) on the same six
   columns.
2. **Does a learned dictionary find structure that `q` does not already carry?** The
   VAMPnet (Mardt, Pasquali, Wu & Noé, Nat. Commun. 9, 5, 2018) and SPIB (Wang & Tiwary,
   J. Chem. Phys. 154, 134111, 2021) are run on a richer per-molecule input. Beside them runs
   our integration: a VAMPnet trained jointly on closure and on the carried target, with
   SLOW-1's plants run on its output.

**What is already public, said before the read.** SLOW-1 has already read the held-out seed
(T293_seed2) with arm A's algorithm, fitted on that seed itself: q loading `0.880`, increment
beyond velocity and `q` `+0.0085`, and `q`'s own carry beyond velocity `+0.028`. Arm A here
is a re-fit on seeds 0 and 1, so its number on seed 2 is not blind. Arm C (deeptime's VAMP)
is the same algebra as arm A: the same covariances, and a different regulariser (ε
truncation rather than the `10⁻³` relative ridge). TICA differs by symmetrising the lagged
covariance. C1 is therefore close to determined by SLOW-1, and it is staked anyway because
the lead asked for it stated. The blind arms are D, E and F.

## Carrier and inputs (every arm, the same)

- **Walks** (read-only): `replace0/slow1/T293_seed{0,1,2}` and `T400_seed0` in the main
  checkout. The rigid operator, 250 waters, 2,501 readouts at 20 fs = 50 ps, oxygen positions
  and velocities. Loaded with `slow1_search.load`, imported, not copied.
- **Split.** TRAIN = T293 seeds 0 and 1 (500 molecule-trajectories). HELD-OUT = T293 seed 2
  (the stake seed). CONTROL = T400 seed 0. Every arm is fitted on TRAIN only, frozen, and
  applied unchanged to HELD-OUT and CONTROL. This includes the input standardisation, which
  uses TRAIN's mean and standard deviation. The nets use TRAIN molecules `i mod 5 ≠ 0` to
  fit and `i mod 5 = 0` as a validation set that is logged and never used to select a
  number.
- **STRUCT (six columns, arms A, B, C):** `q, n33, n50, nb, s2, s2w`, computed by
  `slow1_search.dictionary` exactly as SLOW-1 did. That includes `s2w`'s ξ, fitted on each
  walk's own pooled g(r). At 400 K ξ = ∞, so s2w equals s2 there, and STRUCT has rank 5.
  This is reported and left as it is.
- **SHELL (the richer input, arms D, E, F): 84 columns per molecule per readout.** The input
  is the molecule's twelve nearest oxygens under the minimum image, ordered by distance, as
  a permutation-invariant set:
  - the 12 distances;
  - the 66 cosines `cos ψ_jk` between the unit vectors to neighbours j < k, in distance order;
  - the six STRUCT columns.

  Distance ordering makes the vector invariant to the neighbours' labels. `q` is an exact
  function of the six cosines among the first four neighbours, so a net can re-find it. The
  STRUCT columns are appended so the richer input is a strict superset of the linear arms'
  input, and nothing is lost by moving to it. `n50`, `s2` and `s2w` reach beyond twelve
  neighbours and enter only through the appended columns.
- **Lag.** τ* = **1 ps** (50 readouts) for every arm. This is SLOW-1's τ* on all four walks,
  fixed here and not re-chosen per arm. The nets train at this lag.
- **Output dimension 2 for every arm.** For A, B and C these are the top two
  components: singular functions, TICs, or VAMP singular functions. D and F have a 2-unit
  output lobe. E has a 2-dimensional latent mean. **The leading output `f1`** of any arm is
  the top singular function of the linear VAMP (`reason_search0b.vamp`, the SLOW-1 core) on
  the arm's 2-D output at τ*, fitted on TRAIN. One definition covers every arm, including
  those whose outputs are not ordered (SPIB).

## The arms

- **A. SLOW-1's linear search** (the reference): `reason_search0b.vamp` on STRUCT, pooled
  over TRAIN's transitions at τ*, standardised globally and not centred per molecule, ridge
  `10⁻³`. The output is `Φ_STRUCT · W₀ U[:, :2]`.
- **B. TICA**: `deeptime.decomposition.TICA(lagtime=50, dim=2)` fitted on TRAIN's 500
  trajectories of STRUCT (standardised by TRAIN).
- **C. VAMP**: `deeptime.decomposition.VAMP(lagtime=50, dim=2)`, the same inputs as B.
- **D. VAMPnet**: `deeptime.decomposition.deep.VAMPNet`, VAMP-2 score, `score_mode =
  'regularize'`, ε = 10⁻⁶. The lobe is an MLP 84 → 128 → 128 → 2 with ReLU, shared between
  the instantaneous and lagged sides. Adam, learning rate 5 × 10⁻⁴, batch 2,048,
  **20 epochs, fixed, no early stopping**: the final weights are used.
- **E. SPIB**: the authors' package `spib` 1.1.0 (PyPI, Dedi Wang's code). It installs, so
  it is used rather than a re-implementation. Nonlinear encoder, 2 hidden layers of 128,
  z_dim 2, lagtime 50, **β = 0.01**, learning rate 10⁻³, batch 2,048. Initial labels come
  from k-means with **K = 10** (`deeptime.clustering.KMeans`, seeded) on TRAIN's
  standardised SHELL. Label refinement uses the package's own rule (refinements 8, patience
  2, tolerance 10⁻³). Its input normalisation is the package's `DataNormalize` with TRAIN's
  moments. **Output: the latent mean `z_mean`** (2-D). The target it predicts is the state
  label at t + 1 ps.
- **F. OURS, INTEGRATED**: the lobe and optimiser of D and the same batches. The loss is
  `−[VAMP2(batch) + λ · ΔR²(batch)]`, with **λ = 10 declared here**. `ΔR²` is the held-out
  carried increment within the batch: `R²(y | Z, f) − R²(y | Z)`, where f is the lobe's
  2-D output at t, `Z = (vx, vy, vz, |v|²)` at t, and `y = |r(t + 5 ps) − r(t)|`. It is
  held out by molecule. The ridge (`10⁻³`, standardised, closed form, differentiable) is
  fitted on the batch rows of even molecule index and scored on the odd rows, then the
  reverse, and the two are averaged. The VAMP-2 term is deeptime's `vamp_score` on the same
  batch. The objective targets velocity only, as the lead worded it; "beyond q" is not in
  F's loss. Reported beside it, no stake: **λ = 0** (D's objective through F's training code, a check
  that the two training codes agree), **λ = 1**, and **λ = 100**.
- **Training rows (D, E, F, the same):** time origins `t ∈ [0, T − 250)` at a stride of 5
  readouts (100 fs), so every row has a y. That gives about 196,000 fitting rows and 49,000
  validation rows.
- **Seeds.** Each of D, E and F is trained with **torch/numpy seeds 0, 1, 2**. An arm's
  value is the **mean over the three**, and the **min–max spread** is reported beside it.
  A and B are deterministic. C is deterministic.
- **Device.** The nets train on the **GPU**, an NVIDIA GeForce RTX 4090 Laptop
  (**cuda**), with `torch.use_deterministic_algorithms(True)` and
  `CUBLAS_WORKSPACE_CONFIG=:4096:8`. The numbers are this device class's artifact. The same
  seeds on a CPU are not claimed to reproduce them bitwise (M-DEVICE-CLASS). The spread over
  training seeds is the reported uncertainty. CPU work (dictionaries, linear arms, the
  regressions) is pinned with `taskset -c 16-20`, with one thread per process. Cores 0–15
  and 21–27 run other campaigns and are not touched. No wall clock is compared across core
  classes.
- **Reference R0 (no stake):** the 84 SHELL columns entered directly into the S2 ridge, with
  no dynamics. This asks whether a linear readout of the richer input already carries what
  a net might.

## What is measured, per arm, on HELD-OUT and on CONTROL

The S2 machinery is SLOW-1's. The target is `y(t) = |r(t + 5 ps) − r(t)|` on rows
`t ∈ [10 ps, T − 5 ps)`, the same rows as SLOW-1. The baseline is `Z = (vx, vy, vz, |v|²)`.
The regression is ridge `10⁻³`, held out by molecule in four folds (`i mod 4`) **within the
evaluation walk**, using `slow1_search.ridge_r2`. F denotes the arm's 2-D output.

- **own σ₁** (closure): the top singular value of `reason_search0b.vamp` on the arm's 2-D
  output at τ* on the evaluation walk. Reported beside it: the same at 2, 5, 10 and 20 ps.
- **carry beyond velocity:** `R²(Z + F) − R²(Z)`.
- **carry beyond velocity AND q — THE STATISTIC THAT MATTERS NOW:** `R²(Z + q + F) − R²(Z + q)`.
- Reported beside it, with no stake:
  - beyond velocity and all six STRUCT columns;
  - beyond velocity, STRUCT and the four cage-escape histories `h1, h2, h5, h10`;
  - the same statistics with `f1` in place of F;
  - **`q`'s own carry beyond velocity, `R²(Z + q) − R²(Z)`**. This is the base rate every
    increment sits beside (M-BASE-RATE-OMITTED).
- **Loadings of f1** (`slow1_search.r2_fit`, R² of f1 regressed on the columns, over the
  S2 rows) on:
  - **`q`** (the re-finding question);
  - `s2`, `s2w`, density (`n33, n50`), bond count `nb`, and each of `h1, h2, h5, h10`;
  - all six STRUCT columns jointly (how much of f1 is linear in SLOW-1's dictionary);
  - MOM (`vx, vy, vz, |v|²`).
- **The linear arms retrained on CONTROL itself** (A, B and C fitted on T400): reported,
  no stake. C2's 400 K bar is read on the frozen arms.

## Stakes

- **C1 (the baselines re-find).** On HELD-OUT, B's and C's `f1` each load **≥ 0.8** on `q`.
  If both do, the result is stated as: **"SLOW-1's search added nothing over TICA on this
  problem."** Each arm is read separately if only one does. Reported beside it: A's loading,
  and the |cosine| between A's, B's and C's leading directions in STRUCT's standardised
  coordinates.
- **C2 (the learned dictionary goes beyond q).** The best of D, E and F, by its mean over
  three training seeds of the carry beyond velocity and `q` on HELD-OUT, must meet all three:
  - it is **≥ +0.02** R² on HELD-OUT;
  - that arm's PS-2 is **≤ 0.01** on every training seed;
  - the same arm's carry beyond velocity and `q` at CONTROL is **≤ +0.005** (mean).

  "Best of" is read as an existence claim: at least one learned arm reaches the bar. It is
  not a bound on the others (M-MAX-OVER-SUCCESSES), and every arm is reported. If the mean
  meets the bar and the minimum over training seeds does not, that is stated as "met on the
  mean, not on every training seed".
- **C3 (ours against the field's).** F's mean carry beyond velocity and `q` on HELD-OUT is
  **≥ E's (SPIB's)** mean. Reported beside it: whether the difference exceeds the two arms'
  combined min–max spread over training seeds. If it does not, the result is stated as
  "tied within the training-seed spread" beside the verdict by the letter. What C3 can earn
  is "better or worse than SPIB at these settings on this carried test", nothing wider
  (M-ONE-MODEL-DELTA).
- **Kill for the integration claim:** F's mean is below E's on HELD-OUT, OR F fails any
  plant (PS-1, PS-2, PS-3, PS-4) on any training seed.

## Plants

Every plant runs on every arm, A through F, and each names its carrier and the sector it acts
on, which is nonzero in that carrier by construction. The plants are run on this reader, and
their numbers are printed before any stake line.

- **PS-1** (carrier: every real walk; sector: MOM). MOM's own σ₁ at 1 ps is **< 0.2**, as
  frozen in SLOW-1, which is a property of the walk. And for each arm, **f1's R² on MOM is
  < 0.05** on HELD-OUT and CONTROL: the arm's output is not the momentum.
- **PS-2** (carrier: every real walk; sector: the arm's output paired to another molecule's
  future). The re-paired null takes the arm's output (and `q`) from molecule
  `π(i) = (i + 125) mod 250`, a fixed derangement, and takes Z and y from molecule i. The
  increment beyond velocity must be **≤ 0.01** on HELD-OUT and CONTROL.
- **PS-3** (carrier: SLOW-1's synthetic walk, `slow1_search.synthetic()`, 128 particles ×
  2,501 readouts at 20 fs; sector: the planted column). The per-particle 5 ps
  Ornstein–Uhlenbeck log-mobility, with noise of a quarter of its variance, is appended as
  the seventh STRUCT column (arms A, B, C) and the 85th SHELL column (D, E, F). SHELL and
  STRUCT are computed from the synthetic positions. Each arm is fitted on particles
  `i mod 4 ≠ 0` (nets: training seed 0 only) and read on `i mod 4 = 0`. PASS: the
  planted column regressed on `(f1, f1², f1³)` gives **R² ≥ 0.9**. The cubic admits a
  net's monotone reshaping of the variable it found. For a linear arm it is at least the
  linear R², the SLOW-1 bar. The linear R² is reported beside it.
- **PS-4** (carrier: every real walk; sector: the arm's input, shuffled). Each molecule's
  whole record (input, velocity and position) is permuted in time (`numpy` seed 11, as in
  SLOW-1). The linear arms are refitted, and the nets retrained (training seed 0), on the
  shuffled TRAIN, then applied to the shuffled HELD-OUT. PASS: own σ₁ is **< 0.2** at every
  lag in {1, 2, 5, 10, 20} ps. A failure names per-molecule offsets as the source of that
  arm's closure.
- An arm that fails a plant is **not read**: its stake numbers are printed and marked
  NOT READ. If the arm is F, the integration kill fires.

## Branches

- **(a)** F itself meets C2's three bars, and C3 is met. **The integrated instrument finds
  something `q` does not carry, and beats SPIB on the carried test.**
- **(b)** C2 is met by D or E but not by F, or F meets C2 while C3 fails. **The field's tool
  wins, stated.**
- **(c)** No learned arm meets C2. **Tetrahedral order is all the structure carries on this
  operator at these lags,** as far as these learners, inputs and this carried target can
  see.
- **(e)** A plant fails on the arm a branch would read. Nothing is read from that arm, and
  the failure is the finding about that arm.

The lead's letters are kept. There is no (d).

## What this does not test

- **Water.** The liquid is the rigid operator's (oxygen-only walk, 250 waters). A sentence
  about real supercooled water owes its own carrier.
- **Experiment.** No experimental referee.
- **Softness or GNN propensity prediction as a contest.** The target is SLOW-1's carried
  increment at 5 ps beyond velocity and `q`, from a 2-D output. The field's propensity
  correlations (about 0.9 on 2D Lennard-Jones with GNNs or weighted pair entropy) answer a
  different question, and a low number here is not a verdict on them.
- **Hyperparameter search.** Each learner runs at one declared setting (F at λ = 10 for the
  stake). A loss for D, E or F at these settings is a loss at these settings.

## What could not be built as written (said before the read)

1. **"Angles to its 12 nearest neighbours"** is built as the 66 pairwise cosines among the
   twelve distance-ordered neighbours. The alternative, angles to a molecular frame, needs
   hydrogen positions that the oxygen-only walk does not carry.
2. **SPIB** was not implemented from the paper, because the authors' package installs, and
   using theirs is the more faithful baseline. Its convergence rule is the package's own,
   with the declared refinements, patience and tolerance.
3. **"The held-out carried increment" inside F's loss** is held out by molecule within each
   batch (even and odd indices). It cannot be held out by seed during training without
   putting HELD-OUT into the loss. HELD-OUT is never seen by any arm's fit.
4. **The 400 K control** is read on arms frozen from 293 K training, not refitted, because
   the control asks whether the same instrument carries anything where heterogeneity has
   faded. The linear arms refitted at 400 K are reported beside it.

---
witness: none (a measured campaign: its gates are numeric, and its closure algebra is the VAMP core cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-HOMOG, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS, M-MAX-OVER-SUCCESSES, M-ONE-MODEL-DELTA, M-BASE-RATE-OMITTED. These are the registered ids this text contacts, cited at the audit's demand or by their shape.
- M-PLANT-OBS: every plant is re-derived for THIS reader and each arm, and run before the read.
- M-HOMOG: "local" here means a molecule's neighbour shell, not a spatial-locality claim about a graph family.
- M-PLACEMENT-LOTTERY: the CPU work is pinned with `taskset` to one core class, and no wall clock is compared across classes.
- M-DEVICE-CLASS: the nets are GPU artifacts, and the spread over training seeds is their uncertainty, not bit identity.
- M-MAX-OVER-SUCCESSES: "best of D, E, F" is an existence claim, never a bound.
- M-ONE-MODEL-DELTA: C3 earns "against SPIB at these settings" only. No Markov-state model is built; SPIB's labels are its own.
- M-BASE-RATE-OMITTED: `q`'s own carry is printed beside every increment.

Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant (the planted column carries the OU log-mobility by construction; the re-paired and time-shuffled nulls act on the arm's own output and input, which are nonzero on every real walk).

### Notes on building (2026-09-25; appended after the freeze and BEFORE the read of any real walk; no stake, bar, λ or plant moves)

These were found building `slow2_compare.py` and running its plants on the PS-3 carrier (the
synthetic), which the plants section says runs first. No arm had been read on HELD-OUT or
CONTROL when this note was written. A code-path run on a 40-molecule subset was checked for
crashes only, and its numbers were not used.

1. **SPIB's own convergence rule never fires at these settings.** The package refines when the
   epoch's training-loss change stays under `tolerance = 10⁻³` for `patience + 1` epochs. On
   the synthetic, the change stayed at 3–5 × 10⁻³ for more than 2,000 epochs, with no
   refinement, and the package has no epoch cap. **Built:** the package's own `fit`, with
   `tolerance = ∞` and `patience = 4`, which makes its rule a fixed schedule: a refinement
   every 5 epochs, 8 refinements, 45 epochs. The package code is unchanged. The prereg's
   "patience 2, tolerance 10⁻³" could not terminate, and this is the bounded reading of the
   same rule.
2. **SPIB can collapse to one state, and the package then raises** ("Only one metastable
   state is found!"). **Built:** the error is caught, the collapse is recorded (refinements
   completed, epochs), and the encoder's `z_mean` as trained at that moment is read as E's
   output. A collapsed SPIB is still read, and the collapse is printed beside its numbers.
3. **SPIB's input normalisation** uses TRAIN's moments, the same `Std` object as the other
   nets, as the prereg says. The first draft used the fitting molecules' moments.
4. **PS-3 dry run on the synthetic, all arms at the frozen settings (training seed 0):**

   | arm | PS-3 score | result | note |
   |---|---|---|---|
   | A, B, C | 0.9999 | PASS | |
   | D (VAMPnet via deeptime) | 0.935 | PASS | |
   | E (SPIB) | 0.254 | **FAIL** | collapsed to one state after 2 refinements, 15 epochs; k-means on the 85 standardised columns does not split on the one planted column, and SPIB merges the fast labels |
   | **F (λ = 10)** | **0.684** | **FAIL** | |

   **Diagnostics for F, no stake:**

   | F run | PS-3 score |
   |---|---|
   | λ = 0 through F's training code | 0.935, identical to D's deeptime `partial_fit` |
   | λ = 1 | 0.920 |
   | λ = 10, trained 90 epochs (the step count the real training reaches, about 1,900) | 0.860 |
   | λ = 100 | 0.595 |

   The λ = 0 row confirms the two training codes agree. At λ ≥ 10 the carried term pulls
   the output away from the planted slow column, toward a predictor of the next-5-ps
   displacement that is not the slow variable. Training as long as the real run does not
   rescue it.
   **Consequence, stated before the read:** by the frozen rule, F fails PS-3, so the
   **integration kill fires**, and E fails PS-3, so **E is not read**. The real run
   repeats these plants with the same seeds. The stake numbers of E and F are still
   computed and printed as NOT READ. λ = 1 is reported, as declared, and is not promoted
   to the stake.
5. **"Own σ₁" of a 2-D output** is at most the VAMP of a 2-column dictionary, so it is
   bounded by that dimension and not by STRUCT's six. SLOW-1's STRUCT σ₁ is the six-column
   value. The two are compared as closure of the arm's output, which is what the lead's
   "own σ₁ at τ*" asks.
