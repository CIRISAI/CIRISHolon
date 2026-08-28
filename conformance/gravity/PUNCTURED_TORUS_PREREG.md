# Pre-registration — PUNCTURED-TORUS-1: mass IS deficit angle, discretely

*Frozen 2026-08-28, committed ALONE. Requirement 2's core relation on the
discrete 2+1 instance, plus EINSTEIN-ADM-1's E3 retried on richer moduli.
In continuum 2+1 gravity a point mass makes a conical deficit with the
holonomy around the puncture equal to a rotation by the deficit angle —
mass and geometry are ONE datum. The discrete form: the puncture's
conjugacy class IS the deficit, and the constraint [g_a, g_b] = m is the
flatness-with-source (Einstein-with-matter) equation.*

misfits: contacts M-GAUGE-LAUNDER (mass labels are exact conjugacy classes
used as CONSTRAINTS, never class-marginal observables; the channel is the
orbit label as in ADM-1), M-ONE-MODEL-DELTA (defects are collision form),
M-NULL-MISSTAKE (conservation staked on what the dynamics conserves),
M-PROBE-EIGENSTATE and M-PLANT-SECTOR and M-PLANT-OBS (carriers and
sectors below), M-KINEMATIC-NONLOCAL and M-HOMOG (no locality claim),
M-BARE-CHARGE (the puncture is a gauge SOURCE, not a dressed matter field;
no dressing is claimed), M-COND-PROBE (dynamics is the mapping-class
action, inside the step), M-GAUGE-UNIFORM-MOMENTUM and M-ELECTRIC-BASIS
and M-RING-MIXING and M-LOOP-BLIND (no momentum or electric reading; the
dynamics is a permutation).

## Model

One-vertex once-punctured torus over D4: configurations (g_a, g_b), the
puncture holonomy DEFINED by the surface relation as p = [g_a, g_b].
Mass sector m (a conjugacy class): the states supported on
[g_a, g_b] ∈ m. Gauss: simultaneous conjugation. Dynamics: the punctured
mapping-class generators (the Dehn twists of ADM-1, which fix p up to
conjugation by construction). Refined instance: edge a subdivided
(g_a = g_a1 · g_a2, three edges, extra vertex), same relation.

## Gates (all EXACT)

- **G0**: for m = class(1), the projected sector is nonzero and Gauss
  held. witness: none
- **D1 — THE MASS–DEFICIT RELATION**: the mass spectrum is EXACTLY the
  commutator-realizable classes: the projected m-sector is NONZERO for
  m ∈ {class(1), class(r²)} and EXACTLY EMPTY for every other class of D4
  — the discrete Gauss–Bonnet: a one-handle surface can only source
  deficits lying in the commutator subgroup. The realized deficit angles
  are 0 and π, read off the class. witness: none (the commutator-subgroup
  computation is stated; a Lean brick `d4_commutator_subgroup` is named,
  not claimed)
- **D2 — MASS CONSERVATION**: the mapping-class dynamics preserves the
  puncture class: the m-sector weight is conserved at every step, for
  both realizable m. This is inheritance again — the source term of the
  2+1 Einstein equation conserved through the evolution.
  witness: closed_view_inherits_conservation
- **D3 — REFINEMENT**: D1 and D2 verdicts IDENTICAL on the refined
  instance. witness: none (measured gate)
- **D4g — off-shell defect, ADM-1 E3 retried**: on the m = class(r²)
  sector (richer than ADM-1's flat sector), the orbit-label channel's
  trajectory at depth 8: a firing collision ⇒ off-shell defect > 0; none
  ⇒ recorded not-refuted, NOT a fire. witness: collision_refutes_memoryless
- **B3**: Gauss on every trajectory state, both instances. witness: none

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on.
- **(i) forbidden-mass control**: the projector applied to a class-(r)
  seed must return EXACTLY ZERO — and the plant's conviction is the
  CONSTRUCTION's visibility: the same seed with m = class(r²) must return
  nonzero (so an empty result is the physics, not a dead projector).
  Carrier: the class-(r²) twin, sector: its projected weight, nonzero.
- **(ii) broken twist** (ADM-1B's): the non-covariant twist on the
  (s,1)-orbit carrier fires B3. Carrier as in ADM-1B; sector total.
A missed plant VOIDs.

## Meaning

D1+D2+D3 ⇒ "on the discrete 2+1 instance: mass IS deficit angle (the
puncture class), the realizable mass spectrum is exactly the
Gauss–Bonnet-allowed set, the relation survives refinement, and the source
is conserved by the gravitational dynamics." What this is NOT: continuum
masses, SU(2)'s continuous deficit spectrum (the named successor
SU(2)-via-2T, where the 2T classes give a genuine angle ladder), or local
degrees of freedom.
