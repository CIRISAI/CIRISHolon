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

1. The smooth serving rule, built and gated as above. **Not built.**
2. The step sweep re-run under it. **Not run.**
3. LIQUID-2's gate re-run at the step the rule then selects; the records now under
   `liquid2/gate_provisional_8x/` are at a provisional step and their README says which of
   their numbers survive it being unchosen.
4. `beta`'s derivation needs the shortest and second-shortest contact separation at each of
   the map's 64 nodes. Those geometries are in `ct1/sector_*.json` and `ct2/gd0_*.json`; the
   number has NOT been extracted here.
