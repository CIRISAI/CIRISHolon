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
