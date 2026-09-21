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

## 5. Re-read under Amendment 1 — on the real chains (appended 2026-09-21)

`accord_traces.jsonl` carries the lineage (`action_result.follow_up_thought_id`): 207
chains with every row usable, 444 parent→child transitions, depth incrementing by one on
98.6 % of them (S1 `0.998`, the links are real). Plants: PR-B `3.3 × 10⁻³` (after the
complementary one-hot was dropped), PR-A's re-paired null at `0.16` of the bound.
`reason_search0b.py`, `reason_search0b_read.txt`.

| view | k | held-out | bound | fraction | re-paired null | real / null |
|---|---|---|---|---|---|---|
| depth alone | 1 | 0.998 | 0.83 | 1.21 | 0.010 | 105 |
| DMA sector (plausibility, alignment, k_eff, risk) | 4 | 1.58 | 2.60 | 0.61 | 0.18 | **8.9** |
| process sector (depth, tokens, SPEAK) | 3 | 1.76 | 2.34 | 0.75 | 0.10 | 18 |
| the dictionary | 7 | 3.26 | 3.26 | 1.00 | 0.53 | 6.2 |

Blocks: cross `‖K_PD‖/‖K‖ = 0.35`; `‖K_{P→D}‖ = 0.44`, `‖K_{D→P}‖ = 0.41`, ratio **1.06**.

| stake | verdict |
|---|---|
| S1 the counter is closed | MET (`0.998`) |
| S2 two blocks, cross `< 0.2` | between (`0.35`): coupled, not separable |
| S3 the DMA sector is closed above its null by `≥ 3×` | **MET, `8.9×`** |
| S4 process drives the readings by `≥ 2×` | **undecided, `1.06`** — and the void read's `5×` was an artefact of sibling pairs |

**What it says.** On real parent→child chains the conscience readings are STATE, not
readouts: the four DMA numbers predict their own next values nine times above a null that
keeps every marginal, and they feed the next thought's process exactly as much as the
process feeds them (`0.44` against `0.41`). The void read's "readouts, not state" came from
pooling siblings, and is withdrawn. The two sectors are not separable at one thought
(`0.35`): the H3ERE loop is one coupled object at this cadence, with three closed
directions — a tokens-with-risk mode (`σ = 0.91`), the depth counter (`0.87`), and a
plausibility-with-depth mode (`0.74`). The 11+1 kinds are not in this dictionary, so
nothing here is a reading of the taxonomy; it is the first search on a reasoning
pipeline's own trajectory that survived its plants.

**Branch:** (a) on S1, S3; S2 and S4 between/undecided, reported. The eighteenth instance
for the ledger: the parent link was one field name away in a file already on disk.
