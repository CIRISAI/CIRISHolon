# EMBED-1 — results

*Freeze: `EMBED_PREREG.md` (b8e2f80, committed alone). Instrument:
`engine/crates/holon-chem/src/embed.rs` with the one refactor
`pair::geometry_problem_from_basis`; gates as tests in `holon-chem/tests/embed.rs`;
runner `holon-chem/examples/embed_campaign.rs`, one JSON per node under `embed/`,
resumable by skipping nodes on disk. Every solve `solve_determinant` on the host.
Processor time from `/proc/self/stat` at 100 ticks per second; the wall time is beside
it in every JSON.*

## The verdict, first

**System 1 (the HF dimer) reads BRANCH (a): mutual point-charge embedding with
dipole-exact charges IS the far field.** On every far node the residual fraction is
below the stake by more than an order of magnitude and the absolute residual falls
strictly with distance. Plant (ii) fires at `ρ ≈ 1` on every far node, the Mulliken
control sits at `ρ ≈ 0.10`, and the near sector is reported and not read. **System 2
(the water dimer) is priced and pending**: its G1 pin passed and G0's single
1,002,001-determinant node is running as this document is first committed; its
section is appended by the next commit with the price, the admission, and — if
admitted — its five nodes.

| gate | verdict | the number |
|---|---|---|
| G1 — monomer pins | **PASS**, 2 pins | HF `R = 1.8794379298` bohr, `|dE/dR| = 4.95e-8`; H₂O `r = 1.9435738400` bohr, `θ = 1.6887434037` rad (96.7579°), `|dE/dr| = 1.85e-7`, `|dE/dθ| = 1.07e-7` |
| G2 — the density is exact | **PASS** (3 atoms + the HF monomer isolated and in a test charge) | traces to `1e-12`; the density route's `sqrt(<r²>/N)` equals `atomic_rms_radius` to `1e-10` relative on H, O, F; both charge models neutral to `1e-12`; the dipole-exact charges reproduce the density's dipole to `1e-12` |
| G3 — Hellmann–Feynman | **PASS**, 6 points | worst `|dE/dq − V(R)| = 2.006e-9` hartree per unit charge (stake `1e-7`) |
| G4 — one fixed point | **PASS**, 10 of 10 System-1 nodes | charges from the two starts agree to `≤ 2.0e-12`, `E_emb` to `≤ 6.5e-13`; 3–6 iterations |
| S3 — the far field, System 1 | **BRANCH (a)**, 4 of 4 far nodes | `ρ = 1.55e-2, 7.35e-3, 2.23e-3, 7.62e-4` at 5, 6, 8, 10 Å (stake `≤ 0.25`); residual `6.1e-6 → 1.68e-6 → 2.15e-7 → 3.77e-8` hartree, strictly decreasing |
| plant (i) — the field's sign | **FIRES** | derivative off by `2|V| = 6.06e-2`; carrier `|V| = 3.03e-2` at the nearest point, nonzero in its sector |
| plant (ii) — the double count | **FIRES** | `ρ = 1.053, 1.040, 1.024, 1.016` on the far nodes; carrier `|e_qq| ≥ 5.0e-5` hartree at every far node |
| plant (iii) — the fermionic phase | **FIRES at G3, silent at G2's trace, as staked** | off-diagonal carrier `4.5e-6` hartree per unit charge; the unsigned density's trace is still `10` to `1e-12` |
| G0 — System 2's price | **running** | one `LINEAR` node at 8.0 Å, `C(14,10)² = 1,002,001` determinants |

## System 1 — every node

Supermolecule: 4,356 determinants, Davidson residual `≤ 9.8e-11`, 16–25 iterations;
1.9–3.0 processor-seconds per node including all six embedded runs (two charge models ×
two starts and the plant); 24.6 s for the ten. `q_H` is the donor's hydrogen charge
under each model; `μ_z` the donor's exact dipole from the density (a.u.).

