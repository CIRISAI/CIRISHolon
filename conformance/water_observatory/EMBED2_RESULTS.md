# EMBED-2 — results: the field as the partners' densities, and the residual harvested

*Freeze `EMBED2_PREREG.md` (b88e904, alone). Instrument `holon-chem/src/density_embed.rs`
with the hook `pair::geometry_problem_with_potential`; gates `holon-chem/tests/density_embed.rs`;
runner `examples/embed2_campaign.rs`, JSON per node under `embed2/`. The referee is SEAM-1's own
exact trimers, reused by node and checked by G0; the far-sector floor is the translation null
against SEAM-1's moved trimers. Every solve `solve_determinant` on the host.*

## The verdict, first

**S1 reads BRANCH (b) by its letter: the density field carries LESS of the three-body term
than the charge field at every far node** (`κ_ρ = 3.27e-3, 1.81e-3, 7.97e-4` against
`κ_q = 5.07e-4, 7.81e-4, 6.78e-4` at 5, 6, 8 Å), with `κ_ρ` non-increasing on both posable
pairs. The branch the freeze named for that outcome — "the slow component is not
electrostatic; `R⁻⁹` on the chain is three-body dispersion" — is what the residual wears:
its consecutive-node exponents are **9.34 and 8.91**, its sign is the attractive sign the
Axilrod–Teller term has on a linear triple, and one constant fits all three far nodes.

