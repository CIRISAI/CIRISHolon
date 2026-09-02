# B1b — THE LONG-RANGE RESIDUAL AUDIT, SUCCESSOR FREEZE

*Frozen 2026-09-01, before the instrument carried any of the three changes below and before
any B1b reading existed. B1's `CLASS-MIX-FENCED` verdict is **VOID** and stays VOID: a gate
that fired is not re-argued into passing by its author. This is a different freeze with a
different design, not a re-score of that one, and its git history is the check — this
document lands in its own commit, ahead of the instrument change.*

**misfits:** M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS,
M-BUDGET-LAUNDER, M-STALE-INSTRUMENT, M-PROVENANCE-OVERREACH, M-HOMOG, M-VOLUME-SCALE,
M-VACUOUS-SUCCESS, M-MAX-OVER-SUCCESSES, M-PLANT-OBS, M-PLANT-SECTOR, M-UNTESTED-GAP,
M-EXIT-DISCRIMINATOR, M-KINEMATIC-NONLOCAL.

| id | the contact |
|---|---|
| M-CHEAPER-THAN-ITS-PRICE | the price gate is rebuilt in work units; §3 is entirely about this misfit and how B1 misapplied it |
| M-PLACEMENT-LOTTERY | B1 priced in wall clock time, which is confounded by contention and placement; the correction is a ratio measured inside one process |
| M-DEVICE-CLASS | a curve's cost is a function of the machine it ran on, so an absolute second-count is a specification only alongside a device class; the in-run ratio removes the dependence instead of declaring it |
| M-BUDGET-LAUNDER | VOID classes stay VOID and are printed as such; B1's fired price gate is reported beside this freeze's, never replaced by it |
| M-STALE-INSTRUMENT | one instrument commit produces every number; the arm logs are now committed (B1 §7.3b) and are cited as tree paths |
| M-PROVENANCE-OVERREACH | the manifest refusal names the file it hashed and infers nothing about which run produced it |
| M-HOMOG | a locality audit on a 12-atom box with no bulk; "distant" is bounded by a box diagonal |
| M-VOLUME-SCALE | the discard is a sum over pairs growing as N²; N-scaling is still NOT measured and is still owed |
| M-VACUOUS-SUCCESS | the work count is asserted, and the B_s gate's own informativeness is branch (e) |
| M-MAX-OVER-SUCCESSES | the verdict is a max over frames; no refused frame may drop out of the max |
| M-PLANT-OBS | both plants re-derived for the changed estimator and pre-checked to fire (§7) |
| M-PLANT-SECTOR | each plant names the sector its carrier must be nonzero in (§7) |
| M-UNTESTED-GAP | the ladder is unchanged committed constants; the new price floor is staked against a measured spread rather than interpolated |
| M-EXIT-DISCRIMINATOR | the solver's own exit and route are read and printed, not merely carried — that is half of the work certificate |
| M-KINEMATIC-NONLOCAL | no constraint correlates separated regions in this scene; the separation is trivial here and stated |

---

## 0. WHY THIS EXISTS

B1 measured `CLASS-H` NEGLIGIBLE and returned **VOID** on `CLASS-MIX-FENCED`, because B1's
own price refusal fired: 1021.5 s of curve setup against a 1200 s floor. B1's §7.4 examined
the firing and found the gate, not the curve, at fault — G2 reproduced the O–O curve's
`R_e` to 2.0e-5 bohr and `D_e` to 1.4e-7 Ha, which a fabricated curve does not do, while
the price floor was denominated in seconds taken from a log measured under eight concurrent
trajectory dumps. **Wall clock time cannot separate "did less work" from "did the same work
quicker", and B1 asked it to.**

That verdict stands as VOID. This freeze is the successor B1's §9 named, and it changes
**exactly three** staked things. Everything else — instrument core, trajectories, manifest,
ladder, fence, VOID conditions, stride, the 0.10 fraction — is carried over unchanged.

### 0.1 Provenance of the three changes, disclosed

A successor freeze written by the author of the freeze it replaces must say where its design
came from, because "I changed the gate and now it passes" is the failure mode.

* **The `E_switch` correction predates the mixed class's numbers.** It was found on
  `CLASS-H` and landed in the instrument at `42cab33` *before* the `CLASS-MIX-FENCED` run
  that produced B1's mixed readings was launched. The git history carries that ordering.
* **The `D_s` denominator was already in B1's freeze**, §6.1, as a mandatory reported
  quantity and as branch (e)'s trigger. It is promoted from reported to gated here; it is
  not a new idea introduced to change an answer.
