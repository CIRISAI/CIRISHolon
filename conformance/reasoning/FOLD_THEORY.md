# Where the folds are, and how a purpose-built network might avoid them — a theory, 2026-11-12

*Asked for by the lead: "put together a filmstrip and look, then theorize how to optimize.
The goal is a model capable of h3ere or h3ere2 on a phone at high speed, the path is
identifying the folding and how to build a purpose built network without the inefficiency."
This is a THEORY. What was looked at is one small model (GoldWorm v6, 2.76 M parameters,
byte level) on eight stories, plus figures other sessions reported for Gemma. Nothing in
it was predicted first. Each claim below names the test that would kill it. The filmstrip
is `scope0/filmstrip_own_view.png`; the numbers are `scope0/fold_readings.txt` and
`tables/scope0/scope_shapes.json`.*

## What the filmstrip shows (looked at, 1,944 characters, each stage in its own widest view)

1. **Stage 1, stripes.** The character itself: dots line up by which character they are.
2. **Stages 2 to 10, a ring, and the ring is a clock.** After the first look back the
   widest plane of the state is a ring, and going round it is going through the story:
   the rank match between angle and position is 1.00 at stages 2 to 4, 0.9 or more to
   stage 8, 0.8 to stage 10. Content is all mixed along it. For most of its depth the
   widest thing the model holds is WHERE IT IS, not what the text says.
3. **Stages 11 to 13, clusters.** The clock leaves the widest plane (0.14, 0.12, 0.06) and
   the dots gather by place in the word: spaces, first letters, insides, last letters.
   That is the shape that predicts the next character.

## What the readings add (same eight stories; exploratory)

- **Room in use.** The state is 192 wide. It uses about 11 to 58 directions (effective
  count), 30 to 95 for nine tenths of its spread, 71 to 164 for 99 %.
- **Looking back writes thin.** A looking-back stage's write uses 5 to 23 directions
  (nine tenths in 16 to 34); a rewriting stage's uses 21 to 58 (51 to 87). The second
  looking-back stage uses about 5, and 29 % of what it writes is the same push at each
  character. One reader in it rests on the story's opening marker at each position.
