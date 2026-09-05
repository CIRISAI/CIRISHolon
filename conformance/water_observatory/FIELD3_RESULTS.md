# FIELD-3 — results

*Freeze `FIELD3_PREREG.md` (01a92f5, alone); AMENDMENT 1 (39ba587, before any node's record
existed: the price node's ceiling in core-seconds). Instrument: `holon-render/src/seam.rs`
and the hooks in `sim.rs` (the closure assignment, the seam rule, the wall, the transitions),
`channel.rs` (`Row::Seam` appended), `checkpoint.rs` (v7), `lib.rs` (doors); gates
`tests/seam.rs`; the harvest `examples/field3_harvest.rs` (solve / fit / predict); the arms
`examples/field3_hbonds.rs`; FIELD-2's scenes moved verbatim to `tests/common/field2_scenes.rs`.
JSON under `field3/`. The engine change was adversarially reviewed before its gates were read
(§6).*

## The verdict, first

**The identity and the seam rule land; the harvest reads BRANCH (c) by the freeze's letter,
and the seam arms therefore do not run.** The closure reading finds the water units FIELD-2
could not (2 and 4 where FIELD-2's rule found none), the field binds the starts by 4 and 13
`kT`, and the seam rule drops the +42 mHa the monomer surface was serving across the seam,
exactly, with its books closed and every conservation gate green. The exact dimer's residual
over the field has a definite shape — a wall inside 2.8 Å (`+9.6` mHa at 2.5 Å) and a MISSING
ATTRACTION beyond it, `−1.3` to `−1.5` mHa at 2.9–3.1 Å, a third to a half of the field
itself, decaying faster than any power (exponential constant rising from 1.0 to 1.5 per
bohr) — and only two of six nodes are positive, which is (c): no wall is harvested from a
two-point prefix, the point-charge field is not the seam's whole electrostatics, and FIELD-4
stakes the density field at the seam (charge penetration, EMBED-2's instrument) with
induction as its second suspect. The forward prediction (S2) and retention (S3) were not run,
as the letter says. An UNSTAKED diagnostic (§4, not cashed) with a two-node wall shows the
seam law in the right regime: the dimer holds at 150 K (`f = 0.68`) and mostly breaks at
293 K (`f = 0.03`), the ring 0.90 / 0.47 — a `kT`-sized attraction is what is missing, and
the harvest says where.

| gate | verdict | the number |
|---|---|---|
| G-A1 — the units exist where FIELD-2 found none | **PASS** | closure units dimer 2, ring 4, square 4 (FIELD-2's rule: 0, 0, 1); field binding at the start dimer `−4.083e-3`, ring `−1.202e-2` hartree (stake: negative, ≥ 1e-4); square `−6.42e-4` |
| G-A2 — the identity where nothing is contended | **FAIL by letter, read** | on FIELD-1's four-water walled scene the two rules AGREE at step 2,000 and DISAGREE at the first frame (2 of 125 frames): the pair verdict bonds hydrogen 7 to oxygen 9 across molecules at 5.686 bohr (`E_rel = −1.3e-4`, inside `r_outer = 5.86`), so FIELD-1's rule charged two of the four waters there; the trajectories diverge and `e_field` at step 2,000 is not bit-identical |
| G-B0 — the seam off is the identity | **PASS** | checkpoint BYTES identical over 2,000 steps, seam enabled-then-disabled before the first step (the two postings cancel exactly from a zero column); `Row::Seam` reads `0.0`; the receipt moved on 9 lines for G-A2's reason, not this row's (§5) |
| G-B1 — the books close with the seam on | **PASS** | dimer: columns close (`thermostat +3.20e-3`, `field −4.08e-3`, `seam −2.04e-2` = `w_ext −2.12e-2`), drift peak `1.25e-7` against a tenth of the largest transition `2.04e-3`; ring: `seam −2.85e-1`, drift peak `3.35e-7` against `2.85e-2`; 9 / 54 pairs and 18 / 216 triples dropped, 1 / 6 O–O walls (declared test coefficients, no wall harvested) |
| G-B2 — momentum | **PASS** | net internal force `6.8e-18` against a `2.9e-2` scale; residual `2.6e-14` under `7.9e-11`; `|F_wall(start)| = 8.2e-4` |
| G-B3 — the wall is the derivative | **PASS** | worst relative `2.4e-9` (stake 1e-8) over every oxygen and component |
| G-B4 — the closures contribute exactly nothing across the seam | **PASS (EXACT)** | cross-seam closure contribution pair `+0.0`, three-body `+0.0` to the last bit (bare law: −20.9 / +41.9 mHa); 9 pairs and 18 triples dropped on the dimer |
| G-C0 — the price (A1) | **ADMITTED** | 2.9 Å node: `1,002,001` determinants, 193 Davidson iterations, residual `9.0e-11`, `52,739` core-seconds (ceiling 57,600; floor 1,450); by the freeze's original wall-clock letter it read REFUSED at 2,315 s on 24 threads — the amendment's reason |
| G-C1 — the harvest is the engine's arithmetic | **not run** | no wall was harvested (S1 (c)); the engine's field on the six geometries entered the residual and is in `wall.json` |
| S1 — what the residual is | **BRANCH (c) by letter** | positive nodes: 2.5 Å (`+9.576e-3`), 2.7 Å (`+1.017e-3`); negative from 2.9 Å: `−1.346e-3`, `−1.487e-3`, `−8.40e-4`, `−3.57e-4` — a positive prefix of 2, under the 3 the fit needs; `r(2.5) > 0` (the (c) clause that fired is the count, not the sign at contact) |
| S2 — the forward prediction | **not run** | no `prediction.json` was filed (no wall); the flipped node was not solved |
| S3 — retention under the seam law | **not run** | the letter of (c): "the seam arms do not run"; the OFF arms reproduce FIELD-2's numbers exactly in the diagnostic (§4) |
| plant (i) — the sign | **not run** | acts on the wall; no wall |
| plant (ii) — the triples served across the seam | **FIRES** | cross-unit three-body sum returns to `+4.191372e-2` (FIELD-2: +0.041914), 0 triples dropped |
| plant (iii) — the reaction dropped on the wall | **FIRES** | the internal force sum exceeds `1e-6` of its scale and the momentum residual its bound; carrier `|F_wall| = 8.2e-4` |

## 1. The identity (part A)

The closure reading (`Sim::assign_units`): each hydrogen belongs to the oxygen it is most
bound to by the engine's O–H curve, inside the curve's tabulated reach, with no sign
threshold; a unit is an oxygen with exactly two. It replaces FIELD-1 AMENDMENT 1's rule for
the field's charges, and FIELD-1's rule survives as a reading (`units_by_pair_verdict`) so the
two can be compared.

| start | closure units | FIELD-2's rule | field binding at the start | in `kT` (293 K) |
|---|---|---|---|---|
| dimer (R_OO 5.5) | 2 | 0 | `−4.083e-3` Ha | 4.4 |
| ring of four | 4 | 0 | `−1.202e-2` Ha | 13.0 |
| FIELD-1's square | 4 | 1 | `−6.42e-4` Ha | 0.7 |

The freeze's parenthetical "(FIELD-2's rule: 0, 0, 4)" was wrong for the square — the
measured count is 1 — a transcription from FIELD-1's probe, not a stake. G-A2's letter
("the two rules agree after 2,000 steps on FIELD-1's scene, `e_field` bit-identical") holds
for the assignment and fails for the energy: the rules DISAGREE at the first frame, where
FIELD-1's rule bonds hydrogen 7 to oxygen 9 across molecules at 5.686 bohr on the O–H
curve's tail (`E_rel = −1.3e-4` inside `r_outer = 5.86`), leaving two of four waters
uncharged; they agree again from the third frame (2 of 125 frames differ), but the
trajectories have diverged. That is FIELD-2's diagnosis appearing on FIELD-1's own scene,
and it moved the channel receipt (§5).

## 2. The seam rule (part B)

With the seam on, the pair tables serve only within a unit (or a free atom) and the three-body
surfaces only within a unit; cross-unit contacts are the ledger's. On the dimer at FIELD-2's
start the seam drops 9 pairs and 18 triples, and the closure sector's cross-seam contribution
reads EXACTLY zero (G-B4) where the bare law had `−20.9 / +41.9` mHa. The switch itself posts
`−2.04e-2` hartree to `work.seam` on the dimer — FIELD-2's +21.0 mHa being dropped — so the
ledger and the diagnosis agree to three digits; on the ring it posts `−2.85e-1`. Plant (ii)
(the surfaces served across the seam) returns FIELD-2's `+0.041914` (G-B4's row). The books
close (G-B1), momentum is conserved and plant (iii) breaks it (G-B2), the wall is the
derivative of its energy (G-B3) — these three with DECLARED test coefficients (`A = 0.5`,
`b = 1.2`), since no wall was harvested; they are conservation properties that hold for any
`(A, b)` and the gate output names which it used. Refusals named: the seam with an acuity
frame, with a far sector, with the many-body sector on.

## 3. The harvest (part C)

Six exact nodes on the seam programme's solver (`1,002,001` determinants each, all
`Converged`, residuals `≤ 1e-10`, 181–223 Davidson iterations, 16,557–52,739 core-seconds),
the engine's field on the same geometry with the closure assignment (two units at every node)
and the pin charge:

| R_OO (Å) | R (bohr) | ΔE_exact (Ha) | E_field (Ha) | r = ΔE − E_field (Ha) | r / \|E_field\| |
|---|---|---|---|---|---|
| 2.5 | 4.724 | `+2.507e-3` | `−7.069e-3` | `+9.576e-3` | +1.35 |
| 2.7 | 5.102 | `−4.320e-3` | `−5.337e-3` | `+1.017e-3` | +0.19 |
| 2.9 | 5.480 | `−5.480e-3` | `−4.134e-3` | `−1.346e-3` | −0.33 |
| 3.1 | 5.858 | `−4.757e-3` | `−3.270e-3` | `−1.487e-3` | −0.45 |
| 3.4 | 6.425 | `−3.217e-3` | `−2.377e-3` | `−8.40e-4` | −0.35 |
| 3.7 | 6.992 | `−2.141e-3` | `−1.784e-3` | `−3.57e-4` | −0.20 |

The exact dimer's minimum is near 2.9 Å (`−5.48` mHa); the point-charge field alone has no
minimum inside 3.7 Å and binds `−4.1` mHa at 2.9 Å. The residual is a WALL at contact
(`+9.6` mHa at 2.5 Å, where the exact curve is already repulsive) and an ATTRACTION the field
lacks from 2.9 Å out, largest at 3.1 Å. Its decay beyond 3.1 Å, read as the freeze's (b)
would have read it: log-log slopes `−6.2` (3.1→3.4) and `−10.1` (3.4→3.7), an exponential
constant rising from `1.0` to `1.5` per bohr — faster than any power at the outer interval,
which the freeze names charge PENETRATION (the density field at the seam) — with the inner
interval in the band it names induction. A two-node exponential through 2.5 and 2.7 Å gives
`b = 5.93` per bohr, a wall steepened by the attraction already present at 2.7 Å; it is a
diagnostic constant (§4), not a harvest. The record is `field3/wall.json` (`a = b = 0`,
`s1_branch = "c"`).

## 4. The hydrogen bond, re-asked (part D)

Not run under the freeze. **Diagnostic, unstaked, not cashed** (`field3_hbonds.rs` into a
scratch directory, the two-node wall `A = 1.41e10`, `b = 5.93`, wall `9.6e-5` Ha at 5.5 bohr):
the expectation rule wrote "hold" at both temperatures (binding `−3.99e-3` dimer, `−1.16e-2`
ring, units 2 / 4); the OFF arms reproduce FIELD-2's arms field for field (`f` 0.0000, `T`
321 / 180 / 496 / 356 K); under the seam law:

