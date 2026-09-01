# SATURATION-3 (tables and physics) — detached runs

Lane: `saturation3-water`. Contract: `../SATURATION3_PREREG.md` (7bc8d50).
Sibling: `saturation3-mesh` owns G1 (sharded generation) and G2 (GPU).

| marker | producer | what it is | restart |
|---|---|---|---|
| `g0.DONE` | `holon-chem --example s3_g0` | the blocking cost gate: five staked triple types at their worst compact geometry | `engine/target/release/examples/s3_g0 > s3_runs/g0.log` |
| `oo_reexam.DONE` | `holon-chem --example s3_oo_reexam` | the O-O re-examination: SATURATION-2 P1's own 96-knot curve, production cap vs raised cap, scored at the derived bar | `s3-oo worktree: engine/target/release/examples/s3_oo_reexam 20000 > s3_runs/oo_reexam.log` |
| `bar_margin_after.log` | `holon-chem --example s3_bar_margin` | the nine staked curves re-read under the DERIVED bar (no marker; the run is seconds) | `s3-oo worktree: engine/target/release/examples/s3_bar_margin` |
| `angle_gauge.log` | `holon-chem --example s2_build -- --gauge 4` | the interpolator's own rate on planted analytic functions, and the three c-axis masks on the real surface. Costs no solves for the planted rows | `engine/target/release/examples/s2_build --gauge 4` |
| `angle_slice_collinear.log`, `angle_slice_h2seam.log` | `--example s3_angle_slice` | dE3 walked across each seam, with centred d2/d3 | `s3_angle_slice 41 1.4125` and `s3_angle_slice 61 0.30 0.62 2.621 2.703` |
| `domain.DONE`, `domain_{hhcl,ooh,hclcl,clclcl}.log` | `--example s3_domain` | stage-(1) domain derivation for the four cheap tables: A1 second-smallest-side truncation sweep per triple, apex named on the command line | `s3_domain Cl H H 8 13 17`, `s3_domain H O O ...`, `s3_domain H Cl Cl ...`, `s3_domain Cl Cl Cl ...` (script: scratchpad `dom.sh`) |
| `angle_cross_check.log` | `--example s3_cross_check` | the discriminator: a warm start carried across each corner, to tell a real crossing from a wrong-root solve | `s3_cross_check` and `s3_cross_check 2.621 2.703 0.425 0.450 9` |

## RUN FROM A PINNED WORKTREE, and why

Both runs above were built and run in `/home/emoore/holon-wt/s3-oo`, a detached
worktree pinned at `179db95` — NOT in the shared tree. An uncommitted refactor of
`fci.rs` (371 lines, the eigensolver made generic over a scalar tier) appeared in
the shared tree at 18:02 today, and a binary built there at 18:03 picked it up. A
measurement of the SHIPPED solver cannot be taken in a tree where the shipped
solver is being replaced. Check `grep -c "crate::tier" fci.rs` = 0 before trusting
a rebuild. Anything reproducing these numbers should use the same pin.

That refactor has since landed as `fe18572`, and both readings were re-taken
against it in the shared tree: `bar_margin_after.log` and `oo_trace.log` are
BIT-IDENTICAL to the pinned worktree's. So the numbers transfer to main, and the
f64 instantiation of the generic solver reproduces the crate's hardest solve —
the O-O iteration-capped knot — exactly, which is a check that lane did not have.

## G0 banked

Determinant counts exact, five of five, against the freeze's committed arithmetic.
All converged at 8.1e-11 to 9.8e-11. Nothing within 8x of the 10x kill.

    (H,H,Cl)     605   0.21 s      (H,Cl,Cl)   3,249   1.25 s
    (Cl,Cl,Cl) 9,477   6.47 s      (O,O,H)     9,075   0.38 s
    (O,O,O)  207,025  39.83 s

Two findings that shape the rest:

  * (Cl,Cl,Cl) is OVER the freeze's "<= 5 s/point" class at 1.29x. It does not
    re-scope; the class STATEMENT needs amending. The boundary is inside
    run-to-run variance -- two runs read 5.67 s and 6.47 s with identical
    residuals and iteration counts.
  * THE TWO COST CLASSES ARE BOUND BY DIFFERENT THINGS. (Cl,Cl,Cl) is 73%
    ASSEMBLE (27 orbitals, basis-bound); (O,O,O) is 99% CI (207,025 determinants,
    determinant-bound). The warm start and the GPU kernel both accelerate
    Davidson, so both can only pay on (O,O,O). Size by BASIS SIZE, not by
    determinant count.

## Staked geometry rule (fixed before any solve)

Each side at 0.75 x that pair's own located R_e. Located values:
H-H 1.3887, H-Cl 2.5369, Cl-Cl 4.0241, O-O 2.4421, O-H 1.9909 bohr.

