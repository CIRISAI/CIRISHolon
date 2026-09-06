# CT-2 — results

*Freeze `CT2_PREREG.md` (adf2ea5, alone; corrected before freezing from 67/52 to 64 distinct
geometries / 51 new solves when the runner's delegate found that the tilt family's 3.4 Å/180°
node is `flipped_R3.4` atom for atom). Engine (the lead, 64e34fa): the angular transfer term
with analytic forces on five atoms, checkpoint v12, the door, the derivative gate on it.
Harvest `holon-render/examples/ct2_harvest.rs` (delegate, to the freeze; `map`, `fit`,
`predict`); the arms by `field3_hbonds.rs` reading `wall_ct2.json`. JSON under `ct2/`, every
file valid.*

## The verdict, first

**The first seam law that the boundedness gate admits and that holds the dimer and the ring at
293 K — and a second forward prediction landed. The angular family declared for charge
transfer does NOT carry it: the transfer follows the acceptor's plane normal, not its lone
pairs, and rises when the acceptor turns away. The liquid's condition is met, and the liquid's
door then refused the law's soft tail; that is answered by an amendment, not here.**

1. **The map (G-D0, T0, A1).** The product-start solver reproduced all thirteen exact records
   to `3e-11`–`7e-11` against a bar of `1e-8`, in 14–19 iterations at 1,146–1,441 core-seconds
   where the stock start took 139–222 iterations and 15,518–52,739: the exact dimer is now a
   `1,400`-core-second object. Fifty-one new nodes, every one converged, priced, and inside the
   identity; one candidate was the flipped dimer of record and was counted once. Charge
   transfer is attractive at all 60 nodes whose shortest cross-unit H···O is under 5 bohr.
