# Pre-registration — EMBED-2: the field as the partners' own DENSITIES — what the charges mis-carried, staked

*Frozen 2026-09-04, committed ALONE, before `density_embed.rs` existed. Built by the
lead, no lane. SEAM-1's S1′ read branch (b) on measured ground: mutual point-charge
embedding carries 99.93 % of the HF chain's three-body term on the far sector, and the
part it misses decays more slowly than the term itself over 5 → 6 Å (`R^−3.7` against
`R^−6.1`). A two-centre charge model reproduces a fragment's dipole and nothing above
it; what it cannot carry is the fragment's quadrupole and its charge penetration. This
freeze replaces the charges by the partners' embedded DENSITIES — each fragment solved
in the exact Coulomb potential of the others' electrons and nuclei — and stakes whether
the slow component goes away.*

**Prior art, credited.** Embedding a fragment in a frozen partner density is
Wesolowski and Warshel's frozen-density embedding (J. Phys. Chem. 97, 8050, 1993),
whose full form adds a non-additive kinetic and exchange–correlation functional; the
projection-based form is Manby, Stella, Goodpaster and Miller (J. Chem. Theory Comput.
8, 2564, 2012). What is built here is the COULOMB part only — the partner's electrons
and nuclei as a classical potential, no Pauli term — which is the electrostatic
embedding of Dahlke and Truhlar with the point charges replaced by the density. The
honesty machinery around it is ours; the idea is theirs. No runtime is compared.

**Do not read this as chemistry.** STO-3G, rigid geometries, the same carrier as
SEAM-1; the referee energies are SEAM-1's own exact trimers, reused by file.

---

misfits: contacts **M-FLOOR-UNSTAKED** (every clause below that reads the embedded
residual has a measured floor for that residual, by the same translation null, and
nodes below ten times it are VOID for that clause and named — the misfit this
campaign's predecessor registered); **M-ONE-MODEL-DELTA** (the referee is the exact
trimer; the comparison is exact-vs-embedded and the charge field is a second
embedding reported beside it, never the referee); **M-VACUOUS-SUCCESS** (node counts
on every gate; fewer posable nodes than staked is VOID); **M-NULL-MISSTAKE** (the
residual is staked against the bare three-body term of the same node and against
the charge field's residual of the same node); **M-EXIT-DISCRIMINATOR** (three
outcomes named, none the default; a non-converged density iteration is VOID);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the
sector each acts on — §5); **M-STALE-INSTRUMENT** (this freeze alone; runner, JSON and
results document together); **M-CHEAPER-THAN-ITS-PRICE** (this campaign is cheap BY
DESIGN — no trimer is solved, the exact energies are read from SEAM-1's JSON by node
and geometry — and the runner refuses a node whose pinned geometry does not
reproduce SEAM-1's bare pairwise sum to `1e-10`, so a reused referee cannot sit on a
different geometry); **M-DEVICE-CLASS** (host `f64`, `solve_determinant`, one class);
**M-MAX-OVER-SUCCESSES** (S1 is a for-all over posable pairs); **M-FIXED-POINT-TRAJECTORY**
(two starts, one fixed point of the densities — G2); **M-COND-PROBE** (the
phrase "inside the field" appears; the field is a one-electron term of the
Hamiltonian, applied before the solve); **M-HOMOG** and **M-BARE-CHARGE** (the words
"local" and "charge" appear; nothing homogeneous or gauge-charged is meant);
**M-VOLUME-SCALE** (Ewald named as the periodic exit, not built). Not contacted:
the rest of the registry.

## 0. The claim

For fragment A in the field of partners `{B}` with embedded densities `P_B` (AO basis)
and nuclei `Z_b`:

```
h_A^emb  = h_A  +  Σ_B [ V_nuc(Z_b)  +  J[P_B] ]           J[P_B]_μν = Σ_{λσ∈B} P_B,λσ (μν|λσ)
E_A[{B}] = E_elec(h_A^emb)  +  E_nn(A)  +  Σ_B E_nn(A, Z_b)  −  Σ_B Σ_{a∈A} Z_a V_B^el(R_a)
```

