# VIEW-SEARCH-1 — the closed coarse view as a SEARCH, scored against its variational bound: PREREGISTRATION

*Written 2026-09-20, committed alone, BEFORE the reader exists and before any banked walk is
scored under it. On the owner's question of 2026-09-19: the approximate closure search at
fixed dimension is hard in general, but for LINEAR views in a feature dictionary its optimum
is the top singular subspace of the transfer operator (the variational principle for Markov
processes — Wu & Noé 2020; TICA and Markov state models before it) and any named view can be
scored as a fraction of that bound. This adopts that instrument; it invents nothing. Data:
the banked equilibrium walks (positions and velocities of every oxygen at 20 fs) — 128 waters,
three seeds, 6 ps each, both operators (`transport_seed*`); 432 waters, three seeds, 5 ps, the
rigid operator (`scout432_seed*`). No new dynamics is run.*

## 1. The instrument

**Features.** From each frame: cell fields on a grid `g` (occupancy and the three momentum
components per cell; the total occupancy is dropped as constant) and Fourier modes at the
first harmonic along each axis (`cos, sin` of `k·x` for the density; the same weights on each
velocity component for the current: `8` per axis, `24` in all). The **dictionary** is the
union of the `2×2×2` cell fields (`31`) and the first-harmonic modes (`24`).

**Score.** For features `X_t` and lag `τ`, mean-centred: `C₀₀, C₀τ, Cττ`; the half-whitened
Koopman matrix `K = C₀₀^{−½} C₀τ Cττ^{−½}` (ridge `10⁻⁶ · tr C₀₀ / dim` on the covariances);
its singular values `σ₁ ≥ σ₂ ≥ …`. **The VAMP-2 score of a `k`-dimensional view is the sum of
the squares of ITS OWN top `k` singular values; the bound at dimension `k` is the same sum
for the full dictionary.** A view's **fraction** is score / bound. Its **closure defect** in
the program's sense is the relative residual of the least-squares law, `D² = 1 − Σσᵢ²/k` on
the view's whitened coordinates — the same quantity the fluid instrument prints, now with
the law fitted rather than named. **Held out:** the covariances are estimated on two seeds
and the score evaluated on the third, rotated over the three; the graded number is the
held-out mean, the spread its uncertainty. In-sample scores are printed beside it.

**Timescales.** `tᵢ = −τ / ln σᵢ` for the dictionary's leading singular values, at each lag.

**Lags.** `τ ∈ {20, 40, 100, 200, 400}` fs.

## 2. The views scored

| view | dimension `k` |
|---|---|
| cells `2×2×2`, occupancy + momentum | 31 |
| cells `2×2×2`, occupancy only | 7 |
| Fourier first harmonic, all | 24 |
| Fourier density modes only | 6 |
| Fourier transverse current (components ⟂ `k`) | 12 |
| Fourier longitudinal current (component ∥ `k`) | 6 |
| the position-blind relabelling of the `2×2×2` cells (the placebo) | 31 |

## 3. Stakes, each with its kill

- **S1 — the search finds the long-wavelength modes on its own.** At `τ = 100` fs, the
  dictionary's top-6 singular functions have weight `≥ 0.7` in the first-harmonic Fourier
  subspace (the squared norm of their projection, whitened). **Kill:** weight `< 0.5` — the
  slowest closed directions of a 128-water box are not its longest wavelengths, and the
  program's hand-named charts were named on a wrong premise.
- **S2 — the slowest mode is a shear mode with water's viscosity.** The dictionary's leading
  timescale at `τ = 100` fs belongs to a singular function with weight `≥ 0.5` in the
  transverse-current subspace, and `η = ρ / (k² t₁)` lies in `[0.2, 3.0] × 10⁻³ Pa s` on at
  least two of three seeds. **Kill:** the leading mode is not transverse, or `η` is outside
  the band on two seeds — reported as the model's number. (This is a Green–Kubo-type read at
  equilibrium; RESPONSE-1's R3 reads the same quantity under a drive. Both are banked and
  compared.)
- **S3 — a named Fourier view is near-optimal; a cell view is not.** Held out at `τ = 100`
  fs: the 24-mode Fourier view's fraction `≥ 0.8` of the bound at `k = 24`; the `2×2×2` cell
  view's fraction is BELOW the Fourier view's at equal `k` (its top 24 against the bound's
  top 24). **Kill:** the cell view matches or beats the Fourier view — the cells carry
  closed structure the modes miss, and the response note's "the chart is the modes" is wrong.
- **S4 — the placebo.** The blind relabelling's held-out score at `τ = 100` fs is under
  `0.1` of the bound at `k = 31`. **Kill:** a scrambled partition with a closed law in it
  convicts the reader (branch (e)).

## 4. Plants, each of which must fire before a walk is scored

| plant | must |
|---|---|
| **PV-1** | a synthetic linear system `X_{t+1} = A X_t + noise` with known singular values of `A` (diagonal `0.9, 0.7, 0.5`, three noise directions): the dictionary's top-3 `σ` read back to `3 %` at `N = 3000` frames, and `t₁ = −1/ln 0.9` to `5 %` |
| **PV-2** | time-SHUFFLED frames of a real walk: the held-out score of every view under `0.05` of its in-sample bound — the reader manufactures no closure from order it destroyed |
| **PV-3** | lag `0`: every `σ = 1` to `10⁻⁹` |
| **PV-4** | a view that is a linear image of another (the density modes as combinations of a fine grid): the image's score is `≤` the source's, to rounding — the bound is monotone in the subspace |

## 5. Branches

- **(a)** S1–S4 met → the closed coarse view of this liquid at dimension `k` is DEFINED as
  the top-`k` singular subspace at the cell's cadence, its defect is what the bound leaves
  over, and the hand-named charts are graded against it: the cell charts' fraction is their
  measured inefficiency, the Fourier charts' their near-optimality. RESPONSE-1's chart was
  the search's answer before the search was run.
- **(b)** S1 fails → the slowest directions are not wavelengths; the singular functions are
  printed and the next chart is read off them.
- **(c)** S2 fails on the band → the model's equilibrium viscosity is the reading and the
  band the finding; compared with R3 when the arms land.
- **(d)** S3's cell clause fails → cells carry structure; branch (b)'s print applies.
- **(e)** S4 or a plant fires → nothing is read.

## 6. The floor, and the cost

Finite-sample bias: `N` frames and `d` features inflate every `σ²` by `O(d/N)`; with
`N ≈ 900` (three seeds) and `d = 55` that is `~0.06` per mode — the held-out evaluation
removes it and PV-2 measures what is left. **Shot noise is the same as everywhere on this
box**: at 128 waters the cell fields fluctuate at `1/√⟨n⟩`, so cell scores will be low and
the modes' scores small but resolvable; that is the measurement, not its failure. Cost: a
few seconds per lag per view on one core; the whole campaign under a minute. No core the
arms use.

## 7. What this does not test

Nonlinear views (a chart closed only under a nonlinear law is under-scored by a linear
bound; the drive experiment stays the nonlinear test); views that need the hydrogens
(the walks carry oxygens only); anything the drive changes.

---
*Audit footer, added 2026-09-21 for `Audit/prereg_audit.py` after CI read red since 2026-09-19; no stake, gate, plant or number above moved.*
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-ONE-MODEL-DELTA, M-PLANT-OBS, M-PLANT-SECTOR — the registered ids this text contacts by keyword, cited at the audit's demand.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
