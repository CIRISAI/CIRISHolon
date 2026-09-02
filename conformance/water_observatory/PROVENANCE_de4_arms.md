# PROVENANCE — the dE₄ on/off control arms

*Written 2026-09-01 because `workbench-engine` read the control's run log to caption a
page and found that it records no commit, no binary hash and no gate state — and "same
commit" is the entire content of that control. They were right to stop. Without this
file a reader cannot distinguish my control from a third, differently-built arm, which
would make it exactly the kind of artifact this lane has spent the day refusing.*

## The build both arms ran

| | |
|---|---|
| commit | `21e6be3ed8eb1c9944b18677a022ad89f3639826` |
| binary | `engine/crates/holon-render/examples/waterquench_traj`, release |
| binary sha256 | `462045fefa856ad55756f9baf6352d88a8c2f658bd393795774e3528ada3a349` |
| binary mtime | 2026-09-01 18:04:11 |
| build exit | 0 |
| worktree | detached at `21e6be3`, own `CARGO_TARGET_DIR`, not the shared tree |

**How the hash is known to be the one that ran, stated rather than assumed.** The binary
was built before both arms launched, has not been rebuilt since (mtime unchanged; the only
later builds were of *other* examples, `parse_check` and `block_probe`, which do not touch
it), and it is still on disk — the `--de4=on` arm is executing it as this is written. The
`--de4=off` arm was launched from the same path in the same minute.

**The honest limit.** These facts are recorded in a sidecar, not in the run logs' headers,
because the runs had already started when the gap was pointed out and back-writing a header
into a completed run's log would be manufacturing provenance rather than recording it. The
runner should emit this header itself; that is now owed and is not done.

## Gate state at this commit

| gate | result |
|---|---|
| `protocol_identity` (frozen block byte-equality + the physics-knob inventory) | 2 passed |
| `holon-lens` suite (census, lenses, classifier, plants, file path, quenchlog) | 66 passed |

## The arms, and what makes them one variable

| arm | commit | seed | ozone | `--de4` | `dE4_evals` measured | \|p\| / bound |
|---|---|---|---|---|---|---|
| A | `21e6be3` | `0x53415422` | fenced | on | *(running)* | *(running)* |
| B | `21e6be3` | `0x53415422` | fenced | off | **0** | **3.84e-5** |

`dE4_evals` is the counter `Sim::de4_eval_count`, incremented by the physics itself. Arm B
reporting exactly **0** is the functional proof that the four-body term did not fire —
which a symbol-table check cannot establish, because the symbol is inlined away.

## The shared caveat: the O–O curve is not converged

Both arms report:

```
# WARNING O-O: worst residual 4.81e-6 exceeds CONVERGED_RESIDUAL 1e-9.
```

That is **3.7 orders past the threshold the code sets for itself**, on the curve governing
the oxygen aggregation that dominates these scenes.

**And the EXIT, which the warning does not state and which changes what the number means.**
Routed from B1b's banked W1 run: the O–O solve exits on **`IterationCap` at 5000** — it is
BUDGET-LIMITED, not stagnated. Those are different facts and the discriminator law exists
because they are: stagnation would mean more budget buys nothing and the curve is as good as
this method gets, whereas a cap means the residual is a spending decision and a deeper budget
would move it. So the caveat on every certification in this campaign is "computed under a
curve that ran out of iterations", not "computed under a curve that cannot converge".

**My runner cannot print this today, and that is a plumbing gap rather than an oversight.**
`holon_chem::pair::PairMeta` — everything `generate_pair_table` hands back — carries
`worst_residual` and no exit field at all. The `davidson_iters` that would say so lives in a
different struct and never reaches the caller. So "print the exit beside the residual" needs
`PairMeta` to carry it first; that is a `holon-chem` change and is owed there, not here.
Recorded rather than silently left undone.

**It is IDENTICAL in both arms** — 4.81e-6 in each — so it cannot differentiate them, and
the RELATIVE comparison between A and B is unaffected. What it qualifies is any ABSOLUTE
claim about water: a certified molecule from these runs is certified under an O–O curve
that has not met its own convergence bar, and that belongs in the verdict rather than only
in the run log.

*(For the record, `workbench-engine` reported the two arms at 2.68e-6 and 4.81e-6. The
2.68e-6 is the earlier `fenced` arm at `a3b3d4b`, a different commit; both of the
one-variable arms are 4.81e-6. The distinction matters because a differing residual would
have been a second variable and there is not one.)*

## What this file does not establish

It does not make the O–O curve converged, and it does not turn one seed into a rate. It
establishes only that arms A and B differ in exactly one declared way, which is the
precondition for the comparison meaning anything at all.
