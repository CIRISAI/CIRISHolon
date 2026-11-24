# COSMOS-BASE — the organised share fixed as a ratio of a dynamics to its base: enumerated, fitted, counted. Nothing is forced; two readings are worth a stake

*2026-11-23. A paper campaign, not a preregistered one: no data fetched, nothing frozen. The
script is `cosmos_base.py` (numpy, single thread, about 11 s); it imports the grid, the
star-formation history, the solver, the 14 DESI DR2 numbers, the four published `(w₀, wₐ)`
points and the fit from `cosmos_two_portion.py`, and prints every number below. Not yet
reviewed by a second reader.*

**DISCLOSURE, first.** The lead's sentence of 2026-11-23: "The change to record only ever
makes sense in relation to the base, DM is the base light matter is the dynamics on top, or
DM+LM is the base and the dynamics on top are by tier, etc... - should be easy to figure
out." Before any enumeration the lead's agent did head arithmetic from standard values and
noticed that `Ω_b/Ω_dm ≈ 0.186` and `Ω_m/Ω_DE ≈ 0.46` sit near the shares that today's
quick-looks at DESI DR2 asked for (`0.2` to `0.3` for the matter-like law with no decay;
`0.40` to `0.70`, best `0.50`, for the rent ledger). **So this is not blind with respect to
DR2, the enumeration below is post hoc, and the only prospective tests are DESI DR3 and the
perturbation channel** (M-RETRODICTION-AS-PREDICTION).

## 0. Verdict, first

1. **Read strictly, "relative to the base" is a relabelling.** A ratio that holds at all
   times to a matter-like base makes the organised portion dilute as matter: it IS matter on
   the background (`Ω_m′ = Ω_m + f·Ω_DE`), never phantom, with no crossing. With the matter
   density free the fit cannot see the share at all; with it held at the measured value the
   share `Ω_b/Ω_dm` is excluded by `χ² + 648`. If it clusters it is dark matter by another
   name; if it does not, it cuts `σ₈` to `0.19` of its value. And a ratio of organised to
   static, with both parts of the ratio matter-like, makes the organised portion a constant.
   Either way nothing is added to a constant and matter. This is the horizon pricing's
   problem again (§3).
2. **Read as "the ratio fixes the SIZE today, the growth law is separate", it is no longer a
   relabelling, and two readings do well** (§4): the base over the record, `Ω_m/Ω_DE = 0.4605`,
   under the record's rent ledger reads `χ² = 8.039`, `2.56` better than a constant with NO
   free share, `0.13` from the free-share best, and lands `0.8` standard errors from the
   DESY5 point; and `Ω_b/Ω_dm = 0.1864` under the star-written, matter-diluting law reads
   `8.665`, `1.94` better than a constant, `0.09` from that law's free-share best.
3. **Neither is forced, by count.** The roles admit 9 ratios, 3 things the ratio can be a
   ratio of, and 5 laws: 23 shares, 115 readings. The windows are wide. Chance alone puts
   `3.6` to `5.0` of 23 log-uniform ratios in each window; `4` to `8` landed. The probability
   that at least one lands is `0.98` or more (§5). A hit here is not evidence.
4. **No rule joins the tiers** (§6). By count the rule gives `0.5` (molecule) and `0.6`
   (fluid); by energy `0.108 ± 0.033` (molecule) and nothing for the fluid; by price at most
   `0.085` (fluid) and `0` (the network), with every dropped block's price consistent with
   zero. No one rule is measured at two tiers in one unit.
5. **The writer still cannot pay** (§7): a star-written portion of share `Ω_b/Ω_dm` is 28 to
   47 times the rest mass of every star ever formed, unpaid at fusion efficiency by 3.6 to
   3.8 orders.
6. **At these sizes the portion must NOT follow its base in space** (§8). The perturbation
   note bounds the product of share and bias at `0.063`: at `f = 0.186` the bias is at most
   `0.34`, at `0.46` at most `0.14`. A portion tied to matter point by point is excluded; it
   can be tied to its base only in the mean.
