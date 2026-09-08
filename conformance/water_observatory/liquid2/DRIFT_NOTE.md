# The CT-3 table's drift in the liquid — what was measured, what it points at, what is owed

*A note, not a freeze and not a results file. It exists because LIQUID-2's labelled step sweep
was supposed to choose a step and found an instrument defect instead, and because CT-3's
results file should be able to cite this from the outside. Every number here is from a record
under `liquid2/screen/`; every one of those records says `dry: true` and none of them enters a
gate or a claim.*

## The finding, first

**Serving channel 6 as CT-3's sixty-knot table costs three orders of magnitude of energy
conservation in the 128-water periodic box, and the integration step does not buy it back.**

Two controls, each ONE knob against the same box, the same seed, the same step, the same
1,000 settling and 2,000 counted frames, the ledger rebased at the end of the settling:

| arm | law | drift peak, Ha | against the table arm |
|---|---|---|---|
| `x1_ct2` | CT-2's law, LIQUID-1's own | **4.883551e-6** | 1348x |
| `x1_ct3-noct` | CT-3's law, channel 6 switched OFF | **4.719995e-6** | 1395x |
| `x1_ct3` | CT-3's law, the TABLE served | **6.584682e-3** | — |

The two controls agree with each other to `3.5 %`, which is what says the pipeline is not the
cause: the same runner, the same box and the same settling give LIQUID-1's own order of drift
on LIQUID-1's own law, and give it again on CT-3's law with only channel 6 removed.

## The step is not the cause either

| x the tables' step | dt, au | drift peak | thermostat, Ha | drift / work | mean T, K |
|---|---|---|---|---|---|
| 1 | 1.077481 | 6.5847e-3 | -2.5716e-1 | 2.5605e-2 | 423.8 |
| 2 | 2.154962 | 2.5094e-3 | -3.5502e-1 | 7.0683e-3 | 383.2 |
| 4 | 4.309924 | 4.4232e-3 | -4.2794e-1 | 1.0336e-2 | 347.3 |
| 8 | 8.619848 | 7.4281e-3 | -5.4921e-1 | 1.3525e-2 | 327.8 |
| 16 | 17.239696 | 1.6122e-2 | -3.6225e-1 | 4.4504e-2 | 304.4 |
| 32 | 34.479392 | 1.3878e-2 | -1.8093e-1 | 7.6705e-2 | 295.8 |

Across a **32-fold** change of step the drift moves by a factor of **6**, and not
monotonically. A symplectic integrator's own error on this box goes as `(omega dt)^2`, so a
32-fold step change would move it by `1024`. Whatever this is, it is in the SERVED ENERGY and
not in the integration. The columns close and the momentum residual is three orders under its
bound on every arm; 128 units held on every pass of every one; the `1x` and `8x` arms were run
twice under two knob parametrisations and reproduce BIT FOR BIT.

## The suspect, and CT-3 named it first

The only discontinuity the served law has is its own serving rule. Channel 6 is served ONCE
per unordered pair of units, at that pair's **shortest** cross-unit H...O contact — an ARGMIN
over four candidates (two directions, two hydrogens each). An argmin is discontinuous where it
ties, and at a tie the served energy jumps by the difference of the table at the two
coordinate points.

CT-3 measured that jump and fenced it honestly: over a full turn of the donor in 3,601 samples
at four separations, the worst jump is `1.418e-5` hartree, against its own refusal floor of
`5e-4`. That was a statement about **one dimer**. A 128-water box has `8,128` unordered pairs
of units, each re-ranking four contacts every frame. CT-3's own results file names the same
shape from the other side: *"the map has no node with a pair donating in BOTH directions at
once, which a liquid has and the serving rule cannot represent."*

## The diagnosis

`liquid2/screen/x1_ct3_handovers.json`, a labelled screen (`--handovers`) that counts every
argmin handover per frame and evaluates the served pair energy at BOTH contacts at that
frame's geometry, so the jump is measured and not modelled. The counter reads positions and
writes none, so the arm integrates the identical trajectory with it and without it.

**VERDICT: THE CAUSE IS CONFIRMED.** The handovers' own accumulated jump accounts for
`96.6 %` of the measured drift peak.

| | |
|---|---|
| frames | 2,000 |
| argmin handovers | **3,803**, `1.9015` per frame |
| pair contacts served per frame | 4,062.98 |
| mean absolute jump | `9.030206e-6` Ha |
| worst single jump | `2.012547e-3` Ha, **142x** CT-3's whole-dimer worst |
| sum of absolute jumps | `3.434188e-2` Ha |
| signed sum | `4.858462e-3` Ha |
| **signed running peak** | **`6.361070e-3` Ha** |
| **measured drift peak on the same arm** | **`6.584682e-3` Ha** |
| ratio | **`1.0352`** |

