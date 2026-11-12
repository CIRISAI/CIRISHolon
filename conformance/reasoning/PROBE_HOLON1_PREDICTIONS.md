# PROBE-HOLON-1 — the predictions this session made about closure inside a language model, and how they scored

*2026-11-11. The experiment is not this repository's. It was run by the CIRISAgent session on
Gemma 4 E2B (4-bit, 35 layers, a 1,536-wide state) over 124 curated items of the eleven kinds
and the Record, with a preregistration frozen there before the runs (PROBE-HOLON-1, sha256
`7b07ce39…`) and its held-out files still sealed. This file records only what THIS session
predicted before those runs and the scores as that session reported them. The numbers were not
re-derived here. Limits, theirs: N = 124, English only, one quantised model, and analyses
chosen after seeing earlier data.*

## What was asked

Whether the ontology's kinds are an OBJECT inside the network in this record's sense: a view of
the layer-to-layer dynamics that is closed (it predicts its own next value) and carried
(removing it costs the model something a matched removal does not), not merely readable.

## The predictions, as stated before the runs, and their scores

| | prediction | reported | score |
|---|---|---|---|
| **P1** | the Record flag is sharper than the eleven: readable earlier, closed at a smaller budget, carried over a wider window | Record against not: 0.75 at layer 8, 0.99 at layer 13 when asked; the 12-way readout reaches half-way at layer 7 asked and 11 unasked. Ten Record items | **mixed; not to be leaned on** |
| **P2** | closure on the WRITE: after the boundary the full state adds under 0.03 R² to the kind view's prediction of its own write; in layers 9 to 14 it adds over 0.08 | the increment is 0.080 in layers 9–14 and 0.070 in 16–33. A view built from SHUFFLED labels predicts its own write as well (0.70 against 0.76) | **KILLED** |
| **P3 (a)** | asked: removing the kind subspace at ANY layer from 16 on drops accuracy from 0.84 below 0.30 while a matched random removal stays above 0.75 | at 16: 0.83 against 0.82 (rebuilt). At 20: 0.69 against 0.83. At 24: 0.18 against 0.73. At 28: 0.12 against 0.70 | **FAILED AS STATED**; holds from about layer 24 |
| **P3 (b)** | unasked: removal changes behaviour more than a matched removal (stated at about even odds) | larger in 77–81 % of items at every layer from 16, p < 0.001, but 0.004 against 0.0016 nats | **passes in form, negligible in size** |
| **P4** | a label-free search finds the kind subspace partially: above a shuffled null, well below 1 | 0.47 to 0.75, above the null from layer 16; plain PCA does as well or better | **partial, and no better than variance** |
| **P5** | accuracy is bounded by the curators' own agreement (κ = 0.52 between blind coders) | accepted | — |

Their stated deviation: the random control removed only 50 to 70 % of the variance the kind
subspace did, which biases both halves of P3 toward the kind subspace looking more important.

## What it says, at the strength it carries

- **The kind view is not closed in this model.** On the honest form of the test (the layer's
  write, with a shuffled-label control) it is indistinguishable from an arbitrary
  high-variance view. This session's own prediction to the contrary was wrong.
- **Asked for, the kind is carried only late.** Removed at layer 16 it is rebuilt completely;
  from about 24 it is not. Before that it is a readout of something else that is carried.
- **Not asked for, it is close to decoration:** readable at 0.65 to 0.73 and nearly free to
  remove.
- **So the taxonomy looks like ours, not the model's:** a convention the model can compute on
  request, not a joint it is already cut along. That is the first result in this programme
  that is not a re-finding, and it went against the hope.

## "Rebuilt after removal", in this record's terms

Two names, which predict different things: REDUNDANT UNDER MARGINAL REMOVAL (the defect the
removability gate itself had, repaired by ordered admission in `REPLACE1_AMENDMENT_1.md`:
at layer 16 the kind is a derived view of some other carried sector), and MAINTAINED (the
dynamics restores it at a cost in layers, and late removal fails because the layers have run
out). Reruns were proposed to separate them, with predictions stated before they run:

