# Pre-registration — FIELD-8: data at contact — the wall and the contact term measured where the hole was, the twist family in the wall, the contact term on two classes, a no-hole gate before any arm, a bent-donor bond predicted forward, and the hydrogen bond re-asked with every count read beside its arm's own drift

*Frozen 2026-09-05, committed ALONE, before any close reading or close solve existed. Built
by the lead (the engine, the no-hole gate) with a delegate on the harvest. FIELD-7 held
water's hydrogen bond for 5,900 counted frames with the ledger closed and then lost it to a
hole: the contact term (`−8.97·e^{−1.83 r}` on H···O) and the H–O wall (`8.67·e^{−2.30 r}`),
both fit at `r ≥ 3.4` bohr, sum to an attraction all the way to contact, a cross-unit
contact reached 2.5 bohr, and the pair fused while the lens counted a bond
(M-EXTRAPOLATED-HOLE). Its twisted-bond prediction missed because the contact term, fit on
a line and three bends, over-binds a geometry whose contacts are hydrogen–hydrogen, and
because the wall's own harvest had no twist in it. This freeze puts data where the misses
were: close geometries in the cheap exchange harvest and the twist family beside the tilt
family; close exact nodes on the line so the contact term is measured at contact and not
extrapolated to it; the contact term on two classes; and, before any arm runs, a gate that
walks every cross-unit pair potential of the harvested law inward from 3.0 bohr to contact
and refuses a law that falls. Every retention count is read beside the arm's own drift.*

misfits: contacts **M-EXTRAPOLATED-HOLE** (the no-hole gate G-N0 is this freeze's discharge
of it: the attractive exponential of the contact term is a law only where it is measured,
and the arms are VOID before they run if the summed potentials fall inward);
**M-EMPTY-SECTOR**, **M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants, carriers asserted
nonzero in the sector each acts on), **M-CHEAPER-THAN-ITS-PRICE** (undeformed readings at
FIELD-7's 55–105 core-seconds; exact solves at FIELD-3's record, 13,176–52,739
core-seconds, the close nodes allowed up to the ceiling), **M-EXIT-DISCRIMINATOR**,
**M-STALE-INSTRUMENT**, **M-VACUOUS-SUCCESS** (a retention count on an arm whose drift has
left the honest band is VOID — §2 S3 — the lesson FIELD-7 paid for), **M-NULL-MISSTAKE**,
**M-FIXED-POINT-TRAJECTORY**, **M-UNTESTED-GAP** (the held-out geometry is a bent DONOR, a
kind no fit point contains), **M-FORMAT-FLOOR**, **M-FLOOR-UNSTAKED**, **M-BARE-CHARGE**,
**M-HOMOG**, **M-COND-PROBE**, **M-DEVICE-CLASS**. Not contacted: the rest of the registry.

## 0. What is built and measured

**The engine.** `SeamModel` gains the contact term on the hydrogen–hydrogen class,
`−P_HH·exp(−c_HH·r)` (checkpoint v10), an exact `0.0` off; and a method `hole(q_H)` that
walks each cross-unit class potential — H–O: contact + H–O wall + charges; O–O: wall +
dispersion + charges; H–H: H–H contact + H–H wall + charges — from `3.0` bohr inward to
`0.5` on a `0.05` grid and names the first fall. The arms runner refuses a record `hole()`
names.

**The exchange harvest, extended** (undeformed Heitler–London, one application each):
FIELD-7's 24 (tilt family, `R_OO ∈ {2.7, 2.9, 3.1, 3.4}` × six tilts) reused as records;
NEW: the CLOSE tilt family `R_OO ∈ {2.1, 2.3, 2.5}` × the same six tilts (18), and the TWIST
family — the acceptor rotated `90°` about the O···O axis and then tilted `{0°, 30°, 60°,
90°, 120°, 180°}` about its own x-axis — at `R_OO ∈ {2.3, 2.7, 3.0, 3.4}` (24). 66 geometries.
The three-class wall fit as FIELD-7 (grid `0.5–4.0` step `0.05` per class, weighted
`1/E_exch²`, non-negative by drop-and-refit), tolerance `max(0.05·E_exch, 1e-4)`.

**The exact nodes, extended.** NEW: the linear dimer at `R_OO = 2.3` and `2.1` Å (H···O
`2.35` and `1.97` bohr) — where the exact dimer is repulsive and the contact term must be
measured, not extrapolated. With FIELD-3's six, FIELD-5's 30°-bent, FIELD-6's 45°-bent,
FIELD-4's flipped and FIELD-7's twisted: twelve exact geometries.

**The contact term on two classes.** With the wall held at the harvest, the remainder
`ΔE_exact − [E_q − E_q(40)] − wall` on the twelve exact geometries fit as
`−P_HO·Σ_{H–O} e^{−c_HO r} − P_HH·Σ_{H–H} e^{−c_HH r}`: for each `(c_HO, c_HH)` on the grid
`0.5–4.0` step `0.02` (176² pairs) the weighted linear least-squares amplitudes (weights
`1/ΔE_exact²`), non-negative by drop-and-refit; the pair of least weighted residual.
Dispersion `C₆` by FIELD-6's rule on the four outer linear nodes after the contact terms
(expected `0`; recorded).

**The arms** read `wall8.json`; the OFF arms are FIELD-2's.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1, with every part written separately.

## 2. Gates

