# GRAVITY-BRIDGE-0 — RE-ADJUDICATED: formally VOID under its own frozen prereg

*2026-08-27, second adjudication after independent external re-review (which
REPRODUCED the run exactly: exit 0, matching log). The first adjudication
below claimed "all gates pass"; the review showed that verdict violated the
freeze in three ways, and the review is CORRECT:*

1. **G5 FIRES under its frozen scope.** The prereg staked constraints on
   "every state used in G1–G4"; the instrument checked only the three
   un-kicked states. Applying the instrument's own `constraints_held` to
   the G4 intervention states shows the matter-plaquette kick violates the
   p0 constraint and the control kick violates Gauss at c, on both
   triangulations — a raw edge kick leaves the physical subspace, so the
   staked G4 interventions and the staked G5 scope were jointly
   unsatisfiable as frozen.
2. **The plants ran as a different experiment.** The frozen G8 named its
   plants; both missed (each provably unobservable), and the frozen rule
   says a missed plant VOIDS the campaign. Replacing the plants post-run
   was a post-result rescue, which the freeze forbids.
3. **The freeze had no independent timestamp** — prereg, instrument, and
   results entered git together, so freeze-order is not evidenced.

**Formal verdict: VOID.** What SURVIVES as an exploratory pilot, per the
review's own reading: an exhaustive exact D(S3) gauge-KINEMATICS result —
a central flux sector fixes the distant boundary holonomy class, loop
inversion is exact, both staked analytic support counts match, and the
reading is exactly refinement-invariant. What does NOT survive:
"dynamical curvature" and "reciprocal backreaction" (no joint dynamics; the
matter was the flux itself; the repinning compared superselection sectors;
the kicks were unphysical). BRIDGE0-V2 is preregistered separately, with
its prereg committed BEFORE its instrument, constraint-preserving
interventions, an independent matter register, full-scope G5, and the two
plants specified exactly. The original (superseded) adjudication follows,
kept per house rules — a record is a history.

---

# GRAVITY-BRIDGE-0 — verdict: ALL GATES PASS, BOTH PLANTS FIRE

*2026-08-27. Prereg frozen before the instrument existed
(`GRAVITY_BRIDGE0_PREREG.md`); instrument `bridge0.py`, exact integer
arithmetic end to end; run log `bridge0_run.log`.*

## What was demonstrated, exactly

On the quantum double of S3 (a discrete 2+1D BF-type gauge theory) over a
triangulated disk, with matter as a conjugacy-class flux at the central
plaquette:

- **Curvature is dynamical holonomy** (G1, G2): the vacuum's encircling
  loop reads identity on every support configuration; pinning matter of
  class C makes the DISTANT boundary loop read class C on every support
  configuration — for the 3-cycle class this is a literal discrete conical
  deficit of 120° (S3 ≅ D3 ⊂ O(2)). No prescribed potential exists
  anywhere in the instrument; the holonomy comes from constraints and
  matter content only.
- **Orientation** (G3): loop reversal inverts the holonomy element-wise,
  exactly. (Class-level inversion is trivial in S3 — ambivalent — as the
  prereg staked in advance; a non-ambivalent group is the named follow-up.)
- **Reciprocity with local scope** (G4): re-pinning the matter class moves
  the distant geometric reading; a local geometric kick on the matter
  plaquette's boundary moves the matter reading; the same kick on an edge
  not bounding the matter plaquette moves nothing. Both directions, plus
  the locality control.
- **Constraints Held** (G5): every Gauss average acts as 6·id and every
  flatness/pinning projector fixes every state used, as exact integer
  identities.
- **Refinement invariance** (G6): identical verdicts on the refined
  triangulation — exact, as the topological theory demands, not asymptotic.
- **Independent oracle** (G7): measured supports equal the closed forms
  staked in the prereg before the instrument ran: 216 / 648 / 432 (base)
  and 1296 / 3888 / 2592 (refined) — the pure-gauge parametrization and
  the instrument agree, both graphs, all sectors.
- **The harness can fail** (G8): a wrong-side group action fires the
  constraint gate; a dropped-inverse holonomy word fires the class reading
  on the ρ flux. Both plants fire.

## The plant-history, kept as a finding

THREE candidate plants were provably unobservable and are recorded in the
instrument: a skipped group element launders into uniform orbit weights;
a single vertex's Gauss constraint is redundant on a disk (implied by the
others through the global gauge kernel); the τ-flux broken-word reading is
parity-protected (odd reflection count forces a reflection class). Each
diagnosis is itself a small exact theorem about the model, and the lesson
is the house's: a planted defect must be OBSERVABLE, and proving a plant
invisible is as informative as firing it.

## Pre-verdict instrument repairs (gates' semantics untouched)

The G4 geometry kick was corrected from a ρ-kick to a τ-kick after the
first run: in D3, rotation·reflection = reflection, so a ρ-kick provably
cannot move a τ-flux's class — the staked gate ("a kick changes the
reading") was unmet by a kick the group theory forbids from changing
anything. All repairs preceded the verdict; no gate's meaning moved.

## The fence, restated

This is a finite-group BF toy — the smallest exact instance of the
curvature-as-holonomy bridge, not gravity. Named successors, in order:
a non-ambivalent group (class-level orientation becomes falsifiable); the
non-Abelian quantum-connection API with SU(2) representations (the
reviewer's item 3); 2+1D SU(2) BF with a genuine deficit-angle/mass
relation; only then 3+1D structure. Each is a new prereg.
