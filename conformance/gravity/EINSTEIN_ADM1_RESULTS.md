# EINSTEIN-ADM-1 — ALL GATES PASS, both plants fire

*2026-08-28. Prereg admitted and frozen; ADM-1B amendment (a wrong element
index and a plant carrier whose symmetry hid the defect — M-PLANT-SECTOR's
sixth appearance) admitted before the rerun; log `einstein_adm1` run
output in the commit trail.*

```
GATES: G0=PASS  E1=PASS (28 collisions, all consistent)
       E2=PASS (flatness inherited exactly)
       E3=BRANCH(b): off-shell not refuted at depth 8 (recorded)
       B3=PASS
[plant i] FIRES (on the (s,1)-orbit carrier)   [plant ii] FIRES
```

## What is earned

On the discrete 2+1 instance (one-plaquette D4 torus, exact integers):

1. **The reduced ADM channel is CLOSED under the gravitational dynamics**
   (E1): the moduli-space view of the quantum state evolves autonomously
   under the mapping-class action — 28 trajectory collisions, every one
   consistent, so by `closure_determines_dynamics` the induced coarse map
   is single-valued on all observed data. Classical 2+1 gravity's phase
   space is autonomous exactly where the continuum theory says it must be
   (no local degrees of freedom).
2. **The Einstein equation is INHERITED, not fitted** (E2): flatness — in
   2+1 dimensions the Einstein equation itself — pushes through the
   channel as an exactly conserved quantity of the coarse dynamics, which
   is `closed_view_inherits_conservation` measured. Together with
   CLOSURE-2's template theorems this is the requirement-3 structure
   composed end to end at toy scale: channel, closure, derived dynamics.
3. **The off-shell converse is NOT yet demonstrated** (E3 branch (b)): the
   r²-curved sector's channel trajectory produced no firing collision at
   depth 8 on this 64-configuration model. Recorded as staked — not
   rescued, not extrapolated. A larger complex (more plaquettes, richer
   moduli) is where the off-shell defect has room to appear; that is the
   punctured-torus successor's job, where conjugacy-class particles also
   pose requirement 2's mass–deficit-angle.

## What this is NOT

3+1, continuum, local degrees of freedom, or matter back-reaction — the
named far rungs. The claim is precise: closure DERIVES the discrete 2+1
Einstein dynamics on this model, by the same two theorems the template
proves in general, with every gate exact and both plants firing.

## Re-adjudication after external re-review: E1 was VACUOUS, and the claim is downgraded

The re-review is correct and reproduced here: the flat carrier (the
uniform superposition over the flat sector) is EXACTLY STATIONARY under
the mapping-class permutation — a uniform-over-sector state is invariant
under any sector-preserving permutation — so all 28 "consistent
collisions" were one unchanged state repeating. E1 as run demonstrated
consistency of a stationary point, not closure of a moving channel. The
same holds for the kicked carrier. **The "third fully green campaign"
claim is WITHDRAWN**; registered as M-FIXED-POINT-TRAJECTORY. The
re-review also identified the stronger fact actually available: the
microscopic permutation DESCENDS exactly to a nontrivial permutation of
all 28 conjugacy-orbit labels and preserves flatness on all 64
configurations — a real finite quotient-dynamics statement, checkable
universally rather than along any trajectory.

## Amendment ADM-1C (frozen here, before the rerun)

- **E1' — UNIVERSAL closure** (EXACT, carrier-free): for EVERY one of the
  64 configurations, the orbit label of step(config) is a function of the
  orbit label of config (exhaustive well-definedness of the descended
  map); AND the descended map is a NONTRIVIAL permutation of the 28 orbit
  labels (it moves at least one label — the vacuity check the trajectory
  form lacked); AND it maps flat-sector orbits to flat-sector orbits
  (flatness preserved at the quotient level). This is the quotient-
  dynamics theorem checked by exhaustion, immune to carrier choice.
  witness: closure_determines_dynamics
- **E2' — inheritance on a MOVING carrier** (EXACT): the carrier is the
  Gauss projection of the single flat configuration (s, 1), whose orbit
  label provably moves under the twist (the trajectory is asserted
  non-stationary before the gate is scored, per M-FIXED-POINT-TRAJECTORY);
  flatness weight conserved along its trajectory.
  witness: closed_view_inherits_conservation
- E3, B3, G0 and both plants unchanged from ADM-1B.
misfits: contacts M-FIXED-POINT-TRAJECTORY (registered by this document),
M-PROBE-EIGENSTATE (the moving-carrier assertion), M-STALE-INSTRUMENT
(this amendment and its instrument are committed TOGETHER), M-GAUGE-LAUNDER
and M-ONE-MODEL-DELTA and M-NULL-MISSTAKE and M-PLANT-OBS and
M-PLANT-SECTOR (as admitted; plants' carriers and sectors unchanged —
each asserted nonzero in the sector the plant acts on).
witness: none (amendment header; gate witnesses above). M-HOMOG:
contacted only via the word "local" in prose; no locality claim is staked
in this campaign.

## ADM-1C run — all gates pass on the corrected stakes

```
E1'=PASS (the microscopic step DESCENDS: well-defined on all 64 configs,
          a NONTRIVIAL permutation of the 28 orbit labels, flat-preserving)
E2'=PASS (flatness inherited on a PROVABLY MOVING trajectory --
          stationarity asserted away before scoring)
E3=BRANCH(b)   G0/B3=PASS   plants both FIRE
```

The claim, correctly sized this time: the quantum dynamics descends
EXACTLY to a nontrivial classical dynamics on the discrete ADM phase
space (checked by exhaustion, carrier-free — no trajectory can fake it),
that descended dynamics preserves the 2+1 Einstein constraint at the
quotient level, and the constraint is inherited along a genuinely moving
coarse trajectory. What the re-review said remains true and is the
record's own words: this verifies that the chosen mapping-class
permutation projects consistently — the dynamics is INPUT, its
closure and constraint-preservation are the theorems. "Deriving" the
dynamics itself from a closure principle (picking the mapping-class
action OUT of closure rather than checking it) is the open remainder,
now precisely worded.
