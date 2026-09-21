# REASON-SEARCH-0 — the variational search on a reasoning pipeline's own trajectory: PREREGISTRATION

*Written 2026-09-21, committed alone before the read. The plan of REASON-SEARCH-1 (closed
sectors of the 11+1 kinds on conversation arcs) needs per-turn kind probabilities from an
encoder pass that no banked corpus carries; the messages in the released traces are
scrubbed. What IS banked is the H3ERE pipeline's own telemetry per thought —
`RATCHET/release/data_scrubbed_v1/trace_context.jsonl`, 6,465 thoughts in 1,787 tasks, 4,678
within-task transitions — the readings the conscience layer carries (plausibility, domain
alignment, identity fragility and correlation risk) beside the process variables (thought
depth, action, thought type, tokens, calls). This is the search on THAT trajectory: which
sectors of the pipeline's state are closed at the cadence of one thought, which are
decoupled, and whether the sector the next level carries is the closed one. A telemetry
search, labelled as such; the taxonomy search is priced at the end.*

## 1. The instrument

**Trajectory.** Each task's thoughts in id order (ids are monotone; thought depth confirms
the order). Transitions are within a task only.

**Dictionary (16).** DMA sector (4): `csdma_plausibility_score`, `dsdma_domain_alignment`,
`idma_k_eff`, `idma_correlation_risk`. Process sector (12): `thought_depth`,
`log(tokens_output)`, `llm_calls`, action one-hot (SPEAK, PONDER, TASK_COMPLETE, TOOL,
other), thought type one-hot (standard, follow_up, other). Rows with a missing DMA value
are dropped (53). Features standardised; the score is `view_search.py`'s VAMP-2, held out
by task in four folds; lags 1 and 2 thoughts.

**Views scored.** The DMA sector (4); the process sector (12); `thought_depth` alone (1);
the action one-hot alone (5).

## 2. Stakes, each with its kill

- **S1 — a counter is the cheapest closed view, and the search must rank it first.**
  `thought_depth` alone scores `≥ 0.9` at `k = 1` at lag 1: it is Held-like (depth increments)
  and carries nothing the conscience reads — reading B's "closure is cheap" made concrete,
  and the reason the object needs relevance. **Kill:** depth under `0.5` — the trajectories
  are not ordered as assumed and nothing below is read.
- **S2 — the DMA sector and the process sector are two blocks.** Cross-block
  `‖K_DP‖/‖K‖ < 0.2` at lag 1 after removing depth. **Kill:** `≥ 0.4` — the conscience
  readings are driven by the process step by step and are not a sector of their own.
- **S3 — the DMA sector is closed above its null.** Its held-out VAMP-2 at `k = 4` exceeds
  the time-shuffled score by a factor `≥ 3`. **Kill:** under `1.5` — the readings the next
  level carries do not predict themselves from one thought to the next; they are readouts
  of the process, not state.
- **S4 — the direction of coupling.** `‖K_{P→D}‖ > ‖K_{D→P}‖` by a factor `≥ 2` (the process
  moves the readings, the readings do not move the process at this cadence). **Kill:** the
  reverse by `≥ 2`; equality within `2×` is reported as undecided.

**Plants.** PR-A: time-shuffled rows within task — every held-out score under `0.1` of the
bound. PR-B: lag 0 — every `σ = 1` to the ridge. PR-C: a synthetic two-block chain with a
known one-way coupling reads its direction ratio to `20 %`.

## 3. What this is not

Not the taxonomy. The 11+1 search needs the fp32 encoder over a corpus with the messages
present (`H3ERE2_TUNING`'s 362 items are single before/after pairs, not arcs): one encoder
pass over the wild conversation set, ~2 GPU-hours or a day of CPU, is REASON-SEARCH-1's
price, and its stakes (Manner decoupled; the deep seven a slower sector; deep→Facts
one-way; probability view over argmax) are set in the conversation of 2026-09-21 and will
be preregistered when the data exists.
