# COSMOS, THE PERTURBATION CHANNEL — can the organised portion be read where it clumps? Priced as a misfit campaign, 2026-11-23

*Written on the lead's objection to `TIERS.md` §4's sentence of the same day that the
cosmological rung has "no live observational test": "I am confused, we now have a way to go
back at CMB anomalies." The lead is right that `COSMOS_TWO_PORTION.md` priced only the
BACKGROUND (the expansion history) and never priced clustering. This note prices it. Nothing
is frozen. No dataset, map or likelihood was read: only published summary amplitudes, each
labelled below by how it was read. Every number here is printed by
`cosmos_cmb_channel.py` (numpy, single thread, seconds). It is ORDER-OF-MAGNITUDE LINEAR
THEORY, not a Boltzmann code, and it has not yet had an adversarial review.*

## 0. Verdict, first

1. **There is a live test on this channel, and the sibling stance already stakes it.** It is
   not new today: `void-excess`, `s8-fingerprint` and the clumping clause of
   `dark-balance-extensive` are wagers with kills, none read by a frozen pipeline (§1).
   "No live observational test" was wrong as written and owes a CORRECTION (§9).
2. **It is live only at a large share.** The channel reads the product `P = f·b` (share times
   the organised portion's bias against matter). Today's data reach `|P| ≈ 0.02` to `0.06`.
   At the size the mechanism can pay for, `P` is at most `2 × 10⁻⁴`: 81 to 243 times under
   today's reach. That is about one order nearer than the background channel (1,528 times
   under), and still not readable (§5).
3. **At the share that repairs the shape, a portion that FOLLOWS matter is already
   excluded.** A share of `0.2` to `0.7` with `b = 1` flips the sign of the all-sky
   ISW signal: 6 to 24 standard errors from the measured amplitude. That share may have
   `|b|` of at most `0.09` to `0.31`: it must be nearly smooth (§4).
4. **The stance's `void-excess` sentence has the wrong sign for its own mechanism.** A
   portion that "pools where matter pools" deepens potentials and slows their decay, so it
   WEAKENS the cold imprint of voids. The excess needs a portion that pools where matter is
   NOT (`P < 0`) (§2). That is the sibling record's to check and correct.
5. **The anomalies, read as one mechanism, do not ask for one product.** They ask for
   `P = −0.54` (supervoid ISW), `−0.33` (DES void lensing), `−0.06` (the DES Y6 `S₈` offset),
   `+0.13` (the DESI Legacy voids), a positive `P` of `0.16` to `1.1` (the Planck lensing
   anomaly), against an all-sky bound of `|P| < 0.063`. One small window does not contradict
   itself, `P ≈ −0.04` to `−0.06`, a portion that AVOIDS matter (§3). Nothing forces it.
6. **One stake is freezable on this channel and could come out otherwise** (§7): on one void
   catalogue, the lensing amplitude and the ISW amplitude must sit on a line of known slope
   through `(1, 1)`. The largest existing catalogue, scored after the fact and so not
   counted, already sits off that line by more than three standard errors.

## 1. What the stance already stakes on this channel

All in CIRISOntology `Stance.lean`, all at status WAGER, none read by a frozen pipeline.

| key | what it stakes | its kill | read? |
|---|---|---|---|
| `dark-balance-extensive` | the grand-total reading is "built from a clumpy thing", so it "is at risk of clumping itself"; smoothing it is "either impossible (as a normal substance) or expensive (a preferred-frame sector), and the void measurements will decide which" | dark energy "caught clumping" bears on it; DESI DR3's geometry-only check | no |
| `void-excess` | the too-cold supervoid signal, if real, is the extensive branch clumping; the intensive branch is smooth and predicts none | a pre-registered DESI (later Euclid) supervoid stack returns no excess at high significance | no |
| `s8-fingerprint` | a smooth, fading balance makes today "a fraction of a percent to about a percent" smoother; only the SIGN is firm; "currently explaining nothing" since KiDS-Legacy | `S₈` high, not low; or the intensive branch dying | no |
| `no-early-dark-energy` | no dark energy before structure | an early component of several per cent, agreed across experiments | consistent so far |

Also on the sibling record: `preferred-frame-results.md` (the smooth half is the cuscuton, 19
years old, and is the INTENSIVE branch's mechanism; a single one cannot cross `w = −1`; the
frozen crossing `z = 0.59 ± 0.03` is one free choice deep) and `PLANCK_PILOT_RESULTS.md` (an
instrument pilot on public CMB maps: "NOT a cosmology result ... NOT an anomaly search"; the
order-3 share reads zero on every cell). Nothing on the record stakes the lensing anomaly,
the low quadrupole, the hemispherical asymmetry or the cold spot.

