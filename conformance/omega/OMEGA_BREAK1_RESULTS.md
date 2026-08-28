# OMEGA-BREAK-1 — the adequacy hunt: identity is a function of the ACT vocabulary, and the extension law is a theorem

*2026-08-28. Criterion frozen before the instrument (BREAK_CRITERION.md);
hunts exhaustive and exact (~9M probe-equivalent pairs at |S| ≤ 5, plus
composition, adaptivity, and real-substrate legs); independent verifier
ALL CHECKS PASSED re-run in the lead session; Break.lean builds green in
the tower. All banked together.*

## The answer, in one law

**Breaks exist, and they are exactly characterized.** Probe-equivalent
holons CAN be separated by blind interventions — the rung-3 counterexample
itself falls to a single blind transposition — but across every
intervention class, every size, and both directions of the contingency:

> **T1 (exhaustively tight, machine-checked both halves):** an
> intervention class separates probe-equivalent holons **iff** it contains
> a knob that FAILS TO DESCEND to the gauge quotient (maps a
> gauge-equivalent pair to a gauge-inequivalent one). Zero exceptions in
> either off-diagonal cell, all classes, all sizes.

In Lean: `gauge_safety` (descending knobs can never separate — with the
corollaries that preparations are always safe and that on a CLOSED holon
every view-covariant knob is safe) and `omega_break` (the exhibited knob,
with `knob_does_not_descend` naming exactly why safety does not apply).
Composition probes with view reads: zero breaks. Adaptive (closed-loop)
interventions add nothing over blind ones at the sizes searched. The real
group-substrate leg reproduces the same law: every break knob is a
covariance violator.

## What this does to the ladder

1. **The identity commitment is UPGRADED, not overturned**:
   `Identity.lean`'s SameHolon is the empty-knob end of a GRADED family
   Identity(𝒜) indexed by the admissible act vocabulary 𝒜. Fibers are
   gauge relative to the acts you may perform — precisely gauge theory's
   own rule that only covariant operations are physical, now derived
   inside Ω rather than imported from physics.
2. **Maximality survives by lawful extension**: the hunt demanded
   structure invisible-yet-consequential; it found it, AND found that the
   tuple grows by a derivable law (the intervention face, parameterized by
   𝒜, with T1 as its adequacy criterion) rather than by ad-hoc repair.
   This is the first measured instance of the rung-7 extension mechanism
   working as the review's formulation hoped: the receipt format extended,
   not replaced.
3. **The successor question**: T1 at |S| ≤ 5 is exhaustive but finite —
   its general proof is `gauge_safety` (one direction, done) plus the
   converse (every non-descending knob yields a break on SOME pair),
   staked as the next Lean brick, not claimed.
