# RUNG 2 — AMENDMENT 2: Amendment 1's rule applied to the momentum and energy fields

> **CORRECTION, 2026-09-17, on the first real reading under this amendment — the calibration
> below is WRONG, and the error is the lead's.** The rule `Δ_cell = √⟨n⟩ · Δp` was checked
> against a smoke bundle whose half-box momentum spread was `11.7` au and found to match
> within `10 %`. The smoke was a hot, unsettled box of `2.4` ps whose slowest momentum mode
> had not relaxed; on the three settled seeds the same spread is **`36–44` au, `3.4–4.2×`
> the bin**, and the momentum rung read `6–11` informative transitions — VOID, exactly as
> under Amendment 1. Worse: the section "Checked, not fitted" records the independent-oxygen
> formula as *"`3–4×` too wide … a bin that wide would have passed the momentum field
> vacuously."* **That formula was right.** With the finite-population correction for
> conserved momentum it matches the settled data to `1.1–1.3×`, and the residual is the
> carrier's known rigid-mode heating (`REPLACE-0`: oxygen kinetic temperature `≈ 375` K
> against `T_target = 300`). The lead calibrated a rule on an unrepresentative box and
> wrote the correct physics off as the error — while writing an amendment about scales
> derived in the wrong regime. `RUNG2_AMENDMENT_3.md` carries the repair; this document is
> kept as written beneath this note so the error stays legible.

*Written 2026-09-16, committed alone, BEFORE any velocity-carrying trajectory is read under
it. Amends `RUNG2_PREREG.md` §2.3 in one place: the momentum and energy bins are scaled to the
cell by the same `√⟨n⟩` that `RUNG2_AMENDMENT_1.md` A1 applied to the density field.
Amendment 1 said those two bins were "unchanged"; the first trajectory that carried
velocities showed why they cannot be, and this is the correction, made before the reading
it is for.*

## Found by the first velocity-carrying smoke

No run before 2026-09-16 banked velocities, so no reading of the momentum or energy rung on
the 3D carrier had ever been made — they were degenerate copies of occupancy and the record
said so. The first bundle that carried them (`replace0`, 128 waters, 241 readouts) read at
`2×1×1` under Amendment 1:

| rung | collisions | informative |
|---|---|---|
| Occ (Poisson) | 28,680 | 240 |
| **Mom** | **12** | 24 |
| **Ene** | **5** | 10 |

The density field collides freely and the fields above it almost never do. The reason is the
freeze's `Δp`: *"the thermal momentum of a hydrogen atom, `√(m_H k_B T) = 1.3211` au."* That
is the resolution of ONE atom's momentum. A cell's momentum is a sum over `⟨n⟩` atoms and
fluctuates over a range `√⟨n⟩` times wider — at `⟨n⟩ = 64` the measured spread is `11.7` au,
nine bins wide, so two frames almost never share a momentum reading. `Δe` has the same
fault (`8.3e-3` Ha measured against a bin of `9.5e-4`). This is exactly the fault A1 found in
the density field, one rung up, and it is the same shape as `LIQUID2_AMENDMENT_1.md`: a
scale derived for one regime — a single atom, in cells of one to six atoms — applied to
another.

## The rule, stated once

> **`Δ_cell = √⟨n⟩ · Δ_atom`** for every field of the chart, where `⟨n⟩ = N_atoms / cells`
> is exact arithmetic and `Δ_atom` is the freeze's own per-atom resolution:
> `1` for occupancy, `Δp = √(m_H k_B T)` for momentum, `Δe = k_B T` for energy.
> **For the two continuous fields the factor is `round(√⟨n⟩)`**, an integer — found
> necessary by plant PB-2 and stated here before the reading: a coarse `floor` bin is a
> union of fine `floor` bins only when the bin ratio is an integer, so the refinement
> self-check holds exactly only then. The density field keeps A1's unrounded rule, because
> its underlying value is an integer and `floor(n / Δ)` is a function of `n` for any `Δ`.

A1 was this rule at `Δ_atom = 1`. Stated for all three fields it reads: *a cell's field is
known to within `√⟨n⟩` of the resolution the freeze gave one atom's* — the statistical
scaling of a sum of `⟨n⟩` independent thermal contributions, which is the argument G2 itself
rests on. Nothing new is introduced: no second mass, no second temperature, no measured
input. The freeze's `Δp` and `Δe` are kept as the per-atom scales they are.

