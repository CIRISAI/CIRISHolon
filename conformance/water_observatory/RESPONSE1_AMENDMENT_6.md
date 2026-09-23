# RESPONSE-1 — AMENDMENT 6: the fine-model seed, what it is compared on, and what kills the closure as the model's

*Written 2026-09-23, after the ten rigid arms closed (`RESPONSE1_RESULTS.md`, verdict table) and
BEFORE the fine arm is launched or read. The prereg (§1, §5 (a)) and Amendment 5's verdict owe
"one fine-model seed at the same kick" before any closure number is called the model's rather
than the operator's. This names the run, the comparison, and the kill. Nothing already read moves.*

## The run

One arm: **longitudinal, seed 0 (`0x…4530`), `v_d = 50 m/s`, twelve cycles of 314 readouts,
the fine (flexible, all-atom) model**, on the same 432-water box (`n = 6`, `L = 44.39` bohr).
`replace0 scout … --kick L --kick-mps 50 --kick-cycles 12 --kick-relax 314 --kick-arm flexible`,
eight workers, launcher `replace0_response1_fine.sh`, output `replace0/response1_fine_L_seed0/`,
binary `replace0/replace0_fine` (the closed campaign's `engine/target/release/examples/replace0`
is not overwritten; its provenance stands).

**What is the same as rigid seed 0, bit for bit:** the built box, the 3,000-frame fine settle, the
reference geometry, the stiffness envelope read from `transport_seed0/stiffness.json`, the
projection and the rigid settle by criterion. `--kick-arm` defaults to `rigid`; with it the code
path is textually unchanged and a short kicked scout on 128 waters wrote `rigid.walk` and
`rigid.vwalk` byte-identical to the campaign binary's.

**What is the fine model's, declared here:**

1. **Hand-off.** The settled rigid state is written back to the atoms. That state has empty
   vibrations, so each water's three internal degrees of freedom are then **seeded thermally**:
   every atom gets a Gaussian velocity at `2 × 293 K`, the operator's own projection of that draw
   (its rigid part) is subtracted, and the mass-orthogonal remainder is added. The rigid modes'
   temperature does not change. The factor 2 is there because the sites sit at the reference
   geometry with no vibrational potential energy, so the `≈ 3 kT` per water of internal kinetic
   energy splits into `≈ 3/2 kT` kinetic plus the potential share within a period. The draw is
   deterministic in the seed. Then a FINE settle by the rigid settle's own rule in the same physical time: velocity rescaling every
   `10 × dt_r` to `293 K` on the **six retained modes** of the projected units, blocks of
   `500 × dt_r`, the last five blocks' trend under their scatter, floor `3,000 × dt_r = 1.1 ps`,
   cap ten times it.
2. **The target temperature is on the rigid modes, not on 3N, and the vibrations are seeded
   rather than waited for.** The fine model does not equipartition into its stretches on these
   times. A 128-water smoke settled for 1,430 frames under the 3N stochastic thermostat read
   `294 K` on 3N and **`491 K` on the rigid modes**, with `0.05 kT` per water in the vibrations. A
   3N target would have compared a liquid at about 400 K against the operator's 293 K. Holding
   the rigid modes at 293 K without seeding left the 3N temperature at `196–201 K` after 1,000
   frames on 128 and on 432 waters: the vibrations do not fill from the rigid modes on the
   settle's times. One factor scales every unit's centre-of-mass and rotational
   kinetic energy by the same `s²` (the projection is linear in the velocities), so the rigid-mode
   temperature lands on the target exactly; the vibrations take the same factor and are otherwise
   left alone. The 3N temperature is recorded next to it at every readout.
3. **The kick.** For each water, the mass-weighted centre of mass `x_com` is read as the
   projection reads it (oxygen in the box, hydrogens at minimum image). The increment
   `Δv = v_d (sin(k x_com) − ⟨sin⟩)` is added to **all three atoms' velocities equally**. That
   changes the unit's momentum by `M Δv` and leaves every internal velocity (vibration and
   rotation about the centre of mass) untouched. It is the rigid arm's slab pattern and its mean
   subtraction, applied to the same centres. Between cycles the rescale in (2) is the rigid arm's
   `rescale_rigid`, on the same modes.
4. **The cadence.** The fine step is the tables' hold (`1.0775` au) and is not changed. A readout is
   every **384 fine frames = `10.0082` fs**, not `10` fs, so a cycle of 314 readouts is
   `3.1426` ps. The record carries `readout_fs`, and `walk2traj.py` takes the spacing as an argument
   (`10.0082`). The reader and its `--cycles 12 --relax 314` do not change.
5. **The walk.** `flexible.walk` and `flexible.vwalk` hold the OXYGEN atoms' unwrapped positions
   and velocities, in `rigid.walk`'s format and header. On the fine model the oxygen velocity
   carries the oxygen's share of the intramolecular vibration. The rigid site velocity carries
   none, so the momentum channel of the fine walk has a little more fast noise. That is declared,
   not corrected.

A 128-water smoke of the whole path, with the settles cut short and `2 × 3` readouts, wrote
`flexible.walk` and `flexible.vwalk` with a `# walk 7 L` header and 384 columns (128 oxygens ×
3), which is `rigid.walk`'s format. Its NVE gate held at a peak excursion of `2e-9` Ha per water.
With the seeding, the smoke added `3.15 kT` per water of internal kinetic energy (`3.15` read
back by projection). The rigid modes read `293.000 K` before and after the seeding. The
vibrations then oscillated between `0.7` and `2.2 kT` per water over the six readouts: 128 units
seeded at zero displacement start in phase, and the real settle (at least `1.1` ps) is what
dephases them. The smoke's rigid modes heated in NVE (`294 → 387 K`) because its box was built fresh and
had only 100 rigid steps of settle. The rigid-arm smoke heated the same way. That is not a
reading.

