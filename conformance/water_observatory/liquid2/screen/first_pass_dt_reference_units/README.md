# The first pass of LIQUID-2's step sweep — KEPT, and MARKED

These four arms are real measurements at real steps and they are kept for that reason. What
was wrong was the UNIT the knob was reported in, and the correction is here rather than in a
rewrite.

`Sim::adopt_table_timescale` derives `Timescale::dt_reference = 4.309924` au from the O-H
curve, and the exactness hold then refines it by a factor of four before any frame runs, so
the step actually in force on this box is `1.077481` au — which is the step LIQUID-1 ran at
and the step everything else in this programme means by "the tables' step". This pass swept
multiples of `dt_reference`, so its `step_multiple` field reads 1, 2, 4, 8 where the steps
in force were:

| this pass's label | dt in force, au | in multiples of the TABLES' step |
|---|---|---|
| `step_x1` | 1.077481 (the hold, not the knob) | 1 |
| `step_x2` | 8.619848 | 8 |
| `step_x4` | 17.239696 | 16 |
| `step_x8` | 34.479392 | 32 |

Its `step_au` field is therefore wrong for `step_x1` only (it prints `dt_reference`, while
`dt_in_force_at_the_end_au` prints the truth, `1.077481`); every other field of every arm is
what it says. The corrected sweep is in the parent directory and speaks in the tables' step.
Read together, the two passes cover 1, 2, 4, 8, 16 and 32 times the tables' step.

Every file here says `dry: true` and none of them is a reading.
