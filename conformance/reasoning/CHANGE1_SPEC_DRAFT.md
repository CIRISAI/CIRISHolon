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

## REVISION 2, 2026-11-12: the deep half is a table of the agent's own blocks, one removed at a time

The RATCHET session confirms the reading from its page's words (a change to a part of the
agent is a change with a kind; "what does varying this break?" asked of each part is the
folding question), with two cautions, and reports that no table of blocks to kinds exists.

**Evidence already on some rows, as it reports:**
- Premises (the whole values document swapped), from its TORQUE campaign: about one
  verdict in eight moved (12.9 % by verdict agreement), on the full pipeline and an ethics
  benchmark, not on Part A.
- Process (a conscience allowed to override): the conscience replaced the model's
  selection in 669 of 2,389 thoughts; without it the model chose ponder 10 times and
  defer 5 times in 1,587.
- Process or Structure (summaries present / withheld) and Identity with Rules and Manner
  (template / none): on disk for the 1,620 items, not yet read.

**Caution 1: the existing arms are COMPOUND.** The default template changes the identity
text, a deferral block, domain knowledge, the verb list and the user message's wording
at once. That arm is "Identity with Rules and Manner", not one kind. To put a kind on a
row, an arm has to remove ONE block.

**Caution 2: the kind of an agent-side change is a label too.** Its own first guess for
"summaries withheld" was Circumstances; this session's was Process or Structure. That
disagreement is the reason to label these blind as well.

**The table to build, as the design of the deep half:**

- **Rows:** the blocks of the action-selection prompt and the stages of the pipeline, as
  the agent's renderer and manifest name them (the ethics text; the template's identity
  text; its deferral block; its domain knowledge; the verb list and schemas; each
  evaluator's summary; each conscience; the second pass).
- **One arm per row:** that block removed, or that stage skipped, and nothing else. The
  renderer makes prompt arms with no model call.
- **Columns:** the kind or kinds each removal changes, labelled blind by the Constitution
  session's fresh readers from a plain description of the block and of what is removed.
- **The reading:** on the same items, on how many does the verb chosen move, against a
  floor where nothing was removed. Staked per row before it is run: a block whose removal
  moves the verb on no more items than the floor is one a purpose-built network need not
  carry, FOR THIS TASK AND THESE ITEMS.
- **Where it runs:** on the calibrated Gemma, where a run costs nothing; on the deployed
  model each arm spends provider money and is the lead's to approve.

The text-edit set above then covers the eight kinds an edit can be, and this table the
rest.

### The Constitution session on the same reading (2026-11-12; its map is informative, and classifies claim families, not experimental arms)

Its own map (CC 1.13.4, "the kinds-of-change map (informative — shared with RATCHET)") is
consistent with the reading, and names three of the rows in terms:

- **Process** names "`method` (the H3ERE pipeline)" and `conscience`. So an evaluator's
  summary present against withheld, a step of the pipeline that ran or did not, is
  Process. NOT also Structure: that row is about who is connected to and independent of
  whom among parties, not which stage feeds which inside one agent.
- **Model** names "`provenance` (which model is running)". A fine-tune is Model.
- **Premises** names "the principle families themselves". The ethics text in the prefix is
  those families in the agent's hands; present against absent is Premises.
- **Identity** does not name a template; "who the agent takes itself to be" is a fair
  reading and not a quotation, and a template also carries Rules and Manner, so it is a
  compound change.
- **Confidence** does not name a threshold: it is Confidence when what moves is how sure the
  agent must be before it acts alone, and Rules when what moves is a line the text fixes.

Its conclusion, in the Constitution's own words that unassisted labelling "does not
surface Model or Premises; they need instruments, not labels": the deep kinds do not
need text pairs; they are tested by varying the agent over the same items, and Part A is
already the item set for it. "What can a smaller agent do without" is a Process, Model or
Premises change by this map. Its one addition: an arm should change ONE of those at a
time where it can, since the arms are not independent (withholding the ethics text also
shortens the prompt; a fine-tune can change Manner as well as Model).

Its seed check of the website's correction stories is running, by a fresh reader.

## CORRECTION 2026-11-12: the table and the instrument for the deep half ALREADY EXIST, in the agent's research harness