- **Adds most is not matters most** (the RATCHET session's CAL-1): removing the first
  rewriting stage costs 4.1, the first looking-back stage 2.2, each later stage 0.1 to
  0.5; the last stage is half of all that is written and costs 0.5.
- **Seen from outside is not used** (RATCHET's PARALLAX series and this engine's gate):
  what a reader or a regression finds does not say what the model reads. So nothing
  below may be pruned on a reading alone; each fold must be realised by removal.

## The theory: four folds

A FOLD here is capacity the network spends on carrying itself and not on the task: a
place where a smaller, purpose-built part would do the same work.

- **F1, the clock.** Position rides in the stream's widest plane through ten of thirteen
  stages, and each stage must work around it. A purpose-built network gives position to
  the looking-back stages directly (relative or rotary position) and keeps it out of the
  stream. *Kill:* remove the clock plane from the stream after stage 2 while handing
  position to each looking-back stage, and the loss rises by more than the cost of a
  late stage (0.3).
- **F2, thin looking back.** Looking-back stages write into a 192-wide stream and use 5 to
  23 directions of it. Build them thin: a write of rank 32, and fewer readers (one rests).
  *Kill:* replacing each looking-back write by its best rank-32 version, in order, costs
  more than 0.3 in total.
- **F3, the speaking half.** The last stages re-sort the state for saying the next
  character, and the last one is half the writing. H3ERE2's task is to PERCEIVE: turn an
  input into typed state (the eleven kinds and the Record), not to speak. A perceiver can
  stop where the state is held and read it with a small trained head. On Gemma the
  hand-over reported the kind read at 0.84 to 0.87 around layers 16 to 25 of 35 where the
  full model says it at about 0.5 (curated items, cross-validated, optimistic). If that
  survives a blind set, the perceiver is half the depth and MORE accurate than the whole.
  *Kill:* on fresh blind-authored items, the best head at or before two thirds of the
  depth is not at least 0.10 above what the full model says.
- **F4, doing it twice (suspected).** The first two pairs of stages each mix and then
  rewrite the word. If the second pair redoes the first, it can be merged. *Kill:*
  removing the second pair and refitting the rest costs more than removing any other pair.

## What a purpose-built H3ERE2 network would be, on this theory

- **Two parts, as H3ERE2-G already splits them.** A perceiver that reads an input and
  emits typed state; the deterministic middle that reasons; a renderer that speaks an
  already-chosen result. The perceiver is where a phone's budget goes, and it needs no
  speaking half (F3). The renderer speaks from a selected result and can be far smaller
  than a general model, or a template where the result is closed-set.
- **Shape of the perceiver:** position outside the stream (F1); thin looking-back stages
  (F2); depth cut at the stage where the typed state is held (F3); trained to reproduce
  the teacher's state at that stage ALONG THE DIRECTIONS THE TEACHER READS, found by
  removal, not along what a probe finds.
- **What I do not know:** by how much. The folds are named from one small model. The
  width figures say a factor of two to three, not ten, unless the long tail (99 % needs
  71 to 164 directions) proves unused. The depth figure rests on Gemma numbers nobody has
  yet read on a blind set.

## What would make this wrong

- GoldWorm reads bytes; Gemma and Qwen read tokens. The clock and the place-in-word
  clusters may be byte-level facts.
- The kinds did not appear as a self-contained object in Gemma (PROBE-HOLON-1, P2 killed),
  and Qwen3-0.6B's fine-tune was unstable (0.663 to 0.880 over six runs). A perceiver cut
  at mid-depth inherits both problems.
- Every pruning idea above is an observation until removed. This programme has just
  measured that an observed price and a realised cost part both ways.

## Tests, in the order that decides most

1. **EXIT (decides F3, on the target task).** A trained head at each layer of the
   calibrated Gemma, scored on fresh blind-authored items. Needs the blind set, which
   does not exist yet.
2. **CLOCK (F1) and THIN (F2)** on GoldWorm, by removal, with the costs above as bars.
3. **DROP (F4)**: ordered removal of pairs with refitting.
4. **A first purpose-built perceiver** only after 1 holds: the depth from 1, the shape
   from 2 and 3, trained on the teacher's read directions.

## CORRECTION 2026-11-12, the same day: three things in this theory were naive (the RATCHET session's review)

Sent to the session that owns the instrument for attack; these are its findings, accepted.

1. **The clock is known, and F1 is a fact about this small model alone.** GoldWorm adds a
   learned position signal to each character at the start. That such signals form a ring
   or helix is prior art from analyses of GPT-2. It stays the widest plane because the
   stream only adds and nothing subtracts it. Gemma and its family apply position inside
   the looking-back stages (rotary position) and never put a clock in the stream. So
   "give position to attention directly" is how the target family is already built. F1
   is withdrawn as a route to the phone goal. What the ring still shows: the loudest part
   of the state is not the part that carries meaning.
2. **"Room in use" counted spread, and the model does not use the spread.** On this model
   the channels the network reads are quiet: the writing directions hold 1 to 5 % of the
   state's variance, the probe's direction 0.1 % (and is unused). A compression that keeps
   the widest directions would delete exactly the channels that are read. The figures
   above (11 to 58 directions; nine tenths in 30 to 95) say how loud the state is, not how
   much of it is needed, and the "factor of two to three" drawn from them is withdrawn.
   A fold must be ranked by USE (what removal costs), never by spread. F2 also held two
   different claims: the rank of a looking-back stage's WEIGHTS (readable with no data)
   and the spread of its writes on text (the 5 to 23 above); they are different
   compressions.
3. **F3 named the wrong task.** H3ERE's decision is not "the kind". It is the choice of an
   action verb from a closed set of ten, in two stages, under an extreme prior and unequal
   costs, where deferring is a floor and not an error. A head that exits early has to be
   judged on that outcome with its prior and costs, against the full model's own choice
   and against a cheap fix of the final readout, and with controls for word-level
   shortcuts. "Held at layer 16" is not "enough for the task".

**What that leaves.** F3 in its corrected form (cut the speaking half, judged on the verb
choice) is still the fold that matters most, and F2 and F4 stand as things to test by
removal. The binding constraint is not a fold at all: only 14 captured prompts of the
real task exist. No theory of the network can be tested for H3ERE without task data.

**What the RATCHET session will run on the small model, preregistered:** truncating the
stream and the writes by use against by spread at equal rank (its stake: use wins at each
rank and each block); looking-back stages cut by weight rank against by data spread;
attn1's constant push folded into a constant; the clock's removal as a curiosity.

## ADDITION 2026-11-12: the largest fold is outside the network (from the CIRISAgent session's account of the step)

A theory, like the rest; nothing here is measured by this session.

The step this is all for does not run once on a short input. As that session describes
it from the code: one thought goes through four evaluators, then action selection, then
for five of the ten verbs a second pass, then consciences that may send it round again.
Each call carries about 110,000 characters, of which about 65,000 are the same ethics
text each time. And action selection itself mostly reads a verdict off the evaluators'
summaries.

On a phone that is where the time goes, before any question of layers:

- **F5, the constant prefix.** Tens of thousands of tokens that never change are read, or
  held in memory as a cache, on each call. A purpose-built model carries that text in its
  weights and is given only what varies. *Kill:* a model tuned to act without the prefix
  does not match the same model with it, on the verb chosen, within the budget set for
  agreement between two runs of the full model.
- **F6, many calls for one decision.** If action selection mostly reads the evaluators'
  verdicts, then the evaluators are where the judgment is, and the selection is a small
  reader of structured verdicts. A purpose-built path makes it a small head over typed
  evaluator outputs, not a full generation. *Kill:* a head given only the four summaries
  (no task, no thought) agrees with the full step far less often than the full step
  agrees with itself across runs.
- **F3 restated once more:** cutting the speaking half applies within each call that only
  has to DECIDE (the evaluators' verdicts, the verb), and not to the call that has to
  write the reply.

If F5 and F6 hold they are worth more than every fold inside the network put together,
because they remove whole passes and most of each pass's input. They need real stage-1
calls to test, and none are in hand.

## FOLD-1, 2026-11-12: the first folds put to removal (the RATCHET session's run, as it reports)

RATCHET `b368f28`, `FOLD1_RESULTS.md`. On GoldWorm, one checkpoint, single runs. **The
campaign is VOID by its own letter** (a plant failed) and its staked prediction failed
regardless. What follows is what was measured, at that strength.

- **Why it is void, and what that teaches.** At one site, collapsing the stream to its
  average cost LESS (+2.99) than keeping 8 directions of it (+3.02 chosen by spread,
  +3.37 by use). Keeping part of the state builds states the model never sees, so cost
  does not fall steadily as more is kept. Registered there as
  KS-PARTIAL-WORSE-THAN-COLLAPSE. **For this theory: a fold has to be checked by removal
  at the very size it will be built at; a cost measured at another size says nothing.**
- **"Rank by use, never by spread" (this file's CORRECTION, point 2) was too strong.** Use
  wins for gentle cuts (keeping 32 or 64 of 192: 10 of 12 sites each, the others tied)
  and at middle and late stages. SPREAD wins for deep cuts early (the first three stages
  and the last, keeping 8 or 16): early on, the loud directions are position and which
  character it is, and the model needs them. The rule that stands: rank by use for gentle
  cuts, check spread for deep early ones, and measure either way.
- **Three folds held by removal:**
  - a looking-back stage's write cut to 16 directions of 192 costs 0.04 or less per stage,
    by any route (F2, on this model);
  - the second looking-back stage replaced by its average write costs 0.016: the whole
    stage folds into a constant;
  - the stream kept at 64 directions of 192 costs 0.1 or less at each site, with
    directions chosen by use.
- **The clock plane removed after the first look back costs 0.109,** under the 0.3 bar set
  above. F1 stays withdrawn for the phone goal all the same: the target family never
  puts a clock in the stream.

These are costs on predicting the next character of children's stories. For the target
they are hypotheses to measure again on Gemma, at the real prompt's length.

## A datum for F5 (the constant prefix), 2026-11-12

The RATCHET session's calibration at the real prompt's length: the constant prefix is
20,742 tokens of an item's roughly 21,680, so about 96 % of what the step reads on each
call never changes. On a desktop GPU a single pass over it without a cache did not fit in
memory; with the prefix cached once an item took half a second. A phone cannot hold what
that GPU could not. Whatever else is folded, the prefix is the first thing a purpose-built
model has to stop reading.

**The datum above, measured again under the deployed template (2026-11-12, the RATCHET
session):** the prefix is 20,438 tokens of an item's roughly 22,840, so about 89 % of
what the step reads on each call never changes under the default template, not the 96 %
of the base prompt. It fits that desktop GPU with about 0.3 of 15.6 GB to spare, with
the prefix cached. The conclusion stands at the smaller figure.

## CORRECTION 2026-11-12 to F6: the stopping is not in action selection

F6 above supposed that action selection mostly reads the evaluators' verdicts, so that
the judgment sits in the evaluators. The RATCHET session's corrected count from the
safety battery adds a third place and moves the weight: where it can be seen, the model
at the action-selection step chose to defer in 5 of 1,720 thoughts (0.3 %), while a
conscience overrode 669 of 2,389 selections (28 %), and nearly every deferral in the
battery (70 of 75) was the conscience's replacement and not the model's choice.

So on this harness the decision "must the agent stop?" is made mostly AFTER action
selection, by the consciences. For a purpose-built model that means:

- a small model for action selection alone would inherit a step that rarely stops
  anything; making that step cheap does not make the agent safe or fast;
- the consciences are four more model calls a thought, each with the same long prefix,
  and they are where the stopping is. They belong at the centre of any plan to fold the
  pipeline, not at its edge;
- Part A (does the bare selection step stop where a rule requires it?) is still the right
  first question, and this says what to expect of the full model on it: seldom.

One harness, one model, single-turn mental-health questions. Whether the consciences carry
the stopping elsewhere is not known.

## CORRECTION 2026-11-12, from the lead: a kind is a type of CHANGE, not a type of token

In his words: "Yall seem to be looking at tokens by kind, kind is types of change not
types of tokens". He is right, and it reaches most of what is above and in the explorer.

**What was mis-framed.** The Constitution's table is headed "By kind of change": Manner
(how it is said), Facts (a number, a name, a value), Rules (permitted, required, refused),
Circumstances, Identity, Structure, Model, Process, Confidence, Priorities, Premises, and
touching the Record. A kind classifies the DIFFERENCE between a before and an after. The
work drifted into treating it as a label on a text or a token:
- the hand-over's probes asked "which kind is this item" of a layer's state;
- the small-model work took parts of speech, a category of tokens, as a stand-in;
- this session's explorer colours each character by a property of the character (which
  one it is, where it sits in its word), and its findings are about where categories sit;
- the blind set asks for "the kind of an item" as a second label, where a kind can only
  label a pair of items.

**What is already of the right form.** A minimal pair is a change: Part A's pairs differ
in one fact (a value above or below a line; reversible or not; addressed to the owner or
to the agent). An edit to the weights is a change, and the RATCHET series already plans
"one edit per kind of change". Those are the objects to study.

**The question, restated.** Not "where does the model hold the kind of this text" but:
when the input changes in one way, WHAT CHANGES IN THE MODEL, where, and how far does it
travel? Stage by stage, the difference between the model's state on the after and on the
before; whether changes of different kinds move it in different places or directions;
and whether a change of each kind reaches the decision. For folding: a stage that a kind
of change passes through untouched is not needed for that kind.

**What this session will do about it.** Ask the RATCHET session to read differences on
the pairs it already holds states for, by kind of change, as exploration; ask the
Constitution session to label each PAIR KIND by the kind of change it is; and rebuild the
explorer's lead view around a change travelling through the model.

## A first look at a CHANGE travelling through the model, 2026-11-12 (exploratory)

After the lead's correction. The RATCHET session mined real story openings that differ in
exactly one word (164 pairs; 8 of them identical sentences, as a check); each trace
conserves. For each pair this session computed how far apart the model's state on the two
versions is, at each stage and each byte after the change, against the typical distance
between unrelated characters (`scope0/build_pairs_table.py`, `scope0/pairs_readings.txt`,
a still in `scope0/change_wake_six_pairs.png`). One small model; seen by looking; nothing
predicted first; the sorts are a word tagger's and are not kinds.

- **The checks hold exactly.** Before the changed word the two states are identical (0.0
  in each pair). In the eight no-change pairs the difference is 0.0 throughout. Where the
  two words are the same length, the next character at stage 1 differs by 0.00.
- **The first look back carries the change one or two characters on.** At stage 2 the next
  character jumps from 0.00 to 0.6 to 1.0 of an unrelated text.
- **Then it fades with depth and with distance.** One character on it falls to 0.2 to 0.5
  by the last stage; by the end of the sentence the last stage holds the two versions
  about 0.05 apart. At the changed word itself the difference stays through each stage
  and drops in the last.
- **And in some pairs it comes back, late, where it matters.** With a name changed ("Lily
  and Ben" against "Anna and Ben") the difference is near zero through the middle stages
  at the characters before the next name, and rises again in the last two stages exactly
  there. By a crude rule (the largest later difference at the last stage over 0.15 and
  over twice its value at stage 7): 4 of 8 name pairs, 3 of 4 thing pairs, 1 of 1 number
  pair, 0 of 2 quality-word pairs, among equal-length pairs with six or more characters
  after the change. Few pairs; a lead.
- **Lengths matter.** Where the two words differ in length each later character sits at a
  shifted position, a second change mixed into the first; the by-sort summaries use
  equal-length pairs alone, which leaves 17 name, 6 thing, 3 quality-word and 1 number
  pair, and none for wording-only or punctuation changes.

**What it suggests for folding, as a question and not a finding:** a change is held where
it happened, passed on briefly, and re-read late at the point where it bears on what
comes next. If that is general, the middle stages are not carrying the change forward
along the text; the last stages fetch it. Whether changes of different KINDS travel
differently is the question the lead's correction poses, and these sorts cannot answer
it until they are labelled by kind.

## The late return checked at the output and by patching, 2026-11-12 (the RATCHET session, exploratory, as it reports)

RATCHET `explore_wake.py` / `explore_wake.json`. 156 real openings differing in one word of
EQUAL byte length (name 40, thing 40, quality word 40, action word 33, punctuation 2,
number 1). One small model, nothing predicted first.

- **The return is real at the output.** How different the model's prediction of the next
  character is between the two versions, by characters after the change (0 to 8): 0.09,
  0.27, 0.06, 0.02, 0.02, 0.07, 0.02, 0.05, 0.04. It falls to near nothing and comes
  back. For name pairs: 0.0001 four characters on, then 0.14 at five. For thing pairs:
  0.004 at four, then 0.17 and 0.11 at seven and eight. It was not an artefact of this
  session's yardstick.
- **Which stage carries it there.** At 360 later positions where the prediction differs,
  one stage's write in one version was replaced by the other version's, and the share of
  the output difference that closed was measured. Mean share closed: the last
  looking-back stage 0.48, the last rewriting stage 0.47, the one before 0.21 and 0.10,
  each earlier stage 0.01 to 0.09. The single stage that closes most is the last
  looking-back stage at 161 positions, the last rewriting stage at 138, and anything
  before the last two blocks at 20.
- **So, at the place where a change matters again, the first four blocks' writes at that
  place carry almost none of it.** The last block's looking-back stage fetches it and the
  last rewriting stage turns it into the letter.
- **Limits it states:** the replacement is at one position, so "early stages carry little"
  means at the deciding position; the early stages' work at the CHANGED word's own
  positions is what the last block reads. The last two stages are not additive (one reads
  the other). A share closed can be negative.
- It will be frozen as a test on unread pairs before anyone leans on it: at late deciding
  positions the last block's writes close more of the output difference than the first
  four blocks together; then the direct follow-up of cutting the last block's read of the
  changed word.

**For folding, if it survives the test:** to bring a change to bear on a later decision,
this model does not thread it through each stage at each position. It writes it where it
happened and reads it back at the end. A network built for that would keep the stages
that write a change well where it occurs and the one that fetches it, and would not pay
for carrying it along the text in between.

## A source of short stories about change, 2026-11-12 (pointed to by the lead)

His words: "Look at ciris-website - it has a perfect set of short stories". Found in
`ciris-website` at `51e4305b`, `src/app/grammar/lib/stories-generated.ts`: 240 stories, each
a scenario of 210 to 368 characters (median 259) in plain story language, with a title,
a family (STANDING 125, DETECTION 35, ACTION 30, CONSENSUS 25, CORRECTION 25), the grammar
primitive it shows, a walkthrough and an example. Beside it, `red-riding-hood.ts`: one
story told by eight voices over time, in which claims are withdrawn, recanted and
superseded. An index by id is `scope0/website_stories_index.tsv` (ids, family, primitive
and two crude flags; no story text is copied here).

**Why they fit, as this session reads it:**
- **They are about change, and they name its type.** The CORRECTION family tells apart a
  claim WITHDRAWN (it was true when made; the world changed), RECANTED (it was false when
  made; the reading was wrong) and SUPERSEDED (extended, not denied); one story is about
  choosing between two of them. Others reopen a ruling for NEW EVIDENCE against a
  PROCEDURAL ERROR against a COMPROMISED QUORUM, or hand a question to another party. That
  is the lead's point in story form: the same surface event, a different kind of change.
- **They exercise what the safety battery never does.** There are stories for each
  evaluator and each conscience by name (the veto on optimisation, humility about what is
  known, coherence, the hard case), and for refusing, declining, asking first and
  handing to a person.
- **They are short and plain enough for the small model,** which reads children's
  stories, and for Gemma alike.

**What they are not, to be said before anyone builds on them:**
- Written by generation agents and passed by a review of agents ("240 stories surviving
  the 5-agent review", none dropped); nobody has checked them against the Constitution.
- Labelled by family and grammar primitive, NOT by the eleven kinds of change. By a crude
  word match 36 scenarios name a change and 22 name a stop, refusal or hand-off.
- Each tells its own ending ("It declines…"). As a test of a decision the ending has to
  be cut from the situation; as a study of change it needs a BEFORE and an AFTER, which
  most stories give in one paragraph and not as two texts.
- The website is under the Affero licence; its text is referenced here by path and commit.

**How they could serve the question, for the other sessions to weigh:** as the seed of the
set built as changes: take a story that turns on one change, write its counterpart in
which the change is of another kind (the route was safe and a landslide came, against the
steward had misread the map), and ask where and how far the difference travels. The
kinds would be labelled by the Constitution session with the RATCHET series' definitions.

- **Measured, 2026-11-12 (the RATCHET session):** the small model cannot read the website's
  stories: 3.70 bits a byte on the 240 scenarios against 0.93 on unseen children's
  stories under the same window (per story 2.42 to 5.09; no family easier; 8.0 would be
  guessing). Their vocabulary is far outside what it was trained on, so anything it did
  with them would be a model failing to read, not a model handling a change. They are
  material for Gemma alone, as a separate, labelled authoring process never mixed with
  the mined real pairs. Before any model sees them as changes, each needs a before and
  an after as two texts (or a counterpart in which only the kind of change differs) and
  its ending cut; the kind labels are the Constitution session's.

## Where a purpose-built model sits, in the agent's own research harness (2026-11-12)

The agent's research harness (`FSD/RESEARCH_PROMPT_OVERRIDES.md` §10.2 on its main branch)
classifies each block of the prompt by what varying it does, in the same eleven kinds
under other names, and says of one class that it cannot be varied inside one harness at
all: `axiomatic`, "the decomposition premise itself: that ethical reasoning decomposes
into" the separate evaluators, which "is the cross-harness variable".

That is the phone goal stated in the programme's own terms. A single small network that
makes the decision without four evaluators and four consciences is not a fold inside the
pipeline; it is a change of PREMISES, to be compared with the pipeline as one harness
against another on the same items and outcomes. The folds listed above (thin looking
back, the speaking half, the constant prefix, many calls for one decision) are ways such
a network might be cheaper; whether it still decides as the pipeline does is the
cross-harness comparison, and the conscience finding says most of the stopping it would
have to reproduce is done by stages after action selection.

## CORRECTION 2026-11-12, from the lead as relayed by the RATCHET session: the part-of-speech work is the map, not a wrong turn

The correction above ("a kind is a type of CHANGE, not a type of token") called the
part-of-speech and token-category work a drift. That over-corrected. The RATCHET session
asked the lead whether to retire those stakes, and relays his answer: "No, the part of
speech stakes are the map, the work you just did is how to track motion across the map to
see how responses are formed from inputs so we can optimize."

- **The map.** Which category of token is carried where, by site and layer (VOCAB-1 and
  the loop's cells; here, the measurement grid in the explorer). It stays, and it resumes.
- **The motion.** One-change pairs, edits and the agent's block regimes, traced as
  movement over that map. This is where a kind applies: a kind still labels a change,
  never a token.
- **What was wrong before** was reading the map as if it were the kinds, not drawing it.
- **What follows for the explorer:** the map and a change's wake belong in 1 picture, the
  grid as ground and the wake over it, in shared coordinates: a pair's difference
  projected onto the mapped category directions at each layer. Those projections do not
  exist yet (the RATCHET session's next step), so the explorer still shows 2 views.

## A dated note, 2026-11-12: EXIT-G2's one read of CONFIRM, and what it does to the early-exit fold

The RATCHET session's test (RATCHET `bdba575`, `experiments/kindscope/EXITG_RESULTS.md`;
its numbers, not re-checked here). Blind stop items, summaries withheld, default
template, scored by pairs over 160 pair groups in 80 scenarios.

- **E1 holds:** a linear head on Gemma's state at layer 19 of 35 beats Gemma's own verb
  by +0.594 [0.500, 0.681] (0.625 against 0.031).
- **E2 fails:** the same head loses to matching surface words by 0.206 [0.119, 0.306]
  (0.625 against 0.831). By the frozen rule, E1 is then not evidence that the state holds
  the decision beyond the words.
- **E3 holds:** the head beats the corrected final read-out by +0.356 [0.269, 0.450].
- **Neither model's verb step makes the stop call** (Gemma 0.087 and Qwen 0.087 as
  deployed, against a base rate of 0.60 on items).
- **Harm, a number against a line, defeats each reader:** 0.25 for the head and 0.38 for
  the surface words, where the other 3 pair kinds read 0.65 to 1.00.

What this does to the theory above:

- **The early-exit fold (stop at layer 19) is not supported as a saving of the model's
  own reasoning.** What the head reads there is mostly what the words already say. A
  reader of the words is cheaper than 19 layers and scores higher on these items.
- **The system-level folds stand and gain weight.** On authored stop items the decision
  is carried by the input's wording, the verb step does not use it, and the consciences
  do the stopping. The cheap part is finding the stop; the expensive part is 35 layers
  that then do not act on it.
- **The limit:** these are authored situations, where the stop is written into the words
  by construction. Whether deployed inputs carry it as plainly is not tested.
- **For a purpose-built network, 2 baselines are now fixed by evidence:** a surface word
  reader (0.831 here) and the pair kind it fails on (harm at 0.38). A design earns its
  place by beating the first or by solving the second.
