# ACUITY-B — results

*Freeze: `ACUITY_B_PREREG.md`, ADMITTED, committed with this results document in the
same landing (the lead built the instrument in one sitting; the freeze text was written
and audited before the instrument existed, and the git diff of this commit is the
check). Instrument: `engine/crates/holon-render/src/acuity.rs` and the acuity paths in
`sim.rs`; gates `engine/crates/holon-render/tests/acuity.rs`; reading
`engine/crates/holon-render/examples/acuity_b.rs`; log `acuity_b.log`.*

## Verdict

**BRANCH (a) on the freeze's scene: B's missing piece is the ALLOCATION law, and the
speedup is banked with its defect beside it.** On the staked 64-atom periodic hydrogen
lattice (10 bohr spacing, seeded thermal velocities, 2,048 steps in 32 grain frames),
with the observer's frame a 20-bohr cube at the centre and every holon outside it
carried COARSE:

| reading | value | budget |
|---|---|---|
| G1 momentum residual | 2.242e-13 (bound 9.432e-10) | PASS, exact to roundoff |
| G2 energy drift | 1.792e-4 (bound 2.882e-3); `work.acuity` = +8.048e-3 Ha; columns balance | PASS |
| G3 observed atoms' rms deviation from the full dynamics | **0.0176 bohr** | 0.5 bohr |
| G3 observed row defect peak, full vs framed | 4.343e-2 vs 3.620e-2 → \|diff\|/D_e = **0.041** | 0.10 |
| G4 pairs skipped | **3,153,920 of 4,128,768 = 76.4%** (partition exact) | > 0 |
| transitions (ledgered scene events) | 56 | — |

The theorem half was never run, by design: under the two-box law the zoom never
touches the physics, so any verdict from a thing's own atoms is frame-invariant by
construction. What this instrument measured is the other half — carrying the
unobserved region coarse cost the observed thing 0.018 bohr and 4% of a well depth
while skipping three quarters of the pair evaluations. **The observer's frame selects
FINE ALLOCATION, not thinghood; the tree falls either way.**

## The density series: where allocation becomes load-bearing

The same instrument at denser lattices (readings, not the staked scene — the freeze
staked 10 bohr):

| spacing | rms dev (bohr) | \|Δdefect\|/D_e | saving | T_full (K) | branch |
|---|---|---|---|---|---|
| 10.0 | 0.018 | 0.041 | 76.4% | 6,086 | **(a)** |
| 6.0 | 4.012 | 0.381 | 64.1% | 35,671 | (b) |
| 3.0 | 5.385 | 7.708 | 16.8% | 280,835 | (b) |

At 6 and 3 bohr the lattice's stored potential releases into a hot, dense gas, and the
unobserved region is LOAD-BEARING for the observed atoms — exactly what
`WORKBENCH_FSD.md` §9c's de-allocation law anticipates: a row being buffeted keeps its
fine allocation, and the measured closure defect is the criterion. So the allocation
law has a measured crossover, and the FSD's "load-bearing rows stay fine" clause is now
a curve rather than a sentence: the frame's saving is real where the coarse region's
interaction with the observed thing is weak, and the defect grows past budget where it
is not. (The temperatures are a property of these scenes — hydrogen atoms recombining
into H₂ release the well depth — and are reported, not tuned.)

## Gates, and the plants that fired

`tests/acuity.rs`, 4/4: **G0** the framed step with every atom fine is byte-identical
to the classical step in checkpoint bytes (P-1: one coarse corner atom moves the digest —
fires); **G1** momentum exact under a frame with ≥25% coarse (P-2: dropping the
coarse-side reaction opens the gate — fires); **G2** the ledger closes with the
observer's column posted and `work_columns_ok` (P-3: an unposted transition opens the
drift gate — fires); **G4** `fine + skipped == examined` exactly (P-4: losing the skipped
count breaks the partition — fires). The gate lattice runs at 3 bohr so the plants are
OBSERVABLE (at 10 bohr the transition energies sit under the drift bound and P-3 would
stay silent — planted-defect-must-be-observable, discharged in the freeze's own tests).

**The `None` path moved no bit:** node E's raw-bits dump (`examples/node_e_dump.rs`,
42 lines) is identical to the committed golden except the checkpoint's own length and
digest, which changed by design (format v4 carries the new `acuity` column after
`barostat`). Full `holon-render` suite: 23 binaries, all green.

## Two findings the instrument produced rather than confirmed

1. **Whole-only readouts under engine-side coarsening must add the bank back.** The
   framed run reports T = 1,415 K against the full run's 6,086 K — not a physics
   difference but an ACCOUNTING one: a coarse composite's internal kinetic energy is
   BANKED (its members move at the centre-of-mass velocity), so `e_kin` no longer holds
   it. The two-box law's "whole-only observables from the world box" therefore needs a
   readout law when coarsening runs in the engine: temperature and pressure of the world
   box = fine sector + coarse centre-of-mass sector + the banked internal sector. Named
   for the workbench as a display law, not built here.
2. **The saving has a shape:** 76.4% at the staked frame is `1 − (fine fraction)²` in
   pairs, so the frame's payoff is quadratic in how much of the scene it excludes, while
   the cost (G3) is set by the coarse region's coupling to the observed thing. Those two
   curves crossing is the allocator's operating point, and the acuity law's arithmetic
   (one holon in view at a band's own scale) sits far on the payoff side of it.

## Scope, stated

Pair-sector transition energy is accounted EXACTLY (evaluated once per transitioning
pair and posted). Three- and four-body all-coarse skipping is COUNTED but its transition
energy is not yet posted, so G2 is exact only on pair-only scenes — the freeze's scene.
That accounting is the named follow-up before the certified-water reading (12 atoms,
the census protocol, needs the O-O curve generated), which is the reading that would
let this claim enter the stance at measured strength on the certified object rather
than on a hydrogen lattice.

## What moves in OBJECT.md's read

Branch (a) on the staked scene: weight moves from B toward A. The frame-selection
question has a mechanical answer for ALLOCATION — the measured closure defect decides,
and it decides by density — while the thinghood half was a theorem. What remains of B
is the deeper version (which frame is the thing's own), and that is exactly the
question the tree-falls argument closes: none is needed.
