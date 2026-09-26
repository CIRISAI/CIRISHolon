# ORDER-1 — the order parameter, hunted blind: is there a non-conserved structural mode that the hydrodynamics of this liquid CARRIES? PREREGISTRATION

*Frozen 2026-09-25, committed alone, AFTER `ORDER1_PRIOR_ART.md` (`0880b6c`) and BEFORE any
ORDER-1 quantity is computed on the walks under read. The lead's question: on every tier the
search's carried directions were the conserved quantities, except two "weak ones" — a
non-conserved variable the level above carries anyway (tetrahedral order on the sluggish liquid,
`SLOW1_RESULTS.md`; the conscience sector on reasoning chains). The physical analogue is the ORDER
PARAMETER. ORDER-1 hunts it on walks already on disk; no new dynamics is run. Under the
CORRECTION of 2026-09-25 in `OBJECT.md` a tier is a Closed view the tier above CARRIES, so the
question has two numbers, closed (O1) and carried (O2), and a grade against the literature (O3).*

**What has been looked at before this freeze, stated so it cannot be hidden later.** (i) SLOW-1
already read these walks per molecule: `q`'s structural sector closes at `σ₁ = 0.30–0.35` at 1 ps
on the three 293 K seeds and `0.068` at 400 K (`SLOW1_RESULTS.md`). (ii) The reader
`order1_search.py` was built before this freeze and exercised on two carriers that are NOT the
data under read: a synthetic carrier (a damped 2 ps density oscillator, a 100 ps offset, 0.3 ps
currents) and the three 128-water rigid transport walks at time scale `×0.1` (6 ps each; NOT A
READING). What building found is in §6. No ORDER-1 quantity (`Q_k`, its closure, its increment)
has been computed on any `slow1/` walk.

## 1. Data and the two arms

**Carrier.** `replace0/slow1/T293_seed{0,1,2}` and `T400_seed0` in the main checkout (read-only):
the rigid operator, 250 waters, box 36.992 bohr = 19.575 Å, 2,501 readouts at 20 fs = 50 ps;
`rigid.walk` (oxygen positions, unwrapped, bohr) and `rigid.vwalk` (oxygen velocities, au).
First-harmonic wavevector `k = 2π/L = 0.321 Å⁻¹`. Production temperatures on the rigid modes
`~304` K (293 K arms) and `~410` K (400 K arm), as SLOW-1 recorded.

**Arm D (the collective dipole) is REFUSED by name.** The walks carry `3 × 250` numbers per
readout — oxygen positions only (header `# walk 2501 L`, written by `write_flexible_walk` in
`replace0.rs`, which dumps the oxygen frames). No hydrogen position and no orientation quaternion
is on disk, so the molecular dipole, `M`, and its modes cannot be formed. Arm D's O1–O3 are not
read; the dielectric literature is recorded in the prior art for the arm that would.

**Arm S (the structural mode)** runs.

## 2. The instrument

**Features per readout.** `H` = VIEW-SEARCH-1's first-harmonic Fourier modes, built by
`view_search.fourier_features` (24 columns: `ρ_k` cos/sin and `j_k` x, y, z cos/sin along each
axis); `Q` = `Σ_i (q_i − ⟨q⟩) cos/sin(k x_{i,a})`, `a = x, y, z` (6 columns), `q_i` SLOW-1's
Errington–Debenedetti order from the four nearest oxygens (`slow1_search.structure`), `⟨q⟩` the
walk's mean. Subtracting the constant `⟨q⟩` makes `Q_k` the lead's `Σ q_i e^{ik·r_i}` minus
`⟨q⟩ρ_k`: the trivial image of the density is removed before the search, not by it. Reported
beside: `K` = the same sum of `|v_i|² − ⟨v²⟩` (the only energy-density proxy the walk has; the
heat mode is the named confound of the prior art, §1), and `Q₀` = the box-mean `q`. Every
feature is centred per walk (VIEW-SEARCH-1 Amendment 1, item 5; the density modes' static
offsets are printed as what was removed).

**The primary form: one chain per k-vector.** Modes at different wavevectors are uncorrelated
by translation invariance, and a quarter-wavelength translation maps `(cos, sin) → (sin, −cos)`.
So each first-harmonic k-vector is one chain of `[ρ_c, ρ_s, j^L_c, j^L_s, j^T1_c, j^T1_s,
j^T2_c, j^T2_s, Q_c, Q_s]`, taken twice (as is and quarter-rotated): six chains per walk,
pooled as SLOW-1 pooled molecules. This is the generalized-collective-modes layout (one `k`,
its dynamical variables). The lead's literal dictionary — all 30 columns in one vector, as
VIEW-SEARCH-1 built it — is read beside it and REPORTED, not graded: with 6 independent
k-vector samples per readout it fits cross-k couplings that symmetry forbids (building, §6).

