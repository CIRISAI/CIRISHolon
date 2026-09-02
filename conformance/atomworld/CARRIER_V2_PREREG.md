# CARRIER v2 — the trajectory artifact, and the 3D high-N scene it has to carry

*Frozen 2026-09-02, before a line of the v2 instrument existed, before any v2 file was
written, and before any N-ladder rung was timed or priced. The git order is the check.*

**misfits:** M-VACUOUS-SUCCESS, M-EXIT-DISCRIMINATOR, M-PLANT-OBS, M-PLANT-SECTOR,
M-PROVENANCE-OVERREACH, M-CHEAPER-THAN-ITS-PRICE, M-PLACEMENT-LOTTERY, M-DEVICE-CLASS,
M-STALE-INSTRUMENT, M-VOLUME-SCALE, M-HOMOG, M-UNTESTED-GAP, M-TAG-AS-PROPERTY,
M-NONBIJECTIVE-STEP, M-FIXED-POINT-TRAJECTORY, M-BUDGET-LAUNDER, M-CONJUNCTION-MONOTONE,
M-MAX-OVER-SUCCESSES, M-PRESENTATION-VERDICT, M-BASE-RATE-OMITTED, M-SORTS-NOT-SEPARATES,
M-LOOP-BLIND, M-BARE-CHARGE, M-COND-PROBE, M-MAINTENANCE-LENS, M-PROBE-THE-RESOURCE,
M-IDLE-CALIBRATED-TIMEOUT, M-CACHE-KIND, M-IMPORT-EXECUTES, M-ONE-MODEL-DELTA,
M-PARITY-PROTECT, M-NULL-MISSTAKE, M-POPULATION-CHOICE, M-FINAL-VIEW-COLLISIONS.

---

## 0. WHY THIS FREEZE EXISTS

Two certification campaigns failed on the CARRIER and not on the physics.

* `RUNG2_RESULTS.md` §0: the fluid-element tier is INADMISSIBLE because twelve atoms in
  1.831 × 1.101 nm cannot simultaneously hold ≥100 atoms per cell and move atoms across
  cell faces. Its own table shows the scissor closing — the only grid that transports
  averages **half an atom per cell**.
* `RUNG2_RESULTS.md` §5.3: gates G9b and G9c are **UNDISCHARGED — not computable from the
  artifact**, because the dump "carries positions, velocities, bond bits, time and
  temperature, and no forces and no intervention ledger". `RUNG1_RESULTS.md` §6 wants the
  same two things.
* `CENSUS_RESULTS.md` §14.4: seventeen of eighteen banked trajectories hold `z` at
  placement **bit-exactly for 20,000 frames**. They are declared `dims = 2` and they are
  planar symmetry-locked, so the one arm that escaped the plane was not comparable to
  them and the campaign's one-variable design was defeated.
* `holon-lens/src/traj.rs`: `MAX_DUMP_ATOMS = 16`, forced by a `u128` pair bitset. The
  writer refuses past it, honestly — and a carrier that refuses at 16 atoms cannot serve a
  bar written at 100 atoms per cell.

This node pays those four exits. It builds a format, a 3D high-N generator, and the
receipts; it runs no certification. **This freeze governs the CARRIER's claims only. No
claim about water, about closure, or about any tier's admissibility is staked here, and
none may be read out of this campaign's results.**

---

## 1. THE FORMAT — v2, and what "version" is going to mean

### 1.1 The shape

Little-endian throughout, as v1.

```text
magic        8 bytes   b"HLNTRAJ2"        (v1 is b"HLNTRAJ1" and stays readable)
version      u32       2
content      u32       bitmask: 1 = forces, 2 = ledger. Fixed for the whole file.
n_atoms      u32       no cap below the pair-index envelope (§1.4)
dims_declared u32      RECORDED, NEVER TRUSTED — see §1.3
substeps     u32
seed         u64
n_frames     u64       what the writer INTENDED; the reader counts what is there
dt           f64       placement timestep; NOTHING may derive a duration from it
box_w/h/d    f64 x3
z            u32 x n_atoms
then n_frames frames, each:
  index      u64
  time       f64
  temperature f64
  n_bonds    u32
  bonds      u32 x n_bonds   ASCENDING pair indices, the engine's own enumeration
  atoms      f64 x 6 x n_atoms   x, y, z, vx, vy, vz
  [forces]   f64 x 3 x n_atoms   present iff content & 1
  [ledger]   f64 x 8            present iff content & 2:
             j_ext x/y/z, w_ext hand/thermostat/barostat, ledger_total, l0
```

