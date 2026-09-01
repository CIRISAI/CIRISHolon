# Pre-registration — DE4-TABLE: the four-body (O,H,H,H) surface, tabulated once and leased

*Frozen 2026-09-01, committed ALONE. The four-body term `dE4 = E_FCI(OH3) - E_MBE3(OH3)`
is currently solved FROM SCRATCH inside the trajectory loop, four electronic-structure
solves per quadruple per force evaluation. This freezes the domain, the grid, the
generation route and the acceptance gates for tabulating it, and it does so through the
folded pipeline rather than through a seventh hand-rolled generator (WORKBENCH_FSD.md
clause WB-8.7: a per-composition generator is a DRY residual that must be justified or
avoided, and the dE4 incident is that clause's founding case). Three of this document's
inputs are MEASUREMENTS taken before the freeze and reported below with their instrument;
two of them contradict the brief that ordered the work.*

misfits: contacts M-CHEAPER-THAN-ITS-PRICE (the banked price is re-measured on this
machine before the freeze, at a recorded loadavg, in CPU-seconds rather than wall-seconds
so contention cannot launder it; the GRADIENT path is priced separately from the value
path because the gradient path is the one that runs, and the arithmetic
`claimed wall / priced per-unit cost` is gate G3), M-STALE-INSTRUMENT (the brief that
ordered this work named a symbol `de4_point` that does not exist and a price of ~42 ms
that is off by more than an order of magnitude; every number here is re-measured and every
instrument is named with its path, and this freeze is committed alone),
M-PLANT-SECTOR and M-PLANT-OBS (every plant below names its carrier and asserts it nonzero
in the sector the plant acts on, before the plant is scored),
M-PROVENANCE-OVERREACH (the launch header records binary sha256, repo HEAD, build exit
status and tree-dirty state as four separate MEASURED lines, and labels which of them is
an inference rather than a claim), M-IDLE-CALIBRATED-TIMEOUT (the machine is not quiet —
loadavg 45-58 on 32 cores throughout — so no band, budget or grace period is frozen from a
wall-clock reading; the cost model is in CPU-seconds and the wall conversion is done at
launch from measured core availability), M-VACUOUS-SUCCESS (a node that cannot be
variationally guarded must not score, and a continued node is not a solved geometry),
M-EXIT-DISCRIMINATOR (exit reasons ship as a histogram; no `converged: true` field),
M-MAINTENANCE-LENS (the interpolant is not a repair and no rent-clause reading is taken
from it), M-DEVICE-CLASS (generation is CPU-only here; no GPU adoption is claimed and no
bitwise agreement across device classes is asserted), M-PROBE-THE-RESOURCE (workers are
leased through the arena's own probe, which attempts a thread rather than reading a
reported parallelism), M-BUDGET-LAUNDER (budget exhaustion VOIDs a node loudly and never
scores), M-PARITY-PROTECT (the serpentine traversal's sum-parity rule is named as the
defective one and the reflected rule is used instead; the parity argument is about
traversal order only and carries no physics), M-LOOP-BLIND (the trajectory loop is the
consumer, not an instrument; no holonomy or loop quantity is read),
M-HOMOG (the domain is not spatially homogeneous and no local-vs-distant contrast is
claimed), M-ONE-MODEL-DELTA, M-VOLUME-SCALE, M-BARE-CHARGE, M-GAUGE-LAUNDER,
M-COND-PROBE (not otherwise contacted).

---

## What was measured BEFORE this freeze, and what it overturned

Three instruments ran before any grid was written down. They are named here because two of
them changed the design and one of them changed the claim.

**(M1) The price.** `engine/crates/holon-chem/examples/de4_price.rs`, single-threaded, at
loadavg 57.91 on 32 cores, CPU/wall 0.276 measured in-section.

| quantity | measured |
|---|---|
| `de4_ohhh_fci`, 40 witnesses, wall | min 408.7, median 522.0, p90 4533.4, max 8618.4 ms |
| `de4_ohhh_fci`, MEAN wall | 1551.2 ms — the mean, not the median, is what a grid pays |
| of which the 4-centre FCI | median 280.8 ms |
| the GRADIENT path (1 base + 3 forward differences = 4 solves) | MEAN 9837.5 ms |
| grid-like geometries vs witnesses | 1.18x — the domain is DEARER than the held-out set |

The brief's banked figure was ~42 ms for a value and ~3.3 s for a gradient. Measured, the
value's mean is 1551 ms wall / ~428 ms CPU and the gradient's mean is 9838 ms wall. The
banked price was low by more than an order of magnitude, and under
M-CHEAPER-THAN-ITS-PRICE a cost model that cannot close its own arithmetic is a falsifying
check that has fired: **the pre-freeze price model is REPLACED, not adjusted.** The tail is
the cost driver — the mean is 3x the median — so every budget below is stated on the mean.

**(M2) The canonical form is not canonical.** `quaternary::sort_ohhh_internals` sorts the
three O-H distances and the three H-H distances INDEPENDENTLY. Independent sorting is
invariant under S3 x S3 (order 36), not under the S3 (order 6) that acts on a labelled
geometry: relabelling the hydrogen that carries `R2` also moves which pair owns `R23`.
Thirty-six over six is six, so generically SIX distinct geometries are handed one address.
Exhibited on exact inputs by `de4_price.rs`:

```
A: R_OH [1.9, 2.4, 3.0]  R_HH(12,23,31) [2.6, 3.3, 4.1]
B: R_OH [1.9, 2.4, 3.0]  R_HH(12,23,31) [2.6, 4.1, 3.3]
sort_ohhh_internals(A) = sort_ohhh_internals(B)      <- ONE address
dE4(A) = 0.048136170 Ha   dE4(B) = 0.054491264 Ha    <- TWO values, 6.355e-3 Ha apart
```

A table indexed on that function would store one number where two are owed, and the error
would be 6.4e-3 Ha — twenty-six times the (O,H,H) table's own interpolation scale. The
existing test `tests/quaternary.rs::s3_permutation_invariance_is_bit_exact` passes and
does not catch it, because independent sorting is invariant under strictly MORE than the
group in question, and a test that checks invariance never checks injectivity. **This
freeze does not use `sort_ohhh_internals`.** It uses `canonical_ohhh`, the lexicographic
least of the six relabelled 6-tuples, which is comparisons-only (hence bit-exact) and was
demonstrated in the same run to be S3-invariant AND to separate A from B.

**(M3) A box in the six distances is not a box of geometries.** Four atoms have six
internal degrees of freedom, so six coordinates is the right COUNT, but the six mutual
distances are constrained: the three unit vectors from O exist in R^3 only where the Gram
determinant `G = 1 + 2 u12 u23 u31 - u12^2 - u23^2 - u31^2` is non-negative. Measured over
`R_OH [1.2,6.0]^3 x R_HH [0.9,12.0]^3` at 21^6 = 85,766,121 points:

| | |
|---|---|
| not even a triangle through O (`\|cos\| > 1`) | 86.99% |
| triangles, but not embeddable in R^3 (`G < 0`) | 6.48% |
| **real geometries** | **6.53%** |

A six-distance grid would spend 93.5% of its nodes on configurations that do not exist, and
its interpolation stencil would stand almost entirely on them. The brief's proposed domain
`(R1,R2,R3,R12,R23,R31)` is therefore **NOT the domain frozen here.** Replacing the three
H-H distances by the three H-O-H cosines removes the `|cos| > 1` failure by construction
(the cosine IS the coordinate) and leaves only the Gram condition, measured at
`1 - pi^2/16 = 38.31%` of the cosine cube analytically and 40.30% on a 101^3 sample. The
frozen domain is `(R1, R2, R3, u12, u23, u31)`, and S3 still acts on it as a simultaneous
permutation of the three R-axes and the three u-axes, so a grid node maps to a grid node
and the orbit fill is exact.

**(M4) The seam scan.** `engine/crates/holon-chem/examples/ohhh_seam_scan.rs`, six slices,
25 points each, cold and warm solve at every point, carrying the better vector forward.
Reported in full in the results document; the two readings that bind this freeze are:

* **No warm start beat its cold solve on any slice** (most negative `E_warm - E_cold` over
  all slices: -1.19e-12 Ha, i.e. noise). Every corner located is the SURFACE's, not the
  eigensolver's, and the variational bound says so without appeal.
* **The corners are BORROWED.** On the H2-elimination channel `d3[dE4]/d3[E_FCI] = 10.7`;
  on the (H,H,H) channel, `620.9 / 52.5 = 11.8`. The kink is not in the four-centre ground
  state — it enters through a three-body subterm that `E_MBE3` SUBTRACTS. This is a hazard
  the three-body scans did not have to consider and it is stated as a general fact: a
  many-body term inherits every seam of its own lower-order subterms. A scan looking only
  for crossings of the OH3 ground state would have walked past both.

What (M4) does NOT establish, and gate S1 exists to settle: a third divided difference
measured at ONE resolution cannot tell a slope discontinuity from a large smooth third
derivative. A jump makes `d3` diverge as `h -> 0`; smooth curvature leaves it flat. The
scan ran at one `h`. Under the detector-not-verdict rule this is a reason to look, not a
verdict, and the seam record is not written until S1 has run.

---

## Scope, stated before the gates

This table is a **four-body correction on ONE composition, (O,H,H,H), in STO-3G FCI**, and
nothing else. It is exact-in-model, not a prediction of experiment. It does not extend to
(O,O,H,H) or any other tetramer, it does not license a five-body term, and it says nothing
about thermochemistry against nature.

The pipeline it runs through is the deliverable of equal weight. `holon-tables` has been
folded so ONE leased generator serves the 3-axis / 3-atom surfaces and this 6-axis /
4-atom one; the fold's acceptance is that the existing three-body digest is unchanged
bit for bit, which is gate B1.

**Domain, frozen.** Coordinates `(R1, R2, R3, u12, u23, u31)`: three O-H distances and the
three cosines of the H-O-H angles. `R_HH` is recovered as
`sqrt(R_i^2 + R_j^2 - 2 R_i R_j u_ij)` and is never a coordinate.

| axis | range | map | nodes |
|---|---|---|---|
| `R1, R2, R3` | `[0.9, 6.0]` bohr | exponential stretch, `a = 3.0` | 13 each |
| `u12, u23, u31` | `[-1.0, 0.9975]` | linear | 11 each |

`R_HI = 6.0` bohr is `quaternary::R_CUT`, the MEASURED far-field cutoff: `|dE4|` is
4.9e-5 Ha at 6.1 bohr and 1.7e-6 Ha by 9 bohr (`tests/quaternary.rs`). The three-body
table's `R_HI = 15.0` does NOT transfer and is not used. `U_HI = 0.9975` is the
closed-angle fence, `1 - C_LO^2` with the (O,H,H) table's own `C_LO = 0.05`, adopted
because slice 6 of (M4) measured `d3[E_FCI] = 2.46e5` as `u -> 1` — a coordinate collapse,
not a state crossing, and a fence is what the three-body tables already put there.

**Grid, frozen.** Box `13^3 x 11^3 = 2,924,207` nodes. By Burnside the S3 orbit count is
`(13^3 11^3 + 3 * 13^2 11^2 + 2 * 13 * 11)/6 = 2,985,840/6 = 497,640` canonical nodes, of
which the measured embeddable fraction 0.617 leaves **~307,000 real solves**. Storage:
23.4 MB for the full box in f64, 4.0 MB for the canonical wedge.

**This grid is COARSE and is frozen as coarse on purpose.** Thirteen nodes across 5.1 bohr
is far below the (O,H,H) table's 65, and gate T1 below is a real gate that may FIRE. The
alternative was to stake a resolution this machine cannot pay for; the price model (M1) and
the measured core availability (3-4 free cores of 32) put a 15^3 x 13^3 grid at 52.9
core-hours and a 17^3 x 15^3 at 118, against 21.3 for this one. Naming it v1 and letting
its held-out reading DECIDE the successor's resolution is a design; picking the resolution
we wish we could afford is not.

**The continuation rule, frozen.** A node outside the elliptope is not a geometry. It is
filled by scaling the cosine triple radially toward the origin — `(u12,u23,u31) ->
t*(u12,u23,u31)` for the largest `t <= 1` with `G(t) >= 0` — and marked
`VoidReason::NotAGeometry`. Radial scaling COMMUTES with permuting the three cosines, so
the continuation is S3-equivariant by construction and the orbit fill stays exact. A
continued node carries its status in its record and is **excluded from every accuracy
statistic by construction, not by a filter someone has to remember to apply.**

**NOT in scope, stated so it cannot be quietly claimed later:** any composition but
(O,H,H,H); five-body terms; a basis beyond STO-3G; GPU generation; agreement with
experiment; and any claim that the interpolated surface is accurate where gate T1 says it
is not.

---

## Gates

- **G1 — THE FOLD IS BIT-IDENTICAL (engineering, blocking, EXACT)**: `holon-tables`'
  existing 3-axis path, re-expressed through the folded generator, reproduces the
  pre-fold digest EXACTLY over the g1_gate node set at 1, 4 and 8 workers. Measured before
  any four-body node is solved. Kill: any digest difference kills the fold and the old path
  is restored; the four-body table then waits rather than shipping on a pipeline that moved
  the three-body numbers. witness: none (regression, exact equality)

- **G2 — THE PRICE MODEL CLOSES (M-CHEAPER-THAN-ITS-PRICE, blocking)**: before the run is
  accepted, `claimed wall time x measured cores / nodes solved` must land within 2.0x of
  the banked per-node CPU cost re-measured in the same job by an in-job anchor, not by this
  document's numbers. Kill: a factor beyond 2.0x in EITHER direction VOIDs the artifact —
  too cheap means work not done, too dear means the model is wrong and the successor's
  budget is fiction. Both directions are refusals; this is not a one-sided sanity check.
  witness: none (measured)

- **G3 — THE LAUNCH HEADER IS COMPLETE (contract, EXACT)**: the generation log opens with
  binary sha256, build exit status, repo HEAD, tree-dirty count and launch loadavg, each
  labelled MEASURED, and with the inference from sha256 to HEAD labelled as an inference
  rather than asserted. Kill: a missing or unlabelled field refuses the launch (exit 70)
  rather than warning. witness: none (contract gate)

- **S1 — IS THE CORNER A CORNER? (two-sided, three outcomes, pre-committed)**: re-run
  slices 1 and 5 of the seam scan at 49 points (`h` halved) and compare `max |d3[dE4]|`
  against the 25-point reading. A slope discontinuity makes `d3` scale as `J/h^2`, so
  halving `h` must multiply it by ~4; smooth third-derivative curvature leaves it
  unchanged.
  * **(a) ratio >= 2.5** — a real slope jump. The seam record carries `accepted_floor`
    with the floor computed as `J*h_cell/8` from the measured jump, and a written reason:
    the located corners are NOT axis-aligned (slice 1's corner sits at
    `R1=1.944, R2=R3=2.977, u12=u31=-0.33`, a codimension-1 surface in six dimensions that
    no grid line can lie on), so the seam law's "place a grid line on it" branch is
    UNAVAILABLE and the floor is taken deliberately.
  * **(b) ratio <= 1.6** — smooth. The seam record carries `loci: []` with
    `scanned: true`, `accepted_floor: null` is refused by the schema, so it carries
    `accepted_floor` at the measured interpolation scale with the reason "scanned, no
    state crossing located".
  * **(c) 1.6 < ratio < 2.5** — REFUSED-to-classify, a named dead band, reported as such
    and NOT resolved by picking whichever branch suits the grid.
  Kill: none — every outcome is scorable, which is what makes it a gauge. witness: none
  (measured; the decision tree is this prereg's)

- **S2 — THE SEAM RECORD EXISTS (contract, EXACT)**: the artifact carries a `seams` block
  with `scanned: true`, `instrument: ohhh_seam_scan`, and EXACTLY ONE of `loci` or
  `accepted_floor` non-empty, per `s3_mesh/TRIMER_TABLE_SCHEMA.md`. Kill: a shipped
  surface with neither is refused by the validator. witness: none (contract gate)

- **T1 — HELD-OUT FIDELITY ON THE 40 WITNESSES, VALUE**: the table's interpolant, queried
  at the 40 staked witness geometries, is compared against `de4_ohhh_fci` at the same
  geometries. Max and mean `|error|` are REPORTED; the staked band is
  **max error <= 2.0e-2 Ha**, set from the coarse grid and the estimated seam floor, not
  from hope. Nonzero required (two-sided): an error of exactly zero at all 40 would mean
  the witnesses landed on nodes and the gate measured nothing. Kill: max error > 2.0e-2 Ha
  fires T1, and a fired T1 does not kill the pipeline — it kills THIS RESOLUTION, and the
  successor grid is sized from the measured error and the observed convergence rate.
  witness: none (measured)

- **T2 — HELD-OUT SIGN STRUCTURE**: the sign of the interpolated `dE4` must reproduce the
  ab-initio sign at the 40 witnesses: **exactly 11 attractive and 29 repulsive**
  (`dE4 < 0` counts attractive; the banked mean `|dE4|` is 0.111948 Ha and the max is
  0.228401 Ha). This is a stricter and more meaningful gate than the value band, because
  the eleven attractive geometries are the ones a fitted surface got wrong, and getting the
  count right by luck requires eleven coincidences. Kill: any miscount fires T2 and is
  reported at full prominence with the offending geometries named. witness: none (measured)

- **T3 — HELD-OUT FORCE ACCURACY**: at the same 40 geometries the interpolant's analytic
  gradient is compared against a central finite difference of `de4_ohhh_fci` at
  `h = 1e-4` bohr. Staked band: **max component error <= 5.0e-2 Ha/bohr**, REPORTED
  either way. The value band and the force band are separable: a table can pass T1 and
  fail T3, and the trajectory loop consumes the FORCE. Kill: exceeding the band fires T3
  alone and does not take T1 down with it. witness: none (measured)

- **C1 — DERIVATIVE CONTINUITY ACROSS THE CANONICALISATION BOUNDARY**: the interpolant is
  evaluated on both sides of a relabelling boundary (`R1 = R2`, and `u23 = u31`) and the
  gradient jump is measured. Because the stored function is the restriction of an
  S3-symmetric function and the box is orbit-filled, the jump must be at roundoff:
  **<= 1e-12 Ha/bohr**. Kill: a jump above that means the orbit fill is wrong, which is a
  correctness failure and not a resolution failure. witness: none (measured)

- **S3 — S3 INVARIANCE IS BIT-EXACT (EXACT)**: the table's `eval`, given one geometry
  presented under all six hydrogen relabellings, returns the identical value bit for bit
  and the correspondingly permuted gradient bit for bit. Not "within a tolerance" —
  identical, because `canonical_ohhh` is comparisons-only. Kill: any bit difference.
  witness: none (regression, exact equality)

- **B1 — BOUNDARY DECAY**: the interpolated `|dE4|` at `R_max = 6.0` bohr is
  **<= 1.0e-4 Ha** over the whole boundary shell, consistent with the measured 4.9e-5 Ha
  at 6.1 bohr that set `R_CUT`. Kill: a boundary value above 1e-4 Ha means the cutoff
  severs a live interaction, which is the defect the three-body campaign's any-side cutoff
  had. witness: none (measured)

- **R1 — THE REFEREE ROUTE STAYS**: `de4_ohhh_fci` is retained as the second route and is
  not deleted when the table is wired in. A staked set of at least 20 spot geometries,
  drawn from OUTSIDE the 40 witnesses, is solved both ways and agreement is REPORTED. Two
  routes, one model — the table is the fast path and the direct solve is the referee, and
  the trajectory loop keeps a flag to run the referee. Kill: none (this is a contract, and
  its violation is the absence of the second route). witness: none (contract gate)

---

## plants (carrier and sector per M-PLANT-SECTOR)

Each plant's carrier is asserted nonzero in the sector the plant acts on before the plant
is scored; a plant on an empty sector VOIDs. A missed plant VOIDs.

- **(i) the colliding-canonical-form plant**: building the table addressed by
  `sort_ohhh_internals` instead of `canonical_ohhh` must move the T1 held-out error by
  orders (carrier: the measured 6.355e-3 Ha separation between geometries A and B of (M2),
  asserted nonzero in the sector of geometries whose H-H triple is not sorted the same way
  as their O-H triple — that sector is non-empty and A/B are exhibited members of it).

- **(ii) the continuation plant, both directions**: a node outside the elliptope filled by
  the S3-equivariant radial scaling must leave S3 invariance EXACT (gate S3 still passes),
  AND a deliberately non-equivariant continuation — scaling only `u12` — must BREAK it
  (carrier: both deviations; the second is asserted nonzero because the orbit of a generic
  outside-elliptope node has six distinct members). Neither half alone is evidence.

- **(iii) the swapped-subterm plant**: serving `E_MBE3` the (H,H,H) trimer surface where
  the (O,H,H) water surface belongs must move the stored `dE4` beyond the T1 band
  (carrier: the energy shift, asserted large because the two surfaces differ by more than
  0.1 Ha at water geometry).

- **(iv) the corrupt-node plant**: a single bit flipped in one assembled node after the
  shard digests are taken must be CONVICTED by the certificate (carrier: the conviction,
  demonstrated firing, and guarded by an assert that the flip actually changed the bits so
  a no-op plant cannot score).

- **(v) the stale-price plant**: presenting the run with the brief's 42 ms per node must be
  REFUSED by gate G2's two-sided arithmetic (carrier: the refusal, demonstrated firing —
  42 ms against the measured ~428 ms CPU is a factor 10.2, outside the 2.0x band on the
  cheap side, which is the side M-CHEAPER-THAN-ITS-PRICE was registered for).

---

## Meaning

All gates => "the four-body (O,H,H,H) term is tabulated once, exact-in-model in STO-3G
FCI, on a six-coordinate S3-quotiented domain whose every node is a realisable geometry or
an explicitly marked continuation; the trajectory loop reads it by interpolation instead of
solving four electronic-structure problems per quadruple per step; the surface's corners
were scanned for before its grid froze and were found to be inherited from its own
three-body subterms; and ONE leased pipeline now generates both the three-body and the
four-body surfaces, with the three-body digest unchanged bit for bit."

NOT claimed: accuracy anywhere gate T1 says there is none; any composition but (O,H,H,H);
any basis but STO-3G; five-body terms; that the coarse v1 grid is the right resolution —
T1 and T3 are expected to be informative about that and may fire; that the borrowed corners
have been REMOVED rather than located and paid for; and that a table whose held-out band
fires is fit for the trajectory loop, which is why R1 keeps the referee route.
