# Pre-registration — CT-1: the sixth channel — charge transfer measured as the energy the closures gain by opening, the ledger's table extended by one row, the seam law given a term for it, and the remainder read again

*Frozen 2026-09-05, committed ALONE, before the instrument existed. Built by the lead (the
instrument, the ledger row) with delegates on the harvest and on the sizing of channels 3
and 4. FIELD-6 measured, at the hydrogen-bond minimum, `−10.4` mHa the exact dimer has and
the undeformed Heitler–London product does not: everything beyond first order — the
monomers' mutual polarisation, their correlation, and electrons crossing between them. The
ledger's five channels are all things two closures do while staying closed; electrons
crossing is not one of them. It is the identity's SOFT edge, the smooth precursor of the
hard one FIELD-8 located at 1.95 bohr where a hydrogen changes hands. This freeze measures
it by itself. The dimer's determinant space over the orthogonalised monomer orbitals (the
basis FIELD-5 built and gated) splits by how many electrons each fragment holds; the sector
with exactly the monomers' counts on each is the CLOSED sector, and a full CI restricted to
it — every determinant that moves an electron across masked out of the Hamiltonian
application, the Davidson started from the undeformed product state, which lies in that
sector — is the best the dimer can do without charge transfer. Then

    E_CT(R) = E_exact(R) − E_noCT(R)

is the charge-transfer energy in the orthogonalised-orbital convention (block-localised
wavefunctions: Mo, Gao and Peyerimhoff 2000; absolutely localised molecular orbitals:
Khaliullin, Bell and Head-Gordon 2007; its basis dependence: Stone and Misquitta 2009 —
credited, not compared), and `E_noCT − E_HL(undeformed)` is what the closed sector gains by
polarising and correlating without opening: induction plus the inter-fragment correlation
this basis carries (channels 2 and 3 together, at this level). Three numbers where FIELD-6
had one.*

misfits: contacts **M-EMPTY-SECTOR** (the expectation rule keeps its EMPTY branch; a
harvest whose CT is under the floor at every node VOIDs its term); **M-PLANT-OBS** and
**M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the sector each acts on);
**M-CHEAPER-THAN-ITS-PRICE** (a sector solve is priced at FIELD-5's measured 55–105
core-seconds per Hamiltonian application times its iteration count, recorded; a solve
returning under a tenth of its own count times the floor is refused);
**M-EXIT-DISCRIMINATOR** (every sector solve records its exit and iteration count; an
iteration cap is VOID); **M-STALE-INSTRUMENT**; **M-VACUOUS-SUCCESS** (every sector solve
asserts its vector stayed in the sector — the out-of-sector norm — before its energy is
read); **M-NULL-MISSTAKE**; **M-FIXED-POINT-TRAJECTORY**; **M-UNTESTED-GAP** (the CT term is
fit on the twelve geometries of record and tested on a held-out one); **M-FORMAT-FLOOR**;
**M-FLOOR-UNSTAKED** (the CT floor `1e-6` hartree; the sector solve's residual bar the
solver's own); **M-EXTRAPOLATED-HOLE** (the seam law with the CT term passes G-B0 before
any arm); **M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**, **M-DEVICE-CLASS**;
**M-VOLUME-SCALE** (contacted by keyword: the sector is a subspace of the determinant
lattice, its size a count the record carries, not a grid). Not contacted: the rest of the
registry.

## 0. What is built and measured

