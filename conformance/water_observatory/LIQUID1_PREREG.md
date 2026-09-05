# Pre-registration — LIQUID-1: the periodic liquid — 128 waters at the state point, the seam law under Ewald, and the first three readings of water asked whether they are water's

*Frozen 2026-09-05, committed ALONE, before any liquid box existed. Built by the lead (the
engine's seam-aware image rule) with a delegate on the readouts and the runner. CONDITIONED:
this freeze runs only on the first seam-law harvest whose no-hole gate admits it and whose
retention reads (a) on the dimer and the ring at 293 K (FIELD-8's, or a successor's, named
in the results file); until one exists it is not run and its results file says so. Everything below is derived: the charges from the monomer's density
(FIELD-1), the unit as a closure reading (FIELD-3), the wall on atom pairs from the
undeformed Heitler–London referee over 66 orientations and the contact terms from twelve
exact geometries (FIELD-8), the lattice sum (EWALD-1). Two numbers are CHOSEN and declared
as the state point, the way a temperature is: the density `0.997 g/cm³` and the temperature
`293 K`. Three numbers enter as KILLS and from experiment, the way the stance's kills do —
the position and height of water's first oxygen–oxygen peak, the hydrogen bonds per
molecule, and the self-diffusion coefficient (Soper 2000 and Skinner et al. 2013 for the
structure; Krynicki, Green and Sawyer 1978 and Mills 1973 for `D`; the bands below are
generous on purpose because the referee this law was derived from is a minimal basis) —
and are named as such wherever they appear.*

misfits: contacts **M-EMPTY-SECTOR** (an arm whose unit count leaves the staked value on
any pass is VOID; the expectation rule of §1 has its EMPTY branch); **M-VACUOUS-SUCCESS**
(every arm reports the units on every pass, the wave-vectors summed, the seam's drop
totals, and the boundary-crossing fraction its readouts corrected for); **M-NULL-MISSTAKE**
(the readouts are frozen instruments — the rung-1 lens's hydrogen-bond criterion under the
minimum image, a radial distribution at a declared bin, the lens's own diffusion reading
with its exponent gate); **M-FIXED-POINT-TRAJECTORY** (one seed, declared; the box built by
one builder); **M-UNTESTED-GAP** (the law was fit on dimers; the liquid is the gap, and
every reading below is across it); **M-CHEAPER-THAN-ITS-PRICE** (the cost model is the
force pass at 384 atoms with the lattice sum, MEASURED on the first 100 frames and written
before the counted frames; an arm returning under a tenth of it is refused);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants on the readouts, carriers asserted nonzero
in the sector each acts on); **M-FLOOR-UNSTAKED** (the diffusion reading's floor is the
lens's exponent gate; the peak's floor is its bin); **M-BARE-CHARGE**, **M-HOMOG**
(charges on units; the density is the one uniform thing and it is the state point);
**M-COND-PROBE**, **M-DEVICE-CLASS**, **M-STALE-INSTRUMENT**, **M-FORMAT-FLOOR**;
**M-VOLUME-SCALE** (contacted by keyword: the box's lattice is the START only, and the cell
edge is fixed by the state point's density, not by a grid count). Not contacted: the rest
of the registry.

## 0. What is built and measured

**The engine: the image rule under the seam.** `Sim::legality_radius` today is the tables'
reach (20 bohr), which refuses every liquid cell. Under the seam the tables serve only
within a unit or a free atom, so the reach the minimum image must honour is: the tables'
reach if ANY atom is free, else the larger of the intra-unit reach (`4.0` bohr — no water
unit spans more) and the SEAM terms' reach, the radius past which every cross-unit term of
the harvested law is under `1e-10` hartree (from its own coefficients). `pbc_ok` tells the
truth per pass with the same rule, so a unit dissolving mid-run turns the box illegal and
the arm VOID (§2 L0).

**The box.** 128 waters (384 atoms) on the 128 sites of a body-centred cubic lattice of
`4 × 4 × 4` cells (spacing `L/4 = 7.40` bohr, nearest sites `6.41` bohr = 3.39 Å apart — a
starting oxygen spacing just outside the first shell); each monomer at EMBED-1's pin,
oriented by three LCG-drawn Euler angles (seed `0x4c49_5155_4944`), in a cubic cell of edge
`L = (128 · 18.015 / (N_A · 0.997))^{1/3} = 29.60` bohr; momentum-free thermal velocities at
293 K, the thermostat (τ 2000), the field with the lattice sum, the seam law from
`wall8.json`. `adopt_table_timescale` sets the step.

**The readouts** (`holon-lens`, new): `hbonds_periodic(pos, z, cell)` — the rung-1
criterion under the minimum image; `rdf_oo(pos, z, cell, dr = 0.1 bohr, r_max = L/2)` — the
oxygen–oxygen radial distribution normalised to the ideal gas at the box density; the
lens's existing `diffusion(traj, max_lag)` on the oxygens' unwrapped trajectory, with its
exponent gate (`0.85 ≤ α ≤ 1.15`, else the reading is refused by the lens itself).

**The arm.** 2,000 settling frames, then `100,000` counted frames, one integrator step per
frame; the readouts every 100 frames; the units and `pbc_ok` every frame.

## 1. The expectation, written before the arm (M-EMPTY-SECTOR discharged)

The seam law's binding per molecule at the start (the box's cross-unit energy per water,
`E(box) − 128·E(monomer)` over 128) is written first with its parts; under `−2 kT` the
liquid is expected to hold together; between `−2 kT` and `−kT` no expectation; above `−kT`
expected to evaporate into the cell. Units `< 128` at the start ⇒ VOID.

