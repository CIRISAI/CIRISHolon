# FIELD-7 — results

*Freeze `FIELD7_PREREG.md` (4bbba02, alone). Engine: `SeamModel` gains the walls on the two
further cross-unit classes (H–O beside the contact term, H–H), checkpoint v9, the derivative
gate run over every atom on an all-classes model; harvest `examples/field7_harvest.rs`
(delegate, to the freeze, the wall fit verified against an independent implementation to
`5e-6`); the arms by `field3_hbonds.rs` reading `wall7.json`; the diagnostic
`examples/field7_probe.rs`. JSON under `field7/`.*

## The verdict, first

**The wall transfers across orientation once it lives on the atom pairs; the transferred
contact term has a hole below its data, and the arms found it.** Twenty-four undeformed
exchange readings (one Hamiltonian application each) fit a three-class exponential wall at
21 of 24 within the derived tolerance (**S1 (b)**), and the O–O-only wall fails at 20 of 24
(plant (ii) fires: at one oxygen separation the exchange moves by `5.83` with orientation,
and the hydrogen–hydrogen class carries it — `+6.9` mHa on the flipped dimer where the O–O
class is `+1.2`). The contact term re-fit with that wall held reproduces 8 of 9 exact
geometries, G-C1 holds at `1e-16`, and the twisted-bond prediction was filed before its
solve and READ (c): the twisted dimer is `+0.52` mHa repulsive on the exact solver against a filed `−1.91` — the three-class wall is `1.2` mHa too repulsive there against the referee's own exchange (`10.75` against `9.53`), and the contact term `3.6` mHa too attractive: the remainder is more orientation-dependent than one exponential on the H–O contacts carries. Then the arms: the dimer held its hydrogen bond for 5,900 counted
frames at 293 K with the ledger closed to `1e-7` — the first time this engine has held
water to water for a reason it derived — and at frame 7,892 a cross-unit H···O contact
reached 2.5 bohr, where no data exists, and fell into the hole: the contact term
(`−8.97·e^{−1.83 r}`) decays slower than the H–O wall (`8.67·e^{−2.30 r}`), so below the
shortest fit point their sum is attractive all the way to contact (`−0.83` hartree at 0.8
bohr). The ledger drifted by `1.0` hartree, the pair fused, and the lens kept counting a
hydrogen bond: **S3 reads (a) by its letter and is VOID in substance** — the
vacuous-success shape on a retention count, registered as M-EXTRAPOLATED-HOLE. W0's
determinant-count leg fails by the freeze's own arithmetic (the undeformed state is
nonzero on the full space by construction; the count belonged to the orthogonalised one).
FIELD-8 is named: data at contact for both harvests, and a no-hole gate before any arm.

