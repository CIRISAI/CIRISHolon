# Pre-registration — WILSON-2: the covariant electric term

*Frozen 2026-08-28, committed ALONE. Successor to WILSON-1 (VOID:
M-ELECTRIC-BASIS — the frozen Fourier kernel was not gauge-covariant, and
B3 convicted the freeze at step one). Identical model, observable, and
gates; two changes only, each traceable to a registered misfit.*

misfits: contacts M-ELECTRIC-BASIS (the change this freeze exists for),
M-LOOP-BLIND and M-GAUGE-LAUNDER (the oriented observable and its
class-marginal control, unchanged), M-BARE-CHARGE (dressed pair,
unchanged), M-PLANT-OBS and M-PLANT-SECTOR (plant (ii) now asserts the
ASYMMETRY sector — the sector of the effect the plant must produce — is
nonzero before scoring), M-KINEMATIC-NONLOCAL (no locality claim is staked
here; the pendant lesson is carried, not contested), M-HOMOG and
M-PARITY-PROTECT and M-COND-PROBE (as in WILSON-1, unchanged).

## Change 1 — the electric term, with its invariance argument IN the freeze

`U_E(e) = (1 + ω·L₁ + ω·L₂)/√3` per spoke, where `L_k` shifts the edge's
group element by k.
- invariance: a polynomial in shift operators commutes with every gauge
  action (abelian: left and right shifts coincide and commute with the
  vertex action), so `U_E` is gauge-covariant BY CONSTRUCTION — the
  argument WILSON-1's freeze lacked.
- unitarity: shift eigenvalues are `1+2ω` (k=0) and `1−ω` (k=1,2), each of
  squared modulus 3, so `U_E†U_E = 1` exactly; entries live in Z[ω] with
  the ring's √3 exponent.
witness: none (derivation above; a Lean brick for the eigenvalue check is
named as follow-up, not claimed)

## Change 2 — plant (ii)'s sector assertion

The conjugated-readout plant is scored on a carrier whose far triple is
ASYMMETRIC (w₁ ≠ w₂), asserted before scoring; if no registry state within
4 steps has an asymmetric far triple, plant (ii) is UNPOSABLE and the
campaign VOIDs (never scored on a symmetric carrier again).

## Everything else — carried verbatim from WILSON-1's admitted freeze

Model (Z3 fan disk, Eisenstein-exact, dressed pair, pump conditioned
off-channel inside T), observable (oriented weight triple of the
matter-disjoint plaquette (c,3,4); class-marginal control), and gates:
- **G0** (EXACT) checked first. witness: none (instrument-checked)
- **W1 — separation** (EXACT): triples differ, dressed vs live off-channel,
  within 4 steps. witness: none (measured gate)
- **W2 — orientation** (EXACT): w₁ ≠ w₂ somewhere. witness: none (measured)
- **W3 — Bianchi null** (EXACT): rim triple identical between carriers at
  every step; fires ⇒ instrument broken. witness: none (conservation
  argument in WILSON-1's freeze)
- **W4 — laundering control** (EXACT): the marginal must fail where only
  orientation separates; else orientation-necessity is unearned.
  witness: none (measured control)
- **B3** (EXACT): joint Gauss everywhere — the gate that convicted
  WILSON-1's freeze and is trusted BECAUSE it fired. witness: none

## plants (per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on —
for plant (ii), the sector of the effect: the far-triple ASYMMETRY.
- **(i) wrong-side action at a matter vertex**: must change the dressed
  state (violate its invariance). Carrier: dressed; sector: total.
- **(ii) conjugated readout on an asymmetric carrier**: must flip the
  triple visibly. Carrier: first registry state with w₁ ≠ w₂; sector: the
  asymmetry, asserted nonzero.
A missed plant VOIDs; an unposable plant VOIDs.

## Meaning

Unchanged from WILSON-1: all gates ⇒ the separated oriented observable
sees which channel drove the pump, with the Bianchi null holding and
orientation proven necessary by the control. W1 fires ⇒ loop-blindness
extends to oriented observables in this family. Either way, the record
advances on a validated instrument this time.