### 1.2 Three design decisions, and the reason each is not the other choice

**Bonds are a SPARSE ASCENDING LIST, not a wider bitset.** The bond set is a subset of the
neighbour list, which is `O(N)` at a declared cutoff, so the list is `O(N)` where a bitset
is `O(N²)`: at N = 402 the bitset costs 10.1 kB per frame against roughly 1.6 kB for the
list. Wider-fixed-width is also the wrong KIND of answer —
`Boundaries.no_fixed_width_carrier` is the shape of that argument, and a format that
widens from 128 to 4096 bits has moved its refusal without removing it.

**The bits are still the ENGINE's bits.** v1's one design rule survives unchanged: the
writer reads `Sim::pairs[..pair_count].bonded` and re-derives nothing. Two implementations
of a cluster reading is how the two of them come to disagree.

**`content` is a HEADER field, not a per-frame flag.** A run either recorded forces or it
did not. A per-frame flag would let one file mean two things halfway through, which is
exactly the silent reinterpretation this freeze is about.

### 1.3 DIMS AS MEASURED — the field that must never be believed

`dims_declared` is written because it records what the caller asserted, and it is the ONLY
header field this format explicitly labels untrustworthy. Every reader computes, from the
frames it actually read:

* **`span[k]`** = `max − min` over all atoms and all frames, per box axis, in bohr;
* **`rms[k]`** = the square roots of the eigenvalues of the 3×3 position covariance,
  descending, computed by cyclic Jacobi to a fixed 64-sweep budget (deterministic, no
  external solver, no device-class dependence);
* **`measured_dims`** = the count of `k` with `rms[k] / rms[0] > 1e-6`, and the ratio
  `rms[2]/rms[0]` is printed on EVERY read whatever the verdict, so no reader ever has to
  take the threshold's word for it.

The covariance route rather than the axis spans alone, because a scene locked to an
arbitrary plane has three nonzero axis spans and rank 2. The axis spans are reported too,
because §14.4's finding is stated in `max |z − z₀|` and this campaign must be able to
reproduce that number in its own units.

A file whose `dims_declared` disagrees with `measured_dims` is **reported on every read and
never repaired in place**; the reader returns both and the caller decides. Silently
correcting the header would put the lie one layer down.

### 1.4 The new envelope, stated rather than hidden

Pair indices are `u32`, so the format refuses at `n_pairs ≥ 2³²`, i.e. **N ≥ 92,682**. That
is a boundary and it is written here because `Boundaries.no_fixed_width_carrier` says every
fixed-width carrier has one: the honest move is to name the location, not to claim there
is none. It is 231× the largest scene this campaign will produce.

### 1.5 What the format deliberately does NOT carry

**Per-cell fractional occupancy is not a stored field.** `RUNG2_PREREG_A2`'s fractional
mean-occupancy chart is a function of the positions and a GRID, and a grid is a reader's
choice. Storing it would (i) bake one grid into the artifact — the same trusted-declaration
failure §1.3 exists to close — and (ii) create a second implementation of a reading that
already has one. It is provided as a reader-side derivation over v2 instead, and the
derivation is the only implementation.

---

## 2. THE FORMAT GATES

Every gate is EXACT unless it carries a number. "EXACT" here means bit equality of `f64`
payloads and set equality of bond sets — never a tolerance.