**Closure.** VAMP as in SLOW-1 (`reason_search0b.vamp`: standardised, ridge `10⁻³`), pairs
pooled over chains. Lags `0.1, 0.5, 1, 2, 5, 10, 20` ps.

**Carried.** Ridge regression (standardised, `λ = 10⁻³ N`) of the next longitudinal state
`(ρ_k, j^L_k)(t + τ)` from `(ρ_k, j^L_k)(t)` against `(ρ_k, j^L_k, Q_k)(t)`, pooled over chains;
held out by blocked 5-fold cross-validation in time within each walk (training pairs within `τ`
of the test block dropped); `R²` pooled over the four standardised target columns. `τ = 1` and
`5` ps. The target is the LONGITUDINAL sector because a scalar field couples at linear order only
to `ρ_k` and `j^L_k` (isotropy); the lead's wording named `ρ_k, j_k`, and the transverse currents
enter the reported literal-form increment. Within-walk blocking is the primary form because the
400 K arm has one seed; the seed-held-out increment (fit on two 293 K seeds, tested on the third)
is reported beside it.

## 3. Stakes, each with its kill

- **O1 — closed.** On EVERY 293 K seed, at some lag `≥ 1` ps: `Q_k`'s own `σ₁ ≥ 0.3`, and the
  residual of `Q_k` after its equal-time least-squares projection on its own k-vector's
  hydrodynamic sector (`ρ, j^L, j^T`) keeps `σ₁(residual)/σ₁(Q) ≥ 0.7`. **Kill:** on every 293 K
  seed either `σ₁ < 0.1` at every lag `≥ 1` ps, or the ratio `< 0.3` at every lag `≥ 1` ps
  ("fully in the hydrodynamic span"). Between otherwise.
- **O2 — carried (the stake that matters).** At one `τ ∈ {1, 5}` ps, the `(ρ, j^L)` increment
  from adding `Q_k` is `≥ 0.05` in `R²` on EVERY 293 K seed, and the 400 K increment at that `τ`
  is below the 293 K mean; with PO-2's nulls `≤ 0.01`. **Kill:** the 293 K mean increment
  `< 0.02` at both `τ`. Between otherwise.
- **O3 — re-finding or direction.** On each 293 K seed: the normalised autocorrelation of `Q_k`
  (summed over the three k-vectors) is fitted on `[0.5, 20]` ps by one exponential and by two;
  **TWO TIMES** iff the two-exponential fit halves the residual sum of squares, both amplitudes
  are `≥ 0.1` of the autocorrelation at 0.5 ps, and the times differ by `≥ 3×`. **MIXTURE** iff
  the Q-dominant closed direction of the joint chain dictionary `(ρ, j^L, j^T, Q)` — the first
  singular function with `R² ≥ 0.5` on the `Q` block — at the lag `≥ 1` ps where `Q`'s `σ₁` is
  largest has `R² ≥ 0.2` on the density block. **RE-FINDING** (branch b) iff one time and not a
  mixture: generalized hydrodynamics' relaxing structural variable (Mountain; GCM; the prior art
  §1–2), its time reported beside the single-molecule `q` memory (the same fit on the
  per-molecule `q` autocorrelation) and the α time (`F_s(k = 2.2 Å⁻¹, t) = 1/e`). **DIRECTION**
  (branch a) iff two times or a mixture — and then the two-state literature (prior art §3) is
  what it must be compared against before any claim.

## 4. Plants, each with its carrier and the sector it acts on