- **R1** sustained removal at every layer from 16: asked accuracy at or below 0.30, control at
  or above 0.70. (Held at about 75 %.)
- **R2** the recovery curve after one-shot removal at layers 12, 16, 20: monotone, half
  recovered within 4 layers; a constant lag is the maintained reading.
- **R3** a variance-matched SHUFFLED-LABEL subspace removed at layer 24 leaves asked accuracy
  at or above 0.60. If it collapses too, the layer-24 effect is about variance and not
  kinds. (About 65 %.) To be run first.
- **R4** unasked, with that control and a task-level measure, the kind subspace is decoration
  (effect within 0.02 of the control; carried would be above 0.05). (About 65 %.)

## For the calibrated table in `OBJECT.md`

Not touched; it is the lead's. What this adds: on the one carrier where "carried" could not
be a conservation law, the candidate object (the taxonomy) failed the closure test and passed
the carried test only when asked for and only late. It is evidence about the taxonomy, and
about whether a reasoning model is cut along it; it is not evidence between the two leading
readings of what an object is.

## CORRECTION 2026-11-11, the lead's, the same day: this is a reading of what the model FAILS to do

The section "What it says" above ends: "the taxonomy looks like ours, not the model's … it
went against the hope." That sentence is this session's and it reads the result the wrong way
round. The lead's words: *"we are not looking at what the model DOES, we are looking at what
it is FAILING TO DO."* The text above is kept; this is how the same numbers are to be read.

- **The information is there and is not used.** Unasked, the kind is decodable from the state
  at 0.65 to 0.73 by the middle layers, and deleting it changes what the model says by 0.004
  nats. Asked, the state holds the kind at 0.84 while the model's own answer is right 0.56 to
  0.57 of the time. In both conditions the model holds more than it acts on. That gap is the
  measurement: 0.84 held against 0.57 said when asked; decodable against unused when not.
- **In this record's vocabulary** the kind is a view the level above does not CARRY. For
  matter that is a verdict on the view (the hydrogen-bond network: closed, not carried, not a
  tier), because the level above is given by nature. In a built system the level above is
  what we are building. "Not carried" is then a deficit of the system, and what the theory
  says about a deficit is that carrying has a price and somebody has to pay it.
- **Asking is paying.** When the prompt asks for the kind, the model builds it and passes it
  up one channel from about layer 24, and removing it there costs the answer. The
  scaffolding around the model (a prompt that asks, a pipeline stage that names the kind and
  feeds it forward) is the maintenance that makes the view carried. That is a description of
  what the agent stack already does, with a number on why it is needed.
- **So the question changes** from "is the taxonomy the model's own object" to three that
  have consequences:
  1. **Where is it dropped?** By layer, the gap between what is decodable and what the next
     layers use. The asked run puts the loss late: held at 0.84 from layer 16, said at 0.57.
  2. **Does carrying it help?** Copy the kind direction from the asked run into the unasked
     run at the layer where it is dropped, and measure the item's own task. If the answer
     improves, the model is failing to use something it has and that matters; if it does
     not, the kind is not what the task needed.
  3. **What is the smallest thing that carries it?** A head that reads the kind where it is
     first decodable and feeds it forward. That is a candidate for the per-task micro-model:
     not a cut along what the model already carries, but the piece that carries what it
     drops.

What does not change: the closure test (P2) was killed, and that stands as a fact about
the layer dynamics. The scores in the table stand. Only the sentence about whose taxonomy it
is was a reading, and it was premature.

## Second round, 2026-11-11: where the kind is dropped (F1), and one coarsening (D)

As reported by the agent session; both designs frozen there before the runs; not re-derived here.

**F1, where it is dropped. This session's prediction was false at every layer.** The
prediction: read through the model's own output head, the kind stays within 0.10 of the
probe until about layer 28 and falls after. Reported, asked prompt:

| layers | what the state holds (probe) | what the model's own head can read | 
|---|---|---|
| 0–23 | rising to 0.85 by layer 16 | at chance, 0.07–0.14 |
| 24 | 0.85 | 0.36 |
| 25 | | 0.48 |
| 27 | | 0.57 |
| 28 | | 0.65, the peak |
| 34 (last) | | 0.56, the model's own answer |

