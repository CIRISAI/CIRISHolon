# The DRY-residual register — every irreducible special case the build forced

*WB-8.7, the operator's law of 2026-09-01: the tower is not preparation for the
experiment, it IS the experiment. Every special case, hardcoded branch or
per-composition carve-out the build FORCES us to write is a measurement against
the maximal claim — a witness pair at the architecture level. This register is
where those measurements are banked, and its GROWTH RATE against domain size is
the claim's live falsifier: short and closed = the fold is winning; growing with
the domain = the DRY is wrong, said quantitatively.*

Kept beside `MISFITS.md` because it is the same kind of object one level up: that
registry names defects in how we MEASURE, this one names defects in how the
object FOLDS. An entry here is not a bug report and not a TODO. It is a place the
fold did not close, with the reason it did not, and the condition under which it
would.

**How to read the count.** The falsifier is a rate, not a level, so both numbers
have to be recorded at every reading or the rate is unrecoverable.

| reading | date | materialized carriers | open residuals | closed since last |
|---|---|---|---|---|
| 1 | 2026-09-01 | 3 (C0, C1, C2) | 11 | — (first reading) |
| 2 | 2026-09-01 | 3 + the GPU device class as a second CARRIER of the same solve | 13 | 1 (see below) |

Reading 1 is the baseline: it is not evidence for or against the claim, it is the
zero against which the next reading is a measurement. The next reading is owed at
the next materialized carrier or the next composition added to the water domain,
whichever comes first.

---

## Open residuals

### R-1 — the MPS contraction is written twice, once real and once complex

`q8-mps`'s sweep engine is `f64` throughout (`mps::TensorSite`, `grow_left_mpo`,
`apply_effective_h_mpo`), and real-time dynamics needs complex amplitudes, so
`tdvp.rs` carries its own environment growth and its own effective-Hamiltonian
application. The two are the same contraction with a different scalar.

**Why it did not fold.** The crate has zero runtime dependencies by a
load-bearing rule, so there is no numeric trait to be generic over that it did
not write itself; and making the ground-state path generic would edit the live
full-grid instrument, which `M-STALE-INSTRUMENT` says you do not do while it is
running.

**The fold that exists and was not used.** This engine already HAS the pattern —
the Scalar seam (`holon-chem/src/scalar.rs`: one solver body, `f64`/`Dd`
carriers, promotion as explicit transport, mixing a type error), which
`tower.rs`'s own header cites as the horizontal axis's existence proof. That the
C2 build did not reuse it is the honest content of this entry.

**Exit.** Generalise the sweep engine over the Scalar seam once the full grid
clears; the C2 module then loses roughly 300 lines and the residual closes.
**Owner:** the C2/tower lane.

### R-2 — C0's and C1's prices are invented; only C2's is measured

`C0_ClassicalBO::price_per_substep` returns `1e-6 · N²` and
`C1_RingPolymer::price_per_substep` returns `1e-6 · P · N²`. Neither constant was
ever measured. `C2_MpsTdvp::price_per_substep` is a measurement with its four
data points and its spread recorded (`C2_TDVP_RESULTS.md` §5).

**Why this belongs here rather than in `MISFITS.md`.** Selection in this tower IS
the corridor rule — argmin price subject to the budgets, proved in
`lean/CIRISHolon/Carrier.lean` §5 — so an unmeasured price is not a cosmetic gap:
it is a free parameter inside the only thing that chooses which carrier runs. Two
of the three prices being decorative makes `select_min`'s guarantee vacuous on
those two arms.

**Exit.** Measure both the way C2's was measured, with the spread and the
placement caveat (`M-PLACEMENT-LOTTERY`) stated. **Owner:** the C1 lane, with
C1's ZPE gate.

### R-3 — physical dimension 2 is hardcoded through the MPS stack

`TensorSite`, `CTensorSite`, `MpoSite` and every contraction in `q8-mps` assume a
two-state physical index. Correct for spin-orbitals and wrong the moment a
carrier needs a bosonic mode, a spin-1 site, or a coarse-grained multi-level
site.

**The fold that exists.** WB-8.2 already made exactly this move once, and it is
the tower's own idiom: `AngularShell { l: u8 }` replaced the S/P/D/F enum, on the
principle that *Z prices, Z never branches*. `d` should be a value for the same
reason `l` is.

**Exit.** When a carrier needs `d > 2`. Registered now rather than then, so that
the entry's AGE is visible if it sits here while the domain grows around it.
**Owner:** unassigned — and that is recorded, not hidden.

### R-4 — the corridor rule hardcodes exactly two budgets

