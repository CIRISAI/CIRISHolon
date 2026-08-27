# Q8 MPS RESULTS — in progress

**This file is opened before any reading of the re-run is seen**, per chief-of-staff-2's binding
requirement: the classification table below is pre-registered now, verbatim in team-lead's terms,
and may not be reinterpreted after data exist.

## Pre-registered classification: the four outcomes of G3-primary's per-sweep history

Applied per configuration, crossing G7's convergence call against G3-primary's monotonicity clause
(`hist.windows(2)`, staked slack `1e-9`) on that configuration's `energy_history`:

| G7 | monotonicity | outcome |
|---|---|---|
| stall (`converged=false`) | non-monotone | **(a) oscillation** — promotes toward the CORRECTNESS KILL |
| stall (`converged=false`) | monotone | **(b) slow convergence** — stays a G7 VOID, Q9's stagnation premise intact |
| `converged=true` | monotone | **(c) claim upgraded to warranted** |
| `converged=true` | non-monotone | **(d) right-by-reference, convergence claim RETRACTED with an asterisk** — G2 (which compares against q-seam's exact ground state directly and does not depend on sweep history) may still pass; what is retracted is only the claim "this converged," never the physics |

**(d) is not to be reinterpreted post hoc.** The scope is ALL 8 configurations in the `N ∈ {8,10}`
grid (fence widened from the original 3 stalling ones, per chief-of-staff-2/research-manager: the
early-stop criterion `|E_k − E_{k-1}| <= sweep_tol` alone cannot distinguish a settled state from a
turning point of an oscillation for ANY configuration, not only the ones that visibly failed to
reach it).

G3-primary's own two clauses (floor, monotonicity) are graded for vacuity separately, per
research-manager's conjunction-vacuity discipline: if `worst_rise` sits at machine zero across all
8 configurations, the monotonicity clause was never exercised and G3-primary is graded
floor-only-effective, not two-clause; if `worst_floor_margin` is enormous everywhere, the floor
clause is the vacuous one. Reported with the actual numbers once the run completes.

## Provenance note on the discarded run

A `full_grid_gates` run launched 2026-08-24 07:31:23 (PID 1498505/1498515) was found, on direct
evidence, to predate every defect fix (G3-primary's `energy_history`, the per-spin-orbital
particle-hole anchor, the exact-reference cache): source files were modified 10:36:38–10:44:55 and
the binary rebuilt 10:45:08, all after the process's start time, and `/proc/1498515/exe` resolved to
the ORIGINAL binary file marked `(deleted)` — the kernel keeps a running process bound to the inode
it exec'd, unaffected by later rebuilds overwriting the path. That run is discarded; none of its
per-config panics are findings. Its raw DMRG diagnostics (E, sweeps, discarded weight) are not
reused either, to keep this results file backed by one coherent, fully-instrumented run rather than
a patchwork of two binaries.

## The re-run — COMPLETE, 30,098 s (8h22m), and the four-outcome table resolves cleanly

Recorded by the integrator after the campaign lanes had stood down; the run outlived the
session that launched it. Raw log committed alongside this file
(`output/q8_mps/full_grid_gates.log`) so the readings below are checkable rather than
quoted.

**The table, adjudicated against the four outcomes committed at `3123000` BEFORE any
reading existed:**

| config | sweeps | converged | monotone | outcome |
|---|---|---|---|---|
| N=8 U=0 | 5/20 | yes | yes | **(c)** convergence claim warranted |
| N=8 U=1 | 5/20 | yes | yes | **(c)** |
| N=8 U=4 | 5/20 | yes | yes | **(c)** |
| N=8 U=16 | 20/20 | no | **no** | **(a)** oscillation |
| N=10 U=0 | 20/20 | no | **no** | **(a)** |
| N=10 U=1 | 20/20 | no | **no** | **(a)** |
| N=10 U=4 | 20/20 | no | **no** | **(a)** |
| N=10 U=16 | 20/20 | no | **no** | **(a)** |

**THREE VERDICTS, none of which required a judgement call the table had not already
made.**

1. **The §7 SWEEP KILL FIRES: 5 of 8 VOID against a threshold of 2.** Already ruled
   fired pre-closeout on three configs; the completed run raises the count and changes
   nothing about the adjudication.
2. **CASE (d) NEVER FIRED.** No configuration is converged-and-non-monotone. The five
   monotonicity failures are exactly the five VOID configurations — the seductive case,
   fenced in advance precisely because it would have been reinterpretable after the
   fact, simply did not occur. Per the standing ruling, a G7-VOID configuration is not
   a gate datum (`Posed.adjudicate_void_iff`: VOID is the question never posed), so
   **the correctness kill does NOT fire.**
3. **THE THREE CONVERGED CONFIGURATIONS ARE UPGRADED** from unwarranted-by-criterion to
   warranted: they converge in 5 sweeps with monotone histories and zero discarded
   weight.

**What the stalls actually look like, because "oscillation" spans two very different
magnitudes here and the record should not flatten them.** N=10 U=4 rises by 7e-5 on an
energy of −25.38 with the exact reference matched to 3.5e-6 relative — near-converged,
failing its band by a hair. N=10 U=16 is the pathological one: energy error 1.4e-2,
|m_i| = 0.276, Sz = 0.127, Sz² = 0.368 — the partially-melted Néel signature, with spin
observables far outside their bands on an ansatz that is not symmetry-adapted and
therefore does not forbid them.

**Consequence for Q9, unchanged and still open.** Case (a) across every stall means the
stalls are oscillations rather than slow convergence, which is what put Q9's stagnation
premise in question in the first place. That question is NOT settled here: the
canonical-form / rank-deficiency check is the discriminator, and until it reports, the
honest statement is that Q9's premise is unconfirmed rather than refuted. Nothing in
this run licenses rewriting Q9's design; it licenses waiting for the check that was
built to decide it.

---

## ADDENDUM (2026-08-25): the discriminator reported, and it was the instrument

**This section is written before any reading of the repaired grid re-run is seen**, under the
same discipline the head of this file was opened with. It is adjudicated against the SAME four
outcomes committed at `3123000`; no new outcome is invented for the re-run, and the table above
is not edited.

### What changed under the grid

The section above closed by naming its own discriminator:

> the canonical-form / rank-deficiency check is the discriminator, and until it reports, the
> honest statement is that Q9's premise is unconfirmed rather than refuted.

**It has reported, and it found a defect in the instrument rather than a fact about the physics.**
The two-site Jacobi SVD judged column orthogonality on an ABSOLUTE Gram cross-term. An SVD must
judge it RELATIVE to each column's norm: DMRG carries Schmidt values across many decades, so an
absolutely tiny cross term can still mean two normalized singular vectors are nearly parallel.
Worse, the old loop treated a sweep that failed to shrink the cross-term as *converged*
("stagnation"), so the failure was silent by construction. Failure-first regression, reproduced
here independently before applying the fix:

```text
left block basis lost canonical form: defect=1.7995051073022862e-5
test strong_coupling_sweep_preserves_both_canonical_bases ... FAILED
```

After the repair the same test passes at `defect ~ 9e-15`, and `split_two_site` now ASSERTS
convergence, so a future non-canonical basis is a panic rather than a quiet wrong answer.

This also explains the χ dependence that had no good explanation before: at small χ, truncation
kept the retained spectrum inside the range where an absolute tolerance was accidentally
adequate. The bug needed a wide spread of Schmidt values to bite — which is exactly what strong
coupling and generous χ produce. The grid ran at `CHI_MAX = 256`.

### What this does to the recorded verdict

**The three verdicts above were measured on a since-repaired implementation.** Two of the five
VOID configurations have already been replayed at the grid's own χ=256 and both invert:

| config | recorded | repaired replay |
|---|---|---|
| N=8 U=16 χ=256 | 20/20, `converged=false`, non-monotone — **(a)** | 5 sweeps, `converged=true`, monotone — **(c)** |
| N=10 U=16 χ=256 | 20/20, `converged=false`, non-monotone — **(a)** | 5 sweeps, `converged=true`, monotone — **(c)** |

N=10 U=16 was the configuration this file singled out as "the pathological one." Its canonical
defects now sit at `2.3e-13 / 1.2e-12` and its energy is `3.45e-12` from the cached q-seam
reference. The remaining three VOID configurations (N=10 at U=0, 1, 4) are being re-run on the
same `full_grid_gates` harness that produced the table above — not a different binary — and the
result will be recorded below whichever way it falls, including if it leaves the kill standing.

**Nothing above is retracted by anticipation.** The recorded table stays exactly as measured; a
kill that fired on a defective instrument is not un-fired by a repair, it is re-posed. What is
stated now, before the reading, is the conditional: if the remaining three converge monotonically,
the §7 sweep kill's 5-of-8 VOID count falls below its firing threshold and the kill does NOT fire,
and the three configurations upgrade to case (c) — a warranted convergence claim.

**THRESHOLD CORRECTION, and a defect in the gate itself — both recorded before the re-run's
verdict was read.** The paragraph above first said "falls below its threshold of 2." That is
wrong, and reading `full_grid_gates.rs:277–279` gives the exact logic:

```rust
let sweep_kill_absolute     = void_count > 2;                                  // >= 3
let sweep_kill_proportional = (void_count as f64) > (2.0 / 12.0) * grid_size;  // >= 2 at grid_size 8
let sweep_kill_fires        = sweep_kill_absolute && sweep_kill_proportional;
```

The firing threshold is therefore **3 VOIDs, not 2**.

And the two-reading construction is **vacuous under that conjunction**. `absolute` (≥3) strictly
implies `proportional` (≥2), so `absolute && proportional` *is* `absolute`: the proportional prong
cannot change any verdict, at any `void_count`, for any grid size where the two are ordered this
way. The source comment says the ambiguity was "ruled here under BOTH readings … Both fire on this
data, so nothing is left unadjudicated and the amendment decided neither." That is true of the
5-VOID data and structurally misleading in general — under `&&` the weaker prong is inert, and the
only case it could ever have decided (exactly 2 VOIDs: proportional fires, absolute holds) is
precisely the case the conjunction discards. Amendment 2 shrank the grid 12 → 8 and the
disjunction/conjunction choice was never posed; it silently resolved to the absolute reading alone.

**The gate is NOT being changed now.** Altering an instrument between a recorded run and its
re-run is the one move that would make the comparison unreadable, so the re-run is adjudicated on
the gate exactly as the recorded run was, and this defect is carried as a finding to be settled
afterwards — on the record, in advance, and not as an explanation of whatever the re-run returns.

**Consequence for Q9, stated before the reading — AMENDED before the reading arrived.** The
first version of this paragraph said that if the re-run completes the pattern, "Q9's design needs
re-deriving rather than resuming." That sentence presupposed a Q9 design document. **There is
none.** `Q10_PREREG.md` §10 already established it — `find . -iname "*Q9*"` returns nothing, and
M1-M6 were dropped there for precisely this reason, under the rule *a cross-reference is a warrant
only if its target exists*. I repeated the error the same file had already diagnosed, which is
why this amendment is recorded here rather than edited away.

The honest statement is narrower and sharper. "Q9's stagnation premise" is message-only content;
it has no design in the tree to re-derive. What the repair changes is therefore not a design but
an evidence base: the stalls recorded above were the ONLY written support ever offered for that
premise, and in at least two of five cases they are now known to be an SVD tolerance bug rather
than a fact about the model. If the re-run completes the pattern, the premise does not become
false -- it becomes unsupported, with nothing in the repository standing behind it. Anyone
reviving Q9 owes it a written brief and fresh evidence, not a resumption.

---

## THE RE-ADJUDICATED GRID (2026-08-26) — all eight converge, the §7 sweep kill does NOT fire

Run: `full_grid_gates`, the same harness that produced the table above, on the repaired SVD
(`4bcf0d2`). Detached, 2h28m wall (`8872.21s`, against the recorded run's 8h22m), exit code **0**
where the recorded run exited **101**. Adjudicated against the FOUR OUTCOMES committed at
`3123000`; no outcome was invented for this run.

**The harness's own verdict line, quoted rather than summarised:**

```text
=== SWEEP KILL: 0 of 8 configurations VOID under G7 — absolute reading (>2) holds,
    proportional reading (>2/12 of grid) holds — SWEEP KILL DOES NOT FIRE ===
test result: ok. 1 passed; 0 failed; 0 ignored
```

| config | recorded | repaired re-run | outcome |
|---|---|---|---|
| N=8 U=0 | 5/20 yes, monotone | 5/20 `converged=true` | **(c)** unchanged |
| N=8 U=1 | 5/20 yes, monotone | 5/20 `converged=true` | **(c)** unchanged |
| N=8 U=4 | 5/20 yes, monotone | 5/20 `converged=true` | **(c)** unchanged |
| N=8 U=16 | 20/20 **no**, non-monotone | **5/20 `converged=true`** | **(a) → (c)** |
| N=10 U=0 | 20/20 **no**, non-monotone | **8/20 `converged=true`** | **(a) → (c)** |
| N=10 U=1 | 20/20 **no**, non-monotone | **6/20 `converged=true`** | **(a) → (c)** |
| N=10 U=4 | 20/20 **no**, non-monotone | **6/20 `converged=true`** | **(a) → (c)** |
| N=10 U=16 | 20/20 **no**, non-monotone | **5/20 `converged=true`** | **(a) → (c)** |

**All five VOID configurations inverted. The three that were already (c) are unchanged — that is
the control, and it is the reason this reads as a repair rather than as a loosened gate.** The
worst energy error across the grid is `2.3e-12` relative against a band of `1e-8`, and N=10 U=16 —
the configuration this file singled out as "the pathological one," with energy error 1.4e-2 and
|m_i| = 0.276 — now returns `rel_err = 2.15e-12` and `discarded_max = 3.6e-21`.

### The three verdicts, restated on the repaired instrument

1. **THE §7 SWEEP KILL DOES NOT FIRE.** 0 of 8 VOID against a firing threshold of 3. **This does
   not un-fire the recorded kill.** That kill fired on a defective instrument and is not reversed
   by a repair; it is *re-posed*, and the re-posed question answers negative. Both readings hold,
   so the prong defect recorded above did not bite here either.
2. **CASE (d) STILL NEVER FIRED.** No configuration is converged-and-non-monotone, now across two
   independent runs of the grid. The seductive case, fenced in advance, has not occurred.
3. **ALL EIGHT CONFIGURATIONS ARE WARRANTED CONVERGENCE CLAIMS**, up from three.

### The vacuity grading, which the pre-registration demands and which cuts against this result

§ above required: *"if `worst_rise` sits at machine zero across all 8 configurations, the
monotonicity clause was never exercised and G3-primary is graded floor-only-effective."*

`worst_rise` across the grid: `4.6e-13, −1.2e-13, −1.1e-13, 2.6e-13, 3.4e-13, 8.3e-13, −1.6e-11,
−1.8e-12` — against a band of `1e-9`. That is machine zero across all 8. **G3-primary's
monotonicity clause is therefore graded NOT EXERCISED in this run.**

And the floor clause is in the same condition: `worst_margin` runs `−3.5e-12` to `+2.6e-11`
against a band of `−1e-9`, three orders inside. **So G3-primary is unexercised in BOTH clauses
here, not merely floor-only-effective** — a stronger and less flattering grading than the
pre-registered rule anticipated, recorded because the rule's purpose is served by the honest
reading rather than the nearest listed one. What this means precisely: the re-run gives no
evidence that G3-primary would CATCH a violation. It gives evidence that there is nothing to
catch. Those are different, and only the second is claimed. The monotonicity clause did real work
on the recorded run — it is what produced five case-(a) readings — so the comparison between runs
stands; what does not stand is any claim that this run validated the gate.

### Consequence for Q9, now cashed

The addendum's conditional has been met. Q9's stagnation premise rested on stalls that are now
known, in all five cases, to be an SVD tolerance bug rather than a fact about the model. Per the
amendment above: **the premise is not falsified — it is left unsupported**, with nothing in the
repository standing behind it, and no Q9 design document exists to re-derive. Anyone reviving Q9
owes it a written brief and fresh evidence.

**Standing after this run:** the recorded table is untouched and stays as measured. This section
sits beside it as a second measurement on a repaired instrument, and the difference between them
is the repair.
