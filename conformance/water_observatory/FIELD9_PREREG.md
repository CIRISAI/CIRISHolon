# Pre-registration — FIELD-9: the law where the dynamics go — the wall fit on the range a liquid visits with contact kept for boundedness, the exponent's drift measured and declared as the tolerance, no node past the identity, a doubly bent bond predicted forward, and the hydrogen bond re-asked

*Frozen 2026-09-05, committed ALONE, before any fit ran. Built by the lead (the engine's
boundedness gate) with a delegate on the harvest. FIELD-8 put data at contact and read three
things: one exponent per class cannot span 2.1–3.4 Å within a twentieth, because exchange
steepens inward; the closure identity has a boundary near H···O 1.95 bohr, past which both
units dissolve and the seam law is not defined; and a law whose every class potential is
bounded below and rises to contact is not a hole even when it dips by a fraction of `kT`
first — FIELD-8's law never collapsed, it dissociated, because at the hydrogen-bond minimum
it was a quarter too weak. This freeze fits the wall where the dynamics go and keeps the
close readings for one purpose, boundedness; it measures the exponent's drift across the
two families and DECLARES that as the wall's tolerance instead of typing one; it fits the
contact terms only on exact geometries inside the identity; and it re-asks the bond.
Every reading it uses exists (FIELD-7's and FIELD-8's 66 exchange readings; twelve exact
geometries inside the identity: the line at 2.3–3.7 Å, the 30°- and 45°-bent acceptors, the
flipped and twisted dimers, and the 30°-bent donor); the new solve is the held-out one.*

misfits: contacts **M-EXTRAPOLATED-HOLE** (discharged by the boundedness gate G-B0 below,
which walks every class potential to contact and refuses a fall below its shortest fit
point by more than `kT` or a value at contact under zero); **M-EMPTY-SECTOR** (the
expectation rule's EMPTY branch; an arm whose unit count leaves the staked value is VOID);
**M-PLANT-OBS** and **M-PLANT-SECTOR** (two plants, carriers asserted nonzero in the sector
each acts on); **M-CHEAPER-THAN-ITS-PRICE** (one exact solve at FIELD-3's record; no new
readings); **M-EXIT-DISCRIMINATOR**; **M-STALE-INSTRUMENT**; **M-VACUOUS-SUCCESS** (every arm
reports its units on every pass and its drop totals; a retention count on an arm whose
drift has left the honest band is VOID); **M-NULL-MISSTAKE**; **M-FIXED-POINT-TRAJECTORY**;
**M-UNTESTED-GAP** (the held-out geometry bends BOTH monomers, a kind no fit point has);
**M-FORMAT-FLOOR**; **M-FLOOR-UNSTAKED** (the wall's tolerance is derived in §0 from the
readings, not typed; the reading floor `1e-6`); **M-BARE-CHARGE**, **M-HOMOG**,
**M-COND-PROBE**, **M-DEVICE-CLASS**. Not contacted: the rest of the registry.

## 0. What is built and measured

**The engine.** `SeamModel::bounded(q_H, r_min, kT)`: for each cross-unit class potential
(H–O: contact + wall + charges; O–O: wall + dispersion + charges; H–H: contact + wall +
charges), walked from `r_min` inward to `0.5` bohr on a `0.05` grid, the minimum along the
walk must be no lower than the value at `r_min` by more than `kT` (`9.28e-4` hartree at
293 K), and the value at `0.5` bohr must be positive; the first violation is named with its
class, radius and depth. The arms runner refuses a record `bounded` names. `hole` (the
monotone walk) stays as a reading, not a gate.

**The wall, fit where the dynamics go.** The 48 readings at `R_OO ≥ 2.5` Å — the tilt family
at 2.5, 2.7, 2.9, 3.1, 3.4 Å (30) and the twist family at 2.7, 3.0, 3.4 Å (18) — fit by
FIELD-7's three-class procedure (grid `0.5–4.0` step `0.05` per class, weights `1/E_exch²`,
non-negative by drop-and-refit). The 18 close readings (2.1, 2.3 Å) are NOT fit; they enter
G-B0 only.

