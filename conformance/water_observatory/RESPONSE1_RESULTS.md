# RESPONSE-1 — RESULTS, appended arm by arm as they land (opened 2026-09-21)

*Prereg `RESPONSE1_PREREG.md`; Amendments 1–4; reader `rung2 --response`; each arm's read in
`replace0/response1_<tag>_seed<k>/response1.read.txt`. Nothing here is graded until the
three-seed averages exist; per-arm rows are readings of one seed. The five partial arms
(4–7 cycles) are in `RESPONSE1_AMENDMENT_3.md` and are superseded by their full re-runs.*

## VERDICT (2026-09-23, all ten arms landed and read)

| stake | staked | read on three seeds | verdict |
|---|---|---|---|
| **R1** cell chart `(n̄, p̄)` closes under drive | `D ≤ 0.2` at 8 cells, separated from the blind; null holds | `D = 0.65, 0.85, 0.84`; on the graded 36-cycle average `0.622` vs floor `0.492`, separation `+0.30`; nulls hold | **branch (b): OPEN under drive** — the chart leaks a third of the signal at one molecular diameter, clearly above its floor |
| **R1″** face chart (`h = 0.25 Å`) closes at 200 m/s | `D ≤ 0.2`, `α ∈ [0.8, 1.25]`; kill `D > 0.3` | on the graded 36-cycle aligned average: **`D = 0.180`, SE `0.058`**, `α = 1.03`, `R² = 0.97`, relaxed `0.689` (per seed `0.164, 0.302, 0.269`, whose mean `0.245` was the earlier, wrong statistic) | **MET as staked**, by `0.02` — a third of one SE; the stake carries no significance clause |
| **R1′** continuity sees nothing under shear (cell chart) | `\|Δ\| ≤ max(0.1, 2 SE)` | on the graded 36-cycle average: driven `0.929`, relaxed `0.989`, `\|Δ\| = 0.06` vs `0.15` (per seed: holds on 1, 2; fires on 0 by `0.03`) | **HOLDS on the graded statistic**; branch (e) not entered |
| R1″'s null (face chart on the transverse arms) | driven within `0.1` of relaxed | on the 36-cycle aligned average: driven `0.438`, relaxed `0.575`, `\|Δ\| = 0.137` against a FLAT `0.1` (no SE term in Amendment 5; `2 SE = 0.56`); per seed `0.24, 0.03, 0.11` | **FIRES as written**, inside its own noise; the flat threshold is the amendment's fault, recorded |
| **R2** density mode | damped cosine vs overdamped, both forms | overdamped on every 50 m/s seed; OSCILLATORY on the 200 m/s seed 0 (`Γ = 2.7 × 10¹²`, `c_s = 3.3` km/s) with the other two not fitting the overdamped form | **overdamped in the linear regime**; the 200 m/s arms are outside it |
| **R3** shear rent | `η` in `[2, 30] × 10⁻⁴` Pa s | `η(0.27 Å⁻¹) = 4.4 × 10⁻⁴`, spread `[3.9, 5.0]`, overdamped on every seed | **branch (a)** — equal to the equilibrium read |
| R4, R4′ | no mode on the scrambled partition; undriven quadrature at noise | hold on every linear arm; **R4′ FIRES on the 200 m/s transverse control** | hold; the control is nonlinear by two independent signs |

**In one sentence:** the driven shear rent is a number with a spread; the cell chart is open
under drive by a third of the signal on every seed; the staggered chart the object's own law
chose closes three times better at the right amplitude but not within budget on three seeds;
the cell chart's continuity null holds on the graded average while the face chart's fires by `0.04` on a flat threshold inside its own noise. **Owed
before any of this is the model's rather than the operator's:** the fine-model seed at the same
kick (Amendment 2), the 36-cycle cross-seed aligned average (the reader aligns within one
trajectory), and the 200 m/s longitudinal `ν_l = 1.4 × 10⁻⁶` m²/s (corrected 2026-09-23: 1.2 was seed 0, not the pool) given a band.