2. **What the map says about the angle.** At every separation the transfer RISES as the
   acceptor tilts away from the linear geometry, by ~40 % from 0° to 90°, and stays high to
   180° where the acceptor's own hydrogens face the donor (at 2.9 Å: `−9.3, −10.4, −12.5, −13.2,
   −12.0, −11.9, −13.3` mHa at 0°–180°). The donor angle is steep the other way: bending the
   donor's O–H 15° off the axis cuts the transfer by a sixth, 30° by half, 60° by nine tenths.
   The declared acceptor factor — two lone pairs at an angle `λ ≤ 70°` from the reversed
   bisector — predicts a peak near the lone-pair angle and a fall beyond it, the opposite of
   the data, and the fit answered by choosing `k = 0` (no acceptor factor at all) with a
   gentle donor factor `m = 1`: `P = 1.371, c = 1.40, m = 1, k = 0`. **S1 reads (c)**, 41 of 64
   within tolerance, and the twisted dimer of record is missed by MORE than CT-1 missed it
   (`3.42` vs `2.73` mHa). The direction the data want is the acceptor's plane normal, its
   highest occupied orbital, which the family reaches only at `λ = 90°`, past its grid.
3. **The contacts under the gate serve the line, and the gate passes (C1, G-B0).** With every
   amplitude clamped at the largest value the boundedness walk admits, the contact re-fit
   gave `P_HO = 22.17, c = 2.44` and `P_HH = 0.017, c = 1.02` — the constraint bound at 14,958
   of 30,976 grid points but not at the chosen one, and the residual ratio against the
   unconstrained fit is `1.002`: the gate cost the fit a fifth of a percent. C1 fails by count
   (43 of 64 against 52) with the line at 2.7, 2.9, 3.1 Å all within; the misses are the twist
   family and the doubly varied nodes, where the transfer term's miss propagates. **G-B0
   returns `None`** — every class bounded, positive at contact — the first time in nine seam
   campaigns; the monotone reading beside it names a 0.4 mHa dip at 2.95 bohr, a bounded well.
4. **The arms held (S3 (a)).** At 293 K the dimer keeps its hydrogen bond on 55 % of the counted
   frames and the ring on 69 %, both over the staked half; at 150 K both read 1.000. The
   books closed (drift peaks `2.2e-7` and `6.1e-7` against the OFF arms' `1.7e-6` and
   `1.5e-5`), momentum conserved, units 2 and 4 throughout, mean temperatures 276 K and 267 K
   (within 30 % of target), the OFF arms reproducing FIELD-2 exactly. LIQUID-1's condition —
   the first law to pass its boundedness gate and hold the dimer and the ring — is MET.
5. **S2 landed, branch (a), at the edge.** The twisted, tilted and bent bond at 3.1 Å binds by
   `−3.031` mHa exact against `−3.725` filed before the solve: a 22.9 % miss against a 25 %
   tolerance; the transfer term by itself within its own bar (`−4.91` predicted, `−4.10`
   measured). Two of two forward predictions in the programme have now landed; this one says
   the law is right at a quarter, not better.

| gate | verdict | the number |
|---|---|---|
| G-D0 — the product-start solver | **PASS** | 13 of 13 within `1e-8` (worst `7.1e-11`); 14–19 iterations, 1,146–1,441 core-s, against the stock's 139–222 iterations and 15,518–52,739 core-s |
| T0 — the sector is what it says | **PASS** | 51 of 51 new nodes: `Converged`, residual `≤ 9.9e-9`, dimension `194,481`, order on totals; every solve priced above its floor |
| A1 — the map | **PASS** | `E_CT < −1e-6` at 60 of 60 nodes inside 5 bohr; the four nodes beyond (donor bent 60°/90° at 3.1 Å, shortest H···O 5.2–6.2 bohr) read `−3.7e-4` and `−7.3e-5`, attractive, entered as nulls per §1 |
| P1 — the family contains CT-1 | **PASS** | at `m = k = 0` on CT-1's twelve: `P = 1.474879879`, `c = 1.46`, differences `3.9e-11` and `0` |
| S1 — the angular term | **BRANCH (c)** | `P = 1.371043, c = 1.40, m = 1, k = 0` (λ immaterial at `k = 0`; no parameter at a grid edge); 41 of 64 within `max(0.25·|ΔE_exact|, 5e-4)` (stake 52); the 23 misses: the line's ends (2.3, 2.5, 3.4, 3.7 Å), the tilt family at 60°–120° for 2.7–3.1 Å, the twist family at 30°–90°, two doubly varied nodes, and `twisted_R3.0` (fit `−7.87`… measured `−4.44` mHa; miss `3.42e-3`) |
| C1 — the contacts under the gate | **FAIL by count, the line within** | constrained `P_HO = 22.17048, c_HO = 2.44` (clamped: no), `P_HH = 0.016971, c_HH = 1.02` (clamped: no), one round to the fixed point; unconstrained `24.81/2.48`, `0.01507/0.96`; residual ratio `1.002252`; 43 of 64 within (stake 52); the line at 2.7/2.9/3.1 Å 3 of 3 within (`0.75, 0.86, 0.91` mHa against `1.08, 1.37, 1.19`); no exponent at a grid edge |
| dispersion | **not transferred** | remainder after both contacts `−0.86, +0.91, +0.79, +0.55` mHa at 2.9–3.7 Å (slopes `+0.7, −1.5, −4.2`); `C₆ = 0` |
| G-B0 — bounded, every class named | **PASS — `None`** | the first admitted law; `hole()` reads a dip at 2.95 bohr (`−4.0636e-2 < −4.0208e-2`), bounded |
| G-B3 — the force is the derivative of the angular term | **PASS** | the seam suite's derivative check on every atom of the dimer with the declared angular model (m 2, k 2, λ 55°): worst relative at `h = 1e-5` under `1e-8`; seam 8/8, books and momentum closed |
| G-C1 — the engine's arithmetic, one reference | **PASS** | worst `4.72e-16` on 64 nodes, the formula side an independent evaluation of §0's equations; every node two units, one O–O pair, four H–O pairs |
| plant (ii) — the sign of the transfer term | **FIRES** | `1.960061e-2` against `2·|CT(2.9)| = 1.960061e-2` (difference `9.0e-17`); carrier `9.80e-3` |
| plant (i) — the angle removed | **FIRES** | CT-1's term on `twisted_R3.0`: `−7.169` vs measured `−4.443` mHa, miss `2.725e-3 ≥ 2e-3`; the fitted family's miss beside it `3.424e-3` |
| S2 — a twisted, tilted and bent bond | **BRANCH (a)** | filed `E_pred = −3.724961e-3` (field `−2.129e-3`, CT `−4.910e-3`, contacts `−1.250e-3`/`−2.51e-4`, walls `+7.43e-4`/`+3.351e-3`/`+7.20e-4`); solved `ΔE_exact = −3.030541e-3` (15 iterations, residual `3.4e-9`, 1,448 core-s, 65 s wall); miss `6.94e-4` (22.9 %) against `7.58e-4`; `E_CT` measured `−4.098e-3` against the term's `−4.910e-3`, miss `8.1e-4` against `1.02e-3` |
| S3 — retention, read beside the drift | **BRANCH (a)** | 293 K: dimer `f = 0.5526` (T̄ 276 K, drift peak `2.2e-7` vs OFF `1.7e-6`, units 2, 9/18 pairs and triples dropped per pass, books and momentum closed), ring `f = 0.6913` (`n̄ = 1.48` of 3 bonds, T̄ 267 K, drift `6.1e-7` vs OFF `1.5e-5`, units 4); 150 K: dimer `1.000`, ring `1.000` (`n̄ 3.30`); the OFF arms reproduce FIELD-2 (`f = 0`) |
| L — the liquid | **CONDITION MET; the liquid's own door then refused (§5)** | `LIQUID1_AMENDMENT_2.md` |

## 1. The map, in one table each way

E_CT (mHa) against acceptor tilt (rows) at each separation (columns), the linear node the 0° row:

| tilt | 2.7 Å | 2.9 Å | 3.1 Å | 3.4 Å |
|---|---|---|---|---|
| 0° | −16.29 | −9.34 | −5.02 | −1.80 |
| 30° | −18.09 | −10.43 | −5.63 | −2.02 |
| 60° | −21.53 | −12.48 | −6.77 | −2.44 |
| 90° | −22.84 | −13.22 | −7.18 | −2.61 |
| 120° | −20.74 | −12.03 | −6.65 | −2.55 |
| 150° | −19.48 | −11.93 | −7.09 | −3.10 |
| 180° | −20.55 | −13.26 | −8.25 | −3.82 |

E_CT against the donor's bend at 2.9 Å: `−9.34` (0°), `−7.77` (15°), `−4.54` (30°), `−2.43` (45°),
`−1.07` (60°), `−0.28` (90°). The two angles are not alike: the donor's O–H must point at the
acceptor oxygen, steeply; the acceptor need only turn its plane toward the hydrogen, and any
turn away from the linear geometry helps. That is the shape of the acceptor's highest
occupied orbital (out of the molecular plane) and the donor's antibonding O–H (along the
bond), not of two tetrahedral lone pairs. The declared family had the donor side roughly
right and the acceptor side wrong, and CT-3's family is read off this table: an acceptor
factor on `|u·n|` (the plane normal), a donor factor steeper than `m = 4`.

## 2. What the bounded law is

`wall9`'s three walls; the constrained contacts `P_HO = 22.17 e^{−2.44 r}`, `P_HH = 0.017
e^{−1.02 r}`; the transfer `−1.371 e^{−1.40 r} · ((1 − cos θ_d)/2)` on every cross-unit H–O
pair; `C₆ = 0`. Bounded in every class, positive at contact, a 0.4 mHa dip at 2.95 bohr on
H–O; right within a quarter on 41–43 of 64 dimers and on the line at the minimum; holds the
dimer and the ring at 293 K. Its softest terms reach 16.7 and 18.6 bohr at the `1e-10` budget,
which is where §5 begins.

## 3. Bookkeeping, declared

- The freeze's node count was corrected BEFORE freezing (67/52 → 64/51) on the delegate's
  finding; the duplicate rule is the freeze's letter.
- The runner's declared choices: the node reader slices the exact and sector blocks before
  parsing; the two-class clamp iterates to a fixed point with the round count recorded (one
  round here); `λ` is marked immaterial at `k = 0` so the grid-edge rule cannot fire on a
  parameter the fit never read; G-C1's formula side never calls the engine's `ct_angular`.
- The arms ran on the law as harvested (`r_cut = 0`, no switch), which is the freeze's letter.
- No number enters from outside the engine and its own solver.

## 4. What the two (a)s and the (c) mean together

A law right at a quarter on 64 dimers and bounded everywhere holds a bond at temperature; a
family wrong in shape still lands a forward prediction inside a quarter because the quarter
is wide. Rule 6 counts the landing as support for the law WHERE ITS DATA ARE; S1 (c) says
the angular story attached to the term is not yet the right one. Both are on the record.

## 5. The liquid's door, and the amendment

With LIQUID-1's condition met, the counted arm was launched on this law and its door
REFUSED the 128-water cell: the seam terms' reach at the `1e-10` budget is `18.578` bohr (the
H–H contact's soft exponent; the transfer term reaches 16.67) against a half-edge of
`14.797`. The terms there are worth `4.7e-9` and `1.4e-9` hartree per pair. The answer is
`LIQUID1_AMENDMENT_2.md` (4237ab9, alone, before any counted frame): a declared C² switch on
every seam term at `r_cut = 14.0` bohr, the truncated tail priced on the start box and staked
at `1e-5` hartree per water, the door reading the switch. The liquid's results file carries
what happens next.
