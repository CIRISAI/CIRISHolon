# Pre-registration — FIELD-5: exchange in the embedding — the Heitler–London wall harvested from a referee that cannot collapse, the density field outside contact, dispersion from what remains, a bent hydrogen bond predicted forward, and the hydrogen bond re-asked

*Frozen 2026-09-05, committed ALONE, before the instrument existed. Built by the lead (the
instrument) with a delegate on the harvest. FIELD-4 found that the Coulomb-only density
embedding is not a field at contact: with no exchange between the fragments they polarise
into each other unopposed (−24 mHa at 2.5 Å where the exact dimer repels), so the wall over
it is not a single exponential and no wall was harvested. FIELD-4 also measured, at the
hydrogen-bond minimum, that the point charges' missing attraction is mostly penetration and
induction, with correlation the larger part beyond 3.4 Å. This freeze puts exchange where it
belongs — in the referee. The Heitler–London state is the antisymmetrised product of the two
monomers' exact wavefunctions, built on the seam programme's own solver: the dimer's
determinant space over an orbital basis that is the two monomers' orbitals symmetrically
orthogonalised across the fragments, the product state one vector in that space, its energy
one Hamiltonian application. `E_exch(R) = E_HL − E_A0 − E_B0 − E_es`, with `E_es` the
classical interaction of the two ISOLATED monomer densities (EMBED-2's
`classical_interaction`), is first-order exchange in the orthogonalised-orbital convention
— a referee with the Pauli term in it, from which the wall is harvested. Prior art:
Heitler and London 1927; Löwdin 1950 (symmetric orthogonalisation); Jeziorski, Moszynski
and Szalewicz 1994 (the SAPT names for the pieces; the orthogonalised first-order exchange
differs from SAPT's `E^{(10)}_exch` at order `S⁴`, stated and not corrected). Not compared to
any other code's numbers.*

misfits: contacts **M-EMPTY-SECTOR** (the expectation rule keeps its EMPTY branch; a harvest
with a positive prefix under three VOIDs the arms before they run); **M-PLANT-OBS** and
**M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the sector the plant acts on —
§5); **M-CHEAPER-THAN-ITS-PRICE** (the exact solves are priced by FIELD-3's record,
13,176–52,739 core-seconds per node at `1,002,001` determinants; a Heitler–London evaluation
is ONE Hamiltonian application and is priced at one Davidson iteration of that record,
`70–270` core-seconds; a reading returning under a tenth of its price is refused);
**M-EXIT-DISCRIMINATOR** (every exact solve records its exit and iteration count; an
iteration cap is VOID); **M-STALE-INSTRUMENT** (this freeze alone; module, runners, JSON and
results together); **M-VACUOUS-SUCCESS** (the Heitler–London vector's norm and its
determinant count are asserted before its energy is read; every arm reports its frame count
and running drop totals); **M-NULL-MISSTAKE** (retention on the rung-1 lens, unchanged);
**M-FIXED-POINT-TRAJECTORY** (the OFF arms are FIELD-2's, reproduced bit for bit);
**M-UNTESTED-GAP** (terms fit on one orientation; S2 is the one measurement across the gap,
on a geometry no term was fit to); **M-FORMAT-FLOOR** (FIELD-3's and FIELD-4's records are
read at their printed precision, 12 significant digits, eight digits over every floor);
**M-FLOOR-UNSTAKED** (every harvested residual has the `1e-6` hartree reading floor; the
basis-invariance gate's floor is the solver's residual bar); **M-BARE-CHARGE**, **M-HOMOG**,
**M-COND-PROBE**, **M-DEVICE-CLASS** (as FIELD-3). Not contacted: the rest of the registry.

## 0. What is built and measured

**The instrument** (`holon-chem/src/heitler_london.rs`): for two fragments, the monomers'
exact solves (`E_A0`, `E_B0`, their orbitals `C_A`, `C_B` and CI vectors); the dimer's AO basis
(the concatenation the supermolecule uses); the block-diagonal orbital matrix
`C = diag(C_A, C_B)` symmetrically orthogonalised across the fragments,
`C' = C (CᵀSC)^{−1/2}`; the dimer's molecular integrals over `C'` and the determinant space
over `n_A + n_B` orbitals; the Heitler–London vector — on every dimer determinant whose
α-string and β-string each place exactly the monomers' electron counts on their own
orbitals, the product of the two monomer CI coefficients, and zero on every other
determinant (charge transfer excluded by construction) — normalised, its determinant count
recorded; `E_HL = ⟨Ψ|H|Ψ⟩ + E_nuc` by one `sigma`; `E_es` by EMBED-2's classical interaction
of the isolated monomer densities; `E_exch = E_HL − E_A0 − E_B0 − E_es`. Also the dimer's
full CI in the SAME orthogonalised basis (`fci_in_hl_basis`), for the gate below. Plant
(ii) skips the orthogonalisation (the block-diagonal matrix used as if orthonormal).

**The harvest** (`holon-render/examples/field5_harvest.rs`), on FIELD-3's six linear nodes
(their `ΔE_exact`, `E_q` and FIELD-4's `p(R)` reused as frozen records):
- the wall `(A, b)` from `E_exch(R)` by FIELD-3's rule (weighted log-linear fit over the
  largest contiguous set of the shortest nodes, at least three, within `0.10·|ΔE_exact|` at
  each; `R_x` the last);
- the penetration-and-induction term `(P, c)` from FIELD-4's `p(R)` on the four OUTER nodes
  only (2.9, 3.1, 3.4, 3.7 Å — where FIELD-4 showed the density field is a field), the H–O
  placement, the `c`-grid `0.5–4.0` step 0.01 as FIELD-4;
- dispersion `C₆` from the remainder `ΔE_exact − E_q − p_fit − E_exch` on the outer four
  nodes by weighted least squares on `−C₆/R_OO⁶`, its log-log slopes reported, transferred if
  every slope lies in `[−8, −4]`, else recorded as `0` and named.

**The seam law** is FIELD-4's engine unchanged (`SeamModel { a, b, p, c, c6 }`); the arms
runner reads `wall5.json`.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1, on the extended seam law with all four terms: units and `|binding| < 1e-4`
⇒ VOID; `≤ −2 kT` hold; `> −kT` break; between, no expectation; the four parts written
separately.

## 2. Gates

- **G-H0 — the orthogonalised basis is the dimer's basis.** On the 2.9 Å node, the dimer's
  full CI over `C'` reproduces FIELD-3's `e_super` (`−150.04…`, the record) to `1e-8` hartree
  (a full CI energy is invariant under an orthonormal change of orbital basis; the solver's
  residual bar `1e-9` is the floor), `Converged`, `1,002,001` determinants (EXACT). 1 node.
  witness: none (an invariance; a price)
- **G-H1 — the product state is what it says.** On every node: the Heitler–London vector has
  norm `1` to `1e-12`, its nonzero determinants number exactly `n_det(A) · n_det(B)`
  (`441 × 441 = 194,481`), and `E_HL` at 40 bohr (the acceptor moved) equals `E_A0 + E_B0` to
  `1e-8` (no overlap, no exchange, no electrostatics beyond `1e-8` at that separation).
  witness: none (a norm, a count, a limit)
- **H1 — exchange is a wall.** `E_exch(R) > 1e-6` at all six nodes and non-increasing
  outward (EXACT order).
  witness: none (a measured sign and order)
- **S1 — what the wall is.** **(a)** the log-linear fit of `E_exch` lies within
  `0.10·|ΔE_exact|` at all six nodes ⇒ the wall is one exponential over the whole range and is
  transferred in full. **(b)** a prefix of at least three fits, the outer nodes miss ⇒ the
  wall from the prefix, the outer miss reported (a second exponential is NOT fit; the
  remainder is measured). **(c)** under three ⇒ VOID: no wall, the arms do not run, the shape
  of `E_exch` read for FIELD-6.
  witness: none (a fit against a frozen tolerance)
- **G-C1 — the harvest is the engine's arithmetic, same reference on both sides.** With the
  harvested coefficients loaded, the engine's seam-law interaction on each linear node (its
  DIFFERENCE against the acceptor at 40 bohr) equals `[E_q(R) − E_q(40)] + p_HO(R) + wall(R) +
  disp(R)` from the formulas to `1e-10` hartree, where `E_q(40)` is the engine's own field at
  the reference — FIELD-4's letter-failure named and closed. 6 nodes.
  witness: none (arithmetic)
