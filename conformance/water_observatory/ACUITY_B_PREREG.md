# ACUITY-B — the observer's frame is an ALLOCATION law, and the tree falls either way

*Frozen 2026-09-02, before a line of the coarse-step path existed. The operator's
question: test reading B of `OBJECT.md` §"Where the weight sits" — does the observer's
frame select what IS a thing, or only what gets FINE allocation — and bank the speedup
if the second. Built by the lead, no lane. The git order is the check.*

**misfits:** M-VACUOUS-SUCCESS, M-FIXED-POINT-TRAJECTORY, M-ONE-MODEL-DELTA,
M-NULL-MISSTAKE, M-UNTESTED-GAP, M-CONJUNCTION-MONOTONE, M-BASE-RATE-OMITTED,
M-BUDGET-LAUNDER, M-EXIT-DISCRIMINATOR, M-MAX-OVER-SUCCESSES, M-SORTS-NOT-SEPARATES,
M-TAG-AS-PROPERTY, M-PLANT-OBS, M-PLANT-SECTOR, M-POPULATION-CHOICE,
M-PRESENTATION-VERDICT, M-NONBIJECTIVE-STEP, M-STALE-INSTRUMENT, M-CHEAPER-THAN-ITS-PRICE,
M-PLACEMENT-LOTTERY, M-DEVICE-CLASS, M-CACHE-KIND, M-MAINTENANCE-LENS, M-LOOP-BLIND,
M-COND-PROBE, M-PROBE-THE-RESOURCE, M-IDLE-CALIBRATED-TIMEOUT, M-VOLUME-SCALE, M-HOMOG.

---

## B.0 THE CLAIM, SPLIT INTO ITS THEOREM HALF AND ITS MEASURED HALF

**The theorem half needs no run.** Under the two-box law (`WORKBENCH_FSD.md` §9c) the
zoom never touches the physics: the world box's dynamics is one trajectory whether or not
a scene box is looking at it. Any verdict computed from a thing's own atoms — the
closure leg on the certified molecule — is therefore frame-invariant BY CONSTRUCTION.
The tree falls whether or not anyone is there. Measuring this would be a vacuous pass
(M-VACUOUS-SUCCESS) and is not staked.
witness: `closed_iff_fiber_invariant`

**The measured half is allocation.** Reading B's missing piece is not "who decides
thinghood" but "who decides FINE allocation": the observer's frame selects which holons
run fine; the unobserved are carried COARSE. The only non-trivial question — and the one
that pays — is whether carrying the unobserved region coarse changes the observed thing,
and by how much, against how much work is saved.
witness: `nonfactoring_iff_not_closed`

## B.1 THE INSTRUMENT — a coarse-step path in `Sim`, behind an `AcuityFrame`

`Sim::acuity: Option<AcuityFrame>` — a scene box (centre, half-width) in world
coordinates. When `None`, nothing in this design executes and `Sim::step` is byte-for-byte
the existing step (G0 below is that statement as a gate).

**Membership, per step.** An atom is FINE if it is inside the scene box, or if any member
of its live composite row (`HolonLayer` rows) is inside — a molecule straddling a face is
never torn. Every other atom is COARSE.

**The coarse law.** A coarse composite moves as ONE object on its conserved totals:
its members share the composite's centre-of-mass velocity; the composite is accelerated
by the sum of forces its members receive from FINE atoms (Newton pairs applied on both
sides, so total momentum is conserved exactly); its internal relative velocities are
BANKED at coarsening and RESTORED at re-entry (accounting-only, the `holon.rs` law).
Pairs, triples and quadruples whose members are ALL coarse are NOT EVALUATED — that is
the saving, and inter-composite coarse interaction is the approximation. A lone coarse
atom (no row) is its own composite.

**Every transition is a ledgered scene event.** The energy change a membership
transition makes — banking/restoring internal motion, dropping/re-admitting
coarse-coarse potential — is measured directly (`energy()` before and after the
transition) and posted to a NEW ledger column `work.acuity`. The drift gate therefore
stays closed by construction, and the column IS the energy cost of the observer's
frame, reported beside the speedup and never hidden inside `w_ext`'s other columns.

