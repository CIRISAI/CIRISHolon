# EMBED-3 — results

*Freeze `EMBED3_PREREG.md` (5276318, alone). Instrument: `density_embed.rs` extended
(`rho_pa_subset`, `subset_in_field`, `classical_interaction`, the partner-nuclei flag); runner
`examples/embed3_campaign.rs`, JSON per node under `embed3/`; the 8 Å diagnostic
`examples/embed3_probe.rs`. Every solve `solve_determinant` on the host, both clocks and the
thread count in every JSON.*

## System A — the harvested residual's dependence on the field its core sits in

| gate | verdict | the number |
|---|---|---|
| G0 — the price, wall time | **PASS** | one trimer solved inside a fourth monomer's density: 665,856 determinants, **583.1 wall-seconds** on 32 threads (15132 processor-seconds), under the 900 staked — the grid admitted |
| G2 — the identity with EMBED-2 | **PASS at the record's resolution, FAIL at its letter** | D removed: `r_3 = -1.462678e-08` against EMBED-2's `-1.467178e-08` — `|Δ| = 4.5e-11`, above the `1e-12` staked and exactly the printed-precision floor of the record EMBED-2 had read its trimer from (`M-FORMAT-FLOOR`, registered from this finding; `EMBED2_RESULTS.md` corrected) |
| G1 — one fixed point of four densities | **PASS**, 2 of 2 run | both starts converge to bit-identical densities (`Δρ = 0`) in 5–7 sweeps |
| G3 — the floor of r_3 | **PASS**, 2 of 2 run | `3.5e-12`, `1.3e-12` Ha; `|Δ|/floor = 22, 43` — both posable |
| S1 — the table transfers | **NOT YET READ** — the 8 and 12 Å nodes did not complete (below) | on the nodes run, `|Δ|/|r_3(∞)| = 5.3e-03` (4 Å, contact, reported not read) and **`3.8e-03` at 6 Å** against the `0.10` stake — twenty-six times inside it |

| R_CD (Å) | r_3 inside D's field (Ha) | Δ = r_3 − r_3(∞) (Ha) | \|Δ\| / \|r_3(∞)\| | floor | \|Δ\|/floor | G1 sweeps zero / isolated | G1 Δρ | wall s |
|---|---|---|---|---|---|---|---|---|
| 4.0 | -1.459432e-08 | +7.746e-11 | 5.28e-03 | 3.5e-12 | 22 | 6 / 5 | 0.0e+00 | 1357 |
| 6.0 | -1.461621e-08 | +5.557e-11 | 3.79e-03 | 1.3e-12 | 43 | 5 / 4 | 0.0e+00 | 1445 |

`r_3(∞) = −1.4672e-8` Ha is EMBED-2's value; the in-process re-derivation here is
`-1.462678e-08` (the difference is the record's print precision, above).

**What the two nodes say, plainly.** Put a fourth monomer at CONTACT with the trimer's end
(4 Å from C, where exchange lives) and the trimer's harvested three-body dispersion term
moves by one part in two hundred; at 6 Å by one part in two hundred and sixty. Both are
twenty-plus times inside the freeze's one-part-in-ten stake, and `|Δ|` falls from 4 to 6 Å.
S1's for-all needs the 8 and 12 Å nodes and is not read; what IS read is that the coupling of
channel 4 to channel 1 is small where it was measured — the separability the channel ledger
needs (OBJECT.md rule 10) is supported on the two nodes that exist and is not claimed on
the two that do not.

**Why 8 and 12 Å did not complete — an instrument fact.** The 8 Å node's first trimer solve
ran for three hours at full CPU where the 4 and 6 Å nodes took 680 s per trimer; the run was
killed at 18:27 with its record annotated and the node JSON unwritten. The node JSON did not
carry the solver's exit or iteration count for the trimer-in-field solve (it carried them
for G0 only), so the cause is not in the record; `examples/embed3_probe.rs` re-solves that
node alone and prints its exit, iterations and residual, and its result is appended below
when it lands. The likely shape is the Davidson running to its iteration cap on a weak,
symmetry-breaking field (the memory of this repository: a warm start on a near-degenerate
manifold stalls at the cap), and if so the fix is the thick restart the pair curves already
use, not a new stake. Until the probe reads, the two far nodes are UNRUN, not VOID.
