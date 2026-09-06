# FLUID-0 — AMENDMENT 1: the instrument re-parameterised from its own measured clock, noise and hydrodynamic size, before any census line

*Frozen 2026-09-06, committed alone, BEFORE the census: at the time of this commit
`conformance/mesh/fluid0/census.jsonl` does not exist. Written because the instrument as the
freeze parameterised it FAILED its own gate G0: on FHP-I at `L = 64`, one seed, `A = 0.05`,
the window opened at step `L` and closed at a tenth of the start amplitude, every one of the
sixteen readings refused (`NoDecay` or `TooFewPoints`), so G1, G3 and both plants failed
downstream of it and, by the freeze's §3, the census was not run. `gate_as_frozen.json` and
`gate_as_frozen.log` keep that reading. The delegate's probe (`fluid0_probe.rs`, a
diagnostic, never a reading) measured why, and every number below is one it measured:*

- **the kinetic clock**: the tracer's mean free time on FHP-I at `d = 0.2` is `τ = 2D/v² =
  11.4` steps, and a fitted transport coefficient is flat once the window opens past `1τ`
  (shear `0.5444 → 0.5442`, tracer `4.770 → 4.776` from `1τ` to `16τ`); the freeze opened
  the window at `L = 64` steps, by which time the `k = 1` colour mode at `L = 64` (lifetime
  13.7 steps) was 99 % gone;
- **the noise floor**: the equilibrium standard deviation of the projected amplitude across
  64 seeds is shear `34.0 / 72.4 / 114.5` and colour `21.8 / 45.4 / 89.9` at `L = 64 / 128 /
  256`; the freeze's tenth-of-start floor (`18.9` at `L = 64`) sat UNDER the noise (`28.8`
  RMS), so the last half of every fit was noise and the monotone test fired on it; `R²` fell
  under 0.99 at a close signal-to-noise of 8.2 and held above it from 11.9 up;
- **the hydrodynamic size**: the freeze's own two-wavevector 10 % control reads FHP-I itself
  `PreHydrodynamic` at `L = 64` (shear gap 32.8 %); at `L = 128` the shear gap is 5.3 % but
  the tracer's is 86.4 % (the `k = 2` colour mode's lifetime is under one mean free time);
  at `L = 256` the gaps are 2.0 % (shear) and 8.8 % (tracer);
- **the amplitudes**: the colour is a passive label and `D` is linear in its amplitude to 4 %
  over a factor of twenty (`5.476 / 5.356 / 5.332` at `0.05 / 0.25 / 1.0`); the shear at `A =
  0.10` reaches `R² = 0.9968` on one seed where `A = 0.05` needs four.

misfits: contacts **M-STALE-INSTRUMENT** (the frozen parameters were typed from the shear
intuition — `L` as the transient, a tenth as the floor — and the instrument said so on its
first gate; this amendment replaces every typed number by a measured rule and keeps the
frozen reading in the record); **M-FLOOR-UNSTAKED** (the floor is now a rule on the MEASURED
noise, not a fraction); **M-VOLUME-SCALE** (`L` is set by the hydrodynamic control the freeze
itself carried, at the smallest size that passes it; the control stays a gate);
**M-FIXED-POINT-TRAJECTORY** (two seeds averaged, both declared; the seed rule stated);
**M-CHEAPER-THAN-ITS-PRICE** (the price re-measured on 48 laws across the enumeration and
written before the census); **M-EXIT-DISCRIMINATOR** (the step caps are raised and a cap is
still a refusal, never a number); **M-PLANT-OBS** and **M-PLANT-SECTOR** (plant (ii) becomes
the `0.05`-against-`0.10` linearity check with its carrier as before; plant (i) unchanged;
carriers asserted nonzero in the sector each plant acts on, the colour and the shear
readouts); **M-VACUOUS-SUCCESS** (every reading still asserts its point count, `R²`,
`collisions_fired` and exit before a number is read); **M-BASE-RATE-OMITTED** (the 48-law
preview's refusal rate, 27 of 48 at one seed, is written beside the census's). Not
contacted: the rest of the registry.

## The change (set A)

| parameter | frozen | amended | the measured reason |
|---|---|---|---|
| lattice `L` | 64 | **256** | the smallest size at which FHP-I passes the freeze's own 10 % two-wavevector control on BOTH readouts (2.0 % shear, 8.8 % tracer); `L = 128` fails it on the tracer (86 %) |
| seeds per reading `S` | 1 | **2**: member `s` seeded with `SEED ⊕ (s · 0x9E37_79B9_7F4A_7C15)` (the crate's counter-hash convention; member 0 is the freeze's `0x464c_5549_4430`, member 1 is `0x9E37_3FF5_2A03_3825`), the projections averaged | close signal-to-noise 24.5–53.6 against the floor of 12 (one seed: 16.3–38.0) |
| shear amplitude `A` | 0.05 | **0.10** | `R² = 0.9968` on one seed; linearity to 5 % measured as plant (ii) |
| colour amplitude `A_col` | 0.05 | **1.0** | a passive label; `D` linear to 4 % over a factor of twenty |
| `k_index` | {1, 2} | **{1, 2}** | the whole hydrodynamic set at `L = 256` |
| window start | step `L` | **`4τ`**, `τ = 2D/v²` from the campaign's own FHP-I tracer reading, `= 46` steps on FHP-I; the rule, not the number, is frozen | the fitted values are flat from `1τ` to `16τ` |
| window end | a tenth of the start | **the first step under `max(0.2·|M(t₀)|, 12·σ/√S)`**, `σ` the measured equilibrium sd at that `L` and readout (shear `114.5`, colour `89.9` at `L = 256`), written into `gate.json` | `R² ≥ 0.99` holds from close-SNR 11.9 up |
| step caps | 4,000 | **shear 12,000 (`k = 1`) / 3,000 (`k = 2`); colour 1,500 / 400** | a cap reached is still a refusal by name |
| price | 0.55 s per law at `L = 64`, on the FIRST TEN laws of the enumeration | **measured on TEN laws at a coprime stride (2411, which walks the whole group): `25.8` s per law single-core, the census `4.13` h on cores 24–31 in parallel across laws.** The first ten laws are ten consecutive odometer states with every slow fibre at its identity permutation — the weakest laws in the group and the dearest to run (all ten refuse on the shear where four of ten at the coprime stride fit; the biased price reads 27 % high, `32.7` s). The gate's sample rule is amended to the coprime stride; the biased reading is kept beside it | a sample of the enumeration's head is not a sample of the group |

Gates unchanged in substance, three letters corrected by what the amended instrument read
on FHP-I and the identity: **G1** and **plant (i)** read "REFUSES by name" as ANY refusal exit
(`NoDecay`, `LowR2`, `TooFewPoints`) — the identity's colour wave at `k = 1` exits `LowR2`
rather than `NoDecay` (it streams without damping and the fit has nothing exponential to
hold), and the colour-stays-with-its-direction plant likewise refuses where the blind rule
fits (its decay rate 6.55 times the blind rule's), which is the plant firing by substance; a
plant whose readout refuses where the blind rule fitted FIRES. **G3**'s column leg drives the
shear along the lattice direction 60° from the rows (a hexagonal symmetry axis), not the
literal `ŷ` of the freeze: the literal drive exits `LowR2` at `R² = 0.32` from sound-mode
contamination, and that reading is kept beside the gate. G0 (now on the amended parameters;
the frozen reading kept beside it), G2, P, C, S unchanged — with two additions forced by the
numbers above: **G0 also
asserts** the two-wavevector gaps on FHP-I under 10 % on both readouts (the control that
excluded `L = 64` and `128`), and **C reports** the refusal count against the 48-law preview's
27 of 48 at one seed, so a census whose refusal rate differs from its preview by more than a
factor of two is read as an instrument change, not a finding.
witness: none (two ratios under 10 %, a refusal count against its preview, a 60° leg's ratio; nothing mechanized in Lean)
The expectation of §1 stands and
is sharpened by the preview: nineteen fitted laws read `Sc` from `0.118` to `0.271`, median
`0.162`, FHP-I `0.114` — every one at least thirty-fold under the (b) line. The S stake is
unchanged; the preview is a preview and enters no branch. The amended instrument's own G0
on FHP-I: `ν = 0.5284 / 0.5271` and `D = 4.881 / 4.779` link²/step (the two chiralities),
`Sc = 0.108 / 0.110`, `ν/ν_B = 0.767` against the credited lattice-Boltzmann value — the
Boltzmann approximation's 20 % is now a measured number, reported, not a gate.

## What is NOT changed

The model, the census, the readouts' definitions, the colour-blind rule, the exits, the
Schmidt stake and its bands, the plants' carriers, the cores (24–31, never 0–23 while a water
arm runs), and the kill from experiment (`Sc_water ≈ 430`).
