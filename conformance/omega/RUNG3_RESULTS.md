# Rung 3 — representational completeness: SETTLED, both ways, machine-checked

*2026-08-28. Opus track (rung3-hunt), verified in the lead session
(independent verification script re-run: all counterexamples confirmed,
both iso-present controls pass) and the Lean settled into the tower
(`ProbeConverse.lean`, builds green).*

## The answer

**Probe data does NOT determine Ω in general.** The minimal counterexample
is |S| = 3, |V| = 2: two holons with the same surjective view, both
Closed, both with reversible dynamics, agreeing on every probe at every
depth from every state — and admitting NO Ω-isomorphism (all 12 candidate
bijection pairs exhaustively refuted, each with its named failure). The
difference lives strictly inside a view fiber: one dynamics fixes the
fiber, the other swaps it, and no probe stream can see a permutation of
states that share a stream. Census: counterexample families grow 1, 3, 19,
152, 1523 for |S| = 2…6.

**And probe data DOES determine Ω exactly on the observable sector.**
Exhaustively to |S| = 6 (9,496 probe classes, zero mismatches): every
probe class whose streams separate states contains exactly ONE iso class.
In the Lean: `omegaIso_of_probeAgreement` — the converse holds, with the
identical witnesses, as soon as the holon is observable — and
`observable_transfer` makes observability readable off the probe data
itself.

## The moral, in the review's own vocabulary

Ω's identity is strictly finer than its probe behaviour, and the excess is
FIBER-INTERNAL — precisely the slot the enriched tuple's remaining faces
(measure, transport, cost) would have to pin, or else the fiber-internal
dynamics is genuinely conventional (gauge). This turns the review's
completeness question into a sharp fork, each branch now posable: either a
face of the tuple measures fiber-internal structure (find the probe that
does it), or fiber-internal structure is declared gauge and Ω's identity
is its Moore quotient (freeze that as the definition). Either choice is a
definition-level commitment the ladder's rung 2 must absorb explicitly.