- **G1 — v1 BIT-IDENTITY on the real bank, EXACT.** Every one of the banked census
  trajectories under `census-traj` (23 files, `census_traj_manifest.sha256`), read through
  the v2 reader and re-serialised through a v1 writer, reproduces the original file byte
  for byte. Checked as a digest comparison. **A single mismatch refuses the READER**, and
  the campaign reports the reader as defective; it never reports the banked file as
  defective. witness: `MergeLaw.digest_convicts`
- **G2 — v1 FIELD-IDENTITY, EXACT.** For every banked file, every value the v1 reader
  produces equals the value the v2 reader produces: all 11 header fields, and per frame the
  index, time, temperature, the bond SET, and all 6 f64 per atom. Bit equality on floats.
  witness: `MergeLaw.digest_window_faithful`
- **G3 — version discrimination, EXACT, both directions.** A v2 file offered to the v1
  reader is refused naming the magic found; a v1 file offered to a v2-only path is refused
  naming the version found; neither is reinterpreted. Both refusals carry DISTINCT exit
  codes (§6). A refusal that does not name what it found fails this gate. witness: `none
  (a discrimination between two byte encodings; the Lean tree carries no encoding model,
  and claiming one would be the overreach M-PROVENANCE-OVERREACH names)`
- **G4 — the cap is gone, and the new envelope is where §1.4 says.** A synthetic scene at
  N = 402 and one at N = 4096 round-trip through v2 with the full bond set exact, and the
  writer refuses at 92,682 naming the pair-index envelope. 3 readings, all EXACT.
  witness: `Boundaries.no_fixed_width_carrier`
- **G5 — dims measured, not declared, on the real bank.** Of the 18 parked trajectories
  §14.4 tabulates, the v2 reader must report `span[z]` **exactly 0.0** on the seventeen
  planar ones and **> 10.0 bohr** on `de4_on`, and must report `measured_dims = 2` and
  `3` respectively — from the data alone, with `dims_declared = 2` on all eighteen.
  `rms[2]/rms[0]` is printed for all 18. witness: `none (a measurement of a coordinate
  cloud's rank; no theorem in lean/CIRISHolon states it and none is invented for it)`
- **G6 — the two UNDISCHARGED ledgers become computable FROM THE ARTIFACT.** On a v2 run
  with `content = 3`, a reader that never constructs a `Sim` computes
  `|P(t) − P(0) − J_ext(t)| ≤ momentum_bound` and
  `|ledger(t) − l0 − w_ext(t)| ≤ drift_bound` at every frame. The gate is that both are
  COMPUTABLE and that their values equal the engine's own `momentum_residual` and `drift`
  to 1 ulp at the final frame. **Whether they CLOSE is not gated here** — that is the
  successor's physics question, and "not computable" and "computed and failed" are
  different facts (M-EXIT-DISCRIMINATOR). witness: `Carrier.closed_transports`
- **G7 — the plants all fire.** §3. Every plant fires, or the gate it defends is void.
  9 plants. witness: `MergeLaw.digest_convicts`

---

## 3. THE PLANTS

Each plant names its CARRIER (the artifact the defect is planted in) and the SECTOR the
plant must be nonzero in (M-PLANT-SECTOR). A plant that fires in a sector it was not
planted in is a defect in the instrument, not a pass.

| plant | carrier | sector it must be nonzero in | must |
|---|---|---|---|
| **P-1** one flipped byte | a COPY of a banked v1 file | the file digest | G1 MISMATCH on exactly that file |
| **P-2** header fields transposed | a mutant v2 reader that reads `dims` and `substeps` in the opposite order | the two header integers | G2 fires and NAMES both fields |
| **P-3** one bond dropped | a copy of a v1 file, one frame's bitset with its highest bit cleared | the bond set of exactly one frame | G2 fires on that frame index and no other |
| **P-4** staked z-span | a synthetic 3D scene placed with a z-span of EXACTLY 7.5 bohr | the z coordinate | G5's `span[z]` reads 7.5 to 1 ulp |
| **P-5** truly planar | a synthetic scene with every z identical | the z coordinate | `span[z]` EXACTLY 0.0, `measured_dims = 2`, `rms[2]/rms[0]` exactly 0.0 |
| **P-6** the declaration lies | a v2 file with `dims_declared = 2` whose data spans z by 7.5 bohr | the declared/measured pair | the reader reports the DISAGREEMENT and returns the measurement, not the declaration |
| **P-7** a planted impulse | a synthetic run with a known external impulse posted | `J_ext` | G6's residual stays at the roundoff bound while raw `ΔP` is nonzero at the planted magnitude — the control that separates "the ledger closed" from "nothing happened" (M-VACUOUS-SUCCESS) |
| **P-8** truncation | a v2 file cut mid-frame | the frame count | reads as a short prefix reported INCOMPLETE, never as an error and never padded |
| **P-9** an empty scene | N = 0 and N = 1 | the pair-index arithmetic | round-trips with zero bonds, refuses nothing, and `pair_index` is never called |

