# SLOW-3 — READ: branch (c) on fresh data, as written down before the read. The fixed integration WORKS (it passes every plant, where SLOW-2's failed) and carries +0.0125 beyond velocity and tetrahedral order — the same as the plain VAMPnet, under the +0.02 bar and under a linear regression on the raw shell inputs

*2026-09-27. Prereg `SLOW3_PREREG.md` (frozen alone `9493ed0`); the fix selected on the synthetic plant
and training seeds 0, 1 only (λ = 0.3), the models FROZEN by manifest (`cd88af24…`) before the
fresh seeds finished; the expected branch (c) written into the prereg's notes before the read
(`46e6cff`). Test data: 293 K seeds 3 and 4, simulated fresh on the rebuilt binary; seed 2 never
opened. Read: `slow3/slow3_read.txt` (`slow3_train.py read`, trains nothing). Provenance: the
rebuilt binary's seed-0 walk is byte-identical to SLOW-1's on every data row (the header's row
count differs by construction, 21 against 2,501); `slow3/provenance.txt`.*

| stake | staked | read on the fresh seeds | verdict |
|---|---|---|---|
| **G1** F-fixed passes every plant | PS-1..4, PS-3 at 3/3 training seeds | all pass (momentum σ₁ ≤ 0.007; time-shuffle ≤ 0.09; PS-3 as selected) | **MET** — the integration works where SLOW-2's did not |
| **G2a** carries ≥ +0.02 beyond velocity and q | mean of seeds 3, 4 | **+0.0125** (min over seed × torch seed +0.0111) | **NOT MET — the kill** |
| **G2b** ≥ the plain VAMPnet | | +0.0125 against D's +0.0118 | met, tied within the spread |
| **G3** ≥ SPIB-fixed | | SPIB-fixed fails PS-3 again (collapse); not read | by the letter ≥ |
| **G4** ≥ R0 − 0.005 (the raw 84 shell columns, linear) | | R0 = +0.0182 → bar +0.0132; F +0.0125 | **NOT MET** |

**Branch (c): the carried structure beyond tetrahedral order is below +0.02 on this operator at
these lags, on fresh data.** The fix did what it was for — the integrated net now finds the
planted variable and passes every control — and it adds nothing a plain VAMPnet does not, while
a plain linear regression on the raw neighbour-shell geometry carries more than either (+0.018,
+0.021 and +0.015 per fresh seed). The 400 K control is at zero throughout. SLOW-2's reading
stands, now on data nobody had seen: tetrahedral order is essentially all the structure this
liquid's mobility carries at 1–5 ps, and what little is left is not a compact slow variable.

**Deviations, recorded.** The read script's paths broke in the 2026-09-26 path rewrite (fixed,
`51565c3`); the read was run in its last step by hand on the same frozen models and the same
caches (the training seeds' caches copied from the session scratchpad beside the fresh ones);
the provenance script compares the header line and so printed "DIFFER" on a byte-identical walk.
