# RUNG 2 — AMENDMENT 4: the density bin for a LIQUID, two declared routes read side by side

*Written 2026-09-17, committed alone, BEFORE the re-read. Answers the finding of
`replace0/rung2_mom/RUNG2_MOM_ON_THE_3D_CARRIER.md`: Amendment 1's `Δn = √⟨n⟩` is the
Poisson spread of independent particles and is `3×` a liquid's, so the spatial density
chart was under-resolved and G7 compared charts of unequal resolution (branch (e)). The
operator's question — *why not both and compare?* — is the protocol: two pre-declared bins,
neither fitted to the graded trajectory, read on identical frames, with the comparison
itself a reading.*

## Route (i) — EXTERNAL: water's structure factor as a declared constant

> **`Δn = √( S(0) · ⟨n⟩ · (1 − ⟨n⟩ / N) )`** per species, with
> **`S(0) = ρ k_B T κ_T = 0.0621`** for water at 25 °C from CRC constants
> (`κ_T = 45.24 × 10⁻¹¹ Pa⁻¹`, `ρ = 997.05 kg m⁻³`, `T = 298.15 K`).

`S(0)` enters exactly as `T_target` does: an external protocol constant, not a property read
off the model. The finite-population factor is kept for the same reason Amendment 3 keeps it
for momentum — the particle number is conserved and a half-box is a sample of a closed total
— so this is one rule for every field, at a liquid's fluctuation instead of a gas's. **What
it cannot carry, said now:** `S(0)` is the `k → 0` limit and a cell three molecules wide is
not in it; the cell's surface adds variance the formula lacks. That is the correction route
(ii) measures.

## Route (ii) — CALIBRATED: the liquid's own spread, on held-out seeds

> **`Δn = σ(n)`** measured on the FLEXIBLE arms of the seeds NOT being graded, at the same
> grid, pooled in quadrature. Three seeds, three-fold hold-out. The rigid arm of a seed is
> graded with the same calibration as its flexible arm (the flexible model is the reference
> liquid).

The precedent is LIQUID-2's settling band, set from LIQUID-1's scatter: a scale measured on
a separate trajectory of the same system. **Binning at the spread of the trajectory being
graded is fitting and is refused** — the hold-out is what keeps this a measurement. Its own
uncertainty is stated: the three seeds' `σ(n)` at `2×1×1` are `2.49 / 2.20 / 1.90`, a
`±15 %` spread, which the reading carries.

## The comparison, and what each outcome means — staked before the numbers

Both routes are read on the six files of the momentum run, every grid, every control, with
Amendment 3's momentum and energy bins unchanged. Reported side by side: the two bins, the
spatial and blind `D_A`, and G7's separation under each.

- **Both routes give the same G7 outcome** → the verdict is robust to the density bin, and
  the ratio `Δn(ii) / Δn(i)` is a measured finite-cell correction to `S(0)` at this box.
- **They disagree on G7** → the tier's reading is sensitive to a scale no protocol constant
  fixes, which is itself the finding; nothing is concluded about closure, and the record says
  what a larger cell would be needed to make the two converge.
- **Either route leaves the blind control as closed as the spatial chart** → branch (e)
  stands under that route, with the mechanism now unable to be the bin.

## Plants

| plant | must |
|---|---|
| **PD-1** | `External` and `Calibrated(σ)` readings at the `Occ` rung equal `Poisson`'s when `S(0)⟨n⟩(1−⟨n⟩/N)` and `σ²` are set to `⟨n⟩` — the routes are A1 at a different scale and nothing else |
| **PD-2** | on a zero-sum thermal carrier (`S(0) = 1` by construction) `External` with `S(0) = 1` reproduces `Derived` bit for bit |
| **PD-3** | `refines(Exact, External)` and `refines(Exact, Calibrated)` at every rung — the self-check |
| **PD-4** | `Calibrated(σ)` with a σ from a DIFFERENT seed than the one graded, asserted in the driver: a calibration drawn from the graded file is refused, not warned |

---
*Audit footer, added 2026-09-21 for `Audit/prereg_audit.py` after CI read red since 2026-09-19; no stake, gate, plant or number above moved.*
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR — the registered ids this text contacts by keyword, cited at the audit's demand.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
