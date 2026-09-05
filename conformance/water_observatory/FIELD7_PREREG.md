# Pre-registration — FIELD-7: the wall on atom pairs, harvested over orientations; the remainder transferred into the contact; a twisted hydrogen bond predicted forward; and the hydrogen bond re-asked

*Frozen 2026-09-05, committed ALONE, before the orientation harvest ran. Built by the lead
(the engine's atom-pair wall) with a delegate on the harvest. FIELD-6 harvested a physical
wall from the undeformed Heitler–London referee (`+12.1` mHa at the hydrogen-bond minimum,
one exponential on the oxygen–oxygen separation over five nodes) and found the seam law
built from it `10.4` mHa too repulsive there: the exact dimer has, beyond first order, a
remainder — charge transfer and inter-fragment correlation — that decays like overlap, not
like `R⁻⁶`, and no transferred channel carried it. FIELD-5's free reading found exchange
moving by a factor of six with orientation at one oxygen–oxygen separation (the flipped
dimer against the linear one at 3.4 Å): exchange lives on the overlapping contacts, not on
the oxygens, and an O–O wall cannot carry it across orientation. This freeze does both
things the record asks: the wall is placed on the three cross-unit atom-pair classes and
harvested over a declared set of ORIENTATIONS from the cheap referee (one Hamiltonian
application per geometry), and the remainder is transferred into the channel whose shape
it has — one attractive exponential on the hydrogen–oxygen contact, fit over every exact
geometry the record holds with the wall held fixed. The ledger's table names what the
contact term folds (penetration, induction, charge transfer, correlation); the freeze does
not pretend to separate them at this basis.*

misfits: contacts **M-EMPTY-SECTOR**, **M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants,
carriers asserted nonzero in the sector each acts on; plant (ii)'s carrier is a RATIO the
record has measured on the deformed referee at 5.8 and this freeze stakes at ≥ 2 on the
undeformed one), **M-CHEAPER-THAN-ITS-PRICE** (an undeformed reading priced at FIELD-6's
measured 55–60 core-seconds per geometry; the one exact held-out solve at FIELD-3's record
13,176–52,739 core-seconds), **M-EXIT-DISCRIMINATOR**, **M-STALE-INSTRUMENT**,
**M-VACUOUS-SUCCESS** (every reading's norm and count asserted; every arm's frame count and
drop totals), **M-NULL-MISSTAKE**, **M-FIXED-POINT-TRAJECTORY**, **M-UNTESTED-GAP** (the
contact term is fit on nine exact geometries of two kinds — a line and three bends — and
the held-out geometry is a TWIST neither kind contains), **M-FORMAT-FLOOR**,
**M-FLOOR-UNSTAKED** (reading floor `1e-6`; the wall's tolerance derived as FIELD-6's;
the contact term's tolerance stated in §2), **M-BARE-CHARGE**, **M-HOMOG**, **M-COND-PROBE**,
**M-DEVICE-CLASS**. Not contacted: the rest of the registry.

## 0. What is built and measured

**The engine.** `SeamModel` gains the wall on the two further cross-unit classes:
`A_OH·exp(−b_OH·r)` on cross-unit hydrogen–oxygen pairs (beside the contact term) and
`A_HH·exp(−b_HH·r)` on cross-unit hydrogen–hydrogen pairs; each an exact `0.0` off, so
FIELD-6's engine is the identity when they are (checkpoint v9; the derivative, momentum
and books gates re-run over every atom).

**The orientation set** (declared; the acceptor's own oxygen is the pivot, the donor
untouched): `R_OO ∈ {2.7, 2.9, 3.1, 3.4}` Å × acceptor tilt about the x-axis
`∈ {0°, 30°, 60°, 90°, 120°, 180°}` — 24 geometries, the six linear nodes' `R_OO` subset of
them at tilt 0°. On each: the undeformed `E_exch` (norm, count, `sigma_seconds` recorded).

**The wall, harvested over orientations.** The three-class model
`Σ_OO A_OO e^{−b_OO r} + Σ_OH A_OH e^{−b_OH r} + Σ_HH A_HH e^{−b_HH r}` over the cross-unit
pairs of each geometry, fit to the 24 `E_exch` readings: for each `(b_OO, b_OH, b_HH)` on the
grid `0.5–4.0` per bohr, step `0.05` (71³ triples), the weighted linear least-squares
amplitudes (weights `1/E_exch²`; amplitudes constrained non-negative by dropping a class
whose fitted amplitude is negative and refitting the rest); the triple of least weighted
residual. The per-geometry tolerance is FIELD-6's derived one, `max(0.05·E_exch, 1e-4)`
hartree.

