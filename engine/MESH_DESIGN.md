# MESH_DESIGN — one 3D scene sharded across cores, in-process

Status: design. Numbers marked **HARD** are computed here from the engine's own constants and
reproduce the engine's own tests; **SOFT** are stage values; **PENDING** are owed measurements
and are used in no arithmetic below.
Scope: **local mesh only.** No transport, no envelopes, no attribution, no consent. In-process,
on-machine, one scene, many arenas.
Frame: `INTEGRATION_FRAME.md` — one holon, values only. Binding design decisions: `SANDBOX_4090.md`
D1–D4, G4, G5. Machine-checked constraints: `CIRISOntology/Core/{Locality,GrainFloor,ModeChart}.lean`.
Date: 2026-08-23.

---

## 0. The recommendation, and the one number that carries it

**Shard one 3D scene by cutting its octree at a fixed level, and make it affordable by occlusion,
not by compression.** The arithmetic that decides this:

| the 3D sandbox scene, 0.6 m cube at 0.5 mm grain | nodes | at 144 B | verdict |
|---|---:|---:|---|
| every cell refined to `g0` (2048³) | 8.59e9 | 1.24 TB | **refused, 74× over the card** |
| every cell refined to observer acuity (512³) | 1.534e8 | 22.1 GB | **refused, 1.32× over the card** |
| matter only (fill 0.45) at acuity | 6.90e7 | 9.94 GB | fits, and 59% of the card to render 1.2 mm detail through opaque sand |
| **acuity on the VISIBLE SURFACE only (3 faces)** | **1.049e6** | **151 MB** | **0.9% of the card** |

The saving is **146×** and it is not a heuristic: in 3D the observer's claim is a claim about a
**2-manifold**, because the interior of a sand pile is not visible. `tier.rs`'s
`acuity_cell_estimate` is `across² × fill × 4/3` — an area — and in 2D that happened to be the
whole domain. In 3D it is still an area, and the domain is a volume. **The acuity claim does not
grow with the third power. Only claim-driven refinement does, and that is corridor-local.**

That is the whole reason 3D is affordable, and it is a *claim* property, not a capacity one —
which is `GrainFloor.lean`'s `capacity_irrelevant` and `demand_not_function_of_geometry` read
forward instead of backward.

**The mesh's job is therefore not to make a big volume fit.** Memory was never the constraint
(`SANDBOX_4090` §4: 149 resident against 1.49e8 capacity, a factor of 1.0e6) and it is not the
constraint in 3D either. The mesh's job is **wall-clock**: 1.05e6 acuity nodes is 7,000× the
measured 2D frontier, and one core will not step it. Sharding is how the scene gets stepped, and
the 1.64× locality speedup the CPU prototype measured at fixed total work is taken for free on
top.

---

## 1. What a shard is — and why it is single-tier

A shard **is a `RuntimeArena`**. No new entity class appears anywhere in this document.

* a **shard** is one arena covering an axis-aligned block of the scene, with its own root and
  **the same declared `g0` as every other shard**;
* a **boundary port** is an ordinary holon inside that arena with `boundary: true` — the flag the
  holon already has;
* a **boundary relation** is a pair of `(shard, holon)` indices. Indices are values.

**The cut is an octree level.** Shard boundaries are cell faces of the scene's own octree at some
level `d`, so each shard is a complete set of sub-octrees and `root_grain_units` stays a power of
two on every shard — which is what makes the tree bottom out exactly at `g0`. Cutting each axis
independently gives shard counts `2^(a+b+c)`; the design target is **64 shards** (a=b=c=2), which
is 4 shards per thread at 16 threads and enough granularity for claim-based balance.

**Single tier, and this is not a preference.** `GrainFloor.lean::cert_does_not_transport_across_reroot`
exhibits a claim served at one tier and refused at another, so a certificate earned on one side of
a re-root states nothing on the other. `SANDBOX_4090` G4 is that fence in the engine's words:
until the re-root ledger gate lands, **a multi-tier mesh is uncertified by construction**. Every
shard here declares the same `g0`, so the mesh is a *refinement* relation, not a re-root, and
`SANDBOX_4090` §6's exchange gate is sufficient for it. **This document designs no multi-tier
sharding and the runtime must refuse to construct one** (assert: all shard `g0` equal, at
construction).

---

## 2. The 3D chart

### 2.1 The mode set: **FCHC-24**, and the reason is the warrant, not the cost

`regplus.rs` carries six FHP directions and `Core/Lattice.lean` proves that object has 53
`(N, P)` sectors with dimension histogram 44/7/2. Reproduced here independently: **HARD**, exact
match.

The FHP-6 chart's warrant in 2D is the hexagonal lattice's fourth-order isotropy. The like-for-like
3D chart is therefore **not** the cheap one:

| candidate | directions | states | sectors | max occupancy | max \|p\| per component | fourth-order isotropic |
|---|---:|---:|---:|---:|---:|---|
| FHP-6 (the 2D chart today) | 6 | 64 | 53 | 6 | 2 | yes (2D) |
| D3Q6 simple cubic | 6 | 64 | 54 | 6 | 1 | **no** |
| FCC-12 | 12 | 4,096 | 1,059 | 12 | 4 | **no** |
| **FCHC-24 projected** | **24** | **16,777,216** | **72,047** | **24** | **6** | **yes** |

All counts **HARD**, enumerated. The FCHC-24 sector count is a full enumeration of 2^24 local
states (largest sector dimension 11,740; 10,322 sectors are one-dimensional), computed by the same
routine that reproduces `Core/Lattice.lean`'s 53 sectors with histogram 44/7/2 on FHP-6 as its
control — so the 3D number is checked by an instrument known to give the right 2D answer. The simple-cubic and FCC lattices have cubic point symmetry,
which is insufficient for an isotropic fourth-rank momentum-flux tensor — this is the classical
result that forces 3D lattice gases onto the face-centred **hyper**-cubic lattice in 4D, projected
onto 3D.