* **The price basis is genuinely new**, and it is new because B1's gate fired and was
  examined. That is the honest account.
* **The 0.10 fraction is INHERITED UNCHANGED.** It is not re-chosen, not widened, not
  narrowed. The one number that would most obviously be tuned to produce an outcome is the
  one number this freeze refuses to touch.

**This freeze is written knowing B1's mixed-class readings** (`LONGRANGE_RESULTS.md` §2.2).
It is therefore staked so that it CANNOT be satisfied by that knowledge: the estimator moves
to the LARGER quantity, the denominator gains a SECOND and much tighter gate, and both must
pass. Every change makes the bar harder, not easier. If `CLASS-MIX-FENCED` passes B1b it
passes a strictly stronger test than B1 would have applied.

---

## 1. THE CLASSES

| class | arm directory | role |
|---|---|---|
| **CLASS-MIX-FENCED** — mixed quench, (O,O,O) fenced, 4 O + 8 H | `census-traj/fenced/` | **primary** — the class B1 could not score |
| **CLASS-H** — pure-hydrogen gas, 12 H | `census-traj/hydrogen/` | **control** — B1 scored it NEGLIGIBLE; a harsher design that flips it has convicted itself, not the class |

`CLASS-MIX-SERVED` and `CLASS-O` remain **VOID (V1, missing arm)** and are not re-opened:
the served arm holds zero trajectories because the (O,O,O) surface failed to generate
(`p2_served_arm_refusal.log`), and the oxygen arm was never dumped.

The control class is not decoration. B1b makes the test strictly harder in two coordinates
at once, and a design that reports everything non-negligible has measured its own severity
rather than the engine. `CLASS-H` is the check on that, and its B1 verdict is the reference.

---

## 2. CHANGE 1 — THE PRIMARY ESTIMATOR IS `E_switch`

B1 gated `E_hard`, the hard-cutoff sum, on the stated ground that it was "the larger and
therefore the conservative direction". **B1 measured that claim false.** The C² switch
begins removing energy at `c − W`, *inside* the cutoff, so

```
|E_switch(c)| ≥ |E_hard(c)|   always
```

and B1's own readings were `25.24×` on `CLASS-H` and `1.26×` on `CLASS-MIX-FENCED`. The
conservative label moves with the inequality: `E_switch` is both the larger quantity and the
one `Sim::set_pair_cutoff` would actually apply, since the engine truncates with a switch
and not a step (`sim.rs:2219`).

> **B1b's primary estimator is `E_switch(c*)`**, with `W = PAIR_SWITCH_WIDTH = 2.0` bohr and
> `S₂ = cells::switch_c2` — the engine's own switch, not a second implementation.
> `E_hard`, `E_band` and `E_tail` remain computed and reported for continuity with B1.

---

## 3. CHANGE 2 — THE PRICE IS IN WORK UNITS, NOT WALL CLOCK TIME

M-CHEAPER-THAN-ITS-PRICE is a real misfit and its founding case is real: a fabricated
(O,O,O) "table" that was a hand-shaped analytic function with six fitted constants and no
electronic structure anywhere. A price gate must still exist. It must be denominated in
something a fake cannot produce and a faster kernel cannot deflate.

### 3.1 W1 — the solver certificate

An analytic stub has no determinant space, no solver route, no exit status and no residual.
A real curve has all four, and they are on the `PairMeta` the generator returns.

> For every curve the class needs, the instrument READS AND PRINTS `route`, `exit`,
> `n_det`, `n_basis`, `solver_budget` and `worst_residual`, and **refuses** unless
> `route == Determinant`, `n_det ≥ 2`, `n_basis ≥ 2` and `solver_budget ≥ 1`.

`n_det`, `n_basis` and `solver_budget` are **printed and NOT gated on a value**. This is
deliberate and it is the M-UNTESTED-GAP discipline: nobody has recorded these numbers for
these curves, and a freeze cannot gate a value no prior record carries without inventing it.
B1b puts them on the record so a successor CAN gate them. Stating that here is the
difference between a gap and a hole.

`exit` is printed per curve and is **not** a pass/fail for O–O, whose non-convergence
(`worst residual 2.7e-6`, above `CONVERGED_RESIDUAL = 1e-9`) is inherited from the census
run and already carried by a `# WARNING` line in the committed arm log.

### 3.2 W2 — the in-run cost ratio

The quantity that removes every confound B1 walked into is the cost of the expensive curve
**in units of the cheap curve solved by the same kernel, in the same process, minutes
apart**. A kernel speedup scales both. Contention scales both. Core placement scales both.
The ratio is work-proportional and survives all three.