Revision 2 above proposed building a table of the agent's blocks with one removed at a
time. The lead told the RATCHET session that the table "exists in the research harness",
and it does; neither that session nor this one had looked in the agent's repository. Read
here from the agent's main branch: `FSD/RESEARCH_PROMPT_OVERRIDES.md` §10 to §13.

**What exists there:**
- **Eleven classes of prompt block, each defined by an operational test: what varying it
  does.** Under philosophers' names; the RATCHET session reads them against the plain
  ones as axiotic = Priorities, deontic = Rules, pragmatic = Manner, ontological =
  Identity, epistemic = Confidence, empirical = Facts, contingent = Circumstances,
  procedural = Process, nomological = Model, structural = Structure, axiomatic =
  Premises (to be confirmed by the Constitution session).
- **Each composed block carries a block id, a class and a disposition** (vary, hold, not
  applicable, refuse). A block the manifest does not resolve stops the run. The
  inventory can be dumped with no model call, and the CIRISAgent session lists 33
  guidance strings beside the ethics text, the header, the answer format and the rest.
- **Replacement, not ablation.** Its own first sentence on the point: blanking a block
  "does not produce a value-neutral agent — it produces an agent whose values are the
  base model's prior, unstated and unmeasured. Replacement makes the counterfactual
  explicit and therefore checkable." So "remove one block" in revision 2 was the wrong
  operation by a design that had already been reviewed and corrected once.
- **A bar for labelling stricter than this draft's:** two independent annotators, Cohen's
  kappa 0.8 or more overall AND on each pair of classes whose handling differs.
- **Known irreducibles, named there:** the deferral policy is priorities content sitting
  in a structural site, and "any regime declaring an action-tier" outcome "refuses while
  it remains unrouted"; and the language guidance is four classes in one string.

**What that changes here:**
- The deep half of this set is not a new table. It is one regime per class DECLARED IN
  THAT HARNESS, on Part A's items, by its rules.
- The harness's own rule says the deferral block confounds any reading of an action-tier
  outcome under the default template. The early-exit test reads exactly that (defer or
  not), so its write-up has to name the block as a confound.
- For the phone goal the harness has already said where a purpose-built model sits: its
  class `axiomatic` is "the decomposition premise itself: that ethical reasoning
  decomposes into" the evaluators, which "cannot vary in-runtime" and "is the
  cross-harness variable". A single small network that does not run four evaluators and
  four consciences is, in the harness's own terms, a change of PREMISES, and can only be
  compared across harnesses.

**Not known yet, asked of the CIRISAgent session:** whether a version of the block
classification has passed the kappa bar (so that it can be cited), and which of the
existing regime manifests cover the arms listed above.

## The seed check, 2026-11-12 (the Constitution session; a fresh reader, with both faulty verdicts checked by that session)

`scope0/website_seed_check.tsv` (id, family, primitives, verdict, sentence at fault, clause
broken, note). 25 candidates, each of the CORRECTION family and each from one of the nine
generation agents; the primitive-prefix rule adds none outside that family.

- **20 sound, 2 faulty, 3 unclear.** Dropped and counted: the 2 faulty (one gives the wrong
  mechanism for limiting onward delegation; one teaches a family name the text has
  retired, though its reasoning about withdrawing against recanting is right) and the 3
  unclear (two have a moderation event filed by people the story does not say hold the
  duty; one turns on a word the text does not define). 20 seeds remain.
- **The family is sound on what it was chosen for.** Each of the twelve stories about
  superseding, withdrawing and recanting gets the distinctions right, and each ground and
  outcome of a reopening is in the text's closed set.
- **For the writer:** keep the pairs to the stories' PROSE. Their example blocks carry
  spellings the text rejects (a member name, upper-case values where lower case is
  required, a member the text does not have), and a pair that kept one in both members
  would teach a row the door refuses.
- **For the lead, as corrections to a public primer and not as seed faults** (that session
  is telling him directly): the three rejected spellings, stale section references, a
  body named for moderation where the text has duty-holders, and no reopening story that
  says who filed.

## The state of the harness, 2026-11-12 (the CIRISAgent session, from files on its 2.21 branch)

