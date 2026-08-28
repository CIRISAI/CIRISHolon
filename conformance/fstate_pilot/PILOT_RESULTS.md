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

## Corrected-family rerun (v3 + v4), and the verdict's final form

Corrected family (their gadget byte-structure: `tdg; h; p(θf); h`),
defaults reported, success-checked, medians of 3:

| N | 2 | 4 | 6 | 8 | 10 | 12 | 14 | 16 | 18 | 20 | 22 | 24 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| t (s) | 31.4 | 31.7 | 31.3 | 31.5 | 31.1 | 31.4 | 31.6 | 32.2 | 31.8 | 33.4 | 40.9 | 66.1 |

**Structure: a ~31.4 s configuration floor through N≈18, then liftoff.**
A whole-range straight-line fit is meaningless (it fits the floor; R²
0.62). The honest fit is on FLOOR-SUBTRACTED excess at the liftoff points
(N = 20, 22, 24: excess ≈ 2.0, 9.5, 34.6 s): slope ≈ 0.25–0.35
decades/qubit — which extrapolates the excess at N = 70 to roughly
**10¹³–10¹⁸ s under these defaults**, far ABOVE their 10⁷.

**The pilot's staked question is now answered in its strongest form: the
10⁷-second figure is not a stable property of the problem in either
direction.** The same published recipe yields ~10³ s (a family defect —
retracted above), ~10¹³⁺ s (corrected family, current-Aer defaults), or
their 10⁷ (their configuration/version) — three answers spanning ten-plus
orders, all "Aer extended stabilizer on the N×N face family."
Three-point caveat carried: the liftoff slope rests on three points and
is quoted as a bracket, not a constant.

Consequence for Campaign #2, sharpened: the only configuration-free cost
for this instance is the EXACT one — the face-native stabilizer-rank
evaluation (~2.2×10⁸ branches at the face exponent) — and if
current-Aer defaults genuinely sit at 10¹³⁺ for N = 70, an exact
mesh-scale evaluation beats the incumbent tool by construction, not by
tuning. The submission path is unchanged: build the face-native engine,
price it on our own measured constants.