## Arms read so far

| arm | cycles | SNR (current) | nulls (R4, R4′, density R4, density R4′) | R1 at 8×1×1: D, separation, two-sided floor | R2 / longitudinal current | R3 |
|---|---|---|---|---|---|---|
| L200 seed 0 (control) | 12 | 38 | all hold | **0.735, +0.35, floor 0.60 → KILL on this seed** (over the floor by 0.14) | OSCILLATORY: period 705–735 fs, `c_s ≈ 3,200–3,300` m/s, `Γ ≈ 2.2–2.7 × 10¹²` /s — outside R2's band | — |
| L seed 1 (50 m/s) | 12 | 9.4 | R4, R4′, density R4 hold; **density R4′ fires marginally** (`0.52` vs `3σ = 0.39`) | **0.846, +0.07, floor 0.74 → KILL on this seed** (over by 0.11) | OVERDAMPED: current `λ₂ = 1.04 × 10¹³` /s (`ν_l = 1.45 × 10⁻⁶` m²/s), density from the peak `λ₁ = 5.5 × 10¹²` /s; `c_s = √(λ₁λ₂)/k ≈ 2,800` m/s, `Γ/ω ≈ 1.05` | — |

| T seed 0 (50 m/s, transverse) | 12 | 8.7 | R4 (`0.17` of spatial, under its `3σ` bar), R4′ hold | **R1′: driven `0.759`, relaxed `1.036`, `|Δ| = 0.28` against `max(0.1, 2 SE) = 0.25` → FIRES by `0.03`**; separation from the blind `+0.24` | current OVERDAMPED, `λ = 3.6 × 10¹²` /s (`τ = 279` fs); 7 of 12 tails unrelaxed | **`η = 5.0 × 10⁻⁴` Pa s** `[4.2, 5.8]`, IN BAND — equal to the equilibrium read at this `k` |

| T seed 1 (50 m/s) | 12 | 13.0 | R4 (`0.11`), R4′ hold | **R1′: driven `0.966`, relaxed `0.996`, `|Δ| = 0.03` → HOLDS** | current overdamped | `η = 3.9 × 10⁻⁴` `[1.3, 6.5]`, IN BAND |
| T seed 2 (50 m/s) | 12 | 5.1 | R4 (`0.25`, under its bar), R4′ hold | **R1′: driven `0.991`, relaxed `0.952`, `|Δ| = 0.04` → HOLDS** | current overdamped | `η = 4.4 × 10⁻⁴` `[3.8, 5.0]`, IN BAND |
| L seed 2 (50 m/s) | 12 | 6.8 | all hold | **R1: `0.836` against a two-sided floor of `0.860` → AT FLOOR**, separation `+0.20`; null holds | OVERDAMPED from the peak (SNR 6.6) | — |

