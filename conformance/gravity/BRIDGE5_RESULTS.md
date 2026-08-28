# BRIDGE-5 — VOID, and the misfit is the physics telling us the design was wrong

*2026-08-28. Prereg d8352b0, instrument committed after it, log
`bridge5_run.log`. Verdict under the frozen gates: **VOID** — B3, R1 and R3
fired, and plant (ii) missed. R2 passed. Kept, marked, no rescue.*

## What fired, and the single cause underneath all of it

A two-line diagnostic settles it:

```
vacuum(singlet): 1024 nonzero amplitudes, Gauss HELD
   after one step: Gauss FAILS at vertex 1
vacuum(triplet): 0 nonzero amplitudes          <- annihilated
```

**The triplet sector is EMPTY, and the singlet sector is not preserved by
my pump.** Both follow from one fact the campaign's own earlier amendment
already recorded and I failed to carry forward:

> Gauss acts INDEPENDENTLY at each vertex. `Σ_g ρ₂(g) = 0` (irrep
> orthogonality), so any component that is not invariant under the
> per-vertex action is annihilated by the projector.

The singlet `|00⟩+|11⟩` is invariant under the DIAGONAL action
`ρ₂(g)⊗ρ₂(g)` — but Gauss demands invariance under `ρ₂(g)⊗1` at vertex 1
and `1⊗ρ₂(g)` at vertex 2, SEPARATELY. So:

1. **The triplet is annihilated** (R1 and R3 read identically zero — the
   observable was measuring an empty sector, which is why they fired).
2. **My "gauge-invariant channel" is not gauge-invariant.** The singlet
   projector defined on BARE spinor indices commutes with the diagonal
   action only. Conditioning the pump on it therefore breaks Gauss on the
   very next step — exactly what B3 reports.
3. **Plant (ii) could not fire** because it was checked on the annihilated
   triplet state: a plant applied to zero is invisible. That is misfit M8
   again (observability is instrument-relative), and it VOIDs the campaign
   independently of the gate failures.

R2's PASS is therefore not creditable either: it distinguished a live
sector from an empty one, which is not what "the charged sector sources
geometry" means.

## The misfit, stated for the successor (M14)

**A charged pair is gauge-invariant only when DRESSED BY A WILSON LINE.**
The physical singlet is not `Σ_i |ii⟩` but
`Σ_ij ρ₂(U_γ)_ij |ij⟩` for a path γ from vertex 1 to vertex 2 — the gauge
field carries the invariance, which is precisely why a charged pair is
"screened" in the first place. Every operator conditioned on the matter
channel must be built from that dressed projector, and it will then be
gauge-invariant BY CONSTRUCTION rather than by hope.

This is the campaign series doing its job: the design was wrong in a way
that ordinary code review would not catch, the frozen gates caught it, and
the correction is a specific, derivable object rather than a direction.
BRIDGE-6 will stake the dressed projector, verify its invariance BEFORE
using it, and re-pose R1–R3 on a sector that is actually occupied.