## 2. The mechanism, and its sign

The organised portion follows structure: `δρ_org = b·ρ_org·δ_m`. The Poisson source becomes
`ρ_m δ_m (1 + ε)`, with `ε(a) = P·(Ω_DE/Ω_m)₀·aⁿ`, `P = f·b`, `n = −3 w_org`. Per unit `P`,
`ε` is `2.175` today; at `z = 0.5` it is `0.644` (`n = 3`, an organised density that does not
dilute) or `0.974` (`n = 1.98`, the rent ledger today).

Two amplitudes, each the ratio to a template built on the same galaxy-traced matter:

- lensing reads the potential: `A_κ = 1 + ε`;
- the ISW effect reads the potential's RATE: `A_ISW = [(1 + ε)(g′ − 1) + nε]/(g − 1)`, with
  `g` the growth rate. In the standard model the potential decays (`g − 1 = −0.239` at
  `z = 0.5`), and that decay is the whole signal.

So `dA_ISW/dA_κ = −K`, with `K = 12.35` (`n = 3`) or `8.29` (`n = 1.98`) at `z = 0.5`. `K` is
large because a small, GROWING addition to the source cancels a slow decay. It is positive
for every law on the record (`5.99` for the ledger diluted as matter; `0.93` for a portion
that dilutes exactly as matter).

**The sign.** A portion that follows matter (`P > 0`) raises `A_κ` and LOWERS `A_ISW`: at
`P = +0.05` the ISW amplitude is `0.60`; at `+0.10`, `0.20`; above `P = +0.125` the signal
changes sign and voids read HOT. An ISW EXCESS needs `P < 0`. The stance's sentence ("it
clumps a little itself, pooling where matter pools. Dark energy that pools in and around
voids makes their light-cooling STRONGER") joins two opposite things. From memory, this is
the known direction for clustering dark energy with `w > −1` (Bean and Doré 2004; Hu and
Scranton 2004), not read at source.

## 3. What each published amplitude asks of `P`

| observable | published (how read) | read as this mechanism | `b` at `f = 10⁻⁴` | `b` at `f = 0.2 / 0.4 / 0.7` |
|---|---|---:|---:|---:|
| supervoid ISW, DES Y3 + BOSS | `5.2 ± 1.6`; DES alone `4.1 ± 2.0` (arXiv 1811.07812; search summary of the abstract) | `P = −0.54` (`−0.40`) | `−5,400` | `−2.70 / −1.35 / −0.77` |
| an ISW excess of 2 | — | `P = −0.126` | `−1,300` | `−0.63 / −0.32 / −0.18` |
| DES Y3 void lensing | `0.79 ± 0.12` (arXiv 2203.11306; search summary) | `P = −0.33 ± 0.19` (`n = 3`); `−0.22 ± 0.12` (`n = 1.98`) | `−3,300` | `−1.63 / −0.81 / −0.47` |
| DESI Legacy voids, ISW | `−0.10 ± 0.69`; clusters `1.52 ± 0.72` (arXiv 2105.11936; search summary) | `P = +0.13` to `+0.14` | — | — |
| DESI Legacy superstructures, lensing | `0.811 ± 0.057` (as cited by arXiv 2203.11306, second hand) | `P < 0` | — | — |
| BOSS voids and superclusters, ISW | `1.64 ± 0.53` (Nadathur and Crittenden 2016; search summary) | compatible with `0` or a small `P < 0` | — | — |
| DESI LRG voids, lensing | `1.016 ± 0.054` (A&A 700, A17, 2025; search summary) | `P = +0.04 ± 0.12`; `\|P\| < 0.28` (`0.16` at `n = 1.98`) at two sigma | — | — |
| all-sky ISW × structure | `1.00 ± 0.25` (Planck 2015 XXI; FROM MEMORY) | `\|P\| < 0.063` at two sigma | — | `\|b\| ≤ 0.31 / 0.16 / 0.09` |
| `S₈` offset, DES Y6 3×2pt | `0.789 ± 0.012` (arXiv 2601.14559, January 2026; search summary) against the CMB's `0.832 ± 0.013` (FROM MEMORY): `−5.2 ± 2.1 %` | `P = −0.061` (`n = 3`); `−0.041` (`n = 1.98`) | `−610` | `−0.30 / −0.15 / −0.09` |
| `S₈`, KiDS-Legacy | consistent with the CMB at under one sigma (the stance: `0.73σ`) | `P ≈ 0` | — | — |
| CMB lensing, `A_L` | Planck 2018 `1.180` (FROM MEMORY); ACT DR6 `1.013 ± 0.023` and `1.007 ± 0.057` (arXiv 2503.14452; search summary) | Planck's needs `P ≈ +0.16` to `+1.1`; ACT allows `\|P\| < 0.05` to `0.11` | — | — |
| low quadrupole | low, at a few per cent by chance (FROM MEMORY) | a `P > 0` lowers late ISW power (amplitude `0.55` at `P = +0.05`, `z = 0.3`); sign only | — | — |
| hemispherical asymmetry; the cold spot | about `0.07` at about three sigma; the Eridanus supervoid (arXiv 2112.07699) (FROM MEMORY) | no handle: the mechanism is isotropic; the cold spot is the supervoid excess again | — | — |

