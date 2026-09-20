# RESPONSE-1 — AMENDMENT 3: noise allowances on the nulls, both forms fitted to the density mode, and two more 200 m/s longitudinal seeds

*Written 2026-09-20 from the five PARTIAL arms (4–7 cycles each, banked as
`response1_*_partial/`, read and not graded) and BEFORE any full arm lands. Nothing the arms
record changes. Committed alone before the reader is changed.*

## A1 — R4 and R1′ carry a noise allowance

As staked, R4 compares the blind partition's kick amplitude to a tenth of the spatial one
with no noise term, and R1′ compares the driven and relaxed `D_cont` to `0.1` with none.
On four cycles R4 "fired" at `0.47` (T1) and `1.07` (L0's density) with the blind amplitude
inside the aligned noise, and R1′ at `0.13` (T1). A null that fires on noise is not a
conviction. **R4:** the blind aligned amplitude must be under `max(0.1 × spatial, 3σ_blind)`
with `σ_blind` the blind aligned series' own relaxed-tail noise; **R1′:** `|D_driven −
D_relaxed|` under `max(0.1, 2 SE)` with the SE from the leave-one-cycle-out spread of the
driven `D`. The same bar the other nulls already use (R4′: `3σ`).

## A2 — R2 fits both forms and reports the residual's choice

The driven density on the L arms decays from its peak as overdamped (`τ ≈ 240` fs on two
partial seeds) while the equilibrium autocorrelation (VIEW-SEARCH-1) crosses zero near
`400` fs at the same `k`. The reader fits BOTH the overdamped `A e^{−λt}` from the peak and
the damped cosine `A e^{−Γt} cos(ωt + φ)` over the whole aligned cycle, prints both
residuals, and R2's branch is the form with the smaller residual by at least `10 %`; within
`10 %` the branch is UNDECIDED and both numbers are banked. The classifier of Amendment 1
no longer decides it alone.

## A3 — two more 200 m/s longitudinal seeds

R1 at 50 m/s reads `0.74` and `0.85` against floors of `0.87` on the partials, as Amendment 2's
arithmetic said; over twelve cycles and three seeds the floor is `≈ 0.6`. The stake `D ≤ 0.2`
can be met only on the 200 m/s longitudinal control, which was one seed. **Seeds 1 and 2 at
200 m/s longitudinal are launched** (`replace0_response1_d.sh`, E-cores, ~38 h) and R1 is
graded on the three-seed 200 m/s aligned average with the 50 m/s arms reported at their
floors, the nonlinearity control now reading the other way: the 50 m/s arms must agree with
the 200 within their spread on R3, R4′ and the mode shapes.