— A's electrons attracted by B's nuclei (external centres, as in EMBED-1) and repelled
by B's electrons (the Coulomb matrix of B's density on A's basis), A's nuclei repelled
by B's nuclei and attracted by B's electrons (the partner density's potential at A's
nuclei). Every partner–partner term is EXCLUDED by construction. This is the full
classical electrostatic interaction of A with a frozen B, plus A's polarisation by it.
With every partner density and every partner nucleus removed it is exactly the bare
solve (gate G3); with the partner's density replaced by its point charges it is
EMBED-1's solve.

The densities are iterated to a fixed point (each fragment's embedded density feeds
the others' fields, Gauss–Seidel, largest change in any density-matrix element below
`1e-9`, at most 100 sweeps). The embedded pairwise expansion is SEAM-1's `E_EE-PA`
with density fields in place of charge fields:

```
E_ρ-PA  = Σ_{i<j} E_ij[ρ of the rest] − (N−2) Σ_i E_i[ρ of the others]
r_ρ     = E_ABC − E_ρ-PA            κ_ρ = |r_ρ| / |ΔE_3^bare|
```

The QM–partner terms cancel pairwise between the dimer and monomer sums exactly as
SEAM-1's charge terms did, because the same frozen partner density is used in both.

**S1 — what the charges mis-carried.** On the far sector, `κ_ρ ≤ κ_q` at every node
(the density field carries at least what the charge field did), and on every POSABLE
consecutive pair `κ_ρ` is non-increasing.

## 1. The instrument (`holon-chem/src/density_embed.rs`, new; one hook in `pair.rs`)

- `pair::geometry_problem_with_potential(basis, species, v_extra)`: the assembled
  problem with an additive one-electron term in the AO basis — `J[P_B]` — added to the
  nuclear attraction before the orthonormaliser, the SCF and the transform. Nothing
  else in the chain changes.
- `coulomb_from_partner(basis_A, fragment_B, P_B)`: the combined basis A∪B assembled
  once, its ERIs computed by the existing `ao_integrals`, and the block
  `Σ_{λσ∈B} P_B,λσ (μν|λσ)` returned on A's functions.
- `partner_potential_at_nuclei(fragment_B, P_B, R_a)`: `V_B^el(R_a)` by the
  unit-charge attraction of EMBED-1, contracted with `P_B`.
- `solve_in_densities(fragment, partners: &[(Fragment, P)])`: the energy above, with
  the partners' nuclei as external centres and their densities as `J` and as the
  potential at the fragment's nuclei.
- `embed_densities(fragments, start)`: the fixed point of the densities.
- `rho_pa(fragments, densities)`: the six embedded solves of the expansion.
- The referee: `E_ABC` and `E_PA^bare` read from `seam/hf3_R*.json` by node; the
  translated twin's `E_ABC` from `seam/floor_R*.json`; the runner recomputes the bare
  pairwise sum on its own geometry and REFUSES the node if it differs from the record's
  by more than `1e-10` hartree.

## 2. The system

