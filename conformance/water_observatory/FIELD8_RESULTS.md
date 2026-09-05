# FIELD-8 — results

*Freeze `FIELD8_PREREG.md` (a227318, alone). Engine: `SeamModel` gains the H–H contact term
and the no-hole walk `hole(q_H)` (checkpoint v10); the arms runner refuses a hole by name;
gates 7/7 (derivative `6.8e-10` with every class on). Harvest `examples/field8_harvest.rs`
(delegate, to the freeze; its `fit` run by the lead when the delegate's watcher failed to
wake it). Diagnostic `examples/field7_probe.rs` on the record. JSON under `field8/`.*

## The verdict, first

**Data at contact closed the hole and opened three findings, and by the freeze's letter
every harvest gate reads (c) or FAIL.** The 42 close and twist exchange readings landed
clean (order exact along every family, 56–112 core-seconds each; the twist family peaks at
120°, not 180°, which the tilt family never does), and both close exact nodes converged:
the linear dimer is `+23.3` mHa at 2.3 Å and `+77.2` mHa at 2.1 Å. Then:

1. **One exponential per class cannot span contact to 3.4 Å within a twentieth.** Over 66
   orientations the three-class wall fits 31 (**S1 (c)**): the exponent of exchange steepens
   inward, and a fixed exponent that reaches 2.1 Å under-serves the hydrogen-bond region
   (the O–O amplitude fell from FIELD-7's 1624 to 388 at the same exponent, a factor of four
   at the minimum). FIELD-7's far-data wall misses 16 of the 18 close readings (plant (ii)
   fires), so the far wall was not right at contact either. Neither range's wall is the
   other's.
2. **The closure identity has a boundary, and the 2.1 Å node is past it.** At `R_OO = 2.1` Å
   the donor hydrogen sits 1.97 bohr from the acceptor oxygen and 1.94 from its own; the
   closure reading (lowest `u` on the O–H curve) hands it to the acceptor, both units
   dissolve, and the engine serves the bare tables there — `+86.4` mHa against the exact
   `+77.2`, a reading of its own — while the harvest's formula assumed two units. G-C1 FAILS
   by `9.1e-3` at that node and passes at `1e-16` on the other eleven. A seam law is defined
   where the identity holds; the freeze put a fit point beyond it.
3. **A bounded well is not a hole.** G-N0 as staked (every class potential rising
   monotonically inward from 3.0 bohr) FAILS by letter: the H–O potential falls by `0.3` mHa
   from 3.0 to 2.95 bohr — and then rises to `+1.3` hartree at 0.5, because this time the H–O
   wall's exponent (2.15) exceeds the contact term's (1.98). The diagnostic on the dimer
   confirms it: over 22,000 frames at 293 K and 150 K no cross-unit contact ever came closer
   than its starting 3.56 bohr, the books closed to `1e-7`, and the pair simply DISSOCIATED
   (O–O 18.9 and 9.5 bohr at the end). The law is bounded and too weak at the minimum: C1
   fails there (6 of 12, the misses at 2.7–3.1 Å and every bend), which is the same fact as
   finding 1 seen from the remainder's side.

