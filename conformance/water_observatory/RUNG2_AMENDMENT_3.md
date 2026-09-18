# RUNG 2 — AMENDMENT 3: the momentum and energy bins DERIVED from the cell's own equilibrium fluctuation

*Written 2026-09-17, committed alone, BEFORE the re-read it is for. Replaces Amendment 2's
rule for the two continuous fields; keeps Amendment 1's density rule and its grid and ladder
unchanged. Written because Amendment 2's rule was calibrated on an unrepresentative smoke
box and failed on the first settled data — recorded in the correction at the head of that
document.*

## The rule

For a cell of mean occupancy `⟨n⟩ = N / cells` on a carrier of `N` atoms of mean mass `m̄`
(from the trajectory header; for a single species, that species' mass):

> **`Δp_cell = √( ⟨n⟩ · m̄ · k_B T_target · (1 − ⟨n⟩ / N) )`** per component
> **`Δe_cell = √( 3⟨n⟩ / 2 ) · k_B T_target`**
> **`Δn_cell = √⟨n⟩`** (Amendment 1, unchanged)

Each is one standard deviation of the cell's field at equilibrium for `⟨n⟩` independent
thermal atoms — the same statistics G2's `1/√N` rests on — with two things the freeze's
per-atom `Δp = √(m_H k_B T)` could not carry: the carrier's own mass instead of hydrogen's,
and the finite-population factor `(1 − ⟨n⟩/N)` for a system whose total momentum is
conserved. That factor is exact for a zero-sum sample and it makes the formula say, by
itself, that the one-cell momentum field is constant (`Δp_cell = 0` at `⟨n⟩ = N`) — G3's
vacuity control, derived rather than declared. Kinetic energy is not conserved on its own
(it exchanges with the potential), so `Δe` takes no such factor.

**For the refinement self-check** the two continuous bins are used as the integer multiple
of the freeze's own `Δp`, `Δe` nearest the derived value — the `floor` theorem Amendment 2
found (`round(Δp_cell / Δp)`, minimum 1) — so the freeze's chart refines this one exactly.

## Checked on the data it failed on, before this was written

| grid | `⟨n⟩` | `Δp_cell` derived | measured `σ(P_x)`, 4 files | ratio | `Δe_cell` derived | measured `σ(E)` | ratio |
|---|---|---|---|---|---|---|---|
| 2×1×1 | 64 | `29.8` au (`23 × Δp`) | `39.5` | `1.30` | `9.3e-3` (`10 × Δe`) | `1.14e-2` | `1.20` |
| 4×1×1 | 32 | `25.8` au (`20 × Δp`) | `29.0` | `1.10` | `6.6e-3` (`7 × Δe`) | `8.5e-3` | `1.28` |

The residual `1.1–1.3` is the mode split `REPLACE-0` measured on this model — the oxygens'
rigid modes sit near `375` K while `T_target` is `300` (`√(375/300) = 1.12` for momentum,
`1.25` for energy). The bin is kept at `T_target` because that is a protocol constant and
the heating is a measured property of one carrier; a carrier at its target would sit at
`1.0`. **This check is a check.** The formula was written from the statistics of a
zero-sum thermal sample before the numbers in the table were computed; the smoke that
misled Amendment 2 is excluded from it and the reason is stated there.

## What the momentum rung can and cannot deliver at this size, said now

With `σ/bin ≈ 1.3` a component spans about nine bins, and at `2×1×1` the two cells' momenta
are anticorrelated by conservation, so the chart has of order `9² × 3 ≈ 250` reachable
readings against `300` frames. The momentum rung may reach G4 at `2×1×1` and will not at
`2×2×1` on this run; if it does not, the record will say what readout count would, computed
from the collision rate, and that is a length and not a defect. Nothing here predicts
whether density + momentum closes.

## Plants

| plant | carrier | must |
|---|---|---|
| **PC-1** | `N` independent thermal atoms of one species at `T_target`, walking, zero total momentum enforced per frame | the measured `σ(P_x)` and `σ(E)` at `2×1×1` within `15 %` of the derived bins — the formula is the physics of its own carrier |
| **PC-2** | the same | `Mom` rung: VOID by counting under the freeze's `Δp`; collides and meets G4 under `Derived` |
| **PC-3** | every trajectory, every rung | `refines(Exact, Derived)` — the freeze's chart refines this one; a violation convicts the rounding |
| **PC-4** | P-3's hidden variable with thermal velocities | `NotClosed` at every rung under `Derived` |
| **PC-5** | any carrier, `Occ` rung | `Derived` readings equal `Poisson` readings exactly |

Amendment 2's `CellScale` is kept in the instrument and printed beside `Derived` as the
control that shows what this amendment changed, and as the record of what was wrong.
