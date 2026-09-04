# Pre-registration — EMBED-3: is the harvested residual a TABLE? — its dependence on the field its core sits in, and the water dimer's far field priced in the clock it is spent in

*Frozen 2026-09-04, committed ALONE, before the runner existed. Built by the lead, no lane.
EMBED-2 harvested the three-body residual of an embedded two-body view as a value —
three-body dispersion, `r = −C/R⁹` on the HF chain. A value is a table entry only if it
transfers: the seam would tabulate the embedded residual of a triple and carry it wherever
that triple sits, inside whatever field the rest of the scene makes. This freeze stakes
whether the residual depends on that field, and by how much. Its second system returns to
the water dimer EMBED-1's G0 refused on an inherited processor-time constant, and prices it
in wall time with the thread count declared.*

**Do not read this as chemistry.** STO-3G, rigid geometries. The referee energies are the
engine's own exact solves; the HF chain's three-body value at 5.0 Å is EMBED-2's
(`r_ρ = −1.4672e-8` Ha, floor `1.4e-12`), reused by file and re-derived by G2.

---

misfits: contacts **M-FLOOR-UNSTAKED** (every clause that reads the field dependence has a
measured floor for THAT quantity — the translation null at every node — and nodes below ten
times it are VOID for the clause, named); **M-ONE-MODEL-DELTA** (the referees are exact
solves; the two embeddings are compared to the exact energy, never only to each other);
**M-VACUOUS-SUCCESS** (node counts on every gate); **M-NULL-MISSTAKE** (the dependence is
staked against the residual's own size at the same geometry, never against zero);
**M-EXIT-DISCRIMINATOR** (three outcomes named per system; a non-converged fixed point is
VOID); **M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants with carriers asserted nonzero in
the sector each acts on — §5); **M-STALE-INSTRUMENT** (this freeze alone; runner, JSON and
results document together); **M-CHEAPER-THAN-ITS-PRICE** and **M-PLACEMENT-LOTTERY** (the
price is staked in WALL seconds with the thread count recorded — the clock the budget is
spent in, the lottery over placement named and accepted for one machine class — and a
node arriving under half its measured price is VOID); **M-DEVICE-CLASS** (host `f64`,
`solve_determinant`, one class); **M-MAX-OVER-SUCCESSES** (every S-clause is a for-all);
**M-FIXED-POINT-TRAJECTORY** (two starts, one fixed point — G1); **M-COND-PROBE** ("inside
the field" appears; a one-electron term applied before the solve); **M-HOMOG** and
**M-BARE-CHARGE** (the words "local" and "charge" appear; nothing homogeneous or
gauge-charged is meant); **M-UNTESTED-GAP** (the water far field is read on the far sector
only; the transition nodes are reported, not read); **M-VOLUME-SCALE** (Ewald named, not
built). Not contacted: the rest of the registry.

## 0. The two claims

**System A — the residual's field dependence.** The HF chain A–B–C at `R_FF = 5.0` Å (the
node where `r_3 = −1.4672e-8` Ha) with a FOURTH monomer D placed on the chain beyond C at
`R_CD ∈ {4.0, 6.0, 8.0, 12.0}` Å, all four densities at their mutual fixed point (G1). The
embedded three-body residual of ABC INSIDE D's field is

```
r_3(R_CD) = E_ABC[ρ_D] − ρPA_ABC[ρ_D]
```

where `E_ABC[ρ_D]` is the exact trimer solved in D's density and nuclei, and `ρPA_ABC[ρ_D]`
is the density-embedded pairwise sum over ABC with every monomer and dimer solved in the
field of its ABC partners AND of D. With D removed this is EMBED-2's `r_ρ` (gate G2). The
dependence is `Δ(R_CD) = r_3(R_CD) − r_3(∞)`, `r_3(∞) = −1.4672e-8` Ha.

**S1 — the table transfers.** For `R_CD ≥ 6.0` Å (three nodes), `|Δ| ≤ 0.10 · |r_3(∞)|` at
every node, and `|Δ|` non-increasing across posable consecutive pairs.

