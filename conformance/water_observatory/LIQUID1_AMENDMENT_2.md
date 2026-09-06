# LIQUID-1 — AMENDMENT 2: the seam terms truncated by a declared C² switch, and the door reads the switch

*Frozen 2026-09-06, committed alone, BEFORE any counted arm: at the time of this commit the
only run of `liquid1` without `--dry` is the one this amendment answers, and it wrote
`arm.void` at the DOOR and stepped no frame. Written because the first seam law to pass its
boundedness gate and hold the dimer and the ring (CT-2, `wall_ct2.json`: S3 (a), G-B0 `None`)
was refused by L0's door on the 128-water cell: the law's terms reach `18.578` bohr at the
freeze's `1e-10`-hartree budget against a half-edge of `14.797`. The two terms past the edge
are the H–H contact (`P_HH = 0.017` Ha at `c_HH = 1.02` per bohr, reach 18.58) and the
charge-transfer term (`P_CT = 1.37` at `c_CT = 1.40`, reach 16.67); every wall and the H–O
contact reach 10.7–13.4 bohr. At the half-edge those two terms are worth `4.7e-9` and
`1.4e-9` hartree per pair — real, and small against `kT = 9.3e-4`. The door was right by its
letter: past the half-edge a pair has two images inside the reach and the minimum image is
not unique. The freeze's remedy set — widen the box, shorten the reach, stay unwrapped — is
taken at its middle item, as every periodic force field takes it: the seam terms are cut off
by a smooth switch at a DECLARED radius inside the half-edge, the cut-off tail is PRICED and
reported, and the door reads the switch radius as the seam's reach. Widening the box to the
next body-centred lattice that admits the unswitched law (`6 × 6 × 6`, 432 waters, half-edge
22.2 bohr) would cost about eleven times the force pass to keep a `1e-8`-hartree tail, and is
declined for that reason, here, before the numbers.*

misfits: contacts **M-STALE-INSTRUMENT** (the refusal is the instrument reading its own
rule; the amendment changes the law's far edge, not the rule); **M-CHEAPER-THAN-ITS-PRICE**
(the truncation's price is a number written to the record before the counted frames, not an
assumption: the summed tail over every cross-unit pair of the start box); **M-VACUOUS-SUCCESS**
(the door under the switch prints the switch radius, the unswitched reach, and the half-edge
side by side; a door that admits because a number was made smaller must show which number);
**M-EXTRAPOLATED-HOLE** (contacted by keyword: the switch acts at `12–14` bohr, far outside
every class's fit range and every well; `SeamModel::bounded` and `hole` are unchanged because
they walk `r ≤ r_min ≤ 4.73` bohr where the switch is identically 1); **M-FIRST-VIOLATION-ONLY**
(the door names both the switched and the unswitched reach); **M-PLANT-OBS** and
**M-PLANT-SECTOR** (the two plants of the freeze are unchanged, and so is their shared
carrier — the fraction of first-shell O–O pairs whose minimum-image vector crosses a face,
`≥ 5 %` on the counted frames — asserted nonzero in the sector each plant acts on, the wrapped
positions, before either plant is read); **M-COND-PROBE** (contacted by keyword only: the
switch is a fixed function of the pair distance, not a probe conditioned on the state);
**M-VOLUME-SCALE** (contacted by keyword: the cutoff is a radius in bohr chosen from the box's
half-edge, not a grid count; `L` is still the density's). Not contacted: the rest of the
registry.

## The change

1. **The switch.** `SeamModel` gains `r_cut` (bohr; `0.0` means no switch — every record and
   arm before this amendment is that state, bit for bit). With `r_cut > 0` every cross-unit
   term of the seam law — the three walls, the two contacts, the dispersion, the transfer
   term in either of its forms — is multiplied by
   ```
   S(r) = 1                                   for r ≤ r_on = r_cut − 2
   S(r) = 1 − 10x³ + 15x⁴ − 6x⁵,  x = (r − r_on)/2   for r_on < r < r_cut
   S(r) = 0                                   for r ≥ r_cut
   ```
   the quintic C² step (`S`, `S'`, `S''` continuous at both ends), and the posted force
   carries `S'·U + S·U'`. Beyond `r_cut` a cross-unit pair is not evaluated. The switch is the
   ordinary truncation of a periodic force field (Allen and Tildesley, Computer Simulation of
   Liquids, §5.2), credited, and it is applied to the SEAM terms only: the tables inside a
   unit are untouched, the field is the lattice sum and needs no cutoff.
2. **The door.** `SeamModel::reach(budget)` returns `min(reach, r_cut)` when `r_cut > 0`,
   because the term IS zero past `r_cut`; `Sim::legality_radius` under the seam therefore
   reads `max(4.0, min(reach, r_cut))`. The door prints all three: the unswitched reach, the
   switch radius, the half-edge. LIQUID-1's `r_cut = 14.0` bohr, `r_on = 12.0`, chosen from the
   half-edge `14.797` with `0.8` bohr to spare; the declared constant lives in the runner and
   is written to `door.json`.
3. **The price of the truncation, written before the counted frames.** On the start box the
   runner sums, over every cross-unit pair with `r > r_on`, the unswitched magnitude
   `|U_class(r)|·(1 − S(r))` — the energy the switch removes — and writes it as
   `truncation_tail_hartree` and per water. From the law's own numbers: the largest tail
   term is `4.7e-9` hartree per pair at the half-edge, a water has of order a hundred cross
   pairs between 12 and 14.8 bohr, so the tail is of order `1e-6` hartree per water; the
   STAKE is `≤ 1e-5` hartree per water (a hundredth of `kT`), and an arm whose tail exceeds it
   is VOID at the door with the number printed.
4. **The derivative gate covers the switch.** `tests/seam.rs`'s G-B3 declared model carries an
   `r_cut` that puts the switch inside the dimer scene's cross-unit distances, so the central
   difference on every atom checks `S'·U + S·U'` as it checks every other term. The L0 gate
   asserts `legality_radius = max(4.0, min(reach, r_cut))` to the bit under a declared cutoff.
   witness: none (a finite difference on six atoms; an equality to the bit)

Gates, kill bands, the box, the frame counts, the readouts: unchanged. The counted arm runs on
`wall_ct2.json` with the switch, and its results file names this amendment.
