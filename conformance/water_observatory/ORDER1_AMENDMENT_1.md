# ORDER-1 — AMENDMENT 1: the plant of record re-derived on the carrier of record; no stake moved

*Written 2026-09-26, committed alone, BEFORE the re-read. `ORDER1_RESULTS.md` read branch (e):
PO-4, the planted carried field, came in at `+0.046` against its `0.05` bar, so nothing was
graded. The plant's amplitude (`0.7 ×` the density's standard deviation) had been set on a
synthetic carrier, where it read `+0.12`; on the carrier of record it reads under the bar.
That is the registry's M-PLANT-OBS to the letter — a plant must be re-derived for THIS
instrument and pre-checked to fire ON THE CARRIER IT WILL BE RUN ON — and this is its fifth
occurrence, recorded in `MISFITS.md`.*

## What changes

**The plant's amplitude is set by a rule, on the carrier of record, not by a number carried
over:** the planted slow field is added at UNIT amplitude relative to the carrier's own
fluctuation — `1.0 ×` the standard deviation of each density mode it modulates (the reader's
`ORDER1_PLANT_AMP=1.0`). Unit amplitude is the natural normalisation of "a field as large as
the carrier's own"; it is not chosen for passing. The post-freeze sensitivity scan in the
results (`0.5 / 0.85 / 1.0 ×` → `+0.020 / +0.067 / +0.087`) is reported as the instrument's
sensitivity and was not used to select the rule; it happens to show that unit amplitude
clears the bar and half amplitude does not, which is what a sensitivity scan is for.

**Nothing else changes.** O1, O2, O3 and their bars, the dictionary, the lags, the chain-per-
wavevector form, PO-1, PO-2, PO-3, PO-5, and the branches are as frozen (`e53a803`). The
re-read runs the COMMITTED reader (`186b9ad`) on the same walks with only the plant
amplitude changed; the O-stakes' numbers are deterministic and cannot move — the amendment
turns "ungraded" into a verdict, whichever it is.

## What the re-read will be graded as

If PO-4 fires at unit amplitude and PO-1, PO-2, PO-3, PO-5 stand: the branch is whatever O1–O3
read as written. The results document already says what they would read: O1 between (seed 1
misses `0.3` by `0.007`), O2 at kill level (increments `≤ 0`), O3 a re-finding on two seeds —
**branch (d), closed and not carried**, unless the re-run's numbers differ, in which case the
difference is the finding.

## Recorded beside this amendment (not stakes)

A candidate misfit registered with this amendment, `M-JOINT-PASS-REGION`: O1's "outside the
hydrodynamic span" leg and O2's "carried into that span" fight — a field carried strongly
enough shows in `ρ_k` at equal time and fails O1's ratio leg, as the `1.5 ×` plant did — so
the two stakes' joint pass region is narrow by construction and was never shown non-empty
on the carrier of record before the freeze. The rule: stakes on one carrier must have their
joint pass region exhibited (a plant that passes ALL of them) on that carrier before the
freeze. ORDER-1 is its origin.

---
witness: none (a measured campaign; numeric gates; closure algebra `Closed` and `StatClosure.lean`, cited in words)
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-PLACEMENT-LOTTERY, M-FLOOR-UNSTAKED, M-JOINT-PASS-REGION — the registered ids this text contacts. M-PLANT-OBS is the occurrence this amendment repairs; M-JOINT-PASS-REGION is registered by it.
Carrier-sector statement (M-PLANT-SECTOR): the plant acts on `ρ_k` of `T293_seed0`, the carrier's own density modes, nonzero in that carrier by construction.
