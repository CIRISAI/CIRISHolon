# Pre-registration — FLUID-1: the orientation lattice — the fluid element's carrier with a closure in it: particles that carry a direction, a bond rule read off the molecular tier's measured angular map, the bond's persistence under the rent clause at a chart set by one measured retention, and the Schmidt number, the bond count, the spanning cluster and the block-chart defect asked of it

*Frozen 2026-09-06, committed ALONE, before any run. Built by the lead (the model's rule set,
the chart, the stakes) with a delegate on the instrument. FLUID-0 read branch (c): over all
4,608 REG+ collision laws of the single-species lattice gas the Schmidt number `ν/D` stays
under `0.45`, against water's `435`, so a carrier with no closure in it cannot carry the
liquid's two coefficients at any chart. CT-2 measured what a closure between two waters
looks like from the molecular side: the transfer term rises 40 % as the acceptor turns its
plane normal toward the hydrogen and stays high to 180°, and halves when the donor's O–H
bends 30° off the line; the dimer keeps its bond 55 % of frames at 293 K and always at 150 K;
the liquid box at 293 K reads 1.18 distinct bonds per molecule with its oxygen shell
water-shaped. This freeze puts those facts on the lattice tier's own machinery: each particle
of the lattice gas carries an ORIENTATION, a bond forms between neighbours when the geometry
the map favours holds, a bonded pair is a closure that moves as one and is released by the
rent clause, and the element's transport, bond count, spanning cluster and block-chart
defect are measured with the instruments the two prior campaigns built. The credited family
is the lattice water of Bell (1972) and the Mercedes-Benz model of Ben-Naim (1971) and
Silverstein, Haymet and Dill (1998); what is new is that the bond rule is READ from exact
solves and the carrier's transport is measured by name.*