**The instrument** (`holon-chem/src/heitler_london.rs`): `fci_no_charge_transfer(a, b)` —
the orthogonalised monomer-orbital basis of `setup`; a masked Hamiltonian application (a
`SigmaOp` that applies the lane kernel and zeroes every determinant whose α- or β-string
does not place exactly the monomers' electron counts on their own orbitals); the Davidson
of `solve_determinant_with` on it, started from the undeformed product vector; returns
`E_noCT`, the solution, the sector's determinant count (`441² = 194,481` on the water dimer)
and the out-of-sector norm of the converged vector.

**The readings**, on the twelve exact geometries inside the identity (FIELD-9's list) and
at 40 bohr: `E_exact` (of record), `E_noCT` (new), `E_HL` (undeformed, of record where it
exists, else taken), so that per geometry

    E_exact = E_HL + [E_noCT − E_HL] + E_CT
             = (electrostatics + exchange) + (closed-sector polarisation and correlation) + (charge transfer).

**The ledger.** `channel.rs` gains a sixth record — kind `Identity` (the closure's edge),
arity 2, rate exponential (an overlap-driven quantity), shape `Sum`, prior art as above —
and `Row::Seam` carries it whole. The five are not renumbered; the sixth is appended, with
its own kill in the record's header: a CT term harvested at one basis that vanishes at a
larger one was the basis's, not the seam's (Stone–Misquitta's warning, carried).

**The seam law.** `SeamModel` gains `−P_CT·exp(−c_CT·r)` on cross-unit H–O pairs — the
donor's O–H σ* against the acceptor's lone pair is the contact — an exact `0.0` off
(checkpoint v11). The harvest, in order: (1) the CT term `(P_CT, c_CT)` from `E_CT` on the
twelve by the c-grid rule (`0.5–4.0` step `0.02`, weights `1/max(|ΔE_exact|, 5e-3)²` — the
floor added because FIELD-7/8/9's `1/ΔE_exact²` blew up at the 2.5 Å zero crossing and pinned
every contact fit there; recorded as this freeze's correction of the rule); (2) the contact
term re-fit on what remains, `ΔE_exact − [E_q − E_q(40)] − wall − CT`, same rule, the wall
FIELD-9's; (3) dispersion by FIELD-6's rule.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1, with the CT part written separately.

## 2. Gates

- **T0 — the sector is what it says.** On every geometry: the converged vector's
  out-of-sector norm `≤ 1e-12`; the sector's count `194,481` (EXACT); `E_exact ≤ E_noCT ≤
  E_HL(undeformed)` (variational order, EXACT within `1e-10`); at 40 bohr `|E_CT| ≤ 1e-8`.
  The solve `Converged` with residual `≤ 1e-9`.
  witness: none (a norm, a count, an order, a limit)
- **T1 — charge transfer is attractive and short-ranged.** `E_CT < −1e-6` on the line at
  2.3–3.4 Å and non-increasing in magnitude outward along the line (EXACT order); its
  log-log slope between consecutive linear nodes beyond 2.9 Å steeper than `−6`.
  witness: none (a sign, an order, a slope)
- **T2 — the decomposition, read.** At the linear 2.9 Å node the three parts are written
  beside FIELD-6's `−10.4` remainder; their sum reproduces `E_exact − E_HL` to `1e-10` (an
  identity, EXACT).
  witness: none (arithmetic)
- **S1 — the CT term.** The c-grid fit within `max(0.25·|ΔE_exact|, 5e-4)` at every node
  ⇒ **(a)**; on at least 9 of 12 ⇒ **(b)**, misses named; fewer ⇒ **(c)**, the term not
  transferred, the shape read.
  witness: none (a fit against a stated tolerance)
- **C1 — the remainder after CT.** The contact term re-fit on the eleven-plus-one within
  the same tolerance on at least 10 of 12 AND the line at 2.7, 2.9, 3.1 Å within.
  witness: none (a fit against a stated tolerance)
- **G-B0 — bounded.** FIELD-9's rule on the full law (CT term in).
  witness: none (an order on a grid, with a depth)
- **G-C1 — the engine's arithmetic, one reference.** `1e-10` on the twelve with the CT term
  in the formula.
  witness: none (arithmetic)
- **S2 — predicted forward.** The held-out geometry of FIELD-9 (the doubly bent bond at
  2.9 Å, solved by FIELD-9 — its exact value is KNOWN, so it is NOT the stake) is replaced
  by a new one: the linear dimer at `R_OO = 3.2` Å with the acceptor twisted `90°` about the
  O···O axis and the donor bent `20°` about its own x-axis. `prediction.json` before the
  solve; the solve; then `E_noCT` and `E_HL` on it, so the CT term's transfer is read by
  itself: **(a)** the total within tolerance; **(b)** the total misses but the CT term is
  within `max(0.25·|E_CT|, 2e-4)` of the measured `E_CT` there; **(c)** both.
  witness: none (a prediction filed before its measurement)
- **S3 — retention, read beside the drift.** As FIELD-9.
  witness: none (a measured population against a frozen instrument, with its own drift)

## 3. What each outcome means

T0–T2 passing is the measurement Eric asked for: electron transfer between two closures,
sized by itself, on the engine's own solver, in the ledger as its own row. S1 (a)/(b) with
S3 (a) is a seam law with six channels and every constant derived, and the liquid runs on
it. S1 (c) says charge transfer at this basis is not one exponential on the contact and
names its shape.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

Twelve geometries of five kinds; one twisted-and-bent bond predicted once. Channels 3 and 4
are sized by delegates in this campaign's results and are not stakes here: the water
trimer's exact space is `C(21,15)² ≈ 3·10⁹` determinants, beyond this solver, so channel 4
for water is priced through the density embedding's three-body residual (EMBED-3's
instrument) and named, not harvested.

## 5. Plants

- **(i) The mask dropped.** `fci_no_charge_transfer` with the mask off (the full space,
  started from the product state): its energy must equal `E_exact` of record to `1e-8` on
  the 2.9 Å node — the sector restriction is the whole of `E_CT`. Carrier: `|E_CT(2.9 Å)| ≥
  1e-3`, asserted nonzero in the sector the plant acts on.
- **(ii) The sign of the CT term.** `P_CT → −P_CT` in the engine: G-C1 must fail at 2.9 Å by
  `2·|CT(2.9 Å)|` to `1e-10`; carrier `|CT_term(2.9 Å)| ≥ 1e-4`, asserted nonzero in the
  sector the plant acts on.

## 6. Discipline

Module additions in `heitler_london.rs` (the masked operator, the sector solve, unit tests
on a hydrogen-molecule pair: the order, the mask's identity with the full solve when off,
the 40-bohr limit); `channel.rs` (the sixth record); `seam.rs`/`sim.rs`/`checkpoint.rs`
(the CT term, v11); runner `holon-render/examples/ct1_harvest.rs` (`sector` — the twelve
sector solves and the 40-bohr one, detached; `fit` — T0–T2, S1, C1, G-B0, G-C1, plants,
`wall_ct.json`, `prediction.json`; `predict`); the arms by `field3_hbonds.rs` reading
`wall_ct.json`; results `CT1_RESULTS.md` with the instruments. No number enters from
outside the engine and its own solver.
