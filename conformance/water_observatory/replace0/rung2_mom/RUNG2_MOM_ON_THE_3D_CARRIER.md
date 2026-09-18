# The fluid-element chart on the 3D carrier, density AND momentum — the first graded verdicts, and why the freeze says they are not readings

*Read 2026-09-17 on the three-seed momentum run (`replace0_momentum.sh`, launched 09-16:
the transport arms' branch points, 300 readouts at 10 fs over 3 ps, both arms, velocities
banked). Instrument: `holon-lens/examples/rung2.rs --amend1` under Amendments 1 + 3
(`rung2_amend3.log`); the read under Amendment 2 that failed and led to the correction is
`rung2_amend2.log`. The self-check never fired; zero refusals; G9a holds on every file.*

## The verdict the freeze's branch order gives: (e) — VOID, NO SEPARATION

G7 requires the position-blind chart to score WORSE than the spatial one by `0.05`
absolute. At `2×1×1`, the density rung, `Derived`:

| arm, seed | spatial `D_A` | blind `D_A` | spatial − blind | G7 (`≤ −0.05`) |
|---|---|---|---|---|
| flexible 0 | 0.092 | 0.143 | −0.051 | met, at the bar |
| flexible 1 | 0.168 | 0.091 | +0.077 | fails |
| flexible 2 | 0.243 | 0.164 | +0.079 | fails |
| rigid 0 | 0.207 | 0.044 | +0.163 | fails |
| rigid 1 | 0.273 | 0.072 | +0.201 | fails |
| rigid 2 | 0.115 | 0.064 | +0.051 | fails |

On five of six the scrambled chart is **more** closed than the spatial one. At the momentum
rung spatial and blind are within `0.005` of each other on every arm (`0.955 / 0.960`,
`0.931 / 0.932`). By §5: *"a reading that cannot tell a real chart from a scrambled one is
not a reading"* — branch (e) outranks (c) and (d), and **nothing is concluded about the
tier's closure, in either direction.** The numbers below are published as data, as the
freeze requires, not as verdicts.

## The data

| grid | rung | arm | `D_A` (mean ± spread, 3 seeds) | collisions | informative | grade |
|---|---|---|---|---|---|---|
| 2×1×1 | Occ | flexible | 0.168 ± 0.151 | 18k–23k | 300, 300, 300 | NotClosed |
| 2×1×1 | Occ | rigid | 0.198 ± 0.158 | 16k–23k | 300, 300, 300 | NotClosed |
| 2×1×1 | Mom | flexible | 0.955 ± 0.012 | 182–266 | 170–188 | VoidWorkCount |
| 2×1×1 | Mom | rigid | 0.931 ± 0.057 | 257–300 | 188–209 | NotClosed (1) / VoidWorkCount (2) |
| 2×1×1 | Ene | both | ≈ 1.0 | 11–25 | 19–47 | VoidWorkCount |
| 2×2×1 | Occ | flexible | 0.358 ± 0.199 | 5.6k–11.6k | 296–300 | NotClosed |
| 2×2×1 | Occ | rigid | 0.421 ± 0.153 | 4.8k–8.9k | 297–300 | NotClosed |
| 2×2×1 | Mom | both | ≈ 0.95 | 3–11 | 5–22 | VoidWorkCount |
| 2×2×2 | Occ | both | 0.60–0.64 ± 0.13 | 600–1,000 | 256–271 | NotClosed |

Arm differences are inside the seed spread at every grid and rung; no rigid-vs-flexible
difference is claimed. Leg B at `2×1×1` Occ: `D_B = 0.07–0.21` at full coverage.

**What Amendment 3 bought, measured:** the momentum rung went from `0` collisions (A1) and
`5–10` informative (A2) to `182–300` collisions and `170–209` informative — G4 reached on one
seed and missed by `5–15 %` on the rest, which the amendment said in advance it might.

## Why the control wins, and what it says about Amendment 1

The spatial density chart at `2×1×1` has **three distinct readings** over 300 frames. Its bin
is `Δn = √⟨n⟩ = 8`; the half-box occupancy's measured spread is **`σ(n) = 2.5`**
(`fluct = 0.039 × 64`). The bin is `3.2×` the field's own fluctuation, so the chart is
under-resolved — nearly constant, its few collisions mostly `(8,8) → (8,8)`.

The reason is the liquid. `√⟨n⟩` is the Poisson fluctuation of INDEPENDENT particles; a
liquid's cell count fluctuates at `√(S(0) ⟨n⟩)` with the structure factor at zero wavevector
`S(0) = ρ k_B T κ_T ≈ 0.06` for water — a factor `√0.06 ≈ 0.25` on the spread. G2's
admissibility bar, built on `1/√N`, is therefore *conservative* for a liquid (a good
direction), but the **density bin derived from the same arithmetic is `3–4×` too wide** for
one. The position-blind control mixes atoms across cells by a per-atom permutation, which
destroys the liquid's spatial anticorrelation and gives its occupancy back its Poisson
spread — so the blind chart happens to be resolved at its own scale and the spatial chart
is not. **G7 is comparing charts of unequal effective resolution.** That is a fault in
Amendment 1's density rule on a liquid carrier, not evidence about spatial structure in
either direction — which is exactly what branch (e) says.