misfits: contacts **M-EMPTY-SECTOR** (the expectation is in §1; a lattice whose bonds never
form is a reading, not a zero); **M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants, carriers
asserted nonzero in the sector each acts on, §5); **M-CHEAPER-THAN-ITS-PRICE** (every run
priced on FLUID-0's measured `25.8` s per single-species law at `L = 256`, times the measured
orientation overhead written to `price.json` before the counted runs); **M-EXIT-DISCRIMINATOR**
(FLUID-0's exits, unchanged; a cap is a refusal); **M-STALE-INSTRUMENT** (the bond amplitude
table is READ from CT-2's records at run time, never typed; the chart's one number is read
from CT-2's arms record); **M-VACUOUS-SUCCESS** (every run reports its bond count, its
collisions and its transitions; a run with zero bonds formed cannot pass a bond gate);
**M-NULL-MISSTAKE**; **M-FIXED-POINT-TRAJECTORY** (two seeds, FLUID-0's rule); **M-UNTESTED-GAP**
(§4); **M-FORMAT-FLOOR**; **M-FLOOR-UNSTAKED** (FLUID-0's fit floor rules, unchanged);
**M-VOLUME-SCALE** (`L = 256`, FLUID-0's hydrodynamic size; the block charts at `b ∈ {4, 8,
16, 32}` are node LG's); **M-BASE-RATE-OMITTED** (refusal rates beside every distribution);
**M-FIRST-VIOLATION-ONLY** (every gate names every failing leg); **M-TAG-AS-PROPERTY** (a
bond is a recorded pair, never a label on a particle); **M-BARE-CHARGE**, **M-HOMOG**,
**M-COND-PROBE**, **M-DEVICE-CLASS**; **M-MAINTENANCE-LENS** (contacted by keyword: the rent clause
here is a break probability set from a READ retention and then measured back on the lattice by
G3 and G0 — the bond's persistence is the rule's own count of held steps, never a lens's
verdict on a trajectory). Not contacted: the rest of the registry.

## 0. What is built and measured

**The model.** FHP-6 on the `L × L` torus, `L = 256`, density `d = 0.2` per direction, the
collision law FHP-I (both chiralities as FLUID-0 read them), the colour-aware step of
FLUID-0 generalised: every particle carries an ORIENTATION `o ∈ {0, …, 5}`, one of the six
lattice directions, the direction its donor arm points. Streaming carries orientation with
the particle; at a collision the outgoing particles take the incoming orientations
redistributed uniformly at random by the crate's counter hash (the colour-blind rule of
FLUID-0), UNLESS the particle is bonded (below), in which case its orientation is kept.

**The bond rule, read from the map.** Two particles on neighbouring cells `i` and `j` (the
link from `i` to `j` in direction `δ`) can bond when the donor arm of one points along the
link at the other — `o_i = δ` (particle `i` donates to `j`) — which is the donor condition
the map measured steep (half the transfer at 30°; on the lattice the arm either points along
the link or it does not). The bond's strength depends on the ACCEPTOR's orientation relative
to the link, `φ = angle(o_j, −δ)` in the six lattice values `{0°, 60°, 120°, 180°, …}`, through
an amplitude table `A(φ)` READ at run time from CT-2's tilt family at 2.9 Å
(`ct2/node_tilt_R2.9_t{60,120}.json`, `ct2/sector_linear_R2.9.json` for 0°, the record's
`tilted_R2.9`, `tilt_R2.9_t180`; the value at each lattice angle the measured `E_CT` at the
nearest measured tilt, normalised to 1 at 0°): the acceptor factor rises away from the line
and stays high, as measured, and no shape is declared for it.

**The closure and its rent.** A bonded pair is a closure: on the streaming step it moves as
ONE — both particles take the mover's direction, chosen per step by the counter hash between
the two — so a bond traps a particle while the pair still carries momentum (the property
FLUID-0 said the element must have). Bonds form on a link where the rule allows, with
probability `p_form = 1` (the geometry is the condition), and BREAK each step with probability
`p_break(φ) = exp(−A(φ)·E₀ / kT_lat)`, so the stationary fraction of a bond's lifetime it is
held is set by the rent clause at the lattice's temperature. A particle holds at most two
bonds (a donor arm and an acceptor role), the way a water donates twice at most on the lens.

**The chart, set by ONE measured number.** `E₀/kT_lat` is fixed so that a linear bond
(`φ = 0°`) has the retention CT-2's dimer arm measured at 293 K: `f = 0.5526`
(`ct2/arms.json`, `dimer_293_seam.f`, read at run time). A second chart at `f = 1.000`'s
neighbour is not needed: the 150 K arm read 1.000, and a `p_break` of 0 is the freeze's
`T → 0` limit, run as the cold control below. No other number crosses from the molecular
tier; the lattice's length and time units are not set here (the Schmidt number does not need
them, and the bond count is a pure number).

**The instruments.** FLUID-0's `transport.rs` readouts, unchanged in rule, on the momentum
(shear viscosity `ν`, both wavevectors) and on a passive colour (tracer diffusion `D`): the
colour rides the particle whether bonded or not, so `D` is the self-diffusion of a particle
that is sometimes trapped, which is what a molecule's `D` is. New readouts: the bond count per
particle; the bond graph's largest connected component as a fraction of all particles and
whether it spans the torus (the lens's largest-domain instrument on the explicit bond edge
set); and node LG's block-chart witness rate at `b ∈ {4, 8, 16, 32}` on the orientation lattice.

**The runs.** At the chart (`f = 0.5526`), both chiralities, two seeds, FLUID-0's window and
floor rules and step caps; the cold control (`p_break = 0`); the no-bond control (bonds
forbidden: this must reproduce FLUID-0's FHP-I readings); two plants (§5). Each run's price
measured on its first 200 steps and written before the counted steps.

## 1. The expectation, written before any run (M-EMPTY-SECTOR discharged)

At the chart the bond fraction should sit between the molecular box's reading (1.18
distinct bonds per molecule, ceiling 2 on the lens) and the lattice's own ceiling (2 per
particle); the bond graph is expected to percolate (the box's reading is above the
four-connected threshold); and the Schmidt number is expected to RISE from FLUID-0's `0.11`
by the trapping — by how much is the stake, not the expectation. The no-bond control is
expected to read FLUID-0's `ν = 0.528, D = 4.88, Sc = 0.108` within 5 %. Water's `Sc ≈ 435`
is the kill (FLUID-0 §1's sources), named as such wherever it appears.

## 2. Gates

- **G0 — the rule is the map.** The amplitude table `A(φ)` printed with the record and node
  it was read from at each of the six lattice angles; `A(0°) = 1`; `A(60°) > 1` and `A(120°) >
  1` (the map's rise), EXACT as inequalities on read values; the chart's `E₀/kT_lat` printed
  with the retention it was set from and the run's own measured linear-bond retention within
  5 % of `0.5526` (the rent clause reproduced on the lattice).
  witness: none (values read, one retention reproduced)
- **G1 — the no-bond control is FLUID-0.** With bonds forbidden, `ν`, `D` and `Sc` within 5 %
  of FLUID-0's FHP-I lines (both chiralities), the readouts `Fitted`.
  witness: none (three ratios)
- **G2 — the ledger, exact.** Mass, both momenta, the orientation census (the count of each
  orientation is conserved by streaming and permuted by collisions, so its TOTAL is exact),
  and the bond count's balance (formed − broken = held) integer-identical at every step.
  witness: none (conserved integers)
- **G3 — detailed balance of the bond.** On a link held at fixed geometry (a two-particle
  scene with streaming off), the measured held fraction equals `1/(1 + p_break)` within 2 %
  over 10⁵ steps, for `φ = 0°` and `φ = 120°`.
  witness: none (two fractions)
- **B — the bond count and the phase.** At the chart: bonds per particle (a number in `[0, 2]`)
  and the spanning fraction, both reported; **(a)** the bond graph spans the torus and the
  bonds per particle are within a factor of 1.5 of the molecular box's `1.18`; **(b)** it
  spans but the count is outside that band; **(c)** it does not span.
  witness: none (a count and a component)
- **S — the Schmidt stake.** At the chart: **(a)** `Sc ≥ 100` (water's ratio reached by
  closure alone — a single-species carrier with bound pairs suffices for the element);
  **(b)** `10 ≤ Sc < 100` (closure moves the ratio by the two decades FLUID-0 said no rule could,
  and the rest is the chart's, named); **(c)** `Sc < 10` (trapping at this bond fraction does
  not separate momentum from particles; the element needs a stronger closure than a pair, or
  the network's own dynamics).
  witness: none (a ratio against declared bands)
- **W — the block chart's defect with a closure in the carrier.** Node LG's witness rate at
  `b ∈ {4, 8, 16, 32}` against `W(b) = 1 − (b−2)²/b²`: **(a)** within 10 % at every `b` — the
  closure inside the block does not change the block's closure; **(b)** above it at small `b`
  and converging — bonds crossing the block face add to the defect by a measured amount;
  **(c)** below it — a closure in the carrier makes the block MORE closed than its boundary
  fraction, which would be the first coarse chart on this tier to close by something other
  than conservation.
  witness: `closed_iff_fiber_invariant` (node LG's; the rate is against its law)
- **P — the price.** Measured on the first 200 steps of each run, written before the counted
  steps; every run within `0.1×`–`10×` of its own price.
  witness: none (a price, recorded)

## 3. What each outcome means

S (a) with B (a) is a fluid element with water's closure ratio and water's bond count, derived
from the molecular tier through one measured retention, and CUBE-0 is framed on it under the
acuity rule. S (b) says closure is the right lever and names how much of water's ratio a
pair closure buys; the chart from LIQUID-1's `D` and a measured viscosity then sets the rest.
S (c) says a pair is not enough of a closure and the network's dynamics must be in the
carrier. W (c) would be a finding about the fluid tier itself, entered as such.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

One collision law, one density, one chart set by one number, two seeds; a two-dimensional
lattice standing in for a three-dimensional liquid, with six orientations for a continuum of
them. Nothing here is a claim about water beyond the two kills named.

## 5. Plants

- **(i) The acceptor factor flattened.** `A(φ) ≡ 1`: the bond count must change by more than
  20 % from the chart's run. Carrier: `A(120°) − 1 ≥ 0.2` on the read table, asserted nonzero
  in the sector the plant acts on (the amplitude table).
- **(ii) The rent doubled.** `E₀/kT_lat` doubled: the linear-bond retention must move by more
  than 20 %. Carrier: the chart's retention `0.5526`, asserted nonzero and under 1 in the
  sector the plant acts on (the break probability).

## 6. Discipline

Instrument: `holon-lattice/src/orientation.rs` (the orientation-carrying state, the bond
rule, the bonded streaming, the break rule) beside `transport.rs`; unit tests (G2's
conservation on 500 steps; G3's detailed balance; the no-bond path bit-identical to FLUID-0's
colour-aware step). Runner `holon-lattice/examples/fluid1_lattice.rs`: `gate` (G0–G3, the
plants, the prices), `run` (the chart runs, the cold control, the no-bond control, the
block-chart witness rates), `read` (B, S, W). JSON under `conformance/mesh/fluid1/`; results
`FLUID1_RESULTS.md`. Cores 24–31 only. The only numbers from outside the crate are the
amplitude table and the retention READ from CT-2's records and the kill from experiment.