**System B — the water dimer's far field, priced right.** DIMER-1's `LINEAR` arrangement at
EMBED-1's pins (`r = 1.9435738400` bohr, `θ = 1.6887434037` rad), `R_OO ∈ {4.0, 4.5, 5.0,
6.0, 8.0}` Å, the exact 1,002,001-determinant dimer as referee (the 8.0 Å energy is EMBED-1's
`g0_price.json`, `E = −150.0467322495` Ha, re-derived by G0), the charge-embedded and the
density-embedded monomer sums beside it:

```
ΔE_exact = E_AB − E_A⁰ − E_B⁰
ΔE_q     = E_A[q_B] + E_B[q_A] − Σ q_a q_b/r_ab − E_A⁰ − E_B⁰          (EMBED-1)
ΔE_ρ     = E_A[ρ_B] + E_B[ρ_A] − E_es(ρ_A, ρ_B) − E_A⁰ − E_B⁰
E_es     = nn(A,B) − Σ_a Z_a V_B^el(R_a) − Σ_b Z_b V_A^el(R_b) + tr(P_A J[P_B])
ρ_q, ρ_ρ = |ΔE_exact − ΔE_·| / |ΔE_exact|
```

**S2 — water's far field.** On the far sector (`R_OO ≥ 5.0` Å, three nodes) `ρ_q ≤ 0.25` at
every node with the residual non-increasing on posable pairs — EMBED-1's S3, on water.
`ρ_ρ` is reported beside it and read for the same clause as a second embedding.

## 1. The instrument (`density_embed.rs` extended; runner `examples/embed3_campaign.rs`)

- `rho_pa_subset(fragments, densities, subset)`: the density-embedded pairwise sum over a
  subset, every solve in the field of ALL other fragments (inside and outside the subset).
- `classical_interaction(A, P_A, B, P_B)`: `E_es` above from the two frozen densities and
  nuclei, by the machinery EMBED-2 built; symmetric under `A ↔ B` to `1e-12` (a test on the
  ERI-block extraction).
- The exact trimer in a field: `solve_in_densities` on the joined ABC (18 orbitals) with D as
  partner. The exact water dimer: `supermolecule`. Every solve `solve_determinant`, host.
- The floor of `r_3(R_CD)` at every node by the translation null (`(0.37, 0.21, 0.5)` bohr):
  the whole four-chain moved, every solve repeated, the fixed point re-derived.

## 2. Gates

Node counts stated; fewer than staked is VOID.

- **G0 — prices, in wall time.** System A: one trimer-in-field solve at `R_CD = 12` Å,
  wall seconds and thread count recorded; the four nodes and their nulls (eight trimer
  solves) admitted if `≤ 900` s each. System B: one exact water dimer at `R_OO = 6.0` Å,
  admitted if `≤ 900` s (EMBED-1 measured 451 s on 27 threads); the 8.0 Å record energy
  re-derived to `1e-9` or the node refused. 2 nodes.
  witness: none (measured prices)
- **G1 — one fixed point of four densities.** From the empty field and from the isolated
  densities, the four-fragment iteration converges within 100 sweeps to the same densities
  (`≤ 1e-8` per element) and the same `ρPA_ABC[ρ_D]` (`≤ 1e-10`). 4 nodes. Likewise the two
  water densities at every node. 5 nodes.
  witness: none (a measured fixed-point property)
- **G2 — the identity with EMBED-2.** With D removed, `r_3` equals EMBED-2's `r_ρ(5.0 Å)` to
  `1e-12`. 1 node.
  witness: none (an in-engine identity between two campaigns)
- **G3 — the floor of `r_3`, measured.** `floor(R_CD) = |r_3 − r_3(moved)|` at every node;
  posable iff `|Δ| ≥ 10 · floor`. 4 nodes.
  witness: none (a measured resolution)
- **G4 — `E_es` is symmetric.** `|E_es(A,B) − E_es(B,A)| ≤ 1e-12` on every water node and on
  the HF pair at 5 Å. 6 checks.
  witness: none (an in-engine identity)
- **S1 — the table transfers (System A).** **(a)** `|Δ| ≤ 0.10 · |r_3(∞)|` at every node with
  `R_CD ≥ 6.0` Å AND `|Δ|` non-increasing across posable consecutive pairs (at least one)
  ⇒ the harvested three-body residual is a table to one part in ten beyond 6 Å: the seam
  carries it geometry-indexed, field-blind, with this bound. **(b)** any such node with
  `|Δ| > 0.10 · |r_3(∞)|`, or `|Δ|` rising ⇒ the residual is field-dependent at that level;
  its scaling in `R_CD` is reported (a dipole field's second-order effect goes as `R_CD⁻⁴`),
  and the table gains a field coordinate in the next freeze. **(c)** every node unposable
  ⇒ field-independent to the instrument's resolution, written as such. The 4.0 Å node is
  reported under every branch and read under none (D at 4 Å from C is a contact, and
  contact is exchange).
  witness: none (a measured dependence against a measured value)
- **S2 — water's far field (System B).** **(a)** `ρ_q ≤ 0.25` at 5.0, 6.0 and 8.0 Å with the
  absolute residual non-increasing across consecutive far nodes or below `1e-8` Ha ⇒ the
  charge field is the water dimer's far field, as it was HF's; `ρ_ρ` read for the same
  clause. **(b)** otherwise, reported with scaling. Transition nodes (4.0, 4.5 Å) reported,
  not read.
  witness: none (a measured far-field residual)

## 3. What each outcome means

S1(a) or (c) licenses the seam's three-body table as field-blind at the stated bound; S1(b)
says the table needs a field coordinate and how strongly, which is a design fact, not a
failure. S2(a) closes the water question EMBED-1 left open and licenses the water cores
inside the charge field; S2(b) names what water needs that HF did not. A G0 refusal removes
its system and touches no claim.

## 4. Plants

- **(i) The double count on water.** The charge–charge subtraction omitted from `ΔE_q` at
  every water node. S2 must fire with `ρ_q ≈ 1` on the far sector. Carrier: `|Σ q_a q_b /
  r_ab| ≥ 1e-6` Ha at every far node, asserted nonzero in the sector the plant acts on.
- **(ii) D's nuclei dropped.** In System A, D's nuclei omitted while its density stays, at
  `R_CD = 12` Å. G2 must still pass (D absent is D absent) and S1 must fire: `|Δ| > |r_3(∞)|`.
  Carrier: `nn(C, D) ≥ 1` hartree at 12 Å, asserted nonzero in the sector the plant acts on.

## 5. Provenance and discipline

Runner `holon-chem/examples/embed3_campaign.rs` with arms `hf4` and `water`, JSON per node
under `conformance/water_observatory/embed3/`, resumable; tests
`holon-chem/tests/density_embed.rs` extended with G4 and the subset identity on cheap
fragments; results `EMBED3_RESULTS.md` committed with the runner. Both clocks and the thread
count in every JSON. No number enters from outside the engine.
