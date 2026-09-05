# FIELD-9 — results

*Freeze `FIELD9_PREREG.md` (29e525b, alone). Engine: `SeamModel::bounded(q_H, r_min, kT)` — the
boundedness walk that replaces FIELD-8's monotone letter; the arms runner refuses a record
`bounded` names; gates 7/7 seam, 7/7 ledger, the receipt bit-for-bit. Harvest
`examples/field9_harvest.rs` (delegate, to the freeze; `predict` launched by the delegate).
JSON under `field9/`. No new exchange readings: FIELD-7's and FIELD-8's 66 are the data; one
new exact solve, the held-out doubly bent bond.*

## The verdict, first

**The wall where the dynamics go is transferred, the doubly bent bond confirms the wall for
the third campaign running, and the contact term cannot be both strong enough at the
hydrogen-bond minimum and bounded inside its fit floor: G-B0 REFUSES the harvested law and
the arms are VOID before they run.**

1. **The drift is a quarter, and it is the tolerance (D0).** Across the tilt and twist
   families the single-class exponent of exchange against `R_OO` spreads from `1.68` to
   `2.20` per bohr, a relative spread `δ = 0.2592`; the wall's tolerance is therefore
   `max(0.1296·E_exch, 1e-4)`, declared before the fit. At FIELD-8's typed twentieth the same
   fit would have placed 21 readings outside instead of 2 (plant (ii) fires): the twentieth
   was a number nobody had measured.
2. **The wall on the dynamics' range fits 46 of 48 (S1 (b)).** `A_OO = 948.0, b = 2.40`;
   `A_OH = 22.59, b = 2.20`; `A_HH = 1.525, b = 1.75`; the two misses are the twist family's
   2.7 Å 120° reading (16 % over) and its 3.4 Å 60° one. FIELD-8's finding stands in the
   exponents: fit at `R_OO ≥ 2.5` Å the O–O exponent is 2.40 where the contact-spanning fit
   gave 2.20, and the amplitude 948 where it gave 388.
3. **The contact term at the minimum is the open problem, and it is now bracketed from both
   sides.** With the wall held, the two-class fit on the twelve exact geometries inside the
   identity gives `P_HO = 16.26, c = 2.06` (the H–H class carries nothing, `P_HH = 0`) and is
   within tolerance on 5 of 12 (C1 FAIL): the line at 2.7, 2.9, 3.1 Å is 1.7, 1.9, 1.4 mHa too
   weak, every bend and the flip 2.6–3.1 mHa too weak. And that term, already too weak, is
   already too deep inside its data: the H–O class potential (contact + wall + charges) falls
   from `−41.6` mHa at the fit floor `r_min = 2.78` bohr to a minimum of `−45.6` mHa at
   `2.33` bohr (`4.0` mHa, `4.3 kT` below the floor) before rising to `+1.50` Ha at 0.5 bohr.
   Bounded, positive at contact, and a well at H···O 1.23 Å that a liquid would fall into.
   G-B0 names it and the arms do not run.
4. **The doubly bent bond binds by `−3.14` mHa, and the wall transfers again (S2 (b)).** The
   prediction filed before the solve was `−0.78` mHa; the miss is 75 %, all of it the contact
   term's, while the three-class wall lands within `0.22` mHa of the undeformed exchange on
   that geometry (`6.35` vs `6.13`, tolerance `0.79`). Bending the acceptor as well as the
   donor gains `0.11` mHa over the bent donor alone (`−3.03`); the line's `−5.48` is not
   recovered.

CT-1 (frozen b8fa5e8, amended 2094597) measures the remainder's largest missing piece by
itself: the energy the exact dimer has and the closed sector of the two monomers' own
determinants does not — charge transfer, the ledger's sixth channel — and gives the seam law
a term for it before the contact term is fit again.