`TheoryNode` carries `closure_budget` and `conservation_budget`, and
`select_corridor` tests exactly those two. The house rule is one gate per
conservation law; a third conserved quantity therefore needs a third field and a
third branch, in Rust and in the Lean both (`Node` in `Carrier.lean` §5 mirrors
the same two).

**Why it did not fold.** A budget vector is the obvious generalisation and it was
not written, because the Lean side's `Admissible` becomes a fold over a list and
the three corridor theorems' proofs change shape. That is a real cost and it was
not paid; it is not a reason the fold is wrong.

**Exit.** The third conserved quantity. **Owner:** the tower lane.

### R-5 — the planted-defect switch lives in production code

`tdvp::Mutation` is a four-variant enum branched on inside `Tdvp::step`, in
shipping code, for gate use only.

**Why it stays.** The alternative is a duplicated mutant integrator, which drifts
from the real one and then stops testing it — the defect that makes a mutation
battery reassuring instead of informative. Branching the real integrator is the
cheaper wrong thing, and it is entered here so the register's count is honest
rather than flattering.

**Exit.** None proposed. This entry is expected to stay open, and an entry that
is expected to stay open must say so, or the register's growth rate silently
counts it as a pending win.

### R-6 — the register's own grep-arm is not built

WB-8.7 clause (2) requires the grep-armed audit to extend to code: a hardcoded
species or composition branch must cite either its fold or its residual entry, or
the gate refuses. `Audit/prereg_audit.py` does this for freezes against
`MISFITS.md`; nothing does it for code against this file.

**Consequence, stated plainly.** Until it is built, this register is populated by
hand, so its count measures what someone remembered rather than what the tree
contains — and the falsifier in the header is therefore weaker than it reads. A
register whose coverage is invisible drifts behind the thing it registers, which
is the same finding that dated `prereg_audit.py`'s own CONTACT table.

**Exit.** A `Audit/dry_audit.py` that greps the engine for
composition-conditioned branches and requires a citation. **Owner:** the tower
lane.

### R-7 — bond-orientational order is written twice, once for the plane and once for space

`holon-lens::lens` carries `steinhardt_q` (refuses 2D) and `hexatic_psi6`
(refuses 3D), and `q_tetrahedral` alongside them. Three functions where the
object has one question: how orientationally ordered is a neighbourhood.

**Why it did not fold.** The 2D and 3D scenes arrived from different tiers and
only the 2D one had data, so the general construction was never forced.

**The fold that exists.** Harmonics on `S^(d−1)`, with `psi6` the `d = 2`
instance of the same sum that gives `q6` at `d = 3`. Standard mathematics, not
research; this is a debt, not a wall.

**Exit.** The first campaign that needs both dimensions at once.
**Owner:** the closure-census lane.

### R-8 — the size of a "first shell" is a hardcoded pair of integers

`classifier::classify` uses `want = if dims == 2 { 6 } else { 12 }`.

**The fold.** The first coordination number read off the scene's own radial
distribution function rather than off a table of lattices.

**Why it did not fold.** A twelve-atom scene has too few atoms for a radial
distribution function to have a resolvable first minimum. The fold needs the T3
scale-up, which is exactly the axis WB-8.7 asks to be measured.
**Owner:** the closure-census lane, after T3.

### R-9 — a second periodic table, four entries long

`partition::symbol` maps nuclear charge to `H`/`C`/`N`/`O` and everything else to
`X`. WB-8.7 clause (2) names per-species branches specifically.

**The fold that exists.** `holon_chem::elements` already holds the table this
duplicates.

**Why it did not fold.** `holon-lens` is deliberately zero-dependency so that it
stays testable while `holon-render` is mid-refactor — and it WAS written during
exactly such a window, with its whole suite green on a tree where `holon-render`
did not compile. The isolation bought something real; this table is its price.

**Exit.** Fold when `elements` is split into a `no_std` leaf, the way
`holon-device` was split out for the same reason one tier down.
**Owner:** the closure-census lane.

### R-10 — the trajectory artifact caps scenes at sixteen atoms

`traj::MAX_DUMP_ATOMS = 16`, because `C(16,2) = 120` pair bits fit a `u128`.

**Why this one is the register's own shape.** The engine STOPPED capping scene
size at T3 (`DEFAULT_SCENE_ATOMS` replaced `MAX_ATOMS`), so the artifact now
carries a limit the object does not: a representation constraining a thing that
has outgrown it.

**Exit.** A variable-length bitset sized by the scene, as the engine's own pair
sector already is. Forced by the first scene above sixteen atoms, and it should
be folded BEFORE then rather than by the failure.
**Owner:** the closure-census lane.