| arm | f | n̄ | mean T |
|---|---|---|---|
| dimer 293 K | 0.034 | 0.03 | 276 K |
| dimer 150 K | 0.678 | 0.68 | 145 K |
| ring 293 K | 0.466 | 0.94 | 269 K |
| ring 150 K | 0.903 | 2.16 | 142 K |

Books closed and momentum conserved on every arm. Read as a diagnostic only: the seam law
binds and `kT` unbinds it at 293 K — the shape S3 (b) would have had — and the harvest's
missing `−1.4` mHa (1.5 `kT`) is the size of the gap. It is what FIELD-4 stakes.

## 5. The receipt

The channel receipt (`tests/data/channel_ledger.receipt`) moved on 9 of 46 lines, all in the
`water4` block — FIELD-1's four-water walled scene with the field on after 2,000 steps:
`e_kin`, `e_pair`, `e_three`, `e_field`, `energy`, `ledger`, `w_ext`, `work.field`, `drift`.
The cause is part A, not the appended row: under FIELD-1's rule that scene had two charged
waters at its first frame and four later, with the transitions that implies; under the
closure reading it has four throughout. The appended `Row::Seam` changed no line. Re-banked
under this freeze's cause line, and FIELD-1's record carries the correction.

## 6. The review, and what it changed

