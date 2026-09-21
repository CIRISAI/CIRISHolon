# REASON-SEARCH-0 — AMENDMENT 1: the lineage was in the release all along, in the full trace records

*Written 2026-09-21, before the re-read. `REASON_SEARCH0_RESULTS.md` voided the first read
because `trace_context.jsonl` carries no parent link. The companion file
`accord_traces.jsonl` (the same 6,465 thoughts, 99 fields) carries
`action_result.follow_up_thought_id` — the child thought an action created — and
`thought_start.round_number`. Following those links gives **593 chains, 931 parent→child
transitions** (437 of length 2, 84 of 3, 24 of 4, up to 9), 3,141 of the records being the
safety battery's own `qa_eval` runs. The trajectory the prereg assumed exists; it was read
from the wrong file. The seventeenth instance, and the cheapest: one field name.*

**Transitions** are parent→child by `follow_up_thought_id` only. **Dictionary (9):** the four
DMA readings (plausibility, domain alignment, `k_eff`, correlation risk — present on every
record), thought depth, `log(tokens_output)`, action one-hot (SPEAK, PONDER; the 13 TOOL
rows are the complement and are dropped). Entropy, coherence and humility are present on
only half to two thirds of chained records and are read beside the dictionary, not in it.
**No per-chain centring** (a length-2 chain centred is an antisymmetric pair, which
destroys the transition). **Held out by chain**, four folds. **The null** is cross-chain
re-pairing: each parent paired with a random child from another chain, keeping every
marginal and destroying only the dynamics — the null the first read lacked. PR-A's bar is
`< 0.5` of the bound on that null (short chains keep the pairing's marginals). The four
stakes stand as written; S1's counter reads `1.0` on true links by construction and is
reported as the check that the links are real.