- **S2 — a bent hydrogen bond, predicted forward (rule 6).** The held-out geometry: the
  linear dimer at `R_OO = 2.9` Å with the ACCEPTOR rotated by `30°` about the x-axis through
  its own oxygen (its plane tilted off the O···O axis; the donor and its O–H untouched).
  BEFORE its exact solve, `prediction.json` files the seam law's interaction on it from the
  linear harvest, with its four parts; then the exact solve (`1,002,001` determinants,
  `1,450 ≤ cpu_seconds ≤ 57,600`, `Converged`, residual `≤ 1e-9`); then, after the solve is
  recorded, `E_HL` on the same geometry (cheap) to attribute any miss between the wall and the
  rest. **(a)** `|E_pred − ΔE_exact| ≤ max(0.25·|ΔE_exact|, 5e-4)` ⇒ the seam law transfers
  to a bent bond. **(b)** it misses, and `E_exch(tilted)` from the instrument is within the
  same tolerance of the wall's value there ⇒ the wall transfers and the miss is in the other
  terms, which are named by size. **(c)** it misses and the wall's value misses `E_exch(tilted)`
  too ⇒ the O–O placement of the wall does not transfer to a bent bond; the results say by how
  much.
  witness: none (a prediction filed before its measurement)
- **S3 — retention under the seam law.** FIELD-3's arms as frozen there (dimer, ring; 293 K,
  150 K; OFF reproduced EXACTLY, SEAM with all four terms; 2,000 + 20,000 frames; the rung-1
  lens): **(a)** at 293 K `f_SEAM ≥ 0.5` on both; **(b)** at 150 K only, on both; **(c)**
  neither.
  witness: none (a measured population against a frozen instrument)

