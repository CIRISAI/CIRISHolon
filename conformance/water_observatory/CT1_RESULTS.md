# CT-1 — results

*Freeze `CT1_PREREG.md` (b8fa5e8, alone); `CT1_AMENDMENT_1.md` (2094597, alone, before any
water reading: the closed sector is the block-localised space of the monomers' OWN
determinants, after the orthogonalised sector refuted itself on the hydrogen pair). Instrument
`holon-chem/src/heitler_london.rs::fci_block_localised` (the lead; unit-gated on H₂·H₂: order,
plant (i), the 40-bohr limit). Engine: the sixth ledger record (`ChannelId::ChargeTransfer`,
Identity, arity 2, exponential, a solve, the kill in the record), `SeamModel { p_ct, c_ct }`,
checkpoint v11, the door, `SeamPlant::FlipChargeTransfer`; gates seam 8/8, ledger 7/7 (six
channels; the receipt bit-for-bit), field 7/7, Ewald 6/6, holon-chem 45/45. Harvest
`holon-render/examples/ct1_harvest.rs` (delegate, to the freeze; `sector`, `far`, `fit`,
`predict`). Sizing of channels 3 and 4 by a second delegate (`ct1/sizing/`). JSON under
`ct1/`, every file valid.*

## The verdict, first

**Charge transfer is measured, by itself, on the engine's own solver: it is most of the
hydrogen bond's remainder, it is in the ledger as the sixth channel, and the contact term
fit on what remains serves the bond minimum for the first time — by two terms that are
holes. The arms are VOID again.**

1. **Charge transfer is 88 % of what the undeformed product misses at the minimum (T0–T2).**
   At the linear 2.9 Å node the exact dimer sits `−10.648` mHa below the undeformed
   Heitler–London product; the closed sector — the monomers polarising and correlating each
   other without an electron crossing — recovers `−1.311` of it, and the rest, `−9.337` mHa, is
   charge transfer. Along the line it runs `−40.1, −26.5, −16.3, −9.34, −5.02, −1.80, −0.58` mHa
   from 2.3 to 3.7 Å, attractive everywhere, non-increasing outward, with log-log slopes
   `−9.3, −11.1, −13.4` beyond 2.9 Å — overlap-shaped, not a power. Every sector solve
   converged in 12–14 iterations at residual `≤ 9.6e-9` on the exact `194,481` states, and at 40
   bohr charge transfer is zero to `3e-11` against an exact solve of that geometry.
2. **One exponential on cross-unit H–O carries it to 10 of 12 (S1 (b)).** `P_CT = 1.475` Ha,
   `c_CT = 1.46` per bohr. The two misses say what the shape lacks: the twisted dimer's charge
   transfer is `−4.44` mHa where the pair-distance sum predicts `−7.17` — a geometry in which
   no O–H points at a lone pair, over-counted by a term with no direction in it — and the
   bent acceptor misses by 5 % of its tolerance. Charge transfer has an angle; the term does not.
3. **With charge transfer taken out first, the contact term serves the line (C1 PASS, 11 of
   12, the line at 2.7–3.1 Å all within) — and both contact terms are holes.** The H–O
   contact came out `P_HO = 23.70` at `c = 2.44`, an amplitude that exceeds the H–O wall's
   (`22.59`) with a steeper exponent, so the class potential ends at `−0.41` hartree at 0.5
   bohr; the H–H contact came out `P_HH = 158.8` at `c_HH = 4.00`, the GRID'S CEILING, and its
   class potential is `−20.7` hartree at contact. G-B0 refuses the law (arms VOID). The gate
   as shipped reported only the first violation it met — the H–O class one kT below its floor
   at 2.58 bohr — and said nothing of the abyss two lines later; the engine now names every
   class and both legs (M-FIRST-VIOLATION-ONLY registered), and a fitted exponent at either
   edge of its grid is reported as the grid's number, not the physics'.
4. **The remainder after everything is small and slightly positive outward.** After charges,
   wall, charge transfer and contacts the line's residue is `−0.10, +0.15, +0.33, +0.29` mHa
   at 2.9–3.7 Å; dispersion is not transferred (`C₆ = 0`, the third campaign to read it), and
   the soft charge-transfer exponent over-reaches at 3.7 Å (`−0.97` fit against `−0.58`).

S2, the twisted-and-bent bond predicted forward, LANDED — branch (a), the programme's first
(§4): the law is right where its data are and a hole below them. The liquid still waits: the
first law to pass G-B0 does not exist. What passes next is named in §6.

