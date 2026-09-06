# Pre-registration — FLUID-0: the census's transport character — shear viscosity and tracer diffusion measured for every REG+ collision law, and the Schmidt number as the chart-invariant test of whether a single-species lattice gas can carry the liquid's two coefficients

*Frozen 2026-09-06, committed ALONE, before any law is read. Built by the lead (the freeze,
the Schmidt stake) with a delegate on the instrument and the sweep. Node LG certified the
fluid tier as its own object and read that its block charts do not close except by
conservation (`LG_RESULTS.md`: the defect is EXACTLY the boundary fraction, for every one of
the 4,608 REG+ laws). LIQUID-1, when it runs, will read water's self-diffusion coefficient
from the seam law; a fluid element built on the lattice tier would need that coefficient and
the viscosity as its two parameters. The bridge between the two tiers is therefore a
question that can be asked NOW, without waiting for the liquid: does any collision law in
the census carry both coefficients in water's ratio? Viscosity and diffusion are both
length² per time, so their ratio — the Schmidt number `Sc = ν/D` — is invariant under
whatever length and time a coarse chart assigns to the lattice. Water's is about 430. A
single-species lattice gas moves momentum and particles by the same collisions and is
expected to sit near 1. This freeze measures `ν`, `D` and `Sc` for every law and stakes the
answer. It does not depend on any water campaign, touches no water record, and runs on the
cores CT-2 does not use.*

misfits: contacts **M-EMPTY-SECTOR** (the expectation is written in §1 before the sweep; a
law whose waves do not decay is a REFUSAL by name, never a zero or an infinity);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the sector
each acts on, §5); **M-CHEAPER-THAN-ITS-PRICE** (the price per law measured on the first ten,
written before the census sweep; a sweep returning under a tenth of it is refused);
**M-EXIT-DISCRIMINATOR** (every reading records its exit: `Fitted`, `NoDecay`, `PreHydrodynamic`,
`TooFewPoints`; a run that ends by its step cap without a fit is a refusal);
**M-STALE-INSTRUMENT** (the census is enumerated by the crate at run time, each law identified
by its index in that enumeration AND the FNV digest of its 64-entry table; FHP-I is located by
its table, never by a name — **M-TAG-AS-PROPERTY**); **M-VACUOUS-SUCCESS** (every run reports
`collisions_fired > 0`; a run with zero collisions is the identity law's and is refused as
such); **M-NULL-MISSTAKE** (a refusal is not a measurement of zero transport);
**M-FIXED-POINT-TRAJECTORY** (one seed, declared; density, amplitude, lattice size declared);
**M-UNTESTED-GAP** (§4: one density, one lattice size, two wavevectors, one seed);
**M-FORMAT-FLOOR** (every record validates as JSON; the census as JSON lines);
**M-FLOOR-UNSTAKED** (the fit's floor: `R² ≥ 0.99` on at least 20 points; the decay window
from step `L` to the first step under a tenth of the starting amplitude);
**M-BASE-RATE-OMITTED** (the refusal rate across the census is reported beside the
distribution); **M-VOLUME-SCALE** (`L = 64` declared; the two-wavevector check is the
finite-size control, and a law whose `ν(k₁)` and `ν(k₂)` disagree by more than 10 % is read
`PreHydrodynamic`, not averaged); **M-PRESENTATION-VERDICT**; **M-COND-PROBE**; **M-DEVICE-CLASS**; **M-BARE-CHARGE**
(contacted by keyword only: the model carries no charge; the word appears as "discharged");
**M-HOMOG** (contacted by keyword: the lattice's density is uniform BY CONSTRUCTION and the
seeded waves are the declared, measured departure from it; "local state" is the crate's term
for a cell's six bits). Not contacted: the rest of the registry.

## 0. What is built and measured

**The model.** FHP-6 on the `L × L` torus, `L = 64`, as node LG built it (`holon-lattice`):
six directions on the hexagonal lattice, 64 local states, the conserved label `(N, P_x,
P_y)` per state, 53 sectors; the REG+ collision laws are the sector-preserving bijections,
`Model::collision_laws()` enumerating all `4,608`, the identity law among them; FHP-I's two
chiralities by `Model::fhp_i`. Lattice unit: the link length; time unit: one step; the row
spacing is `√3/2` links, so a wave with `k_index` periods across the `L` rows has
`k = 2π·k_index / (L·√3/2)` per link.

**The instrument** (`holon-lattice::transport`, new; a colour-aware step beside `advance`):

- `shear_viscosity(model, law, L, d, A, k_index, steps, seed)`. Each cell's six directions
  seeded independently with occupation probability `p_dir(j) = d + A·c_{dir,x}·sin(2π
  k_index j / L)` (clamped to `[0, 1]`), a shear wave of the x-momentum along the rows;
  evolved by the crate's own step; every step the row x-momenta (`line_momenta`) projected
  on `sin(2π k_index j / L)` → `M(t)`; from step `L` (the kinetic transient) to the first
  step at which `|M|` falls under a tenth of `|M(L)|`, or the cap, the least-squares slope of
  `ln|M(t)|` against `t` → `γ`; `ν = γ / k²` in link²/step. Reported: `ν`, the window, the
  point count, `R²`, `collisions_fired`, and the exit.
- `tracer_diffusion(model, law, L, d, A, k_index, steps, seed)`. The same seeding at uniform
  density with every particle coloured red with probability `½ + (A/2)·sin(2π k_index j /
  L)` — a colour wave on a fluid at rest. The colour is a second bit-plane carried with its
  particle: streaming moves colour bits with occupation bits, a solid cell reverses both;
  at a collision the state maps `s → C[s]` exactly as in `advance` (the same chirality hash),
  the number of red particles is CONSERVED, and the red bits are reassigned uniformly at
  random among the outgoing occupied directions by the crate's counter hash (the
  colour-blind rule of the two-species lattice gas: d'Humières, Lallemand and Searby 1987;
  Rothman and Zaleski 1997). The row colour excess `Σ_dir colour − ½·Σ_dir occupancy`
  projected on the sine → its decay → `D = γ / k²`. Reported as above.