**PR-4 on the fine arm** (`plant-kick --kick-arm flexible`, same box, every atom at rest):
`|Δp_total|` `2.2e-14` au, `ΔKE` against `½ M v_d² Σ (sin − ⟨sin⟩)²` to `3.5e-16` relative, no atom
moved, zero intra-unit velocity difference, and `x_com` equal to the rigid body's centre to
`7e-15` bohr. PASS at 50 and 200 m/s, both axes. The rigid PR-4 on the same binary reprints the
committed `response1_pr4_50.log` line for line.

## What is compared, against what

Each read uses the reader and the grid that produced the rigid number. The comparator is **rigid
seed 0** (same seed, same kick). The band is the **three rigid 50 m/s longitudinal seeds' range**:

| read | rigid seed 0 | rigid three-seed range (L, 50 m/s) |
|---|---|---|
| **R1**: `D_cont` at `8×1×1`, integral form, cycle-aligned (Amendment 2) | `0.650` | `[0.650, 0.846]` |
| **R1″**: face chart `h = 0.25 Å`, 8 cells (Amendment 5) | `0.482` | `[0.482, 0.492]` |
| **R2**: density mode classification, and `λ₁` from the peak | OVERDAMPED, `4.49 × 10¹²` /s | OVERDAMPED on all three, `[4.49, 5.53] × 10¹²` /s |
| longitudinal current `λ₂` (`ν_l k²`) | `8.25 × 10¹²` /s | `[8.25, 12.27] × 10¹²` /s |

**R3's `η` cannot be compared on this arm.** R3 reads the TRANSVERSE current mode on arm T, and
this is arm L. The brief that asked for "R3's `η`" on one longitudinal fine seed asked for
something this seed cannot give. `η`'s fine-model check needs a fine arm T (rigid three-seed
`η = [3.9, 5.0] × 10⁻⁴` Pa s, seed 0 `5.0`) and is **owed, not run**. The prereg's "one
fine-model seed" is one seed per axis if R3 is to be the model's.

## The kill

> **A fine-model number outside the rigid three-seed range by more than its own standard error**
> (the reader's leave-one-out SE over twelve cycles, or the floor it prints for `D`) means that
> closure number is the OPERATOR's and not the model's. Per Amendment 5, the verdict row it
> belongs to is then re-labelled "operator" in `RESPONSE1_RESULTS.md`. A classification that
> differs (R2 oscillatory where the rigid arms read overdamped) is the same kill. Inside the
> range, or outside it by less than its own SE, the row is the model's **on this seed**. One
> seed cannot narrow the band, and nothing here is pooled with the rigid seeds.

The fine arm's own gates are NVE (peak energy excursion per water within a cycle under `0.1 kT`)
and SETTLED (rigid-mode production temperature within 10 % of 293 K). If either fails, the arm
is VOID, not a kill.

## Cost, corrected before the run

The prereg §6 priced this seed at "`~150` core-hours serial", and the brief for this amendment
carried "`2,171` core-s/ps on eight workers" as the fine model's price. **Both are prices from
the wrong regime.** `2,171` was the OPERATOR's wall-seconds per ps (the pilot's `3,609`
single-core over a measured `1.66×`), already corrected in §6. `150` core-hours is `14,300` s/ps, which is the fine model's measured serial price
on **128** waters (`14,097` s/ps, `transport_seed0/run.json`), carried to 432 waters unscaled.
Measured for this amendment: on 432 waters the fine model runs **`1.35` wall-s per frame** with four
workers on four E-cores (cores 21–24, `taskset`, a loaded machine). One picosecond is `38,369`
frames, so that is **`51,800` wall-s/ps**, `3.6×` the prereg's figure and on a quarter of the
launch's cores. The arm is `1,446,912` production frames (`37.71` ps), plus at least `42,630` for
the fine settle (the criterion cannot fire before its floor; the cap is `426,300`), plus
`3,000` for the first fine settle, plus the rigid settle, which is rigid seed 0's `12,500` steps
by bit identity, `~3.7` h. The minimum is `1.49 M` fine frames. **Wall estimate on the launch's
eight workers on the P-cores 0–15: `14` days at `0.8` s/frame, `17` at `1.0`, `23` at `1.35`
(no speedup over the probe); three weeks is the planning figure.** The P-core, eight-worker price
has not been measured. The run prints its own wall-s/frame every 500 frames of the fine settle and
its wall-s/ps with an ETA every 25 readouts; the first of those replaces this estimate in the
results document. The core class is a confound (M-PLACEMENT-LOTTERY): the probe's E-core figure
is not the launch's.

---
witness: none (a measured comparison: its criteria are numeric ranges read by the frozen `rung2 --response` reader and `r1_closure_test.py`; no gate is a Lean theorem of its own)
**misfits:** M-CHEAPER-THAN-ITS-PRICE, M-COND-PROBE, M-PLACEMENT-LOTTERY, M-PLANT-OBS, M-PLANT-SECTOR, M-STALE-INSTRUMENT, M-RIGID-REFERENCE-PLACEMENT — the registered ids this text contacts by keyword, cited at the audit's demand. The last is contacted in substance, not by keyword: the fine arm starts from the operator's geometry, and the fine settle (floor `1.1` ps, by criterion) is the declared relaxation away from that placement before anything is read.
Carrier-sector statement (M-PLANT-SECTOR): PR-4's carrier is the arms' own 432-water box with every atom at rest, and the sector the plant acts on (the units' centre-of-mass momentum along the kick axis) is nonzero in that carrier by construction of the kick.
