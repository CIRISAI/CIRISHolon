# SEAM-1 — results

*Freeze: `SEAM_PREREG.md` (b58dc0e, alone). Amendment: `SEAM_AMENDMENT_1.md` (36ba0e4 committed
with a message that wrongly claimed admissibility; b8652bc corrected the text, was admitted, and
says so). Instrument `holon-chem/src/seam.rs` on `embed.rs`; gates `holon-chem/tests/seam.rs`;
runner `examples/seam_campaign.rs`, JSON per node under `seam/`; the floor instrument
`examples/seam_floor.rs`. Every solve `solve_determinant` on the host, one device class, both
clocks in every JSON.*

## The verdict, first

**S1 reads BRANCH (b) by its frozen letter, and the reading is kept.** The fraction clause holds
at every posable far node by a factor of 300 or more — the embedding carries 99.93 %, 99.92 %
and 99.93 % of the bare three-body term at 5, 6 and 8 Å — and the monotonicity clause FIRED at
the 5 → 6 Å pair, where κ rose from 5.07e-4 to 7.81e-4. The residuals behind that pair are
2.3e-9 and 1.2e-9 hartree, differences of seven exact solves with a roundoff floor of order
1e-10, and the freeze staked a floor for the denominator only. That is a registered misfit now
(**M-FLOOR-UNSTAKED**), and AMENDMENT 1 measures the floor by a translation null and re-stakes
the clause on posable pairs; its reading, S1′, is in the last section. No claim rounds up: the
closure-at-two-body claim, branch (a), is NOT made by S1.

| gate | verdict | the number |
|---|---|---|
| G0 — the price, in wall time | **PASS**, 1 node | 665,856 determinants, `E = -295.8101104595` Ha, residual `8.5e-11`, 143 Davidson iterations, **264.8 wall-seconds** on 32 threads (7177 processor-seconds), under the 900 staked: all eight nodes admitted |
| G1 — the monomer pin | **PASS** | EMBED-1's `R_HF = 1.8794379298` bohr, `|dE/dR| = 4.95e-8` |
| G2 — one fixed point for three | **PASS**, 8 of 8 nodes | charges from the two starts agree to `≤ 4.3e-12`, `E_EE-PA` to `≤ 2.4e-12`; 4–7 sweeps |
| G3 — zero charges reduce the embedded machinery to the bare one | **PASS, EXACT**, 8 of 8 | `|E_EE-PA(q=0) − E_PA| = 0.0` on every node (bit-identical) |
| S1 — the seam's premise, far sector | **BRANCH (b)** by the letter, 3 posable nodes | fraction clause: `κ = 5.07e-4, 7.81e-4, 6.78e-4` (stake 0.25) — holds; monotonicity clause: fires at 5 → 6 Å |
| plant (i) — the field left out of the pairs | **FIRES** | `κ = 387, 674, 1608` on the far nodes; carrier `|E_AB[q_C] − E_AB| = 5.0e-4, 2.8e-4, 1.1e-4` Ha, nonzero in the sector |
| plant (ii) — the twin confusion (Mulliken) | **FIRES** | `κ(Mulliken) = 0.087, 0.092, 0.096` against the primary's `5e-4 … 8e-4` at every far node; carrier: dipoles differ by 0.13 a.u. |
| S1′ — AMENDMENT 1, the clause on posable pairs | **BRANCH (b) on measured ground**, 3 posable nodes, 2 posable pairs | floors `3.0e-12, 3.4e-12, 3.8e-12` Ha; `|r_emb|/floor = 752, 344, 46` (stake ≥ 10): every far node posable; κ rises on the posable pair 5 → 6 Å and falls on 6 → 8 Å — the rise is physics, not arithmetic |

## Every node

Trimer solves: Davidson residual `≤ 1.0e-10`, 106–143 iterations, 229–290 wall-seconds per node on 32 threads (2031 s and 51600 processor-seconds for the eight). `q_H` is each monomer's hydrogen charge under the primary model at the fixed point; `μ_z` the embedded monomers' exact dipoles.

