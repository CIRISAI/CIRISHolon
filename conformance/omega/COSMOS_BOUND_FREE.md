# COSMOS, BOUND AND FREE — the organised portion split by whether it can be held. Priced as a misfit campaign, 2026-11-23

*Written on the lead's word ("Yep, pursue it, makes sense right?") after this exchange: would
the organised portion of dark energy follow matter, or stay where it was created? "It seems
like a portion would follow matter and a portion may need to stay where it was created? I
guess gravity would have something to say about that, nothing can pin to DM AFAWK." The
lead's agent answered that general relativity sorts it by how fast disturbances travel: a
BOUND part that sits in the wells with matter and a FREE part that cannot be held and
becomes smooth. This note prices that answer. Nothing is frozen. No dataset was read. Every
number is printed by `cosmos_bound_free.py` (numpy, single thread, under a second), which
imports the grid, the star-formation history and the solver from `cosmos_two_portion.py`.
It is ORDER-OF-MAGNITUDE LINEAR THEORY and it has had no adversarial review.*

*Two things are NOT computed here, on purpose, because a reviewer is computing them beside
this note; each appears below as a named input:*
**(R1)** *the crossing redshift of a record written at the star-formation rate and then
diluting, and whether it is fixed by the star-formation history alone;*
**(R2)** *the sign and size of the ISW response to a clustering portion, with the published
all-sky amplitude in place of the one `COSMOS_CMB_CHANNEL.md` quoted from memory.*

## 0. Verdict, first

1. **The split follows from general relativity as stated** (§1). A portion with `w = −1`
   exactly has no gradient and no "where". A changing portion clusters or not according to
   its sound speed: at `c_s² = 1` its contrast on the scales the void measurements use is
   `2.6 × 10⁻⁵` to `4.2 × 10⁻⁴` of matter's; at `c_s² ≤ 10⁻⁶` it follows matter.
2. **But nothing that structure formation actually writes can be either part** (§2).
   Everything that escapes is radiation (`w = 1/3`), everything that stays is matter or heat
   (`w ≥ 0`), all of it dilutes FASTER than a constant, and all of it together is
   `5.2 × 10⁻⁶` of critical today: `7.6 × 10⁻⁶` of dark energy. The free part, if it is dark
   energy, is not made of anything in the inventory. It has to be a smooth negative-pressure
   field whose AMOUNT is set by what is written. Said without softening: that is an
   interacting-dark-energy model with a chosen coupling (§3, §8).
3. **A source alone gives the wrong sign.** A constant plus a field that only accumulates
   is phantom at every epoch (`w = −1.017` today at a share of `0.186`, `−1.041` at
   `0.461`) and never rises above `−1`; DESI DR2 wants `w₀ > −1`. The static portion does
   not evade the sibling's obstruction; the source does not evade it alone. What evades it
   is a source AND a loss together, and then the crossing epoch is where they balance:
   input R1 (§3, §7).
4. **The source has to be paid, and each payer is constrained** (§3). If matter pays, the
   matter density inferred early exceeds the late one by 6.6 %, 13.7 % and 16.4 % at shares
   of `0.186`, `0.386` and `0.461`; if the baryons pay, 42 %, 88 % and 105 % of them are
   gone. If the static portion pays and the field does not dilute, the sum is a constant:
   a relabelling. With no payer it is not general relativity.
5. **The bound fraction is a fork, not a derivation** (§4). Four rules for "what fraction
   of the record stays bound", written before the division, give `0.0095`, `0.989`, `0.666`
   and `1.000`. Data allow at most `0.14` to `0.33`. One rule lands inside with no tuning,
   three land outside at every share. The one that lands inside (by energy today) puts the
   bound part 4 to 13 times under today's reach.
6. **The free part's own observable is the growth rate, not the ISW effect** (§5). A smooth
   evolving portion at the shares `COSMOS_BASE.md` fixed lowers `fσ₈` by 2.3 to 3.1 % at
   `z = 0.3` to `0.5`: `0.3` to `0.4` of a standard error today, about 2 for one bin of the
   final DESI survey, about 3 for one bin of Euclid (forecast errors from memory). It moves
   the ISW amplitude by 3 to 8 %, which is `0.3` to `0.6` of a standard error even at the
   cosmic-variance limit: the ISW effect can never read the free part. Point (4) of the
   lead's agent's answer was wrong on this.
