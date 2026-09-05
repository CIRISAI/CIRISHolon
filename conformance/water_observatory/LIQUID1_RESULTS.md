# LIQUID-1 — results: NOT RUN; the instruments built, gated, and dry-run

*Freeze `LIQUID1_PREREG.md` (1fbbd59, alone, conditioned); `LIQUID1_AMENDMENT_1.md` (19690b8,
alone, before any counted arm). The campaign runs only on the first seam law whose
boundedness gate admits it and whose retention reads (a) on the dimer and the ring; as of
this file NO SUCH LAW EXISTS — FIELD-9's law is refused by G-B0 (an H–O well 4.3 kT below its
fit floor) and CT-1's by G-B0 in two classes (§3 of `CT1_RESULTS.md`). So this file reports
what was built and what the instruments read on themselves. It reads nothing about water,
and every dry-run record under `liquid1/` says so in its own fields
(`law_refused_for_counted_arm: true`, `is_a_reading: false`).*

## What was built

| instrument | where | gate |
|---|---|---|
| the image rule under the seam: `SeamModel::reach(budget)`, `Sim::units_reading()`, `Sim::legality_radius` taking `max(INTRA_UNIT_REACH, model.reach(1e-10))` when every atom is inside a unit, one free atom restoring the tables' reach (the lead) | `holon-render/src/seam.rs`, `src/sim.rs` | `tests/seam.rs::l0_…` — a 28.8-bohr cell refused by the tables (20 bohr) and admitted under FIELD-9's law (seam reach 13.40 bohr, the soft H–H wall reaching farthest), the radius equal to the rule to the bit, a freed hydrogen restoring the refusal; seam suite 8/8 |
| the periodic lens: `hbonds_periodic` (the rung-1 criterion under the minimum image), `rdf_oo` (the O–O radial distribution normalised to the ideal gas at the box density) (delegate) | `holon-lens/src/lens.rs` (+326 lines, nothing removed) | four unit tests: a simple-cubic first shell integrating to 6 neighbours with an exact zero below the spacing; a hydrogen bond across a face seen only under the minimum image; the two lenses agreeing when nothing crosses a face; the periodic lens refusing exactly where the open one does — holon-lens 107/107 |
| the box: `waterbox::liquid_box` — 128 waters on the `4×4×4` bcc sites, `L = (N·M/(N_A·ρ))^{1/3} = 29.5936` bohr at 0.997 g/cm³, monomers at EMBED-1's pin oriented by the MMIX LCG from the freeze's seed with `cos β` uniform (Amendment 1) (delegate) | `holon-render/src/waterbox.rs` (new module) | `tests/liquid.rs`: 384 atoms, 128 oxygens, every nearest-site distance `L·√3/8` to `1e-9` under the minimum image, the edge, every monomer's internal distances the pin's |
| the door on the liquid box | — | `tests/liquid.rs`: without the seam `set_boundary(Periodic)` refuses (20 > 14.80); with the field and a DECLARED law 128 units, no free atom, `legality_radius` equal to the rule to the bit, the door admits |
| the readout chain, end to end without an integrator | — | `tests/liquid.rs`: on displaced copies of the box, first peak 6.25 bohr at height 2.557 with `g → 1.044` in the tail; plant (i) (no minimum image) moves the peak height by 0.405; plant (ii) finds 38 face-crossing hydrogen bonds, every one invisible to the open-box lens; carrier 0.264 against the 0.05 floor; the diffusion lens recovers a planted walk to 2.3 % through the one conversion out of atomic units |
| the runner `liquid1 [--dry] [out]`: door, expectation before any frame, price on the first 100 frames before the counted ones, per-pass L0 (units 128 EXACT, `pbc_ok`), L1, L2, R1–R3 with the freeze's kill bands named as kills, the two plants and their carrier (delegate) | `holon-render/examples/liquid1.rs` | the dry run below; every file it writes validates as JSON |

## The dry run, on the REFUSED law (an instrument check; not a reading)

Run on FIELD-9's `wall9.json`, which the loader refuses for a counted arm by the boundedness
walk's own words and which `--dry` carries past the refusal with the refusal written into
every record.

