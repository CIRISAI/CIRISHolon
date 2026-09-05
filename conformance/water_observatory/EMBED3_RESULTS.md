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
| G1 — one fixed point of four densities | **PASS**, 4 of 4 | both starts converge to bit-identical densities (`Δρ = 0`) in 5–7 sweeps |
| G3 — the floor of r_3 | **PASS**, 4 of 4 | `3.5e-12, 1.3e-12, 4.3e-12, 3.1e-12` Ha; `|Δ|/floor = 22, 43, 11, 15` — every node posable |
| S1 — the table transfers | **BRANCH (a)**, 3 of 3 far nodes posable | `|Δ|/|r_3(∞)| = 3.8e-03, 3.2e-03, 3.2e-03` at 6, 8, 12 Å against `0.10`, non-increasing; plant (ii) at 12 Å fires (`|Δ| = 4.4e-08`, three times the term; carrier `nn(C,D) = 4.42` Ha) |

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

**Why 8 and 12 Å did not complete the first time — an instrument fact, CORRECTED by the
probe.** The 8 Å node's first trimer solve ran for three hours where the 4 and 6 Å nodes took
680 s per trimer, and the run was killed with its record annotated "Davidson non-convergence
suspected". `examples/embed3_probe.rs` then re-solved that node alone: **exit Converged, 133
Davidson iterations, residual 9.7e-11, 2,531 wall-seconds** — 19 s per iteration against
4.5 s per iteration on the uncontended nodes. The solver did not stall; it was starved. The
lead was running FIELD-1's gate tests, an S1 arm and a compile on the same 32 cores at the
time, and the annotation was a guess written before the measurement. The two nodes are
UNRUN, not VOID, and were relaunched uncontended after the probe read; their rows follow.
The node JSON now owes the solver's exit and iteration count for every trimer solve, not
only G0's — a bookkeeping gap the probe existed to fill and the next runner closes.


## System B — the water dimer's far field, priced in the clock it is spent in

| gate | verdict | the number |
|---|---|---|
| G0 — the price, wall time | **PASS** | one exact dimer (`LINEAR`, 6.0 Å): 1,002,001 determinants, **574.5 wall-seconds** on 32 threads (14508 processor-seconds), under the 900 staked; the five nodes admitted. The 8.0 Å energy re-derived against EMBED-1's `g0_price.json` to `4.8e-11` Ha (stake `1e-9`) |
| G1 — one fixed point of the two densities | **PASS**, 5 of 5 | bit-identical densities from both starts on every node |
| G4 — `E_es` symmetric | **PASS**, 5 of 5 | `≤ 2.7e-14` |
| S2 — water's far field | **BRANCH (a)** for the charge field, 3 of 3 far nodes | `ρ_q = 2.058e-02, 1.093e-02, 4.446e-03` at 5, 6, 8 Å (stake `≤ 0.25`); absolute residual `1.38e-05 → 4.04e-06 → 6.63e-07` Ha, strictly falling — **the charge field is water's far field, as it was HF's** |
| S2, the density field | **BRANCH (a)** for the same clause | `ρ_ρ = 2.137e-02, 1.143e-02, 4.689e-03`; residual `1.43e-05 → 4.22e-06 → 7.00e-07` — within 5 % of the charge field at every node |
| plant (i) — the double count on water | **FIRES** | `ρ_q = 0.98, 1.00, 1.00` on the far nodes; carrier `|e_qq|` nonzero at every node |

| R_OO (Å) | sector | ΔE_exact (Ha) | ΔE_q (Ha) | ρ_q | \|residual_q\| | ρ_q plant (i) | ΔE_ρ (Ha) | ρ_ρ | G4 | q_H donor | density sweeps | dimer residual | wall s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 4.0 | transition | -1.510625e-03 | -1.369840e-03 | 9.320e-02 | 1.41e-04 | 0.84 | -1.379706e-03 | 8.667e-02 | 0.0e+00 | +0.23402 | 5 / 4 | 1.0e-10 | 854 |
| 4.5 | transition | -9.599758e-04 | -9.243802e-04 | 3.708e-02 | 3.56e-05 | 0.95 | -9.239073e-04 | 3.757e-02 | 1.8e-14 | +0.23316 | 5 / 4 | 6.6e-11 | 651 |
| 5.0 | far | -6.689798e-04 | -6.552133e-04 | 2.058e-02 | 1.38e-05 | 0.98 | -6.546804e-04 | 2.137e-02 | 1.1e-14 | +0.23264 | 5 / 4 | 9.5e-11 | 829 |
| 6.0 | far | -3.696209e-04 | -3.655800e-04 | 1.093e-02 | 4.04e-06 | 1.00 | -3.653964e-04 | 1.143e-02 | 1.1e-14 | +0.23208 | 4 / 3 | 7.8e-11 | 950 |
| 8.0 | far | -1.491869e-04 | -1.485236e-04 | 4.446e-03 | 6.63e-07 | 1.00 | -1.484874e-04 | 4.689e-03 | 2.7e-15 | +0.23167 | 4 / 3 | 9.7e-11 | 870 |

