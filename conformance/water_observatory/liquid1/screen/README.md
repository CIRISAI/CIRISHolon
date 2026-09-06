# LIQUID-1 diagnostic screen — NOT a reading

Eight short arms of the freeze's box (2,000 settling + 10,000 counted frames, a tenth of the
campaign's) with one or two knobs turned, run in parallel while the counted arm runs. Every
file here carries `dry: true` and a `screen.json` naming the knobs; nothing here enters any
gate, and the only use of these numbers is to choose which hypotheses become PRE-COMMITTED
branches of LIQUID-2's freeze (`prereg decision trees`). The knobs are DECLARED, not derived:

| label | knob | the hypothesis it screens |
|---|---|---|
| base | none | the campaign's own arm at a tenth of its length (a control for the length) |
| T250, T200 | temperature 250 K, 200 K | the energy-scale confound: this basis's bond is 0.7 of real water's, so its 293 K is real water's ~420 K in reduced units; if bonds rise toward 3 at 200 K the deficit is the scale, not a missing channel |
| Q125 | charges × 1.25 | accommodation at the liquid: the empirical dipole enhancement every fixed-charge water model needs (SPC/E: 2.35 D against 1.85 D) as a proxy for cooperative induction |
| C6_45 | C₆ = 45 Ha·bohr⁶ on O–O | attunement: the literature-scale isotropic dispersion this basis cannot produce |
| Q125_C6, T200_C6 | both | the two together; the scale with dispersion |
| PAIRCT | m_ct = k_ct = 0 | the transfer term's angle removed (CT-1's shape with CT-2's coefficients) |

Read: R2 (bonds per molecule) and R1 (the oxygen peak) per variant against the base.