- Both REFUSE by name when the amplitude does not fall monotonically over the window
  (`NoDecay`, the identity law's case), when the fit has fewer than 20 points
  (`TooFewPoints`), or when `R² < 0.99`; and a law whose two wavevectors disagree by more
  than 10 % is read `PreHydrodynamic` and its numbers reported but NOT entered into the
  distribution.

**The runs, per law.** `d = 0.2`, `A = 0.05`, `k_index ∈ {1, 2}`, `steps = 4,000`, seed
`0x464c_5549_4430`, so four runs per law (two shear, two colour); `ν` and `D` are the `k =
1` readings when the `k = 2` readings agree within 10 %; `Sc = ν / D` where both are
`Fitted`.

**The census.** All 4,608 laws, detached and resumable (one JSON line per law in
`census.jsonl`; a law already present is skipped), on cores 24–31 and never on 0–23 while
CT-2 runs. Each line: the index, the digest, whether it is FHP-I (either chirality) or the
identity, `ν(k=1), ν(k=2), D(k=1), D(k=2)`, their exits, `Sc`, `collisions_fired`, seconds.

**The comparison, credited and REPORTED, not gated.** For FHP-I the lattice-Boltzmann
kinematic viscosity is `ν_B(d) = 1 / (12·d·(1−d)³) − 1/8` (Frisch, d'Humières, Hasslacher,
Lallemand, Pomeau and Rivet 1987, Complex Systems 1, 649; Hénon 1987). It is typed from the
literature, so it is written beside the instrument's FHP-I reading and is not a stake
(stake-gates-from-the-freeze's-own-arithmetic).

## 1. The expectation, written before the sweep (M-EMPTY-SECTOR discharged)

A single-species lattice gas carries momentum and particles by the same collisions, and the
published FHP viscosities and diffusivities are of one magnitude at moderate density. The
expectation is therefore branch **(c)** of S below: the largest Schmidt number in the census
is under 10, and no single-species FHP-6 law carries water's two coefficients. Water's
kill band, from experiment and named as a kill wherever it appears: `Sc_water ≈ 430` at
293 K — `ν = 1.00e-2 cm²/s` (`η = 1.00` mPa·s, Kestin, Sokolov and Wakeham 1978, over `ρ =
0.998` g/cm³) and `D = 2.3e-5 cm²/s` (Krynicki, Green and Sawyer 1978; Mills 1973 — the same
`D` LIQUID-1 stakes). A law reaching `Sc ≥ 100` would be the surprise, and (a) says so.

## 2. Gates

- **G0 — the instrument on FHP-I.** At `d = 0.2` both shear runs and both colour runs exit
  `Fitted` with `R² ≥ 0.99` on at least 20 points; `ν(k=1)` and `ν(k=2)` within 10 %;
  `D(k=1)` and `D(k=2)` within 10 %; the Boltzmann `ν_B(0.2)` written beside `ν` with the
  ratio (REPORTED). Both chiralities read; they agree within 10 %.
  witness: none (a fit, its residual, two ratios)
- **G1 — the identity law refuses.** With the identity collision both readouts exit `NoDecay`
  (the wave streams and does not damp) and no number is entered; `collisions_fired = 0` is
  reported as the reason.
  witness: none (an exit, a count)
- **G2 — the ledger, exact.** In every run mass and both momenta are integer-identical at
  every step (the crate's ledger), and the red-particle count is integer-identical at every
  step (EXACT).
  witness: none (conserved integers)
- **G3 — isotropy on FHP-I.** The shear wave of the y-momentum along the columns (the
  crate's column momenta) gives `ν` within 10 % of the row reading.
  witness: none (a ratio)
- **P — the price.** Seconds per law measured on the first ten laws of the enumeration and
  written to `price.json` before the census; the sweep's total within `0.1×` to `10×` of
  `4,608` times that price.
  witness: none (a price, recorded)
- **C — the census, read.** Every one of the 4,608 laws has a line; the counts of each exit;
  the distribution (minimum, median, maximum) of `ν`, `D`, `Sc` over the `Fitted` laws; the
  place of FHP-I and of the identity in it; the number of `PreHydrodynamic` laws.
  witness: none (4,608 lines and their summary)
- **S — the Schmidt stake.** Over the `Fitted` laws: **(a)** some law reaches `Sc ≥ 100` — a
  single-species candidate for water's ratio exists, and it is named by digest; **(b)** the
  largest `Sc` is in `[10, 100)`; **(c)** the largest `Sc` is under 10 — no single-species
  FHP-6 element carries water's two coefficients at once, whatever chart maps its units,
  and the fluid element's carrier needs a second species (a tracer whose diffusion is set
  apart from the momentum's) or a chart that maps the viscosity only.
  witness: none (a maximum against declared bands)

## 3. What each outcome means

S (c), the expectation, says the bridge from the liquid to the fluid element cannot be a
single-species FHP-6 gas with both of the liquid's coefficients; FLUID-1 is then framed on a
two-species carrier or on viscosity alone, before LIQUID-1 has read a number, which is the
wall time this freeze buys. S (a) says a candidate exists and names it; FLUID-1 then asks
whether that law's `ν` and `D`, at the chart's units set by ONE of them, reproduce the other
within LIQUID-1's band. S (b) names the gap in orders of magnitude. G0 failing says the
instrument does not read FHP-I's hydrodynamics at this size and density, and the census is
not run.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

One density, one lattice size, two wavevectors, one seed, one amplitude; 4,608 laws of one
model. Nothing here is a claim about the molecular dynamics or about water; the only water
numbers are the two experimental kills of §1.

## 5. Plants

- **(i) The colour-blind rule removed.** The redistribution replaced by "colour stays with its
  direction index": on FHP-I `D` must change by more than 20 %. Carrier: `D_{FHP-I}(k=1) ≥
  0.05` link²/step, asserted nonzero in the sector the plant acts on (the colour readout).
- **(ii) Linearity.** The shear amplitude doubled to `A = 0.10`: `ν` on FHP-I must agree with
  the `A = 0.05` reading within 5 %. Carrier: `ν_{FHP-I}(k=1) ≥ 0.05` link²/step, asserted
  nonzero in the sector the plant acts on (the shear readout).

## 6. Discipline

Instrument: `holon-lattice/src/transport.rs` (the two readouts, the fits, the exits) and a
colour-aware step in `holon-lattice/src/lattice.rs` beside `advance` (the same table, the same
chirality hash, colour conserved and redistributed by the crate's counter hash); unit tests
(the red count conserved; the identity law's `NoDecay`; the k² law on FHP-I). Runner
`holon-lattice/examples/fluid0_sweep.rs`: `gate` (G0–G3, the plants, `price.json`), `sweep`
(the census, detached, resumable, `census.jsonl`, `sweep.done`), `read` (C and S,
`census_summary.json`). JSON under `conformance/mesh/fluid0/`; results `FLUID0_RESULTS.md`.
Cores 24–31 only. No number enters from outside the crate except the two experimental kills
of §1 and the credited Boltzmann comparison, each named as such where it appears.
