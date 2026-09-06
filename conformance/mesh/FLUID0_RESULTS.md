# FLUID-0 — results

*Freeze `FLUID0_PREREG.md` (c2b2819, alone); `FLUID0_AMENDMENT_1.md` (a4c8b1c, alone, before
any census line). Instrument `holon-lattice::transport` and the colour-aware step beside
`Lattice::advance` (delegate; 49eda04): a shear wave's decay for the kinematic viscosity, a
colour wave's decay for the tracer diffusion, both with named exits; runner
`examples/fluid0_sweep.rs` (`gate`, `sweep`, `read`); the probe `examples/fluid0_probe.rs` that
measured the frozen instrument's failure. Records under `conformance/mesh/fluid0/`:
`gate_as_frozen.*` (the frozen instrument's own reading, kept), `gate.*` and `price.json`
(the amended instrument), `diag.*`, `census.jsonl` (4,608 lines), `sweep.done`,
`census_summary.json`. Every file validates as JSON.*

## The verdict, first

**Branch (c), as staked before any law was read: no single-species FHP-6 collision law
carries water's two transport coefficients at once, whatever length and time a chart assigns
the lattice.** Over the 1,707 laws whose four readings fitted, the Schmidt number `ν/D` runs
from `0.107` to `0.449`, median `0.161`; water's is about `435`. The largest in the census is a
thousandfold under water and thirtyfold under the lowest band the freeze offered. The bridge
from the liquid to a fluid element cannot be a single-species lattice gas with both
coefficients: it needs a carrier with a second species, or a chart that maps the viscosity
only. That was the expectation of §1, and the census confirmed it with a number.

Two findings on the way there, both entered in full:

1. **The instrument as frozen failed its own first gate, and the record keeps it.** At
   `L = 64`, one seed, the window opening at step `L` and closing at a tenth of the start,
   every one of the sixteen FHP-I readings refused. The probe measured the three causes: the
   colour mode's lifetime (13.7 steps) is shorter than the freeze's transient (64), so the
   window opened on a dead signal; the tenth-of-start floor (18.9) sat under the equilibrium
   noise (28.8 RMS); and `L = 64` is not hydrodynamic for FHP-I (a 33 % two-wavevector gap,
   with a mean free path of ten links against a wavelength of 28). Every frozen number was
   typed from the shear intuition and none had been measured. Amendment 1 replaced each by a
   measured rule — `L = 256` (the smallest size passing the freeze's own 10 % control on both
   readouts), two seeds, the window from four mean free times to a floor of twelve times
   the measured noise over `√S`, amplitudes `0.10` shear and `1.0` colour — and the amended
   instrument passed all seven gates. Stake-gates-from-the-freeze's-own-arithmetic, again.
2. **The price gate's sample was the group's weakest corner.** The freeze priced the census
   on the first ten laws of the enumeration, which are ten consecutive odometer states with
   every slow fibre at its identity: the dearest laws there are, reading 27 % high (`32.7` s
   against `25.8`). The amendment moved the sample to a coprime stride and kept the biased
   reading beside it. The census's own total, `110,250` core-seconds, fell inside the band
   of the stride price (`118,890` projected).

| gate | verdict | the number |
|---|---|---|
| G0 — the instrument on FHP-I | **FAIL as frozen; PASS as amended** | frozen: 8 of 8 refuse (`NoDecay`), both chiralities; amended: 8 of 8 `Fitted`, `R²` 0.9968–0.99995, both wavevector gaps under 10 % (shear 2.0 %, tracer 8.8 % on the probe; `ν(k=1)/ν(k=2)` 0.528/0.539 and 0.527/0.521, `D` 4.88/5.13 and 4.78/5.30) |
| the credited comparison | **reported, not gated** | `ν/ν_B(0.2) = 0.767` against the FHP-I lattice-Boltzmann value `0.689` (Frisch et al. 1987; Hénon 1987): the approximation's error is now a measured 23 % |
| G1 — the identity law refuses | **PASS** (letter amended: any refusal exit) | `NoDecay` on both shear legs and the `k = 2` colour, `LowR2` on the `k = 1` colour; `collisions_fired = 0`; no number entered |
| G2 — the ledger, exact | **PASS** | 16 runs, 2 members each, 35,129 step-checks: mass, `P_x`, `P_y` and the red count integer-identical |
| G3 — isotropy on FHP-I | **PASS** | the 60° leg `0.5559` against the rows' `0.5284`, relative `0.0495` (the literal `ŷ` drive exits `LowR2` at `R² = 0.32`, sound-contaminated, kept beside) |
| plant (i) — the colour-blind rule removed | **FIRES** (by the refusal prong, named) | the plant's readout refuses where the blind rule fits; its decay rate `6.55×` the blind rule's |
| plant (ii) — linearity | **PASS** | `ν = 0.5284` at `A = 0.10` against `0.5246` at `0.05`: 0.7 % against a 5 % stake |
| P — the price | **PASS** | `25.80` s per law at the coprime stride (first-ten, biased: `32.76`); the census `110,250` s of core time, `13,790` s wall on 8 cores, inside `0.1×`–`10×` of `118,890` |
| C — the census, read | **complete** | 4,608 of 4,608 laws, 4,608 distinct indices; 1,707 fitted on all four readings; 752 `PreHydrodynamic` (the two wavevectors disagree by more than 10 %); laws refusing anything: 2,901 (63 %), against the 48-law preview's 56 % — ratio 1.12, within the factor of two; `ν` 0.331–2.811 (median 0.463), `D` 2.04–7.73 (median 3.19) link²/step |
| S — the Schmidt stake | **BRANCH (c)** | largest `Sc = 0.4486` (index 654, digest `4b4ceae045ea57af`); water's `434.8` is the kill (Kestin–Sokolov–Wakeham 1978 for `η`; Krynicki–Green–Sawyer 1978 and Mills 1973 for `D`) |

## 1. What the census looks like

The tracer readout fits almost every law (4,550 of 4,608 at `k = 1`); the shear readout
fits fewer than half (2,128), the rest refusing on `R²` — a shear wave on a weakly colliding
law does not damp exponentially at this size, and the instrument says so by name instead of
entering a number. Among the 1,707 laws that fit on all four readings, viscosity spans a
factor of 8.5 and diffusion a factor of 3.8, and their ratio spans only a factor of 4.2:
momentum and particles are carried by the same collisions, and no rule in the group can pull
them apart. FHP-I sits at `Sc = 0.108` and `0.110` (its two chiralities), near the bottom of
the range; the largest `Sc` belongs to a law that damps shear faster than it lets colour
through, and it is still `0.45`.

## 2. What it means for the bridge

Viscosity over diffusion is momentum transport over particle transport, and both are
length² per time, so the ratio does not care what a chart calls a link or a step. Water's
hundreds say its particles are trapped while stress passes: a network. A gas of single
particles cannot make that ratio by any collision rule, because it has nothing to trap a
particle in. So the fluid element above the liquid needs a closure in its carrier — a second
species, or bound pairs — and that is the object contract's own statement of what a tier
is. FLUID-1 is framed on that finding rather than on a coefficient the liquid has not yet
read: the two-species lattice gas (a tracer whose diffusion is set apart from the
momentum's), its `Sc` swept, and the chart that maps it to LIQUID-1's `D` and a viscosity
the engine does not yet measure.

## 3. Bookkeeping, declared

- The frozen instrument's failure is recorded in `gate_as_frozen.*` at its original
  timestamps; the amendment's every number came from the probe and is in the record.
- The runner's declared choices: the fit rule is data (`FitRule`), so the frozen rule and the
  amended one are one code path with two values; the census runs eight workers each taking
  a whole law single-threaded, so every per-law `seconds` is a single-core number; resume is
  by index; the plant gate names which prong fired; `diag` runs at the frozen parameters under
  its own local constants so the amendment cannot rewrite what convicted it.
- The machine suspended for about six hours during the sweep; the process's own clock
  (monotonic) excludes the suspend, and the price check is on that clock.
- The only numbers from outside the crate are the two experimental kills of §1 and the
  credited Boltzmann comparison, each named as such where it appears.