7. **What it is worth:** a stake with the share FIXED, which a free share never was (§9).
   Two fixed readings differ from a constant by 2.7 to 4.2 of DR2's standard errors and from
   each other by 1.8 to 1.9, with crossings at `z = 0.573` and `0.818`.

## 1. The enumeration, written and hashed before any fit

The script prints this text and its sha256
(`70188444afe1971b873ad51b7d03727f934ca6704eb5ea83199914ff37949760`) before it fits anything.

**Ratios of a dynamics to a base**, from the wager's roles (dark matter the capacity,
luminous matter the writer, dark energy the record):

| | ratio | reason | value |
|---|---|---|---|
| r1 | `Ω_b/Ω_dm` | "DM is the base, light matter the dynamics on top" | `0.1864 ± 0.0022` |
| r2 | `Ω_b/Ω_m` | "DM+LM is the base"; the writer's part of it | `0.1564 ± 0.0038` |
| r3 | `Ω_*/Ω_dm` | only the writers (stars) count as dynamics | `0.0138 ± 0.0036` |
| r4 | `Ω_*/Ω_m` | the same on the joint base | `0.0116 ± 0.0030` |
| r5 | `Ω_*/Ω_b` | "the dynamics on top are by tier": writers among light matter | `0.074 ± 0.019` |
| r6 | `Ω_m/Ω_DE` | the base against the record | `0.4605 ± 0.0156` |
| r7 | `Ω_dm/Ω_DE` | the capacity against the record | `0.3863 ± 0.0116` |
| r8 | `Ω_b/Ω_DE` | the writer against the record | `0.0720 ± 0.0022` |
| r9 | `Ω_*/Ω_DE` | the writers against the record | `0.0053 ± 0.0014` |

Densities: Planck 2018 base ΛCDM (arXiv 1807.06209): `Ω_b h² = 0.02237`, `Ω_c h² = 0.1200`,
`h = 0.6736`, `Ω_m = 0.3153`, `Ω_Λ = 0.6847`. Stars: `0.0027` (Fukugita and Peebles 2004)
to `0.0046` (Madau and Dickinson 2014), BOTH FROM MEMORY; the error on r3, r4, r5, r9 is
half that spread.

**What the ratio is a ratio of** (a fork): T1 the organised share itself (`f = r`); T2
organised over static (`f = r/(1+r)`); T3 organised over its base (`f = r·Ω_base/Ω_DE`,
r1 to r5 only).

**When it holds** (a fork): LA at all times, diluting with a matter-like base
(`ρ_org ∝ a⁻³`); LS at all times as organised over static (the organised portion then
constant); L2 today only, growth by the record's rent ledger (`λ = 1.671 H₀`); LD today
only, the record as what stars have written, diluting as matter, no decay; L1 today only,
what stars have written held for ever.

**Count: 23 shares, 115 readings.** The 23 shares run from `0.0053` to `0.4605`.

**The stance already names this fork.** `dark-balance-intensive` is the per-unit reading,
"evened out for size", which is the lead's "in relation to the base"; `dark-balance-extensive`
is the grand total. They are staked in CIRISOntology as RIVALS with DESI DR3 to cut between
them, and the intensive one "can only shrink, never grow": no crossing.

## 2. Does the lead's sentence mean the all-time ratio?

Taken at its word, yes: a change "in relation to the base" is a ratio that travels with the
base. That is LA, and LA is §3. The today-only readings (L2, LD, L1) keep the ratio as a
SIZE and take the time dependence from somewhere else, which is two choices and not one.
This note prices both and says which is which. The lead decides which was meant.

## 3. The all-time laws are relabellings

