# CARRIER v2 — results

*Stakes: `CARRIER_V2_PREREG.md`, ADMITTED by `Audit/prereg_audit.py` and committed at
`7bd3df0` before a line of the instrument existed. The git order is the check. Logs this
document reads: `carrier_v2_bank.log`, `carrier3d_ladder.log`. Every threshold quoted is
one of the freeze's own constants; where a stake turned out to be wrong it is written up as
a finding and NOT repaired in place.*

---

## 0. THE HEADLINE

### THE FOUR CARRIER EXITS ARE PAID, AND THE COMPATIBILITY CLAIM IS A DIGEST RATHER THAN A PROMISE

* **The 16-atom cap is gone.** Bonds are a sparse ascending pair-index list, `O(N)` where
  the `u128` bitset was `O(N²)`. The new envelope is NAMED rather than hidden: N ≥ 92,682,
  where `C(N,2)` stops fitting a `u32`.
* **Forces and the intervention ledger are recordable**, so `RUNG2_RESULTS.md` §5.3's
  G9b/G9c — marked UNDISCHARGED because "not computable from this artifact" — become
  computable by a reader that never constructs a `Sim`. Both readings are implemented and
  tested; whether they CLOSE on a real run is the successor's physics question and is not
  claimed here.
* **`dims` is measured, not believed.** `dims_declared` is recorded and labelled
  untrustworthy in the type's own documentation; the answer comes from the frames.
* **Bit identity holds on the artifacts of record.** 23 of 23 banked files match the
  manifest; 18 of 18 trajectories round-trip through the new reader back to their pinned
  digest; 18 of 18 are field-identical to the v1 reader on every `f64` bit.

**And the reader was convicted twice by this campaign's own plants before any campaign used
it** (§4). One of those two defects is precisely the silent reinterpretation G1 exists to
catch, and it was sitting inside the code G1 runs through.

---

## 1. WHAT RAN, AND WHERE IT LIVES

| | |
|---|---|
| freeze | `conformance/atomworld/CARRIER_V2_PREREG.md` (`7bd3df0`) |
| format | `engine/crates/holon-lens/src/traj2.rs` (`2beb0e6`) |
| plants P-1…P-3 | `engine/crates/holon-lens/tests/carrier_v2_plants.rs` |
| plants P-4…P-9 | unit tests inside `traj2.rs`, beside the code they exercise |
| bank gates | `engine/crates/holon-lens/examples/carrier_v2_bank.rs` (`6e6d67d`) |
| 3D carrier | `engine/crates/holon-md/examples/carrier3d.rs` (`990dfd6`) |
| run state | `conformance/atomworld/CARRIER_V2_RESUME.md` |

---

## 2. THE FORMAT GATES

| gate | reading | verdict |
|---|---|---|
| **G1a** carrier identity | 23 of 23 banked files match `census_traj_manifest.sha256` | **PASS** |
| **G1b** reader identity | 18 of 18 trajectories re-serialise to their pinned digest | **PASS, EXACT** |
| **G2** field identity | 18 of 18; all 11 header fields, and per frame the index, time, temperature, bond SET and all 6 `f64` per atom, compared on BITS | **PASS, EXACT** |
| **G3** version discrimination | a v2 file into the v1 reader is refused naming the magic; a v1 file into the strict v2 path is refused naming the version; exit code 4 both ways | **PASS** |
| **G4** the cap is gone | N = 402 and N = 4096 round-trip with the last pair index set; the writer refuses at 92,683 with exit 7 | **PASS, EXACT** |
| **G5** dims measured | 18 of 18 match `CENSUS_RESULTS.md` §14.4 | **PASS** |
| **G6** the two ledgers computable | both implemented, both tested with planted values, both refusing `None` on a file with no ledger | **PASS** |
| **G7** the plants fire | 9 of 9, and two convicted the reader — §4 | **PASS, and it cost two defects** |

### 2.1 G1a and G1b are two facts, and the freeze's single G1 was one word short

