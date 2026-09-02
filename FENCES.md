# The fence ledger

*Every fence in this tree, enumerated by grep rather than from memory, with an owner and an
exit for each. Standing order (bank-fences lane, GANTT node D, upgraded): **any fence is a
bug waiting to be fixed once the GPU solve is the path.** A fence is never architecture.*

## The law this file lives under

Stated verbatim, because it is not this lane's invention — it is `engine/ci-gates.sh`'s own
ruling on its own allow-list, and this ledger is that ruling applied to the rest of the
tree:

> Team-lead's ruling on the discriminator, worth restating wherever this pattern
> recurs: AN ALLOWLIST ENTRY IS LEGITIMATE ONLY WHEN THE BREAK HAS AN OWNER AND AN
> EXIT; WITHOUT BOTH IT IS SUPPRESSION.
>
> — `engine/ci-gates.sh:566-568`, echoed in `TIERS.md:178`
> ("allowlisted WITH owner and exit criteria, not hidden")

And its physics-side twin, `OBJECT.md`'s law 9:

> **Refusal is a feature.** A tier or stratum outside its certified scope
> refuses, naming the gate whose passing would lift the refusal.

So: **a fence without an owner and an exit is suppression.** A fence WITH both is a
priced, dated, assigned piece of work — which is the only thing that makes an honest
refusal different from a limitation nobody intends to remove. Where this ledger cannot
find an owner it writes `UNOWNED`, and where it cannot find an exit it says so and does
not invent one. Both of those are findings, not gaps in the sweep.

## What counts as a fence here

A fence WITHHOLDS a capability the engine could in principle serve. Three classes, and
every row carries exactly one:

| class | meaning | exit shape |
|---|---|---|
| **PHYSICS-HONESTY** | the physics is not yet certified; serving it would be a number with no ancestor | a named GANTT or campaign node |
| **COMPUTE-PRICED** | affordable the moment the GPU solve is the path | GANTT node **F** |
| **MODEL-FENCE** | a stated limit of the model itself | still NAMED, however far — never "permanent" |

**Not fences, and deliberately excluded** (swept, classified, listed in Appendix A):
schema and provenance checks, which refuse malformed or unsourced INPUT rather than
withholding a capability; and well-posedness checks, which refuse where the mathematics is
undefined. A theorem is not a fence, and filing one here would make the ledger's own
count meaningless.

## Verdict

**50 fences.** PHYSICS-HONESTY 19 · COMPUTE-PRICED 20 · MODEL-FENCE 11.
*(M11 added 2026-09-01 by workbench-engine, at the lead's ruling on the swallowed
3D-build failure — the counts above move with it, because a register whose total
disagrees with its own rows is worse than one that is merely out of date.)*

Plus **10 findings**: one fence with **no exit** (F-3 — and the code says so itself, which
is the model working); three fences whose stated reason is false or stale at HEAD (F-1,
F-10, and F-2 — **F-2 DISCHARGED 2026-09-01 at 8554c14**, class mechanised, see its
entry; the other two stand); four places that carried a break with **no owner**, which by this file's own
law is suppression (F-4's two `CRATE_ALLOW` entries, plus M2 and M10's exit); one fence
whose named class has no reachable member (F-5); one silent zero where a named refusal
belongs (F-6); and one gate that is one-directional (F-9).

**Status of the findings, updated `234aa0e`.** F-4 is **DISCHARGED** (`1839eb7`): both
allow-list entries now carry owner and exit and cite this ledger. F-5 is **ROUTED** onto
GANTT's MPS node. F-8's two unowned model fences (M2, M10) are now a **named triage item**
in GANTT's fence-triage note — still unowned, but with the clock visible, which is the
condition the law actually demands. F-1, F-2, F-3, F-6, F-7, F-9 and F-10 stand as
written. Discharged findings are kept and marked, never deleted: a ledger that drops them
cannot be audited against the state it was written from.

---

## PHYSICS-HONESTY — 19

*The physics is not certified. The exit is a named node, not more hardware.*

