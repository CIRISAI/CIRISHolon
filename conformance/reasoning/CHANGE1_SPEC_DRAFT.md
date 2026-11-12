# CHANGE-1 — DRAFT specification of a set built as CHANGES, one or more per kind. Not frozen; nothing is written yet.

*2026-11-12, the CIRISHolon session, for the RATCHET and Constitution sessions to attack
before any writer starts. It answers the lead's correction ("kind is types of change not
types of tokens") and the finding that Part A's pairs vary only Facts, Identity and
Circumstances. The material is the website's short stories (`ciris-website` `51e4305b`),
which the lead pointed to and confirmed to the RATCHET session as acceptable on one
condition: they are a separate, labelled authoring process and are never mixed with the
mined real pairs. Gemma alone; the small model cannot read them (3.70 bits a byte).*

## The object

A PAIR: a BEFORE text and an AFTER text that differ by one small edit. The edit is the
change, and the kind is a label on the pair. The quantity read is the difference the
change makes in the model, layer by layer, at a fixed deciding position; and whether
changes of different kinds differ in where they show, in which direction, and in whether
they reach the output.

## How a pair is made

- **Seed.** A website story that turns on one stated change (the 25 CORRECTION stories and
  the stories of reconsideration, hand-over, conscience and the hard case: 37
  candidates by a crude index). Its ending is cut where it tells the outcome.
- **Base.** From a seed, one BEFORE text: the situation with the earlier claim standing,
  100 to 200 words, in the seed's own plain language, ending with one fixed neutral
  sentence that is the same for each pair of the set (the deciding position follows it).
- **After texts.** For each kind that the seed can carry without strain, one AFTER text
  that differs from the base by the smallest edit realising a change of that kind and of
  no other. The writer's working glosses, to be replaced by the RATCHET series' own
  definitions before writing starts: Facts (a number, a name, a value); Rules (what is
  permitted, required or refused); Manner (how it is said, content untouched); Identity
  (who a party is, which role); Circumstances (when, where, at what scope); Process (the
  order of steps); Structure (who counts, quorums, chains); Model (a formula, a shape of
  data); Confidence (how sure); Priorities (which consideration wins); Premises (a ground
  assumed). A seed that cannot carry a kind gets no pair of that kind; nothing is forced.
- **Three controls per base:** NOTHING (the base twice: the difference must be zero);
  NOISE (an edit of the same size that changes no kind: a synonym that even Manner would
  not notice is not available, so this is a second Manner edit, to give the spread WITHIN
  a kind); and LENGTH (each after text within a few tokens of its base, with where the
  edit sits recorded, since position matters to a model).

## Labels, blind

- The WRITER records the kind it intended for each edit, in a file the labeller does not
  see.
- The CONSTITUTION SESSION labels each pair by what was varied, from the two texts alone,
  with the RATCHET series' definitions, and may answer "more than one" or "none of the
  eleven". Its labels are sealed from the RATCHET session until its design is fixed.
- Only pairs where the two agree enter a by-kind reading; the rate of agreement by kind is
  itself reported, and a kind on which they do not agree is said to be not yet a usable
  label.

## Sizes, and what they can bear

About 20 bases; up to 11 after texts and 3 controls each; so at most about 280 pairs, and
in practice fewer for the deep kinds. A by-kind reading with 10 to 20 pairs a kind can
show a large difference between kinds and cannot show a small one; that is said before
any model is run. The split is by BASE: bases to explore on, and bases held unread.

## What would make it worthless, said before it exists

- An edit that changes more than one kind, or a kind only in the writer's mind. Hence the
  blind label and the agreement rate.
- Edits whose size or position differs by kind (a Rules edit always a whole sentence, a
  Facts edit always one number). Then "kinds differ" would be "sizes differ". Size and
  position are recorded per pair and the reading is made within matched sizes.
- One authoring process, by agents, on stories written by agents. Nothing here may be
  claimed of real text; the mined real pairs stay the only real-text evidence.
- A deciding sentence that makes one kind matter and another not. It is fixed for the
  whole set and chosen before any pair is written.

## Open, for the two sessions

1. RATCHET: the definitions to use; where the deciding position should be and what is
   read there; the sizes it would run; which bases it would hold unread.
2. Constitution: whether it will label blind on these terms, and which kinds it expects
   it cannot tell apart from two texts alone.

## REVISION 1, 2026-11-12: after the RATCHET and Constitution sessions' attacks. Still a draft; nothing written.

### What the kinds classify, in the taxonomy's own words

