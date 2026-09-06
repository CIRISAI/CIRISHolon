# LIQUID-1 — results: the first liquid of derived constants, read

*Freeze `LIQUID1_PREREG.md` (1fbbd59, alone, conditioned); `LIQUID1_AMENDMENT_1.md` (19690b8)
and `LIQUID1_AMENDMENT_2.md` (4237ab9), each alone, before any counted frame. Condition met by
CT-2 (`wall_ct2.json`: G-B0 `None`, S3 (a)). Engine: the seam-aware image rule with the C²
switch at 14 bohr (the lead); the three-body force direction under the minimum image, found
by this campaign's box and fixed before the counted arm (correction 4). Instruments: the
periodic lens (`hbonds_periodic`, `rdf_oo`), the 128-water bcc box, the runner `liquid1.rs`
(delegate). The counted arm: 2,000 settling and 100,000 counted frames at 0.997 g/cm³, 293 K
target (mean 298.4 K), one seed, on one core at 0.232 s per force pass, 29,948 s of wall
inside the price band. Records under `liquid1/` (`arm.json`, `rdf.json`, `door.json`,
`expectation.json`, `price.json`, `arm.log`); the diagnostic screen under `liquid1/screen/`,
every file of which says `dry: true` and none of which is a reading.*

## The verdict, first

**A liquid, water-shaped in its shell and under-connected in its bonds: the closure identity
held for 100,000 frames at liquid density, the oxygen shell sits at 3.04 Å with 4.2
neighbours, and the hydrogen-bond count is two thirds of water's. By the freeze's letter R1
reads (c) on position, R2 reads (b), and R3 is VOID because the arm is too short to contain a
diffusive regime.**

1. **The molecule survives the liquid (L0).** 128 units on every one of 102,000 passes, no
   free atom, the box legal under the switched law at every frame; momentum residual `4.1e-11`
   against a bound of `3.0e-7`; the receipt columns sum to `w_ext`. The object contract's claim
   that a water molecule is a closed view was made on one molecule in vacuum; it now holds for
   128 in contact at 293 K.
2. **The oxygen structure is water-shaped and 3 % too far (R1 (c) by letter).** The first O–O
   peak at `5.75` bohr = `3.04` Å with height `2.20`: the height inside experiment's band
   `[2.0, 4.0]`, the position `0.09` Å past the band's edge `2.95` Å (experiment 2.8). The
   coordination number to 3.3 Å is `4.20` against water's ~4.5. The second shell is at `6.0` Å
   (`g = 1.14`), where a close-packed liquid puts it, not at 4.5 Å where a tetrahedral network
   puts it: this is a simple liquid of weakly directional molecules, not yet a network.
3. **Two thirds of the bonds (R2 (b)).** `1.184` bonds per molecule on the lens, which counts
   each bond once (a ceiling of 2). Experiment's `3.5`, and the freeze's band `[3.0, 4.0]`,
   count each bond at both ends; in that convention the arm reads `2.37`, 68 % of water. The
   band's letter is kept as written and the verdict is (b); the convention mismatch is a
   correction to the freeze (§4) and LIQUID-2 states its band in the lens's own convention.
   Above the four-connected percolation threshold (~0.8 distinct bonds per molecule), so the
   bond graph most likely spans; the spanning-cluster fraction was not a readout of this
   freeze and is owed (GANTT2).
4. **No diffusive regime in 2.6 ps (R3 VOID).** The lens refused its own reading: over the lag
   window the mean squared displacement goes as `τ^1.64`, not `τ^1`. The arm's 100,000 frames
   at the tables' step (`1.08` au) are 2.6 ps, and water's ballistic-to-diffusive crossover is
   of that order; the refusal is the instrument reading the window, not the liquid. A
   diffusion coefficient needs an arm ten times longer or a longer sampling stride, and
   LIQUID-2 prices it.
5. **The books closed, and one letter fell short (L1).** The drift peak `1.076e-5` against the
   runner's bar `1.0e-5`: 8 % over, on a bar the runner set as its fallback (the freeze's
   rule is a tenth of the largest posted transition, and the two transitions here posted no
   work, so the fallback held). Columns exact, momentum exact. Reported as the letter reads:
   the drift leg FAILS by 8 %; the books are closed to one part in 10⁵ over 102,000 steps.

