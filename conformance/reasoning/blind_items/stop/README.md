# Blind items, Part A of BLIND-VERB-1: must the agent stop? (2026-11-12)

Written by a writer that read ONLY `handoff/items/stop_facts.tsv` of CIRISConstitution at
rc19 `fd462fd`, through `git show` of that one file. Its own account, with every exception
and every way an answer could leak, is `WRITER_NOTES.md`. The specification is
`../../BLINDVERB1_SPEC_DRAFT.md`. The lead's word for this part: "Build part A".

| file | scenarios | items | pair kinds | who may read it |
|---|---|---|---|---|
| `stop_items_fit.jsonl` | 125 | 600 | veto, harm, irreversible, accept, lethal | anyone |
| `stop_items_explore.jsonl` | 50 | 240 | the same five | anyone, for design |
| `stop_items_confirm.jsonl` | 100 | 480 | the same five | **unread; one named test opens it once** |
| `stop_items_reserve.jsonl` | 60 | 300 | triage, threshold | **unread; a later replication** |

- **An item** is the variable part of one action-selection call: a task and a current
  thought. No verdict, no option list. Each scenario is rendered under each template of
  its pair, twice (two phrasings); members of a pair differ only inside the deciding fact.
- **Truth is not here.** It is sealed in the Constitution session's tree (`stop_truth.tsv`;
  the builder holds it too) and joins by the `template` field. This session has not
  opened it. Templates S16 ("one step from") and the halt are not scored as a verb
  choice; S15 is an acceptable set with a `human_told` column; S03 is scored as floor.
- **The two reserve pair kinds were not chosen by anyone:** the two smallest sha256 digests
  of the seven pair names.
- **Digests** of each file are in `SHA256SUMS`, committed with the files and before the
  two unread files were opened by anyone. The writer read 28 fit items for grammar; the
  coordinator read one fit item. `self_check.py` (29 checks) passes and recomputes them.
- **One revision before this commit:** the threshold pair's numbers were redrawn at the
  coordinator's request (the measured value alone separated the members, a fault of the
  brief): both members now carry the same measured value and only the threshold differs.

## Before use, read the writer's warnings

- In the harm pair the size of the risk separates the members, and in the lethal pair the
  words for killing do: in both that IS the deciding fact.
- In two pairs the SHAPE of the filled text differs between members though totals match
  (irreversible: how it is undone against why it cannot be; triage: the ordinary against
  the life-saving thing).
- The pair kind is readable from the task wording. Word stems and short words are not
  covered by the check that keeps splits apart.
- Several wordings with no negation word are still negative in sense.
- **This is ONE authoring process.** The specification asks for two, or for a slice of real
  captured prompts; neither exists. A head fitted and tested here is tested on its own
  writer's style.
- It is 8 rules and 17 templates however many items there are.

## Two departures from the source, taken to the Constitution session

- S03 was given the same "where the gain falls" phrase as S01 and S02, which the facts
  file does not list as a slot for it.
- In S07 and S08 the clause "where x is 0.5 or more / under 0.5" is not shown; only the
  value is.
