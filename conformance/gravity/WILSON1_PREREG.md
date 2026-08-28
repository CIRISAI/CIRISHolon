# Pre-registration — WILSON-1: the separated oriented observable

*Frozen 2026-08-28, committed ALONE before its instrument, run chained
behind BRIDGE-7B. This is the R2-successor design that BRIDGE-7's measured
fire made REQUIRED: a separated, gauge-invariant, ORIENTED Wilson
observable on a non-ambivalent group, replacing every class-marginal
reading this family has staked (M-LOOP-BLIND, now measured physics).*

misfits: contacts M-LOOP-BLIND (this campaign exists because of it; the
class-marginal control below reproduces it deliberately),
M-GAUGE-LAUNDER (the group is chosen NON-AMBIVALENT so orientation cannot
be laundered by conjugacy classes — for Z3, class(ω) ≠ class(ω²) since
classes are singletons and ω⁻¹ = ω²), M-BARE-CHARGE (the charged pair is
Wilson-line dressed, invariance checked as G0), M-PLANT-OBS and
M-PLANT-SECTOR (plants below state carriers and sectors), M-HOMOG (no
locality claim is staked here — separation is from the MATTER SITES, and
the Bianchi null handles the rim), M-COND-PROBE (all dynamics inside T),
M-PARITY-PROTECT (Z3 has no parity sector; the protected-sector failure
mode cannot arise, noted not claimed).

## Model

Gauge group **Z3** on the base fan disk (5 spokes, 5 rim edges as in
BRIDGE-1's `base_graph`, group elements {0,1,2} under addition mod 3).
Amplitudes EXACT in the Eisenstein ring Z[ω], ω = e^{2πi/3}, carried as
integer pairs a+bω with a global 3^{−m/2} exponent (the ζ3 rung of the
engine's ring tower). Matter: a charge pair (+1 at vertex 1, −1 at vertex
2), each site a 3-dim charge register; gauge acts at a matter vertex by
ω^{±q}; the physical pair state is the WILSON-DRESSED line
`Σ_q ω^{q·(line holonomy)}`-weighted, along 1 → c → 2.

Floquet step T (all inside T, per M-COND-PROBE):
`T = U_B · U_E · K_charge`, where `U_B` multiplies by χ(plaquette
holonomy) = ω^{h_p} per plaquette; `U_E` is the exact 3-point unitary
`(1/√3)·Σ_k ω^{jk}` (the Z3 Fourier kernel) on each SPOKE edge; and
`K_charge` injects flux +1 on spoke e* conditioned on the dressed channel
(fires off-channel, inert on-channel), gauge-invariant because every Z3
element is central and the channel projector is the dressed one.

## The observable

`W_far` = the ORIENTED Wilson value of one plaquette **vertex-disjoint from
the matter sites** (a triangle (c, 3, 4): touches neither vertex 1 nor 2).
Its reading is the exact spectral weight triple `(w₀, w₁, w₂)` of the state
over W_far ∈ {1, ω, ω²}. The CONTROL observable is the class-marginal
`(w₀, w₁+w₂)` — the D4-style reading that cannot see orientation.

## Gates

- **G0** (EXACT): the dressed pair state is nonzero after Gauss projection
  and joint Gauss holds; checked FIRST; nothing else reported if it fires.
  witness: none (instrument-checked; construction per M-BARE-CHARGE)
- **W1 — separation** (EXACT): within 4 steps of T, the weight triples of
  the dressed carrier and the live off-channel carrier DIFFER on W_far.
  witness: none (measured gate)
- **W2 — orientation** (EXACT): at some step the triple satisfies
  w₁ ≠ w₂ — the reading distinguishes W from W†, which is the
  non-ambivalent group doing work no D4 class function could.
  witness: none (measured gate)
- **W3 — the Bianchi null** (EXACT): the TOTAL rim-loop weight triple is
  IDENTICAL between the two carriers at every step (abelian Bianchi: a
  spoke pump conserves total flux). This is a built-in validation: if W3
  fires, the instrument is broken, not the physics.
  witness: none (the conservation argument is stated here; a Lean brick is
  named as follow-up, not claimed)
- **W4 — the laundering control** (EXACT): the class-marginal reading
  `(w₀, w₁+w₂)` must FAIL to distinguish the carriers wherever the full
  triple distinguishes them only through w₁ vs w₂. If the marginal alone
  already separates, W4 does not fire but the "orientation is necessary"
  claim is NOT earned and is recorded as unearned.
  witness: none (measured control)
- **B3** (EXACT): joint Gauss on every registry state at every step.
  witness: none (measured gate)

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on
before the plant is scored; a plant on an empty sector VOIDs.

- **(i) wrong-side action at a matter vertex** (apply ω^{−q} where the
  freeze says ω^{+q}): must fire B3. Carrier: the dressed state; sector:
  total amplitude, asserted nonzero before scoring.
- **(ii) orientation-breaking readout** (score W_far with the conjugated
  character, i.e. swap w₁ ↔ w₂ on ONE carrier only): must flip W2's
  reading on that carrier. Carrier: the dressed state; sector: the
  W_far-weight support, asserted nonzero before scoring. This plant
  convicts the readout path itself — a wrong character must be VISIBLE.
- A missed plant VOIDs.

## Meaning

All gates → "a separated, oriented, gauge-invariant Wilson observable
detects WHICH matter channel drove the pump, at a plaquette disjoint from
the matter, while the total-flux null holds and the class-marginal control
confirms orientation was necessary." W1 fires → the loop-blindness extends
even to oriented observables in this family, and the review's first
overclaim hardens further. Either way the record advances. Z3 toy; the
non-abelian successor (order-21 Frobenius group, the smallest non-abelian
non-ambivalent case) is named, not claimed.