The arms did not run (G-N0's letter). S2 reads (b): the bent-donor bond at 2.9 Å binds by `−3.03` mHa on the exact solver against a filed `−1.29`, a 57 % miss, while the three-class wall's value there (`6.39`) is within `0.67` mHa of the undeformed exchange on that geometry (`5.72`): the wall transfers to a bent donor and the miss is the contact term's, the same weakness at the minimum that C1 and the diagnostic read. FIELD-9 is named at the end.

| gate | verdict | the number |
|---|---|---|
| G-F0 — the identity | **PASS** | seam and ledger gates unchanged with the new term at `0.0` (7/7, 7/7) |
| G-F1 — the terms are the derivative | **PASS** | worst relative `6.8e-10` at `h = 1e-5` with every class on (letter's `h = 1e-4`: `5.4e-9`); momentum and books closed |
| W0 — the readings are what they say | **FAIL by letter on the overlap leg, read** | overlaps `0.7134` (twist 2.3 Å 120°) and `0.7430` (close 2.1 Å 180°) below the staked `(0.8, 1]`; every other in window, next lowest `0.836`; floor PASS; order EXACT PASS along all 10 (family, tilt) columns. The window was typed; the two most face-on contacts have that much overlap |
| S1 — the wall over 66 orientations | **BRANCH (c)** | `A_OO = 387.9`, `b_OO = 2.20`; `A_OH = 17.30`, `b_OH = 2.15`; `A_HH = 1.309`, `b_HH = 1.70`; 31 of 66 within `max(0.05·E_exch, 1e-4)` (stake ≥ 53); the misses fill the close family and the hydrogen-bond region of the tilt family |
| plant (ii) — the far-data wall | **FIRES** | FIELD-7's wall misses 16 of the 18 close readings (stake ≥ 9); carrier `E_exch(2.1 Å, 0°) = 0.258` Ha (≥ 0.1) |
| C1 — the contact terms | **FAIL** | `P_HO = 11.74`, `c_HO = 1.98`; `P_HH = 1.2e-4` at `c_HH = 0.50` (the grid's floor — the H–H class carries nothing); 6 of 12 within (stake ≥ 10), both close nodes within; misses: the line at 2.7, 2.9, 3.1 Å and every bend and the flip |
| dispersion | **not transferred** | `C₆ = 0` |
| G-N0 — no hole | **FAIL by letter, read** | `H–O potential falls inward at r = 2.95 bohr (−3.9954e-2 < −3.9237e-2)`: a fall of `0.3` mHa, then a rise to `+1.3` hartree at 0.5 bohr (`b_OH 2.15 > c_HO 1.98`); a bounded well, not FIELD-7's abyss. The arms are VOID by the letter |
| G-C1 — the harvest is the engine's arithmetic | **FAIL at one node, read** | worst `9.12e-3` at the 2.1 Å node, where the engine finds NO units (the closure identity hands the donor hydrogen to the acceptor at H···O 1.97 bohr against its own 1.94); `1e-16` on the other eleven |
| plant (i) — the sign of the H–O contact | **FIRES** | `2.155842e-2` against `2·|p_HO(2.9)| = 2.155842e-2`; carrier `1.08e-2` |
| S2 — a bent donor, predicted forward | **BRANCH (b)** | filed before the solve: `E_pred = −1.287e-3` on the 30°-bent DONOR at 2.9 Å (field `−2.541e-3`, contact H–O `−5.111e-3`, H–H `−2.2e-5`, walls O–O `+2.252e-3`, H–O `+3.843e-3`, H–H `+2.9e-4`); the solve: `ΔE_exact = −3.0313e-3`, `Converged`, 235 iterations, residual `8.6e-11`, 23,660 core-seconds; miss `1.74e-3` (57.5 %) against `7.58e-4` — (a) fails; `|wall − E_exch(bent donor)| = 6.7e-4` against `7.58e-4` — (b): the wall transfers to the bent donor, the miss is the contact term's |
| S3 — retention | **not run (the letter of G-N0)** | the unstaked diagnostic (§3): no collapse, no bond |

## 1. The exchange at contact

| family | R_OO (Å) | 0° | 30° | 60° | 90° | 120° | 180° |
|---|---|---|---|---|---|---|---|
| close | 2.1 | 258 | 267 | 285 | 301 | 341 | 534 |
| close | 2.3 | 124 | 129 | 138 | 147 | 173 | 299 |
| close | 2.5 | 58 | 61 | 65 | 70 | 86 | 164 |
| twist | 2.3 | 124 | 120 | 131 | 243 | 507 | 297 |
| twist | 3.0 | 8.0 | 7.7 | 9.5 | 26.3 | 68.9 | 33.3 |

(mHa, undeformed `E_exch`). Two structures in the numbers: exchange rises monotonically with
tilt in the tilt family and peaks at 120° in the twist family; and the exponent steepens
inward — between 2.5 and 2.1 Å the linear exchange rises by 4.4 while 2.5 to 2.9 falls by
4.8 over the same 0.4 Å, so no single exponent serves both ends within a twentieth.

## 2. The identity's boundary

The exact dimer at 2.1 Å is `+77.2` mHa; at 2.3 Å `+23.3`. At 2.1 Å the donor hydrogen is
1.97 bohr from the acceptor oxygen and 1.94 from its own, both near the O–H curve's minimum
(1.83), and the closure reading gives it to whichever oxygen binds it lower — the acceptor,
by a hair. Both waters dissolve as units and the engine serves the bare tables, which read
`+86.4` mHa there against the exact `+77.2` (11 % off, the closest the bare law has been to
the exact dimer anywhere). That is the proton-transfer regime, and the seam law does not
claim it: the identity is the law's domain, and 2.1 Å is outside it. The 2.3 Å node
(H···O 2.35 bohr) is inside, and the engine and formula agree there to `1e-16`.

## 3. The diagnostic, unstaked

`field7_probe.rs` on the FIELD-8 record, the dimer, 22,000 frames: at 293 K and 150 K the
closest cross-unit H···O over the whole run is the starting 3.56 bohr; the ledger closes to
`1e-7`; the pair separates (O–O 18.9 bohr at 293 K, 9.5 at 150 K, no hydrogen bond at the
end). The law found no hole because it has none; it lost the bond because at the minimum it
is too weak (C1's misses at 2.7–3.1 Å are the same reading).

## 4. S2 — the bent donor

The donor's O–H swung 30° off the axis at 2.9 Å binds by `−3.03` mHa on the exact solver,
less than the line's `−5.48` (bending the donor costs more than bending the acceptor, which
gained). The law predicted `−1.29`. The three-class wall reads `6.39` there against the
undeformed exchange `5.72`, inside the tolerance, so for the second time in three campaigns
the wall transfers across an orientation it was not fit on and the miss belongs to the
contact term, `1.7` mHa too weak on this geometry exactly as it is `1.4` too weak on the
line at the same separation (C1's misses). The remainder's shape across orientations is the
open problem, not the wall's.

## 5. FIELD-9, named

Three corrections, each from a finding above. (1) The wall's fit range is the range the
dynamics visit, not the range the instrument can reach: the tilt and twist families at
`R_OO ≥ 2.5` Å (the linear dimer is already repulsive at 2.5), with the close readings kept
for one purpose — a BOUNDEDNESS gate below the fit range, replacing G-N0's monotone letter:
each class potential's minimum on `[0.5, r_min]` no lower than its value at `r_min` by more
than `kT`, and positive at 0.5 bohr. (2) The exponent's drift measured and DECLARED as the
tolerance — per class, the log-linear exponent fit on each family separately, the spread
between families being the drift — instead of a twentieth typed. (3) No fit point beyond the
identity's boundary: the exact nodes are those where the engine finds the staked units, and
the freeze says so before it solves. Then G-C1, a bent donor (or the one this freeze solves,
if it lands), and the arms read beside their drift. The state point's liquid (LIQUID-1,
frozen and conditioned) waits on the first law that passes.

## 6. Bookkeeping, declared

- The delegate's watcher on the close solves did not wake it; the lead ran `fit` by hand
  and launched `predict`. The records are the runner's either way.
- The freeze's overlap window, monotone gate and 2.1 Å node were the lead's stakes, each
  typed where the instrument or the identity could have told the number; entered in the
  standing lesson.
- No number enters from outside the engine and its own solver.