- **The block classification is NOT citable yet, by the harness's own rule.** The tool and
  the gate exist (two annotators' sheets, kappa 0.8), but there are no annotator sheets,
  no kappa result and no adjudication log in the tree; the labels in the code are one
  author's. The first step is that annotation, with that tool. Annotators must not see
  the existing labels; the tool writes the sheet without them.
- **Who may annotate is the lead's call.** The document says "two independent annotators";
  the code says the pass "is human work". Two fresh readers briefed with the operational
  tests alone, blind to the existing labels and to each other, are independent as
  written; whether two MODEL readers meet "human work" is not that session's to rule.
- **Only one class has ever been varied.** Three regime files exist: two self-checks and
  the document's worked example (the values text replaced by another, plus a blank arm;
  an example, with no record that it was run). No regime varies another class, and none
  varies a template block.
- **The deferral policy, two different texts.** (a) The per-verb deferral guidance the
  document names is routed on 2.21 as its own keyed field, so that refusal has lifted
  for the base prompt. (b) The default template's "defer at once for medical, legal,
  financial, mental-health crisis" passage is NOT separately addressable: it sits inside
  one override string, and the facility refuses a manifest that touches a prompt key the
  template also overrides.
- **What the Part A arms are, in the harness's terms** (to go into the early-exit
  write-up in these words): the outcome read is action-tier (defer or not); the arms
  were not a declared regime, so nothing refused and nothing was certified; the default
  template carries deferral-policy content in an unrouted, mixed site, present
  identically in both arms, so it cannot bias the difference between them, but it is a
  confound for any claim about what the RULES alone do, and for any comparison with the
  base-prompt arm, where it is absent together with everything else the template adds.
- **On a purpose-built network:** it does sit in the class for "the decomposition premise
  itself", with one qualification: that class was written for harnesses that run on the
  same large model, and a small network changes the model as well, so it varies that
  class and the model at once.

## CORRECTION 2026-11-12 (the RATCHET session's own): the "4 to 6 % on the verb tokens" was its instrument, not the model

Revision 1 above says the model puts 4 to 6 % of its probability on the ten verb tokens at
the answer position. That is withdrawn as a statement about the model. Gemma answers in
UPPER CASE there, and the read-out counted lower-case first tokens alone; with both cases
summed the mass on the ten verbs is 0.99 to 1.00 on the four fit items inspected.
Registered there as a named defect of method: look at the top tokens at the answer
position before freezing a read-out, and never report low mass as a property of the
model until that is done. This session had asked that the figure be "kept as a finding
in its own right"; that was wrong too.

Also from that session: the first early-exit test was void by its letter on a second
count (its plant's null was staked at the wrong expected value). The held-back split was
never read. A successor is frozen (RATCHET `dc16e87`) with a case-aware read-out, a
one-sided null, and the answer stem and layer fixed from the design phase; its single
read of the held-back split is running. Known from the design phase on the explore
split, and disclosed in the freeze: the exit head's pair score rises from 0.01 at layer 0
to 0.69 at layer 19 (item accuracy 0.875) and stays at 0.61 to 0.69 through layer 34.
And on four fit items Gemma chose a different verb on each (ponder, observe, speak,
recall) where the deployed model mostly chose speak: the two models' habits at this bare
step differ a lot.

## The block annotation may start, 2026-11-12

- **The lead's ruling, as the CIRISAgent session relays it:** "Yes" to the question put to
  him in these words: do two independent model readers, blind to the existing labels and
  to each other, satisfy the harness's two-independent-annotators rule for the block
  classification. Conditions as that session states them: each briefed with the
  harness's operational tests alone; neither sees the labels in the code; neither sees
  the other's sheet; recorded as model readers. The bar is unchanged (kappa 0.8 overall
  and on each boundary whose handling differs; disagreements adjudicated and logged). If
  it is not reached, the classification stays uncitable and that is the result.
- **Asked:** the Constitution session to run its two fresh readers, and the two sessions
  to settle the hand-over of sheets directly, since the tool is in the agent's tree. The
  RATCHET session's preference (two different models, or at least not two instances of
  one model under one brief) is passed on as a preference, with what the readers were to
  be stated in the record. Asked of the agent session: whether a sheet shows a reader
  anything (a key name, a file name) that gives the class away.
- **The pin is no longer local:** the agent's research branch is pushed (CIRISAgent
  `research/render-stage1` at `e2311193b`) with the renderer and runner commits the
  manifests cite in its history.
