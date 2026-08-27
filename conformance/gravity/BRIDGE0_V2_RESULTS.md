# BRIDGE0-V2 — VOID, recorded and kept

*2026-08-27. Prereg ca42b33, instrument 5e537e2, run log below. Verdict
under the frozen gates: **VOID** — G5 fired and plant (ii) missed.*

- G5 fired on four registry states (both graphs): the constraint checker
  demanded row m carry flux class C_m at p0 on EVERY state including
  PRE-coupling states (vacuum⊗|m⟩), where p0 is still flat — the sector
  constraint was staked state-independent when the physics is
  state-dependent. A prereg design defect, caught by its own full-scope
  gate doing exactly what v1's failure taught us to make it do.
- Plant (ii) missed: with the coupling map T in the path, the broken-word
  reading on the ρ sector no longer leaks the identity — the plant's
  observability argument was inherited from v1's DIFFERENT state family
  and did not transfer. Observability arguments must be re-derived per
  instrument, not carried over.
- Everything else passed (G1–G4, G6, G7, G7b, plant i) — kept as
  exploratory only; a VOID is a VOID.

Misfits M7 (state-dependent constraints must be staked as such) and M8
(plant observability is instrument-relative) feed V3, which replaces the
one-step coupling with genuine discrete-time unitary dynamics — the
"Hamiltonian evolution or bust" pivot. No rescue of v2.

Run log:
```
[base] G1=PASS G2_rho=PASS G2_tau=PASS G3_rho=PASS G3_tau=PASS G4=PASS
G5=FIRE [vac_row_rho, vac_row_tau, X_rho(vacuum), G[loop](T_rho): p0
sector constraint on pre-coupling states] G7_*=PASS G7b=PASS
[refined] identical
[plant i] FIRES  [plant ii] MISSED ({1:{1}})
VERDICT: VOID
```