The freeze wrote G1 as one gate. Running it exposed that it is two: **the bank is
unchanged** and **the reader reinterprets nothing** fail for opposite reasons and call for
opposite responses. A single combined count could not tell a rotted artifact from a
defective instrument. They are reported separately and both pass; this is a refinement of
the freeze's gate, not a weakening of it, and the freeze's own text already carried the
rule it follows — "a single mismatch refuses the READER … it never reports the banked file
as defective".

Five of the twenty-three pins are RUN LOGS, not trajectories. They are digested and not
parsed, which is why G1b/G2/G5's denominator is 18 and G1a's is 23.

### 2.2 G5, and an independent cross-check nobody asked for

```
file                                        dec meas     span z    max|dz|    flatness  frames
hydrogen/seed_0x…5421 … 5428 (8 files)        2    2     0.0000     0.0000     0.000e0   20000
fenced/seed_0x…5421 … 5428   (8 files)        2    2     0.0000     0.0000     0.000e0   20000
de4_off/seed_0x…5422                          2    2     0.0000     0.0000     0.000e0   20000
de4_on/seed_0x…5422                           2    3    22.9546    11.4899    3.560e-1   20000
```

Seventeen files read **exactly** zero — not "below a threshold", the `f64` zero — because
the covariance is accumulated about the first sample rather than about a computed mean
(§3.2). One file declares a dimensionality its data does not carry, and the reader reports
the disagreement without repairing the header.

**The cross-check.** `max|dz|` is `CENSUS_RESULTS.md` §14.4's own statistic, and §14.4
publishes 11.4899 for `de4_on`, measured by a different instrument in a different lane.
This reader reads **11.4899**, difference 0.0000. That is corroboration of the reader
against a prior measurement rather than against its own expectation, and it was not
required by any gate.

### 2.3 The sha256 is this campaign's own, and is gauged before it is used

`holon-lens` has zero dependencies and that is load-bearing — `RUNG2_RESULTS.md` §1
verified its G1 out of band for exactly this reason. So the digest is implemented in the
example rather than imported, which makes it an instrument this campaign has to gauge
rather than a library it may cite. It is checked against three published vectors, the
million-`a` vector (the only one that exercises the streaming buffer and a length field
past 2¹⁶ bits), and a chunked-versus-one-shot agreement check — all **before any artifact
is read**. A digest that agrees with nothing is not evidence.

---

## 3. THE FORMAT, AND THE THREE CHOICES THAT ARE NOT THE OTHER CHOICE

### 3.1 Sparse bonds, not a wider bitset

The bond set is a subset of the neighbour list, which is `O(N)` at a declared cutoff. At
N = 402 the list costs ~1.6 kB per frame against 10.1 kB for a bitset. But the size is the
smaller half of the argument: **a wider fixed bitset is the wrong KIND of answer.**
`Boundaries.no_fixed_width_carrier` proves no fixed-width representation carries an
unbounded index set, so widening from 128 bits to 4096 MOVES a refusal without removing
one. This format's refusal is at the `u32` pair index — N ≥ 92,682 — and it is written into
the header documentation, tested on both sides of the boundary, and given its own exit
code.

### 3.2 The covariance is accumulated about the first sample

`measure` reports per-axis spans AND the covariance's eigenvalue spectrum, because a scene
locked to a plane that is not axis-aligned has three nonzero spans and rank two. The
eigensolve is an in-house 3×3 cyclic Jacobi with a fixed 64-sweep budget: a library
eigensolver would be a device-class dependence (`M-DEVICE-CLASS`) hiding inside a number
that looks like geometry.

Accumulating about the FIRST SAMPLE rather than the running mean is what makes the
seventeen locked trajectories read exactly zero. If every `z` is bit-identical then every
`z − z₀` is exactly zero, so the covariance's third row and column are exactly zero and so
is the third eigenvalue. About a computed mean, rounding would survive in that row and a
symmetry-locked scene would report as faintly three-dimensional — which is the reading
`CENSUS_RESULTS.md` §14.4 needed to be exact.

### 3.3 The flatness threshold's margin is TWO decades, and that is measured

