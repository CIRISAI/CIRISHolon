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
