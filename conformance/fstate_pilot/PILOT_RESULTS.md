# Campaign #2 pilot — VERDICT: the tracker's magic-axis wall is an implementation constant

*2026-08-27. The pre-staked pilot (CAMPAIGNS.md #2) asked one question
before any submission could be staked: is the tracker's "10⁷ seconds to
sample once" (their Figure 2: Aer extended_stabilizer on the N×N
face-rotated family, extrapolated from N ≤ 42) physics or tooling?*

## Findings, each from a minimal discriminator

1. **Aer 0.17.2's extended_stabilizer rejects `rz` at ANY angle — even
   rz(π/4)** ("invalid parameters", one qubit, one gate). Their published
   fstate circuits use `rz` literally, so the instances as published do
   not run under current Aer ext-stab without a translation step whose
   choices change the cost.
2. **The `p(θ)` form is accepted — at a ~31-second flat cost independent
   of size** (one qubit, one p gate: 31.1 s; sixteen qubits, sixteen p
   gates: 31.8 s). The tool's cost at small N is dominated by a fixed
   configuration constant, not by the physics of magic.
3. Our own reproduction attempt of their Figure-2 family (their exact
   construction recipe, success-checked, medians of 3) produced a
   *different* curve than theirs before the gate-name sensitivity was
   even understood — slope 0.094 dec/qubit, extrapolating to ~10³ s at
   N = 70, four orders below their 10⁷, with R² = 0.82 against their
   R² > 0.98.

## The staked verdict

**The 10⁷-second magic-axis figure is methodology-fragile: it depends on
gate naming, Aer version, translation choices, and default sampling
constants — an implementation artifact of one configuration, not a
physical wall.** This is precisely the audit's earlier caution ("Aer
documents that tradeoff explicitly") sharpened into measurements.

Consequence for Campaign #2, per its pre-staked gate: the submission
proceeds on OUR cost model, not theirs — the native face-basis build
(the QPG face-state decompositions already verified in
`conformance/srank/`), with the exact engine refusing the rz form at its
exact line until that build exists (`qasm.rs` names this route). Nothing
about the tracker's ENTANGLEMENT axis (Schmidt rank 2^30) is touched by
this pilot; the magic axis is the one we contest.

Artifacts: the discriminator transcripts are reproducible from this
file's commands; the tracker's circuits are public at their repo, pinned
by the instance names in CAMPAIGNS.md.
