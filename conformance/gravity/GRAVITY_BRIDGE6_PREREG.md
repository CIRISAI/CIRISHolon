# Pre-registration — GRAVITY-BRIDGE-6: the dressed charge

*Frozen 2026-08-28, committed ALONE before its instrument. Successor to
BRIDGE-5 (VOID). Carries M1–M14. Same three repairs as BRIDGE-5 —
endogenous, charged, local — re-posed on a matter sector that is actually
occupied and an operator that is actually gauge-invariant.*

## The correction, VERIFIED BEFORE THE FREEZE (this is the point)

BRIDGE-5 died because a bare spinor-pair singlet is invariant under the
DIAGONAL gauge action but Gauss acts INDEPENDENTLY per vertex, so the
sector was annihilated and the channel operator was not invariant. M14
named the fix: dress the pair with a Wilson line. It is now MEASURED, and
the measurement is why this prereg exists rather than a hope:

```
DRESSED singlet after Gauss projection: 524288 nonzero, Gauss HELD
BARE    singlet after Gauss projection:   1024 nonzero  (and broken by the pump)
```

The physical charged pair is `Σ_ij ρ₂(U_γ)_ij |ij⟩` with γ the path
1 → c → 2, i.e. `U_γ = u(c,1)⁻¹ u(c,2)`. The projector onto it is
CONFIGURATION-DEPENDENT — necessarily, because the dressing IS the gauge
field. That is what "screened charge" means, and it is why the bare
construction could never have worked.

## Gates (all on the dressed sector)

- **G0 — the invariance gate, checked FIRST and separately**: the dressed
  projector `P_W` commutes with the joint Gauss action at every vertex,
  verified exactly on the instrument's own states before any dynamics runs.
  If G0 fires nothing else is reported: an operator that is not invariant
  makes every downstream number meaningless.
- **R1 (endogenous)**: the reciprocity reading moves under iterating `step`
  ALONE, with `G_p` a term of T. The instrument asserts its measurement
  path calls only `step`.
- **R2 (charged source)**: the pump conditioned on `P_W` fires on the
  dressed-charged component and is inert on its complement, and the
  difference reaches the DISTANT loop reading.
- **R3 (local)**: with the spokes-only electric term, the p0-conditioned
  response moves the matter reading and the same operator conditioned on a
  rim plaquette leaves it EXACTLY unchanged.
- **B3**: joint Gauss holds on every registry state at every step.
- **Refinement**: R1–R3 identical on the refined graph, charged sector
  present.
- **Plants, with observability re-derived AND pre-checked on a NONZERO
  state** (M8 twice over — BRIDGE-5's plant died on an empty sector):
  (i) wrong-side gauge action must fire B3; (ii) non-central pump must fire
  B3. The instrument asserts each plant's carrier state is nonzero before
  scoring the plant; a plant applied to zero VOIDS.

## Meaning

All gates → "on an exact discrete gauge theory with a genuinely charged,
Wilson-dressed matter pair under a single joint unitary step: charged
matter sources geometry endogenously, geometry acts back within the same
step, and the influence is spatially local with an inert distant control."
Still a finite-group BF toy; SU(2) via the binary tetrahedral subgroup 2T
is the named successor and the GR phase-space channel remains the open
problem. Any gate fires → that claim dies, kept, marked. No rescue.