- **LA.** `E² = (Ω_m + f·Ω_DE) a⁻³ + (1−f) Ω_DE + radiation`: a constant with
  `Ω_m′ = Ω_m + f·Ω_DE`, at every `f`. With `Ω_m` free the fit returns `χ² = 10.603` at
  `Ω_m′ = 0.2953` whatever `f` is. With the matter density held at Planck's `0.3153`,
  `f = 0.1864` gives `Ω_m′ = 0.4429` and `χ² = 659.1`, `648.5` above a constant; `f = 0.4605`
  gives `2,157.8`.
- **By the identity** `1 + w = share × (1 + w_org)` with `w_org = 0`: `1 + w = share(a) > 0`
  at every epoch. At `f = 0.1864`: `w = −0.814` today, `−0.353` at `z = 1`, `−0.139` at
  `z = 2`. Never below `−1`. DR2's hint includes `w < −1` in the past; this law cannot have it.
- **In perturbations** (linear growth, §8): if the portion clusters as matter it is
  identical to matter by construction. If it is smooth, a share `Ω_b/Ω_dm` leaves `σ₈` at
  `0.193` of its value, and `Ω_b/Ω_DE` at `0.551`.
- **LS.** Organised over static, fixed, with both parts of the ratio matter-like: the
  organised portion is constant. A constant.

**What distinguishes "organised dark energy tracking dark matter" from "a little more
matter and a smaller constant"? On the background, nothing. In linear perturbations,
nothing if it clusters, and a `σ₈` that is far too low if it does not.** A relabelling.

## 4. The ratio as a size today, the law separate

