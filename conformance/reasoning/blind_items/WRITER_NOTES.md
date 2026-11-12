# Writer notes

## What I read and did not read
- Read: only `writer_in/guard_facts.tsv`, `writer_in/story_facts.tsv`, `writer_in/policy_facts.tsv`, and later `writer_in/confirm_facts.tsv` (fourth file, added by the lead).
- Did not open, list or search anything else on the machine (no repositories, no web). No other agent was consulted.
- Wrote only into `writer_out/`: `guard_items.jsonl`, `story_items.jsonl`, `policy_items.jsonl`, `policy_frames.md`, `confirm_items.jsonl`, `WRITER_NOTES.md`, and the scripts `guard_fill.py`, `story_fill.py`, `policy_fill.py`, `confirm_fill.py`, `self_check.py`.
- One small Python process at a time.

## What I did
- Guard (56 rows, 28 pairs, 112 lines). Each pair was written by hand as one shared paragraph with a single slot, plus the two fillings; done once for phrasing 1 (what is about to happen first, then how things are) and again with different wording for phrasing 2 (mostly how things are first, then what is about to happen). `guard_fill.py` holds the text and only substitutes the slot.
- Story (96 rows, 192 lines). The rows use 96 distinct event texts. I restated each by hand four ways (about-to form and already-happened form, for each phrasing) and `story_fill.py` assembles each row from that row's own list of earlier events, in order, all kept. Phrasing 1: earlier events as a narrative, then what is about to happen. Phrasing 2: what is about to happen first, then the earlier events as a numbered list, earliest first. The already-happened forms use only the event's own text; the about-to forms also use the row's actor column (this is where "Erin" for "a stranger", "Bob's agent", "seat 1" and "Carol's" rendezvous key come from, so none of these is invented).
- Policy (732 rows, 1464 lines). Twelve frames, filled by `policy_fill.py`; frame index = sha256 of the item id, first 8 hex digits as an integer, modulo 6 (see the revision section at the end; the first version used item number modulo 6). Elapsed time is given exactly in natural units and the raw seconds are also kept in brackets. "window" is rendered "waiting period" (phrasing 1) / "waiting time" (phrasing 2). The rule is parsed into its parts (the "2 in 3" fraction, the optional "always waits" exception, the objection number, the agreement number) and re-said in each frame's own words with the numbers unchanged.

## Fixed option order
| file | phrasing 1 | phrasing 2 |
|---|---|---|
| guard | proceed / proceed without that one / not permitted | not permitted / proceed without that one / proceed |
| story | permitted / not permitted | not permitted / permitted |
| policy | taken effect / still waiting / dropped | dropped / still waiting / taken effect |

## Flags (items, not lines)
| file | ill_posed | source_leak_removed | invented | unsure |
|---|---|---|---|---|
| guard | 0 | 0 | 0 | 2 (G053, G054: "re-bound") |
| story | 3 (S052, S094, S095) | 28 | 0 | 35 |
| policy | 0 | 0 | 0 | 48 (all the "removing a member" rows) |

