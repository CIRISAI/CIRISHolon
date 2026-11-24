# SEAM-NS-0 — adversarial review of the draft, 2026-11-23: REDESIGN

*An independent reader, given `SEAM_NS0_PREREG_DRAFT.md` at `578b2cc` and the records it
cites, told to break it, and held to the draft's own blinding rule (nothing about either
Navier–Stokes blow-up construction was searched or read). It wrote nothing in the repository;
this file is its report as the lead's agent recorded it, shortened. Its arithmetic is on the
record's fitted forms or on synthetic data, never on a campaign walk. The walks were not
opened.*

**Verdict: REDESIGN of rung 0. S3 and §10's variant are not freezable in any form yet.**

## 1. The three findings, in order

1. **S1's quantity does not separate the two readings it exists to separate, and its open
   plant fails on paper.** Taking the record's two fits at face value (50 m/s:
   `λ₁ = 5.5e12`, `λ₂ = 1.04e13`; 200 m/s: `Γ = 1.7–2.7e12`, `ω = 8.5–9.0e12`) as damped
   oscillators after a kick, the defect of the 200 m/s form against the 50 m/s law over two
   lead windows at the banked grain (392 fs) is `0.15–0.21` on the density mode, astride
   the `0.2` budget, and `0.25–0.36` on the current mode, astride the `0.3` kill. The draft
   never says which mode is graded, how the two are combined, or what a lead window is for
   a mode. P2 as specified reads `0.14` and `0.25`, both under the `0.3` it must exceed, so
   the draft's own §7 sends the campaign to VOID before any read. The same comparison at
   200 fs reads `0.16/0.16`; at 785 fs, `0.50/0.43`.
2. **The record's "overdamped at 50 m/s" looks like an artefact of the estimator at that
   signal-to-noise.** `fit_relaxation` (`engine/crates/holon-lens/src/field.rs`, lines
   1086–1104 as read) ends its window at the first five consecutive points under twice the
   noise; a mode crossing zero at a signal-to-noise of 6 to 9 is cut there, before the
   rebound, and labelled overdamped. A port of the rule, fed the 200 m/s form, reads
   OVERDAMPED in 65 % of trials at SNR 9.4 and 86 % at 6.4 with white noise (84 % and 97 %
   with 50 fs correlated noise), and UNDERDAMPED always at SNR ≥ 38. The 50 m/s aligned
   fits used 11, 11 and 8 points at SNR 6.4, 9.4 and 6.8. On the one like-for-like
   statistic (an exponential from the density peak) the 200 m/s seed reads `6.45e12`
   against `4.49–5.53e12` at 50 m/s: 17 to 44 % apart, not threefold. PR-2's underdamped
   plant was never fired at the 50 m/s signal-to-noise (M-PLANT-OBS). This is a finding
   about RESPONSE-1's record, not only about the draft, and it is NOT yet a correction: it
   rests on a port and on synthetic noise.
3. **Neither S3 nor the variant is a forced prediction.** S3 picks one of three criteria,
   each with an unstated factor of order one between "the number is 1" and "the defect
   exceeds 0.3"; its grid is laid around the band after choosing it. A fourth criterion is
   missing: for a sound mode the acoustic one, `k·v_d/Γ = 1`, at about 3,000 m/s with the
   50 m/s `Γ` and about 750 with the 200 m/s `Γ`. With the record's own longitudinal
   viscosity the inertial criterion reads 3,850 m/s, not 1,180.

## 2. Numbers against the record

All of the draft's §2 and §3 arithmetic reproduces. Mismatches:

- "2,171 core-s/ps" (§8): corrected twice on the record (`RESPONSE1_PREREG.md`, lines
  119–130; Amendment 6, lines 116–118: "Both are prices from the wrong regime").
- "0.460 at 16 cells and 0.429 at 4" (§1): single-seed values from Amendment 5 set beside
  the pooled `0.180`; the pooled like-for-like values are `0.423` and `0.275`.
- "h is about half a rattle amplitude" (§2): Amendment 5 says the molecules that cross a
  face lie within about half a rattle amplitude of it; it gives no amplitude.
