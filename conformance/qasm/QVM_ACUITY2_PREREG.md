# QVM-ACUITY-2 — the dictionary of views: does the search pick the cheapest method per circuit without being told? PREREGISTRATION

*Frozen 2026-09-23, committed alone before any code. QVM-ACUITY-1 (`QVM_ACUITY1_RESULTS.md`,
branch (a)) showed the three moves are sound and certified on Clifford+T — and that on that
tier the search re-derives a known partition, the budget buys little because the branch
terms do not decay, and the exponent is the literature's. The general QVM's claim is not
"stabilizer rank done carefully"; it is that ONE procedure, given a circuit and an acuity,
locates the closed view under which the computation is cheapest — tableau, tensor, sum —
without being told which family the circuit belongs to, prices it before running, and does
only enough of the hard part. That is what today's state of the art does by hand (stim for
Clifford, quizx for Clifford+T, MPS/tensor contraction for low entanglement, dense vectors
for small `n`). This campaign tests the claim where the answer differs by family.*

## 1. The dictionary of views, each with its closure test and its price

| view | closure test (searched, not declared) | price before running | certificate |
|---|---|---|---|
| **TABLEAU** | the closure test of `holon::sector` (each gate's unitary conjugated through the Pauli group: one term + tracked phase ⇒ closed) | `O(n² · gates)`, exact | exact |
| **SUM** (stabilizer rank) | the T-gates the tableau view splits on, in the observable's light cone | `expected_branches(t_eff)` (ACUITY-1's price) | the certified remainder of `holon::acuity` |
| **MPS** (bond dimension) | a PROBE at cadence: the circuit run at bond caps `χ ∈ {2, 4, 8, 16}` on a `2n`-gate prefix, the discarded weight per cut recorded; the view is closed at `χ` if the discarded weight is under the acuity's share | `O(n · gates · χ³)` at the probe's `χ` | the summed discarded weight, an upper bound on the 2-norm error (Verstraete–Cirac) |
| **DENSE** | always closed for `n ≤ 24` | `2^n · gates` | exact |

The search runs every closure test, prices every closed view, and SELECTS the cheapest whose
certificate reaches the acuity. The selection is printed BEFORE the run:
`select: view=<V> price=<P> alternatives=[...]`.

## 2. Instances — four families, mixed and UNLABELLED to the search

Twenty instances per family, `n ∈ {12, 16, 20}`, the observable one amplitude or one 4-qubit
marginal, acuity RELATIVE: `ε ∈ {10⁻¹, 10⁻²} × 2^{−n/2}` for amplitudes and `× 2^{−4}` for
marginals (ACUITY-1's correction).

- **C** Clifford only, depth `20n`. Cheapest known: tableau.
- **T** Clifford + `t ∈ {12, 16, 20}` T-gates, depth `20n`. Cheapest known: the sum.
- **L** brickwork nearest-neighbour circuits, depth `≤ 6`, with `t ≥ 40` T-gates. Cheapest known:
  MPS (entanglement bounded by depth; the T-count kills the sum).
- **D** dense: `n = 12`, depth `20n`, `t ≥ 40`. Cheapest known: the dense vector.

Referee on every instance: the dense vector (`n ≤ 20`) — the search never sees it.

## 3. Stakes, each with its kill

- **S1 — the right view, unlabelled.** On each family the search selects the view the family's
  cheapest known method uses, on `≥ 18` of 20 instances; the selection is printed before the
  run. **Kill:** a family where the selection is right on fewer than 15 of 20, with the
  offending prices printed — the search does not locate the cheap part, and the general
  claim is refused on this dictionary.
- **S2 — the price is honest.** For every instance, the selected view's measured wall is within
  `2 ×` of the best wall among the four views run by hand (all four are run on every
  instance where they fit in 60 s; one that does not is priced as `> 60` s). **Kill:** a family
  where the selected view's wall exceeds `4 ×` the best on more than 5 of 20.
- **S3 — the certificate never lies.** On every instance, `|value − referee| ≤ certificate ≤ ε`
  for the selected view (the tableau and dense views' certificate is `0`; the sum's is
  ACUITY-1's remainder; the MPS view's is its discarded weight). **Kill:** one violation — the
  bound is wrong; nothing is read until repaired.
- **S4 — the budget bites where the terms decay.** On family L, the MPS view at the acuity's
  `χ` against the MPS view at `χ = 64` (the "full" method): wall ratio `≤ 0.25` at `ε = 10⁻¹`
  and `≤ 0.5` at `10⁻²`, with S3 holding. On family T the sum's executed fraction is reported
  as in ACUITY-1 and no stake is placed on it (ACUITY-1 measured that it does not decay).
  **Kill:** the L ratio above `0.75` at `10⁻¹` — the decay the claim relies on is not there.
- **S5 — the comparison class, stated.** Beside every row: stim's wall on family C (the
  reference tableau engine), the full stabilizer-rank sum's wall on T, the `χ = 64` MPS wall on
  L, and the dense wall on D. The claim is that ONE procedure lands within `2 ×` of each
  hand-picked method on its own family; no claim of beating any of them is made.