| plant | carrier | must |
|---|---|---|
| **PO-1** the regression test | the three 128-water rigid transport walks VIEW-SEARCH-1 read (main checkout `replace0/transport_seed{0,1,2}/rigid.walk`, dated 2026-09-17), loaded by ORDER-1's own loader in Å | the hydrodynamic sector as ORDER-1 builds it reproduces VIEW-SEARCH-1's banked held-out reads at 100 fs to `1.5 × 10⁻³`: Fourier-all `13.238` / bound `12.163`, Fourier-density `4.945` / `5.648`, cells `11.767` / `15.119` (`view_search_transport_rigid.txt`) |
| **PO-2** re-paired nulls | each 293 K walk | the `(ρ, j^L)` increment with `Q_k` replaced by (i) its own walk's `Q_k` circularly shifted by half the walk, (ii) the next 293 K seed's `Q_k`: both `≤ 0.01` at both `τ` |
| **PO-3** time-shuffle | every walk | `Q_k` with its readouts permuted (the same permutation on every chain): `σ₁ < 0.1` at every lag `≥ 1` ps |
| **PO-4** planted slow field | `T293_seed0`'s own hydrodynamic sector — the plant acts on the density sector `ρ_k`, which is nonzero in that carrier by construction (it is the carrier's own mode) | a complex Ornstein–Uhlenbeck field `Z_k` per k-vector (5 ps memory) added to `ρ_k` at `0.7 ×` its sd with a 1 ps latency, and a planted `Q_k = Z_k + noise` (5 % of its variance) in place of the real one: O1's legs met at a lag `≥ 1` ps; the top closed direction of the planted `Q_k` loads `R² ≥ 0.9` on `Z_k`; the `(ρ, j^L)` increment `≥ 0.05` at one `τ`; the shifted null `≤ 0.01` |
| **PO-5** the 400 K contrast | the 400 K walk against the 293 K walks | `Q_k`'s own `σ₁` at 1 ps is lower at 400 K than on every 293 K seed — the collective field must show the temperature contrast SLOW-1 read per molecule, or it is not tracking `q` |

Every plant must fire before any stake is read; a failed plant is branch (e).

## 5. Branches

- **(a) DIRECTION** — O1 and O2 met, O3 DIRECTION on every 293 K seed: a closed, carried,
  non-conserved structural mode with two times or a density mixture; compared with the two-state
  literature before anything else is said.
- **(b) RE-FINDING** — O1 and O2 met, O3 RE-FINDING on every 293 K seed: extended hydrodynamics'
  relaxing structural variable, re-found by the search, carried.
- **(c) O1 killed** — no structural mode beyond hydrodynamics at these `k`.
- **(d) O2 killed** — closed, not carried: the network's shape again, one tier up.
- **(e) a plant fails** — nothing is read.
- Anything else is reported as "none cleanly" with every stake's status, as SLOW-1 did.

## 6. What building found (before this freeze; not the data under read)

1. **The literal 30-column dictionary over-fits cross-k couplings.** On the 128-water walks (at
   `×0.1`), `Q`'s `σ₁` read `0.6` in the literal form where the per-k-vector chains read
   `0.05–0.2`, and the literal held-out increment read `+0.16` where the chains read `≤ 0`: six
   independent k-vector samples per readout do not support a 24-to-24 regression. On the
   synthetic carrier the literal blocked cross-validation read `R² ≈ −0.35` on pure-noise
   currents. Hence the chain form is primary and the literal form reported.
2. **The plant's amplitude was set on the synthetic carrier.** At `1.0 ×` the density sd the
   planted field is visible enough in `ρ_k` at equal time that O1's residual ratio falls to
   `0.48` (O1's own ratio leg penalises a field that is carried strongly — named here as a
   tension in the stake, left as written); at `0.7 ×` all PO-4 legs pass there (ratio `≥ 0.7`,
   loading `0.93`, increment `+0.12` at 1 ps, null `≤ 0`). The planted memory, 5 ps on a 50 ps
   walk, is resolved only to about `±0.2` in its autocorrelation at 5 ps.
3. **The resolution floor.** A collective mode has six chain samples per readout; a 50 ps walk
   holds `~10` independent intervals of a 5 ps process. Stakes at lags of 10–20 ps read noise
   of order `0.1–0.2` in `σ₁`; PO-3 measures the shuffled floor and it is printed.

## 7. Reported, not graded

The literal 30-column reads; the seed-held-out increments; `Q`'s increment beyond `(ρ, j^L) + K`
and `K`'s own increment (the heat-mode confound); `K`'s and `Q₀`'s own `σ₁`; the equal-time
correlation of `ρ_k` and `Q_k` (the two-state sign is negative; the prior art expects it weak,
Sedlmeier–Horinek–Netz 2011); the joint dictionary's top singular values; KWW fits.

## 8. Cost and placement

Pure reading: `q` for 250 molecules × 2,501 readouts × 4 walks, VAMP on ten-column chains, ridge
regressions. Minutes on the five cores `taskset -c 16-20` (cores 0–15 and 21–27 belong to other
campaigns); one core class, no price compared across classes.

---
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-PLACEMENT-LOTTERY, M-FLOOR-UNSTAKED — the registered ids this text contacts. M-PLANT-OBS: every plant is re-derived for THIS reader and run on it before the read (PO-1 on the reader's own loader). M-PLANT-SECTOR: PO-4 acts on `ρ_k`, the carrier's own density mode. M-PLACEMENT-LOTTERY: all reads on cores 16–20, one class. M-FLOOR-UNSTAKED: the resolution floor at long lags is named (§6.3) and PO-3 measures it.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