14 DR2 numbers, `Ω_m` free, no share free. A constant: `χ² = 10.603`. The general
two-parameter form: `7.160` (the sibling record's; not recomputed).

| law | free-share best | within 1 of its best | at least 1 better than a constant |
|---|---|---|---|
| L2, rent ledger | `7.907` at `f = 0.562` (33-point grid; `COSMOS_TWO_PORTION.md` has `7.768` at `0.50`) | `0.237` to `1.000` | `0.133` to `1.000` |
| LD, star-written, diluting | `8.579` at `f = 0.237` | `0.075` to `0.422` | `0.075` to `0.422` |
| L1, star-written, held | `10.603` at `f = 0` | `0` to `0.133` | empty |

The forced shares (χ², and the difference from a constant):

| share | `f` | L2 | `w₀, wₐ`, crossing | LD | `w₀, wₐ`, crossing | L1 |
|---|---|---|---|---|---|---|
| r6.T1 `Ω_m/Ω_DE` | `0.4605` | `8.039` (`−2.564`) | `−0.768, −0.739`, `0.573` | `9.865` (`−0.738`) | `−0.687, −0.791`, `0.83` | `+6.295` |
| r7.T1 `Ω_dm/Ω_DE` | `0.3863` | `8.217` (`−2.386`) | `−0.804, −0.622`, `0.575` | `9.173` (`−1.430`) | `−0.732, −0.677`, `0.82` | `+4.689` |
| r6.T2 | `0.3153` | `8.458` (`−2.144`) | `−0.839, −0.509`, `0.576` | `8.746` (`−1.857`) | `−0.777, −0.564`, `0.82` | `+3.380` |
| r7.T2 | `0.2786` | `8.613` (`−1.989`) | `−0.857, −0.451` | `8.625` (`−1.978`) | `−0.801, −0.504` | `+2.787` |
| r1.T1 `Ω_b/Ω_dm` | `0.1864` | `9.103` (`−1.500`) | `−0.904, −0.303`, `0.579` | `8.665` (`−1.938`) | `−0.863, −0.347`, `0.818` | `+1.536` |
| r1.T2, r2.T1 | `0.157` | `9.29` (`−1.31`) | `−0.918, −0.256` | `8.79` (`−1.81`) | `−0.884, −0.295` | `+1.2` |
| r2.T2 | `0.1352` | `9.441` (`−1.161`) | | `8.926` (`−1.677`) | | `+0.986` |
| r5.T1, r8.T1, r1.T3 | `0.072–0.074` | `9.92` (`−0.69`) | `−0.96, −0.12` | `9.5` (`−1.1`) | `−0.945, −0.14` | `+0.45` |
| r3, r4, r9, the T3 of stars | `0.005–0.014` | `−0.05` to `−0.14` | | `−0.10` to `−0.25` | | `+0.03` to `+0.07` |

Against the four published `(w₀, wₐ)` points, in standard errors with an ASSUMED correlation
of `−0.9` (a quick look; the sibling record says not to test in this plane): a constant is
`3.0, 3.1, 3.9, 4.4` away; `Ω_m/Ω_DE` under L2 is `1.8, 1.8, 1.2, 0.8`; `Ω_dm/Ω_DE` under L2
`2.0, 1.4, 1.6, 1.1`; `Ω_b/Ω_dm` under LD `2.4, 2.2, 2.6, 2.4`; under L2 `2.5, 1.6, 2.8, 2.7`.

So the two best readings are not the lead's first one. "DM is the base, light matter on
top" (`0.186`) is a fair fit under the star-written law; the best fixed share is the base
over the record (`0.46`) under the ledger. And `Ω_m/Ω_DE` is the coincidence ratio: it is
`0.46` only today, so a share equal to it is a statement about now.

## 5. Look-elsewhere

23 enumerated shares against each window; the chance that one random ratio in
`[10⁻⁴, 1]` lands, log-uniform and uniform:

| window | hits | `p`, one ratio | expected by chance | P(at least one) |
|---|---|---|---|---|
| L2, within 1 of its best | 4 | `0.16` / `0.76` | `3.6` / `17.5` | `0.980` / `1.000` |
| L2, 1 better than a constant | 8 | `0.22` / `0.87` | `5.0` / `19.9` | `0.997` / `1.000` |
| LD, either window | 7 | `0.19` / `0.35` | `4.3` / `8.0` | `0.992` / `1.000` |
| L1, within 1 of its best | 15 | `0.78` / `0.13` | `18.0` / `3.1` | `1.000` / `0.963` |
| L2, within 1 standard error of the DESY5 point (`0.42` to `0.56`) | 1 (`Ω_m/Ω_DE`) | | | |
| L2, within 1 standard error of the Union3 point (`0.52` to `0.86`) | 0 | | | |
| L2, within 1 standard error of the Pantheon+ point | window empty (closest `1.19` at `0.30`) | | | |

The hits are at the rate chance gives. The data do not fix the share (the window for L2 runs
from `0.24` to `1`), so any ratio of order a tenth to one "works". **This is what "not
forced" means here: the fit is too loose to choose, and the roles supply too many ratios.**

## 6. By tier

The same rule, "organised over static equals dynamics over base", on the tiers the record
has measured. Candidate identifications, written before computing:

| tier | by count | by energy | by price |
|---|---|---|---|
| molecule (`replace0/README.md`: the dropped vibrations, `+0.325 ± 0.099 kT` a water) | 3 vibrations over 6 rigid: `0.500`; over 9: `0.333` | over the rigid unit's `3 kT`: `0.108 ± 0.033` | not measured in this unit |
| fluid element (`OBJECTIVE1_RESULTS.md`: increments of `R²`) | 3 structural and count fields over 5 conserved: `0.600` | not measured | dropped blocks `−0.0031, −0.0017, −0.0030, +0.0033`: consistent with ZERO; the window's edges `0.0033/0.0390 = 0.085` |
| hydrogen-bond network | — | — | exactly `0` |

- No rule has a reading at two tiers in one unit. Count gives `0.5` and `0.6` and is the
  arithmetic of how the views were chosen. Energy has one tier. Price has two, and they read
  zero within error.
- The molecule's `0.108 ± 0.033` window catches one cosmic share (`Ω_b/Ω_m` as organised
  over static, `0.135`); chance expects `1.6` of 23.
