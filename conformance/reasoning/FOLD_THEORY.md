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
