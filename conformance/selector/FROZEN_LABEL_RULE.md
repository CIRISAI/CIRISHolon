# FROZEN LABELLING RULE — written before any refutation number was computed

*Frozen by the selector5-refuter lane. Nothing below was chosen after seeing an
enrichment statistic. The rule is stated in two forms: RULE-A reconstructs what
SELECTOR-5's code actually did, RULE-B states the mathematically defensible
version of the same English sentence.*

## RULE-A — the operative rule, reconstructed from `landscape_sweep.py`

`is_lie_type` is a CONSTRUCTION TAG, assigned per builder, never computed from
the group:

- `build_cyclic`      -> True  ("finite subgroup of U(1)")
- `build_dihedral`    -> True  ("dihedral subgroup of SO(3)")
- `build_alternating` -> True iff deg in {3,4,5}
- `build_symmetric`   -> True iff deg in {2,3,4}
- `build_dicyclic`    -> True  ("binary dihedral in SU(2)")
- `build_binary_*`    -> True  (2T, 2O, 2I)
- `build_frobenius`   -> True iff (p,k) == (3,2)   [i.e. only F_6 = S_3]
- `build_delta_3n2`   -> True  ("finite subgroup of SU(3)")
- `build_heisenberg`  -> True iff p == 2           [i.e. only D_8]
- `build_semidihedral`-> False
- `build_modular_group`->False
- `build_gl2_gf3`     -> True
- `build_direct_product(G1,G2)` -> `G1.is_lie_type and G2.is_lie_type`
- deduplication: on an isomorphism collision the surviving record is
  PROMOTED to True if either colliding record was True (never demoted).

## RULE-B — the defensible rule, stated as mathematics

The English claim under test is "authentic Standard Model gauge & flavor
subgroup". Its only non-vacuous reading is membership in the gauge group:

> **SM(G) := G is isomorphic to a subgroup of SU(3) x SU(2) x U(1).**

Decidable form, used verbatim by the instrument:

> SM(G) holds iff there exist complex representations A (degree 3, det A = 1),
> B (degree 2, det B = 1) and a linear character x of G, with
> ker A INTERSECT ker B INTERSECT ker x = {e}.
> (A and B may be reducible; padding with the trivial representation realises
> homomorphisms into the smaller factors.)

This is exactly "there is an injective homomorphism G -> SU(3) x SU(2) x U(1)",
since a degree-3 determinant-1 representation IS a homomorphism to SU(3), a
degree-2 determinant-1 representation IS a homomorphism to SU(2), a linear
character IS a homomorphism to U(1), and injectivity of the product map is
triviality of the joint kernel.

### Two consequences of RULE-B fixed in advance (both are theorems, not choices)

1. **The generosity theorem.** SU(3) contains U(2) via
   `A |-> diag_block(A, det(A)^{-1})`. Hence EVERY finite group possessing a
   faithful complex representation of degree <= 2 satisfies SM. This includes
   every cyclic group, every dihedral group, every dicyclic / generalized
   quaternion group, 2T, 2O, 2I, every semidihedral group, every modular
   maximal-cyclic group, and every subgroup of any of these.
2. **The rank obstruction.** An abelian subgroup of SU(3) x SU(2) x U(1) has
   rank <= 2 + 1 + 1 = 4. Hence any abelian group of rank >= 5 (e.g. (Z_2)^5,
   (Z_2)^6, (Z_2)^7) fails SM.

## What each rule is used for

- RULE-A is the rule whose numbers reproduce the published 97.8%. It is used
  to reproduce, and to compute the base rate of the published label.
- RULE-B is the blind rule. It is used to compute the honest base rate.
- Neither rule is adjusted after any number is seen. If they disagree, both
  numbers are reported.

## Pools scored (fixed in advance)

P0 = all groups in the population.
P1 = P0 restricted to C1 passers.
P13 = P0 restricted to C1 and C3 passers.
P123 = P0 restricted to C1, C2 and C3 passers (the pool C4 acts on).
S = the full-gauntlet survivor set.

Enrichment is reported as precision(S) minus base-rate(P), for P in
{P0, P1, P13, P123}, under BOTH rules, with a permutation control drawing
|S| groups uniformly at random without replacement from P123 and recording
the distribution of precision.