## 4. Plants, each of which must fire before any instance is read

| plant | must |
|---|---|
| **PQ-1** | a Clifford-only circuit presented with a T-gate that the light cone removes: TABLEAU selected, `t_eff = 0` printed |
| **PQ-2** | family L with the brickwork's depth raised to `20n` (entanglement saturates): the MPS probe's discarded weight exceeds the acuity's share at every `χ ≤ 16` and the view is NOT selected — the probe is a measurement, not a family label |
| **PQ-3** | the MPS certificate on a state with known Schmidt spectrum: the discarded weight is never below the true 2-norm error and within `4 ×` of it |
| **PQ-4** | a wrong price planted on one view (its predicted cost halved): the selection flips on the instances where that view was second — and S2 then catches it on wall |
| **PQ-5** | the selection with the family labels REVEALED equals the selection without them on every instance (the search used no label) |

## 5. Branches

- **(a)** S1–S5 met → the general QVM's claim is measured on a dictionary of four views: one
  procedure, no labels, within `2 ×` of the hand-picked state of the art on each family, with
  a certificate; banked in BENCHMARKS.md and the stance; the next freeze widens the
  dictionary (matchgate / fermionic Gaussian, Pauli propagation for noisy expectations, the
  GPU fold as a device class) and moves off the referee (`n > 20`).
- **(b)** S1 killed on one family → the search's closure test for that view is the fault;
  named, repaired, re-frozen; the other families' rows stand.
- **(c)** S2 killed → the price model is wrong where it fails; the measured walls replace it
  and the campaign is re-read against them.
- **(d)** S4 killed → the budget does not bite on the family it was expected to; the finding
  is the entanglement structure of family L at this depth.
- **(e)** a plant fails or S3 is violated → nothing is read.

## 6. Cost

Eighty instances, four views each, under 60 s a view: under three hours on cores 21–27. The
MPS view is `q8-mps`'s real-tensor MPS applied to a circuit (a gate-by-gate TEBD with SVD
truncation; the crate carries DMRG and real-time dynamics, so the door exists) — or, if its
complex-amplitude path is not there, a dense-free MPS written for this campaign with the
same discarded-weight certificate and gated against the dense referee.

## 7. What this does not test

Beating any named tool; `n > 20` (no referee); noise; matchgates and Pauli propagation
(named as the next freeze); the GPU.

---
witness: `Closed` (Object.lean), `tableau_not_closed_under_rotation` and `tableau_closed_under_hadamard` (Stabilizer.lean) for the TABLEAU view's closure; the MPS view's certificate is Verstraete–Cirac's discarded-weight bound, credited; S1–S5 are measured gates
**misfits:** M-PLANT-OBS, M-PLANT-SECTOR, M-CHEAPER-THAN-ITS-PRICE, M-TRUNCATION-AS-ERRORBAR, M-DEVICE-CLASS, M-PLACEMENT-LOTTERY, M-HOMOG, M-PARITY-PROTECT, M-VACUOUS-SUCCESS, M-FLOOR-UNSTAKED, M-MAINTENANCE-LENS, M-COND-PROBE, M-STALE-INSTRUMENT — contacted by keyword, cited.
Carrier-sector statement (M-PLANT-SECTOR): every plant names its carrier (a Clifford circuit with a removable T, a saturated brickwork, a state with a known Schmidt spectrum, a planted price, the label-revealed run), and the sector the plant acts on is nonzero in that carrier by construction.