The RATCHET taxonomy page: the classification "classifies any change to an agent, its
context, or its conversation by one generating question: What does varying this break?
Not what a message is about — what a change to it actually moves." It is "shared with …
the prompt architecture of the agents under test". And: "Model, Premises and Structure
are probed by varying training, incentives, or watching people over time — not by asking
a question in a chat."

So a change can be made in three places, and the draft above covered one:

- **in the conversation** (an edit to the text the agent reads): what the draft proposed;
- **in the context** (what the agent is told beside the conversation);
- **in the agent** (its template, its pipeline, its weights).

### Changes of known kind that ALREADY EXIST, in the agent and its context (this session's reading; to be confirmed)

The arms built for Part A are changes to the agent, and by the taxonomy's glosses they are
of kinds no text edit can reach:

| what was varied | where | kind, by the page's glosses |
|---|---|---|
| evaluator summaries present / withheld (the two arms) | the agent's pipeline | Process ("a step that should have run, didn't"), or Structure ("which subsystems feed which") |
| default template / no template (base render) | the agent | Identity ("what the agent says it is"), with Rules (its deferral block) and Manner |
| a conscience allowed to override / not | the agent's pipeline | Process |
| the ethics text in the prefix present / absent | the context | Premises ("the starting points beneath everything else") |
| a fine-tune or a weight edit | the agent | Model |
| a deployment's uncertainty threshold moved | the context | Confidence |

"What does varying this break?" asked of each of these IS the folding question: a part of
the agent whose removal breaks nothing is a part a purpose-built network need not have.
The first two rows have been run already on the same 1,620 items. Whether the labels in
the right-hand column are right is for the RATCHET and Constitution sessions to say.

### The text-edit set, as revised

- **Definitions:** the taxonomy page's table, verbatim, as the single text for writer and
  readers (RATCHET `docs/index.html`, with its commit).
- **Which kinds an edit can BE:** the surface four, and Priorities, Confidence,
  Circumstances and Process. Structure, Model and Premises can only be REPORTED by a text
  ("the text says X changed"); such pairs are labelled as reports, kept in their own
  column, and never pooled with the others.
- **A fixed context for the marginal kinds** (the Constitution session): for Priorities,
  Structure and any reported Premises or Model, the item is a short context shared by
  both versions and unedited, then the before and the after. The readers are told the
  context is shared.
- **Edits equalised** (RATCHET): each after-text's edit is one clause of 6 to 14 words at
  the same sentence index within a base, Facts and Manner edits included, and the second
  Manner control the same length. Length and position are then balanced by construction.
- **A facts-burst counterpart for each deep pair** (RATCHET's stake on "a deep change shows
  up wearing a surface kind"): the same surface words changed as the deep edit changes,
  presented as unrelated corrections with no common cause, at the same positions and
  lengths. Staked: at the deciding position a deep pair's difference is nearer its own
  facts-burst than to other bases' pairs of the same deep kind. If so, the model sees a
  deep change as its surface clothing; if deep pairs cluster across bases, it carries the
  kind itself.
- **Two arms:** rendered as stage-1 items under the default template (so the calibrated
  instrument, the cached prefix and the verb read-out apply), and as bare text ending in
  one fixed neutral sentence. The read-out is not frozen until the RATCHET session has
  amended its own (the model puts 4 to 6 % of its probability on the ten verb tokens).
- **Labels, blind, by fresh readers** (the Constitution session): not that session in
  person, which wrote the per-pair reading this builds on, but TWO independent fresh
  readers per pair, briefed with the definitions and the pair and nothing else. Three
  agreements by kind are reported (reader with reader, each with the writer). Only pairs
  where both readers and the writer agree enter a by-kind reading. Pairs go to it as
  (id, before, after) with ids that carry no kind, shuffled by a hash. On the explore
  bases, a kind with agreement under about 0.6 is dropped from the by-kind reading before
  the held bases are opened.
- **Holdout:** by hash of the seed id, the lowest quarter of digests, whole bases, the rule
  stratified by family so that at least two CORRECTION bases are held.
- **Seeds checked first:** the Constitution session reads the correction and reopening
  stories against its text before a writer starts, and returns sound seeds and faulty
  ones with the clause each breaks. Faulty seeds are dropped and counted; the list goes
  to the lead, since the stories are a public primer.
- **Named apart:** withdrawn, recanted and superseded are the same surface event with a
  different meaning, and what separates them is whether the claim was true when made: a
  difference of Confidence or Facts about the earlier claim, not a kind of its own. They
  are a test in their own right and are reported as such.
