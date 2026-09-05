# Pre-registration — FIELD-2: does the fixed-charge field HOLD a hydrogen bond? — a bonded start, two temperatures, the dimer and the cyclic tetramer, with the field's own binding and the charge's geometry sensitivity measured first

*Frozen 2026-09-05, committed ALONE, before the runner existed. Built by the lead. FIELD-1
landed the field as channel 1 with every gate green and read S1 branch (b): from a
parallel-dipole square at 293 K, fixed dipole-exact charges formed no hydrogen bond and
the waters drifted apart. That null had four named suspects — the start configuration,
`kT` against the field's binding, the charges' geometry dependence, and the O···H pair
table's role at bond distance. This freeze isolates the two cheap ones by a RETENTION
measurement: start the molecules hydrogen-bonded by the rung-1 lens's own criterion and
ask whether the field keeps them so, at 293 K and at 150 K, with the field's binding energy
at that start and the charge's variation over thermal geometries measured and written
BEFORE the arms run, so the outcome is expected rather than explained. Polarisation and
the table's role are FIELD-3's if this freeze says they are needed.*

misfits: contacts **M-VACUOUS-SUCCESS** (every arm reports its frame count and the field's
energy; an arm whose molecules never came within the lens's reach is VOID for retention
and says so); **M-NULL-MISSTAKE** (retention is staked on the lens's frozen criterion, the
same instrument rung 1 and FIELD-1 read, never on a distance of the field's own);
**M-EXIT-DISCRIMINATOR** (three outcomes named per system, none the default);
**M-UNTESTED-GAP** (the expectation is derived from the measured binding against `kT`
before any arm runs, so a null at 293 K is either predicted or a finding, and the 150 K
arm exists to tell which); **M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants with carriers
asserted nonzero in the sector each acts on — §4); **M-STALE-INSTRUMENT** (this freeze
alone; runner, JSON and results together); **M-FIXED-POINT-TRAJECTORY** (both arms start
from the same bonded configuration and the same seed; the OFF arm is the control that the
start alone does not hold the bond); **M-BARE-CHARGE**, **M-HOMOG** (the words "charge",
"local" appear; classical charges, nothing homogeneous); **M-COND-PROBE** ("inside the"
appears; a force term, not a post-step operator); **M-DEVICE-CLASS** (native `f64`, one
class). Not contacted: the rest of the registry.

## 0. What is measured first, and the expectation it fixes

- **M1 — the field's binding at the start.** For each system's staked start configuration,
  `E_field(start) − E_field(separated)`, from the engine's own field term
  (`Sim::field_energy_of` at fixed positions; "separated" is the same molecules moved to
  40 bohr apart). Written beside `kT` at 293 K (`9.28e-4` Ha) and 150 K (`4.75e-4` Ha).
- **M2 — the charge's geometry sensitivity.** `q_H` from the library's dipole-exact model
  (`embed::monomer`) over the monomer's thermal range: `r ∈ {pin − 0.15, pin, pin + 0.15}`
  bohr × `θ ∈ {pin − 15°, pin, pin + 15°}`, nine solves; `max |q − q_pin| / q_pin` reported.
- **The expectation, stated from M1 before the arms run:** if the field's binding at the
  start is at least `2 kT` the bond is expected to hold at that temperature; if under `kT`
  it is expected to break; between, no expectation is staked. M2 above `0.10` names the
  geometry dependence as material for FIELD-3 regardless of the arms; under `0.05` it is
  ruled out as FIELD-1's culprit.

## 1. The systems and the arms

**System D — the dimer.** Two rigid-at-start water monomers at EMBED-1's pin in DIMER-1's
`LINEAR` arrangement (donor O–H on the O···O axis, acceptor's C₂ axis on it) at
`R_OO = 5.5` bohr, which the lens reads as one hydrogen bond; open box 30 × 30 × 30 bohr,
thermostat, seeded momentum-free velocities.

**System T — the cyclic tetramer.** Four monomers on a square of side `5.5` bohr, each
donating one O–H to the next around the ring, the free hydrogens alternating above and
below the plane; the lens reads four hydrogen bonds at the start; open box 34 bohr cubed.

**Arms**, each from the same start and seed: field OFF and field ON, at 293 K and at
150 K — four arms per system. 2,000 frames of settling with the thermostat, then 20,000
frames counted, one frame one integrator step (as FIELD-1). Counted per frame by the
rung-1 lens: the number of inter-molecular hydrogen bonds; `f` is the fraction of frames
with at least one, `n̄` the mean count.

## 2. Gates

- **G0 — the books, on the new scenes.** Each ON arm: the receipt columns sum to `w_ext`
  and the honest drift peak is under a tenth of the enabling transition (FIELD-1's
  AMENDMENT 3 reading); momentum residual under its bound. 4 arms.
  witness: none (engine ledger and conservation gates)
- **G1 — the start is bonded.** At frame 0 the lens reads `≥ 1` hydrogen bond on the dimer
  and `≥ 3` on the tetramer, else the system is VOID (the start is not what was staked).
  2 systems.
  witness: none (the instrument read on the staked start)
- **S1 — retention, per system and temperature.** With `f_ON` and `f_OFF` over the counted
  frames: **(a)** at 293 K `f_ON ≥ 0.5` and `f_ON ≥ 10 · f_OFF` ⇒ fixed charges hold the
  hydrogen bond; FIELD-1's null was its start; rung 1's carrier gets this field. **(b)** (a)
  fails at 293 K but holds at 150 K ⇒ the field binds and `kT` unbinds at this charge
  strength; FIELD-3 stakes polarisation and the charge's geometry dependence. **(c)** fails
  at both ⇒ fixed charges do not hold water's hydrogen bond in this engine at all; FIELD-3
  stakes the O···H table's role at bond distance first. `f_OFF ≥ 0.5` on any arm makes
  that arm's ratio unreadable and is reported as the bare force law holding the bond on
  its own. 2 systems × 2 temperatures.
  witness: none (a measured population against a frozen instrument)

## 3. What each outcome means

(a) is the payoff: the engine's water bonds to water for a reason it derived, and the
H-bond network carrier is run next with the field on. (b) is a temperature fact with a
named next model. (c) is the finding that says the field is not where the bond lives in
this basis — the covalent tail in the O···H table would then be the thing to read.

## 4. Plants

- **(i) The sign.** `E_field` negated in the ON arms (FIELD-1's plant iii, on this scene).
  S1 must fail on every ON arm with the plant on: `f_ON(plant) ≤ f_OFF + 0.05`. Carrier:
  `|E_field(start)| ≥ 1e-4` Ha, asserted nonzero in the sector the plant acts on.
- **(ii) The start.** FIELD-1's parallel-dipole square in place of the tetramer's ring,
  field ON at 293 K: `f_ON ≤ 0.05`, reproducing FIELD-1's null on the same field. Carrier:
  `E_field(square start) − E_field(ring start) ≥ 1e-3` Ha, asserted nonzero in the sector
  the plant acts on.

## 5. Discipline

Runner `holon-render/examples/field2_hbonds.rs`, JSON per arm under
`conformance/water_observatory/field2/`, M1 and M2 written first to `expectation.json`;
results `FIELD2_RESULTS.md` committed with the runner. No number enters from outside the
engine.