| gate | verdict | the number |
|---|---|---|
| T0 — the sector is what it says | **PASS** | 12 of 12: `Converged`, residual `3.2e-9`–`9.6e-9` (bar `1e-8`), dimension `194,481` EXACT, `E_exact ≤ E_noCT ≤ E_HL` on totals; metric's smallest eigenvalue `0.78`–`0.98`; the 40-bohr leg on the EXACT solve of that geometry (139 iterations, residual `7.4e-11`, 14,533 core-s): `|E_CT| ≤ 3e-11` (bar `1e-8`); reported beside it, never as it: `E_exact − (E_A0+E_B0) = 2.026405e-6`, `E_noCT − (E_A0+E_B0) = 2.026380e-6` — the linear dimer's electrostatics survive at 40 bohr in both, which is why the monomer sum cannot stand in for the exact (§5) |
| T1 — attractive, short-ranged | **PASS** | `E_CT < −1e-6` at 2.3–3.4 Å; magnitude non-increasing along the whole line; slopes `−9.29, −11.12, −13.36` (stake `< −6`) |
| T2 — the decomposition at 2.9 Å | **PASS** | `ΔE_HL = +5.168` mHa (electrostatics + exchange); closed-sector gain `−1.311` (channels 2 and 3 at this level); `E_CT = −9.337` (channel 6); identity miss `0.0` against `E_exact − E_HL = −10.648` mHa; 87.7 % charge transfer; FIELD-6 read `−10.4` |
| S1 — the CT term | **BRANCH (b)** | `P_CT = 1.474880` Ha, `c_CT = 1.46` per bohr (grid 0.50–4.00 step 0.02; weights `1/max(|ΔE_exact|, 5e-3)²`, this freeze's correction); 10 of 12 within `max(0.25·|ΔE_exact|, 5e-4)`; misses: tilted_R2.9 (`1.617e-3` vs `1.544e-3`), twisted_R3.0 (`2.725e-3` vs `5e-4`: fit `−7.17` vs measured `−4.44` mHa) |
| C1 — the contact term re-fit | **PASS by letter; both terms are holes (§3)** | remainder `ΔE_exact − [E_q − E_q(40)] − wall9 − CT`; `P_HO = 23.7048, c_HO = 2.44`; `P_HH = 158.789, c_HH = 4.00` (the grid's ceiling); 11 of 12 within (stake ≥ 10), the line at 2.7/2.9/3.1 Å 3 of 3 within (`0.26, 0.10, 0.15` mHa); miss: twisted_R3.0 (`2.32e-3` vs `5e-4`) |
| dispersion | **not transferred** | remainder after both contacts `−0.097, +0.150, +0.329, +0.294` mHa at 2.9–3.7 Å; slopes `+6.6, +8.5, −1.3` — not in `[−8, −4]`; `C₆ = 0` |
| G-B0 — bounded (the CT term in) | **REFUSED, the arms VOID** | the gate as run, verbatim: `H–O potential falls to -4.1939e-2 at r = 2.58 bohr, more than kT below its value -4.0977e-2 at its fit floor r_min = 2.781`. The whole walk, from the corrected gate (§3): H–O `−0.405` Ha at 0.5 bohr (392 kT below its floor); H–H falls at 1.26 bohr and reaches `−20.75` Ha at 0.5 bohr (21,675 kT below); O–O bounded, `+286` Ha at contact |
| G-C1 — the engine's arithmetic, one reference | **PASS** | worst `1.41e-16` on the twelve (stake `1e-10`), the CT term in the formula; every node two units, one cross O–O pair, four cross H–O pairs |
| plant (ii) — the sign of the CT term | **FIRES** | `P_CT → −P_CT`: G-C1 miss `1.756279e-2` against `2·|CT(2.9)| = 1.756279e-2` (difference `2.4e-17`); carrier `8.78e-3 ≥ 1e-4` |
| plant (i) — the mask dropped | **FIRES** | the same Davidson on the FULL space from the product start: `E_full = −150.0520630749` against the record's `−150.0520630749` (miss `3.5e-11`, bar `1e-8`), 15 iterations, 1,806 core-s; carrier `|E_CT(2.9)| = 9.34e-3 ≥ 1e-3` |
| S2 — a twisted-and-bent bond, predicted forward | **BRANCH (a)** — the programme's first seam prediction to land | filed before the solve: `E_pred = −3.675083e-3` at `R_OO = 3.2` Å (acceptor twisted 90° about the O···O axis, donor bent 20°): field `−2.429e-3`, CT `−3.037e-3`, contact H–O `−7.06e-4`, H–H `−4e-8`, walls O–O `+4.72e-4`, H–O `+1.884e-3`, H–H `+1.42e-4`, dispersion `0`; the solve: `ΔE_exact = −3.314265e-3`, `Converged`, 222 iterations, residual `8.3e-11`, 30,472 core-s (band 1,450–57,600), 1,444 s wall; miss `3.61e-4` (10.9 %) against `8.29e-4` — (a); and the CT term by itself: measured `E_CT = −2.515e-3` on that geometry (sector 13 iterations, residual `3.4e-9`, order holds) against the term's `−3.037e-3`, miss `5.22e-4` against `6.29e-4` — within too |
| S3 — retention | **not run (VOID by G-B0)** | — |

## 1. The measurement

| node | ΔE_exact (mHa) | E_HL − monomers | closed-sector gain | E_CT | E_CT / remainder |
|---|---|---|---|---|---|
| linear 2.3 Å | +23.31 | +76.36 | -12.98 | -40.07 | 75.5 % |
| linear 2.5 Å | +2.51 | +34.73 | -5.76 | -26.47 | 82.1 % |
| linear 2.7 Å | -4.32 | +14.63 | -2.66 | -16.29 | 86.0 % |
| linear 2.9 Å | -5.48 | +5.17 | -1.31 | -9.34 | 87.7 % |
| linear 3.1 Å | -4.76 | +0.97 | -0.70 | -5.02 | 87.7 % |
| linear 3.4 Å | -3.22 | -1.10 | -0.32 | -1.80 | 84.8 % |
| linear 3.7 Å | -2.14 | -1.39 | -0.17 | -0.58 | 77.3 % |
| bent acceptor 30°, 2.9 Å | -6.18 | +5.49 | -1.24 | -10.43 | 89.3 % |
| bent acceptor 45°, 3.1 Å | -5.43 | +1.37 | -0.59 | -6.22 | 91.3 % |
| flipped, 3.4 Å | +6.01 | +10.21 | -0.38 | -3.82 | 91.0 % |
| twisted 90°+60°, 3.0 Å | +0.52 | +5.61 | -0.65 | -4.44 | 87.3 % |
| bent donor 30°, 2.9 Å | -3.03 | +2.04 | -0.53 | -4.54 | 89.5 % |

(`E_HL − monomers` is electrostatics plus undeformed exchange; the last column is
`E_CT / (E_exact − E_HL)`.) Two readings in the table. Charge transfer is the remainder's
body at every geometry, 75–91 %, and its share RISES with distance out to 3.1 Å and with
every bend and twist — the closed sector's own polarisation-and-correlation is the smaller
part everywhere and is steeper. And charge transfer is largest, `−40` mHa, at 2.3 Å, where the
exact dimer is `+23` mHa repulsive: a third of the exchange wall there is paid back by
electrons crossing, which is the soft face of the identity's hard edge FIELD-8 found at 1.95
bohr.

## 2. What the term transfers and what it does not

`CT(g) = −1.475 · Σ_{cross-unit H–O} e^{−1.46 r}`. On the line and on the bends it holds to a
quarter; on the twisted dimer it over-counts by 60 %. The twisted geometry (acceptor rotated
90° about the axis and tilted 60°) turns every acceptor lone pair away from the donor's O–H
while leaving the four H–O distances near their linear values, and the term, having only
distances, cannot know. The physical contact — the donor's O–H σ* against the acceptor's lone
pair — is one bond's alignment, not four distances. The next shape has that angle in it.

## 3. The holes, named in full

The contact re-fit was asked to carry what remains after charge transfer, and on the twelve
geometries it did (C1). Off the data it does this:

| class | fit floor r_min (bohr) | U(r_min) | walk minimum | U at 0.5 bohr | verdict |
|---|---|---|---|---|---|
| O–O | 4.724 | +56.6 mHa | +56.6 mHa (at r_min) | +286 Ha | bounded |
| H–O | 2.781 | −41.0 mHa | −0.405 Ha at 0.5 | −0.405 Ha | falls 1.0 kT by 2.58 bohr, NEGATIVE at contact |
| H–H | 1.315 | −0.633 Ha | −20.7 Ha at 0.5 | −20.7 Ha | falls from its floor, NEGATIVE at contact |

Two mechanisms, both M-EXTRAPOLATED-HOLE's. The H–O contact's amplitude (`23.7`) exceeds
the H–O wall's (`22.6`) — with `c = 2.44 > b_OH = 2.20` the contact decays faster, so it loses
at long range, but at contact the larger amplitude wins and the class potential is
attractive where nothing is. The H–H contact was driven to the grid's ceiling exponent
(`4.00`) with an amplitude of 159 hartree to serve `−0.7` mHa at the twelve's shortest H–H
contact (3.07 bohr): a term that is nothing on the data and an abyss below it. The G-B0
letter refused the law either way. But the gate's report — the first violation in class
order, `H–O … more than kT below` — understated a two-class abyss as a one-kT dip until the
classes were walked by hand. `SeamModel::bounded` now returns every violating class and both
legs, with the walk's minimum beside the first fall (unit-tested on this law: four
violations named), and M-FIRST-VIOLATION-ONLY is in the registry with its two rules (name
every sector; report a grid-edge parameter as the grid's).

FIELD-9's law, checked the same way: the H–O class ended at `+1.50` Ha at contact and the H–H
class carried nothing, so its first-only report happened to be its whole report.

## 4. S2 — the twisted-and-bent bond, predicted forward and landed

The held-out geometry is one no fit point resembles: the acceptor turned 90° about the
O···O axis and the donor's O–H swung 20° off it, at `R_OO = 3.2` Å. The prediction was filed
with every part before the solve: `−3.675` mHa, of which `−3.04` is the charge-transfer term.
The exact dimer binds by `−3.314` mHa. The miss is `0.36` mHa, 10.9 % of the bond, against a
tolerance of `0.83` — **branch (a)**, the first time in seven seam campaigns that a forward
prediction has landed inside its tolerance (FIELD-6 (c), FIELD-7 (c), FIELD-8 (b), FIELD-9
(b)). The charge-transfer term read by itself on the same geometry: measured `−2.515` mHa
against the term's `−3.037`, within its own tolerance, over by 21 % in the direction the
twisted dimer's miss predicts — the term has no angle, and this geometry turns the acceptor
away. Rule 6 of the discipline: this is support, the only kind there is — a confirmed
advance prediction — and it is support for the law WHERE ITS DATA ARE. The same law is
refused below its data (§3). Both are true.

| | mHa |
|---|---|
| ΔE_exact (222 iterations, residual 8.3e-11, 30,472 core-s) | −3.314 |
| E_pred, filed before the solve | −3.675 |
| of which the CT term | −3.037 |
| E_CT measured (E_exact − E_noCT on this geometry) | −2.515 |
| closed-sector gain | −0.370 |
| E_HL − monomers | −0.430 |

## 5. Channels 3 and 4, sized (§4 of the freeze)

By the second delegate, `ct1/sizing/SIZING.md`, on the engine's own counts and admission door.

- **Channel 4 (three-body dispersion) is out of reach for water on every route this host
  has.** The exact trimer is `2,944,581,696` determinants (54,264 strings per spin); its
  Davidson working set is 2,282 GiB and the door refuses it. The density embedding does not
  reduce it: `subset_in_field` on a three-fragment list has no fourth body to embed in and IS
  the exact trimer — EMBED-3's instrument removes the fourth body, not the third. The
  undeformed Heitler–London product of three monomers does not fit either (51 GiB for one
  energy). What the embedding CAN measure on the ring is channel 2's three-body part:
  three-body induction `+0.0886` mHa, repulsive, on the cyclic trimer at 2.9 Å. The named
  instrument is the labelled MPS route, `price_mpo(21) = 1.76` GiB, provisional. The
  delegate's pairwise leg (three dimer-in-field solves) was stopped by the lead: with the
  trimer refused it could not be cashed into a residual.
- **Channel 3 (pair dispersion) at this basis is an absence, not a measurement gap.** Three
  campaigns now read `C₆ = 0` with overlap-shaped slopes. A minimal basis has no virtual p on
  hydrogen. Adding one p shell puts even the CLOSED sector 24× over this host's RAM
  (`566` GiB for two hydrogens), the full space 800,000× over.

## 6. What this reads for the six-channel law, and what is next

The measurement is what was asked: electron transfer between two closures, sized by itself
on the engine's own solver, in the ledger as its own row, kill included (a term that vanishes
at a larger basis was the basis's — Stone–Misquitta). The seam law is not yet a law the
dynamics may run on, for two reasons the harvest names: the charge-transfer term needs the
bond's angle, and the contact re-fit must be bounded by construction — fit under the
boundedness gate, not checked after it, with amplitudes that cannot exceed the wall's where
the exponent is steeper, and with grid edges reported. CT-2 is named for both. The liquid
(LIQUID-1, conditioned) waits on the first law that passes; its instruments are built and
dry-run (`LIQUID1_RESULTS.md`).

## 7. Bookkeeping, declared

- The far leg of T0 as the delegate first wrote it used `E_A0 + E_B0` in place of `E_exact` at
  40 bohr. The lead refused it before any fit ran and an exact solve of the far geometry was
  added (`far`); the measurement then showed the stand-in would have failed T0 by 200× for a
  reason unrelated to charge transfer (the linear dimer's electrostatics survive at 40 bohr,
  in both the exact and the closed sector). No result was read with the stand-in.
- The contact-fit weights carry this freeze's correction (`1/max(|ΔE_exact|, 5e-3)²`); the
  dispersion closed form uses the same weights rather than mixing two rules in one harvest.
- The gate report understatement (§3) is an engine correction landed with these results
  and a new registry entry; the freeze's G-B0 verdict is unchanged by it.
- Every JSON record this campaign wrote validates; the delegate ran `fit` only on synthetic
  records until the campaign's own existed.
- No number enters from outside the engine and its own solver; the prior art is credited in
  the ledger record.