| R (Å) | sector | ΔE_exact (Ha) | ΔE_emb primary (Ha) | residual (Ha) | ρ primary | ρ Mulliken | ρ plant (ii) | SC iters zero/isolated | G4 Δq | G4 ΔE | q_H primary | q_H Mulliken | μ_z donor | e_qq (Ha) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2.4 | near | +8.051790e-03 | −3.210455e-03 | +1.126e-02 | 1.399 | 1.395 | 2.064 | 6 / 6 | 1.7e-12 | 6.3e-13 | +0.239434 | +0.161220 | +0.450001 | −5.3551e-03 |
| 2.6 | near | +4.181382e-05 | −2.557488e-03 | +2.599e-03 | 62.2 | 60.6 | 154.8 | 6 / 5 | 1.3e-12 | 6.5e-13 | +0.234032 | +0.158009 | +0.439848 | −3.8750e-03 |
| 2.8 | near | −2.078465e-03 | −2.064293e-03 | −1.417e-05 | 6.818e-03 | 4.361e-02 | 1.399 | 5 / 5 | 1.6e-12 | 4.5e-13 | +0.230319 | +0.155781 | +0.432869 | −2.9225e-03 |
| 3.0 | near | −2.209621e-03 | −1.696093e-03 | −5.135e-04 | 2.324e-01 | 2.684e-01 | 0.796 | 5 / 5 | 2.0e-13 | 2.3e-13 | +0.227674 | +0.154185 | +0.427899 | −2.2732e-03 |
| 3.5 | transition | −1.307207e-03 | −1.095590e-03 | −2.116e-04 | 1.619e-01 | 2.163e-01 | 0.858 | 5 / 4 | 7.8e-14 | 0 | +0.223689 | +0.151771 | +0.420409 | −1.3334e-03 |
| 4.0 | transition | −7.961844e-04 | −7.459872e-04 | −5.020e-05 | 6.305e-02 | 1.339e-01 | 1.014 | 4 / 4 | 1.4e-13 | 1.4e-13 | +0.221607 | +0.150509 | +0.416497 | −8.5778e-04 |
| 5.0 | **far** | −3.942182e-04 | −3.881137e-04 | −6.105e-06 | **1.549e-02** | 1.006e-01 | 1.053 | 4 / 4 | 3.8e-14 | 3.7e-13 | +0.219659 | +0.149330 | +0.412835 | −4.2102e-04 |
| 6.0 | **far** | −2.279581e-04 | −2.262820e-04 | −1.676e-06 | **7.353e-03** | 9.833e-02 | 1.040 | 4 / 3 | 2.0e-13 | 1.7e-13 | +0.218834 | +0.148834 | +0.411285 | −2.3873e-04 |
| 8.0 | **far** | −9.632968e-05 | −9.611466e-05 | −2.150e-07 | **2.232e-03** | 9.843e-02 | 1.024 | 4 / 3 | 1.1e-13 | 8.5e-14 | +0.218196 | +0.148452 | +0.410085 | −9.8882e-05 |
| 10.0 | **far** | −4.939784e-05 | −4.936018e-05 | −3.766e-08 | **7.624e-04** | 9.919e-02 | 1.016 | 3 / 3 | 6.0e-14 | 0 | +0.217972 | +0.148319 | +0.409665 | −5.0237e-05 |

**Read under branch (a).** The residual's scaling across the far sector is an
observation the freeze asks for only under branch (b), reported here because it is
free: consecutive-node exponents `−7.09` (5→6 Å), `−7.14` (6→8 Å), `−7.81` (8→10 Å).
That is steeper than dispersion (`R⁻⁶`) and steeper than the dipole–quadrupole term a
two-centre charge model cannot carry (`R⁻⁴`); it is not assigned to a mechanism here
and is not read for anything. What IS read: the field the embedding misses at five
ångström and beyond is under two per cent of the interaction and vanishing faster
than the interaction itself.

**The near sector, reported and not read.** At 2.4 Å the exact interaction is
repulsive (`+8.05e-3`) and the embedded one attractive (`−3.21e-3`): exchange
repulsion is absent from a point-charge field by construction, which is what Build 2's
exact cores are for. At 2.6 Å the exact interaction crosses zero (`+4.2e-5`) and `ρ`
is a division by a near-zero, meaningless by construction. Through the transition the
fraction falls `0.23 → 0.16 → 0.063` from 3.0 to 4.0 Å.

**The Mulliken control.** Its `ρ ≈ 0.10` is flat across the far sector because its
charges carry `0.279` a.u. of the donor's `0.410` a.u. dipole (`q_H = 0.148` against
the dipole-exact `0.218`); it is what a charge model that misses the dipole looks
like, and it is why the primary was chosen dipole-exact. Read for nothing.

## System 2 — the water dimer

G1 passed (above). G0 is running; this section is completed by the next commit.