The engine change was reviewed adversarially before its gates were read. Findings applied:
the wall's virial posted `+r·F` where the engine's convention is `Σ r·dU/dr` (sign flipped);
the field's virial had the same error since FIELD-1 (`+E` for a Coulomb term whose
`r·dU/dr` is `−E`) — corrected, entered in FIELD-1's record, and read by nothing but
`pressure()`; the seam and an acuity frame would double-post through the shared pair loop —
refused by name in both orders (`SeamRefusal::AcuityFrameSet`); the far and many-body
sectors are not fenced by the seam rule — refused by name while either is on
(`FarSectorDeclared`, `ManyBodySectorOn`); the fence's R-3 guard ran after the seam branch —
reordered; a per-pass counter went stale on early returns — reset first; the assignment
seeded its search at `u = 0` (a hydrogen on its oxygen's repulsive wall would have gone
free) — seeded at `+∞` so the rule is the freeze's "lowest `u` inside the reach"; the switch
counted a transition only when its energy was nonzero — counted always; running drop totals
added beside the per-pass counts, because a last pass can be dissociated and a vacuity
check must read a life. Findings not applied, recorded: the fence under the seam is O(N³)
per pass (campaign scale only; the census formula cannot separate dropped from fenced); the
workbench's ledger panel does not yet show `e_seam`.