- **So the cosmos is not a prediction from a lower tier.** What would make it one: the
  fluid's dropped-block price measured to better than its present error, in a unit the
  molecule's handoff energy can be put in. `OBJECT.md` (CORRECTION 2026-11-23, item 5)
  names that measurement for another reason.

## 7. Who pays, in the base's terms

- Under LA nothing is written after the start: no rate, no writer, no bill. It is matter.
- Under L2 the review's figure stands: a share of `0.4` writes `0.26 ρ_DE` a Hubble time.
- Under LD there is no decay, so no rate to sustain, but the stock must exist: at
  `f = Ω_b/Ω_dm` the organised portion is `0.1276` of critical, `2.6` times all baryons and
  28 to 47 times the rest mass of every star formed; at fusion efficiency (`0.007`) it is
  unpaid by 3.6 to 3.8 orders. At `Ω_b/Ω_DE` it equals the baryons' whole rest mass.
- A star-written record the stars can pay for is `Ω_* × 0.007`, a share of `2.8 × 10⁻⁵` to `4.7 × 10⁻⁵`,
  the size the earlier note found. The base ratios do not change the objection. They
  remove it only by not being written, and then §3 applies.

## 8. The perturbation channel at a forced size

Linear growth, order of magnitude, not a Boltzmann code. `COSMOS_CMB_CHANNEL.md` (committed
the same day, `8a98cf3`) reads the product `P = f·b` and bounds it at `|P| < 0.063` from the
all-sky signal (its ISW value quoted from memory).

- **A forced share cannot follow its base in space.** `b ≤ 0.34` at `f = 0.186`;
  `b ≤ 0.14` at `0.46`. The Poisson source of a clustering portion relative to matter's is
  `f·(Ω_DE/Ω_m)·b = 0.405 b` at `Ω_b/Ω_dm`: far from small.
- **Growth.** With the portion smooth, `σ₈` moves by `−0.2 %` to `−0.8 %` (background only).
  Clustering with `b = 1` it rises by `2.2 %` at `0.186` and `5.5 %` at `0.46`: the opposite
  sign from the low-`S₈` readings (Planck `0.832 ± 0.013`; KiDS-1000 `0.759 ± 0.024`; DES Y3
  `0.776 ± 0.017`; KiDS-Legacy `0.815 ± 0.016`; all from memory).
- **A smooth matter-like fraction of `0.014` of matter lowers `σ₈` by 5 %.** That is a share
  of `0.0064` of dark energy; `Ω_b/Ω_dm` asks 29 times that. Only the stars-over-base
  ratios (`0.005` to `0.014`) are of that size, and those are the stars' own mass, already
  counted as matter.

**"Relative to the base" in the mean, smooth in space** is what survives. That is the
intensive branch's own mechanism, which the sibling record shows cannot cross `w = −1`
alone (`preferred-frame-results.md`); a crossing then has to come from the growth law.

## 9. A stake with the share fixed (proposed, not frozen)

The free share made the DR3 stake of `COSMOS_TWO_PORTION.md` unfreezable. Fixing it by a
ratio gives numbers. Two readings, A and B, declared rivals:

| | share | law | `w₀` | `wₐ` | crossing | `χ²` on DR2 |
|---|---|---|---|---|---|---|
| A | `Ω_m/Ω_DE = 0.4605` | rent ledger, `λ = 1.671 H₀` | `−0.768` | `−0.739` | `z = 0.573` | `8.039` |
| B | `Ω_b/Ω_dm = 0.1864` | star-written, diluting, no decay | `−0.863` | `−0.347` | `z = 0.818` | `8.665` |
| constant | `0` | | `−1` | `0` | none | `10.603` |

**Separation on paper**, in DR2's own standard errors (Pantheon+ / DESY5 combinations,
assumed correlation `−0.9`): constant to A `4.2 / 4.1`; constant to B `2.8 / 2.7`; A to B
`1.9 / 1.8`. DR3's errors are smaller, by how much is not on the record.