`Measured::TOL = 1e-6` separates a live principal axis from a locked one. Its margin is not
what it looks like:

* an **axis-aligned** lock reads flatness exactly 0.0;
* an exactly planar but **TILTED** cloud reads **8.0e-9**, because its null direction is a
  cancellation in the covariance and lands near `sqrt(eps)`.

So the threshold sits about 100× above the instrument's own floor, not 10⁸×. A scene whose
genuine out-of-plane extent is below 1e-6 of its in-plane extent is **beyond this
instrument's reach, not measured flat by it**, and the ratio is printed on every read so no
reader has to take the threshold's word for it. The floor is bracketed on both sides by a
test, so a change to the accumulation that moved it toward the threshold fires.

### 3.4 What the format deliberately does NOT carry

**Per-cell fractional occupancy is not a stored field**, and `RUNG2_PREREG_A2`'s
fluid-element representation is served by a derivation instead
(`Trajectory2::mean_cell_occupancy`). Occupancy is a function of the positions and a GRID,
and a grid is a reader's choice: storing it would bake one grid into a file every later
campaign has to live with — the same trusted-declaration failure `dims_declared` exists to
close — and would create a second implementation of a reading that already has one.

The derivation reports the escapees **separately and never folded in**. The walls are soft,
so atoms genuinely leave the declared box; clamping them into the edge cells would
manufacture density exactly where a fluid-element reading is most fragile, and dropping
them silently would make the grid's total disagree with N. Its test asserts the books
balance: grid total × frames + escapees = N × frames.

**The per-atom internal/external force split is not carried.** The recorded force is the
TOTAL, which is what "forces" means and what an integrator uses; the external part's time
integral is in the ledger's `j_ext`. A second per-atom array would double the file for a
distinction nothing downstream has asked for, and that is a stated choice rather than an
oversight.

---

## 4. DEFECTS THIS CAMPAIGN FOUND IN ITSELF

Reported as plainly as the passes, and kept.

1. **`from_v1` SILENTLY FILTERED bond bits above the scene's pair count.** v1 stores 128
   bond bits whatever the scene's size, so a twelve-atom file has 66 real pair bits and 62
   that cannot be pair indices at all. The first lift filtered the set to `0..n_pairs`,
   which reads a self-contradicting file silently, drops the impossible bits, and produces
   a value whose re-serialisation is a DIFFERENT FILE — with every gate green.

   **This is the exact silent reinterpretation G1 exists to catch, and it was inside the
   code G1 runs through.** Found because a plant fixture set bit 119 on a twelve-atom scene
   by mistake. No artifact of record is affected: `bond_bits` only ever sets indices below
   `n_pairs`. But "no artifact we have hits it" and "the reader cannot be fooled by it" are
   different facts, and only the second is a gate. Now refused, naming the offending bit,
   the frame, and the scene's real pair count.

2. **A mid-frame truncation propagated as a parse error.** P-8 stakes that a cut file
   "reads as a short prefix reported INCOMPLETE, never as an error and never padded", and
   the first reader only did that for a cut landing exactly on a frame boundary. Now the
   fragment is dropped and `truncated_frame` reports it — kept SEPARATE from "fewer frames
   than the header promised", because a run killed at a grain boundary and a run killed
   mid-write are different facts and a reader that cannot tell them apart cannot say
   whether the writer was interrupted or the disk filled.

   **v1 files still cannot report this**, and the asymmetry is recorded rather than
   smoothed over: `crate::traj::Trajectory::read` fails on a mid-frame cut and it is a
   banked instrument this campaign does not edit — every digest in the manifest was taken
   through it.

3. **The freeze's G1 was two gates written as one.** §2.1.

4. **A file too short to carry a version exited 3, "a path did not resolve", instead of 4,
   "a format refusal".** `peek_version`'s `?` mapped `UnexpectedEof` onto `TrajError::Io`,
   so a caller handed a truncated or foreign file would be told to check its paths. That is
   the exact failure `M-EXIT-DISCRIMINATOR` names, sitting in the function whose job is to
   discriminate. Found by inspection, not by a gate — the freeze's G3 asked about v1-vs-v2
   and never about a stub. Fixed and tested on both sides: a six-byte file is exit 4 naming
   its size, an absent file is still exit 3.