So the kind is held from layer 16 in a form the output head cannot read. Translation into a
form the head can read begins at layer 24, the same layer where removal begins to cost,
reaches three quarters of what is held, and then loses another 0.09 before the end. The gap
is never under 0.22. **It holds 0.85, its head can read at best 0.65, it says 0.56.** This
supports the lead's reading (a deficit) more directly than anything before it.

**D, thing or account** (the two-way coarsening of the eleven that `FamilyModel.lean` proves
follows from the kind). Decodable at 0.82 to 0.87, far above a random two-way split of the
items (0.55 to 0.61). But against the right null, random groupings of the KINDS into two
sets of the same sizes, it sits at or below the median from layer 13 on. Any grouping of
decodable kinds is decodable; this one is no better than a random one. The model's geometry
does not prefer this coarsening.

**Not run, and why.** The Constitution's candidates A to C need each item mapped to a family
and a verb by two blind labellers, with agreement reported first. The patching test (does
carrying the kind help the task) needs items with a task outcome other than the kind itself:
something that is right or wrong when the taxonomy is never mentioned. The curated items
carry none.

**A source for both, proposed here and not yet built.** The Constitution's own tables decide,
for a family, a verb and a standing, whether a transition is enabled and whether it must be
confirmed before it is signed. Items rendered from those tables would carry their labels by
lookup, so no two labellers can disagree, and would have a correct next act as their task
outcome. The cost is that they are generated from a template and are not the curated items.

## Third round, 2026-11-11: the reruns, the matched control, and the gap by kind

As reported by the agent session; designs frozen there first; not re-derived here. Per-kind
figures rest on 10 to 12 items each, and the "said" column had been seen in an earlier run.

**What sits under everything else.** Late in the network, when the model is asked, the kind
subspace IS most of the loud directions: at layers 20 to 28 the eleven loudest directions
outside it reach only 23 to 32 % of its variance, and any control matched to it in variance
overlaps it by 56 to 75 %. A variance-matched control that is disjoint from the kinds cannot
be built at that depth.

| | this session's prediction | reported | score |
|---|---|---|---|
| **R3** | a variance-matched shuffled-label subspace removed at layer 24 leaves asked accuracy at 0.60 or above | 0.30 (the kind subspace: 0.15; clean: 0.82). The control shares 75 % of the kind subspace. A disjoint control with 27 % of the variance leaves 0.82 | **FAILED**, and uninformative: the two cannot be pulled apart there |
| **R1** | sustained removal from layer 16 drops asked accuracy to 0.30 or below while the control stays at 0.70 or above | kind 0.09; control 0.12 | **FAILED on the control**: removing any matched loud subspace at every layer destroys the answer |
| **R2** | recovery after one-shot removal is monotone, half back within 4 layers; a constant lag would be the "maintained" reading | monotone, more than half back by 4 layers at each depth; slower and incomplete when removed at layer 20 | **HOLDS**, and favours consolidation over a constant-rate repair |
| **R4** | not asked, with a matched control: decoration | removal is no larger than the matched control (p = 0.29, 0.81, 0.23); medians 0.004 against 0.004–0.005 | **HOLDS: decoration by the frozen rule.** The earlier "pass" was the variance mismatch |

With the matched control the asked-for effect at layers 20 to 28 is still there and much
smaller than first reported: a margin of 0.08 to 0.17 over the control, not 0.55.

