# RESPONSE-1 — the fluid element as a response: PREREGISTRATION

*Written 2026-09-19, committed alone, BEFORE the kick exists in the runner and before any
trajectory is read. The reasoning is `FLUID_ELEMENT_RESPONSE.md`; this is the freeze. Every
band below is staked before the numbers, every plant must fire, and the branches are named
in advance. Instrument: `replace0 scout --kick` (to be built) and `rung2 --response` (to be
built), on the rigid operator's 432-water box, read under `RUNG2_AMENDMENT_5/6.md`.*

## 0. What is being tested, and what the arithmetic already says

An equilibrium cell at this size has no fluid-element signal: its coherent flux per face
per window (`0.56` crossings) is five times below the shot noise (`2.7`). A fluid element is
a RESPONSE — to a compression, to a shear — and that is what the continuum equations
describe. So the box is driven, and the chart is asked to close on the response.

Stated before the run, from water's own transport numbers at this box (`L = 2.35` nm,
`k = 2π/L`): **the longitudinal sound mode is overdamped** — `Γ_l = ν_l k²/2 ≈ 1.4 × 10¹³ s⁻¹`
against `ω₀ = c_s k ≈ 4 × 10¹² s⁻¹`, `Γ/ω ≈ 3.4`. A density kick RELAXES, at the slow rate
`ω₀²/2Γ = c_s²/ν_l ≈ 6 × 10¹¹ s⁻¹` (`~1.7` ps), which is `k`-independent and does not separate
the sound speed from the longitudinal viscosity. The **transverse (shear) mode** has no
sound in it and decays at `Γ_s = η k²/ρ` — the shear viscosity directly, `~140` fs at
water's `η`, `~465` fs if the model's is a third of it. An underdamped sound wave would need
`L ≳ 8` nm, `~35,000` waters, and is not this campaign. Two arms follow from this, not one.

## 1. The carrier

