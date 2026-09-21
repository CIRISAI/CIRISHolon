# REASON-SEARCH-0: VOID — the pipeline's thoughts are a forest, not a chain, and the prereg's trajectory model was wrong; what the pooled pairs show is reported, not read

*2026-09-21. Prereg `REASON_SEARCH0_PREREG.md` (alone, `92b1aae`), reader `reason_search0.py`,
output `reason_search0_read.txt`. Data: `RATCHET/release/data_scrubbed_v1/trace_context.jsonl`.*

## 0. Verdict: branch (e), on two counts

1. **Plant PR-B does not fire** (`|σ − 1| = 3.6 × 10⁻²` at lag 0 against the `10⁻²` bar) after
   four collinear or task-constant columns were removed on building (the one-hot
   complements, `TOOL`, `llm_calls`): a residual near-degeneracy remains in the ten-feature
   dictionary. By the prereg, nothing is read.
2. **S1 KILLED as staked, and its kill clause is the finding.** The stake said a counter
   (`thought_depth`) is the cheapest closed view and must score `≥ 0.9`; it scored `0.378`.
   The direct check: **depth increments by one on only 952 of 4,548 consecutive-id pairs
   within a task (21 %)**. The pipeline's thoughts in a task are not a chain — they are a
   forest (a queue with siblings: a seed thought and its follow-ups at equal depths), and
   the released record carries no parent link. On the 21 % that ARE parent→child links the
   counter scores `1.000`, as it must. The prereg's trajectory was a model of the data that
   the data refutes; every transition it pooled was 79 % sibling-to-sibling.

## 1. Reported, not read — what the pooled consecutive-id pairs show

At lag 1 (4,548 pairs, held out by task, per-task centred): the DMA sector (plausibility,
alignment, identity fragility, correlation risk) is `3.5×` above its within-task shuffle
and at the shuffle by lag 2 — a one-thought memory; the process → DMA coupling exceeds
DMA → process by `5.4×` (`0.30` against `0.055`); the two sectors are not clean blocks
(`0.28`). The three leading closed directions are the depth counter, the tokens-with-
TASK_COMPLETE direction, and the action pattern; plausibility enters the third. **Read as
a hypothesis for the re-run:** the conscience layer's numeric readings are driven by the
process and do not steer it at one-thought lag — readouts, not state. If that survives on
real chains it is a finding about the H3ERE design, not about reasoning.

The chain-only extra (`CHAIN_ONLY`) is itself broken on its null: filtering pairs by depth
increment after a within-task shuffle changes the pair set, so its shuffled score (`1.78`)
exceeds the real one; its S3 is not a reading either.

## 2. Corrections on building (five, in the reader's comments)

The one-hot complements and `TOOL` (collinear), `llm_calls` (task-constant), per-task
centring (task constants carried 78 % of the first held-out score as Held views), the
shuffle floor for short trajectories (`0.15–0.25` of the bound, not `0.1`), and the ridge's
own size at lag 0.

## 3. What a valid REASON-SEARCH-0 needs, priced

The unscrubbed trace with `parent_thought_id` (CIRISAgent's own store carries the
lineage; the release drops it), so that transitions are parent→child only, and tasks with
`≥ 3` chained thoughts. Then the same four stakes stand as written, with PR-A's bar at the
derived floor for the chain lengths present. Cost: an export and a minute of compute. The
taxonomy search (REASON-SEARCH-1) still needs the encoder pass, as the prereg §3 says.

## 4. The lesson, for the ledger

The sixteenth instance is a trajectory assumed where the data is a tree — the same fault as
staking a chart before searching for it: the prereg named the object (a chain) and the data
had a different one. The direct ordering check that killed S1 cost one line and should
have preceded the prereg.