### R-11 — a geometric centroid where the physics wants a mass-weighted one

`census::carrier_motion` takes an unweighted centroid because the trajectory
header carries nuclear charge, not mass.

**Bounded rather than load-bearing, and the bound is stated.** The affected
quantity is the split between translation and internal motion. The second motion
statistic the same gate uses — the intra-block separation excursion — is
centroid-free and settles a frozen carrier on its own, so no verdict rests on the
approximation alone. Declared in the function's own doc comment as well as here.

**Exit.** Carry mass in the header beside `Z`.
**Owner:** the closure-census lane.

### R-12 — the same-spin matrix is built twice, once streamed and once whole

`holon_chem::tier::sigma_direct_t` builds the same-spin matrix `F` one ROW at a
time inside its streaming loop, reusing a single `f` buffer and never
materialising the matrix. `holon_gpu::fci::build_same_spin` builds the whole
`nb × nb` and `na × na` matrices so they can be uploaded once and re-used for
every application. Same algebra, two constructions.

**Why it did not fold.** Materialising `F` inside `sigma_direct_t` would change
its memory behaviour — that streaming is the reason the hand-written CPU kernel
beats the same reformulation through OpenBLAS, which pays 372 MB of bandwidth per
sigma to materialise the intermediate (SATURATION-3 G2). It could also move
trailing bits, and every committed table is keyed on those.

**Why the duplication is not silent.** It is GATED by a measurement, not by
hope: the device sigma must reproduce `sigma_direct` to 1e−12 relative before any
timing is reported, and it does, at **3.033e−15** on the real `(O,O,O)` problem.
A divergence between the two constructions fails that gate.

**Exit.** Expose one builder from `holon-chem` that the streaming path consumes
row-wise and the device path consumes whole — a shape the crate already has for
`Scalar`. Worth doing when a second device arm needs `F`, and not before: one
consumer does not justify an abstraction, and the gate is doing the work
meanwhile. **Owner:** the gpu-production lane.

### R-13 — `SigmaOp` is generic over the scalar and `SigmaProvider` is `f64` only

`holon_chem::sigma_op::SigmaOp<T: Scalar>` is generic, matching the solver's own
`Scalar` seam. `SigmaProvider` — the thing that binds a whole solve to one device
class — is `f64` and nothing else, so `solve_determinant_with` is `f64`-only
while `davidson_eigh_from_op` is not.

**Why it did not fold.** The device arm is `f64`: consumer Ada runs FP64 at 1/64
of FP32 and there is no `Dd` GPU kernel. A `Scalar`-generic provider would
advertise a device tier that does not exist, which is worse than an asymmetry —
it is a type that lies about what can be built. The double-double rung is a CPU
rung of D3b's ladder and reaches the solver through
`tier::refine_determinant_dd`, not through a provider.

**Exit.** A second device scalar. Registered now rather than then so the entry's
AGE is visible if the asymmetry outlives its reason. **Owner:** unassigned, and
that is recorded rather than hidden.

---

## Closed since reading 1

**C-1 — the P/E core split is no longer a hardcoded per-machine branch.**
`fci_bench` first carried `if cpu < 16 { "P" } else { "E" }`, which is exactly
the shape WB-8.7 clause (2) refuses. It now reads
`/sys/devices/cpu_{core,atom}/cpus` and CROSS-CHECKS the result against
`MISFITS.md`'s M-PLACEMENT-LOTTERY entry, refusing the run if the machine and the
citation disagree — because on a different box that entry's labels do not apply
and a row labelled from it would be wrong. Derived from the machine, checked
against the citation, and refusing rather than guessing: that is the fold, and it
cost about thirty lines.

## Unevaluated claim-surface (WB-8.7 clause 3)

Not residuals — machinery that does not exist yet, listed because unbuilt
machinery is unevaluated claim-surface and the standing rule that machinery debt
gates campaigns is a requirement that the instrument be complete enough for its
reading to mean something.

* **The C1 → C2 climb.** C2 is a certified node and is not reachable as an edge:
  no state lift, no operator picture change, no measured commuting certificate
  (`c1_to_c2_transport_capability`, a typed stub).
* **Two-site TDVP.** Single-site TDVP cannot grow a bond, so C2 cannot be handed
  a rank-deficient start. The discharge route is 2TDVP and it is not built.
* **C3+ (spinorial/Dirac, QED).** Typed stubs with fences, honestly not on the
  water path.

The founding case for why this list is kept: the dE₄ incident, where an unfolded
path concealed an 80× price surprise precisely because nothing had forced its
shape into the open.