The 432-water box (`n = 6`, `L = 44.39` bohr) on the rigid operator at the banked stiffness
envelope, fine-settled `3,000` frames for the reference geometry, rigid-settled BY CRITERION
(the scout's, `0f3944d`), three seeds (`0x…4530/31/32`). The operator's density and momentum
charts matched the fine model within seed spread on 128 waters and its price is measured
(`2,171` core-s/ps on eight workers, bit-identical). **One fine-model seed at the same kick
is owed** before any closure read here is called the model's, per Amendment 5.

## 2. The drive — declared, and declared small

At the start of each cycle every body receives a velocity increment, by arm:

> **Arm L (longitudinal):** `Δv_x = v_d sin(2π x_com / L)`.
> **Arm T (transverse):**   `Δv_y = v_d sin(2π x_com / L)`.
> **`v_d = 50 m s⁻¹`** — `0.6 %` of the thermal kinetic energy per molecule, linear response.
> **Nonlinearity control:** one seed of each arm at `v_d = 200 m s⁻¹` (`10 %`); the read
> quantities must agree with the `50` arm within their spread, or the `50` arm is reported
> as the reading and the `200` as the departure.

The kick is a pure velocity increment on the rigid bodies' momenta (no position change),
**with the mean of `sin(k x)` over the bodies subtracted so the total momentum change is
exactly zero** — "zero by the symmetry of `sin`" is true in the continuum and the first
smoke read `Δp = 4.44` au on 128 discrete positions; the subtraction is the standard step
and is declared here before any arm runs. The kinetic energy rises by `½ M v_d² ⟨sin²⟩`,
which is `5 %` of the thermal kinetic energy at `200 m/s` and `0.3 %` at `50` (§2's "10 %"
and "0.6 %" were the `sin = 1` figures; the smoke read `0.076 kT` per water at `200`). **Cycles:** kick, `3.14` ps of NVE (four
windows of `τ = 785` fs at `2×2×1`), rescale the rigid momenta to `T_target` (a DECLARED
step, the same rescaling the settle uses — the dissipated kick energy is removed so twelve
cycles do not heat the box by `7 %`), kick again with the sign flipped. **Twelve cycles per
seed** = `38` ps, `48` windows with signal, and each cycle is an independent realisation of
the same response. Readouts every `10` fs so the shear decay has `≥ 5` points per e-fold
even if the model's `η` is three times water's.

## 3. The readouts and their stakes

Read from the banked walk and velocities at every readout, per cycle, then pooled over
cycles and seeds with the spread:

- **R1 — closure under drive (arm L).** `D_cont` on the driven chart at `2×2×1` under
  Amendment 6, restricted to the first two windows of each cycle (where the arithmetic says
  the signal is), against the position-blind chart, G7's form. **Stake:** `D_cont(spatial)
  ≤ 0.2` AND `D_cont(blind) − D_cont(spatial) ≥ 0.05`. **Kill:** either fails. **Null, in the
  same run:** the last window of each cycle (relaxed) must read `D_cont ≥ 0.8` — the signal
  is the drive's and not the instrument's.
- **R1′ — the transverse null (arm T).** `∇·v = 0` under a shear kick, so continuity must
  see NOTHING: `D_cont(spatial)` on arm T's driven windows within `0.1` of arm T's relaxed
  windows. **Kill:** a transverse kick that moves the continuity reading — the leg would be
  reading momentum, not flux.
- **R2 — the density mode (arm L).** `ρ_k(t) = Σ_j cos(k x_j)` over oxygens at `k = 2π/L`,
  per readout. **Branch by the measured damping:** if it decays monotonically (overdamped,
  the expectation), its relaxation rate `λ` is reported as `c_s²/ν_l` with no band — the
  first transport number this model has; if it oscillates, its period gives `c_s`, staked
  inside `[1,000, 2,200] m s⁻¹` (a minimal-basis liquid at imposed density against
  water's `1,497`), and its envelope gives `Γ_l`.
- **R3 — the rent (arm T).** `j_k(t) = Σ_j v_{y,j} cos(k x_j)`, the transverse current mode,
  per readout; its decay `Γ_s` by least squares over the first two e-folds; **`η = ρ Γ_s / k²`.**
  **Stake:** `η ∈ [0.2, 3.0] × 10⁻³ Pa s` — the range common water models span. **Kill:**
  outside it, reported as the model's viscosity with the band it missed. Twelve cycles ×
  three seeds give `η` with a spread; the spread is the uncertainty.
- **R4 — the placebo's placebo.** The same modes read on the position-blind partition
  (`BlindLabel`) of the driven trajectory must show **no coherent mode**: the blind
  `ρ_k` and `j_k` amplitudes at the kick must be under `0.1` of the spatial ones. **Kill:** a
  scrambled chart with a sound wave in it, which would convict the reader.

## 4. Plants, each of which must fire before a trajectory is read

| plant | carrier | must |
|---|---|---|
| **PR-1** | a synthetic damped standing wave `ρ_k(t) = A e^{−λt}` on random positions | the reader returns `λ` to `2 %` and classifies it overdamped |
| **PR-2** | a synthetic underdamped wave `A e^{−Γt} cos(ωt)` | period to `2 %`, `Γ` to `5 %`, classified underdamped |
| **PR-3** | a transverse current mode `j_k = A e^{−Γ_s t}` on thermal velocities (`σ` = the thermal scale) | `Γ_s` to `5 %` when `A ≥ 3σ_noise`; REFUSED with the reason when `A < σ_noise` |
| **PR-4** | the kick itself, applied in the runner to a stationary box | total momentum change zero to `1e−12` AFTER the mean subtraction (the first smoke read `4.44` au before it); kinetic energy up by `½ M Σ (sin(kx_i) − ⟨sin⟩)² v_d²` to `1e−9`; every body's COM unchanged |
| **PR-5** | the scrambled partition of PR-1's carrier | mode amplitude under `0.1` of the spatial one |

## 5. The branches

- **(a)** R1, R1′, R3, R4 met → **the first fluid-element chart to beat its placebo anywhere
  in the programme**, on the operator; the fine-model seed is then run and if it agrees
  within spread the reading is the model's. `η` and `λ` are banked as the model's first
  transport numbers.
- **(b)** R1 fails on `D_cont ≤ 0.2` but separates from the blind → the chart is OPEN under
  drive but the drive is visible: reported as "responds, does not close," with `D_cont` and
  the separation; the exit is a larger `v_d` (bounded by the `200` control) or a longer `τ`.
- **(c)** R1 fails on separation → branch (e) of rung 2 stands under drive as well; the
  driven experiment did not rescue the tier at this size, and the record says so.
- **(d)** R3 outside its band → the model's viscosity is the reading and the band the
  finding; R1's verdict is unaffected.
- **(e)** R1′ or R4 fires → the reader is convicted; nothing is read.
- **(f)** the `200` control disagrees with the `50` arm beyond spread → nonlinear; the `50`
  arm is the reading, the departure is reported, and `v_d = 20` is added.

## 6. Cost, before the kernel

> **CORRECTION 2026-09-19 14:15 CDT, the arms three hours in.** The figure below, `2,171
> core-s/ps on eight workers`, is the WALL seconds per picosecond of an eight-worker run
> (the pilot's `3,609` single-core divided by the measured `1.66×`), not core-seconds — and
> on the arms the pool bought nothing at all: each eight-worker arm ran at `101 %` of one
> CPU, the rigid operator's serial sectors dominating. So an arm is `~26` hours of wall for
> its 38 ps on a P-core (`41` min/ps measured), `~38` on an E-core, not `2.9`; the eight arms
> in three sequential groups would have landed Wednesday. Re-laid out at 14:08: the three
> running arms kept, their queues stopped, the other five launched single-worker one per
> P-core (`replace0_response1_b.sh`) — all eight in flight, the six 50 m/s arms expected by
> Monday 2026-09-21 afternoon and the controls the same day. The eleventh instance of a
> price taken from the wrong regime: wall time written as core time, and a speedup measured
> once at one size assumed for the campaign.

Twelve cycles of `3.14` ps = `38` ps per seed at `2,171` core-s/ps on eight workers:
**`23` core-hours a seed, `2.9` hours of wall on eight cores.** Two arms × three seeds +
two nonlinearity controls = eight runs, `~185` core-hours, one day of wall on eight cores
run sequentially — or a morning on three eight-core groups. Plus the criterion settle
(`1–11` h serial each, or reuse one settled state per seed across arms: the kick is applied
to a checkpoint). The fine-model seed owed: `~150` core-hours serial. Against the fence's
`3,000` a seed.

## 7. What this does not test

That a cell certified under one drive closes under free boundaries (Leg B across boundary
families — RESPONSE-2); the coarse Ewald halo (REPLACE-1); anything at a box where sound is
underdamped; and the momentum equation's stress term, which the chart still does not carry.
It tests whether the coarse chart of this liquid closes on the one thing a fluid element is
for, and reads the model's shear viscosity from the rent while doing so.
