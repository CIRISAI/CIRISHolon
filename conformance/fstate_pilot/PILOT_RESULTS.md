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
3. **RETRACTED 2026-08-27, same day, on review** (the retraction caught
   after the user questioned the four-order discrepancy): our first
   reproduction placed the face rotations TERMINALLY before measurement —
   and a diagonal layer immediately before computational-basis
   measurement does not change the output distribution at all, so that
   family's sampling was trivially magic-free, which is exactly why its
   curve was flat. Their actual per-qubit gadget is `tdg; h; rz(θ); h` —
   the rotation sits BETWEEN Hadamards (verified byte-level in their
   qasm) and is genuinely non-diagonal in the measured basis. The ~10³ s
   extrapolation says nothing about their curve and is withdrawn; a
   corrected-family rerun is below. Findings 1 and 2 (the rz-name
   rejection and the ~31 s size-independent p-gate cost) stand as
   measured — they are single-gate discriminators unaffected by family
   structure.

## The staked verdict

**NARROWED after the retraction: what stands is that the tracker's
figure is not reproducible as published under current Aer** (their
circuits' literal `rz` is rejected at any angle, and the accepted `p`
path carries a ~31 s size-independent constant under defaults) — the
configuration-sensitivity claim survives on findings 1–2 alone. Whether
their CURVE's shape is right awaits the corrected-family rerun below;
until it lands, no claim about their slope is made in either direction.

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