| gate | verdict | the number |
|---|---|---|
| G-E0 — the identity | **PASS** | seam and ledger gates unchanged with the new classes at `0.0` (7/7, 7/7) |
| G-E1 — the terms are the derivative | **PASS, with the letter's step read** | on the harvested law the letter's `h = 1e-4` reads `9.6e-8` (the `O(h²)` truncation on atoms whose seam force is small under steeper terms); at `h = 1e-5` the worst relative is `2.3e-9` (declared all-classes model: `5.3e-9` / `4.6e-10`) — the property holds, the step was FIELD-3's; momentum `7.6e-18` against `2.9e-2`; books closed. The dynamics gates (books, momentum) refuse the FIELD-7 record BY NAME — `has_hole`: the H–O potential falls inward from 3.0 bohr — and run on declared coefficients, saying so (the no-hole rule of M-EXTRAPOLATED-HOLE, applied to the gates first) |
| W0 — the readings are what they say | **FAIL by letter on the count leg, read** | norm `0.9419–0.9992` PASS; floor PASS; order along every tilt EXACT PASS; the count leg staked `194,481` nonzero determinants — the undeformed state is nonzero on all `1,002,001` (the minors spread it over the space by construction); `194,481` is the ORTHOGONALISED product's count, copied from FIELD-5. The instrument is right; the number was the freeze's |
| S1 — the wall over orientations | **BRANCH (b)** | `A_OO = 1623.7`, `b_OO = 2.20`; `A_OH = 8.669`, `b_OH = 2.30`; `A_HH = 2.652`, `b_HH = 1.90` (all three classes kept non-negative); 21 of 24 within `max(0.05·E_exch, 1e-4)`; misses (2.7 Å, 0°), (2.9 Å, 60°), (3.1 Å, 60°), worst `1.16×`; the constrained global optimum on the full grid, verified independently to `5e-6` |
| plant (ii) — the wall on the oxygens only | **FIRES** | fails 20 of 24 (stake ≥ 6); carrier `E_exch(3.4 Å, 180°)/E_exch(3.4 Å, 0°) = 5.83` (≥ 2; the deformed referee read 5.8) |
| C1 — the contact term | **PASS** | `P = 8.971` Ha, `c = 1.83` /bohr with the wall held, 8 of 9 exact points within `max(0.25·|ΔE_exact|, 5e-4)`; the miss is the flipped dimer at `1.13×` (remainder `−5.22e-3` against fit `−3.52e-3`) |
| dispersion | **not transferred** | post-contact slopes on the outer linear nodes `−9.2`, `−5.2`, `−3.9`; `C₆ = 0` |
| G-C1 — the harvest is the engine's arithmetic | **PASS** | worst `1.22e-16` over the nine geometries with every class in the formula |
| plant (i) — the sign of the contact term | **FIRES** | `2.814399e-2` against `2·|p(2.9 Å)| = 2.814399e-2`, to `4.9e-17`; carrier `1.41e-2` |
| §1 — the expectation | **"hold" at both temperatures** | dimer binding `−5.83e-3` (field `−4.08e-3`, seam `−1.75e-3`), ring `−1.209e-2`; units 2 / 4 |
| S2 — a twisted hydrogen bond, predicted forward | **BRANCH (c)** | filed before the solve: `E_pred = −1.912e-3` on the twisted dimer at 3.0 Å (field `−2.255e-3`, contact `−1.040e-2`, wall O–O `+6.221e-3`, H–O `+1.689e-3`, H–H `+2.835e-3`, dispersion `0`); the solve: `ΔE_exact = +5.172e-4` (the twist unbinds the dimer), `Converged`, 226 iterations, residual `8.2e-11`, 24,417 core-seconds; miss `2.43e-3` against `5.0e-4` (the absolute arm) — (a) fails; the wall reads `+1.0745e-2` against the undeformed exchange on that geometry `+9.528e-3` (`+1.22e-3`, 12.8 %, outside `5.0e-4`) — (b) fails ⇒ (c). Holding the wall at the referee's value makes the miss worse (`3.65e-3`): the contact term, at `−1.040e-2` on this geometry, is about `3.6` mHa too attractive |
| S3 — retention under the seam law | **BRANCH (a) BY LETTER, VOID IN SUBSTANCE** | `f_SEAM` dimer 1.0000 / 0.8612, ring 0.7642 / 1.0000 (293 / 150 K) — but the arms' honest drift peaks are `1.0`, `0.16`, `0.096`, `1.8` hartree against `1e-7` on the OFF arms, mean temperatures 441 / 368 / 655 / 488 K against 293 / 150 K targets, and the dimer's final state at 293 K has `units 0`, `e_field 0`, O–O 4.73 bohr: fused. `f_OFF` 0.0000 everywhere (FIELD-2's, reproduced) |

## 1. The wall over orientations

| class | A (Ha) | b (/bohr) | value on the linear 2.9 Å dimer | on the flipped 3.4 Å dimer |
|---|---|---|---|---|
| O–O | 1623.7 | 2.20 | `+9.3` mHa (R 5.48) | `+1.2` mHa (R 6.43) |
| H–O | 8.669 | 2.30 | `+2.6` mHa (the contact at 3.56 bohr dominates) | `+0.5` mHa |
| H–H | 2.652 | 1.90 | `+0.3` mHa | `+6.9` mHa (H···H contacts at ~3.5 bohr) |

Exchange at a fixed oxygen separation is not an oxygen property: the flipped dimer's `8.6`
mHa is carried by the hydrogen–hydrogen class, which the linear geometry barely excites. The
24 readings cost `55–60` core-seconds each; the whole orientation harvest cost less than one
exact node.

## 2. The hole, measured

The cross-unit H···O potential under the harvested law — contact term, H–O wall and the
point charges — inside the data:

| r (bohr) | contact | H–O wall | charges | sum |
|---|---|---|---|---|
| 3.5 | `−0.015` | `+0.003` | `−0.031` | `−0.043` |
| 2.5 | `−0.093` | `+0.028` | `−0.043` | `−0.108` |
| 2.0 | `−0.231` | `+0.087` | `−0.054` | `−0.197` |
| 1.0 | `−1.439` | `+0.869` | `−0.107` | `−0.677` |

(hartree). Every fit point has `r ≥ 3.4` bohr; below it the term with the slower decay wins,
and there is no floor. The probe (`field7_probe.rs`) on the dimer at 293 K: the contact
first reaches 2.5 bohr at frame 7,892 with the ledger still closed to `1.2e-7`, reaches
2.25 at frame 8,527, and the drift is `1.0` hartree by the end; at 150 K the same at frames
4,277 and 4,783, final temperature 1,106 K. The 5,892 counted frames before the fall at 293 K
are a genuine retention reading — the seam law held the bond, with its books closed — and
the freeze has no clause for a bond that holds until the law breaks. The record keeps both
facts.

## 3. What (a)-by-letter means, and what it does not

It does not mean the seam law holds water's hydrogen bond at room temperature: the count
that says so was taken on a fused pair for two thirds of the window. It does mean that
between the charges, the transferred contact attraction and the three-class wall, the law
is BOUND at the start by `−5.8` mHa (6 kT), the expectation rule wrote "hold" for the first
time, and the dynamics agreed for as long as the law was on its data. What breaks it is not
a missing channel but a missing reading: the contact term below 3.4 bohr, where the exact
dimer at 2.5 Å (H···O 2.8 bohr) is already `+2.5` mHa repulsive and the seam law is not.

## 4. S2 — the twisted bond

The twisted geometry — the acceptor turned 90° about the O···O axis and tilted 60°, a kind
no fit point contains — is `+0.52` mHa on the exact solver: the twist unbinds the dimer. The
seam law predicted `−1.91`. Two things miss, in opposite directions. The three-class wall
reads `10.75` mHa where the undeformed referee reads `9.53` on the same geometry — `1.2` mHa
too repulsive, 12.8 % of the reading and outside its own S1 tolerance there — so the wall,
fit on one tilt family, is not yet orientation-complete either. And the contact term, at
`−10.4` mHa on this geometry, is `3.6` mHa too attractive: its shortest contact (3.73 bohr)
is a donor hydrogen pointing at the acceptor oxygen with the acceptor's hydrogens swung out
of plane, and the term that folds charge transfer and correlation on the line and the bends
over-binds it. That is §4's named gap reading back: the contact term's one miss among its
nine fit points was the flipped dimer, the most rotated of them, and the twist is the same
failure, larger. The referee itself is healthy there (overlap `0.993`, one application `105`
core-seconds).

Two measurements come out of the day's three campaigns that FIELD-8 must carry: the wall
needs the twist family in its harvest (cheap — one application per geometry), and the
remainder is not one exponential on H–O across orientations; its shape across the twist is
now measured at one point, and a second class (H–H, the contacts the twist makes) is the
first thing to try, with the same non-negativity discipline the wall used.

## 5. FIELD-8, named

The twist family in the orientation harvest for the wall; the contact term on two classes
(H–O and H–H) over the ten exact geometries the record now holds; and data at contact, in both harvests: close-geometry exchange readings (R_OO 2.1–2.5 Å at every
tilt — cheap) so the H–O and H–H walls are measured where the hole is, and two close exact
nodes on the line (2.3 and 2.1 Å) so the contact term's inner shape is measured rather than
extrapolated; then a NO-HOLE gate — every cross-unit pair potential of the harvested law
must rise monotonically inward from its shortest data point to contact, else the arms are
VOID before they run — and the arms behind it, with the lens count read beside the arm's
own drift. The freeze's standing rule from this campaign: a lens count on an arm whose
drift has left the honest band is not a retention reading.

## 6. Bookkeeping, declared

- The arms ran before S2; the delegate's `predict` waited on FIELD-6's solve to keep the
  price window.
- The harvest's wall fit and its constrained optimum were verified by the delegate against
  an independent implementation (relative `5e-6` on the coefficients, `3.5e-8` hartree on the
  per-geometry wall).
- No number enters from outside the engine and its own solver.
