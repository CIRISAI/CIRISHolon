# ORDER-1 — READ: branch (e) as written. PO-4, the planted carried field, came in at `+0.046` against its bar of `0.05`, so nothing is graded. Reported beside the verdict: the first-harmonic tetrahedral-order field closes at about `q`'s own memory and lies outside the hydrodynamic span, and it is NOT carried. Its increment to the next longitudinal state is `−0.001` to `−0.013` on every 293 K seed, where a planted field at half the density's sd reads `+0.020`. Arm D was refused: the walk carries no orientation.

*2026-09-25. Prior art `ORDER1_PRIOR_ART.md` (`0880b6c`), prereg `ORDER1_PREREG.md` (frozen
alone, `e53a803`), reader `order1_search.py` with `order1_read.sh` (committed `186b9ad`,
unchanged since the freeze). The read is `replace0/order1/order1_read.txt`. The POST-FREEZE
plant-amplitude scan, labelled so, is `order1_plant_scan.{py,sh}` →
`replace0/order1/order1_plant_scan.txt`. Carrier: the SLOW-1 rigid walks (250 waters, 50 ps at
20 fs; three seeds at 293 K, production 304–317 K; one at 400 K, production 417 K), read-only.
Compute: about a minute on `taskset -c 16-20`.*

*Order of events, stated plainly.* The read and the scan were first run in this session with
the reader as frozen, before the reader was committed. A rate limit interrupted the session.
The reader and scripts were then committed (`186b9ad`), both runs were repeated from the
committed files, and the outputs are **byte-identical** to the first runs. The instrument is the
one the prereg describes. None of its lines changed after `e53a803`.

## 0. Verdict

| item | staked | read | verdict |
|---|---|---|---|
| **Arm D** | runs only if orientation is on the walk | `rigid.walk` = oxygen positions only (`3 × 250` per readout) | **REFUSED by name** |
| **PO-1** regression | VIEW-SEARCH-1's held-out reads to `1.5e-3` | Fourier-all `13.238`/`12.163`, density `4.945`/`5.648`, cells `11.767`/`15.119`, all exact | PASS |
| **PO-2** re-paired nulls | `≤ 0.01` | time-shifted `−0.015 … +0.0001`; seed-swapped `−0.007 … +0.005` | PASS |
| **PO-3** time-shuffle | `σ₁ < 0.1` at lags `≥ 1` ps | max `0.018, 0.024, 0.033` (293 K), `0.022` (400 K) | PASS |
| **PO-4** planted carried field | O1 legs; loading `≥ 0.9`; increment `≥ 0.05`; null `≤ 0.01` | legs met (`σ₁ 0.691`, ratio `0.88`); loading **`0.931`**; increment **`+0.0462`** at 1 ps; null `+0.0002` | **FAIL, by `0.004` on the increment** |
| **PO-5** 400 K contrast | 400 K `σ₁` at 1 ps below every 293 K seed | `0.159` < `0.293` | PASS |
| **BRANCH** | | | **(e): a plant fails, nothing is read** |

## 1. Reported, NOT graded: what the stakes would have read (the plant failed)

