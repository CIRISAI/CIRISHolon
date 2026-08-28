# Pre-registration — EINSTEIN-ADM-1: the ADM channel closes on discrete 2+1 gravity

*Frozen 2026-08-28, committed ALONE. The instantiation rung between
CLOSURE-2's template and the 3+1 continuum: in 2+1 dimensions the Einstein
equations ARE the flatness condition (Witten's BF equivalence), the reduced
ADM phase space IS the moduli space of flat connections, and time evolution
is the mapping-class action. All three have exact finite-group analogues,
so "closure derives Einstein dynamics" is POSABLE here, not merely named.
The 3+1 continuum remains the far rung, stated, not claimed.*

misfits: contacts M-GAUGE-LAUNDER (the channel uses exact CONJUGATION-ORBIT
labels of holonomy pairs, never class-marginals of single loops — the
torus moduli space is pairs-up-to-simultaneous-conjugacy, which is the
orbit label itself), M-ONE-MODEL-DELTA (defects below are collision/minimax,
witnessed), M-NULL-MISSTAKE (E2 is staked on the quantity flatness actually
constrains, per arm), M-PROBE-EIGENSTATE (carriers named; the kicked state
of E3 is constructed to be OFF the flat sector, its sector asserted
nonzero), M-PLANT-OBS and M-PLANT-SECTOR (plants below), M-KINEMATIC-NONLOCAL
and M-HOMOG (no locality claim is staked here), M-BARE-CHARGE and
M-COND-PROBE and M-ELECTRIC-BASIS (no matter and no electric term in this
campaign: pure 2+1 BF; the dynamics is the mapping-class action, unitary
and gauge-covariant by construction), M-RING-MIXING (the Dehn action is a
permutation of configurations — no mixing unitary is introduced, so the
ring-scale constraint is not in play), M-LOOP-BLIND (the channel reads the
full orbit label, the finest gauge-invariant datum there is).

## Model

The one-plaquette torus: one vertex, two edges a, b; gauge group **D4**
(nonabelian, so the flat sector is nontrivial). Configuration space:
pairs (g_a, g_b) ∈ D4² (64 points), amplitudes exact integers.
- Flatness (THE 2+1 EINSTEIN EQUATION): the plaquette holonomy is the
  commutator [g_a, g_b]; the Einstein sector is [g_a,g_b] = 1
  (commuting pairs).
- Gauss: simultaneous conjugation (g_a, g_b) ↦ (x g_a x⁻¹, x g_b x⁻¹).
- ADM phase space: the moduli space = commuting pairs modulo
  simultaneous conjugation — the discrete Teichmüller analogue, computed
  exactly by orbit enumeration.
- Dynamics: the mapping-class generators (Dehn twists)
  `S: (g_a,g_b) ↦ (g_b, g_b g_a g_b⁻¹)`-type and `T: (g_a,g_b) ↦ (g_a, g_a g_b)`
  — unitary permutations, gauge-covariant (they commute with simultaneous
  conjugation by construction), flatness-preserving ON the flat sector by
  the commutator identity. The step is `T` then `S`.
- v_ADM: the exact orbit label of (g_a, g_b) under simultaneous
  conjugation (the point of the moduli space the state sits over, as the
  exact weight distribution over orbit labels).

## Gates

- **G0** (EXACT): the flat-sector projected uniform state is nonzero and
  Gauss-invariant. witness: none (instrument-checked)
- **E1 — CLOSURE ON THE EINSTEIN SECTOR** (EXACT): along 8 steps of the
  mapping-class dynamics from the flat carrier, v_ADM's trajectory is
  CLOSED: every collision v(x_i) = v(x_j) has v(x_{i+1}) = v(x_{j+1}) — no
  firing collision exists. The reduced ADM channel is autonomous exactly
  where 2+1 GR says it must be (no local degrees of freedom).
  witness: closure_determines_dynamics
- **E2 — THE DERIVED EINSTEIN EQUATION** (EXACT): flatness, pushed through
  the channel, is inherited: the flat-sector weight is conserved at every
  step (the commutator identity makes the Dehn action preserve
  [g_a,g_b] = 1). This is `closed_view_inherits_conservation` with H =
  the flatness indicator — the 2+1 Einstein constraint, DERIVED as an
  inherited conservation, not fitted.
  witness: closed_view_inherits_conservation
- **E3 — THE CONVERSE, WHICH MAKES E1 MEANINGFUL** (EXACT, two-branch): a
  KICKED carrier (a state with support off the flat sector, sector
  asserted nonzero) must either (a) exhibit a firing v_ADM collision
  within 8 steps — the channel's minimax defect is positive off shell,
  classical 2+1 GR fails exactly where curvature-excitation lives — or
  (b) stay collision-free, recorded as "off-shell not refuted at depth 8",
  NOT a fire. witness: collision_refutes_memoryless
- **B3** (EXACT): Gauss (simultaneous-conjugation invariance) holds on
  every trajectory state, both carriers. witness: none

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on.
- **(i) broken twist**: replace T by the NON-covariant map
  (g_a,g_b) ↦ (g_a, b₀ g_b) for a fixed b₀ ≠ 1 (left-multiplication by a
  non-central constant breaks conjugation-covariance); B3 must fire.
  Carrier: the flat vacuum; sector: total, nonzero.
- **(ii) flatness mutant**: a step that maps one flat configuration to a
  non-flat one (hand-planted single-point swap); E2's conservation must
  FIRE on the mutant. Carrier: the flat vacuum; sector: the flat-sector
  weight, nonzero.
A missed plant VOIDs.

## Meaning

E1+E2 ⇒ "on the discrete 2+1 instance: the reduced ADM channel is closed
under the gravitational (mapping-class) dynamics, and the Einstein
equation (flatness) is inherited through the channel as a derived
conservation — closure DERIVES the 2+1 Einstein dynamics on this model,
by the same two theorems the template proves in general."
E3(a) ⇒ the channel's defect is positive off the Einstein sector — the
order-parameter reading of "classical spacetime suffices exactly on
shell." What this is NOT: 3+1, continuum, or curvature-with-local-dof —
the named successors, in order: punctured torus with conjugacy-class
particles (mass–deficit-angle, requirement 2's rung), then SU(2)-via-2T,
then the continuum question, which remains open research.

## Amendment ADM-1B (frozen 2026-08-28, before the rerun; first run's log kept)

First run: E1 PASS (28 consistent collisions), E2 PASS, plant (ii) FIRES —
but E3 was VOID (the kicked sector was built with a wrong element index:
the D4 commutator subgroup is {1, r²} and the instrument's guess for r²'s
encoding was wrong) and plant (i) MISSED because its FROZEN carrier, the
fully symmetric flat vacuum, has a perturbed image whose support happens
to be conjugation-closed — the defect is invisible on a maximally
symmetric state. That is M-PLANT-SECTOR's sixth appearance, in the
sharpest form yet: symmetry of the carrier can close the very sector the
plant needs open.

1. E3's kicked carrier is the Gauss projection of the [g_a,g_b] = r²
   sector using the instrument's OWN R2 constant (imported, not guessed);
   its off-flat support is asserted nonzero per M-PROBE-EIGENSTATE.
2. Plant (i)'s carrier is the Gauss projection of the single flat
   configuration (s, 1) with s a reflection — chosen because the broken
   twist's image of THIS orbit is provably not conjugation-closed (the
   conjugator that inverts r moves the planted r), so the defect is
   visible; sector (the non-closed support difference) asserted nonzero.
All gates and criteria unchanged. witness: none (carrier corrections; no
criterion moves)