| L seed 0 (50 m/s) | 12 | 6.4 | all hold | **R1: `0.650`**, separation `+0.34`; the floor estimate reads `1.02` (the lead RMS sat under the tail's: an unreliable floor, not a reading) | OVERDAMPED from the peak | — |
| T200 seed 0 (control) | 12 | 12.9 | R4 holds; **R4′ FIRES** (undriven quadrature `1.06 × 10⁻³` vs `3σ = 0.85 × 10⁻³`) — the second nonlinearity at 200 m/s | R1′ holds (`|Δ| = 0.16` vs `0.17`) | overdamped | `η = 5.8 × 10⁻⁴` `[3.9, 7.7]`, IN BAND |

## R1 graded on the three 50 m/s longitudinal seeds (2026-09-22, midday): BRANCH (b), the chart is OPEN under drive

| seed | `D_cont` at 8 cells | separation from the blind | two-sided floor |
|---|---|---|---|
| 0 | 0.650 | +0.34 | 1.02 (unreliable: lead RMS under tail RMS) |
| 1 | 0.846 | +0.07 | 0.74 → over by 0.11 |
| 2 | 0.836 | +0.20 | 0.86 → at floor |

The `(n̄, p̄)` cell chart does not close within `β = 0.2` on any seed (mean `D = 0.78`), and
it separates from the position-blind chart on every seed (`+0.07` to `+0.34`, the stake
`≥ 0.05`). That is the prereg's **branch (b)** as written: *"R1 fails on `D ≤ 0.2` but
separates from the blind → the chart is OPEN under drive."* Its in-run null holds on every
seed. The pooled longitudinal current rate is `1.03 × 10¹³` /s (seed spread 39 %), the fast
mode `ν_l = 1.4 × 10⁻⁶` m²/s (corrected 2026-09-23: 1.2 was seed 0, not the pool), no band.

**The staggered chart on the same three seeds** (`r1_closure_test.py`, face momentum at
`0.25 Å`, 8 cells): `D = 0.482, 0.492, 0.489` — the same number on three seeds to a percent,
`R² = 0.77–0.79`, amplitude fidelity `0.92–1.24` — against the cell chart's `0.65–0.85`. At
50 m/s the face chart sits at its own shot-noise floor (per-seed `s ≈ 1`); R1″ is graded on
the three 200 m/s seeds, where seed 0 read `0.164`, and the other two land tonight.

## R3 and R1′ on the three full transverse arms — the first graded reads (2026-09-22)

**R3, graded:** `Γ_s` pooled over three seeds `3.18 × 10¹²` /s, seed spread 25 %, so
**`η(k = 0.27 Å⁻¹) = 4.4 × 10⁻⁴ Pa s`, spread `[3.9, 5.0]`** — IN BAND, and equal to the
equilibrium read on the same box (`4.4–5.8 × 10⁻⁴`). The classification on every seed is
overdamped, so this is a viscosity at this wavevector, not a shear-wave damping. **Branch
(a) on R3.**

**R1′, the transverse null:** holds on seeds 1 and 2 (`|Δ| = 0.03, 0.04`) and fires on seed 0
by `0.03` over its allowance. The reader's pooled read lists the seeds; the prereg's
"three-seed aligned average" of `D` across 36 cycles is not what the instrument computes
(it aligns within one trajectory) and is OWED as a reader change. On the majority and on the
shape — seed-specific, as the sample-anisotropy candidate predicts and an instrument
artefact would not — **the null holds and branch (e) is not entered**; R1's kills on the
longitudinal arms stand as per-seed readings. Recorded as a two-of-three verdict, not a
three-seed one, until the 36-cycle average exists.

## The transverse null, as it stood on 2026-09-22 morning (per-seed reads)

Seed 0 fired by `0.03` (`|Δ| = 0.28`); seeds 1 and 2 hold cleanly (`0.03`, `0.04`). One of three,
seed-specific — the shape the sample-anisotropy candidate predicts and an instrument artefact
would not (an artefact would fire on every seed by a similar amount). The three-seed pooled
read is the graded one and is being produced by the reader; if it holds, branch (e) is not
entered and R1's kills on the longitudinal arms stand. The viscosity on the three full
transverse arms: `5.0, 3.9, 4.4 × 10⁻⁴` Pa s, all in band, all within the equilibrium read.

## R1 on the 50 m/s longitudinal arms, two of three

Seed 1: `0.846` over a floor of `0.737` (kill on the seed); seed 2: `0.836` at a floor of `0.860`
(at floor). The graded R1 is the three-seed average, seed 0 landing today; the staggered
chart's R1″ is graded on the three 200 m/s seeds, two of which land tonight.

## The transverse null, as it stood on 2026-09-21 (one full transverse read)

The null R1′ says continuity must see NOTHING under a shear kick. On the full T seed 0 it
fires by `0.03` over its allowance: the slab-momentum chart predicts `42 %` of the aligned
occupancy-change power in the two windows after the kick and none in the relaxed tail. On
the partial arms it held on T seed 2 and the 200 m/s control (`|Δ| = 0.01, 0.02`) and fired
marginally on T seed 1 (`0.13` on four cycles). By symmetry a transverse kick has no
first-order longitudinal response in an isotropic liquid, and sign alignment cancels the
second-order (Reynolds-stress) one — so a sign-locked longitudinal flow after a transverse
kick can only come from the sample's own anisotropy, the `1/√N ≈ 5 %` off-diagonal stress of
one 432-molecule configuration, persisting across cycles because the configuration
rearranges on tens of picoseconds. That is a candidate, not a finding. **If R1′ fires on the
three-seed transverse average, branch (e) is entered: the reader is convicted of reading
momentum rather than flux, and R1's kills on the longitudinal arms are voided with it.**
The two remaining transverse arms decide this before anything else is graded.

## What two arms already say

1. **The 200 m/s control is outside linear response.** At 50 m/s the driven density and
   current relax without oscillating; at 200 m/s both oscillate at `~720` fs. The
   nonlinearity control did its job: R2's branch on the linear arm is the overdamped one
   Amendment 1 A3 expected, and the 200 m/s oscillation is an amplitude effect. R1 is
   therefore graded on the 50 m/s arms at their floors (Amendment 3 A3's clause), with
   the 200 m/s arms as the departure.
2. **The model's sound speed is about twice water's.** `c_s ≈ 2,800` m/s from the product of
   the two overdamped rates at 50 m/s, `3,200–3,300` from the control's oscillation — both
   above R2's `[1,000, 2,200]` band, which was set on water; VIEW-SEARCH-1's equilibrium
   "1,470 m/s" came from a first zero crossing at 100 fs lag resolution and is superseded.
   A stiff minimal-basis liquid with water's activation energy and twice its sound speed:
   the model's number, reported as the prereg says.
3. **R1 is not closing within its budget, and the excess is not noise.** Two seeds, two
   amplitudes: `D − floor = 0.14` and `0.11`, with the placebo beaten by `+0.35` and `+0.07`.
   The `(n̄, p̄)` chart on eight cells along the wave leaves a residual that the derived
   floor (discretisation plus both shot noises) does not explain. If the third 50 m/s seed
   agrees, the campaign's R1 verdict is KILL as staked and the residual is the value: the
   quantity the next view must carry (rule "the gap is the value"), with the second
   harmonic of the driven density and the momentum field's sub-cell structure as the two
   named candidates.
4. **One null fired marginally** (the undriven density quadrature on L seed 1 at `4σ` against
   a `3σ` bar). Under Amendment 3 a null fires against noise; at SNR 5.7 on the density this
   is at the edge and is carried as such, not as a conviction, pending the other seeds.

## The closure test on the two arms (2026-09-21 evening; `RESPONSE1_AMENDMENT_5.md`)

Predicting each cell's aligned occupancy change from the momentum of the molecules within
`0.25 Å` of its faces, instead of the slab means, returns the amplitude fidelity to `1` and
takes `D` from `0.735` to **`0.164`** on the 200 m/s control at 8 cells (`R² = 0.97`), and from
`0.846` to `0.492` on the 50 m/s seed; lagging the slab momentum never helps. The residual
R1 measured is the momentum density's molecular layering at one-diameter slabs, and the
closed chart is the staggered one. R1 as staked still KILLS; R1″ is staked on the remaining
arms with `h` fixed.

## `ν_l` against a band stated AFTER the reading (2026-09-23) — a comparison, not a stake

The pooled longitudinal rate on the three 50 m/s seeds is `1.03 × 10¹³` /s (spread 39 %), which
the reader writes as `ν_l = Γ/k² = 1.44 × 10⁻⁶` m²/s (the per-seed values `1.15, 1.45, 1.72`;
the `1.2` quoted earlier was seed 0). No band was frozen for it. The hydrodynamic
identification is `ν_l = (4/3 η + ζ)/ρ`; with the operator's own `η = 4.4 × 10⁻⁴` Pa s and
`ρ ≈ 10³` kg/m³, the band over a bulk-to-shear ratio `ζ/η ∈ [0, 2.7]` (the upper end water's
measured ratio, `ζ ≈ 2.4` mPa s against `0.89` at 25 °C, Holmes–Parker–Povey 2011) is
`[0.6, 1.8] × 10⁻⁶` m²/s. The reading sits inside it and implies `ζ/η ≈ 1.9` for the rigid
operator — smaller than water's, as a molecule with no internal relaxation channel should
be. Two caveats that keep this a comparison: at `k = 0.27 Å⁻¹` the mode is propagating
(`ω = c_s k` with `c_s = 3.3` km/s on the 200 m/s seed), so `Γ` is a sound damping and the
factor between `Γ` and `ν_l k²` is a convention the reader fixed at one; and the
hydrodynamic `ν_l` is `k`-dependent at this `k`. A frozen band belongs in the next campaign.
## The three-seed aligned average (2026-09-23, the reader change owed)

`rung2 --response` now pools, when it is given more than one trajectory: every trajectory's
sign-aligned cycles (the signs `(−1)^c` per trajectory as the per-seed read assigns them, not
re-derived) are averaged into ONE aligned cycle, each cycle weighted equally (36 cycles, 12 per
seed), and the same continuity read is run on it: integral form, the two lead windows against
the relaxed last, 8×1×1 graded with 16 and 4 beside, both floors of Amendments 2 and 4. R4/R4′
are read on the pooled aligned modes. The per-seed lines are unchanged (the old reads diff
clean; only `POOLED …` lines are added). Check: two copies of one trajectory pooled reproduce
that trajectory's per-seed `D`, `s`, floors and R4/R4′ to the printed digit. **R1′'s SE on the
pool is the leave-one-cycle-out jackknife over all 36 cycles** (each cycle of each seed dropped
in turn, the other 35 kept, `D` re-read; `SE = sd(D₋ᵢ) · √(n − 1)`, the per-seed estimator of
Amendment 3 A1 carried to the pool). Outputs: `replace0/response1_pooled_{L,T,L200}.txt`.

| stake | pooled read (36 cycles) | against the stake as written | per-seed verdict in the table |
|---|---|---|---|
| **R1**, arm L 50 m/s, 8×1×1 | `D = 0.622`, blind `0.923`, separation `+0.301`; `s = 2.00`, two-sided floor `0.492` (one-sided `0.519`); relaxed last window `0.872` | `D > 0.2` and over its floor by `0.130` (> 0.05): **KILL as staked on `D ≤ 0.2`, not at floor; separated → branch (b), open under drive**; null holds | **unchanged**: branch (b). The pool sharpens it: per seed one of three sat at its floor; on the graded average the excess over the floor is real (`18 %` of the signal power) |
| R1 on the 200 m/s control (Amendment 3 A3's grading arm) | `D = 0.310`, blind `1.086`, separation `+0.776`; `s = 2.89`, floor `0.342` (one-sided `0.303`); relaxed `0.921` | **AT FLOOR** (over `0.2`, within `0.05` of both floors); separated; null holds | seed 0's per-seed KILL (`0.735` over `0.60`) does **not** survive pooling: on 36 cycles the cell chart reads at its floor at 200 m/s |
| **R1′**, arm T 50 m/s, 8×1×1 | driven `0.929`, relaxed `0.989`, `|Δ| = 0.060`; `2 SE = 0.150` (jackknife over 36) | `|Δ| < max(0.1, 2 SE)`: **HOLDS** | **changes from "holds, two of three" to HOLDS on the graded statistic**; seed 0's firing is inside the pooled noise; branch (e) is not entered |
| **R4 / R4′**, current mode | L: blind/spatial `0.016`, R4′ `2.8e-4` vs `3σ = 1.6e-3`; T: `0.020`, `4.2e-4` vs `6.0e-4`; L200: `0.013`, `1.7e-4` vs `8.3e-4` | hold on all three pooled arms | unchanged (hold) |
| R4 / R4′, density mode (L arms) | L: blind/spatial at the peak `0.207` under its `3σ` bar, R4′ `0.30` vs `1.05`; L200: `0.003`, `0.27` vs `0.44` | hold | **L seed 1's marginal density R4′ does not survive pooling** |

The T200 control is one seed and is not pooled; its R4′ firing stands as read. Amendment 2's
advance arithmetic for the three-seed 50 m/s floor (`s ≈ 5–7`, floor `≈ 0.14–0.22`) is not
what the pool reads: `s = 2.0`, floor `0.49` — the signal-to-noise per window-cell at 8 cells
was over-priced by a factor of about three. **Still owed:** the face chart R1″ (`0.245` in the
verdict table) is the MEAN of three per-seed `D` from `r1_closure_test.py`, not the aligned
average; that script was not changed here.

## R1″ on the three-seed aligned average (2026-09-23, the reader change owed)

`r1_closure_test.py` now takes several arm directories: each arm's table prints as before (the
single-directory output re-run on `response1_L200_seed0` diffs clean against its committed
`r1_closure_test.txt`), then a POOLED table: every arm's sign-aligned per-cycle fields (the
signs `(−1)^c` per arm as the per-arm read assigns them, not re-derived) averaged with equal
weight per cycle (36 cycles, 12 per seed), the same closures read on the pool, and for the face
chart at `h = 0.25 Å`, 8 cells, the leave-one-cycle-out jackknife over all 36 cycles
(`SE = sd(D₋ᵢ) · √(n − 1)`, the estimator rung2's pooled R1′ uses). Check: seed 0 pooled with
itself reproduces its per-seed table to the printed digit (`D = 0.164`, `α = 1.05`). Run on core
29. Outputs: `replace0/r1pp_pooled_{L200,L,T}.txt`.

| R1″ read | pooled (36 cycles), face chart `h = 0.25 Å`, 8 cells | against Amendment 5 as written |
|---|---|---|
| **200 m/s longitudinal (graded)** | `D_lead = 0.180`, jackknife SE `0.058`; `D_tail = 0.689`; `α = 1.03`; `R² = 0.97`. Sensitivity `h = 0.5 Å`: `D = 0.215`, `α = 1.02` | `D ≤ 0.2` and `α ∈ [0.8, 1.25]`: **MET** — by `0.020`, a third of one SE |
| 50 m/s longitudinal (reported) | `D_lead = 0.315`, SE `0.087`; `D_tail = 0.595`; `α = 1.11`; `R² = 0.91` (cell chart on the same pool: `0.622`) | reported, not graded; this script computes no floor, so "at its floor" is carried by rung2's pooled R1 floor for the cell chart (`0.492`), which is not the face chart's |
| transverse null (graded) | driven `D_lead = 0.438`, relaxed `D_tail = 0.575`, `|Δ| = 0.137`, jackknife SE of `|Δ|` `0.278` | `|Δ| ≤ 0.1`: **FIRES** by `0.037` as written — the stake has no SE term; `2 SE = 0.56` would hold it under R1′'s `max(0.1, 2 SE)` form, which Amendment 5 did not adopt |

The pooled 200 m/s `D = 0.180` is not the mean of the per-seed reads (`0.245`): averaging the
aligned fields before forming `D` removes the per-seed incoherent residual, which a mean of
per-seed `D` keeps. The same arithmetic makes the transverse `|Δ|` `0.137`, not the `0.035`
(a difference of per-seed means) in the verdict table. The cell chart's own 200 m/s pooled
`D = 0.310` on this script agrees with rung2's pooled `0.310`.

**The verdict table's R1″ row changes:** BETWEEN → **MET** on the staked statistic
(`0.180 ± 0.058`, `α = 1.03`); and R1″'s null row changes from "holds on the average (`0.035`)"
to **FIRES as written** on the pooled aligned average (`0.137` vs `0.1`, inside its own SE
`0.28`). The lead edits the table.
