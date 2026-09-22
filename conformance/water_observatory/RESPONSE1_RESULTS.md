# RESPONSE-1 — RESULTS, appended arm by arm as they land (opened 2026-09-21)

*Prereg `RESPONSE1_PREREG.md`; Amendments 1–4; reader `rung2 --response`; each arm's read in
`replace0/response1_<tag>_seed<k>/response1.read.txt`. Nothing here is graded until the
three-seed averages exist; per-arm rows are readings of one seed. The five partial arms
(4–7 cycles) are in `RESPONSE1_AMENDMENT_3.md` and are superseded by their full re-runs.*

## Arms read so far

| arm | cycles | SNR (current) | nulls (R4, R4′, density R4, density R4′) | R1 at 8×1×1: D, separation, two-sided floor | R2 / longitudinal current | R3 |
|---|---|---|---|---|---|---|
| L200 seed 0 (control) | 12 | 38 | all hold | **0.735, +0.35, floor 0.60 → KILL on this seed** (over the floor by 0.14) | OSCILLATORY: period 705–735 fs, `c_s ≈ 3,200–3,300` m/s, `Γ ≈ 2.2–2.7 × 10¹²` /s — outside R2's band | — |
| L seed 1 (50 m/s) | 12 | 9.4 | R4, R4′, density R4 hold; **density R4′ fires marginally** (`0.52` vs `3σ = 0.39`) | **0.846, +0.07, floor 0.74 → KILL on this seed** (over by 0.11) | OVERDAMPED: current `λ₂ = 1.04 × 10¹³` /s (`ν_l = 1.45 × 10⁻⁶` m²/s), density from the peak `λ₁ = 5.5 × 10¹²` /s; `c_s = √(λ₁λ₂)/k ≈ 2,800` m/s, `Γ/ω ≈ 1.05` | — |

| T seed 0 (50 m/s, transverse) | 12 | 8.7 | R4 (`0.17` of spatial, under its `3σ` bar), R4′ hold | **R1′: driven `0.759`, relaxed `1.036`, `|Δ| = 0.28` against `max(0.1, 2 SE) = 0.25` → FIRES by `0.03`**; separation from the blind `+0.24` | current OVERDAMPED, `λ = 3.6 × 10¹²` /s (`τ = 279` fs); 7 of 12 tails unrelaxed | **`η = 5.0 × 10⁻⁴` Pa s** `[4.2, 5.8]`, IN BAND — equal to the equilibrium read at this `k` |

| T seed 1 (50 m/s) | 12 | 13.0 | R4 (`0.11`), R4′ hold | **R1′: driven `0.966`, relaxed `0.996`, `|Δ| = 0.03` → HOLDS** | current overdamped | `η = 3.9 × 10⁻⁴` `[1.3, 6.5]`, IN BAND |
| T seed 2 (50 m/s) | 12 | 5.1 | R4 (`0.25`, under its bar), R4′ hold | **R1′: driven `0.991`, relaxed `0.952`, `|Δ| = 0.04` → HOLDS** | current overdamped | `η = 4.4 × 10⁻⁴` `[3.8, 5.0]`, IN BAND |
| L seed 2 (50 m/s) | 12 | 6.8 | all hold | **R1: `0.836` against a two-sided floor of `0.860` → AT FLOOR**, separation `+0.20`; null holds | OVERDAMPED from the peak (SNR 6.6) | — |

## The transverse null, as it stands (2026-09-22 morning, three FULL transverse arms)

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