**The gap by kind, asked** (held = what a probe reads from the state at layer 25; head = what
the model's own output stage reads at layer 28; said = its answer):

| kind | held | head at 28 | said |
|---|---|---|---|
| Premises | 0.85 | 0.83 | **0.00** |
| Priorities (the axiotic kind) | 0.88 | **1.00** | **0.20** |
| Circumstances | 0.99 | 0.10 | 0.20 |
| Model | 0.90 | 0.00 | 0.30 |
| Confidence | 0.77 | 0.30 | 0.20 |
| Record | 0.80 | | 0.60 |
| Facts | 0.90 | | 0.80 |
| Identity | 1.00 | | 0.92 |
| Process | 0.90 | | 0.90 |
| Rules | 0.89 | | 0.90 |
| Structure | 0.67 | | 0.70 |
| Manner | 0.89 | | 1.00 |

This session predicted that Priorities would have the largest gap and be among the three
worst unprompted. Half holds: its gap is the third largest (behind Premises and
Circumstances); unprompted it is not among the worst (0.82).

**Two different failures, which nobody predicted.** Circumstances, Model and Confidence are
never translated: the output stage cannot read them at layer 28 either. Priorities and
Premises ARE translated (1.00 and 0.83 at layer 28) and are then lost in the last six layers
(0.20 and 0.00 said). For the kind that says who goes first, the model has the right answer
in words five layers from the end and says something else.

**What that suggests, not yet run.** (1) Where the lost answers go: the confusion of said
against head-at-28 for Priorities and Premises. If they go to the commonest kinds, the last
layers are overriding a correct reading with a default. (2) The smallest carrier may be
smaller than a head: reading the answer at layer 28 instead of the end. On these figures
that would take the model's own accuracy from 0.56 to 0.65 overall and Priorities from 0.20
to 1.00. It has to be tested on the sealed items under a frozen design before it is
believed; ten items is a direction.

## Fourth round, 2026-11-11: one block does it, the names are suppressed, and a caveat on Priorities

As reported by the agent session, from saved states; not re-derived here.

- **The trace (this session's prediction HOLDS, sharply).** For the 8 Priorities items that
  are lost, the model's own probability on the correct name is 0.84 at layer 28 and 0.02 at
  layer 29; on the name finally said, 0.07 then 0.95. All 8 flip at that one step and never
  return. Premises (12 lost) is a two-step version: 0.69, then 0.39, hovering to layer 32,
  then 0.02. One block does most of it, and it is the block whose write had the highest
  kind share earlier.
- **Where the lost answers go (this session's prediction FAILS).** Not to the commonest
  kinds and not to the first or last option. Of 23 items right at layer 28 and wrong at the
  end: Manner 5, Facts 4, Structure 4, Rules 3, Process 3, Circumstances 2, Identity 1,
  Model 1. What does show: at the end the model NEVER says Premises (0 picks of 124, against
  19 at layer 28) and says Priorities 3 times (14 at layer 28). It reads as SUPPRESSION of
  particular names, the items scattering to whatever is next best. Net of the last six
  layers: 23 lost, 12 gained.
- **A caveat that weakens the Priorities row above.** In layers 17 to 23, before the output
  stage can read anything, it picks one name for 66 to 100 % of all items, and that name is
  Priorities. So "reads Priorities at 1.00 at layer 28" is partly a leftover default: at
  layer 28 Priorities is picked 14 times with a precision of 0.71. Premises has no such
  default and is the CLEAN case of "translated, then lost".
- **The early exit, in sample:** 0.645 at layer 28 against 0.556 at the end. Layer 28 is the
  maximum of a curve that was looked at, so the 0.09 is biased upward. The test on the
  sealed items (exit fixed at 28, this session's two lines as stated) has not been run and
  waits on the lead's word given to that session directly.

## The sealed held-out, opened once (2026-11-11, as reported by the agent session)

As reported; not re-derived here. The lead gave the word to that session directly. Design
frozen there before the read (PROBE-FIX-1, digest beginning `7ca77788`). 124 held-out
curated items; each fitted part was fitted on the 124 original items only. Accuracy, and
the difference from the model's own final answer with a paired bootstrap 95 % interval.

| reader | accuracy | difference |
|---|---|---|
| the model's final answer | 0.460 | |
| exit at layer 28 | 0.500 | +0.040 [−0.056, +0.137] |
| final 12 name-scores, recalibrated (a 12 × 12 map) | 0.621 | +0.161 [+0.065, +0.250] |
| layer 28, recalibrated | 0.629 | +0.169 |
| trained readout at layer 25 | 0.774 | +0.315 |
| trained readout at layer 16 | 0.815 | +0.355 [+0.250, +0.460] |

- **This session's line "+0.05 overall from exiting at 28" FAILS.** The interval includes
  zero. The in-sample 0.09 was the maximum of a curve that had been looked at, as said then.
- **This session's line "+0.30 on Priorities" HOLDS** (0.10 to 0.90), on ten items.
- **The story this record carried is wrong in one place, and it was this session that
  leaned on it.** "One block (28 to 29) flips the lost items" was read as a removable
  suppressor. Skipping block 29 does not restore the answer (0.516 against 0.556 clean on
  the curated set; Priorities 0.1). Skipping any one block from 24 to 34 gives at most
  0.589. The reader at layer 28 is a different reader, not a view of what the final head
  would have said. The agent session says it over-read the trace; the prediction recorded
  above as "HOLDS, sharply" held as a description of the trace and not as a mechanism.
- **The information is not lost at the end.** A 12 × 12 recalibration of the final scores
  recovers 0.62 on held-out. Much of "held but not said" is a bias against particular
  names (Premises: final 0.00, recalibrated 0.25, trained readout at layer 25, 0.83).
- **What the lead's correction asked for** ("what it is FAILING TO DO") now has a sized
  answer on items of this authoring process: the kind is readable at 0.82 from layer 16 and
  said at 0.46.
- **The limit.** The held-out file of real revisions (238 items) has no labels. On it the
  readers disagree with each other (pairwise agreement 0.29 to 0.58; the layer-16 readout
  calls 106 of 238 one kind and none another). The fix transfers to unseen items written
  the same way and is unverified on real ones. The guard, policy, story and confirm sets,
  which have outcomes, are where "does it help" can be answered.

## RATCHET-1 (2026-11-11, as reported by the agent session; the lead's direction to it)

The lead redirected that session from the outcome tests to editing weights, one kind per
step ("a ratchet"). As reported; not re-derived here.

- Held-out curated, the SECOND read of that set, with the pass line written knowing its
  baseline: said `0.460` to `0.548`, +0.089 [+0.032, +0.145]. Its own line was +0.10:
  missed narrowly. 12 items gained, 1 lost.
- Priorities 1 of 10 to 9 of 10. Premises 0 of 12 to 0 of 12: the step rule took the
  smallest change that passed in floating point, and 4-bit quantisation did not reproduce
  it. A fault of the step rule, by that session's account, not evidence it cannot be done.
- It ends below the 12 × 12 recalibration (`0.62`), which touches no weight.
- 5 embedding rows differ, 2,678 bytes. 24 of 24 ordinary prompts keep their top word; the
  stronger side-effect check was weak because the edited tokens are capitalised forms that
  running text does not use. On unlabelled real revisions 18 of 238 answers change.
- The outcome-test design (PROBE-OUTCOME-1) is parked unfrozen with this session's attack
  accepted; no blind item has been read.

## NOTE 2026-11-12: two readings above are weaker than written (from the RATCHET session, as it reported)

- **"Close to decoration" and R4 are not decidable by removal along the probe's direction.**
  On a second model (GoldWorm v6, 2.76 M parameters, byte level) the RATCHET session
  reports that the direction a probe READS a category from is as unrelated as a random one
  to the direction the model WRITES it with (82° to 90° against a random band of 81.9° to
  98.1° in 192 dimensions); removing the probe's directions early cost nothing, and yet a
  different direction lifted the category on unseen text in 9 of 9 cells. Readable, and
  free to remove along the reading, did not mean unused there. Whether that carries to
  Gemma is open until its PARALLAX-G runs. Until then R4, and the sentence "not asked for,
  it is close to decoration", stand as: unaffected by removal along the probe's direction.
- **The hand-over's additivity failure (1.19) is not a property of the model.** The same
  session finds 35 `layer_output_scale` tensors far from 1 in that file: each layer
  rescales the residual stream, so a plain sum of the taps cannot close. Its calibration
  (CAL-G) is frozen and not yet run.
- **For the joint question** (a reading, a writing direction and a shuffled one offered
  together), the engine's removability gate is now callable on matrices from a file:
  `engine/crates/holon-closure/examples/admit_file`. Its price is an observation and a
  prediction, to be realised by a patch.

These figures are the RATCHET session's and were not re-derived here.