**P-2 is the plant this freeze cares most about.** G1 and G2 are the bit-identity claims,
and a bit-identity check that has never been seen to fail is a fence. P-2 manufactures the
exact silent reinterpretation the version tag exists to prevent and requires the check to
convict it.

---

## 4. THE N-LADDER

### 4.1 What is measured, and in what currency

**Wall clock is not a reading in this campaign.** The host is shared and loaded and this
node cannot pin cores; M-PLACEMENT-LOTTERY and M-DEVICE-CLASS are contacted and NOT
discharged. Every price below is in WORK UNITS, defined here before any rung runs:

* **W_pair** — one pair-term evaluation (one entry of the neighbour list, one force-loop
  visit). This is the ladder's primary currency.
* **W_triple** — one triple-term evaluation. Priced separately because its scaling law is
  a different law.
* **W_de4** — one exact four-body solve. Priced separately because it is 3–4 orders of
  magnitude more expensive per unit than W_pair and mixing them would launder the total.

Reported per rung: N, box, number density, `Route` actually taken, cells per axis,
neighbour-list length, W_pair/step, W_triple/step, W_de4 for the whole probe, and the
worker count the pool actually LEASED (not the count requested — M-PROBE-THE-RESOURCE).

### 4.2 The rungs

Water stoichiometry exactly 2:1, so the rungs are the brief's ladder rounded to whole
molecules: **N ∈ {24, 48, 96, 201, 402}** = 8, 16, 32, 67, 134 waters. Box: a cube at
liquid water's number density, **0.01486 atoms/bohr³** (0.0334 molecules/Å³), so the only
thing that changes across the ladder is N. `dE4` stays **ON** at every rung: a fenced arm
would price a carrier nobody is going to run.

### 4.3 The forward stake — F1, written before the instrument exists

`cells.rs` engages `Route::Cells` only when every axis admits ≥ 3 cells at the declared
cutoff AND N ≥ 64. At fixed number density in 3D the box edge grows as N^(1/3), so the
route must switch on somewhere in this ladder, and the pair cost must bend from N² to N.

- **F1 — the log-log slope of W_pair/step against N over the top three rungs (96, 201, 402)
  is ≤ 1.35.** A slope ≥ 1.80 means the cell route never engaged and the ladder has
  measured the O(N²) path under a 3D label. Both outcomes are reported; only the first is
  a pass. witness: `none (a measured scaling exponent of this engine's own neighbour
  build; no theorem states it)`
- **F2 — `Route::Cells` is reported as TAKEN at N = 402, with ≥ 3 cells on every axis.**
  This is the mechanism behind F1 and is reported separately, because a slope can bend for
  the wrong reason and a conjunction that is only ever checked as a total hides which
  conjunct did the work (M-CONJUNCTION-MONOTONE). witness: `none (engine route selection;
  no Lean model of the cell decomposition exists in this tree)`

### 4.4 What can VOID the ladder

Stated in advance so no reading gets rescued after the fact:

* fewer than 3 rungs complete → **VOID, no slope quoted**;
* the pool leases fewer workers than requested at any rung → that rung is reported and
  EXCLUDED from the slope, because it priced a different machine;
* any rung whose bitwise result differs from the serial reference → the whole ladder is
  VOID and the campaign reports a reproducibility defect, not a price.