## 2. Gates

- **L0 — the box is legal and stays so.** `set_boundary(Periodic)` admits the cell under
  the seam-aware rule (the seam reach and the half-edge both printed); on EVERY pass of the
  arm `pbc_ok` holds and the unit count is `128` (EXACT), else the arm is VOID at that frame
  and the record says which.
  witness: none (a per-pass reading)
- **L1 — the books close.** Over the arm the receipt columns sum to `w_ext`; the honest
  drift peak is under a tenth of the largest posted transition; the momentum residual is
  under its bound; the lattice sum's wave-vectors and real pairs are reported per pass.
  witness: none (conservation gates)
- **L2 — the price.** The force pass at 384 atoms with the lattice sum, measured on the
  first 100 frames and written before the counted frames; the arm's total within `0.1×` to
  `10×` of `102,000` passes at that price.
  witness: none (a price, recorded)
- **R1 — the first oxygen–oxygen peak.** From the counted frames' mean `g_OO(r)`: the
  position of the first maximum and its height. KILL bands, from experiment and declared:
  position in `[2.65, 2.95]` Å (`[5.0, 5.6]` bohr), height in `[2.0, 4.0]`. **(a)** both in
  band; **(b)** position in band, height out; **(c)** position out.
  witness: none (a histogram against a declared band)
- **R2 — hydrogen bonds per molecule.** The mean over counted frames of the periodic
  hydrogen-bond count divided by 128. KILL band `[3.0, 4.0]`. **(a)** in band; **(b)** under;
  **(c)** over (a fused or over-structured liquid).
  witness: none (the frozen lens under the minimum image)
- **R3 — self-diffusion.** The lens's `diffusion` on the oxygens over the counted frames,
  converted to cm²/s by the engine's own time unit. If the lens refuses (exponent outside
  its gate) R3 is VOID and says so. KILL band: within a factor of `3` of `2.3e-5` cm²/s.
  **(a)** in band; **(b)** too fast (a fixed-charge law's usual failure); **(c)** too slow or
  frozen.
  witness: none (the lens's reading against a declared band)

## 3. What each outcome means

R1 (a) with R2 (a) is a liquid with water's structure at this level, derived and not
fitted; R3 says how mobile it is. R2 (c) with a drift excursion is the fused shape
FIELD-7 taught; L0 VOID says the closure identity does not survive contention at liquid
density, which is the holon's own claim under test and would be the first finding. Any
(b)/(c) names the next channel by which reading missed.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

A law fit on dimers, asked about a liquid; one seed, one box, one temperature, once.

## 5. Plants

- **(i) The radial distribution without the minimum image.** `rdf_oo` on raw differences:
  the first peak's height must change by more than `20 %`. Carrier: the fraction of O–O
  pairs inside the first shell (`r < 3.2` Å) whose minimum-image vector crosses a face,
  measured on the counted frames, `≥ 5 %`, asserted nonzero in the sector the plant acts on.
- **(ii) The hydrogen-bond count without the minimum image.** `hbonds` (the open-box lens)
  on the wrapped positions: the count per molecule must fall by more than `20 %`. Same
  carrier.

## 6. Discipline

Engine: the seam-aware `legality_radius` and `pbc_ok`, with a gate in `tests/seam.rs` (a
liquid cell admitted under the seam and refused without it, EXACT). Lens: the two readouts
with unit tests (a lattice with a known `g(r)`; a bond across a face). Runner
`holon-render/examples/liquid1.rs`; JSON under `conformance/water_observatory/liquid1/`;
results `LIQUID1_RESULTS.md` with the instruments. The three experimental kill bands are
the only numbers from outside the engine and are named as kills wherever they appear.
