# Pre-registration — FIELD-6: the undeformed Heitler–London referee — exchange with the monomers left alone, the wall harvested from it, the ledger's other channels as FIELD-5 fit them, a bent hydrogen bond predicted forward, and the hydrogen bond re-asked

*Frozen 2026-09-05, committed ALONE, before the instrument touched a water node. Built by the
lead (the instrument) with a delegate on the harvest. FIELD-5 built the Heitler–London state
over orbitals symmetrically orthogonalised across the fragments and measured what that
costs: `E_exch` of `+40` mHa at the hydrogen-bond minimum where the exact dimer binds by
`5.5`, an exponent drifting from 1.88 to 2.04 per bohr across the six nodes, and no single
exponential within the staked tenth. The freeze had said the orthogonalised state differs
from first-order exchange at order `S⁴`; it differs at order `S²`, because orthogonalising
the monomers' full orbital sets DEFORMS each monomer's own wavefunction, and that penalty
was most of the number (on a hydrogen-molecule pair at 3 bohr the deformed state's
"exchange" is 3.7 times the undeformed state's). This freeze uses the undeformed state: the
antisymmetrised product of the two monomers' exact wavefunctions in their OWN orbitals,
expanded in the orthonormalised basis through the minors of the fragment overlap's square
root (`C = C'·M^{1/2}`; a determinant on original orbitals `P` is `Σ_Q det(T[Q,P]) |Q⟩`),
contracted with the two CI vectors, and evaluated by one Hamiltonian application, its own
overlap `⟨v|v⟩` reported and divided out. `E_exch = E_HL − E_A0 − E_B0 − E_es` is then the
first-order exchange proper (Heitler–London 1927, in the non-orthogonal sense), the SAPT
`E^{(10)}_exch` of Jeziorski, Moszynski and Szalewicz 1994 without approximation in `S`.*

misfits: contacts **M-EMPTY-SECTOR**, **M-PLANT-OBS**, **M-PLANT-SECTOR**,
**M-CHEAPER-THAN-ITS-PRICE** (the exact held-out solve priced by FIELD-3's record,
13,176–52,739 core-seconds; an undeformed reading priced at FIELD-5's measured 55–59
core-seconds per node plus the contraction, refused under a tenth of it),
**M-EXIT-DISCRIMINATOR**, **M-STALE-INSTRUMENT**, **M-VACUOUS-SUCCESS** (the product's norm,
its determinant count and its 40-bohr limit asserted before its energy is read),
**M-NULL-MISSTAKE**, **M-FIXED-POINT-TRAJECTORY**, **M-UNTESTED-GAP**, **M-FORMAT-FLOOR**,
**M-FLOOR-UNSTAKED** (the reading floor `1e-6` hartree on every residual; the wall's
tolerance is DERIVED below, not typed), **M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**,
**M-DEVICE-CLASS** — all as FIELD-5. Not contacted: the rest of the registry.

## 0. What is built and measured

**The instrument**: `holon_chem::heitler_london::heitler_london_undeformed` (built and
unit-tested on a hydrogen-molecule pair before this freeze: the undeformed energy lies
between the exact and the orthogonalised, its exchange vanishes at 40 bohr, its overlap is
reported). The orthogonalised state of FIELD-5 is kept as the plant.

**The harvest** (`holon-render/examples/field6_harvest.rs`), on FIELD-3's six linear nodes
with `ΔE_exact`, `E_q`, and FIELD-5's penetration fit `(P, c)` on the outer four nodes reused
as frozen records:
- the wall `(A, b)` from the undeformed `E_exch(R)`: the weighted log-linear fit over the
  largest contiguous set of the SHORTEST nodes (at least three) within, at each node,
  `max(0.10·|ΔE_exact(R)|, 0.05·E_exch(R))` — the second arm DERIVED from FIELD-5's measured
  exponent drift (9 % over the range on the deformed referee; a wall whose exponent drifts
  by `δ` across the fit misses one exponential by about `δ/2` at the ends, and half of the
  measured drift is the stake); `R_x` the last node of the set;
- dispersion `C₆` from the remainder `ΔE_exact − E_q − p_fit − E_exch` on the four outer
  nodes by weighted least squares on `−C₆/R_OO⁶`, transferred iff every log-log slope lies
  in `[−8, −4]`, else `0` and named.

**The seam law** is FIELD-4's engine unchanged; the arms runner reads `wall6.json`.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1.

## 2. Gates

- **G-U0 — the undeformed state is what it says.** On every node: its overlap `⟨v|v⟩` lies
  in `(0.8, 1]` and is reported; `E_exact ≤ E_HL(undeformed) ≤ E_HL(orthogonalised)` (EXACT
  order: variational, and the deformation penalty is non-negative); at 40 bohr
  `|E_exch| ≤ 1e-8` AND `|E_HL − E_A0 − E_B0 − E_es| ≤ 1e-8` (FIELD-5's G-H1 restated with the
  electrostatics on the right side of the equation — its letter-failure closed).
  witness: none (an order, a limit)
- **H1 — exchange is a wall.** `E_exch(R) > 1e-6` at all six nodes, non-increasing outward,
  and `E_exch(2.9 Å) < 0.5 · E_exch^{orth}(2.9 Å)` (the deformation penalty removed is at
  least half of FIELD-5's reading; on the hydrogen pair it was 73 %).
  witness: none (a measured sign, order and ratio)
- **S1 — what the wall is.** **(a)** the fit lies within the derived tolerance at all six
  nodes ⇒ one exponential, transferred in full. **(b)** a prefix of at least three fits and
  the outer nodes miss ⇒ the wall from the prefix, the outer miss reported. **(c)** under
  three ⇒ VOID: no wall, the arms do not run, the shape read for FIELD-7.
  witness: none (a fit against a derived tolerance)
- **G-C1 — the harvest is the engine's arithmetic, same reference both sides.** As FIELD-5,
  `1e-10`, 6 nodes.
  witness: none (arithmetic)
- **S2 — a bent hydrogen bond, predicted forward.** The held-out geometry is NEW (FIELD-5's
  tilted node is being solved and will be known): the linear dimer at `R_OO = 3.1` Å with the
  acceptor rotated by `45°` about the x-axis through its own oxygen. `prediction.json` BEFORE
  the solve, with the four parts; the exact solve (`1,002,001` determinants,
  `1,450 ≤ cpu_seconds ≤ 57,600`, `Converged`, residual `≤ 1e-9`); then the undeformed
  `E_exch` on the same geometry. **(a)** `|E_pred − ΔE_exact| ≤ max(0.25·|ΔE_exact|, 5e-4)`.
  **(b)** it misses and the wall's value there is within the same tolerance of
  `E_exch(tilted)` ⇒ the wall transfers, the miss is in the other terms, named by size.
  **(c)** both miss ⇒ the O–O placement does not transfer to a bent bond, by how much.
  witness: none (a prediction filed before its measurement)
- **S3 — retention under the seam law.** As FIELD-5.
  witness: none (a measured population against a frozen instrument)

## 3. What each outcome means

As FIELD-5 §3, on the undeformed referee.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

As FIELD-5 §4. FIELD-5's tilted node and FIELD-4's flipped node, both solved, are free
readings for the wall (their undeformed `E_exch` beside the wall's value), not stakes.

## 5. Plants

- **(i) The sign of the penetration term.** As FIELD-5 (i).
- **(ii) The deformed referee.** FIELD-5's orthogonalised `E_exch` in place of the undeformed
  one: the wall fit must FAIL the derived tolerance on every set of three or more (FIELD-5
  measured every set failing the tenth by 0.14–2.9 of `|ΔE_exact|`, and the derived tolerance
  at 5 % of `E_exch` is at most `2` mHa there). Carrier: `E_exch^{orth}(2.9 Å) −
  E_exch(2.9 Å) ≥ 1e-3` hartree, asserted nonzero in the sector the plant acts on (the
  deformation penalty).

## 6. Discipline

Runner `holon-render/examples/field6_harvest.rs` (`exchange`, `predict`); the arms by
`field3_hbonds.rs` reading `wall6.json`; JSON under `conformance/water_observatory/field6/`;
results `FIELD6_RESULTS.md` with the module. No number enters from outside the engine and
its own solver.
