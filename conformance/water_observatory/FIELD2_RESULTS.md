# FIELD-2 — results

*Freeze `FIELD2_PREREG.md` (8d02335, alone). Runner `holon-render/examples/field2_hbonds.rs`
(5175c1a; this commit adds the `-- probe` and `-- refs` diagnostics and refreshes the bond
verdicts before M1 — a change that moved no number, below). Instrument: FIELD-1's field
(`field.rs`, `sim.rs`) and the rung-1 lens (`holon_lens::lens::hbonds`). JSON
`field2/expectation.json`, `field2/arms.json`.*

## The verdict, first

**S1 reads BRANCH (c) BY ITS LETTER on both systems at both temperatures, and the campaign
is VOID IN SUBSTANCE: the field was never applied to a hydrogen-bonded configuration.**
M1 — the field's binding at the staked start, measured first by design — is EXACTLY ZERO
on the dimer, the tetramer and the square. Not small: zero. The charge assignment
(FIELD-1 AMENDMENT 1: a water unit is an oxygen with exactly two bonded hydrogens, neither
bonded to another oxygen, by the engine's own pair verdict) finds no unit at any bonded
start, because the O···H pair table's well at the hydrogen-bond distance reads BONDED by
the engine's two-body criterion — donor H to acceptor O at 3.556 bohr: `E_rel = −0.0171`
hartree inside `r_outer = 3.56` — so the donor hydrogen belongs to two oxygens and both
units dissolve. The field's sector was empty on the object under test. Both plants' carriers
fail (`|E_field(start)| = 0` against `≥ 1e-4`; square − ring `= 0` against `≥ 1e-3`), which
VOIDs both plants exactly as M-PLANT-SECTOR says it must, and the same fact voids the ON
arms as a test of the field: bit for bit, the ON dimer and the OFF dimer are the same
trajectory until the molecules have parted (the probe's 400 frames), and the field's first
transition in every ON arm is the moment the units REAPPEAR at separation.

The frozen expectation rule did not catch this. It read "binding under `kT` ⇒ expected to
break" and turned an empty sector into a prediction; the arms then confirmed the
prediction for a mechanism they never exercised. That is a design defect of the freeze,
registered as **M-EMPTY-SECTOR** (`conformance/gravity/MISFITS.md`, armed in the audit).

**A second finding, unstaked and reported as one: the bare force law REPELS at the staked
hydrogen-bond geometry.** The dimer at the start, against the same dimer at 40 bohr, is
`+21.0` mHa uphill — pair sector `−20.9` mHa, three-body sector `+41.9` mHa — with the
three-body figure coming from cross-molecule triples the monomer's (O,H,H) surface and the
H₃ surface are asked to evaluate. One such triple alone (the acceptor oxygen with the
donor's two hydrogens at 3.56 and 6.05 bohr) carries `+20.1` mHa, larger than the whole
hydrogen bond. The released potential is what the arms measured as temperature: the dimer
heats from 198 K to 543 K in 400 frames under a thermostat of `τ = 2000`, and the tetramer
arms average 346–505 K against 150 and 293 K targets. The OFF arms' `f = 0` is not the
absence of a bond-holding force; it is a repulsion. FIELD-1's fourth suspect — the O···H
table's role at bond distance — is therefore measured in two places at once: in the
IDENTITY (the pair verdict bonds across the seam) and in the FORCE (the closure surfaces
serve across the seam).

| gate | verdict | the number |
|---|---|---|
| G0 — the books, four ON arms | **PASS** | receipt columns sum to `w_ext` and the momentum residual is under its bound on all four; honest `drift_peak` 2.7e-6, 3.4e-6, 1.4e-5, 1.8e-5 against enabling transitions of 2.8e-3, 2.9e-3, 1.3e-2, 1.3e-2 hartree (ratios ≤ 1.4e-3, stake ≤ 0.1) |
| G1 — the start is bonded | **PASS** | lens reads 1 on the dimer (stake ≥ 1), 4 on the tetramer (stake ≥ 3) |
| M1 — the field's binding at the start | **0.0 exactly, all three starts** | `E_field(start) = E_field(separated) = 0`; the rule wrote "break" at 293 K and 150 K on every system — the rule's reading, VOID by M-EMPTY-SECTOR |
| M2 — the charge over the thermal grid | **0.3426** | `q_H` from 0.1747 to 0.3107 around the pin's 0.2314 (r ± 0.15 bohr, θ ± 15°); above the 0.10 line: geometry dependence is MATERIAL for FIELD-3 |
| S1 — dimer, 293 K / 150 K | **(c) by letter; VOID in substance** | `f_OFF` 0.0000 / 0.0000, `f_ON` 0.0000 / 0.0000 of 20,000 frames; mean T 321 / 180 K (OFF), 314 / 173 K (ON) |
| S1 — tetramer, 293 K / 150 K | **(c) by letter; VOID in substance** | `f_OFF` 0.0000 / 0.0000, `f_ON` 0.0000 / 0.0000; mean T 496 / 356 K (OFF), 485 / 346 K (ON) |
| plant (i) — the sign | **VOID** (carrier fails) | `|E_field(start)| = 0` against `≥ 1e-4`; `f = 0` on both systems, the inequality holds trivially and says nothing |
| plant (ii) — the start | **VOID** (carrier fails) | `E_field(square) − E_field(ring) = 0` against `≥ 1e-3`; `f = 0` |

The prereg's own VOID clause (M-VACUOUS-SUCCESS: "an arm whose molecules never came within
the lens's reach") does not fire — every arm started inside reach (G1). The void is the
field's, not the lens's.

## The diagnosis, in the engine's own numbers

**The assignment at the dimer's start** (`-- refs`; charges `[0, 0, 0, 0, 0, 0]`). The
bonded pairs by the engine's verdict, with the acceptor's atoms numbered 3–5:

| pair | r (bohr) | `E_rel` (Ha) | `r_outer` | what it is |
|---|---|---|---|---|
| 0–1, 0–2 | 1.944 | −0.122, −0.120 | 2.04, 2.08 | the donor's two O–H bonds |
| 3–4, 3–5 | 1.944 | −0.121, −0.122 | 2.05, 2.06 | the acceptor's two O–H bonds |
| 1–2, 4–5 | 2.906 | −0.058, −0.059 | 2.92, 2.91 | the H–H pairs inside each water |
| **1–3** | **3.556** | **−0.0171** | **3.56** | **the hydrogen bond, read as a bond: donor H to acceptor O** |
| 1–4, 1–5 | 5.060 | −0.0009 | 5.10, 5.22 | the donor H to the acceptor's hydrogens, on the H–H curve's tail |

The verdict is two-body and correct as such: each of these pairs, alone, is a bound
system. Oxygen 3 therefore has THREE bonded hydrogens and hydrogen 1 has TWO bonded
oxygens, and the unit rule — exactly two, none shared — assigns nothing. The tetramer,
with four such contacts, assigns nothing. The rule was right for FIELD-1's parallel square
(7 bohr apart, nothing pointing at anything) and is blind on precisely the configuration
this campaign staked.

**The bare force law at the same start** (`-- refs`; the (O,O,H) class carries no surface
on this scene and its four triples are fenced; the sixteen served triples are the two
intra-molecular (O,H,H), ten cross-molecule (O,H,H) on the monomer's surface, four (H,H,H)
on the H₃ surface):

| configuration | `e_pair` (Ha) | `e_three` (Ha) | total (Ha) | triples served |
|---|---|---|---|---|
| monomer at the pin | −0.303917 | +0.017936 | −0.285980 | 1 |
| dimer at 40 bohr | −0.607833 | +0.035872 | −0.571961 | 2 |
| dimer at the staked start (R_OO 5.5) | −0.628707 | +0.077786 | −0.550921 | 16 (+ 4 fenced) |
| **interaction, start − 40 bohr** | **−0.020873** | **+0.041914** | **+0.021040** | |
| monomer + a lone O at 5.5 bohr, minus the monomer | −0.017748 | +0.020107 | +0.002359 | 1 cross (O,H,H) |

The pair sector binds the dimer by 20.9 mHa — the O···H curve's covalent tail at 3.56 bohr
is most of it — and the three-body sector repays that twice over. The lone-oxygen row
isolates the mechanism: a single (O,H,H) triple with the oxygen 3.56 and 6.05 bohr from the
two hydrogens is, on the monomer's surface, a water molecule with both bonds broken, and
the surface's residual there is +20 mHa. The surface is doing what it was built to do —
cancel the O–H pair curve where the pair curve is not the molecule — but it is being asked
across the seam, on a triple that is not a molecule at all.

**The trajectory** (`-- probe`, dimer, OFF and ON bit-identical): frame 0 R_OO 5.50, O···H
3.56, T 198 K, `e_three` +0.0778; frame 400 R_OO 5.80, O···H 3.97, T 543 K, `e_three`
+0.0385; the lens still reads 1 (its O···H bound is 4.63 bohr) and the pair 1–3 still
reads bonded. The 0.010 hartree the potential lost is the 345 K the six atoms gained. The
thermostat's coupling (`dt/τ ≈ 5e-4` per step) removes it over thousands of frames, which
is why the tetramer arms still average 350–500 K over the counted window.

## What (c) says here, and what it does not

By its letter, (c) says "fixed charges do not hold water's hydrogen bond in this engine at
all; FIELD-3 stakes the O···H table's role at bond distance first." The second half is
exactly right and is now measured rather than suspected. The first half was NOT tested:
no fixed charge acted on any bonded configuration in any arm. Per rule 7 the branch is
recorded as read — (c) — with its substance marked VOID, and the null is not cashed as a
statement about the field.

What the campaign did measure, and banks:

1. **The unit is not a pair verdict.** A two-body bond criterion cannot serve as a
   molecule identity where molecules touch; at the hydrogen bond it bonds across the seam.
   This is the engine's own "16 atoms, 120 bonds" note (`Sim::clusters`) arriving at the
   field. The identity must be a CLOSURE reading — the object the dynamics never splits —
   and the cheapest one is the strongest-bond assignment (each hydrogen to the oxygen it is
   most bound to; a unit is an oxygen with exactly two).
2. **The closure surfaces serve only within the closure.** The (O,H,H) surface is the water
   molecule's own residual and the O–H pair curve is the radical's; between two units they
   are +42 mHa and −21 mHa of the wrong thing. Between units, the holon has exactly five
   channels (OBJECT.md rule 10) and the seam programme (EMBED/SEAM) already showed the far
   field is 98–99.6 % channel 1. The atom-level many-body expansion across a hydrogen bond
   is not converged at third order and will not be at fourth: it is the wrong expansion
   there, and the ledger names the right one.
3. **M2 = 0.34** is material: the charge moves by a third over thermal geometries, so a
   fixed charge is at best a 293 K average.

## FIELD-3, named (not frozen here)

In the order the evidence puts them: (1) the unit as a closure reading (strongest-bond
assignment), with M1 re-measured on the same three starts — the carrier for everything
after; (2) the closure surfaces confined to within-unit triples and pairs, cross-unit
contacts served by the ledger's channels (1: the field; 5: an exchange wall; 4: dispersion
as harvested), with the water dimer's binding curve against the seam programme's exact
dimers as the referee; (3) polarisation and the charge's geometry dependence. A freeze that
reads M1 = 0 again VOIDs itself before its arms run.

## Bookkeeping, declared

- M1 was re-derived AFTER the arms had run once. The first run's runner read the bond
  verdicts before `refresh_pairs` had written them; the suspicion that this produced the
  zero was wrong — the call was added, `expectation.json` came back bit-identical, and the
  zero is the assignment's own reading. The arms were re-run with the corrected runner;
  every physics field of `arms.json` is bit-identical to the first run (only the self-timing
  `seconds` field moved, on two arms).
- Wall time: all eleven arms in 0.6–2.1 s each. Separated molecules leave the pair loops
  nothing to do; the speed is the finding.
- No number enters from outside the engine. The lens is rung 1's frozen instrument.