```
ratio = t(O–O) / t(H–H),  both measured within one run
```

**The spread was measured on three independent runs BEFORE this floor was staked**, because
a boundary staked without checking its spread is how two campaigns in this programme died:

| run | t(H–H) | t(O–O) | ratio |
|---|---|---|---|
| census arm log (`census_traj_arm_fenced.log`) | 3.5 s | 2596.2 s | 741.8 |
| served attempt (`p2_served_arm_refusal.log`) | 2.9 s | 2737.9 s | 944.1 |
| B1's run (`longrange_fenced.log`) | 1.8 s | 999.3 s | 555.2 |

Observed spread **555 – 944, a factor of 1.70** — against the factor of **2.74** that the
absolute O–O second-count spans across the same three runs. The ratio is the more stable
unit by 1.6×, which is the measurement that justifies using it.

> **W2: the run is REFUSED unless `t(O–O) / t(H–H) ≥ 100`.**

The floor sits **5.5× below the lowest ratio ever observed** and roughly 7.5× below their
mean. It is set to catch the failure the misfit was registered for — an artifact arriving
with no solve behind it at all, which reads a ratio near 1 — and deliberately not set near
the observed band, because the observed band is exactly what a legitimate kernel improvement
moves. A gate placed where B1 placed it fires on progress; a gate placed here fires on
fabrication.

**This floor is honest about what it cannot do.** It cannot detect a curve computed by a
correct-but-different method, nor one solved at a different budget. `G2` and `W1` cover
those. W2 covers only "was electronic-structure work done at all, in the proportion this
pair demands".

**`CLASS-H` has no W2 gate**, because it generates one curve and a ratio needs two. Its
price evidence is W1 alone, and this freeze says so rather than manufacturing a floor that
cannot discriminate — the same reasoning B1 gave, and the reason B1's `CLASS-H` price gate
was correctly a no-op.

---

## 4. CHANGE 3 — TWO DENOMINATORS, BOTH GATED

B1 gated `0.10 · B_s`, the drift **bound** — what the integrator was entitled to lose. B1
also reported `B_s / D_s` and found it 84–387 on `CLASS-H` but **2.8e4 – 1.8e5** on
`CLASS-MIX-FENCED`, meaning the mixed class's bound sat five orders above the drift the runs
actually incurred. A gate against a denominator that loose measures very little.

> **B1b gates BOTH, and a class must pass BOTH:**
> **G1a:** `max |E_switch(c*)| < 0.10 · B_s` for every seed — the entitled bound, carried
> over from B1 so the two audits are comparable.
> **G1b:** `max |E_switch(c*)| < 0.10 · D_s` for every seed — the **incurred** drift, the
> scale a reader means by "negligible".

`B_s` and `D_s` are the bound and peak the run itself reported, read from the committed arm
logs `census_traj_arm_hydrogen.log` and `census_traj_arm_fenced.log` (the `drift
<peak>/<bound>` field). They are READ, never recomputed: the bound is built from running
maxima a trajectory earned, which one frame does not carry.

The conjunction is the point. Passing on the bound alone is what B1's branch (e) was
invented to flag; here it is not a flag but a failure, because the freeze that names the
honest denominator must then be willing to be judged by it.

---

## 5. EVERYTHING CARRIED OVER UNCHANGED

The ladder (`6.0`, `9.0`, `10.4`, `14.0`, **`15.0 = c*`**, `41.0` bohr, every entry a
committed constant); `c* = three_body_cutoff() = list_cutoff() = 15.0` bohr at the commit
that produced the artifacts; the estimator's minimum-image form and arena-index summation
order; the table-edge fence and its two parameter-free power-law comparison tails; the
manifest sha256 refusal; the published stride of every 400th frame; scoring every frame of
every admitted seed; and the 0.10 fraction.

### 5.1 **THE TABLE-EDGE FENCE — THE NUMBER IS STILL A LOWER BOUND**

Unchanged from B1 and restated because it governs every sentence of the verdict:

**Past `r_max` the table is an exponential matched in value and slope at the last knot,
while the true long-range interaction is a power law — dispersion r⁻⁶, dipole–dipole r⁻³.
An exponential decays faster than every power law, so the measured discard UNDERSTATES what
a real truncation would drop. Every verdict below is a LOWER BOUND, and every verdict
sentence in the results document carries that clause.** `E_tail_pow6` and `E_tail_pow3`,
matched to the table's own last value with nothing fitted, are reported to size the fence.

---

## 6. THE GATES

