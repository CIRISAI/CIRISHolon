# Pre-registration — GRAVITY-BRIDGE-5: endogenous, local, reciprocal, charged

*Frozen 2026-08-28, committed ALONE before any instrument. Successor to
BRIDGE-1/2/3. This campaign exists to close the THREE overclaims an
external review identified in the BRIDGE record, each of which was one
notch stronger than what was measured. Every one becomes a gate here.*

## The three repairs, as gates

**R1 — ENDOGENOUS (BRIDGE-3 was a conditioned probe).** In BRIDGE-3 the
geometry→matter operator `G_p` was applied AFTER `T`. Here it is a TERM OF
T: the Floquet step is
`T = U_B · U_E · G_p · K_charge`, so the same evolution that creates the
geometry produces the back-reaction. **Gate R1:** the reciprocity readings
must come from iterating T alone, with no operator applied by hand at any
point; the instrument asserts that its measurement path calls only `step`.

**R2 — CHARGED SOURCE (the matter was a spectator).** BRIDGE-1's pump was
controlled by a gauge-NEUTRAL occupation qubit, so the screened ρ₂ pair was
inert. Here the pump is controlled by the CHARGED sector: the flux injected
at p0 is conditioned on the ρ₂ spinor pair's own state (a gauge-invariant
projector onto the pair's singlet vs triplet channel — invariant because it
is a projector onto an irrep sector, which commutes with every gauge
action). **Gate R2:** with the pair in the singlet the pump is inert; with
it in the triplet the pump fires — and the difference must be visible in
the DISTANT loop reading. Both graphs, and the refined graph must carry the
SAME matter sector (BRIDGE-1's refined runs were occupation-only, which is
part of why the spectator problem hid).

**R3 — LOCAL (the control was inert by symmetry, not by distance).**
BRIDGE-3's locality control was inert because the Floquet family is
parity-homogeneous — every plaquette carried identical parity content, so
"distant" and "near" were indistinguishable. Here the electric term is
SPOKES-ONLY (`U_E` acts on the three spokes, not the rim), which breaks the
homogeneity by construction. **Gate R3:** the back-reaction conditioned on
p0's holonomy must MOVE the matter reading, and the same operator
conditioned on a rim plaquette two steps away must leave it EXACTLY
unchanged — an inertness that is now spatial, not symmetric.

## Model

Gauge group D4 and the fan disk as in BRIDGE-1 (base and refined), matter =
the ρ₂ spinor pair at vertices 1 and 2 with the joint Gauss constraint.
Amplitudes exact in Z[ω]; overflow refuses. All operators carry their
gauge-invariance derivation in this file before the instrument exists:

- `K_charge` = Σ_c P_c ⊗ L_{r²}(e*) where P_c projects the spinor pair onto
  its singlet/triplet channel. Gauge-invariant because P_c is a projector
  onto an irrep sector (commutes with ρ₂(g)⊗ρ₂(g) for every g) and r² is
  CENTRAL (commutes with every left/right multiplication).
- `G_p` = P_{parity1}(p) ⊗ σ_z on the pair's channel label. The projector is
  class-diagonal hence gauge-invariant; σ_z on the CHANNEL label (not the
  spinor components) commutes with ρ₂ because the channel is an invariant.
- `U_E` spokes-only: (1 + i·R_{r²}(e))/√2 on the three spoke edges. Unitary
  (r² is an involution) and gauge-invariant (r² central), exactly as in
  BRIDGE-1, but on a subset that breaks parity homogeneity.

## Gates

- **R1, R2, R3** as above.
- **B3 (unchanged)**: the joint Gauss projector holds exactly on every
  registry state at every step, both graphs.
- **Two-route (unchanged discipline)**: every staked reading confirmed by an
  independent combinatorial recursion sharing no code with the evolution.
- **Refinement**: R1–R3 verdicts identical on the refined graph, WITH the
  charged sector present (the fix to BRIDGE-1's occupation-only refinement).
- **Plants, observability re-derived for THIS instrument** (M8): (i) a
  wrong-side gauge action must fire B3; (ii) a NON-central pump (L_r) must
  fire B3 at e*'s endpoints — convicted by exactly the centrality argument
  that licenses the real pump. A missed plant VOIDS.

## Meaning

All gates → "on an exact discrete gauge theory with genuinely charged
matter under a single joint unitary step: charged matter sources geometry
endogenously, geometry acts back within the same step, and the influence is
spatially local with an inert distant control." Still a finite-group BF
toy; SU(2) via the binary tetrahedral subgroup 2T is the named successor,
and the GR phase-space channel remains the open problem, not a rung.
Any gate firing kills its claim and is kept marked. No rescue.