| R (Å) | sector | ΔE₃ bare (Ha) | r_emb primary (Ha) | κ primary | κ Mulliken | κ plant (i) | plant (i) carrier (Ha) | SC sweeps zero/isolated | G2 Δq | G2 ΔE | G3 | q_H (A, B, C) | μ_z (A, B, C) | wall s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2.6 | near | -2.749801e-03 | -9.856961e-04 | 3.58e-01 | 0.541 | 6.3 | 5.47e-03 | 7 / 6 | 3.6e-12 | 1.1e-12 | 0.0e+00 | +0.23701, +0.25388, +0.24090 | +0.4454, +0.4771, +0.4528 | 252 |
| 2.8 | near | -1.039825e-03 | -2.616247e-04 | 2.52e-01 | 0.451 | 11.7 | 3.86e-03 | 6 / 6 | 3.7e-12 | 2.4e-12 | 0.0e+00 | +0.23244, +0.24523, +0.23498 | +0.4369, +0.4609, +0.4416 | 229 |
| 3.0 | near | -4.085647e-04 | -6.083320e-05 | 1.49e-01 | 0.355 | 22.5 | 2.88e-03 | 6 / 5 | 4.3e-12 | 5.1e-13 | 0.0e+00 | +0.22926, +0.23923, +0.23100 | +0.4309, +0.4496, +0.4341 | 253 |
| 3.5 | transition | -6.213990e-05 | -1.071248e-06 | 1.72e-02 | 0.168 | 86.1 | 1.61e-03 | 5 / 5 | 1.8e-12 | 5.7e-14 | 0.0e+00 | +0.22455, +0.23040, +0.22536 | +0.4220, +0.4330, +0.4235 | 246 |
| 4.0 | transition | -1.912905e-05 | -2.159192e-08 | 1.13e-03 | 0.097 | 181.9 | 1.02e-03 | 5 / 5 | 8.3e-14 | 9.7e-13 | 0.0e+00 | +0.22214, +0.22588, +0.22257 | +0.4175, +0.4245, +0.4183 | 256 |
| 5.0 | far | -4.481382e-06 | +2.270724e-09 | 5.07e-04 | 0.087 | 387.5 | 4.96e-04 | 5 / 4 | 6.8e-14 | 5.1e-13 | 0.0e+00 | +0.21991, +0.22173, +0.22007 | +0.4133, +0.4167, +0.4136 | 259 |
| 6.0 | far | -1.474668e-06 | +1.151761e-09 | 7.81e-04 | 0.092 | 673.8 | 2.79e-04 | 4 / 4 | 1.2e-13 | 5.7e-13 | 0.0e+00 | +0.21897, +0.22000, +0.21905 | +0.4115, +0.4135, +0.4117 | 245 |
| 8.0 | far | -2.582758e-07 | +1.751914e-10 | 6.78e-04 | 0.096 | 1608.0 | 1.15e-04 | 4 / 4 | 2.4e-13 | 8.5e-13 | 0.0e+00 | +0.21825, +0.21867, +0.21828 | +0.4102, +0.4110, +0.4102 | 290 |

**The far sector's scaling** (consecutive-node exponents, `|x| ∝ R^−n`):

| pair | bare ΔE₃ | r_emb | κ | |
|---|---|---|---|---|
| 5 → 6 Å | 6.10 | 3.72 | 5.07e-04 → 7.81e-04 | rises |
| 6 → 8 Å | 6.06 | 6.55 | 7.81e-04 → 6.78e-04 | falls |

## What the record says, sector by sector

**Far (read).** The bare three-body term falls as `R^−6.1` — cooperative induction on a
chain of dipoles, as the freeze expected — and the embedding removes all but a part in
1,300 to 2,000 of it at every node. The residual's own scaling is `R^−3.7` over 5 → 6 Å
and `R^−6.6` over 6 → 8 Å; the second pair's residual (`1.75e-10`) is at the arithmetic
floor and its exponent is not readable, which is what AMENDMENT 1 measures. The middle
monomer carries the extra charge cooperativity predicts (`q_H` = 0.2199, **0.2217**, 0.2201
at 5 Å) and the fixed point is start-independent to `1e-13`.