## Checked, not fitted

The rule's size was checked against the smoke bundle's own velocities AFTER it was chosen
and before this was written — a check that it is the right order, never an input to it:

| grid | `⟨n⟩` | `√⟨n⟩ · Δp` | measured `σ(P_x)` | `√⟨n⟩ · Δe` | measured `σ(E)` |
|---|---|---|---|---|---|
| 2×1×1 | 64 | `10.6` au | `11.7` au | `7.6e-3` Ha | `8.3e-3` Ha |
| 4×1×1 | 32 | `7.5` au | `9.0` au | `5.4e-3` Ha | `5.9e-3` Ha |

Within `10`–`20 %` on both fields at both grids. Recorded for honesty: the *other* derivable
candidate, the independent-oxygen formula `√(⟨n⟩ m_O k_B T)`, is `3–4×` too wide — the
oxygens' momenta are anticorrelated by conservation and carried collectively, and a bin that
wide would have read the momentum field as nearly constant and passed it vacuously. The rule
above was chosen for being A1's rule; that it also matches the measured spread is what makes
it usable, and if a future carrier's spread departs from it by more than a factor of two the
departure is a finding about that carrier, reported beside the reading, not a reason to move
the bin.

## What the plants taught before this was read

Two plants failed on their first run and both failures were the plant's, corrected before a
trajectory was read:

- **PB-1's first carrier drew independent OXYGEN velocities**, whose cell momentum
  fluctuates `4×` wider than `√⟨n⟩ · Δp`, and reached two informative transitions. That
  is the coincidence named above from the other side: the rule matches real water's oxygen
  cells because the liquid anticorrelates them, and an independent-oxygen gas is not water.
  The plant's carrier is now hydrogen, for which `√⟨n⟩ · Δp` is the plain statistics of a
  sum and nothing else is assumed; whether water's oxygens fluctuate at that scale is the
  field reading's job, and it was checked there.
- **PB-2 fired at the momentum rung** at unrounded `√⟨n⟩`, which is the theorem about
  `floor` stated in the rule box. The rounding was added for it.
- PB-1 also asserted G4 at the ENERGY rung, which this amendment never staked; the
  assertion is withdrawn to what was staked (collisions increase).

## What changes and what does not

`Density::CellScale` in `holon-lens` scales all three fields; `Density::Poisson` (A1, density
only) and `Density::Exact` (the freeze) are kept and every reading under this amendment
prints all three side by side. The collision form, both legs, the vacuity fence, the work
count, the controls, the refinement checks, `β`, G2 and every branch: unchanged. **Nothing
lowers a bar.**

## Plants

| plant | carrier | must |
|---|---|---|
| **PB-1** | 200 atoms with thermal velocities at `T_target` in `2×1×1`, walking | under A1 (`Poisson`): the `Mom` rung VOID by counting; under `CellScale`: `Mom` collides and meets G4 on the same frames |
| **PB-2** | every trajectory | `refines(Poisson, CellScale)` at every rung — the finer chart refines the coarser; a violation convicts the scaling |
| **PB-3** | P-3's hidden variable, given thermal velocities | `NotClosed` at every rung under `CellScale` — the wider bin hides nothing |
| **PB-4** | any trajectory, `Occ` rung | `CellScale` readings equal `Poisson` readings exactly — at the density rung the two amendments are the same rule |

## The first reading it will be pointed at

The three-seed momentum run launched 2026-09-16 (`replace0_momentum.sh`: the transport
arms' own branch points, 300 readouts at 10 fs over 3 ps, both arms, velocities banked).
Expected and stated now: the `Occ` rung's binned `D_A` reproduces Amendment 1's within seed
spread; the `Mom` rung under `CellScale` reaches G4 at `2×1×1` and `2×2×1`; whether density
+ momentum closes any dynamic grid within `β` is the reading and is not predicted. The
`Mom` and `Ene` rungs under A1's `Poisson` are printed beside it and are expected VOID by
counting — the control that shows what this amendment bought.