**The contact term.** With the wall held at the harvest, `(P, c)` from the remainder
`ΔE_exact − [E_q(R) − E_q(40)] − wall` on every EXACT geometry of record — the six linear
nodes (FIELD-3), the 30°-bent bond at 2.9 Å (FIELD-5), the 45°-bent bond at 3.1 Å (FIELD-6),
the flipped dimer at 3.4 Å (FIELD-4) — nine points; the `c`-grid `0.5–4.0` step `0.01`,
weighted linear `P` per `c`, least weighted residual. Dispersion stays `0` unless the
remainder after the contact term has log-log slopes in `[−8, −4]` on the four outer
linear nodes (FIELD-6's rule).

**The seam law** is then charges + contact term + three-class wall (+ dispersion if
transferred). The arms runner reads `wall7.json`.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1, with the four parts written separately.

## 2. Gates

- **G-E0 — the identity.** With the two new classes at `0.0`, FIELD-6's seam gates and the
  channel receipt unchanged (EXACT bytes and lines).
  witness: none (bytes; the receipt)
- **G-E1 — the terms are the derivative.** Central differences at `h = 1e-4` on every atom
  of the dimer start with every seam term on (the harvested coefficients): relative
  `≤ 1e-8`; momentum and the books as FIELD-4 G-D2.
  witness: none (finite difference; conservation)
- **W0 — the readings are what they say.** Every one of the 24: norm in `(0.8, 1]`,
  `194,481` nonzero determinants, `E_exch > 1e-6`; along each tilt `E_exch` non-increasing
  in `R_OO` (EXACT order).
  witness: none (norm, count, order)
- **S1 — the wall over orientations.** **(a)** all 24 geometries within the derived
  tolerance ⇒ the three-class wall carries exchange across orientation at this level.
  **(b)** at least 18 within, the misses named by geometry ⇒ transferred, the misses
  reported. **(c)** fewer ⇒ VOID: the arms do not run; the misses' pattern read for
  FIELD-8.
  witness: none (a fit against a derived tolerance)
- **C1 — the contact term.** `(P, c)` from the nine exact points; per point
  `|remainder − fit| ≤ max(0.25·|ΔE_exact|, 5e-4)` on at least seven of nine, the misses
  named. Fewer ⇒ the contact term is not one exponential across bends; the arms still run
  (the term is transferred as fit) and S3 is read with that caveat.
  witness: none (a fit against a stated tolerance)
- **G-C1 — the harvest is the engine's arithmetic, one reference.** As FIELD-6, `1e-10`, on
  the six linear nodes and the three bent/flipped geometries.
  witness: none (arithmetic)
- **S2 — a twisted hydrogen bond, predicted forward.** The held-out geometry is of a kind no
  fit point has: the linear dimer at `R_OO = 3.0` Å with the acceptor rotated by `90°` about
  the O···O axis (z) AND tilted by `60°` about its own x-axis (the twist first, then the
  tilt, about the acceptor's oxygen). `prediction.json` BEFORE the solve, with the four
  parts; the exact solve (`1,002,001` determinants, `1,450 ≤ cpu_seconds ≤ 57,600`,
  `Converged`, residual `≤ 1e-9`); then the undeformed `E_exch` on it. **(a)**
  `|E_pred − ΔE_exact| ≤ max(0.25·|ΔE_exact|, 5e-4)`. **(b)** it misses and the wall's value
  there is within the same tolerance of `E_exch(twisted)` ⇒ the wall transfers, the miss is
  the contact term's, named by size. **(c)** both miss.
  witness: none (a prediction filed before its measurement)
- **S3 — retention under the seam law.** As FIELD-5 (dimer, ring; 293 K, 150 K; OFF
  reproduced EXACTLY; SEAM with every term; the rung-1 lens): **(a)**, **(b)**, **(c)** as
  there.
  witness: none (a measured population against a frozen instrument)

## 3. What each outcome means

S1 (a)/(b) with S2 (a) is a seam law whose exchange transfers across orientation, every
constant derived, and S3 (a) opens the periodic liquid on EWALD-1's electrostatics with a
law that has been asked, on a line, three bends, a flip and a twist, whether it is water's.
S3 (b)/(c) with S1/S2 (a) says the remaining channel is the one the contact term folds and
prices the basis for FIELD-8.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

The wall is fit on 24 cheap readings of one family (one pivot, one axis); the contact term
on nine exact points of two kinds; one twist predicted, once.

## 5. Plants

- **(i) The sign of the contact term.** `P → −P` (the engine's `FlipPenetration`): G-C1 must
  fail at the linear 2.9 Å node by `2·|p(2.9 Å)|` to `1e-10`. Carrier `|p(2.9 Å)| ≥ 1e-4`,
  asserted nonzero in the sector the plant acts on.
- **(ii) The wall on the oxygens only.** The O–O-only model fit to the same 24 readings must
  FAIL S1's tolerance on at least 6 of 24 geometries. Carrier: the undeformed
  `E_exch(flipped 3.4 Å) / E_exch(linear 3.4 Å) ≥ 2`, asserted nonzero in the sector the plant
  acts on (the orientation dependence at fixed `R_OO`; the deformed referee read 5.8).

## 6. Discipline

Runner `holon-render/examples/field7_harvest.rs` (`orient` — the 24 readings, W0, the wall
fit, plant (ii), the contact fit, G-C1, `wall7.json`, `prediction.json` BEFORE the twisted
solve; `predict` — the solve, `E_exch` on it, `prediction_check.json`); the arms by
`field3_hbonds.rs` reading `wall7.json`; gates in `tests/seam.rs`; JSON under
`conformance/water_observatory/field7/`; results `FIELD7_RESULTS.md` with the engine
change. No number enters from outside the engine and its own solver.