Three things make this a confirmation rather than a coincidence of magnitudes:

1. **The statistic that matches is the right one.** The engine's `drift_peak` is the running
   extremum of `|ledger - l0|`. The handover analogue of that is the running extremum of the
   SIGNED sum of jumps, and it is `6.361070e-3` against a drift peak of `6.584682e-3` — 3.5 %
   apart. The absolute sum (`3.434188e-2`) is five times too large and the independent
   random-walk estimate (`5.568790e-4`) is twelve times too small, so neither of those would
   have matched, and the one that does is the one the ledger actually accumulates.
2. **The counter did not perturb what it measured.** The arm is BIT-IDENTICAL to `x1_ct3`
   without the counter: the same drift peak to twelve digits, the same mean temperature. The
   counter reads positions and writes none.
3. **The size of the jumps is CT-3's own number, scaled by the liquid.** The mean absolute
   jump, `9.03e-6` Ha, is the same order as the `1.418e-5` CT-3 measured turning one dimer.
   What the liquid adds is not a bigger typical jump but a WORSE tail — `2.01e-3` Ha at its
   worst, 142 times the dimer's — and 3,803 of them where a dimer sweep saw a handful.

The residual `3.4 %` is not accounted for and is not claimed to be: it is the size of what is
left for the other two suspects the ruling named — the switch `S(r)` applied on the contact
distance while the table's own coordinates carry `r`, and the below-knot tail's gradient.
Neither was tested here.

## The fix that is owed

A **declared continuous serving rule** in place of the argmin: a partition of unity over the
four cross-unit H...O contacts,

```
w_k = exp(-beta r_k) / sum_j exp(-beta r_j)
E_pair = sum_k w_k * E_table(coords_k)
```

