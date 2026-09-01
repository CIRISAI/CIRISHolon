# The DRY-residual register — every special case the build FORCED us to write

*Opened 2026-09-01 by the T3 engine-core lane, under FSD-W1 **WB-8.7**: "every irreducible
special case is entered with its reason, and the register's GROWTH RATE against domain size
is the claim's live falsifier: short and closed = the fold is winning; growing with the
domain = the DRY is wrong, said quantitatively."*

This is the sibling of `conformance/gravity/MISFITS.md` and it is read the same way: a row
is a MEASUREMENT, not an apology. The misfit registry records defects the campaigns
produced; this one records places where the architecture's own claim — that the vertical
quotient-by-scale and the horizontal refinement-of-carrier fold covers the domain — did not
reach, and something had to be written by hand.

**How a row earns its place.** A special case goes here when it is (a) a branch on a
specific species, composition, boundary kind, or scale band, or a duplicated
implementation of one idea, AND (b) not removable by the fold as it currently stands. A
branch that IS removable is not registered; it is removed, and the removal is the entry in
the commit message. Two rows below (R-1, R-6) began as candidate residuals and were folded
instead — recorded here as folds, because a register that only ever grows cannot tell you
whether the fold is winning.

**The falsifier, stated as a number.** The domain size is the count of distinct
(composition, carrier, scale band) triples the engine serves. The claim is that residuals
grow SUBLINEARLY in it. Both counts are recorded at the bottom of this file on every
update, so the ratio is a series rather than an impression.

---

## Open residuals