**The tolerance, derived.** Per class, before the three-class fit: the log-linear exponent of
`E_exch` against `R_OO` fit on the tilt family alone and on the twist family alone (at tilt
0°, the two families' 0° rows differ by under a percent, so the families' exponents differ
by orientation, not by construction); the wall's per-reading tolerance is
`max(δ/2 · E_exch, 1e-4)` with `δ` the larger relative spread of the exponent between the
two families over the three classes, written to the record before the fit. (FIELD-5
measured a 9 % drift on one family; FIELD-8's readings will say what it is across two.)

**The contact terms, inside the identity.** With the wall held: the two-class fit (FIELD-8's
procedure) of `ΔE_exact − [E_q − E_q(40)] − wall` on the twelve exact geometries the engine
serves with two units — the line at 2.3, 2.5, 2.7, 2.9, 3.1, 3.4, 3.7 Å, the bent acceptors
(30° at 2.9, 45° at 3.1), the flipped dimer (3.4), the twisted dimer (3.0), the bent donor
(30° at 2.9). The 2.1 Å node is outside the identity and is NOT a fit point. Tolerance per
point `max(0.25·|ΔE_exact|, 5e-4)`. Dispersion by FIELD-6's rule on the outer linear nodes.

**The seam law** is FIELD-8's engine; the arms read `wall9.json`.

## 1. The expectation, written before the arms (M-EMPTY-SECTOR discharged)

As FIELD-4 §1, every part written separately.

## 2. Gates

- **G-G0 — the identity.** The boundedness gate at rest on FIELD-8's seam gates (7/7) and
  the receipt (EXACT).
  witness: none (bytes; the receipt)
- **D0 — the drift, declared.** The two families' exponents per class and their spread `δ`
  are written to `wall9.json` BEFORE the wall fit, with the resulting tolerance rule.
  witness: none (a measurement, then a rule)
- **S1 — the wall where the dynamics go.** **(a)** all 48 within the derived tolerance.
  **(b)** at least 38 (80 %) within, misses named. **(c)** fewer ⇒ VOID: the arms do not run.
  witness: none (a fit against a derived tolerance)
- **G-B0 — bounded.** `bounded(q_H, r_min, kT)` on the harvested law with `r_min` the
  shortest cross-unit distance of each class among the 48 fit readings (the O–O at 2.5 Å,
  the H–O and H–H contacts the geometries make there): returns none. Else the fall is named
  and the arms are VOID before they run.
  witness: none (an order on a grid, with a depth)
- **C1 — the contact terms.** On the twelve: within tolerance on at least 10, the misses
  named; the line at 2.7, 2.9 and 3.1 Å (where the bond lives) all within, else the campaign
  says the minimum is still missed and by how much.
  witness: none (a fit against a stated tolerance)
- **G-C1 — the harvest is the engine's arithmetic, one reference.** `1e-10` on the twelve,
  each served with two units (EXACT count).
  witness: none (arithmetic)
- **S2 — a doubly bent bond, predicted forward.** The held-out geometry: the linear dimer at
  `R_OO = 2.9` Å with the DONOR rotated `30°` about the x-axis through its oxygen AND the
  acceptor rotated `30°` about the x-axis through its own — both monomers bent, the kind no
  fit point has. `prediction.json` BEFORE the solve with every part; the exact solve
  (`1,002,001` determinants, `1,450 ≤ cpu_seconds ≤ 57,600`, `Converged`, residual `≤ 1e-9`);
  then the undeformed `E_exch` on it. **(a)** `|E_pred − ΔE_exact| ≤ max(0.25·|ΔE_exact|,
  5e-4)`. **(b)** it misses and the wall is within that tolerance of `E_exch`. **(c)** both.
  witness: none (a prediction filed before its measurement)
- **S3 — retention, read beside the drift.** As FIELD-8 §2 S3 (dimer, ring; 293 K, 150 K;
  an arm VOID if its drift peak exceeds `100×` the OFF arm's, its mean temperature is more
  than `30 %` from target, or its unit count leaves the staked value): **(a)** at 293 K
  `f_SEAM ≥ 0.5` on both; **(b)** at 150 K only; **(c)** neither; **VOID** if a needed arm is.
  witness: none (a measured population against a frozen instrument, with its own drift)

## 3. What each outcome means

S3 (a) with G-B0 admitting is the law the liquid waits on (LIQUID-1 runs on it). S1 or C1
(c) says the exponential shape does not serve even the dynamics' range at the drift-derived
tolerance, and FIELD-10 stakes a second exponential per class. C1 missing the 2.7–3.1 Å line
says the remainder's shape at the minimum is the open problem, and names its size.

## 4. The gap this crosses, named (M-UNTESTED-GAP)

48 readings of two families, twelve exact points of five kinds, one doubly bent bond
predicted once.

## 5. Plants

- **(i) The sign of the H–O contact term.** `P_HO → −P_HO`: G-C1 must fail at the linear
  2.9 Å node by `2·|p_HO(2.9 Å)|` to `1e-10`; carrier `|p_HO(2.9 Å)| ≥ 1e-4`, asserted nonzero
  in the sector the plant acts on.
- **(ii) The typed tolerance.** The same 48-reading wall fit judged at FIELD-8's typed
  twentieth (`0.05·E_exch`) must place at least 5 more readings outside tolerance than the
  derived rule does; carrier `δ/2 ≥ 0.06` (the derived tolerance wider than the typed one),
  asserted nonzero in the sector the plant acts on (the tolerance itself).

## 6. Discipline

Runner `holon-render/examples/field9_harvest.rs` (`fit` — D0, the wall, S1, G-B0, the
contact terms, C1, G-C1, plants, `wall9.json`, `prediction.json` BEFORE the solve; `predict`
— the doubly bent solve, `E_exch` on it, `prediction_check.json`); the arms by
`field3_hbonds.rs` reading `wall9.json` and refusing an unbounded record; gates in
`tests/seam.rs`; JSON under `conformance/water_observatory/field9/`; results
`FIELD9_RESULTS.md` with the engine change. No number enters from outside the engine and
its own solver.