- **G1a — NEGLIGIBILITY AGAINST THE ENTITLED BOUND.** For every admitted seed and every scored frame, `|E_switch(c*)| < 0.10 · B_s`. The max, its seed and its frame index are printed pass or fail. witness: `DependsWithinUpTo`
- **G1b — NEGLIGIBILITY AGAINST THE INCURRED DRIFT.** For every admitted seed and every scored frame, `|E_switch(c*)| < 0.10 · D_s`. Printed the same way. A class passes only if G1a AND G1b pass. witness: `DependsWithinUpTo`
- **G2 — CURVE IDENTITY.** Every regenerated curve reproduces its arm log's printed `R_e` to 1e-4 bohr, `D_e` to 1e-6 Ha, and worst residual within a factor of 2. Failure VOIDs the class. witness: `none (an equality-of-generation gate has no theorem; it is a measured reproduction against a committed log)`
- **G3 — MANIFEST REFUSAL.** Every `.traj` and arm log is hashed against `census_traj_manifest.sha256`; a mismatch or an unlisted path is REFUSED with both digests and the reason printed, contributing 0 frames. witness: `none (a provenance gate on an artifact; M-PROVENANCE-OVERREACH is its warrant)`
- **G4 — WORK COUNT.** At least 50 frames scored per class, printed for every class including the VOID ones. witness: `none (an anti-vacuity assertion; M-VACUOUS-SUCCESS is its warrant)`
- **G5 — LADDER MONOTONICITY.** `|E_hard(c)|` non-increasing along the ladder, 0 violations permitted. witness: `dependsWithinUpTo_mono_radius`
- **G6 — ZERO CONTROL.** At `c = 41.0` bohr, above the box diagonal, `E_hard` is EXACT 0.0 in every frame; the largest separation actually seen is printed beside it. witness: `none (a control zero that is a fact about the SCENE, not about the instrument's coverage)`
- **G7 — BEYOND-CUTOFF POPULATION.** At least 1% of scored frames must have a nonempty beyond-`c*` population, and the fraction is printed. witness: `none (the eligible-pool rate the reading is drawn from; M-VACUOUS-SUCCESS is its warrant)`
- **G8 — PLANT P2 FIRES**, now against `E_switch`: the injected pair moves `E_switch(c*)` by the independently predicted amount to within 1e-9 relative. witness: `none (a plant-observability check on the estimator; M-PLANT-OBS is its warrant)`
- **W1 — SOLVER CERTIFICATE.** Every curve reports `route == Determinant`, `n_det ≥ 2`, `n_basis ≥ 2`, `solver_budget ≥ 1`; all six solver fields printed. Failure VOIDs the class. witness: `none (a work-evidence gate on an artifact; M-CHEAPER-THAN-ITS-PRICE is its warrant)`
- **W2 — IN-RUN COST RATIO.** For a class needing an O–O curve, `t(O–O) / t(H–H) ≥ 100`, both timed within one run. Failure VOIDs the class. Not applied to a single-curve class. witness: `none (a work-proportional price; M-CHEAPER-THAN-ITS-PRICE and M-PLACEMENT-LOTTERY are its warrants)`

Note the tolerance change on G8, from 1e-12 to 1e-9. `E_switch` sums a switch function over
a band rather than a step over a tail, so its floating-point path is longer; 1e-9 is stated
here rather than discovered afterwards. B1 measured 3.016e-14 and 9.493e-16 on `E_hard`, so
this is slack, and slack that is declared in advance is not slack that was needed.

---

## 7. PLANTS

**P1 — the corruption plant, against G3.** A byte-level copy of an admitted `.traj` with one
byte flipped must be REFUSED by name with both digests printed, while a pristine sibling in
the same directory is admitted in the same pass. **Both prongs are demonstrated**: digest
mismatch, and a byte-valid file under a path the manifest does not list.
*Carrier:* the file's sha256 digest. *Sector:* artifact identity — the carrier is **nonzero
in** that sector by construction, since a flipped byte changes sha256 with probability 1,
and nonzero in no other sector the audit reads.

**P2 — the injected pair, against G8 and now against `E_switch`.** In an admitted frame one
atom is displaced to exactly 16.0 bohr from a chosen partner; `E_switch(c*)` must move by
the amount an independent per-pair path predicts. The prediction carries the displaced
atom's other pairs alongside the target pair, because moving an atom moves every pair it is
in — which is what makes P2 a check of the estimator's inclusion bookkeeping rather than an
identity.
*Carrier:* the pair's switched term at 16.0 bohr. *Sector:* the beyond-cutoff pair sector.
The carrier is **nonzero in** that sector because 16.0 bohr lies within the switch's removal
band and the table's value there is nonzero — B1 measured the H–H carrier at −1.547664e-16
Ha and the O–O carrier at −2.028788e-6 Ha, so the plant is pre-checked to fire on both
classes. The instrument refuses to run the plant if the carrier reads 0.0.