## 7. Bookkeeping, declared

- The harvest ran on 24 threads pinned to cores 0–23 with the engine's builds on the other
  eight; AMENDMENT 1 re-priced G-C0 in core-seconds before the first node's record existed
  and is the reason the price node reads ADMITTED. The first solve process stopped at the
  wall-clock refusal after writing the node; a rebuilt runner skipped it and continued.
- The G-B1/B2/B3 gates ran with declared test coefficients because S1 harvested no wall; the
  gate output names them. G-B4 and plant (ii) use `SeamModel::NO_WALL` as frozen.
- The arms runner and the seam gates were written against the frozen API by a delegate and
  the lead respectively; the engine change was reviewed adversarially by a third (§6).
- No number enters from outside the engine and its own exact solver. The two diagnostics
  (`examples/seam_probe.rs`, the G-A2 frame count; the §4 arms) are labelled as such.

## FIELD-4, named

The density field at the seam: EMBED-2's Coulomb-only frozen-density embedding evaluated on
the six linear geometries, its residual over the exact dimer measured against the point
charges' — the penetration this campaign found is the difference between the two; then
induction (the fixed point EMBED-1 computes and the force law does not) on what remains; the
wall harvested from the residual over the DENSITY field, where it should be a clean
exponential; then S2 and S3 as frozen here, on that wall. A freeze that again reads a
positive prefix under three VOIDs its harvest before its arms.
