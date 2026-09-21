# TSCAN-SEARCH-1 and HBOND-SEARCH-1: the two searches the inventory said could run today — PREREGISTRATION

*Written 2026-09-21, committed alone before either read. Both use `OBJECT.md` "The procedure, as
arithmetic" on data already on disk; no new dynamics except a 5 ps all-atom walk launched
beside them for the H-bond rung's proper read.*

## TSCAN-SEARCH-1 — do the liquid's closed sectors change with temperature?

**Data:** the eight TSCAN-1 walks (128 waters, rigid, 293/350/400/500 K, two seeds, 5 ps at
20 fs). **Instrument:** `view_search.py` as read on VIEW-SEARCH-1, held out by seed (two
folds), lags 20–400 fs. **Stakes.** S1: at every temperature the top-6 singular functions at
100 fs are `≥ 0.9` in the first-harmonic Fourier subspace — the closed sectors are the
long-wavelength modes at every T, not a property of 293 K. Kill: `< 0.7` at any T. S2: the
Fourier chart's held-out fraction of the bound is `≥ 0.8` at every T; the cell chart's is
below it at every T. Kill: the ordering reverses at any T. S3 (reported, no band): the
transverse shear rate `Γ_s(T)` from the mode autocorrelation and the density mode's first
zero crossing — `η(T)` beside `D(T)`, so Stokes–Einstein can be read across the scan.

## HBOND-SEARCH-1 — the network rung, on the two all-atom walks

**Data:** `molsearch1/` (299 K) and `molsearch1_warm_396K/` (396 K): 128 waters with
hydrogens, 527 rows at 1 fs. Short against a bond lifetime (~1 ps), so lags `≤ 100` fs only;
the 5 ps walk launched with this prereg is the proper carrier and will be read under the
same stakes. **Bond:** donor O–H···O with O–O `< 3.5 Å` and angle H–O···O `< 30°`.
**Dictionary per molecule (7):** bonds donated, bonds accepted, the centre-of-mass velocity
(3), the O–H stretch rate mean and the H–O–H angle rate (the internal sector's two slow
directions as read by MOLSEARCH-1). The molecules are the ensemble, held out in four folds.
**Stakes.** S1 — the bond state is a closed sector: the (donated, accepted) view's own top
`σ` at 100 fs is `≥ 0.5` (kill `< 0.2`: bonds turn over faster than a rung can read at this
cadence). S2 — the bond sector and the velocity sector are decoupled at 5 fs, cross-block
`< 0.2` (kill `≥ 0.4`). S3 — relevance to the fluid: the bond state predicts the molecule's
momentum CHANGE over 50 fs (the force the network exerts) with held-out `R²` at least `3×` a
re-paired null (kill `< 1.5×`: the network carries nothing the momentum law needs, and the
rung is a structural readout as RUNG-1 found, not an object of the fluid). **Plants:** lag
0; the re-paired null; a synthetic ensemble with a planted bond→force coupling read to 20 %.
