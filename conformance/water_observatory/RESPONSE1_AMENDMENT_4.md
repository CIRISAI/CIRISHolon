# RESPONSE-1 — AMENDMENT 4: the continuity floor carries the prediction side's noise too

*Written 2026-09-21 on reading the first FULL arm (the 200 m/s longitudinal control, seed 0,
twelve cycles), before any other full arm is read. Reader change only; nothing the arms
record changes.*

Amendment 2 derived `D_cont² ≈ (σ_obs² + σ_pred²)/(S² + σ_obs²)` and then implemented the
floor with `σ_obs` alone. The prediction side — the momentum field integrated over a window —
carries its own shot noise. Both are now read on the relaxed last two windows of the aligned
cycle (`Continuity.rms_predicted`, `driven_floor_two_sided`): `floor² = (D_disc² S² + σ_o² +
σ_p²)/(S² + σ_o²)` with `S² = obs_lead² − σ_o²`. On the control arm: `σ_o = 1.02`, `σ_p = 0.22`
counts, `s = 1.41`, **floor `0.60` (one-sided `0.48`)**; the read `D = 0.735` is above it by
`0.14`, so the verdict on this seed is unchanged by the correction: **KILL as staked** — the
`(n̄, p̄)` chart on eight cells along the wave does not close within `β = 0.2` under a 200 m/s
drive, while beating its placebo by `+0.35` and passing its in-run null (`0.98`). The graded
R1 is the three-seed 200 m/s aligned average (Amendment 3 A3); this is one seed. A candidate
for the excess over the floor, to be tested when the 50 m/s longitudinal arms land: at 200
m/s the response is not linear — the driven density and current oscillate at `705–735` fs
(`c_s ≈ 3,200–3,300` m/s, outside R2's `[1,000, 2,200]` band and twice the equilibrium
period) — and a steepened wave carries a second harmonic whose midpoint floor is `0.36`, not
`0.10`. If the 50 m/s arms read the equilibrium period, the control is outside linear
response and R1 is graded on the 50 m/s arms at their floor.
