# TSCAN-1 READ: the carrier is a LIQUID that diffuses — `D = 6.5 × 10⁻¹⁰` m²/s at 311–317 K, a quarter of water's, with water's activation energy — and the "glass on 40 ps" was the lead's overreach from a 5 ps run

*2026-09-20 23:15 CDT. Prereg `TSCAN1_PREREG.md` (alone, `60776f4`), reader `tscan1_read.py`
(plants PT-1, PT-2 pass), eight arms on the rigid operator at 128 waters, records in
`replace0/tscan1/` (`READ.txt`). No band on `D(T)` was staked; it is the model's number.*

## 0. Verdict

| stake | staked | read | verdict |
|---|---|---|---|
| **S1** arrested at 293 K | `D(293) < 0.3 × 10⁻⁹` m²/s; kill `≥ 1.0` | **`5.4, 7.7 × 10⁻¹⁰`** (rigid modes at 312, 317 K), MSD linear to `0.98` | **neither met nor killed**: the carrier diffuses, at a quarter of water's rate |
| **S2** the arrest is thermal | `D(500)/D(293) > 5` and `r(500) < 0.5` | `18×`; `r = 0.26` | **MET** |

**Branch:** between (a) and (b). The carrier is not a glass. It is a liquid three to four times
more sluggish than water at the same temperature, and its diffusion is Arrhenius with
**`E_a = 19.0` kJ/mol** over 312–526 K against water's 18–20: the barrier is water's, the
prefactor is a third of water's. The rigid operator diffuses (S2), so no flexible scan is
owed on that account.

| `T` target | rigid modes read | `D` (m²/s) seeds 0, 1 | `r = |offset|/sd` | MSD at 2.5 ps |
|---|---|---|---|---|
| 293 | 312, 317 K | `5.36, 7.74 × 10⁻¹⁰` | 0.65, 0.40 | 1.4, 1.7 Å² |
| 350 | 356, 362 | `2.13, 2.55 × 10⁻⁹` | 0.66, 0.33 | 3.7, 4.3 |
| 400 | 400, 412 | `4.54, 4.78 × 10⁻⁹` | 0.29, 0.55 | 7.3, 7.8 |
| 500 | 514, 526 | `1.13, 1.26 × 10⁻⁸` | 0.38, 0.13 | 16.9, 18.3 |

The production ran 6–8 % warm on every arm (the rigid settle's criterion fires before the
NVE temperature has relaxed to the target; inside the 10 % gate, and the `D` are quoted at the
temperature read, not the one targeted).

## 1. The correction: "a glass on 40 ps" was wrong, and the fast side says why

On 2026-09-20 the lead wrote into `OBJECT.md`, `STANCE.md` and `VIEW_SEARCH_RESULTS.md` that
the carrier "does not rearrange its longest-wavelength density in 40 ps" and "is a glass on
that time". The evidence was a static offset of 1–2 counts on the first-harmonic density
modes over 5–6 ps runs. The arithmetic that should have preceded the sentence: a mode with
correlation time `τ_c` read over a run of length `T` has `|mean|/sd ≈ √(τ_c/T)`, so `r = 0.4–0.65`
on 5 ps means **`τ_c ≈ 1–2 ps`** — a relaxing mode, not a frozen one — and it falls to `0.26`
at 500 K as it should. And the longest-wavelength density mode of ANY liquid at these box
sizes is slow: its diffusive relaxation `1/(Dk²)` is `95` ps at 128 waters and `215` at 432 for
this model, and `27` and `61` ps for real water. A pattern that persists over a few
picoseconds at `k = 0.27 Å⁻¹` is what water would show too. **The fifteenth instance**: a
persistence read on a run shorter than the mode's own time, and called a phase. Retracted
in the three documents with this note beside it.

## 2. What stands

The carrier is licensed one step further than before: a liquid of this model, diffusing,
with water's activation energy and a third of its mobility at 293 K — "supercooled" only in
the weak sense that it is slower than water at the same `T`; there is no arrest in the range.
The fluid stakes are stakes on a slow liquid, not on a glass. The `η(k)` reads (VIEW-SEARCH-1,
the partial arms) now sit beside a `D`: the Stokes–Einstein product `Dη/T` at 312 K with
`η ≈ 4–5 × 10⁻⁴` is `1.0 × 10⁻¹⁵` Pa m² K⁻¹ s⁻¹... in ratio to water's (`2.3e-9 × 1.0e-3 / 293`
`= 7.9e-15`), **`0.13`** — the model's viscosity is water-like while its diffusion is a quarter
of water's, so Stokes–Einstein is violated by `~5×` at this `k`, which is either the model's
or the `k`-dependence of `η` at `0.27 Å⁻¹` (a hydrodynamic `η` at `k → 0` would be larger). A
reading to carry, not a stake.
