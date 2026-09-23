# SLOW-1 Amendment 1 — the literature's own candidates in the dictionary, so a re-finding is graded against what the field already has

*Frozen 2026-09-23, committed alone, after `SLOW1_PREREG.md` (`53dcecc`) and the launcher and
reader (`bce3943`), and BEFORE any 50 ps walk exists to be read. The four arms were launched
2026-09-23 16:53 CDT and are still in their fine settle; nothing in this amendment touches
them. Every new column is computed by the reader from the oxygen positions the walk already
carries. At the coordinator's order (2026-09-23). No number in S1, S2, S3 or the plants moves;
the candidate list grows, and S3's re-finding clause names the new candidates.*

## Why

S3 grades "direction" against "re-finding". A re-finding has to be graded against the
candidates the field actually holds, not against the three the prereg named. Credited:

1. **Two-body excess entropy per molecule** and its **weighted form**: Wu, Kob, Wang & Zhang,
   "Connecting heterogeneous dynamics with local entropy", arXiv 2609.05276 (Sept 2026). They
   multiply the integrand of the local pair excess entropy by a weight tied to the length
   scale of the pair correlation function, and reach a structure–dynamics correlation of about
   0.9 for long-time propensity in a two-dimensional Lennard-Jones glass former. The local form
   is Piaggi & Parrinello, J. Chem. Phys. 147, 114112 (2017).
2. **Machine-learned softness**: Cubuk, Schoenholz, Liu et al. (PRL 114, 108001, 2015; Nat.
   Phys. 12, 469, 2016), and arXiv 2406.05868 (2024), where softness's own dynamics describe the
   heterogeneity, so softness is treated as a slow state variable. That is SLOW-1's S1+S2
   question. **Softness is NOT added as a column:** it is a classifier trained on rearrangement
   labels, it is nonlinear, and fitting it here would put the answer into the dictionary.
   §"What this does not test" says what follows from leaving it out.
3. **Water specifically**: the tetrahedral order `q` and the H-bond environment are reported to
   govern a molecule's propensity to join slow domains in bulk supercooled water, and a 2025
   JACS Au paper reports that confinement decouples this. `q` and the bond count are already in
   the dictionary. This is the prior a re-finding would confirm.

## The dictionary, amended

STRUCT gains two columns, so it has six: `q, n33, n50, nb, s2, s2w`. Both new columns come from
the wrapped oxygen positions under the minimum image, with number density `ρ = n / L³`:

- **`s2`** (Piaggi–Parrinello local form):
  `s2_i = −2π ρ ∫₀^{r_m} [g_i(r) ln g_i(r) − g_i(r) + 1] r² dr`, where
  `g_i(r) = (1 / 4πρr²) Σ_{j≠i} (2πσ²)^{−½} exp(−(r − r_ij)² / 2σ²)`, `σ = 0.15 Å`,
  `r_m = 7.5 Å` (below half of the smallest box this reader meets, 7.83 Å for 128 waters),
  and a radial grid of `0.03 Å`. Units of `k_B`.
- **`s2w`** (the weighted form): the same integral with the integrand multiplied by
  `w(r) = exp(r / ξ)`, where `ξ` is the decay length of the envelope of `|g(r) − 1|`. `ξ` is
  fitted on each walk's own pooled `g(r)` by least squares of `ln |g − 1|` at its local maxima
  beyond the first peak, out to `r_m`. **This is ours, not the paper's formula.** The abstract
  of arXiv 2609.05276 describes the weight only in words ("directly related to the length scale
  of the pair correlation function"), and the full text was not read. We choose the weight that
  undoes the decay of structural correlation, so every shell out to `r_m` counts equally. If
  the paper's own weight is obtained before the 50 ps read, a further amendment committed alone
  may add it as a column beside this one. It would not replace this one.

HIST, MOM and MODE are unchanged. Local potential energy is still omitted (not on the walk).

## Stakes: exactly what changes

- **S1**: same numbers, read on the six-column STRUCT block.
- **S2**: same numbers. The candidates `c` taken individually are now `q`; `s2`; `s2w`;
  density `(n33, n50)`; bond count `nb`; `h1, h2, h5, h10`. The exclusion of the candidate `f`
  IS (loading ≥ 0.8) applies to the new candidates as it did to the old ones.
- **S3, the re-finding clause, as it now reads:** the loading of `f` at `τ*` on each named
  candidate `c ∈ {q, s2, s2w, density (n33, n50), bond count (nb)}` is the `R²` of `f`
  regressed on `c`'s columns. A loading **`≥ 0.8` on ANY ONE named candidate** reads branch
  **(b)**, "the literature's candidate, confirmed on this operator", and the candidate is
  named. **No candidate above `0.6`** reads branch **(a)** DIRECTION, with the weights of `f` on
  the six STRUCT columns as the result and the `α₂` check owed. Between `0.6` and `0.8`: a
  partial re-finding, reported, neither branch. If two named candidates both load `≥ 0.8`
  (for example `s2` and `s2w`, or density and bond count), both are named and they are NOT
  claimed separable on this walk.
- **Plants:** unchanged in number and bar. PS-3's synthetic carries the two new columns,
  computed from its own positions, so the planted column now has to win against six
  structural columns instead of four.

## What this does not test

- **No nonlinear predictor is fitted, and no graph neural network.** The field reaches a
  structure–dynamics correlation of about 0.9 with those (and with the weighted pair entropy
  on a 2D Lennard-Jones glass former). This dictionary is linear in its columns, and the stake
  is a closure-and-carried question (does a structural sector close at 1–20 ps and add to the
  next-5-ps displacement beyond the velocity and each candidate), not a propensity-prediction
  contest. A low `R²` here is not a verdict on those methods, and a high one does not beat them.
- **Softness is not in the dictionary** (see Why, item 2). A direction found here could still be
  a projection of softness. That question is open and belongs to a later campaign.
- **No comparison to experiment is claimed.** The liquid is the rigid operator's. The water
  literature in item 3 is cited as the prior a re-finding would confirm, not as a referee.

---
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-HOMOG — the registered ids this text contacts by keyword, cited at the audit's demand. M-HOMOG: "local" here is a molecule's neighbourhood entropy or count, not a spatial-locality claim about a graph family. M-PLANT-OBS: PS-3 is re-run on the amended reader before the read.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row (`SLOW1_PREREG.md`), and the sector the plant acts on is nonzero in that carrier by construction of the plant; the amended PS-3 carrier carries the planted column beside the six structural columns.