7. **The haze is a bound record of ordinary matter** (§6): `10⁵⁵` to `10⁵⁶` erg, one part
   in 600 to 7,600 of the dark energy inside its own volume, and `10⁻²¹` of dark energy when
   averaged over galaxies. No diffuse unexplained background is a candidate either: all are
   radiation and 5 to 11 orders under.
8. **One stake is freezable** (§8): with the share FIXED, "evolving AND smooth" predicts a
   definite pair, a growth-rate deficit and a void-lensing amplitude of one, and "evolving
   AND following matter" predicts a different one.

Against the four questions of the brief: (i) follows from general relativity, YES; (ii) a
derived bound fraction, NO; (iii) evades the obstruction, NOT by the static portion and not
by the source alone, ONLY by a source with a loss, pending R1; (iv) an observable for the
free part at a stated significance and date, YES in kind (the growth rate at about two to
three standard errors a bin in the final DESI and Euclid samples) and NO at the size the
mechanism can pay for.

## 1. The theorems

**(a) `w = −1` exactly means uniform.** For `T^{μν} = −ρ g^{μν}`, conservation gives
`∇_μ T^{μν} = −∂^ν ρ = 0`. So `ρ` is the same everywhere and always. The static portion
neither follows matter nor stays where it was made. It has no "where".

**(b) A fluid does not cluster below its sound horizon.** For a fluid with equation of state
`w` and rest-frame sound speed `c_s`, pressure gradients oppose collapse on scales smaller
than the distance sound travels in a Hubble time, `c_s/(aH)`. Below it, quasi-statically,
`δ_F/δ_m ≈ (1 + w)(3/2) Ω_m (aH/c_s k)²`, capped at `(1 + w)`, its value for `c_s = 0`.
With `w = −0.8`:

| `c_s²` | sound horizon `k_s` (`h`/Mpc) | effective bias at `k = 0.005` | at `0.02` | at `0.1` |
|---:|---:|---:|---:|---:|
| `1` | `3.3 × 10⁻⁴` | `4.2 × 10⁻⁴` | `2.6 × 10⁻⁵` | `1.1 × 10⁻⁶` |
| `10⁻²` | `3.3 × 10⁻³` | `0.042` | `2.6 × 10⁻³` | `1.1 × 10⁻⁴` |
| `10⁻⁴` | `0.033` | `0.20` | `0.20` | `0.011` |
| `10⁻⁶` | `0.33` | `0.20` | `0.20` | `0.20` |

The void and ISW measurements use `k` of about `0.005` to `0.05 h`/Mpc. A portion with
`c_s² = 1` is smooth to a part in ten thousand there; one with `c_s² ≤ 10⁻⁴` follows
matter. And the largest contrast any portion can have is `(1 + w)` of matter's: the nearer
to `w = −1`, the less it can clump at any sound speed. The sibling's cuscuton is the limit
`c_s² = ∞`: exactly smooth at every scale, with no propagating mode.

**(c) "Created in a galaxy, then free."** By the equivalence principle whatever is made in a
galaxy starts with the galaxy's motion. What happens next depends only on its own speed.
Cold (velocity dispersion under the escape speed): it stays in the well. That is "follows
matter", and nothing is pinned to the dark matter; the portion shares the well, as the
baryons do. Relativistic: it leaves at the speed of light, fills the horizon uniformly
within a crossing time, and redshifts as `a⁻⁴`. Negative pressure with `c_s ≈ 1`: it cannot
be localised at all; a local excess disperses at the sound speed, so "where it was created"
is forgotten within a light-crossing time of the region. **"Stays where it was created" is
therefore not an option for a free portion. The two options are "held in the well" and
"everywhere".**

## 2. The inventory: what structure formation writes, and where it goes

Fukugita and Peebles, "The Cosmic Energy Inventory" (ApJ 616, 643, 2004; astro-ph/0406095),
Table 1. **All values FROM MEMORY of that table; the paper was found by search and not
re-read.** The two heats are computed here from a density and a temperature.

