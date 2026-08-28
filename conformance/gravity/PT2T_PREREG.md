# Pre-registration — PT-2T: the deficit-angle LADDER

*Frozen 2026-08-28, committed ALONE. Rung 5's next brick after the
orientation leg: the punctured-torus instrument re-posed over the binary
tetrahedral group 2T ⊂ SU(2) (order 24) — the densest of the small SU(2)
subgroups on the road to the continuous 2+1 connection theory. The
commutator subgroup of 2T is Q8, whose classes give a genuine MULTI-RUNG
mass spectrum: {1} (deficit 0), {−1} (deficit 2π), {±i}, {±j}, {±k}
(deficit π each) — the discrete mass–deficit-angle relation with more
than two rungs, where D4's had exactly two.*

misfits: contacts M-NONBIJECTIVE-STEP (the refined lifts are the PT-1B
both-edges construction with explicit inverses, bijectivity checked by
exhaustion before any gate), M-FIXED-POINT-TRAJECTORY (no trajectory
closure gate is staked; conservation gates are per-arm), M-GAUGE-LAUNDER
(mass labels are exact classes used as constraints; 2T is ambivalent on
some classes and that is immaterial here — no orientation claim is
staked), M-PLANT-OBS and M-PLANT-SECTOR (plants as PT-1B, carriers and
sectors named, each asserted nonzero in the sector the plant acts on),
M-FINAL-VIEW-COLLISIONS (the D4g-analogue enumerates the channel's OWN
collisions), M-ONE-MODEL-DELTA, M-NULL-MISSTAKE, M-PROBE-EIGENSTATE,
M-STALE-INSTRUMENT (instrument with results), M-BARE-CHARGE and
M-COND-PROBE and M-ELECTRIC-BASIS and M-RING-MIXING and
M-GAUGE-UNIFORM-MOMENTUM and M-HOMOG and M-KINEMATIC-NONLOCAL and
M-LOOP-BLIND (not otherwise contacted: no matter field, no electric term,
no locality or momentum-channel claim).

## Model

The once-punctured one-vertex torus over 2T, built in-instrument from the
quaternion presentation (generators i, ω = (−1+i+j+k)/2 as unit
quaternions with integer/half-integer coordinates, exact arithmetic in
Z[1/2]⁴ with the group law verified associative and of order 24 by
exhaustion). Puncture holonomy p = [g_a, g_b]; mass sector = conjugacy
class of p. Dynamics: the same Dehn generators as ADM-1/PT (verified
bijective by exhaustion). Refined instance: PT-1B's both-edges lift over
2T (24⁴ = 331,776 configurations, exact integers).

## Gates (all EXACT)

- **G0**: the class(1) sector nonzero and Gauss-held. witness: none
- **L1 — THE LADDER**: the realizable mass spectrum is EXACTLY the set of
  classes contained in [2T, 2T] = Q8 — five classes, deficits
  {0, 2π, π, π, π} — and every class outside Q8 has an EXACTLY EMPTY
  physical sector (the discrete Gauss–Bonnet, now with five rungs).
  witness: none (the commutator-subgroup computation is in-instrument by
  exhaustion; a Lean brick is named, not claimed)
- **L2 — conservation**: the puncture class is conserved by the
  mapping-class dynamics on every realizable sector. 
  witness: closed_view_inherits_conservation
- **L3 — refinement**: L1 and L2 identical on the both-edges refined
  instance, bijectivity and refined Gauss checked first.
  witness: none (measured gate)
- **B3**: Gauss on every trajectory state, both charts. witness: none

## plants (carrier and sector per M-PLANT-SECTOR)

- **(i) forbidden-mass control with live twin**: a class OUTSIDE Q8 (an
  order-3 class) must project to EXACTLY ZERO while the {−1} twin projects
  nonzero. Carrier: the twin; sector: its projected weight, nonzero.
- **(ii) broken twist**: the non-covariant twist must break Gauss on a
  single-orbit carrier (the PT-1B construction). Carrier as there; sector
  total, nonzero.
A missed plant VOIDs.

## Meaning

L1+L2+L3 ⇒ "the mass–deficit-angle relation holds with a five-rung
spectrum on the first SU(2)-subgroup model, refinement-stable" — the
geometry sequence's last stop before the subgroup chain toward continuous
2+1 gravity. Successors: the 2O/2I refinement of the ladder, then the
continuum limit question.