---

### Notes on building

*Appended 2026-09-23 by the build (`holon::views`, `qvm_acuity2`, `tests/qvm_acuity2_plants.rs`,
`qvm_acuity2_sweep.py`) after the code existed and the campaign had run. Nothing above is moved.
Items 1–2 were repaired in `QVM_ACUITY2_AMENDMENT_1.md`, committed alone before the campaign seed
ran. The rest are where the frozen text and the machine disagree, named so the reading is not
quietly adjusted later. The read is in `QVM_ACUITY2_RESULTS.md`.*

1. **The MPS probe as frozen cannot see depth.** A `2n`-gate prefix is about 1.3 brickwork layers
   and discards nothing, so the linear projection is `0` and the probe closes at `χ = 2`.
   - PQ-2's own carrier: the probe closes at `n = 12, 16, 20`, where the whole circuit at `χ = 2`
     certifies `2.0–3.4`.
   - On the campaign, graded beside the amended probe: S3 is violated on 52 of the 60
     non-Clifford instances.
   - So a plant fails under the frozen text, which is branch (e). Amendment 1 repeats the probe at
     cadence `2n` through the whole circuit.
   - The frozen probe's other flaw: a linear projection of an entanglement that saturates is not
     a bound of any kind.
2. **"The summed discarded weight" does not bound the 2-norm error.** It bounds the squared error.
   `Σ√w` does bound the 2-norm error, by the triangle inequality, given canonical form and no
   renormalisation. PQ-3 reads it at `1.00–1.83×` the true error.
3. **The label for family T is wrong at this `n`.** The sum is not the cheapest known method at
   `n ≤ 20` and `t ≤ 20`: the `2^n` vector is, on 15 of 20 instances, as ACUITY-1 had already
   measured (the crossing near `t = 24` at `n = 20`).
   - S1 therefore stakes the search against a label rather than against the walls, and is killed
     on T while S2 shows those picks were the fastest by hand.
   - A marginal costs the sum `2^{|L|−4}` legs, so no marginal can ever label T correctly at these
     depths.
4. **Family L cannot show decay.** A CX brick has operator Schmidt rank 2, so six entangling
   layers cap every bond at 8. A `χ = 64` cap then costs exactly what the acuity's `χ` costs
   (S4 ratio 1.01).
   - Depth counted over ALL layers could not carry `t ≥ 40` at `n = 12`, so "depth ≤ 6" had to
     mean entangling layers.
   - The decay S4 wants needs bricks of operator Schmidt rank 4, or a relative acuity loose enough
     to license a truncation. `10⁻¹·2^{−n/2}` on an amplitude licenses none.
5. **The prices omit the search.** Every closure test, run on every instance, costs `0.7–8` ms
   (median). The views it selects cost `40 µs – 2 ms`, so the procedure is `112×` stim on C.
   - S2 and S5 as written compare view walls, and both are graded that way.
   - A freeze that means "one procedure within 2×" must price and stake the search itself.
6. **§1's MPS price `n·G·χ³` is the worst case.** The measured cost is per swap-routed two-site op,
   and below `χ ≈ 36` it is dominated by per-op overhead (`c_m0 = 7.7 µs`, against
   `c_m3·χ³ = 0.7 µs` at `χ = 16`).
7. **S3 as written is violated by any `f64` referee against an exact view.** The inequality reads
   `|value − referee| ≤ 0`. It is read with the referee's `10⁻¹²` floor.
8. **"All four views by hand" needs a rule for a view that is not closed.** The rule used: the
   tableau is not run on a T-bearing circuit; the MPS view by hand is the smallest
   `χ ∈ {2…64}` that reaches `ε`; the sum is capped by a killed child process.
   - Without the child process the sum's construction could not honour the 60 s cap.
   - Without the `χ` search the MPS view has no by-hand wall on the deep families.