`homonuclear_radius` must NOT be used for chlorine: it is declared only for
Z <= 10 and silently returns hydrogen's 0.694 bohr for Z = 17.

## THE TRAP, carried forward

`fci::solve` routes past MPS_ROUTE_THRESHOLD = 50,000 determinants into an MPO
builder that reaches six orbitals and HANGS rather than erroring. (O,O,O) at
207,025 over fifteen orbitals is the one staked combo that crosses it. Call
`solve_determinant` explicitly anywhere the space size is not statically obvious.

## The angle-axis obligation is DISCHARGED

Full record in `../SATURATION3_RESULTS.md`. In three lines: the owed rate was
never 16x — gauged on planted functions with the same eval and draw, this
interpolator gives 9.3-10.9x per doubling; the end condition costs nothing (the
masked columns are identical on planted data); and the shortfall is TWO REAL
ELECTRONIC STATE CROSSINGS inside the domain, at theta ~ 174.9 deg on
(1.766, 2.576) and theta ~ 36 deg on (2.621, 2.703), both confirmed physics by a
warm start that never found a lower state. `c = sqrt(1 - cos theta)` compresses
the collinear seam into the LAST grid cell, which is why the bend angle beat it
by 2.2x.

**The ruling this forces on every grid still to be frozen:** uniform angle
refinement cannot beat a corner, so locate each table's seams first and either
put a grid line on one or accept its floor deliberately. Every trimer type here
has a reactive channel; this is not water-specific. STILL OWED: the seam LOCUS.
Two slices found two points on two seams; where the seams run is unmeasured, and
the ruling above needs it before an angle axis is frozen.

## RESCUED REF, do not delete: `rescue/de4-sim-worktree` (7480437)

The two long-running `waterquench` processes were built from a sim.rs whose dE4
dispatch exists in NO COMMIT, while a T3 refactor was being layered onto the same
file in the shared tree — so their source was minutes from being unreconstructible.
Taken with `git stash create` and ref'd as a branch: it touched neither the working
tree nor the index, so the refactoring lane was not disturbed and did not need to
stop. It carries dE4 intact (36 references, 2958 lines).

It is a snapshot of dE4 PLUS PARTIAL T3, not of the binary's source. But the binary
dates itself: `strings` finds none of the T3 markers (`DEFAULT_SCENE_ATOMS`,
`complete_pairs`, `ExternalWork`, `Periodic`), while `nm` finds
`holon_chem::quaternary::de4_ohhh_fci`. So the binary PREDATES the refactor and the
reconstruction target is this ref minus the T3 hunks — a live diff, not archaeology.

`quaternary.rs` itself is committed and gated: `tests/quaternary.rs` asserts all 40
of this lane's staked witnesses with exactly 11 attractive, the sign structure the
referee established when the five-parameter fit was killed. What is uncommitted is
the DISPATCH, not the physics.

## The four table domains — STAGE (1) IN FLIGHT

Per the lead's two-stage ruling: stage (1) is spans and extents and goes to the
mesh lane now; stage (2) is the seam-locus scan and gates each table's
GENERATION, not the handoff.

Rulings already sent to `saturation3-mesh`, not waiting on the sweeps:

  * **WARM IS OFF on all four.** Their own numbers: negative on (H,H,Cl), ~11%
    undiluted on (Cl,Cl,Cl), 6.9% on (O,O,O) with a slowdown at the largest step,
    plateauing rather than growing. Against that, warm moves the answer (0 of 5
    bit-identical) and these are bit-gated artifacts whose identity would then
    include their warm chains. Reconsidered only if (O,O,O) is ever built.
  * **Region shape is mine and ships with each domain**, not separately — it is
    part of the table's identity and must not move to fit the machine.

The apex convention: the two axes are the sides meeting at the atom the table's
symmetry does NOT exchange. Cl for (H,H,Cl), H for (H,Cl,Cl) and (O,O,H), any
vertex for the S3 table. `x_lo` comes from `pair::derive_range` on the apex pair
rather than a new rule, so the table's inner wall and the pair curve's inner wall
are one claim.

NOTE against the sweep's own header: the reported opposite-pair `R_e` is a coarse
3-round locate and reads 1.3884 for H-H against G0's 1.3887. It is REPORTED only;
the shell ladder uses the apex pair, which reproduces G0's 2.5369 exactly.

## Next, per the freeze's sequence

(2) generalize trimer.rs over species (symmetry axis per table; S3 only for the
two homonuclear types) with the warm-start locality sweep -- sized on (O,O,O) per
G0's split; (3) the angle-axis anomaly as a staked design obligation; (4) tables
in cost order; (5) P1-HCl; (6) P2-H2O.