5. **The occupancy grid and `cells.rs` disagreed about who owns the box's upper face.**
   `mean_cell_occupancy` used a half-open `[0, 1)` test, so an atom exactly on the upper
   face read as an escapee; `cells.rs` deliberately adds "a hair of margin so an atom
   exactly on the upper face lands in the last cell rather than one past it". Two
   conventions for which cell owns a face is how the fluid chart and the neighbour list
   come to disagree about where an atom is, with only one of them reported. Now matched to
   the engine, with a test placing atoms exactly on both faces and just past one.

6. **The ladder collected the LEASED worker count and never printed it.** §4.1 of the freeze
   requires reporting "the worker count the pool actually LEASED (not the count requested —
   `M-PROBE-THE-RESOURCE`)". `run_rung` obtains it and the printed row has no column for
   it. The omission changes no number: the ladder's counter wraps the SERIAL executor, and
   term counts are worker-count invariant by `holon-md/tests/bit_identity.rs` — that is the
   crate's whole guarantee. But a required disclosure field was collected and dropped, and
   the run of record does not carry it. Recorded rather than back-filled, because
   re-running the ladder to add a column would cost the O–O curve again and would produce a
   second run of record for one field.

   The same function also **leases workers and drops the pool without releasing the
   leases**. Nothing observes the imbalance — the arena is owned by the pool and dies with
   it — but the lease discipline says leases are paid, and `WorkerPool::retire` is the call
   that was not made.

---

## 5. THE PLANTS

All nine fire. Each names its carrier and the sector it must be nonzero in; each asserts
not only THAT the gate fires but WHERE, because a gate firing in the wrong sector is a
defect in the instrument wearing a pass.

