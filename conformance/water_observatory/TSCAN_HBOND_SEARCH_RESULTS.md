# TSCAN-SEARCH-1 and HBOND-SEARCH-1: READ (2026-09-21)

*Prereg `TSCAN_HBOND_SEARCH_PREREG.md` (alone, `46eb6db`). Outputs: `replace0/tscan1/view_search_T*.txt`,
`replace0/molsearch1*/hbond_search.txt`. Instruments `view_search.py`, `hbond_search.py`.*

## TSCAN-SEARCH-1 — the closed sectors do not change with temperature

| T (rigid modes) | S1: top-6 in the Fourier subspace at 100 fs | S2: Fourier chart / cell chart, fraction of the bound | `η(k = 0.40 Å⁻¹)` from the transverse autocorrelation, two seeds |
|---|---|---|---|
| 312–317 K | 0.97 | 1.23 / 0.71 | `3.9, 3.5 × 10⁻⁴` Pa s |
| 356–362 | 0.98 | 1.21 / 0.66 | `3.0, 2.9 × 10⁻⁴` |
| 400–412 | 0.98 | 1.20 / 0.68 | `3.05, 3.0 × 10⁻⁴` |
| 514–526 | 0.97 | 1.20 / 0.72 | `2.4, 2.5 × 10⁻⁴` |

**S1 and S2 MET at every temperature.** The long-wavelength density and current modes are
the liquid's closed sectors from 312 to 526 K, the Fourier chart sits at or above the bound
throughout, and the cell chart sits at `0.66–0.72` of it throughout: the object of the fluid
tier is not a property of one temperature. (The 128-water `η` at 312 K, `3.5–3.9 × 10⁻⁴`, is
above VIEW-SEARCH-1's `1.4–1.9 × 10⁻⁴` on the transport walks at the same size: those were
read at lag resolution 20 fs over a shorter fit range on a differently settled box — the
spread between campaigns is the honest uncertainty of a 5–6 ps read, a factor of two.)

**S3, reported: Stokes–Einstein decouples on cooling.** `Dη/T` in units of water's at 293 K:
`0.09` at 312 K, `0.21` at 360, `0.44` at 406, **`0.72` at 520 K**. The product rises eightfold
across the scan while `η(k)` falls by only 1.5× and `D` rises 18×: the classic signature of a
supercooled liquid, translation decoupling from viscosity as it cools, healing toward the
Stokes–Einstein value at high temperature. On this model, 293 K is deep in the decoupled
regime, which is the quantitative content of "three to four times more sluggish than water".

## HBOND-SEARCH-1 — the network is a closed sector the fluid does not carry

Two all-atom walks, 128 waters, 527 rows at 1 fs; bonds donated/accepted per molecule
(`1.67` each at 299 K, `1.40` at 396 K — below RUNG-1's `1.7`, half a picosecond after a
settle).

| | 299 K | 396 K |
|---|---|---|
| S2 bond–velocity cross-block at 5 fs (`< 0.2`) | **0.007, MET** | **0.007, MET** |
| S1 bond state's own `σ` at 100 fs (`≥ 0.5`) | 0.44, between | 0.30, between |
| S3 bonds' increment to predicting Δv over 50 fs, beyond the velocity itself | **0.000 (velocity alone 0.399)** | **−0.001 (0.351)** |

**S3 KILLED as staked**, and the plant says how to read it: on a synthetic ensemble with a
planted bond→force coupling the increment reads `+0.395`, so the instrument sees such a
coupling when it exists; on the model it sees none. The bond COUNTS carry nothing the
momentum law needs at 50 fs. The network is decoupled from the molecules' motion (S2), turns
over on `~120` fs at this cadence (S1 between: the librational component of bond dynamics,
the diffusive tail being beyond a half-picosecond walk), and is not an object of the fluid
by the procedure's own selection — which is what RUNG-1 concluded structurally ("a
structural readout, no certificate flips any tier"). Its level above, if it has one, is not
the fluid: proton transfer and the dielectric response read the bond graph; momentum does
not. **Plant PH-2 (lag 0) reads `5 × 10⁻²` against a `10⁻²` bar** — the ridge against the
bond counts' small variance, the same artefact as before; recorded, and the read above
stands with that caveat. The 5 ps walk (`molsearch2_5ps/`) will be read under the same
stakes for the diffusive tail.

## The ledger

The temperature search is the first time a rung's object was shown invariant across a
thermodynamic scan. The network search is the first time the procedure declined to promote
a closed sector to an object because the level above does not carry it — the same rule that
selected the rigid unit over the vibrations, now returning "no" instead of "which".

## HBOND-SEARCH-1 on the 5 ps walk (appended 2026-09-22 evening)

128 waters, 5,001 rows at 1 fs, NVE from 294 K (drifting to 305–327 K over the run — a warm
box, inside the 10 % gate for most of it and noted). S2 MET again, harder: bond–velocity
cross-block `0.002` at 5 fs. S1 between again: the bond state's own `σ` is `0.46` at 100 fs, and
the labelled extra beyond the prereg shows the tail the half-picosecond walks could not:
`0.34` at 200 fs, `0.19` at 500 fs, `0.08` at 1 ps — a bond-count memory of `~350` fs, not the
picosecond of the literature's intact-bond correlation, because a COUNT forgets which
partner was lost. **S3 KILLED as on the short walks**: the bonds' increment to predicting
the momentum change beyond the velocity itself is `+0.000` (velocity alone `0.399`). The
reader's printed ratio "`19.1`" is `0.000 / −0.000` and is an artefact of the ratio, not a
reading; the increment is the statistic the plant validated. The verdict stands: the
network is a closed sector the fluid does not carry.