> **CREDIT** — the house pattern is to credit generously and carry the credit into the Lean
> header, so it is stated here in the form that should be lifted verbatim.
>
> * **FHP-6, the chart this engine already wears:** Frisch, Hasslacher & Pomeau, *Lattice-gas
>   automata for the Navier–Stokes equation*, Phys. Rev. Lett. **56** (1986) 1505. The hexagonal
>   lattice's fourth-order isotropy is theirs, and it is the whole warrant of `Core/Lattice.lean`'s
>   64-state object.
> * **FCHC-24, the chart this design adopts:** d'Humières, Lallemand & Frisch, *Lattice gas models
>   for 3D hydrodynamics*, Europhys. Lett. **2** (1986) 291 — the face-centred hyper-cubic lattice
>   in 4D projected onto 3D, adopted precisely because no 3D Bravais lattice with a single speed
>   has an isotropic fourth-rank tensor. The lineage is theirs; ours is the sector enumeration
>   (72,047) and the ledger-cap consequence.
>
> This is a **convergence**, and the house reading applies: it is a hit, not a strike. The
> mathematics is openly borrowed and the cheap alternatives were rejected on *their* result, not
> on ours.

**Choice: FCHC-24.** Adopting D3Q6 because it is four times cheaper would silently drop the
property that makes the 2D chart mean anything, and would do it in a lane where nothing in the
engine would notice. `ModeChart.lean` already parameterises this correctly — `OccState M` over an
**arbitrary** finite mode set — so the Lean instantiation the mesh needs is `OccState (Fin 24)`
with the FCHC direction table, exactly as `fhpChart : Fin 64 → OccState (Fin 6)` does today.
`level_cap` is the g = 1 case for both.

**The declared cost, stated once:** the fourth momentum component of FCHC is conserved by the
projection and is a **spurious invariant** — a known property of the model, not a defect this
design is hiding. The ledger must either carry it (four momentum lanes) or stop claiming
conservation in it. **Recommendation: carry it.** A dropped lane is a conservation claim we stop
making, and `checked_combine` would stop being able to refuse a violation in it.

### 2.2 Bytes per holon

| | `GrossState` | `RuntimeHolon` | chart `Cell` | resident/holon | card capacity |
|---|---:|---:|---:|---:|---:|
| 2D, `momentum: [i64; 2]` (today) | 32 B | **56 B** (verified `sizeof`) | 24 B | 112 B | 1.493e8 |
| 3D, `[i64; 3]` (drop the spurious lane) | 40 B | 64 B | 32 B | 128 B | 1.306e8 |
| **3D, `[i64; 4]` (recommended)** | **48 B** | **72 B** | **32 B** | **144 B** | **1.161e8** |

`resident/holon` is `SANDBOX_4090` §4's stack: header + child/sibling index + live ledger overlay
+ whole-state pool at W = 2. Card capacity is against the measured 15,944 MiB.

**The 3D holon costs 28.6% more bytes (144 against 112), and the card's holon capacity therefore
falls 22.2% (1.493e8 → 1.161e8).** That is a good trade for the third dimension and it is not
close.

> **Corrected.** This paragraph previously read *"the 3D chart costs 29% of the card's holon
> capacity"*, which attached the per-holon figure to the capacity quantity — two different
> numbers for the same fact, and the wrong one of the pair. Caught by
> `sizing.rs::the_three_d_holon_costs_29_percent_more_and_capacity_falls_22_percent`, which now
> asserts both so they cannot drift apart again. This is the second time an unwitnessed number
> in this document turned out wrong; §10.6 records what that pattern cost.

**This is a `ciris-sim-core` change** (`regplus::GrossState`'s momentum arity, and `LANES` in
`holon-swarm::ledger`). It is not mine to make unilaterally — flagged for the lead, and it is the
one item in this design that reaches outside the mesh crates.

### 2.3 The ledger cap, re-derived for 3D

The cap is set by what the chart writes per **terminal** holon, against the binding lane
(`tier::ledger_for`, `Lane::capacity`). One 0.5 mm quartz grain = **5.2112e18** atoms (**HARD**,
recomputed from `atoms_in_grain`).

| chart | per-leaf occupancy | per-leaf \|momentum\| | binding lane | cap (constituents) | **grains** |
|---|---:|---:|---|---:|---:|
| geometric (writes neither) — the sandbox's chart today | 0 | 0 | `constituents` u64 | 1.8447e19 | **3.54** |
| 2D REG+ FHP-6 | 6 | 2 | `occupancy` u64 | 3.0745e18 | **0.59** |
| **3D REG+ FCHC-24** | **24** | **6** | `occupancy` u64 | **7.6861e17** | **0.1475** |

All **HARD**. Three things to carry:

1. **The 3D REG+ cap is 0.147 grains — exactly 4× tighter than the 2D 0.59, and the factor is
   24/6.** The occupancy lane binds in both. `SANDBOX_4090` §2's headline "0.59 grains is the
   defensible general cap" must read **0.147** the moment a 3D REG+ chart exists.
2. **The geometric cap does not move.** 3.54 grains is dimension-independent, because the
   `constituents` lane does not know what a direction is. The sandbox's own chart writes occupancy
   0 everywhere (gated by `scene::tests::the_sandbox_chart_writes_no_occupancy`), so **the shipping
   sandbox scene's cap is unchanged by going 3D.**
3. **Correction to `tier::LeafWrites::REG_PLUS_MAX`.** It declares `momentum: 3` and its doc
   comment says "3 for either momentum component". Enumerated over the actual `DIRECTIONS` table,
   the true FHP-6 maximum per component is **2**. The headline 0.59 is unaffected — occupancy binds
   either way, and 3 is conservative — but the row in `SANDBOX_4090` §2 that attributes 0.59 to
   momentum saturation is attributing it to the wrong lane. One-line fix, reported not applied.

**And a finding that closes a gap rather than opening one:** `tier::ledger_for` already computes
`ratio³` — the tier ladder's census has been 3D all along while the chart was 2D. Going to an
octree makes the chart agree with the census it was already being measured against. **No
constituent count in the tier table moves.** (`SANDBOX_4090` G7 — the fill 0.45 vs packing 0.36
disagreement — is untouched by this and is still owed.)

### 2.4 The spatial chart

`chart.rs` becomes an octree: `FANOUT` 4 → 8, `Cell { x0, y0, z0, size }`, and one more bit of
the ordinal selecting the z half. `children_seen: Vec<u8>` still fits (8 < 255).
`apportion`/`apportion_exact` are dimension-blind and unchanged — the integer largest-remainder
apportionment is what keeps the ledger exact across the split, and it does not care how many
quadrants there are.

**Correction (F1), and it improved the plan.** An earlier revision of this section named
`chart.rs` as the one file holding the child map. It was **two**: `scene.rs` open-coded the same
quadrant arithmetic independently, and nothing checked the two agreed. The sandbox lane has since
deduplicated them (`1da3b4e`) behind a bit-indexed `Cell::child(ordinal)` at `chart.rs:70`, now
called from `scene.rs:122` and *gated* by an assertion at `scene.rs:700` that the generator's
ordinal and the chart's agree. So the octree flip is now genuinely **one constant and one
function** — `FANOUT` 4 → 8, plus `z0: self.z0 + ((ordinal >> 2) & 1) as f64 * half` inside
`Cell::child`. It was two-and-unchecked when this document first claimed it was one.