| plant | carrier | must | result |
|---|---|---|---|
| **P-1** flipped byte | copy of a v1 file, at a computed offset (frame 3, atom 5, `pos[0]`'s low mantissa byte) | digest mismatch, disagreement confined to a position | **PASS** |
| **P-2** transposed header | a mutant reader taking `dims` and `substeps` in the opposite order | G2 fires naming BOTH fields and nothing else | **PASS** |
| **P-3** dropped bond | one frame's bitset losing its highest legal bit (65) | fires ONCE, on that frame, in the bond sector | **PASS** |
| **P-4** staked z-span | synthetic scene with a 7.5 bohr z span | `span[z]` reads 7.5 exactly | **PASS** |
| **P-5** truly planar | synthetic scene, every z identical | span, flatness EXACTLY 0.0; dims 2 | **PASS** |
| **P-6** the declaration lies | `dims_declared = 2` over data spanning z | reports the disagreement, returns the measurement | **PASS** |
| **P-7** planted impulse | a known 0.5 impulse in x | ledger closes AND the raw momentum moved by the planted amount | **PASS** |
| **P-8** truncation | a v2 file cut mid-frame | short prefix, reported, not an error | **FIRED — see §4.2** |
| **P-9** degenerate scenes | N = 0 and N = 1 | round-trip, refuse nothing | **PASS** |

**P-1 is placed by arithmetic, not by `len / 2`.** The first version put the mutation at
the file's midpoint, which is not necessarily in the sector the plant claims to test. A
plant whose location is a guess cannot support an assertion about where it fired.

**P-7 is the control that separates two things a bare tolerance cannot.** A ledger that
closes because nothing happened passes `residual < eps` exactly as well as one that closes
because the accounting is right. So the plant requires both: residual at the floor AND the
raw `ΔP` at the planted magnitude (`M-VACUOUS-SUCCESS`).

---

## 6. THE N-LADDER — F2 FAILED, AND THE REASON IS EXACT

`carrier3d_ladder.log`. Density 0.01486 atoms/bohr³, pair floor 1e-6 Ha, dE₄ ON, four
frames × 64 substeps per rung, all five rungs completing.

```
    N     edge     route     cells    r_cut    W_pair/step  W_trip/step   W_dE4   3D@0
   24    11.73  Complete   1x1x1   22.000          276.0          0.0        0     yes
   48    14.78  Complete   1x1x1   22.000         1128.0          0.0        0     yes
   96    18.62  Complete   1x1x1   22.000         4547.6          0.0        0     yes
  201    23.83  Complete   1x1x1   22.000        17571.0          0.0        0     yes
  402    30.02  Complete   1x1x1   22.000        54899.4          0.0        0     yes
```

| stake | reading | verdict |
|---|---|---|
| **F1** `d ln W_pair / d ln N` over N ∈ {96, 201, 402} | **1.7403** (1.8856 over all five) | **NEITHER** — between the staked 1.35 and 1.80, reported as such |
| **F2** `Route::Cells` at N = 402 with ≥ 3 cells/axis | `Complete`, 1×1×1 | **FAIL** |
| **G9** genuine 3D placement at frame 0 | yes on all five rungs | **PASS** |

### 6.1 Why F2 failed, mechanically, and why no ladder of this shape could have passed it

**`r_cut = 22.000 bohr at every rung, independent of N.** That is the tell, and tracing it
gives the whole result:

* `Sim::list_cutoff` takes the maximum of the three-body radius, the four-body radius and
  the declared pair switch. dE₄'s radius is `DE4_R_CUT = 6.0`, so it is not the binder.
* The pair switch comes from `derive_pair_cutoff(1e-6)`, which **starts at the table's own
  `r_max` and only walks outward**: `if t.u(base).abs() <= floor { r_in = r_in.max(base) }`.
  The curve is already under the 1e-6 budget at its last knot, so `r_in` is the table's
  `r_max` — 20.0 bohr — and `r_cut = r_in + PAIR_SWITCH_WIDTH = 22.0`.

So **the neighbour radius is set by the TABLE, not by the truncation budget I declared.**
Tightening the floor cannot shrink it; the derivation floors at `r_max` by construction.
`cells.rs` needs three cells per axis, i.e. `edge ≥ 3 × 22.0 = 66.0 bohr`, and at this
density that is

> **N ≥ 0.01486 × 66³ ≈ 4,273 atoms** before the `O(N)` route can engage at all.

The ladder's top rung is 402. **No rung of this ladder could have passed F2**, and that is a
fact about the engine's cutoff arithmetic rather than about the ladder's ambition. The
freeze reasoned "in three dimensions it can" engage and did not check the arithmetic against
`r_max`; the check would have taken one line and is written here so the next freeze makes
it.

### 6.2 F1's 1.74 is fully explained, and is not a partial success

The slope is between the staked bands, which the freeze pre-committed to reporting as
NEITHER rather than as a near-pass. Its value is explained exactly:

* at N = 24, `W_pair = 276.0 = C(24,2)` **exactly** — every pair is inside 22 bohr, so the
  cost is the complete sum;
* at N = 402, `W_pair = 54,899` against `C(402,2) = 80,601` — 68% of pairs, because a
  30.02 bohr box has finally grown comparable to the 22 bohr radius.

So the bend from 2.0 to 1.74 is the CUTOFF beginning to bite inside a `Complete` route, not
the cell route engaging. Reading 1.74 as partial evidence for the cell route would have been
exactly wrong, and F2 is the conjunct that says so — which is why the freeze staked the
mechanism separately from the slope (`M-CONJUNCTION-MONOTONE`).

### 6.3 dE₄ fired ZERO times at every rung

`W_dE4 = 0` at N = 24 through 402, over 256 steps each, with the term ON. The four-body
sector costs nothing on these configurations because its 6.0 bohr triple-hydrogen gate never
closes on a freshly placed lattice. This is reported because a cost model that priced dE₄
from the census's 891 evaluations would be pricing a regime this ladder never entered.

### 6.4 G8 — the scissor's price, and the one piece of good news

Extrapolating at the measured top-rung slope 1.7403 from the N = 402 rung:

| target | grid at 100 atoms/cell | W_pair/step | relative to N = 402 |
|---|---|---|---|
| N = 800 | 2×2×2 | ~181,838 | 3.3× |
| N = 6400 | 4×4×4 | ~6,781,801 | 123.5× |

**These are UPPER BOUNDS, not forecasts**, and the instrument prints them as such: the
slope used is the `Complete`-route slope, and the route changes before either target is
reached.

And that last clause is the finding worth carrying forward. The cell route engages at
N ≈ 4,273 (§6.1), and the scissor's own 4×4×4 bar is **N = 6400, which is above that
threshold**. So the successor's target size is on the far side of the route change: at
N = 6400 the box is 75.52 bohr, `75.52 / 22.0 = 3.43`, and the `O(N)` route is live. The
123.5× figure is therefore an over-estimate by an amount this ladder cannot measure, because
this ladder never reached the regime.

**What the successor is owed, plainly:** a second ladder starting at N ≈ 4,000, which is
where the interesting question begins and where this one stops. Alternatively a shorter pair
table — `r_cut` is `r_max + 2.0` and `r_max` is a property of the tabulated curve, so a
curve tabulated to 10 bohr instead of 20 would move the route threshold down by a factor of
~8 in N. That is a change to the physics artifact, not to the ladder, and it is named here
rather than made.

### 6.5 THE PRODUCTION N DOES NOT MEET THE SCISSOR BAR, AND IS NOT PRETENDED TO

Stated here rather than buried beside the trajectories, because it is the first thing the
successor needs to know.

`RUNG2_RESULTS.md`'s bar is **≥ 100 atoms per cell WITH inter-cell transport**. On the
smallest grid that has faces to transport across, 2×2×2, that is `N ≥ 800`. **The production
N is 402, which gives 50.25 atoms per cell on that grid — half the bar.**

It is 402 and not 804 because the freeze says so. §5 of `CARRIER_V2_PREREG.md` permits
production "only at an N the ladder priced", the ladder's rungs stop at 402, and 804 is an
extrapolation rather than a measurement. Producing at an unpriced N because the bar would
look better is exactly the move the freeze exists to prevent, and the cost of obeying it is
this paragraph rather than a quiet substitution.

What the ladder DOES say about the gap: N = 800 is **3.3×** the N = 402 rung in `W_pair`
(§6.4), still on the `Complete` route, and therefore affordable on this host — roughly two
hours per 20,000-frame seed rather than forty minutes, at ~1.3 GB per trajectory. **The next
ladder should include a rung at 804** (268 waters, exact 2:1 stoichiometry) so that the
scissor-meeting size is a priced rung rather than an extrapolated one, and the production
that follows it can be run under the same rule this one obeyed.

---

## 7. WHAT THIS DOES NOT CLAIM

* **Nothing about water, closure, or any tier's admissibility.** This node built a carrier
  and its receipts; it ran no certification, and no admissibility verdict may be read out
  of it.
* **Nothing about the four-body term's physical effect.** `CENSUS_RESULTS.md` §14.4 says
  the comparison that would settle it was compromised by the plane. A 3D carrier removes
  that confound for the NEXT campaign; it does not retroactively fix the old one, and the
  banked `de4_on` run still has the escape it always had.
* **No timing claim.** The host is shared and at a load average near 59 on 32 cores.
  `M-PLACEMENT-LOTTERY` and `M-DEVICE-CLASS` are contacted and NOT discharged; every
  wall-clock number printed by these instruments is labelled contended.
* **No claim that the v2 curves are the banked curves.** The pair tables generated here run
  under `solver_budget = 5000`, and the solver budget is part of an artifact's identity
  rather than a diagnostic. The O–O curve's worst residual is reported on every run
  precisely because the census's own log carries
  `WARNING O-O: worst residual 2.68e-6 exceeds CONVERGED_RESIDUAL 1e-9` — that curve sits
  under every banked trajectory, and a runner printing only timings would reproduce the
  same physics with the warning invisible.