**The consistency check.** Voids and `S₈` ask for a NEGATIVE product; the lensing anomaly and
the low quadrupole ask for a POSITIVE one; the sizes run from `0.04` to above `1`. Read
through one catalogue at a time:

- DES supervoids: the lensing deficit predicts an ISW amplitude of `3.56 ± 1.44` (`n = 3`)
  or `2.70 ± 0.95` (`n = 1.98`); measured `5.2 ± 1.6`. Apart by `0.8` to `1.4` standard
  errors, and in the quadrant the mechanism requires (lensing low, ISW high).
- DESI Legacy: the cited lensing deficit predicts `3.31 ± 0.69` or `2.53 ± 0.45`; measured
  `−0.10 ± 0.69`. Apart by `3.2` to `3.5` standard errors, and in the WRONG quadrant (both
  low). The lensing number there is second hand and may not be for the same voids.
- The all-sky bound excludes the DES supervoid products outright (`−0.54` and `−0.33`
  against `±0.063`) unless the effect lives only in supervoids, which is a free function.

**The one window that does not contradict itself:** `P = −0.04` to `−0.06`. It gives an
ISW amplitude of `1.32` to `1.48` (1.3 to 1.9 standard errors from the all-sky value, within
`0.6` of BOSS's `1.64`, and 2.3 to 2.4 below DES's), a void lensing amplitude of `0.95` to
`0.98`, and a lensing-read clustering amplitude lower by 3 to 8 %, which is the DES Y6
offset. At the shares that repair the shape it is `b = −0.25`, `−0.12`, `−0.07`. It is a
portion that mildly AVOIDS matter. KiDS-Legacy speaks against it. Nothing on either record
derives it, and it was found by looking at the data.

## 4. The large share: a portion that follows matter is already dead

At the shares that repair the background shape (`f = 0.2` to `0.7`), with `b = 1`:

| share | all-sky ISW amplitude (`n = 3` / `1.98`) | from `1.00 ± 0.25` | void lensing at `z = 0.7` | from `1.016 ± 0.054` |
|---:|---:|---:|---:|---:|
| `0.2` | `−0.60` / `−0.65` | `−6.4σ` / `−6.6σ` | `1.089` / `1.152` | `+1.3σ` / `+2.5σ` |
| `0.4` | `−2.24` / `−2.37` | `−12.9σ` / `−13.5σ` | `1.177` / `1.304` | `+3.0σ` / `+5.3σ` |
| `0.7` | `−4.73` / `−5.09` | `−22.9σ` / `−24.4σ` | `1.310` / `1.532` | `+5.4σ` / `+9.6σ` |

The ISW column rests on a value quoted from memory; the lensing column does not. Either way
the reading is the one the stance already wrote: a large share built from the galaxy count
cannot clump like the galaxy count. It must be smooth to `|b| ≤ 0.31`, `0.16`, `0.09`. So the
two-portion picture at the size that fits the shape IS constrained by this channel today,
and what it is told is "be smooth": the static portion smooth by construction, the organised
portion smooth to a tenth. That is the intensive branch's mechanism (the cuscuton), which
`preferred-frame-results.md` shows cannot cross `w = −1` alone.

## 5. The payable size: what `P` can be

