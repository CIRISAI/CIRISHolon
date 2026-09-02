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

**And the EXIT, which the warning did not state and which changes what the number means.**
The O–H and H–H curves exit `Converged`. The O–O curve exits **`IterationCap`** — and at
every reachable budget: 12 of its 96 knots cap at 4000 and the same 12 at 5000, with nine
still capping at 20000 (measured by the `saturation2-water` lane).

Three consequences, and the third is the one that touches this census:

1. **Budget-limited, not stagnated.** `Stagnated` would mean more iterations buy nothing and
   the caveat is structural; a cap means the residual is a spending decision. So the caveat
   is "ran out of iterations", which is weaker and fixable.
2. **The residual is NOT A BOUND.** Under thick restart a capped residual is a SAMPLE of a
   non-monotone sequence — not an upper bound on the error in either direction. So 4.81e-6
   may not be quoted as an error bar; it is a snapshot of where the solve stopped.
3. **The unconverged region is exactly the region this census's bond criterion reads.** The
   well and everything inside `R_e` converge at 1e-10; the caps are in the DISSOCIATION TAIL
   past about 6 bohr, at a magnitude of 4.3e-6 Ha. And `Sim::refresh_pairs` decides `bonded`
   from `e_rel < 0` and `r < r_outer` — the outer classical turning point, which lives in
   that tail. **So this does touch the certifications rather than sitting harmlessly beside
   them**, and the honest statement is that the membership view's edge criterion is
   evaluated on the part of the O–O curve that did not converge.

**The residual must be quoted WITH ITS BUDGET, because it is not monotone in effort.**
Reconciled with the `saturation2-water` lane, whose `oo_budget_4000_to_5000.log` carries both
columns for this same curve at the same 96 knots: worst residual **2.683e-6 at budget 4000**
and **4.81e-6 at budget 5000**. Mine is the budget-5000 number and theirs was the
budget-4000 one; both are honest readings of the same curve.

The larger budget produced the LARGER worst residual, by a factor of 1.8 **the wrong way**.
So a capped curve's residual is not merely un-tight — it is not monotone in the effort spent,
and the natural reading "more iterations, tighter bound" is false here. Any statement of the
form "certified under a curve whose residual is X" is incomplete without the budget, since
the same curve honestly reports a different X at a different one.

**And the magnitude of what this census inherits, quantified rather than gestured at.** The
caps affect the tail energies by 4.3e-6 Ha — **0.45% of kT at the quench's 300 K target** —
which shifts the outer classical turning point by about **3.7e-4 bohr**. Against intra-block
separations of order 2–6 bohr that is a relative effect near 1e-4 on the criterion that
decides membership. It is a real inheritance and it is a small one, and both halves of that
sentence are load-bearing: small enough that no certification in this document plausibly
turns on it, real enough that it belongs inside the claims rather than beside them.

*(A correction I owe here: I previously reported that `PairMeta` carries no exit field and
that printing the exit needed a chem change. That was false — `pub exit: SolveExit` has been
at `pair.rs:660` since `75cd8ff`. My grep read the first 22 lines of a 64-line struct and I
reported the absence as established. What was actually missing was narrower — the exit was
not written into the shipped JSON — and `saturation2-water` had already fixed that at
`e3d7eb6`. The runner now prints `worst residual X (exit Y)` from the field that was always
there.)*

## What this file does not establish

It does not make the O–O curve converged, and it does not turn one seed into a rate. It
establishes only that arms A and B differ in exactly one declared way, which is the
precondition for the comparison meaning anything at all.