- **G-F0 — the identity.** With the new class at `0.0`, FIELD-7's seam gates and the channel
  receipt unchanged (EXACT).
  witness: none (bytes; the receipt)
- **G-F1 — the terms are the derivative.** Every atom, every seam term on, `h = 1e-5`,
  relative `≤ 1e-8`; momentum and the books as FIELD-4 G-D2 (the dynamics legs on the
  harvested law only if G-N0 admits it, else on declared coefficients, said so).
  witness: none (finite difference; conservation)
- **W0 — the readings are what they say.** Every one of the 66: overlap in `(0.8, 1]`,
  `E_exch > 1e-6`, and along each (family, tilt) `E_exch` non-increasing in `R_OO` (EXACT
  order). The count leg of FIELD-7's W0 is retired: the undeformed state is nonzero on the
  full space by construction.
  witness: none (norm, order)
- **S1 — the wall over 66 orientations.** **(a)** all 66 within tolerance. **(b)** at least 53
  (80 %) within, the misses named by (family, R, tilt). **(c)** fewer ⇒ VOID: the arms do not
  run.
  witness: none (a fit against a derived tolerance)
- **C1 — the contact terms.** Per exact point `|remainder − fit| ≤ max(0.25·|ΔE_exact|,
  5e-4)` on at least 10 of 12, the misses named; and the CLOSE nodes both within it (the
  term is measured at contact or the campaign says it is not).
  witness: none (a fit against a stated tolerance)
- **G-N0 — no hole.** `hole(q_H)` on the harvested law returns none: every cross-unit class
  potential rises monotonically inward from `3.0` bohr to `0.5` (EXACT order on the grid).
  If it names a fall, the harvest is recorded, the fall is named with its class and radius,
  and the arms are VOID before they run.
  witness: none (an order on a grid)
- **G-C1 — the harvest is the engine's arithmetic, one reference.** As FIELD-7, `1e-10`, on
  all twelve exact geometries.
  witness: none (arithmetic)
- **S2 — a bent donor, predicted forward.** The held-out geometry is of a kind no fit point
  has: the linear dimer at `R_OO = 2.9` Å with the DONOR rotated by `30°` about the x-axis
  through its own oxygen (its O–H swung off the O···O axis; the acceptor untouched).
  `prediction.json` BEFORE the solve with every part; the exact solve (`1,002,001`
  determinants, `1,450 ≤ cpu_seconds ≤ 57,600`, `Converged`, residual `≤ 1e-9`); then the
  undeformed `E_exch` on it. **(a)** `|E_pred − ΔE_exact| ≤ max(0.25·|ΔE_exact|, 5e-4)`.
  **(b)** it misses and the wall's value is within that tolerance of `E_exch(bent donor)`.
  **(c)** both miss.
  witness: none (a prediction filed before its measurement)
- **S3 — retention, read beside the drift.** FIELD-3's arms (dimer, ring; 293 K, 150 K; OFF
  reproduced EXACTLY; SEAM with every term). An arm is VOID if its honest drift peak exceeds
  `100×` the matching OFF arm's, or its mean temperature is more than `30 %` from its
  target, or its unit count at the last frame is under the staked count; `f` is read only on
  non-VOID arms. **(a)** at 293 K `f_SEAM ≥ 0.5` on both non-VOID; **(b)** at 150 K only;
  **(c)** neither; **VOID** if a needed arm is VOID.
  witness: none (a measured population against a frozen instrument, with its own drift)

## 3. What each outcome means

G-N0 admitting the law is the campaign's first result; S3 (a) with S2 (a) is a seam law
that holds water to water at room temperature for reasons the engine derived, on a line,
bends of both kinds, a flip and a twist — the periodic liquid opens on EWALD-1's
electrostatics. G-N0 refusing says data at 1.97 bohr was not enough and names where the
law still falls; FIELD-9 would put a node there.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

66 cheap readings of two families and twelve exact points of five kinds; one bent donor
predicted once. FIELD-7's arm-length reading (5,900 frames held) is a free comparison for
S3, not a stake.

## 5. Plants

- **(i) The sign of the H–O contact term.** `P_HO → −P_HO` (the engine's `FlipPenetration`):
  G-C1 must fail at the linear 2.9 Å node by `2·|p_HO(2.9 Å)|` to `1e-10`; carrier
  `|p_HO(2.9 Å)| ≥ 1e-4`, asserted nonzero in the sector the plant acts on.
- **(ii) The far-data wall.** FIELD-7's three-class wall (fit on the 24 far readings) must
  MISS the 18 close readings: on at least 9 of 18, `|wall − E_exch| > 0.05·E_exch`. Carrier:
  `E_exch` at `(2.1 Å, 0°) ≥ 0.1` hartree, asserted nonzero in the sector the plant acts on
  (exchange at contact).

## 6. Discipline

Runner `holon-render/examples/field8_harvest.rs` (`close` — the 42 new readings and the
two close exact solves, detached; `fit` — W0, S1, plant (ii), the contact terms, C1, G-N0,
G-C1, `wall8.json`, `prediction.json` BEFORE the bent-donor solve; `predict` — the solve,
`E_exch` on it, `prediction_check.json`); the arms by `field3_hbonds.rs` reading
`wall8.json` and refusing a hole; gates in `tests/seam.rs`; JSON under
`conformance/water_observatory/field8/`; results `FIELD8_RESULTS.md` with the engine change.
No number enters from outside the engine and its own solver.