| item | of critical | `w` | where it goes | dark energy? |
|---|---:|---:|---|---|
| starlight and reprocessed infrared | `2.0 × 10⁻⁶` | `1/3` | FREE | no: `a⁻⁴` |
| stellar neutrinos (mostly core collapse) | `3.2 × 10⁻⁶` | `1/3` | FREE | no: `a⁻⁴` |
| gravitational waves, massive black-hole mergers | `3.2 × 10⁻⁸` | `1/3` | FREE | no: `a⁻⁴` |
| X and gamma rays | `1.3 × 10⁻⁸` | `1/3` | FREE | no: `a⁻⁴` |
| gravitational waves, stellar binaries | `10⁻⁹` | `1/3` | FREE | no: `a⁻⁴` |
| heat of intracluster gas | `2.4 × 10⁻⁸` | `≈ 0` | BOUND, in the wells | no |
| heat of the warm-hot intergalactic gas | `1.1 × 10⁻⁸` | `≈ 0` | BOUND, in the filaments | no |
| kinetic energy of the intergalactic medium | `10⁻⁸` | `0` | BOUND | no |
| cosmic rays and magnetic fields | `5 × 10⁻⁹` | `1/3` | BOUND, to galaxies | no |
| white dwarfs, neutron stars, black holes (rest mass) | `4.8 × 10⁻⁴` | `0` | BOUND | no: matter |
| nuclear binding released by stars | `6.3 × 10⁻⁶` | — | the source of the light | a deficit |
| binding released by dissipative settling | `1.3 × 10⁻⁵` | — | the source; mostly compact objects | a deficit |

Totals: free and radiation-like today, `5.2 × 10⁻⁶`; bound as energy, `5.0 × 10⁻⁸`; bound
as remnant rest mass, `4.8 × 10⁻⁴`; all binding ever released, `1.9 × 10⁻⁵`. Against dark
energy: the free total is `7.6 × 10⁻⁶` and everything ever released is `2.8 × 10⁻⁵`.

**The obvious problem.** Every free item has `w = 1/3` and every bound item `w ≥ 0`. Each
dilutes faster than a constant, and the lot is five orders under dark energy. Nothing
ordinary that structure formation writes can be the organised portion, bound or free. This
is the same finding as the writer's budget in `COSMOS_TWO_PORTION.md` §9, reached from the
other side: there, the writer could not pay; here, nothing it emits has the right equation
of state even if it could.

## 3. The only consistent form