- "the chart is banked within budget": at 200 m/s only; at 50 m/s it reads `0.315 ± 0.087`.
- Carried out of regime: `λ₂` (fitted at 50 m/s on 8 to 11 points, spent to 800 m/s); the
  `0.2` and `0.3` bars (staked for a chart's defect, spent on a mode's); `η` (transverse,
  spent on a longitudinal mode); `ν` and `c_s` (read at 23.5 Å, spent at 3 to 6 Å, where
  `VIEW_SEARCH_RESULTS.md` already has `η(k)` moving 2.7 ×); the 350 fs bond memory (from a
  sector the momentum law does not carry).

Misfits contacted and not named: M-SORTS-NOT-SEPARATES, M-IDLE-CALIBRATED-TIMEOUT,
M-VALIDATED-NOT-WIRED, M-TRUNCATION-AS-ERRORBAR, M-INEQUALITY-READ-AS-EQUALITY,
M-CHEAPER-THAN-ITS-PRICE, M-EXTRAPOLATED-HOLE, M-POPULATION-CHOICE.

## 3. Is the draft's doubt of "outside linear response" fair?

Fair, and under-argued. The record's sentence is prereg branch (f), which also owes a
20 m/s arm that was never run. Amendment 4's test was conditional on the 50 m/s arms
reading the equilibrium period, and they did not. And the record already holds a zero-drive
reading the draft does not use: `VIEW_SEARCH_RESULTS.md` §2 has the density mode's
autocorrelation crossing zero at 400–500 fs on the same box at equilibrium, which an
overdamped mode's does not. The cleaner rung 0: the equilibrium correlation function
predicts both drives with no fit, by fluctuation and dissipation.

## 4. The variant (predict where sound crosses over)

- **Real water** (pre-2026 literature): about 1,500 m/s at long wavelength; about 3,200 m/s
  at molecular wavelengths; the transition measured by inelastic X-ray scattering between
  `Q = 1` and `4 nm⁻¹` (Sette et al., PRL 77, 83, 1996); first seen by neutrons (Teixeira
  et al., PRL 54, 2681, 1985) after Rahman and Stillinger's simulation of 1974. The settled
  reading is viscoelastic, the crossover where frequency times the structural relaxation
  time is about one (Monaco et al., PRE 60, 5505, 1999; Ruocco and Sette, 2008).
- **Reach: no.** The box's longest wavelength, `k = 0.27 Å⁻¹ = 2.7 nm⁻¹`, is already in
  the middle of the crossover. The hydrodynamic side needs 8,000 to 67,000 waters at
  water's relaxation time, and more for this model.
- **Forced: no.** Free choices: which relaxation time, which point of a transition a factor
  of four wide, and the link from a chart's defect to a dispersion, which is textbook
  generalized hydrodynamics.
- **Blind: no.** The explanation is published, and there is no experiment on the model.
- **Net:** a re-finding, not the 2026-11-13 bar.

## 5. What the record owes, found here

**The sentence "the model's sound speed is about twice water's"**
(`RESPONSE1_RESULTS.md`, item 2 of its second list; repeated in `STANCE.md` and
`GANTT2.md`) compares readings at `k = 0.27 Å⁻¹` with water's long-wavelength value. At
that wavevector real water reads roughly 2,500 to 3,200 m/s. The model is not shown to be
stiffer than water. A dated CORRECTION is made beside it in the same commit as this file.

## 6. What a redesign needs

- The graded mode defined, and a fixed window in femtoseconds (at least 785), chosen so a
  planted threefold change in damping reads above the kill and a planted same law reads
  below the budget, both shown on the carrier's noise before the freeze.
- The 50 m/s law fitted by whole-form least squares on the aligned mode, not from the
  banked rates; the banked rates out of the gate.
- A fit-through plant: fit at the 50 m/s signal-to-noise, predict at four times the amplitude.
- The equilibrium walks as the zero-drive law.
- S3 dropped until rung 0 exists; §3.1 and the variant moved to a note outside any freeze.

## Not verified

The walks; the classifier port against the reader's own noise estimate; the 20 m/s arm's
absence (from a search of the tree); the exact literature values `3,310 ± 250` and
`3,200 ± 100` m/s and a relaxation time of 0.5 to 1.3 ps, which are from memory.
