# ELEMENTS-3 heavy lane — detached compute

Two jobs, both `setsid` + done-marker so a dead session kills only the narration.

## `dimers` — the E1/E2/F1 record

    ./target/release/examples/elements3_dimers \
        > crates/holon-chem/tests/data/elements3_dimers.txt \
        2> output/elements3/dimers.log

stdout is the BANKED RECORD that `tests/elements3_dimers.rs` reads; stderr is the human
log. Marker: `dimers.DONE`.

Cost is dominated by the ERI assembly, quartic in the CARTESIAN basis size, paid once per
geometry — and a curve is about a hundred geometries, because `derive_range` walks and
bisects for the repulsive wall before a single knot is computed and `locate_well` bisects
again for the minimum.

| pair | spherical | Cartesian | note |
|---|---|---|---|
| HCl | 10 | 10 | seconds |
| HBr | 19 | 20 | seconds |
| HI  | 28 | 30 | minutes |
| Br2 | 36 | 38 | minutes |
| Kr2 | 36 | 38 | minutes |
| Xe2 | 54 | 58 | **~an hour** — 58^4 is 11.3M integrals per geometry |

`dimers_firstpass.log` is an earlier run of the same species in a human-only format, kept
because it is an independent repeat of HCl, HBr and HI: same numbers, different process.

## `atoms` — the R1 record

    ./target/release/examples/elements3_atoms > output/elements3/atoms.log

Marker: `atoms.DONE`. Three outcomes per atom and the distinction between them is the
point:

* **solved** — energy, Davidson residual, multiplicity DERIVED from `<S^2>` on the
  converged vector, and the dual-route residual where the `O(N_det^2)` reference route was
  affordable (cap 3e4 determinants);
* **OVER BUDGET** — past this record's declared spending cap of 1.2e6 determinants. A
  budget, not a limit: solvable in principle, not bought here;
* route **`det (forced)`** — the space is past `fci::MPS_ROUTE_THRESHOLD`, so the
  PRODUCTION entry point `fci::solve` would have routed it to DMRG, which is measured to
  reach six orbitals and cannot do any atom in this range. Reached only via
  `solve_determinant`, which has no threshold.

The budget was 2e7 on the first attempt and nickel (9.4M determinants) did not finish in
twenty minutes on a machine at load 38. If it is raised again, raise it alone and watch
nickel, copper, rubidium and cadmium — those four are the next rungs.

