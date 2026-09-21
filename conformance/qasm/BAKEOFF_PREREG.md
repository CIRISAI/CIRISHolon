# The Clifford bake-off, preregistered — holon's flat-sampler path against stim, with the spread the first run lacked

*Written 2026-09-19, committed BEFORE the harness is changed and before any run under it.
The stance carries "ahead of stim 7/7" from three quiet-runner runs of 2026-08-27. Those
runs are honest about what they timed — engine-only, medians of five, sized by n — and they
have the flaw the water observatory found in its own transport gate on 2026-09-13: one
circuit per size (`seed = 1000 + n`), five repetitions of THAT circuit, and a verdict on the
one ratio. Seven single readings. This prereg adds the axis the claim was missing and stakes
the claim again on it.*

## The instrument, unchanged where it was right

Engine-only timing on both sides: holon's `clifford-sample` (gates + one terminal Born sample
on flat planes, seed logged) against `stim.TableauSimulator.do` on the same circuit, circuit
construction excluded from both. Random Clifford circuits of depth `20 n` over
`{x, z, h, s, sdg, cx}` at `n ∈ {64, 128, 256, 512, 1024, 2048, 4096}`, every qubit measured.
Medians of `REPS = 5` per circuit. The CI runner (`bakeoff.yml`, manual dispatch) is the
citable venue; a local run is admitted only with its load average printed in the record.

## What is added

> **Circuit seeds.** `K = 5` independent circuits per size, seeds `1000 + n + 100 k` for
> `k = 0..4`. The per-size reading is the K ratios; the record carries their mean, their
> spread (max − min), and the WORST.

## The stake, and the kill

> **S1 — ahead at every size on the worst circuit:** at each of the seven sizes, the worst
> of the five ratios is `< 1.0`. This is the stance's "7/7", re-staked on the axis it lacked.
> **Kill:** any size at which one circuit's ratio is `≥ 1.0`. Then the stance's line becomes
> "ahead on the mean at m/7, on every circuit at m′/7", with both numbers.
>
> **S2 — the spread is small against the lead:** at each size, `spread < (1 − mean)`. A
> lead smaller than its own scatter is not a lead (`M-SORTS-NOT-SEPARATES` in this suite's
> clothes). **Kill:** any size where the scatter exceeds the lead — reported as "ranks, does
> not separate" at that size.

No new tolerance is invented; `1.0` is the only bar and it is the definition of "ahead".

## What this does not test

Correctness (that is the conformance suite: 650 circuits, error 0.0 against qiskit); the
magic tier (its own head-to-head, `holon-magic-h2h`, and its own open door, Bravyi–Gosset);
anything about stim's absolute speed on other hardware. It tests one sentence in the stance
at the strength that sentence is written.

---
*Audit footer, added 2026-09-21 for `Audit/prereg_audit.py` after CI read red since 2026-09-19; no stake, gate, plant or number above moved.*
witness: none (a measured campaign: its gates are numeric and its closure algebra is `Closed` and `StatClosure.lean`, cited in words above; no gate is a Lean theorem of its own)
**misfits:** M-HOMOG, M-PLANT-OBS — the registered ids this text contacts by keyword, cited at the audit's demand.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier in its own row, and the sector the plant acts on is nonzero in that carrier by construction of the plant.