- **On large scales the bias is the hosts'.** However concentrated the organised portion is
  inside galaxies (their own contrast is `10⁵` to `10⁶`), its fluctuation on the scales
  these measurements use is its MEAN times the linear bias of whatever hosts it: the portion
  is conserved, so concentrating it changes where it sits within a halo and not how many
  halos a region holds. That is `b = 1` to `2`, and about `5` for the rarest clusters. The
  required `b` at the payable shares is `600` to `20,000` in magnitude (§3). Not available.
- **In a supervoid the portion's own contrast is at most about one**, either way: absent at
  the least, doubled at the most if it avoids matter. So `|A_ISW − 1| ≤ 0.004` at
  `f = 10⁻⁴` and a matter deficit of `0.2`, against an anomaly of `1` to `4`.
- **How far under, at the largest Landauer share (`f = 10⁻⁴`):** 81 to 243 times under the
  one-sigma reach of the DES Y6 against CMB comparison (`|P| = 0.016` to `0.024`); 300 to
  901 times at the hydrogen-burning share; and 2.1 to 6.2 times at the share where every
  gram of new stars is turned wholly to energy, which nothing pays. A future half-per-cent
  amplitude measurement reaches `|P| = 0.004` to `0.006`: still 20 to 60 times above the
  Landauer size. The background channel is 1,528 times under.

So the perturbation channel is the more sensitive of the two by about an order, and the
lead's instinct about where to look is right. It does not reach the payable size.

## 6. The static portion and the horizon

Nothing here depends on which horizon prices the static portion. A portion proportional to
`H²` plus a constant is smooth at linear order in this treatment. Event-horizon holographic
dark energy has its own perturbations; they are not priced here.

## 7. The stake that could be frozen on this channel

**Carrier:** one supervoid catalogue fixed in advance (DESI DR2, radius above
`100 h⁻¹ Mpc`, effective redshift `z̄` stated before the stack), against one lensing map and
one temperature map fixed in advance, with a template pipeline frozen before either stack is
read. **Quantities:** `A_κ ± σ_κ` and `A_ISW ± σ_I`, and `K(z̄)` from this script between
`n = 1.98` and `n = 3` (`8.29` to `12.35` at `z̄ = 0.5`).

**The forced part:** any portion that follows or avoids matter puts the pair on
`A_ISW − 1 = −K (A_κ − 1)`. A smooth constant, and the organised portion at any payable
size, put it at `(1, 1)`. On paper: `P = −0.20` reads `(0.871, 2.58)` or `(0.805, 2.58)`;
`P = −0.10` reads `(0.936, 1.79)`; `P = +0.10` reads `(1.064, 0.20)`. With `σ_κ = 0.06` and
`σ_I = 0.7` (the sizes of the DESI Legacy numbers), a lensing deficit of `0.2` is 3.3
standard errors and the line then predicts an ISW amplitude of `2.62` to `3.44`: 2.3 to 3.5
standard errors from one. The ISW arm is the weak one, and that is cosmic variance.

**Branches**, on `t_κ = |A_κ − 1|/σ_κ` and `t_I = |A_ISW − 1|/σ_I`, exactly one per outcome:

- **(a) SMOOTH** — `t_κ < 2` and `t_I < 2`: no clumping portion is read; `|P|` is bounded
  at `2σ_κ/ε(z̄)`; `void-excess` meets its own kill.
- **(b) ON THE LINE** — `t_κ ≥ 3`, and `A_ISW` within two combined standard errors of the
  band `[1 − K_max(A_κ − 1), 1 − K_min(A_κ − 1)]`: a clumping portion is read, with its
  sign: `A_κ < 1` is a portion that AVOIDS matter, `A_κ > 1` one that follows it. The same
  `P` must then meet the all-sky amplitude and the `S₈` offset, or it is a void-only
  effect and not this mechanism.
- **(c) OFF THE LINE** — `t_κ ≥ 3` with `A_ISW` outside that band; or `t_κ < 2` with
  `t_I ≥ 3`: whatever is there is not a clumping dark energy of this kind.
- **(d) BETWEEN** — every other case (`2 ≤ t_κ < 3`; or `t_κ < 2` with `2 ≤ t_I < 3`):
  nothing moves.

Scored after the fact on what is published, and therefore NOT counted: DES supervoids give
`t_κ = 1.75`, `t_I = 2.6`, branch (d); DESI Legacy gives `t_κ = 3.3` and an ISW amplitude
3.2 to 3.5 standard errors off the band, branch (c). The stake is not vacuous: the largest
existing data would already have refused the mechanism, if the second-hand lensing number
is for the same voids.

