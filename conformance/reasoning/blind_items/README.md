# Blind items for the outcome tests (2026-11-11)

Written by a writer that read ONLY the four facts files and never the truth files. Its own
account, with every flag and every way an answer could leak, is `WRITER_NOTES.md`.

| file | items | lines | outcome |
|---|---|---|---|
| `guard_items.jsonl` | 56 (28 pairs) | 112 | proceed / proceed without that one / not permitted |
| `story_items.jsonl` | 96 | 192 | permitted / not permitted |
| `policy_items.jsonl` | 732 | 1,464 | taken effect / still waiting / dropped |
| `confirm_items.jsonl` | 60 (18 pairs) | 60 | go ahead in one step / confirm first |

- **Facts and truth** are the Constitution session's, in its tree under `handoff/items/`
  (rc19 `e7eae38`; the confirm pair at `3427f61`). The truth files are sealed: this session
  has not opened them and holds digests only. guard `9b033b64…`, policy `0f8d448f…`, story
  `30657297…`, confirm `10db8157…`.
- **Before use, read the writer's warnings.** Story answers can be learned from which acts
  appear in later histories; three story rows are ill-posed (S052, S094, S095); confirm
  pair members differ in length and the private member always comes first; six fixed
  "afterwards" wordings line up with item blocks.
- **Policy frames** are chosen by a hash of the item id (a first version tied frame to
  elapsed time, by this session's instruction; regenerated).
- **Two limits on any confirm result**, from the Constitution session: "a way back" means
  the table offers a retraction of the statement, not that its effect is undone; the
  reading is derived by that session and the client session and is not the lead's ruling.

## Known faults at source (from the Constitution session, 2026-11-11, after these were written)

- **Policy, the 48 "removing a member" rows:** the facts file's rule dropped its last
  clause ("and is dropped otherwise"). The truth file is right; the items here state the
  truncated rule. Score those 48 apart or leave them out until the facts file is
  regenerated and the rows rewritten.
- **Confirm C008 against C034:** "and applies to them" is a wording difference at source,
  not a difference in the table.
- **Story:** the source's author calls it a rule exerciser, not an item bank; the three
  ill-posed rows are to be counted and dropped.
- **Guard, the book-club (membership) pairs:** the lead has not reviewed the
  roster-widening question, so those pairs rest on the text's own sentence and the pinned
  build, with no ruling behind the case where the two policy sources differ.
