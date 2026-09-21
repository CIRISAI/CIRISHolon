# VIEW-SEARCH-1 READ: the search finds the long-wavelength modes on its own, the named Fourier chart is at the bound and the cell chart is 70–78 % of it, and the model's equilibrium viscosity is `1.4–1.9 × 10⁻⁴` Pa s at 128 waters and `4.4–5.8 × 10⁻⁴` at 432

*2026-09-20. Prereg `VIEW_SEARCH_PREREG.md`, Amendment 1 (corrections on building), reader
`view_search.py` (plants PV-1..4 pass), outputs `replace0/view_search_{transport_flexible,
transport_rigid,scout432_rigid}.txt`. Data: the banked equilibrium walks — no new dynamics.
Compute: under a minute on two threads.*

## 0. Verdict, under the stakes as written and as amended

| stake | as written | read | as amended |
|---|---|---|---|
| **S1** the top-6 singular functions lie in the first-harmonic Fourier subspace, `≥ 0.7` | **MET** on all three carriers: `R² = 0.99–1.00`, and they are the DENSITY modes (`0.92–0.97`), the transverse currents next | same | — |
| **S2** slowest mode transverse, `η` in `[2, 30] × 10⁻⁴` Pa s | **not readable as instrumented** — the in-sample `σ` are biased to `1` (A1 §2) | from the autocorrelation: **128 waters `1.4–1.9 × 10⁻⁴`** (both operators, three seeds each), **432 waters `4.4–5.8 × 10⁻⁴`** | 432 IN BAND; 128 just under — `η` rises as `k` falls, `k`-dependent as the shear-wave onset predicts (`RESPONSE1_AMENDMENT_1.md`) |
| **S3** Fourier view `≥ 0.8` of the bound; cells below it | **MET**: Fourier-all `1.09–1.11` (above `1`: the named view generalises better than the fitted bound's own subspace); cells `0.70–0.78` | same | — |
| **S4** placebo `< 0.1` of the bound | **FIRES as written** (`0.28–0.42`): branch (e) as the prereg says | the placebo carries single-particle velocity memory, real and non-spatial; the time-shuffled placebo reads `0.04` (PV-2) | **MET as re-staked**: spatial − blind `= +0.36, +0.36, +0.42` (stake `≥ 0.05`) |

**As written, branch (e) is entered on S4 and nothing is read.** As amended the same day with
the wrong premise named, S1, S3 and S4 are met and S2 is read from the autocorrelation with
the band met at 432 and missed by `10–30 %` at 128. The reader can grade both and does.

## 1. What the search found without being told

At `τ = 100` fs on every carrier the six slowest closed directions of the 55-feature
dictionary are, to `R² ≥ 0.99`, combinations of the six first-harmonic density modes with a
transverse-current admixture of `0.1–0.4`. The hand-named charts of RUNG-2 and RESPONSE-1
(the longest-wavelength density and current modes) are the search's answer; the cell
occupancy chart is not — at `k = 7` it reaches `0.27–0.45` of the bound, and with momenta at
`k = 31` `0.70–0.78`. The chart the program has been staking, `(n̄, p̄)` per cell, is a
`70–78 %`-efficient basis for the closed structure a 128- or 432-water box actually has.

## 2. The model's equilibrium transport, read from the walks that were already on disk

| carrier | `k` (Å⁻¹) | `Γ_s` (/ps) per seed | `η = ρΓ_s/k²` (Pa s) | density mode's first zero → period → `c_s` |
|---|---|---|---|---|
| 128 waters, flexible | 0.40 | 2.90, 2.24, 2.38 | `1.8, 1.4, 1.5 × 10⁻⁴` | `~500` fs → `~2.0` ps → `~780` m/s (one seed crosses within 500 fs) |
| 128 waters, rigid | 0.40 | 2.80, 2.96, 3.08 | `1.7, 1.8, 1.9 × 10⁻⁴` | as above |
| 432 waters, rigid | 0.27 | 3.55, 3.15, 4.16 | `5.0, 4.4, 5.8 × 10⁻⁴` | `~400` fs → `~1.6` ps → `~1470` m/s (two seeds) |

The rigid operator's `η` agrees with the flexible model's at 128 waters within the seed
spread (`1.8 ± 0.1` vs `1.6 ± 0.2`): the omission's price on the shear rent is zero to this
resolution — a transport reading REPLACE-0's own gate could not give. Between 128 and 432
waters `Γ_s` barely moves while `k²` falls by `2.2`, so `η(k)` rises by `2.7`: at these
wavevectors the transverse current is not yet hydrodynamic (`η` should be `k`-independent),
which is the shear-wave-onset regime Amendment 1 of RESPONSE-1 named, and it says the R3
band should be read as `η(k)` at `0.27 Å⁻¹`, expected near `5 × 10⁻⁴`. **The density mode
oscillates** — its autocorrelation crosses zero at `400–500` fs — so R2's branch is the
oscillatory one (period gives `c_s`, staked in `[1000, 2200]` m/s: `1470` at 432 is inside)
and RESPONSE-1 Amendment 1's `λ₁ = c_s²/ν_l` overdamped form will not be the fit; its
Amendment 2 integral leg is unaffected. All of this at lag resolution `100` fs on 5–6 ps
walks: numbers to `20–30 %`, sharpened by the arms.

## 3. The static part, printed and set aside

Each seed's density modes sit `1–2` counts off zero for its whole 5–6 ps against a sd of
`1.4–2.7` — a frozen long-wavelength pattern; the box does not rearrange on that time. The
search, uncentred, read those as 20 ps modes. They are Held rather than Closed views on
6 ps. *(Corrected 2026-09-20 evening: NOT a glass. `TSCAN-1` read `D = 6.5 × 10⁻¹⁰` m²/s
at 312 K; `r = 0.4–0.65` on 5 ps is a mode with `τ_c ≈ 1–2` ps, and the diffusive relaxation
of this wavelength is `95–215` ps for this model, `27–61` for water. The offset is what any
liquid shows here. The fifteenth instance.)*

## 4. Branch

(e) as written, on S4's wrong premise; (a) as amended: **the closed coarse view of this
liquid at dimension `k` is the top-`k` singular subspace at the cell's cadence, it is the
long-wavelength Fourier modes, and the hand-named charts are graded against it.** Cost
`0.01` core-hours against RESPONSE-1's `~200`; what the arms add is the DRIVEN read with
its nulls, the tenfold better `η(k)`, and the nonlinearity control.