### 4.5 The scissor arithmetic, staked before it is computed

`RUNG2_RESULTS.md`'s successor bar is **≥ 100 atoms per cell with inter-cell transport
≥ 0.05**. Those two are not independent of N: a grid of `C` cells needs `N ≥ 100·C`, and a
chart with fewer than 2 cells per axis has no faces to transport across, so the smallest
grid that can be asked the question is 2×2×2 = 8 cells and the smallest N that can answer
it is **800**.

- **G8 — the scissor's price is REPORTED, not assumed.** From the ladder's fitted W_pair(N),
  the campaign reports the work-unit price of N = 800 and N = 6400 (a 4×4×4 grid) and says
  plainly whether either is affordable here. If it is not, the deliverable is the NUMBER
  and the refusal, not a smaller run relabelled as sufficient. Arriving under a cost model
  is its own misfit (M-CHEAPER-THAN-ITS-PRICE) and arriving over one is disclosed with its
  cause. witness: `none (an extrapolation of this campaign's own measurement)`

---

## 5. THE PRODUCTION TRAJECTORIES

Run only after §4 reports, and only at an N the ladder priced.

- **G9 — genuine 3D placement, measured.** Every production trajectory reads
  `measured_dims = 3` and `rms[2]/rms[0] > 0.10` **at frame 0** and over the whole run. A
  scene that opens planar is refused before it integrates: §14.4's lock is a PLACEMENT
  property and catching it at the end is catching it too late. witness: `none (see G5)`
- **G10 — the artifact is banked with its receipts.** A `sha256` manifest over every
  produced file, the launch line, the instrument's commit, the pool's leased worker count,
  and the measured-dims line for every file. The manifest pins the files and nothing
  beside them; a provenance line that claims more than the digest establishes is
  M-PROVENANCE-OVERREACH and this campaign does not write one.
- **G11 — content is complete.** Every production file carries `content = 3` (forces AND
  ledger), so G6's two readings are computable on it. 0 files may carry less.

**This node does not certify anything with these trajectories.** They are a carrier for a
successor campaign. Any admissibility verdict read off them here would be this programme's
own forbidden shape.

---

## 6. THE INSTRUMENTS' OWN DISCIPLINE (gate 10a3)

No instrument in this campaign carries a session-keyed or lane-keyed path.

* Paths resolve from **the script's own location**, then a named environment override
  (`HOLON_ARTIFACTS`, `HOLON_CENSUS_TRAJ`), then refusal. No default under `/tmp`, no
  worktree path, no lane name.
* Every launcher takes `--dry-run`, which prints the exact resolved paths, the rung list
  and the work-unit estimate, and exits 0 having created nothing. Testing a launcher by
  launching it is not a test.
* Refusals carry **discriminated exit codes**, fixed here: `2` bad arguments, `3` a path
  did not resolve, `4` a version/magic mismatch, `5` a digest mismatch, `6` a worker-lease
  refusal, `7` an envelope refusal (N too large). A single exit code 1 for everything
  cannot tell a caller which failure it hit (M-EXIT-DISCRIMINATOR).
* Long runs: `setsid`, a `.DONE` marker carrying the exit status, and a `RESUME.md`.
  Session death must kill narration only.

---

## 7. WHAT THIS FREEZE DOES NOT CLAIM

* Nothing about whether water forms, whether any tier closes, or whether the fluid-element
  band is admissible. Those are the successor's, and this document may not be cited for
  them.
* Nothing about the four-body term's physical effect. `CENSUS_RESULTS.md` §14.4 says the
  comparison that would settle it was compromised by the plane; a 3D carrier REMOVES that
  confound for the next campaign and does not retroactively fix the old one.
* No timing claim. The host is loaded; every wall-clock number this campaign prints is
  contended and labelled so.
* No claim that the ladder's slope is a universal scaling law. It is this engine's
  neighbour build at this density on this ladder — an N-convergence statement about one
  spatially homogeneous scene family (M-VOLUME-SCALE, M-HOMOG), not about molecular
  dynamics.