## Resuming

    ls engine/output/elements3/*.DONE
    tail -20 engine/output/elements3/{atoms,dimers}.log

Both logs are line-buffered, so a partial run is readable and each row appears as it
lands. Nothing downstream parses these logs: the gates read the banked record, and
`tests/elements3_dimers.rs` additionally RECOMPUTES HCl live and requires it to reproduce
its banked row bit-for-bit, so a stale record is caught rather than trusted.

## Cost trap, if this is being made faster

Neither knot count nor determinant count is the lever. `derive_range` costs about thirty
solves and `locate_well` about forty more, before and after the knots respectively, and at
58 Cartesian functions that is most of the run. Caching the assembly across geometries, or
screening the ERI, is where the time is.

## The gate that is waiting on the record

The finished E1/E2/F1 gate now lives at `crates/holon-chem/tests/pending/elements3_dimers.rs`,
adopting the water lane's shared quarantine convention (see that directory's README) in
place of the PENDING_ file this lane originally kept here. It is NOT in `tests/` for one
reason: it reads
`crates/holon-chem/tests/data/elements3_dimers.txt`, that file is still being written by
the run above, and a test that fails because its data has not arrived yet would be a red
suite for every other lane on this shared tree.

When `dimers.DONE` appears:

    git mv engine/crates/holon-chem/tests/pending/elements3_dimers.rs \
           engine/crates/holon-chem/tests/elements3_dimers.rs
    cd engine && cargo test --release -p holon-chem --test elements3_dimers -- --nocapture

then commit the gate and `tests/data/elements3_dimers.txt` in the SAME COMMIT — the gate is
meaningless without its record and the record is unchecked without the gate.

What it asserts, so a reader knows what a failure means:

* **E1** — Kr and Xe are one determinant (and Br and I are NOT, so the claim distinguishes
  something), and Kr2 and Xe2 have no well deeper than the schema's `WELL_MIN_DEPTH`, on a
  grid that is checked to be non-degenerate first.
* **E2** — `D_e` falls HCl > HBr > HI, all on the determinant route, and Br2 binds.
* **F1** — **the stake FIRED.** The deficit against experiment does not grow down the
  column; it falls, and changes sign (+0.0214, -0.0013, -0.0149 Ha). The gate asserts the
  measured direction and the two OUTER signs, and deliberately says nothing about the sign
  at bromine, whose deficit is 0.03 eV — the same size as the uncertainty on the
  experimental number it is measured against.
* **liveness** — HCl is RECOMPUTED every run and must reproduce its banked row bit-for-bit,
  along with all six atomic references. That is what keeps the banked heavy rows a
  measurement rather than a memory.

`examples/elements3_counterpoise.rs` is F1's branch (b) and should be run once the record
exists: it reads `R_e` from the record and computes the counterpoise-corrected deficit with
ghost nuclei. If the raw deficit falls and the corrected one grows, F1 died of a
contaminated observable rather than of wrong physics — which is a different finding, and
one that does NOT inherit the dead stake's standing.

## OPEN, and the reason it is not yet a finding: zinc and gallium

The R1 record has zinc coming out a QUINTET and gallium a QUARTET. The periodic table
says singlet and doublet. Germanium onward is correct (3P, 4S, 3P, 2P, 1S, and the same
again in the fifth row), so whatever this is, it is confined to the two elements
immediately after the 3d shell fills.

There is a plausible physical story — STO-3G gives 4p the SAME exponents as 4s, which
makes 4p artificially compact and therefore artificially low, and Hund exchange then
over-stabilises a high-spin 4s(1)4p(3) configuration over the closed 4s(2). That story
is not evidence, and it is exactly the shape of thing that is comfortable to believe.

There is a competing explanation with the same signature: **Davidson converging to an
excited state.** A converged residual says the returned pair IS an eigenpair, not that
it is the LOWEST one; the start vector is built from the Hamiltonian diagonal, and a
ground state with little overlap on that start can be stepped over. Zinc is 665,856
determinants, which is where that becomes plausible.

`zn_diagnose.log` is the discriminator, and it needs no external table: the smallest
DIAGONAL element of H is the energy of the best single determinant in the same orbital
basis, and it is a variational upper bound on the true ground state. If the reported
FCI energy sits ABOVE it, the solver missed. Germanium is the control and has already
returned — FCI 4.026e-2 hartree BELOW its best determinant, so the check itself works.
Calcium is the second control; gallium and zinc are the subjects, ordered last because
they cost most.

**Do not report the high-spin result as a property of the model until that log shows
gallium and zinc below their best determinants.** If either comes out above, it is a
solver bug and the R1 record's rows for it are wrong, not interesting.

Whatever the answer, `tests/elements3_atoms.rs` deliberately does NOT gate zinc or
gallium: it covers the nine p-block atoms cheap enough to solve, all of which are
correct. Adding the two anomalous ones to that gate before knowing which cause is at
work would freeze an unexplained number into the suite.


## Status at the end of this lane's session

LANDED (all gates green; 51 tests across elements, spherical_d, w1_masks, md, fci,
p1_radius, plus 3 in elements3_atoms):

* W1 — verified, not re-implemented. See the note on c03f282 below.
* T1 — the table to xenon, three-layer gates, plants firing.
* AMENDMENT A1 — the d convention declared, R1 rescoped, P1's third rule, audited.
* The spherical-d correction, with the wrong-transform plant firing two-sided against
  all five of the freeze's counts.
* R1 heavy half — nine multiplicities from `<S^2>`, two sigma routes to 2.1e-14, and
  the 50-digit referee for krypton and xenon (1.0e-11 and 5.3e-11 hartree).
* The referee d-build — CART[2], `_self_overlap` general in l, per-component sqrt(3),
  the 5x6 projection, the Z = 19..54 table parsed from `elements.rs`.
* P1's third radius rule, with the label plant.
* Zn/Ga confirmed real by the best-determinant discriminator.

STILL RUNNING, and what to do when each finishes:

* `dimers.log` / `dimers.txt` — Xe2 is the last row. When `dimers.DONE` appears, follow
  "The gate that is waiting on the record" above: move `PENDING_elements3_dimers.rs`
  into `crates/holon-chem/tests/`, run it, and commit the gate WITH
  `tests/data/elements3_dimers.txt`. Kr2 already reports NO WELL over 3.278..10.240
  bohr, so E1's first negative is in hand; Br2 bound at R_e 4.417139, D_e 0.079640 Ha.
* `atoms.log` — indium onward. Nothing gates on it; it is R1's record.
* `referee_nobles.txt` — DONE, and already banked at
  `crates/holon-chem/tests/data/elements3_referee_nobles.txt` with its gate.

OWED, and named so it is not mistaken for done:

* The 50-digit referee reaches Kr and Xe because they are ONE determinant and need no
  eigensolve. The other seven referee-eligible atoms (Ge, As, Se, Br, Sb, Te, I) need a
  50-digit FCI, which mpmath does not reach at Ge's 23,409 determinants. R1's referee
  threshold of 3e4 is a fine threshold; the arithmetic does not follow it that far.
* The MPO-builder upgrade in `q8-mps` (AMENDMENT A1.2's named successor) is what turns
  the sixteen route-less atoms into routed ones. Not this lane's.

## The 4p counterfactual — running, prediction already committed

`p4_counterfactual.log` / `.DONE`, from `examples/p4_counterfactual.rs`.

The high-spin readings for zinc and gallium are established as genuine ground states of
this model (`zn_diagnose.log`). A MECHANISM was offered for why — STO-3G gives 4p the
same exponents as 4s, making it artificially compact and low, so Hund exchange
over-stabilises a high-spin `4s(1)4p(3)` over the closed `4s(2)`. That is a story, and a
story that explains a result is not evidence for itself.

The counterfactual scales the 4p shell's exponents by `lambda < 1`, making that shell
strictly more diffuse and higher in energy, leaving every other shell exactly as
declared. **The prediction was committed before the run (ec11eb3): if the mechanism is
the cause, the ground state reverts as lambda falls — gallium 2S+1 from 4 to 2, zinc
from 5 to 1. If it does not revert down to lambda = 0.12, the shared-exponent story is
WRONG and the high-spin result has another cause.**

`lambda = 1.0` is the control and must reproduce the declared reading (gallium quartet,
E = -1900.978972651). An earlier run confirmed that, and also read `lambda = 0.7` as
unchanged.

This is NOT a correction and NOT a proposal to change the basis. STO-3G's 4p exponents
are part of the declared model; a `lambda != 1` basis is a different model, computed only
to ask which of its features carries the effect. Nothing it produces enters a gate.

### The trap inside it, which cost two dead runs

`solve_basis` CANNOT be used here. It calls `solve`, which routes anything past
`MPS_ROUTE_THRESHOLD` to the MPS/DMRG path, and that path is measured to reach six
orbitals. Gallium's minimal sector is 124,848 determinants and its raised sector 55,080 —
both past the threshold, so both hang in the MPO builder rather than returning anything.
`solve_determinant` is the entry point with no threshold, and it needs the hand assembly
the example does. This is not visible at the call site and the shortcut looks strictly
better right up until it does not return.

The consequence is that the example runs on a Cholesky-orthonormalised basis WITHOUT the
SCF rotation — exact (full CI is invariant under orbital rotation) but slow, several
minutes per point. That is a cost, not a bias.

### Counterfactual result so far — the mechanism is looking FALSIFIED

    lambda   E (hartree)        <S^2>     2S+1   verdict
      1.00   -1900.978972651   3.750000   4.000  unchanged   (control, reproduces exactly)
      0.70   -1900.974063484   3.750000   4.000  unchanged   (from the earlier run)
      0.50   -1900.899658862   3.750000   4.000  unchanged
      0.25   -1900.707970083   3.750000   4.000  unchanged
      0.12   -1899.999959075   8.750000   6.000  ROSE to a sextet

Four readings, spanning a factor of eight in 4p exponent and 271 millihartree of energy
cost, and `<S^2>` has not moved by so much as a digit. The perturbation is biting — 4p is
genuinely occupied, or diffusing it would be free — and the quartet does not care.

The last point is the surprise: at lambda = 0.12 the multiplicity went UP, to a sextet.
It never fell at any lambda.

**On the pre-registered prediction, that falsifies the shared-exponent mechanism.** The
high-spin ground state at the 3d/4s boundary is real (`zn_diagnose`) and the explanation
I offered for it is wrong. It should be recorded that way: an ELEMENTS-3 finding about
STO-3G's edge, with a mechanism proposed, tested by its own counterfactual, and REJECTED.

What that leaves open, for whoever picks it up: if 4p's radial extent is not the cause,
the next candidates are the 3d shell (which for gallium shares its exponents with 3s3p,
not with 4sp) and the near-degeneracy of 4s with 4p in a minimal basis independent of how
diffuse either is. Both are testable the same way — scale one shell, leave the rest
declared, watch `<S^2>`. Neither has been tested.


### One more defect, in the instrument rather than the model

The `verdict` column in the run that produced the table above was ONE-DIRECTIONAL: it
asked only whether the multiplicity had fallen, so it printed "unchanged" against the
lambda = 0.12 row where 2S+1 had risen from 4 to 6. The numbers were right and the label
was wrong. Fixed to test both directions (commit 431b85d); the log carries a header
recording the wrong label rather than being regenerated clean.

Worth stating plainly because it is the same shape three times over in this project: a
check that can only see the direction it expects reports every other direction as nothing
happening. I built it into a discriminator whose entire purpose was to be two-sided.


## FINAL STATE — every gate has a verdict

Xe2 finished; the E1/E2/F1 gate came out of `tests/pending` in the same commit as its
data (4faf9d0), per that directory's third rule.

| gate | verdict |
|---|---|
| W1 | VERIFIED (not implemented — c03f282 was this lane's own swept work). Bit-identical over 18 atoms and 40 pair points; plant fires at 1.1e-2 Ha, silent below. |
| T1 | DISCHARGED. Tabulation pin + ratio band + universality; plants fire; the ratio band's floor is stated rather than tuned away. |
| R1 | DISCHARGED ON THE RESCOPE (A1.2, corrected by A3.1). Nine multiplicities from `<S^2>`, two sigma routes to 2.1e-14, 50-digit referee on Kr and Xe. Seven referee atoms OWED. |
| E1 | DISCHARGED. Kr and Xe one determinant; Kr2 and Xe2 unbound on the A3.2 grid. |
| E2 | DISCHARGED. 0.148293 > 0.145398 > 0.132360 Ha; Br2 binds. |
| F1 | FIRED, kept marked dead. Deficit falls and changes sign. |
| P1 | Rule shipped and declared (A2.1); label plant fires. Palette regeneration for d-bearing species is the lead's to sequence. |

26 gates in the ELEMENTS-3 set, all green.

## What is genuinely owed, so nobody reads the table above as "finished"

* **Seven of the nine referee-eligible atoms** have no 50-digit reference. Kr and Xe are
  reachable only because they are one determinant. The successor is the string-driven
  sigma rewrite in `mixtures1_referee/FEASIBILITY.md` — one successor, two campaigns.
* **The high-spin anomaly has no explanation.** Real by the best-determinant
  discriminator; the shared-exponent mechanism is falsified by its own counterfactual.
  Untested next candidates: the 3d shell (which for gallium shares exponents with 3s3p,
  not 4sp) and the 4s/4p near-degeneracy independent of diffuseness.
* **F1's counterpoise question is open.** `elements3_counterpoise.rs` is built and reads
  `R_e` from the now-complete record; it has not been run. If the raw deficit falls and
  the corrected one grows, F1 died of a contaminated observable rather than wrong
  physics — a different finding that does NOT inherit the dead stake's standing.
* **The determinant-route reachability boundary** above ~1e7 determinants is unmeasured
  (A3.1). Cobalt is plausibly reachable; yttrium is not.

## Three defects of this lane's own, with their fixes, so the pattern is visible

All three were the same shape — a check that could only see what it expected.

1. `Species::n_basis()` kept summing CARTESIAN components after the projection landed, so
   the registry said xenon was 29 functions while the engine assembled 27. Survived
   because the only covering test ran Z <= 18, which has no d shells.
2. AMENDMENT A1.2 read `feasibility`'s refusal — a statement about the AUTOMATIC route —
   as "unreachable". Corrected by A3.1 after the MIXTURES-1 lane measured SiO at 34
   seconds through `solve_determinant`.
3. The counterfactual's verdict column tested only for a FALL, and printed "unchanged"
   against a row where the multiplicity had risen from 4 to 6.

## The wasm stack trap was NOT W1 — diagnosis and verified fix

Reported to this lane as a W1 regression: the browser artifact traps with
`memory access out of bounds` on the cheapest solve, at the wasm-ld default 1 MiB stack,
where a pre-W1 build works. Native suites are fully green against the same source.

**It is not W1.** The bisect point used (1a13c49) predates both W1 and another lane's
f-shell work, which is UNCOMMITTED in the working tree — so a build from "current source"
contains both, and the comparison cannot separate them.

Two-sided, in a throwaway worktree at clean HEAD (which HAS the full widening,
`MAX_ORB = 64`, `Mask = u64`, `Det = u128`):

| build | stack | result |
|---|---|---|
| clean HEAD | 1 MiB default | smoke.mjs **fully green** |
| clean HEAD + ONLY `LMAX 2->3, IMAX 4->5, TMAX 12->16, RMAX 8->12` | 1 MiB default | **TRAP**, same function indices (348 … 127) |

### Why, with the arithmetic

`MAX_ORB` sizes NO stack array anywhere in `fci.rs` — it is an assertion bound and a
default argument, and the masks live in `Vec`s. The widening moves scalars from 4 to 8
bytes, and 8 to 16 on two checking routes the production path never calls.

`RTensor` is what overflows, through a construct that LOOKS like a heap allocation. Its
`work` field is already `Box`'d, but `Box::new(<array literal>)` materialises the literal
on the STACK before moving it, and `opt-level=z` does not reliably elide that:

    RMAX = 8  : r inline  17,496 B + Box::new temporary 157,464 B =  174,960 B
    RMAX = 12 : r inline  52,728 B + Box::new temporary 685,464 B =  738,192 B

738 KB of a 1 MiB stack in one struct construction, before any other frame — plus
`ETable` going 7,200 to 13,824 bytes with three live per primitive pair.

### The fix, verified green at LMAX 3 / RMAX 12 on the DEFAULT 1 MiB stack

    work: Box<[[[[D2; RMAX + 1]; RMAX + 1]; RMAX + 1]]>,        // boxed SLICE
    work: vec![[[[D2::c(0.0); RMAX+1]; RMAX+1]; RMAX+1]; RMAX+1].into_boxed_slice(),

`vec!` builds ONE element (53 KB) on the stack and clones it into heap storage instead of
materialising all 685 KB there. Indexing is unchanged. **No `-zstack-size` bump is needed**,
so the 1 MiB default survives as the only stack-growth detector the artifact has.

NOT applied to the shared tree: it is the f-shell lane's uncommitted work, and editing
another lane's in-flight file is the hazard rather than the fix. Diagnosis, arithmetic and
patch were sent to them and to the lead.

**The pattern worth keeping:** `Box::new(<big array literal>)` is a stack allocation
wearing a heap allocation's clothes, and it is invisible in review precisely because the
type says `Box`.

The wasm is reproducible from HEAD and correct — the clean-HEAD build passed the full
smoke, Cl2 refusing with 21 and all 122/82 engine calls resolving. Rebuilding the shipped
artifact is unblocked whenever the lead sequences it.

## RESOLVED — the stack fix and the artifact both landed

Commits 3b37b8e (fix) and 4536244 (artifact). The f-shell work was committed without the
fix, so a fresh wasm from HEAD trapped exactly as measured; the lead released the
editing restraint once it was committed.

* `RTensor::work` is now a boxed SLICE built through `vec!`, not a boxed array built
  through `Box::new(<literal>)`. Peak stack for that construction drops from 685,464 to
  52,728 bytes.
* `viewer/smoke.mjs` PASSES at the wasm-ld DEFAULT 1 MiB stack. No `-zstack-size`, no
  change to `build-web.sh` — the 1 MiB default stays as the only stack-growth detector
  the artifact has, which is what made this a loud failure rather than a quiet frame.
* The artifact and both `docs/` mirrors are rebuilt from HEAD, byte-identical across two
  clean rebuilds (sha256 42eb761f…), all three copies hashing the same. The
  reproducibility gap is closed.

## The successor named in A2.2 has LANDED — and the owed items are unblocked, NOT discharged

`bb1a07a` rebuilt the MPO builder on channels and `d5276f0` measured it on real STO-3G
integrals: SiO's MPO build went from "did not finish in 12 hours" to 0.07 s.

What that does and does not change for this campaign:

* **`pair::MPS_MAX_ORBITALS` is still 6.** It has not been re-derived against the new
  builder, so every route verdict in A1.2/A3.1 still rests on the OLD measurement. The
  sixteen route-less atoms are unblocked in principle and unmeasured in fact.
* Re-deriving that constant is the single cheapest thing that would move R1's scope, and
  it is a measurement, not an assumption — the same discipline that produced the 6.
* The 50-digit referee's seven owed atoms are a SEPARATE wall (an mpmath eigensolve, not
  an MPO build) and are unaffected by this; their successor is still
  `mixtures1_referee/FEASIBILITY.md`'s string-driven sigma rewrite.

Also closed: the Sc–Fe DMRG claim this lane logged neutrally in A1.2 has been
independently scrutinised from the other side (d5276f0) and reached the same verdict —
"five FIVE-ORBITAL Hubbard-Kanamori MODEL spaces… not scandium: not 21 electrons, not
STO-3G integrals, and an active space is not an atom." The discrepancy is resolved in
favour of the measurement.

## F1's counterpoise question — ANSWERED, and it CLOSES rather than rescues

`counterpoise.log`, run against the completed record at its own `R_e`:

| | D_e raw | D_e counterpoise | BSSE | deficit raw | deficit corrected |
|---|---|---|---|---|---|
| HCl | 0.148293 | 0.132164 | 0.016129 | +0.021415 | +0.037545 |
| HBr | 0.145398 | 0.122989 | 0.022409 | −0.001268 | +0.021142 |
| HI  | 0.132360 | 0.112306 | 0.020054 | −0.014836 | +0.005218 |

**Neither observable grows.** The corrected deficit falls too. So the
contaminated-observable rescue FAILS and F1 is dead on both — the question closes rather
than passing to a successor's stake.

Two things the correction does establish and that are worth keeping:

* the **sign change was a BSSE artifact** — corrected, the model underbinds all three, so
  "the model overbinds HBr and HI" was a property of the uncorrected observable;
* the **FALL is real and survives correction**, which is the part F1 got wrong, and it is
  not explained by basis-set superposition.

BSSE is 0.016–0.022 Ha and is not monotone (rises HCl→HBr, falls slightly at HI), so it is
not a clean function of partner size either.

## ELEMENTS3_RESULTS.md is written

`conformance/atomworld/ELEMENTS3_RESULTS.md`, SATURATION-1 pattern: full scorecard with
measured numbers, the two findings-not-passes sections, the defect ledger (four defects,
all the same shape), the route-scope table with its three separated criteria, and the
cross-lane items. The lead verifies it against the prereg and amendments.


## R1's record: indium did NOT converge, and the instrument did not say so

The detached run completed (`EXIT=0`), and one row was wrong in a way the record could not
have told you:

    49  In  27  1026675  -5677.203746332331   3.98e-1  4.216  ...  det (forced)

Residual **3.98e-1** against ~1e-10 everywhere else — nine orders above the crate's declared
`pair::CONVERGED_RESIDUAL` — so Davidson hit its 1200-iteration cap. Both the energy and the
multiplicity are meaningless. **2S+1 = 4.216 is the independent tell**: the Hamiltonian is
spin-free, so a converged eigenvector is a spin eigenstate and 2S+1 must be an INTEGER.
4.216 is not a multiplicity at all.

`elements3_atoms.rs` printed the residual and **checked nothing**, so the row appeared in a
column of measurements looking like one. That is precisely the failure
`CONVERGED_RESIDUAL`'s own doc comment describes — *"emitted looking perfectly healthy,
carrying a wrong energy, with the evidence sitting in a field no consumer is required to
read"* — and the crate had already written the constant for it. I did not use it. **Fifth
instance of the family, and the first where the fix already existed and was not reached
for.**

The instrument now applies both checks and REFUSES the row rather than printing it:

* `sol.residual <= CONVERGED_RESIDUAL`;
* 2S+1 integral to 1e-6 — free, and it catches the same failure for a reader who never
  looks at the residual column.

`atoms_unverdicted.log` preserves the original run. It is kept rather than deleted for the
same reason the counterfactual log keeps its wrong-label header: a regenerated-clean record
lies about its own history.

**Indium is therefore NOT a solved row.** The Davidson cap is `#[doc(hidden)]`
`DAVIDSON_MAX_ITER`, whose doc reserves it for `tests/front_door.rs` and says production
never touches it — so raising it to force convergence is not available to this record, and
"did not converge at the production cap" is the honest status rather than a number obtained
by reaching around the contract.


## Gate (2) is decided in advance: the MPS constant cannot reach my refusals

I am blocked on mixtures-engine's re-derived `MPS_MAX_ORBITALS` before the A4 route
rescope. While their run was in flight I read the ladder's design instead of waiting on its
answer, and the answer turns out not to matter for my table. Two reasons, both structural.

**Reach in orbitals.** `mps_ladder.rs`'s `LADDER` is seven two-centre pairs topping out at
S2. Sulfur is `s1/sp2/sp3` = 9 functions, so S2 is 18 orbitals. My route table's refusals
sit at 18 orbitals (Sc..Cu), 22 (Rb, Sr) and 27 (Y..Cd). Fourteen of the sixteen route-less
species are above anything the ladder can return.

**The 18-orbital rung is at the wrong filling.** S2 is 32 electrons in 18 orbitals:
C(18,16)^2 = 23,409 determinants — *germanium's FCI space exactly*, a row this record
already solves by determinant at 1.44e-14 residual. The refused rows share the orbital
count and nothing else:

| species | orb | elec | determinants | vs S2 |
|---|---|---|---|---|
| Sc | 18 | 21 | 1,392,554,592 | 59,500x |
| Ti | 18 | 22 | 1,012,766,976 | 43,300x |
| Cr | 18 | 24 | 344,622,096 | 14,700x |
| Fe | 18 | 26 | 73,410,624 | 3,140x |
| Cu | 18 | 29 | 2,496,960 | 107x |
| Zn | 18 | 30 | 665,856 | 28x |
| Ge | 18 | 32 | 23,409 | 1x (= S2) |

(Derived binomials checked against `atoms.log` — they reproduce its counts digit for digit,
which is what makes this an argument about the record rather than about arithmetic.)

So "18 orbitals" names two problems that differ by four and a half orders of magnitude, and
the ladder measures the cheap end. **This is not a claim that DMRG fails at mid-filling — I
have no measurement either way, and neither does the ladder.** That is the whole content:
the ladder's own header says a reach without a budget is not a measurement, and the same
rule binds on the electron-count axis. A new constant licenses the MPO *build*; it does not
license the *verdict*, because nothing measured the verdict there. Third axis, flagged and
rested on nothing: every ladder rung is a two-centre molecule and every refused row is a
one-centre atom, where a 1D ansatz has no locality to exploit.

**Consequence for A4, fixed before the number arrives** (so the rescope cannot be tuned to
whatever lands): route labels move from "no automatic route" to "automatic route available"
for species at or below the new constant, and nothing else moves. No refused row becomes a
measured row. If the number comes back below 18 the table does not change at all.

**Offered to mixtures-engine:** the missing mid-filled rung exists in this record. Zinc is
18 orbitals, 30 electrons, 665,856 determinants, 28x S2 — and an exact reference is
computable there where at 1.4e9 it is not. Caveat travels with it: zinc is the row the
convergence gate REFUSED at residual 2.72e-10 against a 1e-10 bar. Different quantity from
their 1e-8 energy stake and comfortably inside it, but it is not handed over as clean.
Copper (2,496,960 det) is the next rung if they want more separation.

### Correction, same day: the filling axis is measured now, and it breaks the constant

Forty minutes after I wrote that no measurement bore on the filling axis, mixtures-engine's
ladder produced the rung that does:

    HCl   10 orb   18 elec       100 det   chi 32   +6.122e-11   55.9 s   REACHED
    NaH   10 orb   12 elec    44,100 det   chi 32   +4.391e-3   396.2 s   BUDGET

Same orbital count, 441x apart in determinants, opposite verdicts, NaH short by five orders.
The results doc carries this as a correction to my own sentence rather than as a quiet
strengthening — it was true when written and false shortly after, and which of those it was
is part of the record.

**The consequence for their constant, sent to them:** `mps_ladder.rs`'s closing line
nominates `best_reached` (largest orbital count that reached) as the new
`MPS_MAX_ORBITALS`, but `pair.rs:1014` uses the constant as `n_orb <= MPS_MAX_ORBITALS` — a
door admitting everything at or BELOW it. With a failure at 10 and a success at 10, and any
reach at 14 or 18 above them, the printed constant would open a door over a measured
failure. A maximum over a set containing a failure is not a bound on that set. For this use
the honest number is `first_wall - 1` = **9** on current data, and their code already
computes and prints `first_wall`; it is the closing sentence that picks the wrong one.

Same defect family as the one-directional checks in the ledger: `best_reached` can only see
the direction it expects, and the failures at and below it are invisible to it.

**This lane's reading rule, now in the results doc:** read the constant as the wall minus
one; if the ladder's two numbers disagree, report the disagreement rather than resolve it in
this lane's favour.

### And the constant turns out to be inert: the arm it guards is unreachable

Chased down whether a raised `MPS_MAX_ORBITALS` would reroute HCl and break the dimer
record's bit-identity gate. It would not — `automatic_route` tests determinant count FIRST,
HCl is 100 determinants, so it stays on the determinant route whatever the constant says.
But the same reading found something larger:

    if n_det <= MPS_ROUTE_THRESHOLD   { Determinant }    // 50,000
    else if n_orb <= MPS_MAX_ORBITALS { Mps }            // 6
    else                              { NoneAvailable }

Both conditions must hold to select `Mps`. Six orbitals admit at most C(6,3)^2 = **400**
determinants against a threshold of 50,000, so anything inside `MPS_MAX_ORBITALS` is already
inside the determinant threshold. **The `Mps` arm cannot be selected for any input.** No test
constructs it; `holon-render/src/lib.rs:1563` maps it to a viewer route code that can never
be produced.

**My own second error in that paragraph today.** ELEMENTS3_RESULTS.md said my sixteen
refusals "rest on a superseded measurement". They rest on `MPS_ROUTE_THRESHOLD` alone.
`MPS_MAX_ORBITALS` enters no verdict this record contains. Corrected in the doc as a
correction.

Max determinants by orbital count — 6:400, 8:4,900, 9:15,876, **10:63,504** — so the arm
first goes live at ten orbitals, and at exactly ten the window is n_det in (50,000, 63,504],
which is half-filling and nothing else. **The first spaces a raised constant would route
automatically to MPS are ten-orbital half-filled ones — the immediate neighbourhood of NaH,
the one measured failure.** Raised to 18 instead, it flips Sc–Cu to "automatic route
available" in one step, on a rung measured at germanium's filling.

**Owed here:** a gate in `tests/elements3_atoms.rs` that my route verdicts are functions of
the determinant threshold and invariant to `MPS_MAX_ORBITALS` at its present value — the
fact the published table now rests on. Deliberately NOT written yet: compiling contends for
CPU with the indium run I am waiting on, and it lands in the same suite pass that verifies
the regenerated record. The broader exclusivity gate belongs in pair.rs and was offered to
mixtures-engine rather than taken.

### PRE-REGISTERED, 2026-08-30T20:27:08Z, before SiO and S2 printed

    STAKED BEFORE THE ROWS EXIST -- the ladder is mid-run, SiO and S2 have not printed.
    
    Ladder so far:
      H2    2 orb        4 det  REACHED
      LiH   6 orb      225 det  REACHED
      HCl  10 orb      100 det  REACHED
      NaH  10 orb   44,100 det  BUDGET   <- the only failure
      ClF  14 orb      196 det  REACHED
    
    Every REACHED has <= 225 determinants. The single BUDGET has 44,100. Across the successes
    orbital count spans 2..14 and predicts nothing; determinant count separates them perfectly.
    
    TWO HYPOTHESES, and the two unrun rungs split them in OPPOSITE directions:
    
      H_orb : the verdict tracks ORBITAL COUNT (the constant's own axis)
      H_det : the verdict tracks how correlated the space is, for which determinant count is
              the crude proxy and nonzeros the better one
    
      SiO   14 orb  132,496 det    H_orb: REACHED (same 14 as ClF)   H_det: BUDGET
      S2    18 orb   23,409 det    H_orb: BUDGET  (highest in ladder) H_det: REACHED
    
    I stake H_det: SiO BUDGET, S2 REACHED.
    
    WHAT EACH OUTCOME MEANS, fixed now so no result can be reinterpreted after the fact:
    
     (a) SiO BUDGET and S2 REACHED -- H_det confirmed on its own pre-registered discriminator.
         The ladder's highest orbital count reaches while a lower one fails, so orbital count
         is demonstrably not the axis. best_reached = 18 with measured failures at 10 and 14
         INSIDE the door that constant opens. The strongest form of the defect.
    
     (b) SiO REACHED and S2 BUDGET -- H_orb confirmed, my argument is wrong, and the constant's
         axis is sound. I report that as plainly as I would report (a), and the wall-minus-one
         rule becomes the whole of my correction rather than the axis claim.
    
     (c) both REACHED -- no discrimination from this pair; NaH stands alone and the axis
         question stays open. My reading rule survives on NaH alone, the axis claim does not.
    
     (d) both BUDGET -- consistent with H_det on SiO but S2 confounds with per-sweep cost at
         18 orbitals; NOT support for H_det, and I will say so.
    
    The confound I can already name: bigger orbital count means slower sweeps, so a BUDGET at
    high n_orb is ambiguous between "harder state" and "ran out of clock". This is why ClF vs
    SiO is the load-bearing comparison -- SAME orbital count, 676x apart in determinants, so
    per-sweep cost is held fixed and only correlation varies. S2 is the weaker leg and I am
    grading it as such in advance.

### Staked leg one CONFIRMED: SiO BUDGET at the same orbital count where ClF reached

The prediction committed at e16acb7, before either row existed, was SiO BUDGET and S2
REACHED. SiO has printed:

    ClF   14 orb      196 det   chi 32   +5.457e-12   5 sweeps   317.3 s   REACHED
    SiO   14 orb  132,496 det   chi 32   +1.118e-2    6 sweeps   587.3 s   BUDGET

**Same orbital count. 676x apart in determinants. Opposite verdicts.** SiO stopped six
orders from the 1e-8 stake. This is the leg I named load-bearing in advance, precisely
because holding orbital count fixed holds per-sweep cost fixed and leaves correlation as the
only thing varying. It is now the SECOND same-orbital-count pair to split this way, after
HCl (100 det, REACHED) against NaH (44,100 det, BUDGET) at 10 orbitals.

Two independent orbital counts, two splits, both in the same direction: the verdict tracks
how correlated the space is, not how many orbitals it has.

**S2 is still pending and it is the weak leg**, graded weak before the run and still graded
weak now: at 18 orbitals a BUDGET confounds a harder state with slower sweeps, so only a
REACHED there is informative for me. Not grading the stake until it prints.

### The log I cited for SiO was overwritten by a restart — treating the rerun as replication

At 15:42 `mps_ladder.log` went from seven data rows back to three: mixtures-engine restarted
the ladder (new process 1000621 under `timeout 21600`, fresh log beginning again at H2, with
different timings — LiH 2.8 s against 2.4 s, HCl MPO 0.02 s against 0.01 s, so a new run and
not a resumed one). The header is byte-identical in every parameter that matters: stake 1e-8,
budget 300 s per cell, chi ladder [32,64,128,256,512], sweeps in chunks of 3, tol 1e-11.

**So the SiO row I cited as confirming leg one no longer exists in any log.** What survives
is my transcription of it, in the section above and in commit fd88711:

    SiO   14 orb  132,496 det   chi 32   +1.118e-2   6 sweeps   587.3 s   BUDGET

A transcription is not the artifact. By this lane's own rule — a cross-reference needs a
target, and a claim is checked against the primary record rather than against my report of
it — that citation is currently unsupported, and it is unsupported in the direction that
favours me, since SiO BUDGET is the result I predicted.

**The repair is better than the original**, and costs nothing but patience: the restart runs
the same configuration over the same ladder, so it will reach SiO again. That makes the
rerun a REPLICATION of the staked leg rather than a reconstruction of it. Holding the
confirmation as provisional until the new run's SiO row prints:

* if it comes back BUDGET, leg one is confirmed twice, on two runs, and the transcription is
  validated by something other than my own note;
* if it comes back REACHED, my transcription or my reading of it was wrong, and I report that
  the staked leg FAILED — the same way I committed to reporting it before any row existed.

The stake itself is untouched by the restart. It named SiO and S2 as pairs, not as rows of a
particular run, and all four outcomes still carry the meanings fixed at e16acb7.