| gate | verdict | the number |
|---|---|---|
| G-G0 — the identity | **PASS** | seam 7/7 and ledger 7/7 with `bounded` in the engine; `tests/data/channel_ledger.receipt` reproduced bit-for-bit |
| D0 — the drift, declared | **RECORDED before the fit** | single-class exponents 1.682–2.200 per bohr over 10 (family, tilt) columns, `δ_single = 0.2592`; per-class spread over the three classes kept in both families `δ_class = 0.1474`; `δ = 0.2592` → tolerance `max(0.1296·E_exch, 1e-4)` (`d0.json`) |
| S1 — the wall where the dynamics go | **BRANCH (b)** | 46 of 48 within (stake ≥ 38); `A_OO = 948.05, b_OO = 2.40`; `A_OH = 22.586, b_OH = 2.20`; `A_HH = 1.525, b_HH = 1.75`; misses: twist 2.7 Å 120° (model `0.2005` vs `0.1726`), twist 3.4 Å 60° |
| plant (ii) — the typed tolerance | **FIRES** | the same fit at `0.05·E_exch`: 21 outside against 2 (19 more; stake ≥ 5); carrier `δ/2 = 0.1296 ≥ 0.06` |
| G-B0 — bounded | **REFUSED, the arms VOID** | verbatim: `H–O potential falls to -4.2908e-2 at r = 2.68 bohr, more than kT below its value -4.1621e-2 at its fit floor r_min = 2.781`; `r_min` O–O 4.724315, H–O 2.780741, H–H 1.314606 bohr (the 48 fit geometries' shortest of each class); the walk's full shape (§2): minimum `−45.6` mHa at 2.33 bohr, `+1.50` Ha at 0.5 bohr. The monotone reading beside it: a fall at 2.95 bohr |
| C1 — the contact terms | **FAIL** | `P_HO = 16.261, c_HO = 2.06`; `P_HH = 0` (dropped at the grid floor); 5 of 12 within `max(0.25·|ΔE_exact|, 5e-4)` (stake ≥ 10); the line at 2.7/2.9/3.1 Å 0 of 3 within — misses 1.72, 1.90, 1.44 mHa against tolerances 1.08, 1.37, 1.19; the bends and the flip miss by 2.87, 2.63, 3.11, 1.55 mHa; within: 2.3, 2.5, 3.4, 3.7 Å and the twisted dimer |
| dispersion | **not transferred** | after the contact term the outer line's remainder is `−1.90, −1.44, −0.70, −0.28` mHa at 2.9–3.7 Å, log-log slopes `−4.09, −7.77, −10.84` — not all in `[−8, −4]`; `C₆ = 0` recorded |
| G-C1 — the engine's arithmetic, one reference | **PASS** | worst `1.08e-16` on the twelve (stake `1e-10`); every node served with two units, one cross O–O pair, four cross H–O pairs |
| plant (i) — the sign of the H–O contact | **FIRES** | miss `2.246458e-2` against `2·|p_HO(2.9)| = 2.246458e-2` (difference `1.2e-16`); carrier `1.12e-2 ≥ 1e-4` |
| S2 — a doubly bent bond, predicted forward | **BRANCH (b)** | filed before the solve: `E_pred = −7.804e-4` (field `−1.972e-3`, contact H–O `−5.163e-3`, walls O–O `+1.840e-3`, H–O `+4.122e-3`, H–H `+3.93e-4`); the solve: `ΔE_exact = −3.143038e-3`, `Converged`, 209 iterations, residual `7.9e-11`, 30,935 core-seconds (band 1,450–57,600), 1,403 s wall on 24 threads; miss `2.363e-3` (75.2 %) against `7.86e-4` — (a) fails; `|wall − E_exch| = 2.21e-4` against `7.86e-4` — (b) |
| S3 — retention, read beside the drift | **not run (VOID by G-B0)** | — |

## 1. The tolerance the readings gave

The single-class exponent per (family, tilt) column on the fit set (`d0.json`): tilt family
2.162, 2.159, 2.152, 2.117, 1.966, 1.732 per bohr at 0°–180°; twist family 2.196, 2.200,
2.072, … down to 1.682. The exponent steepens with face-on contact in the tilt family and
softens with twist; a fixed exponent per class serves the set only to within that spread,
and the freeze's rule made the spread the tolerance. Plant (ii) is the measure of what the
typed twentieth would have cost: nineteen readings.

## 2. The well inside the floor

The H–O class potential of the harvested law, `−16.26·e^{−2.06 r} + 22.59·e^{−2.20 r} +
q_H·q_O/r` with `q_H = 0.2314`:

| r (bohr) | 2.78 (r_min) | 2.68 | 2.33 (minimum) | 1.95 (the identity's boundary) | 0.5 |
|---|---|---|---|---|---|
| U (mHa) | −41.6 | −42.9 | −45.6 | −38.2 | +1,499 |

Bounded, positive at contact, and `4.3 kT` deep below its fit floor, with its minimum at
H···O 1.23 Å — well outside the identity's boundary (1.03 Å) and well inside any hydrogen
bond. FIELD-8's finding ("a bounded well is not a hole") is true of it; FIELD-9's gate says
a well this deep inside the data is not a law the dynamics may run on, and the two are not
in conflict: the well is real to the fit and false to the physics because the exponential
that serves the twelve geometries from 2.3 to 3.7 Å cannot also serve 2.3 bohr. The contact
term needs more than one shape, or a piece of it needs its own shape. CT-1 takes the second
route first.

## 3. The contact fit's weights, read

The freeze's contact fit weights each point `1/ΔE_exact²`. Two of the twelve sit near the
zero crossing of the exact interaction — the line at 2.5 Å (`+2.5` mHa) and the twisted
dimer (`+0.52` mHa) — and take weights 5× and 110× the hydrogen-bond minimum's. Both are
within tolerance in the record, and the line at 2.7–3.1 Å is not: the fit was pinned where
the energy is smallest. CT-1's freeze puts a floor under the weight (`1/max(|ΔE_exact|,
5e-3)²`) and records this as its correction of the rule.

## 4. Bookkeeping, declared

- The delegate launched `predict` itself; the prediction was filed before the solve (the
  `fit.log` line and the `prediction.json` timestamp precede `predict.log`).
- The records of FIELD-6 through FIELD-9 were written with a `+` sign on positive numbers
  (`{:+.12e}`), which is not JSON. The values are unchanged; the `+` tokens were removed
  from every affected file in `field6/`–`field9/` (FIELD_RESULTS.md CORRECTIONS 3) and
  the writer in `field9_harvest.rs` corrected. The gate loaders read by string and were
  never affected.
- The G-B0 `kT` budget was typed in the freeze; the well it refused is `4.3 kT`, so the
  refusal does not hang on the budget's size.
- No number enters from outside the engine and its own solver.