| # | WHERE (file:symbol) | WHAT it refuses | OWNER | EXIT | GANTT |
|---|---|---|---|---|---|
| P1 | `holon-chem/src/ozone.rs:412` `generate()` | the (O,O,O) three-body surface — returns `None`, so the dynamics fences every OOO triple | ozone / atomworld lane | seam scan run first (`examples/ozone_seam_scan.rs` exists) → ~14k S-reduced nodes of genuine FCI through the leased generator → the OOH-grade certification gates | D → G, priced by F |
| P2 | `holon-render/src/sim.rs:494` `fence_untabulated` (set at `:2559` from `fenced_triples()`) | evaluating any triple whose surface is untabulated; counts them instead of guessing | ozone / atomworld lane | P1 lands → the counter goes to zero (the FSD's own acceptance: `fence = [4×8]` today, `fence 0` after) | D → G |
| P3 | `holon-chem/src/tower.rs:159` `TransportRefusal::UntabulatedSeamFence` | transport across an electronic seam with no tabulated data at that coordinate | s3_mesh / seam-scan lane | the seam tables (`ohhh_seam_scan.rs`, `ozone_seam_scan.rs`) | H → MPS |
| P4 | `holon-render/src/trimer_bank.rs:149` `SeamRecord::AcceptedFloor` | locating the seams — surfaces ship declaring an accepted interpolation error near them instead | s3_mesh / trimer lane | locate the loci; the variant's own doc calls this "the campaign's own position while the locus is still owed" | H |
| P5 | `holon-render/src/bank.rs:170` `Refusal::DmrgUnvalidated` | any DMRG curve while gate D1's validation is not recorded | elements-referee / mixtures-referee | record D1's validation | D |
| P6 | `holon-sandbox/src/tier.rs:154` `Refusal::NoValidatedEvaluator` | dynamics on T1/T2 — no force constants in the tree, so a run would be a number with no ancestor | T2 DFT lane (`T2_DFT_REFERENCE.md`) | the Phase-2 elastic tensor, 6 of 12 strain runs done → the T2 gate. **Exit is machine-readable: `Refusal::unlock()`** | G |
| P7 | `holon-sandbox/src/tier.rs:159` `Refusal::NoGravityChart` | certified dynamics for any scene with weight (`relativity.rs` is flat by design) | curved-tier lane | the curved-tier certificate (PROGRAM.md A3). **Exit in `unlock()`** | G |
| P8 | `ciris-sim-core/src/bridge.rs:168` `WeakFieldRefusal::ExceedsWeakField` | weak-field certification where ε exceeds the screen | sim-core relativity lane | a stronger-field chart family (Schwarzschild exact); unlock text at `:192` | G |
| P9 | `bridge.rs:171` `WeakFieldRefusal::ExpansionScale` | scenes where the `(HL/c)²` background term dominates | same | the FRW chart family; unlock text at `:196` | G |
| P10 | `bridge.rs:175` `WeakFieldRefusal::UnsupportedPotentialFamily` | scenes needing a potential outside the v1 family (e.g. a flat rotation-curve disk) | same | the v2 logarithmic-potential family; unlock text at `:201` | G |
| P11 | `holon-chem/src/tower.rs:649` `C2_MpsTdvp` fence (i) | C2 as a **climb** — `c1_to_c2_transport_capability` is not built, so C2 is reachable as a NODE only | C2 / tower lane | build the C1→C2 picture change (ring-polymer nuclei → real-time electronic carrier) | E, MPS |
| P12 | `holon-chem/src/tower.rs:649` `C2_MpsTdvp` fence (ii) | growing a bond dimension — single-site TDVP cannot, so a start must already carry its rank | C2 / tower lane | two-site TDVP, named in the doc as the discharge route | MPS |
| P13 | `docs/workbench/smoke.mjs:684` `holon_phase_call` | the blind phase-classifier panel (WB-5.5) — "fenced on the page because none exists" | workbench-engine | write the classifier, wire the panel, delete the entry | G |
| P14 | `smoke.mjs:685` `holon_q_tet` | the order-parameter panel (WB-5.5) — "because none are computed" | workbench-engine | compute them, delete the entry | G |
| P15 | `smoke.mjs:686` `holon_water_table_begin` | the (O,H,H) surface panel — "for want of an ABI door" | workbench-engine | ship the ABI door; the surface itself is not the blocker | D → G |
| P16 | `smoke.mjs:687` `holon_refinement_active` | the local-refinement panel (WB-1.2) — "because none exists" | mesher | build refinement, delete the entry | G |
| P17 | `WORKBENCH_FSD.md:277` C3+ spinorial/QED nodes | materialized spinorial and QED carriers — they exist as "visible STUBS with fences — reachable, not materialized" | tower lane | WB-8.4's discharge law: pay the price, transport to the adequate carrier | MPS, G |
| P18 | `holon/src/zx.rs:43` | handing a shorter circuit back to the runner — "there is NO extraction back to a circuit" | zx-native | build the circuit extractor (the T-count oracle and scalar are certified already) | off-graph |
| P19 | `holon-chem/tests/mixtures_referee.rs:293` `#[ignore]` | grading the staked pairs against the 50-digit referee | mixtures-referee | commit the drop to `tests/data/mixtures1/`, delete the `#[ignore]`, re-pin `MIXTURES1_REFEREE_DIGEST` — and do both, per the test's own header | D |

*(A twentieth row was drafted here for the ELEMENTS-1 referee gate and removed on
verification: that fence is already discharged. Its instructions are still in the tree —
see finding **F-10**.)*

## COMPUTE-PRICED — 20

*Affordable once the GPU solve is the path. Exit is GANTT node **F** for every row; where
F alone is not sufficient the second requirement is named.*

| # | WHERE (file:symbol) | WHAT it refuses | OWNER | EXIT | GANTT |
|---|---|---|---|---|---|
| C1 | `holon-render/src/bank.rs:78` `IN_BROWSER_DET_LIMIT = 1024` | solving a pair at or above 1024 determinants in the browser at load | workbench-engine / c1-browser | F, or a shipped referee-pinned table | F |
| C2 | `bank.rs:108` `IN_BROWSER_BASIS_LIMIT = 6` | solving a pair above six basis functions — MEASURED, and the driver: Cl–Cl is only 324 determinants but 18 basis functions and costs 95.95 s, because the integral transform is a high power of the BASIS and runs before any determinant is enumerated | workbench-engine / c1-browser | F | F |
| C3 | `bank.rs:180` `Refusal::SplitViolated` | a curve arriving on the wrong side of that split (browser host only) | workbench-engine | F lifts C1/C2 and this with them | F |
| C4 | `holon-render/src/lib.rs:1700` `holon_bank_generate_pair` | spending on a pair past the det limit — checked BEFORE the solve, not after | workbench-engine | F | F |
| C5 | `holon-chem/src/fci.rs:1085` `HARD_DETERMINANT_CAP = 2_000_000` | `solve_determinant` outright above it. **21 of the 54 registered atoms sit past this** (see `conformance/atomworld/PERIODIC_AVAILABILITY.md`) | holon-chem | F, then raise deliberately — the constant's doc is explicit that it exists so a careless caller does not wait on such a space unknowingly | F, D |
| C6 | `fci.rs:1092` `MPS_ROUTE_THRESHOLD = 50_000` | the determinant route above it; hands the space to MPS/DMRG. **27 of 54 atoms land past this** | holon-chem | F | F, D |
| C7 | `pair.rs:997` `MPS_MAX_ORBITALS = 9` | the automatic MPS arm above nine orbitals. MEASURED: LiH at six took 528 s to build its MPO, HCl at ten never finished | mps / tower lane | F is necessary and **not sufficient**: the const's own header rules that the fix is a TWO-PART door (orbitals for the build, a filling-aware axis for the reach), "a designed change with its own measurement, not a bigger number here" | F, MPS |
| C8 | `pair.rs:1029` `MPS_MAX_DETERMINANTS = 1024` | the MPS arm above 1024 determinants. MEASURED at χ=32 under a 300 s per-cell budget | mps / tower lane | F, plus a larger χ under a larger budget — explicitly untested, and the doc says so | F, MPS |
| C9 | `q8-mps/src/dmrg.rs:70` `REFUSAL_THRESHOLD = 1e-4` via `RefusalPolicy::Typed` | a sweep whose worst bond discards more Schmidt weight than the ledger allows. Self-typed FLOOR: "a larger `chi_max` serves this request" | q8-mps | F, then a larger χ_max | F |
| C10 | `holon-chem/src/rpmd.rs:697` `RefereeRefusal::Unconverged` | a level whose Lanczos Ritz residual exceeded tolerance | c1-rpmd | F, then more iterations | F |
| C11 | `rpmd.rs:699` `RefereeRefusal::GridNotConverged` | a level that moved more than tolerance when the grid was halved | c1-rpmd | F, then a finer grid | F |
| C12 | `rpmd.rs:701` `RefereeRefusal::BoxNotConverged` | a level that moved more than tolerance when the box was widened | c1-rpmd | F, then a wider box | F |
| C13 | `rpmd.rs:703` `RefereeRefusal::InstrumentsDisagree` | a level where the independent Numerov instrument disagreed past tolerance | c1-rpmd | F, then tighter runs on both | F |
| C14 | `holon-chem/src/tower.rs:157` `TransportRefusal::ClosureDefectExceeded` | transport whose commuting-square defect `‖[H,P]‖` exceeds budget | tower lane | F, then tighter transport | F |
| C15 | `tower.rs:158` `TransportRefusal::NonCommutingRetract` | transport whose round-trip retract residual exceeds tolerance | tower lane | F, then tighter transport | F |
| C16 | `holon-resource/src/tier.rs:256` `Routing::Exhausted` | an ask past the top rung — "no rung above this one" | holon-resource | F supplies the new rung | F |
| C17 | `holon-resource/src/tier.rs:265` `Routing::Unmeasured` | neither satisfies nor refuses: the ask is past what the rung has been MEASURED to reach, but that measurement was a lower bound and nobody has tried | holon-resource | run the measurement the variant names — it names its own exit by construction | F |
| C18 | `engine/ci-gates.sh:574` `CRATE_ALLOW["holon-gpu"]` | running holon-gpu's 12 determinism tests in CI — GitHub runners have no NVIDIA GPU (tested green on the 4090 dev box) | gpu-mesh lane / team-lead | a CI runner with a GPU, "at which point this entry converts to a real invocation" | F |
| C19 | `ci-gates.sh:575` `CRATE_ALLOW["q8-mps"]` | the rest of q8-mps beyond gate 16b's `c2_tdvp_gates` — a live full-grid run is hours deep and a gate must never run `--ignored` full-grid tests | the C2 / tower lane | the grid completes; "this entry converts to a plain `-p q8-mps`" | F |
| C20 | `q8-mps/tests/full_grid_gates.rs:30` `#[ignore]` | the full-grid validation on a default `cargo test` — minutes per configuration | q8-mps | F shortens it; today, run explicitly with `-- --ignored` | F |

## MODEL-FENCE — 11

*A stated limit of the model. The exit is named in every row, however far. "Permanent"
does not appear in this table.*

| # | WHERE (file:symbol) | WHAT it refuses | OWNER | EXIT | GANTT |
|---|---|---|---|---|---|
| M1 | `holon-chem/src/elements.rs:2619` `MAX_Z = 54` | every element past xenon; the registry has no row for it | elements / atomworld | **declare the Z ≥ 55 rows** — the block below argon is emitted by `conformance/atomworld/elements3_transcribe.py` from a pinned tabulation, so the exit is a generator run plus its gates. NOT "build f integrals": see finding **F-1** | D |
| M2 | the non-relativistic electronic Hamiltonian — `holon-chem/src/md.rs` and `src/fci.rs` build kinetic, nuclear-attraction and two-electron Coulomb integrals over real contracted Gaussians and contain no mass–velocity, Darwin or spin–orbit term | a defensible answer for **Z > 36**, whatever the route says. Staked at krypton; 18 of 54 registered species carry it | **UNOWNED** — no lane in GANTT.md owns relativity for chemistry | **the relativistic solver rung**: a scalar-relativistic one-electron correction (ZORA or Douglas–Kroll–Hess) first, which needs no new two-electron machinery; spin–orbit second, which does. Not liftable by compute — no amount of GPU makes a Hamiltonian relativistic | successor to D |
| M3 | `holon-chem/src/elements.rs:345` `homonuclear_radius() -> Option<f64>` | a scene radius for 44 of the 54 registered species — `None` past neon | elements / atomworld | measure 44 homonuclear equilibria (half the located `R_e`, the way the ten in `docs/atoms/species_palette.json` were made). The doc is explicit that the exit is measuring, "not writing more arms" | D → heavy-element scenes |
| M4 | `holon-chem/src/trimer.rs:85-86` `MAX_ORB = 3`, `MAX_DET = 9` | the fast trimer path above three hydrogens — fixed-size arrays | holon-chem | the general n-centre route, which already exists and is held to this path at 1e-12 hartree by `tests/trimer.rs`. A scope limit on an optimisation, not on a capability | none needed |
| M5 | `holon-render/src/sim.rs:112` `GravityRefusal::PeriodicBox` | a gravitational field on a periodic box: `m·g·y` is linear and the wrap makes it discontinuous, so an atom crossing the top face gains `m·g·H` with nothing having done the work and the balance gate opens by that jump every crossing | holon-render | **a non-periodic chart — walls, or an open box** — named in the refusal's own `plain()`. Within a periodic chart this is a ceiling, not a floor: see finding **F-3** | none |
| M6 | `holon-sandbox/src/scene.rs:306` `LawRefusal::UnderResolved` | a cohesive law where cell spacing ≥ 2·l_ch — no positive softening branch exists for the homogenized bilinear law at that grain | mesher | a finer mesh; the refusal is a statement about the grain, not about the material | G |
| M7 | `holon-chem/tests/mixtures_referee.rs:82` `ScopeRefusal::OutOfScope` | grading any species with Z past `MIXTURES1_REFEREE_Z_MAX` (18, argon). A limit of the REFEREE's model, not the engine's — and refused BY NAME rather than silently skipped, so the pass is not vacuous | mixtures-referee | extend the referee past argon | D |
| M8 | `WORKBENCH_FSD.md:333` | a 2D fallback renderer — the Bevy/WebGPU shell is "the only rendering. No 2D fallback — a fence with the reason, not a degraded mode" | workbench-engine | the viewer's own device. Deliberate: the alternative is a silently degraded chart, which WB-5.2 forbids | none |
| M9 | `docs/workbench/smoke.mjs:683` `holon_set_pressure` | a pressure SETPOINT door — "WB-2.2's control IS the box (`holon_box_scale`); pressure is the readout, not a target" | workbench-engine | a design change, not work owed. **But the prose fence beside it is false: see finding F-2** | none |
| M10 | `holon-chem/tests/ion_core.rs:288` `the_electron_affinity_gate_fired_oh_minus_sits_above_neutral_oh` — the fence is carried in the gate's own NAME; staked as row **I-5** of `conformance/water_observatory/ION_STAKING.md` | **anions are unbound in STO-3G.** OH⁻ sits **+0.3055 Ha above** neutral OH (`E(OH) − E(OH⁻) = −0.305545907904 Ha`), so this model's anion energies may not be used as affinities. Fences all anion-mediated chemistry — hydroxide chains, autoionization's OH⁻ half. **Grotthuss via H₃O⁺ is NOT fenced**: the proton affinity passes on the identical seam at `+0.379432332077 Ha`. Discriminated to the BASIS, not the charged seam: H⁻/H shows the same sign with a one-determinant CI space where no sector rule can be wrong, and cations pass the same path | recorded by ion-core in `ION_STAKING.md` I-5; **the exit itself is UNOWNED** — I-5's owner column reads "not this node; a basis lane… nothing technical — it is unowned, not blocked" | **the diffuse-basis rung (I-5)**: a named model upgrade. Discharge is the SAME two gates in `ion_core.rs`, un-retuned, re-run against a basis carrying diffuse functions, with the STO-3G readings kept beside the new ones. `holon-chem` declares exactly one basis (`sto3g.rs`) and adding a second is a crate-shaped decision, not a test fix | C → A (I-2's charged tables inherit the constraint) |
| M11 | `docs/atoms3d/index.html` `upgradeFenceFromBuildStatus()`, stamped by `.github/workflows/pages.yml` | showing a viewer a GENERIC "neither backend would load" when the real cause is that CI never built the artifact. The two want opposite responses — one is the viewer's hardware, the other is ours — and the page could not tell them apart | workbench-engine | none owed; this IS the exit. `pages.yml` keeps `continue-on-error` on the 3D build (a broken bundle must not take the site down) and now STAMPS the outcome, commit and run number into `docs/build-status.json`; the page reads it on the failure path only and upgrades its own fence to name the cause. Measured need: holon-render-3d did not compile for two days (fixed 245f601) with the headless gate structurally blind, this step swallowing the failure, and the page fencing gracefully — three correct greens adding up to nobody being told | none |

---

## Findings

*Nine. Reported rather than repaired, because seven of them are outside this lane's files
and the eighth is a design question. Each names the lane that owns the fix.*

**F-1 — `MAX_Z`'s stated reason is contradicted by its own file.**
`elements.rs:2617` says the registry stops at xenon "because the next shell needs f
functions (l = 3) and the integral machinery has none." The integral machinery HAS them:
`ShellKind::F4`/`F5` exist with `l() = 3` and `n_functions() = 7`, `md.rs:498` branches on
`l == 3`, `md::SPHERICAL_F` (`md.rs:333`) is the 7×10 projector that removes the three
p-type contaminants, and `tests/spherical_f.rs` gates it. The same file says so plainly at
`:78-82`: "`ShellKind::F4` and `F5` exist with no constructor reaching them and the
integrals are gated without a consumer." So M1's real exit is **declaring the rows**, not
building the machinery — a materially cheaper piece of work than the comment implies, and
the sort of mis-stated exit that keeps a fence standing longer than it needs to. Owner:
elements / atomworld lane. One comment.

**F-2 — a shipped fence's justification is false at HEAD, and the mechanised gate cannot
see it.**
`docs/workbench/app.js:24` tells the viewer: "P^-0.05 box scaling as NPT → a FENCE. There
is no barostat in this engine." There is. `holon-render/src/barostat.rs` implements an
isotropic MTK barostat with a Nosé–Hoover chain; `sim.rs:675` holds it as
`pub barostat: Option<Box<crate::barostat::Barostat>>`; `sim.rs:3282` branches on
`barostat_on()`; `BarostatRefusal` is a typed refusal with three variants; and
`tests/t3_barostat.rs` gates it against an ideal gas. What does not exist is a pressure
SETPOINT door in the ABI — which is exactly what M9's `FENCE_JUSTIFYING_ABSENCES` entry
claims, correctly. **The two claims are different, and only the narrow one is checked.**
`smoke.mjs`'s gate watches for `holon_set_pressure` appearing; it cannot watch prose in a
neighbouring file that asserts something broader. This is the failure mode the gate was
built to prevent, arriving one file over. Owner: workbench-engine. Fix: make `app.js:24`
say what `smoke.mjs:683` says.

> **DISCHARGED 2026-09-01 at 8554c14** by workbench-engine, and the finding was worth more
> than the sentence it corrected. TWO claims were false, not one: the page's header also
> listed GRAVITY among the absent, months after that lane landed it. Rather than edit two
> sentences, the CLASS is now mechanised — `smoke.mjs` carries a table of (export that
> proves a capability exists, phrase that would deny it), and the build fails with the
> correction to make if the export resolves while the phrase is in the shipped text,
> comments included, since `app.js:24` is where this finding lived. The gate then caught
> its own author: the first correction kept the false sentence as a QUOTATION inside a
> historical note and failed the page for containing the phrase. Restating a claim to
> disown it puts the claim back — the second instance of that shape on this page — so both
> are paraphrased now. M9's narrow claim was correct throughout and is unchanged.

**F-3 — exactly one fence in this tree has no exit, and the code says so itself.**
`ciris-sim-core/src/bridge.rs:180` `WeakFieldRefusal::RequiresSpacelikeSignal` refuses a
claim requiring influence outside the light cone, and `bridge.rs:217-219`
`is_ceiling()` returns `true` for it and only for it: "a ceiling is invariant under every
re-root within the chart family; a floor is lifted by one." No exit is offered here
because none exists and inventing one would be worse than the fence. It is filed as a
finding rather than a table row precisely because it is the one place where the standing
order — every fence is a bug with a fix path — does not apply, and the tree already knew
that and typed it. **This is the model working.** No owner needed.

**F-4 — DISCHARGED at `1839eb7`. Two entries inside the very allow-list that supplies this
ledger's law were, by that law, suppression.**
As found, `ci-gates.sh:576-577` read:

    ["q-seam"]="uncovered, ownership untriaged (chief-of-staff-2, 2026-08-24)"
    ["sphere-demo"]="uncovered, ownership untriaged (chief-of-staff-2, 2026-08-24)"

Neither named an owner; neither named an exit. Against `ci-gates.sh:566-568`'s own ruling —
"AN ALLOWLIST ENTRY IS LEGITIMATE ONLY WHEN THE BREAK HAS AN OWNER AND AN EXIT; WITHOUT
BOTH IT IS SUPPRESSION" — these two were suppression by the file's own definition. They
were not in the COMPUTE-PRICED table above because they are not compute-priced: nobody had
established that they were anything. The other five entries in the block each named a
reason, and the two that are deferrals rather than exclusions (C18, C19) each named an
owner and an exit.

**Both now carry Owner: team-lead and Exit: "real invocation or crate retirement, decided
at the fence-ledger triage", citing this finding by name.** The break itself is unchanged —
both crates are still uncovered by CI — and that is the point of the discharge: the ruling
is about whether a break is ACCOUNTED FOR, not about whether it is fixed. The clock is now
visible in `GANTT.md`'s fence-triage note rather than absent.

*Kept in the record and marked, not deleted. A ledger that quietly drops a discharged
finding cannot be audited against the state it was written from — and it would be
committing F-10, one finding down this page.*

**F-5 — the MPS route is a named band with no reachable member.**
`pair::MPS_MAX_DETERMINANTS` (1024) is smaller than `fci::MPS_ROUTE_THRESHOLD` (50,000),
so a space big enough to be ROUTED to MPS is necessarily bigger than the sweeps' measured
REACH, and `AutomaticRoute::Mps` cannot be selected by any input. This is deliberate,
documented at `pair.rs:1016-1020`, and asserted by
`the_mps_arm_is_unreachable_at_the_current_constants` in `tests/pair.rs` — the arm is kept
so the day the sweeps improve the fix is one constant. Recorded here because the
consequence is not local: **27 of the 54 registered atoms classify as MPS-ROUTE and none
of them has an automatic route at all**, and 21 of those are also past
`HARD_DETERMINANT_CAP`, so the by-hand fallback refuses too. Any reader of an availability
table who takes "MPS-ROUTE" for a route will be wrong 27 times. The generated table states
this in its own header, derived at run time from the two constants rather than asserted, so
the sentence moves when they do.

**ROUTED at `1839eb7`**, onto `GANTT.md`'s MPS node where the seam work will meet it: "the
seam work must move the cap or the routing, not just add the seam." The finding stands —
nothing about the constants has changed — but it is now a constraint the receiving node
carries rather than a fact filed only here.

**F-6 — one silent zero where a named refusal belongs.**
`holon-chem/src/ozone.rs:217` `OzoneTable::eval` returns `(0.0, [0.0; 3])` in three places:
when `!self.loaded` (`:219`), when `x` or `y` falls outside `[R_LO, R_HI]` (`:236`), and
when the degenerate denominator is below 1e-12 (`:240`). The first is backstopped —
`generate()` returns `None` today, and `sim.rs`'s `fence_untabulated` counts what that
fences, so the absence is visible. The other two are not: an out-of-range or degenerate
geometry silently contributes exactly zero three-body energy, with no counter, no doc
comment and nothing on the page. WB-5.2's rule is "never faked, never interpolated across,
never silently zeroed", and a zero returned for a geometry outside the grid is the third
of those. Owner: ozone / atomworld lane, alongside P1 — the surface lands and this path
becomes live.

**F-7 — a sibling example carries the exact `choose` form its own crate forbids by name.**
`pair.rs:1140-1152` documents at length why `acc = acc.saturating_mul(n - i) / (i + 1)` is
wrong in the dangerous direction (once the multiply saturates the divide pulls the result
back down, so the count UNDERSTATES and a space reads as cheaper than it is; `C(64,32)`
measured 5.76e17 against a true 1.83e18). `examples/elements3_atoms.rs:164-169` is that
exact form. It is LATENT — it bites only at `n ≥ 64` with `k` near `n/2`, and the largest
atom in the registry has 27 orbitals — but it is latent in an example whose whole job is to
print determinant counts. This lane's new generator uses exact `u128` with an explicit
`None` for "cannot be represented", which cannot understate at all. Owner: elements3 lane.

**F-8 — two fences in this ledger have a named exit and nobody to walk through it, and
they are the same shape.**
M2, the non-relativistic Hamiltonian past Z = 36, names no owner: no node in `GANTT.md`
owns relativity for chemistry and no campaign has staked it. M10, anions unbound in
STO-3G, is *recorded* by ion-core — but ION_STAKING.md's I-5 row says of the exit itself
"not this node; a basis lane… **nothing technical — it is unowned, not blocked**", which
is a lane declaring, correctly and in writing, that it is not the owner.

Both are BASIS-or-Hamiltonian upgrades: a crate-shaped decision about what physics the
model contains, too large for the lane that discovered it and too specific for any lane
that exists. By this file's own law an unowned break is suppression however well
documented, and that both of these are impeccably documented is exactly why they need
saying — **a fence does not become owned by being described well.** The exits are named
(M2's relativistic rung, M10's diffuse-basis rung), which is what keeps them fences rather
than limitations. What they lack is somebody whose job it is. Owner: to be assigned, and
the assignment is one decision covering both.

**Now a named triage item** (`1839eb7`, GANTT.md's closing fence-triage note), listed
beside F-4's two crates and awaiting the lead's decision. That does not discharge it —
neither fence has an owner yet — but it moves them from *unrecorded* to *recorded and
pending*, which is the difference between suppression and a queue.

**F-9 — the fence-justifying-absence gate is one-directional.**
`smoke.mjs:682-695` checks that every entry in `FENCE_JUSTIFYING_ABSENCES` is still
absent — if an export appears, the fence text has become false and the gate says so, with
an instruction rather than a complaint. It is a good gate and it caught two fences into
service already (`holon_set_gravity` at `:354`, `holon_box_scale` at `:628`). What it
cannot do is the other direction: it cannot detect a fence rendered on the page whose entry
was never added to the block, and it cannot see a justification stated anywhere but in the
block itself. F-2 is exactly that gap, occurring. Asking the enforcement question the house
rule asks — *what would this command NOT catch?* — the answer is: a new fence, and a false
reason. Owner: workbench-engine.

**F-10 — a discharged fence's instructions are still standing, and this ledger nearly filed
them as a fence.**
`holon-chem/tests/pair.rs:323-337` carries a block headed "IGNORED UNTIL THE REFEREE
LANDS", instructing a reader to "DELETE the `#[ignore]` below and re-pin
ELEMENTS1_REFEREE_DIGEST from the failure message the digest check prints." Both halves are
already done: `grep -n '#\[ignore' engine/crates/holon-chem/tests/pair.rs` returns nothing
but that comment's own mention of the word, `r2_the_first_row_matches_the_fifty_digit_referee`
at `:456` is a live `#[test]`, and `ELEMENTS1_REFEREE_DIGEST` is pinned at `:383` to
`0x54ef_d889`. The referee landed and the gate runs; only the instruction survived.

This is F-2's mirror image — there, a live fence's stated reason had gone false; here, a
discharged fence's text outlived it — and the two together are the argument for this ledger
existing at all: **fence prose drifts in both directions, and only the mechanised half of a
fence is checked.** It is recorded as a finding rather than fixed because it is one comment
in another lane's test file, and because the near-miss is the interesting part: the sweep's
own `#[ignore]` grep counted this file as a hit, and only reading the primary artifact
showed the hit was a comment about an `#[ignore]` that no longer exists. A grep count is a
place to look, not a verdict. Owner: elements-referee. One comment block, deleted.

---

## Appendix A — swept, and NOT fences

*Listed so the enumeration is complete and the classification is reasoned rather than
scoped. These refuse malformed INPUT or undefined mathematics; they withhold no capability,
and filing them as fences would make the count above meaningless.*

**Schema / provenance gates** — refuse artifacts that are malformed, unsourced or
self-contradictory:

- `holon-render/src/bank.rs:164` `Refusal`, 8 of 9 variants (`RouteUndeclared`,
  `DmrgClaimedExact`, `UncertaintyMissing`, `DmrgUncertaintyMissing`, `SplitViolated`\*,
  `CurveNotLoaded`, `UncertaintyExceedsResolution`, `UncertaintyExceedsWell`). The ninth,
  `DmrgUnvalidated`, is P5; `SplitViolated` also appears as C3 because it enforces a
  compute-priced split.
- `holon-render/src/trimer_bank.rs:156` `TrimerRefusal`, all 19 variants — coordinates
  missing, counts mismatched, axes non-monotone, digest absent, VOIDs counted but not
  named. Every one refuses an artifact that cannot be checked, not a physics it cannot do.
- `holon-chem/src/ozone.rs:336` `from_text` — parse and shape validation.
- `ciris-sim-core/src/bridge.rs:183` `WeakFieldRefusal::Undeclarable` — a non-finite or
  non-positive envelope. The type's own doc: "not a physics verdict."
- `holon-sandbox/src/scene.rs:308` `LawRefusal::NoMaterial` — the tier declares no material
  chart at all.
- `holon-resource/src/tier.rs:251` `Routing::NoClaim`.
- `holon-render/src/barostat.rs:209` `ScaleRefusal::BadFactor` — the factor is not a
  positive finite number.

**Well-posedness checks** — refuse where the mathematics is undefined, not where the
engine is short of anything:

- `holon-render/src/barostat.rs:217/219/221` `BarostatRefusal::{NotPeriodic, TooFewAtoms,
  DegenerateBox}` — with walls the container carries momentum flux the internal virial
  cannot see, so the controlled number would not be the pressure; fewer than two atoms have
  no virial; a box with no volume has nothing to change.
- `holon-render/src/barostat.rs:211` `ScaleRefusal::CollapsesBox`.

**Deliberate non-gates** — `#[ignore]`d because they are measurements, not checks. Each
states its reason and its manual invocation:

- `holon/tests/sample.rs:676` `cost_curve` — "a measurement, not a check: the quadratic in
  branch count is the honest price of the exact route and there is no pass/fail to attach
  to it."
- `q8-mps/tests/c2_tdvp_gates.rs:468` `c2_price_measurement` — "timing, not correctness…
  a wall-clock number inside a gate is a placement lottery (M-PLACEMENT-LOTTERY)."

**Owed verifications adjacent to fences** — work named as owed rather than implied done.
Not capability withheld, but recorded so the sweep is honest about them:

- `holon-render/src/trimer_bank.rs:334` — the digest field's presence is checked, not
  recomputed; "verification is owed and is named as owed rather than implied by carrying
  the field."
- `holon-mesh/src/sizing.rs:28` — §8's M-G2 interior-cell resolution measurement "is still
  owed. Nothing in this module substitutes for it."
- `holon-tables/src/generate.rs:210` — "the counters are live; the ledger's copy of them is
  not yet."

---

## Appendix B — how this ledger was built, so it can be rebuilt

The sweep is over the **whole tracked tree** (1,218 files at `892c982`), not over a
directory chosen in advance — a scoped grep reported as complete is a registered failure
mode in this programme. The commands and their hit counts:

```
git grep -n -E "^\s*(pub )?enum [A-Za-z0-9_]*Refusal" -- '*.rs'          12 enums
git grep -n "FENCE_JUSTIFYING_ABSENCES"                                   5 hits, 1 file
git grep -n "CRATE_ALLOW"                                                 8 hits, 2 files
git grep -n "holon_bank_in_browser"                                      12 hits, 7 files
git grep -n "fence_untabulated"                                          24 hits
git grep -n "refusal_reason"                                              7 hits, 6 files
git grep -n -A2 '#\[ignore'                                               8 hits, 5 files
git grep -n -E "unimplemented!|todo!\(" -- '*.rs'                         0 hits
git grep -n -i -E "\bfence[sd]?\b" -- '*.rs' '*.mjs' '*.js' '*.sh' '*.py'  348 hits
git grep -n -i "fence" -- '*FSD*'                                        17 hits
git grep -rn "MPS_ROUTE_THRESHOLD"                                       55 hits
```

The 348-hit fence-word sweep is the backstop: it is deliberately loose, and every hit was
triaged rather than sampled. `unimplemented!`/`todo!()` returning **zero** across the tree
is itself a reading — this codebase does not fence by panicking, it fences by typed
refusal, which is why an enumeration like this one is possible at all.

Two cautions for whoever rebuilds this. The `#[ignore]` line reports **hits, not sites**:
of its 8 hits only 4 are actual attributes, the other 4 being prose that mentions the word
— and one whole file's hit is a comment about an `#[ignore]` that no longer exists (F-10),
which this ledger nearly filed as a live fence. And the `Refusal`-enum regex finds 12 enums
by NAME; it does not find `RefusalPolicy`'s associated `Refusal` struct
(`q8-mps/src/dmrg.rs:75`), `SeamRecord` (`trimer_bank.rs:141`), `AutomaticRoute`
(`pair.rs:1046`) or `Routing` (`holon-resource/src/tier.rs:233`), all of which carry
fences under other names. Those four were reached through the loose fence-word sweep, which
is what the loose sweep is for. **Every line number in this ledger was read back out of the
primary artifact before it was written down**; three were wrong on first draft, one row
turned out not to be a fence at all, and one row's owner was wrong until the staking
document was read (M10 — the lane that *recorded* the fence is not the lane that owns its
exit, and only I-5's own text says so).

A hazard this sweep did not hit but a rebuild might: **counting references is not counting
consumers.** ion-core disclosed that the `api_surface` scanner counts rustdoc `[name]`
links as calls, so a symbol referenced only from documentation reads as live. Nothing in
the commands above infers liveness from a reference count — every hit was triaged by
reading it — but F-10 is the same defect in its cheapest form: a `#[ignore]` grep counted a
comment *about* an `#[ignore]` and nearly promoted it to a fence. If a future sweep
automates the triage, doc links and prose mentions are the two false consumers to exclude
first.

Companion deliverable: `conformance/atomworld/PERIODIC_AVAILABILITY.md`, GANTT node D's
probed availability table, generated by
`engine/crates/holon-chem/examples/periodic_availability.rs` and gated by
`engine/crates/holon-chem/tests/periodic_availability.rs`. C5, C6, M1, M2 and M3 in this
ledger are the fences that table measures the cost of.