| id | the special case | where | why it could not be folded | what would discharge it |
|---|---|---|---|---|
| **R-2** | The pair sector's cutoff is OPT-IN and truncating; the three- and four-body sectors are cutoff-local unconditionally and exactly. One idea — "interactions are local" — with two implementations. | `holon-render/src/cells.rs`, `sim.rs::compute_forces` | Not a code accident: the sectors genuinely differ. `TrimerTable`/`WaterTable`/`OohTable`/`OzoneTable` return an EXACT zero outside their domain, so skipping a distant triple computes the same number for less. `PotentialTable` continues past its last knot as `hi_a·exp(-hi_b·dr)`, which is never zero, so skipping a distant pair DROPS ENERGY. A single unconditional cutoff would silently truncate the pair sector; a single opt-in one would leave the many-body sectors paying `O(N³)` for exact zeros. | A pair curve with compact support — i.e. a table that declares its own truncation radius and the error it accepts there, the way the three-body tables already declare `R_HI`. Then both sectors are "local because the table says so" and the branch disappears. |
| **R-3** | `fenced_triples` returns 0 when only the (O,O,H) or ozone surfaces are loaded and neither H₃ nor H₂O is, even though such a scene has fenced compositions. | `holon-render/src/sim.rs::fenced_triples` | It reproduces the pre-T3 early return in `accumulate_three_body` exactly. The fence incidence is a PINNED CAMPAIGN NUMBER (frozen P2: "the four OOO triples stay HONESTLY FENCED at exactly 4/seed"), and the T3 change was supposed to move the enumeration's cost, not its readings. Making the count correct here would have moved a number for a reason unrelated to T3, inside the same commit. | A separate, declared change that fixes the early return and re-pins the fence incidence — with the ozone arm's rerun, where the fence is expected to move anyway. |
| **R-4** | The three-body composition dispatch is four hardcoded branches: `n_h == 3 → trimer`, `n_o == 1 && n_h == 2 → water`, `n_o == 2 && n_h == 1 → ooh`, `n_o == 3 → ozone`. | `holon-render/src/sim.rs::accumulate_three_body`, `::served` | The general door already exists — `TrimerBank::find([za, zb, zc])` is composition-keyed and is tried FIRST — but the four in-memory surfaces predate it and are not expressed through it. They are generated in-process (`generate_trimer_table`, `generate_water_table`) rather than loaded as provenanced artifacts, so they have no `TrimerProvenance` to be admitted by. | Emitting the four in-memory surfaces through the same provenanced door the shipped ones use. Then the dispatch is one lookup and the branches go. Note the shape is already halved: `served()` and the force loop share one statement of what is served, so the fence and the forces cannot disagree. |
| **R-5** | The four-body sector is (O,H,H,H) only, by hardcoded `z == 8` / `z == 1` tests, and the hub is assumed to be the oxygen. | `holon-render/src/sim.rs::accumulate_four_body` | There is exactly one four-body surface (`holon_chem::quaternary::de4_ohhh_fci`) and it takes its arguments in that order. There is no four-body bank to key by composition, so there is nothing to dispatch through. | A four-body bank with the three-body bank's shape: composition key, declared domain, provenance gate. R-5 and R-4 discharge together or not at all. |
| **R-8** | The C1 carrier's own operators cannot carry the engine's own banked surface, so C1's physics computes pair energies and forces DIRECTLY from a `&dyn Pes` instead of through `RingPolymerOp`/`ClassicalPotentialOp`. Two statements of "the energy of this carrier's state". | `holon-chem/src/rpmd.rs::{pair_forces_3d, ring_energy_3d}` vs `tower.rs::{ClassicalPotentialOp, RingPolymerOp}::evaluate_energy` | The carrier's operators hold `Option<fn(r: f64) -> f64>` — a BARE function pointer, which cannot close over state. So no potential that owns anything can be transported through the carrier's declared operator: not the banked cubic-Hermite table (it owns a `Table`), not a `PairCache`, not a species-keyed bank. The C1 gate needs ~10^9 evaluations and the exact solver costs 64.5 µs a call, so it MUST use the table, and its physics therefore goes around the operator it is supposed to be the physics of. The same type also makes the derived `PartialEq` meaningless, and rustc says so on every build: *"function pointer comparisons do not produce meaningful results since their addresses are not guaranteed to be unique."* | Change `pair_energy_fn` to a shared trait object (`Arc<dyn Pes>`) and replace the derived `PartialEq` with an explicit identity. `ring_energy_3d` then becomes `P * op.evaluate_energy(state) + kinetic` — one statement — and nothing else moves: the compile-fail doctest and the transport tests never compare operators. Owned by the tower lane, because it moves a public type in `tower.rs`. |
| **R-7** | `Atom::mass()` branches on `species.z == 1` and returns the rounded constant `M_H = 1837.152` instead of `species.mass_me()`, which every other element gets. | `holon-render/src/sim.rs::Atom::mass` | PRE-EXISTING, not introduced by T3, and it is not a no-op: `HYDROGEN.mass_u * M_E_PER_U = 1.00782503207 × 1822.888486 = 1837.15268…`, so the branch changes hydrogen's mass by 6.8e-4 mₑ, a relative 3.7e-7. Removing it would move every hydrogen trajectory in the campaign and break the standing bit-identity gates (`tests/mixtures.rs::b1_all_hydrogen_is_bit_identical_to_the_single_table`), so it cannot be folded inside a scale-up commit. | A declared re-pin: delete the branch, re-bank the B1 reference dump, and state the 3.7e-7 mass change in the commit. Worth doing on its own, not as a side effect. See also the note below — the field it reads is DOC'd as a nuclear mass and HOLDS an atomic one. |
| **R-12** | The PAIR sector has two doors to one curve, and one of them is a hardcoded H₂ special case. `holon_table_generate` reaches H–H through `holon_chem::stream_table`'s bespoke s-only path; `holon_bank_generate_pair` reaches the SAME curve through the general N-centre route. One idea — "the pair potential of species (A,B)" — with two implementations, one species-specific. | `holon-render/src/lib.rs::generate_table` vs `::holon_bank_generate_pair` | MEASURED, and the number is why this is a residual rather than a tidy-up: in Chromium on the development machine the two doors cost **0.16 ms/knot** and **~0.5 s fixed + 58 ms/knot** for physics that agrees on R_e and D_e to the six digits the workbench page displays — about **90×** at the 160 knots a page load uses (76 ms against 7.0 s). The workbench cannot simply drop the fast door: 7 s of a main thread for a curve already available in 76 ms fails M-CHEAPER-THAN-ITS-PRICE as a runtime law. It cannot drop the slow one either, because that is the only door for every pair that is not H₂. `lib.rs`'s own header already records that the two doors differ in PROVENANCE (`stream_table`'s `Meta` carries no residual, so the uncertainty it stamps is a declared zero standing in for an unmeasured quantity) — the price gap is a second way they are not one implementation, and it was not recorded anywhere until the workbench timed them. | Give `holon_chem::table::Meta` a measured residual and route H₂ through `generate_pair_table` like everything else, OR keep the fast path and express it as a cache/fast-route INSIDE the general door, so callers see one door and the special case is an optimisation the door owns rather than an API the caller must choose between. The second is cheaper and removes the caller-visible fork, which is the half that bit here. |
| **R-13** | TWO INTEGRATORS. `Sim::step` is velocity Verlet on the physical Hamiltonian; `Sim::step_npt` is the MTK Trotter factorization on an extended one whose box is a degree of freedom. One `step`, two bodies, chosen by `barostat_on()`. | `holon-render/src/sim.rs::step`, `barostat.rs::step_npt` | Writing NPT as a special case of NVE is precisely how a barostat becomes a rescale hack (WB-7.2's `P^-0.05`), so the duplication is deliberate. The relation that DOES hold is proved rather than assumed: at infinite barostat mass with the chains idle, MTK reproduces Verlet BIT FOR BIT (`tests/t3_barostat.rs::npt_reduces_to_nve_at_infinite_barostat_mass`, worst coordinate difference 0.0 over 320 steps), because every barostat factor is an exact 1.0 in that limit. | Running NVE THROUGH the MTK path with the barostat frozen. It is bit-identical, so it would be sound — and it is not free: the NVE path would then compute three exponentials and two chain updates per step in order to multiply by one, and every banked campaign run would have to be re-pinned against the new call sequence. A COST decision, not a correctness one, and it should be made when someone measures the cost. |
| **R-14** | `holon-md` depends on `holon-tables` for ONE type, `WorkerProbe` — a table-generation crate pulled into a molecular-dynamics driver for twenty lines of thread probing. | `holon-md/Cargo.toml`, `holon-tables/src/worker.rs` | `holon-resource` deliberately supplies no worker probe ("the pool owner supplies one"), and `holon-tables` is where the workspace's first pool owner wrote it. A second implementation of "can the OS give me a thread" in `holon-md` would be a duplication with no reason behind it, so the dependency is the LESSER residual — but the probe's own refusal message still reads "the table mesh probes workers only", which is now false at one of its two call sites. | Move `WorkerProbe` into `holon-resource` behind a `std` feature, or into a small crate both depend on, and re-word its refusal. Blocked only on `holon-resource`'s zero-dependency stance, which a feature gate preserves. |
| **R-15** | THE REGISTER THAT MEASURES DUPLICATION EXISTS TWICE. `conformance/gravity/DRY_RESIDUALS.md` and `conformance/water_observatory/DRY_RESIDUALS.md` are both tracked, both opened under WB-8.7, both carry the same headline claim, and both number from R-1 — so R-1…R-13 name two rows each. | the two files | Not a code residual and it is registered anyway, because WB-8.7's definition does not say "in the engine": a special case the build forced us to write, whose growth rate is the falsifier. Two registers is one idea with two implementations, which is the exact shape every row below it describes — and it is the shape in the instrument rather than in the thing measured, which makes it the more serious kind. It could not be folded on the day it was found because the two files have different owners and a third lane rewriting a file two lanes already collided in is how this whole sequence started; `tower-complete` deliberately left it for an owner rather than take it, which was right. | ONE register, or ids namespaced by domain (`GRAV-R-9`, `WO-R-9`) with each file's header saying which it is and pointing at the other. The prefix half is done above as the cheap immediate fix; merging is the real discharge and needs both owners. Until then the SERIES readings at the foot of each file are computed over different domains and are not comparable — which is the concrete cost, not a tidiness argument. |


## Citing a row from this register: use the `WO-` prefix

**There are TWO tracked DRY registers in this tree and they both number from R-1.**
`conformance/gravity/DRY_RESIDUALS.md` runs R-1…R-13 and this one runs R-1…R-15, so every
bare `R-N` for N ≤ 13 names two different rows. Right now "R-9" is this file's ring-polymer
propagator fold AND that file's four-entry periodic table; "R-11" is an `unsafe impl Sync`
fold AND a geometric centroid where the physics wants a mass-weighted one.

**Cite rows from this register as `WO-R-9`, and from the other as `GRAV-R-9`** — or give the
path. A bare id is ambiguous across the tree, and the tell that it already was: the
disclosure that produced the provenance section below had to spell out the full path to say
which R-8 it meant. The duplication itself is registered as WO-R-15.

## Row provenance — who wrote which row, and a correction to what git says

**`git log -S` will tell you every row below R-7 was written by commit 4bec9e2 (T3
engine-core). That is false, and it is an artifact worth understanding rather than a
clerical detail.** This file was UNTRACKED while three lanes wrote into it. Git shows
nothing for an untracked path, so the interleave was invisible to every diff, and when T3
committed the file the whole of it — including four rows from the C1/tower lane and one
from workbench-engine — entered the history under one authorship. The sweep ran in
REVERSE: not a lane taking someone's hunk, but a lane's hunk being taken by whoever
committed first.

| rows | lane | what they were measuring |
|---|---|---|
| R-1 … R-7, R-13, R-14 | T3 engine-core | the storage/locality/PBC/threading/NPT surface |
| R-8, and the folds R-9, R-10, R-11 | `c1-rpmd` | the ring-polymer carrier: its operator's field type, and three duplications inside `rpmd.rs` that folded |
| R-12 | workbench-engine | the pair-curve door, timed from the browser |

This matters for the SERIES below and not only for credit. The register's falsifier is a
GROWTH RATE against domain size, and the three lanes were moving different axes — C1
enlarged the carrier axis, workbench measured an existing door, T3 enlarged the engine
without touching the domain at all. A ratio computed as if one lane produced all fourteen
rows would be comparing growth against a domain nobody's work actually doubled. Read each
lane's rows against the axis that lane moved; the two readings at the bottom already do
this and this table is what lets a third one.

**This table has now been wrong in both directions, and the second time was worse.** Its
first version grouped R-8 with the folds R-9/R-10/R-11 under one lane. I then "corrected" it
to split them across two lanes, on a name supplied to me from outside the artifact — and the
split was wrong. The first version had been right. Correcting a correct thing into an
incorrect one, on someone's recollection, is a worse failure than the original sweep,
because it carried the authority of a fix.

**The rule that ends this, and it is mechanical rather than testimonial: every fold row
already names its own SYMBOL, and a symbol has a commit.**

    git log -S '<symbol>' -- <file>          # whose row is this

Run for the three folds, all three answer `95d4262` — `c1-rpmd`'s commit, the one that
landed `rpmd.rs` after main was left declaring a module that did not exist. And the
converse check is as cheap: `git show --stat --name-only` over a lane's landings says
whether that lane has ever touched this file at all (`tower-complete`: zero, five times
out of five). No memory, no inference, and nobody's availability — which matters, because
asking depends on the right lane being reachable, and that is exactly what delayed this
disclosure by a session. Credit for the rule: `tower-complete`, who checked it before
replying to a message crediting them with work they had not done.

**The durable fix is upstream of this file****The durable fix is upstream of this file**: a register meant to be written by many lanes
should be committed EMPTY on the day it is opened, so that every later row arrives as its
author's own diff. Untracked is not a neutral state — it is a state in which collaboration
leaves no trace.

## Folded rather than registered

| id | what it would have been | how it folded instead |
|---|---|---|
| **R-1** | `wall_energy_force` special-casing `Boundary::Periodic` alongside `Boundary::Open`, i.e. a second `== ` test bolted onto the first. | Folded into `Boundary::has_walls()`, one predicate the wall term and anything else that cares can ask. The two wall-less boundaries are wall-less for different physical reasons and the same code reason, and the predicate says which is which. |
| **R-6** | The four-body sector's C² switching function, written inline with its own exponents, plus a second copy for the pair truncation. | Folded into `cells::switch_c2`, called by both. A second copy of a switching function is a second place for the exponents to be wrong, and the symptom — an energy leak with no obvious source — is not one the gates would have located quickly. |
| **R-9** | The exact free-ring-polymer propagator, written twice: once on momenta for the 1D PIMD sampler, once on velocities for the 3D ring dynamics. | Folded into `rpmd::FreeRingPropagator`, called by both. Not a hypothetical: the momentum copy multiplied by the mass and divided back, which is not the identity in f64, and it cost the `P = 1` classical-limit gate its bit-exactness — 1.05e-11 bohr of drift over 5000 steps where two spellings of one algorithm should have agreed to the last bit. After the fold the gate reads exactly 0.0. |
| **R-10** | A second derivation of the pair-curve grid range (WALL_CEILING at the inner end, TAIL_TOLERANCE at the outer) for the H–H table the C1 sampler consumes. | Folded: `rpmd::banked_range` delegates to `pair::derive_range`. The two agree to 0.392722458 vs 0.392722459 bohr, which is the two solvers' bisection convergence and not a disagreement about the rule. Noted while folding: `examples/emit_curve.rs` still hardcodes `(0.3, 10.0)`, a THIRD statement of the same intent that nothing keeps in step — not registered here because it is a fallback-artifact script, not engine physics, but it is where this would come back. |
| **R-11** | Two `unsafe impl Sync` blocks, promising that a `Cell` counter inside a `Pes` is only ever touched from one thread. | Folded into `AtomicU64`/`AtomicUsize` with `Relaxed`. The promise was true of every caller written so far, which is exactly the problem: it was a claim about caller behaviour written where no caller can see it, on a counter read once at the end of a run, for no measurable saving. |

---

## A note that is not a residual but wants recording

~~`holon_chem::elements::Species::mass_u` is documented as "Nuclear mass of the most
abundant isotope" and its hydrogen value, `1.00782503207 u`, is the ATOMIC mass of ¹H —
nucleus plus electron.~~ **DISCHARGED 2026-09-01 by the C1 lane**, which owns `holon-chem`
and was declaring `rpmd::MASS_U_DEUTERIUM` under the same convention. The doc comment now
says ATOMIC and says why the value is nonetheless the right one for a Born–Oppenheimer
dynamics. No number moved — only the sentence was wrong. `sim.rs`'s R-7 branch is a
separate thing and still open.

---

## A second note that is not a residual: two committed artifacts that no gate compares to their source

Recorded 2026-09-01 by the workbench-engine lane while building this page's engine, because
it is a live instance of the shape `ci-gates.sh` gate 10 exists to prevent and does not cover.

`ci-gates.sh` gate 10 compares `crates/holon-sandbox/viewer/holon_sandbox.wasm` to what its
source builds, and it reproduces byte-for-byte on rustc 1.95.0 (verified). Nothing does the
same for holon-render's two committed artifacts, and both are stale:

| artifact | committed | what its own source builds (rustc 1.95.0, at 052957b) |
|---|---|---|
| `crates/holon-render/viewer/holon_render.wasm` | 300,756 bytes | 310,849 bytes |
| `docs/atoms/holon_render.wasm` | two commits behind `viewer/`'s copy | — |

`docs/atoms/` is shipped VERBATIM by `pages.yml`, so the deployed atom viewer is running an
engine older than its own page claims. `docs/workbench/` avoids inheriting the problem by
having `pages.yml` rebuild its wasm from the commit being deployed; the same step would fix
`docs/atoms/` and `viewer/`, and is not this lane's to add to another page's artifacts.

---

## The series

| date | domain size (composition × carrier × band triples served) | open residuals | ratio |
|---|---|---|---|
| 2026-09-01 | 5 three-body compositions served (H₃, OHH, OOH, OOO, shipped bank) × 1 carrier (C0) × 1 band (atomistic) = 5 | 5 | 1.00 |
| 2026-09-01 (C1 lands) | the same 5 × **2 carriers (C0, C1)** × 1 band = 10 | 6 | 0.60 |

*First reading. A ratio of 1.00 on the first entry says nothing on its own — it is the
baseline the next one is read against. The prediction the fold makes, and what would
falsify it: adding C1 (ring-polymer nuclei) multiplies the domain by two and should add ZERO
rows, because the carrier axis is exactly what `CertifiedTransport` was built to absorb. If
C1 lands and this table grows, the fold is wrong on its own headline axis and WB-8.4's
verdict is recorded rather than argued with.*

### T3's own reading, added 2026-09-01 by the engine-core lane

T3 (dynamic storage, cell lists, PBC, checkpoint/replay, NPT, threading) did not enlarge the
DOMAIN at all — the same five compositions, the same carrier, the same band — so it is a
clean test of a different question: **does making the engine bigger make the register
longer?**

It added TWO rows (R-13 the two integrators, R-14 the borrowed worker probe) and FOLDED two
candidates that would otherwise have been rows (R-1 the wall-less boundaries, R-6 the
duplicated C² switch). It also surfaced one PRE-EXISTING residual nobody had entered (R-7,
the hydrogen mass branch) — which is the register working rather than growing.

Net, on the axis T3 actually moved — engine surface, not domain — is +2 rows for: heap-backed
state at any N, one cell decomposition serving three interaction sectors, periodic
boundaries with a demonstrated-firing translation plant, exact checkpoint and replay, a
leased-worker force evaluation that is bit-identical across worker counts, and an
extended-Lagrangian barostat with its own conserved quantity. Neither new row is a
composition carve-out, which is the kind the fold's headline claim is actually about, and
both have concrete discharges.

**THE PREDICTION'S VERDICT: WOUNDED, NOT CLEAN. C1 landed and the table grew by ONE row.**
The staked number was zero. Doubling the domain added exactly one residual (**R-8**), so
the ratio improved from 1.00 to 0.60 and the growth is sublinear — the register's own
headline claim survives — but "should add ZERO rows" is a different, sharper stake and it
did not. Three further candidates were considered and FOLDED rather than registered (R-9,
R-10, R-11), which is the outcome the carrier axis was supposed to deliver everywhere.

What the one row says, stated plainly because it is the interesting part: the carrier axis
absorbed the STATE (`RingPolymerState`, the lift, the centroid retract, the commuting
certificate — all of it worked and none of it needed a special case) and did NOT absorb the
OPERATOR. `Contribution<C>`, `AdditiveOperator`, `CertifiedTransport` are all generic and
all held; the thing that did not hold is one field type, `Option<fn(f64) -> f64>`, which is
generic in nothing and cannot carry a potential that owns anything. So the failure is not
in the fold's shape — it is in one concrete type sitting inside it, and R-8's discharge is
a type change rather than a redesign. That distinction is worth more than the row count:
the axis is right and one of its inhabitants is under-specified.