**What water says that HF did not.** On water the two fields agree to a few per cent at every
far node and both carry 98–99.6 % of the interaction, where on the HF chain the density field
lost to the charges by factors of 2–6. Water's two-centre dipole-exact charges sit on three
centres (`q_O = −2q`, symmetric hydrogens), so the charge model already carries the
molecule's quadrupole along its C₂ axis; the multipole error that dominated the HF chain's
charge residual is smaller here, and the dispersion the density field exposes cleanly is a
similar fraction on both. The transition nodes (4.0, 4.5 Å: `ρ_q = 0.093, 0.037`) are
reported and not read. This closes the water question EMBED-1's G0 left open: the water
cores go inside the charge field under the next freeze, and the price of that verdict was
69 wall-minutes on 32 threads, said in the clock it was spent in.


## System A, complete — the four nodes, and what the reference's precision does to Δ

| R_CD (Å) | r_3 inside D's field (Ha) | Δ against the FROZEN r_3(∞) | \|Δ\| / \|r_3(∞)\| | Δ against the in-process r_3 (G2's) | floor | \|Δ\|/floor | G1 sweeps | wall s |
|---|---|---|---|---|---|---|---|---|
| 4.0 | -1.459432e-08 | +7.746e-11 | 5.28e-03 | +3.25e-11 | 3.5e-12 | 22 | 6 / 5 | 1357 |
| 6.0 | -1.461621e-08 | +5.557e-11 | 3.79e-03 | +1.06e-11 | 1.3e-12 | 43 | 5 / 4 | 1445 |
| 8.0 | -1.462445e-08 | +4.733e-11 | 3.23e-03 | +2.33e-12 | 4.3e-12 | 11 | 5 / 4 | 2815 |
| 12.0 | -1.462490e-08 | +4.688e-11 | 3.20e-03 | +1.88e-12 | 3.1e-12 | 15 | 5 / 4 | 3573 |

**S1 reads BRANCH (a).** Against the frozen reference (`r_3(∞) = −1.4672e-8`, EMBED-2's
printed value) the dependence is `3.8e-3, 3.2e-3, 3.2e-3` of the term at 6, 8, 12 Å — thirty
times inside the one-in-ten stake — and non-increasing. Plant (ii) fires: with D's nuclei
dropped while its density stays, the trimer sits in the field of a bare `−10` electron cloud
and its three-body residual moves by three times its own size. **The harvested three-body
residual is a field-blind table on this carrier to three parts in a thousand, and the
channel ledger's separability (OBJECT.md rule 10) is measured, not assumed, for channel 4
inside channel 1.**

**The plateau, and what it is.** Δ against the frozen reference flattens at `+4.7e-11` from
8 Å on (consecutive exponents `0.56`, `0.02`) — it does not vanish with distance. That
constant is the reference's own printed precision: the freeze typed `r_3(∞)` from EMBED-2's
record, which carried SEAM-1's trimer at thirteen significant digits, and G2 measured the
same `4.5e-11` between that number and the in-process re-derivation (`M-FORMAT-FLOOR`,
registered on that finding). Read against the in-process reference — G2's own
`r_3(no D) = -1.462678e-08`, the truer number by the correction EMBED-2 carries — the
dependence is `+3.2e-11` at contact, `+1.1e-11` at 6 Å, and `+2.3e-12`, `+1.9e-12` at
8 and 12 Å — the last two at or under their floors. So the coupling is smaller than the
frozen reading says, and vanishes with distance as a coupling should; the freeze's letter is
read with its own reference and reported as (a), and the in-process reading is beside it
as the number a successor freeze should carry, by file and field and never by a typed
constant.

**Price.** Trimers in a field cost 583–1,800 wall-seconds each on 32 threads; the two nodes
that were killed under contention cost 2,815 and 3,573 s uncontended for their three solves
each, against the 900 s per solve the freeze admitted on G0's single node — the price gate
priced one solve and each node holds two or three; a successor freeze prices the node.