| gate | verdict | the number |
|---|---|---|
| L0 — the box is legal and stays so | **PASS** | door admitted (switched reach `14.00` vs half-edge `14.80`; unswitched `18.58`; truncation tail `9.9e-7` Ha per water against the `1e-5` stake); 128 units and `pbc_ok` on every pass |
| L1 — the books close | **columns PASS, momentum PASS, drift leg FAIL by letter** | `w_ext = −0.767` Ha, all in the thermostat column; drift peak `1.076e-5` vs `1e-5`; momentum `4.1e-11` vs `3.0e-7`; 15,624 wave-vectors, 37,950 real pairs, seam pairs served 3,537 O–O and 14,268 H–O per pass |
| L2 — the price | **PASS** | `0.2320` s per pass; `29,948` s against `23,660` expected, inside `0.1×`–`10×` (the machine suspended twice; the process's clock excludes it) |
| §1 — the expectation | **"break" written; the box HELD** | cross-unit energy `+4.5e-3` Ha per water at the random start (the walls first); the settled liquid held its units and its shell, so the expectation rule's "evaporate" branch was wrong for this law — recorded, not repaired |
| R1 — the first oxygen peak | **BRANCH (c)** | position `5.75` bohr (`3.04` Å) against `[5.0, 5.6]`; height `2.203` against `[2.0, 4.0]`; `n(3.3 Å) = 4.20`, `n(3.5 Å) = 5.84`; second shell at 6.0 Å |
| R2 — hydrogen bonds per molecule | **BRANCH (b)** | `1.1843` on the lens (each bond once) against `[3.0, 4.0]`; `2.37` in the band's both-ends convention against 3.5 |
| R3 — self-diffusion | **VOID** | the lens's exponent gate: MSD `∝ τ^1.64` over 250 lags of 100 frames; no diffusive regime in 2.6 ps |
| carrier | **nonzero** | 26.0 % of first-shell O–O pairs cross a face (floor 5 %) |
| plant (i) — the radial distribution without the minimum image | **FIRES** | peak height `1.63` against `2.20`, moved 26 % (needs > 20 %) |
| plant (ii) — the bond count without the minimum image | **FIRES** | `0.865` against `1.184`, fell 27 % (needs > 20 %) |

## 1. What the liquid is

Every constant came from two molecules. Put 128 of them in a box at water's density and
temperature and they hold their identities, take a shell at 3.0 Å with four neighbours, and
bond to each other at two thirds of water's count, in a packing that is a simple liquid's
rather than a tetrahedral network's. The deficit is orientation: the neighbours are there and
pointed too loosely. The diagnostic screen (§3) says which knobs move it and which do not.

## 2. Radial distribution, for the record

| r (Å) | 2.6 | 2.8 | 3.0 | 3.1 | 3.2 | 3.4 | 3.6 | 4.0 | 4.5 | 5.0 | 6.0 | 7.0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| g_OO | 0.14 | 1.22 | 2.19 | 2.15 | 2.02 | 1.53 | 1.22 | 0.93 | 0.75 | 0.82 | 1.14 | 0.96 |

(1,000 readout frames; bin 0.1 bohr; `rdf.json`.)

## 3. The screen, labelled (not a reading)

Eight short arms with one knob turned each, then four stacks, a tenth of the campaign's
length, in `liquid1/screen/` with a README. Bonds per molecule at frame 10,000 against the
base's `0.882`: cooling to 250 K `0.90`, to 200 K `0.92`; charges × 1.25 `0.98`; a literature
dispersion `0.92`; both `1.00`; the transfer term's angle removed `0.86`; the bond criterion
widened to 45° `1.20`; everything against bonding `0.45`. Two stacks with the transfer term
pointed at the acceptor's plane normal (the family's own corner) and everything for bonding
DISSOLVED a unit within 1,500–2,400 frames — a proton transfer the referee cannot hold (OH⁻
unbound at this basis) — and the pointing-only screens at the family's amplitude heated the
box and dissolved a unit too. Reading, for LIQUID-2's branches only: direction is the lever,
the neighbours are already there, and the amplitude must be refit with the direction in it
(CT-3's table term); the energy scale is not the story; the criterion is a minority of the
deficit.

## 4. Corrections to the freeze's letter

- **R2's band is in the both-ends convention; the lens counts each bond once.** Experiment's
  3.5 per molecule counts a bond for its donor and its acceptor; `hbonds_periodic` returns one
  entry per donor hydrogen, ceiling 2. The verdict is given by the letter; LIQUID-2's band is
  `[1.5, 2.0]` on the lens's count with the conversion stated.
- **R3's window.** The freeze staked a diffusion coefficient on 100,000 frames at the tables'
  step without pricing the crossover; the lens refused, correctly. LIQUID-2 prices the arm
  length or the sampling stride from the measured MSD exponent.
- **L1's drift bar.** The rule (a tenth of the largest posted transition) had no transition to
  bind on; the runner's fallback `1e-5` was the bar, and the arm missed it by 8 %. LIQUID-2
  states the bar as a fraction of the thermostat's posted work.

## 5. Bookkeeping, declared

- The two door refusals before this arm (the unswitched law's 18.6-bohr reach; the periodic
  three-body force bug) are in `LIQUID1_AMENDMENT_2.md` and `FIELD_RESULTS.md` correction 4.
- The machine suspended twice during the arm; the process's monotonic clock excludes the
  sleeps and the price check is on it.
- No number enters from outside the engine except the three experimental kills, named as
  kills where they appear.