**Read sideways (the operator's correction, 2026-09-04): this campaign did not fail to remove
a residual; it MEASURED the three-body dispersion energy of the carrier**, exactly, to a floor
of `1e-12` hartree, as the residual of an embedded closed view. On the far sector

```
r_ρ(R) = −C / R⁹,   C = 8.52 Ha·bohr⁹   (one constant, three nodes)
```

| R (Å) | measured r_ρ (Ha) | −C/R⁹ | miss |
|---|---|---|---|
| 5.0 | -1.467e-08 | -1.419e-08 | +3.3 % |
| 6.0 | -2.672e-09 | -2.751e-09 | -3.0 % |
| 8.0 | -2.057e-10 | -2.065e-10 | -0.4 % |

The understanding of the gap and the understanding of the value are the same number: the
defect of the two-body embedded view is the quantity the three-body view carries, and the
engine's ladder already carries such quantities as tables. What embedding changed is that
the residual left to tabulate has NO slow tail — the field took it — which is why an
embedded expansion terminates and the bare one (dE5) could not.

| gate | verdict | the number |
|---|---|---|
| G0 — the referee is the record's | **PASS**, 8 of 8 | this run's bare pairwise sum equals SEAM-1's to `≤ 4.2e-11` Ha on every node |
| G1 — Hellmann–Feynman on the density field | **PASS**, 2 points | `|dE/dλ − (⟨ρ_A|J⟩ − Σ Z V)| = 3.8e-10` (5 Å), `7.6e-10` (3 Å) |
| G2 — one fixed point of the densities | **PASS**, 8 of 8 | the two starts converge to bit-identical densities and sums (`Δρ = 0`, `ΔE = 0`) in 4–7 sweeps |
| G3 — the reduction is exact | **PASS**, 8 of 8 | no partners anywhere: equal to the bare sum to the bit |
| G4 — the floor of r_ρ | **PASS**, 3 of 3 | `1.4e-12, 8.5e-13, 1.1e-13` Ha; `|r_ρ|/floor = 10,325, 3,133, 1,810` — every far node posable |
| S1 | **BRANCH (b)** | `κ_ρ > κ_q` at 5, 6 and 8 Å; `κ_ρ` non-increasing on both posable pairs; residual `∝ R^−9.3, R^−8.9`, attractive |
| plant (i) — the Coulomb sign | **FIRES** at G1 | derivative off by `2⟨ρ_A|J⟩ = 21.3` Ha; carrier `⟨ρ_A|J⟩ = 10.6` Ha at 5 Å, nonzero in the sector |
| plant (ii) — the partners' nuclei dropped | **FIRES** at S1, silent at G3 | `κ_ρ = 5,405, 7,758, 13,735` on the far nodes; carrier `E_nn(A, Z_b) = 10.7` Ha at 5 Å |

## Every node

| R (Å) | sector | ΔE₃ bare (Ha) | κ_q (charges, SEAM-1) | κ_ρ (densities) | r_ρ (Ha) | κ plant (ii) | G0 | sweeps zero / isolated | G2 Δρ | floor | \|r_ρ\|/floor | wall s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2.6 | near | -2.7498e-03 | 3.585e-01 | 4.025e-01 | -1.1068e-03 | 267 | 4.1e-11 | 7 / 6 | 0.0e+00 | — | — | 36 |
| 2.8 | near | -1.0398e-03 | 2.516e-01 | 2.932e-01 | -3.0485e-04 | 479 | 2.0e-11 | 7 / 6 | 0.0e+00 | — | — | 37 |
| 3.0 | near | -4.0856e-04 | 1.489e-01 | 1.844e-01 | -7.5345e-05 | 736 | 4.2e-11 | 7 / 6 | 0.0e+00 | — | — | 38 |
| 3.5 | transition | -6.2140e-05 | 1.724e-02 | 3.419e-02 | -2.1246e-06 | 1838 | 2.8e-11 | 6 / 5 | 0.0e+00 | — | — | 72 |
| 4.0 | transition | -1.9129e-05 | 1.129e-03 | 8.696e-03 | -1.6635e-07 | 3227 | 1.9e-12 | 6 / 5 | 0.0e+00 | — | — | 73 |
| 5.0 | far | -4.4814e-06 | 5.067e-04 | 3.274e-03 | -1.4672e-08 | 5405 | 3.1e-11 | 5 / 4 | 0.0e+00 | 1.4e-12 | 10325 | 65 |
| 6.0 | far | -1.4747e-06 | 7.810e-04 | 1.812e-03 | -2.6715e-09 | 7758 | 9.3e-12 | 5 / 4 | 0.0e+00 | 8.5e-13 | 3133 | 48 |
| 8.0 | far | -2.5828e-07 | 6.783e-04 | 7.965e-04 | -2.0572e-10 | 13735 | 7.6e-12 | 5 / 4 | 0.0e+00 | 1.1e-13 | 1810 | 46 |

**Far-sector scaling.** `r_ρ`: 9.34 (5 → 6 Å), 8.91 (6 → 8 Å); the bare term: 6.10, 6.06.

## What the two fields say together

**Why the charges looked better at 5–6 Å.** SEAM-1's charge residual was smaller there and
decayed with a changing exponent (`R^−3.7` then `R^−6.6`). With the density result beside it
that reads as two errors cancelling: the charge model's own multipole error (positive,
slowly decaying) against the same negative `R⁻⁹` dispersion the density field exposes
cleanly. At 8 Å the two embeddings agree to within 20 %, where the cancellation runs out.
SEAM-1's monotonicity firing was therefore physics, and not the physics S1′ named for it —
a cancellation crossover, not mis-carried induction. A reading, labelled as one; what is
measured is the two residuals.

**What the harvest fixes for the seam.** Classical embedding ends here: once the
electrostatics is exact, the far-field defect of an embedded two-body expansion is dispersion,
`R⁻⁹`, with a measured coefficient — the derived defect form the fold rule asks for, in
chemistry. Its reach follows from the coefficient: at the tables' declared per-term
uncertainty (`5e-5` Ha) the term is below threshold beyond about 2 Å. At contact both fields
miss 36–40 % of the three-body term (2.6 Å) — Pauli exchange, exponential — which only an
exact core on the triple carries. Build 2's shape: exact cores on pairs and contact triples
inside the field; the embedded residual tabulated for what remains; the far field bounded by
`C/R⁹`.

**Not measured here, staked next.** The harvested residual depends on the field its core sits
in; that dependence is what the next freeze stakes before any table is built from it. One
carrier, one basis, no water: the water triple (21 orbitals) is the labelled-MPS base's job.