**The stake, on DESI DR3's own direct fit** (BAO with the acoustic point, `Ω_m` free, NO
free share, no free rate; then repeated with each supernova set, each graded alone):

- **(a) A:** `χ²(A) ≤ χ²(constant) − [4]` and `χ²(A) ≤ χ²(B) − [2]`.
- **(b) B:** `χ²(B) ≤ χ²(constant) − [4]` and `χ²(B) ≤ χ²(A) − [2]`.
- **(c) the split, share undecided:** both beat a constant by `[4]` and differ by less
  than `[2]`.
- **(d) a constant:** neither beats a constant by `[2]`. Both readings are dead at these
  sizes; what remains is a share too small to read.
- **(e) between:** anything else. Nothing moves.

Every bracketed bar is a named blank for the lead. `[4]` is not from DR2 (DR2 gives `2.56`
and `1.94`, so on DR2 itself the outcome is (e)). The rate `1.671 H₀` and both laws ARE
from DR2 and from the sibling's fit; that is the stake's contamination and it is stated.
A second, independent kill on the perturbation channel: either reading with `|b| > 0.34`
(B) or `> 0.14` (A) is already excluded; the stake there is the perturbation note's §7.

What the stake does NOT test: whether the share is the ratio for a REASON. A pass for A or
B at these sizes would still leave the writer unpaid (§7) and the tiers unjoined (§6).

## 10. Misfits contacted

- **M-RETRODICTION-AS-PREDICTION:** the whole enumeration; disclosed at the top.
- **M-POPULATION-CHOICE:** the nine ratios are a chosen census. Named exclusions: neutrinos,
  photons, black holes, gas phases, and any ratio of rates (star-formation rate over a base).
- **M-BASE-RATE-OMITTED:** answered by §5.
- **M-KILL-DISSOLVED:** fixing the share restores a kill the free share dissolved (§9 (d)).
- **M-SORTS-NOT-SEPARATES:** A and B are `1.8` to `1.9` DR2 standard errors apart; whether
  DR3 separates them is not shown.
- **M-BAR-FROM-THE-READ:** the rate, the laws, and the choice of A and B as the two to stake.
- **M-FOREIGN-DOMAIN-CORROBORATION:** the by-tier comparison across units (§6), refused.
- **M-IDLE-CALIBRATED-TIMEOUT** (a reference from one regime compared in another):
  `Ω_m/Ω_DE` holds only today.
- **M-SCREEN-SCOPE-UNSTATED:** searched for this note: this repository's `conformance/omega`,
  `TIERS.md`, `OBJECT.md`, and the sibling's `Stance.lean` at the seven keys named. Not
  searched: the literature on dark energy tied to baryon or dark-matter fractions
  (interacting dark energy, cosmologically coupled black holes), which is prior art for
  some of these ratios.

## 11. Open to the reviewer

1. **The reading of the lead's sentence.** §2 takes "in relation to the base" as an all-time
   ratio and finds a relabelling. If the lead meant the size only, §4 is the answer and the
   law is a second choice the sentence does not make.
2. **The star densities are from memory**, and r3, r4, r5 and r9 rest on them.
3. **The fit is the sibling's 14 numbers with an acoustic point**, not a likelihood; the
   `(w₀, wₐ)` distances assume a correlation.
4. **The growth numbers are a two-line ODE.** The bound `|P| < 0.063` is the other note's
   and rests on an ISW amplitude quoted from memory.
5. **The look-elsewhere prior** (log-uniform on `[10⁻⁴, 1]`) is a choice; uniform gives a
   different expected count and the same conclusion.
6. **The stake's two laws were chosen after seeing which fit.** A reviewer may fairly ask
   for all 23 × 3 to be staked, with the count of passes expected by chance stated.
7. **Not done:** the interacting case (the static portion feeding the organised one), and a
   ratio of rates in place of a ratio of stocks, which is closer to "the change to record".