---

## 8. THE BRANCHES — every answer's meaning, stated up front

* **BRANCH (a) — NEGLIGIBLE.** G1a and G1b both pass for the class, and every other gate
  passes. → Cutoff-locality at `c* = 15.0` bohr discards less than a tenth of both the
  entitled bound and the incurred drift, measured with the estimator the engine would
  actually apply. **For `CLASS-MIX-FENCED` this would close B1's question for neutral
  oxygen-bearing scenes and B2 is not required by this measurement.** The lower-bound clause
  and the ionic-scope fence (§10) travel in the same sentence, every time.
* **BRANCH (b) — NON-NEGLIGIBLE.** G1a or G1b fails. → **The B2 Ewald-class requirement
  FIRES for that class.** The failing gate is named — bound or drift — the failing seeds and
  frames are listed, and the measured ratio is the size of the bill B2 must pay. This is a
  RESULT, not a failure to get one: a clean boundary number is as bankable as a clean
  negligible, and the campaign exists to learn which.
* **BRANCH (c) — SPLIT.** G1a passes and G1b fails, or the reverse. → **Reported as
  NON-NEGLIGIBLE with the split named**, because the conjunction is what is staked. The
  plain reading is stated in the verdict sentence: the discard is small against what the
  integrator was *entitled* to lose and not against what it *actually* lost. A split is the
  most informative outcome available and it is never rounded to either side.
* **BRANCH (d) — VOID.** Any of V1–V7 fires. → No verdict about that class in either
  direction, the condition named, the class printed with VOID in its verdict column, and the
  VOID structure at the head of the results document.
* **BRANCH (e) — UNINFORMATIVE BOUND.** The class passes both gates but `B_s / D_s > 1e3`
  on any seed. → Still a pass; reported as `NEGLIGIBLE (G1a uninformative)`, recording that
  the bound gate carried no information and the verdict rests on G1b alone. The label
  travels wherever the verdict is quoted.

**Pre-committed follow-up.** If a class lands in (b) or (c), the same instrument is run at
the smaller ladder radii with no threshold changed, and the radius at which the class first
crosses `0.10 · D_s` is reported as B2's design input. No new estimator, no new denominator,
no re-chosen frames.

---

## 9. VOID CONDITIONS

- **V1 — MISSING ARM.** Zero parked `.traj` files. Known: `CLASS-MIX-SERVED`, `CLASS-O`.
- **V2 — MANIFEST MISMATCH.** Refusals leaving a class under 50 scorable frames.
- **V3 — CURVE IDENTITY.** G2 fails for any curve the class needs.
- **V4 — ESTIMATOR DEFECT.** G5 or G6 fails anywhere.
- **V5 — EMPTY POPULATION.** G7 fails.
- **V6 — NO SOLVER CERTIFICATE.** W1 fails.
- **V7 — BELOW THE WORK FLOOR.** W2 fails on a class that needs it.

A VOID class is never scored, never inferred from a sibling, and never reported with a
number in its verdict column. **B1's `CLASS-MIX-FENCED` VOID is not erased by this freeze**;
it is a fact about B1 and is reported beside B1b's outcome whatever that outcome is.

---

## 10. WHAT THIS MEASUREMENT CANNOT SAY

Carried from B1 unchanged, written before the answer.

1. **Pair sector only.** The many-body surfaces return exact zeros outside their domains and
   discard nothing by truncation. witness: `DependsWithinExact`
2. **No ionic species.** Every nucleus is neutral H or O and every curve decays
   exponentially. The r⁻¹ case — the one that makes B2 near-certain once node C ships ionic
   scenes — is **not touched**. A branch (a) verdict here is not a statement that B2 is
   unnecessary, and using it to defer B2 for ionic scenes is a misuse of it.
3. **Twelve atoms, two dimensions, walls.** The gated ratio is N-dependent and its N-scaling
   is NOT measured (M-VOLUME-SCALE). Still owed.
4. **The number is a LOWER BOUND** past the table edge (§5.1).
5. **The census scenes discard nothing as they stand.** `pair_switch == None`, so the pair
   sector runs the complete `N²/2` sum. This is a counterfactual about the O(N) route a
   scaled-up scene must take, and quoting it as "the engine discards X" is quoting it wrong.
