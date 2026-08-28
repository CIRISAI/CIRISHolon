# WILSON-1 — VOID: the frozen prereg itself carried the defect, and B3 caught it at step one

*2026-08-28. Prereg admitted and frozen; instrument after it; log
`wilson1_run.log`. Verdict: **VOID** — B3 fired on every evolved state, W3
fired as a consequence, plant (ii) missed on a symmetric carrier.*

## The defect is in the FREEZE, not the code

The prereg specified `U_E` as the Z3 Fourier kernel per spoke. **A
basis-change kernel is not gauge-covariant** — the electric term must be a
function of SHIFT operators (which commute with every gauge action; the
Fourier kernel instead maps the magnetic basis to the electric one and
picks up position-dependent phases under a gauge transform). Gauss fired on
the first step, on every state, both carriers — the constraint machinery
convicting the frozen design in one move. Registered as
**M-ELECTRIC-BASIS**; the audit's misfit registry now greps for it.

Two subsidiary readings, both downstream of the broken term and therefore
NOT adjudicated: W1/W2's "passes" and W4's unearned note are meaningless
under non-covariant dynamics; plant (ii) missed because the dressed
carrier's far triple is exactly symmetric (w₁ = w₂), i.e. the plant acted
on an empty ASYMMETRY sector — M-PLANT-SECTOR's fifth appearance, now with
the sharpened rule: the sector a plant needs is the sector of the EFFECT it
must produce, not merely the operator's support.

## The fix, verified in-ring before WILSON-2 freezes

`U_E(e) = (1 + ω·L₁ + ω·L₂)/√3` — a polynomial in shift operators, hence
gauge-invariant by construction; unitary because its shift-eigenvalues are
`1+2ω, 1−ω, 1−ω`, each of squared modulus 3; exact in Z[ω] with the ring's
√3 exponent. WILSON-2 stakes it with a per-operator invariance line.