| item | value |
|---|---|
| door | ADMITTED: seam reach 13.3988 bohr, tables 20.0, half-edge 14.7968 (edge 29.5936); 128 units, 0 free atoms; `legality_radius` equal to the seam rule to the bit |
| expectation (§1), written before any frame | cross-unit energy `+4.55` mHa PER WATER (field row `+12.4` mHa, seam row `+570` mHa over the box) against `kT = 0.928` mHa → the pre-registered branch is **break** (expected to evaporate): random orientations at the bcc start put the walls ahead of the bonds |
| price (L2), on the first 100 frames | `0.398` s per force pass at 384 atoms with the lattice sum (15,624 wave-vectors, 36,206 real pairs) on 4 cores; the counted arm of 102,000 passes prices at ~11 hours on 4 cores, to be re-measured on the cores it runs on |
| L0, per pass | **VOID at settling frame 82**: a unit dissolved, the tables' reach (20 bohr) returned, `pbc_ok` went false; `arm.void` names the frame and the reason. No counted frame ran; R1–R3 saw no data |

What the void says, labelled: under a law with an H–O well below its data, a box at liquid
density and 293 K loses a unit within 82 frames. That is the refused law behaving as its gate
predicted, and it is the L0 rule working — the closure identity is watched every pass and
the arm stops the moment it fails. It says nothing about water or about a bounded law.

## Corrections to the freeze's letter (Amendment 1)

The nearest-site formula (`L·√3/8`, not `L·√3/4`; the number was right); the generator and the
orientation measure pinned; the dry run named as an instrument check; two conversions stated.

## Declared choices in the instruments (the delegate's, each flagged, none silent)

- The law loader searches the sibling campaign directories for the newest record (the freeze
  names `wall8.json`; its conditioning clause admits a successor); every file records which
  record was read and whether the loader refused it for a counted arm.
- `rebase()` runs after the boundary switch, so the wrap itself does not land in `drift_peak`
  and L1 measures the dynamics rather than the setup.
- The first peak is the smallest `r` above 1 bohr that beats both neighbours (the freeze
  names only the bin as the floor); the first-shell radius is `3.2 Å` through the module's own
  bohr constant, `6.047124` bohr.
- In `--dry` mode L2 compares against the dry arm's own 2,200 passes and says so.
- Plant (i) is the lens itself run at a `1e6`-bohr cell so no image reduction fires, rescaled
  by density — the histogram's change and nothing else, with no second implementation to
  drift; `min_image` leaves an axis unreduced when its edge is not finite and positive, which
  is what makes that call exact.
- The declared first peak and the diffusion conversion live in `waterbox.rs`, so the arm and
  its gate cannot disagree about either. The conversion types ONE constant, `BOHR_M`
  (`sim.rs:89`); the time unit is the lens's own (`traj.rs:52`, `AU_TIME_FS`), cross-checked
  at run time against `sim.rs:91`'s `AU_TIME_S` to `1e-9`:
  `bohr²/fs → cm²/s = (BOHR_M·100)²/1e-15 = 2.800285e-2`.
- The diffusion lens's wall-saturation gate applies on the periodic box as well: it fires if
  the unwrapped mean squared displacement passes a quarter edge squared, and the runner
  records the refusal verbatim and marks R3 VOID. A counted arm must choose `max_lag` so that
  a liquid's MSD stays under that bound, or the reading refuses itself.
- The generator and the orientation, as written: `x ← 6364136223846793005·x +
  1442695040888963407`, `(x >> 11) / 2⁵³`, seed `0x4c49_5155_4944`; `α = 2πu`, `β =
  arccos(1 − 2u)`, `γ = 2πu`, z-y-z.

## What runs the campaign

The first law that passes G-B0 in every class and holds the dimer and the ring at 293 K. CT-1
named what that law needs (`CT1_RESULTS.md` §6): the charge-transfer term with the bond's
angle in it, and the contact terms fit UNDER the boundedness gate rather than checked after
it. When it exists, the runner is `liquid1` without `--dry`, on 24 threads, and this file is
replaced by the reading.