SEAM-1's carrier unchanged: the HF chain of three at `R_FF ∈ {2.6, 2.8, 3.0, 3.5, 4.0,
5.0, 6.0, 8.0}` Å, monomers at `R_HF = 1.8794379298` bohr; far sector `≥ 5.0` Å (3
nodes), near `≤ 3.0` Å (3 nodes, reported, not read), two transition nodes. The floor
of `r_ρ` at each far node by the translation null (`(0.37, 0.21, 0.5)` bohr), using the
moved trimer energies SEAM-1's floor run banked.

## 3. Gates

Node counts stated on every gate; fewer than staked is VOID.

- **G0 — the referee is the record's.** On every node the runner's own bare pairwise sum
  equals SEAM-1's to `1e-10` hartree, else the node is refused. 8 nodes.
  witness: none (an identity between two runs of one instrument)
- **G1 — the density field's derivative is exact (Hellmann–Feynman on the field).** For
  the HF monomer A at 5.0 Å from a frozen partner B: scale B's density by `λ` and take
  the central difference `(E_A[λ+h] − E_A[λ−h])/2h` at `λ = 1`, `h = 1e-4`; it must
  equal `Σ_μν P_A,μν J[P_B]_μν − Σ_a Z_a V_B^el(R_a)` from A's embedded density to
  `≤ 1e-7` hartree. 1 configuration, and the same at 3.0 Å. 2 points.
  witness: none (Hellmann–Feynman on an exact eigenvector; arithmetic)
- **G2 — one fixed point of the densities.** From zero partner densities and from the
  isolated monomers' densities, the iteration converges within 100 sweeps to the same
  densities (`≤ 1e-8` per element) and the same `E_ρ-PA` (`≤ 1e-10`). 8 nodes.
  witness: none (a measured fixed-point property)
- **G3 — the reduction is exact.** With every partner density and nucleus removed,
  `E_ρ-PA` equals the bare sum to `1e-12` on every node. 8 nodes.
  witness: none (an in-engine identity)
- **G4 — the floor of `r_ρ`, measured.** `floor_ρ(R) = |r_ρ(R) − r_ρ(R, moved)|` at 5, 6,
  8 Å; a node is posable iff `|r_ρ| ≥ 10 · floor_ρ`. 3 nodes.
  witness: none (a measured resolution)
- **S1 — THE DECISION.** On the far sector:
  **(a)** `κ_ρ ≤ κ_q` at every node (`κ_q = 5.07e-4, 7.81e-4, 6.78e-4` from SEAM-1) AND
  `κ_ρ` non-increasing across every posable consecutive pair, with at least one posable
  pair ⇒ the density field carries what the charges mis-carried and its residual
  vanishes at least as fast as the term: the seam is built on density embedding.
  **(b)** `κ_ρ > κ_q` at any node, or a posable pair with `κ_ρ` rising ⇒ the slow
  component is not electrostatic; its scaling is reported, `R^−9` on the chain being
  three-body dispersion, and the next freeze stakes the exchange–correlation or
  dispersion term by name.
  **(c)** every far node UNPOSABLE (`|r_ρ| < 10 · floor_ρ`) ⇒ the residual is below the
  instrument's resolution: the density-embedded expansion closes at two-body on this
  carrier to `3–4e-12` hartree, a stronger statement than (a), written as such and only
  then. Near and transition `κ_ρ` reported under every branch, read under none.
  witness: none (a measured residual against two measured terms)

## 4. What each outcome means

(a) or (c) licenses the water cores inside a DENSITY field under the next freeze, with
(c) additionally setting the resolution any three-body claim on this carrier may use.
(b) says the missing term is not classical electrostatics at all, which is what
Wesolowski's functional and Manby's projector exist for, and the next freeze stakes one
of them with its price. G0 refusing a node removes it and touches no claim.

## 5. Plants

- **(i) The Coulomb sign.** `J[P_B]` enters with the wrong sign (A's electrons attracted
  by B's). G1 must fire: the derivative disagrees by `2 Σ P_A J[P_B]`. Carrier:
  `Σ_μν P_A,μν J[P_B]_μν ≥ 1e-3` hartree at 5.0 Å, asserted nonzero in the sector the
  plant acts on.
- **(ii) The partner's nuclei dropped.** The external centres omitted while `J` stays.
  G3 must still pass (nothing removed twice) and S1 must fire with `κ_ρ > 1` at every
  far node — an unbalanced partner is a bare charge of `−N_B`. Carrier: `E_nn(A, Z_b)`
  at 5.0 Å ≥ `1` hartree, asserted nonzero in the sector the plant acts on.

## 6. Provenance and discipline

Runner `holon-chem/examples/embed2_campaign.rs`, JSON per node under
`conformance/water_observatory/embed2/`, resumable; tests `holon-chem/tests/density_embed.rs`
carry G1, G2, G3 and both plants; results `EMBED2_RESULTS.md` committed with the runner.
Every solve `solve_determinant` on the host, both clocks in every JSON. No number enters
from outside the engine; the referee is the engine's own record, checked by G0.