with `beta` DERIVED from the map's own records rather than chosen — from the smallest
separation between a node's shortest and second-shortest contact together with the table's own
resolution floor `7.091e-4` hartree, `beta` set so the secondary weight at that separation puts
the second contact below the floor's fraction of `E_CT`. The gradient is analytic on all five
atoms of all four contacts and the finite-difference gate (CT-3's G-B3) is re-run on it. At
every node of the map one contact dominates, so G-C1's numbers should move by less than the
resolution floor, and by how much is a number that campaign must report.

**The gate on the fix:** the served law's drift at `1x` comes within `2x` of the
channel-6-off control (`4.719995e-6`), on the same box, seed, settling and frame count. Only
then does LIQUID-2's step rule — the largest step whose drift stays under L1's bar — have
anything to select, and only then can R3's window and the campaign's price be derived.

## What is owed, plainly

*Superseded in part by the section **PAID (2026-09-08)** at the end of this note: items 1,
2 and 5 are now paid and their numbers are there. The "Not built" and "Not run" markings
below are kept as this list stood when the defect was diagnosed, because a list that
quietly rewrites itself is not a record.*

1. The smooth serving rule, built and gated as above. **Not built.**
2. The step sweep re-run under it. **Not run.**
3. LIQUID-2's gate re-run at the step the rule then selects; the records now under
   `liquid2/gate_provisional_8x/` are at a provisional step and their README says which of
   their numbers survive it being unchosen.
4. A labelled screen logging BOTH the cross-unit potential energy and the bond count through
   one settling, to establish whether the network is flat where the energy criterion fires.
   The equilibration variable was moved off the bond count so that R2 stays a forward
   prediction, and the corrected criterion fires at `7,100` frames where the withdrawn one
   fired at `11,850`. **Not run.**
5. `beta`'s derivation needs the shortest and second-shortest contact separation at each of
   the map's 64 nodes. Those geometries are in `ct1/sector_*.json` and `ct2/gd0_*.json`; the
   number has NOT been extracted here.


## Correction (2026-09-07, second external review): the smooth rule's force, and the lens

- For `E = Σ_k w_k E_k` with `w_k = e^{−βr_k}/Σ_j e^{−βr_j}` the force is
  `F = −Σ_k w_k ∇E_k − Σ_k E_k ∇w_k`; the specification above named only the first term. The
  second is `∇w_k = w_k(−β∇r_k + β Σ_j w_j ∇r_j)` on the atoms of every contact, and the
  finite-difference gate must run on the RECORDED handover geometries (the liquid's worst
  cases), on hydrogen permutations, molecular exchange, contact ties and periodic crossings —
  the table's own nodes would miss the failure this note measured.
- The diffusion lens's wall-saturation cap (`lens.rs`, written for `Boundary::Walls`) is applied
  to unwrapped PERIODIC displacements, which do not saturate; the window above was sized to that
  cap and must not be. The lens becomes boundary-aware, the exponent gate stays, finite-size
  effects are assessed apart, and the slope fit and the exponent check share one declared lag
  interval.
- The step is chosen by NVE runs from identical checkpoints at several steps over EQUAL physical
  durations, by energy fluctuation and observable convergence — the sweep above varied duration
  with the step.


## PAID (2026-09-08): the smooth serving rule is built, derived, gated, and the drift is closed

*Records under `conformance/water_observatory/ct3/smooth/`, every one `dry: true`. The engine
change is `seam.rs`'s `CtServe::{Argmin, Blend}` with `CtTable::serve_blend`, and
`sim.rs::accumulate_seam`'s blend branch; the runner is `examples/ct3_smooth.rs`; the gates are
`tests/ct3_smooth.rs` and `ct3/smooth/gate.json`. The argmin path is untouched — CT-3's own
`ct3/gate.json` re-runs field for field, timings aside.*

### The finding, first

**The drift defect is closed.** Same box, same seed, same settling, same 2,000 counted frames,
same ledger rebase — the three arms now read:

| arm | serving rule | drift peak, Ha | against the channel-6-off control |
|---|---|---|---|
| `x1_ct3-noct` (this session) | channel 6 OFF | **4.719994550e-6** | 1.0000 |
| `drift_ct3-blend` | the BLEND | **4.984881116e-6** | **1.0561** |
| `x1_ct3` (the note's own arm) | the ARGMIN | 6.584681837e-3 | 1394.6 |

The gate was `2x` the control. The blend comes in at `1.06x` — and **1,321 times** below the
argmin it replaces. The control was RE-MEASURED here rather than cited and reproduces LIQUID-2's
own record exactly (`4.71999455e-6`), which is what says this is LIQUID-2's box and not a box
that resembles it.

### The rule, as built

```text
E_pair = Σ_k w_k · S(r_k) · E_table(coords_k)          over the FOUR cross-unit H···O contacts
w_k    = e^{−β r_k} / Σ_j e^{−β r_j}
−∇E    = −Σ_k w_k ∇Ẽ_k − Σ_k Ẽ_k ∇w_k,   ∇w_k = w_k(−β ∇r_k + β Σ_j w_j ∇r_j)
```

on all five atoms of all four contacts. Both force terms are carried — the second is the one
this note's first specification omitted and the second review caught — collected as
`−β Σ_k w_k (Ẽ_k − E) ∇r_k`, with `∇r_k` on contact `k`'s own hydrogen and acceptor oxygen. The
switch applies PER CONTACT: a contact at or past `r_cut` contributes an exact zero to the energy
and to both force terms while still carrying its weight, so the pair is skipped only when every
contact is out.

`CtServe::Argmin` is the default and the selector lives on the TABLE, so every record written
before this rule still reads the argmin, bit for bit.

### β, DERIVED — every input a record's, nothing typed

At each of the map's 64 nodes, the shortest and second-shortest cross-unit H···O contact. Over
four contacts the second's weight is at most `1/(1 + e^{βΔr})`; require it below the table's own
resolution floor as a fraction of that node's `|E_CT|`, which is `β > ln(|E_CT|/floor − 1)/Δr`:

| | |
|---|---|
| the table's resolution floor | `7.090642e-4` Ha — the largest pole spread in `ct3/ct_table.json` |
| the geometry records' own resolution | `1e-10` bohr — the decimals `ct2/node_*.json` and `ct2/gd0_*.json` print |
| the smallest REAL separation `Δr_min` | `0.05675989533` bohr at `twist_R3.4_t120` (`\|E_CT\| = 5.494829e-3`, fraction `0.129042101`) — the recipe there gives `33.640917` |
| the BINDING node (the largest requirement, adopted) | `twist_R2.7_t120`: `Δr = 0.06371274533` bohr, `\|E_CT\| = 2.453586e-2`, fraction `0.0288990973` |
| **β** | **`55.16353128` per bohr** |

The adopted value is the largest requirement over every node, not the requirement at `Δr_min`,
so the criterion holds at EVERY node with content and not only at the closest pair of contacts.

**Two things the derivation found that this note did not know.**

1. **The map holds two EXACT ties.** `donor_R2.9_b90` and `donor_R3.1_b90` — the donor bent 90°
   puts both its hydrogens equidistant from the acceptor's oxygen by the builder's own symmetry.
   Their measured separations are `6.116e-12` and `5.762e-12` bohr, which is round-off in
   geometries recorded to ten decimals. **At those two nodes CT-3's argmin is a coin flip**, and
   no finite `β` satisfies the criterion. They are named in `beta.json` and what the blend does
   there is MEASURED by gate (a) rather than assumed — their `|E_CT|` (`1.64e-4`, `7.33e-5`) is
   itself under the table's floor.
2. **Five nodes where the criterion is vacuous** — `donor_R2.9_b60`, `donor_R3.1_b60`,
   `twist_R3.4_t60`, `donor_R3.1_b45`, `linear_R3.7` — because the floor already exceeds half
   the node's own transfer energy. They constrain nothing and are excluded by name.

### The gates

**(a) THE VALUE — PASS.** At every one of the 64 nodes the blend differs from the argmin by less
than the table's own resolution floor. Worst `1.499839e-5` Ha at `twist_R2.7_t120` (the binding
node, as the derivation predicts), against the floor `7.090642e-4` — **47× under**.

**(b) THE FORCE — PASS**, 518 geometries, `h = 1e-5`, bar `1e-8` relative:

| class | geometries | worst relative | worst translation residual |
|---|---|---|---|
| the map's 64 nodes | 64 | `1.197931e-10` | `2.818926e-18` |
| LIQUID-2's RECORDED handover geometries | 32 | `3.066621e-10` | `2.602085e-18` |
| hydrogen permutations of those | 96 | `3.066621e-10` | `4.249187e-18` |
| molecule exchanges of those | 32 | `3.066621e-10` | `2.561679e-18` |
| contact TIES | 6 | `4.707751e-12` | `1.085411e-19` |
| periodic crossings (one box each axis, the wrap INSIDE the difference loop) | 288 | `3.074640e-10` | `3.903128e-18` |

The forces sum to zero everywhere (`4.249187e-18` worst) and a pair read across a face is the
same pair to `2.693360e-13` relative. The ties are the map's own two plus one bisected on each
of CT-3's four G-A0 separations; the worst tie residual is `8.881784e-16` bohr, so they are ties
and not near-ties. The handover geometries are LIQUID-2's own worst cases, dumped by re-running
the 1× argmin arm (`ct3/smooth/handovers.json`) — the class the note said the map's nodes would
miss, and the class where the worst force error actually lands.

**(c) THE CONTINUITY — PASS.** CT-3's own G-A0 sweep, the donor turned through a full turn at
`linear_R2.7/2.9/3.1/3.4`, read at TWO resolutions — because the largest step between adjacent
samples of a SMOOTH function is `O(dθ)` and halves when the samples double, while a JUMP is the
same size however finely it is approached. That ratio is the measurement of continuity; the raw
step at one resolution is not.

| | 3,600 samples | 7,200 samples | ratio |
|---|---|---|---|
| the BLEND's largest step | `3.890611e-5` Ha | `1.945313e-5` Ha | **`2.0000`** |
| the ARGMIN's largest JUMP | `1.573326e-5` Ha | `1.502985e-5` Ha | **`1.0468`** |

The blend halves exactly: it is continuous, and its largest step is the sweep's own resolution,
18× under the table's floor `7.090642e-4`. The argmin's jump does not shrink, which is what a
discontinuity is — and that leg is the control on the first, because a sweep too coarse to
resolve a handover would let the blend pass for the wrong reason. The argmin's largest STEP on
the same sweep is `3.890611e-5`, identical to the blend's: the two rules share the smooth
background and differ only in the jump.

**(d) THE DRIFT — PASS.** The table above.

### The step, chosen honestly (`ct3/smooth/nve.json`, `dry: true`)

NVE — thermostat OFF — from ONE settled checkpoint (all four arms report the same checkpoint
digest, `1149619867449410921` over 20,626 bytes, so they are shown to have started from one
state rather than four that resemble each other), at 1×, 2×, 4× and 8× the tables' step over
EQUAL physical durations of `0.1` ps. The previous sweep varied the duration WITH the step,
which the second review named; this one does not. Durations agree to `1.043e-4` ps against the
coarsest arm's own frame of `2.085e-4` ps, which is the finest four integer frame counts can be
made to agree.

| × the tables' step | dt, fs | frames | ps | energy fluctuation RMS, Ha | drift / ps, Ha | O–O first peak, bohr | bonds per water |
|---|---|---|---|---|---|---|---|
| **1** | 0.026063 | 3,837 | 0.10000 | **1.601946e-6** | **4.696827e-5** | 6.25 | 1.41955 |
| 2 | 0.052126 | 1,918 | 0.09998 | 6.397148e-6 | 1.919698e-4 | 6.25 | 1.42041 |
| 4 | 0.104252 | 959 | 0.09998 | 2.559353e-5 | 7.784843e-4 | 6.25 | 1.41953 |
| 8 | 0.208504 | 480 | 0.10008 | 1.027272e-4 | 3.158647e-3 | 6.25 | 1.42109 |

**The drift is now QUADRATIC in the step, and that is the whole point.** Each doubling
multiplies the drift per ps by `4.09`, `4.06`, `4.06` and the fluctuation RMS by `3.99`, `4.00`,
`4.01` — `(ω dt)²`, a symplectic integrator's own error. This note's opening observation was
that a **32-fold** change of step moved the argmin's drift by a factor of **6**, and not
monotonically, which is what said the drift was in the SERVED ENERGY and not in the integration.
It is now in the integration, where it belongs. That is a second, independent confirmation of
the diagnosis, and it is a stronger one than the ratio in gate (d), because it is a SHAPE and
not a single number.

**The rule picks `x1` — the tables' own step, `1.077481` au = `0.026063` fs.** The rule was: the
largest step whose drift per ps is within `2x` the `1x` run's and whose observables agree with
the `1x` run within their own spread. `x2` is already `4.09x` the `1x` drift, so nothing above
`1x` survives the first clause. The observables do not discriminate: the O–O first peak is
`6.25` bohr on every arm (the RDF's declared `0.1`-bohr bin is coarser than any difference
between them) and the bond count spans `1.41953` to `1.42109`, a spread of `7.5e-4`, at which
resolution `x2` and `x8` miss `x1` and `x4` does not — noise, not signal, and it changes no
verdict. **There is no free step under the smooth rule**, and the provisional `8x` of
`liquid2/gate_provisional_8x/` is not it.

**Three fences on this screen, in its own words.** (i) It is a SCREEN and every file says
`dry: true`; it chooses a step and reads nothing about water. (ii) The settling is the screen's
own 1,000 frames = 26 fs, far short of LIQUID-2's equilibration criterion of 7,100 frames, so
the arms run at `488` K and read `1.42` bonds per water and an O–O peak at `6.25` bohr — none of
which is a statement about liquid water, and all of which is the same for all four arms, which
is what a step comparison needs. (iii) `0.1` ps is a declared duration, not a converged one;
what it is long enough for is the drift RATE, which is what selects the step.

### What is now paid

1. **The smooth serving rule, built and gated as above. PAID.** `CtServe::Blend`, β derived,
   gates (a)–(d) all PASS, `ct3/smooth/gate.json` `"admitted": true`, and eight unit tests in
   `tests/ct3_smooth.rs` including a cheap standing 54-water drift guard.
2. **The step sweep re-run under it. PAID as a labelled screen** — the table above. It is NVE
   from ONE settled checkpoint over EQUAL physical durations, which the previous sweep was not,
   and it selects `x1`.
3. LIQUID-2's gate re-run at the step the rule selects. **Still owed** — this lane chose the
   step; it did not re-run LIQUID-2.
4. The labelled screen logging cross-unit potential energy and bond count through one settling.
   **Still not run.**
5. **β's derivation needs the shortest-to-second-shortest separation at each of the map's 64
   nodes. PAID** — extracted, in `ct3/smooth/beta.json`, with every node's row and its own
   requirement.

### Two fences this campaign owes its own reader

- **The box is `Boundary::Periodic`, and this lane's first pass left it open.** LIQUID-2 sets
  the boundary at the END of its own door, after the law and the step; a runner that rebuilds
  the box has to set it too. The channel-6-off control then read `5.034204065e-6` where the
  record reads `4.719994550e-6` — a 6.7 % miss that looks like nothing and is a different
  dynamics. The control is re-measured against the record for exactly this reason, and it is
  what caught it.
- **The handover counter is not bit-identical to LIQUID-2's.** Re-running the 1× argmin arm
  reproduces the TRAJECTORY exactly — drift peak `6.584681837e-3` against the record's
  `6.584681837e-3`, worst single jump `2.012547255e-3` against the record's
  `2.012547255e-3` — but this counter registers 3,762 handovers where LIQUID-2's registered
  3,803. The difference is in the two counters, not in the arm, and it was NOT chased down. The
  gate needs the worst geometries and the worst one is the same one; nothing here rests on the
  count.