A smooth component with negative pressure (`c_s ≈ 1`, or the sibling's cuscuton) whose
amount is set by the writing rate:

`dρ_F/dt + 3H(1 + w_F) ρ_F = Q(t)`,  `Q = α c² ψ(t)`,

with `ψ` the cosmic star-formation rate (Madau and Dickinson 2014, equation 15, the form the
record already uses) and `α` a pure number: grams of field per gram of star formed.

**(a) Size.** All stars ever formed are `8.1 × 10⁸` solar masses a cubic megaparsec:
`0.0064` of critical (mean formation redshift `1.50`). With `w_F = −1` the share today is
`α × 0.0064/0.685`:

| `α` | share |
|---|---:|
| `0.007` (hydrogen burning) | `6.5 × 10⁻⁵` |
| `Ω_b/Ω_dm = 0.186` (the base ratio used as a coupling) | `1.7 × 10⁻³` |
| `1` (the whole rest mass) | `9.4 × 10⁻³` |
| needed for a share of `0.186` / `0.386` / `0.461` | `α = 19.9` / `41.3` / `49.3` |

So the shares that fit the shape need twenty to fifty grams of field for each gram of star.
**`α` is chosen, not derived.** The base ratio used as a coupling gives two parts in a
thousand, which no instrument reads.

**(b) Sign.** With `w_F = −1` and a positive source the dark energy only grows:

| share today | `w(0)` | `w(0.5)` | `w(1)` | crossing |
|---:|---:|---:|---:|---|
| `0.186` | `−1.017` | `−1.038` | `−1.058` | none |
| `0.461` | `−1.041` | `−1.099` | `−1.163` | none |
| `1` | `−1.088` | `−1.240` | `−1.476` | none |

Phantom at every epoch, and never above `−1`: the opposite of DR2's sign today.

**(c) The sink, if matter pays.** Then `dρ_m/dt + 3Hρ_m = −Q` and matter no longer dilutes
exactly as `a⁻³`. What was taken at `a_e` would be `a_e³` of itself today; the weighted
mean of `a_e³` over the star-formation history is `0.164`. So matter is missing today by:

| share | of critical | of all matter | of the baryons, if they pay | of the dark matter, if it pays |
|---:|---:|---:|---:|---:|
| `0.186` | `0.021` | `6.6 %` | `42 %` | `7.9 %` |
| `0.386` | `0.043` | `13.7 %` | `88 %` | `16.4 %` |
| `0.461` | `0.052` | `16.4 %` | `105 %` | `19.5 %` |

The matter density inferred from the microwave background would exceed the late one by the
third column. If the baryons pay, most of them are gone, against a baryon census that
(FROM MEMORY: big-bang nucleosynthesis and the microwave background against the dispersion
of fast radio bursts) closes to about ten or twenty per cent. If the dark matter pays, it
is a decaying dark matter, bounded (FROM MEMORY) at a few per cent since recombination.
Published fits of interacting dark energy to DR2 exist in other coupling forms (arXiv
2601.03122, 2503.21652, 2504.00985, 2609.03062; read as search summaries only): they find a
weak, form-dependent preference for some transfer, and the last of them finds it weakens
when `w` may evolve. No bound was found in the form `Q ∝ ψ`.

**(d) The sink, if the static portion pays.** Then `d(ρ_static + ρ_F)/dt = −3H(1 + w_F)ρ_F`.
With `w_F = −1` the sum is a constant and both parts are uniform: nothing is added to a
constant. A relabelling, as `COSMOS_BASE.md` §3 found for the strict base reading and the
review found for the horizon pricing. And a "static" portion that pays is not static.

**(e) No sink.** The stress-energy is not conserved. Not general relativity.

**(f) What the shape needs.** `1 + w_eff = share × (1 + w_F) − Q/(3Hρ_DE)`. The first term
is the fall, the second the rise. The crossing is where `Q = 3H(1 + w_F)ρ_F`. The record
already holds one case: with the field diluting as matter (`w_F = 0`) the crossing is
`z = 0.818` at a share of `0.186` (`COSMOS_BASE.md` §4). **Its dependence on the dilution
exponent, and whether it is independent of the amplitude, is input R1, owed from the
review.** Note what a matter-like field costs: by §1(b) a field with `w_F = 0` and
`c_s = 1` is smooth but dilutes as matter, and one with `w_F = 0` and `c_s = 0` IS matter.

## 4. The bound fraction

With a share `f`, a bound fraction `β` that follows matter with the hosts' bias `b`, the
perturbation channel reads `P = f·β·b`. `COSMOS_CMB_CHANNEL.md` §3 bounds `|P| < 0.063` at
two standard errors from the all-sky ISW amplitude, a value quoted there from memory
(input R2 may move it). That allows:

| share | `β` at `b = 1` | at `b = 2` | at `b = 5` (clusters) |
|---:|---:|---:|---:|
| `0.19` | `≤ 0.33` | `≤ 0.17` | `≤ 0.066` |
| `0.39` | `≤ 0.16` | `≤ 0.08` | `≤ 0.032` |
| `0.46` | `≤ 0.14` | `≤ 0.07` | `≤ 0.027` |

Is a `β` that small derivable? Four rules for "the fraction of what is written that stays
bound", from the inventory of §2, written down before the division:

| rule | `β` | `P` at `f = 0.19 / 0.39 / 0.46`, `b = 1` | |
|---|---:|---:|---|
| (i) by energy today: bound heat, motion and cosmic rays over bound plus free radiation | `0.0095` | `0.0018 / 0.0037 / 0.0043` | ALLOWED |
| (ii) by energy today, counting remnant rest mass as bound | `0.989` | `0.19 / 0.39 / 0.46` | EXCLUDED |
| (iii) by binding released: what stayed as a deficit in compact objects over all released | `0.666` | `0.13 / 0.26 / 0.31` | EXCLUDED |
| (iv) by entropy (Egan and Lineweaver 2010, FROM MEMORY: black holes dominate by fourteen orders) | `1.000` | `0.19 / 0.39 / 0.46` | EXCLUDED |

**Said loudly, both ways.** Rule (i) lands inside the allowed range with no tuning: `0.0095`
against a ceiling of `0.14` to `0.33`. Rules (ii), (iii) and (iv) land outside it at every
share. So `β` is a second free number: one reading of four is allowed, and it is the one
that makes the bound part invisible. Under rule (i) the bound product is 4 to 13 times
under today's one-standard-error reach (`0.016` to `0.024`) and `0.9` to `3.3` times under
a future half-per-cent measurement. Rule (iv), the one nearest to "a record is a count of
bits", is the one most firmly excluded: by entropy the record is all in black holes, which
follow matter.

## 5. The free part's own observable

A smooth evolving portion acts on growth, lensing and the ISW effect only through `H(a)`.
For the `(w₀, wₐ)` each candidate of `COSMOS_BASE.md` §4 implies, against a constant with
the same `Ω_m`, `H₀` and early amplitude, in linear theory:

| candidate | `(w₀, wₐ)` | `σ₈` | `fσ₈` at `z = 0.3 / 0.5 / 0.8 / 1.0` | CMB lensing power | ISW amplitude at `0.3 / 0.5 / 1.0` |
|---|---|---:|---|---:|---|
| A: `Ω_m/Ω_DE`, rent ledger | `(−0.768, −0.739)` | `−0.37 %` | `−3.08 / −2.29 / −0.70 / +0.12 %` | `+0.16 %` | `1.067 / 1.082 / 1.038` |
| `Ω_dm/Ω_DE`, rent ledger | `(−0.804, −0.622)` | `−0.30 %` | `−2.60 / −1.93 / −0.59 / +0.10 %` | `+0.16 %` | `1.056 / 1.070 / 1.033` |
| B: `Ω_b/Ω_dm`, star-written, diluting | `(−0.863, −0.347)` | `−0.79 %` | `−2.65 / −2.32 / −1.46 / −0.95 %` | `−0.66 %` | `1.046 / 1.063 / 1.069` |
| `Ω_b/Ω_dm`, rent ledger | `(−0.904, −0.303)` | `−0.12 %` | `−1.24 / −0.91 / −0.26 / +0.07 %` | `+0.14 %` | `1.028 / 1.034 / 1.017` |
| DR2's own point with DESY5, for scale | `(−0.752, −0.86)` | `+0.01 %` | `−2.69 / −1.73 / +0.04 / +0.88 %` | `+0.70 %` | `1.066 / 1.076 / 1.005` |

In standard errors (errors FROM MEMORY except the ACT lensing amplitude; forecasts are
the collaborations' stated aims as recalled, not read):

| candidate | `σ₈` today (4 %) | `fσ₈` today (8 % a bin) | final DESI (1.5 %) | Euclid (1 %) | lensing today (2.3 %) | SO (1 %) | CMB-S4 (0.5 %) | ISW today (25 %) | ISW at the cosmic-variance limit (13 %) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| A | `0.09` | `0.39` | `2.05` | `3.08` | `0.07` | `0.16` | `0.33` | `0.33` | `0.63` |
| `Ω_dm/Ω_DE` | `0.08` | `0.32` | `1.73` | `2.60` | `0.07` | `0.16` | `0.33` | `0.28` | `0.53` |
| B | `0.20` | `0.33` | `1.77` | `2.65` | `0.29` | `0.66` | `1.33` | `0.25` | `0.49` |
| `Ω_b/Ω_dm`, ledger | `0.03` | `0.16` | `0.83` | `1.24` | `0.06` | `0.14` | `0.27` | `0.14` | `0.26` |

**Reading.** (1) The ISW effect cannot read the free part, now or ever: half a standard
error at the cosmic-variance limit. The large angles are where a CLUSTERING portion shows
(`COSMOS_CMB_CHANNEL.md`); a smooth one is nearly invisible there. (2) CMB lensing cannot
either, except B at CMB-S4, marginally. (3) The growth rate at `z = 0.3` to `0.5` is the
one handle: 2 to 3 % low, about two standard errors a bin in the final DESI sample and
about three in Euclid's; several bins together would do better, and that combination is not
computed here. (4) The background reads it first and best: `COSMOS_BASE.md` §9 puts A and B
at 2.7 to 4.2 of DR2's standard errors from a constant. What the growth rate adds is a
CONSISTENCY relation: for a smooth portion the growth deficit is fixed by the background and
carries no new freedom. A measured deficit larger or smaller than this is a bound part, or
a sink in matter.

## 6. The haze, and a screen of the diffuse backgrounds

**The haze** (the microwave excess toward the Galactic centre, WMAP then Planck, co-spatial
with the Fermi gamma-ray bubbles): conventionally hard-spectrum synchrotron from energetic
electrons inside bubbles blown by a past outburst or wind from the centre (arXiv 1005.5480;
1505.03634; 2203.02526; read as search summaries). Its energy is estimated from
`7.7 × 10⁵⁴` erg (a wind) to `10⁵⁶` (an explosion), with one jet model at `2 × 10⁵⁷`. The
dark energy inside the bubbles' own volume (two lobes of 5 × 3 × 3 kpc, at `3.28` keV a
cubic centimetre) is `5.8 × 10⁵⁸` erg: the bubbles hold one part in 7,600, 580 or 29 of it.
One such pair a galaxy, averaged over space, is `10⁻²²` to `10⁻²¹` of dark energy. It is a
BOUND record of ordinary matter and fields: an illustration of "written, and stayed in the
well". Nothing is claimed from it.

**The diffuse backgrounds that are not fully explained**, as candidates for a free written
record (values FROM MEMORY):

| background | size | of dark energy | `w` |
|---|---:|---:|---:|
| radio synchrotron excess (ARCADE 2, about `1.15` K at 1 GHz) | `10⁻³` nW m⁻² sr⁻¹ | `8.5 × 10⁻¹¹` | `1/3` |
| optical excess (New Horizons: 8 ± 2 nW claimed in 2022, about 3 ± 2 after the 2024 revision) | `3` nW | `2.4 × 10⁻⁷` | `1/3` |
| the whole optical and infrared background, for scale | `80` nW | `6.4 × 10⁻⁶` | `1/3` |
| the unresolved gamma-ray background | `8 × 10⁻³` nW | `6.4 × 10⁻¹⁰` | `1/3` |

All are radiation, diluting as `a⁻⁴`, and 5 to 11 orders under dark energy. None is the
free portion. This is a screen, not a claim, and its scope is these four.

## 7. Against the sibling's preferred-frame verdict

`CIRISOntology/scratchpad/preferred-frame-results.md` (2026-07-24) is the prior work this
note stands on, and it is credited: the smooth half is the cuscuton (Afshordi, Chung, Doran
and Geshnizjani 2007), with `ρ = V(φ)`, an infinite sound speed and no propagating mode; and
a single one can never cross `w = −1`, because `1 + w = ε μ²|φ̇|/V` has the sign of a
discrete choice. "The crest and the crossing are the same event."

- **Confirmed here:** the free part, as general relativity allows it, IS that smooth
  sector. §1 reaches the same object from the other side.
- **What a static portion changes: nothing.** By `1 + w = share × (1 + w_F)` a static
  portion scales the departure from `−1` and cannot change its sign. A static portion plus
  one cuscuton is sign-locked exactly as one cuscuton is.
- **What a source changes: the obstruction's premise.** The obstruction assumes the field's
  density is a function of the field alone, so it is monotone. A sourced field's density
  rises while it is written and falls when the writing stops, if the field also loses. Then
  the EFFECTIVE `w` (read from how the density changes) crosses `−1` while the field's own
  `1 + w_F` keeps one sign. So the crossing is possible with one smooth field and a source.
- **The price is the one the sibling named, moved and made smaller.** Two opposed cuscutons
  made the crossing epoch a free input. Here it is set by where the source equals the loss:
  the star-formation history is MEASURED, so one input is gone, but the loss rate (the
  dilution exponent) remains, unless R1 shows the crossing does not depend on it.
- **Contradicted: nothing.** Its U4 (computed honestly, the per-unit balance also rises and
  falls) and its U8 (DR2 wants the crossing at `0.37` to `0.50`, not `0.59`) both stand and
  both bear on R1.
- **Stance keys.** `dark-balance-intensive` (smooth, never phantom) is the free part with
  no source. `dark-balance-extensive` (a running total that crested) is the free part with
  a source and a loss; its own text says smoothing it needs a preferred-frame sector.
  `dark-medium` ("the paper the books are written on") is the payer of §3(c): if the dark
  matter pays, it is being written ON, at a cost of 8 to 20 % of it. `gravity-audit` is
  marked DEAD in the stance, so nothing here leans on it.

## 8. The stake, and what is left that is ours

**The stake** (not frozen; the bars in brackets are named blanks and come from DR2).
Carrier: DESI DR3's direct fit for the background, with the share FIXED at A
(`Ω_m/Ω_DE` under the rent ledger) or B (`Ω_b/Ω_dm` under the star-written law) as
`COSMOS_BASE.md` §9 writes them; one void catalogue fixed in advance for the lensing
amplitude `A_κ` (as `COSMOS_CMB_CHANNEL.md` §7); one growth-rate measurement at
`z = 0.3` to `0.5`.

The separation, on paper:

| reading | background | `fσ₈` at `z = 0.3` | void lensing `A_κ` at `z = 0.7` |
|---|---|---:|---:|
| a smooth constant | `(−1, 0)` | `0` | `1.000` |
| A, wholly free | `(−0.768, −0.739)` | `−3.08 %` | `1.000` |
| A, bound fraction `0.0095` (rule (i)) | same | same to first order | `1.002` |
| A, bound fraction `0.3` | same | — | `1.061` (`+1.1σ` today, `+3.1σ` at a future `± 0.02`) |
| A, wholly bound | same | — | `1.204` (`+3.8σ` today) |
| B, wholly free | `(−0.863, −0.347)` | `−2.65 %` | `1.000` |
| B, wholly bound | same | — | `1.082` (`+1.5σ` today, `+4.1σ` at `± 0.02`) |

Branches, exactly one per outcome, on `t_bg` (evolution preferred over a constant, in
standard errors), `t_κ = |A_κ − 1|/σ_κ` and the growth deficit `g`:

- **(a) CONSTANT** — `t_bg < [2]`: no organised portion of readable size; A and B are dead
  at their fixed shares.
- **(b) FREE** — `t_bg ≥ [3]`, the fixed-share fit within `[+2]` of the general form,
  `t_κ < 2`, and `g` within two standard errors of the smooth prediction: an evolving,
  smooth portion is read at a fixed share. The bound fraction is bounded at `2σ_κ/ε`.
- **(c) BOUND** — `t_bg ≥ [3]` and `t_κ ≥ 3` with `A_κ > 1`: a portion that follows matter
  is read, and `P` is measured. It must then meet the all-sky ISW amplitude (R2) or it is
  dead on its own channel.
- **(d) NOT THIS** — `t_bg ≥ [3]`, `t_κ < 2`, and `g` off the smooth prediction by three
  standard errors or more: the evolution is real and the portion is not a smooth,
  uncoupled one. A sink in matter (§3(c)) is the first suspect.
- **(e) BETWEEN** — every other case. Nothing moves.

**Kill of the bound/free picture as a whole:** (c) with `A_κ < 1` (a portion that AVOIDS
matter at a large share), which neither a bound nor a free part gives.

**What this is, named.** A smooth negative-pressure field, sourced at the star-formation
rate with a chosen amplitude and a chosen payer, is an INTERACTING DARK ENERGY model with
coupling `Q = α c² ψ(t)`. Prior art, to be credited:

- **Cosmologically coupled black holes** (Croker and Weiner 2019; Farrah, Croker and
  others 2023, arXiv 2302.07878; Croker and others 2024, "DESI Dark Energy Time Evolution
  is Recovered by Cosmologically Coupled Black Holes", arXiv 2405.12282): dark energy
  sourced at the star-formation rate through black-hole production, not diluting, with the
  baryons as payer, and a claimed fit to DESI with fewer parameters than `w₀wₐ`. That is
  §3 with `w_F = −1` and the baryons paying. Its mechanism is disputed (Mistele's comment,
  arXiv 2304.09817; gravitational-wave bounds, Amendola and others). Read as search
  summaries only. By §3(b) a non-diluting sourced component alone is phantom throughout,
  so how that paper matches `w₀ > −1` was not checked.
- **Holographic dark information energy** (Gough 2008, 2011): a dark-energy density from
  the information held by stars and structure, credited already in the sibling's
  `PRIOR_ART_SWEEP.md`.
- **Interacting dark energy** in general, and its DR2 fits (the four arXiv numbers in §3).
- **The cuscuton** and the spatially covariant crossing models (arXiv 2508.01378), via the
  sibling's study.

**What is left that is ours:** not the mechanism. (1) The split read as a theorem and
priced on both channels at once, with the bound fraction shown to be a fork. (2) The
consistency relation of §5 as a stake with the share fixed. (3) The observation that a
source evades the sibling's obstruction where a static portion does not. (4) The two-portion
reading of the lower tiers (`OBJECT.md`, CORRECTION 2026-11-23), which this note neither
supports nor harms: the cosmos stays a wager.

## 9. Misfits contacted

- **M-RETRODICTION-AS-PREDICTION:** the shares, both laws and the picture itself were
  written after DR2. Only DR3 and future void and growth measurements are prospective.
- **M-POPULATION-CHOICE:** four rules for `β` (§4), named; a fifth would move it again.
  Three coupling forms and three payers (§3), named.
- **M-KILL-DISSOLVED:** adding a free bound fraction lets any void-lensing result be
  absorbed in branches (b) and (c). The kill kept is the sign (`A_κ < 1`).
- **M-BOUND-HYPOTHESIS-UNMET:** the `0.063` ceiling rests on an all-sky amplitude quoted
  from memory and on a quasi-static formula (R2).
- **M-SORTS-NOT-SEPARATES:** the growth deficits of A and B differ by `0.4 %` at
  `z = 0.3`; the growth rate separates either from a constant and not A from B.
- **M-FOREIGN-DOMAIN-CORROBORATION:** the haze is a different kind of record; nothing is
  taken from it.
- **M-SCREEN-SCOPE-UNSTATED:** searched: both repositories' files named above, and five web
  searches on 2026-11-23 (the energy inventory; cosmologically coupled black holes;
  interacting dark energy after DR2; the bubbles and backgrounds; forecasts). Not searched:
  backreaction and averaging; structure-coupled quintessence; chameleon and symmetron
  fields; any paper's full text.
- **M-ONE-MODEL-DELTA:** "not this" in branch (d) is a defect against one smooth model.
- **M-BASE-RATE-OMITTED:** forecast errors are per bin and from memory; the combined
  significance over bins is not computed.

## 10. Open to the reviewer

1. **§5's linear growth holds `Ω_m` and `H₀` fixed.** A real fit holds the acoustic angle,
   and `H₀` moves; the growth deficit could change by a factor of order one. Check first.
2. **The inventory is from memory.** Every row of §2 should be read against Fukugita and
   Peebles's table; rule (i)'s `0.0095` rests on two heats computed here from round numbers.
3. **Rule (i) may be the wrong numerator.** It counts thermal and kinetic energy as "bound
   record" and remnants as not record at all. A reader who counts remnants gets rule (ii)
   and an exclusion. The fork is real; which rule the wager means is the stance's to say.
4. **§3(b) versus Croker and others 2024.** If a non-diluting sourced component is phantom
   throughout, either their fit is to a different sign than DR2's summary, or their model
   has a loss this note does not have. Not checked.
5. **The matter deficits of §3(c) weight by the star-formation history under a constant
   background.** A share of `0.46` changes the background enough to move `0.164`.
6. **§1(b)'s formula is quasi-static and ignores the field's own anisotropic stress and
   the time dependence of `w`.** Near the sound horizon it is good to a factor of two.
7. **The forecasts and current errors in §5 are recalled, not read.** The two-to-three
   standard errors a bin could be one or five.
8. **R1 and R2 are not in this note.** If R1 finds the crossing fixed by the star-formation
   history alone, §7's "price" loses its last free input and the stake of §8 gains a
   predicted redshift. If R2 moves the all-sky ceiling, §4's allowed `β` moves with it.