This is the third scale in this instrument derived for one regime and applied to another
(the density bin for a gas, the momentum bin for one hydrogen, the momentum bin for an
unrelaxed box), and the first found by a control rather than by a VOID.

## The momentum field, separately

`D_A ≈ 0.94` at `2×1×1` on both arms and both charts: two frames sharing a density and
half-box-momentum reading go to different readings `94 %` of the time, at a 10 fs readout.
The half-box momentum at 128 waters is dominated by the incoherent thermal sum, which
decorrelates on the collision time (tens of fs), not by the coherent hydrodynamic mode
(period `≈ 1` ps at this box). It is not a slow variable at this size and cadence. That is a
statement about the carrier, and it is why G7 finds no separation there either.

## What would repair the instrument, not done here

A density bin at the liquid's own fluctuation scale needs `S(0)`, which is a property of the
model, not a protocol constant. Two honest routes, both a call for the operator: (i) declare
water's experimental `S(0)` as an external constant the way `T_target` is; (ii) measure
`σ(n)` on a separate calibration trajectory, as LIQUID-2's band was set from LIQUID-1's
scatter. Binning at the spread of the trajectory being graded is fitting and is refused.
Until one is chosen, the 3D carrier's fluid-element chart is VOID by G7, with the numbers
above on the record.


---

## Read again under `RUNG2_AMENDMENT_4.md` (2026-09-17, same day, same six files): two liquid bins, side by side

`rung2 --amend4`; log `rung2_amend4.log`. Route (i) `External`: water's `S(0) = 0.0621` from
CRC constants. Route (ii) `Calibrated`: `σ(n)` from the flexible arms of the seeds NOT being
graded, three-fold hold-out. Momentum and energy as Amendment 3 under both. The self-check
never fired; zero refusals.

**The two bins, and the ratio between them — a measurement:**

| grid | `⟨n⟩` | `Δn` (i) External | `Δn` (ii) Calibrated, per seed | `(ii)/(i)` | own `σ(n)`, not used |
|---|---|---|---|---|---|
| 2×1×1 | 64 | 1.41 | 2.06, 2.21, 2.35 | **1.57** | 2.49, 2.20, 1.90 |
| 2×2×1 | 32 | 1.22 | 1.92, 2.03, 2.11 | **1.65** | 2.21, 2.01, 1.83 |
| 2×2×2 | 16 | 1.00 | 1.72, 1.74, 1.77 | **1.74** | 1.80, 1.75, 1.68 |

The liquid's cells fluctuate `1.6–1.7×` more than the `k → 0` structure factor says, and the
excess GROWS as the cell shrinks — the finite-cell correction to `S(0)` at this box, which
the amendment said route (i) could not carry and route (ii) would measure. It did.

**The two routes agree on G7, at every grid that grades:**

| grid | route (i): files separated | route (ii): files separated | grade of those cells |
|---|---|---|---|
| 2×1×1 | **0 of 6** | **0 of 6** | NotClosed, 299–300 informative |
| 2×2×1 | **0 of 6** | **0 of 6** | NotClosed, 234–284 informative |
| 2×2×2 | 0 of 6 | 3 of 6 | VoidWorkCount (95–197) — not graded |

**Branch (e) stands under both routes.** With the density field resolved at the liquid's
own fluctuation — `9–11` distinct readings at `2×1×1` where Amendment 1 gave `3` — the
position-blind chart is still as closed as the spatial one on every graded cell. The bin was
the *mechanism* by which G7 failed on the 16th; it is now ruled out as the *cause*. The
instrument does not discriminate a two- or four-cell spatial partition of 128 waters from a
scrambled one, and by §5 nothing is concluded about the tier from these grids. The first
hint of separation appears at eight cells under route (ii), on three files of six, where
the work count is not met.

**The density chart's defect, both routes, properly resolved:**

| grid | route | flexible `D_A` | rigid `D_A` |
|---|---|---|---|
| 2×1×1 | (i) | 0.512 ± 0.052 | 0.527 ± 0.006 |
| 2×1×1 | (ii) | 0.465 ± 0.155 | 0.468 ± 0.143 |
| 2×2×1 | (i) | 0.778 ± 0.064 | 0.789 ± 0.042 |
| 2×2×1 | (ii) | 0.672 ± 0.085 | 0.688 ± 0.054 |

Robust to the bin (the two routes agree within seed spread), robust to the arm, and
`23–40×` above `β = 0.02`. Published as data under branch (e).

**What the comparison bought.** One question the operator asked — *why not both and
compare?* — turned an open decision into two measurements: the finite-cell correction to
`S(0)` (`1.6–1.7×`, growing as cells shrink), and the fact that branch (e) does not depend on
the bin. What it leaves is a size statement sharper than the 16th's: on this carrier the
grids with enough transitions to grade have too few cells for spatial structure to
distinguish itself from a random partition, and the grid where it begins to (`2×2×2`) has
too few transitions. Both move the same way with a larger box.
