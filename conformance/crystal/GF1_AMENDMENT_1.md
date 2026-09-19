# GF1 — AMENDMENT 1: the instrument's true price, the non-local magic defined so it can vary, and the ladder re-sized to what can be read exactly

*Written 2026-09-19, committed alone, AFTER the instrument was built and its plants fired
(`q8-mps/src/magic.rs`, `tests/gf1_plants.rs`, merged at `526befb`) and BEFORE any vacuum
ladder is read. Amends `GF1_PREREG.md` in the three places its correction names. The stakes
S1, S2 and S4 stand; S3 is re-staked on a quantity that is not a theorem's constant.*

## A1 — the price, derived this time

The exact fourth-power sum over Pauli strings is a four-replica contraction: `χ⁸` doubles of
state per site and `64 χ⁹` multiply-adds per site. Measured on the merged instrument, single
thread: `N = 24, χ = 8` in `42` s; `N = 16, χ = 10` in `62` s; `χ = 12` refused at `13.8` GB.
**The exact instrument reads `χ ≤ 11`.** Its floor is `≤ 10⁻¹⁴` absolute (replica against brute
enumeration at `N = 10`: `3.6 × 10⁻¹⁵`; two contraction orders at `N = 24`: `7.9 × 10⁻¹⁴`, after
the χ⁸-term overlap was moved to pairwise summation — the sequential sum had left
`7.8 × 10⁻¹³`), which retires the prereg's `4^N ε` estimate.

Beyond `χ = 11` two exits exist and neither is built: a compressed Pauli MPS
(Tarabunga, Tirrito, Bañuls, Dalmonte 2024), which is approximate and would need its own
error gate; and perfect Pauli sampling with a statistical bar (Lami & Collura 2023), which
is exact in expectation and carries its own variance. **If a ladder point needs `χ > 11` to
pass its variance gate, the point is REFUSED rather than read approximately**, and the
refusal is a reading about the vacuum's bond dimension, banked as such.

## A2 — the non-local magic, defined so a theorem does not hold it fixed

`M₂` is invariant under every Clifford, so minimising it over local Cliffords does nothing.
What prices a stabilizer-rank simulation beyond a product-state prefactor is the magic that
survives the best LOCAL UNITARY frame — the minimum of `M₂` over `⊗_j U_j` with `U_j` a
continuous single-site rotation. On a real MPS that is one angle per site:

> **`M₂^nl = min_{θ_1..θ_N} M₂( ⊗_j R_y(θ_j) |ψ⟩ )`**, `R_y(θ)` the real rotation on the
> physical index, by coordinate descent over sites with a golden-section line search on each
> `θ_j ∈ [0, π)`, three sweeps, the minimum over sweeps; deterministic (sites in order, the
> search seeded from `θ = 0`).

A product state's `M₂^nl` is zero exactly (a single-site rotation takes each factor to
`|0⟩`) — plant P3 is re-staked on that, and it can now fire. A GHZ state's is not zero: its
magic is in the entanglement. `M₂^nl ≤ M₂` by construction, and the gap `M₂ − M₂^nl` is the
local, removable part.

**S3, re-staked:** at each coupling `x`, the least-squares slope of `M₂^nl(N)` in `N` over the
volumes read is under `0.01` per site. **Kill unchanged:** a slope `≥ 0.01` at any coupling
kills Fold III as written. **What would make this stake vacuous, said now:** if `M₂^nl` is
found extensive with the SAME density as `M₂` (i.e. the local frame removes nothing on the
vacuum), then Fold III's "area-law" has no quantity to be true of, and branch (b) is entered
on that finding rather than on a slope.

## A3 — the variance gate, declared

A state is a vacuum for this campaign when `q8-mps::variance::energy_variance` reports
**`⟨(H − E)²⟩ / N ≤ 10⁻³`** in the model's energy units — the threshold plant P6 used and
passed. A state above it is refused before magic is read.

## The ladder, re-sized to the exact instrument

Couplings `x ∈ {0.25, 1, 4, 16}`; volumes `N ∈ {8, 12, 16, 24, 32, 48}`; **`χ ≤ 11`**, the
smallest χ passing A3 at each point. P6 already showed `x = 4, N = 16` passes at `χ = 10`
(variance `6.8 × 10⁻⁵` per site) with `M₂ = 5.7004` stable to `6.9 × 10⁻⁴` against `χ = 8`.
Whether `N = 48` passes at `χ ≤ 11` is unknown and is the first thing the ladder reports;
if it does not, the volume range is what it is and S1/S3 are read on the points that pass.

## Cost, re-derived

DMRG for the 24 points at `χ ≤ 11`: minutes each, under an hour. Exact `M₂`: `N = 48,
χ = 11` at `64 χ⁹ ≈ 1.5 × 10¹¹` per site, seconds to a minute per state. `M₂^nl`: three sweeps
of `N` line searches of `~20` evaluations each — `~3,000` `M₂` evaluations per state at `N = 48`,
**an hour per state on one thread, under a day for the ladder on 16.** The prereg's "under
40 core-hours" survives; its "one to two days to build" was three hours of a subagent plus
the corrections here.

## Plants added

| plant | must |
|---|---|
| **P3′** | a product of H-type states rotated by random single-site unitaries: `M₂^nl = 0` to `10⁻⁹`, where `M₂` itself is `0.415 N` — the new minimiser removes what the Clifford one could not |
| **P7** | a GHZ state: `M₂^nl = M₂ = 0` (stabilizer) — no false magic manufactured by the search |
| **P8** | a random MPS at `N = 12, χ = 6`: `M₂^nl ≤ M₂`, the fourth sweep improves the third by under `10⁻⁶`, and two different site orders agree to `10⁻⁶` (the descent is not order-trapped at this size) |
| **P9** | `χ = 12` requested: REFUSED by name with the lease it would need |