## 3. What each outcome means

S1 (a)/(b) is the first wall this programme harvests from a referee that has exchange in it;
with G-C1 it is a seam law every constant of which is derived. S2 (a) is the first transfer
across orientation this programme can claim; (b) localises what does not transfer to a
named channel; (c) says the O–O placement is the limit and FIELD-6 stakes an atom-pair
wall (O–O, O–H, H–H) from `E_exch` on more than one orientation. S3 (a) opens the periodic
liquid on EWALD-1's electrostatics; (b) and (c) are read with the binding at the start.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

Four terms fit on one line, one bent geometry predicted, once. FIELD-4's flipped dimer
(`+6.009` mHa, solved and recorded) is NOT a stake here — its exact value is known — but
`E_HL` on it is a free reading the results report beside the wall's value there.

## 5. Plants

- **(i) The sign of the penetration term.** `P → −P` (the engine's `FlipPenetration`). G-C1
  must fail at 2.9 Å by `2·|p_HO(2.9 Å)|` to `1e-10`. Carrier: `|p_HO(2.9 Å)| ≥ 1e-4`, asserted
  nonzero in the sector the plant acts on.
- **(ii) The orthogonalisation skipped.** The block-diagonal orbitals used as if orthonormal
  on the 2.9 Å node: G-H0's full CI must miss the record by `≥ 1e-2` hartree. Carrier: the
  largest cross-fragment orbital overlap `|(CᵀSC)_{ij}|, i ∈ A, j ∈ B`, `≥ 1e-3`, asserted
  nonzero in the sector the plant acts on (the overlap the orthogonalisation removes).

## 6. Discipline

Module `holon-chem/src/heitler_london.rs` with its unit tests (the norm, the count, the
40-bohr limit on a small pair); runners `holon-render/examples/field5_harvest.rs`
(`exchange` — the six `E_exch` readings, G-H1, the fits, `wall5.json`, G-C1 by the engine,
`prediction.json` BEFORE the tilted solve; `invariance` — G-H0 and plant (ii), one full
solve; `predict` — refuses without `prediction.json`, the tilted exact solve, then `E_HL` on
it, `prediction_check.json`); the arms by `field3_hbonds.rs` reading `wall5.json`; JSON
under `conformance/water_observatory/field5/`; results `FIELD5_RESULTS.md` with the module.
Exact solves detached on the declared thread count. No number enters from outside the
engine and its own solver.