**What it cannot do:** support or refute the portion at the Landauer size (it lands in (a)
either way). It tests the large, clumping portion only.

## 8. Misfits contacted

- **M-RETRODICTION-AS-PREDICTION** — every anomaly here was known before the amendment;
  "the anomalies fit" can only ever be a retrodiction. The window of §3 was found by
  looking.
- **M-BASE-RATE-OMITTED** — about six CMB anomalies exist at two to three sigma each;
  picking the ones a mechanism can reach, after the fact, has a base rate nobody on the
  record has priced.
- **M-POPULATION-CHOICE** — which catalogues count (DES and Granett say excess; DESI
  Legacy, BOSS and the all-sky amplitude do not). All are in the table; none is dropped.
- **M-FOREIGN-DOMAIN-CORROBORATION** — an unexplained excess in someone else's measurement
  is not support; the stance says so itself.
- **M-ONE-MODEL-DELTA** — branch (c) reads "not this mechanism" from one linear model.
- **M-EXTRAPOLATED-HOLE** — linear, quasi-static theory spent on supervoids and on scales
  near the horizon, where the ISW signal lives.
- **M-SORTS-NOT-SEPARATES** — the ISW arm separates the line from `(1, 1)` by only 2.3 to
  3.5 standard errors at a lensing deficit of `0.2`; below that the stake rests on lensing.
- **M-SCREEN-SCOPE-UNSTATED** — the literature here was read through search summaries of
  abstracts, and four values are from memory; said at each row.
- **M-KILL-DISSOLVED** — if the organised portion is allowed any `b`, `void-excess`'s kill
  (no excess) is absorbed by `b = 0`. The stake above names the sign and the line so that
  it is not.
- **M-BOUND-HYPOTHESIS-UNMET** — the bound `b ≤ 5` assumes the portion is carried by matter
  and conserved; a portion that is a field of its own is not bound by it.

## 9. Proposed CORRECTION to `TIERS.md` §4 (not made here)

> **CORRECTION 2026-11-23 to items 4 and 5 of the paragraph ADOPTED the same day.** "No live
> observational test" was true of the expansion history only. The perturbation channel was
> not priced, and the sibling stance already stakes it (`void-excess`, `s8-fingerprint`, the
> clumping clause of `dark-balance-extensive`), unread. Priced in linear theory
> (`conformance/omega/COSMOS_CMB_CHANNEL.md`, not yet reviewed): the channel reads the share
> times the organised portion's bias; today's data reach a product of `0.02` to `0.06`; the
> size the mechanism can pay for is at most `2 × 10⁻⁴`, 81 to 243 times under, where the
> expansion history is 1,528 times under. At the share that repairs the shape, a portion
> that follows matter is already excluded and it must be smooth to a tenth. So: a live test
> exists for a LARGE organised portion, it already constrains one, and one stake on it is
> freezable (the lensing and ISW amplitudes of one void catalogue on a line of known
> slope). No test, on either channel, reaches the Landauer size. Item 5's "what would
> reopen it" gains a third entry: a frozen supervoid stack that lands on that line.

## 10. Open to the reviewer

1. **The ISW formula.** `A_ISW` is taken as the ratio of the potential's logarithmic rate at
   one redshift, quasi-static, with no scale dependence. The real signal integrates along
   the line of sight through scales near the horizon, where a clustering component's own
   pressure and sound speed matter. `K` could be off by a factor of order one; the SIGN
   should be checked first.
2. **The all-sky amplitude `1.00 ± 0.25` is from memory** and carries the strongest
   exclusion in §4. If it is `±0.4`, the `b = 1` exclusion at a share of `0.2` falls to
   about four standard errors; the lensing column still excludes `0.4` and above.
3. **The DESI Legacy lensing number is second hand** and may be for voids and clusters
   together. The "wrong quadrant" reading depends on it.
4. **`b ≤ 5` is an argument, not a measurement,** and it assumes the portion is carried by
   matter. It decides the payable-size answer.
5. **The background is held fixed.** A share of `0.2` to `0.7` changes `H(a)` and the growth
   through it; the two effects are not combined here.
6. **The `S₈` reading uses one redshift (`0.4`) and one number per survey.** DES Y6 and
   KiDS-Legacy disagree with each other at about the size of the effect.
7. **The stake's bars are conventional (two and three standard errors) and its `K` band
   comes from two laws.** A law with `n` between `0` and `1.4` has `K` between `0.93` and
   `5.99` and would need its own band; the stake should name the law before the stack.