| stake | 293 K seeds 0 / 1 / 2 | 400 K | would read |
|---|---|---|---|
| **O1** `Q`'s own `σ₁` at 1 ps; residual ratio after projecting out its own k-vector's `ρ, j^L, j^T` | `0.310 / 0.293 / 0.360`; ratio `0.98 / 1.04 / 0.95` | `0.159`; `0.96` | **between.** Seed 1 misses `0.3` by `0.007`. The ratio clause holds everywhere: `Q_k` is outside the hydrodynamic span (equal-time `R²` of `Q_k` on its k-vector's sector is `0.038 / 0.012 / 0.040`) |
| **O2** `(ρ, j^L)` increment, τ = 1 ps | `−0.0032 / −0.0007 / −0.0033` | `−0.0004` | **KILL**: mean `−0.0024` at 1 ps and `−0.0088` at 5 ps, both below `0.02` |
| O2, τ = 5 ps | `−0.0126 / −0.0049 / −0.0089` | `+0.0001` | |
| **O3** one time or two; mixture | one time on all three. `τ(Q)` `2.50 / 3.11 / 1.69` ps against the self `q` memory `2.20 / 1.91 / 2.27` (ratio `1.14 / 1.63 / 0.75`); α time `3.70 / 2.40 / 3.60` ps. Density `R²` of the Q-dominant direction `0.217 / — / 0.154` | `τ(Q) 1.23` vs self `0.48` | split by the letter: DIRECTION / RE-FINDING / RE-FINDING. Seed 0's "mixture" is `0.017` over a bar that sits at the noise level (§3) |

**Had PO-4 fired, the branch would have been (d) by O2's kill: closed, not carried.**

### Checks beside the read (reported)

- **Seed-held-out increments** (τ = 1 ps): `−0.0008, −0.0029, −0.0018`. At 5 ps: `−0.0010, −0.0001, −0.0016`.
- **Beyond `(ρ, j^L) + K`:** the same increments, to `10⁻⁴`.
- **The energy proxy `K_k`:** its own increment is at most `0.002` in size, and its own `σ₁` at 1 ps is `≤ 0.06`. It relaxes on a kinetic timescale, so it is not the heat mode. The heat-mode confound is therefore **untested, not excluded**: the walk carries no potential energies.
- **The literal 30-column form:** every increment is negative (`−0.01` to `−0.14`). Its `Q` `σ₁` is `0.59–0.61` at 1 ps. That is the over-fitting of cross-k couplings the prereg named (§6.1).
- **Static coupling:** `corr(ρ_k, Q_k) = −0.18, −0.11, −0.18` at 293 K. The sign is the two-state sign: more tetrahedral means less dense. It is weak, as Sedlmeier–Horinek–Netz report. At 400 K it is `+0.06`.

## 2. The instrument's sensitivity (POST-FREEZE, NOT a plant of record)

The same plant on the same carrier (`T293_seed0`), varying only its amplitude:

| amplitude (× density sd) | 0.5 | **0.7 (frozen)** | 0.85 | 1.0 | 1.5 |
|---|---|---|---|---|---|
| `(ρ, j^L)` increment at 1 ps | `+0.020` | **`+0.046`** | `+0.067` | `+0.087` | `+0.136` |
| O1 residual ratio at 1 ps | `0.95` | `0.88` | `0.81` | `0.74` | `0.53` (O1 leg fails) |
| PO-4 as a whole | fail | **fail** | pass | pass | fail |

This moves no verdict. It does two things:

1. **It scales the null result.** A field planted at half the density's sd reads `+0.020`. The real `Q_k` reads `−0.003 ± 0.003`, which is at the nulls' spread. Nothing that carries even a quarter of that planted field is present at τ = 1 ps.
2. **It shows the window between O1 and O2.** The window where PO-4 passes, amplitude `0.85–1.0`, is narrow. A field carried strongly enough is also visible in `ρ_k` at equal time, so O1's ratio leg rejects it (at `1.5`). The stake's two legs pull against each other. This tension was named in the prereg (§6.2) and was not amended.

## 3. Reading against the prior art

Every number is what `ORDER1_PRIOR_ART.md` §5, item 1, expected at ambient T and this k.

- **The collective field behaves like the self field.** `Q_k`'s decay is the single-molecule `q` memory, to within the noise: ratio `0.75–1.63` on 50 ps. That is the self part at `F_s ≈ 1`.
- **It is weakly coupled to density**, with the two-state sign.
- **It is not carried into the longitudinal hydrodynamics** at `k = 0.32 Å⁻¹` on 1–5 ps. Generalized hydrodynamics and MCT put the structural relaxation's effect on the sound pair through the longitudinal stress memory. At this k and T that effect is below this instrument's resolution: about `0.005` in `R²`, with the plant's `+0.020` at half amplitude as the yardstick.

This is a **re-finding in kind**: extended hydrodynamics' relaxing structural variable, a local relaxation summed over the box. By the stakes as written it would have been branch (d): **closed but not carried.** That is the same shape the hydrogen-bond network showed (`TSCAN_HBOND_SEARCH_RESULTS.md`). SLOW-1's small carried increment for per-molecule `q` (`+0.028–0.040` into five-picosecond MOBILITY) does not reappear here. The carrying SLOW-1 measured is into a single-particle quantity, and it does not propagate to the collective density–current sector at the box's longest wavelength.

**For the lead's question.** On this carrier the order parameter is a closed view that the level above does NOT carry. The "weak one" on the sluggish liquid carries only into a per-molecule observable, not into the hydrodynamics. The two-state literature's distinctive signatures did not appear at the read's resolution: two times, a collective component slower than the self one, or a carried Q–ρ mixture. This is a statement about this operator at 304–317 K and `k = 0.32 Å⁻¹`. It is not about supercooled water.

## 4. What the prereg got wrong

1. **PO-4's amplitude was calibrated on a synthetic carrier** whose density sector differed from the real one. On the synthetic carrier `0.7` gave `+0.12`. On the carrier of record it gave `+0.046`. The plant was under-sized by `0.004`, and branch (e) follows from that alone. There is no 50 ps walk off the read with which to calibrate. An honest calibration needed a relative bar, for example the plant's increment at least `5×` the nulls' spread, or the amplitude chosen on the real carrier's hydrodynamic sector with `Q` never computed.
2. **O1's `σ₁ ≥ 0.3` at 1 ps sat on the knife-edge** of the known answer. SLOW-1 had already read `0.30–0.35` for per-molecule `q`, and the collective field reads `0.29–0.36`.
3. **O1's residual-ratio leg and O2 are in tension** (§2). A strongly carried field fails "not in the hydrodynamic span".
4. **O3's two-time test began at 0.5 ps.** The autocorrelation falls from `1` to `0.6` within 0.1 ps, so the fast part is outside the window. The two-exponential fits then degenerate to one time (two equal times). The KWW exponents above `1` are artefacts of fitting noisy tails. The mixture bar of `0.2` sits at the noise of the Q-dominant direction's density loading (`0.15–0.22`, varying with lag).
5. **O2's 400 K clause** ("less at 400 K") is empty when the 293 K increments are `≤ 0`.
6. **The lead's literal dictionary** (all 30 columns in one vector) over-fits symmetry-forbidden cross-k couplings. It was caught in building, and the per-k-vector chain form was made primary before the freeze.
7. **The heat mode cannot be tested on these walks** (no energies). `K_k` is not a proxy for it.

## 5. What would move this

A walk that banks orientations, hydrogens or per-molecule energies. That would allow Arm D (Debye's collective dipole, whose main peak Hansen et al. 2016 place as supramolecular, NOT structural) and a real test of the heat mode. A larger box would give a smaller k, where the structural memory's lever on sound is larger relative to hydrodynamic damping. A supercooled arm, where the two-state picture predicts a collective component. None of these walks is on disk.

---
witness: none (a measured campaign; numeric gates; closure algebra `Closed` and `StatClosure.lean`, cited in words)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-PLACEMENT-LOTTERY, M-FLOOR-UNSTAKED. M-PLANT-OBS: the plant of record failed on its carrier after passing on a synthetic one. This is the registry's shape again, a plant calibrated off the observable's own carrier. M-FLOOR-UNSTAKED: the resolution (`~0.005` in `R²`) is read from the nulls' spread and reported, not staked.