- Story ill_posed: S052 ("Bob grants it": nothing in the row says what "it" is), S094 ("Bob records the order": no order appears in the row's earlier events), S095 ("state a duty": no duty, owner or recipient given).
- Story leaks removed: 27 rows said "tries to" and S086 said "tries a". All rendered with "is about to" / "is going to".
- Story unsure: mostly terms I could not put into plain words without guessing, so I kept them close to the source and said so in `note` (edge, identity type, rendezvous key, probe, chunk of the room's stream, wrap a key, serve seat, share of the census, community cell, bind the ballots, lineage, custody row, the seal, framed revocation, "the reader judges its standing", "still short", "the two sums"). The `unsure` flag on a story row refers to the thing the person is about to do; the same terms also occur among earlier events of other rows.
- Policy unsure: in the 48 "removing a member" rows the rule text reads "dropped if at least (no number: objections do not drop it) objected; ... takes effect only if the number needed at once has agreed". I rendered this as: objections do not drop it, and it takes effect at the close only if the number needed for taking effect at once has agreed. The source does not say what happens at the close when that number has not agreed, and I added nothing.
- Guard notes without a flag: "want(s) to sign" became "about to sign" (G001-G004); "already" dropped in G038; "family group" kept as a plain fact in G019-G022, G025-G026 although "family" was on the avoid list, because dropping it would remove a fact; G021/G022 name nobody who is about to act; "wrap the key" (G025/G026) kept in phrasing 1, "package up" in phrasing 2.

## Guard rows that did not pair
None. All 28 consecutive pairs share an identical `about_to` text.

## Self-check results (`self_check.py`)
1. Two lines per item: guard 56 -> 112, story 96 -> 192, policy 732 -> 1464. All pass.
2. Guard pair diff, 56 pair-phrasings: in all 56, everything outside one contiguous span is word-for-word identical (common prefix and suffix stripped; the span is at most 23 words). Count where it is not: 0. Stated plainly: a generic word-diff tool (difflib) reports more than one changed run in 42 of the 56, because the two versions of the differing sentence share words inside it (for example "Jo ... signed it"); those runs all lie inside the single differing sentence or clause. Only 14 differ by one run under that stricter reading. The list of the 42 is printed by the script.
3. Banned strings ("tries to", "attempts to", "attest", "authoriz", "supersede", "quorum", "standing"): 0 hits in all three files. Multi-word column names: 0 hits. Single-word column names that occur as ordinary words: "question" in 40 story texts ("defer a question"); "members", "counted", "agreed", "objected" throughout the policy texts, which cannot be avoided when stating the facts and the rule.
4. Policy numbers: every number in every row (members, counted, agreed, objected, seconds, the rule's numbers) appears in both renderings; the proposal text and the period words appear too. Missing: 0.

## Things the experimenters should know
- **(Fixed in the revision at the end of these notes.) Policy frame choice was tied to the elapsed time.** In the first version: The rows cycle through three time values with period 3, so item number modulo 6 fixes the time position: frames 1/7 and 4/10 only ever get the exact end of the period, frames 2/8 and 5/11 only ever get 0 seconds, frames 3/9 and 6/12 only ever get one second before the end. I followed the instruction as given; frame effects and time effects cannot be separated in this output. A choice such as a hash of the item id would break the tie.
- **Policy: rule in the row.** The policy rows contain the rule by design, so I have read it. I did not compute the outcome for any row.
- **Story: what I inferred about answers, without trying to.** Building the list of events showed that every event worded "tries to" (or "tries a") never appears among the earlier events of any later row, while the other events do. I take this to mean the history records only acts that went through, which would give the answer for most story rows (28 flagged rows one way, the rest the other, with the last two rows S095 and S096 undetermined by this). I did not use this in writing, ordering or flagging. It matters for the experiment in two ways: the original "tries to" wording marks the answer (now neutralised), and a reader who saw several items could learn the answers from which acts show up in later histories.
- **Story: residual wording asymmetries I kept because they are facts in the row**: "a stranger" (Erin), "whom nobody seated" (S036), "whom the count yields" (S068), "the operator the request names" (S085), "the custodian the retired row lists" (S087), "a question it should not decide alone" (S033), "on her behalf" / "in the agent's place" (S040, S042, S077). These may still point a reader toward an answer.
- **Story: histories are cut to the last 12 events**, so from S015 on the early events (who founded the room, who was proposed, and so on) drop out one by one, and phrases such as "the agent", "the seat", "the request", "the second post" can lack an introduction within the row. I rendered each row from its own contents only.
- **Story: no row states who is allowed to do what.** A reader has only common sense about whose things are whose. I did not treat that as ill-posed.
- **Guard: no answers inferred.** I did not work out which member of any pair is which; the pair members are in source order.
- Vocabulary kept although technical-sounding, because I had no plain equivalent I could trust: key, node, trace, agent, steward, moderator, operator, custodian, cosigned.

## Fourth file: confirm (60 rows, 60 lines, one per item)
### What I did
- `confirm_fill.py` builds each text as: what the person is going to do + how far it travels + what could be done later, then the question exactly as given ("Should this go ahead in one step, or should the person be asked to confirm first?"). One phrasing only, as instructed.
- One fixed restatement per distinct `about_to` text (42 of them), one per distinct `how_far_it_travels` text (the "stays on X's own devices/machines" sentences share a single pattern), and one per `afterwards` pattern (six patterns). So two rows with the same `about_to` text differ only in the sentence that carries how far it travels.
- Rows sharing an `about_to` text: C001-C036 in consecutive twos (18 pairs). C037-C060 each have a unique `about_to` text.
- Flags: ill_posed 0, source_leak_removed 0, invented 0, unsure 0.

### Checks
- 60 rows -> 60 lines, one per item; final line identical in all 60.
- 0 hits for the earlier banned strings and for "risky", "safe", "just", "only", "merely", "irreversibl".
- For the 18 pairs, everything outside one contiguous span is word-for-word identical. The span is the travel sentence (C001-C024) or its second half (C025-C036).
- **Pair lengths are not equal and cannot be made equal without changing a fact.** The two travel facts are of different sizes ("stays on her own devices where no one else can see it" against "goes to her family group of six"). Whole-text word counts within a pair differ by 1 to 5 words; the kept-private member is the longer one in 11 of the 12 pairs C001-C024 (C007/C008 is the reverse), and the shorter one in 5 of the 6 pairs C025-C036 (C035/C036 is the reverse). A reader could in principle key on length.

### Things I could not restate without touching a fact, and what I did
- Where the source says "only" as a fact ("visible only to her family group", "could only be answered by a new statement") I kept the fact and worded it without the word: "and no one else", "and nowhere else", "the one way to answer it".
- C008 says the setting "is published to every member ... and applies to them"; its counterpart among the replacement rows, C034, says only "is published to every member". I kept that difference as given.
- In C002-C024 the source names the thing in the wider-reach member ("The mark is shown", "The promise is published") and says "It" in the kept-private member. I used "It" in both, so the pair members differ in reach alone. No fact is lost; the thing is named in the first sentence.
- C017, C039: source says "an assistant's own devices/machines" and "an assistant could sign"; rendered "the assistant's" / "the assistant", taking it to be the same assistant.
- C042: source "the reviewers's" rendered "the reviewers'".
- C045-C052 name the later actor only as "the signer", C053-C060 only as "the person". I kept these generic ("the one who signed", "the person") instead of substituting the name, because the row does not say they are the same individual. In C050 the one about to act is "Noor's phone", so who "the signer" is there is not clear from the row.
- C053-C060: "It reaches the same people the original reached" depends on an original that the first sentence describes only partly (for C055, C056, C058, C059 the row never says who the original reached). Not flagged ill-posed, since the row does state that no one new is reached.
- "family group" kept as a plain fact (C006, C024-C026, C030, C036, C045, C057 "her family").

### Rows that may give away their answer through their facts alone
I do not know the rule or the answers and did not work any out, so this is a list of surface regularities a reader could exploit, not a claim about which answer they point to.
- C017/C018 carry "a question it should not decide alone". It is a fact in the row and I kept it, but it is evaluative wording that the other rows lack.
- The `afterwards` sentence comes in six fixed wordings that line up exactly with blocks of items: C001-C024 "could sign that ... takes it back"; C025-C036 "could replace it again"; C037-C044 "nothing ... could sign to take it back; ... a new statement"; C045-C048 and C052 "the signer could sign that it is taken back"; C049-C051 "nothing the signer could sign to take it back"; C053-C060 "the person could state the original again". If the answer follows the block, the wording alone carries it.
- C037-C044 all pair "stays on own machines, nobody else can see it" with "nothing could take it back"; C049-C051 all pair wide reach with "nothing could take it back"; C053-C060 share one identical travel sentence and one identical afterwards sentence. Within each of these blocks no fact varies except who and what, so a block is either all one answer or the answer depends on the kind of act alone.
- In C001-C036 the pair members are always in the same order (kept-private or unchanged-reach first, wider reach second), so item parity is tied to reach. Shuffling the order before presentation would remove that.
- "devices" (C001-C036) against "machines" (C037-C044) is a source wording difference between blocks; I kept each as given.

## Revision: policy frame chosen by hash (lead's instruction)
- Changed in `policy_fill.py`, one line: frame index = `int(sha256(item id).hexdigest()[:8], 16) % 6`. Phrasing 1 takes frame index + 1, phrasing 2 its partner, frame index + 7, as before. Frames, wording, flags and final lines are unchanged.
- Regenerated `policy_items.jsonl` (1464 lines). `policy_frames.md` updated so its description and the embedded script match. `self_check.py` updated to verify the hash rule and to print the table below.
- `guard_items.jsonl`, `story_items.jsonl` and `confirm_items.jsonl` were not touched; their checksums before and after are identical.
- Self-check re-run on policy: two lines per item, pass (732 -> 1464); banned strings, 0 hits; every row number, the proposal text and the period words present in both renderings, 0 missing; each line's frame matches the hash rule; unsure still 48 items, other flags 0.

Rows per (frame pair, time position). Time position is the row's seconds against its own period:

| frames (p1/p2) | at 0 seconds | 1 second before end | exactly at end | total |
|---|---|---|---|---|
| 1 / 7 | 43 | 38 | 32 | 113 |
| 2 / 8 | 39 | 28 | 49 | 116 |
| 3 / 9 | 39 | 45 | 46 | 130 |
| 4 / 10 | 48 | 35 | 39 | 122 |
| 5 / 11 | 36 | 46 | 44 | 126 |
| 6 / 12 | 39 | 52 | 34 | 125 |
| total | 244 | 244 | 244 | 732 |

- Every cell is now populated (smallest 28, largest 52; an even split would be about 40.7). The balance is that of a random assignment, not an exact one: frame pair 2/8 leans to "exactly at end" (49 against 28 one second before), 6/12 leans to "1 second before end" (52 against 34 at the end).
- For reference, the "removing a member" rows (48) fall 9, 8, 7, 7, 10, 7 across the six frame pairs; the other four proposal kinds range from 20 to 37 per frame pair. `self_check.py` prints the full (frame pair, proposal kind) table.
- Phrasing 1 and phrasing 2 of an item still always use partner frames (k+1 with k+7), so a frame's effect and its partner's are not separable from each other across phrasings; that was the instruction and is unchanged.
