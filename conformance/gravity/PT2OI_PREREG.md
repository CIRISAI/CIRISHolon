# Pre-registration — PT-2OI: the deficit ladders of 2O and 2I

*Frozen 2026-08-28, committed ALONE with the recon module
(`binary_groups.py`) whose numbers it stakes. The advance predictions were
derived BEFORE this freeze by exhaustive computation, cross-checked by an
independent enumeration route and by character theory (Frobenius counts,
hand-matched), and spot-run in the lead session: recon crosscheck ALL
PASS. The instrument now tests them as frozen gates.*

misfits: contacts M-GAUGE-LAUNDER twice over — (a) mass labels are exact
conjugacy CLASSES; (b) the recon found 2O SPLITS the π deficit across a
realizable class (size 6) and an exactly-empty class (size 12), so a gate
staked on deficit ANGLES would launder the wrong sector: mass is the
CLASS, deficit a derived non-injective label, staked accordingly.
M-NONBIJECTIVE-STEP (base Dehn lifts verified bijective by exhaustion; the
2I refinement leg is certified by the pullback lemmas below INSTEAD of a
dense run, and each lemma is verified by exhaustion). M-PLANT-OBS and
M-PLANT-SECTOR (2I is PERFECT — verified, pinning it as SL(2,5) — so the
forbidden-mass plant has no candidate class and is REPLACED by the
closed-torus control, with carriers and sectors named; each plant's
carrier is asserted nonzero in the sector the plant acts on).
M-FINAL-VIEW-COLLISIONS and M-FIXED-POINT-TRAJECTORY (no trajectory-
closure gate staked; conservation is per-arm). M-ONE-MODEL-DELTA,
M-NULL-MISSTAKE, M-PROBE-EIGENSTATE, M-STALE-INSTRUMENT (recon module,
instrument and results all committed), M-BARE-CHARGE, M-COND-PROBE,
M-ELECTRIC-BASIS, M-RING-MIXING, M-GAUGE-UNIFORM-MOMENTUM, M-HOMOG,
M-KINEMATIC-NONLOCAL, M-LOOP-BLIND (not otherwise contacted).

## Advance predictions (recon, exact, staked verbatim)

- **2O (order 48, ring Z[√2], exhaustively verified)**: commutator set =
  commutator subgroup = 2T (width 1). Spectrum: **5 rungs**, deficits
  {0, 2π/3, π, 4π/3, 2π}, with EXACT per-class occupancies of the 2304
  configurations: 384, 864, 672, 288, 96 — and classes at deficits
  {π/2, 3π/2} plus the SIZE-12 π-class all EXACTLY ZERO.
- **2I (order 120, ring Z[φ], exhaustively verified PERFECT ⇒ SL(2,5))**:
  every one of the 9 classes realizable — **no forbidden sector** —
  deficits {0, 2π/5, 2π/3, 4π/5, π, 6π/5, 4π/3, 8π/5, 2π}, occupancies
  1080, 2400, 4320, 720, 1920, 2400, 720, 720, 120 of 14400.
- Ladder across the sequence: D4: 2 → 2T: 3 → 2O: 5 → 2I: 9.

## Gates (all EXACT)

- **G0**: identity-class sector nonzero and Gauss-held, both groups.
  witness: none
- **L1 — the ladders, quantitatively**: the per-class puncture-sector
  occupancies equal the staked integers EXACTLY, empty classes exactly
  zero, both groups. witness: none (recon derivation; character-theory
  cross-check recorded)
- **L1b — the split-π gate**: in 2O, the size-6 π-class is occupied (672)
  and the size-12 π-class is EXACTLY empty — deficit does not determine
  realizability; mass is the class. witness: none (measured gate)
- **L2 — conservation**: puncture class conserved under the mapping-class
  dynamics on every realizable rung, both groups.
  witness: closed_view_inherits_conservation
- **L3 — refinement**: 2O by DENSE both-edges lift (bijectivity first,
  refined Gauss on trajectory states, spectrum identical). 2I by the
  PULLBACK CERTIFICATE: lemmas L-A (fiber size |G|²), L-B/C (gauge
  equivariance of the splitting map), L-D (free midpoint actions), L-E
  (Dehn lifts intertwine), L-F (lifts bijective, explicit inverses), each
  verified by exhaustion at ≤|G|³ — jointly implying the dense refined
  spectrum, Gauss condition and sector ratios equal the base EXACTLY
  (composition stated in the recon note; validated dense at 2O).
  witness: none (measured lemmas; the composition argument is in
  DERIVATION.md, vendored with the recon)
- **B3**: Gauss on every trajectory state. witness: none

## plants (carrier and sector per M-PLANT-SECTOR)

- **(i) closed-torus control** (replaces forbidden-mass, which 2I's
  perfectness forbids): filling the puncture ([g_a,g_b] = 1) must leave
  EXACTLY the identity sector: 8 of 9 sectors exactly empty in 2I, 7 of 8
  in 2O, with live-twin counts EXACTLY 1080 (2I) and 384 (2O). Carrier:
  the closed-torus projected state; sector: the identity-class weight,
  asserted nonzero.
- **(ii) broken twist**: the non-covariant twist breaks Gauss on a
  single-orbit carrier. Carrier as PT-2T; sector total, nonzero.
A missed plant VOIDs.

## Meaning

All gates ⇒ "the mass–deficit ladder lengthens 2 → 3 → 5 → 9 along the
SU(2) subgroup chain, with exact integer occupancies predicted in advance;
at 2O the deficit label degenerates (mass is the class); at 2I — the
binary cover of the icosahedron, SL(2,5) — there is NO forbidden sector:
every deficit is sourceable." Successor: the continuum-limit question,
which no finite rung settles and none is claimed to.