---

## 3. Resident-set arithmetic for a concrete 3D scene

**Scene:** 0.6 m cube, `g0` = 0.5 mm, sand to 45% of the height, one ball impact. Observer acuity
`0.6 × 3/900` = **2.00 mm**, which is 4 `g0` (**HARD**, matches `tier.rs`'s own assertion).

| | value |
|---|---:|
| octree divisions to acuity | 9 (512 across, 1.1719 mm leaf) |
| octree divisions to `g0` | 11 (2048 across) |
| acuity leaves on one visible face | 2.6214e5 |
| octree nodes above one such face (`× 4/3`) | 3.4953e5 |
| **acuity claim, 3 visible faces** | **1.049e6 nodes = 151 MB at 144 B** |
| claim-driven interior (impact corridor) | **PENDING** |

**The interior number is owed and is used in no arithmetic here.** The 2D measurement is 149
resident / 110 active cells at grading 2, and its 3D analogue is a surface where the 2D one was a
curve — but `SANDBOX_4090` §4 records that geometric extrapolation of residency was wrong by
**five orders of magnitude** on this exact quantity, so extrapolating it again would be repeating a
paid mistake. It is the 3D counterpart of G9 and it is measured, not guessed.

**What that means for sharding:** 1.05e6 nodes over 64 shards is **16,384 nodes/shard = 2.36 MB**,
which is L2/L3-resident on every core. That is the working-set locality the CPU prototype's
1.64× single-threaded speedup came from, and it survives the move to 3D.

**And it means the shards are wildly uneven by geometry**, because a surface-dominated resident set
puts almost nothing in most interior shards. This is not a nuisance to be smoothed; it is §5.

---

## 4. Boundary exchange

**Protocol: snapshot-then-apply over disjoint pairs, integer lanes only, fixed merge order.**
`SANDBOX_4090` D3, unchanged — it was measured exactly conserving and bit-identical, and the mesh
inherits it rather than re-deriving it.

### 4.1 A port is a boundary CELL, not a face aggregate

The one place the mesh must differ from the CPU prototype. In `holon-swarm` a link owns one port
holon carrying an aggregate. A spatial stencil cannot read an aggregate: a lossy summary cannot
output what it discarded, which is `Core/Coordination.lean::not_computable_from` in the exchange's
clothes. So:

* a **port** is a boundary cell holon, `boundary: true`;
* a **link** relates two such cells across a shard face;
* the **halo** is every port within `n·r` of the face, and its depth is the payload bound below.

Disjointness (D3's second reason) then holds per cell rather than per face, and the apply phase
stays lock-free. Where two shards meet at different refinement levels, one coarse port serves
several fine links — that is the case disjointness does **not** cover, and it is exactly why
snapshot-then-apply (D3's first reason) is the load-bearing one and must not be traded away for it.
`exchange.rs::snapshot_planning_is_order_free_where_live_planning_is_not` is the existing witness.

### 4.2 The payload bound is a theorem

`Core/Locality.lean::depends_within_comp` and `iterate_depends_within`: `n` steps of a
radius-`r` update depend within `n·r`, and **nothing deeper**. So:

> **A shard stepping `n` times between exchanges needs halo depth exactly `n·r` cells, and the
> exchange payload is `n·r · L² · sizeof(GrossState)` per face.**

For the granular contact solver `r = 1` cell. At `n = 1`, `r = 1`, `L = 128` (a 64-shard cut of
the 512-across acuity octree): **16,384 cells × 48 B = 786 kB per face per exchange** at full face
occupancy — and far less in practice, because the resident set is a surface and most faces are
nearly empty.

`iterate_factors_through_ball` is the replay warrant: a shard's interior after `n` steps is a
**function** of (its initial data, its halo log), so any shard is verifiable by deterministic
replay from its own receipts. That is what makes the gate in §6 checkable rather than trusted.

### 4.3 Exact conservation is arithmetic, not tolerance

Integer lanes only on the exchange path. Integer `+` is exactly associative, so the merge is
order-independent **by construction** (D1's "the one part of the system whose determinism needs no
discipline at all"), and `checked_add` refuses overflow rather than wrapping. No epsilon appears
anywhere in the exchange. Floats — positions, velocities, whole-state — **never cross a shard
boundary in a reduction**; D4 bans cross-lane float reductions on the certified path and §6's
mutation set is what proves the ban is enforced rather than merely written down.

### 4.4 Merge order

Canonical and fixed at construction: pairs are ordered by `(lo shard, hi shard, face, cell index)`,
with `lo < hi` fixed from shard indices and never from visit order. The gate re-plans from the
snapshot, so a wrong order cannot produce a right answer — but the order is fixed anyway, because
a determinism claim that depends on a property nobody wrote down is a claim about luck.

---

## 5. Scheduling

### 5.1 Disjoint pairs by edge colouring

Arena adjacency for a face stencil is a 3D grid graph, Δ = 6. Colour its **edges** by
(axis, parity of the lower endpoint's coordinate on that axis): **6 perfect matchings, meeting
Vizing's lower bound exactly.** Six exchange sub-rounds in 3D where 2D needs four.

**Colouring is a scheduling property, not a correctness one.** Snapshot-then-apply already gives
determinism regardless of order (§4.1). What colouring buys is a lock-free apply phase when a port
serves several links, which in 3D it must at every refinement-level mismatch. Saying this the other
way round — "the mesh is deterministic because the pairs are coloured" — would be a false warrant
that survives every test.

### 5.2 Balance by claim, never by geometry

`GrainFloor.lean::demand_not_function_of_geometry` exhibits one tier — one size, one grain —
serving one claim and refusing another. **A geometry-based load balancer is wrong by
construction**, and §3 says so with numbers: a surface-dominated resident set makes cell-count
balance assign equal work to shards holding 1e5 and 0 active cells.

The claim-derived weight is available and cheap, because `incremental.rs` asks the model
**exactly one call per holon at every size** (measured 1.000 at 33 / 145 / 577 / 2,321 holons,
against the shipped certifier's 13 → 42,323). So:

> **Shard weight = its active-cell count from the previous round.** One call per holon means
> certification cost is *linear in active cells with a constant of exactly one*, so last round's
> active count is a direct measure of this round's work — not a proxy for it.

Assignment: sort shards by weight descending, greedily fill threads (LPT). Re-balanced every `k`
rounds, `k` declared, with the reassignment itself deterministic (ties by shard index) so the
balance cannot make the run non-reproducible.

---

## 6. The gate, and its mutations

**Staked, non-negotiable: the meshed run is BIT-IDENTICAL to the single-threaded run on the same
scene.** Not 99.9%. Compared on every holon's four integer ledger lanes and every whole-state f64
by `.to_bits()`, over threads ∈ {1,2,4,8,16} × repeats, and under natural / reversed / strided
visit order. The sequential reference is written independently of the meshed path — one code path
with a flag would make the claim nearly vacuous.

The five existing legs carry over unchanged (`gate.rs` L1–L5), and **L3 — plan conformance — is
the one that matters**: `SANDBOX_4090` §6 found that doubling or dropping a transfer on *both*
sides passes L1/L2/L4/L5/L6 and only L3 fires. A balance-based boundary gate is blind by
construction.

### The reorder mutation, and the trap in it

The brief asks that **a deliberately reordered merge be caught.** Stated naively that test cannot
pass, and the reason is the design working: with snapshot-then-apply over integer lanes, reordering
the merge produces the **identical** result. A mutation test that reorders and asserts "the answer
changed" would fail against a correct implementation, and the tempting repair — weakening it until
it passes — ships a gate that cannot fail.

So the reorder mutation is split, and both halves are required:

| # | mutation | must |
|---|---|---|
| **M1a** | reorder arena visit order, integer lanes | **NOT fire** — this is the determinism claim, and firing means the merge is order-dependent |
| **M1b** | plant a cross-shard f64 reduction, then reorder | **FIRE** — proves M1a's harness has teeth and that D4's float-reduction ban is enforced, not merely written |
| M2 | plan from live values instead of the published snapshot | fire |
| M3 | apply before every snapshot is published (drop the barrier) | fire |
| M4 | double the transfer on **both** sides | fire on **L3 only** — re-runs the prototype's finding in the mesh |
| M5 | read a halo cell deeper than `n·r` | fire on the locality assert (§4.2) |
| M6 | orient a pair from visit order instead of shard index | fire |
| M7 | balance by cell count instead of claim | **NOT fire** — balance must not change results, only wall-clock |

M1b is the load-bearing one. Without it, M1a is a test that passes because nothing is being
checked, which is the same defect `weakness_a_ledger_only_gate_is_blind_to_a_broken_composition`
already pins in the prototype.

### The locality gate

Instrument shard reads over one stepping window; assert **every read of a non-owned cell is at
chart-distance ≤ `n·r` from the shard's face**, and that the allocated halo depth is exactly `n·r`.
M5 (widen the stencil to `r+1`) must fire. This is `depends_within_comp` executing, not citing.

---

## 7. Sequencing, and one deviation stated up front

1. **MESH_DESIGN.md** — this document. Stop-point: the lead reviews §2 (mode set, cap) against the
   Lean 3D instantiation of `ModeChart`, which is the lead's work and must agree with §2.1.
2. **G5 — DONE.** `FractureModel` and `ImpactModel` are `Send`. The pre-registered gate
   (`tests/g5_send.rs`) passed with its assertion unchanged and is now unconditional, so a
   regression breaks the default build.

   **The fix was not the one the brief assumed, and the difference is the finding.** The brief
   said "refactor to a passed workspace", which would have meant threading `&mut WallChart`
   through `RuntimeBoundaryModel` and `BoundarySelector` — **14 impls across 5 files in 2
   crates**, including a blanket `impl<F: Fn(..)>` that every closure selector rides. Measured
   before writing anything, and far wider than the 10 sites the brief scoped.
   
   It was not necessary. **`Rc` is what makes the solvers `!Send`, not `RefCell`** —
   `RefCell<T>` is `Send` whenever `T` is, while `Rc<T>` is `Send` for no `T` at all. The
   interior mutability has to STAY, because `RuntimeBoundaryModel::refinement_priority` takes
   `&self` and the chart memoizes distances lazily. What had to go was the **second owner**.
   
   Two owners existed because `certify_runtime_adaptive` takes the model and the materializer
   (holding the selector) as two `&mut` arguments at once, so neither could hold the chart
   exclusively. `TipSpacingSelector` was reading exactly one number from its handle — the
   parent cell's size — which is derivable without a chart at all: the fanout-4 tree halves
   `grain_units` and cell size *together*, so size is `side_m · grain_units / root_grain`, and
   `grain_units` is already in `ChildBoundaryContext`. **Bit-identical, not approximate:**
   `root_grain` is a power of two and `grain_units` halves, so the ratio is a power of two and
   the division is exact in binary floating point.
   
   Net: `Rc` deleted from both files, no trait touched, no lock introduced, **165/165
   `ciris-sim-core` tests unchanged** against the pre-refactor baseline.

   **The gate is PRE-REGISTERED, before the refactor exists** (`tests/g5_send.rs`). Today
   `cargo test --features g5` **fails to compile**, with the compiler naming
   `Rc<RefCell<WallChart>>` for both models — that failure *is* the before-state, established by
   the compiler rather than asserted by us. When G5 lands it compiles and passes with no edit.
   The default build stays green meanwhile, and `the_mesh_side_is_already_send` proves at
   compile time that every mesh type is `Send` today, so **G5 is genuinely the only thing in the
   way** rather than the first of several.

   A first version of that gate used autoref specialisation to report Send-ness as a runtime
   `bool`, and it failed in the dangerous direction: the specialised arm was never reached, the
   probe returned `false` for *every* type including `u64`, and the before-state assertion
   passed for entirely the wrong reason. Its control (`u64` must probe as `Send`) is what caught
   it. Replaced by a plain `fn assert_send<T: Send>()`, which cannot be vacuous because it is
   the compiler's own bound check. Recorded because it is the third instrument in this lane to
   need proof that it could fail at all. `incremental.rs::Workspace` is the
   in-repo precedent: reusable, allocation-on-growth-only, passed by `&mut`.
   **Coordination note, and it is live:** `ciris-sim-core/src/fracture.rs` is currently modified in
   the working tree and `ciris-sim-core/src/impact.rs` is untracked — both are mid-edit by another
   lane. Per the shared-tree rule I have touched neither, and will not until that lane reports
   clear. Pathspec commits only.
3. **Mesh runtime** — shard one scene across threads, with §6's gate.
4. **Scaling** — same scene at 1/2/4/8/16 threads, holon-steps/s, honest serial fraction.

**Deviation, flagged rather than absorbed: the runtime lands on the 2D scene first.** The
bit-identity gate, the locality gate and the whole mutation set are **dimension-independent**, and
the 2D scene exists today while the 3D one needs the octree chart of §2.4 plus the core
`GrossState` arity change of §2.2. Proving the gate on the scene that exists, then landing 3D
behind a gate already known to have teeth, is strictly safer than debugging a new chart and a new
concurrency structure against each other. **The 2× binding rule is then measured on the 3D scene,
as the brief requires — it is not being quietly re-scoped to 2D**, only ordered after it. If the
lead wants 3D first, say so and it goes first; it costs the octree chart before the first gate run.

---

## 8. Gaps

| # | gap | scope |
|---|---|---|
| **M-G1** | **`GrossState` momentum arity is a core change.** 3D FCHC needs 4 lanes (or 3 and a dropped conservation claim). Reaches `regplus.rs` and `holon-swarm::ledger::LANES`. **Lands as ONE coordinated commit, merge-gated by the lead** — a window in which the arity disagrees across crates is a broken workspace for every lane. Diff shape is §9; it is not written until the lead has seen it. | shape delivered, write blocked on approval |
| **M-G2** | **3D claim-driven resident set is PENDING** — the 3D counterpart of `SANDBOX_4090` G9. Geometric extrapolation was wrong by 5 dex once already and is not repeated here. | measure, do not guess |
| **M-G11** | **Three 3D broadphase hazards are already recorded** and must not be re-found: the 13-of-26 neighbour stencil, the 512-clamp becoming ~1.6 GB of index arrays once cubed, and the oversized-set `O(N)` term. Source: `JULES_3D_TRIAGE.md` §3.3. Carried here so whoever writes the real 3D broadphase reads them first; none of them touches the mesh's exchange or gate. | inherited, for the 3D stage |
| **M-G3** | **CLOSED, and the Lean/engine split is stated rather than glossed.** FCHC-24 enumerated: 16,777,216 local states, **72,047** `(N, P)` sectors, largest dimension 11,740 — the 3D analogue of `Core/Lattice.lean`'s 53. **What the Lean will carry:** `fchcChart : Fin 2^24 → OccState (Fin 24)` with injectivity (the `testBit` argument generalizes from `fhpChart_injective`), plus the per-slot and `level_cap` caps at the 24-mode set. **What the Lean will NOT carry: the 72,047 sector count.** 2^24 is beyond the kernel's reach by `decide` at this project's discipline, and `native_decide` is not house style — so the sector count stays **ENGINE-checked**, with the FHP-6 = 53 / 44 / 7 / 2 reproduction as its instrument validation. The doc promises no mechanization that will not exist. | done |
| **M-G4** | **CLOSED at `ff27476`**, and closed better than reported. This lane reported that `LeafWrites::REG_PLUS_MAX.momentum` declared 3 where the enumerated FHP-6 maximum is 2, and proposed a one-line fix. The sandbox lane instead made the constant **derived rather than asserted**: `REG_PLUS_MAX` now reads `{ occupancy: 6, momentum: 2 }` (`tier.rs:230`) with `the_reg_plus_maxima_are_enumerated_not_asserted` (`tier.rs:949`) computing both maxima from the lattice, so the constant cannot drift from the six directions again. Headline cap unaffected throughout — occupancy binds either way. | done |
| **M-G5** | **Multi-tier sharding is out of scope and must be refused at construction** until the re-root ledger gate lands (G4). Assert all shard `g0` equal. | fence, not a gap to close here |
| **M-G6** | The barrier was the CPU prototype's scaling limit (≥90% efficiency to 8 threads, 62% at 16 — three `Barrier`s per round). 3D adds **six** colour sub-rounds where 2D has four, so the barrier count per round rises. Whether that binds before 16 threads is unmeasured. | measure in step 4 |
| **M-G7** | `SANDBOX_4090` G7 (fill 0.45 vs packing 0.36) is untouched and still owed by the tier lane. | not mine |

---

## 9. M-G1 diff shape — the `GrossState` momentum arity change

Requested by the lead before any of it is written. **Nothing below is written until it is
approved.** The design goal is the lead's constraint made structural: *there must be no window in
which the arity disagrees across crates.*

### 9.1 Blast radius, measured not estimated

**RECOUNTED (F3), and the first count was wrong by about double.** The triage caught this
document undercounting `holon-sandbox`'s share. Re-measuring on the current tree showed the miss
was not confined to that crate, and the cause is worth stating because it is a search defect, not
an arithmetic one:

> The original count grepped for the LABEL — `momentum: [`, `momentum[0]`, `[i64; 2]`. But
> `GrossState::aggregate(constituents, occupancy, [a, b])` passes momentum **positionally**, with
> no label to match. Every `aggregate()` call site was therefore invisible to the search, and
> `aggregate()` is how most of the tree constructs a ledger entry. **Two whole crates were absent
> from the table for this reason** — not just the files the triage named.

Current, measured: `relativity.rs`'s 35 `momentum` mentions remain out of scope (special
relativity's `FourMomentum`, a deliberately parallel object that does not ride `GrossState`).
Everything else that breaks under a 2 → 4 arity change:

| crate | `aggregate()` call sites | notes |
|---|---:|---|
| `ciris-sim-core` | 18 | incl. `regplus.rs`, `descriptor.rs` (6), `impact.rs`, `fracture.rs`, `holon.rs`, `runtime.rs`, `material.rs`, `mechanical.rs`, 3 examples |
| `holon-sandbox` | 4 | `scene.rs:136`, `scene.rs:604`, `incremental.rs:996`, `incremental.rs:1018` |
| `holon-swarm` | 3 | `shard.rs`, `ledger.rs` |
| **`holon-ball-game`** | 3 | **absent from the first table entirely** |
| `holon-mesh` | 1 | this lane's own crate, which did not exist when the first table was written |
| **`ciris-sim-component`** | 1 | **absent from the first table entirely**, and it is the WIT adapter — deliberately OUTSIDE the workspace, so it needs its own build in the same commit |

Plus **25 `momentum: [` literals** (excluding field declarations) and **4 test assertions**
comparing `.momentum` against a two-element literal — including `scene.rs:786` and `scene.rs:802`,
the two the triage named. §9.3's zero-extending constructor covers the construction sites by
design; **the test assertions need the same treatment and get it in commit P**, since a test that
hard-codes the arity is exactly as brittle as production code that does.

Totals: **~59 breaking sites across 23 files in 6 crates**, against the first table's 11 files in
3 crates. This does not change the two-commit plan — it strengthens the case for it, because 30
positional call sites is far too many to hand-edit safely, and a constructor handles them
mechanically.

**`impact.rs` and `fracture.rs` are on that list, and they are `holon-cracktip`'s mid-edit files.**
The flip therefore waits on the same clearance G5 does — one signal unblocks both, and they should
be sequenced G5 first so the arity flip lands on already-`Send` solvers. **`ciris-sim-component`
is a second coordination point**: outside the workspace, so `cargo test` at the root will not
catch its breakage and the commit must build it explicitly.

### 9.2 The silent-truncation hazard, checked for and absent

The failure mode that would make this change dangerous is a read site that indexes momentum lanes
0 and 1 in a loop and silently ignores 2 and 3 — a change that compiles, passes, and is wrong.
**Verified absent.** Every ledger read site writes an explicit array (`combine`, `checked_combine`,
`to_lanes`, `from_lanes`, `LedgerDelta::checked_add`/`checked_neg`/`checked_mul`), so an arity
mismatch is a **type error**. The three `0..2` loops in the tree are over twin sectors, generations
and certificate observables — none is momentum. There is no `iter().take(2)` and no
`momentum.len()`-blind reduction on the ledger path.

That is what makes the two-commit shape safe: **the compiler, not review, is the instrument.**

### 9.3 Commit P — preparatory, arity-PRESERVING, no-op

Landable at any time, by anyone, with no coordination, because it changes no behaviour.

* `regplus.rs`: add `pub const MOMENTUM_LANES: usize = 2;`; change the field to
  `momentum: [i64; MOMENTUM_LANES]`.
* `holon-swarm::ledger`: `pub const LANES: usize = 4` becomes `2 + regplus::MOMENTUM_LANES`, so
  the two crates stop carrying the same number independently.
* Add a zero-extending constructor so 2D call sites stop hard-coding two elements; migrate the 28
  array-literal construction sites onto it. Zero-extension is *correct*, not a fudge: a 2D chart
  genuinely writes zero into the third and fourth lanes.
* **Gate:** every existing test passes **bit-identically**, and `size_of::<GrossState>() == 32`
  still holds. If either moves, commit P is wrong and is reverted rather than argued with.

### 9.4 Commit F — the flip, atomic, merge-gated by the lead

* `MOMENTUM_LANES: 2 → 4`. **One line**, plus whatever the compiler flags — and §9.2 establishes
  it flags everything.
* `size_of::<GrossState>()` 32 → 48; `RuntimeHolon` 56 → 72; resident/holon 112 → 144.
  `SANDBOX_4090` §4's memory table and this document's §2.2 move in the same commit or the
  repository is left stating two different numbers.
* **One prose pin must move or it goes stale:** `relativity.rs:55` says the ledger's momentum "is
  `[i64; 2]`". Re-pin it — and re-pin it **without** letting it read as a fix. The SR/REG+ misfit
  is *not* resolved by this change: its blocker is the f64 → integer quantisation decision that
  belongs to `sector_table_is_pmu_table`, not the lane count. A four-lane integer ledger is still
  not a home for a continuum `P^mu`. Widening the array and quietly dropping the recorded misfit
  would be the exact failure the misfit was recorded to prevent.

### 9.5 What this does NOT do

It does not make the 2D chart write four lanes of anything, it does not touch `Core/Lattice.lean`'s
64-state object, and it does not by itself deliver a 3D chart — it only makes the ledger able to
hold one. The octree chart (§2.4) and the FCHC direction table are separate work behind it.

---

## 10. Measured — the gate harness, `crates/holon-mesh`

Landed 2026-08-23 on the 2D scene, per §7's sequencing. Standalone crate (`holon-swarm`'s
empty-`[workspace]` precedent), so no shared manifest is touched. **31 tests, 0 failures,
debug and release.**

### 10.1 The gate holds

`meshed == unsharded`, **bit-identical** — every cell's four integer lanes by equality, every
whole-state f64 by `to_bits()` — across partitions 1×1 … 8×3, horizons `n = 1…6`, all three
visit orders, thread counts **1/2/4/8/16**, and 8 repeats of the threaded run. Three
independently written paths make it non-vacuous: an unsharded reference that knows nothing of
shards, halos, boards or threads; a sequential mesh; a threaded mesh.

**Why it is achievable bit-for-bit and not merely approximately.** An edge colour is a
**perfect matching** (proved over every cell of six grids), so within a colour no cell is
written twice. Hence applying in any order gives the same answer, and an edge's plan is a
function of its two endpoints' pre-colour values — *which shard owns them is not an input to
that function*. Partitioning therefore cannot change the answer.

**And that is a real constraint on any solver that wants to ride the mesh**, stated here
because it is load-bearing and easy to violate: a sweep whose writes are visible to later
reads **within the same phase** — a true Gauss-Seidel — is *not* shardable bit-identically,
because shard boundaries then become physically visible. Red/black is fine: each colour is a
Jacobi phase. This belongs with D4 as a fifth determinism condition.

### 10.2 The mutation table, and what it cost to make it able to fail

| # | mutation | required | result |
|---|---|---|---|
| M1a | reorder shards and edges, integer lanes | **NOT fire** | does not fire |
| M1b | planted cross-shard f64 reduction, then reorder | fire | fires |
| M2 | halo read from peers' live state instead of a snapshot | fire | fires |
| M3 | one halo refresh skipped | fire | fires |
| M4 | transfer doubled on **both** sides | fire | fires, and **only** by re-derivation |
| M5 | halo one cell shallower than `n·r` | fire | fires at every `n` |
| M6 | pair oriented from the shard's local view | fire | fires |
| M7 | partition changed (what a balancer does) | **NOT fire** | does not fire |

M4 reproduces `SANDBOX_4090` §6's finding in the mesh, and the test asserts the other half
explicitly: **the scene total stays bit-identical under the doubled transfer**, so a global-sum
gate is provably blind to it. M7 is the balance control — balance is schedule, never physics.

**Three mutations did not fire when first written, and all three were defects in the
mutation rather than in the mesh.** They are kept in the code because they are the finding:
the point of splitting the reorder mutation was that the naive form *cannot* fail, and then
the half that *must* fire turned out to be just as easy to build unobservable.

* **The float reduction took four attempts.** (1) A sum of similar-sized positives has an
  ordering spread of about **one ULP** of the total, which any feedback scale rounds straight
  back away. (2) Multiplying by the energy it fed made the feedback amplify itself **~26× per
  sweep**, driving every cell to the same 1e17 value — at which point the two orders agreed
  again. (3) Summing `momentum·1e12 + occupancy` sums **integers below 2^53, where float
  addition is exact**, so it was perfectly order-independent. (4) What works: summands
  spanning thirteen decades, so each small term is only *partially* absorbed into a large and
  order-dependent running total, normalised to a weighted mean so the per-sweep gain is 0.8
  and the defect settles instead of running away.
* **Orientation-by-swap was a no-op**, because the transfer rule is exactly antisymmetric
  under truncation-toward-zero: swapping the plan inputs *and* the apply targets cancels. Worth
  keeping as a fact about the rule — orientation is not load-bearing *for an exactly
  antisymmetric transfer*. The canonical `lo < hi` orientation stays anyway, because a rule
  that rounds asymmetrically would make it load-bearing again.
* **"Refresh the halo late" preserved the invariant.** It shifted the refresh boundary by one
  sweep while still giving each halo exactly `n` sweeps of use, so the answer stayed correct.
  A defect that preserves the invariant is not a defect; replaced by the live-read defect.

### 10.3 The horizon — and a KILL on this document's own claim

`Core/Locality.lean::iterate_depends_within` proves `n·r` **suffices**; it does not say
necessary. So the crate measures rather than assumes.

> **KILLED, and kept marked.** An earlier revision of this section read *"the bound is tight on
> this stencil"*. **That is false.** It was measured when the colour decomposition had four
> colours — a 2D-only configuration — and stated without naming the colour schedule as part of
> the configuration. Generalising to the six colours 3D needs exposed `n` values at which a halo
> of `n·r − 1` is perfectly sufficient. This is the house's own recurring failure mode: the
> substance survived, the warrant did not.

Swept over five geometries × `n = 1…8`, `C` = a shallower halo is caught, `-` = it sufficed:

```text
  flat  16x12x1 : n1:C n2:C n3:-  n4:C n5:-  n6:-  n7:- n8:-
  cube    8x8x8 : n1:C n2:C n3:C  n4:C n5:-  n6:C  n7:- n8:-
  cube 12x12x12 : n1:C n2:C n3:C  n4:C n5:-  n6:C  n7:- n8:-
  slab  12x8x6  : n1:C n2:C n3:-  n4:C n5:C  n6:-  n7:- n8:-
  slab  12x8x4  : n1:C n2:C n3:C  n4:C n5:C  n6:C  n7:- n8:-
```

**What survives, and it is the load-bearing part:** at `n = 1, 2, 4` the shallower halo is
caught on **every** geometry swept, so the horizon is doing real work rather than being
decorative. **What died:** any claim that `n·r` cannot be smaller. At `n ≥ 7` a halo of
`n·r − 1` sufficed everywhere tested.

**Why `n·r` over-counts.** One colour sweep moves data across only *half* the edges of *one*
axis, so its effective radius is strictly less than the 1 that `n·r` charges it. How much less
depends on which colours the exchange window happens to contain — hence geometry- and
`n`-dependence rather than a uniform answer.

None of this weakens the mesh. `n·r` remains **proved sufficient**, the gate confirms
`meshed == unsharded` at every `n` with the full halo, and a conservative halo is the safe
direction to be wrong in. Both the surviving claim and the negative are pinned by tests
(`a_shallower_halo_is_caught_at_the_depths_where_the_bound_is_load_bearing` and
`at_large_n_a_shallower_halo_is_sufficient_and_the_bound_has_slack`), so the falsification
cannot be quietly re-claimed.

### 10.3b The mesh is 3D, and §5.1's six-colour prediction is confirmed

The scene is now `w × h × d`, with **`d = 1` as the 2D case** — one object, not two code paths,
so the 2D and 3D gates are literally the same assertions on a thicker grid.

| | staked in §5.1 | measured |
|---|---|---|
| edge colours in 3D | 6 (Δ = 6, meeting Vizing's lower bound) | **6**, and max degree is 6 — the decomposition attains the bound |
| edge colours in 2D | 4 | 4 — the two z-colours are empty at `d = 1` |
| each colour a perfect matching | yes | verified over every cell of 11 grids, 2D and 3D |
| colours cover each adjacency once | yes | edge count matches `(w−1)hd + w(h−1)d + wh(d−1)`, and cross-checks against the neighbour stencil |

**The gate holds in 3D**: `meshed == unsharded`, bit-identical, over partitions
1×1×1 … 4×3×2, horizons `n = 1…6`, all three visit orders, and threads 1/2/4/8/16.

Also measured, confirming §2's surface-to-volume argument: **at equal shard size (4,096 cells) a
3D shard's cross-edge fraction is more than 2× a 2D shard's** — the penalty §2 predicted, and
the one §0's occlusion argument buys back by making the 3D resident set surface-dominated.

### 10.4 Scaling — INDICATIVE ONLY, and the reason is stated not buried

Host load average **13.7 on 32 cores**; per-configuration spreads run **7–76%** even at the
median of 7 trials. **No wall-clock speedup measured here is defensible as a hardware
number**, and the 2× binding rule cannot be adjudicated on this host today.

What *is* outside the noise is a **6× size effect at 16 threads**:

| cells per shard (64 shards) | speedup at 16 threads |
|---:|---:|
| 64 | 1.15× |
| 256 | 2.42× |
| 4,096 | **6.80×** |

**Scaling is governed by work-per-shard-per-barrier**, which is the prototype's barrier
finding in the form the mesh feels it. The design consequence is favourable: §3's 3D target —
1.05e6 nodes over 64 shards = **16,384 per shard** — sits *above* the best measured point.

Two honest negatives from this measurement. A first pass reported single trials whose
single-thread baseline moved 70% between runs, which would have made every derived speedup
meaningless; the bench now takes medians and prints spread and load. And an apparent
**`n`-dependence in that first pass did NOT survive medians** and is not claimed — raising the
colours-per-exchange did not measurably improve scaling here.

### 10.5 Gaps this opened

| # | gap | scope |
|---|---|---|
| **M-G8** | **`holon-mesh` is not CI-wired**, exactly as `holon-swarm` is not: both are standalone, so `ci-gates.sh`'s `-p <crate>` form does not reach them. Adding a gate means editing a shared file, which exceeds this lane's brief. | lead's call |
| **M-G9** | **A quiet host is owed** before any scaling number is quotable, and before the 2× binding rule can be adjudicated. | blocks deliverable 4's verdict |
| **M-G10** | The Gauss-Seidel exclusion (§10.1) is a **fifth determinism condition** and belongs alongside `SANDBOX_4090` D4, which does not currently carry it. | reported to that document's owner |
| **M-G12** | **The two ledgers are built and their first verdicts are in, and BOTH of the first run's headline findings were the instrument, not the engine.** The D-ledger (joules, six named channels, `holon-sandbox::sim`) reported that the sandbox scene creates 1.14 J and that the landscape tier's channels explained 24% of its dissipation. Neither was an engine defect: the gain was `total_energy_j`'s overlap term measuring from `radius[i]+radius[j]` where the contact force measures from `rest_gap` (+1.1534 J of a +1.1389 J gain, switched on as the throw woke the pairs it sums over), and the 76% was `contact_damping_j` charged at grain-grain contacts but not at the projectile's (4.82e7 J, 99% of that residual). Corrected, every tier dissipates and the balance closes at **93.6% / 95.4% / 99.2%** (sandbox / grain / landscape) with a positive residual everywhere. The sigma-ledger (nats, three merge sites) is separate and stands. **What is still open**: the projectile's contact potential is absent from `E(t)`; a projectile contact against an ASLEEP cell drops the momentum instead of delivering it, which is an engine-side non-conservation rather than an accounting gap and is **not separably sized yet** because it overlaps the damping channel. | D-ledger verdict CLOSED; sleeper-contact drop OPEN and owned here |
| **M-G13** | **The balance gate's warrant is NOETHER, not `Core/Habit.lean`, and the gate is now CHART-RELATIVE.** The withdrawn warrant (a stable step is injective, an injective step produces nothing, so energy creation is unpredicted) conflated energy with entropy: injectivity is about information, and an injective map creates energy freely — `v ↦ 1.01·v` is a bijection that manufactures joules. Corrected on the spine at `561471f`. The live warrant is narrower and ordinary: at the flat, Newtonian, fixed-volume, fixed-particle-number tiers with static gravity, time-translation symmetry gives a conserved energy by Noether, so a secular gain is a defect. **Where the chart lacks that symmetry the gate must refuse, not pass or fail** — the cosmic tier carries an expansion background, and in an expanding universe total vacuum energy grows with volume (predicted and observed), so a gate firing there would report correct physics as a defect. `BalanceApplicability` implements this as a typed refusal and keeps **static curvature separate from expansion**: a static metric has a timelike Killing vector, so conserved energy exists and only the instrument is missing (`total_energy_j` sums the Newtonian expression, not the Killing energy). The discriminator is delegated to the weak-field screen's own `(H L/c)²` arithmetic rather than decided from the tier name — Cosmic's 30 Mpc patch (ε_bg 4.549e-5, certifies) reads *conserved quantity not computed*, its 100 Mpc patch (ε_bg 5.054e-4, `ExpansionScale`) reads *no time-translation symmetry*. Balance checked at 3 tiers, refused at 5. | done |

### 10.6 The pattern: unwitnessed numbers in this document have a defect rate

Three times now a number in this document has been carried by prose or by a scratchpad script
rather than by something that runs, and **two of the three were wrong**:

| number | backing when written | verdict once witnessed |
|---|---|---|
| FCHC-24 = 72,047 sectors (§2.1) | scratchpad script | correct — but the claim "engine-checked" was false until `fchc.rs` existed |
| the horizon bound is TIGHT (§10.3) | measured under 4 colours, stated unconditionally | **FALSE**, killed by `mutations.rs` |
| the 3D chart costs 29% of capacity (§2.2) | arithmetic in prose | **WRONG** — 28.6% more bytes, 22.2% less capacity; caught by `sizing.rs` |

`sizing.rs` now derives §0's whole table, including the **146× occlusion saving** the entire
design rests on, and pins the scaling law under it: the acuity claim is a claim about a
2-manifold, so it grows as `4^d` where volume grows as `8^d`, and **the saving therefore doubles
with every subdivision** — it is not one lucky number at this scene's size, and a finer tier makes
occlusion worth more, never less.

**The assumption is separated from the arithmetic, and only the arithmetic is checked.** That
observer acuity reaches only the *visible surface* — that an opaque pile's interior need not be
resolved — is the load-bearing premise, and `sizing.rs` computes what it is worth **if** it holds
without establishing that it does. Its kill is stated in that module's header: if a certified
frontier on a real 3D scene resolves interior cells to acuity, the saving is not 146× and the
budget must be re-derived from the measured frontier. **M-G2 is that measurement and it is still
owed.**

The house rule this pays for: a headline number with no witness that runs is a number that will
drift, and the drift is invisible because the number looks fine.

### 10.7 §5.2's balancer is DEFERRED, with a measured trigger

§5.2 says balance by **claim**, never by geometry, on the authority of
`GrainFloor.lean::demand_not_function_of_geometry`. That theorem says demand is not a function
of geometry **in general**. Whether it is one for a *particular* scene is a measurement — and
building a balancer before taking it would be shipping a feature that cannot be tested.

Measured, 64 shards, max work / mean work per shard:

| claim shape | imbalance |
|---|---:|
| **uniform** — every cell steps every colour, which is this mesh's scene today | **1.01 – 1.23** |
| **corridor-local** — the shape the sandbox certifier actually produces | **2.37 – 7.61** |

**Decision: do not build the balancer yet.** Under a uniform claim a geometric partition is
already within 1–23% of perfect, so a claim-based scheduler has nothing to recover and its test
would pass on an empty difference.

**But the trigger is now precise rather than vague, and the cost of ignoring it is quantified.**
A perfect schedule's makespan is bounded below by its largest single shard, so at imbalance `f`
over `S` shards, **speedup can never exceed `S / f` however many threads are supplied**. At the
worst measured corridor that is `64 / 7.61 ≈ 8.4×` — barely half of 16 threads, wasted no matter
what the hardware offers. So:

> **Build the claim-based balancer when the scene carries a non-uniform claim.** Not before, and
> not on general principle.

This is `demand_not_function_of_geometry` converted from a theorem into a number: identical
shards, identical grain, up to **7.6× different cost**. Both regimes are pinned by tests
(`balance.rs`), including a control that the work metric varies at all — because a ratio of
constants would have made the whole comparison vacuous.

**Scope note:** the corridor is a *model* of the certifier's demand, not the certifier.
`SANDBOX_4090` §4 measured the real contact frontier as corridor-local, which is what the model
is shaped after; M-G2's owed measurement is what would replace it with the real thing.