## B.2 THE GATES

- **G0 — identity, EXACT.** With `acuity = Some(frame covering the whole world box)`
  every atom is fine, and after `N = 2000` steps the `Checkpoint` bytes (which carry
  `w_ext`, the work columns, `l0`, `p0`, `j_ext`) are IDENTICAL to the `None` run. Not a
  positions digest: a ring that got the bookkeeping wrong fails with every position right.
  witness: `closed_iff_fiber_invariant`
- **G1 — momentum, exact to roundoff.** Under a frame that leaves at least 25% of atoms
  coarse for the whole run, `momentum_residual_peak <= momentum_bound` at every grain
  boundary (the existing gate, unchanged). The coarse law must not break Newton's third
  law; the plant is dropping the fine-side reaction.
  witness: `none (an engine conservation gate, not a Lean claim)`
- **G2 — the ledger closes, with the observer's column visible.** `drift() <= drift_bound`
  at every boundary AND `work.acuity` is reported; the plant is applying a transition
  without posting it, which must open the drift gate.
  witness: `none (engine ledger gate)`
- **G3 — the observed thing.** On a scene with the frame around ONE composite, the
  observed atoms' trajectory under coarse-outside stepping stays within
  `rms_dev <= 0.5 bohr` of the full-dynamics trajectory over `2000` steps, and the
  observed row's `closure_defect_peak` differs from the full run's by `<= 0.10` of the
  well depth. Pre-committed: this is the number that DECIDES B, and both directions are
  results (B.4).
  witness: `nonfactoring_iff_not_closed`
- **G4 — work saved, an exact integer count.** `pairs_skipped + pairs_fine == pairs
  examined` on every step (the counter is a partition, not an estimate), and the
  speedup is reported as `pairs_skipped / examined` plus triples and quadruples skipped.
  A run in which nothing is coarse must report `0` skipped (the counter's own control).
  witness: `none (a work counter)`

## B.3 PLANTS — each names the sector the plant acts on and must FIRE before its gate is believed

- **P-1 (sector: the coarse flag)** — a frame that marks one atom coarse while covering
  the world box must move the G0 digest; a G0 that passes with a coarse atom present has
  not compared bookkeeping. Nonzero in: the checkpoint bytes.
- **P-2 (sector: the fine-coarse reaction)** — drop the reaction on the fine side of a
  fine-coarse pair; G1 must fire. Nonzero in: `momentum_residual_peak`.
- **P-3 (sector: the transition ledger)** — apply a coarsening without posting
  `work.acuity`; G2 must fire. Nonzero in: `drift()`.
- **P-4 (sector: the work counter)** — count a skipped pair as fine; G4's partition
  identity must fail. Nonzero in: `pairs_skipped + pairs_fine - examined`.

## B.4 THE BRANCHES, staked before the measurement

- **(a) G3 inside budget, G4 saving > 0.** B's missing piece IS the allocation law: the
  frame selects fine allocation, the coarse carriage costs the observed thing less than
  the budget, and the speedup banks as a number with `work.acuity` beside it. The
  OBJECT.md read moves weight from B into A.
- **(b) G3 over budget.** The unobserved region is LOAD-BEARING for the observed thing
  (it was buffeting the molecule) — exactly what `WORKBENCH_FSD.md` §9c's de-allocation
  law anticipates: load-bearing rows keep fine allocation, and the measured defect is the
  criterion. The instrument then reports the defect-vs-frame curve, not a verdict.
- **(c) G0/G1/G2 fail.** The instrument is defective; nothing about B is learned and
  nothing is banked.

## B.5 SCENE AND COST

First reading on the shipped hydrogen curve (`viewer/h2_potential.json`): a 3D periodic
lattice of 64 H atoms at 10 bohr spacing (the `t3_scale.rs` builder), thermalised,
frame = a 12-bohr half-width cube at the centre. Priced in the work counter, not
seconds. The certified-water reading (12 atoms, the census protocol) is the named
follow-up and needs the O-O curve generated; it is not this freeze's run.
