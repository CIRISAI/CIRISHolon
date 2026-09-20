# TSCAN-1 — the model's own phase against temperature: PREREGISTRATION

*Written 2026-09-20, committed alone before the runs. The liquid carrier's phase at 293 K is
unmeasured (`OBJECT.md`, "what the record does not license"): its longest-wavelength density
does not rearrange in 40 ps and no campaign has read a diffusion constant. This reads the
rigid operator's self-diffusion and density-pattern persistence at four temperatures.*

**Carrier.** 64 waters (`n_cells = 4`) on the rigid operator, the banked stiffness envelope,
imposed liquid density, `--temperature-k T` for `T ∈ {293, 350, 400, 500}` K, seeds 0 and 1,
fine settle 1,000 frames, rigid settle by criterion (floor 1,000), production 250 readouts of
20 fs = 5 ps. Eight arms on E-cores, ~1 h each.

**Reads.** (i) `D(T)` from the oxygen MSD over lags 0.5–2.5 ps by least squares (`6Dt`),
VOID if the MSD is not linear to 30 % over that range; (ii) the density-pattern persistence:
the first-harmonic density modes' static offset over the run against their within-run sd,
`r = |offset| / sd` averaged over the six modes; (iii) the temperature reached on the rigid
modes, within 10 % of the target or the point is refused.

**Stakes.** S1: `D(293) < 0.3 × 10⁻⁹ m²/s` (water's is `2.3`) — the carrier is arrested at
293 K; **kill:** `D(293) ≥ 1.0 × 10⁻⁹`, the glass reading is wrong and the frozen pattern
needs another explanation. S2: `D` rises by more than `5×` between 293 and 500 K and `r`
falls below `0.5` at 500 K — the arrest is thermal, not the operator's. **Kill:** `D` flat
within `2×` across the range, which would say the rigid operator itself cannot diffuse
(then the flexible model must be scanned, ~5× the price). No band on `D(T)` is staked —
it is the model's number.

**Plants.** PT-1: the MSD reader on a synthetic random walk of known `D` returns it to 3 %.
PT-2: the persistence reader on a synthetic frozen pattern plus noise returns `r` to 5 %.

**Branches.** (a) S1 and S2 met: the carrier is a supercooled liquid of this model at 293 K,
a liquid above some `T*` bracketed by the scan; the fluid stakes are re-read as stakes on a
glass at 293 K and a scan at `T*` is the next fluid campaign. (b) S1 killed: not a glass;
the frozen pattern is investigated as a finite-size or operator artefact. (c) S2 killed:
the operator is the cause; the flexible scan is owed. (d) a point refused on temperature:
reported, the others read.
