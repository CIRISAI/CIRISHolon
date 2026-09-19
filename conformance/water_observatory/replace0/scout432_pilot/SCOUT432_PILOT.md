# The fluid-element scout, pilot stage: 432 waters on the rigid operator, three seeds, 5 ps

*Run 2026-09-18 (`replace0_scout_pilot.sh`; records `scout432_seed0..2/scout.json`, walks
appended at every readout). Purpose, staked in `RUNG2_AMENDMENT_5.md`: MEASURE the price per
picosecond at the first admissible box and the averaged fields' held-out spreads at the
cadence, before the ~200 ps run is launched. Read at zero compute under Amendments 4 and 5
(`rung2_amend4.log`, `rung2_amend5.log`). Every gate below is the freeze's.*

## The pilot's own gates, three seeds

| | seed 0 | seed 1 | seed 2 |
|---|---|---|---|
| NVE, peak excursion per water (bar `9.3e-5` Ha) | `2.0e-7` | `2.6e-7` | `1.6e-7` |
| SETTLED, production mean on the six modes (target 293 K, bar ±10 %) | 320.2 K | 316.7 K | 306.9 K |
| **production price** | **3,609** | **3,609** | **3,606** core-s/ps |
| total wall, fine settle + rigid settle + 5 ps | 7.3 h | 7.3 h | 7.3 h |

The fixed 1.1 ps rigid settle left the boxes 5–9 % warm — inside the gate, and the reason the
scout's settle now ends by LIQUID-2's criterion (`0f3944d`). The price is `11 %` above the
linear extrapolation from 128 (`3,250`); it is the number the full run is priced on.

**A diagnostic bug found by this run and fixed in the code, not the record:** `observe`
normalised bonds and cross-unit energy by the 128-water constant, so `scout.json`'s series
carries `bonds_per_water` and `cross_unit_per_water` too large by `432/128 = 3.375`. The
pilot's bonds on the lens are `1.80–1.84` per water (the 128-water liquid read `1.69`). The
walks, which every rung-2 reading uses, are unaffected.

## Instantaneous chart at the first ADMISSIBLE grid (Amendment 4, both routes)

`2×2×1`, 108 per cell, fluctuation 0.025–0.031 — **G2 admissible for the first time in the
programme.** Read at the 20 fs instantaneous cadence, which Amendment 5 says is the wrong
cadence for a fluid element; this is the control the cadence read will be compared against.

| seed | `Δn` (i) | `Δn` (ii) | spatial `D_A` (ii) | G7 (ii) | G7 (i) |
|---|---|---|---|---|---|
| 0 | 2.24 | 3.35 | 0.866 | +0.017, no | −0.040, no |
| 1 | 2.24 | 3.04 | 0.917 | +0.022, no | −0.012, no |
| 2 | 2.24 | 3.03 | 0.941 | −0.026, no | −0.021, no |

No separation on any file under either route; `D_A` 0.87–0.94. At `2×1×1` (216 per cell)
`D_A` = 0.66–0.70, against 0.47–0.53 at 128 waters' `2×1×1` (64 per cell): **at an
instantaneous readout the defect RISES with cell occupancy** — a bigger cell holds more thermal
modes, and read instantaneously it is a noisier number. That is the cadence fault in the exit
`N*`'s own axis, and it is why the full run is read at `τ = a/c_s`.

## The cadence read (Amendment 5), on 5 ps

Six windows of 785 fs per seed at `2×2×1`: VOID by work count on every file, as staked. The
held-out spreads of the averaged fields, which the full run's bins will be:

| seed | `σ(n_avg)` held-out | own (not used) | `σ(p_avg)` au | `σ(e_avg)` Ha |
|---|---|---|---|---|
| 0 | 1.305 | 1.246 | 25.4 | 5.4e-3 |
| 1 | 1.243 | 1.368 | 25.2 | 5.0e-3 |
| 2 | 1.308 | 1.240 | 23.1 | 5.2e-3 |

Stable across seeds to ±5 %, so the hold-out will calibrate the full run to that precision.
The averaged half-box momentum's spread at 432 is `25` au against an instantaneous `~60`
(a `2.4×` fall over 39 frames where independent draws would fall `6.2×`) — the same slow
correlated content the 128-water read showed, at the admissible size.

## The full run, priced on the measured number

`250 × 785 fs = 196 ps` a seed, at `3,609 core-s/ps`: **`197` core-hours a seed, `8.2` days
of wall on one core each**, plus the criterion settle (floor 1.1 ps, cap 11 ps: `1–11` h).
Three seeds in parallel: **about 8.5 days of three cores.** Six seeds on six cores costs the
same wall and halves the hold-out's uncertainty; the machine has the cores when other work
allows. Walk files: `10,001` rows × 432 oxygens, ~`250` MB per file, `1.5` GB for three seeds.


---

## The continuity leg on the pilot (Amendment 6, 2026-09-18, zero compute)

Read at Amendment 5's cadence on every grid the leg admits (three or more cells per split
axis). `rung2 --amend6`, log `rung2_amend6.log`.

| grid | `⟨n⟩` | `τ` | transitions | `D_cont` spatial (3 seeds) | blind | separation | shot-noise floor, est. |
|---|---|---|---|---|---|---|---|
| 4×4×4 | 6.75 | 392 fs | 11 | 1.017, 1.010, 1.015 | 1.02 | +0.01 | ~0.7 |
| 8×4×4 | 3.4 | 196 fs | 24 | 0.992, 0.991, 0.997 | 1.01 | +0.02 | ~0.8 |
| 8×8×4 | 1.7 | 196 fs | 24 | 0.996, 0.998, 1.001 | 1.02 | +0.02 | ~0.9 |

The momentum field predicts the density field's motion **no better than a scrambled one**, on
every seed and every admissible grid, and every reading sits at the shot-noise floor for its
crossing count. The leg has no power at this size: a face on this box sees a few molecules
cross per window, and a ratio of counts that small cannot resolve a flux. The collision form
and the named law agree, from opposite sides, that 432 waters is below the fluid element —
and the named law says what size would let it speak: hundreds of crossings per face per
window, thousands of molecules per cell.