**Transition and near (reported, not read).** The fraction the embedding carries falls
smoothly toward contact: `κ = 0.0011` (4.0 Å), `0.017` (3.5 Å), `0.15` (3.0 Å), `0.25`
(2.8 Å), `0.36` (2.6 Å). At the hydrogen-bond distance the point-charge field leaves a
third of the three-body term uncarried — non-additive exchange, which no charge model
holds and which is what Build 2's exact cores are for at contact.

**The controls.** Plant (i) — the pairs solved in vacuum while the monomers stay
embedded — leaves the QM–charge terms uncancelled and reads a residual hundreds of times
the three-body term, exactly the bookkeeping the identity G3 protects. The Mulliken control
carries 91 % of the term where the primary carries 99.9 %: the twin confusion (manner for
fact) priced again, at a tenth of the term.

## AMENDMENT 1 — the floor, measured, and S1′

*`SEAM_AMENDMENT_1.md` (admitted at b8652bc), run after it; `examples/seam_floor.rs`;
`seam/floor_R*.json`. The whole chain moved by `(0.37, 0.21, 0.5)` bohr, every one of the
seven solves repeated there with the charges re-derived, and the residual's change is the
node's floor.*

| R (Å) | r_emb (Ha) | r_emb, moved (Ha) | floor | \|r_emb\| / floor | ΔE₃ floor | E_ABC floor | sweeps base / moved | wall s |
|---|---|---|---|---|---|---|---|---|
| 5.0 | +2.2666e-09 | +2.2696e-09 | 3.01e-12 | 752 | 3.2e-12 | 3.4e-12 | 5 / 5 | 752 |
| 6.0 | +1.1519e-09 | +1.1485e-09 | 3.35e-12 | 343 | 3.4e-12 | 3.2e-12 | 4 / 4 | 825 |
| 8.0 | +1.7684e-10 | +1.8065e-10 | 3.81e-12 | 46 | 2.0e-12 | 2.4e-12 | 4 / 4 | 783 |

**Every far node is posable** (`|r_emb|/floor ≥ 10` by a factor of 5 to 75): the arithmetic
floor of the seven-solve difference is `3–4e-12` hartree, thirty times below the estimate
in A1.1, and the residuals of `2.3e-9`, `1.2e-9` and `1.8e-10` are physics at this
resolution — the 8 Å node included, which A1.1 had expected to be unreadable and which the
measurement admits at 46 times its floor.

**S1′ reads BRANCH (b) on measured ground.** The posable pair 5 → 6 Å has κ rising
(`5.07e-4 → 7.81e-4`); the posable pair 6 → 8 Å has it falling (`7.81e-4 → 6.78e-4`). By
A1.3 one rising posable pair is (b): the embedded residual's decay is NOT uniformly at
least as fast as the bare three-body term's — `R^−3.7` against `R^−6.1` over 5 → 6 Å, then
`R^−6.6` against `R^−6.1` over 6 → 8 Å — so what the point-charge field misses has a
slowly-decaying component visible at 5–6 Å that the bare term does not have. Its size is a
part in 1,300 to 2,000 of the term; its shape is the thing the next freeze must stake.

**What is licensed (A1.4).** The seam's premise on this carrier at the fraction level: exact
cores solved inside the field carry 99.9 % of the three-body term the bare ladder cannot
terminate, and the water cores go inside the field under the next freeze. What is NOT
licensed, by two readings of the same clause: the statement that the residual vanishes at
least as fast as the term. The next freeze stakes what the charges mis-carry. Two
candidates, in the order they should be tried, both buildable on `embed.rs`: (1) the field
as the partners' own DENSITIES rather than their charges — the fragment solved in the exact
Coulomb potential of the embedded partner densities, which removes the multipole truncation
entirely and needs the inter-fragment Coulomb matrix and nothing else; (2) charges plus a
point dipole per fragment, the classical polarisable-embedding step. Either is EMBED-2.

**One more thing the floor run measured.** The fixed point is translation-invariant to the
same floor: the moved chain's charges converge in the same number of sweeps to the same
energies within `3e-12`, and the bare three-body term moves by `2–3e-12` — which is the
resolution any future clause on this carrier may claim, and no better.
